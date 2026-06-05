//! Socket implementation for POSIX networking
//! 
//! This module provides the core socket types and functionality for the
//! POSIX networking subsystem.

use std::net::{SocketAddr, IpAddr};
use std::time::{Duration, Instant};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::tls::TLSSession;

/// Socket handle representing a network socket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketHandle {
    /// Unique socket identifier
    pub id: String,
    /// Socket type (TCP, UDP, etc.)
    pub socket_type: SocketType,
    /// Current socket state
    pub state: SocketState,
    /// Process capability that owns this socket
    pub process_cap: String,
    /// Network namespace this socket belongs to
    pub namespace: String,
    /// Capabilities granted to this socket
    pub capabilities: Vec<String>,
    /// Creation timestamp
    pub created_at: Instant,
    /// Buffer size
    pub buffer_size: usize,
    /// Non-blocking mode
    pub non_blocking: bool,
    /// Socket options
    pub reuse_addr: bool,
    pub keep_alive: bool,
    pub broadcast: bool,
    pub linger: Option<Duration>,
    /// Buffer sizes
    pub send_buffer_size: usize,
    pub recv_buffer_size: usize,
    /// Timeouts
    pub send_timeout: Option<Duration>,
    pub recv_timeout: Option<Duration>,
    /// Last error
    pub error: Option<i32>,
    /// Local address
    pub local_addr: Option<SocketAddr>,
    /// Peer address
    pub peer_addr: Option<SocketAddr>,
    /// TLS session (if applicable)
    pub tls_session: Option<Arc<TLSSession>>,
}

/// Socket types supported by the broker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SocketType {
    /// TCP socket
    Tcp,
    /// UDP socket
    Udp,
    /// Raw socket
    Raw,
    /// Unix domain socket
    Unix,
    /// Packet socket
    Packet,
}

/// Socket states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SocketState {
    /// Socket created but not bound
    Created,
    /// Socket bound to local address
    Bound,
    /// Socket listening for connections
    Listening,
    /// Socket connected to peer
    Connected,
    /// Socket shutdown for reading
    ShutdownRead,
    /// Socket shutdown for writing
    ShutdownWrite,
    /// Socket shutdown for both reading and writing
    ShutdownBoth,
    /// Socket closed
    Closed,
    /// Socket in error state
    Error,
}

/// Socket capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SocketCapability {
    /// Can read from socket
    Read,
    /// Can write to socket
    Write,
    /// Can bind socket
    Bind,
    /// Can listen on socket
    Listen,
    /// Can accept connections
    Accept,
    /// Can connect to remote address
    Connect,
    /// Can set socket options
    SetOpt,
    /// Can get socket options
    GetOpt,
    /// Can shutdown socket
    Shutdown,
    /// Can close socket
    Close,
    /// Can poll socket
    Poll,
}

impl SocketHandle {
    /// Create a new socket handle
    pub fn new(
        id: String,
        socket_type: SocketType,
        process_cap: String,
        namespace: String,
    ) -> Self {
        Self {
            id,
            socket_type,
            state: SocketState::Created,
            process_cap,
            namespace,
            capabilities: vec!["socket:read".to_string(), "socket:write".to_string()],
            created_at: Instant::now(),
            buffer_size: crate::DEFAULT_SOCKET_BUFFER_SIZE,
            non_blocking: false,
            reuse_addr: false,
            keep_alive: false,
            broadcast: false,
            linger: None,
            send_buffer_size: crate::DEFAULT_SOCKET_BUFFER_SIZE,
            recv_buffer_size: crate::DEFAULT_SOCKET_BUFFER_SIZE,
            send_timeout: None,
            recv_timeout: None,
            error: None,
            local_addr: None,
            peer_addr: None,
            tls_session: None,
        }
    }

