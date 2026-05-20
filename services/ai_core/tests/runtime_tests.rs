//! Current-contract runtime tests for AI Core.

use std::collections::HashMap;

use aetheris_ai_core::metrics::{AiCoreMetricsCollector, MetricsConfig, PrivacyMode};
use aetheris_ai_core::runtime::{
    AcceleratorKind, HegPolicy, OperatorKind, RuntimeConfig, RuntimeManager, RuntimeRequest,
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
async fn runtime_manager_initializes_and_generates_mock_response() {
    let mut manager = RuntimeManager::new(test_config()).unwrap();
    manager.initialize().await.unwrap();
    assert!(manager.is_ready().await);

    let response = manager
        .generate_response(&request("What is Polymera?", None))
        .await
        .unwrap();
    assert!(response.generated_text.contains("Mock response to:"));
    assert_eq!(response.tokens_generated, 10);
    assert!(response.heg_plan_id.unwrap().starts_with("heg-"));
    assert!(response
        .audit_event
        .unwrap()
        .contains("runtime.heg.plan_generated"));

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.total_tokens_generated, 10);
    assert!(stats.last_heg_plan_id.unwrap().starts_with("heg-"));
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

fn assigned(plan: &aetheris_ai_core::runtime::HegPlan, operator: OperatorKind) -> AcceleratorKind {
    plan.nodes
        .iter()
        .find(|node| node.operator == operator)
        .unwrap()
        .assigned
}
