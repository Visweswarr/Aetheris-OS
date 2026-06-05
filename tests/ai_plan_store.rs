//! Comprehensive tests for AI Plan Store with deterministic IDs and persistence

use std::fs;
use std::path::Path;
use std::collections::{HashMap, BTreeMap};
use tempfile::TempDir;
use serde_json::{Value, json};
use tokio::time::{sleep, Duration};

use aetheris_ai::plan_store::{PlanStore, PlanRecord, PlanMetadata, PlanStatus, ListQuery};
use aetheris_ai::planner::{Plan, Step, StepPriority, StepStatus};

fn create_test_plan(id: &str, name: &str, creator: &str) -> Plan {
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
            Step {
                id: format!("{}-step-2", id),
                name: "Execute".to_string(),
                description: "Execute the test".to_string(),
                step_type: "action".to_string(),
                priority: StepPriority::High,
                status: StepStatus::Pending,
                dependencies: vec![format!("{}-step-1", id)],
                estimated_duration_secs: Some(30),
                tool_id: Some("test-tool".to_string()),
                expected_outputs: vec!["result.txt".to_string()],
                sub_steps: vec![],
            },
        ],
        tags: vec!["test".to_string(), "automated".to_string()],
        creator: creator.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        estimated_duration_secs: 40,
        actual_duration_secs: None,
        progress: 0.0,
    }
}

/// Test deterministic Plan ID generation
#[tokio::test]
async fn test_deterministic_plan_ids() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Create identical plans multiple times
    let plan1 = create_test_plan("test-plan", "Test Plan", "creator");
    let plan2 = create_test_plan("test-plan", "Test Plan", "creator");
    
    let record1 = store.store_plan(&plan1, "creator").await.unwrap();
    let record2 = store.store_plan(&plan2, "creator").await.unwrap();
    
    // IDs should be identical for identical plans
    assert_eq!(record1.metadata.plan_id, record2.metadata.plan_id);
    
    // But revisions should increment if storing the same plan twice
    assert_eq!(record1.metadata.revision, 1);
    assert_eq!(record2.metadata.revision, 2);
}

/// Test canonical JSON serialization consistency
#[tokio::test]
async fn test_canonical_json_consistency() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Create plans with same data but different field order in JSON
    let mut plan1 = create_test_plan("test", "Test", "creator");
    let mut plan2 = create_test_plan("test", "Test", "creator");
    
    // Ensure fields are identical
    plan1.tags.sort();
    plan2.tags.sort();
    
    let id1 = store.generate_plan_id(&plan1).await.unwrap();
    let id2 = store.generate_plan_id(&plan2).await.unwrap();
    
    assert_eq!(id1, id2, "Identical plans should generate identical IDs");
}

/// Test plan storage and retrieval
#[tokio::test]
async fn test_plan_storage_and_retrieval() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    let plan = create_test_plan("storage-test", "Storage Test", "creator");
    let record = store.store_plan(&plan, "creator").await.unwrap();
    
    // Test retrieval
    let retrieved = store.get_plan_by_id(&record.metadata.plan_id).await.unwrap();
    assert!(retrieved.is_some());
    
    let retrieved_record = retrieved.unwrap();
    assert_eq!(retrieved_record.metadata.plan_id, record.metadata.plan_id);
    assert_eq!(retrieved_record.plan.name, plan.name);
    assert_eq!(retrieved_record.plan.user_goal, plan.user_goal);
    assert_eq!(retrieved_record.metadata.status, PlanStatus::Pending);
    assert_eq!(retrieved_record.metadata.user_id, "creator");
}

/// Test plan approval workflow
#[tokio::test]
async fn test_plan_approval_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    let plan = create_test_plan("approval-test", "Approval Test", "creator");
    let record = store.store_plan(&plan, "creator").await.unwrap();
    
    // Plan should start as pending
    assert_eq!(record.metadata.status, PlanStatus::Pending);
    assert!(record.metadata.approved_by.is_none());
    assert!(record.metadata.approved_at.is_none());
    
    // Approve the plan
    let approved_record = store.approve_plan(&record.metadata.plan_id, "approver").await.unwrap();
    
    // Check approval status
    assert_eq!(approved_record.metadata.status, PlanStatus::Approved);
    assert_eq!(approved_record.metadata.approved_by, Some("approver".to_string()));
    assert!(approved_record.metadata.approved_at.is_some());
    assert_eq!(approved_record.metadata.revision, 2); // Should increment
    
    // Test double approval (should be idempotent)
    let double_approved = store.approve_plan(&record.metadata.plan_id, "approver2").await.unwrap();
    assert_eq!(double_approved.metadata.status, PlanStatus::Approved);
    assert_eq!(double_approved.metadata.approved_by, Some("approver".to_string())); // Original approver
    assert_eq!(double_approved.metadata.revision, 2); // No change
}

