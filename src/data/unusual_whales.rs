use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://api.unusualwhales.com/api";

pub struct UnusualWhalesClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct FlowResponse {
    data: Option<Vec<FlowEntry>>,
}

#[derive(Debug, Deserialize)]
struct FlowEntry {
    #[allow(dead_code)]
    option_symbol: String,
    side: String,
    premium: f64,
    volume: u64,
    #[allow(dead_code)]
    open_interest: u64,
    #[allow(dead_code)]
    strike: f64,
    #[allow(dead_code)]
    expiry: String,
    option_type: String,
}

pub struct OptionsFlowSummary {
    pub call_premium: f64,
    pub put_premium: f64,
    pub call_volume: u64,
    pub put_volume: u64,
    pub bullish_sweep_count: usize,
    pub bearish_sweep_count: usize,
}

impl UnusualWhalesClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_flow_summary(&self, symbol: &str) -> Result<OptionsFlowSummary> {
        let url = format!(
            "{}/stock/{}/flow?limit=50",
            BASE_URL,
            symbol.to_lowercase()
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .json::<FlowResponse>()
            .await?;

        let entries = resp.data.unwrap_or_default();
        let mut summary = OptionsFlowSummary {
            call_premium: 0.0,
            put_premium: 0.0,
            call_volume: 0,
            put_volume: 0,
            bullish_sweep_count: 0,
            bearish_sweep_count: 0,
        };

        for entry in &entries {
            if entry.option_type.to_lowercase() == "call" {
                summary.call_premium += entry.premium;
                summary.call_volume += entry.volume;
                if entry.side.to_lowercase() == "ask" {
                    summary.bullish_sweep_count += 1;
                }
            } else {
                summary.put_premium += entry.premium;
                summary.put_volume += entry.volume;
                if entry.side.to_lowercase() == "bid" {
                    summary.bearish_sweep_count += 1;
                }
            }
        }
        Ok(summary)
    }

    #[allow(dead_code)]
    pub async fn get_unusual_activity(&self, symbol: &str) -> Result<Vec<String>> {
        let flow = self.get_flow_summary(symbol).await?;
        let mut alerts = Vec::new();
        let ratio = if flow.put_premium > 0.0 {
            flow.call_premium / flow.put_premium
        } else {
            f64::MAX
        };
        if ratio > 3.0 {
            alerts.push(format!(
                "Heavy call flow: ${:.0}K calls vs ${:.0}K puts",
                flow.call_premium / 1000.0,
                flow.put_premium / 1000.0
            ));
        }
        if flow.bullish_sweep_count >= 3 {
            alerts.push(format!(
                "{} bullish sweeps detected",
                flow.bullish_sweep_count
            ));
        }
        Ok(alerts)
    }
}
