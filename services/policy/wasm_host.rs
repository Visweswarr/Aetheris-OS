use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasmtime::{Caller, Engine, Linker, Store, Val, ValType};

/// WASM host environment for policy execution
#[derive(Clone)]
pub struct WasmHost {
    /// Host functions available to WASM modules
    host_functions: Arc<Mutex<HostFunctions>>,
    /// Resource limits
    resource_limits: ResourceLimits,
    /// Execution context
    execution_context: Arc<Mutex<ExecutionContext>>,
}

/// Host functions available to WASM modules
#[derive(Clone)]
struct HostFunctions {
    /// Available functions
    functions: HashMap<String, HostFunction>,
    /// Function call statistics
    call_stats: HashMap<String, FunctionCallStats>,
}

/// Individual host function
#[derive(Clone)]
struct HostFunction {
    /// Function name
    name: String,
    /// Function implementation (takes i32/i64 params, returns i32/i64)
    implementation: fn(&[Val]) -> Result<Val, String>,
    /// Function metadata
    metadata: FunctionMetadata,
}

/// Function metadata
#[derive(Clone)]
struct FunctionMetadata {
    /// Function description
    description: String,
    /// Parameter types
    parameter_types: Vec<String>,
    /// Return type
    return_type: String,
    /// Whether function is safe
    is_safe: bool,
    /// Maximum execution time
    max_execution_time: Duration,
}

/// Function call statistics
#[derive(Clone, Default)]
struct FunctionCallStats {
    /// Total calls
    total_calls: u64,
    /// Successful calls
    successful_calls: u64,
    /// Failed calls
    failed_calls: u64,
    /// Total execution time
    total_execution_time: Duration,
    /// Average execution time
    average_execution_time: Duration,
}

/// Resource limits for WASM execution
#[derive(Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Maximum function calls
    pub max_function_calls: u32,
    /// Maximum table size
    pub max_table_size: u32,
    /// Maximum instances
    pub max_instances: u32,
    /// Maximum tables
    pub max_tables: u32,
    /// Maximum memories
    pub max_memories: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_execution_time: Duration::from_millis(100),
            max_function_calls: 1000,
            max_table_size: 10000,
            max_instances: 10,
            max_tables: 10,
            max_memories: 10,
        }
    }
}

/// Execution context for WASM modules
#[derive(Clone)]
struct ExecutionContext {
    /// Start time of execution
    start_time: Instant,
    /// Function call count
    function_call_count: u32,
    /// Memory usage in bytes
    memory_usage_bytes: usize,
    /// Execution trace
    execution_trace: Vec<ExecutionEvent>,
}

/// Execution event for tracing
#[derive(Clone)]
struct ExecutionEvent {
    /// Event timestamp
    timestamp: Instant,
    /// Event type
    event_type: ExecutionEventType,
    /// Event details
    details: String,
}

/// Execution event types
#[derive(Clone)]
enum ExecutionEventType {
    FunctionCall,
    MemoryAllocation,
    ResourceLimit,
    SecurityViolation,
    Error,
}

/// WASM host errors
#[derive(Error, Debug)]
pub enum WasmHostError {
    #[error("Host function not found: {0}")]
    HostFunctionNotFound(String),
    
    #[error("Host function execution failed: {0}")]
    HostFunctionError(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Invalid function parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Function execution timeout after {0:?}")]
    FunctionTimeout(Duration),
    
    #[error("Memory allocation failed: {0} bytes")]
    MemoryAllocationFailed(usize),
    
    #[error("WASM linking error: {0}")]
    LinkingError(String),
}

impl WasmHost {
    /// Create a new WASM host
    pub fn new() -> Result<Self, WasmHostError> {
        let mut host_functions = HostFunctions {
            functions: HashMap::new(),
            call_stats: HashMap::new(),
        };
        
        // Register built-in host functions
        Self::register_builtin_functions(&mut host_functions)?;
        
        Ok(Self {
            host_functions: Arc::new(Mutex::new(host_functions)),
            resource_limits: ResourceLimits::default(),
            execution_context: Arc::new(Mutex::new(ExecutionContext::new())),
        })
    }
    
