/// Shared data structures used by the monitor, strategies, and presence modules.

// ─── Presence payload ─────────────────────────────────────────────────────────

/// Everything `PresenceManager::update` needs to push one Discord activity.
/// Produced by an `AppDetector` and consumed directly by `PresenceManager`.
#[derive(Clone)]
pub struct PresenceData {
    /// Line 1 of the Discord card (dynamic, e.g. "Engineering warpcord-win").
    pub details: String,
    /// Line 2 of the Discord card (dynamic, e.g. "Claude Code").
    pub state: String,
    /// Asset key for the large icon (must exist in the Developer Portal).
    pub large_image: &'static str,
    /// Hover text for the large icon.
    pub large_text: &'static str,
    /// Asset key for the small overlay icon (bottom-right of the large icon).
    pub small_image: &'static str,
    /// Hover text for the small icon.
    pub small_text: &'static str,
}
