//! Socket Broker - Main networking broker service
//! 
//! This module implements the core socket broker that mediates all network
//! operations through capability-based access control and policy enforcement.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};
use std::io::{self, ErrorKind};

use tokio::sync::{mpsc, RwLock};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::time::{timeout, sleep};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use dashmap::DashMap;
use tracing::{info, warn, error, debug, instrument};

use crate::socket::{SocketHandle, SocketType, SocketState, SocketCapability};
use crate::ns::{NetworkNamespace, NamespaceClass};
use crate::policy::{NetworkPolicy, PolicyDecision, PolicyEngine};
use crate::events::{EventLoop, EventToken, EventType};
use crate::tls::TLSManager;
use crate::errors;

/// Configuration for the socket broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    /// Maximum number of sockets per process
    pub max_sockets_per_process: usize,
    /// Default socket buffer size
    pub default_buffer_size: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Listen backlog
    pub listen_backlog: i32,
    /// Enable TLS support
    pub enable_tls: bool,
    /// Enable QUIC support
    pub enable_quic: bool,
    /// Enable libp2p overlay
    pub enable_libp2p: bool,
    /// Policy engine configuration
    pub policy_config: PolicyEngine,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            max_sockets_per_process: crate::MAX_SOCKETS_PER_PROCESS,
            default_buffer_size: crate::DEFAULT_SOCKET_BUFFER_SIZE,
            connection_timeout: Duration::from_secs(crate::DEFAULT_CONNECTION_TIMEOUT),
            listen_backlog: crate::DEFAULT_LISTEN_BACKLOG,
            enable_tls: true,
            enable_quic: true,
            enable_libp2p: true,
            policy_config: PolicyEngine::default(),
        }
    }
}

/// Request types for the socket broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrokerRequest {
    /// Create a new socket
    Socket {
        domain: i32,
        socket_type: i32,
        protocol: i32,
        process_cap: String,
        namespace: String,
    },
    /// Bind socket to address
    Bind {
        socket_id: String,
        address: SocketAddr,
        process_cap: String,
    },
    /// Listen on socket
    Listen {
        socket_id: String,
        backlog: i32,
        process_cap: String,
    },
    /// Accept connection
    Accept {
        socket_id: String,
        process_cap: String,
    },
    /// Connect to address
    Connect {
        socket_id: String,
        address: SocketAddr,
        process_cap: String,
    },
    /// Send data
    Send {
        socket_id: String,
        data: Vec<u8>,
        flags: i32,
        process_cap: String,
    },
    /// Receive data
    Recv {
        socket_id: String,
        buffer_size: usize,
        flags: i32,
        process_cap: String,
    },
    /// Close socket
    Close {
        socket_id: String,
        process_cap: String,
    },
    /// Get socket option
    GetSockOpt {
        socket_id: String,
        level: i32,
        optname: i32,
        process_cap: String,
    },
    /// Set socket option
    SetSockOpt {
        socket_id: String,
        level: i32,
        optname: i32,
        value: Vec<u8>,
        process_cap: String,
    },
    /// Shutdown socket
    Shutdown {
        socket_id: String,
        how: i32,
        process_cap: String,
    },
    /// Get address info
    GetAddrInfo {
        node: Option<String>,
        service: Option<String>,
        hints: Option<AddrInfoHints>,
        process_cap: String,
    },
    /// Poll sockets for events
    Poll {
        sockets: Vec<PollRequest>,
        timeout: i64,
        process_cap: String,
    },
    /// Create network namespace
    CreateNamespace {
        name: String,
        class: NamespaceClass,
        process_cap: String,
    },
    /// Set process namespace
    SetNamespace {
        process_cap: String,
        namespace: String,
    },
}

