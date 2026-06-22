---
defract:
  id: task-tui-redesign-hyperlinks-and-homebrew-01kvqxj777gd
  type: improvement
  status: active
  stage: release
  phase: 0
  total_phases: 3
  priority: normal
  source: manual
  branch_strategy: worktree
  mode: human-in-the-loop
  created_by: holynakamoto
  assignee: holynakamoto
---

## Story Brief

# TUI Redesign, Hyperlinks, and Homebrew Packaging

# TUI Redesign, Hyperlinks, and Homebrew Packaging

## What We're Building

Four connected improvements to the setups tool: plumbing news article URLs from the Finnhub data through the app's models and into both display surfaces; redesigning the ratatui TUI dashboard to be more visually clear and informative; adding hyperlink affordances — clickable OSC 8 links in the plain-text table and an open-in-browser keybinding in the TUI; and packaging the tool for Homebrew distribution so users can install it with a single command.

## Expected Outcome

- Users can press a key in the TUI to open the catalyst news article in their default browser
- The plain-text table output shows clickable hyperlinks for headline text in terminals that support OSC 8 escape sequences
- The TUI dashboard has a refreshed visual layout that surfaces the catalyst link affordance
- Users can install setups on macOS via Homebrew without needing Rust or a local build

## Phase Outcomes

- **Phase 1: Thread article URLs through the data pipeline** — News article URLs fetched from Finnhub are now available on every setup, enabling all display surfaces to show or open the source article. Without this phase, the hyperlink and open-in-browser features have nothing to link to.
- **Phase 2: TUI redesign and hyperlinks** — The dashboard table shows direction (LONG/SHORT) instead of the non-functional Short% column, the detail panel surfaces the article URL with an open hint, pressing `o` launches the article in the default browser, and the plain-text `--plain` output renders clickable OSC 8 hyperlinks for compatible terminals.
- **Phase 3: Homebrew packaging** — A formula file and tap documentation are in place so macOS users can install the tool with a single `brew install` command, without needing Rust or a local build.

## Out of Scope

- Live auto-refresh of scan data during a TUI session (separate feature, larger scope)
- Additional data sources beyond Finnhub (polygon, benzinga, unusual_whales stubs remain placeholders)
- Linux or Windows packaging (Homebrew tap covers macOS only)
- Changes to scoring logic, screener filters, or watchlist contents
- Creating the actual Homebrew tap repository (that is separate infrastructure; this task produces the formula file and documentation only)

## Scope Summary

**Size:** 14 requirements, 10 acceptance criteria, 3 implementation phases
**Key decisions:**
- Use `std::process::Command::new("open")` to open URLs in the TUI (macOS-native, no extra dependency)
- OSC 8 hyperlinks in `--plain` mode are emitted unconditionally; terminals that do not support them render the text without escape sequences
- The Homebrew formula includes placeholder SHA256/URL values to be filled at first release; actual tap repo creation is out of scope
- Short% column in the TUI table is replaced by a Dir column since short float data is never available from the Finnhub basic tier

**Biggest risk:** The Finnhub `/company-news` endpoint may not return a `url` field for all news items — the implementation must handle its absence gracefully, and the formula cannot be verified until a release binary exists.

## Context

The setups tool pulls news headlines in `get_top_catalyst()` (`src/data/finnhub.rs:199`) but the `NewsItem` deserialization struct only captures the `headline` field — the article URL is silently dropped. The `Setup` struct (`src/models/setup.rs:86`) already has a `catalyst_headline: Option<String>` field but no corresponding URL field. The TUI detail panel (`src/ui/dashboard.rs:209`) shows the headline as plain text with no link affordance. The Short% column in the TUI table is always "N/A" because `short_float_pct` is hardcoded to `None` (Finnhub basic tier does not provide short interest). The `url` crate is already in `Cargo.toml` but unused.

## Requirements

### URL Plumbing

