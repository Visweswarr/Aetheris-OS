use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasmtime::{Engine, Instance, Module, Store};

use crate::wasm_host::WasmHost;

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    /// Whether the operation is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: String,
    /// Policy code/identifier
    pub code: Option<String>,
    /// Additional constraints or metadata
    pub constraints: Option<serde_json::Value>,
    /// Evaluation metadata
    pub metadata: EvaluationMetadata,
}

/// Evaluation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetadata {
    /// Policy name that was evaluated
    pub policy_name: String,
    /// Evaluation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Policy version/hash
    pub policy_version: Option<String>,
}

/// Policy engine configuration
#[derive(Debug, Clone)]
pub struct PolicyEngineConfig {
    /// Maximum execution time for policy evaluation
    pub max_execution_time: Duration,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Whether to enable audit logging
    pub enable_audit_logging: bool,
    /// Audit log file path
    pub audit_log_path: Option<PathBuf>,
    /// Policy cache size
    pub policy_cache_size: usize,
    /// Whether to enable metrics collection
    pub enable_metrics: bool,
}

impl Default for PolicyEngineConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_millis(100),
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            enable_audit_logging: true,
            audit_log_path: Some(PathBuf::from("/var/log/polymera/policy_audit.log")),
            policy_cache_size: 100,
            enable_metrics: true,
        }
    }
}

/// Policy engine errors
#[derive(Error, Debug)]
pub enum PolicyEngineError {
    #[error("Failed to load policy file: {0}")]
    PolicyLoadError(String),
    
    #[error("Failed to compile WASM module: {0}")]
    WasmCompileError(String),
    
    #[error("Failed to instantiate WASM module: {0}")]
    WasmInstantiateError(String),
    
    #[error("Policy evaluation failed: {0}")]
    EvaluationError(String),
    
    #[error("Policy execution timeout after {0:?}")]
    ExecutionTimeout(Duration),
    
    #[error("Memory limit exceeded: {0} bytes")]
    MemoryLimitExceeded(usize),
    
    #[error("Invalid policy input: {0}")]
    InvalidInput(String),
    
    #[error("Policy not found: {0}")]
    PolicyNotFound(String),
    
    #[error("WASM host error: {0}")]
    WasmHostError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Policy engine statistics
#[derive(Debug, Clone, Default)]
pub struct PolicyEngineStats {
    /// Total policy evaluations
    pub total_evaluations: u64,
    /// Successful evaluations
    pub successful_evaluations: u64,
    /// Failed evaluations
    pub failed_evaluations: u64,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Average execution time
    pub average_execution_time: Duration,
    /// Policy cache hits
    pub cache_hits: u64,
    /// Policy cache misses
    pub cache_misses: u64,
    /// Memory usage in bytes
    pub current_memory_usage: usize,
}

/// Cached policy entry
#[derive(Debug, Clone)]
struct CachedPolicy {
    module: Module,
    last_used: Instant,
    usage_count: u64,
}

/// Main policy engine
pub struct PolicyEngine {
    /// WASM engine
    wasm_engine: Engine,
    /// Policy cache
    policy_cache: Arc<Mutex<HashMap<String, CachedPolicy>>>,
    /// Configuration
    config: PolicyEngineConfig,
    /// Statistics
    stats: Arc<Mutex<PolicyEngineStats>>,
    /// WASM host for policy execution
    wasm_host: WasmHost,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new(config: PolicyEngineConfig) -> Result<Self, PolicyEngineError> {
        let wasm_engine = Engine::default();
        let wasm_host = WasmHost::new()?;
        
        Ok(Self {
            wasm_engine,
            policy_cache: Arc::new(Mutex::new(HashMap::new())),
            config,
            stats: Arc::new(Mutex::new(PolicyEngineStats::default())),
            wasm_host,
        })
    }
    
    /// Load a policy from a WASM file
    pub fn load_policy(&self, policy_path: &Path) -> Result<String, PolicyEngineError> {
        let policy_name = policy_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| PolicyEngineError::PolicyLoadError("Invalid policy path".to_string()))?;
        
        // Check if policy is already cached
        {
            let cache = self.policy_cache.lock().unwrap();
            if cache.contains_key(policy_name) {
                let mut stats = self.stats.lock().unwrap();
                stats.cache_hits += 1;
                return Ok(policy_name.to_string());
            }
        }
        
        // Load and compile policy
        let wasm_bytes = fs::read(policy_path)
            .map_err(|e| PolicyEngineError::PolicyLoadError(e.to_string()))?;
        
        let module = Module::new(&self.wasm_engine, wasm_bytes)
            .map_err(|e| PolicyEngineError::WasmCompileError(e.to_string()))?;
        
