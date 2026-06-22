---
defract:
  id: task-would-like-to-add-entry-and-exit-01kvqsdz9c20
  type: task
  status: active
  stage: implementation
  phase: 0
  total_phases: 2
  priority: normal
  source: manual
  branch_strategy: worktree
  mode: human-in-the-loop
  created_by: holynakamoto
  assignee: holynakamoto
---

## Story Brief

Would like to add entry and exit positions, heat maps or matricies showing show sector by sector correlations, ai powere news synthesis.  Earnings and event calendar integration, sentiment and narrative tracking.  Advanced charting with AI overlays. Backtesting historical analogs, order flow and liquidity analisys.  Conversational AI analyst, anomoly detection, scenario modeling.

# Enhance setups tool with advanced trader analysis features

# Enhance setups tool with advanced trader analysis features

## What We're Building

The builder's vision is to evolve setups from a pre-market gap scanner into a comprehensive trading research platform. This task delivers the first, highest-value slice of that vision: making every scanned setup *actionable* and *event-aware*. Each setup will show a suggested entry, stop-loss, and profit target with a risk/reward ratio, and will flag when a company has earnings imminent. The larger feature areas in the brief (correlations, AI news synthesis, conversational analyst, backtesting, and more) are captured as follow-up work rather than attempted here.

## Expected Outcome

- For every setup, a trader sees a concrete suggested entry price, a stop-loss level, and a profit target instead of only a score and a gap percentage.
- Each setup shows a risk/reward ratio so a trader can judge at a glance whether the trade is worth taking.
- Traders can tune how aggressive the stop and target are without editing code.
- Setups carry an earnings warning so a trader knows when a company reports soon and the position carries event risk.
- Entry, exit, and earnings information appears everywhere setups are shown today (the interactive dashboard, the plain-text table, and the single-symbol lookup).

## Phase Outcomes

- **Phase 1: Actionable entry and exit levels** — Traders get a ready-to-use trade plan for every setup: where to get in, where to cut losses, where to take profit, and the risk/reward of doing so. They can dial the aggressiveness up or down to match their own risk appetite.
- **Phase 2: Earnings awareness** — Traders are warned when a setup's company reports earnings soon, so they avoid being blindsided by event risk on a position they were about to take.

## Out of Scope

The brief lists many ambitious feature areas. To ship value quickly and validate the direction, the following are deliberately deferred to separate follow-up work and are proposed for the backlog:

- Sector-by-sector correlation heat maps and matrices (needs historical cross-symbol price history we do not currently collect).
- AI-powered news synthesis and a conversational AI analyst (introduces a new language-model integration and is a substantial subsystem of its own).
- Sentiment and narrative tracking, anomaly detection, and scenario modeling (each is an independent analytical capability).
- Advanced charting with AI overlays (the tool is terminal-based today; rich charting implies a new presentation surface).
- Backtesting against historical analogs and order-flow / liquidity analysis (both require historical and order-level data feeds not available on the current data plan).

Within this task specifically, entry/exit levels use a transparent percentage-and-gap model from data we already fetch; volatility-based (ATR) stops that would require pulling intraday candle history are out of scope and noted as a future refinement.

## Scope Summary

**Size:** 12 requirements, 9 acceptance criteria, 2 implementation phases
**Key decisions:**
- Entry/stop/target are computed from existing quote data (`prev_close`, `premarket_price`) plus configurable risk knobs — zero new data dependencies in Phase 1.
- ATR-based stops are deferred because intraday candle data is not fetched and is restricted on Finnhub's free tier.
- The 9 remaining brief feature areas are deferred and proposed as backlog items, not scoped here.
**Biggest risk:** The entry/exit model is heuristic, not advice; level placement (especially default stop distance on very large gaps) must be sensible enough that traders trust the numbers.

## Context

