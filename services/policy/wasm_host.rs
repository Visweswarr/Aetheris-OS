use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasmtime::{Caller, Linker, ResourceLimiter, Store, Val};

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
    /// Function implementation
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
        // JSON functions
        Self::register_function(
            host_functions,
            "json_parse",
            "Parse JSON string to object",
            vec!["string"],
            "object",
            true,
            Duration::from_micros(100),
            |args| Self::json_parse(args),
        )?;
        
        Self::register_function(
            host_functions,
            "json_stringify",
            "Convert object to JSON string",
            vec!["object"],
            "string",
            true,
            Duration::from_micros(100),
            |args| Self::json_stringify(args),
        )?;
        
        // String functions
        Self::register_function(
            host_functions,
            "string_length",
            "Get string length",
            vec!["string"],
            "number",
            true,
            Duration::from_micros(10),
            |args| Self::string_length(args),
        )?;
        
        Self::register_function(
            host_functions,
            "string_substring",
            "Extract substring",
            vec!["string", "number", "number"],
            "string",
            true,
            Duration::from_micros(10),
            |args| Self::string_substring(args),
        )?;
        
        // Array functions
        Self::register_function(
            host_functions,
            "array_length",
            "Get array length",
            vec!["array"],
            "number",
            true,
            Duration::from_micros(10),
            |args| Self::array_length(args),
        )?;
        
        Self::register_function(
            host_functions,
            "array_push",
            "Add element to array",
            vec!["array", "any"],
            "array",
            true,
            Duration::from_micros(10),
            |args| Self::array_push(args),
        )?;
        
        // Math functions
        Self::register_function(
            host_functions,
            "math_max",
            "Get maximum of two numbers",
            vec!["number", "number"],
            "number",
            true,
            Duration::from_micros(10),
            |args| Self::math_max(args),
        )?;
        
        Self::register_function(
            host_functions,
            "math_min",
            "Get minimum of two numbers",
            vec!["number", "number"],
            "number",
            true,
            Duration::from_micros(10),
            |args| Self::math_min(args),
        )?;
        
        // Time functions
        Self::register_function(
            host_functions,
            "time_now",
            "Get current timestamp",
            vec![],
            "number",
            true,
            Duration::from_micros(10),
            |args| Self::time_now(args),
        )?;
        
        // Logging functions
        Self::register_function(
            host_functions,
            "log_info",
            "Log info message",
            vec!["string"],
            "void",
            true,
            Duration::from_micros(10),
            |args| Self::log_info(args),
        )?;
        
        Self::register_function(
            host_functions,
            "log_warning",
            "Log warning message",
            vec!["string"],
            "void",
            true,
            Duration::from_micros(10),
            |args| Self::log_warning(args),
        )?;
        
        Self::register_function(
            host_functions,
            "log_error",
            "Log error message",
            vec!["string"],
            "void",
            true,
            Duration::from_micros(10),
            |args| Self::log_error(args),
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
    
    /// Link host functions to WASM module
    pub fn link_functions(&self, linker: &mut Linker<Self>) -> Result<(), WasmHostError> {
        let host_functions = self.host_functions.lock().unwrap();
        
        for (name, function) in &host_functions.functions {
            let function_name = name.clone();
            let function_impl = function.implementation;
            
            linker.func_wrap(name, function_name.as_str(), move |caller: Caller<Self>, params: &[Val]| {
                Self::execute_host_function(caller, params, &function_name, function_impl)
            })?;
        }
        
        Ok(())
    }
    
    /// Execute a host function
    fn execute_host_function(
        mut caller: Caller<Self>,
        params: &[Val],
        function_name: &str,
        implementation: fn(&[Val]) -> Result<Val, String>,
    ) -> Result<Val, wasmtime::Trap> {
        let start_time = Instant::now();
        
        // Update execution context
        {
            let mut context = caller.data().execution_context.lock().unwrap();
            context.function_call_count += 1;
            context.execution_trace.push(ExecutionEvent {
                timestamp: Instant::now(),
                event_type: ExecutionEventType::FunctionCall,
                details: format!("Calling host function: {}", function_name),
            });
        }
        
        // Update function call statistics
        {
            let mut host_functions = caller.data().host_functions.lock().unwrap();
            if let Some(stats) = host_functions.call_stats.get_mut(function_name) {
                stats.total_calls += 1;
                stats.total_execution_time += start_time.elapsed();
                stats.average_execution_time = Duration::from_micros(
                    stats.total_execution_time.as_micros() as u64 / stats.total_calls
                );
            }
        }
        
        // Execute function
        match implementation(params) {
            Ok(result) => {
                // Update success statistics
                if let Some(stats) = caller.data().host_functions.lock().unwrap().call_stats.get_mut(function_name) {
                    stats.successful_calls += 1;
                }
                Ok(result)
            }
            Err(error) => {
                // Update failure statistics
                if let Some(stats) = caller.data().host_functions.lock().unwrap().call_stats.get_mut(function_name) {
                    stats.failed_calls += 1;
                }
                
                // Log error
                let mut context = caller.data().execution_context.lock().unwrap();
                context.execution_trace.push(ExecutionEvent {
                    timestamp: Instant::now(),
                    event_type: ExecutionEventType::Error,
                    details: format!("Host function error: {}", error),
                });
                
                Err(wasmtime::Trap::new(error))
            }
        }
    }
    
    /// Built-in host function implementations
    
    // JSON functions
    fn json_parse(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("json_parse requires exactly 1 argument".to_string());
        }
        
        let json_str = args[0].unwrap_str()
            .map_err(|_| "First argument must be a string".to_string())?;
        
        // For now, return the string as-is since we can't easily convert to WASM types
        // In a real implementation, you'd parse JSON and convert to appropriate WASM values
        Ok(Val::String(json_str.to_string()))
    }
    
    fn json_stringify(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("json_stringify requires exactly 1 argument".to_string());
        }
        
        // Convert WASM value to string representation
        let value_str = match &args[0] {
            Val::I32(n) => n.to_string(),
            Val::I64(n) => n.to_string(),
            Val::F32(f) => f.to_string(),
            Val::F64(f) => f.to_string(),
            Val::String(s) => s.clone(),
            _ => "undefined".to_string(),
        };
        
        Ok(Val::String(value_str))
    }
    
    // String functions
    fn string_length(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("string_length requires exactly 1 argument".to_string());
        }
        
        let length = args[0].unwrap_str()
            .map_err(|_| "First argument must be a string".to_string())?
            .len() as i32;
        
        Ok(Val::I32(length))
    }
    
    fn string_substring(args: &[Val]) -> Result<Val, String> {
        if args.len() != 3 {
            return Err("string_substring requires exactly 3 arguments".to_string());
        }
        
        let s = args[0].unwrap_str()
            .map_err(|_| "First argument must be a string".to_string())?;
        let start = args[1].unwrap_i32()
            .map_err(|_| "Second argument must be a number".to_string())? as usize;
        let end = args[2].unwrap_i32()
            .map_err(|_| "Third argument must be a number".to_string())? as usize;
        
        if start >= s.len() || end > s.len() || start >= end {
            return Err("Invalid substring indices".to_string());
        }
        
        let substring = &s[start..end];
        Ok(Val::String(substring.to_string()))
    }
    
    // Array functions
    fn array_length(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("array_length requires exactly 1 argument".to_string());
        }
        
        // For now, return 0 since we can't easily determine array length in WASM
        // In a real implementation, you'd track array metadata
        Ok(Val::I32(0))
    }
    
    fn array_push(args: &[Val]) -> Result<Val, String> {
        if args.len() != 2 {
            return Err("array_push requires exactly 2 arguments".to_string());
        }
        
        // For now, return the array as-is since we can't easily modify arrays in WASM
        // In a real implementation, you'd modify the array and return it
        Ok(args[0].clone())
    }
    
    // Math functions
    fn math_max(args: &[Val]) -> Result<Val, String> {
        if args.len() != 2 {
            return Err("math_max requires exactly 2 arguments".to_string());
        }
        
        let a = args[0].unwrap_i32()
            .map_err(|_| "First argument must be a number".to_string())?;
        let b = args[1].unwrap_i32()
            .map_err(|_| "Second argument must be a number".to_string())?;
        
        Ok(Val::I32(a.max(b)))
    }
    
    fn math_min(args: &[Val]) -> Result<Val, String> {
        if args.len() != 2 {
            return Err("math_min requires exactly 2 arguments".to_string());
        }
        
        let a = args[0].unwrap_i32()
            .map_err(|_| "First argument must be a number".to_string())?;
        let b = args[1].unwrap_i32()
            .map_err(|_| "Second argument must be a number".to_string())?;
        
        Ok(Val::I32(a.min(b)))
    }
    
    // Time functions
    fn time_now(_args: &[Val]) -> Result<Val, String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        Ok(Val::I64(timestamp))
    }
    
    // Logging functions
    fn log_info(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("log_info requires exactly 1 argument".to_string());
        }
        
        let message = args[0].unwrap_str()
            .map_err(|_| "First argument must be a string".to_string())?;
        
        eprintln!("[INFO] {}", message);
        Ok(Val::I32(0))
    }
    
    fn log_warning(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("log_warning requires exactly 1 argument".to_string());
        }
        
        let message = args[0].unwrap_str()
            .map_err(|_| "First argument must be a string".to_string())?;
        
        eprintln!("[WARNING] {}", message);
        Ok(Val::I32(0))
    }
    
    fn log_error(args: &[Val]) -> Result<Val, String> {
        if args.len() != 1 {
            return Err("log_error requires exactly 1 argument".to_string());
        }
        
        let message = args[0].unwrap_str()
            .map_err(|_| "First argument must be a string".to_string())?;
        
        eprintln!("[ERROR] {}", message);
        Ok(Val::I32(0))
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
}

