use libp2p::{PeerId, Multiaddr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use tracing::{debug, info, warn, error};
use chrono::{DateTime, Utc, Duration};

use super::rendezvous::PeerInfo;

#[derive(Debug, Error)]
pub enum PeerBookError {
    #[error("Failed to save peer book: {0}")]
    SaveFailed(String),
    
    #[error("Failed to load peer book: {0}")]
    LoadFailed(String),
    
    #[error("Peer not found: {0}")]
    PeerNotFound(PeerId),
    
    #[error("Invalid peer data: {0}")]
    InvalidPeerData(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerBookEntry {
    pub id: PeerId,
    pub addresses: Vec<Multiaddr>,
    pub namespace: String,
    pub ttl: u64,
    pub discovered_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
    pub rtt: Option<u64>, // RTT in milliseconds
    pub connection_count: u32,
    pub last_connection: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

impl PeerBookEntry {
    pub fn new(peer_info: PeerInfo) -> Self {
        Self {
            id: peer_info.id,
            addresses: peer_info.addresses,
            namespace: peer_info.namespace,
            ttl: peer_info.ttl,
            discovered_at: peer_info.discovered_at,
            last_seen: peer_info.last_seen,
            rtt: peer_info.rtt.map(|d| d.as_millis() as u64),
            connection_count: 0,
            last_connection: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn update_rtt(&mut self, rtt: std::time::Duration) {
        self.rtt = Some(rtt.as_millis() as u64);
        self.last_seen = Some(Utc::now());
    }
    
    pub fn record_connection(&mut self) {
        self.connection_count += 1;
        self.last_connection = Some(Utc::now());
        self.last_seen = Some(Utc::now());
    }
    
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expiration_time = self.discovered_at + Duration::seconds(self.ttl as i64);
        now > expiration_time
    }
    
    pub fn get_avg_rtt(&self) -> Option<u64> {
        self.rtt
    }
    
    pub fn get_connection_stats(&self) -> (u32, Option<DateTime<Utc>>) {
        (self.connection_count, self.last_connection)
    }
}

pub struct PeerBook {
    peers: Arc<RwLock<HashMap<PeerId, PeerBookEntry>>>,
    config: PeerBookConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerBookConfig {
    pub max_peers: usize,
    pub cleanup_interval: u64, // seconds
    pub persistence_enabled: bool,
    pub persistence_path: String,
    pub auto_cleanup: bool,
}

impl Default for PeerBookConfig {
    fn default() -> Self {
        Self {
            max_peers: 1000,
            cleanup_interval: 300, // 5 minutes
            persistence_enabled: true,
            persistence_path: "data/peerbook.json".to_string(),
            auto_cleanup: true,
        }
    }
}

impl PeerBook {
    /// Create a new peer book
    pub fn new(config: PeerBookConfig) -> Self {
        info!("Creating new peer book with config: {:?}", config);
        
        let peer_book = Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            config,
        };
        
        // Start auto-cleanup if enabled
        if config.auto_cleanup {
            peer_book.start_auto_cleanup();
        }
        
        peer_book
    }
    
    /// Load peer book from storage
    pub async fn load() -> Result<Self, PeerBookError> {
        let config = PeerBookConfig::default();
        
        if !config.persistence_enabled {
            return Ok(Self::new(config));
        }
        
        let peer_book = Self::new(config);
        
        // Try to load from file
        if let Err(e) = peer_book.load_from_file().await {
            warn!("Failed to load peer book from file: {}, starting fresh", e);
        }
        
        Ok(peer_book)
    }
    
    /// Add or update a peer
    pub async fn add_peer(&self, peer_info: PeerInfo) -> Result<(), PeerBookError> {
        let mut peers = self.peers.write().await;
        
        let entry = PeerBookEntry::new(peer_info);
        let peer_id = entry.id;
        
        // Check if we're at capacity
        if peers.len() >= self.config.max_peers {
            // Remove oldest peer to make room
            if let Some((oldest_id, _)) = peers.iter()
                .min_by_key(|(_, entry)| entry.discovered_at) {
                let oldest_id = *oldest_id;
                peers.remove(&oldest_id);
                debug!("Removed oldest peer {} to make room for new peer", oldest_id);
            }
        }
        
        peers.insert(peer_id, entry);
        info!("Added peer to peer book: {}", peer_id);
        
        // Save to file if persistence is enabled
        if self.config.persistence_enabled {
            if let Err(e) = self.save_to_file().await {
                warn!("Failed to save peer book: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Get a peer by ID
    pub async fn get_peer(&self, peer_id: &PeerId) -> Result<PeerBookEntry, PeerBookError> {
        let peers = self.peers.read().await;
        
        peers.get(peer_id)
            .cloned()
            .ok_or_else(|| PeerBookError::PeerNotFound(*peer_id))
    }
    
    /// Update peer RTT
    pub async fn update_peer_rtt(&self, peer_id: &PeerId, rtt: std::time::Duration) -> Result<(), PeerBookError> {
        let mut peers = self.peers.write().await;
        
        if let Some(entry) = peers.get_mut(peer_id) {
            entry.update_rtt(rtt);
            debug!("Updated RTT for peer {}: {:?}", peer_id, rtt);
        } else {
            return Err(PeerBookError::PeerNotFound(*peer_id));
        }
        
        Ok(())
    }
    
    /// Record a connection to a peer
    pub async fn record_connection(&self, peer_id: &PeerId) -> Result<(), PeerBookError> {
        let mut peers = self.peers.write().await;
        
        if let Some(entry) = peers.get_mut(peer_id) {
            entry.record_connection();
            debug!("Recorded connection for peer {}", peer_id);
        } else {
            return Err(PeerBookError::PeerNotFound(*peer_id));
        }
        
        Ok(())
    }
    
    /// Add metadata to a peer
    pub async fn add_peer_metadata(
        &self,
        peer_id: &PeerId,
        key: String,
        value: String,
    ) -> Result<(), PeerBookError> {
        let mut peers = self.peers.write().await;
        
        if let Some(entry) = peers.get_mut(peer_id) {
            entry.add_metadata(key.clone(), value.clone());
            debug!("Added metadata for peer {}: {} = {}", peer_id, key, value);
        } else {
            return Err(PeerBookError::PeerNotFound(*peer_id));
        }
        
        Ok(())
    }
    
    /// List all peers
    pub async fn list_peers(&self) -> Result<Vec<PeerBookEntry>, PeerBookError> {
        let peers = self.peers.read().await;
        
        Ok(peers.values().cloned().collect())
    }
    
    /// List peers by namespace
    pub async fn list_peers_by_namespace(&self, namespace: &str) -> Result<Vec<PeerBookEntry>, PeerBookError> {
        let peers = self.peers.read().await;
        
        Ok(peers.values()
            .filter(|entry| entry.namespace == namespace)
            .cloned()
            .collect())
    }
    
    /// Get peer count
    pub async fn get_peer_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }
    
    /// Get peers with RTT information
    pub async fn get_peers_with_rtt(&self) -> Result<Vec<PeerBookEntry>, PeerBookError> {
        let peers = self.peers.read().await;
        
        Ok(peers.values()
            .filter(|entry| entry.rtt.is_some())
            .cloned()
            .collect())
    }
    
    /// Get peers by connection count
    pub async fn get_peers_by_connection_count(&self, min_connections: u32) -> Result<Vec<PeerBookEntry>, PeerBookError> {
        let peers = self.peers.read().await;
        
        Ok(peers.values()
            .filter(|entry| entry.connection_count >= min_connections)
            .cloned()
            .collect())
    }
    
    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &PeerId) -> Result<(), PeerBookError> {
        let mut peers = self.peers.write().await;
        
        if peers.remove(peer_id).is_some() {
            info!("Removed peer from peer book: {}", peer_id);
            
            // Save to file if persistence is enabled
            if self.config.persistence_enabled {
                if let Err(e) = self.save_to_file().await {
                    warn!("Failed to save peer book: {}", e);
                }
            }
        } else {
            return Err(PeerBookError::PeerNotFound(*peer_id));
        }
        
        Ok(())
    }
    
    /// Clean up expired peers
    pub async fn cleanup_expired_peers(&self) -> Result<usize, PeerBookError> {
        let mut peers = self.peers.write().await;
        let mut expired_peers = Vec::new();
        
        // Find expired peers
        for (peer_id, entry) in peers.iter() {
            if entry.is_expired() {
                expired_peers.push(*peer_id);
            }
        }
        
        // Remove expired peers
        let removed_count = expired_peers.len();
        for peer_id in expired_peers {
            peers.remove(&peer_id);
            debug!("Removed expired peer: {}", peer_id);
        }
        
        if removed_count > 0 {
            info!("Cleaned up {} expired peers", removed_count);
            
            // Save to file if persistence is enabled
            if self.config.persistence_enabled {
                if let Err(e) = self.save_to_file().await {
                    warn!("Failed to save peer book after cleanup: {}", e);
                }
            }
        }
        
        Ok(removed_count)
    }
    
    /// Get peer book statistics
    pub async fn get_stats(&self) -> Result<PeerBookStats, PeerBookError> {
        let peers = self.peers.read().await;
        
        let total_peers = peers.len();
        let peers_with_rtt = peers.values().filter(|entry| entry.rtt.is_some()).count();
        let total_connections: u32 = peers.values().map(|entry| entry.connection_count).sum();
        
        let avg_rtt = if peers_with_rtt > 0 {
            let total_rtt: u64 = peers.values()
                .filter_map(|entry| entry.rtt)
                .sum();
            Some(total_rtt / peers_with_rtt as u64)
        } else {
            None
        };
        
        let namespaces: std::collections::HashSet<String> = peers.values()
            .map(|entry| entry.namespace.clone())
            .collect();
        
        Ok(PeerBookStats {
            total_peers,
            peers_with_rtt,
            total_connections,
            avg_rtt,
            namespace_count: namespaces.len(),
            namespaces: namespaces.into_iter().collect(),
        })
    }
    
    /// Start auto-cleanup task
    fn start_auto_cleanup(&self) {
        let peer_book = self.clone();
        let cleanup_interval = self.config.cleanup_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(cleanup_interval)
            );
            
            loop {
                interval.tick().await;
                
                if let Err(e) = peer_book.cleanup_expired_peers().await {
                    warn!("Auto-cleanup failed: {}", e);
                }
            }
        });
    }
    
    /// Save peer book to file
    async fn save_to_file(&self) -> Result<(), PeerBookError> {
        let peers = self.peers.read().await;
        let data = serde_json::to_string_pretty(&*peers)
            .map_err(|e| PeerBookError::SaveFailed(e.to_string()))?;
        
        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(&self.config.persistence_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| PeerBookError::SaveFailed(e.to_string()))?;
        }
        
        std::fs::write(&self.config.persistence_path, data)
            .map_err(|e| PeerBookError::SaveFailed(e.to_string()))?;
        
        debug!("Saved peer book to {}", self.config.persistence_path);
        Ok(())
    }
    
    /// Load peer book from file
    async fn load_from_file(&self) -> Result<(), PeerBookError> {
        let data = std::fs::read_to_string(&self.config.persistence_path)
            .map_err(|e| PeerBookError::LoadFailed(e.to_string()))?;
        
        let peers: HashMap<PeerId, PeerBookEntry> = serde_json::from_str(&data)
            .map_err(|e| PeerBookError::LoadFailed(e.to_string()))?;
        
        let mut peer_map = self.peers.write().await;
        *peer_map = peers;
        
        info!("Loaded {} peers from {}", peer_map.len(), self.config.persistence_path);
        Ok(())
    }
}

impl Clone for PeerBook {
    fn clone(&self) -> Self {
        Self {
            peers: Arc::clone(&self.peers),
            config: self.config.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerBookStats {
    pub total_peers: usize,
    pub peers_with_rtt: usize,
    pub total_connections: u32,
    pub avg_rtt: Option<u64>,
    pub namespace_count: usize,
    pub namespaces: Vec<String>,
}

impl std::fmt::Display for PeerBookStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Peer Book Stats: {} total peers, {} with RTT, {} total connections, {} namespaces", 
            self.total_peers,
            self.peers_with_rtt,
            self.total_connections,
            self.namespace_count
        )?;
        
        if let Some(avg_rtt) = self.avg_rtt {
            write!(f, ", avg RTT: {}ms", avg_rtt)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::identity;
    
    #[tokio::test]
    async fn test_peer_book_creation() {
        let config = PeerBookConfig::default();
        let peer_book = PeerBook::new(config);
        
        assert_eq!(peer_book.get_peer_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_add_and_get_peer() {
        let peer_book = PeerBook::new(PeerBookConfig::default());
        
        let peer_id = PeerId::random();
        let addresses = vec!["/ip4/127.0.0.1/tcp/8080".parse().unwrap()];
        
        let peer_info = PeerInfo {
            id: peer_id,
            addresses,
            namespace: "test".to_string(),
            ttl: 3600,
            discovered_at: Utc::now(),
            last_seen: None,
            rtt: None,
        };
        
        peer_book.add_peer(peer_info).await.unwrap();
        
        let entry = peer_book.get_peer(&peer_id).await.unwrap();
        assert_eq!(entry.id, peer_id);
        assert_eq!(entry.namespace, "test");
    }
    
    #[tokio::test]
    async fn test_peer_book_stats() {
        let peer_book = PeerBook::new(PeerBookConfig::default());
        
        let stats = peer_book.get_stats().await.unwrap();
        assert_eq!(stats.total_peers, 0);
        assert_eq!(stats.peers_with_rtt, 0);
        assert_eq!(stats.total_connections, 0);
    }
}
