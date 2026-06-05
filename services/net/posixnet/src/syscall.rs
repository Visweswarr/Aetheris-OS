//! Syscall interface for POSIX networking
//! 
//! This module provides the syscall interface for socket operations.

use std::collections::HashMap;
use std::net::{SocketAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::socket::{Socket, SocketType, SocketFamily, SocketState};
use crate::readiness::{ReadinessManager, ReadinessEvent, ReadinessInterest};
use crate::audit::{AuditLogger, AuditEvent, AuditLevel};

/// Syscall request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallRequest {
    /// Request ID
    pub request_id: String,
    /// Syscall type
    pub syscall: SyscallType,
    /// Process capability
    pub process_cap: String,
    /// Arguments
    pub args: HashMap<String, serde_json::Value>,
    /// Timestamp
    pub timestamp: Instant,
}

/// Syscall response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallResponse {
    /// Request ID
    pub request_id: String,
    /// Success
    pub success: bool,
    /// Return value
    pub return_value: Option<i32>,
    /// Error code
    pub error_code: Option<i32>,
    /// Error message
    pub error_message: Option<String>,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Timestamp
    pub timestamp: Instant,
}

/// Syscall types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyscallType {
    /// Create socket
    Socket,
    /// Bind socket
    Bind,
    /// Listen on socket
    Listen,
    /// Accept connection
    Accept,
    /// Connect socket
    Connect,
    /// Send data
    Send,
    /// Receive data
    Recv,
    /// Send message
    SendMsg,
    /// Receive message
    RecvMsg,
    /// Shutdown socket
    Shutdown,
    /// Close socket
    Close,
    /// Get socket options
    GetSockOpt,
    /// Set socket options
    SetSockOpt,
    /// Select operation
    Select,
    /// Poll operation
    Poll,
    /// Epoll create
    EpollCreate,
    /// Epoll control
    EpollCtl,
    /// Epoll wait
    EpollWait,
    /// Fcntl operation
    Fcntl,
}

/// Syscall broker
pub struct SyscallBroker {
    /// Active sockets
    sockets: Arc<RwLock<HashMap<String, Arc<Socket>>>>,
    /// Readiness manager
    readiness_manager: Arc<ReadinessManager>,
    /// Audit logger
    audit_logger: Arc<AuditLogger>,
    /// Statistics
    stats: Arc<RwLock<SyscallStats>>,
}

/// Syscall statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyscallStats {
    /// Total syscalls
    pub total_syscalls: u64,
    /// Successful syscalls
    pub successful_syscalls: u64,
    /// Failed syscalls
    pub failed_syscalls: u64,
    /// Active sockets
    pub active_sockets: u64,
    /// Total bytes sent
    pub total_bytes_sent: u64,
    /// Total bytes received
    pub total_bytes_received: u64,
    /// Total connections
    pub total_connections: u64,
}

impl SyscallBroker {
    /// Create a new syscall broker
    pub fn new() -> Self {
        Self {
            sockets: Arc::new(RwLock::new(HashMap::new())),
            readiness_manager: Arc::new(ReadinessManager::new()),
            audit_logger: Arc::new(AuditLogger::new()),
            stats: Arc::new(RwLock::new(SyscallStats::default())),
        }
    }

