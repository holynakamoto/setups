use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use crate::models::Ticker;

const BASE_URL: &str = "https://api.polygon.io";

pub struct PolygonClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct SnapshotResponse {
    ticker: Option<SnapshotTicker>,
    #[allow(dead_code)]
    status: String,
}

#[derive(Debug, Deserialize)]
struct SnapshotTicker {
    day: Option<DayBar>,
    #[serde(rename = "prevDay")]
    prev_day: Option<DayBar>,
    #[serde(rename = "lastTrade")]
    last_trade: Option<LastTrade>,
}

#[derive(Debug, Deserialize)]
struct DayBar {
    v: Option<f64>,
    c: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct LastTrade {
    p: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TickerDetailsResponse {
    results: Option<TickerDetails>,
}

#[derive(Debug, Deserialize)]
struct TickerDetails {
    #[serde(rename = "share_class_shares_outstanding")]
    shares_outstanding: Option<u64>,
    market_cap: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct AggregatesResponse {
    results: Option<Vec<AggBar>>,
}

#[derive(Debug, Deserialize)]
struct AggBar {
    v: f64,
}

impl PolygonClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_snapshot(&self, symbol: &str) -> Result<Option<(f64, f64, u64)>> {
        let url = format!(
            "{}/v2/snapshot/locale/us/markets/stocks/tickers/{}?apiKey={}",
            BASE_URL, symbol, self.api_key
        );
        let resp: SnapshotResponse = self.client.get(&url).send().await?.json().await?;
        let ticker = match resp.ticker {
            Some(t) => t,
            None => return Ok(None),
        };
        let prev_close = ticker.prev_day.as_ref().and_then(|d| d.c).unwrap_or(0.0);
        let premarket_price = ticker
            .last_trade
            .as_ref()
            .and_then(|t| t.p)
            .unwrap_or(prev_close);
        let premarket_volume = ticker
            .day
            .as_ref()
            .and_then(|d| d.v)
            .unwrap_or(0.0) as u64;
        Ok(Some((prev_close, premarket_price, premarket_volume)))
    }

    pub async fn get_avg_volume(&self, symbol: &str) -> Result<u64> {
        let url = format!(
            "{}/v2/aggs/ticker/{}/range/1/day/2025-01-01/2026-01-01?adjusted=true&sort=desc&limit=30&apiKey={}",
            BASE_URL, symbol, self.api_key
        );
        let resp: AggregatesResponse = self.client.get(&url).send().await?.json().await?;
        let bars = resp.results.unwrap_or_default();
        if bars.is_empty() {
            return Ok(0);
        }
        let avg = bars.iter().map(|b| b.v).sum::<f64>() / bars.len() as f64;
        Ok(avg as u64)
    }

    pub async fn get_ticker_details(&self, symbol: &str) -> Result<(Option<u64>, Option<f64>)> {
        let url = format!(
            "{}/v3/reference/tickers/{}?apiKey={}",
            BASE_URL, symbol, self.api_key
        );
        let resp: TickerDetailsResponse = self.client.get(&url).send().await?.json().await?;
        match resp.results {
            Some(d) => Ok((d.shares_outstanding, d.market_cap)),
            None => Ok((None, None)),
        }
    }

    pub async fn build_ticker(&self, symbol: &str) -> Result<Ticker> {
        let snapshot = self
            .get_snapshot(symbol)
            .await?
            .ok_or_else(|| anyhow!("No snapshot data for {}", symbol))?;
        let (prev_close, premarket_price, premarket_volume) = snapshot;
        let avg_volume = self.get_avg_volume(symbol).await?;
        let (float_shares, market_cap) = self.get_ticker_details(symbol).await?;
        Ok(Ticker {
            symbol: symbol.to_uppercase(),
            prev_close,
            premarket_price,
            premarket_volume,
            avg_volume_30d: avg_volume,
            float_shares,
            short_float_pct: None,
            market_cap,
        })
    }

    pub async fn get_gappers(&self, min_gap_pct: f64, limit: usize) -> Result<Vec<String>> {
        let url = format!(
            "{}/v2/snapshot/locale/us/markets/stocks/gainers?apiKey={}",
            BASE_URL, self.api_key
        );

        #[derive(Deserialize)]
        struct GainersResp {
            tickers: Option<Vec<GainerEntry>>,
        }
        #[derive(Deserialize)]
        struct GainerEntry {
            ticker: String,
            #[serde(rename = "todaysChangePerc")]
            change_pct: f64,
        }

        let resp: GainersResp = self.client.get(&url).send().await?.json().await?;
        let symbols = resp
            .tickers
            .unwrap_or_default()
            .into_iter()
            .filter(|t| t.change_pct.abs() >= min_gap_pct)
            .take(limit)
            .map(|t| t.ticker)
            .collect();
        Ok(symbols)
    }
}
