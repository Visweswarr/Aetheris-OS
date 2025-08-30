use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub mod redact;

use redact::{RedactionConfig, RedactionEngine, RedactionRule};

/// Logging configuration for Polymera OS services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: LogLevel,
    /// Output format (json, text)
    pub format: LogFormat,
    /// Output destination (stdout, stderr, file)
    pub output: LogOutput,
    /// File path for file output
    pub file_path: Option<String>,
    /// Enable correlation IDs
    pub enable_correlation_ids: bool,
    /// Correlation ID header name
    pub correlation_header: String,
    /// Enable structured logging
    pub structured_logging: bool,
    /// Enable sampling
    pub enable_sampling: bool,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Redaction configuration
    pub redaction: RedactionConfig,
    /// Custom fields to include in all logs
    pub custom_fields: HashMap<String, String>,
    /// Enable request/response logging
    pub enable_request_logging: bool,
    /// Enable performance logging
    pub enable_performance_logging: bool,
}

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

/// Log formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Text,
}

/// Log outputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::Stdout,
            file_path: None,
            enable_correlation_ids: true,
            correlation_header: "X-Correlation-ID".to_string(),
            structured_logging: true,
            enable_sampling: false,
            sampling_rate: 1.0,
            redaction: RedactionConfig::default(),
            custom_fields: HashMap::new(),
            enable_request_logging: true,
            enable_performance_logging: false,
        }
    }
}

/// Correlation ID context for request tracing
#[derive(Debug, Clone)]
pub struct CorrelationContext {
    /// Unique correlation ID
    pub correlation_id: String,
    /// Request start time
    pub start_time: Instant,
    /// Additional context fields
    pub context: HashMap<String, String>,
}

impl CorrelationContext {
    /// Create a new correlation context
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            start_time: Instant::now(),
            context: HashMap::new(),
        }
    }

    /// Create with existing correlation ID
    pub fn with_id(correlation_id: String) -> Self {
        Self {
            correlation_id,
            start_time: Instant::now(),
            context: HashMap::new(),
        }
    }

    /// Add context field
    pub fn add_field(&mut self, key: String, value: String) {
        self.context.insert(key, value);
    }

    /// Get context field
    pub fn get_field(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }

    /// Get request duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for CorrelationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Structured logger for Polymera OS
#[derive(Debug)]
pub struct PolymeraLogger {
    config: LoggingConfig,
    redaction_engine: RedactionEngine,
    correlation_context: Arc<RwLock<Option<CorrelationContext>>>,
}

impl PolymeraLogger {
    /// Create a new logger instance
    pub fn new(config: LoggingConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let redaction_engine = RedactionEngine::new(config.redaction.clone())?;
        
        Ok(Self {
            config,
            redaction_engine,
            correlation_context: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize the logging system
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(self.config.level.to_string().to_lowercase()));

        let mut registry = tracing_subscriber::registry().with(env_filter);

        // Add formatting layer
        match self.config.format {
            LogFormat::Json => {
                let json_layer = fmt::layer()
                    .with_timer(UtcTime::rfc_3339())
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_file(true)
                    .with_line_number(true)
                    .json()
                    .with_current_span(true)
                    .with_span_list(true);

                registry = registry.with(json_layer);
            }
            LogFormat::Text => {
                let text_layer = fmt::layer()
                    .with_timer(UtcTime::rfc_3339())
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_current_span(true)
                    .with_span_list(true);

                registry = registry.with(text_layer);
            }
        }

        // Add sampling if enabled
        if self.config.enable_sampling {
            let sampling_layer = tracing_subscriber::layer::Filter::new(move |metadata| {
                // Simple random sampling - in production use proper sampling strategies
                rand::random::<f64>() < self.config.sampling_rate
            });
            registry = registry.with(sampling_layer);
        }

        // Initialize the subscriber
        registry.init();

        Ok(())
    }

    /// Set correlation context for current thread
    pub async fn set_correlation_context(&self, context: CorrelationContext) {
        let mut ctx = self.correlation_context.write().await;
        *ctx = Some(context);
    }

    /// Get current correlation context
    pub async fn get_correlation_context(&self) -> Option<CorrelationContext> {
        let ctx = self.correlation_context.read().await;
        ctx.clone()
    }

    /// Clear correlation context
    pub async fn clear_correlation_context(&self) {
        let mut ctx = self.correlation_context.write().await;
        *ctx = None;
    }

    /// Log with correlation context
    pub async fn log_with_context(
        &self,
        level: LogLevel,
        message: &str,
        fields: HashMap<String, serde_json::Value>,
    ) {
        let mut log_fields = fields.clone();
        
        // Add correlation context if enabled
        if self.config.enable_correlation_ids {
            if let Some(context) = self.get_correlation_context().await {
                log_fields.insert("correlation_id".to_string(), serde_json::Value::String(context.correlation_id.clone()));
                log_fields.insert("request_duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(context.duration().as_millis() as f64).unwrap()));
                
                // Add custom context fields
                for (key, value) in &context.context {
                    log_fields.insert(format!("ctx_{}", key), serde_json::Value::String(value.clone()));
                }
            }
        }

        // Add custom fields
        for (key, value) in &self.config.custom_fields {
            log_fields.insert(format!("custom_{}", key), serde_json::Value::String(value.clone()));
        }

        // Redact sensitive fields
        let redacted_fields = self.redaction_engine.redact_fields(&log_fields);

        // Log using tracing
        match level {
            LogLevel::Trace => tracing::trace!(?redacted_fields, "{}", message),
            LogLevel::Debug => tracing::debug!(?redacted_fields, "{}", message),
            LogLevel::Info => tracing::info!(?redacted_fields, "{}", message),
            LogLevel::Warn => tracing::warn!(?redacted_fields, "{}", message),
            LogLevel::Error => tracing::error!(?redacted_fields, "{}", message),
        }
    }

    /// Log request details
    pub async fn log_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        duration: Duration,
        request_id: Option<&str>,
    ) {
        if !self.config.enable_request_logging {
            return;
        }

        let mut fields = HashMap::new();
        fields.insert("method".to_string(), serde_json::Value::String(method.to_string()));
        fields.insert("path".to_string(), serde_json::Value::String(path.to_string()));
        fields.insert("status_code".to_string(), serde_json::Value::Number(serde_json::Number::from(status_code)));
        fields.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(duration.as_millis() as f64).unwrap()));
        
