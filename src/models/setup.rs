use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use crate::models::Ticker;

/// Default near-term window (in calendar days) within which an upcoming
/// earnings date is treated as event risk worth flagging.
pub const EARNINGS_IMMINENT_WINDOW_DAYS: i64 = 5;

/// Pure predicate: is `earnings` on or after `today` and within `window_days`?
/// Past-dated earnings never count. Kept free-standing so it is unit-testable
/// without reaching for the wall clock.
pub fn earnings_is_imminent(earnings: NaiveDate, today: NaiveDate, window_days: i64) -> bool {
    if earnings < today {
        return false;
    }
    (earnings - today).num_days() <= window_days
}

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

/// A direction-aware trade plan derived from the entry reference price and the
/// configurable stop / reward knobs. All levels are absolute prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeLevels {
    pub entry: f64,
    pub stop: f64,
    pub target: f64,
    pub risk_reward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup {
    pub ticker: Ticker,
    pub catalyst: CatalystType,
    pub catalyst_headline: Option<String>,
    pub unusual_options_calls: Option<f64>,
    pub unusual_options_puts: Option<f64>,
    pub score: SetupScore,
    pub levels: TradeLevels,
    pub next_earnings: Option<NaiveDate>,
}

impl Setup {
    pub fn direction(&self) -> &'static str {
        if self.ticker.gap_pct() >= 0.0 { "LONG" } else { "SHORT" }
    }

    /// True when the next earnings date is known and falls within `window_days`
    /// of today (and is not in the past).
    pub fn earnings_imminent(&self, window_days: i64) -> bool {
        self.next_earnings
            .map(|d| earnings_is_imminent(d, chrono::Utc::now().date_naive(), window_days))
            .unwrap_or(false)
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
            levels: TradeLevels { entry: premarket, stop: premarket, target: premarket, risk_reward: 0.0 },
            next_earnings: None,
        }
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
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

    // ── earnings imminence ───────────────────────────────────────────────────

    #[test]
    fn earnings_within_window_is_imminent() {
        let today = date(2026, 6, 22);
        // 3 days out, window 5 → imminent
        assert!(earnings_is_imminent(date(2026, 6, 25), today, 5));
    }

    #[test]
    fn earnings_on_window_boundary_is_imminent() {
        let today = date(2026, 6, 22);
        // exactly 5 days out, window 5 → imminent
        assert!(earnings_is_imminent(date(2026, 6, 27), today, 5));
    }

    #[test]
    fn earnings_today_is_imminent() {
        let today = date(2026, 6, 22);
        assert!(earnings_is_imminent(today, today, 5));
    }

    #[test]
    fn earnings_beyond_window_is_not_imminent() {
        let today = date(2026, 6, 22);
        // 6 days out, window 5 → not imminent
        assert!(!earnings_is_imminent(date(2026, 6, 28), today, 5));
    }

    #[test]
    fn past_earnings_is_not_imminent() {
        let today = date(2026, 6, 22);
        assert!(!earnings_is_imminent(date(2026, 6, 20), today, 5));
    }

    #[test]
    fn earnings_imminent_none_is_false() {
        // a setup with no known earnings date is never imminent
        assert!(!make_setup(110.0, 100.0).earnings_imminent(5));
    }
}
