//! Statistical drift checks used by CI gates.

use std::collections::HashMap;

use crate::contracts::{DriftMetric, DriftReport};

pub fn compare_metrics(
    model_id: impl Into<String>,
    baseline: &HashMap<String, f64>,
    current: &HashMap<String, f64>,
    tolerances: &HashMap<String, f64>,
) -> DriftReport {
    let mut metrics = Vec::new();

    for (name, baseline_value) in baseline {
        let current_value = current.get(name).copied().unwrap_or(f64::NAN);
        let tolerance = tolerances.get(name).copied().unwrap_or(0.05);
        let delta = if baseline_value.abs() > f64::EPSILON {
            ((current_value - baseline_value) / baseline_value).abs()
        } else {
            (current_value - baseline_value).abs()
        };

        metrics.push(DriftMetric {
            name: name.clone(),
            baseline: *baseline_value,
            current: current_value,
            tolerance,
            passed: current_value.is_finite() && delta <= tolerance,
        });
    }

    DriftReport {
        model_id: model_id.into(),
        passed: metrics.iter().all(|metric| metric.passed),
        metrics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_metric_drift() {
        let baseline = HashMap::from([("accuracy".to_string(), 0.95)]);
        let current = HashMap::from([("accuracy".to_string(), 0.80)]);
        let tolerance = HashMap::from([("accuracy".to_string(), 0.05)]);

        let report = compare_metrics("model-1", &baseline, &current, &tolerance);
        assert!(!report.passed);
    }
}
