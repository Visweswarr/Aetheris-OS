//! Policy Engine Test - Basic functionality tests

use polymera_policy::{
    PolicyEngine, PolicyEngineConfig,
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Polymera OS Policy Engine Test ===\n");

    // Test policy engine creation
    test_policy_engine_creation()?;
    println!();

    // Test resource limits
    test_resource_limits()?;
    println!();

    // Test error handling
    test_error_handling()?;
    println!();

    println!("=== All tests completed successfully! ===");
    Ok(())
}

fn test_policy_engine_creation() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Policy Engine Creation...");

    // Test default configuration
    let config = PolicyEngineConfig::default();
    let engine = PolicyEngine::new(config)?;
    println!("  ✓ Default policy engine created");

    // Test custom configuration
    let custom_config = PolicyEngineConfig {
        max_execution_time: Duration::from_millis(500),
        max_memory_bytes: 128 * 1024 * 1024, // 128MB
        enable_audit_logging: true,
        audit_log_path: Some(std::path::PathBuf::from("/tmp/policy_audit.log")),
        policy_cache_size: 200,
        enable_metrics: true,
    };
    
    let _custom_engine = PolicyEngine::new(custom_config)?;
    println!("  ✓ Custom policy engine created");

    // Test statistics
    let stats = engine.get_stats();
    assert_eq!(stats.total_evaluations, 0);
    assert_eq!(stats.successful_evaluations, 0);
    assert_eq!(stats.failed_evaluations, 0);
    println!("  ✓ Engine statistics initialized correctly");

    // Test cache operations
    engine.clear_cache();
    let cache_info = engine.get_cache_info();
    assert!(cache_info.is_empty());
    println!("  ✓ Cache operations working correctly");

    Ok(())
}

fn test_resource_limits() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Resource Limits...");

    // Test default resource limits
    use polymera_policy::ResourceLimits;
    let default_limits = ResourceLimits::default();
    assert_eq!(default_limits.max_memory_bytes, 64 * 1024 * 1024); // 64MB
    assert_eq!(default_limits.max_execution_time, Duration::from_millis(100));
    assert_eq!(default_limits.max_function_calls, 1000);
    println!("  ✓ Default resource limits correct");

    // Test custom resource limits
    let custom_limits = ResourceLimits {
        max_memory_bytes: 128 * 1024 * 1024, // 128MB
        max_execution_time: Duration::from_millis(200),
        max_function_calls: 2000,
        max_table_size: 20000,
        max_instances: 20,
        max_tables: 20,
        max_memories: 20,
    };
    
    assert_eq!(custom_limits.max_memory_bytes, 128 * 1024 * 1024);
    assert_eq!(custom_limits.max_execution_time, Duration::from_millis(200));
    assert_eq!(custom_limits.max_function_calls, 2000);
    println!("  ✓ Custom resource limits correct");

    // Test WASM host with custom limits
    use polymera_policy::WasmHost;
    let wasm_host = WasmHost::with_limits(custom_limits)?;
    println!("  ✓ WASM host with custom limits created");

    // Test host function names
    let function_names = wasm_host.get_function_names();
    assert!(function_names.contains(&"math_max".to_string()));
    assert!(function_names.contains(&"math_min".to_string()));
    assert!(function_names.contains(&"time_now".to_string()));
    println!("  ✓ Host function registration working");

    Ok(())
}

fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Error Handling...");

    // Test policy engine error types
    use polymera_policy::PolicyEngineError;
    
    let policy_load_error = PolicyEngineError::PolicyLoadError(
        "File not found".to_string()
    );
    assert!(policy_load_error.to_string().contains("Failed to load policy file"));
    println!("  ✓ Policy load error handling working");

    let wasm_compile_error = PolicyEngineError::WasmCompileError(
        "Invalid WASM".to_string()
    );
    assert!(wasm_compile_error.to_string().contains("Failed to compile WASM module"));
    println!("  ✓ WASM compile error handling working");

    let evaluation_error = PolicyEngineError::EvaluationError(
        "Policy execution failed".to_string()
    );
    assert!(evaluation_error.to_string().contains("Policy evaluation failed"));
    println!("  ✓ Policy evaluation error handling working");

    let timeout_error = PolicyEngineError::ExecutionTimeout(
        Duration::from_millis(100)
    );
    assert!(timeout_error.to_string().contains("Policy execution timeout"));
    println!("  ✓ Execution timeout error handling working");

    let memory_error = PolicyEngineError::MemoryLimitExceeded(
        128 * 1024 * 1024
    );
    assert!(memory_error.to_string().contains("Memory limit exceeded"));
    println!("  ✓ Memory limit error handling working");

    // Test WASM host error types
    use polymera_policy::WasmHostError;
    
    let host_function_error = WasmHostError::HostFunctionNotFound(
        "nonexistent_function".to_string()
    );
    assert!(host_function_error.to_string().contains("Host function not found"));
    println!("  ✓ Host function error handling working");

    let resource_limit_error = WasmHostError::ResourceLimitExceeded(
        "Memory limit exceeded".to_string()
    );
    assert!(resource_limit_error.to_string().contains("Resource limit exceeded"));
    println!("  ✓ Resource limit error handling working");

    Ok(())
}
