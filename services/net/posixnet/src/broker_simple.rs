//! Simple Socket Broker for P4-03-A1
//! 
//! This module implements a focused socket broker for the core POSIX socket operations
//! and readiness APIs as specified in P4-03-A1.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use crate::socket::{Socket, SocketType, SocketFamily, SocketState};
use crate::readiness::{ReadinessManager, ReadinessEvent, ReadinessInterest};
use crate::syscall::{SyscallBroker, SyscallRequest, SyscallResponse, SyscallType};
use crate::audit::{AuditLogger, AuditLevel};

/// Simple socket broker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBrokerConfig {
    /// Maximum sockets per process
    pub max_sockets_per_process: usize,
    /// Default buffer size
    pub default_buffer_size: usize,
    /// Enable audit logging
    pub enable_audit: bool,
    /// Audit log size
    pub audit_log_size: usize,
}

impl Default for SimpleBrokerConfig {
    fn default() -> Self {
        Self {
            max_sockets_per_process: 1024,
            default_buffer_size: 65536,
            enable_audit: true,
            audit_log_size: 10000,
        }
    }
}

/// Simple socket broker
pub struct SimpleSocketBroker {
    /// Configuration
    config: SimpleBrokerConfig,
    /// Active sockets
    sockets: Arc<RwLock<HashMap<String, Arc<Socket>>>>,
    /// Process socket mappings
    process_sockets: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Readiness manager
    readiness_manager: Arc<ReadinessManager>,
    /// Syscall broker
    syscall_broker: Arc<SyscallBroker>,
    /// Audit logger
    audit_logger: Arc<AuditLogger>,
    /// Statistics
    stats: Arc<RwLock<BrokerStats>>,
}

/// Broker statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrokerStats {
    /// Total sockets created
    pub sockets_created: u64,
    /// Total sockets destroyed
    pub sockets_destroyed: u64,
    /// Total connections
    pub total_connections: u64,
    /// Total bytes sent
    pub total_bytes_sent: u64,
    /// Total bytes received
    pub total_bytes_received: u64,
    /// Total syscalls processed
    pub total_syscalls: u64,
    /// Successful syscalls
    pub successful_syscalls: u64,
    /// Failed syscalls
    pub failed_syscalls: u64,
}

impl SimpleSocketBroker {
    /// Create a new simple socket broker
    pub fn new(config: SimpleBrokerConfig) -> Self {
        Self {
            config: config.clone(),
            sockets: Arc::new(RwLock::new(HashMap::new())),
            process_sockets: Arc::new(RwLock::new(HashMap::new())),
            readiness_manager: Arc::new(ReadinessManager::new()),
            syscall_broker: Arc::new(SyscallBroker::new()),
            audit_logger: Arc::new(AuditLogger::with_buffer_size(config.audit_log_size)),
            stats: Arc::new(RwLock::new(BrokerStats::default())),
        }
    }

    /// Process a syscall request
    pub async fn process_syscall(&self, request: SyscallRequest) -> SyscallResponse {
        let start_time = Instant::now();
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_syscalls += 1;
        drop(stats);
        
        // Log audit event
        if self.config.enable_audit {
            self.audit_logger.log_event(crate::audit::AuditEvent {
                level: AuditLevel::Info,
                message: format!("Syscall: {:?}", request.syscall),
                process_cap: request.process_cap.clone(),
                timestamp: request.timestamp,
                data: Some(serde_json::to_value(&request.args).unwrap_or_default()),
            }).await;
        }
        
        // Process the syscall
        let response = self.syscall_broker.process_request(request).await;
        
        // Update statistics based on result
        let mut stats = self.stats.write().await;
        if response.success {
            stats.successful_syscalls += 1;
        } else {
            stats.failed_syscalls += 1;
        }
        
        // Log completion
        if self.config.enable_audit {
            self.audit_logger.log_event(crate::audit::AuditEvent {
                level: if response.success { AuditLevel::Info } else { AuditLevel::Warning },
                message: format!("Syscall completed: {:?} in {:?}", response.request_id, start_time.elapsed()),
                process_cap: "system".to_string(),
                timestamp: Instant::now(),
                data: Some(serde_json::to_value(&response).unwrap_or_default()),
            }).await;
        }
        
        response
    }

