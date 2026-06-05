//! Unit tests for plan visualization and approval UI with plan store integration

use std::fs;
use tempfile::TempDir;
use aetheris_ai::plan_ui::{UiPlan, UiStep, approve_plan, store_plan_for_ui, get_plan_for_ui, list_plans_for_ui, render_plan_cli, init_plan_store};
use aetheris_ai::plan_store::{PlanStore, ListQuery};
use aetheris_ai::planner::{Plan, Step, StepPriority, StepStatus};

fn create_test_plan(id: &str, name: &str) -> Plan {
    Plan {
        id: id.to_string(),
        name: name.to_string(),
        description: format!("A test plan for {}", name),
        user_goal: format!("Test the plan functionality for {}", name),
        version: 1,
        priority: StepPriority::Normal,
        status: StepStatus::Pending,
        steps: vec![
            Step {
                id: format!("{}-step-1", id),
                name: "Initialize".to_string(),
                description: "Initialize the test".to_string(),
                step_type: "action".to_string(),
                priority: StepPriority::Normal,
                status: StepStatus::Pending,
                dependencies: vec![],
                estimated_duration_secs: Some(10),
                tool_id: None,
                expected_outputs: vec![],
                sub_steps: vec![],
            },
        ],
        tags: vec!["test".to_string()],
        creator: "test-user".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        estimated_duration_secs: 10,
        actual_duration_secs: None,
        progress: 0.0,
    }
}

fn create_test_plan() -> Plan {
    create_test_plan("test-plan-123", "Test Plan")
}

#[test]
fn test_ui_plan_from_plan() {
    let plan = create_test_plan();
    let ui_plan = UiPlan::from_plan(&plan);
    
    assert_eq!(ui_plan.id, plan.id);
    assert_eq!(ui_plan.steps.len(), 1);
    assert_eq!(ui_plan.steps[0].name, "Initialize");
    assert!(!ui_plan.approved);
    assert!(ui_plan.approved_by.is_none());
}

#[test]
fn test_store_and_get_plan() {
    let plan = create_test_plan();
    store_plan_for_ui(&plan);
    
    let ui_plan = get_plan_for_ui(&plan.id).unwrap();
    assert_eq!(ui_plan.id, plan.id);
    assert_eq!(ui_plan.metadata.get("name"), Some(&plan.name));
}

#[test]
fn test_approve_plan_idempotent() {
    let plan = create_test_plan();
    store_plan_for_ui(&plan);
    
    let user_id = "test-user";
    let approved1 = approve_plan(&plan.id, user_id).unwrap();
    let approved2 = approve_plan(&plan.id, user_id).unwrap();
    
    assert_eq!(approved1.id, approved2.id);
    assert!(approved1.approved);
    assert!(approved2.approved);
    assert_eq!(approved1.approved_by, approved2.approved_by);
    assert_eq!(approved1.approved_at, approved2.approved_at);
}

#[test]
fn test_list_plans_for_ui() {
    let plan1 = create_test_plan("plan-1", "Plan 1");
    let plan2 = create_test_plan("plan-2", "Plan 2");
    
    store_plan_for_ui(&plan1);
    store_plan_for_ui(&plan2);
    
    let plans = list_plans_for_ui();
    assert!(plans.len() >= 2);
    
    let plan_ids: Vec<String> = plans.iter().map(|p| p.id.clone()).collect();
    assert!(plan_ids.contains(&plan1.id));
    assert!(plan_ids.contains(&plan2.id));
}

#[test]
fn test_render_plan_cli() {
    let plan = create_test_plan("render-test", "Render Test Plan");
    store_plan_for_ui(&plan);
    
    let ui_plan = get_plan_for_ui(&plan.id).unwrap();
    let rendered = render_plan_cli(&ui_plan);
    
    assert!(rendered.contains(&plan.id));
    assert!(rendered.contains(&plan.name));
    assert!(rendered.contains(&plan.description));
    assert!(rendered.contains("PENDING"));
    assert!(rendered.contains("Steps:"));
}

#[test]
fn test_ui_plan_metadata() {
    let plan = create_test_plan("metadata-test", "Metadata Test");
    store_plan_for_ui(&plan);
    
    let ui_plan = get_plan_for_ui(&plan.id).unwrap();
    
    assert_eq!(ui_plan.metadata.get("name"), Some(&plan.name));
    assert_eq!(ui_plan.metadata.get("description"), Some(&plan.description));
    assert_eq!(ui_plan.metadata.get("user_goal"), Some(&plan.user_goal));
    assert_eq!(ui_plan.metadata.get("creator"), Some(&plan.creator));
}

