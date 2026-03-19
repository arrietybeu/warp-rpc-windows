/// Neovim detector — priority 1.
///
/// Detection triggers
/// ──────────────────
///   • Window title contains "nvim" or "neovim"  (nvim inside Warp Terminal
///     causes Warp's title to reflect the command), OR
///   • `nvim.exe` / `neovim.exe` is in the process list  (standalone GUIs
///     such as Neovide where the process name differs from the window title).
///
/// Expected title formats
/// ──────────────────────
///   Inside Warp:   "Neovim — C:\Projects\tram-quy\src\main.rs"
///   Warp (short):  "nvim src/main.rs — Warp"
///   Standalone:    "NVIM"
///
/// Parsing strategy
/// ────────────────
///   • If an em/en dash is present, the path lives on the RIGHT of the dash.
///   • Otherwise strip the leading "nvim"/"neovim" keyword and use the rest.
///   filename = last path component   (e.g. "main.rs")
///   project  = grandparent directory (e.g. "tram-quy" from …/tram-quy/src/main.rs)
use std::path::Path;

use crate::models::{PresenceData, ProcessInfo};
use crate::strategies::AppDetector;

pub struct NeovimDetector;

impl AppDetector for NeovimDetector {
    fn detect(&self, window_title: &str, processes: &[ProcessInfo]) -> Option<PresenceData> {
        let title_lower = window_title.to_lowercase();

        let nvim_in_title =
            title_lower.contains("nvim") || title_lower.contains("neovim");
        let nvim_process = processes
            .iter()
            .any(|p| p.name == "nvim.exe" || p.name == "neovim.exe");

        if !nvim_in_title && !nvim_process {
            return None;
        }

        let (details, state) = parse_nvim_title(window_title);

        Some(PresenceData {
            details,
            state,
            large_image: "neovim",
            large_text: "Neovim",
            small_image: "warp",
            small_text: "Warp Pro",
        })
    }
}

// ─── Title parsing ────────────────────────────────────────────────────────────

fn parse_nvim_title(raw: &str) -> (String, String) {
    // When a dash separator is present the path is always on the right side.
    // "Neovim — C:\Projects\tram-quy\src\main.rs"
    //          ↑ path starts here
    let path_part: &str = if let Some(rhs) = raw.split_once('\u{2014}') {
        rhs.1.trim() // em dash
    } else if let Some(rhs) = raw.split_once('\u{2013}') {
        rhs.1.trim() // en dash
    } else {
        // No separator — strip the leading "nvim"/"neovim" keyword.
        // e.g. "nvim src/main.rs" → "src/main.rs"
        let lower = raw.to_lowercase();
        if let Some(pos) = lower.find("neovim") {
            raw[pos + 6..].trim()
        } else if let Some(pos) = lower.find("nvim") {
            raw[pos + 4..].trim()
        } else {
            raw.trim()
        }
    };

    if path_part.is_empty() {
        return ("In Neovim".to_owned(), "No file open".to_owned());
    }

    let path = Path::new(path_part);

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown");

    // Prefer the grandparent directory as "project" — it skips intermediate
    // "src" / "lib" folders and lands on the actual project root.
    // e.g. …/tram-quy/src/main.rs  →  grandparent = "tram-quy"
    let project = path
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .or_else(|| {
            // Fallback: immediate parent (no "src"-like level)
            path.parent()
                .and_then(|p| p.file_name())
                .and_then(|s| s.to_str())
        })
        .filter(|s| !s.is_empty())
        .unwrap_or("Unknown Project");

    (
        format!("Editing in Neovim: {filename}"),
        format!("Project: {project}"),
    )
}
