//! QUIC stream implementation
//! 
//! This module provides QUIC stream functionality for the QUIC service.

use std::time::Instant;
use serde::{Deserialize, Serialize};

/// QUIC stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicStream {
    /// Stream ID
    pub id: String,
    /// Connection ID
    pub connection_id: String,
    /// Stream type
    pub stream_type: StreamType,
    /// Stream state
    pub state: StreamState,
    /// Process capability
    pub process_cap: String,
    /// Creation timestamp
    pub created_at: Instant,
    /// Last activity timestamp
    pub last_activity: Instant,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Buffer size
    pub buffer_size: usize,
    /// Send buffer
    pub send_buffer: Vec<u8>,
    /// Receive buffer
    pub recv_buffer: Vec<u8>,
}

/// Stream type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamType {
    /// Unidirectional stream
    Unidirectional,
    /// Bidirectional stream
    Bidirectional,
    /// HTTP/3 request stream
    Http3Request,
    /// HTTP/3 response stream
    Http3Response,
    /// Control stream
    Control,
}

/// Stream state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamState {
    /// Stream created
    Created,
    /// Stream opening
    Opening,
    /// Stream open
    Open,
    /// Stream closing
    Closing,
    /// Stream closed
    Closed,
    /// Stream error
    Error,
}

/// Stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub stream_id: String,
    pub connection_id: String,
    pub stream_type: StreamType,
    pub state: StreamState,
    pub created_at: Instant,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub process_cap: String,
}

impl QuicStream {
    /// Create a new QUIC stream
    pub fn new(
        id: String,
        connection_id: String,
        stream_type: StreamType,
        process_cap: String,
        buffer_size: usize,
    ) -> Self {
        Self {
            id,
            connection_id,
            stream_type,
            state: StreamState::Created,
            process_cap,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            buffer_size,
            send_buffer: Vec::new(),
            recv_buffer: Vec::new(),
        }
    }

    /// Get stream information
    pub fn get_info(&self) -> StreamInfo {
        StreamInfo {
            stream_id: self.id.clone(),
            connection_id: self.connection_id.clone(),
            stream_type: self.stream_type.clone(),
            state: self.state.clone(),
            created_at: self.created_at,
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
            process_cap: self.process_cap.clone(),
        }
    }

    /// Check if stream is ready for I/O
    pub fn is_ready_for_io(&self) -> bool {
        self.state == StreamState::Open
    }

    /// Check if stream has error
    pub fn has_error(&self) -> bool {
        self.state == StreamState::Error
    }

    /// Get stream age
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Get time since last activity
    pub fn time_since_last_activity(&self) -> std::time::Duration {
        self.last_activity.elapsed()
    }
}