`setups` is a Rust CLI that scans a hardcoded Finnhub watchlist for pre-market gappers, scores them, and renders results in a ratatui TUI (`src/ui/dashboard.rs`), a plain-text table (`src/main.rs:print_table`), or a single-symbol lookup (`Command::Symbol`). The `Setup` model (`src/models/setup.rs`) already exposes `direction()` (LONG/SHORT from gap sign), and `Ticker` (`src/models/ticker.rs`) carries `prev_close` and `premarket_price`. `src/analysis/indicators.rs` holds pure, currently-unused functions (`atr`, `vwap`, `ema`) under `#![allow(dead_code)]`. The `FinnhubClient` (`src/data/finnhub.rs`) fetches data through a shared `get<T>()` helper and sleeps 1.1s between calls to respect the 60/min free-tier cap. This task adds derived trade levels (Phase 1) and one new Finnhub endpoint for earnings dates (Phase 2), wiring both through all three output surfaces.

## Requirements

### Entry / Exit Levels (Phase 1)

- R1: Each setup computes a suggested entry, stop-loss, and profit target, direction-aware using the existing `Setup::direction()` (LONG for gap-ups, SHORT for gap-downs). (New levels live on a struct attached to `Setup` in `src/models/setup.rs`.)
- R2: Levels are derived only from data already fetched — `Ticker::premarket_price` as the entry reference and a configurable stop distance — so Phase 1 adds no new API calls.
- R3: A risk/reward ratio is computed from the entry-to-stop distance versus the entry-to-target distance and exposed on the levels struct.
- R4: Stop and target placement are controlled by two new CLI flags, `--stop-pct` (default 5.0) and `--reward-mult` (default 2.0), threaded through `ScreenerConfig` (`src/analysis/screener.rs`) the same way existing filters are. The target distance equals the stop distance multiplied by the reward multiple.
- R5: The level-computation logic is a pure, unit-testable function (extending `src/analysis/indicators.rs` or a sibling module), independent of any I/O.
- R6: Entry, stop, target, and risk/reward are rendered in the TUI detail panel (`render_detail`), the plain-text table (`print_table`), and the `Symbol` subcommand output.

### Earnings Calendar (Phase 2)

- R7: For each scored setup, the next upcoming earnings date is fetched from the Finnhub earnings calendar endpoint (`/calendar/earnings?from=&to=&symbol=`) via a new `FinnhubClient` method, reusing the existing `get<T>()` helper and 1.1s rate-limit sleep.
- R8: The next earnings date (when known) is stored on the setup model as an optional field, leaving setups without an upcoming earnings date unaffected.
- R9: A setup whose earnings fall within a near-term window (default 5 calendar days) is flagged as "imminent" so the UI can surface event risk.
- R10: Earnings date and the imminent flag are displayed in the TUI detail panel, the plain-text table, and the `Symbol` subcommand output.
- R11: The earnings fetch degrades gracefully — a failed or empty response leaves the setup with no earnings date and never aborts the scan (mirroring the existing `get_top_catalyst` error handling).
- R12: The added earnings call runs once per scored setup (≤ `top_n`), not once per watchlist symbol, to bound the extra scan time.

## Acceptance Criteria

- [ ] Running `cargo run -- --plain` shows an Entry, Stop, Target, and R:R value for each setup row; verified by inspecting the printed table.
- [ ] For a gap-up (LONG) setup, the stop is below the entry and the target is above it; for a gap-down (SHORT) setup, the stop is above the entry and the target is below it; verified by a unit test over both directions.
- [ ] `cargo run -- --stop-pct 10 --reward-mult 3` produces wider stops and proportionally further targets than the defaults; verified by comparing output for the same symbol.
- [ ] The risk/reward ratio equals the reward multiple within rounding for the default model; verified by a unit test.
- [ ] The level-computation function has unit tests covering LONG, SHORT, and a boundary (e.g. zero/near-zero gap) case; verified by `cargo test`.
- [ ] The TUI detail panel and the `cargo run -- symbol NVDA` output both display the entry, stop, target, and R:R; verified by running each.
- [ ] When a setup's company has earnings within the near-term window, an imminent-earnings indicator appears in the plain table and TUI detail; verified by running a scan during a known earnings week or with a stubbed date.
- [ ] A setup with no upcoming earnings date renders without error and shows a neutral placeholder (e.g. "—"); verified by running a scan.
- [ ] `cargo build` and `cargo test` succeed with no new warnings introduced for the touched modules; verified by running both.

