mod data;
mod analysis;
mod models;
mod ui;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;

use data::FinnhubClient;
use analysis::{Screener, screener::ScreenerConfig};
use models::{Setup, earnings_is_imminent, EARNINGS_IMMINENT_WINDOW_DAYS};
use ui::Dashboard;

#[derive(Parser)]
#[command(name = "setups", about = "Pre-market trade setup scanner — powered by Finnhub")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Minimum gap % to include (default: 5.0)
    #[arg(long, default_value = "5.0")]
    min_gap: f64,

    /// Minimum relative volume multiplier (default: 1.5)
    #[arg(long, default_value = "1.5")]
    min_rvol: f64,

    /// Minimum price filter (default: 2.0)
    #[arg(long, default_value = "2.0")]
    min_price: f64,

    /// Maximum price filter (default: 500.0)
    #[arg(long, default_value = "500.0")]
    max_price: f64,

    /// Number of top setups to display (default: 20)
    #[arg(long, default_value = "20")]
    top: usize,

    /// Stop-loss distance from entry, in percent (default: 5.0)
    #[arg(long, default_value = "5.0")]
    stop_pct: f64,

    /// Profit-target distance as a multiple of the stop distance (default: 2.0)
    #[arg(long, default_value = "2.0")]
    reward_mult: f64,

    /// Print as plain text table instead of opening the TUI dashboard
    #[arg(long)]
    plain: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Scan a specific symbol and print its stats
    Symbol { symbol: String },
    /// Run the full pre-market scan (default)
    Scan,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("setups=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let cli = Cli::parse();

    let api_key = std::env::var("FINNHUB_API_KEY")
        .context("FINNHUB_API_KEY not set — get a free key at finnhub.io and add it to .env")?;

    let config = ScreenerConfig {
        min_gap_pct: cli.min_gap,
        min_relative_volume: cli.min_rvol,
        min_price: cli.min_price,
        max_price: cli.max_price,
        top_n: cli.top,
        stop_pct: cli.stop_pct,
        reward_mult: cli.reward_mult,
    };

    match cli.command.unwrap_or(Command::Scan) {
        Command::Scan => {
            print_market_status();
            let screener = Screener::new(FinnhubClient::new(api_key));
            info!("Starting pre-market scan...");
            let setups = screener.scan(&config).await?;
            if setups.is_empty() {
                println!("No setups found matching your criteria (try lowering --min-gap).");
                return Ok(());
            }
            if cli.plain || !is_interactive_tty() {
                print_table(&setups);
            } else {
                Dashboard::new(setups).run()?;
            }
        }
        Command::Symbol { symbol } => {
            let client = FinnhubClient::new(api_key);
            let ticker = client.build_ticker_fresh(&symbol).await?;
            let (headline, catalyst) = client
                .get_top_catalyst(&symbol)
                .await?
                .unwrap_or_else(|| ("No recent news".to_string(), models::CatalystType::Unknown));

            println!("Symbol:          {}", ticker.symbol);
            println!("Pre-market:      ${:.2}", ticker.premarket_price);
            println!("Prev close:      ${:.2}", ticker.prev_close);
            println!("Gap:             {:+.2}%", ticker.gap_pct());
            println!("Relative volume: {:.2}x", ticker.relative_volume());

            let is_long = ticker.gap_pct() >= 0.0;
            let levels = analysis::indicators::trade_levels(
                ticker.premarket_price,
                is_long,
                config.stop_pct,
                config.reward_mult,
            );
            println!("Direction:       {}", if is_long { "LONG" } else { "SHORT" });
            println!("Entry:           ${:.2}", levels.entry);
            println!("Stop:            ${:.2}", levels.stop);
            println!("Target:          ${:.2}", levels.target);
            println!("Risk/Reward:     {:.2}", levels.risk_reward);

            let next_earnings = client.get_next_earnings(&symbol).await;
            let earnings_str = match next_earnings {
                Some(d) => {
                    let today = chrono::Utc::now().date_naive();
                    if earnings_is_imminent(d, today, EARNINGS_IMMINENT_WINDOW_DAYS) {
                        format!("{} (imminent)", d)
                    } else {
                        d.to_string()
                    }
                }
                None => "—".to_string(),
            };
            println!("Earnings:        {}", earnings_str);

            println!("Catalyst:        {}", catalyst);
            println!("News:            {}", headline);
        }
    }

    Ok(())
}

