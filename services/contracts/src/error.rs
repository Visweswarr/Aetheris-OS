//! Error types for the contracts service

use thiserror::Error;

/// Contract service errors
#[derive(Error, Debug)]
pub enum ContractError {
    #[error("WASM execution error: {0}")]
    WasmError(String),

    #[error("Gas limit exceeded: {0}")]
    GasExceeded(String),

    #[error("Memory limit exceeded: {0}")]
    MemoryExceeded(String),

    #[error("Execution timeout: {0}")]
    Timeout(String),

    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("Resource allocation error: {0}")]
    ResourceAllocationError(String),

    #[error("DAO service error: {0}")]
    DAOError(#[from] dao::DAOError),

    #[error("Chain service error: {0}")]
    ChainError(#[from] chain::ChainError),

    #[error("Capability error: {0}")]
    CapabilityError(String),

    #[error("ZK proof error: {0}")]
    ZKProofError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),
}
