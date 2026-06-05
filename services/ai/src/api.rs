//! Task Planner HTTP API - Phase 5
//! 
//! Simple HTTP API for task planning functionality that can be used by
//! external applications, shell scripts, and other services.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::planner::{TaskPlanner, StubTaskPlanner, Plan, PlanRequest, PlanStep};
use crate::policy::AiPolicy;
use crate::error::AiError;

/// HTTP API request for creating a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlanRequest {
    /// User goal/objective
    pub goal: String,
    /// Additional context or constraints
    pub context: Option<String>,
    /// Required tools/capabilities
    pub required_tools: Option<Vec<String>>,
    /// Plan priority (low, normal, high, critical)
    pub priority: Option<String>,
    /// Maximum execution time in seconds
    pub max_execution_time: Option<u64>,
    /// Tags for the plan
    pub tags: Option<Vec<String>>,
}

/// HTTP API response for creating a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlanResponse {
    /// Success status
    pub success: bool,
    /// Generated plan ID
    pub plan_id: Option<String>,
    /// Full plan data
    pub plan: Option<PlanData>,
    /// Error message if failed
    pub error: Option<String>,
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    /// LLM usage statistics
    pub llm_stats: Option<LlmStats>,
}

/// HTTP API response for getting a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPlanResponse {
    /// Success status
    pub success: bool,
    /// Plan data
    pub plan: Option<PlanData>,
    /// Error message if failed
    pub error: Option<String>,
}

/// HTTP API response for listing plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPlansResponse {
    /// Success status
    pub success: bool,
    /// List of plans
    pub plans: Vec<PlanSummary>,
    /// Total count (for pagination)
    pub total_count: usize,
    /// Next cursor for pagination
    pub next_cursor: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Simplified plan data for HTTP API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanData {
    /// Plan ID
    pub id: String,
    /// Plan name
    pub name: String,
    /// Plan description
    pub description: String,
    /// Original user goal
    pub user_goal: String,
    /// Plan version
    pub version: u32,
    /// Plan priority
    pub priority: String,
    /// Plan status
    pub status: String,
    /// Plan steps
    pub steps: Vec<StepData>,
    /// Plan tags
    pub tags: Vec<String>,
    /// Plan creator
    pub creator: String,
    /// Created timestamp (ISO 8601)
    pub created_at: String,
    /// Last updated timestamp (ISO 8601)
    pub updated_at: String,
    /// Estimated duration in seconds
    pub estimated_duration_secs: u64,
    /// Actual execution time in seconds (if completed)
    pub actual_duration_secs: Option<u64>,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f64,
}

/// Simplified step data for HTTP API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepData {
    /// Step ID
    pub id: String,
    /// Step name
    pub name: String,
    /// Step description
    pub description: String,
    /// Step type
    pub step_type: String,
    /// Step priority
    pub priority: String,
    /// Step status
    pub status: String,
    /// Step dependencies
    pub dependencies: Vec<String>,
    /// Estimated duration in seconds
    pub estimated_duration_secs: Option<u64>,
    /// Tool to invoke (if any)
    pub tool_id: Option<String>,
    /// Expected outputs
    pub expected_outputs: Vec<String>,
    /// Sub-steps
    pub sub_steps: Vec<StepData>,
}

/// Plan summary for listing
#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct PlanSummary {
    /// Plan ID
    pub id: String,
    /// Plan name
    pub name: String,
    /// User goal
    pub user_goal: String,
    /// Plan priority
    pub priority: String,
    /// Plan status
    pub status: String,
    /// Created timestamp (ISO 8601)
    pub created_at: String,
    /// Step count
    pub step_count: usize,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f64,
    /// Plan creator
    pub creator: String,
    /// Plan revision number
    pub revision: u32,
}

/// LLM usage statistics for HTTP API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStats {
    /// Input tokens used
    pub input_tokens: u32,
    /// Output tokens generated
    pub output_tokens: u32,
    /// Inference time in milliseconds
    pub inference_time_ms: u64,
    /// Model used
    pub model: String,
}

/// Task Planner HTTP API handler
pub struct PlannerApi {
    planner: Arc<dyn TaskPlanner>,
    policy: Arc<AiPolicy>,
}

