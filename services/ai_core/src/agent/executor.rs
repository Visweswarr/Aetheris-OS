//! Step-by-Step Execution Engine with State Machine

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

use crate::error::{AiCoreError, Result};
use crate::tools::{ToolRegistry, ToolResult};
use crate::ipc::ToolCallRequest;
use crate::intents::SystemActionContext;
use super::planner::{TaskPlan, TaskStep};
use super::policy::PolicyEnforcer;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepState { Pending, CheckingPolicy, AwaitingApproval, Running, Completed, Failed, Cancelled, Retrying }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionContext {
    pub step: TaskStep,
    pub state: StepState,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub error: Option<String>,
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
}

impl StepExecutor {
    pub fn new(tool_registry: Arc<ToolRegistry>, policy_enforcer: Arc<PolicyEnforcer>) -> Self {
        Self { tool_registry, policy_enforcer }
    }

    pub async fn execute_plan(&self, plan: TaskPlan, user_context: SystemActionContext) -> Result<Vec<StepExecutionContext>> {
        // Executing plan
        
        let mut state = self.initialize_execution_state(plan, user_context);
        let execution_order = self.topological_sort(&state.plan.steps)?;
        
        for step_id in execution_order {
            let step = state.plan.steps.iter().find(|s| s.step_id == step_id).cloned()
                .ok_or_else(|| AiCoreError::InvalidInput(format!("Step not found: {}", step_id)))?;
            
            state.current_step_id = Some(step_id.clone());
            
            match self.execute_step(step, &mut state).await {
                Ok(_) => { state.completed_steps += 1; }
                Err(e) => {
                    state.failed_steps += 1;
                    eprintln!("Step {} failed: {}", step_id, e);
                    let ctx = state.step_contexts.get(&step_id).unwrap();
                    if ctx.step.is_destructive { return Err(e); }
                }
            }
        }
        
        state.completed_at = Some(SystemTime::now());
        Ok(state.step_contexts.values().cloned().collect())
    }

    fn initialize_execution_state(&self, plan: TaskPlan, user_context: SystemActionContext) -> TaskExecutionState {
        let step_contexts = plan.steps.iter().map(|step| {
            (step.step_id.clone(), StepExecutionContext {
                step: step.clone(), state: StepState::Pending,
                started_at: None, completed_at: None, error: None, retry_count: 0,
            })
        }).collect();
        
        TaskExecutionState {
            plan, step_contexts, current_step_id: None, user_context,
            started_at: SystemTime::now(), completed_at: None, completed_steps: 0, failed_steps: 0,
        }
    }

    async fn execute_step(&self, step: TaskStep, state: &mut TaskExecutionState) -> Result<ToolResult> {
        let context = state.step_contexts.get_mut(&step.step_id).unwrap();
        context.state = StepState::CheckingPolicy;
        
        let policy_result = self.policy_enforcer.check_step_execution(&step, &state.user_context).await?;
        if !policy_result.allowed {
            context.state = StepState::Failed;
            context.error = Some(policy_result.reason.clone());
            return Err(AiCoreError::CapDenied(policy_result.reason));
        }
        
        context.state = StepState::Running;
        context.started_at = Some(SystemTime::now());
        
        let call_id = format!("call-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let request = ToolCallRequest {
            tool_name: step.tool_name.clone(),
            call_id,
            parameters: step.parameters.clone(),
            arguments: String::new(),
            cap_token: None,
            user_id: Some(state.user_context.user_id.clone()),
            session_id: Some(state.user_context.session_id.clone()),
        };
        
        let result = self.tool_registry.execute_tool(&request).await?;
        context.state = StepState::Completed;
        context.completed_at = Some(SystemTime::now());
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
        
        let mut queue: Vec<String> = in_degree.iter()
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
                    if *count == 0 { queue.push(neighbor.clone()); }
                }
            }
        }
        
        if result.len() != steps.len() {
            return Err(AiCoreError::InvalidInput("Topological sort failed".to_string()));
        }
        Ok(result)
    }
}
