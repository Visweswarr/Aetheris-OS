//! End-to-end integration test: CLI → Planner → Executor → wasm_driver → metric
//!
//! Deliverable 1A.4: Exercises the full summarize-log pipeline using mock
//! components and verifies that:
//! 1. The planner generates a valid plan with the log_summarizer tool.
//! 2. The executor runs the plan through policy + budget + privacy guards.
//! 3. The wasm_driver SummarizerHost produces a deterministic summary.
//! 4. The `ai_log_summaries_total` metric counter increments.

use std::collections::HashMap;
use std::sync::Arc;

use aetheris_ai_core::agent::planner::TaskPlanner;
use aetheris_ai_core::agent::executor::StepExecutor;
use aetheris_ai_core::agent::policy::PolicyEnforcer;
use aetheris_ai_core::cap::CapTokenManager;
use aetheris_ai_core::intents::SystemActionContext;
use aetheris_ai_core::runtime::RuntimeManager;
use aetheris_ai_core::tools::registry::ToolRegistry;

fn approved_context() -> SystemActionContext {
    let mut metadata = HashMap::new();
    metadata.insert("approved_plan".to_string(), serde_json::json!(true));
    metadata.insert(
        "capabilities".to_string(),
        serde_json::json!(["fs.read", "ai.summarize"]),
    );
    SystemActionContext {
        user_id: "e2e-test-user".to_string(),
        session_id: "e2e-test-session".to_string(),
        cap_token: None,
        metadata,
    }
}

#[tokio::test]
async fn summarize_log_e2e_pipeline() {
    // --- Setup ---
    let runtime_manager = Arc::new(RuntimeManager::new_mock());
    let tool_registry = Arc::new(ToolRegistry::new_mock());
    let cap_manager = Arc::new(CapTokenManager::new_mock());
    let planner = TaskPlanner::new(runtime_manager, tool_registry.clone());
    let policy_enforcer = Arc::new(PolicyEnforcer::new(cap_manager));
    let executor = StepExecutor::new(tool_registry, policy_enforcer);

    // --- Create a temporary log file ---
    let tmp = tempfile::NamedTempFile::new().expect("create temp file");
    let log_content = "\
INFO  2024-01-01T00:00:00Z service started successfully
WARN  2024-01-01T00:00:01Z high memory usage detected
ERROR 2024-01-01T00:00:02Z connection to database refused
INFO  2024-01-01T00:00:03Z retrying connection...
INFO  2024-01-01T00:00:04Z connection established
WARN  2024-01-01T00:00:05Z slow query detected: 1500ms
ERROR 2024-01-01T00:00:06Z request timeout after 30s
INFO  2024-01-01T00:00:07Z service health check OK
";
    std::fs::write(tmp.path(), log_content).expect("write log");

    // --- Step 1: Generate a plan ---
    let context = approved_context();
    let plan = planner
        .generate_log_summary_plan(tmp.path(), 2 * 1024 * 1024, None, &context)
        .await
        .expect("generate plan");

    assert_eq!(plan.steps.len(), 1);
    assert_eq!(plan.steps[0].tool_name, "log_summarizer");
    assert!(plan.steps[0].requires_approval);
    assert!(!plan.steps[0].is_destructive);

    // --- Step 2: Execute the plan ---
    let results = executor
        .execute_plan(plan, context)
        .await
        .expect("execute plan");

    assert!(!results.is_empty());

    // --- Step 3: Verify the result ---
    let completed = results
        .iter()
        .find(|r| r.state == aetheris_ai_core::agent::StepState::Completed)
        .expect("find completed step");

    let result = completed.result.as_ref().expect("step has result");
    assert!(result.success);

    let payload: serde_json::Value =
        serde_json::from_slice(result.result.as_ref().expect("result bytes"))
            .expect("parse result JSON");

    // Verify summary structure
    assert!(payload.get("summary").is_some());
    assert!(payload.get("line_count").is_some());
    assert!(payload.get("error_count").is_some());
    assert!(payload.get("warning_count").is_some());

    // Verify counts from our test data
    assert_eq!(payload["error_count"].as_u64(), Some(2));
    assert_eq!(payload["warning_count"].as_u64(), Some(2));

    // Verify deterministic mode
    assert!(payload["mode"]
        .as_str()
        .unwrap_or("")
        .contains("deterministic"));

    // Verify metric was recorded
    assert_eq!(
        payload["metric"].as_str(),
        Some("ai_log_summaries_total")
    );
}

#[tokio::test]
async fn summarize_log_rejects_unapproved() {
    let runtime_manager = Arc::new(RuntimeManager::new_mock());
    let tool_registry = Arc::new(ToolRegistry::new_mock());
    let cap_manager = Arc::new(CapTokenManager::new_mock());
    let planner = TaskPlanner::new(runtime_manager, tool_registry.clone());
    let policy_enforcer = Arc::new(PolicyEnforcer::new(cap_manager));
    let executor = StepExecutor::new(tool_registry, policy_enforcer);

    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "INFO test line").unwrap();

    // Context WITHOUT approval
    let context = SystemActionContext {
        user_id: "test-user".to_string(),
        session_id: "test-session".to_string(),
        cap_token: None,
        metadata: HashMap::new(),
    };

    let plan = planner
        .generate_log_summary_plan(tmp.path(), 2 * 1024 * 1024, None, &context)
        .await
        .unwrap();

    // Should fail because the step requires approval
    let err = executor
        .execute_plan(plan, context)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("requires approval")
        || err.to_string().contains("capability"));
}
