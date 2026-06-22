---
defract:
  version: 1
  generated_at: "2026-06-22T00:00:00Z"
  updated_at: "2026-06-22T00:00:00Z"
  source: extracted
---

# Project Profile

## Overview

`setups` is a Rust CLI tool for pre-market trade setup scanning. It pulls quotes and news from the Finnhub API, scores each gapper on gap %, relative volume, catalyst type, squeeze potential, and options flow, then displays the ranked results in either a ratatui TUI dashboard or a plain-text table.

## Stack

- **Runtime**: Rust 2021 edition (tokio async runtime)
- **TUI**: ratatui 0.29 + crossterm 0.28
- **HTTP client**: reqwest 0.12 (rustls-tls)
- **CLI parsing**: clap 4 (derive)
- **Serialization**: serde / serde_json
- **Logging**: tracing + tracing-subscriber (env-filter)
- **Package manager**: Cargo

## Conventions

- All async I/O goes through `tokio::main` — no blocking calls in hot paths
- API calls to Finnhub are rate-limited manually with `tokio::time::sleep(1100ms)` between requests to stay under the free-tier 60/min cap (evidenced in `src/data/finnhub.rs`)
- Models derive `Serialize`/`Deserialize` for potential future persistence
- `indicators.rs` contains standalone pure functions (VWAP, ATR, EMA, gap_pct, relative_volume) that are currently unused (`#![allow(dead_code)]`)
- Dashboard renders a static snapshot of scan results; there is no live auto-refresh loop
- `short_float_pct` is always `None` today — the squeeze score component is wired up but never fires because Finnhub basic metrics don't return short-interest data

## File Structure

```
setups/
├── Cargo.toml              # package manifest, all dependencies
├── .env                    # local secrets (gitignored)
├── .env.example            # documents required env vars
└── src/
    ├── main.rs             # CLI entry point (clap), scan/symbol dispatch, print_table
    ├── models/
    │   ├── mod.rs          # re-exports
    │   ├── ticker.rs       # Ticker struct, gap_pct(), relative_volume(), is_low_float()
    │   └── setup.rs        # Setup, SetupScore, CatalystType, grade()
    ├── data/
    │   ├── mod.rs          # re-exports FinnhubClient
    │   ├── finnhub.rs      # Finnhub API client, hardcoded WATCHLIST (~70 symbols), get_gappers(), build_ticker(), get_top_catalyst(), classify_headline()
    │   ├── polygon.rs      # stub / placeholder
    │   ├── benzinga.rs     # stub / placeholder
    │   ├── unusual_whales.rs # stub / placeholder
    │   └── ortex.rs        # stub / placeholder
    ├── analysis/
    │   ├── mod.rs          # re-exports Screener, screener::ScreenerConfig
    │   ├── screener.rs     # Screener::scan() — filters, enriches, scores, sorts
    │   ├── scoring.rs      # score_setup() and sub-scorers for each signal component
    │   └── indicators.rs   # pure technical functions: VWAP, ATR, EMA, gap_pct, rvol (unused)
    └── ui/
        ├── mod.rs          # re-exports Dashboard
        └── dashboard.rs    # ratatui TUI: header, scrollable table, detail panel, footer
```

## Key Dependencies

### Core
- `tokio@1` — async runtime (full features)
- `reqwest@0.12` — HTTP client with rustls-tls
- `ratatui@0.29` — terminal UI framework
- `crossterm@0.28` — cross-platform terminal control
- `clap@4` — CLI argument parsing (derive macros)
- `serde@1` + `serde_json@1` — JSON serialization
- `chrono@0.4` — date/time for news lookups and market session detection
- `dotenvy@0.15` — `.env` file loading
- `anyhow@1` — ergonomic error handling
- `tracing@0.1` + `tracing-subscriber@0.3` — structured logging

## Build Commands

| Command | Description |
|---------|-------------|
| `cargo build` | Debug build |
| `cargo build --release` | Optimized release build (LTO enabled) |
| `cargo run` | Run full pre-market scan (TUI mode) |
| `cargo run -- --plain` | Run scan, plain-text table output |
| `cargo run -- symbol NVDA` | Look up a single symbol |
| `cargo run -- --min-gap 3.0 --min-rvol 2.0 --top 10` | Custom scan filters |

## Project-Specific Notes

- **Required env var**: `FINNHUB_API_KEY` must be set (free key from finnhub.io). The `.env` file is gitignored; `.env.example` documents the key name.
- **Watchlist is hardcoded** in `src/data/finnhub.rs` (~70 symbols across semis, mega-cap tech, biotech, EV, crypto, leveraged ETFs). A full scan takes ~80s at free-tier rate limits.
- **Stub data sources** (`polygon.rs`, `benzinga.rs`, `unusual_whales.rs`, `ortex.rs`) exist as placeholders for future integration; none are currently called.
- **Squeeze and options scores always return 0** for the scan flow because `short_float_pct`, `days_to_cover`, `call_put_ratio`, and `bullish_sweeps` are all hardcoded to 0/1 in `screener.rs:83-86`.
- **TUI navigation**: `↑/k` up, `↓/j` down, `q`/`Ctrl-C` quit. No refresh keybinding; re-run the binary to get fresh data.