impl PlannerApi {
    /// Create a new planner API
    pub fn new(policy: Arc<AiPolicy>) -> Self {
        let planner = Arc::new(StubTaskPlanner::new(policy.clone())) as Arc<dyn TaskPlanner>;
        Self { planner, policy }
    }

    /// Initialize the planner with default configuration
    pub async fn init(&self) -> Result<(), AiError> {
        use crate::planner::PlannerConfig;
        let _ = crate::plan_ui::init_plan_store().await;
        let config = PlannerConfig::default();
        self.planner.init(config).await
    }

    /// Handle create plan request (now with persistent store)
    pub async fn create_plan(&self, request: CreatePlanRequest) -> CreatePlanResponse {
        let start_time = std::time::Instant::now();
        
        // Convert HTTP request to internal format
        let priority = request.priority.as_deref().unwrap_or("normal");
        let plan_priority = match priority {
            "low" => crate::planner::StepPriority::Low,
            "high" => crate::planner::StepPriority::High,
            "critical" => crate::planner::StepPriority::Critical,
            _ => crate::planner::StepPriority::Normal,
        };

        let plan_request = PlanRequest {
            goal: request.goal,
            context: request.context,
            required_tools: request.required_tools.unwrap_or_default(),
            priority: plan_priority,
            max_execution_time: request.max_execution_time,
            tags: request.tags.unwrap_or_default(),
            metadata: HashMap::new(),
        };

        // Generate plan
        match self.planner.generate_plan(plan_request).await {
            Ok(result) => {
                let generation_time_ms = start_time.elapsed().as_millis() as u64;
                
                if result.success {
                    if let Some(plan) = result.plan {
                        // Store plan in persistent store with deterministic ID
                        match crate::plan_ui::store_plan_for_ui(&plan, "api-user").await {
                            Ok(ui_plan) => CreatePlanResponse {
                                success: true,
                                plan_id: Some(ui_plan.id.clone()),
                                plan: Some(Self::convert_ui_plan_to_api(&ui_plan)),
                                error: None,
                                generation_time_ms,
                                llm_stats: Some(Self::convert_llm_stats(&result.llm_stats)),
                            },
                            Err(e) => CreatePlanResponse {
                                success: false,
                                plan_id: None,
                                plan: None,
                                error: Some(format!("Failed to store plan: {}", e)),
                                generation_time_ms,
                                llm_stats: Some(Self::convert_llm_stats(&result.llm_stats)),
                            }
                        }
                    } else {
                        CreatePlanResponse {
                            success: false,
                            plan_id: None,
                            plan: None,
                            error: Some("Plan generation succeeded but no plan returned".to_string()),
                            generation_time_ms,
                            llm_stats: Some(Self::convert_llm_stats(&result.llm_stats)),
                        }
                    }
                } else {
                    CreatePlanResponse {
                        success: false,
                        plan_id: None,
                        plan: None,
                        error: result.error,
                        generation_time_ms,
                        llm_stats: Some(Self::convert_llm_stats(&result.llm_stats)),
                    }
                }
            }
            Err(e) => {
                let generation_time_ms = start_time.elapsed().as_millis() as u64;
                CreatePlanResponse {
                    success: false,
                    plan_id: None,
                    plan: None,
                    error: Some(e.to_string()),
                    generation_time_ms,
                    llm_stats: None,
                }
            }
        }
    }

    /// Handle get plan request (now uses persistent store)
    pub async fn get_plan(&self, plan_id: &str) -> GetPlanResponse {
        match crate::plan_ui::get_plan_for_ui(plan_id) {
            Some(ui_plan) => GetPlanResponse {
                success: true,
                plan: Some(Self::convert_ui_plan_to_api(&ui_plan)),
                error: None,
            },
            None => GetPlanResponse {
                success: false,
                plan: None,
                error: Some("Plan not found".to_string()),
            }
        }
    }

    /// Handle plan approval request (enhanced with revisions)
    pub async fn approve_plan(
        &self,
        plan_id: &str,
        user_id: &str,
        revision: Option<u32>,
        modifications: Option<Vec<PlanStep>>,
    ) -> GetPlanResponse {
        match crate::plan_ui::approve_plan(plan_id, user_id, revision, modifications).await {
            Ok(ui_plan) => GetPlanResponse {
                success: true,
                plan: Some(Self::convert_ui_plan_to_api(&ui_plan)),
                error: None,
            },
            Err(e) => GetPlanResponse {
                success: false,
                plan: None,
                error: Some(e.to_string()),
            }
        }
    }

