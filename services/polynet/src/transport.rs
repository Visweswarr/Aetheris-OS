use libp2p::{
    core::{upgrade, transport, transport::Transport},
    identity, noise, quic, tcp, yamux,
    PeerId, Multiaddr,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn, error};

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Failed to create transport: {0}")]
    TransportCreation(String),
    
    #[error("Failed to bind to address: {0}")]
    BindFailed(String),
    
    #[error("Failed to dial peer: {0}")]
    DialFailed(String),
    
    #[error("Failed to listen on address: {0}")]
    ListenFailed(String),
    
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub bind_addr: String,
    pub enable_quic: bool,
    pub enable_tcp: bool,
    pub enable_noise: bool,
    pub enable_mdns: bool,
    pub connection_timeout: Option<u64>,
    pub keep_alive_interval: Option<u64>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:0".to_string(),
            enable_quic: true,
            enable_tcp: true,
            enable_noise: true,
            enable_mdns: true,
            connection_timeout: Some(30),
            keep_alive_interval: Some(15),
        }
    }
}

pub struct PolynetTransport {
    config: TransportConfig,
    transport: Box<dyn Transport<Output = (PeerId, libp2p::Stream)> + Clone>,
    local_peer_id: PeerId,
}

impl PolynetTransport {
    pub async fn new(
        config: TransportConfig,
        local_peer_id: PeerId,
    ) -> Result<Self, TransportError> {
        info!("Creating polynet transport with config: {:?}", config);
        
        let transport = Self::build_transport(&config, local_peer_id).await?;
        
        Ok(Self {
            config,
            transport,
            local_peer_id,
        })
    }
    
    async fn build_transport(
        config: &TransportConfig,
        local_peer_id: PeerId,
    ) -> Result<Box<dyn Transport<Output = (PeerId, libp2p::Stream)> + Clone>, TransportError> {
        let mut transports = Vec::new();
        
        // Build QUIC transport if enabled
        if config.enable_quic {
            match Self::build_quic_transport(local_peer_id).await {
                Ok(quic_transport) => {
                    info!("QUIC transport enabled");
                    transports.push(quic_transport);
                }
                Err(e) => {
                    warn!("Failed to create QUIC transport: {}, falling back to TCP", e);
                }
            }
        }
        
        // Build TCP transport if enabled
        if config.enable_tcp {
            match Self::build_tcp_transport(local_peer_id).await {
                Ok(tcp_transport) => {
                    info!("TCP transport enabled");
                    transports.push(tcp_transport);
                }
                Err(e) => {
                    warn!("Failed to create TCP transport: {}", e);
                }
            }
        }
        
        if transports.is_empty() {
            return Err(TransportError::TransportCreation(
                "No transport protocols could be created".to_string()
            ));
        }
        
        // Combine transports
        let combined_transport = if transports.len() == 1 {
            transports.remove(0)
        } else {
            let mut combined = transports.remove(0);
            for transport in transports {
                combined = combined.or_transport(transport);
            }
            combined
        };
        
        // Apply noise encryption if enabled
        let encrypted_transport = if config.enable_noise {
            Self::apply_noise_encryption(combined_transport, local_peer_id).await?
        } else {
            combined_transport
        };
        
        // Apply yamux multiplexing
        let multiplexed_transport = encrypted_transport
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(&identity::Keypair::generate()).unwrap())
            .multiplex(yamux::YamuxConfig::default())
            .boxed();
        