## Implementation Phases

### Phase 1: Actionable entry and exit levels
**Scope:** Give every setup a concrete, direction-aware trade plan — suggested entry, stop-loss, profit target, and risk/reward — that traders can tune to their own risk tolerance, surfaced across all three output formats.
**Files:**
- `src/analysis/indicators.rs` — add a pure `trade_levels(...)` function (entry/stop/target/R:R) with unit tests; remove the function from the dead-code allowance as it becomes used.
- `src/models/setup.rs` — add a `TradeLevels` struct and an optional/owned field on `Setup`.
- `src/analysis/screener.rs` — extend `ScreenerConfig` with `stop_pct` and `reward_mult`; compute levels for each setup in `scan()`.
- `src/main.rs` — add `--stop-pct` and `--reward-mult` clap flags, thread them into `ScreenerConfig`, render levels in `print_table` and the `Symbol` arm.
- `src/ui/dashboard.rs` — render entry/stop/target/R:R in `render_detail` (and optionally a compact column in the table).
**Verification:**
- `cargo test` passes, including new unit tests for LONG, SHORT, and boundary cases.
- `cargo run -- --plain` shows Entry/Stop/Target/R:R columns with correct directional placement.
- `cargo run -- --stop-pct 10 --reward-mult 3` visibly widens stops and targets.
- `cargo run -- symbol NVDA` prints the four new fields.
- `cargo build` succeeds with no new warnings in touched modules.
**Estimated effort:** Medium

### Phase 2: Earnings awareness
**Scope:** Warn traders when a setup's company reports earnings soon, so they can account for event risk before taking a position; the warning appears wherever setups are shown.
**Files:**
- `src/data/finnhub.rs` — add a `get_next_earnings(symbol)` method hitting `/calendar/earnings`, with graceful error handling and the 1.1s sleep; add the response deserialization structs.
- `src/models/setup.rs` (or `ticker.rs`) — add an optional `next_earnings` date field and an `earnings_imminent(window_days)` helper.
- `src/analysis/screener.rs` — fetch the next earnings date once per scored setup and populate the model.
- `src/main.rs` — display earnings date / imminent flag in `print_table` and the `Symbol` arm.
- `src/ui/dashboard.rs` — display earnings date / imminent flag in `render_detail`.
**Verification:**
- `cargo build` and `cargo test` succeed.
- A scan run shows an earnings date (or "—") per setup, and an imminent indicator when within the window (validated with a known earnings week or stubbed date).
- A symbol with no upcoming earnings renders without error.
- Scan time increases by at most one extra API call per scored setup (not per watchlist symbol).
**Estimated effort:** Medium

## Edge Cases

- **Near-zero gap:** When `gap_pct()` is ~0, direction defaults to LONG (per existing `direction()`); levels still compute sensibly and the boundary is unit-tested.
- **Very large gap with percentage stop:** A fixed `--stop-pct` keeps the stop a bounded distance from entry rather than referencing the gap-fill, avoiding absurdly wide stops on 30%+ gappers; documented as a deliberate model choice.
- **Missing prev_close / zero price:** Setups already filter out zero quotes upstream; level math guards against division by zero and returns a neutral result if encountered.
- **Earnings endpoint empty or failing:** Leaves `next_earnings` as `None`, renders "—", and never aborts the scan.
- **Earnings date in the past or today:** Only future-dated earnings within the window count as "imminent"; past dates are ignored.
- **Off-hours / weekend scans:** Levels and earnings display identically; no dependency on live session state.

## Technical Notes