/// Response types for the socket broker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrokerResponse {
    /// Socket created
    SocketCreated { socket_id: String, fd: i32 },
    /// Socket bound
    SocketBound,
    /// Socket listening
    SocketListening,
    /// Connection accepted
    ConnectionAccepted { socket_id: String, fd: i32, peer_addr: SocketAddr },
    /// Connected
    Connected { peer_addr: SocketAddr },
    /// Data sent
    DataSent { bytes_sent: usize },
    /// Data received
    DataReceived { data: Vec<u8>, peer_addr: Option<SocketAddr> },
    /// Socket closed
    SocketClosed,
    /// Socket option value
    SocketOption { value: Vec<u8> },
    /// Socket option set
    SocketOptionSet,
    /// Socket shutdown
    SocketShutdown,
    /// Address info
    AddressInfo { addresses: Vec<SocketAddr> },
    /// Poll results
    PollResults { events: Vec<PollEvent> },
    /// Namespace created
    NamespaceCreated { namespace_id: String },
    /// Namespace set
    NamespaceSet,
    /// Error response
    Error { error_code: i32, message: String },
}

/// Address info hints for getaddrinfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddrInfoHints {
    pub ai_family: i32,
    pub ai_socktype: i32,
    pub ai_protocol: i32,
    pub ai_flags: i32,
}

/// Poll request for socket polling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollRequest {
    pub socket_id: String,
    pub events: i16,
}

/// Poll event result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollEvent {
    pub socket_id: String,
    pub events: i16,
    pub revents: i16,
}

/// Main socket broker service
pub struct SocketBroker {
    /// Broker configuration
    config: BrokerConfig,
    /// Active sockets
    sockets: Arc<DashMap<String, SocketHandle>>,
    /// Process socket mappings
    process_sockets: Arc<DashMap<String, Vec<String>>>,
    /// Network namespaces
    namespaces: Arc<DashMap<String, NetworkNamespace>>,
    /// Process namespace mappings
    process_namespaces: Arc<DashMap<String, String>>,
    /// Policy engine
    policy_engine: Arc<PolicyEngine>,
    /// Event loop
    event_loop: Arc<EventLoop>,
    /// TLS manager
    tls_manager: Arc<TLSManager>,
    /// Request/response channels
    request_tx: mpsc::UnboundedSender<BrokerRequest>,
    response_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<BrokerResponse>>>>,
    /// Audit log
    audit_log: Arc<Mutex<Vec<String>>>,
    /// Performance metrics
    metrics: Arc<Mutex<BrokerMetrics>>,
}

/// Performance metrics for the broker
#[derive(Debug, Default)]
struct BrokerMetrics {
    pub sockets_created: u64,
    pub sockets_destroyed: u64,
    pub connections_accepted: u64,
    pub connections_established: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub policy_decisions: u64,
    pub policy_denials: u64,
}

impl SocketBroker {
    /// Create a new socket broker
    pub fn new(config: BrokerConfig) -> Self {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (response_tx, response_rx) = mpsc::unbounded_channel();
        
        let broker = Self {
            config: config.clone(),
            sockets: Arc::new(DashMap::new()),
            process_sockets: Arc::new(DashMap::new()),
            namespaces: Arc::new(DashMap::new()),
            process_namespaces: Arc::new(DashMap::new()),
            policy_engine: Arc::new(config.policy_config),
            event_loop: Arc::new(EventLoop::new()),
            tls_manager: Arc::new(TLSManager::new()),
            request_tx,
            response_rx: Arc::new(Mutex::new(Some(response_rx))),
            audit_log: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(BrokerMetrics::default())),
        };

        // Start the broker event loop
        let broker_clone = broker.clone_for_task();
        tokio::spawn(async move {
            broker_clone.run_event_loop(request_rx, response_tx).await;
        });

