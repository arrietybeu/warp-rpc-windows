// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod monitor;
mod presence;
mod strategies;

use strategies::{
    AppDetector,
    claude::ClaudeDetector,
    cocos2dx::Cocos2dxDetector,
    neovim::NeovimDetector,
    rust::RustDetector,
    warp::WarpDetector,
};

use std::{
    thread,
    time::{Duration, Instant},
};

// ─── Configuration ────────────────────────────────────────────────────────────
//
//  Get your Client ID from:
//  https://discord.com/developers/applications → your app → OAuth2 → Client ID
//
const CLIENT_ID: u64 = 1143468205365530624;

const POLL_INTERVAL: Duration = Duration::from_secs(5);

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let mut discord = presence::PresenceManager::new(CLIENT_ID);
    let mut monitor = monitor::SystemMonitor::new();

    // Priority-ordered detector chain — first Some(_) wins.
    //
    //  1. NeovimDetector   — nvim / Neovim editing session
    //  2. RustDetector     — Cargo / Rust build session
    //  3. Cocos2dxDetector — Cocos2dx game project  (asset key: "cocos2dx")
    //  4. ClaudeDetector   — Claude AI session       (title contains "claude")
    //  5. WarpDetector     — Warp Terminal fallback  (always fires when Warp focused)
    let detectors: Vec<Box<dyn AppDetector>> = vec![
        Box::new(NeovimDetector),
        Box::new(RustDetector),
        Box::new(Cocos2dxDetector),
        Box::new(ClaudeDetector),
        Box::new(WarpDetector),
    ];

    // Session timer resets only when no detector fires (presence fully cleared).
    let mut session_start: Option<Instant> = None;

    loop {
        // monitor.snapshot() returns None when Warp is not the focused window,
        // which short-circuits the entire detector chain and clears presence.
        let result = monitor
            .snapshot()
            .and_then(|snap| {
                detectors
                    .iter()
                    .find_map(|d| d.detect(&snap.title, &snap.processes))
            });

        match result {
            Some(data) => {
                let started_at = *session_start.get_or_insert_with(Instant::now);
                discord.update(&data, started_at);
            }
            None => {
                if session_start.take().is_some() {
                    discord.clear();
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
