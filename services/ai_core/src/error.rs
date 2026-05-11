//! Error handling module for AI Core Service

use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};
use std::sync::OnceLock;
use std::fmt;
use serde::{Deserialize, Serialize};

/// Result type alias for AI Core Service
pub type Result<T> = std::result::Result<T, AiCoreError>;

/// AI Core Service error types
#[derive(Debug, Clone)]
pub enum AiCoreError {
    ConfigError(String), Config(String), ModelError(String), ToolError(String),
    ToolNotFound(String), ToolNotInitialized(String), CapabilityError(String),
    CapDenied(String), CapabilityDenied(String), ProtocolError(String),
    SerializationError(String), IoError(String), NetworkError(String),
    TimeoutError(String), AuthError(String), AuthorizationError(String),
    RateLimitError(String), ResourceError(String), InternalError(String),
    ValidationError(String), NotFoundError(String), NotFound(String),
    AlreadyExistsError(String), InvalidArgumentError(String), InvalidInput(String),
    UnsupportedOperationError(String), NotSupported(String), NotImplemented(String),
    ServiceUnavailableError(String), ExternalServiceError(String), ToolTimeout(String),
    ModelTimeout(String), MemoryAllocationError(String), CircuitBreakerOpen(String),
    QuotaExceeded(String), DependencyUnavailable(String), ConfigValidationError(String),
    FeatureNotAvailable(String), DataCorruptionError(String), ConcurrentAccessError(String),
}

impl fmt::Display for AiCoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigError(s) | Self::Config(s) => write!(f, "Configuration error: {}", s),
            Self::ModelError(s) => write!(f, "Model error: {}", s),
            Self::ToolError(s) => write!(f, "Tool error: {}", s),
            Self::ToolNotFound(s) => write!(f, "Tool not found: {}", s),
            Self::ToolNotInitialized(s) => write!(f, "Tool not initialized: {}", s),
            Self::CapabilityError(s) => write!(f, "Capability error: {}", s),
            Self::CapDenied(s) | Self::CapabilityDenied(s) => write!(f, "Capability denied: {}", s),
            Self::ProtocolError(s) => write!(f, "Protocol error: {}", s),
            Self::SerializationError(s) => write!(f, "Serialization error: {}", s),
            Self::IoError(s) => write!(f, "I/O error: {}", s),
            Self::NetworkError(s) => write!(f, "Network error: {}", s),
            Self::TimeoutError(s) => write!(f, "Timeout error: {}", s),
            Self::AuthError(s) => write!(f, "Authentication error: {}", s),
            Self::AuthorizationError(s) => write!(f, "Authorization error: {}", s),
            Self::RateLimitError(s) => write!(f, "Rate limit error: {}", s),
            Self::ResourceError(s) => write!(f, "Resource error: {}", s),
            Self::InternalError(s) => write!(f, "Internal error: {}", s),
            Self::ValidationError(s) => write!(f, "Validation error: {}", s),
            Self::NotFoundError(s) | Self::NotFound(s) => write!(f, "Not found: {}", s),
            Self::AlreadyExistsError(s) => write!(f, "Already exists: {}", s),
            Self::InvalidArgumentError(s) | Self::InvalidInput(s) => write!(f, "Invalid argument: {}", s),
            Self::UnsupportedOperationError(s) | Self::NotSupported(s) | Self::NotImplemented(s) => write!(f, "Not supported: {}", s),
            Self::ServiceUnavailableError(s) => write!(f, "Service unavailable: {}", s),
            Self::ExternalServiceError(s) => write!(f, "External service error: {}", s),
            Self::ToolTimeout(s) => write!(f, "Tool timeout: {}", s),
            Self::ModelTimeout(s) => write!(f, "Model timeout: {}", s),
            Self::MemoryAllocationError(s) => write!(f, "Memory allocation error: {}", s),
            Self::CircuitBreakerOpen(s) => write!(f, "Circuit breaker open: {}", s),
            Self::QuotaExceeded(s) => write!(f, "Quota exceeded: {}", s),
            Self::DependencyUnavailable(s) => write!(f, "Dependency unavailable: {}", s),
            Self::ConfigValidationError(s) => write!(f, "Configuration validation error: {}", s),
            Self::FeatureNotAvailable(s) => write!(f, "Feature not available: {}", s),
            Self::DataCorruptionError(s) => write!(f, "Data corruption error: {}", s),
            Self::ConcurrentAccessError(s) => write!(f, "Concurrent access error: {}", s),
        }
    }
}

