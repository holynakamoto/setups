use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub prev_close: f64,
    pub premarket_price: f64,
    pub premarket_volume: u64,
    pub avg_volume_30d: u64,
    pub float_shares: Option<u64>,
    pub short_float_pct: Option<f64>,
    pub market_cap: Option<f64>,
}

impl Ticker {
    pub fn gap_pct(&self) -> f64 {
        if self.prev_close == 0.0 {
            return 0.0;
        }
        ((self.premarket_price - self.prev_close) / self.prev_close) * 100.0
    }

    pub fn relative_volume(&self) -> f64 {
        if self.avg_volume_30d == 0 {
            return 0.0;
        }
        self.premarket_volume as f64 / self.avg_volume_30d as f64
    }

    #[allow(dead_code)]
    pub fn is_low_float(&self) -> bool {
        self.float_shares
            .map(|f| f < 20_000_000)
            .unwrap_or(false)
    }
}
