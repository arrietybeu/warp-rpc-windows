/// Rust / Cargo detector — priority 2.
///
/// Detection triggers (any one is sufficient)
/// ───────────────────────────────────────────
///   • Title contains "cargo"  — running a cargo command
///   • Title contains "rust"   — Rust-related activity
///   • Title contains ".rs"    — editing a Rust source file
///
/// NOTE: We intentionally do NOT check for `cargo.exe` in the process list.
/// With the foreground-window model the only reliable signal is the active
/// tab's title.  A cargo build running in a DIFFERENT tab would otherwise
/// trigger this detector for unrelated tabs (e.g. a Claude Code tab).
///
/// Details line: "đang Debug rust ở folder <folder>"
/// State line mapping
/// ──────────────────
///   "cargo build …"  →  "Running: Build"
///   "cargo check …"  →  "Running: Check"
///   "cargo run …"    →  "Running: Run"
///   "cargo test …"   →  "Running: Test"
///   "cargo clippy …" →  "Running: Clippy"
///   "cargo fmt …"    →  "Running: Fmt"
///   (other)          →  "Running: Cargo"
use std::path::Path;

use crate::models::PresenceData;
use crate::strategies::AppDetector;

/// (lowercase subcommand, display label)
const SUBCOMMANDS: &[(&str, &str)] = &[
    ("build",  "Build"),
    ("check",  "Check"),
    ("run",    "Run"),
    ("test",   "Test"),
    ("clippy", "Clippy"),
    ("fmt",    "Fmt"),
    ("doc",    "Doc"),
    ("bench",  "Bench"),
    ("clean",  "Clean"),
];

pub struct RustDetector;

impl AppDetector for RustDetector {
    fn detect(&self, window_title: &str) -> Option<PresenceData> {
        let title_lower = window_title.to_lowercase();

        let rust_in_title = title_lower.contains("cargo")
            || title_lower.contains("rust")
            || title_lower.contains(".rs");

        if !rust_in_title {
            return None;
        }

        let folder = extract_folder(window_title);
        let state = detect_cargo_task(&title_lower);

        Some(PresenceData {
            details: format!("[Debug] {folder}"),
            state,
            large_image: "rust",
            large_text: "Rust / Cargo",
            small_image: "warp",
            small_text: "Warp Pro",
        })
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn detect_cargo_task(title_lower: &str) -> String {
    for &(cmd, label) in SUBCOMMANDS {
        if title_lower.contains(&format!("cargo {cmd}")) {
            return format!("Running: {label}");
        }
    }
    "Running: Cargo".to_owned()
}

/// Extract the leaf folder name from a Warp window title.
///
/// Warp formats:
///   "cargo run — tram-quy — Warp"  →  "tram-quy"
///   "cargo run"                     →  "cargo run"  (no separator)
fn extract_folder(title: &str) -> String {
    // Warp title format: "<command> — <folder> — Warp"
    // Split on em dash and take the middle segment, falling back to the full title.
    let parts: Vec<&str> = title.split('\u{2014}').collect();

    let candidate = if parts.len() >= 2 {
        // Middle segment is the folder; last segment is " Warp"
        parts[parts.len() - 2].trim()
    } else {
        title.trim()
    };

    let leaf = Path::new(candidate)
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(candidate);

    if leaf.is_empty() { "unknown".to_owned() } else { leaf.to_owned() }
}
