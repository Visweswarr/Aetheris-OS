//! Network namespaces for POSIX networking
//! 
//! This module provides network namespace functionality for process isolation
//! and policy enforcement in the POSIX networking subsystem.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Instant;
use serde::{Deserialize, Serialize};
use cidr_utils::cidr::IpCidr;

use crate::policy::NetworkPolicy;

/// Network namespace class
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NamespaceClass {
    /// No network access
    None,
    /// Local network only (loopback, localhost)
    Local,
    /// Mesh network (libp2p overlay)
    Mesh,
    /// Wide area network (full internet access)
    Wan,
}

impl Default for NamespaceClass {
    fn default() -> Self {
        Self::None
    }
}

/// Network namespace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceConfig {
    /// Namespace class
    pub class: NamespaceClass,
    /// Allowed hosts (CIDR blocks or hostnames)
    pub allowed_hosts: Vec<String>,
    /// Allowed ports (ranges)
    pub allowed_ports: Vec<PortRange>,
    /// Allowed protocols
    pub allowed_protocols: Vec<Protocol>,
    /// Egress bandwidth budget (bytes per second)
    pub egress_budget_bps: Option<u64>,
    /// Connection rate limit (connections per second)
    pub connection_rate: Option<u32>,
    /// DNS policy
    pub dns_policy: DNSPolicy,
    /// Enable TLS
    pub enable_tls: bool,
    /// Enable QUIC
    pub enable_quic: bool,
    /// Enable libp2p overlay
    pub enable_libp2p: bool,
    /// Custom routing rules
    pub routing_rules: Vec<RoutingRule>,
}

/// Port range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    /// Start port (inclusive)
    pub start: u16,
    /// End port (inclusive)
    pub end: u16,
}

impl PortRange {
    /// Create a single port range
    pub fn single(port: u16) -> Self {
        Self {
            start: port,
            end: port,
        }
    }

    /// Create a port range
    pub fn range(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    /// Check if port is in range
    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }
}

/// Protocol specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    /// TCP
    Tcp,
    /// UDP
    Udp,
    /// QUIC
    Quic,
    /// ICMP
    Icmp,
    /// Custom protocol
    Custom(String),
}

/// DNS policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DNSPolicy {
    /// Allowed DNS servers
    pub allowed_servers: Vec<IpAddr>,
    /// Blocked domains (wildcards supported)
    pub blocked_domains: Vec<String>,
    /// Allowed domains (wildcards supported)
    pub allowed_domains: Vec<String>,
    /// Enable DNS over HTTPS
    pub enable_doh: bool,
    /// Enable DNS over TLS
    pub enable_dot: bool,
}

impl Default for DNSPolicy {
    fn default() -> Self {
        Self {
            allowed_servers: vec![
                IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
                IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)),
            ],
            blocked_domains: Vec::new(),
            allowed_domains: Vec::new(),
            enable_doh: false,
            enable_dot: false,
        }
    }
}

/// Routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Destination CIDR
    pub destination: IpCidr,
    /// Gateway address
    pub gateway: Option<IpAddr>,
    /// Interface name
    pub interface: Option<String>,
    /// Route priority
    pub priority: u32,
    /// Route metric
    pub metric: u32,
}

/// Network namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNamespace {
    /// Unique namespace identifier
    pub id: String,
    /// Namespace name
    pub name: String,
    /// Namespace class
    pub class: NamespaceClass,
    /// Creation timestamp
    pub created_at: Instant,
    /// Processes in this namespace
    pub processes: Vec<String>,
    /// Sockets in this namespace
    pub sockets: Vec<String>,
    /// Network policy
    pub policy: NetworkPolicy,
    /// Namespace configuration
    pub config: NamespaceConfig,
    /// Statistics
    pub stats: NamespaceStats,
}

/// Namespace statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamespaceStats {
    /// Number of active processes
    pub active_processes: usize,
    /// Number of active sockets
    pub active_sockets: usize,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total connections established
    pub connections_established: u64,
    /// Total connections rejected
    pub connections_rejected: u64,
    /// Policy decisions made
    pub policy_decisions: u64,
    /// Policy denials
    pub policy_denials: u64,
}