        broker
    }

    /// Clone the broker for async tasks
    fn clone_for_task(&self) -> Arc<Self> {
        Arc::new(Self {
            config: self.config.clone(),
            sockets: self.sockets.clone(),
            process_sockets: self.process_sockets.clone(),
            namespaces: self.namespaces.clone(),
            process_namespaces: self.process_namespaces.clone(),
            policy_engine: self.policy_engine.clone(),
            event_loop: self.event_loop.clone(),
            tls_manager: self.tls_manager.clone(),
            request_tx: self.request_tx.clone(),
            response_rx: Arc::new(Mutex::new(None)),
            audit_log: self.audit_log.clone(),
            metrics: self.metrics.clone(),
        })
    }

    /// Send a request to the broker
    pub async fn send_request(&self, request: BrokerRequest) -> Result<BrokerResponse, String> {
        self.request_tx.send(request).map_err(|e| e.to_string())?;
        
        // Wait for response
        let response_rx = self.response_rx.lock().unwrap().take();
        if let Some(mut rx) = response_rx {
            rx.recv().await.ok_or_else(|| "Broker response channel closed".to_string())
        } else {
            Err("No response channel available".to_string())
        }
    }

    /// Run the main event loop
    async fn run_event_loop(
        self: Arc<Self>,
        mut request_rx: mpsc::UnboundedReceiver<BrokerRequest>,
        response_tx: mpsc::UnboundedSender<BrokerResponse>,
    ) {
        info!("Socket broker event loop started");

        while let Some(request) = request_rx.recv().await {
            let response = self.handle_request(request).await;
            
            if let Err(e) = response_tx.send(response) {
                error!("Failed to send response: {}", e);
                break;
            }
        }

        info!("Socket broker event loop stopped");
    }

    /// Handle a broker request
    #[instrument(skip(self))]
    async fn handle_request(&self, request: BrokerRequest) -> BrokerResponse {
        let start = Instant::now();
        
        let response = match request {
            BrokerRequest::Socket { domain, socket_type, protocol, process_cap, namespace } => {
                self.handle_socket(domain, socket_type, protocol, process_cap, namespace).await
            },
            BrokerRequest::Bind { socket_id, address, process_cap } => {
                self.handle_bind(socket_id, address, process_cap).await
            },
            BrokerRequest::Listen { socket_id, backlog, process_cap } => {
                self.handle_listen(socket_id, backlog, process_cap).await
            },
            BrokerRequest::Accept { socket_id, process_cap } => {
                self.handle_accept(socket_id, process_cap).await
            },
            BrokerRequest::Connect { socket_id, address, process_cap } => {
                self.handle_connect(socket_id, address, process_cap).await
            },
            BrokerRequest::Send { socket_id, data, flags, process_cap } => {
                self.handle_send(socket_id, data, flags, process_cap).await
            },
            BrokerRequest::Recv { socket_id, buffer_size, flags, process_cap } => {
                self.handle_recv(socket_id, buffer_size, flags, process_cap).await
            },
            BrokerRequest::Close { socket_id, process_cap } => {
                self.handle_close(socket_id, process_cap).await
            },
            BrokerRequest::GetSockOpt { socket_id, level, optname, process_cap } => {
                self.handle_get_sock_opt(socket_id, level, optname, process_cap).await
            },
            BrokerRequest::SetSockOpt { socket_id, level, optname, value, process_cap } => {
                self.handle_set_sock_opt(socket_id, level, optname, value, process_cap).await
            },
            BrokerRequest::Shutdown { socket_id, how, process_cap } => {
                self.handle_shutdown(socket_id, how, process_cap).await
            },
            BrokerRequest::GetAddrInfo { node, service, hints, process_cap } => {
                self.handle_get_addr_info(node, service, hints, process_cap).await
            },
            BrokerRequest::Poll { sockets, timeout, process_cap } => {
                self.handle_poll(sockets, timeout, process_cap).await
            },
            BrokerRequest::CreateNamespace { name, class, process_cap } => {
                self.handle_create_namespace(name, class, process_cap).await
            },
            BrokerRequest::SetNamespace { process_cap, namespace } => {
                self.handle_set_namespace(process_cap, namespace).await
            },
        };

        let duration = start.elapsed();
        self.log_audit(&format!("Request handled in {:?}: {:?}", duration, response));
        
        response
    }

    /// Handle socket creation
    async fn handle_socket(
        &self,
        domain: i32,
        socket_type: i32,
        protocol: i32,
        process_cap: String,
        namespace: String,
    ) -> BrokerResponse {
        // Check if process has socket creation capability
        if !self.check_socket_capability(&process_cap, "socket:create") {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Insufficient socket creation capability".to_string(),
            };
        }

        // Check socket limits
        if let Some(sockets) = self.process_sockets.get(&process_cap) {
            if sockets.len() >= self.config.max_sockets_per_process {
                return BrokerResponse::Error {
                    error_code: errors::EMFILE,
                    message: "Too many open sockets".to_string(),
                };
            }
        }

        // Validate domain and type
        let socket_type_enum = match (domain, socket_type) {
            (crate::families::AF_INET, crate::types::SOCK_STREAM) => SocketType::Tcp,
            (crate::families::AF_INET, crate::types::SOCK_DGRAM) => SocketType::Udp,
            (crate::families::AF_INET6, crate::types::SOCK_STREAM) => SocketType::Tcp,
            (crate::families::AF_INET6, crate::types::SOCK_DGRAM) => SocketType::Udp,
            _ => {
                return BrokerResponse::Error {
                    error_code: errors::EAFNOSUPPORT,
                    message: "Address family not supported".to_string(),
                };
            }
        };

        // Create socket handle
        let socket_id = Uuid::new_v4().to_string();
        let socket = SocketHandle {
            id: socket_id.clone(),
            socket_type: socket_type_enum,
            state: SocketState::Created,
            process_cap: process_cap.clone(),
            namespace: namespace.clone(),
            capabilities: vec!["socket:read".to_string(), "socket:write".to_string()],
            created_at: Instant::now(),
            buffer_size: self.config.default_buffer_size,
            non_blocking: false,
            reuse_addr: false,
            keep_alive: false,
            broadcast: false,
            linger: None,
            send_buffer_size: self.config.default_buffer_size,
            recv_buffer_size: self.config.default_buffer_size,
            send_timeout: None,
            recv_timeout: None,
            error: None,
            local_addr: None,
            peer_addr: None,
            tls_session: None,
        };

        // Store socket
        self.sockets.insert(socket_id.clone(), socket);
        
        // Add to process socket list
        self.process_sockets.entry(process_cap.clone())
            .or_insert_with(Vec::new)
            .push(socket_id.clone());

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.sockets_created += 1;
        }

        self.log_audit(&format!("Socket created: {} for process {}", socket_id, process_cap));

        BrokerResponse::SocketCreated {
            socket_id,
            fd: 0, // Mock file descriptor
        }
    }

    /// Handle socket binding
    async fn handle_bind(
        &self,
        socket_id: String,
        address: SocketAddr,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let mut socket = match self.sockets.get_mut(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Check policy
        let policy_decision = self.policy_engine.check_bind(&process_cap, &address).await;
        if !policy_decision.allowed {
            self.log_audit(&format!("Bind denied by policy: {} to {}", process_cap, address));
            return BrokerResponse::Error {
                error_code: errors::EACCES,
                message: format!("Bind denied: {}", policy_decision.reason),
            };
        }

        // Set local address
        socket.local_addr = Some(address);
        socket.state = SocketState::Bound;

        self.log_audit(&format!("Socket bound: {} to {}", socket_id, address));

        BrokerResponse::SocketBound
    }

    /// Handle socket listening
    async fn handle_listen(
        &self,
        socket_id: String,
        backlog: i32,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let mut socket = match self.sockets.get_mut(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Check state
        if socket.state != SocketState::Bound {
            return BrokerResponse::Error {
                error_code: errors::EINVAL,
                message: "Socket not bound".to_string(),
            };
        }

        // Set listening state
        socket.state = SocketState::Listening;

        self.log_audit(&format!("Socket listening: {} with backlog {}", socket_id, backlog));

        BrokerResponse::SocketListening
    }

    /// Handle connection acceptance
    async fn handle_accept(
        &self,
        socket_id: String,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let socket = match self.sockets.get(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Check state
        if socket.state != SocketState::Listening {
            return BrokerResponse::Error {
                error_code: errors::EINVAL,
                message: "Socket not listening".to_string(),
            };
        }

        // Mock connection acceptance
        let new_socket_id = Uuid::new_v4().to_string();
        let peer_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);

        // Create new socket for accepted connection
        let accepted_socket = SocketHandle {
            id: new_socket_id.clone(),
            socket_type: socket.socket_type.clone(),
            state: SocketState::Connected,
            process_cap: process_cap.clone(),
            namespace: socket.namespace.clone(),
            capabilities: socket.capabilities.clone(),
            created_at: Instant::now(),
            buffer_size: socket.buffer_size,
            non_blocking: socket.non_blocking,
            reuse_addr: socket.reuse_addr,
            keep_alive: socket.keep_alive,
            broadcast: socket.broadcast,
            linger: socket.linger,
            send_buffer_size: socket.send_buffer_size,
            recv_buffer_size: socket.recv_buffer_size,
            send_timeout: socket.send_timeout,
            recv_timeout: socket.recv_timeout,
            error: None,
            local_addr: socket.local_addr,
            peer_addr: Some(peer_addr),
            tls_session: None,
        };

        // Store accepted socket
        self.sockets.insert(new_socket_id.clone(), accepted_socket);
        
        // Add to process socket list
        self.process_sockets.entry(process_cap.clone())
            .or_insert_with(Vec::new)
            .push(new_socket_id.clone());

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.connections_accepted += 1;
        }

        self.log_audit(&format!("Connection accepted: {} from {}", new_socket_id, peer_addr));

        BrokerResponse::ConnectionAccepted {
            socket_id: new_socket_id,
            fd: 0, // Mock file descriptor
            peer_addr,
        }
    }

    /// Handle connection establishment
    async fn handle_connect(
        &self,
        socket_id: String,
        address: SocketAddr,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let mut socket = match self.sockets.get_mut(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Check policy
        let policy_decision = self.policy_engine.check_connect(&process_cap, &address).await;
        if !policy_decision.allowed {
            self.log_audit(&format!("Connect denied by policy: {} to {}", process_cap, address));
            return BrokerResponse::Error {
                error_code: errors::EACCES,
                message: format!("Connect denied: {}", policy_decision.reason),
            };
        }

        // Set connected state
        socket.state = SocketState::Connected;
        socket.peer_addr = Some(address);

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.connections_established += 1;
        }

        self.log_audit(&format!("Socket connected: {} to {}", socket_id, address));

        BrokerResponse::Connected { peer_addr: address }
    }

    /// Handle data sending
    async fn handle_send(
        &self,
        socket_id: String,
        data: Vec<u8>,
        flags: i32,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let socket = match self.sockets.get(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Check state
        if socket.state != SocketState::Connected {
            return BrokerResponse::Error {
                error_code: errors::ENOTCONN,
                message: "Socket not connected".to_string(),
            };
        }

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.bytes_sent += data.len() as u64;
        }

        self.log_audit(&format!("Data sent: {} bytes on socket {}", data.len(), socket_id));

        BrokerResponse::DataSent {
            bytes_sent: data.len(),
        }
    }

    /// Handle data receiving
    async fn handle_recv(
        &self,
        socket_id: String,
        buffer_size: usize,
        flags: i32,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let socket = match self.sockets.get(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Check state
        if socket.state != SocketState::Connected {
            return BrokerResponse::Error {
                error_code: errors::ENOTCONN,
                message: "Socket not connected".to_string(),
            };
        }

        // Mock data reception
        let mock_data = vec![0u8; buffer_size.min(1024)];
        let peer_addr = socket.peer_addr;

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.bytes_received += mock_data.len() as u64;
        }

        self.log_audit(&format!("Data received: {} bytes on socket {}", mock_data.len(), socket_id));

        BrokerResponse::DataReceived {
            data: mock_data,
            peer_addr,
        }
    }

    /// Handle socket closing
    async fn handle_close(
        &self,
        socket_id: String,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let socket = match self.sockets.get(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Remove socket
        self.sockets.remove(&socket_id);
        
        // Remove from process socket list
        if let Some(mut sockets) = self.process_sockets.get_mut(&process_cap) {
            sockets.retain(|id| id != &socket_id);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.sockets_destroyed += 1;
        }

        self.log_audit(&format!("Socket closed: {} by process {}", socket_id, process_cap));

        BrokerResponse::SocketClosed
    }

    /// Handle get socket option
    async fn handle_get_sock_opt(
        &self,
        socket_id: String,
        level: i32,
        optname: i32,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let socket = match self.sockets.get(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Mock socket option value
        let value = match optname {
            crate::options::SO_RCVBUF => socket.recv_buffer_size.to_le_bytes().to_vec(),
            crate::options::SO_SNDBUF => socket.send_buffer_size.to_le_bytes().to_vec(),
            crate::options::SO_ERROR => {
                if let Some(error) = socket.error {
                    error.to_le_bytes().to_vec()
                } else {
                    0i32.to_le_bytes().to_vec()
                }
            },
            _ => {
                return BrokerResponse::Error {
                    error_code: errors::ENOPROTOOPT,
                    message: "Socket option not supported".to_string(),
                };
            }
        };

        BrokerResponse::SocketOption { value }
    }

    /// Handle set socket option
    async fn handle_set_sock_opt(
        &self,
        socket_id: String,
        level: i32,
        optname: i32,
        value: Vec<u8>,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let mut socket = match self.sockets.get_mut(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Set socket option
        match optname {
            crate::options::SO_RCVBUF => {
                if value.len() >= 4 {
                    socket.recv_buffer_size = i32::from_le_bytes([value[0], value[1], value[2], value[3]]) as usize;
                }
            },
            crate::options::SO_SNDBUF => {
                if value.len() >= 4 {
                    socket.send_buffer_size = i32::from_le_bytes([value[0], value[1], value[2], value[3]]) as usize;
                }
            },
            crate::options::SO_NONBLOCK => {
                socket.non_blocking = value.len() > 0 && value[0] != 0;
            },
            crate::options::SO_REUSEADDR => {
                socket.reuse_addr = value.len() > 0 && value[0] != 0;
            },
            crate::options::SO_KEEPALIVE => {
                socket.keep_alive = value.len() > 0 && value[0] != 0;
            },
            crate::options::SO_BROADCAST => {
                socket.broadcast = value.len() > 0 && value[0] != 0;
            },
            _ => {
                return BrokerResponse::Error {
                    error_code: errors::ENOPROTOOPT,
                    message: "Socket option not supported".to_string(),
                };
            }
        }

        BrokerResponse::SocketOptionSet
    }

    /// Handle socket shutdown
    async fn handle_shutdown(
        &self,
        socket_id: String,
        how: i32,
        process_cap: String,
    ) -> BrokerResponse {
        // Get socket
        let mut socket = match self.sockets.get_mut(&socket_id) {
            Some(s) => s,
            None => {
                return BrokerResponse::Error {
                    error_code: errors::EBADF,
                    message: "Invalid socket".to_string(),
                };
            }
        };

        // Check capability
        if socket.process_cap != process_cap {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Socket access denied".to_string(),
            };
        }

        // Update socket state based on shutdown type
        match how {
            0 => socket.state = SocketState::ShutdownRead,  // SHUT_RD
            1 => socket.state = SocketState::ShutdownWrite, // SHUT_WR
            2 => socket.state = SocketState::ShutdownBoth,  // SHUT_RDWR
            _ => {
                return BrokerResponse::Error {
                    error_code: errors::EINVAL,
                    message: "Invalid shutdown type".to_string(),
                };
            }
        }

        self.log_audit(&format!("Socket shutdown: {} type {}", socket_id, how));

        BrokerResponse::SocketShutdown
    }

    /// Handle get address info
    async fn handle_get_addr_info(
        &self,
        node: Option<String>,
        service: Option<String>,
        hints: Option<AddrInfoHints>,
        process_cap: String,
    ) -> BrokerResponse {
        // Check capability
        if !self.check_socket_capability(&process_cap, "dns:resolve") {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Insufficient DNS resolution capability".to_string(),
            };
        }

        // Mock DNS resolution
        let addresses = if let Some(hostname) = node {
            match hostname.as_str() {
                "localhost" => vec![
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
                    SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), 0),
                ],
                _ => vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 0)],
            }
        } else {
            vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)]
        };

        BrokerResponse::AddressInfo { addresses }
    }

    /// Handle socket polling
    async fn handle_poll(
        &self,
        sockets: Vec<PollRequest>,
        timeout: i64,
        process_cap: String,
    ) -> BrokerResponse {
        // Check capability
        if !self.check_socket_capability(&process_cap, "socket:poll") {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Insufficient polling capability".to_string(),
            };
        }

        let mut events = Vec::new();

        for poll_req in sockets {
            if let Some(socket) = self.sockets.get(&poll_req.socket_id) {
                // Check capability
                if socket.process_cap != process_cap {
                    continue;
                }

                // Mock poll events based on socket state
                let revents = match socket.state {
                    SocketState::Connected => poll_req.events,
                    SocketState::Listening => poll_req.events & 0x0001, // POLLIN
                    _ => 0,
                };

                events.push(PollEvent {
                    socket_id: poll_req.socket_id,
                    events: poll_req.events,
                    revents,
                });
            }
        }

        BrokerResponse::PollResults { events }
    }

    /// Handle namespace creation
    async fn handle_create_namespace(
        &self,
        name: String,
        class: NamespaceClass,
        process_cap: String,
    ) -> BrokerResponse {
        // Check capability
        if !self.check_socket_capability(&process_cap, "namespace:create") {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Insufficient namespace creation capability".to_string(),
            };
        }

        let namespace_id = Uuid::new_v4().to_string();
        let namespace = NetworkNamespace {
            id: namespace_id.clone(),
            name: name.clone(),
            class,
            created_at: Instant::now(),
            processes: Vec::new(),
            sockets: Vec::new(),
            policy: NetworkPolicy::default(),
        };

        self.namespaces.insert(namespace_id.clone(), namespace);

        self.log_audit(&format!("Namespace created: {} ({}) by {}", name, namespace_id, process_cap));

        BrokerResponse::NamespaceCreated { namespace_id }
    }

    /// Handle namespace setting
    async fn handle_set_namespace(
        &self,
        process_cap: String,
        namespace: String,
    ) -> BrokerResponse {
        // Check capability
        if !self.check_socket_capability(&process_cap, "namespace:set") {
            return BrokerResponse::Error {
                error_code: errors::EPERM,
                message: "Insufficient namespace setting capability".to_string(),
            };
        }

        // Check if namespace exists
        if !self.namespaces.contains_key(&namespace) {
            return BrokerResponse::Error {
                error_code: errors::ENOENT,
                message: "Namespace not found".to_string(),
            };
        }

        self.process_namespaces.insert(process_cap.clone(), namespace.clone());

        self.log_audit(&format!("Process {} set to namespace {}", process_cap, namespace));

        BrokerResponse::NamespaceSet
    }

    /// Check socket capability
    fn check_socket_capability(&self, process_cap: &str, required_cap: &str) -> bool {
        // In real implementation, would check actual capabilities
        process_cap.contains(required_cap) || process_cap.contains("all")
    }

    /// Log audit event
    fn log_audit(&self, message: &str) {
        let mut audit_log = self.audit_log.lock().unwrap();
        audit_log.push(format!("[{}] {}", chrono::Utc::now().to_rfc3339(), message));
        if audit_log.len() > 1000 {
            audit_log.remove(0);
        }
    }

    /// Get broker metrics
    pub fn get_metrics(&self) -> BrokerMetrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> Vec<String> {
        self.audit_log.lock().unwrap().clone()
    }
}

impl Clone for BrokerMetrics {
    fn clone(&self) -> Self {
        Self {
            sockets_created: self.sockets_created,
            sockets_destroyed: self.sockets_destroyed,
            connections_accepted: self.connections_accepted,
            connections_established: self.connections_established,
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
            policy_decisions: self.policy_decisions,
            policy_denials: self.policy_denials,
        }
    }
}