    /// Handle list plans request (with pagination and filtering)
    pub async fn list_plans(
        &self,
        status: Option<String>,
        limit: Option<usize>,
        cursor: Option<String>,
    ) -> ListPlansResponse {
        use crate::plan_store::PlanStatus;
        
        let filter_status = match status.as_deref() {
            Some("pending") => Some(PlanStatus::Pending),
            Some("approved") => Some(PlanStatus::Approved),
            _ => None,
        };
        
        let response = crate::plan_ui::list_plans_for_ui(filter_status, limit, cursor);
        
        let plan_summaries: Vec<PlanSummary> = response.plans.iter()
            .map(|record| PlanSummary {
                id: record.metadata.plan_id.clone(),
                name: record.plan.name.clone(),
                user_goal: record.plan.user_goal.clone(),
                priority: format!("{:?}", record.plan.priority),
                status: match record.metadata.status {
                    PlanStatus::Pending => "pending".to_string(),
                    PlanStatus::Approved => "approved".to_string(),
                },
                created_at: record.metadata.created_at.to_rfc3339(),
                step_count: record.plan.steps.len(),
                progress: record.plan.calculate_progress(),
                creator: record.metadata.user_id.clone(),
                revision: record.metadata.revision,
            })
            .collect();
            
        ListPlansResponse {
            success: true,
            plans: plan_summaries,
            total_count: response.total_count,
            next_cursor: response.next_cursor,
            error: None,
        }
    }

    /// Convert UiPlan to API PlanData
    fn convert_ui_plan_to_api(ui_plan: &crate::plan_ui::UiPlan) -> PlanData {
        let created_at = ui_plan
            .metadata
            .get("created_at")
            .cloned()
            .or_else(|| ui_plan.approved_at.clone())
            .unwrap_or_default();
        let revision = ui_plan
            .metadata
            .get("revision")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(1);

        PlanData {
            id: ui_plan.id.clone(),
            name: ui_plan.metadata.get("name").cloned().unwrap_or_default(),
            description: ui_plan.metadata.get("description").cloned().unwrap_or_default(),
            user_goal: ui_plan.metadata.get("user_goal").cloned().unwrap_or_default(),
            version: revision,
            priority: "normal".to_string(),
            status: if ui_plan.approved {
                "approved".to_string()
            } else {
                "pending".to_string()
            },
            steps: ui_plan
                .steps
                .iter()
                .map(|s| StepData {
                    id: format!("step_{}", s.number),
                    name: s.name.clone(),
                    description: s.description.clone(),
                    step_type: "action".to_string(),
                    priority: "normal".to_string(),
                    status: s.status.clone(),
                    dependencies: vec![],
                    estimated_duration_secs: None,
                    tool_id: None,
                    expected_outputs: vec![],
                    sub_steps: vec![],
                })
                .collect(),
            tags: vec![],
            creator: ui_plan
                .approved_by
                .clone()
                .or_else(|| ui_plan.metadata.get("creator").cloned())
                .unwrap_or_default(),
            created_at: created_at.clone(),
            updated_at: ui_plan.approved_at.clone().unwrap_or(created_at),
            estimated_duration_secs: 0,
            actual_duration_secs: None,
            progress: if ui_plan.approved { 1.0 } else { 0.0 },
        }
    }

    /// Delete a plan
    pub async fn delete_plan(&self, plan_id: &str) -> Result<(), AiError> {
        self.planner.delete_plan(plan_id).await
    }

    /// Convert internal Plan to API PlanData
    fn convert_plan_to_api(plan: &Plan) -> PlanData {
        PlanData {
            id: plan.id.clone(),
            name: plan.name.clone(),
            description: plan.description.clone(),
            user_goal: plan.user_goal.clone(),
            version: plan.version,
            priority: Self::priority_to_string(&plan.priority),
            status: Self::status_to_string(&plan.status),
            steps: plan.steps.iter().map(Self::convert_step_to_api).collect(),
            tags: plan.tags.clone(),
            creator: plan.creator.clone(),
            created_at: Self::timestamp_to_iso8601(plan.created_at),
            updated_at: Self::timestamp_to_iso8601(plan.updated_at),
            estimated_duration_secs: plan.estimated_duration_us / 1_000_000,
            actual_duration_secs: plan.actual_duration_us.map(|d| d / 1_000_000),
            progress: plan.calculate_progress(),
        }
    }

