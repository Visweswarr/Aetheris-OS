//! On-device privacy classification and redaction for Phase 5-A.

use std::collections::HashMap;

use rand::{rngs::StdRng, Rng, SeedableRng};
use regex::Regex;
use serde_json::Value;

use crate::contracts::{
    sha256_hex, PrivacyDecision, PrivacyFinding, PrivacyRequest, RedactionMode,
};
use crate::error::{AiCoreError, Result};

#[derive(Debug, Clone)]
struct Detector {
    kind: String,
    regex: Regex,
    confidence: u8,
}

#[derive(Debug, Clone)]
pub struct PrivacyEngine {
    detectors: Vec<Detector>,
    sensitive_field_patterns: Vec<String>,
}

impl PrivacyEngine {
    pub fn new() -> Result<Self> {
        let detectors = vec![
            Detector {
                kind: "email".to_string(),
                regex: Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b")
                    .map_err(|e| AiCoreError::ConfigError(e.to_string()))?,
                confidence: 95,
            },
            Detector {
                kind: "phone".to_string(),
                regex: Regex::new(r"\b(?:\+?1[-.\s]?)?(?:\(?\d{3}\)?[-.\s]?)\d{3}[-.\s]?\d{4}\b")
                    .map_err(|e| AiCoreError::ConfigError(e.to_string()))?,
                confidence: 80,
            },
            Detector {
                kind: "ssn".to_string(),
                regex: Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")
                    .map_err(|e| AiCoreError::ConfigError(e.to_string()))?,
                confidence: 98,
            },
            Detector {
                kind: "secret".to_string(),
                regex: Regex::new(
                    r"(?i)\b(api[_-]?key|token|password|passwd|secret)\s*[:=]\s*[^\s,;]+",
                )
                .map_err(|e| AiCoreError::ConfigError(e.to_string()))?,
                confidence: 90,
            },
            Detector {
                kind: "phi".to_string(),
                regex: Regex::new(r"(?i)\b(diagnosis|patient|medical record|mrn|prescription)\b")
                    .map_err(|e| AiCoreError::ConfigError(e.to_string()))?,
                confidence: 70,
            },
        ];

        Ok(Self {
            detectors,
            sensitive_field_patterns: vec![
                "email".to_string(),
                "phone".to_string(),
                "ssn".to_string(),
                "password".to_string(),
                "token".to_string(),
                "secret".to_string(),
                "api_key".to_string(),
                "address".to_string(),
                "patient".to_string(),
                "diagnosis".to_string(),
                "mrn".to_string(),
            ],
        })
    }

    pub fn classify_and_redact(&self, request: PrivacyRequest) -> PrivacyDecision {
        let mut findings = Vec::new();
        let mut redacted_fields = HashMap::new();

        for (field, value) in request.fields {
            findings.extend(self.classify_field(&field, &value));
            let redacted_value = self.redact_value(&field, &value, request.mode);
            redacted_fields.insert(field, redacted_value);
        }

        let local_only = findings
            .iter()
            .any(|finding| finding.kind == "phi" || finding.confidence >= 90);

        PrivacyDecision {
            request_id: request.request_id,
            findings,
            redacted_fields,
            local_only,
        }
    }

    pub fn classify_text(&self, field: &str, text: &str) -> Vec<PrivacyFinding> {
        let mut findings = Vec::new();

        if self.is_sensitive_field(field) {
            findings.push(PrivacyFinding {
                field: field.to_string(),
                kind: "sensitive_field".to_string(),
                confidence: 85,
            });
        }

        for detector in &self.detectors {
            if detector.regex.is_match(text) {
                findings.push(PrivacyFinding {
                    field: field.to_string(),
                    kind: detector.kind.clone(),
                    confidence: detector.confidence,
                });
            }
        }

        findings
    }

    pub fn redact_text(&self, text: &str, mode: RedactionMode) -> String {
        if mode == RedactionMode::None {
            return text.to_string();
        }

        let mut redacted = text.to_string();
        for detector in &self.detectors {
            redacted = detector
                .regex
                .replace_all(&redacted, |caps: &regex::Captures| {
                    self.redacted_string(&caps[0], mode)
                })
                .to_string();
        }
        redacted
    }

    fn classify_field(&self, field: &str, value: &Value) -> Vec<PrivacyFinding> {
        match value {
            Value::String(text) => self.classify_text(field, text),
            Value::Object(map) => map
                .iter()
                .flat_map(|(key, value)| self.classify_field(&format!("{}.{}", field, key), value))
                .collect(),
            Value::Array(values) => values
                .iter()
                .enumerate()
                .flat_map(|(index, value)| {
                    self.classify_field(&format!("{}[{}]", field, index), value)
                })
                .collect(),
            _ => {
                if self.is_sensitive_field(field) {
                    vec![PrivacyFinding {
                        field: field.to_string(),
                        kind: "sensitive_field".to_string(),
                        confidence: 85,
                    }]
                } else {
                    Vec::new()
                }
            }
        }
    }

