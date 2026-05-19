//! Error types for the DAO service

use thiserror::Error;

/// DAO service errors
#[derive(Error, Debug)]
pub enum DAOError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Proposal error: {0}")]
    ProposalError(String),

    #[error("Vote error: {0}")]
    VoteError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Governance error: {0}")]
    GovernanceError(String),

    #[error("Chain error: {0}")]
    ChainError(#[from] chain::ChainError),

    #[error("DID resolution error: {0}")]
    DIDError(String),

    #[error("Capability error: {0}")]
    CapabilityError(String),

    #[error("Signature verification error: {0}")]
    SignatureError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("UUID parse error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Timestamp parse error: {0}")]
    TimestampParseError(#[from] chrono::ParseError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Proposal not found: {0}")]
    ProposalNotFound(String),

    #[error("Vote not found: {0}")]
    VoteNotFound(String),

    #[error("Voting period ended: {0}")]
    VotingPeriodEnded(String),

    #[error("Voting period not started: {0}")]
    VotingPeriodNotStarted(String),

    #[error("Already voted: {0}")]
    AlreadyVoted(String),

    #[error("Insufficient votes: {0}")]
    InsufficientVotes(String),

    #[error("Proposal already executed: {0}")]
    ProposalAlreadyExecuted(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Crypto error: {0}")]
    CryptoError(String),
}