    /// Check if socket has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.contains(&capability.to_string())
    }

    /// Add a capability to the socket
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Remove a capability from the socket
    pub fn remove_capability(&mut self, capability: &str) {
        self.capabilities.retain(|cap| cap != capability);
    }

    /// Check if socket is in a valid state for the given operation
    pub fn is_valid_for_operation(&self, operation: &str) -> bool {
        match operation {
            "bind" => self.state == SocketState::Created,
            "listen" => self.state == SocketState::Bound,
            "accept" => self.state == SocketState::Listening,
            "connect" => self.state == SocketState::Created || self.state == SocketState::Bound,
            "send" => self.state == SocketState::Connected,
            "recv" => self.state == SocketState::Connected,
            "close" => true,
            "shutdown" => self.state == SocketState::Connected,
            _ => false,
        }
    }

    /// Get socket age
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Check if socket is ready for I/O
    pub fn is_ready_for_io(&self) -> bool {
        matches!(self.state, SocketState::Connected | SocketState::Listening)
    }

    /// Check if socket is in error state
    pub fn has_error(&self) -> bool {
        self.state == SocketState::Error || self.error.is_some()
    }

    /// Set socket error
    pub fn set_error(&mut self, error: i32) {
        self.error = Some(error);
        self.state = SocketState::Error;
    }

    /// Clear socket error
    pub fn clear_error(&mut self) {
        self.error = None;
        if self.state == SocketState::Error {
            self.state = SocketState::Created;
        }
    }

    /// Get socket statistics
    pub fn get_stats(&self) -> SocketStats {
        SocketStats {
            socket_id: self.id.clone(),
            socket_type: self.socket_type.clone(),
            state: self.state.clone(),
            created_at: self.created_at,
            age: self.age(),
            local_addr: self.local_addr,
            peer_addr: self.peer_addr,
            buffer_size: self.buffer_size,
            send_buffer_size: self.send_buffer_size,
            recv_buffer_size: self.recv_buffer_size,
            non_blocking: self.non_blocking,
            has_error: self.has_error(),
            error_code: self.error,
            capabilities: self.capabilities.clone(),
        }
    }
}

/// Socket statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketStats {
    pub socket_id: String,
    pub socket_type: SocketType,
    pub state: SocketState,
    pub created_at: Instant,
    pub age: Duration,
    pub local_addr: Option<SocketAddr>,
    pub peer_addr: Option<SocketAddr>,
    pub buffer_size: usize,
    pub send_buffer_size: usize,
    pub recv_buffer_size: usize,
    pub non_blocking: bool,
    pub has_error: bool,
    pub error_code: Option<i32>,
    pub capabilities: Vec<String>,
}

/// Socket address information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketAddrInfo {
    pub family: i32,
    pub socket_type: i32,
    pub protocol: i32,
    pub address: SocketAddr,
    pub canonical_name: Option<String>,
}

/// Socket message for sendmsg/recvmsg
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketMessage {
    pub data: Vec<u8>,
    pub address: Option<SocketAddr>,
    pub control_messages: Vec<ControlMessage>,
    pub flags: i32,
}

/// Control message for socket operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMessage {
    pub level: i32,
    pub message_type: i32,
    pub data: Vec<u8>,
}

/// Socket option information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketOption {
    pub level: i32,
    pub name: i32,
    pub value: Vec<u8>,
}

/// Socket buffer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketBuffer {
    pub send_buffer: Vec<u8>,
    pub recv_buffer: Vec<u8>,
    pub send_buffer_size: usize,
    pub recv_buffer_size: usize,
    pub send_buffer_used: usize,
    pub recv_buffer_used: usize,
}

impl SocketBuffer {
    /// Create a new socket buffer
    pub fn new(send_size: usize, recv_size: usize) -> Self {
        Self {
            send_buffer: Vec::with_capacity(send_size),
            recv_buffer: Vec::with_capacity(recv_size),
            send_buffer_size: send_size,
            recv_buffer_size: recv_size,
            send_buffer_used: 0,
            recv_buffer_used: 0,
        }
    }

    /// Check if send buffer has space
    pub fn can_send(&self, size: usize) -> bool {
        self.send_buffer_used + size <= self.send_buffer_size
    }

    /// Check if recv buffer has data
    pub fn can_recv(&self) -> bool {
        self.recv_buffer_used > 0
    }

    /// Add data to send buffer
    pub fn add_send_data(&mut self, data: &[u8]) -> Result<usize, String> {
        if !self.can_send(data.len()) {
            return Err("Send buffer full".to_string());
        }

        self.send_buffer.extend_from_slice(data);
        self.send_buffer_used += data.len();
        Ok(data.len())
    }

    /// Get data from recv buffer
    pub fn get_recv_data(&mut self, max_size: usize) -> Vec<u8> {
        let size = max_size.min(self.recv_buffer_used);
        if size == 0 {
            return Vec::new();
        }

        let data = self.recv_buffer.drain(..size).collect();
        self.recv_buffer_used -= size;
        data
    }

    /// Add data to recv buffer
    pub fn add_recv_data(&mut self, data: &[u8]) -> Result<usize, String> {
        if self.recv_buffer_used + data.len() > self.recv_buffer_size {
            return Err("Recv buffer full".to_string());
        }

        self.recv_buffer.extend_from_slice(data);
        self.recv_buffer_used += data.len();
        Ok(data.len())
    }