        Ok(multiplexed_transport)
    }
    
    async fn build_quic_transport(
        local_peer_id: PeerId,
    ) -> Result<Box<dyn Transport<Output = (PeerId, libp2p::Stream)> + Clone>, TransportError> {
        let keypair = identity::Keypair::generate();
        
        let quic_transport = quic::async_std::Transport::new(quic::Config::new(&keypair))
            .map(|(peer_id, stream), _| (peer_id, stream.into()))
            .boxed();
        
        Ok(quic_transport)
    }
    
    async fn build_tcp_transport(
        local_peer_id: PeerId,
    ) -> Result<Box<dyn Transport<Output = (PeerId, libp2p::Stream)> + Clone>, TransportError> {
        let tcp_transport = tcp::async_io::Transport::new(tcp::Config::default())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(&identity::Keypair::generate()).unwrap())
            .multiplex(yamux::YamuxConfig::default())
            .boxed();
        
        Ok(tcp_transport)
    }
    
    async fn apply_noise_encryption(
        transport: Box<dyn Transport<Output = (PeerId, libp2p::Stream)> + Clone>,
        local_peer_id: PeerId,
    ) -> Result<Box<dyn Transport<Output = (PeerId, libp2p::Stream)> + Clone>, TransportError> {
        let keypair = identity::Keypair::generate();
        
        let encrypted_transport = transport
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(&keypair).unwrap())
            .boxed();
        
        Ok(encrypted_transport)
    }
    
    pub async fn listen_on(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), TransportError> {
        info!("Listening on address: {}", addr);
        
        match self.transport.listen_on(addr.clone()) {
            Ok(_) => {
                info!("Successfully listening on {}", addr);
                Ok(())
            }
            Err(e) => {
                error!("Failed to listen on {}: {}", addr, e);
                Err(TransportError::ListenFailed(e.to_string()))
            }
        }
    }
    
    pub async fn dial(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), TransportError> {
        info!("Dialing peer at address: {}", addr);
        
        match self.transport.dial(addr.clone()) {
            Ok(_) => {
                info!("Successfully dialed {}", addr);
                Ok(())
            }
            Err(e) => {
                error!("Failed to dial {}: {}", addr, e);
                Err(TransportError::DialFailed(e.to_string()))
            }
        }
    }
    
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }
    
    pub fn config(&self) -> &TransportConfig {
        &self.config
    }
    
    pub async fn get_connection_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            local_peer_id: self.local_peer_id,
            transport_protocols: self.get_enabled_protocols(),
            bind_address: self.config.bind_addr.clone(),
        }
    }
    
    fn get_enabled_protocols(&self) -> Vec<String> {
        let mut protocols = Vec::new();
        
        if self.config.enable_quic {
            protocols.push("QUIC".to_string());
        }
        if self.config.enable_tcp {
            protocols.push("TCP".to_string());
        }
        if self.config.enable_noise {
            protocols.push("Noise".to_string());
        }
        if self.config.enable_mdns {
            protocols.push("mDNS".to_string());
        }
        
        protocols
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub local_peer_id: PeerId,
    pub transport_protocols: Vec<String>,
    pub bind_address: String,
}

impl std::fmt::Display for ConnectionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Peer ID: {}, Protocols: [{}], Bind: {}", 
            self.local_peer_id,
            self.transport_protocols.join(", "),
            self.bind_address
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity;
    
    #[tokio::test]
    async fn test_transport_creation() {
        let config = TransportConfig::default();
        let peer_id = PeerId::random();
        
        let transport = PolynetTransport::new(config, peer_id).await;
        assert!(transport.is_ok());
    }
    
    #[tokio::test]
    async fn test_transport_config() {
        let config = TransportConfig {
            bind_addr: "127.0.0.1:0".to_string(),
            enable_quic: true,
            enable_tcp: false,
            enable_noise: true,
            enable_mdns: false,
            connection_timeout: Some(60),
            keep_alive_interval: Some(30),
        };
        
        assert_eq!(config.bind_addr, "127.0.0.1:0");
        assert!(config.enable_quic);
        assert!(!config.enable_tcp);
        assert!(config.enable_noise);
        assert!(!config.enable_mdns);
        assert_eq!(config.connection_timeout, Some(60));
        assert_eq!(config.keep_alive_interval, Some(30));
    }
    
    #[tokio::test]
    async fn test_connection_info() {
        let config = TransportConfig::default();
        let peer_id = PeerId::random();
        
        let transport = PolynetTransport::new(config, peer_id).await.unwrap();
        let info = transport.get_connection_info().await;
        
        assert_eq!(info.local_peer_id, peer_id);
        assert!(!info.transport_protocols.is_empty());
        assert_eq!(info.bind_address, "0.0.0.0:0");
    }
}