    /// Process syscall request
    pub async fn process_request(&self, request: SyscallRequest) -> SyscallResponse {
        let start_time = Instant::now();
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_syscalls += 1;
        drop(stats);
        
        // Log audit event
        self.audit_logger.log_event(AuditEvent {
            level: AuditLevel::Info,
            message: format!("Syscall: {:?}", request.syscall),
            process_cap: request.process_cap.clone(),
            timestamp: request.timestamp,
            data: Some(serde_json::to_value(&request.args).unwrap_or_default()),
        }).await;
        
        let result = match request.syscall {
            SyscallType::Socket => self.handle_socket(request).await,
            SyscallType::Bind => self.handle_bind(request).await,
            SyscallType::Listen => self.handle_listen(request).await,
            SyscallType::Accept => self.handle_accept(request).await,
            SyscallType::Connect => self.handle_connect(request).await,
            SyscallType::Send => self.handle_send(request).await,
            SyscallType::Recv => self.handle_recv(request).await,
            SyscallType::SendMsg => self.handle_sendmsg(request).await,
            SyscallType::RecvMsg => self.handle_recvmsg(request).await,
            SyscallType::Shutdown => self.handle_shutdown(request).await,
            SyscallType::Close => self.handle_close(request).await,
            SyscallType::GetSockOpt => self.handle_getsockopt(request).await,
            SyscallType::SetSockOpt => self.handle_setsockopt(request).await,
            SyscallType::Select => self.handle_select(request).await,
            SyscallType::Poll => self.handle_poll(request).await,
            SyscallType::EpollCreate => self.handle_epoll_create(request).await,
            SyscallType::EpollCtl => self.handle_epoll_ctl(request).await,
            SyscallType::EpollWait => self.handle_epoll_wait(request).await,
            SyscallType::Fcntl => self.handle_fcntl(request).await,
        };
        
        let duration = start_time.elapsed();
        
        // Update statistics based on result
        let mut stats = self.stats.write().await;
        if result.success {
            stats.successful_syscalls += 1;
        } else {
            stats.failed_syscalls += 1;
        }
        
        // Log completion
        self.audit_logger.log_event(AuditEvent {
            level: if result.success { AuditLevel::Info } else { AuditLevel::Warning },
            message: format!("Syscall completed: {:?} in {:?}", request.syscall, duration),
            process_cap: request.process_cap.clone(),
            timestamp: Instant::now(),
            data: Some(serde_json::to_value(&result).unwrap_or_default()),
        }).await;
        
        result
    }

    /// Handle socket syscall
    async fn handle_socket(&self, request: SyscallRequest) -> SyscallResponse {
        let domain = request.args.get("domain")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let socket_type = request.args.get("type")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let protocol = request.args.get("protocol")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        
        // Check capability
        if !self.check_network_capability(&request.process_cap).await {
            return self.create_error_response(
                &request.request_id,
                libc::EPERM,
                "Network capability required"
            );
        }
        
        // Create socket
        let socket_id = format!("socket_{}", uuid::Uuid::new_v4());
        let socket = Arc::new(Socket::new(
            socket_id.clone(),
            SocketFamily::from_i32(domain),
            SocketType::from_i32(socket_type),
            protocol,
        ));
        
        // Store socket
        let mut sockets = self.sockets.write().await;
        sockets.insert(socket_id.clone(), socket);
        let socket_count = sockets.len();
        drop(sockets);
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.active_sockets = socket_count as u64;
        
        SyscallResponse {
            request_id: request.request_id,
            success: true,
            return_value: Some(0), // Mock file descriptor
            error_code: None,
            error_message: None,
            data: Some(serde_json::json!({
                "socket_id": socket_id,
                "fd": 0
            })),
            timestamp: Instant::now(),
        }
    }

