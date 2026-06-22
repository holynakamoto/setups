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

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn ticker(premarket: f64, prev_close: f64, vol: u64, avg_vol: u64, float: Option<u64>) -> Ticker {
        Ticker {
            symbol: "TEST".to_string(),
            prev_close,
            premarket_price: premarket,
            premarket_volume: vol,
            avg_volume_30d: avg_vol,
            float_shares: float,
            short_float_pct: None,
            market_cap: None,
        }
    }

    // ── gap_pct ──────────────────────────────────────────────────────────────

    #[test]
    fn gap_pct_long_10_pct() {
        assert!((ticker(110.0, 100.0, 0, 0, None).gap_pct() - 10.0).abs() < EPS);
    }

    #[test]
    fn gap_pct_short_10_pct() {
        assert!((ticker(90.0, 100.0, 0, 0, None).gap_pct() - (-10.0)).abs() < EPS);
    }

    #[test]
    fn gap_pct_flat() {
        assert!((ticker(100.0, 100.0, 0, 0, None).gap_pct() - 0.0).abs() < EPS);
    }

    #[test]
    fn gap_pct_zero_prev_close_guard() {
        assert_eq!(ticker(110.0, 0.0, 0, 0, None).gap_pct(), 0.0);
    }

    // ── relative_volume ──────────────────────────────────────────────────────

    #[test]
    fn relative_volume_one_and_a_half() {
        assert!((ticker(100.0, 100.0, 1_500, 1_000, None).relative_volume() - 1.5).abs() < EPS);
    }

    #[test]
    fn relative_volume_exactly_double() {
        assert!((ticker(100.0, 100.0, 2_000, 1_000, None).relative_volume() - 2.0).abs() < EPS);
    }

    #[test]
    fn relative_volume_zero_avg_guard() {
        assert_eq!(ticker(100.0, 100.0, 1_000, 0, None).relative_volume(), 0.0);
    }

    // ── is_low_float ─────────────────────────────────────────────────────────

    #[test]
    fn is_low_float_below_20m_is_true() {
        assert!(ticker(100.0, 100.0, 0, 0, Some(5_000_000)).is_low_float());
    }

    #[test]
    fn is_low_float_exactly_20m_is_false() {
        assert!(!ticker(100.0, 100.0, 0, 0, Some(20_000_000)).is_low_float());
    }

    #[test]
    fn is_low_float_above_20m_is_false() {
        assert!(!ticker(100.0, 100.0, 0, 0, Some(100_000_000)).is_low_float());
    }

    #[test]
    fn is_low_float_none_is_false() {
        assert!(!ticker(100.0, 100.0, 0, 0, None).is_low_float());
    }
}