    fn redact_value(&self, field: &str, value: &Value, mode: RedactionMode) -> Value {
        if mode == RedactionMode::None {
            return value.clone();
        }

        if self.is_sensitive_field(field) {
            return self.redacted_value(value, mode);
        }

        match value {
            Value::String(text) => Value::String(self.redact_text(text, mode)),
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(key, value)| (key.clone(), self.redact_value(key, value, mode)))
                    .collect(),
            ),
            Value::Array(values) => Value::Array(
                values
                    .iter()
                    .map(|value| self.redact_value(field, value, mode))
                    .collect(),
            ),
            _ => value.clone(),
        }
    }

    fn redacted_value(&self, value: &Value, mode: RedactionMode) -> Value {
        match mode {
            RedactionMode::None => value.clone(),
            RedactionMode::Placeholder => Value::String("[REDACTED]".to_string()),
            RedactionMode::Hash => Value::String(format!(
                "sha256:{}",
                sha256_hex(value.to_string().as_bytes())
            )),
            RedactionMode::Drop => Value::Null,
        }
    }

    fn redacted_string(&self, text: &str, mode: RedactionMode) -> String {
        match mode {
            RedactionMode::None => text.to_string(),
            RedactionMode::Placeholder => "[REDACTED]".to_string(),
            RedactionMode::Hash => format!("sha256:{}", sha256_hex(text.as_bytes())),
            RedactionMode::Drop => String::new(),
        }
    }

    fn is_sensitive_field(&self, field: &str) -> bool {
        let field = field.to_lowercase();
        self.sensitive_field_patterns
            .iter()
            .any(|pattern| field.contains(pattern))
    }
}

impl Default for PrivacyEngine {
    fn default() -> Self {
        Self::new().expect("default privacy detectors must compile")
    }
}

pub fn sample_laplace_with_rng<R: Rng + ?Sized>(
    epsilon: f64,
    sensitivity: f64,
    rng: &mut R,
) -> f64 {
    let epsilon = if epsilon <= 0.0 { f64::EPSILON } else { epsilon };
    let scale = sensitivity / epsilon;
    let u = rng.gen::<f64>() - 0.5;
    let sign = if u < 0.0 { 1.0 } else { -1.0 };
    scale * sign * (1.0 - 2.0 * u.abs()).ln()
}

pub fn apply_noise_seeded(value: f64, epsilon: f64, sensitivity: f64, seed: u64) -> f64 {
    let mut rng = StdRng::seed_from_u64(seed);
    value + sample_laplace_with_rng(epsilon, sensitivity, &mut rng)
}

pub fn export_metric_with_privacy(
    value: f64,
    epsilon: f64,
    sensitivity: f64,
    seed: u64,
    telemetry_allowed: bool,
) -> Result<f64> {
    if !telemetry_allowed {
        return Err(AiCoreError::CapDenied(
            "telemetry.send capability required for off-device metric export".to_string(),
        ));
    }
    Ok(apply_noise_seeded(value, epsilon, sensitivity, seed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_and_redacts_pii() {
        let engine = PrivacyEngine::new().unwrap();
        let mut fields = HashMap::new();
        fields.insert(
            "body".to_string(),
            Value::String("Contact ada@example.com".to_string()),
        );

        let decision = engine.classify_and_redact(PrivacyRequest {
            request_id: "privacy-1".to_string(),
            user_id: None,
            fields,
            mode: RedactionMode::Placeholder,
        });

        assert!(decision
            .findings
            .iter()
            .any(|finding| finding.kind == "email"));
        assert_eq!(
            decision.redacted_fields["body"],
            Value::String("Contact [REDACTED]".to_string())
        );
    }

    #[test]
    fn seeded_dp_noise_is_deterministic() {
        let a = apply_noise_seeded(10.0, 1.0, 1.0, 42);
        let b = apply_noise_seeded(10.0, 1.0, 1.0, 42);
        assert_eq!(a, b);
        assert_ne!(a, 10.0);
    }

    #[test]
    fn off_device_metric_export_requires_telemetry_capability() {
        let denied = export_metric_with_privacy(5.0, 1.0, 1.0, 7, false).unwrap_err();
        assert!(denied.to_string().contains("telemetry.send"));

        let allowed = export_metric_with_privacy(5.0, 1.0, 1.0, 7, true).unwrap();
        assert_ne!(allowed, 5.0);
    }
}
