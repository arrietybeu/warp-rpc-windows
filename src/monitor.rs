/// Process + window-title detection for Warp Terminal.
///
/// Strategy
/// --------
/// 1. Use `sysinfo` to enumerate live processes and collect every PID whose
///    image name matches `warp.exe` (case-insensitive).
/// 2. Use `EnumWindows` to walk all top-level windows.  For each visible
///    window that belongs to one of those PIDs, read its title with
///    `GetWindowTextW`.  The first non-empty title wins.
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
};

const WARP_EXE: &str = "warp.exe";

// ─── Internal state passed through the EnumWindows callback ──────────────────

struct SearchState {
    target_pids: Vec<u32>,
    result: Option<String>,
}

/// Called by `EnumWindows` for every top-level window.
/// Returns FALSE (stop) once we have a title; TRUE (continue) otherwise.
unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: lparam is always a valid &mut SearchState cast to isize,
    // and all WinAPI calls below are safe given a valid HWND.
    unsafe {
        let state = &mut *(lparam.0 as *mut SearchState);

        // Skip invisible windows (minimised chrome, background helpers, …).
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        if !state.target_pids.contains(&pid) {
            return BOOL(1);
        }

        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buf);

        if len > 0 {
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            let trimmed = title.trim().to_owned();
            if !trimmed.is_empty() {
                state.result = Some(trimmed);
                return BOOL(0); // stop – we have what we need
            }
        }

        BOOL(1) // continue to the next window
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

pub struct WarpWatcher {
    sys: System,
}

impl WarpWatcher {
    pub fn new() -> Self {
        Self { sys: System::new() }
    }

    /// Returns the window title of the running Warp Terminal, or `None` if
    /// Warp is not open / has no visible window with a title.
    pub fn window_title(&mut self) -> Option<String> {
        // Refresh only the process list – no CPU/memory/disk overhead.
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            ProcessRefreshKind::everything(),
        );

        let pids: Vec<u32> = self
            .sys
            .processes()
            .values()
            .filter(|p| p.name().eq_ignore_ascii_case(WARP_EXE))
            .map(|p| p.pid().as_u32())
            .collect();

        if pids.is_empty() {
            return None;
        }

        let mut state = SearchState { target_pids: pids, result: None };

        // SAFETY: state lives for the entire duration of EnumWindows.
        unsafe {
            let _ = EnumWindows(
                Some(enum_callback),
                LPARAM(&mut state as *mut SearchState as isize),
            );
        }

        state.result
    }
}
