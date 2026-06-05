//! QUIC/HTTP3 Service for Aetheris OS
//! 
//! This crate provides QUIC transport and HTTP/3 support for the POSIX networking
//! subsystem with capability-based access control and policy enforcement.

pub mod quicd;
pub mod bindings;
pub mod stream;
pub mod connection;
pub mod server;
pub mod client;

// Re-export main types
pub use quicd::{QuicDaemon, QuicConfig, QuicRequest, QuicResponse};
pub use stream::{QuicStream, StreamState, StreamType};
pub use connection::{QuicConnection, ConnectionState, ConnectionInfo};
pub use server::{QuicServer, ServerConfig};
pub use client::{QuicClient, ClientConfig};

/// Version of the QUIC service
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocol version for QUIC
pub const PROTOCOL_VERSION: &str = "/quic/1.0.0";

/// Default QUIC port
pub const DEFAULT_QUIC_PORT: u16 = 443;

/// Default stream buffer size
pub const DEFAULT_STREAM_BUFFER_SIZE: usize = 65536;

/// Maximum number of streams per connection
pub const MAX_STREAMS_PER_CONNECTION: usize = 1000;

/// Default connection timeout
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 30;

/// Default stream timeout
pub const DEFAULT_STREAM_TIMEOUT: u64 = 60;

/// QUIC error types
#[derive(Debug, thiserror::Error)]
pub enum QuicError {
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Stream error: {0}")]
    Stream(String),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("TLS error: {0}")]
    Tls(String),
    
    #[error("Policy error: {0}")]
    Policy(String),
    
    #[error("Capability error: {0}")]
    Capability(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for QUIC operations
pub type QuicResult<T> = Result<T, QuicError>;
