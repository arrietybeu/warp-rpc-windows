/// System monitor — sticky-presence design.
///
/// Each poll tick:
///   1. Refresh the process list (sysinfo).
///   2. Check whether `warp.exe` is in the process list at all.
///      → No  : return `PollResult::Gone`  (presence cleared).
///      → Yes : continue.
///   3. Call `GetForegroundWindow` to get the window the user is looking at.
///   4. Check whether that window's PID (or its parent) belongs to `warp.exe`.
///      → Yes : read the window title, return `PollResult::Active(snapshot)`.
///      → No  : return `PollResult::Background`  (warp is running but not focused).
///
/// The three-state result lets `main.rs` implement sticky presence:
///   Active     → update Discord with live tab info.
///   Background → keep the last known presence, append " (Background)" to details.
///   Gone       → clear Discord entirely.
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};

// ─── Public types ─────────────────────────────────────────────────────────────

/// Result of one monitor poll tick.
pub enum PollResult {
    /// Warp is the focused window — title of the active tab is available.
    Active(SystemSnapshot),
    /// Warp is running but is not the foreground window (user is in a browser,
    /// Discord, etc.).  The last-known presence should be kept on Discord with
    /// a "(Background)" marker.
    Background,
    /// `warp.exe` is not running at all — Discord presence should be cleared.
    Gone,
}

pub struct SystemSnapshot {
    /// Title of the active (focused) Warp tab.
    pub title: String,
}

// ─── Monitor ──────────────────────────────────────────────────────────────────

pub struct SystemMonitor {
    sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self { sys: System::new() }
    }

    /// Poll the current Warp state.  One sysinfo refresh per tick — no extra scans.
    pub fn poll(&mut self) -> PollResult {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            ProcessRefreshKind::everything(),
        );

        // Collect all PIDs whose executable name is warp.exe.
        let warp_pids: Vec<u32> = self
            .sys
            .processes()
            .values()
            .filter(|p| {
                p.name()
                    .to_str()
                    .unwrap_or("")
                    .eq_ignore_ascii_case("warp.exe")
            })
            .map(|p| p.pid().as_u32())
            .collect();

        // If Warp is not running at all, nothing to show.
        if warp_pids.is_empty() {
            return PollResult::Gone;
        }

        // Ask Windows which window the user is currently looking at.
        let (hwnd, fg_pid) = foreground_window();

        // Check whether the foreground window belongs to Warp — directly or via
        // a GPU/renderer child process (which owns the actual window on some configs).
        // Also ignore our own debug console window that can briefly steal focus.
        let our_pid = std::process::id();
        let is_warp_window = warp_pids.contains(&fg_pid)
            || fg_pid == our_pid
            || {
                self.sys
                    .process(Pid::from_u32(fg_pid))
                    .and_then(|p| p.parent())
                    .map(|parent_pid| warp_pids.contains(&parent_pid.as_u32()))
                    .unwrap_or(false)
            };

        if !is_warp_window {
            #[cfg(debug_assertions)]
            eprintln!(
                "[warp-rpc] monitor: fg_pid={fg_pid} not warp — background mode"
            );
            return PollResult::Background;
        }

        // Warp IS focused — read the active tab's title.
        // unwrap_or_default(): an empty tab title is valid; WarpDetector handles it.
        let title = window_text(hwnd).unwrap_or_default();
        PollResult::Active(SystemSnapshot { title })
    }
}

// ─── WinAPI helpers ───────────────────────────────────────────────────────────

/// Returns the `(HWND, PID)` pair for the currently focused window.
fn foreground_window() -> (HWND, u32) {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        (hwnd, pid)
    }
}

/// Reads the window title of `hwnd`.  Returns `None` for empty / unreadable titles.
fn window_text(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len > 0 {
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            let trimmed = title.trim().to_owned();
            if !trimmed.is_empty() { Some(trimmed) } else { None }
        } else {
            None
        }
    }
}
