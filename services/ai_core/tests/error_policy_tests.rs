/**
 * @file error_policy_tests.rs
 * @brief Comprehensive tests for AI Core Service error policy and retry system
 */

use aetheris_ai_core::error::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_error_codes_and_classifications() {
    // Test tool timeout error
    let tool_timeout = AiCoreError::tool_timeout("Tool execution timed out");
    assert_eq!(tool_timeout.error_code(), 1023);
    assert!(tool_timeout.is_retryable());
    assert!(!tool_timeout.is_client_error());
    assert!(tool_timeout.is_server_error());
    
    // Test model timeout error
    let model_timeout = AiCoreError::model_timeout("Model inference timed out");
    assert_eq!(model_timeout.error_code(), 1024);
    assert!(model_timeout.is_retryable());
    
    // Test configuration error
    let config_error = AiCoreError::config("Invalid configuration");
    assert_eq!(config_error.error_code(), 1001);
    assert!(!config_error.is_retryable());
    assert!(config_error.is_client_error());
    
    // Test network error
    let network_error = AiCoreError::network("Connection failed");
    assert_eq!(network_error.error_code(), 1009);
    assert!(network_error.is_retryable());
}

#[tokio::test]
async fn test_retry_policy_creation() {
    let policy = RetryPolicy {
        max_attempts: 5,
        initial_delay_ms: 200,
        max_delay_ms: 10000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
        seed: Some(12345),
    };
    
    assert_eq!(policy.max_attempts, 5);
    assert_eq!(policy.initial_delay_ms, 200);
    assert_eq!(policy.max_delay_ms, 10000);
    assert_eq!(policy.backoff_multiplier, 2.0);
    assert_eq!(policy.jitter_factor, 0.1);
    assert_eq!(policy.seed, Some(12345));
}

#[tokio::test]
async fn test_retry_context_creation() {
    let policy = RetryPolicy::default();
    let context = RetryContext::new(policy);
    
    assert_eq!(context.attempt, 0);
    assert!(context.attempts().is_empty());
    assert!(context.last_error().is_none());
    assert!(!context.is_exhausted());
}

#[tokio::test]
async fn test_retry_context_should_retry() {
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay_ms: 100,
        max_delay_ms: 1000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.0,
        seed: None,
    };
    
    let context = RetryContext::new(policy);
    
    // Test retryable error
    let retryable_error = AiCoreError::network("Connection failed");
    assert!(context.should_retry(&retryable_error));
    
    // Test non-retryable error
    let non_retryable_error = AiCoreError::config("Invalid config");
    assert!(!context.should_retry(&non_retryable_error));
}

#[tokio::test]
async fn test_retry_context_delay_calculation() {
    let policy = RetryPolicy {
        max_attempts: 5,
        initial_delay_ms: 100,
        max_delay_ms: 1000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.0, // No jitter for deterministic testing
        seed: None,
    };
    
    let context = RetryContext::new(policy);
    
    // First attempt should have no delay
    assert_eq!(context.next_delay(), Duration::from_millis(0));
    
    // Create a context with attempts to test delay calculation
    let mut context = RetryContext::new(policy);
    context.record_attempt(AiCoreError::network("Test error"));
    
    // Second attempt should have initial delay
    let delay = context.next_delay();
    assert_eq!(delay, Duration::from_millis(100));
    
    // Record another attempt
    context.record_attempt(AiCoreError::network("Test error"));
    
    // Third attempt should have doubled delay
    let delay = context.next_delay();
    assert_eq!(delay, Duration::from_millis(200));
    
    // Record another attempt
    context.record_attempt(AiCoreError::network("Test error"));
    
    // Fourth attempt should have quadrupled delay
    let delay = context.next_delay();
    assert_eq!(delay, Duration::from_millis(400));
}

#[tokio::test]
async fn test_retry_context_delay_capping() {
    let policy = RetryPolicy {
        max_attempts: 10,
        initial_delay_ms: 100,
        max_delay_ms: 500, // Cap at 500ms
        backoff_multiplier: 2.0,
        jitter_factor: 0.0,
        seed: None,
    };
    
    let mut context = RetryContext::new(policy);
    
    // Record several attempts to exceed the cap
    for _ in 0..5 {
        context.record_attempt(AiCoreError::network("Test error"));
    }
    
    // Delay should be capped at max_delay_ms
    let delay = context.next_delay();
    assert_eq!(delay, Duration::from_millis(500));
}

