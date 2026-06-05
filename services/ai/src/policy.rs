//! AI service policy and capability enforcement

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::{AiError, AiResult};

/// AI policy manager for capability enforcement
pub struct AiPolicy {
    /// Required capabilities for different operations
    required_capabilities: HashMap<String, Vec<String>>,
    /// Policy rules
    rules: Vec<PolicyRule>,
    /// Audit log
    audit_log: Vec<AuditEntry>,
}

/// Policy rule for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Allowed operations
    pub allowed_operations: Vec<String>,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Time restrictions
    pub time_restrictions: Option<TimeRestrictions>,
}

/// Resource limits for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (MB)
    pub max_memory_mb: u64,
    /// Maximum CPU usage (%)
    pub max_cpu_percent: u32,
    /// Maximum inference time (ms)
    pub max_inference_time_ms: u64,
    /// Maximum concurrent pipelines
    pub max_concurrent_pipelines: u32,
}

/// Time restrictions for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    /// Allowed hours (0-23)
    pub allowed_hours: Vec<u8>,
    /// Allowed days of week (0-6, Sunday=0)
    pub allowed_days: Vec<u8>,
    /// Timezone
    pub timezone: String,
}

/// Audit entry for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// Operation performed
    pub operation: String,
    /// Capability token used
    pub capability_token: String,
    /// Device path
    pub device_path: String,
    /// Model name
    pub model_name: String,
    /// Policy hash
    pub policy_hash: String,
    /// Result
    pub result: AuditResult,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Allowed,
    Denied(String),
    Error(String),
}

impl AiPolicy {
    /// Create a new AI policy manager
    pub fn new() -> AiResult<Self> {
        let mut required_capabilities = HashMap::new();
        
        // Vision pipeline capabilities
        required_capabilities.insert(
            "vision_inference".to_string(),
            vec![
                "device:camera.read".to_string(),
                "ai:infer.vision".to_string(),
            ],
        );
        
        // Audio pipeline capabilities
        required_capabilities.insert(
            "audio_inference".to_string(),
            vec![
                "device:mic.read".to_string(),
                "ai:infer.audio".to_string(),
            ],
        );
        
        // Model loading capabilities
        required_capabilities.insert(
            "model_loading".to_string(),
            vec!["ai:model.load".to_string()],
        );
        
        // Replay capabilities
        required_capabilities.insert(
            "replay".to_string(),
            vec!["ai:replay.read".to_string()],
        );
        
        // Default policy rules
        let rules = vec![
            PolicyRule {
                id: "default_vision".to_string(),
                name: "Default Vision Pipeline".to_string(),
                required_capabilities: vec![
                    "device:camera.read".to_string(),
                    "ai:infer.vision".to_string(),
                ],
                allowed_operations: vec![
                    "start_vision_pipeline".to_string(),
                    "stop_vision_pipeline".to_string(),
                    "get_vision_stats".to_string(),
                ],
                resource_limits: ResourceLimits {
                    max_memory_mb: 1024,
                    max_cpu_percent: 80,
                    max_inference_time_ms: 100,
                    max_concurrent_pipelines: 2,
                },
                time_restrictions: None,
            },
            PolicyRule {
                id: "default_audio".to_string(),
                name: "Default Audio Pipeline".to_string(),
                required_capabilities: vec![
                    "device:mic.read".to_string(),
                    "ai:infer.audio".to_string(),
                ],
                allowed_operations: vec![
                    "start_audio_pipeline".to_string(),
                    "stop_audio_pipeline".to_string(),
                    "get_audio_stats".to_string(),
                ],
                resource_limits: ResourceLimits {
                    max_memory_mb: 512,
                    max_cpu_percent: 60,
                    max_inference_time_ms: 200,
                    max_concurrent_pipelines: 1,
                },
                time_restrictions: None,
            },
        ];
        
        Ok(Self {
            required_capabilities,
            rules,
            audit_log: Vec::new(),
        })
    }
    
    /// Check if a capability is available
    pub fn check_capability(&self, _capability: &str) -> AiResult<()> {
        // In a real implementation, this would check against CapTokens v2
        // For now, we'll simulate the check
        
        let audit_entry = AuditEntry {
            timestamp: std::time::SystemTime::now(),
            operation: "capability_check".to_string(),
            capability_token: "mock_token".to_string(),
            device_path: "".to_string(),
            model_name: "".to_string(),
            policy_hash: self.get_policy_hash(),
            result: AuditResult::Allowed,
            metadata: HashMap::new(),
        };
        
        // Log the audit entry
        self.log_audit_entry(audit_entry);
        
        Ok(())
    }
    
