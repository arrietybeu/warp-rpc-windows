/// Strategy pattern for per-application Discord Rich Presence detection.
///
/// Priority chain (first `Some` wins):
///   1. neovim   — nvim / Neovim session (file editing)
///   2. rust     — Cargo / Rust session
///   3. cocos2dx — Cocos2dx game project
///   4. claude   — Claude AI session (title contains "claude")
///   5. warp     — Warp Terminal fallback (always fires if warp.exe is running)
use crate::models::{PresenceData, ProcessInfo};

pub mod claude;
pub mod cocos2dx;
pub mod neovim;
pub mod rust;
pub mod warp;

// ─── Trait ────────────────────────────────────────────────────────────────────

pub trait AppDetector: Send + Sync {
    /// Return `Some(PresenceData)` to claim the Discord presence for this tick,
    /// or `None` to defer to the next detector in the chain.
    ///
    /// `window_title` – Warp's window title while warp.exe is running;
    ///                  foreground window title otherwise.
    /// `processes`    – all running processes (names are pre-lowercased).
    fn detect(&self, window_title: &str, processes: &[ProcessInfo]) -> Option<PresenceData>;
}
