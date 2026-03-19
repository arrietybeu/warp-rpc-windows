# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Warpcord-Win** — a lightweight Windows tray-less background process that monitors Warp Terminal and updates the user's Discord Rich Presence in real time.

## Commands

```bash
cargo build                  # debug build (console window visible)
cargo build --release        # release build (no console window)
cargo run                    # run in debug mode
cargo clippy -- -D warnings  # lint
cargo fmt                    # format
```

No tests exist yet; the project is a single binary with no library crate.

## Configuration

Before building, set your Discord Application **Client ID** in `src/main.rs`:

```rust
const CLIENT_ID: u64 = 0; // ← replace with your real ID
```

Get it from <https://discord.com/developers/applications> → your app → OAuth2 → Client ID.

The asset keys `warp` and `claude` must also be uploaded in the Discord Developer Portal under **Rich Presence → Art Assets**.

## Architecture

```
src/
├── main.rs        – entry point; 5-second polling loop; orchestrates the two modules
├── monitor.rs     – WarpWatcher: sysinfo process scan → EnumWindows → GetWindowTextW
└── presence.rs    – PresenceManager: discord-presence Client wrapper
```

### Data flow

1. `WarpWatcher::window_title()` calls `sysinfo` to collect PIDs of `warp.exe`, then calls `EnumWindows` with a `SearchState` passed via `LPARAM`. The `unsafe extern "system"` callback (`enum_callback`) filters by PID + visibility and reads the title with `GetWindowTextW`.
2. The title (or `None`) is returned to the main loop.
3. `PresenceManager::update()` calls `discord_presence::Client::set_activity` with:
   - `large_image` = `"claude"` if title contains "claude" (case-insensitive), else `"warp"`
   - `timestamps.start` = Unix epoch of when Warp was first detected (derived from `Instant`)
4. On Warp exit, `PresenceManager::clear()` clears the presence.

### Key design decisions

- **`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`** suppresses the console only in release; debug keeps it for log output.
- The `System` instance in `WarpWatcher` is reused across polls (`refresh_processes_specifics` is called each iteration) to avoid reinitializing the sysinfo context.
- All Discord errors are silently swallowed (`Result::ok()`). On failure `update()` reconnects once and retries; it never panics.

## Key Dependencies

| Crate | Purpose |
|---|---|
| `discord-presence 3` | Discord IPC RPC client |
| `sysinfo 0.31` | Cross-platform process enumeration |
| `windows 0.58` | `EnumWindows`, `GetWindowTextW`, `GetWindowThreadProcessId`, `IsWindowVisible` |
