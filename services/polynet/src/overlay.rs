//! libp2p Overlay for mesh networking
//! 
//! This module provides libp2p-based overlay networking for the mesh class
//! with DID identity, NAT traversal, and gossipsub for control topics.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use tracing::{info, warn, error, debug, instrument};

use libp2p::{
    core::transport::Boxed,
    swarm::{NetworkBehaviour, Swarm, SwarmBuilder},
    PeerId, Multiaddr, Transport,
    identity::{Keypair, secp256k1},
    noise, yamux, mplex,
    quic, tcp,
    identify, ping, kad, gossipsub,
    mdns,
};

use super::{
    did::DidPeerId,
    mesh::{MeshNode, MeshNodeConfig, MeshConfig},
    peerbook::{PeerBook, PeerBookEntry},
    transport::{PolynetTransport, TransportConfig},
};

/// Overlay network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    /// Enable QUIC transport
    pub enable_quic: bool,
    /// Enable TCP transport
    pub enable_tcp: bool,
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Enable DHT (Kademlia)
    pub enable_dht: bool,
    /// Enable GossipSub
    pub enable_gossipsub: bool,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<Multiaddr>,
    /// Listen addresses
    pub listen_addresses: Vec<Multiaddr>,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Ping timeout
    pub ping_timeout: Duration,
    /// DHT query timeout
    pub dht_query_timeout: Duration,
    /// GossipSub configuration
    pub gossipsub_config: GossipSubConfig,
    /// NAT traversal configuration
    pub nat_traversal: NatTraversalConfig,
}

/// GossipSub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipSubConfig {
    /// Control topics
    pub control_topics: Vec<String>,
    /// Message validation
    pub message_validation: bool,
    /// Flood publish
    pub flood_publish: bool,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// History length
    pub history_length: usize,
    /// History gossip length
    pub history_gossip_length: usize,
}

/// NAT traversal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalConfig {
    /// Enable NAT traversal
    pub enable: bool,
    /// STUN servers
    pub stun_servers: Vec<String>,
    /// TURN servers
    pub turn_servers: Vec<TurnServer>,
    /// Hole punching timeout
    pub hole_punching_timeout: Duration,
    /// Relay timeout
    pub relay_timeout: Duration,
}

/// TURN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    /// Server address
    pub address: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Realm
    pub realm: Option<String>,
}

/// Overlay network behaviour
#[derive(NetworkBehaviour)]
pub struct OverlayBehaviour {
    /// Identify protocol
    pub identify: identify::Behaviour,
    /// Ping protocol
    pub ping: ping::Behaviour,
    /// Kademlia DHT
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// GossipSub
    pub gossipsub: gossipsub::Behaviour,
    /// mDNS discovery
    pub mdns: mdns::tokio::Behaviour,
}

/// Overlay network node
pub struct OverlayNode {
    /// Node configuration
    config: OverlayConfig,
    /// libp2p swarm
    swarm: Swarm<OverlayBehaviour>,
    /// Peer ID
    peer_id: PeerId,
    /// DID peer ID
    did_peer_id: DidPeerId,
    /// Peer book
    peer_book: Arc<RwLock<PeerBook>>,
    /// Active connections
    connections: Arc<Mutex<HashMap<PeerId, ConnectionInfo>>>,
    /// Control topics
    control_topics: Arc<Mutex<HashMap<String, TopicInfo>>>,
    /// NAT traversal state
    nat_state: Arc<Mutex<NatState>>,
    /// Event channels
    event_tx: mpsc::UnboundedSender<OverlayEvent>,
    /// Running flag
    running: Arc<Mutex<bool>>,
    /// Statistics
    stats: Arc<Mutex<OverlayStats>>,
}

/// Connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Peer ID
    pub peer_id: PeerId,
    /// Connection address
    pub address: Multiaddr,
    /// Connection state
    pub state: ConnectionState,
    /// Connection time
    pub connected_at: Instant,
    /// Last activity
    pub last_activity: Instant,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Protocols
    pub protocols: Vec<String>,
}

/// Connection state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionState {
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Disconnecting
    Disconnecting,
    /// Disconnected
    Disconnected,
    /// Error
    Error,
}

/// Topic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    /// Topic name
    pub name: String,
    /// Topic hash
    pub hash: String,
    /// Subscribers
    pub subscribers: Vec<PeerId>,
    /// Message count
    pub message_count: u64,
    /// Last message time
    pub last_message: Option<Instant>,
}

/// NAT traversal state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatState {
    /// Public address
    pub public_address: Option<Multiaddr>,
    /// NAT type
    pub nat_type: NatType,
    /// Hole punching attempts
    pub hole_punching_attempts: u32,
    /// Successful hole punches
    pub successful_hole_punches: u32,
    /// Relay connections
    pub relay_connections: Vec<PeerId>,
}

