use crate::types::events::Event;

#[derive(Debug, Clone)]
pub struct GdsResult {
    pub gds: Option<f64>,
    pub n_levels: usize,
    pub stress_levels: Vec<f64>,
    pub completion_rates: Vec<f64>,
    pub monotonicity: Option<f64>,
    pub smoothness: Option<f64>,
    pub na_reason: Option<String>,
}

fn compute_monotonicity(rates: &[f64]) -> Option<f64> {
    if rates.len() < 2 { return None; }
    let pairs = rates.len() - 1;
    let mono = rates.windows(2).filter(|w| w[1] <= w[0]).count();
    Some(mono as f64 / pairs as f64)
}

fn compute_smoothness(rates: &[f64]) -> Option<f64> {
    if rates.len() < 2 { return None; }

    let steps: Vec<f64> = rates.windows(2)
        .map(|w| (w[0] - w[1]).max(0.0))
        .collect();
    let total_drop: f64 = steps.iter().sum();

    if total_drop == 0.0 { return Some(1.0); }

    let nonzero: Vec<f64> = steps.iter().copied().filter(|&s| s > 0.0).collect();
    let n_nonzero = nonzero.len();

    if n_nonzero <= 1 {
        return if steps.len() == 1 { Some(1.0) } else { Some(0.0) };
    }

    let p: Vec<f64> = nonzero.iter().map(|&s| s / total_drop).collect();
    let h: f64 = -p.iter().map(|&pi| pi * pi.ln()).sum::<f64>();
    let h_max = (n_nonzero as f64).ln();

    Some(if h_max > 0.0 { h / h_max } else { 1.0 })
}