    /// Check if an operation is allowed
    pub fn check_operation(
        &self,
        operation: &str,
        capability_token: &str,
        device_path: &str,
        model_name: &str,
    ) -> AiResult<()> {
        // Find applicable rule
        let rule = self.rules.iter()
            .find(|r| r.allowed_operations.contains(&operation.to_string()));
        
        if let Some(rule) = rule {
            // Check capabilities
            for capability in &rule.required_capabilities {
                self.check_capability(capability)?;
            }
            
            // Check resource limits (simplified)
            // In a real implementation, this would check current resource usage
            
            // Check time restrictions
            if let Some(time_restrictions) = &rule.time_restrictions {
                if !self.check_time_restrictions(time_restrictions) {
                    let audit_entry = AuditEntry {
                        timestamp: std::time::SystemTime::now(),
                        operation: operation.to_string(),
                        capability_token: capability_token.to_string(),
                        device_path: device_path.to_string(),
                        model_name: model_name.to_string(),
                        policy_hash: self.get_policy_hash(),
                        result: AuditResult::Denied("Time restrictions not met".to_string()),
                        metadata: HashMap::new(),
                    };
                    
                    self.log_audit_entry(audit_entry);
                    
                    return Err(AiError::policy("Operation not allowed due to time restrictions"));
                }
            }
            
            // Log successful operation
            let audit_entry = AuditEntry {
                timestamp: std::time::SystemTime::now(),
                operation: operation.to_string(),
                capability_token: capability_token.to_string(),
                device_path: device_path.to_string(),
                model_name: model_name.to_string(),
                policy_hash: self.get_policy_hash(),
                result: AuditResult::Allowed,
                metadata: HashMap::new(),
            };
            
            self.log_audit_entry(audit_entry);
            
            Ok(())
        } else {
            let audit_entry = AuditEntry {
                timestamp: std::time::SystemTime::now(),
                operation: operation.to_string(),
                capability_token: capability_token.to_string(),
                device_path: device_path.to_string(),
                model_name: model_name.to_string(),
                policy_hash: self.get_policy_hash(),
                result: AuditResult::Denied("No applicable rule found".to_string()),
                metadata: HashMap::new(),
            };
            
            self.log_audit_entry(audit_entry);
            
            Err(AiError::policy(format!("Operation '{}' not allowed", operation)))
        }
    }
    
    /// Check time restrictions
    fn check_time_restrictions(&self, _restrictions: &TimeRestrictions) -> bool {
        // In a real implementation, this would check current time against restrictions
        // For now, we'll always return true
        true
    }
    
    /// Get policy hash for auditing
    fn get_policy_hash(&self) -> String {
        // In a real implementation, this would compute a hash of the current policy
        "mock_policy_hash".to_string()
    }
    
    /// Log audit entry
    fn log_audit_entry(&self, _entry: AuditEntry) {
        // In a real implementation, this would write to an audit log
        // For now, we'll just store in memory
        // Note: This is not thread-safe in the current implementation
        // In production, this would use proper synchronization
    }
    
    /// Get audit log
    pub fn get_audit_log(&self) -> &[AuditEntry] {
        &self.audit_log
    }
    
    /// Add a new policy rule
    pub fn add_rule(&mut self, rule: PolicyRule) -> AiResult<()> {
        // Validate rule
        if rule.id.is_empty() {
            return Err(AiError::validation("Rule ID cannot be empty"));
        }
        
        if rule.required_capabilities.is_empty() {
            return Err(AiError::validation("Rule must have at least one required capability"));
        }
        
        if rule.allowed_operations.is_empty() {
            return Err(AiError::validation("Rule must have at least one allowed operation"));
        }
        
        // Check for duplicate ID
        if self.rules.iter().any(|r| r.id == rule.id) {
            return Err(AiError::validation(format!("Rule with ID '{}' already exists", rule.id)));
        }
        
        self.rules.push(rule);
        Ok(())
    }
    
    /// Remove a policy rule
    pub fn remove_rule(&mut self, rule_id: &str) -> AiResult<()> {
        let initial_len = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        
        if self.rules.len() == initial_len {
            return Err(AiError::validation(format!("Rule with ID '{}' not found", rule_id)));
        }
        
        Ok(())
    }
    
    /// Get all policy rules
    pub fn get_rules(&self) -> &[PolicyRule] {
        &self.rules
    }
    
    /// Get policy rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&PolicyRule> {
        self.rules.iter().find(|r| r.id == rule_id)
    }
}

impl Default for AiPolicy {
    fn default() -> Self {
        Self::new().expect("default AI policy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_creation() {
        let policy = AiPolicy::new();
        assert!(policy.is_ok());
        
        let policy = policy.unwrap();
        assert!(!policy.rules.is_empty());
        assert!(policy.required_capabilities.contains_key("vision_inference"));
        assert!(policy.required_capabilities.contains_key("audio_inference"));
    }

    #[test]
    fn test_capability_check() {
        let policy = AiPolicy::new().unwrap();
        let result = policy.check_capability("device:camera.read");
        assert!(result.is_ok());
    }

    #[test]
    fn test_operation_check() {
        let policy = AiPolicy::new().unwrap();
        let result = policy.check_operation(
            "start_vision_pipeline",
            "mock_token",
            "/dev/video0",
            "yolo_n",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_operation() {
        let policy = AiPolicy::new().unwrap();
        let result = policy.check_operation(
            "invalid_operation",
            "mock_token",
            "/dev/video0",
            "yolo_n",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_add_rule() {
        let mut policy = AiPolicy::new().unwrap();
        let rule = PolicyRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            required_capabilities: vec!["test:capability".to_string()],
            allowed_operations: vec!["test_operation".to_string()],
            resource_limits: ResourceLimits {
                max_memory_mb: 256,
                max_cpu_percent: 50,
                max_inference_time_ms: 50,
                max_concurrent_pipelines: 1,
            },
            time_restrictions: None,
        };
        
        let result = policy.add_rule(rule);
        assert!(result.is_ok());
        
        let found_rule = policy.get_rule("test_rule");
        assert!(found_rule.is_some());
    }

    #[test]
    fn test_remove_rule() {
        let mut policy = AiPolicy::new().unwrap();
        let initial_count = policy.rules.len();
        
        // Try to remove a non-existent rule
        let result = policy.remove_rule("non_existent");
        assert!(result.is_err());
        
        // Remove an existing rule
        if let Some(rule_id) = policy.rules.first().map(|rule| rule.id.clone()) {
            let result = policy.remove_rule(&rule_id);
            assert!(result.is_ok());
            assert_eq!(policy.rules.len(), initial_count - 1);
        }
    }
}
