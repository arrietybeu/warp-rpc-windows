/// Cocos2dx detector — priority 3.
///
/// Asset key: `cocos2dx`  (must match the key uploaded in the Developer Portal)
///
/// Detection triggers (any one is sufficient)
/// ───────────────────────────────────────────
///   • `cocoscreator.exe` / `cocosdashboard.exe` in the process list
///   • Title contains "cocos" or "cocos2dx"
///   • Title contains ".cpp" or ".h"  (C++ source files — Cocos2dx projects
///     are C++ based; this catches editors opened inside Warp on those files)
///
/// Note: NeovimDetector and RustDetector have higher priority and will absorb
/// titles like "nvim Game.cpp" before this detector sees them, so the ".cpp"
/// / ".h" heuristic only fires for non-nvim Cocos2dx workflows.
use crate::models::{PresenceData, ProcessInfo};
use crate::strategies::AppDetector;

pub struct Cocos2dxDetector;

impl AppDetector for Cocos2dxDetector {
    fn detect(&self, window_title: &str, processes: &[ProcessInfo]) -> Option<PresenceData> {
        let title_lower = window_title.to_lowercase();

        let cocos_process = processes.iter().any(|p| {
            p.name.contains("cocoscreator") || p.name.contains("cocosdashboard")
        });
        let cocos_in_title = title_lower.contains("cocos")
            || title_lower.contains("cocos2dx")
            || title_lower.contains(".cpp")
            || title_lower.contains(".h");

        if !cocos_process && !cocos_in_title {
            return None;
        }

        Some(PresenceData {
            details: "Cocos2dx Game Forging ⚔️".to_owned(),
            state: "Project: Trảm Quỷ Online".to_owned(),
            large_image: "cocos2dx",
            large_text: "Cocos2dx",
            small_image: "warp",
            small_text: "Warp Pro",
        })
    }
}
