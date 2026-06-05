//! Polymera OS Mesh Networking Service
//! 
//! This crate provides a libp2p-based mesh networking foundation for Polymera OS
//! with QUIC transport, rendezvous discovery, and DID-derived peer IDs.

pub mod transport;
pub mod rendezvous;
pub mod did;
pub mod peerbook;
pub mod mesh;
pub mod overlay;
pub mod tests;

// Re-export main types for convenience
pub use transport::{PolynetTransport, TransportConfig, ConnectionInfo};
pub use rendezvous::{RendezvousService, RendezvousConfig, PeerInfo, PolynetBehaviour};
pub use did::DidPeerId;
pub use peerbook::{PeerBook, PeerBookEntry, PeerBookStats};
pub use mesh::{MeshNode, MeshNodeConfig, MeshConfig};
pub use overlay::{OverlayNode, OverlayConfig, OverlayBehaviour, OverlayEvent, OverlayStats};

// Re-export libp2p types that are commonly used
pub use libp2p::{PeerId, Multiaddr};

/// Version of the polynet service
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Protocol version for polynet
pub const PROTOCOL_VERSION: &str = "/polynet/1.0.0";

/// Default namespace for polynet
pub const DEFAULT_NAMESPACE: &str = "polymera-os";

/// Default TTL for peer records (1 hour)
pub const DEFAULT_PEER_TTL: u64 = 3600;

/// Default discovery interval (5 minutes)
pub const DEFAULT_DISCOVERY_INTERVAL: u64 = 300;

/// Default cleanup interval (1 minute)
pub const DEFAULT_CLEANUP_INTERVAL: u64 = 60;

/// Default maximum connections per node
pub const DEFAULT_MAX_CONNECTIONS: usize = 100;

/// Default connection timeout (30 seconds)
pub const DEFAULT_CONNECTION_TIMEOUT: u64 = 30;

/// Default ping interval (30 seconds)
pub const DEFAULT_PING_INTERVAL: u64 = 30;

/// Default maximum peers in peer book
pub const DEFAULT_MAX_PEERS: usize = 1000;

/// Default peer book cleanup interval (5 minutes)
pub const DEFAULT_PEER_BOOK_CLEANUP_INTERVAL: u64 = 300;

/// Default peer book persistence path
pub const DEFAULT_PEER_BOOK_PATH: &str = "data/peerbook.json";

/// Default transport bind address
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0:0";

/// Default transport configuration
pub fn default_transport_config() -> TransportConfig {
    TransportConfig {
        bind_addr: DEFAULT_BIND_ADDR.to_string(),
        enable_quic: true,
        enable_tcp: true,
        enable_noise: true,
        enable_mdns: true,
        connection_timeout: Some(DEFAULT_CONNECTION_TIMEOUT),
        keep_alive_interval: Some(DEFAULT_PING_INTERVAL),
    }
}

/// Default rendezvous configuration
pub fn default_rendezvous_config() -> RendezvousConfig {
    RendezvousConfig {
        server: None,
        namespace: DEFAULT_NAMESPACE.to_string(),
        ttl: DEFAULT_PEER_TTL,
        discovery_interval: Some(DEFAULT_DISCOVERY_INTERVAL),
        max_peers: Some(DEFAULT_MAX_PEERS),
    }
}

/// Default mesh configuration
pub fn default_mesh_config() -> MeshConfig {
    MeshConfig {
        max_connections: DEFAULT_MAX_CONNECTIONS,
        connection_timeout: DEFAULT_CONNECTION_TIMEOUT,
        ping_interval: DEFAULT_PING_INTERVAL,
        discovery_interval: DEFAULT_DISCOVERY_INTERVAL,
        enable_auto_discovery: true,
        enable_auto_cleanup: true,
    }
}

/// Default peer book configuration
pub fn default_peer_book_config() -> peerbook::PeerBookConfig {
    peerbook::PeerBookConfig {
        max_peers: DEFAULT_MAX_PEERS,
        cleanup_interval: DEFAULT_PEER_BOOK_CLEANUP_INTERVAL,
        persistence_enabled: true,
        persistence_path: DEFAULT_PEER_BOOK_PATH.to_string(),
        auto_cleanup: true,
    }
}

/// Default mesh node configuration
pub fn default_mesh_node_config() -> MeshNodeConfig {
    MeshNodeConfig {
        transport: default_transport_config(),
        rendezvous: default_rendezvous_config(),
        peer_book: default_peer_book_config(),
        mesh: default_mesh_config(),
    }
}

/// Create a new mesh node with default configuration
pub async fn create_mesh_node() -> Result<MeshNode, mesh::MeshNodeError> {
    let did_peer_id = DidPeerId::generate()
        .map_err(|e| mesh::MeshNodeError::TransportCreation(e.to_string()))?;
    
    let transport_config = default_transport_config();
    let rendezvous_config = default_rendezvous_config();
    
    MeshNode::new(did_peer_id, transport_config, rendezvous_config).await
}

