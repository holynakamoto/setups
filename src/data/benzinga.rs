use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use crate::models::CatalystType;

const BASE_URL: &str = "https://api.benzinga.com/api/v2";

pub struct BenzingaClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct NewsResponse {
    #[serde(default)]
    data: Vec<NewsItem>,
}

#[derive(Debug, Deserialize)]
struct NewsItem {
    headline: String,
    #[allow(dead_code)]
    #[serde(default)]
    teaser: String,
    #[allow(dead_code)]
    channels: Option<Vec<Channel>>,
}

#[derive(Debug, Deserialize)]
struct Channel {
    #[allow(dead_code)]
    name: String,
}

impl BenzingaClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn get_news(&self, symbol: &str) -> Result<Vec<(String, CatalystType)>> {
        let url = format!(
            "{}/news?tickers={}&token={}&pageSize=5",
            BASE_URL, symbol, self.api_key
        );
        let resp: NewsResponse = self.client.get(&url).send().await?.json().await?;
        let items = resp
            .data
            .into_iter()
            .map(|item| {
                let catalyst = classify_headline(&item.headline);
                (item.headline, catalyst)
            })
            .collect();
        Ok(items)
    }

    pub async fn get_top_catalyst(&self, symbol: &str) -> Result<Option<(String, CatalystType)>> {
        let mut items = self.get_news(symbol).await?;
        if items.is_empty() {
            return Ok(None);
        }
        let (headline, catalyst) = items.remove(0);
        Ok(Some((headline, catalyst)))
    }
}

fn classify_headline(headline: &str) -> CatalystType {
    let h = headline.to_lowercase();
    if h.contains("earnings") && (h.contains("beat") || h.contains("top") || h.contains("exceed")) {
        CatalystType::EarningsBeat
    } else if h.contains("earnings") && (h.contains("miss") || h.contains("below")) {
        CatalystType::EarningsMiss
    } else if h.contains("fda") && (h.contains("approv") || h.contains("grant") || h.contains("clear")) {
        CatalystType::FdaApproval
    } else if h.contains("fda") && (h.contains("reject") || h.contains("deny") || h.contains("refuse")) {
        CatalystType::FdaRejection
    } else if h.contains("merger") || h.contains("merges") {
        CatalystType::Merger
    } else if h.contains("acqui") || h.contains("buyout") || h.contains("takeover") {
        CatalystType::Acquisition
    } else if h.contains("upgrade") || h.contains("outperform") || h.contains("buy rating") {
        CatalystType::AnalystUpgrade
    } else if h.contains("downgrade") || h.contains("underperform") || h.contains("sell rating") {
        CatalystType::AnalystDowngrade
    } else if h.contains("contract") || h.contains("award") || h.contains("deal") {
        CatalystType::ContractWin
    } else if h.len() > 10 {
        CatalystType::GeneralNews
    } else {
        CatalystType::Unknown
    }
}