/// NAT type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NatType {
    /// No NAT
    None,
    /// Full cone NAT
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT
    Symmetric,
    /// Unknown
    Unknown,
}

/// Overlay event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverlayEvent {
    /// Peer connected
    PeerConnected { peer_id: PeerId, address: Multiaddr },
    /// Peer disconnected
    PeerDisconnected { peer_id: PeerId },
    /// Message received
    MessageReceived { topic: String, message: Vec<u8>, peer_id: PeerId },
    /// DHT query result
    DhtQueryResult { query_id: String, result: Vec<PeerId> },
    /// NAT traversal result
    NatTraversalResult { success: bool, public_address: Option<Multiaddr> },
    /// Error event
    Error { error: String },
}

/// Overlay statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverlayStats {
    /// Total connections
    pub total_connections: u64,
    /// Active connections
    pub active_connections: u64,
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// DHT queries
    pub dht_queries: u64,
    /// GossipSub messages
    pub gossipsub_messages: u64,
    /// NAT traversal attempts
    pub nat_traversal_attempts: u64,
    /// Successful NAT traversals
    pub successful_nat_traversals: u64,
}

impl OverlayNode {
    /// Create a new overlay node
    pub fn new(config: OverlayConfig, did_peer_id: DidPeerId) -> Result<Self, String> {
        // Create identity from DID
        let keypair = did_peer_id.to_keypair()?;
        let peer_id = PeerId::from(keypair.public());

        // Create transport
        let transport = Self::create_transport(&config)?;

        // Create behaviour
        let behaviour = Self::create_behaviour(&config, peer_id)?;

        // Create swarm
        let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id)
            .build();

        // Create event channel
        let (event_tx, _event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            config,
            swarm,
            peer_id,
            did_peer_id,
            peer_book: Arc::new(RwLock::new(PeerBook::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
            control_topics: Arc::new(Mutex::new(HashMap::new())),
            nat_state: Arc::new(Mutex::new(NatState {
                public_address: None,
                nat_type: NatType::Unknown,
                hole_punching_attempts: 0,
                successful_hole_punches: 0,
                relay_connections: Vec::new(),
            })),
            event_tx,
            running: Arc::new(Mutex::new(false)),
            stats: Arc::new(Mutex::new(OverlayStats::default())),
        })
    }

    /// Create transport
    fn create_transport(config: &OverlayConfig) -> Result<Boxed<(PeerId, libp2p::core::muxing::StreamMuxerBox)>, String> {
        let mut transport = if config.enable_quic {
            quic::tokio::Transport::new(quic::Config::new(&Keypair::generate()))
                .map(|(peer_id, conn), _| (peer_id, libp2p::core::muxing::StreamMuxerBox::new(conn)))
                .boxed()
        } else {
            tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(noise::Config::new(&Keypair::generate()).unwrap())
                .multiplex(yamux::Config::default())
                .boxed()
        };

        Ok(transport)
    }

    /// Create behaviour
    fn create_behaviour(config: &OverlayConfig, peer_id: PeerId) -> Result<OverlayBehaviour, String> {
        // Create identify behaviour
        let identify = identify::Behaviour::new(identify::Config::new(
            "/polynet/1.0.0".to_string(),
            Keypair::generate().public(),
        ));

        // Create ping behaviour
        let ping = ping::Behaviour::new(ping::Config::new().with_keep_alive(true));

        // Create Kademlia DHT
        let store = kad::store::MemoryStore::new(peer_id);
        let kademlia = kad::Behaviour::new(peer_id, store);

        // Create GossipSub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(config.gossipsub_config.heartbeat_interval)
            .history_length(config.gossipsub_config.history_length)
            .history_gossip_length(config.gossipsub_config.history_gossip_length)
            .flood_publish(config.gossipsub_config.flood_publish)
            .build()
            .map_err(|e| format!("Failed to create GossipSub config: {}", e))?;
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(Keypair::generate()),
            gossipsub_config,
        ).map_err(|e| format!("Failed to create GossipSub behaviour: {}", e))?;