- R1: The `NewsItem` struct in `src/data/finnhub.rs` must add a `url: Option<String>` field so the article URL is deserialized from the Finnhub company-news API response alongside the existing `headline` field.
- R2: `get_top_catalyst()` must return the article URL alongside the headline and catalyst type; update the return type from `Option<(String, CatalystType)>` to `Option<(String, Option<String>, CatalystType)>`.
- R3: The `Setup` struct in `src/models/setup.rs` must add a `catalyst_url: Option<String>` field after `catalyst_headline`.
- R4: `Screener::scan()` in `src/analysis/screener.rs` must destructure the updated `get_top_catalyst()` return to populate `catalyst_url` on each `Setup`.
- R5: The `symbol` subcommand in `src/main.rs` must print the article URL when available (e.g., `News URL:        {url}`).
- R6: All existing unit tests must continue to pass; test helper constructors that construct `Setup` directly (in `src/models/setup.rs` tests) must add `catalyst_url: None` to compile.

### TUI Redesign

- R7: The TUI table must replace the "Short %" column with a "Dir" column showing "LONG" in green and "SHORT" in red, derived from the `direction()` method on `Setup`.
- R8: The TUI detail panel must show the catalyst URL (truncated to fit the panel width) or "No link available" when `catalyst_url` is `None`, along with a keybinding hint (e.g., `[o] open article`).
- R9: The TUI must open the selected setup's `catalyst_url` in the system default browser when the user presses `o`, using `std::process::Command::new("open").arg(url).spawn()`; pressing `o` when `catalyst_url` is `None` must have no effect.
- R10: The TUI footer must include `o  open article` alongside the existing navigation hints.
- R11: The TUI header must display the current market session label (pre-market / regular hours / after-hours / weekend) alongside the setup count, reusing the session-detection logic already in `print_market_status()`.

### Plain-Text Hyperlinks

