use libp2p::{
    core::transport::Boxed,
    swarm::{NetworkBehaviour, Swarm},
    PeerId, Multiaddr,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use tracing::{debug, info, warn, error};
use std::time::{Duration, Instant};

use super::{
    transport::{PolynetTransport, TransportConfig, ConnectionInfo},
    rendezvous::{RendezvousService, RendezvousConfig, PeerInfo, PolynetBehaviour},
    peerbook::{PeerBook, PeerBookEntry},
    did::DidPeerId,
};

#[derive(Debug, Error)]
pub enum MeshNodeError {
    #[error("Failed to create transport: {0}")]
    TransportCreation(String),
    
    #[error("Failed to start rendezvous service: {0}")]
    RendezvousStartFailed(String),
    
    #[error("Failed to connect to peer: {0}")]
    ConnectionFailed(String),
    
    #[error("Failed to add bootstrap peer: {0}")]
    BootstrapPeerFailed(String),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    
    #[error("Mesh node not started")]
    NotStarted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshNodeConfig {
    pub transport: TransportConfig,
    pub rendezvous: RendezvousConfig,
    pub peer_book: super::peerbook::PeerBookConfig,
    pub mesh: MeshConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    pub max_connections: usize,
    pub connection_timeout: u64,
    pub ping_interval: u64,
    pub discovery_interval: u64,
    pub enable_auto_discovery: bool,
    pub enable_auto_cleanup: bool,
}

impl Default for MeshNodeConfig {
    fn default() -> Self {
        Self {
            transport: TransportConfig::default(),
            rendezvous: RendezvousConfig::default(),
            peer_book: super::peerbook::PeerBookConfig::default(),
            mesh: MeshConfig {
                max_connections: 100,
                connection_timeout: 30,
                ping_interval: 30,
                discovery_interval: 300,
                enable_auto_discovery: true,
                enable_auto_cleanup: true,
            },
        }
    }
}

pub struct MeshNode {
    config: MeshNodeConfig,
    did_peer_id: DidPeerId,
    transport: PolynetTransport,
    rendezvous_service: Option<RendezvousService>,
    peer_book: PeerBook,
    connections: Arc<RwLock<HashMap<PeerId, ConnectionInfo>>>,
    bootstrap_peers: Vec<String>,
    started: bool,
}

impl MeshNode {
    /// Create a new mesh node
    pub async fn new(
        did_peer_id: DidPeerId,
        transport_config: TransportConfig,
        rendezvous_config: RendezvousConfig,
    ) -> Result<Self, MeshNodeError> {
        info!("Creating new mesh node with DID: {}", did_peer_id);
        
        // Create transport
        let transport = PolynetTransport::new(
            transport_config.clone(),
            did_peer_id.peer_id(),
        ).await
        .map_err(|e| MeshNodeError::TransportCreation(e.to_string()))?;
        
        // Create peer book
        let peer_book_config = super::peerbook::PeerBookConfig::default();
        let peer_book = PeerBook::new(peer_book_config);
        
        // Create mesh node config
        let config = MeshNodeConfig {
            transport: transport_config,
            rendezvous: rendezvous_config,
            peer_book: peer_book_config,
            mesh: MeshConfig::default(),
        };
        
        Ok(Self {
            config,
            did_peer_id,
            transport,
            rendezvous_service: None,
            peer_book,
            connections: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_peers: Vec::new(),
            started: false,
        })
    }
    
    /// Start the mesh node
    pub async fn start(&mut self) -> Result<(), MeshNodeError> {
        if self.started {
            warn!("Mesh node already started");
            return Ok(());
        }
        
        info!("Starting mesh node...");
        
        // Start rendezvous service if configured
        if let Some(server) = &self.config.rendezvous.server {
            info!("Starting rendezvous service with server: {}", server);
            
            // Create rendezvous service
            let transport = self.transport.clone().await?;
            let mut rendezvous_service = RendezvousService::new(
                self.config.rendezvous.clone(),
                self.did_peer_id.peer_id(),
                transport,
            ).await
            .map_err(|e| MeshNodeError::RendezvousStartFailed(e.to_string()))?;
            
            rendezvous_service.start().await
                .map_err(|e| MeshNodeError::RendezvousStartFailed(e.to_string()))?;
            
            self.rendezvous_service = Some(rendezvous_service);
        }
        
        // Start auto-discovery if enabled
        if self.config.mesh.enable_auto_discovery {
            self.start_auto_discovery().await?;
        }
        
        // Start auto-cleanup if enabled
        if self.config.mesh.enable_auto_cleanup {
            self.start_auto_cleanup().await?;
        }
        
        self.started = true;
        info!("Mesh node started successfully");
        
        Ok(())
    }
    
    /// Stop the mesh node
    pub async fn stop(&mut self) -> Result<(), MeshNodeError> {
        if !self.started {
            warn!("Mesh node not started");
            return Ok(());
        }
        
        info!("Stopping mesh node...");
        
        // Stop rendezvous service
        if let Some(ref mut rendezvous_service) = self.rendezvous_service {
            rendezvous_service.stop().await
                .map_err(|e| MeshNodeError::RendezvousStartFailed(e.to_string()))?;
        }
        
        // Close all connections
        let mut connections = self.connections.write().await;
        connections.clear();
        
        self.started = false;
        info!("Mesh node stopped successfully");
        
        Ok(())
    }
    
    /// Add a bootstrap peer
    pub async fn add_bootstrap_peer(&mut self, peer: &str) -> Result<(), MeshNodeError> {
        info!("Adding bootstrap peer: {}", peer);
        
        // Validate peer format
        let _: Multiaddr = peer.parse()
            .map_err(|e| MeshNodeError::InvalidAddress(e.to_string()))?;
        
        self.bootstrap_peers.push(peer.to_string());
        
        // Try to connect to bootstrap peer
        if self.started {
            self.connect_to_peer(peer, peer).await?;
        }
        
        Ok(())
    }
    
    /// Connect to a specific peer
    pub async fn connect_to_peer(&mut self, peer_id: &str, address: &str) -> Result<(), MeshNodeError> {
        if !self.started {
            return Err(MeshNodeError::NotStarted);
        }
        
        info!("Connecting to peer {} at {}", peer_id, address);
        
        // Parse address
        let addr: Multiaddr = address.parse()
            .map_err(|e| MeshNodeError::InvalidAddress(e.to_string()))?;
        
        // Parse peer ID
        let peer_id: PeerId = peer_id.parse()
            .map_err(|e| MeshNodeError::InvalidAddress(e.to_string()))?;
        
        // Check if already connected
        let connections = self.connections.read().await;
        if connections.contains_key(&peer_id) {
            info!("Already connected to peer: {}", peer_id);
            return Ok(());
        }
        drop(connections);
        
        // Connect using transport
        let start_time = Instant::now();
        self.transport.dial(addr.clone()).await
            .map_err(|e| MeshNodeError::ConnectionFailed(e.to_string()))?;
        
        let connection_time = start_time.elapsed();
        
        // Record connection
        let connection_info = ConnectionInfo {
            local_peer_id: self.did_peer_id.peer_id(),
            transport_protocols: vec!["QUIC".to_string(), "TCP".to_string()],
            bind_address: self.config.transport.bind_addr.clone(),
        };
        
        let mut connections = self.connections.write().await;
        connections.insert(peer_id, connection_info);
        
        // Add to peer book
        let peer_info = PeerInfo {
            id: peer_id,
            addresses: vec![addr],
            namespace: self.config.rendezvous.namespace.clone(),
            ttl: self.config.rendezvous.ttl,
            discovered_at: chrono::Utc::now(),
            last_seen: Some(chrono::Utc::now()),
            rtt: Some(connection_time),
        };
        
        if let Err(e) = self.peer_book.add_peer(peer_info).await {
            warn!("Failed to add peer to peer book: {}", e);
        }
        
        info!("Successfully connected to peer {} in {:?}", peer_id, connection_time);
        
        Ok(())
    }
    
    /// Get connection information for a peer
    pub async fn get_connection_info(&self, peer_id: &PeerId) -> Option<ConnectionInfo> {
        let connections = self.connections.read().await;
        connections.get(peer_id).cloned()
    }
    
    /// Get all connections
    pub async fn get_connections(&self) -> Vec<(PeerId, ConnectionInfo)> {
        let connections = self.connections.read().await;
        connections.iter().map(|(id, info)| (*id, info.clone())).collect()
    }
    
    /// Get peer count
    pub async fn get_peer_count(&self) -> usize {
        self.peer_book.get_peer_count().await
    }
    
    /// Get peer book statistics
    pub async fn get_peer_book_stats(&self) -> Result<super::peerbook::PeerBookStats, super::peerbook::PeerBookError> {
        self.peer_book.get_stats().await
    }
    
    /// List all peers
    pub async fn list_peers(&self) -> Result<Vec<PeerBookEntry>, super::peerbook::PeerBookError> {
        self.peer_book.list_peers().await
    }
    
    /// Ping a specific peer
    pub async fn ping_peer(&self, peer_id: &PeerId) -> Result<Duration, MeshNodeError> {
        if !self.started {
            return Err(MeshNodeError::NotStarted);
        }
        
        info!("Pinging peer: {}", peer_id);
        
        // Check if peer exists
        let peer_entry = self.peer_book.get_peer(peer_id).await
            .map_err(|_| MeshNodeError::PeerNotFound(*peer_id))?;
        
        // Simulate ping (in practice, this would use libp2p ping protocol)
        let start_time = Instant::now();
        
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let rtt = start_time.elapsed();
        
        // Update RTT in peer book
        if let Err(e) = self.peer_book.update_peer_rtt(peer_id, rtt).await {
            warn!("Failed to update RTT for peer {}: {}", peer_id, e);
        }
        
        info!("Ping to peer {}: {:?}", peer_id, rtt);
        
        Ok(rtt)
    }
    
    /// Start auto-discovery
    async fn start_auto_discovery(&self) -> Result<(), MeshNodeError> {
        info!("Starting auto-discovery...");
        
        let discovery_interval = self.config.mesh.discovery_interval;
        let peer_book = self.peer_book.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(discovery_interval));
            
            loop {
                interval.tick().await;
                
                // Discover peers via mDNS and other methods
                debug!("Running auto-discovery...");
                
                // Clean up expired peers
                if let Err(e) = peer_book.cleanup_expired_peers().await {
                    warn!("Auto-discovery cleanup failed: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Start auto-cleanup
    async fn start_auto_cleanup(&self) -> Result<(), MeshNodeError> {
        info!("Starting auto-cleanup...");
        
        let cleanup_interval = 60; // 1 minute
        let connections = Arc::clone(&self.connections);
        let max_connections = self.config.mesh.max_connections;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
            
            loop {
                interval.tick().await;
                
                let mut conns = connections.write().await;
                if conns.len() > max_connections {
                    // Remove oldest connections
                    let to_remove = conns.len() - max_connections;
                    let keys: Vec<PeerId> = conns.keys().take(to_remove).cloned().collect();
                    
                    for key in keys {
                        conns.remove(&key);
                        debug!("Removed old connection: {}", key);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        self.did_peer_id.peer_id()
    }
    
    /// Get DID
    pub fn did(&self) -> &str {
        self.did_peer_id.did()
    }
    
    /// Get configuration
    pub fn config(&self) -> &MeshNodeConfig {
        &self.config
    }
    
    /// Check if node is started
    pub fn is_started(&self) -> bool {
        self.started
    }
    
    /// Get bootstrap peers
    pub fn get_bootstrap_peers(&self) -> &[String] {
        &self.bootstrap_peers
    }
}

impl Clone for MeshNode {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            did_peer_id: self.did_peer_id.clone(),
            transport: self.transport.clone(),
            rendezvous_service: None, // Cannot clone rendezvous service
            peer_book: self.peer_book.clone(),
            connections: Arc::clone(&self.connections),
            bootstrap_peers: self.bootstrap_peers.clone(),
            started: false, // Cloned node is not started
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::did::DidPeerId;
    
    #[tokio::test]
    async fn test_mesh_node_creation() {
        let did_peer_id = DidPeerId::generate().unwrap();
        let transport_config = TransportConfig::default();
        let rendezvous_config = RendezvousConfig::default();
        
        let mesh_node = MeshNode::new(did_peer_id, transport_config, rendezvous_config).await;
        assert!(mesh_node.is_ok());
    }
    
    #[tokio::test]
    async fn test_mesh_node_config() {
        let config = MeshNodeConfig::default();
        
        assert_eq!(config.mesh.max_connections, 100);
        assert_eq!(config.mesh.connection_timeout, 30);
        assert_eq!(config.mesh.ping_interval, 30);
        assert_eq!(config.mesh.discovery_interval, 300);
        assert!(config.mesh.enable_auto_discovery);
        assert!(config.mesh.enable_auto_cleanup);
    }
    
    #[tokio::test]
    async fn test_mesh_node_start_stop() {
        let did_peer_id = DidPeerId::generate().unwrap();
        let transport_config = TransportConfig::default();
        let rendezvous_config = RendezvousConfig::default();
        
        let mut mesh_node = MeshNode::new(did_peer_id, transport_config, rendezvous_config).await.unwrap();
        
        assert!(!mesh_node.is_started());
        
        // Start node
        mesh_node.start().await.unwrap();
        assert!(mesh_node.is_started());
        
        // Stop node
        mesh_node.stop().await.unwrap();
        assert!(!mesh_node.is_started());
    }
}