    /// Handle bind syscall
    async fn handle_bind(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let address = request.args.get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let port = request.args.get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;
        
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(socket_id) {
            Some(socket) => socket.clone(),
            None => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EBADF,
                    "Invalid socket"
                );
            }
        };
        drop(sockets);
        
        // Parse address
        let socket_addr = match self.parse_address(address, port) {
            Ok(addr) => addr,
            Err(e) => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EINVAL,
                    &format!("Invalid address: {}", e)
                );
            }
        };
        
        // Bind socket
        match socket.bind(socket_addr).await {
            Ok(_) => {
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(0),
                    error_code: None,
                    error_message: None,
                    data: None,
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::EADDRINUSE,
                    &format!("Bind failed: {}", e)
                )
            }
        }
    }

    /// Handle listen syscall
    async fn handle_listen(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let backlog = request.args.get("backlog")
            .and_then(|v| v.as_i64())
            .unwrap_or(128) as i32;
        
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(socket_id) {
            Some(socket) => socket.clone(),
            None => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EBADF,
                    "Invalid socket"
                );
            }
        };
        drop(sockets);
        
        // Listen on socket
        match socket.listen(backlog).await {
            Ok(_) => {
                // Notify readiness for accept
                self.readiness_manager.notify_event(
                    socket_id.to_string(),
                    ReadinessEvent::Accept,
                    None,
                ).await.ok();
                
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(0),
                    error_code: None,
                    error_message: None,
                    data: None,
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::EINVAL,
                    &format!("Listen failed: {}", e)
                )
            }
        }
    }

    /// Handle accept syscall
    async fn handle_accept(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(socket_id) {
            Some(socket) => socket.clone(),
            None => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EBADF,
                    "Invalid socket"
                );
            }
        };
        drop(sockets);
        
        // Accept connection
        match socket.accept().await {
            Ok((new_socket, client_addr)) => {
                let new_socket_id = format!("socket_{}", uuid::Uuid::new_v4());
                
                // Store new socket
                let mut sockets = self.sockets.write().await;
                sockets.insert(new_socket_id.clone(), Arc::new(new_socket));
                let socket_count = sockets.len();
                drop(sockets);
                
                // Update statistics
                let mut stats = self.stats.write().await;
                stats.active_sockets = socket_count as u64;
                stats.total_connections += 1;
                
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(0), // Mock file descriptor
                    error_code: None,
                    error_message: None,
                    data: Some(serde_json::json!({
                        "socket_id": new_socket_id,
                        "client_address": client_addr.to_string(),
                        "fd": 0
                    })),
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::EAGAIN,
                    &format!("Accept failed: {}", e)
                )
            }
        }
    }

    /// Handle connect syscall
    async fn handle_connect(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let address = request.args.get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let port = request.args.get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;
        
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(socket_id) {
            Some(socket) => socket.clone(),
            None => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EBADF,
                    "Invalid socket"
                );
            }
        };
        drop(sockets);
        
        // Parse address
        let socket_addr = match self.parse_address(address, port) {
            Ok(addr) => addr,
            Err(e) => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EINVAL,
                    &format!("Invalid address: {}", e)
                );
            }
        };
        
        // Connect socket
        match socket.connect(socket_addr).await {
            Ok(_) => {
                // Notify readiness for write
                self.readiness_manager.notify_event(
                    socket_id.to_string(),
                    ReadinessEvent::Write,
                    None,
                ).await.ok();
                
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(0),
                    error_code: None,
                    error_message: None,
                    data: None,
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::ECONNREFUSED,
                    &format!("Connect failed: {}", e)
                )
            }
        }
    }

    /// Handle send syscall
    async fn handle_send(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let data = request.args.get("data")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let flags = request.args.get("flags")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(socket_id) {
            Some(socket) => socket.clone(),
            None => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EBADF,
                    "Invalid socket"
                );
            }
        };
        drop(sockets);
        
        // Send data
        let data_bytes = data.as_bytes();
        match socket.send(data_bytes, flags).await {
            Ok(bytes_sent) => {
                // Update statistics
                let mut stats = self.stats.write().await;
                stats.total_bytes_sent += bytes_sent as u64;
                
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(bytes_sent),
                    error_code: None,
                    error_message: None,
                    data: Some(serde_json::json!({
                        "bytes_sent": bytes_sent
                    })),
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::EAGAIN,
                    &format!("Send failed: {}", e)
                )
            }
        }
    }

    /// Handle recv syscall
    async fn handle_recv(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let buffer_size = request.args.get("buffer_size")
            .and_then(|v| v.as_i64())
            .unwrap_or(4096) as usize;
        let flags = request.args.get("flags")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(socket_id) {
            Some(socket) => socket.clone(),
            None => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EBADF,
                    "Invalid socket"
                );
            }
        };
        drop(sockets);
        
        // Receive data
        match socket.recv(buffer_size, flags).await {
            Ok((data, bytes_received)) => {
                // Update statistics
                let mut stats = self.stats.write().await;
                stats.total_bytes_received += bytes_received as u64;
                
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(bytes_received),
                    error_code: None,
                    error_message: None,
                    data: Some(serde_json::json!({
                        "data": String::from_utf8_lossy(&data),
                        "bytes_received": bytes_received
                    })),
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::EAGAIN,
                    &format!("Recv failed: {}", e)
                )
            }
        }
    }

    /// Handle sendmsg syscall
    async fn handle_sendmsg(&self, request: SyscallRequest) -> SyscallResponse {
        // Mock implementation
        self.handle_send(request).await
    }

    /// Handle recvmsg syscall
    async fn handle_recvmsg(&self, request: SyscallRequest) -> SyscallResponse {
        // Mock implementation
        self.handle_recv(request).await
    }

    /// Handle shutdown syscall
    async fn handle_shutdown(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let how = request.args.get("how")
            .and_then(|v| v.as_i64())
            .unwrap_or(2) as i32; // SHUT_RDWR
        
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(socket_id) {
            Some(socket) => socket.clone(),
            None => {
                return self.create_error_response(
                    &request.request_id,
                    libc::EBADF,
                    "Invalid socket"
                );
            }
        };
        drop(sockets);
        
        // Shutdown socket
        match socket.shutdown(how).await {
            Ok(_) => {
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(0),
                    error_code: None,
                    error_message: None,
                    data: None,
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::ENOTCONN,
                    &format!("Shutdown failed: {}", e)
                )
            }
        }
    }

    /// Handle close syscall
    async fn handle_close(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Remove socket
        let mut sockets = self.sockets.write().await;
        if sockets.remove(socket_id).is_some() {
            let socket_count = sockets.len();
            drop(sockets);
            
            // Update statistics
            let mut stats = self.stats.write().await;
            stats.active_sockets = socket_count as u64;
            
            SyscallResponse {
                request_id: request.request_id,
                success: true,
                return_value: Some(0),
                error_code: None,
                error_message: None,
                data: None,
                timestamp: Instant::now(),
            }
        } else {
            drop(sockets);
            self.create_error_response(
                &request.request_id,
                libc::EBADF,
                "Invalid socket"
            )
        }
    }

    /// Handle getsockopt syscall
    async fn handle_getsockopt(&self, _request: SyscallRequest) -> SyscallResponse {
        // Mock implementation
        SyscallResponse {
            request_id: _request.request_id,
            success: true,
            return_value: Some(0),
            error_code: None,
            error_message: None,
            data: None,
            timestamp: Instant::now(),
        }
    }

    /// Handle setsockopt syscall
    async fn handle_setsockopt(&self, _request: SyscallRequest) -> SyscallResponse {
        // Mock implementation
        SyscallResponse {
            request_id: _request.request_id,
            success: true,
            return_value: Some(0),
            error_code: None,
            error_message: None,
            data: None,
            timestamp: Instant::now(),
        }
    }

    /// Handle select syscall
    async fn handle_select(&self, request: SyscallRequest) -> SyscallResponse {
        let read_sockets = request.args.get("read_sockets")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();
        let write_sockets = request.args.get("write_sockets")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();
        let error_sockets = request.args.get("error_sockets")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();
        let timeout = request.args.get("timeout")
            .and_then(|v| v.as_f64())
            .map(|t| Duration::from_secs_f64(t));
        
        match self.readiness_manager.select(
            read_sockets,
            write_sockets,
            error_sockets,
            timeout,
        ).await {
            Ok(result) => {
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(
                        result.read_ready.len() + 
                        result.write_ready.len() + 
                        result.error_ready.len()
                    ),
                    error_code: None,
                    error_message: None,
                    data: Some(serde_json::to_value(&result).unwrap_or_default()),
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::EINVAL,
                    &format!("Select failed: {}", e)
                )
            }
        }
    }

    /// Handle poll syscall
    async fn handle_poll(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_ids = request.args.get("socket_ids")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();
        let interests = request.args.get("interests")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64()).map(|i| ReadinessInterest::from(i as u32)).collect())
            .unwrap_or_default();
        let timeout = request.args.get("timeout")
            .and_then(|v| v.as_f64())
            .map(|t| Duration::from_secs_f64(t));
        
        match self.readiness_manager.poll(socket_ids, interests, timeout).await {
            Ok(events) => {
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(events.len() as i32),
                    error_code: None,
                    error_message: None,
                    data: Some(serde_json::to_value(&events).unwrap_or_default()),
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::EINVAL,
                    &format!("Poll failed: {}", e)
                )
            }
        }
    }

    /// Handle epoll_create syscall
    async fn handle_epoll_create(&self, _request: SyscallRequest) -> SyscallResponse {
        // Mock implementation
        SyscallResponse {
            request_id: _request.request_id,
            success: true,
            return_value: Some(0), // Mock epoll file descriptor
            error_code: None,
            error_message: None,
            data: None,
            timestamp: Instant::now(),
        }
    }

    /// Handle epoll_ctl syscall
    async fn handle_epoll_ctl(&self, _request: SyscallRequest) -> SyscallResponse {
        // Mock implementation
        SyscallResponse {
            request_id: _request.request_id,
            success: true,
            return_value: Some(0),
            error_code: None,
            error_message: None,
            data: None,
            timestamp: Instant::now(),
        }
    }

    /// Handle epoll_wait syscall
    async fn handle_epoll_wait(&self, request: SyscallRequest) -> SyscallResponse {
        let epoll_fd = request.args.get("epoll_fd")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let max_events = request.args.get("max_events")
            .and_then(|v| v.as_i64())
            .unwrap_or(128) as usize;
        let timeout = request.args.get("timeout")
            .and_then(|v| v.as_i64())
            .map(|t| Duration::from_millis(t));
        
        match self.readiness_manager.epoll_wait(epoll_fd, max_events, timeout).await {
            Ok(events) => {
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(events.len() as i32),
                    error_code: None,
                    error_message: None,
                    data: Some(serde_json::to_value(&events).unwrap_or_default()),
                    timestamp: Instant::now(),
                }
            }
            Err(e) => {
                self.create_error_response(
                    &request.request_id,
                    libc::EINVAL,
                    &format!("Epoll wait failed: {}", e)
                )
            }
        }
    }

    /// Handle fcntl syscall
    async fn handle_fcntl(&self, request: SyscallRequest) -> SyscallResponse {
        let socket_id = request.args.get("socket_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let cmd = request.args.get("cmd")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let arg = request.args.get("arg")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        
        // Handle O_NONBLOCK
        if cmd == 4 { // F_SETFL
            if arg & 0x800 { // O_NONBLOCK
                // Set socket to non-blocking mode
                let sockets = self.sockets.read().await;
                if let Some(socket) = sockets.get(socket_id) {
                    // Mock setting non-blocking mode
                    drop(sockets);
                    
                    SyscallResponse {
                        request_id: request.request_id,
                        success: true,
                        return_value: Some(0),
                        error_code: None,
                        error_message: None,
                        data: None,
                        timestamp: Instant::now(),
                    }
                } else {
                    drop(sockets);
                    self.create_error_response(
                        &request.request_id,
                        libc::EBADF,
                        "Invalid socket"
                    )
                }
            } else {
                SyscallResponse {
                    request_id: request.request_id,
                    success: true,
                    return_value: Some(0),
                    error_code: None,
                    error_message: None,
                    data: None,
                    timestamp: Instant::now(),
                }
            }
        } else {
            SyscallResponse {
                request_id: request.request_id,
                success: true,
                return_value: Some(0),
                error_code: None,
                error_message: None,
                data: None,
                timestamp: Instant::now(),
            }
        }
    }

    /// Check network capability
    async fn check_network_capability(&self, process_cap: &str) -> bool {
        // Mock capability check - in real implementation would check actual capabilities
        process_cap.contains("net:socket") || process_cap.contains("net:all")
    }

    /// Parse address string
    fn parse_address(&self, address: &str, port: u16) -> Result<SocketAddr, String> {
        if address.is_empty() {
            return Ok(SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), port));
        }
        
        if let Ok(ipv4) = address.parse::<Ipv4Addr>() {
            Ok(SocketAddr::new(ipv4.into(), port))
        } else if let Ok(ipv6) = address.parse::<Ipv6Addr>() {
            Ok(SocketAddr::new(ipv6.into(), port))
        } else {
            Err(format!("Invalid address: {}", address))
        }
    }

    /// Create error response
    fn create_error_response(
        &self,
        request_id: &str,
        error_code: i32,
        error_message: &str,
    ) -> SyscallResponse {
        SyscallResponse {
            request_id: request_id.to_string(),
            success: false,
            return_value: Some(-1),
            error_code: Some(error_code),
            error_message: Some(error_message.to_string()),
            data: None,
            timestamp: Instant::now(),
        }
    }

    /// Get syscall statistics
    pub async fn get_stats(&self) -> SyscallStats {
        self.stats.read().await.clone()
    }
}

impl Default for SyscallBroker {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert ReadinessInterest from u32
impl From<u32> for ReadinessInterest {
    fn from(value: u32) -> Self {
        match value {
            0x001 => ReadinessInterest::Read,
            0x004 => ReadinessInterest::Write,
            0x008 => ReadinessInterest::Error,
            0x010 => ReadinessInterest::Accept,
            _ => ReadinessInterest::Read,
        }
    }
}
