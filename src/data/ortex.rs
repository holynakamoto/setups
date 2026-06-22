use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://api.ortex.com/v1";

pub struct OrtexClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct ShortInterestResponse {
    data: Option<ShortData>,
}

#[derive(Debug, Deserialize)]
struct ShortData {
    #[allow(dead_code)]
    short_interest: Option<f64>,
    short_float: Option<f64>,
    borrow_rate: Option<f64>,
    days_to_cover: Option<f64>,
    shares_on_loan: Option<u64>,
}

pub struct ShortData2 {
    pub short_float_pct: f64,
    #[allow(dead_code)]
    pub borrow_rate: f64,
    pub days_to_cover: f64,
    #[allow(dead_code)]
    pub shares_on_loan: u64,
}

impl OrtexClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_short_data(&self, symbol: &str) -> Result<Option<ShortData2>> {
        let url = format!("{}/short-interest/{}", BASE_URL, symbol.to_lowercase());
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await?
            .json::<ShortInterestResponse>()
            .await?;

        Ok(resp.data.map(|d| ShortData2 {
            short_float_pct: d.short_float.unwrap_or(0.0),
            borrow_rate: d.borrow_rate.unwrap_or(0.0),
            days_to_cover: d.days_to_cover.unwrap_or(0.0),
            shares_on_loan: d.shares_on_loan.unwrap_or(0),
        }))
    }

    #[allow(dead_code)]
    pub fn squeeze_score(&self, short_float_pct: f64, days_to_cover: f64, gap_pct: f64) -> f64 {
        let short_score = (short_float_pct / 50.0).min(1.0) * 40.0;
        let dtc_score = (days_to_cover / 10.0).min(1.0) * 30.0;
        let gap_score = (gap_pct.abs() / 20.0).min(1.0) * 30.0;
        short_score + dtc_score + gap_score
    }
}
