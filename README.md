# setups

Pre-market trade setup scanner. Pulls quotes and news from Finnhub, scores each gapper on gap %, relative volume, catalyst type, squeeze potential, and options flow, then displays ranked results in a TUI dashboard or plain-text table.

## Installation

### Homebrew (macOS)

```sh
brew tap holynakamoto/setups
brew install setups
```

After installing, set your Finnhub API key:

```sh
export FINNHUB_API_KEY=your_key_here
```

A free key is available at [finnhub.io](https://finnhub.io). See [`.env.example`](.env.example) for the full list of supported environment variables.

### From source

Requires Rust 1.70+.

```sh
git clone https://github.com/holynakamoto/setups.git
cd setups
cp .env.example .env
# Edit .env and set FINNHUB_API_KEY
cargo build --release
./target/release/setups
```

## Usage

```sh
# Full pre-market scan (TUI dashboard)
setups

# Plain-text table output
setups --plain

# Custom scan filters
setups --min-gap 3.0 --min-rvol 2.0 --top 10

# Look up a single symbol
setups symbol NVDA
```

### TUI keybindings

| Key | Action |
|-----|--------|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `o` | Open catalyst article in browser |
| `q` / `Ctrl-C` | Quit |

## Configuration

Copy `.env.example` to `.env` and fill in your API key:

```sh
cp .env.example .env
```

Required:

| Variable | Description |
|----------|-------------|
| `FINNHUB_API_KEY` | Finnhub API key (free tier supported) |
