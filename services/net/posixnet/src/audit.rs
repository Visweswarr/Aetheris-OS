//! Audit logging for POSIX networking
//! 
//! This module provides audit logging for all network operations.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Audit event levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AuditLevel {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event level
    pub level: AuditLevel,
    /// Event message
    pub message: String,
    /// Process capability
    pub process_cap: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Additional data
    pub data: Option<serde_json::Value>,
}

/// Audit logger
pub struct AuditLogger {
    /// Event buffer
    events: Arc<RwLock<VecDeque<AuditEvent>>>,
    /// Maximum buffer size
    max_buffer_size: usize,
    /// Statistics
    stats: Arc<RwLock<AuditStats>>,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditStats {
    /// Total events logged
    pub total_events: u64,
    /// Events by level
    pub events_by_level: std::collections::HashMap<String, u64>,
    /// Events dropped due to buffer overflow
    pub events_dropped: u64,
    /// Last cleanup time
    pub last_cleanup: Option<Instant>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::new())),
            max_buffer_size: 10000,
            stats: Arc::new(RwLock::new(AuditStats::default())),
        }
    }

    /// Create audit logger with custom buffer size
    pub fn with_buffer_size(max_buffer_size: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::new())),
            max_buffer_size,
            stats: Arc::new(RwLock::new(AuditStats::default())),
        }
    }

    /// Log an audit event
    pub async fn log_event(&self, event: AuditEvent) {
        let mut events = self.events.write().await;
        let mut stats = self.stats.write().await;
        
        // Add event to buffer
        if events.len() >= self.max_buffer_size {
            // Remove oldest event
            events.pop_front();
            stats.events_dropped += 1;
        }
        
        events.push_back(event.clone());
        
        // Update statistics
        stats.total_events += 1;
        let level_key = format!("{:?}", event.level);
        *stats.events_by_level.entry(level_key).or_insert(0) += 1;
    }

    /// Log socket creation
    pub async fn log_socket_creation(
        &self,
        process_cap: String,
        socket_id: String,
        family: String,
        socket_type: String,
        protocol: i32,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Info,
            message: "Socket created".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "socket_id": socket_id,
                "family": family,
                "type": socket_type,
                "protocol": protocol
            })),
        }).await;
    }

    /// Log socket bind
    pub async fn log_socket_bind(
        &self,
        process_cap: String,
        socket_id: String,
        address: String,
        port: u16,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Info,
            message: "Socket bound".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "socket_id": socket_id,
                "address": address,
                "port": port
            })),
        }).await;
    }

    /// Log socket listen
    pub async fn log_socket_listen(
        &self,
        process_cap: String,
        socket_id: String,
        backlog: i32,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Info,
            message: "Socket listening".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "socket_id": socket_id,
                "backlog": backlog
            })),
        }).await;
    }

    /// Log socket accept
    pub async fn log_socket_accept(
        &self,
        process_cap: String,
        socket_id: String,
        client_address: String,
        new_socket_id: String,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Info,
            message: "Connection accepted".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "socket_id": socket_id,
                "client_address": client_address,
                "new_socket_id": new_socket_id
            })),
        }).await;
    }

    /// Log socket connect
    pub async fn log_socket_connect(
        &self,
        process_cap: String,
        socket_id: String,
        address: String,
        port: u16,
        success: bool,
    ) {
        self.log_event(AuditEvent {
            level: if success { AuditLevel::Info } else { AuditLevel::Warning },
            message: if success { "Socket connected".to_string() } else { "Socket connection failed".to_string() },
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "socket_id": socket_id,
                "address": address,
                "port": port,
                "success": success
            })),
        }).await;
    }

    /// Log socket send
    pub async fn log_socket_send(
        &self,
        process_cap: String,
        socket_id: String,
        bytes_sent: usize,
        target_address: Option<String>,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Info,
            message: "Data sent".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "socket_id": socket_id,
                "bytes_sent": bytes_sent,
                "target_address": target_address
            })),
        }).await;
    }

    /// Log socket receive
    pub async fn log_socket_recv(
        &self,
        process_cap: String,
        socket_id: String,
        bytes_received: usize,
        source_address: Option<String>,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Info,
            message: "Data received".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "socket_id": socket_id,
                "bytes_received": bytes_received,
                "source_address": source_address
            })),
        }).await;
    }

    /// Log socket close
    pub async fn log_socket_close(
        &self,
        process_cap: String,
        socket_id: String,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Info,
            message: "Socket closed".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "socket_id": socket_id
            })),
        }).await;
    }

    /// Log capability denial
    pub async fn log_capability_denial(
        &self,
        process_cap: String,
        required_capability: String,
        operation: String,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Warning,
            message: "Capability denied".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "required_capability": required_capability,
                "operation": operation
            })),
        }).await;
    }

    /// Log policy violation
    pub async fn log_policy_violation(
        &self,
        process_cap: String,
        policy_rule: String,
        operation: String,
        reason: String,
    ) {
        self.log_event(AuditEvent {
            level: AuditLevel::Warning,
            message: "Policy violation".to_string(),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "policy_rule": policy_rule,
                "operation": operation,
                "reason": reason
            })),
        }).await;
    }

    /// Log security event
    pub async fn log_security_event(
        &self,
        process_cap: String,
        event_type: String,
        description: String,
        severity: AuditLevel,
    ) {
        self.log_event(AuditEvent {
            level: severity,
            message: format!("Security event: {}", event_type),
            process_cap,
            timestamp: Instant::now(),
            data: Some(serde_json::json!({
                "event_type": event_type,
                "description": description,
                "severity": format!("{:?}", severity)
            })),
        }).await;
    }

    /// Get recent events
    pub async fn get_recent_events(&self, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(count).cloned().collect()
    }

    /// Get events by level
    pub async fn get_events_by_level(&self, level: AuditLevel, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter()
            .rev()
            .filter(|event| event.level == level)
            .take(count)
            .cloned()
            .collect()
    }

    /// Get events by process
    pub async fn get_events_by_process(&self, process_cap: &str, count: usize) -> Vec<AuditEvent> {
        let events = self.events.read().await;
        events.iter()
            .rev()
            .filter(|event| event.process_cap == process_cap)
            .take(count)
            .cloned()
            .collect()
    }

    /// Get audit statistics
    pub async fn get_stats(&self) -> AuditStats {
        self.stats.read().await.clone()
    }

    /// Clear audit log
    pub async fn clear_log(&self) {
        let mut events = self.events.write().await;
        events.clear();
        
        let mut stats = self.stats.write().await;
        stats.total_events = 0;
        stats.events_by_level.clear();
        stats.events_dropped = 0;
    }

    /// Cleanup old events
    pub async fn cleanup_old_events(&self, max_age: Duration) -> usize {
        let now = Instant::now();
        let mut events = self.events.write().await;
        let mut removed_count = 0;
        
        while let Some(front) = events.front() {
            if now.duration_since(front.timestamp) > max_age {
                events.pop_front();
                removed_count += 1;
            } else {
                break;
            }
        }
        
        let mut stats = self.stats.write().await;
        stats.last_cleanup = Some(now);
        
        removed_count
    }

    /// Export audit log to JSON
    pub async fn export_to_json(&self) -> Result<String, String> {
        let events = self.events.read().await;
        let events_vec: Vec<&AuditEvent> = events.iter().collect();
        
        serde_json::to_string_pretty(&events_vec)
            .map_err(|e| format!("Failed to serialize audit log: {}", e))
    }

    /// Export audit log to CSV
    pub async fn export_to_csv(&self) -> Result<String, String> {
        let events = self.events.read().await;
        let mut csv = String::new();
        
        // CSV header
        csv.push_str("timestamp,level,process_cap,message,data\n");
        
        // CSV rows
        for event in events.iter() {
            let timestamp = event.timestamp.elapsed().as_secs();
            let level = format!("{:?}", event.level);
            let data = event.data.as_ref()
                .map(|d| d.to_string().replace('\n', " ").replace(',', ";"))
                .unwrap_or_default();
            
            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                timestamp,
                level,
                event.process_cap,
                event.message.replace(',', ";"),
                data
            ));
        }
        
        Ok(csv)
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
