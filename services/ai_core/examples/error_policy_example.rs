/**
 * @file error_policy_example.rs
 * @brief Example demonstrating AI Core Service error policy and retry system
 */
use aetheris_ai_core::error::*;
use std::time::Duration;
use tokio::time::sleep;

type ExampleResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> ExampleResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 AI Core Service Error Policy Example");
    println!("======================================");

    // Initialize error handling
    init_global_error_audit_logger();

    // Load error hints from YAML
    let errors_yaml_path = "assets/errors.yaml";
    if std::path::Path::new(errors_yaml_path).exists() {
        init_global_error_hints_manager(errors_yaml_path)?;
        println!("✅ Error hints loaded from {}", errors_yaml_path);
    } else {
        println!("⚠️  Error hints file not found at {}", errors_yaml_path);
    }

    println!();

    // Demonstrate error codes and classifications
    demonstrate_error_codes();

    println!();

    // Demonstrate retry policies
    demonstrate_retry_policies().await?;

    println!();

    // Demonstrate exponential backoff
    demonstrate_exponential_backoff().await?;

    println!();

    // Demonstrate deterministic retries
    demonstrate_deterministic_retries().await?;

    println!();

    // Demonstrate error hints
    demonstrate_error_hints();

    println!();

    // Demonstrate CBOR audit logging
    demonstrate_audit_logging().await?;

    println!();
    println!("✅ Error policy example completed successfully!");

    Ok(())
}

fn demonstrate_error_codes() {
    println!("📋 Error Codes and Classifications:");
    println!("===================================");

    let errors = vec![
        (
            "Tool Timeout",
            AiCoreError::tool_timeout("Tool execution timed out"),
        ),
        (
            "Model Timeout",
            AiCoreError::model_timeout("Model inference timed out"),
        ),
        ("Network Error", AiCoreError::network("Connection failed")),
        (
            "Configuration Error",
            AiCoreError::config("Invalid configuration"),
        ),
        (
            "Capability Denied",
            AiCoreError::cap_denied("Insufficient permissions"),
        ),
        (
            "Circuit Breaker Open",
            AiCoreError::circuit_breaker_open("Service temporarily disabled"),
        ),
    ];

    for (name, error) in errors {
        println!("  {} (Code: {}):", name, error.error_code());
        println!("    Retryable: {}", error.is_retryable());
        println!("    Client Error: {}", error.is_client_error());
        println!("    Server Error: {}", error.is_server_error());
        println!();
    }
}

async fn demonstrate_retry_policies() -> ExampleResult<()> {
    println!("🔄 Retry Policies:");
    println!("==================");

    // Create different retry policies
    let policies = vec![
        ("Default", RetryPolicy::default()),
        (
            "Aggressive",
            RetryPolicy {
                max_attempts: 5,
                initial_delay_ms: 50,
                max_delay_ms: 2000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
                seed: None,
            },
        ),
        (
            "Conservative",
            RetryPolicy {
                max_attempts: 2,
                initial_delay_ms: 500,
                max_delay_ms: 10000,
                backoff_multiplier: 3.0,
                jitter_factor: 0.2,
                seed: None,
            },
        ),
        (
            "Deterministic",
            RetryPolicy {
                max_attempts: 3,
                initial_delay_ms: 100,
                max_delay_ms: 1000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
                seed: Some(12345),
            },
        ),
    ];

    for (name, policy) in policies {
        println!("  {} Policy:", name);
        println!("    Max attempts: {}", policy.max_attempts);
        println!("    Initial delay: {}ms", policy.initial_delay_ms);
        println!("    Max delay: {}ms", policy.max_delay_ms);
        println!("    Backoff multiplier: {}", policy.backoff_multiplier);
        println!("    Jitter factor: {}", policy.jitter_factor);
        println!("    Seed: {:?}", policy.seed);
        println!();
    }

    Ok(())
}

async fn demonstrate_exponential_backoff() -> ExampleResult<()> {
    println!("⏱️  Exponential Backoff Demonstration:");
    println!("=====================================");

    let policy = RetryPolicy {
        max_attempts: 5,
        initial_delay_ms: 100,
        max_delay_ms: 2000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.0, // No jitter for clear demonstration
        seed: None,
    };

    let mut context = RetryContext::new(policy);

    println!("  Attempt delays with exponential backoff:");
    for i in 1..=5 {
        if i > 1 {
            let delay = context.next_delay();
            println!("    Attempt {}: {}ms", i, delay.as_millis());
        } else {
            println!("    Attempt {}: 0ms (first attempt)", i);
        }

        context.record_attempt(AiCoreError::network(&format!("Attempt {}", i)));
    }

    println!();
    Ok(())
}

