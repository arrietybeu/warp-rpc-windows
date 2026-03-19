/// System monitor — foreground-window-first design.
///
/// Each poll tick:
///   1. Refresh the process list (sysinfo).
///   2. Confirm `warp.exe` is running at all — if not, return `None`.
///   3. Call `GetForegroundWindow` to get the window the user is looking at.
///   4. Verify that window's PID (or its parent's PID) belongs to a `warp.exe`
///      process.  Warp on Windows can host the active window under a GPU/render
///      child process rather than the main warp.exe — checking one level up in
///      the process tree makes the check robust to that.
///      If the active window is NOT Warp (browser, Discord, etc.), return `None`
///      so the caller can clear the Discord presence immediately.
///   5. Read that window's title with `GetWindowTextW` — this is always the
///      title of the active Warp tab, so switching tabs updates presence on
///      the very next poll tick.
///
/// Returning `Option` gives `main.rs` a clean signal: `None` means "nothing
/// to show", `Some` means "show this snapshot".
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};

// ─── Public types ─────────────────────────────────────────────────────────────

pub struct SystemSnapshot {
    /// Title of the active (focused) Warp window — reflects the current tab.
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

    /// Returns `Some(snapshot)` only when a Warp window is in the foreground.
    /// Returns `None` if Warp is not running or the user has switched away from it.
    pub fn snapshot(&mut self) -> Option<SystemSnapshot> {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            ProcessRefreshKind::everything(),
        );

        // Bail out early if Warp is not running at all.
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

        if warp_pids.is_empty() {
            return None;
        }

        // Ask Windows which window the user is currently looking at.
        let (hwnd, fg_pid) = foreground_window();

        // Check whether the foreground window belongs to Warp — either directly
        // (fg_pid is a warp.exe PID) or indirectly (fg_pid is a child of warp.exe).
        // The indirect check is necessary because Warp on Windows can host tabs
        // under GPU/renderer subprocesses whose PID differs from the main warp.exe.
        //
        // We also explicitly ignore our own PID: in debug builds this process has a
        // visible console window, and Windows can briefly give it keyboard focus
        // right after we write to it (eprintln!).  That transient focus steal must
        // not be misread as "user switched away from Warp".
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

        #[cfg(debug_assertions)]
        if !is_warp_window {
            eprintln!(
                "[warp-rpc] monitor: fg_pid={fg_pid} not in warp_pids={warp_pids:?} — skipping"
            );
        }

        if !is_warp_window {
            return None;
        }

        // The foreground window IS a Warp window — read its title.
        // unwrap_or_default() is intentional: a blank/new Warp tab returns an
        // empty window title. We must NOT propagate None here, because that
        // would skip every detector (including WarpDetector) and clear the
        // Discord presence. An empty title is handled gracefully downstream —
        // WarpDetector will show a generic "Warp Terminal Session" fallback.
        let title = window_text(hwnd).unwrap_or_default();

        Some(SystemSnapshot { title })
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

/// Reads the window title of `hwnd`. Returns `None` for empty / unreadable titles.
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