- Follow the existing config-threading pattern: new knobs go on `ScreenerConfig` (`src/analysis/screener.rs:11`) and are populated from clap flags in `src/main.rs:70`, mirroring `min_gap`/`min_rvol`.
- Keep the level math as a pure function in `src/analysis/indicators.rs` (alongside `atr`/`vwap`/`ema`) so it is unit-testable with no I/O, consistent with the existing pure-function convention noted in the project profile.
- New Finnhub calls must reuse `FinnhubClient::get<T>()` and the `tokio::time::sleep(Duration::from_millis(1100))` rate-limit pattern (`src/data/finnhub.rs:81`, `:139`), and degrade gracefully like `get_top_catalyst` (`:187`) which uses `.unwrap_or_default()`.
- Models derive `Serialize`/`Deserialize`; new fields on `Setup`/`Ticker` should follow suit per the existing convention.
- Render new TUI content within the existing detail panel (`render_detail`, fixed `Constraint::Length(10)` block in `render`); adding a compact table column is optional and must keep column constraints in sync with the header in `render_table`.
- No emoji or exclamation marks in any rendered output, matching the current UI tone.

### Dependencies

Phase 2 builds on the model field plumbing established in Phase 1 but is otherwise independent; Phase 1 is fully shippable on its own.

## Implementation Notes

## Phase 2: Earnings awareness — complete

Every scored setup now carries its next upcoming earnings date and an imminent-earnings flag, surfaced across the plain table, the `symbol` subcommand, and the TUI detail panel. The fetch degrades gracefully and runs at most once per scored setup.

### What was built

- **`src/data/finnhub.rs`** — added `get_next_earnings(symbol) -> Option<NaiveDate>` hitting `/calendar/earnings?from=&to=&symbol=` over a 90-day horizon, reusing the shared `get<T>()` helper and the 1.1s rate-limit sleep. It degrades like `get_top_catalyst`: a failed or empty response resolves to `unwrap_or_default()` and the method returns the earliest non-past date (or `None`), never aborting the scan. Added `EarningsCalendarResp` (with `#[derive(Default)]`) and `EarningsEntry` deserialization structs.
- **`src/models/setup.rs`** — added `next_earnings: Option<NaiveDate>` to `Setup`; a `Setup::earnings_imminent(window_days)` method; a free-standing pure `earnings_is_imminent(earnings, today, window_days)` predicate (past dates never count, today and the window boundary do); and `EARNINGS_IMMINENT_WINDOW_DAYS` (default 5). Six unit tests cover within-window, on-boundary, today, beyond-window, past-date, and the None case.
- **`src/models/mod.rs`** — re-export `earnings_is_imminent` and `EARNINGS_IMMINENT_WINDOW_DAYS`.
- **`src/analysis/screener.rs`** — sets `next_earnings: None` at build time, then after sorting and truncating to `top_n`, fetches the earnings date once per surviving setup. This bounds the extra calls to ≤ `top_n` (R12) rather than once per candidate.
- **`src/main.rs`** — `print_table` gains an `EARNINGS` column (date, or "—", with a `SOON` suffix when imminent); the `Symbol` subcommand fetches and prints an `Earnings:` line with an `(imminent)` marker.
- **`src/ui/dashboard.rs`** — `render_detail` gains an `Earnings` line showing the date (or "—"), bold-yellow with an `(imminent)` marker when within the window. Fits within the existing 10-line detail block.

### Deviations from plan

- None of substance. The imminence threshold lives in a shared `EARNINGS_IMMINENT_WINDOW_DAYS` constant rather than a CLI flag, since R9 specifies a fixed default window and no flag was requested.

### Verification

- `cargo build` clean, no new warnings.
- `cargo test`: 99 passed (93 baseline + 6 new earnings tests).
- Plain table shows an `EARNINGS` column; `symbol` and TUI detail show the earnings line; missing dates render "—".

### Files changed

`src/data/finnhub.rs`, `src/models/setup.rs`, `src/models/mod.rs`, `src/analysis/screener.rs`, `src/main.rs`, `src/ui/dashboard.rs`.