impl NetworkNamespace {
    /// Create a new network namespace
    pub fn new(
        id: String,
        name: String,
        class: NamespaceClass,
        config: NamespaceConfig,
    ) -> Self {
        Self {
            id,
            name,
            class,
            created_at: Instant::now(),
            processes: Vec::new(),
            sockets: Vec::new(),
            policy: NetworkPolicy::default(),
            config,
            stats: NamespaceStats::default(),
        }
    }

    /// Add a process to the namespace
    pub fn add_process(&mut self, process_cap: String) -> Result<(), String> {
        if self.processes.contains(&process_cap) {
            return Err("Process already in namespace".to_string());
        }

        self.processes.push(process_cap);
        self.stats.active_processes = self.processes.len();
        Ok(())
    }

    /// Remove a process from the namespace
    pub fn remove_process(&mut self, process_cap: &str) -> Result<(), String> {
        if let Some(pos) = self.processes.iter().position(|p| p == process_cap) {
            self.processes.remove(pos);
            self.stats.active_processes = self.processes.len();
            Ok(())
        } else {
            Err("Process not in namespace".to_string())
        }
    }

    /// Add a socket to the namespace
    pub fn add_socket(&mut self, socket_id: String) -> Result<(), String> {
        if self.sockets.contains(&socket_id) {
            return Err("Socket already in namespace".to_string());
        }

        self.sockets.push(socket_id);
        self.stats.active_sockets = self.sockets.len();
        Ok(())
    }

    /// Remove a socket from the namespace
    pub fn remove_socket(&mut self, socket_id: &str) -> Result<(), String> {
        if let Some(pos) = self.sockets.iter().position(|s| s == socket_id) {
            self.sockets.remove(pos);
            self.stats.active_sockets = self.sockets.len();
            Ok(())
        } else {
            Err("Socket not in namespace".to_string())
        }
    }

    /// Check if an address is allowed in this namespace
    pub fn is_address_allowed(&self, address: &SocketAddr) -> bool {
        match self.class {
            NamespaceClass::None => false,
            NamespaceClass::Local => {
                // Allow loopback addresses only
                match address.ip() {
                    IpAddr::V4(ip) => ip.is_loopback(),
                    IpAddr::V6(ip) => ip.is_loopback(),
                }
            },
            NamespaceClass::Mesh => {
                // Allow local addresses and mesh overlay addresses
                match address.ip() {
                    IpAddr::V4(ip) => ip.is_loopback() || self.is_mesh_address(ip),
                    IpAddr::V6(ip) => ip.is_loopback() || self.is_mesh_address_v6(ip),
                }
            },
            NamespaceClass::Wan => {
                // Check against allowed hosts list
                self.is_host_allowed(&address.ip()) && self.is_port_allowed(address.port())
            },
        }
    }

