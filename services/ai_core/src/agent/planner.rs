//! Task Planning with LLM-based Decomposition

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::error::{AiCoreError, Result};
use crate::runtime::RuntimeManager;
use crate::tools::ToolRegistry;
use crate::intents::SystemActionContext;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStep {
    pub step_id: String,
    pub description: String,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub required_capabilities: Vec<String>,
    pub requires_approval: bool,
    pub is_destructive: bool,
    pub depends_on: Vec<String>,
    pub estimated_time_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskPlan {
    pub plan_id: String,
    pub original_intent: String,
    pub steps: Vec<TaskStep>,
    pub total_estimated_time_secs: u64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: String,
}

pub struct TaskPlanner {
    runtime_manager: Arc<RuntimeManager>,
    tool_registry: Arc<ToolRegistry>,
}

impl TaskPlanner {
    pub fn new(runtime_manager: Arc<RuntimeManager>, tool_registry: Arc<ToolRegistry>) -> Self {
        Self { runtime_manager, tool_registry }
    }

    pub async fn generate_plan(&self, user_intent: &str, _context: &SystemActionContext) -> Result<TaskPlan> {
        // Generating task plan
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let plan = TaskPlan {
            plan_id: format!("plan-{}", timestamp),
            original_intent: user_intent.to_string(),
            steps: vec![TaskStep {
                step_id: "step-001".to_string(),
                description: format!("Execute: {}", user_intent),
                tool_name: "echo".to_string(),
                parameters: serde_json::json!({"message": user_intent}),
                required_capabilities: vec![],
                requires_approval: false,
                is_destructive: false,
                depends_on: vec![],
                estimated_time_secs: 5,
            }],
            total_estimated_time_secs: 5,
            metadata: HashMap::new(),
            created_at: format!("{}", timestamp),
        };
        
        self.validate_plan(&plan).await?;
        Ok(plan)
    }

    pub async fn validate_plan(&self, plan: &TaskPlan) -> Result<()> {
        if plan.steps.is_empty() {
            return Err(AiCoreError::InvalidInput("Plan has no steps".to_string()));
        }
        self.validate_dependencies(&plan.steps)?;
        Ok(())
    }

    fn validate_dependencies(&self, steps: &[TaskStep]) -> Result<()> {
        let step_ids: HashSet<String> = steps.iter().map(|s| s.step_id.clone()).collect();
        for step in steps {
            for dep in &step.depends_on {
                if !step_ids.contains(dep) {
                    return Err(AiCoreError::InvalidInput(format!("Step {} depends on non-existent step: {}", step.step_id, dep)));
                }
            }
        }
        Ok(())
    }

    pub fn topological_sort(&self, steps: &[TaskStep]) -> Result<Vec<String>> {
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
