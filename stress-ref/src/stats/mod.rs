use crate::types::report::AggregateStats;

/// Two-tailed 95% t-distribution critical values for df=1..29.
/// For df >= 30, normal approximation z=1.96 is adequate.
const T_CRIT_95: [f64; 29] = [
    12.706, 4.303, 3.182, 2.776, 2.571,
    2.447, 2.365, 2.306, 2.262, 2.228,
    2.201, 2.179, 2.160, 2.145, 2.131,
    2.120, 2.110, 2.101, 2.093, 2.086,
    2.080, 2.074, 2.069, 2.064, 2.060,
    2.056, 2.052, 2.048, 2.045,
];

fn critical_value(n: usize) -> f64 {
    let df = n - 1;
    if df >= 1 && df <= 29 {
        T_CRIT_95[df - 1]
    } else {
        1.96
    }
}

/// Compute mean, sample std dev, and 95% CI over included values.
/// N/A values are excluded but counted. CI reported without clamping.
/// Uses t-distribution for n < 30, normal approximation for n >= 30.
pub fn summarize(values: &[Option<f64>]) -> AggregateStats {
    let included: Vec<f64> = values.iter().filter_map(|v| *v).collect();
    let n_na = values.iter().filter(|v| v.is_none()).count();

    if included.is_empty() {
        return AggregateStats {
            mean: None, std: None, ci95_low: None, ci95_high: None,
            n_included: 0, n_na,
        };
    }

    let n = included.len();
    let mean = included.iter().sum::<f64>() / n as f64;

    if n == 1 {
        return AggregateStats {
            mean: Some(mean), std: Some(0.0),
            ci95_low: Some(mean), ci95_high: Some(mean),
            n_included: 1, n_na,
        };
    }

    let var = included.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let std = var.sqrt();
    let se = std / (n as f64).sqrt();
    let z = critical_value(n);

    AggregateStats {
        mean: Some(mean),
        std: Some(std),
        ci95_low: Some(mean - z * se),
        ci95_high: Some(mean + z * se),
        n_included: n,
        n_na,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_stats() {
        let result = summarize(&[Some(0.5), Some(0.5), Some(0.5)]);
        assert_eq!(result.mean, Some(0.5));
        assert_eq!(result.std, Some(0.0));
        assert_eq!(result.n_included, 3);
    }

    #[test]
    fn all_na() {
        let result = summarize(&[None, None]);
        assert!(result.mean.is_none());
        assert_eq!(result.n_na, 2);
    }

    #[test]
    fn mixed() {
        let result = summarize(&[Some(1.0), None, Some(0.5)]);
        assert_eq!(result.n_included, 2);
        assert_eq!(result.n_na, 1);
        assert!((result.mean.unwrap() - 0.75).abs() < 0.01);
    }

    #[test]
    fn t_distribution_wider_than_z_for_small_n() {
        let vals = vec![Some(0.5), Some(0.6), Some(0.7), Some(0.4), Some(0.8)];
        let result = summarize(&vals);
        let mean = result.mean.unwrap();
        let std = result.std.unwrap();
        let se = std / 5.0_f64.sqrt();
        let z_ci_half = 1.96 * se;
        let actual_ci_half = mean - result.ci95_low.unwrap();
        // t(df=4)=2.776 > 1.96, so actual CI should be wider
        assert!(actual_ci_half > z_ci_half);
    }
}