/// BP-1: Graceful Degradation Score.
/// GDS = (1/n) * sum(C_i) where C_i is completion rate at stress level i.
pub fn compute_gds(events: &[Event], expected_levels: Option<&[f64]>) -> GdsResult {
    let mut levels = Vec::new();
    let mut rates = Vec::new();

    for e in events {
        if let (Some(sl), Some(cr)) = (e.stress_level, e.completion_rate) {
            levels.push(sl);
            rates.push(cr);
        }
    }

    if levels.is_empty() {
        return GdsResult {
            gds: None, n_levels: 0,
            stress_levels: vec![], completion_rates: vec![],
            monotonicity: None, smoothness: None,
            na_reason: Some("No (stress_level, completion_rate) evidence found".to_string()),
        };
    }

    for &r in &rates {
        if !(0.0..=1.0).contains(&r) {
            return GdsResult {
                gds: None, n_levels: levels.len(),
                stress_levels: levels, completion_rates: rates,
                monotonicity: None, smoothness: None,
                na_reason: Some(format!("completion_rate out of bounds: {r}")),
            };
        }
    }

    // Tolerance-based expected level enforcement
    if let Some(expected) = expected_levels {
        let missing: Vec<f64> = expected
            .iter()
            .filter(|&&exp| !levels.iter().any(|&got| (exp - got).abs() < 1e-9))
            .copied()
            .collect();
        if !missing.is_empty() {
            return GdsResult {
                gds: None, n_levels: levels.len(),
                stress_levels: levels, completion_rates: rates,
                monotonicity: None, smoothness: None,
                na_reason: Some(format!("Missing declared stress levels: {missing:?}")),
            };
        }
    }

    let n = rates.len() as f64;
    let gds = (rates.iter().sum::<f64>() / n).clamp(0.0, 1.0);

    // Sort by stress level for disclosure
    let mut paired: Vec<(f64, f64)> = levels.into_iter().zip(rates).collect();
    paired.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let s_sorted: Vec<f64> = paired.iter().map(|p| p.0).collect();
    let c_sorted: Vec<f64> = paired.iter().map(|p| p.1).collect();
    let mono = compute_monotonicity(&c_sorted);
    let smooth = compute_smoothness(&c_sorted);

    GdsResult {
        gds: Some(gds),
        n_levels: s_sorted.len(),
        stress_levels: s_sorted,
        completion_rates: c_sorted,
        monotonicity: mono,
        smoothness: smooth,
        na_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::events::{EventLog, EventType};

    #[test]
    fn perfect_gds() {
        let mut log = EventLog::new("t", "W1-A");
        log.emit(EventType::RunStart);
        for &level in &[0.1, 0.2, 0.3] {
            let e = log.emit(EventType::WorkUnitEnd);
            e.stress_level = Some(level);
            e.completion_rate = Some(1.0);
        }
        log.emit(EventType::RunEnd);

        let result = compute_gds(log.events(), Some(&[0.1, 0.2, 0.3]));
        assert_eq!(result.gds, Some(1.0));
    }

    #[test]
    fn degrading_gds() {
        let mut log = EventLog::new("t", "W1-A");
        log.emit(EventType::RunStart);
        for (&level, &rate) in [0.1, 0.2, 0.3].iter().zip(&[1.0, 0.5, 0.0]) {
            let e = log.emit(EventType::WorkUnitEnd);
            e.stress_level = Some(level);
            e.completion_rate = Some(rate);
        }
        log.emit(EventType::RunEnd);

        let result = compute_gds(log.events(), Some(&[0.1, 0.2, 0.3]));
        assert!((result.gds.unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn no_evidence_na() {
        let mut log = EventLog::new("t", "W1-A");
        log.emit(EventType::RunStart);
        log.emit(EventType::RunEnd);
        let result = compute_gds(log.events(), None);
        assert!(result.gds.is_none());
    }

    #[test]
    fn smoothness_uniform() {
        let mut log = EventLog::new("t", "W1-A");
        log.emit(EventType::RunStart);
        for (&level, &rate) in [0.1, 0.2, 0.3].iter().zip(&[1.0, 0.8, 0.6]) {
            let e = log.emit(EventType::WorkUnitEnd);
            e.stress_level = Some(level);
            e.completion_rate = Some(rate);
        }
        log.emit(EventType::RunEnd);
        let r = compute_gds(log.events(), Some(&[0.1, 0.2, 0.3]));
        assert!((r.smoothness.unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    fn smoothness_cliff_drop() {
        let mut log = EventLog::new("t", "W1-A");
        log.emit(EventType::RunStart);
        for (&level, &rate) in [0.1, 0.2, 0.3].iter().zip(&[1.0, 1.0, 0.4]) {
            let e = log.emit(EventType::WorkUnitEnd);
            e.stress_level = Some(level);
            e.completion_rate = Some(rate);
        }
        log.emit(EventType::RunEnd);
        let r = compute_gds(log.events(), Some(&[0.1, 0.2, 0.3]));
        assert!((r.smoothness.unwrap() - 0.0).abs() < 0.01);
    }

    #[test]
    fn monotonicity_decreasing() {
        let mut log = EventLog::new("t", "W1-A");
        log.emit(EventType::RunStart);
        for (&level, &rate) in [0.1, 0.2, 0.3].iter().zip(&[1.0, 0.8, 0.6]) {
            let e = log.emit(EventType::WorkUnitEnd);
            e.stress_level = Some(level);
            e.completion_rate = Some(rate);
        }
        log.emit(EventType::RunEnd);
        let r = compute_gds(log.events(), Some(&[0.1, 0.2, 0.3]));
        assert_eq!(r.monotonicity, Some(1.0));
    }

    #[test]
    fn monotonicity_non_monotonic() {
        let mut log = EventLog::new("t", "W1-A");
        log.emit(EventType::RunStart);
        for (&level, &rate) in [0.1, 0.2, 0.3].iter().zip(&[0.5, 0.8, 0.6]) {
            let e = log.emit(EventType::WorkUnitEnd);
            e.stress_level = Some(level);
            e.completion_rate = Some(rate);
        }
        log.emit(EventType::RunEnd);
        let r = compute_gds(log.events(), Some(&[0.1, 0.2, 0.3]));
        assert!((r.monotonicity.unwrap() - 0.5).abs() < 0.01);
    }
}