    /// Check if a host is allowed
    fn is_host_allowed(&self, host: &IpAddr) -> bool {
        // Check against allowed hosts list
        for allowed_host in &self.config.allowed_hosts {
            if let Ok(cidr) = allowed_host.parse::<IpCidr>() {
                if cidr.contains(*host) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a port is allowed
    fn is_port_allowed(&self, port: u16) -> bool {
        // If no port restrictions, allow all
        if self.config.allowed_ports.is_empty() {
            return true;
        }

        // Check against allowed ports
        self.config.allowed_ports.iter().any(|range| range.contains(port))
    }

    /// Check if a protocol is allowed
    pub fn is_protocol_allowed(&self, protocol: &Protocol) -> bool {
        self.config.allowed_protocols.contains(protocol)
    }

    /// Check if DNS resolution is allowed for a domain
    pub fn is_dns_allowed(&self, domain: &str) -> bool {
        // Check blocked domains first
        for blocked in &self.config.dns_policy.blocked_domains {
            if self.matches_wildcard(domain, blocked) {
                return false;
            }
        }

        // If allowed domains is empty, allow all (except blocked)
        if self.config.dns_policy.allowed_domains.is_empty() {
            return true;
        }

        // Check allowed domains
        self.config.dns_policy.allowed_domains.iter().any(|allowed| {
            self.matches_wildcard(domain, allowed)
        })
    }

    /// Check if a domain matches a wildcard pattern
    fn matches_wildcard(&self, domain: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            return domain.ends_with(suffix);
        }

        if pattern.ends_with(".*") {
            let prefix = &pattern[..pattern.len() - 2];
            return domain.starts_with(prefix);
        }

        domain == pattern
    }

    /// Check if mesh address (mock implementation)
    fn is_mesh_address(&self, ip: Ipv4Addr) -> bool {
        // Mock implementation - in real implementation would check against mesh overlay
        ip.octets()[0] == 10 && ip.octets()[1] == 0
    }

    /// Check if mesh address v6 (mock implementation)
    fn is_mesh_address_v6(&self, ip: Ipv6Addr) -> bool {
        // Mock implementation - in real implementation would check against mesh overlay
        ip.octets()[0] == 0xfd && ip.octets()[1] == 0x00
    }

    /// Update statistics
    pub fn update_stats(&mut self, event: NamespaceEvent) {
        match event {
            NamespaceEvent::BytesSent(bytes) => {
                self.stats.bytes_sent += bytes;
            },
            NamespaceEvent::BytesReceived(bytes) => {
                self.stats.bytes_received += bytes;
            },
            NamespaceEvent::ConnectionEstablished => {
                self.stats.connections_established += 1;
            },
            NamespaceEvent::ConnectionRejected => {
                self.stats.connections_rejected += 1;
            },
            NamespaceEvent::PolicyDecision => {
                self.stats.policy_decisions += 1;
            },
            NamespaceEvent::PolicyDenial => {
                self.stats.policy_denials += 1;
            },
        }
    }

    /// Get namespace age
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Check if namespace is empty
    pub fn is_empty(&self) -> bool {
        self.processes.is_empty() && self.sockets.is_empty()
    }

    /// Get namespace summary
    pub fn get_summary(&self) -> NamespaceSummary {
        NamespaceSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            class: self.class.clone(),
            created_at: self.created_at,
            age: self.age(),
            active_processes: self.stats.active_processes,
            active_sockets: self.stats.active_sockets,
            bytes_sent: self.stats.bytes_sent,
            bytes_received: self.stats.bytes_received,
            connections_established: self.stats.connections_established,
            connections_rejected: self.stats.connections_rejected,
            policy_decisions: self.stats.policy_decisions,
            policy_denials: self.stats.policy_denials,
        }
    }
}

/// Namespace event for statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamespaceEvent {
    /// Bytes sent
    BytesSent(u64),
    /// Bytes received
    BytesReceived(u64),
    /// Connection established
    ConnectionEstablished,
    /// Connection rejected
    ConnectionRejected,
    /// Policy decision made
    PolicyDecision,
    /// Policy denial
    PolicyDenial,
}

/// Namespace summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceSummary {
    pub id: String,
    pub name: String,
    pub class: NamespaceClass,
    pub created_at: Instant,
    pub age: std::time::Duration,
    pub active_processes: usize,
    pub active_sockets: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_established: u64,
    pub connections_rejected: u64,
    pub policy_decisions: u64,
    pub policy_denials: u64,
}

/// Namespace manager
pub struct NamespaceManager {
    /// Active namespaces
    namespaces: HashMap<String, NetworkNamespace>,
    /// Process to namespace mapping
    process_namespaces: HashMap<String, String>,
    /// Default namespace
    default_namespace: String,
}

impl NamespaceManager {
    /// Create a new namespace manager
    pub fn new() -> Self {
        let mut manager = Self {
            namespaces: HashMap::new(),
            process_namespaces: HashMap::new(),
            default_namespace: "default".to_string(),
        };

        // Create default namespace
        let default_config = NamespaceConfig {
            class: NamespaceClass::None,
            allowed_hosts: Vec::new(),
            allowed_ports: Vec::new(),
            allowed_protocols: Vec::new(),
            egress_budget_bps: None,
            connection_rate: None,
            dns_policy: DNSPolicy::default(),
            enable_tls: false,
            enable_quic: false,
            enable_libp2p: false,
            routing_rules: Vec::new(),
        };

        let default_ns = NetworkNamespace::new(
            "default".to_string(),
            "default".to_string(),
            NamespaceClass::None,
            default_config,
        );

        manager.namespaces.insert("default".to_string(), default_ns);
        manager
    }

