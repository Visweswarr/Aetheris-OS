//! Current-contract runtime tests for AI Core.

use std::collections::HashMap;
use std::process::Command;

use aetheris_ai_core::metrics::{AiCoreMetricsCollector, MetricsConfig, PrivacyMode};
use aetheris_ai_core::runtime::{
    backend::RuntimeBackendKind, AcceleratorKind, HegPolicy, OperatorKind, RuntimeConfig,
    RuntimeManager, RuntimeRequest,
};

fn test_config() -> RuntimeConfig {
    RuntimeConfig {
        model_path: "test_models/".to_string(),
        model_name: "test_model".to_string(),
        device: "cpu".to_string(),
        num_threads: 2,
        context_length: 512,
        batch_size: 1,
        deterministic: true,
        seed: Some(42),
        memory_pool_size: 100 * 1024 * 1024,
        enable_profiling: false,
    }
}

fn request(prompt: &str, workload: Option<&str>) -> RuntimeRequest {
    let mut metadata = HashMap::new();
    if let Some(workload) = workload {
        metadata.insert("workload".to_string(), workload.to_string());
    }
    RuntimeRequest {
        prompt: prompt.to_string(),
        max_tokens: Some(64),
        temperature: Some(0.0),
        stop_sequences: vec!["\n\n".to_string()],
        metadata,
    }
}

#[tokio::test]
async fn runtime_manager_initializes_and_generates_deterministic_response() {
    let mut manager = RuntimeManager::new(test_config()).unwrap();
    manager.initialize().await.unwrap();
    assert!(manager.is_ready().await);

    let response = manager
        .generate_response(&request("What is Polymera?", None))
        .await
        .unwrap();
    assert!(response.generated_text.contains("Deterministic Local Analysis:"));
    assert!(!response.generated_text.contains("Mock response to:"));
    assert_eq!(response.backend_kind, RuntimeBackendKind::DeterministicLocal);
    assert_eq!(
        response
            .backend_metadata
            .get("execution_mode")
            .map(String::as_str),
        Some("deterministic_local")
    );
    assert_eq!(
        response
            .backend_metadata
            .get("remote_execution")
            .map(String::as_str),
        Some("false")
    );
    let plan_id = response.heg_plan_id.clone().unwrap();
    assert!(plan_id.starts_with("heg-"));
    assert_eq!(
        response.backend_metadata.get("heg_plan_id").map(String::as_str),
        Some(plan_id.as_str())
    );
    assert!(response
        .audit_event
        .as_ref()
        .unwrap()
        .contains("runtime.heg.plan_generated"));
    assert!(response
        .audit_event
        .as_ref()
        .unwrap()
        .contains("runtime.backend.executed"));
    assert!(response
        .audit_event
        .as_ref()
        .unwrap()
        .contains("execution_hash="));

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.total_tokens_generated, response.tokens_generated as u64);
    assert!(stats.last_heg_plan_id.unwrap().starts_with("heg-"));
}

#[tokio::test]
async fn runtime_execution_is_byte_identical_for_same_request_and_policy() {
    let manager = RuntimeManager::new(test_config()).unwrap();
    let req = request("What is Polymera?", None);
    let policy = HegPolicy::default();

    let res_a = manager.generate_response_with_policy(&req, &policy).await.unwrap();
    let res_b = manager.generate_response_with_policy(&req, &policy).await.unwrap();

    assert_eq!(res_a.generated_text, res_b.generated_text);
    assert_eq!(res_a.tokens_generated, res_b.tokens_generated);
    assert_eq!(res_a.generated_text.as_bytes(), res_b.generated_text.as_bytes());
    assert_eq!(res_a.backend_metadata, res_b.backend_metadata);
    assert_eq!(res_a.heg_plan_id, res_b.heg_plan_id);
}

#[tokio::test]
async fn heg_plan_id_survives_execution() {
    let manager = RuntimeManager::new(test_config()).unwrap();
    let req = request("Analyze system logs", None);
    let policy = HegPolicy::default();

    let plan = manager.plan_execution_graph_with_policy(&req, &policy).unwrap();
    let plan_id = plan.graph_id.clone();

    let response = manager.generate_response_with_policy(&req, &policy).await.unwrap();
    assert_eq!(response.heg_plan_id.unwrap(), plan_id);
}

