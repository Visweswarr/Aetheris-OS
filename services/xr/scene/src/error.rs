//! Error types for the XR scene service

use thiserror::Error;

/// Scene service errors
#[derive(Error, Debug)]
pub enum SceneError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Avatar not found: {0}")]
    AvatarNotFound(String),

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("Policy denied: {0}")]
    PolicyDenied(String),

    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    #[error("Scene graph full: maximum nodes reached")]
    SceneGraphFull,

    #[error("Avatar limit reached: maximum avatars reached")]
    AvatarLimitReached,

    #[error("Invalid transform: {0}")]
    InvalidTransform(String),

    #[error("Invalid component: {0}")]
    InvalidComponent(String),

    #[error("Invalid DID: {0}")]
    InvalidDID(String),

    #[error("Invalid snapshot format: {0}")]
    InvalidSnapshotFormat(String),

    #[error("Service already running")]
    ServiceAlreadyRunning,

    #[error("Service not running")]
    ServiceNotRunning,

    #[error("NGFS error: {0}")]
    NGFSError(#[from] ngfs::NGFSError),

    #[error("DAO error: {0}")]
    DAOError(#[from] polymera_dao::DAOError),

    #[error("Capability error: {0}")]
    CapabilityError(#[from] cap_tokens::CapabilityError),

    #[error("DID error: {0}")]
    DIDError(#[from] polymera_did::DIDError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("CBOR error: {0}")]
    CBORError(#[from] serde_cbor::Error),

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

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for scene operations
pub type SceneResult<T> = Result<T, SceneError>;
