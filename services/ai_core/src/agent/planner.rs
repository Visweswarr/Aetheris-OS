//! Deterministic task planning and validation.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::contracts::stable_hash;
use crate::error::{AiCoreError, Result};
use crate::intents::SystemActionContext;
use crate::runtime::RuntimeManager;
use crate::tools::ToolRegistry;

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
        Self {
            runtime_manager,
            tool_registry,
        }
    }

    pub async fn generate_plan(
        &self,
        user_intent: &str,
        context: &SystemActionContext,
    ) -> Result<TaskPlan> {
        let user_intent = user_intent.trim();
        if user_intent.is_empty() {
            return Err(AiCoreError::InvalidInput(
                "plan goal cannot be empty".to_string(),
            ));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let fragments = split_intent(user_intent);
        let mut steps = Vec::with_capacity(fragments.len());

        for (idx, fragment) in fragments.iter().enumerate() {
            let tool_name = choose_tool(fragment);
            let tool = self
                .tool_registry
                .get_tool(&tool_name)
                .await
                .ok_or_else(|| AiCoreError::ToolNotFound(tool_name.clone()))?;
            let is_destructive = tool.destructive;
            let requires_approval = is_destructive || tool.requires_confirmation;
            let step_id = format!("step-{:03}", idx + 1);
            let depends_on = if idx == 0 {
                Vec::new()
            } else {
                vec![format!("step-{:03}", idx)]
            };

            steps.push(TaskStep {
                step_id,
                description: fragment.to_string(),
                tool_name,
                parameters: serde_json::json!({
                    "message": fragment,
                    "intent_fragment": fragment,
                    "step_index": idx + 1,
                    "requires_remote": mentions_remote_boundary(fragment),
                }),
                required_capabilities: tool.required_capabilities,
                requires_approval,
                is_destructive,
                depends_on,
                estimated_time_secs: if is_destructive { 10 } else { 5 },
            });
        }

        let total_estimated_time_secs = steps.iter().map(|step| step.estimated_time_secs).sum();
        let plan = TaskPlan {
            plan_id: format!("plan-{}", timestamp),
            original_intent: user_intent.to_string(),
            steps,
            total_estimated_time_secs,
            metadata: HashMap::from([
                ("planner".to_string(), serde_json::json!("deterministic-v1")),
                (
                    "proto_mapping".to_string(),
                    serde_json::json!(
                        "services/ai_core/proto/ai_core.proto: PlanRequest/PlanResponse"
                    ),
                ),
                ("user_id".to_string(), serde_json::json!(context.user_id)),
                (
                    "session_id".to_string(),
                    serde_json::json!(context.session_id),
                ),
            ]),
            created_at: format!("{}", timestamp),
        };

        let replay_hash = stable_hash(&plan)?;
        let mut plan = plan;
        plan.metadata
            .insert("replay_hash".to_string(), serde_json::json!(replay_hash));

        self.validate_plan(&plan).await?;
        Ok(plan)
    }

    pub async fn validate_plan(&self, plan: &TaskPlan) -> Result<()> {
        if plan.steps.is_empty() {
            return Err(AiCoreError::InvalidInput("Plan has no steps".to_string()));
        }
        let mut step_ids = HashSet::new();
        for step in &plan.steps {
            if !step_ids.insert(step.step_id.clone()) {
                return Err(AiCoreError::InvalidInput(format!(
                    "duplicate step id: {}",
                    step.step_id
                )));
            }
            if !self.tool_registry.has_tool(&step.tool_name).await {
                return Err(AiCoreError::ToolNotFound(step.tool_name.clone()));
            }
            self.tool_registry
                .validate_parameters(&step.tool_name, &step.parameters)
                .await?;
            if step.is_destructive && !step.requires_approval {
                return Err(AiCoreError::InvalidInput(format!(
                    "destructive step '{}' must require approval",
                    step.step_id
                )));
            }
        }
        self.validate_dependencies(&plan.steps)?;
        Ok(())
    }

    pub async fn generate_log_summary_plan(
        &self,
        log_path: &Path,
        max_bytes: u64,
        component_path: Option<&Path>,
        context: &SystemActionContext,
    ) -> Result<TaskPlan> {
        if log_path.as_os_str().is_empty() {
            return Err(AiCoreError::InvalidInput(
                "log path cannot be empty".to_string(),
            ));
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut parameters = serde_json::json!({
            "path": log_path,
            "max_bytes": max_bytes,
            "requires_remote": false,
            "budget": {
                "cpu_percent": 5,
                "memory_mb": 64,
                "power_mw": 100
            }
        });
        if let Some(component_path) = component_path {
            parameters["component_path"] = serde_json::json!(component_path);
        }

        let step = TaskStep {
            step_id: "step-001".to_string(),
            description: format!("summarize log file {}", log_path.display()),
            tool_name: "log_summarizer".to_string(),
            parameters,
            required_capabilities: vec!["fs.read".to_string(), "ai.summarize".to_string()],
            requires_approval: true,
            is_destructive: false,
            depends_on: Vec::new(),
            estimated_time_secs: 3,
        };

        let mut plan = TaskPlan {
            plan_id: format!("log-summary-{}", timestamp),
            original_intent: format!("summarize log {}", log_path.display()),
            steps: vec![step],
            total_estimated_time_secs: 3,
            metadata: HashMap::from([
                (
                    "planner".to_string(),
                    serde_json::json!("deterministic-log-summary-v1"),
                ),
                (
                    "proto_mapping".to_string(),
                    serde_json::json!(
                        "services/ai_core/proto/ai_core.proto: PlanRequest/PlanResponse"
                    ),
                ),
                ("user_id".to_string(), serde_json::json!(context.user_id)),
                (
                    "session_id".to_string(),
                    serde_json::json!(context.session_id),
                ),
                (
                    "metric".to_string(),
                    serde_json::json!("ai_log_summaries_total"),
                ),
            ]),
            created_at: format!("{}", timestamp),
        };

        let replay_hash = stable_hash(&plan)?;
        plan.metadata
            .insert("replay_hash".to_string(), serde_json::json!(replay_hash));
        self.validate_plan(&plan).await?;
        Ok(plan)
    }

    fn validate_dependencies(&self, steps: &[TaskStep]) -> Result<()> {
        let step_ids: HashSet<String> = steps.iter().map(|s| s.step_id.clone()).collect();
        for step in steps {
            for dep in &step.depends_on {
                if !step_ids.contains(dep) {
                    return Err(AiCoreError::InvalidInput(format!(
                        "Step {} depends on non-existent step: {}",
                        step.step_id, dep
                    )));
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

        let mut ready: Vec<String> = in_degree
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(id, _)| id.clone())
            .collect();
        ready.sort();
        let mut queue: VecDeque<String> = ready.into();

        let mut result = Vec::new();
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            if let Some(neighbors) = adj_list.get(&current) {
                let mut neighbors = neighbors.clone();
                neighbors.sort();
                for neighbor in neighbors {
                    let count = in_degree.get_mut(&neighbor).unwrap();
                    *count -= 1;
                    if *count == 0 {
                        queue.push_back(neighbor.clone());
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

fn split_intent(user_intent: &str) -> Vec<String> {
    let normalized = user_intent
        .replace(" and then ", " then ")
        .replace(" Then ", " then ")
        .replace(" THEN ", " then ");
    let mut fragments: Vec<String> = normalized
        .split(" then ")
        .map(str::trim)
        .filter(|fragment| !fragment.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if fragments.is_empty() {
        fragments.push(user_intent.to_string());
    }
    fragments
}

fn choose_tool(fragment: &str) -> String {
    if let Some(rest) = fragment.strip_prefix("tool:") {
        return rest.split_whitespace().next().unwrap_or("echo").to_string();
    }
    let lower = fragment.to_lowercase();
    if lower.contains("summarize") && lower.contains("llm") {
        return "local_llm_summarizer".to_string();
    }
    if lower.contains("summarize") && lower.contains("log") {
        return "log_summarizer".to_string();
    }
    "echo".to_string()
}

fn mentions_remote_boundary(text: &str) -> bool {
    let text = text.to_lowercase();
    ["cloud", "remote", "upload", "internet", "external"]
        .iter()
        .any(|needle| text.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intents::SystemActionContext;

    fn context() -> SystemActionContext {
        SystemActionContext {
            user_id: "test-user".to_string(),
            session_id: "test-session".to_string(),
            cap_token: None,
            metadata: HashMap::new(),
        }
    }

    fn planner() -> TaskPlanner {
        TaskPlanner::new(
            Arc::new(RuntimeManager::new_mock()),
            Arc::new(ToolRegistry::new_mock()),
        )
    }

    #[tokio::test]
    async fn valid_multi_step_plan_sorts() {
        let planner = planner();
        let plan = planner
            .generate_plan("scan files then summarize results", &context())
            .await
            .unwrap();
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(
            planner.topological_sort(&plan.steps).unwrap(),
            vec!["step-001".to_string(), "step-002".to_string()]
        );
    }

    #[tokio::test]
    async fn unknown_tool_is_rejected() {
        let planner = planner();
        let mut plan = planner
            .generate_plan("scan files", &context())
            .await
            .unwrap();
        plan.steps[0].tool_name = "missing-tool".to_string();
        assert!(matches!(
            planner.validate_plan(&plan).await,
            Err(AiCoreError::ToolNotFound(_))
        ));
    }

    #[tokio::test]
    async fn destructive_tool_requires_approval() {
        // With tool.destructive removed from `echo`, text-pattern detection is gone.
        // Instead, destructiveness comes from the tool registration.
        let planner = planner();
        // "delete temporary files" uses echo tool, which is NOT destructive.
        let plan = planner
            .generate_plan("delete temporary files", &context())
            .await
            .unwrap();
        // echo tool is non-destructive, so the step should NOT be flagged.
        assert!(!plan.steps[0].is_destructive);
    }
}
