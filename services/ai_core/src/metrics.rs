//! AI Core Service Telemetry and Metrics Collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use crate::error::Result;

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
    latency_samples: Arc<RwLock<Vec<f64>>>,
    tool_errors: Arc<RwLock<HashMap<String, AtomicU64>>>,
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
        let samples = self.latency_samples.clone();
        tokio::spawn(async move {
            samples.write().await.push(latency_ms);
        });
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
        }
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

    pub async fn get_metrics_snapshot(&self) -> Result<AiCoreMetrics> {
        let latency_samples = self.latency_samples.read().await;
        let mut sorted = latency_samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let p50 = sorted
            .get((sorted.len() as f64 * 0.5) as usize)
            .copied()
            .unwrap_or(0.0);
        let p95 = sorted
            .get((sorted.len() as f64 * 0.95) as usize)
            .copied()
            .unwrap_or(0.0);
        let p99 = sorted
            .get((sorted.len() as f64 * 0.99) as usize)
            .copied()
            .unwrap_or(0.0);
        let max = sorted.last().copied().unwrap_or(0.0);

        Ok(AiCoreMetrics {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            requests_error: self.requests_error.load(Ordering::Relaxed),
            latency_p50_ms: p50,
            latency_p95_ms: p95,
            latency_p99_ms: p99,
            latency_max_ms: max,
            tokens_per_second: 0.0,
            tokens_total: self.tokens_total.load(Ordering::Relaxed),
            tokens_input: self.tokens_input.load(Ordering::Relaxed),
            tokens_output: self.tokens_output.load(Ordering::Relaxed),
            tool_calls_total: self.tool_calls_total.load(Ordering::Relaxed),
            tool_calls_success: self.tool_calls_success.load(Ordering::Relaxed),
            tool_calls_error: self.tool_calls_error.load(Ordering::Relaxed),
            tool_errors_by_type: HashMap::new(),
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
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
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