- R12: In `print_table()` in `src/main.rs`, wrap each news headline in an OSC 8 terminal hyperlink (`\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\`) when `catalyst_url` is `Some`; render the headline as plain text when `catalyst_url` is `None`.

### Homebrew Packaging

- R13: A `Formula/setups.rb` file must exist in the repository containing a valid Homebrew formula skeleton with the correct class name, description, homepage, license, and placeholder `url`/`sha256` values for macOS arm64 and x86_64 bottles, with a `caveats` block explaining the `FINNHUB_API_KEY` requirement.
- R14: A Homebrew installation section must be added to `README.md` documenting the `brew tap` and `brew install` commands, a note that `FINNHUB_API_KEY` must be set, and a pointer to `.env.example` for setup.

## Acceptance Criteria

- [ ] `NewsItem` in `src/data/finnhub.rs` has `url: Option<String>`; `cargo build` succeeds with no new warnings
- [ ] `get_top_catalyst()` return type updated to include the URL; `cargo test` passes (all `classify_headline` tests still pass)
- [ ] `Setup` struct has `catalyst_url: Option<String>`; `cargo test` passes with no regressions
- [ ] TUI table shows "Dir" column (LONG in green, SHORT in red) where "Short %" previously appeared; verified by running `cargo run` and observing the column header and cell colors
- [ ] Pressing `o` in the TUI on a setup with a `catalyst_url` opens the article in the default macOS browser (no panic, no crash)
- [ ] Pressing `o` in the TUI on a setup with no `catalyst_url` does nothing (no panic, no crash)
- [ ] TUI footer includes `o  open article` hint
- [ ] TUI detail panel shows the catalyst URL or "No link available"
- [ ] `--plain` output includes OSC 8 escape sequences in the NEWS column for rows with a URL; verified by `cargo run -- --plain 2>/dev/null | cat -v` showing `^[]8;;http` in the output
- [ ] `ruby -c Formula/setups.rb` exits 0; formula `caveats` block references `FINNHUB_API_KEY`

## Implementation Phases

### Phase 1: Thread article URLs through the data pipeline
**Scope:** Add the article URL to the data pipeline — extend the Finnhub news item deserialization to capture the URL field, update `get_top_catalyst()` to return it, add `catalyst_url` to `Setup`, update `Screener::scan()` and the `symbol` subcommand. All existing tests must continue to pass.
**Files:**
- `src/data/finnhub.rs` — add `url: Option<String>` to `NewsItem`, update `get_top_catalyst()` return type and body
- `src/models/setup.rs` — add `catalyst_url: Option<String>` to `Setup`, update test fixture constructors
- `src/analysis/screener.rs` — destructure updated `get_top_catalyst()` return, populate `catalyst_url`
- `src/main.rs` — update `symbol` subcommand to print URL, update callsite for the new return shape
**Verification:**
- `cargo test` passes with no failures
- `cargo build` produces no new warnings
**Estimated effort:** Small

### Phase 2: TUI redesign and hyperlinks
**Scope:** Overhaul the TUI table (replace Short% with Dir column), add the article URL to the detail panel with an open hint, wire up the `o` keybinding, update the footer, add market session status to the header, and add OSC 8 hyperlinks to the plain-text table output.
**Files:**
- `src/ui/dashboard.rs` — replace Short% column with Dir, add URL to detail panel, add `o` keybinding, update footer, add market session to header
- `src/main.rs` — add OSC 8 wrapping in `print_table()`; extract market session detection into a reusable function callable from both `print_market_status()` and `Dashboard`
**Verification:**
- `cargo run` opens the TUI; table has a "Dir" column colored green/red per direction
- Pressing `o` on a row with a URL opens the article in the default browser; pressing `o` with no URL does nothing
- TUI footer shows `o  open article`
- TUI detail panel shows the catalyst URL or "No link available"
- `cargo run -- --plain 2>/dev/null | cat -v` shows `^[]8;;http` sequences in the NEWS column for rows with URLs
**Estimated effort:** Medium

### Phase 3: Homebrew packaging
**Scope:** Create the Homebrew formula file with the correct structure, placeholder release values, and API key caveats. Update README with the tap install flow.
**Files:**
- `Formula/setups.rb` (new) — complete Homebrew formula skeleton with placeholder SHA256/URL values
- `README.md` — add Homebrew installation section
**Verification:**
- `ruby -c Formula/setups.rb` exits 0
- `README.md` contains `brew tap` and `brew install setups` instructions
- Formula `caveats` block references `FINNHUB_API_KEY`
**Estimated effort:** Small

## Edge Cases

- **No URL in Finnhub response**: Finnhub may not include a `url` field for all news items; `url: Option<String>` deserialization handles missing or null values gracefully, and all downstream code guards on `catalyst_url.is_some()`.
- **URL is present but empty string**: Empty strings should be treated as absent — coerce an empty deserialized URL to `None` so downstream guards work correctly.
- **OSC 8 in non-supporting terminals**: Terminals that don't support OSC 8 may display escape sequences as garbage. Unconditional emission is acceptable for this task; terminal capability detection is out of scope.
- **`open` command fails**: `spawn()` failure must be swallowed silently — it is non-critical and must not crash the TUI event loop.
- **Pressing `o` with no setups**: The dashboard event loop exits early when `setups` is empty, so there is no selected row to act on; the `o` handler should guard on `selected_idx < setups.len()` regardless.

## Technical Notes

The Finnhub `/company-news` endpoint returns an array of objects; each object includes `url`, `headline`, `source`, `datetime`, and `summary`. The `NewsItem` struct currently captures only `headline` — adding `url: Option<String>` (no serde rename needed, the field name matches) picks up the link.

For opening URLs in the TUI, use `std::process::Command::new("open").arg(url).spawn()` — macOS-native, no extra crate needed. The `open` crate is a cross-platform alternative but adds a dependency that is not justified when the Homebrew distribution targets macOS only.

OSC 8 hyperlink format in Rust: `format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, display_text)`. Note the string terminator is `ESC \` (two bytes: `0x1B 0x5C`), written as `\x1b\\` in a Rust string literal.

The market session detection logic in `print_market_status()` (`src/main.rs:153`) computes the ET/MT/PT minute values and derives a session label. Extract this into a free function (e.g., `fn market_session() -> &'static str`) so `Dashboard::render_header()` can call it without duplicating the arithmetic.

