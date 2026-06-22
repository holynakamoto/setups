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