        if let Some(id) = request_id {
            fields.insert("request_id".to_string(), serde_json::Value::String(id.to_string()));
        }

        let level = if status_code >= 400 { LogLevel::Warn } else { LogLevel::Info };
        self.log_with_context(level, "HTTP Request", fields).await;
    }

    /// Log performance metrics
    pub async fn log_performance(
        &self,
        operation: &str,
        duration: Duration,
        metadata: HashMap<String, serde_json::Value>,
    ) {
        if !self.config.enable_performance_logging {
            return;
        }

        let mut fields = metadata.clone();
        fields.insert("operation".to_string(), serde_json::Value::String(operation.to_string()));
        fields.insert("duration_ms".to_string(), serde_json::Value::Number(serde_json::Value::Number::from_f64(duration.as_millis() as f64).unwrap()));

        self.log_with_context(LogLevel::Info, "Performance Metric", fields).await;
    }

    /// Create a correlation context and set it for the current thread
    pub async fn with_correlation_context<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let context = CorrelationContext::new();
        self.set_correlation_context(context.clone()).await;
        
        let result = f();
        
        self.clear_correlation_context().await;
        result
    }

    /// Create a correlation context with existing ID
    pub async fn with_existing_correlation_context<F, R>(&self, correlation_id: String, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let context = CorrelationContext::with_id(correlation_id);
        self.set_correlation_context(context.clone()).await;
        
        let result = f();
        
        self.clear_correlation_context().await;
        result
    }
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_trace {
    ($logger:expr, $msg:expr, $($key:expr => $val:expr),*) => {
        let mut fields = std::collections::HashMap::new();
        $(fields.insert($key.to_string(), serde_json::Value::String($val.to_string()));)*
        $logger.log_with_context(crate::observability::logging::LogLevel::Trace, $msg, fields).await;
    };
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $msg:expr, $($key:expr => $val:expr),*) => {
        let mut fields = std::collections::HashMap::new();
        $(fields.insert($key.to_string(), serde_json::Value::Value::String($val.to_string()));)*
        $logger.log_with_context(crate::observability::logging::LogLevel::Debug, $msg, fields).await;
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $msg:expr, $($key:expr => $val:expr),*) => {
        let mut fields = std::collections::HashMap::new();
        $(fields.insert($key.to_string(), serde_json::Value::String($val.to_string()));)*
        $logger.log_with_context(crate::observability::logging::LogLevel::Info, $msg, fields).await;
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $msg:expr, $($key:expr => $val:expr),*) => {
        let mut fields = std::collections::HashMap::new();
        $(fields.insert($key.to_string(), serde_json::Value::String($val.to_string()));)*
        $logger.log_with_context(crate::observability::logging::LogLevel::Warn, $msg, fields).await;
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $msg:expr, $($key:expr => $val:expr),*) => {
        let mut fields = std::collections::HashMap::new();
        $(fields.insert($key.to_string(), serde_json::Value::Value::String($val.to_string()));)*
        $logger.log_with_context(crate::observability::logging::LogLevel::Error, $msg, fields).await;
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, LogLevel::Info);
        assert_eq!(config.format, LogFormat::Json);
        assert!(config.enable_correlation_ids);
        assert!(config.structured_logging);
    }

    #[tokio::test]
    async fn test_correlation_context() {
        let context = CorrelationContext::new();
        assert!(!context.correlation_id.is_empty());
        assert!(context.duration().as_millis() < 100);
    }

    #[tokio::test]
    async fn test_logger_creation() {
        let config = LoggingConfig::default();
        let logger = PolymeraLogger::new(config);
        assert!(logger.is_ok());
    }

    #[tokio::test]
    async fn test_correlation_context_management() {
        let config = LoggingConfig::default();
        let logger = PolymeraLogger::new(config).unwrap();
        
        let context = CorrelationContext::new();
        logger.set_correlation_context(context.clone()).await;
        
        let retrieved = logger.get_correlation_context().await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().correlation_id, context.correlation_id);
        
        logger.clear_correlation_context().await;
        let cleared = logger.get_correlation_context().await;
        assert!(cleared.is_none());
    }
}
