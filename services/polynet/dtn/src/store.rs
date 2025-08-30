use super::envelope::{DtnEnvelope, EnvelopeError};
use dashmap::DashMap;
use sled::{Db, Tree};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tracing::{debug, info, warn, error};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("Envelope not found: {0}")]
    EnvelopeNotFound(String),
    
    #[error("Storage full: {0}")]
    StorageFull(String),
    
    #[error("Duplicate envelope: {0}")]
    DuplicateEnvelope(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    pub max_envelopes: usize,
    pub max_envelope_size: usize,
    pub max_ttl: u64,
    pub eviction_policy: EvictionPolicy,
    pub respect_priority: bool,
    pub detect_duplicates: bool,
    pub cleanup_interval: u64,
    pub persistence_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionPolicy {
    Fifo,      // First in, first out
    Lru,       // Least recently used
    Priority,  // Priority-based
    Ttl,       // Time-to-live based
    Size,      // Size-based
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreStats {
    pub total_envelopes: usize,
    pub total_storage_bytes: u64,
    pub total_received: u64,
    pub total_delivered: u64,
    pub total_expired: u64,
    pub total_dropped: u64,
    pub avg_storage_time: f64,
    pub last_cleanup: SystemTime,
}

pub struct DtnStore {
    config: StoreConfig,
    db: Db,
    envelopes: DashMap<String, DtnEnvelope>,
    stats: Arc<RwLock<StoreStats>>,
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
}

impl DtnStore {
    /// Create a new DTN store
    pub fn new(config: StoreConfig) -> Result<Self, StoreError> {
        info!("Creating DTN store with config: {:?}", config);
        
        // Create database
        let db = sled::open(&config.persistence_path)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        let store = Self {
            config,
            db,
            envelopes: DashMap::new(),
            stats: Arc::new(RwLock::new(StoreStats {
                total_envelopes: 0,
                total_storage_bytes: 0,
                total_received: 0,
                total_delivered: 0,
                total_expired: 0,
                total_dropped: 0,
                avg_storage_time: 0.0,
                last_cleanup: SystemTime::now(),
            })),
            cleanup_task: None,
        };
        
        // Start cleanup task
        store.start_cleanup_task();
        
        Ok(store)
    }
    
    /// Store an envelope
    pub async fn store_envelope(&self, envelope: DtnEnvelope) -> Result<(), StoreError> {
        let envelope_id = envelope.id.clone();
        
        // Check if envelope is expired
        if envelope.is_expired() {
            warn!("Attempting to store expired envelope: {}", envelope_id);
            return Err(StoreError::DuplicateEnvelope(
                "Cannot store expired envelope".to_string()
            ));
        }
        
        // Check for duplicates if enabled
        if self.config.detect_duplicates && self.envelopes.contains_key(&envelope_id) {
            warn!("Duplicate envelope detected: {}", envelope_id);
            return Err(StoreError::DuplicateEnvelope(
                format!("Envelope {} already exists", envelope_id)
            ));
        }
        
        // Check storage capacity
        if self.envelopes.len() >= self.config.max_envelopes {
            self.evict_envelope().await?;
        }
        
        // Check envelope size
        let envelope_size = self.calculate_envelope_size(&envelope);
        if envelope_size > self.config.max_envelope_size {
            return Err(StoreError::StorageFull(
                format!("Envelope size {} exceeds maximum {}", envelope_size, self.config.max_envelope_size)
            ));
        }
        
        // Store envelope
        let envelope_clone = envelope.clone();
        self.envelopes.insert(envelope_id.clone(), envelope_clone);
        
        // Persist to database
        self.persist_envelope(&envelope).await?;
        
        // Update statistics
        self.update_stats_received(envelope_size).await;
        
        info!("Stored envelope: {} (size: {} bytes)", envelope_id, envelope_size);
        
        Ok(())
    }
    
    /// Retrieve an envelope by ID
    pub async fn retrieve_envelope(&self, envelope_id: &str) -> Result<Option<DtnEnvelope>, StoreError> {
        // Check in-memory cache first
        if let Some(envelope) = self.envelopes.get(envelope_id) {
            let envelope = envelope.clone();
            
            // Check if expired
            if envelope.is_expired() {
                info!("Retrieved expired envelope: {}", envelope_id);
                // Remove expired envelope
                self.envelopes.remove(envelope_id);
                self.update_stats_expired().await;
                return Ok(None);
            }
            
            return Ok(Some(envelope));
        }
        
        // Try to load from database
        if let Some(envelope) = self.load_envelope_from_db(envelope_id).await? {
            // Check if expired
            if envelope.is_expired() {
                info!("Loaded expired envelope from DB: {}", envelope_id);
                self.remove_envelope_from_db(envelope_id).await?;
                self.update_stats_expired().await;
                return Ok(None);
            }
            
            // Add to memory cache
            self.envelopes.insert(envelope_id.to_string(), envelope.clone());
            return Ok(Some(envelope));
        }
        
        Ok(None)
    }
    
    /// Remove an envelope
    pub async fn remove_envelope(&self, envelope_id: &str) -> Result<bool, StoreError> {
        // Remove from memory
        let removed = self.envelopes.remove(envelope_id).is_some();
        
        // Remove from database
        if removed {
            self.remove_envelope_from_db(envelope_id).await?;
        }
        
        Ok(removed)
    }
    
    /// List all envelopes
    pub async fn list_envelopes(&self) -> Vec<DtnEnvelope> {
        self.envelopes
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// List envelopes by destination
    pub async fn list_envelopes_by_destination(&self, destination: &str) -> Vec<DtnEnvelope> {
        self.envelopes
            .iter()
            .filter(|entry| entry.value().destination == destination)
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// List envelopes by message type
    pub async fn list_envelopes_by_type(&self, message_type: &str) -> Vec<DtnEnvelope> {
        self.envelopes
            .iter()
            .filter(|entry| entry.value().message_type == message_type)
            .map(|entry| entry.value().clone())
            .collect()
    }
    
    /// Get store statistics
    pub async fn get_stats(&self) -> StoreStats {
        self.stats.read().await.clone()
    }
    
    /// Clean up expired envelopes
    pub async fn cleanup_expired(&self) -> Result<usize, StoreError> {
        let mut expired_count = 0;
        let mut expired_ids = Vec::new();
        
        // Find expired envelopes
        for entry in self.envelopes.iter() {
            if entry.value().is_expired() {
                expired_ids.push(entry.key().clone());
                expired_count += 1;
            }
        }
        
        // Remove expired envelopes
        for envelope_id in expired_ids {
            self.remove_envelope(&envelope_id).await?;
        }
        
        // Update statistics
        if expired_count > 0 {
            let mut stats = self.stats.write().await;
            stats.total_expired += expired_count as u64;
            stats.last_cleanup = SystemTime::now();
        }
        
        info!("Cleaned up {} expired envelopes", expired_count);
        
        Ok(expired_count)
    }
    
    /// Evict an envelope based on policy
    async fn evict_envelope(&self) -> Result<(), StoreError> {
        let eviction_policy = self.config.eviction_policy;
        
        let envelope_to_remove = match eviction_policy {
            EvictionPolicy::Fifo => {
                // Remove oldest envelope
                self.envelopes
                    .iter()
                    .min_by_key(|entry| entry.value().created_at)
                    .map(|entry| entry.key().clone())
            }
            EvictionPolicy::Lru => {
                // For simplicity, remove random envelope (LRU would need tracking)
                self.envelopes
                    .iter()
                    .next()
                    .map(|entry| entry.key().clone())
            }
            EvictionPolicy::Priority => {
                // Remove lowest priority envelope
                self.envelopes
                    .iter()
                    .min_by_key(|entry| entry.value().priority)
                    .map(|entry| entry.key().clone())
            }
            EvictionPolicy::Ttl => {
                // Remove envelope with shortest remaining TTL
                self.envelopes
                    .iter()
                    .min_by_key(|entry| entry.value().remaining_ttl())
                    .map(|entry| entry.key().clone())
            }
            EvictionPolicy::Size => {
                // Remove largest envelope
                self.envelopes
                    .iter()
                    .max_by_key(|entry| self.calculate_envelope_size(entry.value()))
                    .map(|entry| entry.key().clone())
            }
        };
        
        if let Some(envelope_id) = envelope_to_remove {
            info!("Evicting envelope: {} (policy: {:?})", envelope_id, eviction_policy);
            self.remove_envelope(&envelope_id).await?;
            self.update_stats_dropped().await;
        }
        
        Ok(())
    }
    
    /// Calculate envelope size in bytes
    fn calculate_envelope_size(&self, envelope: &DtnEnvelope) -> usize {
        // Rough estimation of envelope size
        envelope.id.len() +
        envelope.source.len() +
        envelope.destination.len() +
        envelope.message_type.len() +
        envelope.payload.len() +
        envelope.nonce.len() +
        envelope.security.signature.len() +
        envelope.security.signer_public_key.len() +
        envelope.metadata.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>()
    }
    
    /// Persist envelope to database
    async fn persist_envelope(&self, envelope: &DtnEnvelope) -> Result<(), StoreError> {
        let envelope_id = &envelope.id;
        let envelope_data = serde_json::to_vec(envelope)
            .map_err(|e| StoreError::SerializationError(e.to_string()))?;
        
        let tree = self.db.open_tree("envelopes")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.insert(envelope_id.as_bytes(), envelope_data)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.flush()
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Load envelope from database
    async fn load_envelope_from_db(&self, envelope_id: &str) -> Result<Option<DtnEnvelope>, StoreError> {
        let tree = self.db.open_tree("envelopes")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        if let Some(data) = tree.get(envelope_id.as_bytes())
            .map_err(|e| StoreError::DatabaseError(e.to_string()))? {
            let envelope: DtnEnvelope = serde_json::from_slice(&data)
                .map_err(|e| StoreError::DeserializationError(e.to_string()))?;
            Ok(Some(envelope))
        } else {
            Ok(None)
        }
    }
    
    /// Remove envelope from database
    async fn remove_envelope_from_db(&self, envelope_id: &str) -> Result<(), StoreError> {
        let tree = self.db.open_tree("envelopes")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.remove(envelope_id.as_bytes())
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.flush()
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Update statistics for received envelope
    async fn update_stats_received(&self, envelope_size: usize) {
        let mut stats = self.stats.write().await;
        stats.total_received += 1;
        stats.total_envelopes = self.envelopes.len();
        stats.total_storage_bytes += envelope_size as u64;
    }
    
    /// Update statistics for expired envelope
    async fn update_stats_expired(&self) {
        let mut stats = self.stats.write().await;
        stats.total_expired += 1;
        stats.total_envelopes = self.envelopes.len();
    }
    
    /// Update statistics for dropped envelope
    async fn update_stats_dropped(&self) {
        let mut stats = self.stats.write().await;
        stats.total_dropped += 1;
        stats.total_envelopes = self.envelopes.len();
    }
    
    /// Start cleanup task
    fn start_cleanup_task(&self) {
        let store = self.clone();
        let cleanup_interval = self.config.cleanup_interval;
        
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = store.cleanup_expired().await {
                    error!("Cleanup task failed: {}", e);
                }
            }
        });
        
        self.cleanup_task = Some(handle);
    }
}

impl Clone for DtnStore {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            db: self.db.clone(),
            envelopes: self.envelopes.clone(),
            stats: Arc::clone(&self.stats),
            cleanup_task: None, // Cannot clone task handle
        }
    }
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            max_envelopes: 1000,
            max_envelope_size: 1024 * 1024, // 1MB
            max_ttl: 86400, // 24 hours
            eviction_policy: EvictionPolicy::Ttl,
            respect_priority: true,
            detect_duplicates: true,
            cleanup_interval: 300, // 5 minutes
            persistence_path: "data/dtn_store".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::envelope::{DtnEnvelope, RoutingStrategy, AckType};
    
    #[tokio::test]
    async fn test_store_creation() {
        let config = StoreConfig::default();
        let store = DtnStore::new(config).unwrap();
        
        let stats = store.get_stats().await;
        assert_eq!(stats.total_envelopes, 0);
        assert_eq!(stats.total_received, 0);
    }
    
    #[tokio::test]
    async fn test_store_envelope() {
        let config = StoreConfig::default();
        let store = DtnStore::new(config).unwrap();
        
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        store.store_envelope(envelope.clone()).await.unwrap();
        
        let stats = store.get_stats().await;
        assert_eq!(stats.total_envelopes, 1);
        assert_eq!(stats.total_received, 1);
    }
    
    #[tokio::test]
    async fn test_retrieve_envelope() {
        let config = StoreConfig::default();
        let store = DtnStore::new(config).unwrap();
        
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        let envelope_id = envelope.id.clone();
        store.store_envelope(envelope.clone()).await.unwrap();
        
        let retrieved = store.retrieve_envelope(&envelope_id).await.unwrap();
        assert!(retrieved.is_some());
        
        let retrieved_envelope = retrieved.unwrap();
        assert_eq!(retrieved_envelope.id, envelope_id);
    }
    
    #[tokio::test]
    async fn test_expired_envelope_cleanup() {
        let config = StoreConfig::default();
        let store = DtnStore::new(config).unwrap();
        
        let mut envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            1, // 1 second TTL
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        store.store_envelope(envelope.clone()).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Cleanup should remove expired envelope
        let cleaned = store.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);
        
        let stats = store.get_stats().await;
        assert_eq!(stats.total_expired, 1);
    }
    
    #[tokio::test]
    async fn test_duplicate_detection() {
        let mut config = StoreConfig::default();
        config.detect_duplicates = true;
        let store = DtnStore::new(config).unwrap();
        
        let envelope = DtnEnvelope::new(
            "did:polynet:source".to_string(),
            "did:polynet:dest".to_string(),
            3600,
            "test_message".to_string(),
            b"test payload".to_vec(),
            100,
        );
        
        // Store first time
        store.store_envelope(envelope.clone()).await.unwrap();
        
        // Try to store again
        let result = store.store_envelope(envelope).await;
        assert!(result.is_err());
        
        if let Err(StoreError::DuplicateEnvelope(_)) = result {
            // Expected error
        } else {
            panic!("Expected DuplicateEnvelope error");
        }
    }
    
    #[tokio::test]
    async fn test_eviction_policy() {
        let mut config = StoreConfig::default();
        config.max_envelopes = 2;
        config.eviction_policy = EvictionPolicy::Priority;
        let store = DtnStore::new(config).unwrap();
        
        // Store two envelopes with different priorities
        let envelope1 = DtnEnvelope::new(
            "did:polynet:source1".to_string(),
            "did:polynet:dest1".to_string(),
            3600,
            "test_message1".to_string(),
            b"test payload1".to_vec(),
            50, // Lower priority
        );
        
        let envelope2 = DtnEnvelope::new(
            "did:polynet:source2".to_string(),
            "did:polynet:dest2".to_string(),
            3600,
            "test_message2".to_string(),
            b"test payload2".to_vec(),
            100, // Higher priority
        );
        
        store.store_envelope(envelope1).await.unwrap();
        store.store_envelope(envelope2).await.unwrap();
        
        // Store third envelope should trigger eviction
        let envelope3 = DtnEnvelope::new(
            "did:polynet:source3".to_string(),
            "did:polynet:dest3".to_string(),
            3600,
            "test_message3".to_string(),
            b"test payload3".to_vec(),
            75, // Medium priority
        );
        
        store.store_envelope(envelope3).await.unwrap();
        
        let stats = store.get_stats().await;
        assert_eq!(stats.total_envelopes, 2); // Should have evicted one
        assert_eq!(stats.total_dropped, 1); // Should have dropped one
    }
}