impl std::error::Error for AiCoreError {}

impl AiCoreError {
    pub fn config(msg: &str) -> Self { Self::ConfigError(msg.to_string()) }
    pub fn model(msg: &str) -> Self { Self::ModelError(msg.to_string()) }
    pub fn tool(msg: &str) -> Self { Self::ToolError(msg.to_string()) }
    pub fn capability(msg: &str) -> Self { Self::CapabilityError(msg.to_string()) }
    pub fn cap_denied(msg: &str) -> Self { Self::CapDenied(msg.to_string()) }
    pub fn protocol(msg: &str) -> Self { Self::ProtocolError(msg.to_string()) }
    pub fn serialization(msg: &str) -> Self { Self::SerializationError(msg.to_string()) }
    pub fn io(msg: &str) -> Self { Self::IoError(msg.to_string()) }
    pub fn network(msg: &str) -> Self { Self::NetworkError(msg.to_string()) }
    pub fn timeout(msg: &str) -> Self { Self::TimeoutError(msg.to_string()) }
    pub fn auth(msg: &str) -> Self { Self::AuthError(msg.to_string()) }
    pub fn authorization(msg: &str) -> Self { Self::AuthorizationError(msg.to_string()) }
    pub fn rate_limit(msg: &str) -> Self { Self::RateLimitError(msg.to_string()) }
    pub fn resource(msg: &str) -> Self { Self::ResourceError(msg.to_string()) }
    pub fn internal(msg: &str) -> Self { Self::InternalError(msg.to_string()) }
    pub fn validation(msg: &str) -> Self { Self::ValidationError(msg.to_string()) }
    pub fn not_found(msg: &str) -> Self { Self::NotFoundError(msg.to_string()) }
    pub fn already_exists(msg: &str) -> Self { Self::AlreadyExistsError(msg.to_string()) }
    pub fn invalid_argument(msg: &str) -> Self { Self::InvalidArgumentError(msg.to_string()) }
    pub fn unsupported_operation(msg: &str) -> Self { Self::UnsupportedOperationError(msg.to_string()) }
    pub fn service_unavailable(msg: &str) -> Self { Self::ServiceUnavailableError(msg.to_string()) }
    pub fn external_service(msg: &str) -> Self { Self::ExternalServiceError(msg.to_string()) }
    pub fn tool_timeout(msg: &str) -> Self { Self::ToolTimeout(msg.to_string()) }
    pub fn model_timeout(msg: &str) -> Self { Self::ModelTimeout(msg.to_string()) }
    pub fn memory_allocation(msg: &str) -> Self { Self::MemoryAllocationError(msg.to_string()) }
    pub fn circuit_breaker_open(msg: &str) -> Self { Self::CircuitBreakerOpen(msg.to_string()) }
    pub fn quota_exceeded(msg: &str) -> Self { Self::QuotaExceeded(msg.to_string()) }
    pub fn dependency_unavailable(msg: &str) -> Self { Self::DependencyUnavailable(msg.to_string()) }
    pub fn config_validation(msg: &str) -> Self { Self::ConfigValidationError(msg.to_string()) }
    pub fn feature_not_available(msg: &str) -> Self { Self::FeatureNotAvailable(msg.to_string()) }
    pub fn data_corruption(msg: &str) -> Self { Self::DataCorruptionError(msg.to_string()) }
    pub fn concurrent_access(msg: &str) -> Self { Self::ConcurrentAccessError(msg.to_string()) }


