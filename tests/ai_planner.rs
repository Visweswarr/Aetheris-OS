//! AI Planner Tests - Phase 5
//! 
//! Comprehensive unit tests for task planning functionality including
//! plan generation, step execution, error handling, and deterministic serialization.

use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tokio;
use serde_json;

// Import the AI service modules
use aetheris_ai::planner::{
    TaskPlanner, StubTaskPlanner, Plan, PlanStep, PlanRequest, PlanResult,
    PlannerConfig, StepPriority, ExecutionStrategy, PlanStatus, PlanStepStatus
};
use aetheris_ai::api::{PlannerApi, CreatePlanRequest, CreatePlanResponse};
use aetheris_ai::policy::AiPolicy;
use aetheris_ai::error::AiError;

/// Test fixtures and utilities
struct TestFixtures {
    planner: Arc<dyn TaskPlanner>,
    api: PlannerApi,
    policy: Arc<AiPolicy>,
}

impl TestFixtures {
    async fn new() -> Self {
        let policy = Arc::new(AiPolicy::default());
        let planner = Arc::new(StubTaskPlanner::new(policy.clone())) as Arc<dyn TaskPlanner>;
        let api = PlannerApi::new(policy.clone());
        
        // Initialize both planner and API
        let config = PlannerConfig::default();
        planner.init(config).await.unwrap();
        api.init().await.unwrap();
        
        Self { planner, api, policy }
    }
    
    fn sample_plan_request() -> PlanRequest {
        PlanRequest {
            goal: "Create a simple web application with user authentication".to_string(),
            context: Some("Build a secure web app with login functionality".to_string()),
            required_tools: vec!["npm".to_string(), "node".to_string(), "database".to_string()],
            priority: StepPriority::Normal,
            max_execution_time: Some(3600),
            tags: vec!["web".to_string(), "development".to_string(), "auth".to_string()],
            metadata: HashMap::new(),
        }
    }
    
    fn sample_api_request() -> CreatePlanRequest {
        CreatePlanRequest {
            goal: "Deploy a microservice to Kubernetes".to_string(),
            context: Some("Deploy with proper scaling and monitoring".to_string()),
            required_tools: Some(vec!["kubectl".to_string(), "docker".to_string()]),
            priority: Some("high".to_string()),
            max_execution_time: Some(1800),
            tags: Some(vec!["kubernetes".to_string(), "deployment".to_string()]),
        }
    }
}

#[tokio::test]
async fn test_plan_creation_and_basic_properties() {
    let plan = Plan::new(
        "Test Plan",
        "A test plan for unit testing",
        "Complete a test goal successfully",
        "test-creator"
    );
    
    assert_eq!(plan.name, "Test Plan");
    assert_eq!(plan.description, "A test plan for unit testing");
    assert_eq!(plan.user_goal, "Complete a test goal successfully");
    assert_eq!(plan.creator, "test-creator");
    assert_eq!(plan.version, 1);
    assert!(matches!(plan.status, PlanStatus::Generating));
    assert!(plan.steps.is_empty());
    assert_eq!(plan.calculate_progress(), 0.0);
}

#[tokio::test]
async fn test_plan_step_creation_and_dependencies() {
    let mut step1 = PlanStep::new("First Step", "Initialize the project", "setup");
    let mut step2 = PlanStep::new("Second Step", "Configure dependencies", "configuration");
    let step3 = PlanStep::new("Third Step", "Build the application", "build");
    
    // Test step properties
    assert_eq!(step1.name, "First Step");
    assert_eq!(step1.step_type, "setup");
    assert_eq!(step1.priority, StepPriority::Normal);
    assert!(matches!(step1.status, PlanStepStatus::Pending));
    assert!(step1.dependencies.is_empty());
    
    // Test dependency management
    step2.add_dependency(&step1.id);
    assert_eq!(step2.dependencies.len(), 1);
    assert!(step2.dependencies.contains(&step1.id));
    
    // Test readiness checking
    assert!(step1.is_ready_to_execute(&[])); // No dependencies
    assert!(!step2.is_ready_to_execute(&[])); // Has dependency not completed
    assert!(step2.is_ready_to_execute(&[step1.id.clone()])); // Dependency completed
    
    // Test tool assignment
    let tool_params = serde_json::json!({"param1": "value1", "param2": 42});
    step1.set_tool("npm", tool_params.clone());
    assert_eq!(step1.tool_id, Some("npm".to_string()));
    assert_eq!(step1.tool_params, Some(tool_params));
}

