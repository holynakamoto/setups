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