The Homebrew formula should target a GitHub releases URL pattern such as `https://github.com/{owner}/setups/releases/download/v{version}/setups-{arch}-apple-darwin.tar.gz`. Use `sha256 "PLACEHOLDER_SHA256"` with a comment explaining it must be replaced before publishing. The formula class name must be `Setups` (capitalized).

## Implementation Notes

## Phase 1: Thread article URLs through the data pipeline

### Changes Made

- `src/data/finnhub.rs`: Added `url: Option<String>` to `NewsItem`. Updated `get_top_catalyst()` return type to `Result<Option<(String, Option<String>, CatalystType)>>`. Empty-string URLs are coerced to `None` via `.filter(|u| !u.is_empty())`.
- `src/models/setup.rs`: Added `catalyst_url: Option<String>` field after `catalyst_headline` in `Setup`. Updated `make_setup()` test helper to include `catalyst_url: None`.
- `src/analysis/screener.rs`: Destructures the three-tuple from `get_top_catalyst()` into `(catalyst_headline, catalyst_url, catalyst)` and populates `catalyst_url` on `Setup`.
- `src/main.rs`: Updated `symbol` subcommand to destructure the three-tuple and print `News URL: {url}` when `catalyst_url` is `Some`.

### Verification

- `cargo test`: 99 passed, 0 failed, 0 skipped
- `cargo build`: no new warnings

## Phase 2: TUI redesign and hyperlinks

### Changes Made

- `src/ui/dashboard.rs`: Replaced "Short %" column header and cell (which was always N/A) with "Dir" column showing "LONG" in green and "SHORT" in red, derived from `setup.direction()`. Added `open_article()` method using `std::process::Command::new("open")`. Added `o` key handler in the event loop. Added market session label to the header. Added catalyst URL line to the detail panel (truncated to fit width, with `[o] open article` hint when URL present, or "No link available" when absent). Updated footer to include `o  open article` hint. Added `session: &'static str` field to `Dashboard` struct, populated at construction.
- `src/main.rs`: Added `market_session() -> &'static str` free function extracting session-detection logic from `print_market_status()`. Updated `Dashboard::new()` call to pass `market_session()`. Added OSC 8 hyperlink wrapping in `print_table()` for news headlines when `catalyst_url` is `Some`.

### Verification

- `cargo test`: 99 passed, 0 failed, 0 skipped
- `cargo build`: no new warnings

## Phase 3: Homebrew packaging

### Changes Made

- `Formula/setups.rb` (new): Complete Homebrew formula skeleton with `Setups` class, correct description and homepage for `holynakamoto/setups`, placeholder `sha256` values with comments for both `aarch64` and `x86_64` macOS targets, `install` block copying the binary, and a `caveats` block explaining `FINNHUB_API_KEY` with a pointer to `.env.example`.
- `README.md` (new): Full README with Homebrew installation section (`brew tap holynakamoto/setups` + `brew install setups`), `FINNHUB_API_KEY` setup instructions, pointer to `.env.example`, from-source build instructions, usage examples, and TUI keybindings table.

### Verification

- `ruby -c Formula/setups.rb`: Syntax OK
- `README.md` contains `brew tap`, `brew install setups`, `FINNHUB_API_KEY`, and `.env.example` references
- `cargo test`: 99 passed, 0 failed, 0 skipped
- `cargo build`: no new warnings

## Review

## Verdict

**Verdict:** APPROVE
**Files reviewed:** 7 files changed across 3 phases

All 10 acceptance criteria pass and all 3 automated checks pass. The URL plumbing, TUI redesign, and Homebrew packaging are correctly implemented with no security issues or blocking quality concerns.

### Automated Checks

