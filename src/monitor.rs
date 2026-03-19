/// System monitor: gathers the window title and process list each poll tick.
///
/// Title resolution strategy
/// ─────────────────────────
/// 1. If `warp.exe` is running, search every top-level window with
///    `EnumWindows` and return Warp's title regardless of focus. This keeps
///    the presence active even when the user switches to another app.
/// 2. Otherwise fall back to `GetForegroundWindow` so standalone tools
///    (Neovim GUI, CocosCreator) are still detected when Warp is absent.
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
};

use crate::models::ProcessInfo;

// ─── Public snapshot ──────────────────────────────────────────────────────────

pub struct SystemSnapshot {
    /// Most relevant window title for this poll tick (see module-level docs).
    pub title: String,
    /// All running processes with lower-cased names.
    pub processes: Vec<ProcessInfo>,
}

// ─── EnumWindows helper ───────────────────────────────────────────────────────

struct EnumState {
    target_pids: Vec<u32>,
    result: Option<String>,
}

unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let state = &mut *(lparam.0 as *mut EnumState);

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
                return BOOL(0); // stop – found it
            }
        }

        BOOL(1) // continue
    }
}

/// Read the title of the current foreground window.
fn foreground_title() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
                .trim()
                .to_owned()
        } else {
            String::new()
        }
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

pub struct SystemMonitor {
    sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self { sys: System::new() }
    }

    pub fn snapshot(&mut self) -> SystemSnapshot {
        // Refresh only what we need: process names.
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            ProcessRefreshKind::everything(),
        );

        let processes: Vec<ProcessInfo> = self
            .sys
            .processes()
            .values()
            .map(|p| ProcessInfo {
                name: p.name().to_str().unwrap_or("").to_lowercase(),
            })
            .collect();

        // Collect Warp PIDs for the EnumWindows search.
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

        let title = if !warp_pids.is_empty() {
            // Warp is running — find its visible window regardless of focus.
            let mut state = EnumState { target_pids: warp_pids, result: None };
            unsafe {
                let _ = EnumWindows(
                    Some(enum_callback),
                    LPARAM(&mut state as *mut EnumState as isize),
                );
            }
            state.result.unwrap_or_else(foreground_title)
        } else {
            // Warp not running — use whatever window is in focus.
            foreground_title()
        };

        SystemSnapshot { title, processes }
    }
}
