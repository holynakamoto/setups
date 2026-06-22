use anyhow::Result;
use tracing::info;
use crate::data::FinnhubClient;
use crate::models::{Setup, CatalystType};
use crate::analysis::scoring::score_setup;
use crate::analysis::indicators::trade_levels;

pub struct Screener {
    finnhub: FinnhubClient,
}

pub struct ScreenerConfig {
    pub min_gap_pct: f64,
    pub min_relative_volume: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub top_n: usize,
    pub stop_pct: f64,
    pub reward_mult: f64,
}

impl Default for ScreenerConfig {
    fn default() -> Self {
        Self {
            min_gap_pct: 5.0,
            min_relative_volume: 1.5,
            min_price: 2.0,
            max_price: 500.0,
            top_n: 20,
            stop_pct: 5.0,
            reward_mult: 2.0,
        }
    }
}

impl Screener {
    pub fn new(finnhub: FinnhubClient) -> Self {
        Self { finnhub }
    }

    pub async fn scan(&self, config: &ScreenerConfig) -> Result<Vec<Setup>> {
        info!("Scanning for pre-market gappers >= {:.1}%", config.min_gap_pct);

        let candidates = self
            .finnhub
            .get_gappers(config.min_gap_pct, config.top_n * 2)
            .await?;

        info!("Found {} candidates, enriching data...", candidates.len());

        let mut setups = Vec::new();
        for (symbol, cached_quote) in candidates {
            // Price filter on the cached quote before spending more API calls
            if cached_quote.current < config.min_price || cached_quote.current > config.max_price {
                continue;
            }

            let ticker = match self.finnhub.build_ticker(&symbol, &cached_quote).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!("Skipping {}: {}", symbol, e);
                    continue;
                }
            };

            // Only apply RVOL filter when there is actual pre-market volume.
            // During off-hours (weekends, evenings) volume is 0 — don't filter on it.
            let has_active_volume = ticker.premarket_volume > 0;
            if has_active_volume
                && ticker.avg_volume_30d > 0
                && ticker.relative_volume() < config.min_relative_volume
            {
                continue;
            }

            let (catalyst_headline, catalyst_url, catalyst) =
                match self.finnhub.get_top_catalyst(&symbol).await {
                    Ok(Some((h, u, c))) => (Some(h), u, c),
                    _ => (None, None, CatalystType::Unknown),
                };

            let score = score_setup(
                ticker.gap_pct(),
                ticker.relative_volume(),
                &catalyst,
                0.0,
                0.0,
                1.0,
                0,
            );

            let is_long = ticker.gap_pct() >= 0.0;
            let levels = trade_levels(
                ticker.premarket_price,
                is_long,
                config.stop_pct,
                config.reward_mult,
            );

            setups.push(Setup {
                ticker,
                catalyst,
                catalyst_headline,
                catalyst_url,
                unusual_options_calls: None,
                unusual_options_puts: None,
                score,
                levels,
                next_earnings: None,
            });
        }

        setups.sort_by(|a, b| {
            b.score.total
                .partial_cmp(&a.score.total)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        setups.truncate(config.top_n);

        // Fetch the next earnings date once per scored setup (≤ top_n), not per
        // watchlist symbol, to bound the extra scan time. Done after truncation
        // so only the surviving top-N incur the extra rate-limited call.
        for setup in setups.iter_mut() {
            setup.next_earnings = self.finnhub.get_next_earnings(&setup.ticker.symbol).await;
        }

        info!("Returning {} scored setups", setups.len());
        Ok(setups)
    }
}