#[tokio::test]
async fn test_plan_step_duration_calculation() {
    let mut step1 = PlanStep::new("Step 1", "First step", "action");
    step1.estimated_duration_us = Some(60_000_000); // 1 minute
    
    let mut sub_step1 = PlanStep::new("Sub Step 1", "First sub step", "sub-action");
    sub_step1.estimated_duration_us = Some(30_000_000); // 30 seconds
    
    let mut sub_step2 = PlanStep::new("Sub Step 2", "Second sub step", "sub-action");  
    sub_step2.estimated_duration_us = Some(45_000_000); // 45 seconds
    
    step1.sub_steps.push(sub_step1);
    step1.sub_steps.push(sub_step2);
    
    // Total should be 1 minute + 30 seconds + 45 seconds = 2 minutes 15 seconds
    assert_eq!(step1.get_total_estimated_duration(), 135_000_000);
}

#[tokio::test]
async fn test_planner_initialization_and_configuration() {
    let fixtures = TestFixtures::new().await;
    
    // Test that planner was initialized successfully
    let stats = fixtures.planner.get_stats().await.unwrap();
    assert_eq!(stats.total_plans, 0);
    assert_eq!(stats.active_plans, 0);
    assert_eq!(stats.completed_plans, 0);
    assert_eq!(stats.failed_plans, 0);
}

#[tokio::test]
async fn test_plan_generation_success() {
    let fixtures = TestFixtures::new().await;
    let request = TestFixtures::sample_plan_request();
    
    let result = fixtures.planner.generate_plan(request.clone()).await.unwrap();
    
    assert!(result.success);
    assert!(result.error.is_none());
    assert!(result.generation_time_us > 0);
    assert_eq!(result.llm_stats.model, "llama3-8b");
    
    let plan = result.plan.unwrap();
    assert_eq!(plan.user_goal, request.goal);
    assert_eq!(plan.tags, request.tags);
    assert_eq!(plan.priority, request.priority);
    assert!(!plan.steps.is_empty());
    assert!(matches!(plan.status, PlanStatus::Ready));
    
    // Verify plan was stored
    let retrieved_plan = fixtures.planner.get_plan(&plan.id).await.unwrap();
    assert_eq!(retrieved_plan.id, plan.id);
    assert_eq!(retrieved_plan.user_goal, plan.user_goal);
}

#[tokio::test]
async fn test_plan_generation_deterministic() {
    let fixtures = TestFixtures::new().await;
    let request = TestFixtures::sample_plan_request();
    
    // Generate two plans with the same request
    let result1 = fixtures.planner.generate_plan(request.clone()).await.unwrap();
    let result2 = fixtures.planner.generate_plan(request.clone()).await.unwrap();
    
    assert!(result1.success);
    assert!(result2.success);
    
    let plan1 = result1.plan.unwrap();
    let plan2 = result2.plan.unwrap();
    
    // Plans should have same structure (deterministic generation)
    assert_eq!(plan1.steps.len(), plan2.steps.len());
    assert_eq!(plan1.user_goal, plan2.user_goal);
    
    // But different IDs (they are separate instances)
    assert_ne!(plan1.id, plan2.id);
}

