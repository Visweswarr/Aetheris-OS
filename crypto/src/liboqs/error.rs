use std::fmt;
use std::error::Error;

/// Result type for liboqs operations
pub type LibOqsResult<T> = Result<T, LibOqsError>;

/// Error types for liboqs operations
#[derive(Debug, Clone)]
pub enum LibOqsError {
    /// General error with message
    General(String),
    /// Invalid parameter error
    InvalidParameter(String),
    /// Memory allocation error
    Memory,
    /// Cryptographic operation failed
    Crypto(String),
    /// Verification failed
    Verification,
    /// Unknown error
    Unknown(i32),
}

impl LibOqsError {
    /// Create a new general error
    pub fn general<S: Into<String>>(message: S) -> Self {
        LibOqsError::General(message.into())
    }

    /// Create a new invalid parameter error
    pub fn invalid_parameter<S: Into<String>>(message: S) -> Self {
        LibOqsError::InvalidParameter(message.into())
    }

    /// Create a new crypto error
    pub fn crypto<S: Into<String>>(message: S) -> Self {
        LibOqsError::Crypto(message.into())
    }

    /// Check if the error is a verification failure
    pub fn is_verification_failure(&self) -> bool {
        matches!(self, LibOqsError::Verification)
    }

    /// Check if the error is a memory allocation failure
    pub fn is_memory_error(&self) -> bool {
        matches!(self, LibOqsError::Memory)
    }

    /// Check if the error is a parameter validation failure
    pub fn is_parameter_error(&self) -> bool {
        matches!(self, LibOqsError::InvalidParameter(_))
    }
}

impl fmt::Display for LibOqsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibOqsError::General(msg) => write!(f, "General error: {}", msg),
            LibOqsError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            LibOqsError::Memory => write!(f, "Memory allocation failed"),
            LibOqsError::Crypto(msg) => write!(f, "Cryptographic operation failed: {}", msg),
            LibOqsError::Verification => write!(f, "Verification failed"),
            LibOqsError::Unknown(code) => write!(f, "Unknown error: {}", code),
        }
    }
}

impl Error for LibOqsError {}

impl From<i32> for LibOqsError {
    fn from(code: i32) -> Self {
        match code {
            0 => LibOqsError::General("Success".to_string()),
            -1 => LibOqsError::General("General error".to_string()),
            -2 => LibOqsError::InvalidParameter("Invalid parameter".to_string()),
            -3 => LibOqsError::Memory,
            -4 => LibOqsError::Crypto("Cryptographic operation failed".to_string()),
            -5 => LibOqsError::Verification,
            _ => LibOqsError::Unknown(code),
        }
    }
}

impl From<LibOqsError> for i32 {
    fn from(error: LibOqsError) -> Self {
        match error {
            LibOqsError::General(_) => -1,
            LibOqsError::InvalidParameter(_) => -2,
            LibOqsError::Memory => -3,
            LibOqsError::Crypto(_) => -4,
            LibOqsError::Verification => -5,
            LibOqsError::Unknown(code) => code,
        }
    }
}

impl From<std::io::Error> for LibOqsError {
    fn from(err: std::io::Error) -> Self {
        LibOqsError::General(format!("I/O error: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for LibOqsError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        LibOqsError::General(format!("UTF-8 error: {}", err))
    }
}

impl From<std::num::TryFromIntError> for LibOqsError {
    fn from(err: std::num::TryFromIntError) -> Self {
        LibOqsError::InvalidParameter(format!("Integer conversion error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let general = LibOqsError::general("test message");
        assert!(matches!(general, LibOqsError::General(_)));

        let param = LibOqsError::invalid_parameter("invalid param");
        assert!(matches!(param, LibOqsError::InvalidParameter(_)));

        let crypto = LibOqsError::crypto("crypto failed");
        assert!(matches!(crypto, LibOqsError::Crypto(_)));
    }

    #[test]
    fn test_error_display() {
        let error = LibOqsError::Memory;
        assert_eq!(error.to_string(), "Memory allocation failed");

        let error = LibOqsError::Verification;
        assert_eq!(error.to_string(), "Verification failed");

        let error = LibOqsError::General("test");
        assert_eq!(error.to_string(), "General error: test");
    }

    #[test]
    fn test_error_conversion() {
        let error = LibOqsError::from(-3);
        assert!(matches!(error, LibOqsError::Memory));

        let error = LibOqsError::from(-5);
        assert!(matches!(error, LibOqsError::Verification));

        let code: i32 = LibOqsError::Memory.into();
        assert_eq!(code, -3);
    }

    #[test]
    fn test_error_checks() {
        let error = LibOqsError::Verification;
        assert!(error.is_verification_failure());

        let error = LibOqsError::Memory;
        assert!(error.is_memory_error());

        let error = LibOqsError::InvalidParameter("test".to_string());
        assert!(error.is_parameter_error());
    }
}
