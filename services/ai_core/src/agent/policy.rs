//! Policy Enforcement, Safety Rails, and Human-in-the-Loop (HITL) support.
//!
//! ## HITL reject/modify (Track 2.1)
//! The executor calls `handle_hitl_decision()` before running a step that
//! requires approval.  The decision can be Accept, Reject, or Modify.
//! `Modify` does NOT mutate the original request (mini-castor invariant);
//! instead it creates a `ModifiedDecision` that pairs the original request
//! with the human feedback and triggers a re-plan.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::planner::TaskStep;
use crate::cap::CapTokenManager;
use crate::error::Result;
use crate::intents::SystemActionContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckResult {
    pub allowed: bool,
    pub requires_approval: bool,
    pub reason: String,
    pub matched_rules: Vec<String>,
}

pub struct PolicyEnforcer {
    cap_token_manager: Arc<CapTokenManager>,
    destructive_patterns: Vec<String>,
    max_file_size_bytes: u64,
    max_operations_per_plan: usize,
}

impl PolicyEnforcer {
    pub fn new(cap_token_manager: Arc<CapTokenManager>) -> Self {
        Self {
            cap_token_manager,
            destructive_patterns: vec![
                "delete",
                "remove",
                "rm",
                "drop",
                "truncate",
                "overwrite",
                "format",
                "wipe",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            max_file_size_bytes: 1024 * 1024 * 1024,
            max_operations_per_plan: 50,
        }
    }

    pub fn new_mock() -> Self {
        Self {
            cap_token_manager: Arc::new(CapTokenManager::new_mock()),
            destructive_patterns: vec!["delete", "remove"]
                .into_iter()
                .map(String::from)
                .collect(),
            max_file_size_bytes: 1024 * 1024 * 1024,
            max_operations_per_plan: 50,
        }
    }

    pub async fn check_step_execution(
        &self,
        step: &TaskStep,
        context: &SystemActionContext,
    ) -> Result<PolicyCheckResult> {
        // Checking policy for step

        let mut result = PolicyCheckResult {
            allowed: true,
            requires_approval: false,
            reason: String::new(),
            matched_rules: Vec::new(),
        };

        if step.is_destructive {
            result.requires_approval = true;
            result
                .matched_rules
                .push("destructive_action_rule".to_string());
            // Step flagged as destructive
        }

        if self.contains_dangerous_pattern(&step.parameters) {
            result.requires_approval = true;
            result
                .matched_rules
                .push("dangerous_pattern_rule".to_string());
            // Step contains dangerous pattern
        }

        let missing_capabilities: Vec<String> = step
            .required_capabilities
            .iter()
            .filter(|capability| !context_has_capability(context, capability))
            .cloned()
            .collect();
        if !missing_capabilities.is_empty() {
            result.requires_approval = true;
            result.reason = format!(
                "step '{}' needs capability approval for {}",
                step.step_id,
                missing_capabilities.join(", ")
            );
            result
                .matched_rules
                .push("capability_approval_rule".to_string());
        }

        if let Some(file_size) = self.extract_file_size(&step.parameters) {
            if file_size > self.max_file_size_bytes {
                result.allowed = false;
                result.reason = format!(
                    "File size {} exceeds limit {}",
                    file_size, self.max_file_size_bytes
                );
                result
                    .matched_rules
                    .push("file_size_limit_rule".to_string());
                return Ok(result);
            }
        }

        Ok(result)
    }

    fn contains_dangerous_pattern(&self, params: &serde_json::Value) -> bool {
        let params_str = params.to_string().to_lowercase();
        self.destructive_patterns
            .iter()
            .any(|p| params_str.contains(p))
    }

    fn extract_file_size(&self, params: &serde_json::Value) -> Option<u64> {
        if let Some(obj) = params.as_object() {
            for key in &["file_size", "size", "filesize", "max_size"] {
                if let Some(v) = obj.get(*key) {
                    if let Some(size) = v.as_u64() {
                        return Some(size);
                    }
                    if let Some(s) = v.as_str() {
                        if let Ok(size) = s.parse() {
                            return Some(size);
                        }
                    }
                }
            }
        }
        None
    }

    pub async fn validate_plan(
        &self,
        steps: &[TaskStep],
        context: &SystemActionContext,
    ) -> Result<PolicyCheckResult> {
        let mut result = PolicyCheckResult {
            allowed: true,
            requires_approval: false,
            reason: String::new(),
            matched_rules: Vec::new(),
        };

        if steps.len() > self.max_operations_per_plan {
            result.allowed = false;
            result.reason = format!(
                "Plan has {} steps, exceeds limit of {}",
                steps.len(),
                self.max_operations_per_plan
            );
            return Ok(result);
        }

        for step in steps {
            let step_result = self.check_step_execution(step, context).await?;
            if !step_result.allowed {
                return Ok(step_result);
            }
            if step_result.requires_approval {
                result.requires_approval = true;
                result.matched_rules.extend(step_result.matched_rules);
            }
        }

        Ok(result)
    }
}

/// HITL decision from a human operator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitlDecision {
    /// Accept the step as-is.
    Accept,
    /// Reject the step entirely.
    Reject { reason: String },
    /// Request modification — preserves the original request and pairs it
    /// with human feedback.  The planner should re-plan using this feedback.
    Modify { feedback: String },
}

