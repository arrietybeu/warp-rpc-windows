/// Warp Terminal detector — priority 4 (guaranteed fallback).
///
/// This detector always returns `Some` — it is the last in the chain and is
/// only reached when no higher-priority detector claimed the presence.  The
/// monitor already confirmed `warp.exe` is the foreground process, so the
/// process-list guard is intentionally omitted here.
///
/// Claude detection is absorbed here:
///   • Title contains "claude" → large_image = "claude"
///   • Otherwise              → large_image = "warp"
///
/// State line: "Context: <folder>" extracted from Warp's "folder — Warp" title format.
use crate::models::PresenceData;
use crate::strategies::AppDetector;

pub struct WarpDetector;

impl AppDetector for WarpDetector {
    fn detect(&self, window_title: &str) -> Option<PresenceData> {
        let folder = extract_folder(window_title);

        let (large_image, large_text) = if window_title.to_lowercase().contains("claude") {
            ("claude", "Claude AI")
        } else {
            ("warp", "Warp Pro")
        };

        Some(PresenceData {
            details: "Warp Terminal Session".to_owned(),
            state: format!("Context: {folder}"),
            large_image,
            large_text,
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
    use std::path::Path;

    // Take the left-hand side of the " — Warp" separator.
    let lhs = title
        .split('\u{2014}') // em dash
        .next()
        .or_else(|| title.split('\u{2013}').next()) // en dash
        .unwrap_or(title)
        .trim();

    // Path::file_name() robustly extracts the last component from any path
    // format (forward slash, backslash, UNC, bare name, etc.).
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
