//! Canonical Phase 5-A API contracts.
//!
//! These structs are the service-local source of truth for the Rust API and
//! for binding/schema generation. They intentionally use serde-compatible
//! shapes so the same records can be emitted as CBOR for replay and audit.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{AiCoreError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    Text,
    Audio,
    Vision,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[derive(Default)]
pub enum Priority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Urgent = 3,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FusionPayload {
    Text(String),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionInput {
    pub modality: Modality,
    pub payload: FusionPayload,
    pub mime_type: Option<String>,
    pub model_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionRequest {
    pub request_id: String,
    pub session_id: String,
    pub user_id: Option<String>,
    pub priority: Priority,
    pub inputs: Vec<FusionInput>,
    pub context: HashMap<String, String>,
    pub trace_id: Option<String>,
    pub max_latency_ms: Option<u64>,
    pub deterministic: bool,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionResponse {
    pub request_id: String,
    pub session_id: String,
    pub response_text: String,
    pub fused_modalities: Vec<Modality>,
    pub confidence: f32,
    pub privacy: Option<PrivacyDecision>,
    pub budget: Option<BudgetDecision>,
    pub trace_id: Option<String>,
    pub latency_ms: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskGraphStatus {
    Draft,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskGraph {
    pub graph_id: String,
    pub goal: String,
    pub nodes: Vec<TaskNode>,
    pub edges: Vec<TaskEdge>,
    pub status: TaskGraphStatus,
    pub created_at: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskNode {
    pub node_id: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub required_capabilities: Vec<String>,
    pub priority: Priority,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    Onnx,
    Gguf,
    Whisper,
    HuggingFace,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    Ed25519,
    EcdsaP256Sha256,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelSignature {
    pub algorithm: SignatureAlgorithm,
    pub public_key_b64: String,
    pub signature_b64: String,
    pub signer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelManifest {
    pub model_id: String,
    pub path: String,
    pub format: ModelFormat,
    pub modality: Modality,
    pub version: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub license: String,
    pub signatures: Vec<ModelSignature>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerificationResult {
    pub hash_verified: bool,
    pub signature_verified: bool,
    pub policy_allowed: bool,
    pub errors: Vec<String>,
}

impl VerificationResult {
    pub fn is_allowed(&self) -> bool {
        self.hash_verified && self.signature_verified && self.policy_allowed && self.errors.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdapterState {
    Registered,
    Attached,
    Training,
    Ready,
    Detached,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterManifest {
    pub adapter_id: String,
    pub base_model_id: String,
    pub rank: u32,
    pub alpha: f32,
    pub target_modules: Vec<String>,
    pub version: String,
    pub sha256: Option<String>,
    pub state: AdapterState,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrivacyRequest {
    pub request_id: String,
    pub user_id: Option<String>,
    pub fields: HashMap<String, serde_json::Value>,
    pub mode: RedactionMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedactionMode {
    None,
    Placeholder,
    Hash,
    Drop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyFinding {
    pub field: String,
    pub kind: String,
    pub confidence: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrivacyDecision {
    pub request_id: String,
    pub findings: Vec<PrivacyFinding>,
    pub redacted_fields: HashMap<String, serde_json::Value>,
    pub local_only: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetRequest {
    pub request_id: String,
    pub priority: Priority,
    pub cpu_percent: u8,
    pub memory_mb: u64,
    pub power_mw: u64,
    pub expected_duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetDecision {
    pub request_id: String,
    pub granted: bool,
    pub reason: String,
    pub queue_position: Option<usize>,
    pub active_cpu_percent: u8,
    pub active_memory_mb: u64,
    pub active_power_mw: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayRecord {
    pub event_id: String,
    pub sequence: u64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub canonical_hash: String,
    pub timestamp_90khz: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayRun {
    pub run_id: String,
    pub records: Vec<ReplayRecord>,
    pub tolerance: ReplayTolerance,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReplayTolerance {
    pub absolute_float: f64,
    pub relative_float: f64,
}

impl Default for ReplayTolerance {
    fn default() -> Self {
        Self {
            absolute_float: 0.0001,
            relative_float: 0.001,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayComparison {
    pub equal: bool,
    pub mismatches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftMetric {
    pub name: String,
    pub baseline: f64,
    pub current: f64,
    pub tolerance: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftReport {
    pub model_id: String,
    pub metrics: Vec<DriftMetric>,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub actor: String,
    pub action: String,
    pub subject: String,
    pub allowed: bool,
    pub reason: String,
    pub timestamp_90khz: u64,
    pub trace_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub fn timestamp_90khz() -> u64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_secs().saturating_mul(90_000) + u64::from(duration.subsec_micros()) * 90 / 1_000
}

pub fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_cbor::to_vec(value).map_err(|e| AiCoreError::SerializationError(e.to_string()))
}

pub fn stable_hash<T: Serialize>(value: &T) -> Result<String> {
    let bytes = to_canonical_cbor(value)?;
    Ok(sha256_hex(&bytes))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{:02x}", byte)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_hash_is_repeatable() {
        let request = FusionRequest {
            request_id: "req-1".to_string(),
            session_id: "sess-1".to_string(),
            user_id: None,
            priority: Priority::Normal,
            inputs: vec![FusionInput {
                modality: Modality::Text,
                payload: FusionPayload::Text("hello".to_string()),
                mime_type: Some("text/plain".to_string()),
                model_id: None,
                metadata: HashMap::new(),
            }],
            context: HashMap::new(),
            trace_id: Some("trace-1".to_string()),
            max_latency_ms: Some(50),
            deterministic: true,
            seed: Some(42),
        };

        assert_eq!(stable_hash(&request).unwrap(), stable_hash(&request).unwrap());
    }
}