#[tokio::test]
async fn test_plan_listing_and_retrieval() {
    let fixtures = TestFixtures::new().await;
    
    // Initially no plans
    let initial_plans = fixtures.planner.list_plans().await.unwrap();
    assert!(initial_plans.is_empty());
    
    // Create multiple plans
    let request1 = PlanRequest {
        goal: "Goal 1".to_string(),
        context: None,
        required_tools: vec![],
        priority: StepPriority::High,
        max_execution_time: None,
        tags: vec!["tag1".to_string()],
        metadata: HashMap::new(),
    };
    
    let request2 = PlanRequest {
        goal: "Goal 2".to_string(),
        context: None,
        required_tools: vec![],
        priority: StepPriority::Low,
        max_execution_time: None,
        tags: vec!["tag2".to_string()],
        metadata: HashMap::new(),
    };
    
    let result1 = fixtures.planner.generate_plan(request1).await.unwrap();
    let result2 = fixtures.planner.generate_plan(request2).await.unwrap();
    
    assert!(result1.success);
    assert!(result2.success);
    
    let plan1 = result1.plan.unwrap();
    let plan2 = result2.plan.unwrap();
    
    // List all plans
    let all_plans = fixtures.planner.list_plans().await.unwrap();
    assert_eq!(all_plans.len(), 2);
    
    let plan_ids: Vec<_> = all_plans.iter().map(|p| &p.id).collect();
    assert!(plan_ids.contains(&&plan1.id));
    assert!(plan_ids.contains(&&plan2.id));
    
    // Retrieve individual plans
    let retrieved1 = fixtures.planner.get_plan(&plan1.id).await.unwrap();
    let retrieved2 = fixtures.planner.get_plan(&plan2.id).await.unwrap();
    
    assert_eq!(retrieved1.id, plan1.id);
    assert_eq!(retrieved1.user_goal, "Goal 1");
    assert_eq!(retrieved2.id, plan2.id);
    assert_eq!(retrieved2.user_goal, "Goal 2");
}

#[tokio::test]
async fn test_plan_deletion() {
    let fixtures = TestFixtures::new().await;
    let request = TestFixtures::sample_plan_request();
    
    let result = fixtures.planner.generate_plan(request).await.unwrap();
    let plan = result.plan.unwrap();
    
    // Plan should exist
    assert!(fixtures.planner.get_plan(&plan.id).await.is_ok());
    
    // Delete plan
    assert!(fixtures.planner.delete_plan(&plan.id).await.is_ok());
    
    // Plan should no longer exist
    assert!(fixtures.planner.get_plan(&plan.id).await.is_err());
    
    // Deleting non-existent plan should fail
    assert!(fixtures.planner.delete_plan("non_existent_id").await.is_err());
}

#[tokio::test]
async fn test_step_execution() {
    let fixtures = TestFixtures::new().await;
    let request = TestFixtures::sample_plan_request();
    
    let result = fixtures.planner.generate_plan(request).await.unwrap();
    let plan = result.plan.unwrap();
    
    assert!(!plan.steps.is_empty());
    let first_step_id = &plan.steps[0].id;
    
    // Execute the first step
    let executed_step = fixtures.planner.execute_step(&plan.id, first_step_id).await.unwrap();
    
    assert!(matches!(executed_step.status, PlanStepStatus::Completed { .. }));
    assert!(executed_step.actual_outputs.is_some());
    
    // Verify the step was updated in the plan
    let updated_plan = fixtures.planner.get_plan(&plan.id).await.unwrap();
    let updated_step = updated_plan.steps.iter().find(|s| s.id == *first_step_id).unwrap();
    assert!(matches!(updated_step.status, PlanStepStatus::Completed { .. }));
}

#[tokio::test]
async fn test_plan_progress_calculation() {
    let mut plan = Plan::new("Test", "Test plan", "Test goal", "test");
    
    // Empty plan should have 0% progress
    assert_eq!(plan.calculate_progress(), 0.0);
    
    // Add steps with different statuses
    let mut step1 = PlanStep::new("Step 1", "First step", "action");
    step1.status = PlanStepStatus::Completed { completed_at: 0, execution_time_us: 1000 };
    
    let mut step2 = PlanStep::new("Step 2", "Second step", "action");
    step2.status = PlanStepStatus::Completed { completed_at: 0, execution_time_us: 1000 };
    
    let step3 = PlanStep::new("Step 3", "Third step", "action");
    // step3 remains pending
    
    plan.add_step(step1);
    plan.add_step(step2);
    plan.add_step(step3);
    
    // 2 out of 3 steps completed = 66.67%
    let progress = plan.calculate_progress();
    assert!((progress - 0.6666666666666666).abs() < 0.0001);
}