    pub fn error_code(&self) -> u32 {
        match self {
            Self::ConfigError(_) | Self::Config(_) => 1001,
            Self::ModelError(_) => 1002,
            Self::ToolError(_) | Self::ToolNotFound(_) | Self::ToolNotInitialized(_) => 1003,
            Self::CapabilityError(_) => 1004,
            Self::CapDenied(_) | Self::CapabilityDenied(_) => 1005,
            Self::ProtocolError(_) => 1006,
            Self::SerializationError(_) => 1007,
            Self::IoError(_) => 1008,
            Self::NetworkError(_) => 1009,
            Self::TimeoutError(_) => 1010,
            Self::AuthError(_) => 1011,
            Self::AuthorizationError(_) => 1012,
            Self::RateLimitError(_) => 1013,
            Self::ResourceError(_) => 1014,
            Self::InternalError(_) => 1015,
            Self::ValidationError(_) => 1016,
            Self::NotFoundError(_) | Self::NotFound(_) => 1017,
            Self::AlreadyExistsError(_) => 1018,
            Self::InvalidArgumentError(_) | Self::InvalidInput(_) => 1019,
            Self::UnsupportedOperationError(_) | Self::NotSupported(_) | Self::NotImplemented(_) => 1020,
            Self::ServiceUnavailableError(_) => 1021,
            Self::ExternalServiceError(_) => 1022,
            Self::ToolTimeout(_) => 1023,
            Self::ModelTimeout(_) => 1024,
            Self::MemoryAllocationError(_) => 1025,
            Self::CircuitBreakerOpen(_) => 1026,
            Self::QuotaExceeded(_) => 1027,
            Self::DependencyUnavailable(_) => 1028,
            Self::ConfigValidationError(_) => 1029,
            Self::FeatureNotAvailable(_) => 1030,
            Self::DataCorruptionError(_) => 1031,
            Self::ConcurrentAccessError(_) => 1032,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self,
            Self::NetworkError(_) | Self::TimeoutError(_) | Self::ServiceUnavailableError(_) |
            Self::ExternalServiceError(_) | Self::ToolTimeout(_) | Self::ModelTimeout(_) |
            Self::CircuitBreakerOpen(_) | Self::DependencyUnavailable(_) | Self::ConcurrentAccessError(_)
        )
    }
}

impl From<io::Error> for AiCoreError {
    fn from(err: io::Error) -> Self { Self::IoError(err.to_string()) }
}

impl From<serde_json::Error> for AiCoreError {
    fn from(err: serde_json::Error) -> Self { Self::SerializationError(err.to_string()) }
}

impl From<serde_cbor::Error> for AiCoreError {
    fn from(err: serde_cbor::Error) -> Self { Self::SerializationError(err.to_string()) }
}

impl From<prost::DecodeError> for AiCoreError {
    fn from(err: prost::DecodeError) -> Self { Self::ProtocolError(err.to_string()) }
}

impl From<prost::EncodeError> for AiCoreError {
    fn from(err: prost::EncodeError) -> Self { Self::ProtocolError(err.to_string()) }
}

impl From<tokio::time::error::Elapsed> for AiCoreError {
    fn from(err: tokio::time::error::Elapsed) -> Self { Self::TimeoutError(err.to_string()) }
}

impl From<Box<dyn std::error::Error>> for AiCoreError {
    fn from(err: Box<dyn std::error::Error>) -> Self { Self::InternalError(err.to_string()) }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AiCoreError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self { Self::InternalError(err.to_string()) }
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32, pub initial_delay_ms: u64, pub max_delay_ms: u64,
    pub backoff_multiplier: f64, pub jitter_factor: f64, pub seed: Option<u64>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_attempts: 3, initial_delay_ms: 100, max_delay_ms: 5000, backoff_multiplier: 2.0, jitter_factor: 0.1, seed: None }
    }
}

/// Retry attempt information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryAttempt {
    pub attempt: u32, pub delay_ms: u64, pub timestamp: u64, pub error: String, pub error_code: u32, pub is_final: bool,
}

/// Retry context for tracking retry state
#[derive(Debug, Clone)]
pub struct RetryContext {
    policy: RetryPolicy, attempt: u32, start_time: Instant, last_error: Option<AiCoreError>, attempts: Vec<RetryAttempt>,
}

impl RetryContext {
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy, attempt: 0, start_time: Instant::now(), last_error: None, attempts: Vec::new() }
    }
    pub fn should_retry(&self, error: &AiCoreError) -> bool { self.attempt < self.policy.max_attempts && error.is_retryable() }
    pub fn next_delay(&self) -> Duration {
        if self.attempt == 0 { return Duration::from_millis(0); }
        let base_delay = self.policy.initial_delay_ms as f64 * self.policy.backoff_multiplier.powi((self.attempt - 1) as i32);
        Duration::from_millis(base_delay.min(self.policy.max_delay_ms as f64) as u64)
    }
    pub fn record_attempt(&mut self, error: AiCoreError) {
        self.attempt += 1;
        self.last_error = Some(error.clone());
        let delay = if self.attempt == 1 { 0 } else { self.next_delay().as_millis() as u64 };
        self.attempts.push(RetryAttempt {
            attempt: self.attempt, delay_ms: delay,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            error: error.to_string(), error_code: error.error_code(), is_final: self.attempt >= self.policy.max_attempts,
        });
    }
    pub fn total_duration(&self) -> Duration { self.start_time.elapsed() }
    pub fn attempts(&self) -> &[RetryAttempt] { &self.attempts }
    pub fn last_error(&self) -> Option<&AiCoreError> { self.last_error.as_ref() }
    pub fn is_exhausted(&self) -> bool { self.attempt >= self.policy.max_attempts }
}