        // Cache the policy
        {
            let mut cache = self.policy_cache.lock().unwrap();
            if cache.len() >= self.config.policy_cache_size {
                self.evict_oldest_policy(&mut cache);
            }
            
            cache.insert(policy_name.to_string(), CachedPolicy {
                module,
                last_used: Instant::now(),
                usage_count: 1,
            });
            
            let mut stats = self.stats.lock().unwrap();
            stats.cache_misses += 1;
        }
        
        Ok(policy_name.to_string())
    }
    
    /// Evaluate a policy with input data
    pub fn evaluate(
        &self,
        policy_name: &str,
        input: &serde_json::Value,
        rule: Option<&str>,
    ) -> Result<PolicyResult, PolicyEngineError> {
        let start_time = Instant::now();
        
        // Get cached policy
        let cached_policy = {
            let mut cache = self.policy_cache.lock().unwrap();
            let entry = cache.get_mut(policy_name)
                .ok_or_else(|| PolicyEngineError::PolicyNotFound(policy_name.to_string()))?;
            
            entry.last_used = Instant::now();
            entry.usage_count += 1;
            
            entry.module.clone()
        };
        
        // Create WASM store
        let mut store = Store::new(&self.wasm_engine, self.wasm_host.clone());
        
        // Set resource limits
        store.limiter(|state| &mut state.limiter);
        
        // Instantiate module
        let instance = Instance::new(&mut store, &cached_policy, &[])
            .map_err(|e| PolicyEngineError::WasmInstantiateError(e.to_string()))?;
        
        // Prepare input data
        let input_json = serde_json::to_string(input)
            .map_err(|e| PolicyEngineError::SerializationError(e.to_string()))?;
        
        // Execute policy evaluation
        let result = self.execute_policy_evaluation(&mut store, &instance, &input_json, rule)?;
        
        // Update statistics
        let execution_time = start_time.elapsed();
        self.update_stats(execution_time, true);
        
        // Audit logging
        if self.config.enable_audit_logging {
            self.log_audit_event(policy_name, input, &result, execution_time)?;
        }
        
        Ok(result)
    }
    
    /// Execute policy evaluation in WASM
    fn execute_policy_evaluation(
        &self,
        store: &mut Store<WasmHost>,
        instance: &Instance,
        input_json: &str,
        rule: Option<&str>,
    ) -> Result<PolicyResult, PolicyEngineError> {
        // Get the main function
        let main_func = instance.get_func(store, "main")
            .ok_or_else(|| PolicyEngineError::WasmHostError("Main function not found".to_string()))?;
        
        // Prepare function parameters
        let rule_name = rule.unwrap_or("allow");
        let params = format!("{{\"input\": {}, \"rule\": \"{}\"}}", input_json, rule_name);
        
        // Execute with timeout
        let result = self.execute_with_timeout(store, &main_func, &params)?;
        
        // Parse result
        self.parse_policy_result(&result, rule_name)
    }
    
    /// Execute WASM function with timeout
    fn execute_with_timeout(
        &self,
        store: &mut Store<WasmHost>,
        func: &wasmtime::Func,
        params: &str,
    ) -> Result<String, PolicyEngineError> {
        // Set execution timeout
        let start_time = Instant::now();
        
        // Execute function
        let result = func.call(store, &[params.into()])
            .map_err(|e| PolicyEngineError::EvaluationError(e.to_string()))?;
        
        // Check timeout
        if start_time.elapsed() > self.config.max_execution_time {
            return Err(PolicyEngineError::ExecutionTimeout(self.config.max_execution_time));
        }
        
        // Convert result to string
        let result_str = result[0].unwrap_str()
            .map_err(|e| PolicyEngineError::WasmHostError(e.to_string()))?;
        
        Ok(result_str.to_string())
    }
    
    /// Parse policy evaluation result
    fn parse_policy_result(
        &self,
        result_json: &str,
        rule_name: &str,
    ) -> Result<PolicyResult, PolicyEngineError> {
        let result: serde_json::Value = serde_json::from_str(result_json)
            .map_err(|e| PolicyEngineError::SerializationError(e.to_string()))?;
        
        // Extract basic fields
        let allowed = result.get("allowed")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let reason = result.get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("No reason provided")
            .to_string();
        
        let code = result.get("code")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let constraints = result.get("constraints").cloned();
        
        // Create metadata
        let metadata = EvaluationMetadata {
            policy_name: rule_name.to_string(),
            timestamp: chrono::Utc::now(),
            execution_time_us: 0, // Will be set by caller
            memory_usage_bytes: 0, // Will be set by caller
            policy_version: None,
        };
        
        Ok(PolicyResult {
            allowed,
            reason,
            code,
            constraints,
            metadata,
        })
    }
    