    /// Create a socket
    pub async fn create_socket(
        &self,
        domain: i32,
        socket_type: i32,
        protocol: i32,
        process_cap: String,
    ) -> Result<String, String> {
        // Check capability
        if !self.check_network_capability(&process_cap).await {
            return Err("Network capability required".to_string());
        }
        
        // Check socket limits
        let process_sockets = self.process_sockets.read().await;
        if let Some(sockets) = process_sockets.get(&process_cap) {
            if sockets.len() >= self.config.max_sockets_per_process {
                return Err("Too many open sockets".to_string());
            }
        }
        drop(process_sockets);
        
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
        drop(sockets);
        
        // Add to process socket list
        let mut process_sockets = self.process_sockets.write().await;
        process_sockets.entry(process_cap.clone())
            .or_insert_with(Vec::new)
            .push(socket_id.clone());
        drop(process_sockets);
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.sockets_created += 1;
        drop(stats);
        
        // Log audit event
        if self.config.enable_audit {
            self.audit_logger.log_socket_creation(
                process_cap,
                socket_id.clone(),
                format!("{:?}", SocketFamily::from_i32(domain)),
                format!("{:?}", SocketType::from_i32(socket_type)),
                protocol,
            ).await;
        }
        
        Ok(socket_id)
    }

    /// Bind a socket
    pub async fn bind_socket(
        &self,
        socket_id: String,
        address: String,
        port: u16,
        process_cap: String,
    ) -> Result<(), String> {
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(&socket_id) {
            Some(socket) => socket.clone(),
            None => return Err("Invalid socket".to_string()),
        };
        drop(sockets);
        
        // Check capability
        if socket.process_cap != process_cap {
            return Err("Socket access denied".to_string());
        }
        
        // Parse address
        let socket_addr = self.parse_address(&address, port)?;
        
        // Bind socket
        socket.bind(socket_addr).await?;
        
        // Log audit event
        if self.config.enable_audit {
            self.audit_logger.log_socket_bind(
                process_cap,
                socket_id,
                address,
                port,
            ).await;
        }
        
        Ok(())
    }

    /// Listen on a socket
    pub async fn listen_socket(
        &self,
        socket_id: String,
        backlog: i32,
        process_cap: String,
    ) -> Result<(), String> {
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(&socket_id) {
            Some(socket) => socket.clone(),
            None => return Err("Invalid socket".to_string()),
        };
        drop(sockets);
        
        // Check capability
        if socket.process_cap != process_cap {
            return Err("Socket access denied".to_string());
        }
        
        // Listen on socket
        socket.listen(backlog).await?;
        
        // Notify readiness for accept
        self.readiness_manager.notify_event(
            socket_id.clone(),
            ReadinessEvent::Accept,
            None,
        ).await.ok();
        
        // Log audit event
        if self.config.enable_audit {
            self.audit_logger.log_socket_listen(
                process_cap,
                socket_id,
                backlog,
            ).await;
        }
        
        Ok(())
    }

