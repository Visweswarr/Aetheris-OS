//! Error types for the chain service

use thiserror::Error;

/// Chain service errors
#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Web3 connection error: {0}")]
    Web3Error(#[from] web3::Error),

    #[error("Ethereum transaction error: {0}")]
    EthereumError(String),

    #[error("Contract interaction error: {0}")]
    ContractError(String),

    #[error("DID resolution error: {0}")]
    DIDError(String),

    #[error("Signature verification error: {0}")]
    SignatureError(String),

    #[error("Batch submission error: {0}")]
    BatchError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    #[error("Transaction timeout: {0}")]
    TransactionTimeout(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),
}

impl From<web3::contract::Error> for ChainError {
    fn from(err: web3::contract::Error) -> Self {
        ChainError::ContractError(err.to_string())
    }
}

impl From<web3::ethabi::Error> for ChainError {
    fn from(err: web3::ethabi::Error) -> Self {
        ChainError::ContractError(err.to_string())
    }
}