    /// Clear send buffer
    pub fn clear_send_buffer(&mut self) {
        self.send_buffer.clear();
        self.send_buffer_used = 0;
    }

    /// Clear recv buffer
    pub fn clear_recv_buffer(&mut self) {
        self.recv_buffer.clear();
        self.recv_buffer_used = 0;
    }
}

/// Socket event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SocketEvent {
    /// Socket is ready for reading
    ReadReady,
    /// Socket is ready for writing
    WriteReady,
    /// Socket has an error
    Error,
    /// Socket is closed
    Closed,
    /// Socket is connected
    Connected,
    /// Socket is listening
    Listening,
}

/// Socket event handler trait
pub trait SocketEventHandler: Send + Sync {
    /// Handle a socket event
    fn handle_event(&self, socket_id: &str, event: SocketEvent);
}

/// Socket manager for handling multiple sockets
pub struct SocketManager {
    sockets: std::collections::HashMap<String, SocketHandle>,
    event_handlers: Vec<Box<dyn SocketEventHandler>>,
}

impl SocketManager {
    /// Create a new socket manager
    pub fn new() -> Self {
        Self {
            sockets: std::collections::HashMap::new(),
            event_handlers: Vec::new(),
        }
    }

    /// Add a socket to the manager
    pub fn add_socket(&mut self, socket: SocketHandle) {
        self.sockets.insert(socket.id.clone(), socket);
    }

    /// Remove a socket from the manager
    pub fn remove_socket(&mut self, socket_id: &str) -> Option<SocketHandle> {
        self.sockets.remove(socket_id)
    }

    /// Get a socket by ID
    pub fn get_socket(&self, socket_id: &str) -> Option<&SocketHandle> {
        self.sockets.get(socket_id)
    }

    /// Get a mutable socket by ID
    pub fn get_socket_mut(&mut self, socket_id: &str) -> Option<&mut SocketHandle> {
        self.sockets.get_mut(socket_id)
    }

    /// Get all sockets
    pub fn get_all_sockets(&self) -> &std::collections::HashMap<String, SocketHandle> {
        &self.sockets
    }

    /// Add an event handler
    pub fn add_event_handler(&mut self, handler: Box<dyn SocketEventHandler>) {
        self.event_handlers.push(handler);
    }

    /// Emit an event to all handlers
    pub fn emit_event(&self, socket_id: &str, event: SocketEvent) {
        for handler in &self.event_handlers {
            handler.handle_event(socket_id, event);
        }
    }

    /// Get socket count
    pub fn socket_count(&self) -> usize {
        self.sockets.len()
    }

    /// Get sockets by process capability
    pub fn get_sockets_by_process(&self, process_cap: &str) -> Vec<&SocketHandle> {
        self.sockets
            .values()
            .filter(|socket| socket.process_cap == process_cap)
            .collect()
    }

    /// Get sockets by namespace
    pub fn get_sockets_by_namespace(&self, namespace: &str) -> Vec<&SocketHandle> {
        self.sockets
            .values()
            .filter(|socket| socket.namespace == namespace)
            .collect()
    }

    /// Get socket statistics
    pub fn get_stats(&self) -> SocketManagerStats {
        let mut stats = SocketManagerStats {
            total_sockets: self.sockets.len(),
            sockets_by_type: std::collections::HashMap::new(),
            sockets_by_state: std::collections::HashMap::new(),
            sockets_by_process: std::collections::HashMap::new(),
            sockets_by_namespace: std::collections::HashMap::new(),
        };

        for socket in self.sockets.values() {
            // Count by type
            let type_count = stats.sockets_by_type.entry(socket.socket_type.clone()).or_insert(0);
            *type_count += 1;

            // Count by state
            let state_count = stats.sockets_by_state.entry(socket.state.clone()).or_insert(0);
            *state_count += 1;

            // Count by process
            let process_count = stats.sockets_by_process.entry(socket.process_cap.clone()).or_insert(0);
            *process_count += 1;

            // Count by namespace
            let namespace_count = stats.sockets_by_namespace.entry(socket.namespace.clone()).or_insert(0);
            *namespace_count += 1;
        }

        stats
    }
}

/// Socket manager statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketManagerStats {
    pub total_sockets: usize,
    pub sockets_by_type: std::collections::HashMap<SocketType, usize>,
    pub sockets_by_state: std::collections::HashMap<SocketState, usize>,
    pub sockets_by_process: std::collections::HashMap<String, usize>,
    pub sockets_by_namespace: std::collections::HashMap<String, usize>,
}

impl Default for SocketManager {
    fn default() -> Self {
        Self::new()
    }
}
