/// Claude AI session detector — priority 4.
///
/// Fires when the window title contains "claude" and none of the higher-
/// priority detectors (Neovim, Rust, Cocos2dx) have already claimed the
/// presence. This covers the case where the user is running Claude Code
/// directly inside Warp Terminal without any other recognised tool active.
///
/// Assets
/// ──────
///   large: `claude`  (starburst C logo)
///   small: `warp`    (Warp Pro icon overlay)
use crate::models::{PresenceData, ProcessInfo};
use crate::strategies::AppDetector;

pub struct ClaudeDetector;

impl AppDetector for ClaudeDetector {
    fn detect(&self, window_title: &str, _processes: &[ProcessInfo]) -> Option<PresenceData> {
        if !window_title.to_lowercase().contains("claude") {
            return None;
        }

        Some(PresenceData {
            details: "Claude AI-Assisted Dev".to_owned(),
            state: "Refining Codebase".to_owned(),
            large_image: "claude",
            large_text: "Claude AI",
            small_image: "warp",
            small_text: "Warp Pro",
        })
    }
}
