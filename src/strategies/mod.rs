/// Strategy pattern for per-application Discord Rich Presence detection.
///
/// Priority chain (first `Some` wins):
///   1. neovim — nvim / Neovim session (file editing)
///   2. rust   — Cargo / Rust session
///   3. git    — Git source control activity
///   4. warp   — Warp Terminal fallback (guaranteed Some; also handles claude titles)
use crate::models::PresenceData;

pub mod git;
pub mod neovim;
pub mod rust;
pub mod warp;

// ─── Trait ────────────────────────────────────────────────────────────────────

pub trait AppDetector: Send + Sync {
    /// Return `Some(PresenceData)` to claim the Discord presence for this tick,
    /// or `None` to defer to the next detector in the chain.
    ///
    /// Detection is title-only: the foreground-window approach makes the active
    /// tab's title the single reliable signal.  Process-list checks incorrectly
    /// fire when a matching process is running in a *different* tab.
    fn detect(&self, window_title: &str) -> Option<PresenceData>;
}
