use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use crate::policy::schema::{PlanPreviewV1, PlanDiffV1, ActionV1, EditSpec, PathSpec};

/// Simulation engine for policy evaluation
pub struct SimulationEngine {
    /// Maximum diff size
    max_diff_size: usize,
    /// Whether to enable detailed logging
    verbose: bool,
}

impl SimulationEngine {
    /// Create a new simulation engine
    pub fn new(max_diff_size: usize) -> Self {
        Self {
            max_diff_size,
            verbose: false,
        }
    }

    /// Set verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Simulate plan execution and produce diff
    pub fn simulate(
        &self,
        preview: &PlanPreviewV1,
        wm_snapshot: u64,
        caps: &[u64],
    ) -> PlanDiffV1 {
        if self.verbose {
            // In a real implementation, this would log simulation steps
        }

        let mut diff = PlanDiffV1 {
            adds: Vec::new(),
            removes: Vec::new(),
            edits: Vec::new(),
        };

        // Simulate what would happen when executing the plan
        for (index, action) in preview.actions.iter().enumerate() {
            match self.simulate_action(action, wm_snapshot, caps) {
                ActionSimResult::Add => {
                    diff.adds.push(action.clone());
                }
                ActionSimResult::Remove => {
                    diff.removes.push(index as u32);
                }
                ActionSimResult::Edit(new_action) => {
                    diff.edits.push(EditSpec {
                        action_index: index as u32,
                        new_action,
                    });
                }
                ActionSimResult::Keep => {
                    // Action remains unchanged
                }
            }
        }

        // Ensure diff size is within limits
        if diff.total_size() > self.max_diff_size {
            self.truncate_diff(&mut diff);
        }

        diff
    }

    /// Apply redactions to a plan diff
    pub fn apply_redactions(&self, diff: &mut PlanDiffV1, redactions: &[PathSpec]) {
        for redaction in redactions {
            self.apply_redaction(diff, redaction);
        }
    }

    /// Simulate a single action
    fn simulate_action(
        &self,
        action: &ActionV1,
        _wm_snapshot: u64,
        _caps: &[u64],
    ) -> ActionSimResult {
        // Simple simulation logic based on action kind
        match action.kind {
            1 => ActionSimResult::Add,      // Create action
            2 => ActionSimResult::Remove,   // Delete action
            3 => ActionSimResult::Edit(ActionV1 {
                kind: action.kind,
                params: self.sanitize_params(&action.params),
            }),
            _ => ActionSimResult::Keep,     // Default: keep unchanged
        }
    }

    /// Sanitize action parameters
    fn sanitize_params(&self, params: &BTreeMap<String, String>) -> BTreeMap<String, String> {
        let mut sanitized = BTreeMap::new();
        
        for (key, value) in params {
            if !key.contains("secret") && !key.contains("password") {
                sanitized.insert(key.clone(), value.clone());
            } else {
                sanitized.insert(key.clone(), "[REDACTED]".to_string());
            }
        }
        
        sanitized
    }

    /// Apply a single redaction
    fn apply_redaction(&self, diff: &mut PlanDiffV1, redaction: &PathSpec) {
        // Parse the path specification
        if let Some((action_index, field_path)) = self.parse_path(&redaction.path) {
            if action_index < diff.adds.len() as u32 {
                self.redact_action_field(&mut diff.adds[action_index as usize], field_path);
            }
            
            // Check edits
            for edit in &mut diff.edits {
                if edit.action_index == action_index {
                    self.redact_action_field(&mut edit.new_action, field_path);
                }
            }
        }
    }

    /// Parse path specification
    fn parse_path(&self, path: &str) -> Option<(u32, &str)> {
        // Simple path parsing for "actions[N].field" format
        if path.starts_with("actions[") {
            if let Some(end_bracket) = path.find(']') {
                if let Some(dot_pos) = path[end_bracket..].find('.') {
                    let index_str = &path[8..end_bracket];
                    if let Ok(index) = index_str.parse::<u32>() {
                        let field_path = &path[end_bracket + dot_pos + 1..];
                        return Some((index, field_path));
                    }
                }
            }
        }
        None
    }

    /// Redact a field in an action
    fn redact_action_field(&self, action: &mut ActionV1, field_path: &str) {
        match field_path {
            "params.secret" | "params.password" => {
                if let Some(value) = action.params.get_mut("secret") {
                    *value = "[REDACTED]".to_string();
                }
                if let Some(value) = action.params.get_mut("password") {
                    *value = "[REDACTED]".to_string();
                }
            }
            "kind" => {
                action.kind = 0; // Redact action kind
            }
            _ => {
                // Unknown field, no redaction
            }
        }
    }

