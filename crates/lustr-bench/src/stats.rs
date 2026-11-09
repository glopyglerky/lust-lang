use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub struct Summary {
    pub method: &'static str,
    pub pairs: usize,
    pub raw_ratios: Vec<f64>,
    pub log_ratios: Vec<f64>,
    pub geometric_ratio: f64,
    pub bootstrap_95: [f64; 2],
    pub baseline_median: f64,
    pub candidate_median: f64,
    pub baseline_p95: f64,
    pub candidate_p95: f64,
    pub exact_sign_flip_p: Option<f64>,
    pub bootstrap_resamples: usize,
    pub seed: u64,
}
// SplitMix64; wrapping arithmetic is part of the specified generator.
pub fn random(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e3779b97f4a7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}
fn index(state: &mut u64, n: usize) -> usize {
    let n = n as u64;
    let threshold = n.wrapping_neg() % n;
    loop {
        let r = random(state);
        if r >= threshold {
            return (r % n) as usize;
        }
    }
}
fn mean(xs: &[f64]) -> f64 {
    xs.iter().map(|x| x / xs.len() as f64).sum()
}
fn median(xs: &[f64]) -> f64 {
    let n = xs.len();
    if n % 2 == 1 { xs[n / 2] } else { xs[n / 2 - 1] / 2.0 + xs[n / 2] / 2.0 }
}
pub fn summarize(pairs: &[(f64, f64)], resamples: usize, seed: u64) -> Result<Summary, String> {
    if pairs.is_empty()
        || resamples < 100
        || resamples > 1_000_000
        || pairs.len() > 10000
        || pairs.iter().any(|(a, b)| !a.is_finite() || !b.is_finite() || *a <= 0.0 || *b <= 0.0)
    {
        return Err("positive finite samples and 100..=1000000 resamples required".into());
    }
    let logs: Vec<_> = pairs.iter().map(|(a, b)| a.ln() - b.ln()).collect();
    let ratios: Vec<_> = logs.iter().map(|x| x.exp()).collect();
    let estimate = mean(&logs).exp();
    if !estimate.is_finite() || estimate == 0.0 || ratios.iter().any(|x| !x.is_finite() || *x == 0.0) {
        return Err("ratio outside finite representable range".into());
    }
    let mut rng = seed;
    let mut boots = Vec::with_capacity(resamples);
    for _ in 0..resamples {
        let mut sample = Vec::with_capacity(logs.len());
        for _ in &logs {
            sample.push(logs[index(&mut rng, logs.len())]);
        }
        let v = mean(&sample).exp();
        if !v.is_finite() || v == 0.0 {
            return Err("bootstrap ratio overflow".into());
        }
        boots.push(v);
    }
    boots.sort_by(f64::total_cmp);
    let observed = mean(&logs).abs();
    // Conservative floating tie tolerance, applied symmetrically to all assignments.
    let tolerance = 64.0 * f64::EPSILON * logs.iter().map(|x| x.abs()).sum::<f64>().max(1.0);
    let p = if logs.len() <= 16 {
        let total = 1u64 << logs.len();
        let extreme = (0..total)
            .filter(|mask| {
                let xs: Vec<_> = logs.iter().enumerate().map(|(i, x)| if mask & (1 << i) == 0 { -*x } else { *x }).collect();
                mean(&xs).abs() + tolerance >= observed
            })
            .count();
        Some(extreme as f64 / total as f64)
    } else {
        None
    };
    let mut a: Vec<_> = pairs.iter().map(|x| x.0).collect();
    a.sort_by(f64::total_cmp);
    let mut b: Vec<_> = pairs.iter().map(|x| x.1).collect();
    b.sort_by(f64::total_cmp);
    Ok(Summary {
        method: "paired-log-v1/splitmix64/rejection-index/percentile",
        pairs: pairs.len(),
        raw_ratios: ratios,
        log_ratios: logs,
        geometric_ratio: estimate,
        bootstrap_95: [boots[(0.025 * (resamples - 1) as f64).floor() as usize], boots[(0.975 * (resamples - 1) as f64).ceil() as usize]],
        baseline_median: median(&a),
        candidate_median: median(&b),
        baseline_p95: a[(0.95 * a.len() as f64).ceil() as usize - 1],
        candidate_p95: b[(0.95 * b.len() as f64).ceil() as usize - 1],
        exact_sign_flip_p: p,
        bootstrap_resamples: resamples,
        seed,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constant_and_null() {
        let s = summarize(&[(2., 1.); 8], 1000, 7).unwrap();
        assert!((s.geometric_ratio - 2.).abs() < 1e-12);
        assert_eq!(s.bootstrap_95, [2., 2.]);
        assert_eq!(s.exact_sign_flip_p, Some(2. / 256.));
        let n = summarize(&[(3., 3.); 8], 1000, 7).unwrap();
        assert_eq!(n.exact_sign_flip_p, Some(1.));
        assert_eq!(n.geometric_ratio, 1.);
    }
    #[test]
    fn symmetry_seed_bounds() {
        let p = [(1., 2.), (4., 2.), (3., 2.), (8., 2.)];
        let s = summarize(&p, 2000, 19).unwrap();
        assert_eq!(s, summarize(&p, 2000, 19).unwrap());
        let rev: Vec<_> = p.iter().map(|(a, b)| (*b, *a)).collect();
        let r = summarize(&rev, 2000, 19).unwrap();
        assert!((s.geometric_ratio * r.geometric_ratio - 1.).abs() < 1e-12);
        assert_eq!(s.exact_sign_flip_p, r.exact_sign_flip_p);
        assert!((s.bootstrap_95[0] * r.bootstrap_95[1] - 1.).abs() < 1e-12);
        assert!(s.bootstrap_95[0] >= 0.5 && s.bootstrap_95[1] <= 4.00000001);
        assert_eq!(s.baseline_median, 3.5);
        assert_eq!(s.baseline_p95, 8.);
        assert_eq!(summarize(&[(1., 1.); 17], 100, 0).unwrap().exact_sign_flip_p, None);
    }
    #[test]
    fn invalid() {
        for x in [0., -1., f64::NAN, f64::INFINITY] {
            assert!(summarize(&[(x, 1.)], 100, 0).is_err());
        }
        assert!(summarize(&[], 100, 0).is_err());
        assert!(summarize(&[(1., 1.)], 0, 0).is_err());
        assert!(summarize(&[(f64::MAX, f64::MIN_POSITIVE)], 100, 0).is_err());
    }
    #[test]
    fn ties_and_known_p() {
        let s = summarize(&[(2., 1.), (2., 1.), (1., 1.)], 1000, 1).unwrap();
        assert_eq!(s.exact_sign_flip_p, Some(0.5));
    }
}
