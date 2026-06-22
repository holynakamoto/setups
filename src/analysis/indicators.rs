#![allow(dead_code)]

pub fn vwap(prices: &[f64], volumes: &[f64]) -> f64 {
    if prices.is_empty() || prices.len() != volumes.len() {
        return 0.0;
    }
    let numerator: f64 = prices.iter().zip(volumes.iter()).map(|(p, v)| p * v).sum();
    let denominator: f64 = volumes.iter().sum();
    if denominator == 0.0 { 0.0 } else { numerator / denominator }
}

pub fn relative_volume(current_vol: u64, avg_vol: u64) -> f64 {
    if avg_vol == 0 { return 0.0; }
    current_vol as f64 / avg_vol as f64
}

pub fn gap_pct(current: f64, prev_close: f64) -> f64 {
    if prev_close == 0.0 { return 0.0; }
    ((current - prev_close) / prev_close) * 100.0
}

pub fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<f64> {
    if highs.len() < 2 {
        return vec![];
    }
    let n = highs.len();
    let mut trs = Vec::with_capacity(n - 1);
    for i in 1..n {
        let tr = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
        trs.push(tr);
    }
    if trs.len() < period {
        return vec![];
    }
    let mut atrs = Vec::new();
    let first: f64 = trs[..period].iter().sum::<f64>() / period as f64;
    atrs.push(first);
    for i in period..trs.len() {
        let prev = *atrs.last().unwrap();
        atrs.push((prev * (period - 1) as f64 + trs[i]) / period as f64);
    }
    atrs
}

pub fn ema(values: &[f64], period: usize) -> Vec<f64> {
    if values.len() < period {
        return vec![];
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::new();
    let seed: f64 = values[..period].iter().sum::<f64>() / period as f64;
    result.push(seed);
    for &v in &values[period..] {
        let prev = *result.last().unwrap();
        result.push(v * k + prev * (1.0 - k));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    // ── vwap ────────────────────────────────────────────────────────────────

    #[test]
    fn vwap_equal_weights() {
        // (10*2 + 20*2) / 4 = 15.0
        assert!((vwap(&[10.0, 20.0], &[2.0, 2.0]) - 15.0).abs() < EPS);
    }

    #[test]
    fn vwap_unequal_weights() {
        // (10*3 + 30*1) / 4 = 60/4 = 15.0
        assert!((vwap(&[10.0, 30.0], &[3.0, 1.0]) - 15.0).abs() < EPS);
    }

    #[test]
    fn vwap_heavy_weight_on_first_bar() {
        // (10*3 + 20*1) / 4 = 50/4 = 12.5
        assert!((vwap(&[10.0, 20.0], &[3.0, 1.0]) - 12.5).abs() < EPS);
    }

    #[test]
    fn vwap_empty_slices() {
        assert_eq!(vwap(&[], &[]), 0.0);
    }

    #[test]
    fn vwap_mismatched_lengths() {
        assert_eq!(vwap(&[1.0, 2.0], &[1.0]), 0.0);
    }

    #[test]
    fn vwap_zero_volume_denominator() {
        assert_eq!(vwap(&[10.0], &[0.0]), 0.0);
    }

    // ── gap_pct ─────────────────────────────────────────────────────────────

    #[test]
    fn gap_pct_positive() {
        // (110 - 100) / 100 * 100 = 10.0 %
        assert!((gap_pct(110.0, 100.0) - 10.0).abs() < EPS);
    }

    #[test]
    fn gap_pct_negative() {
        // (90 - 100) / 100 * 100 = -10.0 %
        assert!((gap_pct(90.0, 100.0) - (-10.0)).abs() < EPS);
    }

    #[test]
    fn gap_pct_zero_prev_close_guard() {
        assert_eq!(gap_pct(110.0, 0.0), 0.0);
    }

    // ── relative_volume ──────────────────────────────────────────────────────

    #[test]
    fn relative_volume_one_and_a_half() {
        assert!((relative_volume(150, 100) - 1.5).abs() < EPS);
    }

    #[test]
    fn relative_volume_zero_avg_guard() {
        assert_eq!(relative_volume(1_000, 0), 0.0);
    }

    // ── atr ─────────────────────────────────────────────────────────────────

    #[test]
    fn atr_single_period_result() {
        // Two TRs both = 7.0 (H-C[i-1] dominates); period=2 → ATR=[7.0]
        let h = [100.0, 105.0, 110.0];
        let l = [95.0, 100.0, 105.0];
        let c = [98.0, 103.0, 108.0];
        let result = atr(&h, &l, &c, 2);
        assert_eq!(result.len(), 1);
        assert!((result[0] - 7.0).abs() < EPS);
    }

    #[test]
    fn atr_single_bar_returns_empty() {
        assert!(atr(&[100.0], &[95.0], &[98.0], 2).is_empty());
    }

    #[test]
    fn atr_period_larger_than_tr_count_returns_empty() {
        // 2 bars → 1 TR; period=5 > 1 → empty
        let result = atr(&[100.0, 105.0], &[95.0, 100.0], &[98.0, 103.0], 5);
        assert!(result.is_empty());
    }

    // ── ema ──────────────────────────────────────────────────────────────────

    #[test]
    fn ema_basic_three_outputs() {
        // values=[2,4,6,8], period=2, k=2/3
        // seed = (2+4)/2 = 3.0
        // v=6: 6*(2/3) + 3*(1/3) = 4+1 = 5.0
        // v=8: 8*(2/3) + 5*(1/3) = 16/3+5/3 = 7.0
        let result = ema(&[2.0, 4.0, 6.0, 8.0], 2);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 3.0).abs() < EPS);
        assert!((result[1] - 5.0).abs() < EPS);
        assert!((result[2] - 7.0).abs() < EPS);
    }

    #[test]
    fn ema_insufficient_values_returns_empty() {
        assert!(ema(&[1.0], 2).is_empty());
    }

    #[test]
    fn ema_exactly_period_length_returns_seed_only() {
        // values=[4.0, 6.0], period=2 → seed=(4+6)/2=5.0, no more values
        let result = ema(&[4.0, 6.0], 2);
        assert_eq!(result.len(), 1);
        assert!((result[0] - 5.0).abs() < EPS);
    }
}