async fn demonstrate_deterministic_retries() -> ExampleResult<()> {
    println!("🎲 Deterministic Retries Demonstration:");
    println!("======================================");

    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay_ms: 100,
        max_delay_ms: 1000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
        seed: Some(12345), // Fixed seed for deterministic behavior
    };

    // Simulate a failing operation with retries
    let mut attempt_count = 0;
    let start_time = std::time::Instant::now();

    let result = retry_with_backoff(policy, || {
        attempt_count += 1;
        Box::pin(async move {
            println!("    Attempt {}: Executing operation...", attempt_count);

            if attempt_count < 3 {
                Err(AiCoreError::network("Temporary network failure"))
            } else {
                Ok("Operation succeeded!")
            }
        })
    })
    .await;

    let duration = start_time.elapsed();

    match result {
        Ok(success) => {
            println!("    ✅ {}", success);
            println!("    Total attempts: {}", attempt_count);
            println!("    Total duration: {:?}", duration);
        }
        Err(error) => {
            println!("    ❌ Operation failed: {}", error);
            println!("    Total attempts: {}", attempt_count);
            println!("    Total duration: {:?}", duration);
        }
    }

    println!();
    Ok(())
}

fn demonstrate_error_hints() {
    println!("💡 Error Hints Demonstration:");
    println!("=============================");

    let errors = vec![
        AiCoreError::tool_timeout("Tool execution timed out"),
        AiCoreError::config("Invalid configuration"),
        AiCoreError::network("Connection failed"),
        AiCoreError::cap_denied("Insufficient permissions"),
    ];

    for error in errors {
        println!("  Error: {}", error);
        println!("    Code: {}", error.error_code());

        if let Some(hint) = get_error_hint(&error) {
            println!("    Title: {}", hint.title);
            println!("    Message: {}", hint.message);
            println!("    Hint: {}", hint.hint);
            println!("    Severity: {}", hint.severity);
            println!("    Retryable: {}", hint.retryable);
            println!("    User Action: {}", hint.user_action);
        } else {
            println!("    No hint available for this error code");
        }
        println!();
    }
}

async fn demonstrate_audit_logging() -> ExampleResult<()> {
    println!("📝 CBOR Audit Logging Demonstration:");
    println!("====================================");

    // Create some test errors with retry contexts
    let errors = vec![
        (
            AiCoreError::tool_timeout("Tool execution timed out"),
            "tool_timeout_test",
        ),
        (AiCoreError::network("Connection failed"), "network_test"),
        (AiCoreError::config("Invalid configuration"), "config_test"),
    ];

    for (error, test_name) in errors {
        println!("  Logging error: {}", test_name);

        // Create a retry context
        let policy = RetryPolicy::default();
        let mut context = RetryContext::new(policy);

        // Simulate some retry attempts
        for i in 1..=2 {
            context.record_attempt(error.clone());
            if i < 2 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        // Log the error
        log_error_audit(&error, Some(context));
    }

    // Export audit entries
    if let Some(logger) = get_global_error_audit_logger() {
        let cbor_data = logger.export_cbor()?;
        println!("  ✅ Exported {} bytes of audit data", cbor_data.len());

        // Parse and display some audit information
        let entries: Vec<ErrorAuditEntry> = serde_cbor::from_slice(&cbor_data)?;
        println!("  📊 Total audit entries: {}", entries.len());

        for (i, entry) in entries.iter().enumerate() {
            println!(
                "    Entry {}: Code {}, {} retry attempts",
                i + 1,
                entry.error_code,
                entry.retry_attempts.len()
            );
        }
    }

    println!();
    Ok(())
}

// Helper function to simulate a failing operation
async fn simulate_failing_operation(
    attempt: u32,
    should_succeed_after: u32,
) -> std::result::Result<String, AiCoreError> {
    if attempt < should_succeed_after {
        Err(AiCoreError::network(&format!(
            "Simulated failure on attempt {}",
            attempt
        )))
    } else {
        Ok(format!("Success on attempt {}", attempt))
    }
}

// Example of using retry_with_backoff in a real scenario
async fn demonstrate_real_world_retry() -> ExampleResult<()> {
    println!("🌍 Real-World Retry Scenario:");
    println!("=============================");

    // Simulate a tool that fails twice then succeeds
    let policy = RetryPolicy {
        max_attempts: 5,
        initial_delay_ms: 100,
        max_delay_ms: 2000,
        backoff_multiplier: 2.0,
        jitter_factor: 0.1,
        seed: Some(42),
    };

    let result = retry_with_backoff(policy, || {
        static mut ATTEMPT: u32 = 0;
        unsafe {
            ATTEMPT += 1;
        }
        Box::pin(async move { simulate_failing_operation(unsafe { ATTEMPT }, 3).await })
    })
    .await;

    match result {
        Ok(success) => println!("  ✅ {}", success),
        Err(error) => println!("  ❌ Failed: {}", error),
    }

    Ok(())
}
