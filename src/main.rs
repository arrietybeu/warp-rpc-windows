// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod monitor;
mod presence;
mod strategies;

use strategies::{
    AppDetector,
    git::GitDetector,
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
    //  1. NeovimDetector — nvim / Neovim editing session
    //  2. RustDetector   — Cargo / Rust build session
    //  3. GitDetector    — Git source control activity
    //  4. WarpDetector   — Warp Terminal fallback (always Some; handles claude titles too)
    let detectors: Vec<Box<dyn AppDetector>> = vec![
        Box::new(NeovimDetector),
        Box::new(RustDetector),
        Box::new(GitDetector),
        Box::new(WarpDetector),
    ];

    // Session timer resets only when no detector fires (presence fully cleared).
    let mut session_start: Option<Instant> = None;

    // Require this many consecutive None results before clearing presence.
    // A single missed poll (Warp-internal subprocess briefly stealing focus
    // during tab switches) must not wipe the Discord card.
    let mut none_streak: u32 = 0;
    const CLEAR_AFTER: u32 = 2;

    loop {
        // monitor.snapshot() returns None when Warp is not the focused window,
        // which short-circuits the entire detector chain and clears presence.
        let result = monitor.snapshot().and_then(|snap| {
            // ── Debug output (stripped from release builds) ───────────────────
            #[cfg(debug_assertions)]
            {
                eprintln!("[warp-rpc] title    = {:?}", snap.title);
                let fired = detectors
                    .iter()
                    .enumerate()
                    .find_map(|(i, d)| d.detect(&snap.title).map(|p| (i, p)));
                if let Some((i, data)) = fired {
                    eprintln!(
                        "[warp-rpc] detector = #{} | details = {:?} | state = {:?}",
                        i + 1,
                        data.details,
                        data.state,
                    );
                    return Some(data);
                }
                eprintln!("[warp-rpc] detector = none (all returned None)");
                return None;
            }
            // ── Release path — no debug overhead ──────────────────────────────
            #[cfg(not(debug_assertions))]
            detectors
                .iter()
                .find_map(|d| d.detect(&snap.title))
        });

        match result {
            Some(data) => {
                none_streak = 0;
                let started_at = *session_start.get_or_insert_with(Instant::now);
                discord.update(&data, started_at);
            }
            None => {
                none_streak += 1;
                #[cfg(debug_assertions)]
                eprintln!("[warp-rpc] none_streak={none_streak}/{CLEAR_AFTER}");
                if none_streak >= CLEAR_AFTER {
                    if session_start.take().is_some() {
                        discord.clear();
                    }
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