#[tokio::test]
async fn test_plan_serialization_deterministic() {
    let fixtures = TestFixtures::new().await;
    let request = TestFixtures::sample_plan_request();
    
    let result = fixtures.planner.generate_plan(request).await.unwrap();
    let plan = result.plan.unwrap();
    
    // Serialize the plan
    let serialized1 = fixtures.planner.serialize_plan(&plan).await.unwrap();
    let serialized2 = fixtures.planner.serialize_plan(&plan).await.unwrap();
    
    // Serialization should be deterministic
    assert_eq!(serialized1, serialized2);
    assert!(!serialized1.is_empty());
    
    // Deserialize and verify
    let deserialized = fixtures.planner.deserialize_plan(&serialized1).await.unwrap();
    assert_eq!(deserialized.id, plan.id);
    assert_eq!(deserialized.user_goal, plan.user_goal);
    assert_eq!(deserialized.steps.len(), plan.steps.len());
}

#[tokio::test]
async fn test_planner_error_handling() {
    let fixtures = TestFixtures::new().await;
    
    // Test getting non-existent plan
    let result = fixtures.planner.get_plan("non_existent_id").await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AiError::Validation(msg) => assert!(msg.contains("Plan not found")),
        _ => panic!("Expected validation error"),
    }
    
    // Test executing step on non-existent plan
    let result = fixtures.planner.execute_step("non_existent_plan", "step_id").await;
    assert!(result.is_err());
    
    // Test invalid serialization data
    let invalid_data = b"invalid cbor data";
    let result = fixtures.planner.deserialize_plan(invalid_data).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_create_plan() {
    let fixtures = TestFixtures::new().await;
    let request = TestFixtures::sample_api_request();
    
    let response = fixtures.api.create_plan(request.clone()).await;
    
    assert!(response.success);
    assert!(response.plan_id.is_some());
    assert!(response.plan.is_some());
    assert!(response.error.is_none());
    assert!(response.generation_time_ms > 0);
    assert!(response.llm_stats.is_some());
    
    let plan_data = response.plan.unwrap();
    assert_eq!(plan_data.user_goal, request.goal);
    assert_eq!(plan_data.priority, "high");
    assert_eq!(plan_data.status, "ready");
    assert!(!plan_data.steps.is_empty());
}

#[tokio::test]
async fn test_api_get_plan() {
    let fixtures = TestFixtures::new().await;
    let request = TestFixtures::sample_api_request();
    
    // Create a plan first
    let create_response = fixtures.api.create_plan(request).await;
    assert!(create_response.success);
    let plan_id = create_response.plan_id.unwrap();
    
    // Get the plan
    let get_response = fixtures.api.get_plan(&plan_id).await;
    assert!(get_response.success);
    assert!(get_response.plan.is_some());
    assert!(get_response.error.is_none());
    
    let plan_data = get_response.plan.unwrap();
    assert_eq!(plan_data.id, plan_id);
}

#[tokio::test]
async fn test_api_list_plans() {
    let fixtures = TestFixtures::new().await;
    
    // Initially no plans
    let initial_response = fixtures.api.list_plans().await;
    assert!(initial_response.success);
    assert_eq!(initial_response.total_count, 0);
    
    // Create a few plans
    let request1 = CreatePlanRequest {
        goal: "API Test Goal 1".to_string(),
        context: None,
        required_tools: None,
        priority: Some("normal".to_string()),
        max_execution_time: None,
        tags: Some(vec!["api-test".to_string()]),
    };
    
    let request2 = CreatePlanRequest {
        goal: "API Test Goal 2".to_string(),
        context: None,
        required_tools: None,
        priority: Some("high".to_string()),
        max_execution_time: None,
        tags: Some(vec!["api-test".to_string()]),
    };
    
    fixtures.api.create_plan(request1).await;
    fixtures.api.create_plan(request2).await;
    
    // List plans
    let list_response = fixtures.api.list_plans().await;
    assert!(list_response.success);
    assert_eq!(list_response.total_count, 2);
    assert_eq!(list_response.plans.len(), 2);
    
    // Verify plan summaries
    let goals: Vec<_> = list_response.plans.iter().map(|p| &p.user_goal).collect();
    assert!(goals.contains(&&"API Test Goal 1".to_string()));
    assert!(goals.contains(&&"API Test Goal 2".to_string()));
}