/// Test plan listing with filters and pagination
#[tokio::test]
async fn test_plan_listing_and_pagination() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Create multiple plans
    let plan1 = create_test_plan("list-test-1", "List Test 1", "creator1");
    let plan2 = create_test_plan("list-test-2", "List Test 2", "creator2");
    let plan3 = create_test_plan("list-test-3", "List Test 3", "creator1");
    
    let record1 = store.store_plan(&plan1, "creator1").await.unwrap();
    let record2 = store.store_plan(&plan2, "creator2").await.unwrap();
    let record3 = store.store_plan(&plan3, "creator1").await.unwrap();
    
    // Approve one plan
    store.approve_plan(&record2.metadata.plan_id, "approver").await.unwrap();
    
    // Test listing all plans
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 10,
        offset: 0,
    };
    let response = store.list_plans(&query).await.unwrap();
    assert_eq!(response.total_count, 3);
    assert!(!response.has_more);
    
    // Test filtering by status
    let query = ListQuery {
        status: Some("pending".to_string()),
        creator: None,
        limit: 10,
        offset: 0,
    };
    let response = store.list_plans(&query).await.unwrap();
    assert_eq!(response.plans.len(), 2); // plan1 and plan3
    
    let query = ListQuery {
        status: Some("approved".to_string()),
        creator: None,
        limit: 10,
        offset: 0,
    };
    let response = store.list_plans(&query).await.unwrap();
    assert_eq!(response.plans.len(), 1); // plan2
    
    // Test filtering by creator
    let query = ListQuery {
        status: None,
        creator: Some("creator1".to_string()),
        limit: 10,
        offset: 0,
    };
    let response = store.list_plans(&query).await.unwrap();
    assert_eq!(response.plans.len(), 2); // plan1 and plan3
    
    // Test pagination
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 2,
        offset: 0,
    };
    let response = store.list_plans(&query).await.unwrap();
    assert_eq!(response.plans.len(), 2);
    assert!(response.has_more);
    
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 2,
        offset: 2,
    };
    let response = store.list_plans(&query).await.unwrap();
    assert_eq!(response.plans.len(), 1);
    assert!(!response.has_more);
}

/// Test NDJSON persistence format
#[tokio::test]
async fn test_ndjson_persistence_format() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("plans.ndjson");
    
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    let plan = create_test_plan("ndjson-test", "NDJSON Test", "creator");
    let record = store.store_plan(&plan, "creator").await.unwrap();
    
    // Check that log file exists
    assert!(log_path.exists());
    
    // Read and parse the log file
    let log_content = fs::read_to_string(&log_path).unwrap();
    let lines: Vec<&str> = log_content.lines().collect();
    assert_eq!(lines.len(), 1); // One event so far
    
    // Parse the JSON line
    let event: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(event["event_type"], "PlanCreated");
    assert_eq!(event["plan_id"], record.metadata.plan_id);
    assert_eq!(event["user_id"], "creator");
    
    // Approve the plan (should add another line)
    store.approve_plan(&record.metadata.plan_id, "approver").await.unwrap();
    
    let updated_content = fs::read_to_string(&log_path).unwrap();
    let updated_lines: Vec<&str> = updated_content.lines().collect();
    assert_eq!(updated_lines.len(), 2); // Two events now
    
    let approval_event: Value = serde_json::from_str(updated_lines[1]).unwrap();
    assert_eq!(approval_event["event_type"], "PlanApproved");
    assert_eq!(approval_event["plan_id"], record.metadata.plan_id);
    assert_eq!(approval_event["approved_by"], "approver");
}

/// Test crash recovery through log replay
#[tokio::test]
async fn test_crash_recovery_replay() {
    let temp_dir = TempDir::new().unwrap();
    
    // First session: create and approve some plans
    {
        let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        let plan1 = create_test_plan("recovery-1", "Recovery Test 1", "creator");
        let plan2 = create_test_plan("recovery-2", "Recovery Test 2", "creator");
        
        let record1 = store.store_plan(&plan1, "creator").await.unwrap();
        let record2 = store.store_plan(&plan2, "creator").await.unwrap();
        
        store.approve_plan(&record1.metadata.plan_id, "approver").await.unwrap();
        
        // Store should have 2 plans, 1 approved
        let query = ListQuery {
            status: None,
            creator: None,
            limit: 10,
            offset: 0,
        };
        let response = store.list_plans(&query).await.unwrap();
        assert_eq!(response.total_count, 2);
    }
    
    // Second session: simulate restart by creating new store instance
    {
        let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        
        // Data should be recovered from log
        let query = ListQuery {
            status: None,
            creator: None,
            limit: 10,
            offset: 0,
        };
        let response = store.list_plans(&query).await.unwrap();
        assert_eq!(response.total_count, 2);
        
        // Check approval status is restored
        let approved_query = ListQuery {
            status: Some("approved".to_string()),
            creator: None,
            limit: 10,
            offset: 0,
        };
        let approved_response = store.list_plans(&approved_query).await.unwrap();
        assert_eq!(approved_response.plans.len(), 1);
        
        let pending_query = ListQuery {
            status: Some("pending".to_string()),
            creator: None,
            limit: 10,
            offset: 0,
        };
        let pending_response = store.list_plans(&pending_query).await.unwrap();
        assert_eq!(pending_response.plans.len(), 1);
    }
}

