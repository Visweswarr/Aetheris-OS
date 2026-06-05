//! AI Core Service Telemetry and Metrics Collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::error::Result;
use crate::runtime::{AcceleratorKind, HegPlan, OperatorKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval_secs: u64,
    pub retention_days: u64,
    pub enable_ngfs: bool,
    pub ngfs_shard_path: String,
    pub enable_cloud_export: bool,
    pub cloud_export_endpoint: Option<String>,
    pub privacy_mode: PrivacyMode,
    pub max_samples_per_metric: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyMode {
    Disabled,
    LocalOnly,
    Anonymized,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiCoreMetrics {
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_error: u64,
    pub latency_p50_ms: f64,
    pub latency_p95_ms: f64,
    pub latency_p99_ms: f64,
    pub latency_max_ms: f64,
    pub tokens_per_second: f64,
    pub tokens_total: u64,
    pub tokens_input: u64,
    pub tokens_output: u64,
    pub tool_calls_total: u64,
    pub tool_calls_success: u64,
    pub tool_calls_error: u64,
    pub tool_errors_by_type: HashMap<String, u64>,
    pub model_loads_total: u64,
    pub model_inferences_total: u64,
    pub model_memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub disk_usage_mb: f64,
    pub active_sessions: u64,
    pub sessions_total: u64,
    pub session_duration_avg_secs: f64,
    pub ai_log_summaries_total: u64,
    pub ai_plans_generated_total: u64,
    pub ai_ltm_events_total: u64,
    pub ai_browser_summaries_total: u64,
    pub ai_capability_denials_total: u64,
    pub ai_heg_plans_total: u64,
    pub ai_heg_prefill_to_npu_total: u64,
    pub ai_heg_decode_to_igpu_total: u64,
    pub ai_heg_ddr_pressure_score: u64,
    pub ai_llm_calls_total: u64,
    pub ai_hitl_rejects_total: u64,
    pub ai_hitl_modifies_total: u64,
    pub ai_budget_exhausted_total: u64,
    pub ai_runtime_backend_executions_total: u64,
    pub ai_runtime_backend_errors_total: u64,
    pub ai_runtime_local_tokens_total: u64,
    pub ai_runtime_backend_last_latency_ms: u64,
    pub ai_runtime_model_attempts_total: u64,
    pub ai_runtime_model_success_total: u64,
    pub ai_runtime_model_failures_total: u64,
    pub ai_runtime_model_last_latency_ms: u64,
    pub ai_runtime_model_tokens_total: u64,
    pub timestamp: u64,
}

pub struct AiCoreMetricsCollector {
    config: MetricsConfig,
    requests_total: AtomicU64,
    requests_success: AtomicU64,
    requests_error: AtomicU64,
    tool_calls_total: AtomicU64,
    tool_calls_success: AtomicU64,
    tool_calls_error: AtomicU64,
    tokens_total: AtomicU64,
    tokens_input: AtomicU64,
    tokens_output: AtomicU64,
    model_loads_total: AtomicU64,
    model_inferences_total: AtomicU64,
    sessions_total: AtomicU64,
    ai_log_summaries_total: AtomicU64,
    ai_plans_generated_total: AtomicU64,
    ai_ltm_events_total: AtomicU64,
    ai_browser_summaries_total: AtomicU64,
    ai_capability_denials_total: AtomicU64,
    ai_heg_plans_total: AtomicU64,
    ai_heg_prefill_to_npu_total: AtomicU64,
    ai_heg_decode_to_igpu_total: AtomicU64,
    ai_heg_ddr_pressure_score: AtomicU64,
    ai_llm_calls_total: AtomicU64,
    ai_hitl_rejects_total: AtomicU64,
    ai_hitl_modifies_total: AtomicU64,
    ai_budget_exhausted_total: AtomicU64,
    ai_runtime_backend_executions_total: AtomicU64,
    ai_runtime_backend_errors_total: AtomicU64,
    ai_runtime_local_tokens_total: AtomicU64,
    ai_runtime_backend_last_latency_ms: AtomicU64,
    ai_runtime_model_attempts_total: AtomicU64,
    ai_runtime_model_success_total: AtomicU64,
    ai_runtime_model_failures_total: AtomicU64,
    ai_runtime_model_last_latency_ms: AtomicU64,
    ai_runtime_model_tokens_total: AtomicU64,
    latency_samples: Arc<RwLock<Vec<f64>>>,
    tool_errors: Arc<RwLock<HashMap<String, AtomicU64>>>,
}

#[derive(Default)]
pub struct MockNgfsClient {
    stored: RwLock<HashMap<String, AiCoreMetrics>>,
}

impl MockNgfsClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn store_metrics(&self, metrics: &AiCoreMetrics) -> Result<String> {
        let shard_id = format!("metrics-{}", metrics.timestamp);
        self.stored
            .write()
            .await
            .insert(shard_id.clone(), metrics.clone());
        Ok(shard_id)
    }

    pub async fn retrieve_metrics(&self, shard_id: &str) -> Result<AiCoreMetrics> {
        self.stored
            .read()
            .await
            .get(shard_id)
            .cloned()
            .ok_or_else(|| crate::error::AiCoreError::NotFound(shard_id.to_string()))
    }

    pub async fn list_metrics_shards(&self) -> Result<Vec<String>> {
        Ok(self.stored.read().await.keys().cloned().collect())
    }
}

pub fn default_metrics_config() -> MetricsConfig {
    MetricsConfig {
        enabled: true,
        collection_interval_secs: 60,
        retention_days: 30,
        enable_ngfs: false,
        ngfs_shard_path: "~/.aetheris/metrics/ai_core".to_string(),
        enable_cloud_export: false,
        cloud_export_endpoint: None,
        privacy_mode: PrivacyMode::LocalOnly,
        max_samples_per_metric: 10000,
    }
}

impl AiCoreMetricsCollector {
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_error: AtomicU64::new(0),
            tool_calls_total: AtomicU64::new(0),
            tool_calls_success: AtomicU64::new(0),
            tool_calls_error: AtomicU64::new(0),
            tokens_total: AtomicU64::new(0),
            tokens_input: AtomicU64::new(0),
            tokens_output: AtomicU64::new(0),
            model_loads_total: AtomicU64::new(0),
            model_inferences_total: AtomicU64::new(0),
            sessions_total: AtomicU64::new(0),
            ai_log_summaries_total: AtomicU64::new(0),
            ai_plans_generated_total: AtomicU64::new(0),
            ai_ltm_events_total: AtomicU64::new(0),
            ai_browser_summaries_total: AtomicU64::new(0),
            ai_capability_denials_total: AtomicU64::new(0),
            ai_heg_plans_total: AtomicU64::new(0),
            ai_heg_prefill_to_npu_total: AtomicU64::new(0),
            ai_heg_decode_to_igpu_total: AtomicU64::new(0),
            ai_heg_ddr_pressure_score: AtomicU64::new(0),
            ai_llm_calls_total: AtomicU64::new(0),
            ai_hitl_rejects_total: AtomicU64::new(0),
            ai_hitl_modifies_total: AtomicU64::new(0),
            ai_budget_exhausted_total: AtomicU64::new(0),
            ai_runtime_backend_executions_total: AtomicU64::new(0),
            ai_runtime_backend_errors_total: AtomicU64::new(0),
            ai_runtime_local_tokens_total: AtomicU64::new(0),
            ai_runtime_backend_last_latency_ms: AtomicU64::new(0),
            ai_runtime_model_attempts_total: AtomicU64::new(0),
            ai_runtime_model_success_total: AtomicU64::new(0),
            ai_runtime_model_failures_total: AtomicU64::new(0),
            ai_runtime_model_last_latency_ms: AtomicU64::new(0),
            ai_runtime_model_tokens_total: AtomicU64::new(0),
            latency_samples: Arc::new(RwLock::new(Vec::new())),
            tool_errors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        // Starting metrics collection
        Ok(())
    }

    pub async fn stop(&mut self) { /* Stopping metrics collection */
    }

    pub fn record_request(&self, success: bool, latency_ms: f64) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        if success {
            self.requests_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.requests_error.fetch_add(1, Ordering::Relaxed);
        }
        if let Ok(mut samples) = self.latency_samples.try_write() {
            samples.push(latency_ms);
        } else {
            let samples = self.latency_samples.clone();
            tokio::spawn(async move {
                samples.write().await.push(latency_ms);
            });
        }
    }

    pub fn record_tokens(&self, input_tokens: u64, output_tokens: u64) {
        self.tokens_total
            .fetch_add(input_tokens + output_tokens, Ordering::Relaxed);
        self.tokens_input.fetch_add(input_tokens, Ordering::Relaxed);
        self.tokens_output
            .fetch_add(output_tokens, Ordering::Relaxed);
    }

    pub fn record_tool_call(&self, success: bool, _tool_name: Option<&str>) {
        self.tool_calls_total.fetch_add(1, Ordering::Relaxed);
        if success {
            self.tool_calls_success.fetch_add(1, Ordering::Relaxed);
        } else {
            self.tool_calls_error.fetch_add(1, Ordering::Relaxed);
            if let Some(tool_name) = _tool_name {
                if let Ok(mut errors) = self.tool_errors.try_write() {
                    errors
                        .entry(tool_name.to_string())
                        .or_insert_with(|| AtomicU64::new(0))
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    pub fn record_llm_call(&self) {
        self.ai_llm_calls_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_hitl_reject(&self) {
        self.ai_hitl_rejects_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_hitl_modify(&self) {
        self.ai_hitl_modifies_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_budget_exhausted(&self) {
        self.ai_budget_exhausted_total
            .fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_runtime_execution(&self, success: bool, tokens: u64, latency_ms: u64) {
        self.ai_runtime_backend_executions_total.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.ai_runtime_backend_errors_total.fetch_add(1, Ordering::Relaxed);
        }
        self.ai_runtime_local_tokens_total.fetch_add(tokens, Ordering::Relaxed);
        self.ai_runtime_backend_last_latency_ms.store(latency_ms, Ordering::Relaxed);
    }

    pub fn record_runtime_model_execution(&self, success: bool, tokens: u64, latency_ms: u64) {
        self.ai_runtime_model_attempts_total.fetch_add(1, Ordering::Relaxed);
        if success {
            self.ai_runtime_model_success_total.fetch_add(1, Ordering::Relaxed);
            self.ai_runtime_model_tokens_total.fetch_add(tokens, Ordering::Relaxed);
        } else {
            self.ai_runtime_model_failures_total.fetch_add(1, Ordering::Relaxed);
        }
        self.ai_runtime_model_last_latency_ms.store(latency_ms, Ordering::Relaxed);
    }

    pub fn record_model_load(&self) {
        self.model_loads_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_model_inference(&self) {
        self.model_inferences_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_session(&self) {
        self.sessions_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_log_summary(&self) {
        self.ai_log_summaries_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_ai_plan_generated(&self) {
        self.ai_plans_generated_total
            .fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_ltm_event(&self) {
        self.ai_ltm_events_total.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_browser_summary(&self) {
        self.ai_browser_summaries_total
            .fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_capability_denial(&self) {
        self.ai_capability_denials_total
            .fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_heg_plan(&self, plan: &HegPlan) {
        self.ai_heg_plans_total.fetch_add(1, Ordering::Relaxed);
        self.ai_heg_ddr_pressure_score
            .store(plan.ddr_pressure_score as u64, Ordering::Relaxed);
        for node in &plan.nodes {
            if node.operator == OperatorKind::Prefill && node.assigned == AcceleratorKind::Npu {
                self.ai_heg_prefill_to_npu_total
                    .fetch_add(1, Ordering::Relaxed);
            }
            if node.operator == OperatorKind::Decode && node.assigned == AcceleratorKind::Igpu {
                self.ai_heg_decode_to_igpu_total
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub async fn get_metrics_snapshot(&self) -> Result<AiCoreMetrics> {
        let latency_samples = self.latency_samples.read().await;
        let mut sorted = latency_samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p50 = percentile(&sorted, 0.50);
        let p95 = percentile(&sorted, 0.95);
        let p99 = percentile(&sorted, 0.99);
        let max = sorted.last().copied().unwrap_or(0.0);

        let tool_errors_by_type = self
            .tool_errors
            .read()
            .await
            .iter()
            .map(|(name, count)| (name.clone(), count.load(Ordering::Relaxed)))
            .collect();
        Ok(AiCoreMetrics {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            requests_error: self.requests_error.load(Ordering::Relaxed),
            latency_p50_ms: p50,
            latency_p95_ms: p95,
            latency_p99_ms: p99,
            latency_max_ms: max,
            tokens_per_second: if max > 0.0 {
                self.tokens_total.load(Ordering::Relaxed) as f64 / (max / 1000.0)
            } else {
                self.tokens_total.load(Ordering::Relaxed) as f64
            },
            tokens_total: self.tokens_total.load(Ordering::Relaxed),
            tokens_input: self.tokens_input.load(Ordering::Relaxed),
            tokens_output: self.tokens_output.load(Ordering::Relaxed),
            tool_calls_total: self.tool_calls_total.load(Ordering::Relaxed),
            tool_calls_success: self.tool_calls_success.load(Ordering::Relaxed),
            tool_calls_error: self.tool_calls_error.load(Ordering::Relaxed),
            tool_errors_by_type,
            model_loads_total: self.model_loads_total.load(Ordering::Relaxed),
            model_inferences_total: self.model_inferences_total.load(Ordering::Relaxed),
            model_memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            disk_usage_mb: 0.0,
            active_sessions: 0,
            sessions_total: self.sessions_total.load(Ordering::Relaxed),
            session_duration_avg_secs: 0.0,
            ai_log_summaries_total: self.ai_log_summaries_total.load(Ordering::Relaxed),
            ai_plans_generated_total: self.ai_plans_generated_total.load(Ordering::Relaxed),
            ai_ltm_events_total: self.ai_ltm_events_total.load(Ordering::Relaxed),
            ai_browser_summaries_total: self.ai_browser_summaries_total.load(Ordering::Relaxed),
            ai_capability_denials_total: self
                .ai_capability_denials_total
                .load(Ordering::Relaxed),
            ai_heg_plans_total: self.ai_heg_plans_total.load(Ordering::Relaxed),
            ai_heg_prefill_to_npu_total: self
                .ai_heg_prefill_to_npu_total
                .load(Ordering::Relaxed),
            ai_heg_decode_to_igpu_total: self
                .ai_heg_decode_to_igpu_total
                .load(Ordering::Relaxed),
            ai_heg_ddr_pressure_score: self.ai_heg_ddr_pressure_score.load(Ordering::Relaxed),
            ai_llm_calls_total: self.ai_llm_calls_total.load(Ordering::Relaxed),
            ai_hitl_rejects_total: self.ai_hitl_rejects_total.load(Ordering::Relaxed),
            ai_hitl_modifies_total: self.ai_hitl_modifies_total.load(Ordering::Relaxed),
            ai_budget_exhausted_total: self.ai_budget_exhausted_total.load(Ordering::Relaxed),
            ai_runtime_backend_executions_total: self.ai_runtime_backend_executions_total.load(Ordering::Relaxed),
            ai_runtime_backend_errors_total: self.ai_runtime_backend_errors_total.load(Ordering::Relaxed),
            ai_runtime_local_tokens_total: self.ai_runtime_local_tokens_total.load(Ordering::Relaxed),
            ai_runtime_backend_last_latency_ms: self.ai_runtime_backend_last_latency_ms.load(Ordering::Relaxed),
            ai_runtime_model_attempts_total: self.ai_runtime_model_attempts_total.load(Ordering::Relaxed),
            ai_runtime_model_success_total: self.ai_runtime_model_success_total.load(Ordering::Relaxed),
            ai_runtime_model_failures_total: self.ai_runtime_model_failures_total.load(Ordering::Relaxed),
            ai_runtime_model_last_latency_ms: self.ai_runtime_model_last_latency_ms.load(Ordering::Relaxed),
            ai_runtime_model_tokens_total: self.ai_runtime_model_tokens_total.load(Ordering::Relaxed),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    pub fn anonymize_metrics(&self, metrics: &AiCoreMetrics) -> AiCoreMetrics {
        let mut anonymized = metrics.clone();
        if !matches!(self.config.privacy_mode, PrivacyMode::Full) {
            anonymized.tool_errors_by_type.clear();
        }
        anonymized
    }

    pub fn set_ngfs_client(&mut self, _client: Arc<MockNgfsClient>) {}

    pub async fn cleanup_old_samples(&self) -> Result<()> {
        let mut samples = self.latency_samples.write().await;
        if samples.len() > self.config.max_samples_per_metric {
            let keep_from = samples.len() - self.config.max_samples_per_metric;
            samples.drain(0..keep_from);
        }
        Ok(())
    }
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = (q.clamp(0.0, 1.0) * sorted.len() as f64 - 1.0)
        .clamp(0.0, (sorted.len() - 1) as f64);
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let weight = rank - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}

static GLOBAL_METRICS_COLLECTOR: OnceLock<Arc<AiCoreMetricsCollector>> = OnceLock::new();

pub fn init_global_metrics_collector(config: MetricsConfig) -> Result<Arc<AiCoreMetricsCollector>> {
    let collector = Arc::new(AiCoreMetricsCollector::new(config));
    let _ = GLOBAL_METRICS_COLLECTOR.set(collector.clone());
    Ok(collector)
}

pub fn get_global_metrics_collector() -> Option<Arc<AiCoreMetricsCollector>> {
    GLOBAL_METRICS_COLLECTOR.get().cloned()
}

pub fn record_request(success: bool, latency_ms: f64) {
    if let Some(c) = get_global_metrics_collector() {
        c.record_request(success, latency_ms);
    }
}

pub fn record_tokens(input_tokens: u64, output_tokens: u64) {
    if let Some(c) = get_global_metrics_collector() {
        c.record_tokens(input_tokens, output_tokens);
    }
}

pub fn record_tool_call(success: bool, tool_name: Option<&str>) {
    if let Some(c) = get_global_metrics_collector() {
        c.record_tool_call(success, tool_name);
    }
}

pub fn record_model_load() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_model_load();
    }
}

pub fn record_model_inference() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_model_inference();
    }
}

pub fn record_session() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_session();
    }
}

pub fn record_log_summary() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_log_summary();
    }
}

pub fn record_ai_plan_generated() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_ai_plan_generated();
    }
}

pub fn record_ltm_event() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_ltm_event();
    }
}

pub fn record_browser_summary() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_browser_summary();
    }
}

pub fn record_capability_denial() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_capability_denial();
    }
}

pub fn record_heg_plan(plan: &HegPlan) {
    if let Some(c) = get_global_metrics_collector() {
        c.record_heg_plan(plan);
    }
}

pub fn record_llm_call() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_llm_call();
    }
}

pub fn record_hitl_reject() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_hitl_reject();
    }
}

pub fn record_hitl_modify() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_hitl_modify();
    }
}

pub fn record_budget_exhausted() {
    if let Some(c) = get_global_metrics_collector() {
        c.record_budget_exhausted();
    }
}

pub fn record_runtime_execution(success: bool, tokens: u64, latency_ms: u64) {
    if let Some(c) = get_global_metrics_collector() {
        c.record_runtime_execution(success, tokens, latency_ms);
    }
}

pub fn record_runtime_model_execution(success: bool, tokens: u64, latency_ms: u64) {
    if let Some(c) = get_global_metrics_collector() {
        c.record_runtime_model_execution(success, tokens, latency_ms);
    }
}
