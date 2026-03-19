// Hide the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod monitor;
mod presence;
mod strategies;

use models::PresenceData;
use monitor::PollResult;
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
    //  4. WarpDetector   — guaranteed fallback (always Some; handles claude titles)
    let detectors: Vec<Box<dyn AppDetector>> = vec![
        Box::new(NeovimDetector),
        Box::new(RustDetector),
        Box::new(GitDetector),
        Box::new(WarpDetector),
    ];

    // Sticky presence state.
    //
    //  last_presence — the most recent PresenceData produced while Warp was active.
    //                  Shown (with a "(Background)" suffix) when Warp is running but
    //                  not in the foreground.  Cleared only when warp.exe exits.
    //  session_start — when the current Warp session began.  Drives the Discord
    //                  elapsed-time timer.  Persists across detector changes and
    //                  background/foreground transitions.
    let mut last_presence: Option<PresenceData> = None;
    let mut session_start: Option<Instant> = None;

    loop {
        match monitor.poll() {
            // ── Active: Warp is the focused window ────────────────────────────
            PollResult::Active(snap) => {
                #[cfg(debug_assertions)]
                eprintln!("[warp-rpc] active  | title = {:?}", snap.title);

                if let Some(data) = run_detectors(&detectors, &snap.title) {
                    last_presence = Some(data.clone());
                    let started_at = *session_start.get_or_insert_with(Instant::now);
                    discord.update(&data, started_at);
                }
            }

            // ── Background: Warp is running but not focused ───────────────────
            //
            // Keep last_presence on Discord with "(Background)" appended to
            // details so the user can see they have an active terminal session
            // without being at the keyboard.
            PollResult::Background => {
                #[cfg(debug_assertions)]
                eprintln!("[warp-rpc] background | warp running but not foreground");

                if let Some(ref data) = last_presence {
                    let started_at = *session_start.get_or_insert_with(Instant::now);
                    let mut bg = data.clone();
                    bg.details = format!("{} (Background)", data.details);
                    discord.update(&bg, started_at);
                }
            }

            // ── Gone: warp.exe is not running ────────────────────────────────
            PollResult::Gone => {
                #[cfg(debug_assertions)]
                eprintln!("[warp-rpc] gone    | warp.exe not running — clearing");

                last_presence = None;
                if session_start.take().is_some() {
                    discord.clear();
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Run the detector chain and return the first match, logging in debug builds.
fn run_detectors(
    detectors: &[Box<dyn AppDetector>],
    title: &str,
) -> Option<PresenceData> {
    #[cfg(debug_assertions)]
    {
        let fired = detectors
            .iter()
            .enumerate()
            .find_map(|(i, d)| d.detect(title).map(|p| (i, p)));
        if let Some((i, data)) = fired {
            eprintln!(
                "[warp-rpc]           detector = #{} | details = {:?} | state = {:?}",
                i + 1,
                data.details,
                data.state,
            );
            return Some(data);
        }
        eprintln!("[warp-rpc]           detector = none");
        return None;
    }

    #[cfg(not(debug_assertions))]
    detectors.iter().find_map(|d| d.detect(title))
}
