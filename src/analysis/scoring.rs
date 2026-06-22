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

#[cfg(test)]
mod tests {
    use super::score_gap;
    use super::score_volume;
    use super::score_catalyst;
    use super::score_squeeze;
    use super::score_options;
    use super::score_setup;
    use crate::models::CatalystType;

    const EPS: f64 = 1e-9;

    // ── gap score tiers ──────────────────────────────────────────────────────

    #[test]
    fn gap_score_below_5_pct_is_linear() {
        // (4.0/5.0)*12.0 = 9.6
        assert!((score_gap(4.0) - 9.6).abs() < EPS);
    }

    #[test]
    fn gap_score_at_5_pct_boundary() {
        assert!((score_gap(5.0) - 12.0).abs() < EPS);
    }

    #[test]
    fn gap_score_at_10_pct_boundary() {
        assert!((score_gap(10.0) - 17.0).abs() < EPS);
    }

    #[test]
    fn gap_score_at_15_pct_boundary() {
        assert!((score_gap(15.0) - 20.0).abs() < EPS);
    }

    #[test]
    fn gap_score_at_20_pct_boundary() {
        assert!((score_gap(20.0) - 22.0).abs() < EPS);
    }

    #[test]
    fn gap_score_at_30_pct_max() {
        assert!((score_gap(30.0) - 25.0).abs() < EPS);
    }

    #[test]
    fn gap_score_negative_uses_absolute_value() {
        // -15% gap should score identically to +15%
        assert!((score_gap(-15.0) - 20.0).abs() < EPS);
    }

    // ── volume score tiers ───────────────────────────────────────────────────

    #[test]
    fn volume_score_below_2x_is_linear() {
        // (1.0/2.0)*10.0 = 5.0
        assert!((score_volume(1.0) - 5.0).abs() < EPS);
    }

    #[test]
    fn volume_score_at_2x_boundary() {
        assert!((score_volume(2.0) - 10.0).abs() < EPS);
    }

    #[test]
    fn volume_score_at_3x_boundary() {
        assert!((score_volume(3.0) - 15.0).abs() < EPS);
    }

    #[test]
    fn volume_score_at_5x_boundary() {
        assert!((score_volume(5.0) - 20.0).abs() < EPS);
    }

    #[test]
    fn volume_score_at_10x_max() {
        assert!((score_volume(10.0) - 25.0).abs() < EPS);
    }

    #[test]
    fn volume_score_negative_rvol_clamped_to_zero() {
        assert!((score_volume(-1.0) - 0.0).abs() < EPS);
    }

    // ── catalyst score ───────────────────────────────────────────────────────

    #[test]
    fn catalyst_score_earnings_beat_is_max() {
        assert!((score_catalyst(&CatalystType::EarningsBeat) - 20.0).abs() < EPS);
    }

    #[test]
    fn catalyst_score_fda_approval_is_max() {
        assert!((score_catalyst(&CatalystType::FdaApproval) - 20.0).abs() < EPS);
    }

    #[test]
    fn catalyst_score_acquisition() {
        assert!((score_catalyst(&CatalystType::Acquisition) - 18.0).abs() < EPS);
    }

    #[test]
    fn catalyst_score_merger() {
        assert!((score_catalyst(&CatalystType::Merger) - 16.0).abs() < EPS);
    }

    #[test]
    fn catalyst_score_unknown_is_zero() {
        assert!((score_catalyst(&CatalystType::Unknown) - 0.0).abs() < EPS);
    }

    // ── squeeze score ────────────────────────────────────────────────────────

    #[test]
    fn squeeze_score_max_at_full_values() {
        // short=40% (1.0 * 8 = 8), dtc=5 (1.0 * 7 = 7) → 15.0
        assert!((score_squeeze(40.0, 5.0, 10.0) - 15.0).abs() < EPS);
    }

    #[test]
    fn squeeze_score_half_short_float() {
        // short=20% (0.5 * 8 = 4), dtc=5 (7) → 11.0
        assert!((score_squeeze(20.0, 5.0, 10.0) - 11.0).abs() < EPS);
    }

    #[test]
    fn squeeze_score_negative_gap_returns_zero() {
        assert!((score_squeeze(40.0, 5.0, -10.0) - 0.0).abs() < EPS);
    }

    // ── options score ────────────────────────────────────────────────────────

    #[test]
    fn options_score_max_at_full_values() {
        // ratio=5 (1.0 * 8 = 8), sweeps=5 (1.0 * 7 = 7) → 15.0
        assert!((score_options(5.0, 5, 10.0) - 15.0).abs() < EPS);
    }

    #[test]
    fn options_score_zero_sweeps() {
        // ratio=5 (8), sweeps=0 (0) → 8.0
        assert!((score_options(5.0, 0, 10.0) - 8.0).abs() < EPS);
    }

    #[test]
    fn options_score_negative_gap_returns_zero() {
        assert!((score_options(5.0, 5, -10.0) - 0.0).abs() < EPS);
    }

    // ── total score cap ──────────────────────────────────────────────────────

    #[test]
    fn total_score_at_maximum_inputs_equals_100() {
        // All components at max: 25 + 25 + 20 + 15 + 15 = 100
        let s = score_setup(30.0, 10.0, &CatalystType::EarningsBeat, 40.0, 5.0, 5.0, 5);
        assert!((s.total - 100.0).abs() < EPS);
    }

    #[test]
    fn total_score_cannot_exceed_100() {
        let s = score_setup(50.0, 50.0, &CatalystType::EarningsBeat, 100.0, 20.0, 20.0, 20);
        assert!(s.total <= 100.0);
    }

    #[test]
    fn score_setup_all_unknown_with_zero_rvol() {
        let s = score_setup(0.0, 0.0, &CatalystType::Unknown, 0.0, 0.0, 0.0, 0);
        assert!((s.total - 0.0).abs() < EPS);
    }
}
