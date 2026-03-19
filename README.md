# warp-rpc-windows

A lightweight Windows background process that monitors **Warp Terminal** and updates your **Discord Rich Presence** in real time.

## Features

- Detects the active Warp tab's title every 5 seconds via `GetForegroundWindow`
- Priority detector chain — shows the most specific activity automatically:
  1. **Neovim** — title contains `nvim` / `neovim`
  2. **Rust/Cargo** — title contains `cargo` / `rust` / `.rs`
  3. **Git** — title contains `git`
  4. **Warp Terminal** (fallback) — always fires; shows `claude` asset if title contains `claude`
- Clears presence immediately when you alt-tab away from Warp
- Debounce: transient focus-steals (tab switching) don't wipe the presence card

## Setup

### 1. Discord Developer Portal

1. Go to <https://discord.com/developers/applications> → create an app
2. **General Information → Name** → set to `Warp Pro` (this becomes the "playing" label)
3. **OAuth2 → Client ID** → copy the ID
4. **Rich Presence → Art Assets** → upload images with these exact keys:
   - `warp` — Warp Terminal logo
   - `claude` — Claude AI logo
   - `neovim` — Neovim logo
   - `rust` — Rust / Ferris logo
   - `git` — Git logo

### 2. Configure Client ID

Edit `src/main.rs`:

```rust
const CLIENT_ID: u64 = 0; // ← replace with your Client ID
```

### 3. Build

```bash
cargo build --release        # no console window
cargo build                  # debug build (console visible, debug logs printed)
cargo run                    # run in debug mode
```

The release binary is at `target/release/warp-rpc-windows.exe`. Add it to Windows startup via Task Scheduler or the Startup folder.

## Architecture

```
src/
├── main.rs              – polling loop (5 s), debounce, detector chain
├── models.rs            – PresenceData struct
├── monitor.rs           – SystemMonitor: GetForegroundWindow → PID check → GetWindowTextW
├── presence.rs          – PresenceManager: single persistent discord-presence Client
└── strategies/
    ├── mod.rs           – AppDetector trait: fn detect(&self, title: &str) -> Option<PresenceData>
    ├── neovim.rs        – priority 1
    ├── rust.rs          – priority 2
    ├── git.rs           – priority 3
    └── warp.rs          – priority 4 (guaranteed fallback)
```

## Key Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `discord-presence` | 3 | Discord IPC RPC client |
| `sysinfo` | 0.31 | Process enumeration (warp.exe PID list) |
| `windows` | 0.58 | `GetForegroundWindow`, `GetWindowTextW`, `GetWindowThreadProcessId` |
