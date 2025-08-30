# Polymera OS Logging System

Comprehensive logging and redaction system for Polymera OS services with privacy protection, correlation IDs, and structured logging.

## 🚀 Features

- **JSON & Text Logging**: Structured logging in multiple formats
- **Field Redaction**: Automatic PII protection with configurable rules
- **Correlation IDs**: Request tracing across distributed systems
- **Sampling**: Configurable log sampling for performance
- **Privacy-First**: Zero PII exposure by default
- **Structured Fields**: Rich metadata and context tracking

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
polymera-observability = { path = "../observability", features = ["full"] }
```

## 🔧 Quick Start

### Basic Logging Setup

```rust
use polymera_observability::logging::{
    LoggingConfig, PolymeraLogger, LogLevel, LogFormat
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure logging
    let config = LoggingConfig {
        level: LogLevel::Info,
        format: LogFormat::Json,
        enable_correlation_ids: true,
        structured_logging: true,
        ..Default::default()
    };

    // Create and initialize logger
    let logger = PolymeraLogger::new(config)?;
    logger.init()?;

    // Log with context
    let mut fields = HashMap::new();
    fields.insert("user_id".to_string(), serde_json::Value::String("12345".to_string()));
    fields.insert("action".to_string(), serde_json::Value::String("login".to_string()));
    
    logger.log_with_context(
        LogLevel::Info,
        "User authentication attempt",
        fields,
    ).await;

    Ok(())
}
```

### Using Logging Macros

```rust
use polymera_observability::logging::{log_info, log_error};

async fn process_user(user_id: &str, email: &str) {
    // These fields will be automatically redacted
    log_info!(logger, "Processing user", 
        "user_id" => user_id,
        "email" => email,
        "operation" => "create"
    );
    
    // Handle errors
    if let Err(e) = perform_operation() {
        log_error!(logger, "Operation failed", 
            "error" => e.to_string(),
            "user_id" => user_id
        );
    }
}
```

## 🔒 Privacy & Redaction

### Automatic PII Detection

The system automatically detects and redacts sensitive fields:

```rust
let mut fields = HashMap::new();
fields.insert("username".to_string(), serde_json::Value::String("john_doe".to_string()));
fields.insert("password".to_string(), serde_json::Value::String("secret123".to_string()));
fields.insert("email".to_string(), serde_json::Value::String("john@example.com".to_string()));
fields.insert("credit_card".to_string(), serde_json::Value::String("1234-5678-9012-3456".to_string()));

// Sensitive fields are automatically redacted
logger.log_with_context(LogLevel::Info, "User data", fields).await;
```

**Output:**
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "INFO",
  "message": "User data",
  "fields": {
    "username": "john_doe",
    "password": "[REDACTED]",
    "email": "[REDACTED]",
    "credit_card": "[REDACTED]"
  }
}
```

### Redaction Modes

```rust
use polymera_observability::logging::redact::{RedactionConfig, RedactionMode};

let config = LoggingConfig {
    redaction: RedactionConfig {
        mode: RedactionMode::Hash,  // Hash sensitive values
        // or RedactionMode::Drop,   // Remove sensitive fields entirely
        // or RedactionMode::Partial, // Show partial values (e.g., ****1234)
        ..Default::default()
    },
    ..Default::default()
};
```

### Custom Redaction Rules

```rust
use polymera_observability::logging::redact::{
    RedactionRuleBuilder, RedactionMode
};

let mut config = LoggingConfig::default();
config.redaction.custom_rules.push(
    RedactionRuleBuilder::new("internal_id".to_string())
        .mode(RedactionMode::Hash)
        .priority(10)
        .build()
);

config.redaction.custom_rules.push(
    RedactionRuleBuilder::new("temp_.*".to_string())
        .mode(RedactionMode::Drop)
        .regex()
        .priority(5)
        .build()
);
```

## 🆔 Correlation IDs

### Request Tracing

```rust
use polymera_observability::logging::{CorrelationContext, PolymeraLogger};

async fn handle_request(logger: &PolymeraLogger, request_id: &str) {
    // Create correlation context
    let context = CorrelationContext::with_id(request_id.to_string());
    context.add_field("endpoint".to_string(), "/api/users".to_string());
    context.add_field("method".to_string(), "POST".to_string());
    
    // Set context for current thread
    logger.set_correlation_context(context).await;
    
    // All logs in this thread will include correlation ID
    logger.log_with_context(
        LogLevel::Info,
        "Processing request",
        HashMap::new(),
    ).await;
    
    // Clear context when done
    logger.clear_correlation_context().await;
}
```

### Automatic Correlation Context

```rust
// Automatically create and manage correlation context
let result = logger.with_correlation_context(|| {
    // Your business logic here
    // Correlation ID is automatically available
    perform_operation()
});

// Context is automatically cleared
```

### Cross-Service Correlation