#[tokio::test]
async fn test_api_priority_conversion() {
    let fixtures = TestFixtures::new().await;
    
    let test_cases = vec![
        ("low", "low"),
        ("normal", "normal"), 
        ("high", "high"),
        ("critical", "critical"),
        ("invalid", "normal"), // Should default to normal
    ];
    
    for (input_priority, expected_priority) in test_cases {
        let request = CreatePlanRequest {
            goal: format!("Test goal with {} priority", input_priority),
            context: None,
            required_tools: None,
            priority: Some(input_priority.to_string()),
            max_execution_time: None,
            tags: None,
        };
        
        let response = fixtures.api.create_plan(request).await;
        assert!(response.success);
        
        let plan_data = response.plan.unwrap();
        assert_eq!(plan_data.priority, expected_priority);
    }
}

#[tokio::test]
async fn test_api_error_handling() {
    let fixtures = TestFixtures::new().await;
    
    // Test getting non-existent plan
    let response = fixtures.api.get_plan("non_existent_id").await;
    assert!(!response.success);
    assert!(response.plan.is_none());
    assert!(response.error.is_some());
}

#[tokio::test]
async fn test_planner_statistics_tracking() {
    let fixtures = TestFixtures::new().await;
    
    // Initial stats
    let initial_stats = fixtures.planner.get_stats().await.unwrap();
    assert_eq!(initial_stats.total_plans, 0);
    assert_eq!(initial_stats.active_plans, 0);
    
    // Create a plan
    let request = TestFixtures::sample_plan_request();
    let result = fixtures.planner.generate_plan(request).await.unwrap();
    assert!(result.success);
    
    // Stats should be updated
    let updated_stats = fixtures.planner.get_stats().await.unwrap();
    assert_eq!(updated_stats.total_plans, 1);
    assert_eq!(updated_stats.active_plans, 1);
    
    // Delete the plan
    let plan = result.plan.unwrap();
    fixtures.planner.delete_plan(&plan.id).await.unwrap();
    
    // Active plans should decrease
    let final_stats = fixtures.planner.get_stats().await.unwrap();
    assert_eq!(final_stats.total_plans, 1); // Total doesn't decrease
    assert_eq!(final_stats.active_plans, 0); // Active decreases
}

#[tokio::test]
async fn test_plan_ready_steps_calculation() {
    let fixtures = TestFixtures::new().await;
    let request = TestFixtures::sample_plan_request();
    
    let result = fixtures.planner.generate_plan(request).await.unwrap();
    let mut plan = result.plan.unwrap();
    
    // Initially, first step should be ready (no dependencies)
    let ready_steps = plan.get_ready_steps();
    assert_eq!(ready_steps.len(), 1);
    assert_eq!(ready_steps[0].id, plan.steps[0].id);
    
    // Mark first step as completed
    if let Some(first_step) = plan.steps.get_mut(0) {
        first_step.status = PlanStepStatus::Completed {
            completed_at: 12345,
            execution_time_us: 1000,
        };
    }
    
    // Now second step should be ready (depends on first)
    let ready_steps = plan.get_ready_steps();
    if plan.steps.len() > 1 {
        assert!(!ready_steps.is_empty());
        // The exact step depends on the generated plan structure
    }
}

#[tokio::test]
async fn test_plan_revision_tracking() {
    let mut plan = Plan::new("Test Plan", "Description", "Goal", "creator");
    
    assert_eq!(plan.version, 1);
    assert!(plan.revisions.is_empty());
    
    // Add a revision
    plan.add_revision("Updated requirements", "updater");
    
    assert_eq!(plan.version, 2);
    assert_eq!(plan.revisions.len(), 1);
    assert_eq!(plan.revisions[0].revision, 2);
    assert_eq!(plan.revisions[0].changes, "Updated requirements");
    assert_eq!(plan.revisions[0].author, "updater");
    
    // Add another revision
    plan.add_revision("Fixed dependencies", "fixer");
    
    assert_eq!(plan.version, 3);
    assert_eq!(plan.revisions.len(), 2);
}

