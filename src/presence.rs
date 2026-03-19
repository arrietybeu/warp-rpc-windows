/// Discord Rich Presence manager.
///
/// Wraps `discord_presence::Client` (v3.x) and handles:
/// - Initial connection (best-effort; Discord may not be running).
/// - Dynamic large-image key ("claude" vs "warp") based on window title.
/// - Elapsed-time timestamp anchored to when Warp was first detected.
/// - Silent error handling so a closed Discord never crashes the app.
use discord_presence::Client;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const DETAILS: &str = "Working in Warp Terminal";
const IMG_WARP: &str = "warp";
const IMG_CLAUDE: &str = "claude";

// ─── Manager ──────────────────────────────────────────────────────────────────

pub struct PresenceManager {
    client_id: u64,
    client: Option<Client>,
}

impl PresenceManager {
    pub fn new(client_id: u64) -> Self {
        let mut mgr = Self { client_id, client: None };
        mgr.connect();
        mgr
    }

    /// Update Rich Presence for the given window title.
    /// Silently retries the connection once if the first attempt fails.
    pub fn update(&mut self, title: &str, started_at: Instant) {
        if self.try_update(title, started_at).is_none() {
            // Discord may have been launched after us – attempt reconnect.
            self.connect();
            self.try_update(title, started_at);
        }
    }

    /// Clear the Rich Presence (called when Warp closes).
    pub fn clear(&mut self) {
        if let Some(client) = &mut self.client {
            let _ = client.clear_activity();
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn connect(&mut self) {
        let mut client = Client::new(self.client_id);
        client.start();
        self.client = Some(client);
    }

    /// Returns `Some(())` on success, `None` on any error.
    fn try_update(&mut self, title: &str, started_at: Instant) -> Option<()> {
        let client = self.client.as_mut()?;

        let large_image = if title.to_ascii_lowercase().contains("claude") {
            IMG_CLAUDE
        } else {
            IMG_WARP
        };

        // Convert the Instant back to a Unix timestamp Discord understands.
        let elapsed_secs = started_at.elapsed().as_secs();
        let start_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs()
            .saturating_sub(elapsed_secs);

        client
            .set_activity(|act| {
                act.details(DETAILS)
                    .state(title)
                    .assets(|a| a.large_image(large_image).large_text("Warp Terminal"))
                    .timestamps(|t| t.start(start_unix))
            })
            .ok()?;

        Some(())
    }
}