/// Create a new mesh node with custom configuration
pub async fn create_mesh_node_with_config(
    config: MeshNodeConfig,
) -> Result<MeshNode, mesh::MeshNodeError> {
    let did_peer_id = DidPeerId::generate()
        .map_err(|e| mesh::MeshNodeError::TransportCreation(e.to_string()))?;
    
    MeshNode::new(did_peer_id, config.transport, config.rendezvous).await
}

/// Create a new mesh node with existing DID
pub async fn create_mesh_node_with_did(
    did_key: &str,
) -> Result<MeshNode, mesh::MeshNodeError> {
    let did_peer_id = DidPeerId::from_did_key(did_key)
        .map_err(|e| mesh::MeshNodeError::TransportCreation(e.to_string()))?;
    
    let transport_config = default_transport_config();
    let rendezvous_config = default_rendezvous_config();
    
    MeshNode::new(did_peer_id, transport_config, rendezvous_config).await
}

/// Utility function to parse a multiaddr string
pub fn parse_multiaddr(addr: &str) -> Result<Multiaddr, String> {
    addr.parse().map_err(|e| e.to_string())
}

/// Utility function to parse a peer ID string
pub fn parse_peer_id(peer_id: &str) -> Result<PeerId, String> {
    peer_id.parse().map_err(|e| e.to_string())
}

/// Utility function to format a peer ID for display
pub fn format_peer_id(peer_id: &PeerId) -> String {
    peer_id.to_string()
}

/// Utility function to format a multiaddr for display
pub fn format_multiaddr(addr: &Multiaddr) -> String {
    addr.to_string()
}

/// Utility function to get the peer ID from a multiaddr
pub fn get_peer_id_from_multiaddr(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().find_map(|component| {
        if let libp2p::multiaddr::Protocol::P2p(peer_id) = component {
            Some(peer_id)
        } else {
            None
        }
    })
}

/// Utility function to create a multiaddr with peer ID
pub fn create_multiaddr_with_peer_id(
    protocol: &str,
    address: &str,
    port: u16,
    peer_id: &PeerId,
) -> Result<Multiaddr, String> {
    let addr_str = format!("{}/ip4/{}/tcp/{}/p2p/{}", protocol, address, port, peer_id);
    addr_str.parse().map_err(|e| e.to_string())
}

/// Utility function to validate a multiaddr
pub fn is_valid_multiaddr(addr: &str) -> bool {
    addr.parse::<Multiaddr>().is_ok()
}

/// Utility function to validate a peer ID
pub fn is_valid_peer_id(peer_id: &str) -> bool {
    peer_id.parse::<PeerId>().is_ok()
}

/// Utility function to generate a random peer ID
pub fn generate_random_peer_id() -> PeerId {
    PeerId::random()
}

/// Utility function to generate a random DID
pub fn generate_random_did() -> Result<String, did::DidError> {
    let did_peer_id = DidPeerId::generate()?;
    Ok(did_peer_id.did().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_configs() {
        let transport_config = default_transport_config();
        assert_eq!(transport_config.bind_addr, DEFAULT_BIND_ADDR);
        assert!(transport_config.enable_quic);
        assert!(transport_config.enable_tcp);
        
        let rendezvous_config = default_rendezvous_config();
        assert_eq!(rendezvous_config.namespace, DEFAULT_NAMESPACE);
        assert_eq!(rendezvous_config.ttl, DEFAULT_PEER_TTL);
        
        let mesh_config = default_mesh_config();
        assert_eq!(mesh_config.max_connections, DEFAULT_MAX_CONNECTIONS);
        assert_eq!(mesh_config.connection_timeout, DEFAULT_CONNECTION_TIMEOUT);
        
        let peer_book_config = default_peer_book_config();
        assert_eq!(peer_book_config.max_peers, DEFAULT_MAX_PEERS);
        assert_eq!(peer_book_config.persistence_path, DEFAULT_PEER_BOOK_PATH);
    }
    
    #[test]
    fn test_utility_functions() {
        let peer_id = generate_random_peer_id();
        let peer_id_str = format_peer_id(&peer_id);
        assert!(is_valid_peer_id(&peer_id_str));
        
        let addr = "/ip4/127.0.0.1/tcp/8080".parse::<Multiaddr>().unwrap();
        let addr_str = format_multiaddr(&addr);
        assert!(is_valid_multiaddr(&addr_str));
        
        let parsed_addr = parse_multiaddr(&addr_str).unwrap();
        assert_eq!(addr, parsed_addr);
        
        let parsed_peer_id = parse_peer_id(&peer_id_str).unwrap();
        assert_eq!(peer_id, parsed_peer_id);
    }
    
    #[test]
    fn test_multiaddr_utilities() {
        let peer_id = generate_random_peer_id();
        let addr = create_multiaddr_with_peer_id("http", "127.0.0.1", 8080, &peer_id).unwrap();
        
        let extracted_peer_id = get_peer_id_from_multiaddr(&addr).unwrap();
        assert_eq!(peer_id, extracted_peer_id);
    }
    
    #[tokio::test]
    async fn test_mesh_node_creation() {
        let mesh_node = create_mesh_node().await;
        assert!(mesh_node.is_ok());
        
        let mesh_node = create_mesh_node_with_config(default_mesh_node_config()).await;
        assert!(mesh_node.is_ok());
    }
}