```rust
// Extract correlation ID from incoming request
let correlation_id = extract_correlation_id_from_headers(&request.headers);

// Use existing correlation ID
let result = logger.with_existing_correlation_context(correlation_id, || {
    // Continue the trace across service boundaries
    process_request()
});
```

## 📊 Structured Logging

### Rich Field Metadata

```rust
let mut fields = HashMap::new();
fields.insert("operation".to_string(), serde_json::Value::String("user_creation".to_string()));
fields.insert("user_type".to_string(), serde_json::Value::String("premium".to_string()));
fields.insert("source".to_string(), serde_json::Value::String("api".to_string()));
fields.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));

logger.log_with_context(
    LogLevel::Info,
    "User account created successfully",
    fields,
).await;
```

### Performance Logging

```rust
use std::time::Instant;

async fn expensive_operation(logger: &PolymeraLogger) {
    let start = Instant::now();
    
    // Perform operation
    let result = perform_expensive_task().await;
    
    let duration = start.elapsed();
    
    let mut metadata = HashMap::new();
    metadata.insert("result_size".to_string(), serde_json::Value::Number(serde_json::Number::from(result.len())));
    metadata.insert("cache_hit".to_string(), serde_json::Value::Bool(false));
    
    logger.log_performance("expensive_operation", duration, metadata).await;
}
```

### Request/Response Logging

```rust
async fn log_http_request(
    logger: &PolymeraLogger,
    method: &str,
    path: &str,
    status_code: u16,
    duration: Duration,
    request_id: Option<&str>,
) {
    logger.log_request(method, path, status_code, duration, request_id).await;
}

// Usage
log_http_request(
    &logger,
    "POST",
    "/api/users",
    201,
    Duration::from_millis(150),
    Some("req_12345"),
).await;
```

## 🎯 Sampling

### Configurable Sampling

```rust
let config = LoggingConfig {
    enable_sampling: true,
    sampling_rate: 0.1,  // Log only 10% of entries
    ..Default::default()
};
```

### Conditional Sampling

```rust
// Sample based on log level
let config = LoggingConfig {
    enable_sampling: true,
    sampling_rate: 1.0,  // Full sampling for now
    ..Default::default()
};

// In production, implement more sophisticated sampling
if should_sample_log(level, context) {
    logger.log_with_context(level, message, fields).await;
}
```

## 🔧 Configuration

### Complete Configuration Example

```rust
let config = LoggingConfig {
    level: LogLevel::Debug,
    format: LogFormat::Json,
    output: LogOutput::Stdout,
    enable_correlation_ids: true,
    correlation_header: "X-Request-ID".to_string(),
    structured_logging: true,
    enable_sampling: true,
    sampling_rate: 0.5,
    redaction: RedactionConfig {
        enabled: true,
        mode: RedactionMode::Placeholder,
        placeholder: "[SENSITIVE]".to_string(),
        enable_regex: true,
        case_sensitive: false,
        enable_partial: true,
        partial_length: 4,
        ..Default::default()
    },
    custom_fields: {
        let mut fields = HashMap::new();
        fields.insert("service".to_string(), "user-service".to_string());
        fields.insert("environment".to_string(), "production".to_string());
        fields.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        fields
    },
    enable_request_logging: true,
    enable_performance_logging: true,
};
```

### Environment-Based Configuration

```rust
let config = LoggingConfig {
    level: std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .parse()
        .unwrap_or(LogLevel::Info),
    format: if std::env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()) == "text" {
        LogFormat::Text
    } else {
        LogFormat::Json
    },
    enable_correlation_ids: std::env::var("ENABLE_CORRELATION_IDS")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true),
    ..Default::default()
};
```

## 🧪 Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pii_redaction() {
        let config = LoggingConfig::default();
        let logger = PolymeraLogger::new(config).unwrap();
        
        let mut fields = HashMap::new();
        fields.insert("password".to_string(), serde_json::Value::String("secret123".to_string()));
        fields.insert("email".to_string(), serde_json::Value::String("test@example.com".to_string()));
        
        // Verify sensitive fields are redacted
        let redacted = logger.redaction_engine.redact_fields(&fields);
        assert_eq!(redacted["password"], serde_json::Value::String("[REDACTED]".to_string()));
        assert_eq!(redacted["email"], serde_json::Value::String("[REDACTED]".to_string()));
    }

    #[tokio::test]
    async fn test_correlation_context() {
        let config = LoggingConfig::default();
        let logger = PolymeraLogger::new(config).unwrap();
        
        let context = CorrelationContext::new();
        logger.set_correlation_context(context.clone()).await;
        
        let retrieved = logger.get_correlation_context().await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().correlation_id, context.correlation_id);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_logging_integration() {
    let config = LoggingConfig {
        level: LogLevel::Debug,
        format: LogFormat::Json,
        ..Default::default()
    };
    
    let logger = PolymeraLogger::new(config).unwrap();
    logger.init().unwrap();
    
    // Test logging with correlation context
    let result = logger.with_correlation_context(|| {
        // Simulate request processing
        "success"
    });
    
    assert_eq!(result, "success");
}
```

## 📈 Best Practices

### 1. Always Use Correlation IDs

```rust
// Good: Include correlation ID in all logs
logger.with_correlation_context(|| {
    log_info!(logger, "Processing request", "step" => "validation");
    log_info!(logger, "Request validated", "step" => "processing");
    log_info!(logger, "Request completed", "step" => "success");
});

