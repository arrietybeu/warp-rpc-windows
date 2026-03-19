/// Cocos Creator detector — priority 3.
///
/// Fires when:
///   • `cocoscreator.exe` / `cocosdashboard.exe` is in the process list, OR
///   • The window title contains "cocos" (handles edge-cases like CocosCreator
///     appearing as the foreground window with a custom title).
///
/// Project name extraction
/// ───────────────────────
/// "CocosCreator - MyGame"  →  state: "MyGame"
/// "Cocos Dashboard"        →  state: "Cocos Project"
use crate::models::{PresenceData, ProcessInfo};
use crate::strategies::AppDetector;

pub struct CocosDetector;

impl AppDetector for CocosDetector {
    fn detect(&self, window_title: &str, processes: &[ProcessInfo]) -> Option<PresenceData> {
        let title_lower = window_title.to_lowercase();

        let cocos_process = processes.iter().any(|p| {
            p.name.contains("cocoscreator") || p.name.contains("cocosdashboard")
        });
        let cocos_in_title = title_lower.contains("cocos");

        if !cocos_process && !cocos_in_title {
            return None;
        }

        let project = extract_cocos_project(window_title);

        Some(PresenceData {
            details: "Crafting with Cocos Creator".to_owned(),
            state: project,
            large_image: "cocos",
            large_text: "Cocos Creator",
            small_image: "warp",
            small_text: "Warp Pro",
        })
    }
}

// ─── Title parsing ────────────────────────────────────────────────────────────

fn extract_cocos_project(title: &str) -> String {
    // "CocosCreator - MyGame" → "MyGame"
    title
        .splitn(2, '-')
        .nth(1)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Cocos Project".to_owned())
}