/// Error audit logger
pub struct ErrorAuditLogger {
    entries: std::sync::Arc<std::sync::Mutex<Vec<ErrorAuditEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAuditEntry {
    pub error_code: u32, pub error_message: String, pub error_type: String, pub timestamp: u64,
    pub session_id: Option<String>, pub request_id: Option<String>,
    pub retry_attempts: Vec<RetryAttempt>, pub context: HashMap<String, String>, pub stack_trace: Option<String>,
}

impl ErrorAuditLogger {
    pub fn new() -> Self { Self { entries: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())) } }
    pub fn log_error(&self, error: &AiCoreError, context: Option<RetryContext>) {
        let entry = ErrorAuditEntry {
            error_code: error.error_code(), error_message: error.to_string(), error_type: format!("{:?}", error),
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            session_id: None, request_id: None,
            retry_attempts: context.map(|c| c.attempts().to_vec()).unwrap_or_default(),
            context: HashMap::new(), stack_trace: None,
        };
        if let Ok(mut entries) = self.entries.lock() { entries.push(entry); }
    }
    pub fn export_cbor(&self) -> std::result::Result<Vec<u8>, serde_cbor::Error> {
        let entries = self.entries.lock().unwrap();
        serde_cbor::to_vec(&*entries)
    }
    pub fn clear(&self) { if let Ok(mut entries) = self.entries.lock() { entries.clear(); } }
}

impl Default for ErrorAuditLogger { fn default() -> Self { Self::new() } }

static GLOBAL_ERROR_AUDIT_LOGGER: OnceLock<ErrorAuditLogger> = OnceLock::new();

pub fn init_global_error_audit_logger() { let _ = GLOBAL_ERROR_AUDIT_LOGGER.set(ErrorAuditLogger::new()); }
pub fn get_global_error_audit_logger() -> Option<&'static ErrorAuditLogger> { GLOBAL_ERROR_AUDIT_LOGGER.get() }
pub fn log_error_audit(error: &AiCoreError, context: Option<RetryContext>) {
    if let Some(logger) = get_global_error_audit_logger() { logger.log_error(error, context); }
}

pub async fn retry_with_backoff<F, T, E>(policy: RetryPolicy, mut operation: F) -> Result<T>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, E>> + Send>>,
    E: Into<AiCoreError>,
{
    let mut context = RetryContext::new(policy);
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                let ai_error: AiCoreError = error.into();
                log_error_audit(&ai_error, Some(context.clone()));
                if !context.should_retry(&ai_error) { return Err(ai_error); }
                context.record_attempt(ai_error.clone());
                if context.is_exhausted() { return Err(ai_error); }
                tokio::time::sleep(context.next_delay()).await;
            }
        }
    }
}

/// Error hint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHint { pub code: String, pub title: String, pub message: String, pub hint: String, pub severity: String, pub retryable: bool, pub user_action: String }

/// Error hints manager
pub struct ErrorHintsManager { hints: HashMap<u32, ErrorHint> }

impl ErrorHintsManager {
    pub fn new() -> Self { Self { hints: HashMap::new() } }
    pub fn get_hint(&self, error_code: u32) -> Option<&ErrorHint> { self.hints.get(&error_code) }
}

impl Default for ErrorHintsManager { fn default() -> Self { Self::new() } }

static GLOBAL_ERROR_HINTS_MANAGER: OnceLock<ErrorHintsManager> = OnceLock::new();

pub fn init_global_error_hints_manager<P: AsRef<std::path::Path>>(_path: P) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = GLOBAL_ERROR_HINTS_MANAGER.set(ErrorHintsManager::new());
    Ok(())
}
pub fn get_global_error_hints_manager() -> Option<&'static ErrorHintsManager> { GLOBAL_ERROR_HINTS_MANAGER.get() }
pub fn get_error_hint(error: &AiCoreError) -> Option<&'static ErrorHint> { get_global_error_hints_manager().and_then(|m| m.get_hint(error.error_code())) }
pub fn get_retry_policy_for_error(_error: &AiCoreError) -> RetryPolicy { RetryPolicy::default() }
