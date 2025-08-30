use libp2p::{
    core::upgrade,
    identity, noise, rendezvous, swarm::{NetworkBehaviour, Swarm},
    PeerId, Multiaddr,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn, error};

#[derive(Debug, Error)]
pub enum RendezvousError {
    #[error("Failed to create rendezvous client: {0}")]
    ClientCreation(String),
    
    #[error("Failed to register with rendezvous server: {0}")]
    RegistrationFailed(String),
    
    #[error("Failed to discover peers: {0}")]
    DiscoveryFailed(String),
    
    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),
    
    #[error("Rendezvous server not configured")]
    NoServerConfigured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendezvousConfig {
    pub server: Option<String>,
    pub namespace: String,
    pub ttl: u64,
    pub discovery_interval: Option<u64>,
    pub max_peers: Option<usize>,
}

impl Default for RendezvousConfig {
    fn default() -> Self {
        Self {
            server: None,
            namespace: "polymera-os".to_string(),
            ttl: 3600, // 1 hour
            discovery_interval: Some(300), // 5 minutes
            max_peers: Some(100),
        }
    }
}

#[derive(NetworkBehaviour)]
pub struct PolynetBehaviour {
    pub rendezvous: rendezvous::client::Behaviour,
    pub identify: libp2p::identify::Behaviour,
    pub ping: libp2p::ping::Behaviour,
    pub mdns: libp2p::mdns::tokio::Behaviour,
}

impl PolynetBehaviour {
    pub fn new(local_peer_id: PeerId, config: &RendezvousConfig) -> Self {
        let rendezvous = rendezvous::client::Behaviour::new(
            identity::Keypair::generate(),
        );
        
        let identify = libp2p::identify::Behaviour::new(
            libp2p::identify::Config::new(
                "/polynet/1.0.0".to_string(),
                local_peer_id,
            )
        );
        
        let ping = libp2p::ping::Behaviour::new(
            libp2p::ping::Config::default()
                .with_interval(std::time::Duration::from_secs(30))
                .with_timeout(std::time::Duration::from_secs(10)),
        );
        
        let mdns = libp2p::mdns::tokio::Behaviour::new(
            libp2p::mdns::Config::default(),
        ).unwrap();
        
        Self {
            rendezvous,
            identify,
            ping,
            mdns,
        }
    }
}

pub struct RendezvousService {
    config: RendezvousConfig,
    swarm: Swarm<PolynetBehaviour>,
    discovered_peers: HashMap<PeerId, PeerInfo>,
    local_peer_id: PeerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub namespace: String,
    pub ttl: u64,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub rtt: Option<std::time::Duration>,
}