    /// Create WASM host with custom resource limits
    pub fn with_limits(limits: ResourceLimits) -> Result<Self, WasmHostError> {
        let mut host = Self::new()?;
        host.resource_limits = limits;
        Ok(host)
    }
    
    /// Register built-in host functions
    fn register_builtin_functions(host_functions: &mut HostFunctions) -> Result<(), WasmHostError> {
        // Math functions - these work with i32/i64 which are valid WASM types
        Self::register_function(
            host_functions,
            "math_max",
            "Get maximum of two numbers",
            vec!["i32".to_string(), "i32".to_string()],
            "i32",
            true,
            Duration::from_micros(10),
            |args| Self::math_max(args),
        )?;
        
        Self::register_function(
            host_functions,
            "math_min",
            "Get minimum of two numbers",
            vec!["i32".to_string(), "i32".to_string()],
            "i32",
            true,
            Duration::from_micros(10),
            |args| Self::math_min(args),
        )?;
        
        Self::register_function(
            host_functions,
            "math_abs",
            "Get absolute value",
            vec!["i32".to_string()],
            "i32",
            true,
            Duration::from_micros(10),
            |args| Self::math_abs(args),
        )?;
        
        // Time functions
        Self::register_function(
            host_functions,
            "time_now",
            "Get current timestamp (seconds since epoch)",
            vec![],
            "i64",
            true,
            Duration::from_micros(10),
            |_args| Self::time_now(),
        )?;
        
        Self::register_function(
            host_functions,
            "time_now_millis",
            "Get current timestamp in milliseconds",
            vec![],
            "i64",
            true,
            Duration::from_micros(10),
            |_args| Self::time_now_millis(),
        )?;
        
        // Logging functions (take i32 log level, return i32 status)
        Self::register_function(
            host_functions,
            "log_level",
            "Log with specified level (0=debug, 1=info, 2=warn, 3=error)",
            vec!["i32".to_string()],
            "i32",
            true,
            Duration::from_micros(10),
            |args| Self::log_level(args),
        )?;
        
        // Utility functions
        Self::register_function(
            host_functions,
            "random_i32",
            "Generate random i32",
            vec![],
            "i32",
            true,
            Duration::from_micros(10),
            |_args| Self::random_i32(),
        )?;
        
        Self::register_function(
            host_functions,
            "random_i64",
            "Generate random i64",
            vec![],
            "i64",
            true,
            Duration::from_micros(10),
            |_args| Self::random_i64(),
        )?;
        
        Ok(())
    }
    
    /// Register a host function
    fn register_function(
        host_functions: &mut HostFunctions,
        name: &str,
        description: &str,
        parameter_types: Vec<String>,
        return_type: &str,
        is_safe: bool,
        max_execution_time: Duration,
        implementation: fn(&[Val]) -> Result<Val, String>,
    ) -> Result<(), WasmHostError> {
        let function = HostFunction {
            name: name.to_string(),
            implementation,
            metadata: FunctionMetadata {
                description: description.to_string(),
                parameter_types,
                return_type: return_type.to_string(),
                is_safe,
                max_execution_time,
            },
        };
        
        host_functions.functions.insert(name.to_string(), function);
        host_functions.call_stats.insert(name.to_string(), FunctionCallStats::default());
        
        Ok(())
    }
    
    /// Call a host function by name
    pub fn call_function(&self, name: &str, params: &[Val]) -> Result<Val, WasmHostError> {
        let host_functions = self.host_functions.lock().unwrap();
        
        let function = host_functions.functions.get(name)
            .ok_or_else(|| WasmHostError::HostFunctionNotFound(name.to_string()))?;
        
        let start_time = Instant::now();
        
        // Update execution context
        {
            let mut context = self.execution_context.lock().unwrap();
            context.function_call_count += 1;
            context.execution_trace.push(ExecutionEvent {
                timestamp: Instant::now(),
                event_type: ExecutionEventType::FunctionCall,
                details: format!("Calling host function: {}", name),
            });
        }
        
        // Execute function
        let result = (function.implementation)(params)
            .map_err(|e| WasmHostError::HostFunctionError(e))?;
        
        // Update statistics
        drop(host_functions);
        {
            let mut host_functions = self.host_functions.lock().unwrap();
            if let Some(stats) = host_functions.call_stats.get_mut(name) {
                stats.total_calls += 1;
                stats.successful_calls += 1;
                stats.total_execution_time += start_time.elapsed();
                if stats.total_calls > 0 {
                    stats.average_execution_time = Duration::from_micros(
                        stats.total_execution_time.as_micros() as u64 / stats.total_calls
                    );
                }
            }
        }
        
        Ok(result)
    }
    
