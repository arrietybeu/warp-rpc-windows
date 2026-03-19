/// Discord Rich Presence manager — pure transport layer.
///
/// All asset / label logic lives in the `strategies` module.
/// This module only concerns itself with the Discord IPC connection and
/// translating a `PresenceData` value into an API call.
///
/// NOTE: App name ("Warp Pro") and the "Đang chơi" (Playing) label are
/// Discord Developer Portal settings — they cannot be set from code.
use crate::models::PresenceData;
use discord_presence::Client;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

    /// Push a presence update. Reconnects once silently on failure.
    pub fn update(&mut self, data: &PresenceData, started_at: Instant) {
        if self.try_update(data, started_at).is_none() {
            self.connect();
            self.try_update(data, started_at);
        }
    }

    /// Clear the Rich Presence (called when no detector fires).
    pub fn clear(&mut self) {
        if let Some(client) = &mut self.client {
            let _ = client.clear_activity();
        }
    }

    // ── Private ───────────────────────────────────────────────────────────────

    fn connect(&mut self) {
        let mut client = Client::new(self.client_id);
        client.start();
        self.client = Some(client);
    }

    fn try_update(&mut self, data: &PresenceData, started_at: Instant) -> Option<()> {
        let client = self.client.as_mut()?;

        let elapsed_secs = started_at.elapsed().as_secs();
        let start_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs()
            .saturating_sub(elapsed_secs);

        // Copy/slice the fields before the move closure so we don't capture
        // `data` (a reference) inside a potentially 'static closure.
        let details = data.details.as_str();
        let state   = data.state.as_str();
        let l_img   = data.large_image;
        let l_txt   = data.large_text;
        let s_img   = data.small_image;
        let s_txt   = data.small_text;

        client
            .set_activity(|act| {
                act.details(details)
                    .state(state)
                    .assets(|a| {
                        a.large_image(l_img)
                            .large_text(l_txt)
                            .small_image(s_img)
                            .small_text(s_txt)
                    })
                    .timestamps(|t| t.start(start_unix))
            })
            .ok()?;

        Some(())
    }
}
