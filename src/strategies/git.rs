/// Git detector — priority 3.
///
/// Detection triggers
/// ──────────────────
///   • Window title contains "git"  (git commands running inside Warp)
///
/// State line: "Git Activity in <folder>" where <folder> is extracted
/// from the left-hand side of Warp's title separator.
use std::path::Path;

use crate::models::PresenceData;
use crate::strategies::AppDetector;

pub struct GitDetector;

impl AppDetector for GitDetector {
    fn detect(&self, window_title: &str) -> Option<PresenceData> {
        if !window_title.to_lowercase().contains("git") {
            return None;
        }

        let folder = extract_folder(window_title);

        Some(PresenceData {
            details: "Managing Source Control".to_owned(),
            state: format!("Git Activity in {folder}"),
            large_image: "git",
            large_text: "Git",
            small_image: "warp",
            small_text: "Warp Pro",
        })
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Extract the project / folder name from a Warp window title.
///
/// "tram-quy — Warp"  →  "tram-quy"
/// "C:\Users\Huy\Game — Warp"  →  "Game"
/// "Warp"  →  "Warp Terminal"
fn extract_folder(title: &str) -> String {
    let lhs = title
        .split('\u{2014}') // em dash
        .next()
        .or_else(|| title.split('\u{2013}').next()) // en dash
        .unwrap_or(title)
        .trim();

    let leaf = Path::new(lhs)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(lhs);

    if leaf.is_empty() || leaf.eq_ignore_ascii_case("warp") {
        "Warp Terminal".to_owned()
    } else {
        leaf.to_owned()
    }
}
