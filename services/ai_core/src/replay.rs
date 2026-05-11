//! Deterministic replay records with adaptive numeric tolerances.

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;

use crate::contracts::{
    stable_hash, timestamp_90khz, ReplayComparison, ReplayRecord, ReplayRun, ReplayTolerance,
};
use crate::error::Result;

#[derive(Debug, Clone, Default)]
pub struct ReplayRecorder {
    records: Arc<RwLock<Vec<ReplayRecord>>>,
}

impl ReplayRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record(&self, event_type: impl Into<String>, payload: Value) -> Result<ReplayRecord> {
        let mut records = self.records.write().await;
        let sequence = records.len() as u64;
        let event_type = event_type.into();
        let record = ReplayRecord {
            event_id: format!("replay-{}-{}", event_type, sequence),
            sequence,
            event_type,
            canonical_hash: stable_hash(&payload)?,
            payload,
            timestamp_90khz: timestamp_90khz(),
        };
        records.push(record.clone());
        Ok(record)
    }

    pub async fn run(&self, run_id: impl Into<String>, tolerance: ReplayTolerance) -> ReplayRun {
        ReplayRun {
            run_id: run_id.into(),
            records: self.records.read().await.clone(),
            tolerance,
        }
    }

    pub async fn clear(&self) {
        self.records.write().await.clear();
    }
}

pub fn compare_replay(expected: &ReplayRun, actual: &ReplayRun) -> ReplayComparison {
    let tolerance = actual.tolerance;
    let mut mismatches = Vec::new();

    if expected.records.len() != actual.records.len() {
        mismatches.push(format!(
            "record count mismatch: expected {}, got {}",
            expected.records.len(),
            actual.records.len()
        ));
    }

    for (index, (left, right)) in expected.records.iter().zip(actual.records.iter()).enumerate() {
        if left.event_type != right.event_type {
            mismatches.push(format!(
                "record {} event type mismatch: expected {}, got {}",
                index, left.event_type, right.event_type
            ));
        }

        compare_json(
            &left.payload,
            &right.payload,
            &format!("records[{}].payload", index),
            tolerance,
            &mut mismatches,
        );
    }

    ReplayComparison {
        equal: mismatches.is_empty(),
        mismatches,
    }
}

fn compare_json(
    expected: &Value,
    actual: &Value,
    path: &str,
    tolerance: ReplayTolerance,
    mismatches: &mut Vec<String>,
) {
    match (expected, actual) {
        (Value::Number(left), Value::Number(right)) => {
            let Some(left) = left.as_f64() else {
                if expected != actual {
                    mismatches.push(format!("{} integer mismatch: expected {}, got {}", path, expected, actual));
                }
                return;
            };
            let Some(right) = right.as_f64() else {
                mismatches.push(format!("{} number type mismatch: expected {}, got {}", path, expected, actual));
                return;
            };

            let absolute = (left - right).abs();
            let relative = if left.abs() > f64::EPSILON {
                absolute / left.abs()
            } else {
                absolute
            };

            if absolute > tolerance.absolute_float && relative > tolerance.relative_float {
                mismatches.push(format!(
                    "{} float mismatch: expected {}, got {}, abs {}, rel {}",
                    path, left, right, absolute, relative
                ));
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                mismatches.push(format!("{} array length mismatch: expected {}, got {}", path, left.len(), right.len()));
            }
            for (index, (left, right)) in left.iter().zip(right.iter()).enumerate() {
                compare_json(left, right, &format!("{}[{}]", path, index), tolerance, mismatches);
            }
        }
        (Value::Object(left), Value::Object(right)) => {
            for (key, left_value) in left {
                if let Some(right_value) = right.get(key) {
                    compare_json(left_value, right_value, &format!("{}.{}", path, key), tolerance, mismatches);
                } else {
                    mismatches.push(format!("{} missing key {}", path, key));
                }
            }
            for key in right.keys() {
                if !left.contains_key(key) {
                    mismatches.push(format!("{} unexpected key {}", path, key));
                }
            }
        }
        _ => {
            if expected != actual {
                mismatches.push(format!("{} mismatch: expected {}, got {}", path, expected, actual));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn replay_allows_small_float_drift() {
        let recorder = ReplayRecorder::new();
        recorder.record("fusion", json!({"confidence": 0.9000})).await.unwrap();
        let expected = recorder.run("expected", ReplayTolerance::default()).await;

        let recorder = ReplayRecorder::new();
        recorder.record("fusion", json!({"confidence": 0.90005})).await.unwrap();
        let actual = recorder.run("actual", ReplayTolerance::default()).await;

        assert!(compare_replay(&expected, &actual).equal);
    }
}