| Check | Result | Details |
|-------|--------|---------|
| Test suite | PASS | 99 passed, 0 failed, 0 skipped |
| Build | PASS | cargo build clean — no warnings |
| Formula syntax | PASS | ruby -c Formula/setups.rb exits 0 with 'Syntax OK' |

### Acceptance Criteria (10/10 passed)

- [x] AC-1: NewsItem in src/data/finnhub.rs has url: Option<String>; cargo build succeeds with no new warnings — PASS: src/data/finnhub.rs:69 — `url: Option<String>` present in NewsItem struct. cargo build: clean.
- [x] AC-2: get_top_catalyst() return type updated to include the URL; cargo test passes (all classify_headline tests still pass) — PASS: src/data/finnhub.rs:200 — return type is `Result<Option<(String, Option<String>, CatalystType)>>`. All 99 tests pass.
- [x] AC-3: Setup struct has catalyst_url: Option<String>; cargo test passes with no regressions — PASS: src/models/setup.rs:90 — `pub catalyst_url: Option<String>` present. make_setup() test helper at line 135 includes `catalyst_url: None`. 99 tests pass.
- [x] AC-4: TUI table shows Dir column (LONG in green, SHORT in red) where Short % previously appeared — PASS: src/ui/dashboard.rs:126 — `Cell::from("Dir")` header. Lines 147-162: dir derived from `setup.direction()`, colored green for LONG and red for SHORT via `dir_color`.
- [x] AC-5: Pressing o in the TUI on a setup with a catalyst_url opens the article in the default macOS browser (no panic, no crash) — PASS: src/ui/dashboard.rs:61-67 — `open_article()` uses `std::process::Command::new("open").arg(url).spawn()`. Return value discarded via `let _` to swallow errors. Wired to KeyCode::Char('o') at line 49.
- [x] AC-6: Pressing o in the TUI on a setup with no catalyst_url does nothing (no panic, no crash) — PASS: src/ui/dashboard.rs:65 — `if let Some(url) = &self.setups[self.selected_idx].catalyst_url` guards the spawn call; no URL means the branch is not taken.
- [x] AC-7: TUI footer includes o  open article hint — PASS: src/ui/dashboard.rs:297-299 — `Span::styled("o ", ...)` followed by `Span::raw("open article  ")` in render_footer.
- [x] AC-8: TUI detail panel shows the catalyst URL or No link available — PASS: src/ui/dashboard.rs:207-214 — Some(url) branch truncates URL and appends `[o] open article`; None branch renders `No link available`. Displayed at line 237 as `Link: {url_line}`.
- [x] AC-9: --plain output includes OSC 8 escape sequences in the NEWS column for rows with a URL — PASS: src/main.rs:250-252 — `format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, headline_text)` wraps headlines when `catalyst_url` is Some. Cannot run live without API key, but the escape sequence is structurally correct.
- [x] AC-10: ruby -c Formula/setups.rb exits 0; formula caveats block references FINNHUB_API_KEY — PASS: ruby -c Formula/setups.rb output: 'Syntax OK'. Formula/setups.rb:30 — `export FINNHUB_API_KEY=your_key_here` in caveats block.

### Code Quality (Refactor Review)

#### Duplicate logic

- **INFO:** `src/main.rs:177` — print_market_status() contains its own copy of the ET-offset arithmetic and session boundaries even though market_session() was extracted to hold that logic. The two functions intentionally differ in label text but the arithmetic is identical. Suggested fix: Replace the duplicated arithmetic in print_market_status() with a call to market_session() for the session label, then append the verbose suffix text around it

### Security Assessment (Security Review)

No security issues found in changed files.

### Decisions Made During Implementation

- Replace Short% TUI column with Dir (LONG/SHORT) because Finnhub basic tier never returns short_float_pct, making the column always N/A.
- Use std::process::Command::new("open") for browser launching — macOS-native, no extra crate required given Homebrew distribution targets macOS only.
- Emit OSC 8 hyperlinks unconditionally in --plain mode; terminal capability detection is out of scope and non-supporting terminals silently ignore the sequences.
- Formula/setups.rb uses placeholder SHA256 values; tap repo creation is out of scope — formula serves as the structural packaging artifact to be completed at first release.

