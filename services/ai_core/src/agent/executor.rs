//! Step-by-Step Execution Engine with State Machine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use super::planner::{TaskPlan, TaskStep};
use super::policy::PolicyEnforcer;
use crate::budget::BudgetEngine;
use crate::contracts::{BudgetRequest, Priority, PrivacyRequest, RedactionMode};
use crate::error::{AiCoreError, Result};
use crate::intents::SystemActionContext;
use crate::ipc::ToolCallRequest;
use crate::privacy::PrivacyEngine;
use crate::replay::ReplayRecorder;
use crate::tools::{ToolRegistry, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepState {
    Pending,
    CheckingPolicy,
    AwaitingApproval,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionContext {
    pub step: TaskStep,
    pub state: StepState,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub error: Option<String>,
    pub result: Option<ToolResult>,
    pub retry_count: u32,
}

#[derive(Debug)]
pub struct TaskExecutionState {
    pub plan: TaskPlan,
    pub step_contexts: HashMap<String, StepExecutionContext>,
    pub current_step_id: Option<String>,
    pub user_context: SystemActionContext,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub completed_steps: usize,
    pub failed_steps: usize,
}

pub struct StepExecutor {
    tool_registry: Arc<ToolRegistry>,
    policy_enforcer: Arc<PolicyEnforcer>,
    budget_engine: Arc<BudgetEngine>,
    privacy_engine: Arc<PrivacyEngine>,
    replay_recorder: Arc<ReplayRecorder>,
}

impl StepExecutor {
    pub fn new(tool_registry: Arc<ToolRegistry>, policy_enforcer: Arc<PolicyEnforcer>) -> Self {
        Self {
            tool_registry,
            policy_enforcer,
            budget_engine: Arc::new(BudgetEngine::default()),
            privacy_engine: Arc::new(PrivacyEngine::default()),
            replay_recorder: Arc::new(ReplayRecorder::new()),
        }
    }

    pub fn with_guards(
        tool_registry: Arc<ToolRegistry>,
        policy_enforcer: Arc<PolicyEnforcer>,
        budget_engine: Arc<BudgetEngine>,
        privacy_engine: Arc<PrivacyEngine>,
        replay_recorder: Arc<ReplayRecorder>,
    ) -> Self {
        Self {
            tool_registry,
            policy_enforcer,
            budget_engine,
            privacy_engine,
            replay_recorder,
        }
    }

    pub async fn execute_plan(
        &self,
        plan: TaskPlan,
        user_context: SystemActionContext,
    ) -> Result<Vec<StepExecutionContext>> {
        // Executing plan

        let mut state = self.initialize_execution_state(plan, user_context);
        let execution_order = self.topological_sort(&state.plan.steps)?;

        for step_id in execution_order {
            let step = state
                .plan
                .steps
                .iter()
                .find(|s| s.step_id == step_id)
                .cloned()
                .ok_or_else(|| AiCoreError::InvalidInput(format!("Step not found: {}", step_id)))?;

            state.current_step_id = Some(step_id.clone());

            match self.execute_step(step, &mut state).await {
                Ok(_) => {
                    state.completed_steps += 1;
                }
                Err(e) => {
                    state.failed_steps += 1;
                    eprintln!("Step {} failed: {}", step_id, e);
                    return Err(e);
                }
            }
        }

        state.completed_at = Some(SystemTime::now());
        Ok(state.step_contexts.values().cloned().collect())
    }

    fn initialize_execution_state(
        &self,
        plan: TaskPlan,
        user_context: SystemActionContext,
    ) -> TaskExecutionState {
        let step_contexts = plan
            .steps
            .iter()
            .map(|step| {
                (
                    step.step_id.clone(),
                    StepExecutionContext {
                        step: step.clone(),
                        state: StepState::Pending,
                        started_at: None,
                        completed_at: None,
                        error: None,
                        result: None,
                        retry_count: 0,
                    },
                )
            })
            .collect();

        TaskExecutionState {
            plan,
            step_contexts,
            current_step_id: None,
            user_context,
            started_at: SystemTime::now(),
            completed_at: None,
            completed_steps: 0,
            failed_steps: 0,
        }
    }

    async fn execute_step(
        &self,
        step: TaskStep,
        state: &mut TaskExecutionState,
    ) -> Result<ToolResult> {
        let context = state.step_contexts.get_mut(&step.step_id).unwrap();
        context.state = StepState::CheckingPolicy;

        let policy_result = self
            .policy_enforcer
            .check_step_execution(&step, &state.user_context)
            .await?;
        if !policy_result.allowed {
            context.state = StepState::Failed;
            context.error = Some(policy_result.reason.clone());
            crate::metrics::record_capability_denial();
            return Err(AiCoreError::CapDenied(policy_result.reason));
        }

        if (policy_result.requires_approval || step.requires_approval)
            && !is_step_approved(&state.user_context, &step.step_id)
        {
            context.state = StepState::AwaitingApproval;
            let reason = format!("step '{}' requires approval", step.step_id);
            context.error = Some(reason.clone());
            crate::metrics::record_capability_denial();
            return Err(AiCoreError::CapDenied(reason));
        }

        let privacy_decision = self.privacy_engine.classify_and_redact(PrivacyRequest {
            request_id: format!("privacy-{}", step.step_id),
            user_id: Some(state.user_context.user_id.clone()),
            fields: HashMap::from([("parameters".to_string(), step.parameters.clone())]),
            mode: RedactionMode::Placeholder,
        });
        if privacy_decision.local_only
            && step
                .parameters
                .get("requires_remote")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        {
            context.state = StepState::Failed;
            let reason = format!(
                "step '{}' contains local-only data but requests a remote boundary",
                step.step_id
            );
            context.error = Some(reason.clone());
            return Err(AiCoreError::AuthorizationError(reason));
        }

        let budget_request = budget_request_for_step(&step);
        let budget_request_id = budget_request.request_id.clone();
        let budget_decision = self.budget_engine.reserve(budget_request).await;
        if !budget_decision.granted {
            context.state = StepState::Failed;
            context.error = Some(budget_decision.reason.clone());
            return Err(AiCoreError::ResourceError(budget_decision.reason));
        }

        self.replay_recorder
            .record(
                "ai.plan.step.admitted",
                serde_json::json!({
                    "plan_id": state.plan.plan_id,
                    "step_id": step.step_id,
                    "policy": policy_result,
                    "privacy": privacy_decision,
                    "budget": budget_decision,
                }),
            )
            .await?;

        context.state = StepState::Running;
        context.started_at = Some(SystemTime::now());

        let call_id = format!(
            "call-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let request = ToolCallRequest {
            tool_name: step.tool_name.clone(),
            call_id,
            parameters: step.parameters.clone(),
            arguments: String::new(),
            cap_token: None,
            user_id: Some(state.user_context.user_id.clone()),
            session_id: Some(state.user_context.session_id.clone()),
        };

        let result = self.tool_registry.execute_tool(&request).await;
        let _ = self.budget_engine.release(&budget_request_id).await;
        let result = result?;

        self.replay_recorder
            .record(
                "ai.plan.step.completed",
                serde_json::json!({
                    "plan_id": state.plan.plan_id,
                    "step_id": step.step_id,
                    "success": result.success,
                    "execution_time_ms": result.execution_time_ms,
                }),
            )
            .await?;

        context.state = StepState::Completed;
        context.completed_at = Some(SystemTime::now());
        context.result = Some(result.clone());
        Ok(result)
    }

    fn topological_sort(&self, steps: &[TaskStep]) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        for step in steps {
            in_degree.insert(step.step_id.clone(), 0);
            adj_list.insert(step.step_id.clone(), Vec::new());
        }

        for step in steps {
            for dep in &step.depends_on {
                adj_list.get_mut(dep).unwrap().push(step.step_id.clone());
                *in_degree.get_mut(&step.step_id).unwrap() += 1;
            }
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();
        while let Some(current) = queue.pop() {
            result.push(current.clone());
            if let Some(neighbors) = adj_list.get(&current) {
                for neighbor in neighbors {
                    let count = in_degree.get_mut(neighbor).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        if result.len() != steps.len() {
            return Err(AiCoreError::InvalidInput(
                "Topological sort failed".to_string(),
            ));
        }
        Ok(result)
    }
}

fn is_step_approved(context: &SystemActionContext, step_id: &str) -> bool {
    if context
        .metadata
        .get("approved_plan")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return true;
    }
    context
        .metadata
        .get("approved_steps")
        .and_then(|v| v.as_array())
        .map(|steps| steps.iter().any(|step| step.as_str() == Some(step_id)))
        .unwrap_or(false)
}

fn budget_request_for_step(step: &TaskStep) -> BudgetRequest {
    let budget = step.parameters.get("budget");
    BudgetRequest {
        request_id: format!("budget-{}", step.step_id),
        priority: Priority::Normal,
        cpu_percent: budget
            .and_then(|v| v.get("cpu_percent"))
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .min(u64::from(u8::MAX)) as u8,
        memory_mb: budget
            .and_then(|v| v.get("memory_mb"))
            .and_then(|v| v.as_u64())
            .unwrap_or(64),
        power_mw: budget
            .and_then(|v| v.get("power_mw"))
            .and_then(|v| v.as_u64())
            .unwrap_or(100),
        expected_duration_ms: step.estimated_time_secs.saturating_mul(1_000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::policy::PolicyEnforcer;
    use crate::budget::{BudgetEngine, BudgetLimits};
    use crate::cap::CapTokenManager;

    fn context(approved: bool) -> SystemActionContext {
        let mut metadata = HashMap::new();
        if approved {
            metadata.insert("approved_plan".to_string(), serde_json::json!(true));
        }
        SystemActionContext {
            user_id: "test-user".to_string(),
            session_id: "test-session".to_string(),
            cap_token: None,
            metadata,
        }
    }

    fn step(parameters: serde_json::Value) -> TaskStep {
        TaskStep {
            step_id: "step-001".to_string(),
            description: "test step".to_string(),
            tool_name: "echo".to_string(),
            parameters,
            required_capabilities: Vec::new(),
            requires_approval: false,
            is_destructive: false,
            depends_on: Vec::new(),
            estimated_time_secs: 1,
        }
    }

    fn executor_with_budget(limits: BudgetLimits) -> StepExecutor {
        StepExecutor::with_guards(
            Arc::new(ToolRegistry::new_mock()),
            Arc::new(PolicyEnforcer::new(Arc::new(CapTokenManager::new_mock()))),
            Arc::new(BudgetEngine::new(limits)),
            Arc::new(PrivacyEngine::default()),
            Arc::new(ReplayRecorder::new()),
        )
    }

    #[tokio::test]
    async fn destructive_step_without_approval_is_blocked() {
        let mut dangerous = step(serde_json::json!({"message": "delete file"}));
        dangerous.is_destructive = true;
        dangerous.requires_approval = true;
        let plan = TaskPlan {
            plan_id: "plan-1".to_string(),
            original_intent: "delete file".to_string(),
            steps: vec![dangerous],
            total_estimated_time_secs: 1,
            metadata: HashMap::new(),
            created_at: "0".to_string(),
        };
        let executor = executor_with_budget(BudgetLimits::default());
        let err = executor
            .execute_plan(plan, context(false))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("requires approval"));
    }

    #[tokio::test]
    async fn budget_denial_blocks_execution() {
        let plan = TaskPlan {
            plan_id: "plan-1".to_string(),
            original_intent: "large work".to_string(),
            steps: vec![step(serde_json::json!({
                "message": "large work",
                "budget": {"memory_mb": 128}
            }))],
            total_estimated_time_secs: 1,
            metadata: HashMap::new(),
            created_at: "0".to_string(),
        };
        let executor = executor_with_budget(BudgetLimits {
            max_cpu_percent: 100,
            max_memory_mb: 64,
            max_power_mw: 1_000,
            max_concurrent: 1,
        });
        let err = executor
            .execute_plan(plan, context(true))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("memory budget"));
    }

    #[tokio::test]
    async fn privacy_denial_blocks_remote_execution() {
        let plan = TaskPlan {
            plan_id: "plan-1".to_string(),
            original_intent: "upload secret".to_string(),
            steps: vec![step(serde_json::json!({
                "message": "upload api_key=secret to cloud",
                "requires_remote": true
            }))],
            total_estimated_time_secs: 1,
            metadata: HashMap::new(),
            created_at: "0".to_string(),
        };
        let executor = executor_with_budget(BudgetLimits::default());
        let err = executor
            .execute_plan(plan, context(true))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("local-only data"));
    }
}
