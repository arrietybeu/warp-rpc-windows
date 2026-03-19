/// System monitor — foreground-window-first design.
///
/// Each poll tick:
///   1. Refresh the process list (sysinfo).
///   2. Confirm `warp.exe` is running at all — if not, return `None`.
///   3. Call `GetForegroundWindow` to get the window the user is looking at.
///   4. Verify that window's PID belongs to a `warp.exe` process.
///      If the active window is NOT Warp (browser, Discord, etc.), return `None`
///      so the caller can clear the Discord presence immediately.
///   5. Read that window's title with `GetWindowTextW` — this is always the
///      title of the active Warp tab, so switching tabs updates presence on
///      the very next poll tick.
///
/// Returning `Option` gives `main.rs` a clean signal: `None` means "nothing
/// to show", `Some` means "show this snapshot".
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
};

use crate::models::ProcessInfo;

// ─── Public types ─────────────────────────────────────────────────────────────

pub struct SystemSnapshot {
    /// Title of the active (focused) Warp window — reflects the current tab.
    pub title: String,
    /// All running processes with lower-cased names.
    pub processes: Vec<ProcessInfo>,
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

        // Collect all processes (lower-cased names for detectors).
        let processes: Vec<ProcessInfo> = self
            .sys
            .processes()
            .values()
            .map(|p| ProcessInfo {
                name: p.name().to_str().unwrap_or("").to_lowercase(),
            })
            .collect();

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

        // If the focused window does not belong to Warp, produce no snapshot.
        // This clears presence the moment the user alt-tabs away from Warp.
        if !warp_pids.contains(&fg_pid) {
            return None;
        }

        // The foreground window IS a Warp window — read its title.
        // This is the title of the active tab, so switching tabs is reflected
        // on the very next poll without any special handling.
        let title = window_text(hwnd)?;

        Some(SystemSnapshot { title, processes })
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
