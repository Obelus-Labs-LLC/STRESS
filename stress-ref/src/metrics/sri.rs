use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct SriResult {
    pub sri: Option<f64>,
    pub na_reason: Option<String>,
}

/// A named weighting profile for domain-specific SRI computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightProfile {
    pub name: String,
    /// Weights keyed by proxy name ("gds", "arr", "ist", "rec", "cfr"). Must sum to 1.0.
    pub weights: BTreeMap<String, f64>,
}

/// Compute weighted SRI using domain-specific proxy weights (weighted geometric mean).
/// SRI = exp(sum(w_i * ln(proxy_i))) * 100, clamped to [0, 100].
pub fn compute_weighted_sri(
    proxies: &BTreeMap<&str, Option<f64>>,
    profile: &WeightProfile,
) -> SriResult {
    let required = ["gds", "arr", "ist", "rec", "cfr"];

    // Validate all required proxies exist
    for &name in &required {
        match proxies.get(name) {
            None => return SriResult { sri: None, na_reason: Some(format!("missing proxy: {}", name)) },
            Some(None) => return SriResult { sri: None, na_reason: Some(format!("{} is N/A", name)) },
            Some(Some(_)) => {}
        }
    }

    // Validate weights sum to ~1.0
    let weight_sum: f64 = required.iter()
        .filter_map(|&k| profile.weights.get(k))
        .sum();
    if (weight_sum - 1.0).abs() > 0.01 {
        return SriResult { sri: None, na_reason: Some(format!("weights sum to {}, expected 1.0", weight_sum)) };
    }

    // Zero proxy -> SRI = 0
    if required.iter().any(|&k| proxies.get(k).unwrap().unwrap() == 0.0) {
        return SriResult { sri: Some(0.0), na_reason: None };
    }

    // Weighted geometric mean: exp(sum(w_i * ln(proxy_i))) * 100
    let log_sum: f64 = required.iter()
        .map(|&k| {
            let value = proxies.get(k).unwrap().unwrap();
            let weight = profile.weights.get(k).copied().unwrap_or(0.2);
            weight * value.ln()
        })
        .sum();

    let sri = log_sum.exp() * 100.0;
    let sri = sri.clamp(0.0, 100.0);
    SriResult { sri: Some(sri), na_reason: None }
}

