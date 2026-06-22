use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use crate::models::{CatalystType, Ticker};

const BASE_URL: &str = "https://finnhub.io/api/v1";

// ~70 high-alpha symbols most likely to produce pre-market gaps.
// Kept small so the full scan finishes in ~80s at Finnhub's free-tier rate
// limit of 60 calls/min (1 call per 1.1s sequential).
const WATCHLIST: &[&str] = &[
    // Semiconductors
    "NVDA","AMD","INTC","QCOM","MU","AMAT","LRCX","KLAC","MRVL","SMCI","ARM","AVGO","TSM","ASML",
    // Mega-cap tech
    "AAPL","MSFT","GOOGL","AMZN","META","TSLA",
    // High-beta software / cloud
    "PLTR","CRWD","PANW","NET","SNOW","CRM","ADBE","NOW","DDOG","ZS",
    // Crypto / fintech
    "COIN","HOOD","SOFI","AFRM","UPST","MSTR",
    // Consumer / platforms
    "RBLX","UBER","LYFT","PTON",
    // Biotech / pharma (high FDA catalyst frequency)
    "MRNA","BNTX","NVAX","LLY","BIIB","GILD","AMGN","REGN","VRTX","SAVA","AXSM",
    // EV
    "RIVN","LCID","NIO","XPEV","LI",
    // Momentum / meme adjacent
    "GME","SPCE",
    // China ADRs (high overnight gap frequency)
    "BABA","JD","PDD","BIDU",
    // Leveraged ETFs (sector sentiment)
    "SOXL","TQQQ","ARKK",
];

pub struct CachedQuote {
    pub current: f64,
    pub prev_close: f64,
    pub volume: u64,
}

#[derive(Debug, Deserialize)]
struct QuoteResp {
    #[serde(rename = "c")]
    current: f64,
    #[serde(rename = "pc")]
    prev_close: f64,
    #[serde(rename = "v")]
    volume: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct MetricResponse {
    metric: Option<Metric>,
}

#[derive(Debug, Deserialize)]
struct Metric {
    #[serde(rename = "10DayAverageTradingVolume")]
    avg_volume_10d: Option<f64>,
    #[serde(rename = "marketCapitalization")]
    market_cap: Option<f64>,
    #[serde(rename = "shareOutstanding")]
    shares_outstanding: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct NewsItem {
    headline: String,
}

pub struct FinnhubClient {
    client: Client,
    api_key: String,
}

impl FinnhubClient {
    pub fn new(api_key: String) -> Self {
        Self { client: Client::new(), api_key }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let sep = if path.contains('?') { '&' } else { '?' };
        let url = format!("{}{}{}token={}", BASE_URL, path, sep, self.api_key);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Finnhub {} {}", resp.status(), path));
        }
        Ok(resp.json::<T>().await?)
    }

    /// Scan the watchlist for gappers.
    /// Returns cached quote data so callers can avoid re-fetching quotes.
    /// Sequential with 1.1s between calls to stay under the 60/min free-tier limit.
    pub async fn get_gappers(
        &self,
        min_gap_pct: f64,
        limit: usize,
    ) -> Result<Vec<(String, CachedQuote)>> {
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<&str> = WATCHLIST.iter().copied().filter(|s| seen.insert(*s)).collect();

        let mut gappers = Vec::new();

        for (i, &sym) in unique.iter().enumerate() {
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(1100)).await;
            }

            let q: QuoteResp = match self.get(&format!("/quote?symbol={}", sym)).await {
                Ok(q) => q,
                Err(e) => {
                    tracing::warn!("quote {}: {}", sym, e);
                    continue;
                }
            };

            if q.prev_close == 0.0 || q.current == 0.0 {
                continue;
            }

            let gap = ((q.current - q.prev_close) / q.prev_close) * 100.0;
            if gap.abs() >= min_gap_pct {
                gappers.push((sym.to_string(), CachedQuote {
                    current: q.current,
                    prev_close: q.prev_close,
                    volume: q.volume.unwrap_or(0.0) as u64,
                }));
                if gappers.len() >= limit * 2 {
                    break;
                }
            }
        }

