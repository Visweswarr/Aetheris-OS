//! QUIC connection implementation
//! 
//! This module provides QUIC connection functionality for the QUIC service.

use std::net::SocketAddr;
use std::time::Instant;
use serde::{Deserialize, Serialize};

/// QUIC connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConnection {
    /// Connection ID
    pub id: String,
    /// Remote address
    pub address: SocketAddr,
    /// SNI hostname
    pub sni: Option<String>,
    /// Connection state
    pub state: ConnectionState,
    /// Process capability
    pub process_cap: String,
    /// Network namespace
    pub namespace: String,
    /// Creation timestamp
    pub created_at: Instant,
    /// Last activity timestamp
    pub last_activity: Instant,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Stream IDs
    pub streams: Vec<String>,
    /// TLS session
    pub tls_session: Option<String>,
}

/// Connection state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    /// Connection created
    Created,
    /// Connection connecting
    Connecting,
    /// Connection established
    Established,
    /// Connection closing
    Closing,
    /// Connection closed
    Closed,
    /// Connection error
    Error,
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub connection_id: String,
    pub address: SocketAddr,
    pub sni: Option<String>,
    pub state: ConnectionState,
    pub created_at: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub stream_count: usize,
    pub process_cap: String,
    pub namespace: String,
}

impl QuicConnection {
    /// Create a new QUIC connection
    pub fn new(
        id: String,
        address: SocketAddr,
        sni: Option<String>,
        process_cap: String,
        namespace: String,
    ) -> Self {
        Self {
            id,
            address,
            sni,
            state: ConnectionState::Created,
            process_cap,
            namespace,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            streams: Vec::new(),
            tls_session: None,
        }
    }

    /// Get connection information
    pub fn get_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            connection_id: self.id.clone(),
            address: self.address,
            sni: self.sni.clone(),
            state: self.state.clone(),
            created_at: self.created_at,
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
            stream_count: self.streams.len(),
            process_cap: self.process_cap.clone(),
            namespace: self.namespace.clone(),
        }
    }

    /// Check if connection is ready for I/O
    pub fn is_ready_for_io(&self) -> bool {
        self.state == ConnectionState::Established
    }

    /// Check if connection has error
    pub fn has_error(&self) -> bool {
        self.state == ConnectionState::Error
    }

    /// Get connection age
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Get time since last activity
    pub fn time_since_last_activity(&self) -> std::time::Duration {
        self.last_activity.elapsed()
    }
}