impl RendezvousService {
    pub async fn new(
        config: RendezvousConfig,
        local_peer_id: PeerId,
        transport: libp2p::core::transport::Boxed<(PeerId, libp2p::Stream)>,
    ) -> Result<Self, RendezvousError> {
        info!("Creating rendezvous service with config: {:?}", config);
        
        let behaviour = PolynetBehaviour::new(local_peer_id, &config);
        
        let mut swarm = Swarm::new(transport, behaviour, local_peer_id);
        
        // Listen on all interfaces
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())?;
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())?;
        
        Ok(Self {
            config,
            swarm,
            discovered_peers: HashMap::new(),
            local_peer_id,
        })
    }
    
    pub async fn start(&mut self) -> Result<(), RendezvousError> {
        info!("Starting rendezvous service...");
        
        // Register with rendezvous server if configured
        if let Some(server) = &self.config.server {
            self.register_with_server(server).await?;
        }
        
        // Start discovery loop
        self.start_discovery_loop().await?;
        
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<(), RendezvousError> {
        info!("Stopping rendezvous service...");
        
        // Unregister from rendezvous server if configured
        if let Some(server) = &self.config.server {
            self.unregister_from_server(server).await?;
        }
        
        Ok(())
    }
    
    async fn register_with_server(&mut self, server: &str) -> Result<(), RendezvousError> {
        info!("Registering with rendezvous server: {}", server);
        
        let server_addr: Multiaddr = server.parse()
            .map_err(|e| RendezvousError::InvalidNamespace(e.to_string()))?;
        
        let server_peer_id = self.get_peer_id_from_address(&server_addr)
            .ok_or_else(|| RendezvousError::InvalidNamespace("No peer ID in address".to_string()))?;
        
        // Register our peer information
        let record = rendezvous::Record {
            namespace: self.config.namespace.clone(),
            peer_id: self.local_peer_id,
            addresses: vec![], // Will be filled by identify protocol
            ttl: self.config.ttl,
        };
        
        self.swarm.behaviour_mut().rendezvous.register(
            Some(server_peer_id),
            record,
        );
        
        info!("Registration request sent to rendezvous server");
        Ok(())
    }
    
    async fn unregister_from_server(&mut self, server: &str) -> Result<(), RendezvousError> {
        info!("Unregistering from rendezvous server: {}", server);
        
        let server_addr: Multiaddr = server.parse()
            .map_err(|e| RendezvousError::InvalidNamespace(e.to_string()))?;
        
        let server_peer_id = self.get_peer_id_from_address(&server_addr)
            .ok_or_else(|| RendezvousError::InvalidNamespace("No peer ID in address".to_string()))?;
        
        // Unregister our peer information
        let record = rendezvous::Record {
            namespace: self.config.namespace.clone(),
            peer_id: self.local_peer_id,
            addresses: vec![],
            ttl: 0, // Immediate expiration
        };
        
        self.swarm.behaviour_mut().rendezvous.register(
            Some(server_peer_id),
            record,
        );
        
        info!("Unregistration request sent to rendezvous server");
        Ok(())
    }
    
    async fn start_discovery_loop(&mut self) -> Result<(), RendezvousError> {
        info!("Starting peer discovery loop...");
        
        let discovery_interval = self.config.discovery_interval.unwrap_or(300);
        
        // Start discovery timer
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(discovery_interval)
        );
        
        loop {
            interval.tick().await;
            
            // Discover peers from rendezvous server
            if let Some(server) = &self.config.server {
                if let Err(e) = self.discover_peers_from_server(server).await {
                    warn!("Failed to discover peers from server: {}", e);
                }
            }
            
            // Discover peers via mDNS
            self.discover_peers_via_mdns().await;
            
            // Clean up expired peers
            self.cleanup_expired_peers().await;
        }
    }
    
    async fn discover_peers_from_server(&mut self, server: &str) -> Result<(), RendezvousError> {
        let server_addr: Multiaddr = server.parse()
            .map_err(|e| RendezvousError::InvalidNamespace(e.to_string()))?;
        
        let server_peer_id = self.get_peer_id_from_address(&server_addr)
            .ok_or_else(|| RendezvousError::InvalidNamespace("No peer ID in address".to_string()))?;
        
        // Discover peers in our namespace
        self.swarm.behaviour_mut().rendezvous.discover(
            Some(server_peer_id),
            None,
            Some(self.config.namespace.clone()),
            None,
        );
        
        Ok(())
    }
    
    async fn discover_peers_via_mdns(&mut self) {
        // mDNS discovery is handled automatically by the behaviour
        // We just need to process the discovered peers
        debug!("mDNS discovery active");
    }
    
    async fn cleanup_expired_peers(&mut self) {
        let now = chrono::Utc::now();
        let mut expired_peers = Vec::new();
        
        for (peer_id, peer_info) in &self.discovered_peers {
            let expiration_time = peer_info.discovered_at + chrono::Duration::seconds(peer_info.ttl as i64);
            if now > expiration_time {
                expired_peers.push(*peer_id);
            }
        }
        
        for peer_id in expired_peers {
            self.discovered_peers.remove(&peer_id);
            debug!("Removed expired peer: {}", peer_id);
        }
    }
    
    fn get_peer_id_from_address(&self, addr: &Multiaddr) -> Option<PeerId> {
        addr.iter().find_map(|component| {
            if let libp2p::multiaddr::Protocol::P2p(peer_id) = component {
                Some(peer_id)
            } else {
                None
            }
        })
    }
    
    pub fn add_discovered_peer(&mut self, peer_info: PeerInfo) {
        self.discovered_peers.insert(peer_info.id, peer_info);
        info!("Added discovered peer: {}", peer_info.id);
    }
    
    pub fn get_discovered_peers(&self) -> Vec<PeerInfo> {
        self.discovered_peers.values().cloned().collect()
    }
    
    pub fn get_peer_info(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.discovered_peers.get(peer_id)
    }
    
    pub fn update_peer_rtt(&mut self, peer_id: &PeerId, rtt: std::time::Duration) {
        if let Some(peer_info) = self.discovered_peers.get_mut(peer_id) {
            peer_info.rtt = Some(rtt);
            peer_info.last_seen = Some(chrono::Utc::now());
            debug!("Updated RTT for peer {}: {:?}", peer_id, rtt);
        }
    }
    
    pub fn get_peer_count(&self) -> usize {
        self.discovered_peers.len()
    }
    
    pub fn get_config(&self) -> &RendezvousConfig {
        &self.config
    }
    
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }
}

impl std::fmt::Display for PeerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Peer {} (namespace: {}, TTL: {}s, discovered: {})", 
            self.id,
            self.namespace,
            self.ttl,
            self.discovered_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity;
    
    #[tokio::test]
    async fn test_rendezvous_config() {
        let config = RendezvousConfig {
            server: Some("12D3KooW...".to_string()),
            namespace: "test-namespace".to_string(),
            ttl: 1800,
            discovery_interval: Some(120),
            max_peers: Some(50),
        };
        
        assert_eq!(config.server, Some("12D3KooW...".to_string()));
        assert_eq!(config.namespace, "test-namespace");
        assert_eq!(config.ttl, 1800);
        assert_eq!(config.discovery_interval, Some(120));
        assert_eq!(config.max_peers, Some(50));
    }
    
    #[tokio::test]
    async fn test_peer_info() {
        let peer_id = PeerId::random();
        let addresses = vec!["/ip4/127.0.0.1/tcp/8080".parse().unwrap()];
        
        let peer_info = PeerInfo {
            id: peer_id,
            addresses,
            namespace: "test".to_string(),
            ttl: 3600,
            discovered_at: chrono::Utc::now(),
            last_seen: None,
            rtt: None,
        };
        
        assert_eq!(peer_info.id, peer_id);
        assert_eq!(peer_info.namespace, "test");
        assert_eq!(peer_info.ttl, 3600);
    }
}