fn print_market_status() {
    use chrono::{Datelike, Timelike, Utc, Weekday};

    let now = Utc::now();
    let weekday = now.weekday();
    let utc_mins = now.hour() * 60 + now.minute();

    // All offsets in summer (daylight) time — EDT/MDT/PDT
    // Winter: ET=-5, MT=-7, PT=-8 (subtract 60 from each threshold below)
    let et_mins  = (utc_mins + 24 * 60).saturating_sub(4 * 60) % (24 * 60);
    let mt_mins  = (utc_mins + 24 * 60).saturating_sub(6 * 60) % (24 * 60);
    let pt_mins  = (utc_mins + 24 * 60).saturating_sub(7 * 60) % (24 * 60);

    let fmt = |m: u32| format!("{:02}:{:02}", m / 60, m % 60);

    // Pre-market: 4:00–9:30 AM ET = 480–570 mins
    // Regular:    9:30 AM–4:00 PM ET = 570–960 mins
    let is_weekend  = matches!(weekday, Weekday::Sat | Weekday::Sun);
    let in_premarket = !is_weekend && et_mins >= 480 && et_mins < 570;
    let in_regular   = !is_weekend && et_mins >= 570 && et_mins < 960;

    let session = if is_weekend {
        "WEEKEND — markets closed"
    } else if in_premarket {
        "PRE-MARKET — live signals active"
    } else if in_regular {
        "REGULAR HOURS"
    } else {
        "AFTER HOURS / OVERNIGHT"
    };

    println!("  {}  |  ET {}  MT {}  PT {}", session, fmt(et_mins), fmt(mt_mins), fmt(pt_mins));
    if is_weekend {
        println!("  Gap data reflects Friday close. RVOL filter inactive until Monday 4:00 AM ET / 2:00 AM MT / 1:00 AM PT.");
    } else if !in_premarket {
        println!("  Pre-market opens 4:00 AM ET  |  2:00 AM MT  |  1:00 AM PT");
    }
    println!();
}

fn is_interactive_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

fn print_table(setups: &[Setup]) {
    println!(
        "\n{:<8} {:>8} {:>8} {:>6} {:>9} {:>8} {:>8} {:>8} {:>5} {:>6}  {:<18}  {:>5}  {:<16}  {}",
        "SYMBOL", "PRICE", "GAP%", "RVOL", "SHORT%", "ENTRY", "STOP", "TARGET", "R:R", "SCORE", "CATALYST", "GRADE", "EARNINGS", "NEWS"
    );
    println!("{}", "-".repeat(168));
    for s in setups {
        let short = s
            .ticker
            .short_float_pct
            .map(|p| format!("{:.1}%", p))
            .unwrap_or_else(|| "N/A".into());
        let earnings = match s.next_earnings {
            Some(d) => {
                if s.earnings_imminent(EARNINGS_IMMINENT_WINDOW_DAYS) {
                    format!("{} SOON", d)
                } else {
                    d.to_string()
                }
            }
            None => "—".to_string(),
        };
        let news = s
            .catalyst_headline
            .as_deref()
            .unwrap_or("—")
            .chars()
            .take(40)
            .collect::<String>();
        println!(
            "{:<8} {:>8.2} {:>+7.1}% {:>5.1}x {:>9} {:>8.2} {:>8.2} {:>8.2} {:>5.2} {:>6.0}  {:<18}  {:>5}  {:<16}  {}",
            s.ticker.symbol,
            s.ticker.premarket_price,
            s.ticker.gap_pct(),
            s.ticker.relative_volume(),
            short,
            s.levels.entry,
            s.levels.stop,
            s.levels.target,
            s.levels.risk_reward,
            s.score.total,
            format!("{}", s.catalyst).chars().take(18).collect::<String>(),
            s.score.grade(),
            earnings,
            news,
        );
    }
    println!();
}
