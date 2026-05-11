//! Policy Enforcement and Safety Rails

use std::sync::Arc;
use serde::{Deserialize, Serialize};


use crate::error::Result;
use crate::cap::CapTokenManager;
use crate::intents::SystemActionContext;
use super::planner::TaskStep;

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
            destructive_patterns: vec!["delete", "remove", "rm", "drop", "truncate", "overwrite", "format", "wipe"]
                .into_iter().map(String::from).collect(),
            max_file_size_bytes: 1024 * 1024 * 1024,
            max_operations_per_plan: 50,
        }
    }

    pub fn new_mock() -> Self {
        Self {
            cap_token_manager: Arc::new(CapTokenManager::new_mock()),
            destructive_patterns: vec!["delete", "remove"].into_iter().map(String::from).collect(),
            max_file_size_bytes: 1024 * 1024 * 1024,
            max_operations_per_plan: 50,
        }
    }

    pub async fn check_step_execution(&self, step: &TaskStep, _context: &SystemActionContext) -> Result<PolicyCheckResult> {
        // Checking policy for step
        
        let mut result = PolicyCheckResult {
            allowed: true,
            requires_approval: false,
            reason: String::new(),
            matched_rules: Vec::new(),
        };
        
        if step.is_destructive {
            result.requires_approval = true;
            result.matched_rules.push("destructive_action_rule".to_string());
            // Step flagged as destructive
        }
        
        if self.contains_dangerous_pattern(&step.parameters) {
            result.requires_approval = true;
            result.matched_rules.push("dangerous_pattern_rule".to_string());
            // Step contains dangerous pattern
        }
        
        if let Some(file_size) = self.extract_file_size(&step.parameters) {
            if file_size > self.max_file_size_bytes {
                result.allowed = false;
                result.reason = format!("File size {} exceeds limit {}", file_size, self.max_file_size_bytes);
                result.matched_rules.push("file_size_limit_rule".to_string());
                return Ok(result);
            }
        }
        
        Ok(result)
    }

    fn contains_dangerous_pattern(&self, params: &serde_json::Value) -> bool {
        let params_str = params.to_string().to_lowercase();
        self.destructive_patterns.iter().any(|p| params_str.contains(p))
    }

    fn extract_file_size(&self, params: &serde_json::Value) -> Option<u64> {
        if let Some(obj) = params.as_object() {
            for key in &["file_size", "size", "filesize", "max_size"] {
                if let Some(v) = obj.get(*key) {
                    if let Some(size) = v.as_u64() { return Some(size); }
                    if let Some(s) = v.as_str() { if let Ok(size) = s.parse() { return Some(size); } }
                }
            }
        }
        None
    }

    pub async fn validate_plan(&self, steps: &[TaskStep], context: &SystemActionContext) -> Result<PolicyCheckResult> {
        let mut result = PolicyCheckResult {
            allowed: true, requires_approval: false, reason: String::new(), matched_rules: Vec::new(),
        };
        
        if steps.len() > self.max_operations_per_plan {
            result.allowed = false;
            result.reason = format!("Plan has {} steps, exceeds limit of {}", steps.len(), self.max_operations_per_plan);
            return Ok(result);
        }
        
        for step in steps {
            let step_result = self.check_step_execution(step, context).await?;
            if !step_result.allowed { return Ok(step_result); }
            if step_result.requires_approval {
                result.requires_approval = true;
                result.matched_rules.extend(step_result.matched_rules);
            }
        }
        
        Ok(result)
    }
}