    /// Convert internal PlanStep to API StepData
    fn convert_step_to_api(step: &PlanStep) -> StepData {
        StepData {
            id: step.id.clone(),
            name: step.name.clone(),
            description: step.description.clone(),
            step_type: step.step_type.clone(),
            priority: Self::priority_to_string(&step.priority),
            status: Self::step_status_to_string(&step.status),
            dependencies: step.dependencies.clone(),
            estimated_duration_secs: step.estimated_duration_us.map(|d| d / 1_000_000),
            tool_id: step.tool_id.clone(),
            expected_outputs: step.expected_outputs.clone(),
            sub_steps: step.sub_steps.iter().map(Self::convert_step_to_api).collect(),
        }
    }

    /// Convert Plan to PlanSummary
    fn convert_plan_to_summary(plan: &Plan) -> PlanSummary {
        PlanSummary {
            id: plan.id.clone(),
            name: plan.name.clone(),
            user_goal: plan.user_goal.clone(),
            priority: Self::priority_to_string(&plan.priority),
            status: Self::status_to_string(&plan.status),
            created_at: Self::timestamp_to_iso8601(plan.created_at),
            step_count: plan.steps.len(),
            progress: plan.calculate_progress(),
            creator: plan.creator.clone(),
            revision: plan.version,
        }
    }

    /// Convert LlmUsageStats to API format
    fn convert_llm_stats(stats: &crate::planner::LlmUsageStats) -> LlmStats {
        LlmStats {
            input_tokens: stats.input_tokens,
            output_tokens: stats.output_tokens,
            inference_time_ms: stats.inference_time_us / 1000,
            model: stats.model.clone(),
        }
    }

    /// Convert priority enum to string
    fn priority_to_string(priority: &crate::planner::StepPriority) -> String {
        match priority {
            crate::planner::StepPriority::Low => "low".to_string(),
            crate::planner::StepPriority::Normal => "normal".to_string(),
            crate::planner::StepPriority::High => "high".to_string(),
            crate::planner::StepPriority::Critical => "critical".to_string(),
        }
    }

    /// Convert plan status to string
    fn status_to_string(status: &crate::planner::PlanStatus) -> String {
        match status {
            crate::planner::PlanStatus::Generating => "generating".to_string(),
            crate::planner::PlanStatus::Ready => "ready".to_string(),
            crate::planner::PlanStatus::Executing { .. } => "executing".to_string(),
            crate::planner::PlanStatus::Completed { .. } => "completed".to_string(),
            crate::planner::PlanStatus::Failed { .. } => "failed".to_string(),
            crate::planner::PlanStatus::Cancelled { .. } => "cancelled".to_string(),
            crate::planner::PlanStatus::Paused { .. } => "paused".to_string(),
        }
    }

    /// Convert step status to string
    fn step_status_to_string(status: &crate::planner::PlanStepStatus) -> String {
        match status {
            crate::planner::PlanStepStatus::Pending => "pending".to_string(),
            crate::planner::PlanStepStatus::Executing { .. } => "executing".to_string(),
            crate::planner::PlanStepStatus::Completed { .. } => "completed".to_string(),
            crate::planner::PlanStepStatus::Failed { .. } => "failed".to_string(),
            crate::planner::PlanStepStatus::Skipped { .. } => "skipped".to_string(),
            crate::planner::PlanStepStatus::Blocked { .. } => "blocked".to_string(),
        }
    }

    /// Convert 90kHz timestamp to ISO 8601 string
    fn timestamp_to_iso8601(timestamp: u64) -> String {
        use std::time::{UNIX_EPOCH, Duration};
        
        // Convert from 90kHz timebase to nanoseconds
        let nanos = timestamp * 1_000_000_000 / 90_000;
        let duration = Duration::from_nanos(nanos);
        let sys_time = UNIX_EPOCH + duration;
        
        // Format as ISO 8601
        let datetime: chrono::DateTime<chrono::Utc> = sys_time.into();
        datetime.to_rfc3339()
    }
}