// Bad: Logging without context
log_info!(logger, "Processing request");
log_info!(logger, "Request completed");
```

### 2. Structure Your Log Fields

```rust
// Good: Consistent field naming
fields.insert("user_id".to_string(), serde_json::Value::String(user_id.to_string()));
fields.insert("operation".to_string(), serde_json::Value::String("user_update".to_string()));
fields.insert("result".to_string(), serde_json::Value::String("success".to_string()));

// Bad: Inconsistent field names
fields.insert("userId".to_string(), serde_json::Value::String(user_id.to_string()));
fields.insert("op".to_string(), serde_json::Value::String("update".to_string()));
```

### 3. Use Appropriate Log Levels

```rust
// Trace: Detailed debugging information
log_trace!(logger, "Entering function", "function" => "process_user");

// Debug: General debugging information
log_debug!(logger, "Processing user data", "user_id" => user_id);

// Info: General information about application flow
log_info!(logger, "User created successfully", "user_id" => user_id);

// Warn: Warning conditions
log_warn!(logger, "Rate limit approaching", "current_rate" => current_rate);

// Error: Error conditions
log_error!(logger, "Failed to create user", "error" => error.to_string());
```

### 4. Include Relevant Context

```rust
// Good: Rich context for debugging
let mut fields = HashMap::new();
fields.insert("user_id".to_string(), serde_json::Value::String(user_id.to_string()));
fields.insert("request_id".to_string(), serde_json::Value::String(request_id.to_string()));
fields.insert("ip_address".to_string(), serde_json::Value::String(ip.to_string()));
fields.insert("user_agent".to_string(), serde_json::Value::String(user_agent.to_string()));

// Bad: Minimal context
let mut fields = HashMap::new();
fields.insert("message".to_string(), serde_json::Value::String("Error occurred".to_string()));
```

## 🔍 Troubleshooting

### Common Issues

1. **Logs not appearing**
   - Check log level configuration
   - Verify logger initialization
   - Check environment variables

2. **PII not being redacted**
   - Verify redaction is enabled
   - Check field name patterns
   - Test with known sensitive fields

3. **Correlation IDs missing**
   - Ensure correlation context is set
   - Check thread safety
   - Verify context propagation

4. **Performance issues**
   - Enable sampling for high-volume logs
   - Use appropriate log levels
   - Consider async logging for heavy operations

### Debug Mode

```rust
let config = LoggingConfig {
    level: LogLevel::Trace,  // Enable all log levels
    format: LogFormat::Text, // Use human-readable format
    ..Default::default()
};
```

### Log Analysis

```bash
# View logs in real-time
tail -f application.log | jq '.'

# Filter by correlation ID
grep "correlation_id.*req_12345" application.log

# Find sensitive data exposure
grep -v "\[REDACTED\]" application.log | grep -E "(password|email|credit_card)"

# Analyze log levels
jq '.level' application.log | sort | uniq -c
```

## 📚 API Reference

### Core Types

- `LoggingConfig` - Main configuration structure
- `PolymeraLogger` - Main logging interface
- `CorrelationContext` - Request correlation management
- `RedactionConfig` - Privacy protection configuration
- `RedactionEngine` - PII redaction engine

### Log Levels

- `LogLevel::Trace` - Most verbose
- `LogLevel::Debug` - Debugging information
- `LogLevel::Info` - General information
- `LogLevel::Warn` - Warning conditions
- `LogLevel::Error` - Error conditions

### Redaction Modes

- `RedactionMode::None` - No redaction
- `RedactionMode::Placeholder` - Replace with placeholder
- `RedactionMode::Hash` - Hash sensitive values
- `RedactionMode::Drop` - Remove fields entirely
- `RedactionMode::Partial` - Show partial values

### Macros

- `log_trace!()` - Trace level logging
- `log_debug!()` - Debug level logging
- `log_info!()` - Info level logging
- `log_warn!()` - Warning level logging
- `log_error!()` - Error level logging

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

MIT OR Apache-2.0

## 🆘 Support

- **Documentation**: [Polymera OS Docs](https://docs.polymera-os.org)
- **Issues**: [GitHub Issues](https://github.com/polymera-os/polymera-os/issues)
- **Discord**: [Polymera OS Community](https://discord.gg/polymera-os)
