//! Policy Engine Service for Polymera OS
//! 
//! This service provides a WebAssembly-based policy engine that can evaluate
//! Rego policies in a sandboxed environment. It includes:
//! 
//! - Policy compilation from Rego to WASM
//! - Sandboxed policy execution
//! - Host function API for policy interaction
//! - Resource limits and security controls
//! - Comprehensive audit logging
//! - Performance monitoring and metrics

pub mod engine;
pub mod wasm_host;

// Re-export main types
pub use engine::{
    PolicyEngine,
    PolicyEngineConfig,
    PolicyEngineError,
    PolicyEngineStats,
    PolicyResult,
    EvaluationMetadata,
};

pub use wasm_host::{
    WasmHost,
    WasmHostError,
    ResourceLimits,
};

/// Policy service version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Policy service description
pub const DESCRIPTION: &str = "WebAssembly-based Policy Engine for Polymera OS";

/// Default policy evaluation timeout
pub const DEFAULT_EVALUATION_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);

/// Default maximum memory usage for policy evaluation
pub const DEFAULT_MAX_MEMORY_BYTES: usize = 64 * 1024 * 1024; // 64MB

/// Default policy cache size
pub const DEFAULT_POLICY_CACHE_SIZE: usize = 100;

/// Default audit log path
pub const DEFAULT_AUDIT_LOG_PATH: &str = "/var/log/polymera/policy_audit.log";

/// Policy evaluation context
#[derive(Debug, Clone)]
pub struct PolicyContext {
    /// Request ID for tracking
    pub request_id: String,
    /// User making the request
    pub user: Option<String>,
    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request source (IP, service, etc.)
    pub source: Option<String>,
    /// Additional context data
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for PolicyContext {
    fn default() -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            user: None,
            timestamp: chrono::Utc::now(),
            source: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl PolicyContext {
    /// Create a new policy context
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a policy context with user
    pub fn with_user(user: String) -> Self {
        Self {
            user: Some(user),
            ..Self::default()
        }
    }
    
    /// Create a policy context with source
    pub fn with_source(source: String) -> Self {
        Self {
            source: Some(source),
            ..Self::default()
        }
    }
    
    /// Add metadata to the context
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
    
    /// Set metadata value
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
}

/// Policy evaluation request
#[derive(Debug, Clone)]
pub struct PolicyRequest {
    /// Policy to evaluate
    pub policy_name: String,
    /// Input data for evaluation
    pub input: serde_json::Value,
    /// Rule to evaluate (optional, defaults to "allow")
    pub rule: Option<String>,
    /// Evaluation context
    pub context: PolicyContext,
}

impl PolicyRequest {
    /// Create a new policy request
    pub fn new(policy_name: String, input: serde_json::Value) -> Self {
        Self {
            policy_name,
            input,
            rule: None,
            context: PolicyContext::new(),
        }
    }
    
    /// Set the rule to evaluate
    pub fn with_rule(mut self, rule: String) -> Self {
        self.rule = Some(rule);
        self
    }
    
    /// Set the evaluation context
    pub fn with_context(mut self, context: PolicyContext) -> Self {
        self.context = context;
        self
    }
    
    /// Set the user in the context
    pub fn with_user(mut self, user: String) -> Self {
        self.context.user = Some(user);
        self
    }
    
    /// Set the source in the context
    pub fn with_source(mut self, source: String) -> Self {
        self.context.source = Some(source);
        self
    }
    
    /// Add metadata to the context
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.set_metadata(key, value);
        self
    }
}

/// Policy evaluation response
#[derive(Debug, Clone)]
pub struct PolicyResponse {
    /// Evaluation result
    pub result: PolicyResult,
    /// Request context
    pub context: PolicyContext,
    /// Evaluation metadata
    pub evaluation_metadata: EvaluationMetadata,
}

impl PolicyResponse {
    /// Create a new policy response
    pub fn new(result: PolicyResult, context: PolicyContext) -> Self {
        Self {
            result,
            context,
            evaluation_metadata: result.metadata.clone(),
        }
    }
    
    /// Check if the request was allowed
    pub fn is_allowed(&self) -> bool {
        self.result.allowed
    }
    
    /// Get the reason for the decision
    pub fn reason(&self) -> &str {
        &self.result.reason
    }
    
    /// Get the policy code
    pub fn code(&self) -> Option<&str> {
        self.result.code.as_deref()
    }
    
    /// Get the constraints
    pub fn constraints(&self) -> Option<&serde_json::Value> {
        self.result.constraints.as_ref()
    }
}

/// Policy service interface
pub trait PolicyService {
    /// Evaluate a policy request
    fn evaluate(&self, request: &PolicyRequest) -> Result<PolicyResponse, PolicyEngineError>;
    
    /// Load a policy from a file
    fn load_policy(&self, policy_path: &std::path::Path) -> Result<String, PolicyEngineError>;
    
