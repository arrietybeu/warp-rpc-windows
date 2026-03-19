# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`warp-rpc-windows` is a Rust project (edition 2024) targeting Windows. Currently a scaffold — the codebase consists of a single `src/main.rs` entry point with no dependencies yet.

## Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Run
cargo run

# Run tests
cargo test

# Run a single test by name
cargo test <test_name>

# Lint
cargo clippy

# Format
cargo fmt
```

## Architecture

- `src/main.rs` — program entry point
- The project name (`warp-rpc`) suggests this will implement an RPC (Remote Procedure Call) layer, likely using the [warp](https://github.com/seanmonstar/warp) web framework, targeting Windows specifically