#[tokio::test]
async fn test_deterministic_retry_with_seed() {
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay_ms: 100,
        max_delay_ms: 1000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
        seed: Some(12345), // Fixed seed for deterministic behavior
    };
    
    let mut context1 = RetryContext::new(policy.clone());
    let mut context2 = RetryContext::new(policy);
    
    // Record the same attempts
    context1.record_attempt(AiCoreError::network("Test error"));
    context2.record_attempt(AiCoreError::network("Test error"));
    
    // Delays should be identical with the same seed
    let delay1 = context1.next_delay();
    let delay2 = context2.next_delay();
    assert_eq!(delay1, delay2);
}

#[tokio::test]
async fn test_retry_context_attempt_recording() {
    let policy = RetryPolicy::default();
    let mut context = RetryContext::new(policy);
    
    // Record first attempt
    let error1 = AiCoreError::network("First error");
    context.record_attempt(error1.clone());
    
    assert_eq!(context.attempt, 1);
    assert_eq!(context.attempts().len(), 1);
    assert_eq!(context.attempts()[0].attempt, 1);
    assert_eq!(context.attempts()[0].error_code, error1.error_code());
    assert_eq!(context.attempts()[0].is_final, false);
    
    // Record second attempt
    let error2 = AiCoreError::timeout("Second error");
    context.record_attempt(error2.clone());
    
    assert_eq!(context.attempt, 2);
    assert_eq!(context.attempts().len(), 2);
    assert_eq!(context.attempts()[1].attempt, 2);
    assert_eq!(context.attempts()[1].error_code, error2.error_code());
}

#[tokio::test]
async fn test_retry_context_exhaustion() {
    let policy = RetryPolicy {
        max_attempts: 2,
        initial_delay_ms: 100,
        max_delay_ms: 1000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.0,
        seed: None,
    };
    
    let mut context = RetryContext::new(policy);
    
    // First attempt
    context.record_attempt(AiCoreError::network("Test error"));
    assert!(!context.is_exhausted());
    
    // Second attempt
    context.record_attempt(AiCoreError::network("Test error"));
    assert!(context.is_exhausted());
}

#[tokio::test]
async fn test_error_audit_logger() {
    let logger = ErrorAuditLogger::new();
    
    // Log an error
    let error = AiCoreError::tool_timeout("Tool execution timed out");
    let context = RetryContext::new(RetryPolicy::default());
    logger.log_error(&error, Some(context));
    
    // Export as CBOR
    let cbor_data = logger.export_cbor().unwrap();
    assert!(!cbor_data.is_empty());
    
    // Clear entries
    logger.clear();
    let cbor_data_after_clear = logger.export_cbor().unwrap();
    assert!(cbor_data_after_clear.is_empty());
}