#[tokio::test]
async fn payload_dependent_audit_changes() {
    let manager = RuntimeManager::new(test_config()).unwrap();
    let req_a = request("first distinct prompt", None);
    let req_b = request("second completely distinct prompt", None);

    let res_a = manager.generate_response(&req_a).await.unwrap();
    let res_b = manager.generate_response(&req_b).await.unwrap();

    let audit_a = res_a.audit_event.unwrap();
    let audit_b = res_b.audit_event.unwrap();

    assert_ne!(audit_a, audit_b);
}

#[tokio::test]
async fn metrics_reliability() {
    use aetheris_ai_core::metrics::{init_global_metrics_collector, get_global_metrics_collector, default_metrics_config};

    // Initialize global metrics collector if not already initialized
    let _ = init_global_metrics_collector(default_metrics_config());

    let collector = get_global_metrics_collector().expect("global metrics collector should exist");

    let initial_snapshot = collector.get_metrics_snapshot().await.unwrap();
    let initial_executions = initial_snapshot.ai_runtime_backend_executions_total;
    let initial_errors = initial_snapshot.ai_runtime_backend_errors_total;
    let initial_tokens = initial_snapshot.ai_runtime_local_tokens_total;

    let manager = RuntimeManager::new(test_config()).unwrap();

    // Success path execution
    let req_success = request("Hello status please", None);
    let res_success = manager.generate_response(&req_success).await.unwrap();

    let snapshot_after_success = collector.get_metrics_snapshot().await.unwrap();
    assert!(
        snapshot_after_success.ai_runtime_backend_executions_total >= initial_executions + 1
    );
    assert!(
        snapshot_after_success.ai_runtime_backend_errors_total >= initial_errors
    );
    assert!(
        snapshot_after_success.ai_runtime_local_tokens_total >= initial_tokens + res_success.tokens_generated as u64
    );

    // Error path execution
    let req_error = request("   ", None);
    let err = manager.generate_response(&req_error).await.unwrap_err();
    assert!(err.to_string().contains("runtime prompt must not be empty"));

    let snapshot_after_error = collector.get_metrics_snapshot().await.unwrap();
    assert!(
        snapshot_after_error.ai_runtime_backend_executions_total >= initial_executions + 2
    );
    assert!(
        snapshot_after_error.ai_runtime_backend_errors_total >= initial_errors + 1
    );
}

#[test]
fn heg_plan_is_byte_identical_for_same_request_and_policy() {
    let manager = RuntimeManager::new_mock();
    let req = request("summarize a browser page", Some("browser"));
    let policy = HegPolicy::default();

    let plan_a = manager
        .plan_execution_graph_with_policy(&req, &policy)
        .unwrap();
    let plan_b = manager
        .plan_execution_graph_with_policy(&req, &policy)
        .unwrap();

    assert_eq!(serde_json::to_vec(&plan_a).unwrap(), serde_json::to_vec(&plan_b).unwrap());
    assert_eq!(plan_a.deterministic_hash, plan_b.deterministic_hash);
}

#[test]
fn heg_default_routes_prefill_to_npu_and_decode_to_igpu() {
    let manager = RuntimeManager::new_mock();
    let plan = manager
        .plan_execution_graph(&request("answer with local context", None))
        .unwrap();

    assert_eq!(assigned(&plan, OperatorKind::Prefill), AcceleratorKind::Npu);
    assert_eq!(assigned(&plan, OperatorKind::Decode), AcceleratorKind::Igpu);
}

#[test]
fn heg_falls_back_to_cpu_when_accelerators_are_disabled() {
    let manager = RuntimeManager::new_mock();
    let policy = HegPolicy {
        npu_enabled: false,
        igpu_enabled: false,
        ..HegPolicy::default()
    };
    let plan = manager
        .plan_execution_graph_with_policy(&request("answer", None), &policy)
        .unwrap();

    assert_eq!(assigned(&plan, OperatorKind::Prefill), AcceleratorKind::Cpu);
    assert_eq!(assigned(&plan, OperatorKind::Decode), AcceleratorKind::Cpu);
}