    /// Accept a connection
    pub async fn accept_connection(
        &self,
        socket_id: String,
        process_cap: String,
    ) -> Result<(String, String), String> {
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(&socket_id) {
            Some(socket) => socket.clone(),
            None => return Err("Invalid socket".to_string()),
        };
        drop(sockets);
        
        // Check capability
        if socket.process_cap != process_cap {
            return Err("Socket access denied".to_string());
        }
        
        // Accept connection
        let (new_socket, client_addr) = socket.accept().await?;
        let new_socket_id = format!("socket_{}", uuid::Uuid::new_v4());
        
        // Store new socket
        let mut sockets = self.sockets.write().await;
        sockets.insert(new_socket_id.clone(), Arc::new(new_socket));
        drop(sockets);
        
        // Add to process socket list
        let mut process_sockets = self.process_sockets.write().await;
        process_sockets.entry(process_cap.clone())
            .or_insert_with(Vec::new)
            .push(new_socket_id.clone());
        drop(process_sockets);
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_connections += 1;
        drop(stats);
        
        // Log audit event
        if self.config.enable_audit {
            self.audit_logger.log_socket_accept(
                process_cap,
                socket_id,
                client_addr.to_string(),
                new_socket_id.clone(),
            ).await;
        }
        
        Ok((new_socket_id, client_addr.to_string()))
    }

    /// Connect a socket
    pub async fn connect_socket(
        &self,
        socket_id: String,
        address: String,
        port: u16,
        process_cap: String,
    ) -> Result<(), String> {
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(&socket_id) {
            Some(socket) => socket.clone(),
            None => return Err("Invalid socket".to_string()),
        };
        drop(sockets);
        
        // Check capability
        if socket.process_cap != process_cap {
            return Err("Socket access denied".to_string());
        }
        
        // Parse address
        let socket_addr = self.parse_address(&address, port)?;
        
        // Connect socket
        socket.connect(socket_addr).await?;
        
        // Notify readiness for write
        self.readiness_manager.notify_event(
            socket_id.clone(),
            ReadinessEvent::Write,
            None,
        ).await.ok();
        
        // Log audit event
        if self.config.enable_audit {
            self.audit_logger.log_socket_connect(
                process_cap,
                socket_id,
                address,
                port,
                true,
            ).await;
        }
        
        Ok(())
    }

    /// Send data
    pub async fn send_data(
        &self,
        socket_id: String,
        data: Vec<u8>,
        flags: i32,
        process_cap: String,
    ) -> Result<usize, String> {
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(&socket_id) {
            Some(socket) => socket.clone(),
            None => return Err("Invalid socket".to_string()),
        };
        drop(sockets);
        
        // Check capability
        if socket.process_cap != process_cap {
            return Err("Socket access denied".to_string());
        }
        
        // Send data
        let bytes_sent = socket.send(&data, flags).await?;
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_bytes_sent += bytes_sent as u64;
        drop(stats);
        
        // Log audit event
        if self.config.enable_audit {
            self.audit_logger.log_socket_send(
                process_cap,
                socket_id,
                bytes_sent,
                None,
            ).await;
        }
        
        Ok(bytes_sent)
    }

    /// Receive data
    pub async fn receive_data(
        &self,
        socket_id: String,
        buffer_size: usize,
        flags: i32,
        process_cap: String,
    ) -> Result<(Vec<u8>, usize), String> {
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(&socket_id) {
            Some(socket) => socket.clone(),
            None => return Err("Invalid socket".to_string()),
        };
        drop(sockets);
        
        // Check capability
        if socket.process_cap != process_cap {
            return Err("Socket access denied".to_string());
        }
        
        // Receive data
        let (data, bytes_received) = socket.recv(buffer_size, flags).await?;
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_bytes_received += bytes_received as u64;
        drop(stats);
        
        // Log audit event
        if self.config.enable_audit {
            self.audit_logger.log_socket_recv(
                process_cap,
                socket_id,
                bytes_received,
                None,
            ).await;
        }
        
        Ok((data, bytes_received))
    }

    /// Close a socket
    pub async fn close_socket(
        &self,
        socket_id: String,
        process_cap: String,
    ) -> Result<(), String> {
        // Remove socket
        let mut sockets = self.sockets.write().await;
        if sockets.remove(&socket_id).is_some() {
            drop(sockets);
            
            // Remove from process socket list
            let mut process_sockets = self.process_sockets.write().await;
            if let Some(sockets) = process_sockets.get_mut(&process_cap) {
                sockets.retain(|id| id != &socket_id);
            }
            drop(process_sockets);
            
            // Update statistics
            let mut stats = self.stats.write().await;
            stats.sockets_destroyed += 1;
            drop(stats);
            
            // Log audit event
            if self.config.enable_audit {
                self.audit_logger.log_socket_close(
                    process_cap,
                    socket_id,
                ).await;
            }
            
            Ok(())
        } else {
            drop(sockets);
            Err("Invalid socket".to_string())
        }
    }