    /// Evict oldest policy from cache
    fn evict_oldest_policy(&self, cache: &mut HashMap<String, CachedPolicy>) {
        let oldest_key = cache.iter()
            .min_by_key(|(_, policy)| policy.last_used)
            .map(|(key, _)| key.clone());
        
        if let Some(key) = oldest_key {
            cache.remove(&key);
        }
    }
    
    /// Update engine statistics
    fn update_stats(&self, execution_time: Duration, success: bool) {
        let mut stats = self.stats.lock().unwrap();
        
        stats.total_evaluations += 1;
        if success {
            stats.successful_evaluations += 1;
        } else {
            stats.failed_evaluations += 1;
        }
        
        stats.total_execution_time += execution_time;
        stats.average_execution_time = Duration::from_micros(
            stats.total_execution_time.as_micros() as u64 / stats.total_evaluations
        );
    }
    
    /// Log audit event
    fn log_audit_event(
        &self,
        policy_name: &str,
        input: &serde_json::Value,
        result: &PolicyResult,
        execution_time: Duration,
    ) -> Result<(), PolicyEngineError> {
        if let Some(audit_path) = &self.config.audit_log_path {
            let audit_event = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "policy_name": policy_name,
                "input": input,
                "result": result,
                "execution_time_us": execution_time.as_micros(),
                "user_agent": "PolymeraOS-PolicyEngine/1.0"
            });
            
            let audit_line = serde_json::to_string(&audit_event)? + "\n";
            
            // Ensure audit log directory exists
            if let Some(parent) = audit_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Append to audit log
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(audit_path)?;
            
            use std::io::Write;
            file.write_all(audit_line.as_bytes())?;
        }
        
        Ok(())
    }
    
    /// Get engine statistics
    pub fn get_stats(&self) -> PolicyEngineStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Clear policy cache
    pub fn clear_cache(&self) {
        let mut cache = self.policy_cache.lock().unwrap();
        cache.clear();
    }
    
    /// Get cache information
    pub fn get_cache_info(&self) -> HashMap<String, (Instant, u64)> {
        let cache = self.policy_cache.lock().unwrap();
        cache.iter()
            .map(|(key, policy)| (key.clone(), (policy.last_used, policy.usage_count)))
            .collect()
    }
    
    /// Reload a policy from disk
    pub fn reload_policy(&self, policy_path: &Path) -> Result<(), PolicyEngineError> {
        let policy_name = policy_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| PolicyEngineError::PolicyLoadError("Invalid policy path".to_string()))?;
        
        // Remove from cache
        {
            let mut cache = self.policy_cache.lock().unwrap();
            cache.remove(policy_name);
        }
        
        // Reload
        self.load_policy(policy_path)?;
        
        Ok(())
    }
    
    /// Validate policy syntax without execution
    pub fn validate_policy(&self, policy_path: &Path) -> Result<(), PolicyEngineError> {
        let wasm_bytes = fs::read(policy_path)
            .map_err(|e| PolicyEngineError::PolicyLoadError(e.to_string()))?;
        
        Module::new(&self.wasm_engine, wasm_bytes)
            .map_err(|e| PolicyEngineError::WasmCompileError(e.to_string()))?;
        
        Ok(())
    }
}

impl Drop for PolicyEngine {
    fn drop(&mut self) {
        // Clean up resources
        self.clear_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    #[test]
    fn test_policy_engine_creation() {
        let config = PolicyEngineConfig::default();
        let engine = PolicyEngine::new(config);
        assert!(engine.is_ok());
    }
    
    #[test]
    fn test_policy_engine_config_default() {
        let config = PolicyEngineConfig::default();
        assert_eq!(config.max_execution_time, Duration::from_millis(100));
        assert_eq!(config.max_memory_bytes, 64 * 1024 * 1024);
        assert!(config.enable_audit_logging);
    }
    
    #[test]
    fn test_policy_engine_stats() {
        let config = PolicyEngineConfig::default();
        let engine = PolicyEngine::new(config).unwrap();
        
        let stats = engine.get_stats();
        assert_eq!(stats.total_evaluations, 0);
        assert_eq!(stats.successful_evaluations, 0);
        assert_eq!(stats.failed_evaluations, 0);
    }
    
    #[test]
    fn test_policy_engine_cache_operations() {
        let config = PolicyEngineConfig::default();
        let engine = PolicyEngine::new(config).unwrap();
        
        // Test cache operations
        engine.clear_cache();
        let cache_info = engine.get_cache_info();
        assert!(cache_info.is_empty());
    }
}