#[tokio::test]
async fn test_retry_with_backoff_success() {
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay_ms: 10, // Short delay for testing
        max_delay_ms: 100,
        backoff_multiplier: 2.0,
        jitter_factor: 0.0,
        seed: None,
    };
    
    let mut attempt_count = 0;
    let result = retry_with_backoff(policy, || {
        attempt_count += 1;
        Box::pin(async move {
            if attempt_count == 1 {
                Err(AiCoreError::network("Temporary failure"))
            } else {
                Ok("success")
            }
        })
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(attempt_count, 2);
}

#[tokio::test]
async fn test_retry_with_backoff_exhaustion() {
    let policy = RetryPolicy {
        max_attempts: 2,
        initial_delay_ms: 10,
        max_delay_ms: 100,
        backoff_multiplier: 2.0,
        jitter_factor: 0.0,
        seed: None,
    };
    
    let mut attempt_count = 0;
    let result = retry_with_backoff(policy, || {
        attempt_count += 1;
        Box::pin(async move {
            Err(AiCoreError::network("Persistent failure"))
        })
    }).await;
    
    assert!(result.is_err());
    assert_eq!(attempt_count, 2);
}

#[tokio::test]
async fn test_retry_with_backoff_non_retryable_error() {
    let policy = RetryPolicy::default();
    
    let mut attempt_count = 0;
    let result = retry_with_backoff(policy, || {
        attempt_count += 1;
        Box::pin(async move {
            Err(AiCoreError::config("Configuration error"))
        })
    }).await;
    
    assert!(result.is_err());
    assert_eq!(attempt_count, 1); // Should not retry non-retryable errors
}

#[tokio::test]
async fn test_error_hints_manager_creation() {
    let manager = ErrorHintsManager::new();
    
    assert!(manager.get_all_hints().is_empty());
    assert!(manager.get_all_categories().is_empty());
}

#[tokio::test]
async fn test_error_hints_manager_load_from_yaml() {
    // Create a temporary YAML file for testing
    let yaml_content = r#"
errors:
  "1023":
    code: "E_TOOL_TIMEOUT"
    title: "Tool Timeout"
    message: "The tool execution timed out."
    hint: "The tool may be taking longer than expected."
    severity: "warning"
    retryable: true
    user_action: "Retry the tool execution"
  "1001":
    code: "E_CONFIG_ERROR"
    title: "Configuration Error"
    message: "Configuration error occurred."
    hint: "Check your configuration."
    severity: "error"
    retryable: false
    user_action: "Fix configuration"

categories:
  timeout:
    codes: [1023]
    title: "Timeout Issues"
    description: "Errors related to timeouts"

retry_policies:
  default:
    max_attempts: 3
    initial_delay_ms: 100
    max_delay_ms: 5000
    backoff_multiplier: 2.0
    jitter_factor: 0.1

user_actions:
  retry:
    - "Wait and try again"
    - "Check network connection"
"#;
    
    let temp_file = std::env::temp_dir().join("test_errors.yaml");
    std::fs::write(&temp_file, yaml_content).unwrap();
    
    let manager = ErrorHintsManager::load_from_yaml(&temp_file).unwrap();
    
    // Test error hint loading
    let hint = manager.get_hint(1023).unwrap();
    assert_eq!(hint.code, "E_TOOL_TIMEOUT");
    assert_eq!(hint.title, "Tool Timeout");
    assert!(hint.retryable);
    
    let hint = manager.get_hint(1001).unwrap();
    assert_eq!(hint.code, "E_CONFIG_ERROR");
    assert_eq!(hint.title, "Configuration Error");
    assert!(!hint.retryable);
    
    // Test category loading
    let category = manager.get_category(1023).unwrap();
    assert_eq!(category.title, "Timeout Issues");
    
    // Test retry policy loading
    let policy = manager.get_retry_policy("default").unwrap();
    assert_eq!(policy.max_attempts, 3);
    assert_eq!(policy.initial_delay_ms, 100);
    
    // Test user actions loading
    let actions = manager.get_user_actions("retry").unwrap();
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[0], "Wait and try again");
    
    // Clean up
    std::fs::remove_file(&temp_file).unwrap();
}

#[tokio::test]
async fn test_global_error_hints_manager() {
    // Create a temporary YAML file
    let yaml_content = r#"
errors:
  "1023":
    code: "E_TOOL_TIMEOUT"
    title: "Tool Timeout"
    message: "The tool execution timed out."
    hint: "The tool may be taking longer than expected."
    severity: "warning"
    retryable: true
    user_action: "Retry the tool execution"

retry_policies:
  default:
    max_attempts: 3
    initial_delay_ms: 100
    max_delay_ms: 5000
    backoff_multiplier: 2.0
    jitter_factor: 0.1
"#;
    
    let temp_file = std::env::temp_dir().join("test_global_errors.yaml");
    std::fs::write(&temp_file, yaml_content).unwrap();
    
    // Initialize global manager
    init_global_error_hints_manager(&temp_file).unwrap();
    
    // Test getting error hint
    let error = AiCoreError::tool_timeout("Test timeout");
    let hint = get_error_hint(&error).unwrap();
    assert_eq!(hint.code, "E_TOOL_TIMEOUT");
    assert_eq!(hint.title, "Tool Timeout");
    
    // Test getting retry policy
    let policy = get_retry_policy_for_error(&error);
    assert_eq!(policy.max_attempts, 3);
    assert_eq!(policy.initial_delay_ms, 100);
    
    // Clean up
    std::fs::remove_file(&temp_file).unwrap();
}

#[tokio::test]
async fn test_xor_shift_rng() {
    let mut rng1 = XorShiftRng::new(12345);
    let mut rng2 = XorShiftRng::new(12345);
    
    // Same seed should produce same sequence
    for _ in 0..10 {
        assert_eq!(rng1.next_f64(), rng2.next_f64());
    }
    
    // Different seeds should produce different sequences
    let mut rng3 = XorShiftRng::new(54321);
    assert_ne!(rng1.next_f64(), rng3.next_f64());
}