    /// Set socket to non-blocking mode
    pub async fn set_non_blocking(
        &self,
        socket_id: String,
        non_blocking: bool,
        process_cap: String,
    ) -> Result<(), String> {
        // Get socket
        let sockets = self.sockets.read().await;
        let socket = match sockets.get(&socket_id) {
            Some(socket) => socket.clone(),
            None => return Err("Invalid socket".to_string()),
        };
        drop(sockets);
        
        // Check capability
        if socket.process_cap != process_cap {
            return Err("Socket access denied".to_string());
        }
        
        // Set non-blocking mode
        socket.set_non_blocking(non_blocking).await?;
        
        Ok(())
    }

    /// Subscribe to readiness events
    pub async fn subscribe_readiness(
        &self,
        socket_id: String,
        interests: Vec<ReadinessInterest>,
        edge_triggered: bool,
        process_cap: String,
    ) -> Result<String, String> {
        self.readiness_manager.subscribe(
            socket_id,
            interests,
            edge_triggered,
            process_cap,
        ).await
    }

    /// Unsubscribe from readiness events
    pub async fn unsubscribe_readiness(
        &self,
        subscription_id: String,
    ) -> Result<(), String> {
        self.readiness_manager.unsubscribe(&subscription_id).await
    }

    /// Poll for readiness events
    pub async fn poll_readiness(
        &self,
        socket_ids: Vec<String>,
        interests: Vec<ReadinessInterest>,
        timeout: Option<std::time::Duration>,
    ) -> Result<Vec<crate::readiness::ReadinessEventData>, String> {
        self.readiness_manager.poll(socket_ids, interests, timeout).await
    }

    /// Check network capability
    async fn check_network_capability(&self, process_cap: &str) -> bool {
        // Mock capability check - in real implementation would check actual capabilities
        process_cap.contains("net:socket") || process_cap.contains("net:all")
    }

    /// Parse address string
    fn parse_address(&self, address: &str, port: u16) -> Result<std::net::SocketAddr, String> {
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
        
        if address.is_empty() {
            return Ok(std::net::SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port));
        }
        
        if let Ok(ipv4) = address.parse::<Ipv4Addr>() {
            Ok(std::net::SocketAddr::new(ipv4.into(), port))
        } else if let Ok(ipv6) = address.parse::<Ipv6Addr>() {
            Ok(std::net::SocketAddr::new(ipv6.into(), port))
        } else {
            Err(format!("Invalid address: {}", address))
        }
    }

    /// Get broker statistics
    pub async fn get_stats(&self) -> BrokerStats {
        self.stats.read().await.clone()
    }

    /// Get readiness statistics
    pub async fn get_readiness_stats(&self) -> crate::readiness::ReadinessStats {
        self.readiness_manager.get_stats().await
    }

    /// Get audit log
    pub async fn get_audit_log(&self, count: usize) -> Vec<crate::audit::AuditEvent> {
        self.audit_logger.get_recent_events(count).await
    }

    /// Get active sockets
    pub async fn get_active_sockets(&self) -> Vec<String> {
        let sockets = self.sockets.read().await;
        sockets.keys().cloned().collect()
    }

    /// Get sockets for process
    pub async fn get_process_sockets(&self, process_cap: &str) -> Vec<String> {
        let process_sockets = self.process_sockets.read().await;
        process_sockets.get(process_cap).cloned().unwrap_or_default()
    }
}

impl Default for SimpleSocketBroker {
    fn default() -> Self {
        Self::new(SimpleBrokerConfig::default())
    }
}