/// Test concurrent access safety
#[tokio::test]
async fn test_concurrent_access() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Clone store handle for concurrent access
    let store_clone = store.clone();
    
    let mut handles = vec![];
    
    // Spawn multiple tasks to store plans concurrently
    for i in 0..10 {
        let store_handle = store_clone.clone();
        let handle = tokio::spawn(async move {
            let plan = create_test_plan(&format!("concurrent-{}", i), &format!("Concurrent Test {}", i), "creator");
            store_handle.store_plan(&plan, "creator").await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap().unwrap();
        results.push(result);
    }
    
    // Verify all plans were stored
    assert_eq!(results.len(), 10);
    
    // Verify no duplicate IDs (each plan should have unique content)
    let mut ids = std::collections::HashSet::new();
    for result in results {
        assert!(ids.insert(result.metadata.plan_id), "Duplicate plan ID found");
    }
    
    // Verify total count
    let query = ListQuery {
        status: None,
        creator: None,
        limit: 20,
        offset: 0,
    };
    let response = store.list_plans(&query).await.unwrap();
    assert_eq!(response.total_count, 10);
}

/// Test error handling for invalid operations
#[tokio::test]
async fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    // Test approval of non-existent plan
    let result = store.approve_plan("non-existent-plan", "approver").await;
    assert!(result.is_err());
    
    // Test retrieval of non-existent plan
    let result = store.get_plan_by_id("non-existent-plan").await.unwrap();
    assert!(result.is_none());
    
    // Test invalid status filter
    let query = ListQuery {
        status: Some("invalid-status".to_string()),
        creator: None,
        limit: 10,
        offset: 0,
    };
    let response = store.list_plans(&query).await.unwrap();
    assert_eq!(response.plans.len(), 0); // Should return empty for invalid status
}

/// Test plan ID format and uniqueness
#[tokio::test]
async fn test_plan_id_format_and_uniqueness() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    let plan = create_test_plan("id-test", "ID Test", "creator");
    let id = store.generate_plan_id(&plan).await.unwrap();
    
    // Blake3 hash should be 64 characters (256 bits in hex)
    assert_eq!(id.len(), 64);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    
    // Different plans should generate different IDs
    let mut different_plan = plan.clone();
    different_plan.name = "Different Name".to_string();
    let different_id = store.generate_plan_id(&different_plan).await.unwrap();
    assert_ne!(id, different_id);
    
    // Even small changes should result in different IDs
    let mut slightly_different = plan.clone();
    slightly_different.description = format!("{} modified", plan.description);
    let slightly_different_id = store.generate_plan_id(&slightly_different).await.unwrap();
    assert_ne!(id, slightly_different_id);
}

/// Test metadata preservation through operations
#[tokio::test]
async fn test_metadata_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let store = PlanStore::new(temp_dir.path().to_path_buf()).await.unwrap();
    
    let plan = create_test_plan("metadata-test", "Metadata Test", "creator");
    let record = store.store_plan(&plan, "creator").await.unwrap();
    
    // Verify initial metadata
    assert_eq!(record.metadata.user_id, "creator");
    assert_eq!(record.metadata.revision, 1);
    assert_eq!(record.metadata.status, PlanStatus::Pending);
    assert!(record.metadata.approved_by.is_none());
    assert!(record.metadata.approved_at.is_none());
    
    // Approve and verify metadata changes
    let approved = store.approve_plan(&record.metadata.plan_id, "approver").await.unwrap();
    assert_eq!(approved.metadata.user_id, "creator"); // Original creator preserved
    assert_eq!(approved.metadata.revision, 2); // Revision incremented
    assert_eq!(approved.metadata.status, PlanStatus::Approved);
    assert_eq!(approved.metadata.approved_by, Some("approver".to_string()));
    assert!(approved.metadata.approved_at.is_some());
    
    // Plan data should remain unchanged
    assert_eq!(approved.plan.name, plan.name);
    assert_eq!(approved.plan.user_goal, plan.user_goal);
    assert_eq!(approved.plan.steps.len(), plan.steps.len());
}