use crate::models::{setup::SetupScore, CatalystType};

pub fn score_setup(
    gap_pct: f64,
    relative_volume: f64,
    catalyst: &CatalystType,
    short_float_pct: f64,
    days_to_cover: f64,
    call_put_ratio: f64,
    bullish_sweeps: usize,
) -> SetupScore {
    let gap_score = score_gap(gap_pct);
    let volume_score = score_volume(relative_volume);
    let catalyst_score = score_catalyst(catalyst);
    let squeeze_score = score_squeeze(short_float_pct, days_to_cover, gap_pct);
    let options_score = score_options(call_put_ratio, bullish_sweeps, gap_pct);

    let total = gap_score + volume_score + catalyst_score + squeeze_score + options_score;

    SetupScore {
        gap_score,
        volume_score,
        catalyst_score,
        squeeze_score,
        options_score,
        total: total.min(100.0),
    }
}

fn score_gap(gap_pct: f64) -> f64 {
    let abs_gap = gap_pct.abs();
    if abs_gap >= 30.0 {
        25.0
    } else if abs_gap >= 20.0 {
        22.0
    } else if abs_gap >= 15.0 {
        20.0
    } else if abs_gap >= 10.0 {
        17.0
    } else if abs_gap >= 5.0 {
        12.0
    } else {
        (abs_gap / 5.0) * 12.0
    }
}

fn score_volume(rvol: f64) -> f64 {
    if rvol >= 10.0 {
        25.0
    } else if rvol >= 5.0 {
        20.0
    } else if rvol >= 3.0 {
        15.0
    } else if rvol >= 2.0 {
        10.0
    } else {
        (rvol.max(0.0) / 2.0) * 10.0
    }
}

fn score_catalyst(catalyst: &CatalystType) -> f64 {
    match catalyst {
        CatalystType::EarningsBeat => 20.0,
        CatalystType::FdaApproval => 20.0,
        CatalystType::Acquisition => 18.0,
        CatalystType::Merger => 16.0,
        CatalystType::ContractWin => 14.0,
        CatalystType::AnalystUpgrade => 12.0,
        CatalystType::EarningsMiss => 10.0,
        CatalystType::FdaRejection => 10.0,
        CatalystType::AnalystDowngrade => 10.0,
        CatalystType::GeneralNews => 6.0,
        CatalystType::Unknown => 0.0,
    }
}

fn score_squeeze(short_float_pct: f64, days_to_cover: f64, gap_pct: f64) -> f64 {
    if gap_pct < 0.0 {
        return 0.0;
    }
    let short_component = (short_float_pct / 40.0).min(1.0) * 8.0;
    let dtc_component = (days_to_cover / 5.0).min(1.0) * 7.0;
    short_component + dtc_component
}

fn score_options(call_put_ratio: f64, bullish_sweeps: usize, gap_pct: f64) -> f64 {
    if gap_pct < 0.0 {
        return 0.0;
    }
    let ratio_score = (call_put_ratio / 5.0).min(1.0) * 8.0;
    let sweep_score = (bullish_sweeps.min(5) as f64 / 5.0) * 7.0;
    ratio_score + sweep_score
}