/// SRI via geometric mean: (GDS * ARR * IST * REC * CFR)^(1/5) * 100.
/// Geometric mean ensures zero resilience in any dimension drives SRI toward 0.
/// N/A if any proxy is N/A.
pub fn compute_sri(proxies: &BTreeMap<&str, Option<f64>>) -> SriResult {
    let required = ["gds", "arr", "ist", "rec", "cfr"];

    let missing: Vec<&&str> = required
        .iter()
        .filter(|k| !proxies.contains_key(*k))
        .collect();
    if !missing.is_empty() {
        return SriResult {
            sri: None,
            na_reason: Some(format!("missing proxies: {missing:?}")),
        };
    }

    let na: Vec<&&str> = required
        .iter()
        .filter(|k| proxies.get(*k).copied().flatten().is_none())
        .collect();
    if !na.is_empty() {
        return SriResult {
            sri: None,
            na_reason: Some(format!("SRI N/A because proxies N/A: {na:?}")),
        };
    }

    let values: Vec<f64> = required
        .iter()
        .map(|k| proxies.get(k).unwrap().unwrap())
        .collect();

    // Zero proxy -> SRI = 0 (zero resilience in any dimension = zero overall)
    if values.iter().any(|&v| v == 0.0) {
        return SriResult { sri: Some(0.0), na_reason: None };
    }

    // Geometric mean: (product)^(1/5) * 100
    let log_sum: f64 = values.iter().map(|v| v.ln()).sum::<f64>();
    let sri = (log_sum / 5.0).exp() * 100.0;
    let sri = sri.clamp(0.0, 100.0);

    SriResult {
        sri: Some(sri),
        na_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_sri() {
        let mut proxies = BTreeMap::new();
        for k in &["gds", "arr", "ist", "rec", "cfr"] {
            proxies.insert(*k, Some(1.0));
        }
        let result = compute_sri(&proxies);
        assert_eq!(result.sri, Some(100.0));
    }

    #[test]
    fn partial_na() {
        let mut proxies = BTreeMap::new();
        proxies.insert("gds", Some(1.0));
        proxies.insert("arr", None);
        proxies.insert("ist", Some(1.0));
        proxies.insert("rec", Some(1.0));
        proxies.insert("cfr", Some(1.0));
        let result = compute_sri(&proxies);
        assert!(result.sri.is_none());
    }

    #[test]
    fn geometric_mean() {
        let mut proxies = BTreeMap::new();
        proxies.insert("gds", Some(0.8));
        proxies.insert("arr", Some(0.6));
        proxies.insert("ist", Some(1.0));
        proxies.insert("rec", Some(0.4));
        proxies.insert("cfr", Some(0.2));
        let result = compute_sri(&proxies);
        let expected = (0.8_f64 * 0.6 * 1.0 * 0.4 * 0.2).powf(1.0 / 5.0) * 100.0;
        assert!((result.sri.unwrap() - expected).abs() < 0.1);
    }

    #[test]
    fn zero_proxy_means_zero_sri() {
        let mut proxies = BTreeMap::new();
        proxies.insert("gds", Some(1.0));
        proxies.insert("arr", Some(1.0));
        proxies.insert("ist", Some(1.0));
        proxies.insert("rec", Some(1.0));
        proxies.insert("cfr", Some(0.0));
        let result = compute_sri(&proxies);
        assert_eq!(result.sri, Some(0.0));
    }

    #[test]
    fn test_weighted_equal_matches_unweighted() {
        let mut proxies = BTreeMap::new();
        proxies.insert("gds", Some(0.8));
        proxies.insert("arr", Some(0.6));
        proxies.insert("ist", Some(1.0));
        proxies.insert("rec", Some(0.4));
        proxies.insert("cfr", Some(0.2));

        let equal_profile = WeightProfile {
            name: "equal".to_string(),
            weights: BTreeMap::from([
                ("gds".to_string(), 0.2),
                ("arr".to_string(), 0.2),
                ("ist".to_string(), 0.2),
                ("rec".to_string(), 0.2),
                ("cfr".to_string(), 0.2),
            ]),
        };

        let unweighted = compute_sri(&proxies);
        let weighted = compute_weighted_sri(&proxies, &equal_profile);
        assert!((unweighted.sri.unwrap() - weighted.sri.unwrap()).abs() < 0.01);
    }

    #[test]
    fn test_satellite_profile() {
        // IST=0.35 weight, with IST=1.0 → higher SRI than data center profile
        let mut proxies = BTreeMap::new();
        proxies.insert("gds", Some(0.5));
        proxies.insert("arr", Some(0.5));
        proxies.insert("ist", Some(1.0));
        proxies.insert("rec", Some(0.5));
        proxies.insert("cfr", Some(0.2));

        let satellite = WeightProfile {
            name: "satellite-leo".to_string(),
            weights: BTreeMap::from([
                ("gds".to_string(), 0.20),
                ("arr".to_string(), 0.20),
                ("ist".to_string(), 0.35),
                ("rec".to_string(), 0.15),
                ("cfr".to_string(), 0.10),
            ]),
        };

        let dc = WeightProfile {
            name: "data-center".to_string(),
            weights: BTreeMap::from([
                ("gds".to_string(), 0.20),
                ("arr".to_string(), 0.20),
                ("ist".to_string(), 0.10),
                ("rec".to_string(), 0.15),
                ("cfr".to_string(), 0.35),
            ]),
        };

        let sat_result = compute_weighted_sri(&proxies, &satellite);
        let dc_result = compute_weighted_sri(&proxies, &dc);
        assert!(sat_result.sri.unwrap() > dc_result.sri.unwrap());
    }

    #[test]
    fn test_data_center_profile() {
        // CFR=0.35 weight, with CFR=0.2 → lower SRI
        let mut proxies = BTreeMap::new();
        proxies.insert("gds", Some(0.5));
        proxies.insert("arr", Some(0.5));
        proxies.insert("ist", Some(1.0));
        proxies.insert("rec", Some(0.5));
        proxies.insert("cfr", Some(0.2));

        let dc = WeightProfile {
            name: "data-center".to_string(),
            weights: BTreeMap::from([
                ("gds".to_string(), 0.20),
                ("arr".to_string(), 0.20),
                ("ist".to_string(), 0.10),
                ("rec".to_string(), 0.15),
                ("cfr".to_string(), 0.35),
            ]),
        };

        let equal = WeightProfile {
            name: "equal".to_string(),
            weights: BTreeMap::from([
                ("gds".to_string(), 0.2),
                ("arr".to_string(), 0.2),
                ("ist".to_string(), 0.2),
                ("rec".to_string(), 0.2),
                ("cfr".to_string(), 0.2),
            ]),
        };

        let dc_result = compute_weighted_sri(&proxies, &dc);
        let equal_result = compute_weighted_sri(&proxies, &equal);
        // Data center profile with low CFR (0.2) and high CFR weight (0.35) → lower SRI
        assert!(dc_result.sri.unwrap() < equal_result.sri.unwrap());
    }
}