impl ResourceLimiter for WasmHost {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        // Check memory limits
        if desired > self.resource_limits.max_memory_bytes {
            return Err(wasmtime::Error::new(
                format!("Memory limit exceeded: {} bytes", desired)
            ));
        }
        
        // Update execution context
        {
            let mut context = self.execution_context.lock().unwrap();
            context.memory_usage_bytes = desired;
            context.execution_trace.push(ExecutionEvent {
                timestamp: Instant::now(),
                event_type: ExecutionEventType::MemoryAllocation,
                details: format!("Memory allocation: {} -> {} bytes", current, desired),
            });
        }
        
        Ok(true)
    }
    
    fn table_growing(
        &mut self,
        current: u32,
        desired: u32,
        maximum: Option<u32>,
    ) -> Result<bool, wasmtime::Error> {
        if desired > self.resource_limits.max_table_size {
            return Err(wasmtime::Error::new(
                format!("Table size limit exceeded: {}", desired)
            ));
        }
        
        Ok(true)
    }
    
    fn instances(&mut self, count: u32) -> Result<bool, wasmtime::Error> {
        if count > self.resource_limits.max_instances {
            return Err(wasmtime::Error::new(
                format!("Instance limit exceeded: {}", count)
            ));
        }
        
        Ok(true)
    }
    
    fn tables(&mut self, count: u32) -> Result<bool, wasmtime::Error> {
        if count > self.resource_limits.max_tables {
            return Err(wasmtime::Error::new(
                format!("Table limit exceeded: {}", count)
            ));
        }
        
        Ok(true)
    }
    
    fn memories(&mut self, count: u32) -> Result<bool, wasmtime::Error> {
        if count > self.resource_limits.max_memories {
            return Err(wasmtime::Error::new(
                format!("Memory limit exceeded: {}", count)
            ));
        }
        
        Ok(true)
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
        let stats = host.get_function_stats();
        
        // Check that built-in functions are registered
        assert!(stats.contains_key("json_parse"));
        assert!(stats.contains_key("string_length"));
        assert!(stats.contains_key("math_max"));
        assert!(stats.contains_key("time_now"));
        assert!(stats.contains_key("log_info"));
    }
    
    #[test]
    fn test_execution_context() {
        let host = WasmHost::new().unwrap();
        let context = host.get_execution_context();
        
        assert_eq!(context.function_call_count, 0);
        assert_eq!(context.memory_usage_bytes, 0);
        assert!(context.execution_trace.is_empty());
    }
}
