//! Model Runtime Adapter for AI Core Service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::contracts::sha256_hex;
use crate::error::Result;

pub mod backend;
pub mod config;
pub mod gguf;
pub mod heg;
pub mod onnx;

pub use heg::{
    audit_event_for_plan, AcceleratorKind, HegEdge, HegNode, HegPlan, HegPolicy, OperatorKind,
    PlacementDecision, ResourceProfile,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeConfig {
    pub model_path: String,
    pub model_name: String,
    pub device: String,
    pub num_threads: usize,
    pub context_length: usize,
    pub batch_size: usize,
    pub deterministic: bool,
    pub seed: Option<u64>,
    pub memory_pool_size: usize,
    pub enable_profiling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeRequest {
    pub prompt: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_sequences: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResponse {
    pub generated_text: String,
    pub tokens_generated: u32,
    pub processing_time_ms: u64,
    pub backend_kind: backend::RuntimeBackendKind,
    pub backend_metadata: HashMap<String, String>,
    pub heg_plan_id: Option<String>,
    pub placement_summary: Vec<PlacementDecision>,
    pub audit_event: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeStats {
    pub total_requests: u64,
    pub total_tokens_generated: u64,
    pub total_processing_time_ms: u64,
    pub avg_tokens_per_second: f32,
    pub memory_usage_mb: u32,
    pub active_sessions: u32,
    pub uptime_seconds: u64,
    pub last_request_timestamp: u64,
    pub last_heg_plan_id: Option<String>,
}

pub struct RuntimeManager {
    config: RuntimeConfig,
    stats: Arc<RwLock<RuntimeStats>>,
    backend: Arc<dyn backend::RuntimeBackend>,
}

impl RuntimeManager {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        Ok(Self {
            config,
            stats: Arc::new(RwLock::new(RuntimeStats::default())),
            backend: Arc::new(backend::DeterministicLocalBackend::new()),
        })
    }

    pub fn new_mock() -> Self {
        Self {
            config: RuntimeConfig::default(),
            stats: Arc::new(RwLock::new(RuntimeStats::default())),
            backend: Arc::new(backend::DeterministicLocalBackend::new()),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Runtime manager initialized
        Ok(())
    }

    pub async fn generate_response(&self, request: &RuntimeRequest) -> Result<RuntimeResponse> {
        self.generate_response_with_policy(request, &HegPolicy::default()).await
    }

    pub async fn generate_response_with_policy(
        &self,
        request: &RuntimeRequest,
        policy: &HegPolicy,
    ) -> Result<RuntimeResponse> {
        let start = std::time::Instant::now();
        let heg_plan = self.plan_execution_graph_with_policy(request, policy)?;
        crate::metrics::record_heg_plan(&heg_plan);
        let audit_event = audit_event_for_plan(&heg_plan);

        // Execute via backend
        let exec_req = backend::RuntimeExecutionRequest {
            prompt: request.prompt.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stop_sequences: request.stop_sequences.clone(),
            heg_plan: Some(heg_plan.clone()),
            metadata: request.metadata.clone(),
        };

        let exec_res = self.backend.execute(&exec_req).await;
        let latency_ms = start.elapsed().as_millis() as u64;

        let response = match exec_res {
            Ok(res) => {
                crate::metrics::record_runtime_execution(true, res.tokens_generated as u64, latency_ms);
                let execution_hash = runtime_execution_hash(
                    self.backend.kind(),
                    &res.generated_text,
                    res.tokens_generated,
                    &res.metadata,
                );
                let backend_audit = format!(
                    "{}\nruntime.backend.executed backend={:?} tokens={} latency_ms={} execution_hash={}",
                    audit_event,
                    self.backend.kind(),
                    res.tokens_generated,
                    latency_ms,
                    execution_hash,
                );
                RuntimeResponse {
                    generated_text: res.generated_text,
                    tokens_generated: res.tokens_generated,
                    processing_time_ms: res.processing_time_ms,
                    backend_kind: self.backend.kind(),
                    backend_metadata: res.metadata,
                    heg_plan_id: Some(heg_plan.graph_id.clone()),
                    placement_summary: heg_plan.decisions.clone(),
                    audit_event: Some(backend_audit),
                }
            }
            Err(e) => {
                crate::metrics::record_runtime_execution(false, 0, latency_ms);
                return Err(e);
            }
        };

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.total_tokens_generated += response.tokens_generated as u64;
        stats.total_processing_time_ms += response.processing_time_ms;
        stats.last_heg_plan_id = response.heg_plan_id.clone();
        stats.last_request_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(response)
    }

    pub async fn get_stats(&self) -> RuntimeStats {
        self.stats.read().await.clone()
    }
    pub async fn is_ready(&self) -> bool {
        true
    }

    pub fn plan_execution_graph(&self, request: &RuntimeRequest) -> Result<HegPlan> {
        self.plan_execution_graph_with_policy(request, &HegPolicy::default())
    }

    pub fn plan_execution_graph_with_policy(
        &self,
        request: &RuntimeRequest,
        policy: &HegPolicy,
    ) -> Result<HegPlan> {
        heg::plan_request(request, policy)
    }
}

fn runtime_execution_hash(
    backend_kind: backend::RuntimeBackendKind,
    generated_text: &str,
    tokens_generated: u32,
    metadata: &HashMap<String, String>,
) -> String {
    let mut pairs: Vec<_> = metadata.iter().collect();
    pairs.sort_by(|(left, _), (right, _)| left.cmp(right));
    let mut payload = format!(
        "backend={:?}\ntokens={}\ntext={}\n",
        backend_kind, tokens_generated, generated_text
    );
    for (key, value) in pairs {
        payload.push_str(key);
        payload.push('=');
        payload.push_str(value);
        payload.push('\n');
    }
    sha256_hex(payload.as_bytes())
}