/// A modified-decision record (mini-castor invariant: the original request is
/// never mutated).  This is logged as an audit trail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModifiedDecision {
    pub original_step: TaskStep,
    pub human_feedback: String,
    pub timestamp: u64,
}

impl PolicyEnforcer {
    /// Handle a HITL decision for a step that requires approval.
    /// Returns `Ok(true)` if the step may proceed, `Ok(false)` if rejected,
    /// and `Err` with modified feedback if the human requested a modification.
    pub fn handle_hitl_decision(
        &self,
        step: &TaskStep,
        decision: &HitlDecision,
    ) -> std::result::Result<bool, ModifiedDecision> {
        match decision {
            HitlDecision::Accept => {
                Ok(true)
            }
            HitlDecision::Reject { reason } => {
                crate::metrics::record_hitl_reject();
                eprintln!(
                    "[HITL] Step '{}' REJECTED: {}",
                    step.step_id, reason
                );
                Ok(false)
            }
            HitlDecision::Modify { feedback } => {
                crate::metrics::record_hitl_modify();
                let modified = ModifiedDecision {
                    original_step: step.clone(),
                    human_feedback: feedback.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };
                eprintln!(
                    "[HITL] Step '{}' MODIFY requested: {}",
                    step.step_id, feedback
                );
                Err(modified)
            }
        }
    }
}

fn context_has_capability(context: &SystemActionContext, capability: &str) -> bool {
    context
        .metadata
        .get("capabilities")
        .and_then(|value| value.as_array())
        .map(|caps| {
            caps.iter()
                .filter_map(|cap| cap.as_str())
                .any(|cap| cap == capability || cap == "admin")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_step() -> TaskStep {
        TaskStep {
            step_id: "step-001".to_string(),
            description: "test step".to_string(),
            tool_name: "echo".to_string(),
            parameters: serde_json::json!({"message": "hello"}),
            required_capabilities: Vec::new(),
            requires_approval: false,
            is_destructive: false,
            depends_on: Vec::new(),
            estimated_time_secs: 1,
        }
    }

    #[test]
    fn hitl_accept_allows() {
        let enforcer = PolicyEnforcer::new_mock();
        let result = enforcer.handle_hitl_decision(&test_step(), &HitlDecision::Accept);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn hitl_reject_blocks() {
        let enforcer = PolicyEnforcer::new_mock();
        let result = enforcer.handle_hitl_decision(
            &test_step(),
            &HitlDecision::Reject { reason: "too risky".into() },
        );
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn hitl_modify_preserves_original() {
        let enforcer = PolicyEnforcer::new_mock();
        let step = test_step();
        let result = enforcer.handle_hitl_decision(
            &step,
            &HitlDecision::Modify { feedback: "use safer path".into() },
        );
        let modified = result.unwrap_err();
        assert_eq!(modified.original_step.step_id, "step-001");
        assert_eq!(modified.human_feedback, "use safer path");
    }
}
