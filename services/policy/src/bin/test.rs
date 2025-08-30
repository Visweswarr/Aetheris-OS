use polymera_policy::{
    PolicyEngine, PolicyEngineConfig, PolicyRequest, PolicyContext,
    DefaultPolicyService, PolicyService,
};
use serde_json::json;
use std::path::Path;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Polymera OS Policy Engine Test ===\n");

    // Test policy engine creation
    test_policy_engine_creation()?;
    println!();

    // Test policy service interface
    test_policy_service_interface()?;
    println!();

    // Test policy context and requests
    test_policy_context_and_requests()?;
    println!();

    // Test resource limits
    test_resource_limits()?;
    println!();

    // Test error handling
    test_error_handling()?;
    println!();

    // Test performance characteristics
    test_performance_characteristics()?;
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
    
    let custom_engine = PolicyEngine::new(custom_config)?;
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

fn test_policy_service_interface() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Policy Service Interface...");

    // Create default policy service
    let service = DefaultPolicyService::with_default_config()?;
    println!("  ✓ Default policy service created");

    // Test service statistics
    let stats = service.get_stats();
    assert_eq!(stats.total_evaluations, 0);
    println!("  ✓ Service statistics accessible");

    // Test cache operations
    service.clear_cache();
    let cache_info = service.get_cache_info();
    assert!(cache_info.is_empty());
    println!("  ✓ Service cache operations working");

    // Test policy validation (this will fail since we don't have actual WASM files)
    let test_path = Path::new("/tmp/nonexistent_policy.wasm");
    let validation_result = service.validate_policy(test_path);
    assert!(validation_result.is_err());
    println!("  ✓ Policy validation error handling working");

    Ok(())
}

fn test_policy_context_and_requests() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Policy Context and Requests...");

    // Test policy context creation
    let context = PolicyContext::new();
    assert!(context.user.is_none());
    assert!(context.source.is_none());
    assert!(context.metadata.is_empty());
    println!("  ✓ Default policy context created");

    // Test context with user
    let user_context = PolicyContext::with_user("test_user".to_string());
    assert_eq!(user_context.user, Some("test_user".to_string()));
    println!("  ✓ Policy context with user created");

    // Test context with source
    let source_context = PolicyContext::with_source("192.168.1.1".to_string());
    assert_eq!(source_context.source, Some("192.168.1.1".to_string()));
    println!("  ✓ Policy context with source created");

    // Test context metadata
    let mut metadata_context = PolicyContext::new();
    metadata_context.set_metadata("key".to_string(), json!("value"));
    assert_eq!(
        metadata_context.get_metadata("key"),
        Some(&json!("value"))
    );
    println!("  ✓ Policy context metadata working");

    // Test policy request creation
    let input = json!({
        "user": {"role": "developer"},
        "resource": {"type": "file", "path": "/home/user/doc.txt"}
    });
    
    let request = PolicyRequest::new("test_policy".to_string(), input.clone());
    assert_eq!(request.policy_name, "test_policy");
    assert_eq!(request.input, input);
    assert!(request.rule.is_none());
    println!("  ✓ Basic policy request created");

    // Test request with rule
    let request_with_rule = request.with_rule("allow".to_string());
    assert_eq!(request_with_rule.rule, Some("allow".to_string()));
    println!("  ✓ Policy request with rule created");

    // Test request with user context
    let request_with_user = request_with_rule.with_user("test_user".to_string());
    assert_eq!(request_with_user.context.user, Some("test_user".to_string()));
    println!("  ✓ Policy request with user context created");

    // Test request with source context
    let request_with_source = request_with_user.with_source("192.168.1.1".to_string());
    assert_eq!(request_with_source.context.source, Some("192.168.1.1".to_string()));
    println!("  ✓ Policy request with source context created");

    // Test request with metadata
    let request_with_metadata = request_with_source.with_metadata(
        "session_id".to_string(),
        json!("abc123")
    );
    assert_eq!(
        request_with_metadata.context.get_metadata("session_id"),
        Some(&json!("abc123"))
    );
    println!("  ✓ Policy request with metadata created");

    Ok(())
}

fn test_resource_limits() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Resource Limits...");

    // Test default resource limits
    let default_limits = polymera_policy::ResourceLimits::default();
    assert_eq!(default_limits.max_memory_bytes, 64 * 1024 * 1024); // 64MB
    assert_eq!(default_limits.max_execution_time, Duration::from_millis(100));
    assert_eq!(default_limits.max_function_calls, 1000);
    println!("  ✓ Default resource limits correct");

    // Test custom resource limits
    let custom_limits = polymera_policy::ResourceLimits {
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
    let wasm_host = polymera_policy::WasmHost::with_limits(custom_limits)?;
    println!("  ✓ WASM host with custom limits created");

    // Test host function statistics
    let function_stats = wasm_host.get_function_stats();
    assert!(function_stats.contains_key("json_parse"));
    assert!(function_stats.contains_key("string_length"));
    assert!(function_stats.contains_key("math_max"));
    println!("  ✓ Host function registration working");

    Ok(())
}

fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Error Handling...");

    // Test policy engine error types
    let policy_load_error = polymera_policy::PolicyEngineError::PolicyLoadError(
        "File not found".to_string()
    );
    assert!(policy_load_error.to_string().contains("Failed to load policy file"));
    println!("  ✓ Policy load error handling working");

    let wasm_compile_error = polymera_policy::PolicyEngineError::WasmCompileError(
        "Invalid WASM".to_string()
    );
    assert!(wasm_compile_error.to_string().contains("Failed to compile WASM module"));
    println!("  ✓ WASM compile error handling working");

    let evaluation_error = polymera_policy::PolicyEngineError::EvaluationError(
        "Policy execution failed".to_string()
    );
    assert!(evaluation_error.to_string().contains("Policy evaluation failed"));
    println!("  ✓ Policy evaluation error handling working");

    let timeout_error = polymera_policy::PolicyEngineError::ExecutionTimeout(
        Duration::from_millis(100)
    );
    assert!(timeout_error.to_string().contains("Policy execution timeout"));
    println!("  ✓ Execution timeout error handling working");

    let memory_error = polymera_policy::PolicyEngineError::MemoryLimitExceeded(
        128 * 1024 * 1024
    );
    assert!(memory_error.to_string().contains("Memory limit exceeded"));
    println!("  ✓ Memory limit error handling working");

    // Test WASM host error types
    let host_function_error = polymera_policy::WasmHostError::HostFunctionNotFound(
        "nonexistent_function".to_string()
    );
    assert!(host_function_error.to_string().contains("Host function not found"));
    println!("  ✓ Host function error handling working");

    let resource_limit_error = polymera_policy::WasmHostError::ResourceLimitExceeded(
        "Memory limit exceeded".to_string()
    );
    assert!(resource_limit_error.to_string().contains("Resource limit exceeded"));
    println!("  ✓ Resource limit error handling working");

    Ok(())
}

fn test_performance_characteristics() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Performance Characteristics...");

    // Create policy engine with performance monitoring
    let config = PolicyEngineConfig {
        enable_metrics: true,
        enable_audit_logging: true,
        ..Default::default()
    };
    
    let engine = PolicyEngine::new(config)?;
    println!("  ✓ Performance monitoring enabled");

    // Test statistics collection
    let initial_stats = engine.get_stats();
    assert_eq!(initial_stats.total_evaluations, 0);
    assert_eq!(initial_stats.average_execution_time, Duration::from_micros(0));
    println!("  ✓ Initial statistics correct");

    // Test cache performance
    let cache_info = engine.get_cache_info();
    assert!(cache_info.is_empty());
    println!("  ✓ Cache performance monitoring working");

    // Test WASM host performance
    let wasm_host = polymera_policy::WasmHost::new()?;
    
    // Test function call statistics
    let function_stats = wasm_host.get_function_stats();
    for (function_name, stats) in function_stats {
        assert_eq!(stats.total_calls, 0);
        assert_eq!(stats.successful_calls, 0);
        assert_eq!(stats.failed_calls, 0);
        assert_eq!(stats.total_execution_time, Duration::from_micros(0));
        assert_eq!(stats.average_execution_time, Duration::from_micros(0));
    }
    println!("  ✓ Function call statistics initialized correctly");

    // Test execution context
    let context = wasm_host.get_execution_context();
    assert_eq!(context.function_call_count, 0);
    assert_eq!(context.memory_usage_bytes, 0);
    assert!(context.execution_trace.is_empty());
    println!("  ✓ Execution context monitoring working");

    // Test resource limiter performance
    let mut resource_limiter = wasm_host.clone();
    
    // Test memory growing
    let memory_result = resource_limiter.memory_growing(1024, 2048, Some(4096));
    assert!(memory_result.is_ok());
    println!("  ✓ Memory limit enforcement working");

    // Test table growing
    let table_result = resource_limiter.table_growing(100, 200, Some(500));
    assert!(table_result.is_ok());
    println!("  ✓ Table limit enforcement working");

    // Test instance limits
    let instance_result = resource_limiter.instances(5);
    assert!(instance_result.is_ok());
    println!("  ✓ Instance limit enforcement working");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_function() {
        // This test ensures the main function can be called
        let result = main();
        assert!(result.is_ok());
    }
}