#[tokio::test]
async fn test_error_audit_entry_serialization() {
    let entry = ErrorAuditEntry {
        error_code: 1023,
        error_message: "Tool timeout".to_string(),
        error_type: "ToolTimeout".to_string(),
        timestamp: 1234567890,
        session_id: Some("session-123".to_string()),
        request_id: Some("request-456".to_string()),
        retry_attempts: vec![
            RetryAttempt {
                attempt: 1,
                delay_ms: 0,
                timestamp: 1234567890,
                error: "Network error".to_string(),
                error_code: 1009,
                is_final: false,
            }
        ],
        context: std::collections::HashMap::new(),
        stack_trace: None,
    };
    
    // Test JSON serialization
    let json = serde_json::to_string(&entry).unwrap();
    let deserialized: ErrorAuditEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.error_code, entry.error_code);
    assert_eq!(deserialized.error_message, entry.error_message);
    
    // Test CBOR serialization
    let cbor = serde_cbor::to_vec(&entry).unwrap();
    let deserialized_cbor: ErrorAuditEntry = serde_cbor::from_slice(&cbor).unwrap();
    assert_eq!(deserialized_cbor.error_code, entry.error_code);
    assert_eq!(deserialized_cbor.error_message, entry.error_message);
}

#[tokio::test]
async fn test_retry_policy_config_conversion() {
    let config = RetryPolicyConfig {
        max_attempts: 5,
        initial_delay_ms: 200,
        max_delay_ms: 10000,
        backoff_multiplier: 2.5,
        jitter_factor: 0.2,
    };
    
    let policy: RetryPolicy = config.into();
    
    assert_eq!(policy.max_attempts, 5);
    assert_eq!(policy.initial_delay_ms, 200);
    assert_eq!(policy.max_delay_ms, 10000);
    assert_eq!(policy.backoff_multiplier, 2.5);
    assert_eq!(policy.jitter_factor, 0.2);
    assert_eq!(policy.seed, None);
}

#[tokio::test]
async fn test_error_hint_serialization() {
    let hint = ErrorHint {
        code: "E_TOOL_TIMEOUT".to_string(),
        title: "Tool Timeout".to_string(),
        message: "The tool execution timed out.".to_string(),
        hint: "The tool may be taking longer than expected.".to_string(),
        severity: "warning".to_string(),
        retryable: true,
        user_action: "Retry the tool execution".to_string(),
    };
    
    // Test JSON serialization
    let json = serde_json::to_string(&hint).unwrap();
    let deserialized: ErrorHint = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.code, hint.code);
    assert_eq!(deserialized.title, hint.title);
    assert_eq!(deserialized.retryable, hint.retryable);
    
    // Test CBOR serialization
    let cbor = serde_cbor::to_vec(&hint).unwrap();
    let deserialized_cbor: ErrorHint = serde_cbor::from_slice(&cbor).unwrap();
    assert_eq!(deserialized_cbor.code, hint.code);
    assert_eq!(deserialized_cbor.title, hint.title);
    assert_eq!(deserialized_cbor.retryable, hint.retryable);
}

#[tokio::test]
async fn test_retry_context_total_duration() {
    let policy = RetryPolicy::default();
    let context = RetryContext::new(policy);
    
    // Wait a bit
    sleep(Duration::from_millis(10)).await;
    
    let duration = context.total_duration();
    assert!(duration >= Duration::from_millis(10));
}

#[tokio::test]
async fn test_error_audit_logger_thread_safety() {
    let logger = std::sync::Arc::new(ErrorAuditLogger::new());
    let mut handles = vec![];
    
    // Spawn multiple threads to log errors concurrently
    for i in 0..10 {
        let logger_clone = logger.clone();
        let handle = tokio::spawn(async move {
            let error = AiCoreError::network(&format!("Error {}", i));
            let context = RetryContext::new(RetryPolicy::default());
            logger_clone.log_error(&error, Some(context));
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Export and verify all entries were logged
    let cbor_data = logger.export_cbor().unwrap();
    let entries: Vec<ErrorAuditEntry> = serde_cbor::from_slice(&cbor_data).unwrap();
    assert_eq!(entries.len(), 10);
}