#[test]
fn test_approval_logging() {
    let plan = create_test_plan("log-test", "Log Test Plan");
    store_plan_for_ui(&plan);
    
    let user_id = "test-approver";
    let _ = approve_plan(&plan.id, user_id);
    
    // Test passes if no panic occurs (logging is best-effort)
    // In a real test environment, you would verify the log file contents
}

/// Integration test with plan store
#[tokio::test]
async fn test_plan_store_integration() {
    let temp_dir = TempDir::new().unwrap();
    let plan_store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    let plan = create_test_plan("integration-test", "Integration Test");
    
    // Store plan using the plan store directly
    let record = plan_store.store_plan(&plan, "creator").await.unwrap();
    
    // Verify it can be retrieved by UI functions (once proper integration is complete)
    // For now, we test the plan store directly
    let retrieved = plan_store.get_plan_by_id(&record.metadata.plan_id).await.unwrap();
    assert!(retrieved.is_some());
    
    let retrieved_record = retrieved.unwrap();
    assert_eq!(retrieved_record.plan.name, plan.name);
    assert_eq!(retrieved_record.metadata.user_id, "creator");
}

/// Test pagination with plan store
#[tokio::test]
async fn test_pagination_integration() {
    let temp_dir = TempDir::new().unwrap();
    let plan_store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Create multiple plans
    for i in 1..=15 {
        let plan = create_test_plan(&format!("page-test-{}", i), &format!("Page Test {}", i));
        plan_store.store_plan(&plan, "creator").await.unwrap();
    }
    
    // Test pagination
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 5,
        offset: 0,
    };
    let page1 = plan_store.list_plans(&query).await.unwrap();
    assert_eq!(page1.plans.len(), 5);
    assert!(page1.has_more);
    assert_eq!(page1.total_count, 15);
    
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 5,
        offset: 5,
    };
    let page2 = plan_store.list_plans(&query).await.unwrap();
    assert_eq!(page2.plans.len(), 5);
    assert!(page2.has_more);
    
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 5,
        offset: 10,
    };
    let page3 = plan_store.list_plans(&query).await.unwrap();
    assert_eq!(page3.plans.len(), 5);
    assert!(!page3.has_more);
}

/// Test idempotency with revision tracking
#[tokio::test]
async fn test_idempotency_with_revisions() {
    let temp_dir = TempDir::new().unwrap();
    let plan_store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    let plan = create_test_plan("idempotent-test", "Idempotent Test");
    
    // Store same plan multiple times
    let record1 = plan_store.store_plan(&plan, "creator").await.unwrap();
    let record2 = plan_store.store_plan(&plan, "creator").await.unwrap();
    let record3 = plan_store.store_plan(&plan, "creator").await.unwrap();
    
    // Should have same ID but different revisions
    assert_eq!(record1.metadata.plan_id, record2.metadata.plan_id);
    assert_eq!(record2.metadata.plan_id, record3.metadata.plan_id);
    assert_eq!(record1.metadata.revision, 1);
    assert_eq!(record2.metadata.revision, 2);
    assert_eq!(record3.metadata.revision, 3);
    
    // Only one plan should exist in listing
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 10,
        offset: 0,
    };
    let response = plan_store.list_plans(&query).await.unwrap();
    assert_eq!(response.total_count, 1);
    
    // But it should be the latest revision
    let latest_plan = &response.plans[0];
    assert_eq!(latest_plan.metadata.revision, 3);
}

/// Test concurrent UI operations
#[tokio::test]
async fn test_concurrent_ui_operations() {
    let temp_dir = TempDir::new().unwrap();
    let plan_store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Create multiple plans concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let store = plan_store.clone();
        let handle = tokio::spawn(async move {
            let plan = create_test_plan(&format!("concurrent-ui-{}", i), &format!("Concurrent UI {}", i));
            store.store_plan(&plan, &format!("creator-{}", i)).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    // Verify all plans exist
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 20,
        offset: 0,
    };
    let response = plan_store.list_plans(&query).await.unwrap();
    assert_eq!(response.total_count, 10);
    
    // Test concurrent approvals
    let mut approval_handles = vec![];
    for plan_record in &response.plans[0..5] {
        let store = plan_store.clone();
        let plan_id = plan_record.metadata.plan_id.clone();
        let handle = tokio::spawn(async move {
            store.approve_plan(&plan_id, "approver").await
        });
        approval_handles.push(handle);
    }
    
    // Wait for approvals
    for handle in approval_handles {
        handle.await.unwrap().unwrap();
    }
    
    // Verify approvals
    let approved_query = ListQuery {
        status: Some("approved".to_string()),
        creator: None,
        limit: 10,
        offset: 0,
    };
    let approved_response = plan_store.list_plans(&approved_query).await.unwrap();
    assert_eq!(approved_response.plans.len(), 5);
}
