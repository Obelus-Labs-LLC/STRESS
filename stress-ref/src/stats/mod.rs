use crate::types::report::AggregateStats;
use statrs::distribution::{ContinuousCDF, StudentsT};

/// Compute two-tailed 95% critical value using t-distribution.
/// Falls back to z=1.96 for df >= 30.
fn critical_value(n: usize) -> f64 {
    let df = n - 1;
    if df >= 1 && df < 30 {
        StudentsT::new(0.0, 1.0, df as f64)
            .unwrap()
            .inverse_cdf(0.975)
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

/// Cohen's d effect size between two groups.
/// d = (mean_a - mean_b) / pooled_std
/// Returns None if either group has fewer than 2 values or pooled std is zero.
pub fn cohens_d(group_a: &[f64], group_b: &[f64]) -> Option<f64> {
    if group_a.len() < 2 || group_b.len() < 2 {
        return None;
    }
    let n_a = group_a.len() as f64;
    let n_b = group_b.len() as f64;
    let m_a = group_a.iter().sum::<f64>() / n_a;
    let m_b = group_b.iter().sum::<f64>() / n_b;
    let var_a = group_a.iter().map(|x| (x - m_a).powi(2)).sum::<f64>() / (n_a - 1.0);
    let var_b = group_b.iter().map(|x| (x - m_b).powi(2)).sum::<f64>() / (n_b - 1.0);
    let pooled_var = ((n_a - 1.0) * var_a + (n_b - 1.0) * var_b) / (n_a + n_b - 2.0);
    let pooled_std = pooled_var.sqrt();
    if pooled_std == 0.0 {
        return None;
    }
    Some((m_a - m_b) / pooled_std)
}

/// Detect outliers using Modified Z-score with Median Absolute Deviation.
/// Returns indices of values where |modified_z| > threshold.
pub fn mad_outliers(values: &[f64], threshold: f64) -> Vec<usize> {
    if values.len() < 3 {
        return vec![];
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let median = if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    };

    let mut abs_devs: Vec<f64> = values.iter().map(|x| (x - median).abs()).collect();
    abs_devs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mad = if abs_devs.len() % 2 == 0 {
        (abs_devs[abs_devs.len() / 2 - 1] + abs_devs[abs_devs.len() / 2]) / 2.0
    } else {
        abs_devs[abs_devs.len() / 2]
    };

    if mad == 0.0 {
        return vec![];
    }

    values
        .iter()
        .enumerate()
        .filter(|(_, x)| {
            let modified_z = 0.6745 * (*x - median) / mad;
            modified_z.abs() > threshold
        })
        .map(|(i, _)| i)
        .collect()
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
    fn cohens_d_identical_groups() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((cohens_d(&a, &b).unwrap() - 0.0).abs() < 0.01);
    }

    #[test]
    fn cohens_d_different_groups() {
        let a = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let d = cohens_d(&a, &b).unwrap();
        assert!(d > 4.0); // Large effect size
    }

    #[test]
    fn cohens_d_insufficient_data() {
        assert!(cohens_d(&[1.0], &[2.0, 3.0]).is_none());
    }

    #[test]
    fn mad_detects_outlier() {
        let values = vec![1.0, 1.1, 0.9, 1.0, 1.05, 0.95, 10.0];
        let outliers = mad_outliers(&values, 3.5);
        assert!(outliers.contains(&6)); // 10.0 is the outlier
    }

    #[test]
    fn mad_no_outliers() {
        let values = vec![1.0, 1.1, 0.9, 1.0, 1.05, 0.95];
        let outliers = mad_outliers(&values, 3.5);
        assert!(outliers.is_empty());
    }

    #[test]
    fn mad_constant_values() {
        let values = vec![5.0, 5.0, 5.0, 5.0];
        let outliers = mad_outliers(&values, 3.5);
        assert!(outliers.is_empty());
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