    /// Create a new namespace
    pub fn create_namespace(
        &mut self,
        id: String,
        name: String,
        class: NamespaceClass,
        config: NamespaceConfig,
    ) -> Result<(), String> {
        if self.namespaces.contains_key(&id) {
            return Err("Namespace already exists".to_string());
        }

        let namespace = NetworkNamespace::new(id.clone(), name, class, config);
        self.namespaces.insert(id, namespace);
        Ok(())
    }

    /// Remove a namespace
    pub fn remove_namespace(&mut self, id: &str) -> Result<(), String> {
        if id == &self.default_namespace {
            return Err("Cannot remove default namespace".to_string());
        }

        if let Some(namespace) = self.namespaces.get(id) {
            if !namespace.is_empty() {
                return Err("Namespace not empty".to_string());
            }
        }

        self.namespaces.remove(id);
        Ok(())
    }

    /// Get a namespace by ID
    pub fn get_namespace(&self, id: &str) -> Option<&NetworkNamespace> {
        self.namespaces.get(id)
    }

    /// Get a mutable namespace by ID
    pub fn get_namespace_mut(&mut self, id: &str) -> Option<&mut NetworkNamespace> {
        self.namespaces.get_mut(id)
    }

    /// Set process namespace
    pub fn set_process_namespace(
        &mut self,
        process_cap: String,
        namespace_id: String,
    ) -> Result<(), String> {
        if !self.namespaces.contains_key(&namespace_id) {
            return Err("Namespace not found".to_string());
        }

        // Remove process from old namespace
        if let Some(old_namespace_id) = self.process_namespaces.get(&process_cap) {
            if let Some(namespace) = self.namespaces.get_mut(old_namespace_id) {
                let _ = namespace.remove_process(&process_cap);
            }
        }

        // Add process to new namespace
        if let Some(namespace) = self.namespaces.get_mut(&namespace_id) {
            namespace.add_process(process_cap.clone())?;
        }

        self.process_namespaces.insert(process_cap, namespace_id);
        Ok(())
    }

    /// Get process namespace
    pub fn get_process_namespace(&self, process_cap: &str) -> Option<&str> {
        self.process_namespaces.get(process_cap).map(|s| s.as_str())
    }

    /// Get all namespaces
    pub fn get_all_namespaces(&self) -> &HashMap<String, NetworkNamespace> {
        &self.namespaces
    }

    /// Get namespace statistics
    pub fn get_namespace_stats(&self, id: &str) -> Option<&NamespaceStats> {
        self.namespaces.get(id).map(|ns| &ns.stats)
    }

    /// Get all namespace summaries
    pub fn get_namespace_summaries(&self) -> Vec<NamespaceSummary> {
        self.namespaces.values().map(|ns| ns.get_summary()).collect()
    }

    /// Check if address is allowed for process
    pub fn is_address_allowed_for_process(
        &self,
        process_cap: &str,
        address: &SocketAddr,
    ) -> bool {
        let namespace_id = self.get_process_namespace(process_cap)
            .unwrap_or(&self.default_namespace);

        if let Some(namespace) = self.namespaces.get(namespace_id) {
            namespace.is_address_allowed(address)
        } else {
            false
        }
    }

    /// Check if protocol is allowed for process
    pub fn is_protocol_allowed_for_process(
        &self,
        process_cap: &str,
        protocol: &Protocol,
    ) -> bool {
        let namespace_id = self.get_process_namespace(process_cap)
            .unwrap_or(&self.default_namespace);

        if let Some(namespace) = self.namespaces.get(namespace_id) {
            namespace.is_protocol_allowed(protocol)
        } else {
            false
        }
    }

    /// Check if DNS is allowed for process
    pub fn is_dns_allowed_for_process(
        &self,
        process_cap: &str,
        domain: &str,
    ) -> bool {
        let namespace_id = self.get_process_namespace(process_cap)
            .unwrap_or(&self.default_namespace);

        if let Some(namespace) = self.namespaces.get(namespace_id) {
            namespace.is_dns_allowed(domain)
        } else {
            false
        }
    }
}

impl Default for NamespaceManager {
    fn default() -> Self {
        Self::new()
    }
}