        // Create mDNS
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            peer_id,
        ).map_err(|e| format!("Failed to create mDNS behaviour: {}", e))?;

        Ok(OverlayBehaviour {
            identify,
            ping,
            kademlia,
            gossipsub,
            mdns,
        })
    }

    /// Start the overlay node
    pub async fn start(&mut self) -> Result<(), String> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Err("Overlay node already running".to_string());
        }
        *running = true;
        drop(running);

        // Start listening
        for addr in &self.config.listen_addresses {
            self.swarm.listen_on(addr.clone())
                .map_err(|e| format!("Failed to listen on {}: {}", addr, e))?;
        }

        // Connect to bootstrap peers
        for peer_addr in &self.config.bootstrap_peers {
            self.swarm.dial(peer_addr.clone())
                .map_err(|e| format!("Failed to dial bootstrap peer {}: {}", peer_addr, e))?;
        }

        // Subscribe to control topics
        for topic in &self.config.gossipsub_config.control_topics {
            self.swarm.behaviour_mut().gossipsub.subscribe(
                &gossipsub::IdentTopic::new(topic.clone())
            ).map_err(|e| format!("Failed to subscribe to topic {}: {}", topic, e))?;
        }

        info!("Overlay node started with peer ID: {}", self.peer_id);
        Ok(())
    }

    /// Stop the overlay node
    pub async fn stop(&mut self) -> Result<(), String> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Err("Overlay node not running".to_string());
        }
        *running = false;
        drop(running);

        // Close all connections
        self.swarm.disconnect_peers();

        info!("Overlay node stopped");
        Ok(())
    }

    /// Connect to a peer
    pub async fn connect_to_peer(&mut self, peer_addr: Multiaddr) -> Result<(), String> {
        self.swarm.dial(peer_addr.clone())
            .map_err(|e| format!("Failed to connect to peer {}: {}", peer_addr, e))?;

        info!("Connecting to peer: {}", peer_addr);
        Ok(())
    }

    /// Disconnect from a peer
    pub async fn disconnect_from_peer(&mut self, peer_id: PeerId) -> Result<(), String> {
        self.swarm.disconnect_peer(peer_id);
        info!("Disconnected from peer: {}", peer_id);
        Ok(())
    }

    /// Publish a message to a topic
    pub async fn publish_message(&mut self, topic: String, message: Vec<u8>) -> Result<(), String> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.swarm.behaviour_mut().gossipsub.publish(topic, message)
            .map_err(|e| format!("Failed to publish message: {}", e))?;
        Ok(())
    }

    /// Subscribe to a topic
    pub async fn subscribe_to_topic(&mut self, topic: String) -> Result<(), String> {
        let topic = gossipsub::IdentTopic::new(topic.clone());
        self.swarm.behaviour_mut().gossipsub.subscribe(topic)
            .map_err(|e| format!("Failed to subscribe to topic {}: {}", topic, e))?;
        Ok(())
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe_from_topic(&mut self, topic: String) -> Result<(), String> {
        let topic = gossipsub::IdentTopic::new(topic.clone());
        self.swarm.behaviour_mut().gossipsub.unsubscribe(&topic)
            .map_err(|e| format!("Failed to unsubscribe from topic {}: {}", topic, e))?;
        Ok(())
    }

    /// Query DHT for peers
    pub async fn query_dht(&mut self, key: Vec<u8>) -> Result<(), String> {
        let query_id = Uuid::new_v4().to_string();
        self.swarm.behaviour_mut().kademlia.get_closest_peers(key);
        info!("DHT query started: {}", query_id);
        Ok(())
    }

    /// Get peer information
    pub async fn get_peer_info(&self, peer_id: PeerId) -> Option<ConnectionInfo> {
        self.connections.lock().unwrap().get(&peer_id).cloned()
    }

    /// Get all connected peers
    pub async fn get_connected_peers(&self) -> Vec<ConnectionInfo> {
        self.connections.lock().unwrap().values().cloned().collect()
    }

    /// Get overlay statistics
    pub fn get_stats(&self) -> OverlayStats {
        self.stats.lock().unwrap().clone()
    }

    /// Check if node is running
    pub fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    /// Get peer ID
    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Get DID peer ID
    pub fn get_did_peer_id(&self) -> &DidPeerId {
        &self.did_peer_id
    }
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enable_quic: true,
            enable_tcp: true,
            enable_mdns: true,
            enable_dht: true,
            enable_gossipsub: true,
            bootstrap_peers: Vec::new(),
            listen_addresses: vec![
                "/ip4/0.0.0.0/tcp/0".parse().unwrap(),
                "/ip6/::/tcp/0".parse().unwrap(),
            ],
            connection_timeout: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            dht_query_timeout: Duration::from_secs(60),
            gossipsub_config: GossipSubConfig {
                control_topics: vec![
                    "polynet/control".to_string(),
                    "polynet/discovery".to_string(),
                ],
                message_validation: true,
                flood_publish: false,
                heartbeat_interval: Duration::from_secs(1),
                history_length: 100,
                history_gossip_length: 10,
            },
            nat_traversal: NatTraversalConfig {
                enable: true,
                stun_servers: vec![
                    "stun.l.google.com:19302".to_string(),
                    "stun1.l.google.com:19302".to_string(),
                ],
                turn_servers: Vec::new(),
                hole_punching_timeout: Duration::from_secs(30),
                relay_timeout: Duration::from_secs(60),
            },
        }
    }
}