    /// Get list of available function names
    pub fn get_function_names(&self) -> Vec<String> {
        let host_functions = self.host_functions.lock().unwrap();
        host_functions.functions.keys().cloned().collect()
    }
    
    /// Built-in host function implementations
    
    // Math functions
    fn math_max(args: &[Val]) -> Result<Val, String> {
        if args.len() != 2 {
            return Err("math_max requires exactly 2 arguments".to_string());
        }
        
        let a = match &args[0] {
            Val::I32(n) => *n,
            _ => return Err("First argument must be i32".to_string()),
        };
        let b = match &args[1] {
            Val::I32(n) => *n,
            _ => return Err("Second argument must be i32".to_string()),
        };
        
        Ok(Val::I32(a.max(b)))
    }
    
    fn math_min(args: &[Val]) -> Result<Val, String> {
        if args.len() != 2 {
            return Err("math_min requires exactly 2 arguments".to_string());
        }
        
        let a = match &args[0] {
            Val::I32(n) => *n,
            _ => return Err("First argument must be i32".to_string()),
        };
        let b = match &args[1] {
            Val::I32(n) => *n,
            _ => return Err("Second argument must be i32".to_string()),
        };
        
        Ok(Val::I32(a.min(b)))
    }
    
    fn math_abs(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("math_abs requires exactly 1 argument".to_string());
        }
        
        let a = match &args[0] {
            Val::I32(n) => *n,
            _ => return Err("First argument must be i32".to_string()),
        };
        
        Ok(Val::I32(a.abs()))
    }
    
    // Time functions
    fn time_now() -> Result<Val, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        Ok(Val::I64(timestamp))
    }
    
    fn time_now_millis() -> Result<Val, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        
        Ok(Val::I64(timestamp))
    }
    
    // Logging functions
    fn log_level(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("log_level requires exactly 1 argument".to_string());
        }
        
        let level = match &args[0] {
            Val::I32(n) => *n,
            _ => return Err("First argument must be i32".to_string()),
        };
        
        let level_str = match level {
            0 => "DEBUG",
            1 => "INFO",
            2 => "WARN",
            3 => "ERROR",
            _ => "UNKNOWN",
        };
        
        eprintln!("[{}] WASM log event", level_str);
        Ok(Val::I32(0)) // Success
    }
    
    // Random functions
    fn random_i32() -> Result<Val, String> {
        // Simple pseudo-random using time
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i32;
        
        Ok(Val::I32(timestamp))
    }
    
    fn random_i64() -> Result<Val, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        
        Ok(Val::I64(timestamp))
    }
    
    /// Get host function statistics
    pub fn get_function_stats(&self) -> HashMap<String, FunctionCallStats> {
        self.host_functions.lock().unwrap().call_stats.clone()
    }
    
    /// Get execution context
    pub fn get_execution_context(&self) -> ExecutionContext {
        self.execution_context.lock().unwrap().clone()
    }
    
    /// Reset execution context
    pub fn reset_execution_context(&self) {
        let mut context = self.execution_context.lock().unwrap();
        *context = ExecutionContext::new();
    }
    
    /// Get resource limits
    pub fn get_resource_limits(&self) -> &ResourceLimits {
        &self.resource_limits
    }
    
    /// Check if memory limit would be exceeded
    pub fn check_memory_limit(&self, requested_bytes: usize) -> bool {
        let context = self.execution_context.lock().unwrap();
        context.memory_usage_bytes + requested_bytes <= self.resource_limits.max_memory_bytes
    }
    
    /// Check if function call limit would be exceeded
    pub fn check_function_call_limit(&self) -> bool {
        let context = self.execution_context.lock().unwrap();
        context.function_call_count < self.resource_limits.max_function_calls
    }
    
    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: usize) {
        let mut context = self.execution_context.lock().unwrap();
        context.memory_usage_bytes = bytes;
        context.execution_trace.push(ExecutionEvent {
            timestamp: Instant::now(),
            event_type: ExecutionEventType::MemoryAllocation,
            details: format!("Memory usage updated to {} bytes", bytes),
        });
    }
}