    /// Reload a policy from disk
    fn reload_policy(&self, policy_path: &std::path::Path) -> Result<(), PolicyEngineError>;
    
    /// Validate a policy without execution
    fn validate_policy(&self, policy_path: &std::path::Path) -> Result<(), PolicyEngineError>;
    
    /// Get service statistics
    fn get_stats(&self) -> PolicyEngineStats;
    
    /// Clear the policy cache
    fn clear_cache(&self);
    
    /// Get cache information
    fn get_cache_info(&self) -> std::collections::HashMap<String, (std::time::Instant, u64)>;
}

/// Default policy service implementation
pub struct DefaultPolicyService {
    engine: PolicyEngine,
}

impl DefaultPolicyService {
    /// Create a new default policy service
    pub fn new(config: PolicyEngineConfig) -> Result<Self, PolicyEngineError> {
        let engine = PolicyEngine::new(config)?;
        Ok(Self { engine })
    }
    
    /// Create a default policy service with default configuration
    pub fn with_default_config() -> Result<Self, PolicyEngineError> {
        let config = PolicyEngineConfig::default();
        Self::new(config)
    }
}

impl PolicyService for DefaultPolicyService {
    fn evaluate(&self, request: &PolicyRequest) -> Result<PolicyResponse, PolicyEngineError> {
        let result = self.engine.evaluate(
            &request.policy_name,
            &request.input,
            request.rule.as_deref(),
        )?;
        
        Ok(PolicyResponse::new(result, request.context.clone()))
    }
    
    fn load_policy(&self, policy_path: &std::path::Path) -> Result<String, PolicyEngineError> {
        self.engine.load_policy(policy_path)
    }
    
    fn reload_policy(&self, policy_path: &std::path::Path) -> Result<(), PolicyEngineError> {
        self.engine.reload_policy(policy_path)
    }
    
    fn validate_policy(&self, policy_path: &std::path::Path) -> Result<(), PolicyEngineError> {
        self.engine.validate_policy(policy_path)
    }
    
    fn get_stats(&self) -> PolicyEngineStats {
        self.engine.get_stats()
    }
    
    fn clear_cache(&self) {
        self.engine.clear_cache()
    }
    
    fn get_cache_info(&self) -> std::collections::HashMap<String, (std::time::Instant, u64)> {
        self.engine.get_cache_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_context_creation() {
        let context = PolicyContext::new();
        assert!(context.user.is_none());
        assert!(context.source.is_none());
        assert!(context.metadata.is_empty());
    }
    
    #[test]
    fn test_policy_context_with_user() {
        let user = "test_user".to_string();
        let context = PolicyContext::with_user(user.clone());
        assert_eq!(context.user, Some(user));
    }
    
    #[test]
    fn test_policy_context_with_source() {
        let source = "192.168.1.1".to_string();
        let context = PolicyContext::with_source(source.clone());
        assert_eq!(context.source, Some(source));
    }
    
    #[test]
    fn test_policy_context_metadata() {
        let mut context = PolicyContext::new();
        context.set_metadata("key".to_string(), serde_json::json!("value"));
        
        assert_eq!(
            context.get_metadata("key"),
            Some(&serde_json::json!("value"))
        );
    }
    
    #[test]
    fn test_policy_request_creation() {
        let input = serde_json::json!({"test": "data"});
        let request = PolicyRequest::new("test_policy".to_string(), input.clone());
        
        assert_eq!(request.policy_name, "test_policy");
        assert_eq!(request.input, input);
        assert!(request.rule.is_none());
    }
    
    #[test]
    fn test_policy_request_with_rule() {
        let input = serde_json::json!({"test": "data"});
        let request = PolicyRequest::new("test_policy".to_string(), input)
            .with_rule("deny".to_string());
        
        assert_eq!(request.rule, Some("deny".to_string()));
    }
    
    #[test]
    fn test_policy_request_with_user() {
        let input = serde_json::json!({"test": "data"});
        let request = PolicyRequest::new("test_policy".to_string(), input)
            .with_user("test_user".to_string());
        
        assert_eq!(request.context.user, Some("test_user".to_string()));
    }
    
    #[test]
    fn test_policy_response_creation() {
        let result = PolicyResult {
            allowed: true,
            reason: "Test reason".to_string(),
            code: Some("TEST_CODE".to_string()),
            constraints: None,
            metadata: EvaluationMetadata {
                policy_name: "test".to_string(),
                timestamp: chrono::Utc::now(),
                execution_time_us: 100,
                memory_usage_bytes: 1024,
                policy_version: None,
            },
        };
        
        let context = PolicyContext::new();
        let response = PolicyResponse::new(result, context);
        
        assert!(response.is_allowed());
        assert_eq!(response.reason(), "Test reason");
        assert_eq!(response.code(), Some("TEST_CODE"));
    }
    
    #[test]
    fn test_default_policy_service_creation() {
        let service = DefaultPolicyService::with_default_config();
        assert!(service.is_ok());
    }
}