        Ok(gappers)
    }

    /// Build a full Ticker using a pre-fetched quote (no extra quote API call).
    pub async fn build_ticker(&self, symbol: &str, cached: &CachedQuote) -> Result<Ticker> {
        tokio::time::sleep(Duration::from_millis(1100)).await;
        let metrics: Option<MetricResponse> = self
            .get(&format!("/stock/metric?symbol={}&metric=all", symbol))
            .await
            .ok();

        let avg_volume = metrics.as_ref()
            .and_then(|m| m.metric.as_ref())
            .and_then(|m| m.avg_volume_10d)
            .map(|v| (v * 1_000_000.0) as u64)
            .unwrap_or(0);

        let float_shares = metrics.as_ref()
            .and_then(|m| m.metric.as_ref())
            .and_then(|m| m.shares_outstanding)
            .map(|s| (s * 1_000_000.0) as u64);

        let market_cap = metrics.as_ref()
            .and_then(|m| m.metric.as_ref())
            .and_then(|m| m.market_cap)
            .map(|mc| mc * 1_000_000.0);

        Ok(Ticker {
            symbol: symbol.to_uppercase(),
            prev_close: cached.prev_close,
            premarket_price: cached.current,
            premarket_volume: cached.volume,
            avg_volume_30d: avg_volume,
            float_shares,
            short_float_pct: None,
            market_cap,
        })
    }

    /// Standalone ticker build for the `symbol` subcommand (no cached quote).
    pub async fn build_ticker_fresh(&self, symbol: &str) -> Result<Ticker> {
        let q: QuoteResp = self.get(&format!("/quote?symbol={}", symbol)).await?;
        if q.prev_close == 0.0 && q.current == 0.0 {
            return Err(anyhow!("No quote data for {}", symbol));
        }
        let cached = CachedQuote {
            current: q.current,
            prev_close: q.prev_close,
            volume: q.volume.unwrap_or(0.0) as u64,
        };
        self.build_ticker(symbol, &cached).await
    }

    pub async fn get_top_catalyst(&self, symbol: &str) -> Result<Option<(String, CatalystType)>> {
        use chrono::Utc;
        tokio::time::sleep(Duration::from_millis(1100)).await;
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let week_ago = (Utc::now() - chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();
        let items: Vec<NewsItem> = self
            .get(&format!(
                "/company-news?symbol={}&from={}&to={}",
                symbol, week_ago, today
            ))
            .await
            .unwrap_or_default();

        Ok(items.into_iter().next().map(|item| {
            let catalyst = classify_headline(&item.headline);
            (item.headline, catalyst)
        }))
    }
}

fn classify_headline(headline: &str) -> CatalystType {
    let h = headline.to_lowercase();
    if h.contains("earnings")
        && (h.contains("beat") || h.contains("top") || h.contains("exceed") || h.contains("surpass"))
    {
        CatalystType::EarningsBeat
    } else if h.contains("earnings")
        && (h.contains("miss") || h.contains("below") || h.contains("disappoint"))
    {
        CatalystType::EarningsMiss
    } else if h.contains("fda")
        && (h.contains("approv") || h.contains("grant") || h.contains("clear") || h.contains("accept"))
    {
        CatalystType::FdaApproval
    } else if h.contains("fda")
        && (h.contains("reject") || h.contains("deny") || h.contains("refuse") || h.contains("complete response"))
    {
        CatalystType::FdaRejection
    } else if h.contains("merger") || h.contains("merges with") {
        CatalystType::Merger
    } else if h.contains("acqui") || h.contains("buyout") || h.contains("takeover") || h.contains("to buy") {
        CatalystType::Acquisition
    } else if h.contains("upgrade") || h.contains("outperform") || h.contains("overweight") || h.contains("buy rating") {
        CatalystType::AnalystUpgrade
    } else if h.contains("downgrade") || h.contains("underperform") || h.contains("underweight") || h.contains("sell rating") {
        CatalystType::AnalystDowngrade
    } else if h.contains("contract") || h.contains("award") || h.contains("wins deal") {
        CatalystType::ContractWin
    } else if headline.len() > 10 {
        CatalystType::GeneralNews
    } else {
        CatalystType::Unknown
    }
}
