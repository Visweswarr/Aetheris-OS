//! Phase 6 runnable proof tests.
//!
//! These tests back `scripts/phase6-demo.ps1`: HITL reject/modify, budget
//! denial/refund, suspend/resume state replay, and divergence detection.

use std::collections::HashMap;
use std::sync::Arc;

use aetheris_ai_core::agent::{
    policy::{HitlDecision, PolicyEnforcer},
    StepExecutionContext, StepExecutor, StepState, TaskPlan, TaskStep,
};
use aetheris_ai_core::budget::{BudgetEngine, BudgetLimits};
use aetheris_ai_core::cap::CapTokenManager;
use aetheris_ai_core::contracts::{BudgetRequest, Priority, ReplayTolerance};
use aetheris_ai_core::intents::SystemActionContext;
use aetheris_ai_core::replay::ReplayRecorder;
use aetheris_ai_core::tools::{ToolRegistry, ToolResult};

fn demo_step() -> TaskStep {
    TaskStep {
        step_id: "phase6-step-001".to_string(),
        description: "phase 6 proof step".to_string(),
        tool_name: "echo".to_string(),
        parameters: serde_json::json!({"message": "phase 6 proof"}),
        required_capabilities: Vec::new(),
        requires_approval: true,
        is_destructive: true,
        depends_on: Vec::new(),
        estimated_time_secs: 1,
    }
}

fn demo_plan(step: TaskStep) -> TaskPlan {
    TaskPlan {
        plan_id: "phase6-demo-plan".to_string(),
        original_intent: "prove phase 6 runtime controls".to_string(),
        steps: vec![step],
        total_estimated_time_secs: 1,
        metadata: HashMap::new(),
        created_at: "0".to_string(),
    }
}

fn demo_context() -> SystemActionContext {
    let mut metadata = HashMap::new();
    metadata.insert("approved_plan".to_string(), serde_json::json!(true));
    SystemActionContext {
        user_id: "phase6-demo-user".to_string(),
        session_id: "phase6-demo-session".to_string(),
        cap_token: None,
        metadata,
    }
}

fn demo_tool_result() -> ToolResult {
    ToolResult {
        tool_id: "echo".to_string(),
        success: true,
        result: Some(br#"{"ok":true}"#.to_vec()),
        error_message: None,
        execution_time_ms: 1,
        metadata: HashMap::new(),
    }
}

#[test]
fn phase6_hitl_reject_and_modify_are_explicit() {
    let enforcer = PolicyEnforcer::new(Arc::new(CapTokenManager::new_mock()));
    let step = demo_step();

    let rejected = enforcer
        .handle_hitl_decision(
            &step,
            &HitlDecision::Reject {
                reason: "operator denied file access".to_string(),
            },
        )
        .expect("reject returns a blocking decision");
    assert!(!rejected);

    let modified = enforcer
        .handle_hitl_decision(
            &step,
            &HitlDecision::Modify {
                feedback: "summarize metadata only".to_string(),
            },
        )
        .expect_err("modify preserves original and returns feedback for re-plan");
    assert_eq!(modified.original_step, step);
    assert_eq!(modified.human_feedback, "summarize metadata only");
}

#[tokio::test]
async fn phase6_budget_denial_and_refund_are_observable() {
    let engine = BudgetEngine::new(BudgetLimits {
        max_cpu_percent: 100,
        max_memory_mb: 64,
        max_power_mw: 1_000,
        max_concurrent: 1,
    });

    let granted = engine
        .reserve(BudgetRequest {
            request_id: "phase6-ok".to_string(),
            priority: Priority::Normal,
            cpu_percent: 10,
            memory_mb: 32,
            power_mw: 100,
            expected_duration_ms: 10,
        })
        .await;
    assert!(granted.granted);
    assert_eq!(engine.report().await.active_reservations, 1);

    assert!(engine.release_or_refund("phase6-ok", false).await);
    assert_eq!(engine.report().await.active_reservations, 0);

    let denied = engine
        .reserve(BudgetRequest {
            request_id: "phase6-too-large".to_string(),
            priority: Priority::Normal,
            cpu_percent: 10,
            memory_mb: 128,
            power_mw: 100,
            expected_duration_ms: 10,
        })
        .await;
    assert!(!denied.granted);
    assert!(denied.reason.contains("memory budget"));
}

#[tokio::test]
async fn phase6_persisted_task_state_resumes_cached_step() {
    let temp = tempfile::tempdir().expect("temp task home");
    let previous_userprofile = std::env::var("USERPROFILE").ok();
    std::env::set_var("USERPROFILE", temp.path());

    let step = demo_step();
    let plan = demo_plan(step.clone());
    let context = demo_context();
    let result = demo_tool_result();

    let mut completed = HashMap::new();
    completed.insert(
        step.step_id.clone(),
        StepExecutionContext {
            step,
            state: StepState::Completed,
            started_at: None,
            completed_at: None,
            error: None,
            result: Some(result.clone()),
            retry_count: 0,
        },
    );

    let state_path = StepExecutor::persist_state(
        &plan,
        &completed,
        "phase6-step-001",
        &context,
    )
    .expect("persist state");
    assert!(state_path.exists());

    let loaded = StepExecutor::load_persisted_state("phase6-demo-plan")
        .expect("load persisted state");
    let executor = StepExecutor::new(
        Arc::new(ToolRegistry::new_mock()),
        Arc::new(PolicyEnforcer::new(Arc::new(CapTokenManager::new_mock()))),
    );
    let resumed = executor.resume_task(loaded).await.expect("resume task");
    assert_eq!(resumed.len(), 1);
    assert_eq!(resumed[0].state, StepState::Completed);
    assert_eq!(resumed[0].result.as_ref().unwrap().tool_id, result.tool_id);

    if let Some(value) = previous_userprofile {
        std::env::set_var("USERPROFILE", value);
    }
}

#[tokio::test]
async fn phase6_replay_divergence_is_detected() {
    let expected_recorder = ReplayRecorder::new();
    expected_recorder
        .record("tool.call", serde_json::json!({"tool": "echo", "args": "safe"}))
        .await
        .unwrap();
    let expected = expected_recorder
        .run("expected", ReplayTolerance::default())
        .await;

    let actual_recorder = ReplayRecorder::new();
    actual_recorder
        .record("tool.call", serde_json::json!({"tool": "echo", "args": "changed"}))
        .await
        .unwrap();

    let divergence = actual_recorder
        .resume_from_log(&expected.records)
        .await
        .expect_err("changed replay payload must diverge");
    assert_eq!(divergence.index, 0);
    assert_eq!(divergence.expected_event, "tool.call");
    assert_eq!(divergence.actual_event, "tool.call");
    assert_ne!(divergence.expected_hash, divergence.actual_hash);
}
