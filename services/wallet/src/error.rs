//! Error types for the wallet service

use thiserror::Error;

/// Wallet service error types
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Capability denied")]
    CapabilityDenied,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid key type: {0}")]
    InvalidKeyType(String),
    
    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("DID resolution failed: {0}")]
    DIDResolutionFailed(String),
    
    #[error("DID creation failed: {0}")]
    DIDCreationFailed(String),
    
    #[error("PDV operation failed: {0}")]
    PDVOperationFailed(String),
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("PQC operation failed: {0}")]
    PQCOperationFailed(String),
    
    #[error("HSM operation failed: {0}")]
    HSMOperationFailed(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<serde_json::Error> for WalletError {
    fn from(err: serde_json::Error) -> Self {
        WalletError::SerializationError(err.to_string())
    }
}

impl From<uuid::Error> for WalletError {
    fn from(err: uuid::Error) -> Self {
        WalletError::InternalError(format!("UUID error: {}", err))
    }
}

impl From<chrono::ParseError> for WalletError {
    fn from(err: chrono::ParseError) -> Self {
        WalletError::InternalError(format!("Chrono error: {}", err))
    }
}