#[test]
fn heg_keeps_browser_log_and_tool_work_cpu_local() {
    let manager = RuntimeManager::new_mock();
    let plan = manager
        .plan_execution_graph(&request(
            "browser page log tool: summarize locally",
            Some("browser log tool"),
        ))
        .unwrap();

    assert_eq!(assigned(&plan, OperatorKind::BrowserAssist), AcceleratorKind::Cpu);
    assert_eq!(assigned(&plan, OperatorKind::LogSummary), AcceleratorKind::Cpu);
    assert_eq!(assigned(&plan, OperatorKind::ToolCall), AcceleratorKind::Cpu);
}

#[test]
fn heg_ddr_pressure_cap_moves_decode_to_cpu() {
    let manager = RuntimeManager::new_mock();
    let policy = HegPolicy {
        max_ddr_pressure: 10,
        ..HegPolicy::default()
    };
    let plan = manager
        .plan_execution_graph_with_policy(&request("answer", None), &policy)
        .unwrap();

    assert_eq!(assigned(&plan, OperatorKind::Decode), AcceleratorKind::Cpu);
    assert!(plan
        .decisions
        .iter()
        .any(|decision| decision.reason.contains("ddr pressure")));
}

#[tokio::test]
async fn heg_metrics_increment_from_plan() {
    let collector = AiCoreMetricsCollector::new(MetricsConfig {
        enabled: true,
        collection_interval_secs: 60,
        retention_days: 1,
        enable_ngfs: false,
        ngfs_shard_path: String::new(),
        enable_cloud_export: false,
        cloud_export_endpoint: None,
        privacy_mode: PrivacyMode::LocalOnly,
        max_samples_per_metric: 128,
    });
    let manager = RuntimeManager::new_mock();
    let plan = manager
        .plan_execution_graph(&request("answer", None))
        .unwrap();

    collector.record_heg_plan(&plan);
    let snapshot = collector.get_metrics_snapshot().await.unwrap();
    assert_eq!(snapshot.ai_heg_plans_total, 1);
    assert_eq!(snapshot.ai_heg_prefill_to_npu_total, 1);
    assert_eq!(snapshot.ai_heg_decode_to_igpu_total, 1);
    assert_eq!(
        snapshot.ai_heg_ddr_pressure_score,
        plan.ddr_pressure_score as u64
    );
}

#[test]
fn cli_runtime_run_writes_dashboard_metrics_file() {
    let temp = tempfile::tempdir().unwrap();
    let metrics_path = temp.path().join("ai_metrics.js");
    let exe = std::env::var("CARGO_BIN_EXE_ai_core_cli")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_ai_core_cli").map(str::to_string))
        .expect("CARGO_BIN_EXE_ai_core_cli should be available for integration tests");

    let output = Command::new(exe)
        .arg("runtime-run")
        .arg("--prompt")
        .arg("hello from runtime test")
        .arg("--prefer")
        .arg("cpu")
        .arg("--dashboard-metrics")
        .arg(&metrics_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "runtime-run failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let metrics = std::fs::read_to_string(metrics_path).unwrap();
    let payload = metrics_json(&metrics);
    assert_eq!(payload["ai_runtime_backend_executions_total"], 1);
    assert_eq!(payload["ai_runtime_local_tokens_total"], 14);
    let generated_text = payload["last_heg_plan"]["generated_text"].as_str().unwrap();
    assert!(generated_text.contains("deterministic local assistant"));
    assert!(!generated_text.contains("Mock response to:"));
}

fn metrics_json(text: &str) -> serde_json::Value {
    let start = text.find('{').unwrap();
    let end = text.rfind('}').unwrap();
    serde_json::from_str(&text[start..=end]).unwrap()
}

fn assigned(plan: &aetheris_ai_core::runtime::HegPlan, operator: OperatorKind) -> AcceleratorKind {
    plan.nodes
        .iter()
        .find(|node| node.operator == operator)
        .unwrap()
        .assigned
}
