/// Rust / Cargo detector — priority 2.
///
/// Detection triggers (any one is sufficient)
/// ───────────────────────────────────────────
///   • Title contains "cargo"              — running a cargo command
///   • Title contains "rust"               — Rust-related activity
///   • Title contains ".rs"                — editing a Rust source file
///   • `cargo.exe` in the process list     — background build/check
///
/// State line mapping
/// ──────────────────
///   "cargo build …"  →  "Running: Build"
///   "cargo check …"  →  "Running: Check"
///   "cargo run …"    →  "Running: Run"
///   "cargo test …"   →  "Running: Test"
///   "cargo clippy …" →  "Running: Clippy"
///   "cargo fmt …"    →  "Running: Fmt"
///   (other)          →  "Running: Cargo"
use crate::models::{PresenceData, ProcessInfo};
use crate::strategies::AppDetector;

/// (lowercase subcommand, display label)
const SUBCOMMANDS: &[(&str, &str)] = &[
    ("build",   "Build"),
    ("check",   "Check"),
    ("run",     "Run"),
    ("test",    "Test"),
    ("clippy",  "Clippy"),
    ("fmt",     "Fmt"),
    ("doc",     "Doc"),
    ("bench",   "Bench"),
    ("clean",   "Clean"),
];

pub struct RustDetector;

impl AppDetector for RustDetector {
    fn detect(&self, window_title: &str, processes: &[ProcessInfo]) -> Option<PresenceData> {
        let title_lower = window_title.to_lowercase();

        let rust_in_title = title_lower.contains("cargo")
            || title_lower.contains("rust")
            || title_lower.contains(".rs");
        let cargo_process = processes.iter().any(|p| p.name == "cargo.exe");

        if !rust_in_title && !cargo_process {
            return None;
        }

        let state = detect_cargo_task(&title_lower);

        Some(PresenceData {
            details: "Rust Engineering Session".to_owned(),
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
        // Match "cargo <cmd>" to avoid false positives on unrelated words.
        if title_lower.contains(&format!("cargo {cmd}")) {
            return format!("Running: {label}");
        }
    }
    "Running: Cargo".to_owned()
}