impl ExecutionContext {
    /// Create new execution context
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            function_call_count: 0,
            memory_usage_bytes: 0,
            execution_trace: Vec::new(),
        }
    }
    
    /// Get elapsed time since execution started
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get function call count
    pub fn function_call_count(&self) -> u32 {
        self.function_call_count
    }
    
    /// Get memory usage
    pub fn memory_usage_bytes(&self) -> usize {
        self.memory_usage_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_host_creation() {
        let host = WasmHost::new();
        assert!(host.is_ok());
    }
    
    #[test]
    fn test_wasm_host_with_limits() {
        let limits = ResourceLimits {
            max_memory_bytes: 128 * 1024 * 1024, // 128MB
            max_execution_time: Duration::from_millis(200),
            max_function_calls: 2000,
            max_table_size: 20000,
            max_instances: 20,
            max_tables: 20,
            max_memories: 20,
        };
        
        let host = WasmHost::with_limits(limits);
        assert!(host.is_ok());
    }
    
    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(limits.max_execution_time, Duration::from_millis(100));
        assert_eq!(limits.max_function_calls, 1000);
    }
    
    #[test]
    fn test_host_function_registration() {
        let host = WasmHost::new().unwrap();
        let names = host.get_function_names();
        
        // Check that built-in functions are registered
        assert!(names.contains(&"math_max".to_string()));
        assert!(names.contains(&"math_min".to_string()));
        assert!(names.contains(&"time_now".to_string()));
        assert!(names.contains(&"log_level".to_string()));
    }
    
    #[test]
    fn test_execution_context() {
        let host = WasmHost::new().unwrap();
        let context = host.get_execution_context();
        
        assert_eq!(context.function_call_count(), 0);
        assert_eq!(context.memory_usage_bytes(), 0);
    }
    
    #[test]
    fn test_math_max() {
        let host = WasmHost::new().unwrap();
        let result = host.call_function("math_max", &[Val::I32(5), Val::I32(10)]);
        assert!(result.is_ok());
        match result.unwrap() {
            Val::I32(n) => assert_eq!(n, 10),
            _ => panic!("Expected I32"),
        }
    }
    
    #[test]
    fn test_math_min() {
        let host = WasmHost::new().unwrap();
        let result = host.call_function("math_min", &[Val::I32(5), Val::I32(10)]);
        assert!(result.is_ok());
        match result.unwrap() {
            Val::I32(n) => assert_eq!(n, 5),
            _ => panic!("Expected I32"),
        }
    }
    
    #[test]
    fn test_time_now() {
        let host = WasmHost::new().unwrap();
        let result = host.call_function("time_now", &[]);
        assert!(result.is_ok());
        match result.unwrap() {
            Val::I64(n) => assert!(n > 0),
            _ => panic!("Expected I64"),
        }
    }
    
    #[test]
    fn test_function_not_found() {
        let host = WasmHost::new().unwrap();
        let result = host.call_function("nonexistent", &[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_memory_limit_check() {
        let host = WasmHost::new().unwrap();
        assert!(host.check_memory_limit(1024)); // 1KB should be fine
        assert!(!host.check_memory_limit(128 * 1024 * 1024)); // 128MB should exceed
    }
    
    #[test]
    fn test_function_call_limit_check() {
        let host = WasmHost::new().unwrap();
        assert!(host.check_function_call_limit());
    }
}
