// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod monitor;
mod presence;

use std::{
    thread,
    time::{Duration, Instant},
};

// ─── Configuration ────────────────────────────────────────────────────────────
//
//  1. Go to https://discord.com/developers/applications
//  2. Create (or select) an application and copy the "Client ID" (a 64-bit
//     integer shown under OAuth2 → General).
//  3. Paste it here.
//
const CLIENT_ID: u64 = 1143468205365530624; // <── replace with your Client ID

const POLL_INTERVAL: Duration = Duration::from_secs(5);

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let mut discord = presence::PresenceManager::new(CLIENT_ID);
    let mut watcher = monitor::WarpWatcher::new();

    // The instant Warp was first detected; cleared when Warp exits.
    let mut warp_start: Option<Instant> = None;

    loop {
        match watcher.window_title() {
            Some(title) => {
                // Record start time only the first time we see Warp.
                let started_at = *warp_start.get_or_insert_with(Instant::now);
                discord.update(&title, started_at);
            }
            None => {
                // Warp just closed – clear the presence once, then forget.
                if warp_start.take().is_some() {
                    discord.clear();
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