    /// Truncate diff to fit size limits
    fn truncate_diff(&self, diff: &mut PlanDiffV1) {
        while diff.total_size() > self.max_diff_size {
            if !diff.adds.is_empty() {
                diff.adds.pop();
            } else if !diff.removes.is_empty() {
                diff.removes.pop();
            } else if !diff.edits.is_empty() {
                diff.edits.pop();
            } else {
                break;
            }
        }
    }
}

/// Result of simulating an action
#[derive(Debug, Clone)]
enum ActionSimResult {
    /// Action should be added
    Add,
    /// Action should be removed
    Remove,
    /// Action should be edited
    Edit(ActionV1),
    /// Action should remain unchanged
    Keep,
}

impl PlanDiffV1 {
    /// Calculate total size of the diff
    pub fn total_size(&self) -> usize {
        let adds_size: usize = self.adds.iter().map(|a| a.total_size()).sum();
        let removes_size = self.removes.len() * 4; // u32 per remove
        let edits_size: usize = self.edits.iter().map(|e| e.total_size()).sum();
        
        adds_size + removes_size + edits_size
    }

    /// Check if diff is empty
    pub fn is_empty(&self) -> bool {
        self.adds.is_empty() && self.removes.is_empty() && self.edits.is_empty()
    }

    /// Get number of changes
    pub fn change_count(&self) -> usize {
        self.adds.len() + self.removes.len() + self.edits.len()
    }
}

impl ActionV1 {
    /// Calculate size of action
    pub fn total_size(&self) -> usize {
        let kind_size = 2; // u16
        let params_size: usize = self.params.iter()
            .map(|(k, v)| k.len() + v.len() + 2) // key + value + overhead
            .sum();
        
        kind_size + params_size
    }
}

impl EditSpec {
    /// Calculate size of edit spec
    pub fn total_size(&self) -> usize {
        4 + self.new_action.total_size() // u32 + ActionV1
    }
}

/// Create a simulation engine with default settings
pub fn create_simulation_engine() -> SimulationEngine {
    SimulationEngine::new(32 * 1024) // 32 KiB default limit
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;

    #[test]
    fn test_simulation_engine_creation() {
        let engine = SimulationEngine::new(1024);
        assert_eq!(engine.max_diff_size, 1024);
        assert!(!engine.verbose);
    }

    #[test]
    fn test_plan_diff_size_calculation() {
        let diff = PlanDiffV1 {
            adds: vec![ActionV1 { kind: 1, params: BTreeMap::new() }],
            removes: vec![0, 1],
            edits: vec![],
        };
        
        assert!(!diff.is_empty());
        assert_eq!(diff.change_count(), 3);
        assert!(diff.total_size() > 0);
    }

    #[test]
    fn test_action_sanitization() {
        let mut params = BTreeMap::new();
        params.insert("normal".to_string(), "value".to_string());
        params.insert("secret".to_string(), "password123".to_string());
        
        let action = ActionV1 { kind: 1, params };
        let sanitized = SimulationEngine::new(1024).sanitize_params(&action.params);
        
        assert_eq!(sanitized.get("normal"), Some(&"value".to_string()));
        assert_eq!(sanitized.get("secret"), Some(&"[REDACTED]".to_string()));
    }

    #[test]
    fn test_path_parsing() {
        let engine = SimulationEngine::new(1024);
        
        let result = engine.parse_path("actions[5].params.secret");
        assert_eq!(result, Some((5, "params.secret")));
        
        let result = engine.parse_path("invalid_path");
        assert_eq!(result, None);
    }

    #[test]
    fn test_redaction_application() {
        let mut diff = PlanDiffV1 {
            adds: vec![ActionV1 {
                kind: 1,
                params: {
                    let mut params = BTreeMap::new();
                    params.insert("secret".to_string(), "value".to_string());
                    params
                },
            }],
            removes: vec![],
            edits: vec![],
        };
        
        let redactions = vec![PathSpec { path: "actions[0].params.secret".to_string() }];
        let engine = SimulationEngine::new(1024);
        
        engine.apply_redactions(&mut diff, &redactions);
        
        assert_eq!(diff.adds[0].params.get("secret"), Some(&"[REDACTED]".to_string()));
    }
}
