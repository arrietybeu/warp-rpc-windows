/// Warp Terminal detector — priority 5 (default fallback).
///
/// Fires whenever `warp.exe` is running, regardless of window focus or title
/// content. All higher-priority detectors have already declined by the time
/// this runs, so no secondary keyword checks are needed here.
///
/// State line: "Context: <folder>" where <folder> is extracted from Warp's
/// window title using the "folder — Warp" format.
use crate::models::{PresenceData, ProcessInfo};
use crate::strategies::AppDetector;

pub struct WarpDetector;

impl AppDetector for WarpDetector {
    fn detect(&self, window_title: &str, processes: &[ProcessInfo]) -> Option<PresenceData> {
        if !processes.iter().any(|p| p.name == "warp.exe") {
            return None;
        }

        let folder = extract_folder(window_title);

        Some(PresenceData {
            details: "Warp Terminal Session".to_owned(),
            state: format!("Context: {folder}"),
            large_image: "warp",
            large_text: "Warp Pro",
            small_image: "warp",
            small_text: "Warp Pro",
        })
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extract the project / folder name from a Warp window title.
///
/// Warp formats:
///   "tram-quy-online — Warp"      →  "tram-quy-online"
///   "C:\Users\Huy\Game — Warp"    →  "Game"
///   "Warp"                         →  "Warp Terminal"
fn extract_folder(title: &str) -> String {
    // Take the left-hand side of the " — Warp" suffix.
    let lhs = title
        .split('\u{2014}') // em dash
        .next()
        .or_else(|| title.split('\u{2013}').next()) // en dash
        .unwrap_or(title)
        .trim();

    // For filesystem paths keep only the last component.
    let leaf = if lhs.contains('\\') || lhs.contains('/') {
        lhs.split(['\\', '/'])
            .filter(|s| !s.is_empty())
            .last()
            .unwrap_or(lhs)
    } else {
        lhs
    };

    if leaf.is_empty() || leaf.eq_ignore_ascii_case("warp") {
        "Warp Terminal".to_owned()
    } else {
        leaf.to_owned()
    }
}