/// Generate API documentation examples
pub fn generate_api_examples() -> serde_json::Value {
    serde_json::json!({
        "endpoints": {
            "/ai/plan": {
                "method": "POST",
                "description": "Create a new task plan",
                "request_example": {
                    "goal": "Create a simple web application with user authentication",
                    "context": "Need to build a secure web app with login functionality",
                    "required_tools": ["npm", "node", "database"],
                    "priority": "normal",
                    "max_execution_time": 3600,
                    "tags": ["web", "development", "authentication"]
                },
                "response_example": {
                    "success": true,
                    "plan_id": "plan_123e4567-e89b-12d3-a456-426614174000",
                    "plan": {
                        "id": "plan_123e4567-e89b-12d3-a456-426614174000",
                        "name": "Plan for: Create a simple web application with user authentication",
                        "description": "Generated plan to accomplish: Create a simple web application with user authentication",
                        "user_goal": "Create a simple web application with user authentication",
                        "version": 1,
                        "priority": "normal",
                        "status": "ready",
                        "steps": [
                            {
                                "id": "step_1",
                                "name": "Analyze Create a simple web application with user authentication",
                                "description": "Analyze the requirements and constraints for: Create a simple web application with user authentication",
                                "step_type": "analysis",
                                "priority": "normal",
                                "status": "pending",
                                "dependencies": [],
                                "estimated_duration_secs": 180,
                                "tool_id": null,
                                "expected_outputs": [],
                                "sub_steps": []
                            }
                        ],
                        "tags": ["web", "development", "authentication"],
                        "creator": "ai-planner",
                        "created_at": "2025-09-23T10:30:00Z",
                        "updated_at": "2025-09-23T10:30:00Z",
                        "estimated_duration_secs": 900,
                        "actual_duration_secs": null,
                        "progress": 0.0
                    },
                    "error": null,
                    "generation_time_ms": 250,
                    "llm_stats": {
                        "input_tokens": 35,
                        "output_tokens": 100,
                        "inference_time_ms": 125,
                        "model": "llama3-8b"
                    }
                }
            },
            "/ai/plan/{plan_id}": {
                "method": "GET",
                "description": "Get a specific plan by ID",
                "response_example": {
                    "success": true,
                    "plan": {
                        "id": "plan_123e4567-e89b-12d3-a456-426614174000",
                        "name": "Plan for: Create a simple web application with user authentication",
                        "status": "ready",
                        "progress": 0.0
                    },
                    "error": null
                }
            },
            "/ai/plans": {
                "method": "GET",
                "description": "List all plans",
                "response_example": {
                    "success": true,
                    "plans": [
                        {
                            "id": "plan_123e4567-e89b-12d3-a456-426614174000",
                            "name": "Plan for: Create a simple web application with user authentication",
                            "user_goal": "Create a simple web application with user authentication",
                            "priority": "normal",
                            "status": "ready",
                            "created_at": "2025-09-23T10:30:00Z",
                            "step_count": 5,
                            "progress": 0.0
                        }
                    ],
                    "total_count": 1,
                    "error": null
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::AiPolicy;

    #[tokio::test]
    async fn test_planner_api_creation() {
        let policy = Arc::new(AiPolicy::default());
        let api = PlannerApi::new(policy);
        assert!(api.init().await.is_ok());
    }

    #[tokio::test]
    async fn test_create_plan_api() {
        let policy = Arc::new(AiPolicy::default());
        let api = PlannerApi::new(policy);
        api.init().await.unwrap();

        let request = CreatePlanRequest {
            goal: "Test goal".to_string(),
            context: Some("Test context".to_string()),
            required_tools: None,
            priority: Some("normal".to_string()),
            max_execution_time: None,
            tags: Some(vec!["test".to_string()]),
        };

        let response = api.create_plan(request).await;
        assert!(response.success);
        assert!(response.plan_id.is_some());
    }

    #[tokio::test]
    async fn test_list_plans_api() {
        let policy = Arc::new(AiPolicy::default());
        let api = PlannerApi::new(policy);
        api.init().await.unwrap();

        // Create a plan first
        let request = CreatePlanRequest {
            goal: "Test goal".to_string(),
            context: None,
            required_tools: None,
            priority: None,
            max_execution_time: None,
            tags: None,
        };
        let create_response = api.create_plan(request).await;
        assert!(create_response.success);

        // List plans
        let list_response = api.list_plans(None, None, None).await;
        assert!(list_response.success);
        assert!(list_response.total_count >= 1);
    }
}
