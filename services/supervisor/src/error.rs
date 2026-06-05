//! Supervisor error types.

use thiserror::Error;

/// Errors emitted by supervisor operations.
#[derive(Debug, Error)]
pub enum SupError {
    /// A child failed to start at all.
    #[error("failed to start child '{name}': {reason}")]
    StartFailed {
        /// Child name.
        name: String,
        /// Reason from the spawner.
        reason: String,
    },

    /// The configured restart-rate limit was exceeded for this supervisor.
    #[error("restart rate limit exceeded: {restarts} restarts within {seconds}s")]
    RateLimitExceeded {
        /// Number of restarts observed in the window.
        restarts: u32,
        /// Window size, seconds.
        seconds: u64,
    },

    /// Supervisor config file failed to parse.
    #[error("config parse error: {0}")]
    Config(String),
}

/// Convenience alias.
pub type SupResult<T> = Result<T, SupError>;
