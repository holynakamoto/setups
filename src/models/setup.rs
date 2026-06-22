use serde::{Deserialize, Serialize};
use crate::models::Ticker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CatalystType {
    EarningsBeat,
    EarningsMiss,
    FdaApproval,
    FdaRejection,
    Merger,
    Acquisition,
    AnalystUpgrade,
    AnalystDowngrade,
    ContractWin,
    GeneralNews,
    Unknown,
}

impl std::fmt::Display for CatalystType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            CatalystType::EarningsBeat => "Earnings Beat",
            CatalystType::EarningsMiss => "Earnings Miss",
            CatalystType::FdaApproval => "FDA Approval",
            CatalystType::FdaRejection => "FDA Rejection",
            CatalystType::Merger => "Merger",
            CatalystType::Acquisition => "Acquisition",
            CatalystType::AnalystUpgrade => "Analyst Upgrade",
            CatalystType::AnalystDowngrade => "Analyst Downgrade",
            CatalystType::ContractWin => "Contract Win",
            CatalystType::GeneralNews => "General News",
            CatalystType::Unknown => "No Catalyst",
        };
        write!(f, "{}", label)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupScore {
    pub gap_score: f64,
    pub volume_score: f64,
    pub catalyst_score: f64,
    pub squeeze_score: f64,
    pub options_score: f64,
    pub total: f64,
}

impl SetupScore {
    pub fn grade(&self) -> &'static str {
        match self.total as u32 {
            90..=100 => "A+",
            80..=89  => "A",
            70..=79  => "B",
            60..=69  => "C",
            _        => "D",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup {
    pub ticker: Ticker,
    pub catalyst: CatalystType,
    pub catalyst_headline: Option<String>,
    pub unusual_options_calls: Option<f64>,
    pub unusual_options_puts: Option<f64>,
    pub score: SetupScore,
}

impl Setup {
    pub fn direction(&self) -> &'static str {
        if self.ticker.gap_pct() >= 0.0 { "LONG" } else { "SHORT" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Ticker;

    fn make_score(total: f64) -> SetupScore {
        SetupScore { gap_score: 0.0, volume_score: 0.0, catalyst_score: 0.0, squeeze_score: 0.0, options_score: 0.0, total }
    }

    fn make_setup(premarket: f64, prev_close: f64) -> Setup {
        Setup {
            ticker: Ticker {
                symbol: "TEST".to_string(),
                prev_close,
                premarket_price: premarket,
                premarket_volume: 0,
                avg_volume_30d: 0,
                float_shares: None,
                short_float_pct: None,
                market_cap: None,
            },
            catalyst: CatalystType::Unknown,
            catalyst_headline: None,
            unusual_options_calls: None,
            unusual_options_puts: None,
            score: make_score(0.0),
        }
    }

    // ── grade ────────────────────────────────────────────────────────────────

    #[test]
    fn grade_a_plus_at_90() {
        assert_eq!(make_score(90.0).grade(), "A+");
    }

    #[test]
    fn grade_a_plus_at_100() {
        assert_eq!(make_score(100.0).grade(), "A+");
    }

    #[test]
    fn grade_a_at_80() {
        assert_eq!(make_score(80.0).grade(), "A");
    }

    #[test]
    fn grade_a_at_89() {
        assert_eq!(make_score(89.0).grade(), "A");
    }

    #[test]
    fn grade_b_at_70() {
        assert_eq!(make_score(70.0).grade(), "B");
    }

    #[test]
    fn grade_c_at_60() {
        assert_eq!(make_score(60.0).grade(), "C");
    }

    #[test]
    fn grade_d_below_60() {
        assert_eq!(make_score(59.0).grade(), "D");
    }

    #[test]
    fn grade_d_at_zero() {
        assert_eq!(make_score(0.0).grade(), "D");
    }

    // ── direction ────────────────────────────────────────────────────────────

    #[test]
    fn direction_long_on_positive_gap() {
        assert_eq!(make_setup(110.0, 100.0).direction(), "LONG");
    }

    #[test]
    fn direction_short_on_negative_gap() {
        assert_eq!(make_setup(90.0, 100.0).direction(), "SHORT");
    }

    #[test]
    fn direction_long_on_flat_price() {
        // 0.0 gap is >= 0.0 → LONG
        assert_eq!(make_setup(100.0, 100.0).direction(), "LONG");
    }

    // ── catalyst display ─────────────────────────────────────────────────────

    #[test]
    fn catalyst_display_earnings_beat() {
        assert_eq!(format!("{}", CatalystType::EarningsBeat), "Earnings Beat");
    }

    #[test]
    fn catalyst_display_unknown_label() {
        assert_eq!(format!("{}", CatalystType::Unknown), "No Catalyst");
    }
}