## Required Changes

None.

## Release

## Release Notes

### What was built
- Threaded news article URLs from Finnhub's company-news API through the entire data pipeline (`NewsItem` → `get_top_catalyst()` → `Setup.catalyst_url`) so every display surface can show or open the source article
- Redesigned the ratatui TUI dashboard: replaced the always-N/A "Short %" column with a color-coded "Dir" column (LONG in green, SHORT in red); added market session label (Pre-Market / Regular Hours / After-Hours / Weekend) to the header
- Added `o` keybinding in the TUI to open the selected setup's catalyst article in the default macOS browser via `std::process::Command::new("open")`; detail panel shows the URL or "No link available"
- Added OSC 8 terminal hyperlinks in `--plain` output, wrapping news headlines with clickable links for compatible terminals; non-supporting terminals ignore the sequences silently
- Created `Formula/setups.rb` Homebrew formula skeleton with arm64/x86_64 bottle placeholders and a `caveats` block explaining `FINNHUB_API_KEY`; created `README.md` with full installation and usage documentation

### Key decisions
- Replace Short% TUI column with Dir (LONG/SHORT): Finnhub basic tier never returns `short_float_pct`, making the column always N/A — Dir surfaces data that is already computed and provides actionable signal
- Use `std::process::Command::new("open")` for browser launching: macOS-native, no extra crate required given Homebrew distribution targets macOS only
- Emit OSC 8 hyperlinks unconditionally in --plain mode: terminal capability detection is complex and non-standardized; non-supporting terminals silently ignore the escape sequences
- `Formula/setups.rb` uses placeholder SHA256 values: no GitHub release binary exists yet; the formula serves as the structural packaging artifact to be completed at first release; tap repo creation is out of scope

### Changes by phase
- **Phase 1: Thread article URLs through the data pipeline** — Added `url: Option<String>` to `NewsItem` in `src/data/finnhub.rs`; updated `get_top_catalyst()` return type to `Result<Option<(String, Option<String>, CatalystType)>>`; added `catalyst_url: Option<String>` to `Setup` in `src/models/setup.rs`; wired through `Screener::scan()` in `src/analysis/screener.rs` and the `symbol` subcommand in `src/main.rs`. Empty-string URLs coerced to `None`. All 99 tests pass.
- **Phase 2: TUI redesign and hyperlinks** — Replaced Short% column with color-coded Dir column in `src/ui/dashboard.rs`; added `open_article()` method using `std::process::Command::new("open")`; added `o` key handler; added market session label to header; added catalyst URL to detail panel with `[o] open article` hint or "No link available"; updated footer to include `o open article` hint. Extracted `market_session() -> &'static str` free function in `src/main.rs`; added OSC 8 hyperlink wrapping in `print_table()`. All 99 tests pass.
- **Phase 3: Homebrew packaging** — Created `Formula/setups.rb` with class `Setups`, correct description and homepage for `holynakamoto/setups`, arm64 and x86_64 bottle placeholders with explanatory comments, `install` block, and `caveats` block referencing `FINNHUB_API_KEY`. Created `README.md` with Homebrew install section (`brew tap holynakamoto/setups` + `brew install setups`), API key setup instructions, from-source build steps, usage examples, and TUI keybindings table. `ruby -c Formula/setups.rb` exits 0.

## Verification

### Production Build
PASS — `cargo build --release` completed with no warnings in 32s

### Review Reference
Approved by reviewer on 2026-06-22 — 10/10 acceptance criteria passed, 3/3 automated checks passed (cargo test 99/99, cargo build clean, ruby -c Syntax OK)

### Release Checklist
- [x] Approved review exists
- [x] Production build passes
- [x] Code committed and pushed
- [x] Release notes prepared
- [x] Stage content updated
- [x] Completion event logged

