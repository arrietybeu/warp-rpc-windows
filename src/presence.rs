/// Discord Rich Presence manager — pure transport layer.
///
/// All asset / label logic lives in the `strategies` module.
/// This module only concerns itself with the Discord IPC connection and
/// translating a `PresenceData` value into an API call.
///
/// ## Why we do NOT manually reconnect on `set_activity` failure
///
/// The `discord-presence` 3.x client spawns a background thread that owns the
/// IPC socket and handles reconnection automatically.  If we drop the `Client`
/// value (what `connect()` used to do on failure) we close that socket — which
/// makes Discord wipe the presence card immediately.  The brand-new client then
/// isn't ready either, so the retry also fails: net result is a presence that
/// disappears and never recovers until we happen to succeed two polls in a row.
///
/// Instead we hold a single `Client` for the entire lifetime of the process.
/// A transient `set_activity` error (e.g. Discord is slow to respond) is logged
/// in debug mode and silently ignored; the background thread keeps the socket
/// alive and the next poll will succeed.  We only recreate the client when it is
/// genuinely absent — which can't currently happen, but the guard is kept as
/// defensive code.
///
/// NOTE: App name ("Warp Pro") and the "Đang chơi" label are Discord Developer
/// Portal settings and cannot be set from code.
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
        mgr.ensure_connected();
        mgr
    }

    /// Push a presence update.
    ///
    /// If `set_activity` fails we log the error (debug builds only) and return
    /// without touching the client.  The library's internal thread will keep
    /// the IPC socket alive and the update will succeed on the next poll.
    pub fn update(&mut self, data: &PresenceData, started_at: Instant) {
        self.ensure_connected();

        if self.try_update(data, started_at).is_none() {
            #[cfg(debug_assertions)]
            eprintln!(
                "[warp-rpc] presence: set_activity failed \
                 (discord not ready — will retry next poll)"
            );
            // Do NOT call connect() here.  Dropping the Client closes the IPC
            // socket which immediately clears the Discord presence card and
            // makes the reconnect race even worse.
        }
    }

    /// Clear the Rich Presence (called when no detector fires / Warp loses focus).
    pub fn clear(&mut self) {
        if let Some(client) = &mut self.client {
            let _ = client.clear_activity();
        }
    }

    // ── Private ───────────────────────────────────────────────────────────────

    /// Creates the client exactly once.  Subsequent calls are no-ops so we
    /// never accidentally destroy an active IPC connection.
    fn ensure_connected(&mut self) {
        if self.client.is_none() {
            let mut client = Client::new(self.client_id);
            client.start();
            self.client = Some(client);
        }
    }

    fn try_update(&mut self, data: &PresenceData, started_at: Instant) -> Option<()> {
        let client = self.client.as_mut()?;

        let elapsed_secs = started_at.elapsed().as_secs();
        let start_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs()
            .saturating_sub(elapsed_secs);

        // Bind to local variables before the closure so we don't capture `data`
        // (a short-lived reference) inside a potentially 'static closure.
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
