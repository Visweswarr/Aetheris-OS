//! AI service error types and handling

use std::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// AI service error types
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum AiError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Model loading error
    #[error("Model loading error: {0}")]
    ModelLoading(String),
    
    /// Inference error
    #[error("Inference error: {0}")]
    Inference(String),
    
    /// Backend error
    #[error("Backend error: {0}")]
    Backend(String),
    
    /// Device error
    #[error("Device error: {0}")]
    Device(String),
    
    /// Capability error
    #[error("Capability error: {0}")]
    Capability(String),
    
    /// Policy error
    #[error("Policy error: {0}")]
    Policy(String),
    
    /// Replay error
    #[error("Replay error: {0}")]
    Replay(String),
    
    /// Encoding error
    #[error("Encoding error: {0}")]
    Encoding(String),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    /// Timeout error
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    /// Resource error
    #[error("Resource error: {0}")]
    Resource(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AiError {
    /// Create a configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }
    
    /// Create a model loading error
    pub fn model_loading(msg: impl Into<String>) -> Self {
        Self::ModelLoading(msg.into())
    }
    
    /// Create an inference error
    pub fn inference(msg: impl Into<String>) -> Self {
        Self::Inference(msg.into())
    }
    
    /// Create a backend error
    pub fn backend(msg: impl Into<String>) -> Self {
        Self::Backend(msg.into())
    }
    
    /// Create a device error
    pub fn device(msg: impl Into<String>) -> Self {
        Self::Device(msg.into())
    }
    
    /// Create a capability error
    pub fn capability(msg: impl Into<String>) -> Self {
        Self::Capability(msg.into())
    }
    
    /// Create a policy error
    pub fn policy(msg: impl Into<String>) -> Self {
        Self::Policy(msg.into())
    }
    
    /// Create a replay error
    pub fn replay(msg: impl Into<String>) -> Self {
        Self::Replay(msg.into())
    }
    
    /// Create an encoding error
    pub fn encoding(msg: impl Into<String>) -> Self {
        Self::Encoding(msg.into())
    }
    
    /// Create an IO error
    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }
    
    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }
    
    /// Create a deserialization error
    pub fn deserialization(msg: impl Into<String>) -> Self {
        Self::Deserialization(msg.into())
    }
    
    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }
    
    /// Create a resource error
    pub fn resource(msg: impl Into<String>) -> Self {
        Self::Resource(msg.into())
    }
    
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            AiError::Timeout(_) | 
            AiError::Resource(_) | 
            AiError::Io(_)
        )
    }
    
    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            AiError::Configuration(_) => ErrorCategory::Configuration,
            AiError::ModelLoading(_) => ErrorCategory::Model,
            AiError::Inference(_) => ErrorCategory::Inference,
            AiError::Backend(_) => ErrorCategory::Backend,
            AiError::Device(_) => ErrorCategory::Device,
            AiError::Capability(_) => ErrorCategory::Security,
            AiError::Policy(_) => ErrorCategory::Security,
            AiError::Replay(_) => ErrorCategory::Replay,
            AiError::Encoding(_) => ErrorCategory::Encoding,
            AiError::Io(_) => ErrorCategory::Io,
            AiError::Serialization(_) => ErrorCategory::Serialization,
            AiError::Deserialization(_) => ErrorCategory::Serialization,
            AiError::Timeout(_) => ErrorCategory::Timeout,
            AiError::Resource(_) => ErrorCategory::Resource,
            AiError::Validation(_) => ErrorCategory::Validation,
            AiError::Internal(_) => ErrorCategory::Internal,
        }
    }
}

/// Error categories for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    Configuration,
    Model,
    Inference,
    Backend,
    Device,
    Security,
    Replay,
    Encoding,
    Io,
    Serialization,
    Timeout,
    Resource,
    Validation,
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::Model => write!(f, "Model"),
            ErrorCategory::Inference => write!(f, "Inference"),
            ErrorCategory::Backend => write!(f, "Backend"),
            ErrorCategory::Device => write!(f, "Device"),
            ErrorCategory::Security => write!(f, "Security"),
            ErrorCategory::Replay => write!(f, "Replay"),
            ErrorCategory::Encoding => write!(f, "Encoding"),
            ErrorCategory::Io => write!(f, "IO"),
            ErrorCategory::Serialization => write!(f, "Serialization"),
            ErrorCategory::Timeout => write!(f, "Timeout"),
            ErrorCategory::Resource => write!(f, "Resource"),
            ErrorCategory::Validation => write!(f, "Validation"),
            ErrorCategory::Internal => write!(f, "Internal"),
        }
    }
}

/// Result type for AI operations
pub type AiResult<T> = Result<T, AiError>;

/// Convert from std::io::Error
impl From<std::io::Error> for AiError {
    fn from(err: std::io::Error) -> Self {
        AiError::io(err.to_string())
    }
}

/// Convert from serde_cbor::Error
impl From<serde_cbor::Error> for AiError {
    fn from(err: serde_cbor::Error) -> Self {
        AiError::serialization(err.to_string())
    }
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for AiError {
    fn from(err: serde_json::Error) -> Self {
        AiError::serialization(err.to_string())
    }
}

/// Convert from tokio::time::error::Elapsed
impl From<tokio::time::error::Elapsed> for AiError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AiError::timeout(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AiError::configuration("test config error");
        assert!(matches!(err, AiError::Configuration(_)));
        assert_eq!(err.category(), ErrorCategory::Configuration);
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_retryable_errors() {
        let timeout_err = AiError::timeout("test timeout");
        assert!(timeout_err.is_retryable());
        
        let io_err = AiError::io("test io error");
        assert!(io_err.is_retryable());
        
        let resource_err = AiError::resource("test resource error");
        assert!(resource_err.is_retryable());
    }

    #[test]
    fn test_error_categories() {
        let config_err = AiError::configuration("test");
        assert_eq!(config_err.category(), ErrorCategory::Configuration);
        
        let security_err = AiError::capability("test");
        assert_eq!(security_err.category(), ErrorCategory::Security);
        
        let model_err = AiError::model_loading("test");
        assert_eq!(model_err.category(), ErrorCategory::Model);
    }

    #[test]
    fn test_error_conversions() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let ai_err: AiError = io_err.into();
        assert!(matches!(ai_err, AiError::Io(_)));
    }
}