#[tokio::test] 
async fn test_execution_strategy_types() {
    let mut step = PlanStep::new("Test Step", "Description", "action");
    
    // Test different execution strategies
    step.execution_strategy = ExecutionStrategy::Sequential;
    assert!(matches!(step.execution_strategy, ExecutionStrategy::Sequential));
    
    step.execution_strategy = ExecutionStrategy::Parallel;
    assert!(matches!(step.execution_strategy, ExecutionStrategy::Parallel));
    
    step.execution_strategy = ExecutionStrategy::Conditional("condition".to_string());
    if let ExecutionStrategy::Conditional(condition) = &step.execution_strategy {
        assert_eq!(condition, "condition");
    } else {
        panic!("Expected conditional strategy");
    }
    
    step.execution_strategy = ExecutionStrategy::Retry { max_attempts: 3, delay_ms: 1000 };
    if let ExecutionStrategy::Retry { max_attempts, delay_ms } = step.execution_strategy {
        assert_eq!(max_attempts, 3);
        assert_eq!(delay_ms, 1000);
    } else {
        panic!("Expected retry strategy");
    }
}

// Integration test combining multiple features
#[tokio::test]
async fn test_end_to_end_planning_workflow() {
    let fixtures = TestFixtures::new().await;
    
    // 1. Create a plan via API
    let api_request = CreatePlanRequest {
        goal: "Complete end-to-end test workflow".to_string(),
        context: Some("Integration test for planning system".to_string()),
        required_tools: Some(vec!["test-tool".to_string()]),
        priority: Some("high".to_string()),
        max_execution_time: Some(600),
        tags: Some(vec!["e2e".to_string(), "integration".to_string()]),
    };
    
    let create_response = fixtures.api.create_plan(api_request.clone()).await;
    assert!(create_response.success);
    let plan_id = create_response.plan_id.unwrap();
    
    // 2. Verify plan was created correctly
    let get_response = fixtures.api.get_plan(&plan_id).await;
    assert!(get_response.success);
    let plan_data = get_response.plan.unwrap();
    assert_eq!(plan_data.user_goal, api_request.goal);
    assert_eq!(plan_data.priority, "high");
    
    // 3. Get the plan via planner interface
    let internal_plan = fixtures.planner.get_plan(&plan_id).await.unwrap();
    assert_eq!(internal_plan.id, plan_id);
    assert!(!internal_plan.steps.is_empty());
    
    // 4. Execute first step
    let first_step_id = &internal_plan.steps[0].id;
    let executed_step = fixtures.planner.execute_step(&plan_id, first_step_id).await.unwrap();
    assert!(matches!(executed_step.status, PlanStepStatus::Completed { .. }));
    
    // 5. Verify plan progress updated
    let updated_response = fixtures.api.get_plan(&plan_id).await;
    let updated_plan = updated_response.plan.unwrap();
    assert!(updated_plan.progress > 0.0);
    
    // 6. Serialize and deserialize plan
    let updated_internal_plan = fixtures.planner.get_plan(&plan_id).await.unwrap();
    let serialized = fixtures.planner.serialize_plan(&updated_internal_plan).await.unwrap();
    let deserialized = fixtures.planner.deserialize_plan(&serialized).await.unwrap();
    assert_eq!(deserialized.id, updated_internal_plan.id);
    assert_eq!(deserialized.steps.len(), updated_internal_plan.steps.len());
    
    // 7. Verify stats updated
    let stats = fixtures.planner.get_stats().await.unwrap();
    assert!(stats.total_plans > 0);
    assert!(stats.active_plans > 0);
    
    // 8. List plans and verify our plan is included
    let list_response = fixtures.api.list_plans().await;
    assert!(list_response.success);
    let plan_ids: Vec<_> = list_response.plans.iter().map(|p| &p.id).collect();
    assert!(plan_ids.contains(&&plan_id));
    
    // 9. Clean up - delete the plan
    assert!(fixtures.planner.delete_plan(&plan_id).await.is_ok());
    
    // 10. Verify plan no longer exists
    let final_get_response = fixtures.api.get_plan(&plan_id).await;
    assert!(!final_get_response.success);
}