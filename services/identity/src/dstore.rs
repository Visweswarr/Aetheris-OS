use super::did::{DidDocument, DidKey, DidMetadata, DidError};
use dashmap::DashMap;
use sled::{Db, Tree};
use std::path::PathBuf;
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
    
    #[error("DID not found: {0}")]
    DidNotFound(String),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Storage full: {0}")]
    StorageFull(String),
}

#[derive(Debug, Clone)]
pub struct StoreConfig {
    pub max_dids: usize,
    pub max_keys_per_did: usize,
    pub cleanup_interval: u64,
    pub persistence_path: PathBuf,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            max_dids: 10000,
            max_keys_per_did: 100,
            cleanup_interval: 3600, // 1 hour
            persistence_path: PathBuf::from("data/did_store"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoreStats {
    pub total_dids: usize,
    pub total_keys: usize,
    pub total_documents: usize,
    pub last_cleanup: SystemTime,
}

pub struct DidStore {
    config: StoreConfig,
    db: Db,
    did_documents: DashMap<String, DidDocument>,
    did_keys: DashMap<String, Vec<DidKey>>,
    did_metadata: DashMap<String, DidMetadata>,
    stats: Arc<RwLock<StoreStats>>,
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
}

impl DidStore {
    /// Create a new DID store
    pub async fn new(persistence_path: PathBuf) -> Result<Self, StoreError> {
        let config = StoreConfig {
            persistence_path,
            ..Default::default()
        };
        
        info!("Creating DID store with config: {:?}", config);
        
        // Create database
        let db = sled::open(&config.persistence_path)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        let store = Self {
            config,
            db,
            did_documents: DashMap::new(),
            did_keys: DashMap::new(),
            did_metadata: DashMap::new(),
            stats: Arc::new(RwLock::new(StoreStats {
                total_dids: 0,
                total_keys: 0,
                total_documents: 0,
                last_cleanup: SystemTime::now(),
            })),
            cleanup_task: None,
        };
        
        // Start cleanup task
        store.start_cleanup_task();
        
        Ok(store)
    }
    
    /// Store a DID document
    pub async fn store_did_document(
        &self,
        did: &str,
        document: &DidDocument,
    ) -> Result<(), StoreError> {
        let did_id = did.to_string();
        
        // Check storage capacity
        if self.did_documents.len() >= self.config.max_dids {
            return Err(StoreError::StorageFull(
                format!("Maximum DIDs ({}) exceeded", self.config.max_dids)
            ));
        }
        
        // Store in memory
        self.did_documents.insert(did_id.clone(), document.clone());
        
        // Persist to database
        self.persist_did_document(did, document).await?;
        
        // Update metadata if not exists
        if !self.did_metadata.contains_key(&did_id) {
            let metadata = DidMetadata {
                created: document.created,
                updated: document.updated,
                version_id: document.version_id.clone(),
                next_update: document.next_update,
                deactivated: false,
                deactivated_reason: None,
            };
            self.did_metadata.insert(did_id.clone(), metadata);
        }
        
        // Update statistics
        self.update_stats_document_stored().await;
        
        info!("Stored DID document: {}", did);
        
        Ok(())
    }
    
    /// Retrieve a DID document
    pub async fn get_did_document(&self, did: &str) -> Result<Option<DidDocument>, StoreError> {
        // Check in-memory cache first
        if let Some(document) = self.did_documents.get(did) {
            return Ok(Some(document.clone()));
        }
        
        // Try to load from database
        if let Some(document) = self.load_did_document_from_db(did).await? {
            // Add to memory cache
            self.did_documents.insert(did.to_string(), document.clone());
            return Ok(Some(document));
        }
        
        Ok(None)
    }
    
    /// Store a DID key
    pub async fn store_did_key(
        &self,
        did: &str,
        key: &DidKey,
    ) -> Result<(), StoreError> {
        let did_id = did.to_string();
        
        // Check key limit per DID
        let current_keys = self.did_keys.get(&did_id).map(|keys| keys.len()).unwrap_or(0);
        if current_keys >= self.config.max_keys_per_did {
            return Err(StoreError::StorageFull(
                format!("Maximum keys per DID ({}) exceeded", self.config.max_keys_per_did)
            ));
        }
        
        // Get or create key list for this DID
        let mut keys = self.did_keys.get(&did_id)
            .map(|k| k.clone())
            .unwrap_or_default();
        
        // Add or update key
        if let Some(existing_key_index) = keys.iter().position(|k| k.id == key.id) {
            keys[existing_key_index] = key.clone();
        } else {
            keys.push(key.clone());
        }
        
        // Store updated key list
        self.did_keys.insert(did_id.clone(), keys);
        
        // Persist to database
        self.persist_did_key(did, key).await?;
        
        // Update statistics
        self.update_stats_key_stored().await;
        
        info!("Stored DID key: {} for DID: {}", key.id, did);
        
        Ok(())
    }
    
    /// Retrieve a DID key
    pub async fn get_did_key(&self, did: &str, key_id: &str) -> Result<Option<DidKey>, StoreError> {
        // Check in-memory cache first
        if let Some(keys) = self.did_keys.get(did) {
            if let Some(key) = keys.iter().find(|k| k.id == key_id) {
                return Ok(Some(key.clone()));
            }
        }
        
        // Try to load from database
        if let Some(key) = self.load_did_key_from_db(did, key_id).await? {
            // Add to memory cache
            self.store_did_key(did, &key).await?;
            return Ok(Some(key));
        }
        
        Ok(None)
    }
    
    /// Get all keys for a DID
    pub async fn get_did_keys(&self, did: &str) -> Result<Vec<DidKey>, StoreError> {
        // Check in-memory cache first
        if let Some(keys) = self.did_keys.get(did) {
            return Ok(keys.clone());
        }
        
        // Try to load from database
        let keys = self.load_did_keys_from_db(did).await?;
        
        // Add to memory cache
        if !keys.is_empty() {
            self.did_keys.insert(did.to_string(), keys.clone());
        }
        
        Ok(keys)
    }
    
    /// Revoke a DID key
    pub async fn revoke_did_key(&self, did: &str, key_id: &str) -> Result<(), StoreError> {
        let did_id = did.to_string();
        
        // Update in memory
        if let Some(mut keys) = self.did_keys.get_mut(&did_id) {
            if let Some(key) = keys.iter_mut().find(|k| k.id == key_id) {
                key.revoked = true;
                info!("Revoked DID key: {} for DID: {}", key_id, did);
            }
        }
        
        // Persist revocation to database
        self.persist_key_revocation(did, key_id).await?;
        
        Ok(())
    }
    
    /// Get DID metadata
    pub async fn get_did_metadata(&self, did: &str) -> Result<Option<DidMetadata>, StoreError> {
        // Check in-memory cache first
        if let Some(metadata) = self.did_metadata.get(did) {
            return Ok(Some(metadata.clone()));
        }
        
        // Try to load from database
        let metadata = self.load_did_metadata_from_db(did).await?;
        
        // Add to memory cache
        if let Some(ref meta) = metadata {
            self.did_metadata.insert(did.to_string(), meta.clone());
        }
        
        Ok(metadata)
    }
    
    /// Deactivate a DID
    pub async fn deactivate_did(&self, did: &str, reason: Option<String>) -> Result<(), StoreError> {
        let did_id = did.to_string();
        
        // Update metadata
        if let Some(mut metadata) = self.did_metadata.get_mut(&did_id) {
            metadata.deactivated = true;
            metadata.deactivated_reason = reason.clone();
            metadata.updated = chrono::Utc::now();
        } else {
            // Create metadata if it doesn't exist
            let metadata = DidMetadata {
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
                version_id: Uuid::new_v4().to_string(),
                next_update: None,
                deactivated: true,
                deactivated_reason: reason.clone(),
            };
            self.did_metadata.insert(did_id, metadata);
        }
        
        // Persist to database
        self.persist_did_deactivation(did, reason).await?;
        
        info!("Deactivated DID: {}", did);
        
        Ok(())
    }
    
    /// List all DIDs
    pub async fn list_dids(&self) -> Result<Vec<String>, StoreError> {
        let dids: Vec<String> = self.did_documents.keys().map(|k| k.clone()).collect();
        Ok(dids)
    }
    
    /// Get store statistics
    pub async fn get_stats(&self) -> StoreStats {
        self.stats.read().await.clone()
    }
    
    /// Clean up expired keys and documents
    pub async fn cleanup_expired(&self) -> Result<usize, StoreError> {
        let mut cleaned_count = 0;
        let now = chrono::Utc::now();
        
        // Clean up expired keys
        for entry in self.did_keys.iter() {
            let mut keys = entry.value().clone();
            let original_len = keys.len();
            
            keys.retain(|key| {
                if let Some(expires) = key.expires {
                    expires > now
                } else {
                    true // No expiration
                }
            });
            
            if keys.len() != original_len {
                self.did_keys.insert(entry.key().clone(), keys);
                cleaned_count += original_len - keys.len();
            }
        }
        
        // Update statistics
        if cleaned_count > 0 {
            let mut stats = self.stats.write().await;
            stats.last_cleanup = SystemTime::now();
        }
        
        info!("Cleaned up {} expired keys", cleaned_count);
        
        Ok(cleaned_count)
    }
    
    /// Persist DID document to database
    async fn persist_did_document(
        &self,
        did: &str,
        document: &DidDocument,
    ) -> Result<(), StoreError> {
        let document_data = serde_json::to_vec(document)
            .map_err(|e| StoreError::SerializationError(e.to_string()))?;
        
        let tree = self.db.open_tree("did_documents")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.insert(did.as_bytes(), document_data)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.flush()
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Load DID document from database
    async fn load_did_document_from_db(&self, did: &str) -> Result<Option<DidDocument>, StoreError> {
        let tree = self.db.open_tree("did_documents")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        if let Some(data) = tree.get(did.as_bytes())
            .map_err(|e| StoreError::DatabaseError(e.to_string()))? {
            let document: DidDocument = serde_json::from_slice(&data)
                .map_err(|e| StoreError::DeserializationError(e.to_string()))?;
            Ok(Some(document))
        } else {
            Ok(None)
        }
    }
    
    /// Persist DID key to database
    async fn persist_did_key(
        &self,
        did: &str,
        key: &DidKey,
    ) -> Result<(), StoreError> {
        let key_data = serde_json::to_vec(key)
            .map_err(|e| StoreError::SerializationError(e.to_string()))?;
        
        let tree = self.db.open_tree("did_keys")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        let key_path = format!("{}:{}", did, key.id);
        tree.insert(key_path.as_bytes(), key_data)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.flush()
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Load DID key from database
    async fn load_did_key_from_db(
        &self,
        did: &str,
        key_id: &str,
    ) -> Result<Option<DidKey>, StoreError> {
        let tree = self.db.open_tree("did_keys")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        let key_path = format!("{}:{}", did, key_id);
        if let Some(data) = tree.get(key_path.as_bytes())
            .map_err(|e| StoreError::DatabaseError(e.to_string()))? {
            let key: DidKey = serde_json::from_slice(&data)
                .map_err(|e| StoreError::DeserializationError(e.to_string()))?;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }
    
    /// Load all DID keys from database
    async fn load_did_keys_from_db(&self, did: &str) -> Result<Vec<DidKey>, StoreError> {
        let tree = self.db.open_tree("did_keys")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        let mut keys = Vec::new();
        let did_prefix = format!("{}:", did);
        
        for entry in tree.iter() {
            let (key, value) = entry
                .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
            
            let key_str = String::from_utf8_lossy(&key);
            if key_str.starts_with(&did_prefix) {
                let key_data: DidKey = serde_json::from_slice(&value)
                    .map_err(|e| StoreError::DeserializationError(e.to_string()))?;
                keys.push(key_data);
            }
        }
        
        Ok(keys)
    }
    
    /// Persist key revocation to database
    async fn persist_key_revocation(
        &self,
        did: &str,
        key_id: &str,
    ) -> Result<(), StoreError> {
        let tree = self.db.open_tree("revoked_keys")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        let key_path = format!("{}:{}", did, key_id);
        let revocation_data = serde_json::to_vec(&chrono::Utc::now())
            .map_err(|e| StoreError::SerializationError(e.to_string()))?;
        
        tree.insert(key_path.as_bytes(), revocation_data)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.flush()
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Load DID metadata from database
    async fn load_did_metadata_from_db(&self, did: &str) -> Result<Option<DidMetadata>, StoreError> {
        let tree = self.db.open_tree("did_metadata")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        if let Some(data) = tree.get(did.as_bytes())
            .map_err(|e| StoreError::DatabaseError(e.to_string()))? {
            let metadata: DidMetadata = serde_json::from_slice(&data)
                .map_err(|e| StoreError::DeserializationError(e.to_string()))?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }
    
    /// Persist DID deactivation to database
    async fn persist_did_deactivation(
        &self,
        did: &str,
        reason: Option<String>,
    ) -> Result<(), StoreError> {
        let tree = self.db.open_tree("deactivated_dids")
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        let deactivation_data = serde_json::to_vec(&(chrono::Utc::now(), reason))
            .map_err(|e| StoreError::SerializationError(e.to_string()))?;
        
        tree.insert(did.as_bytes(), deactivation_data)
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        tree.flush()
            .map_err(|e| StoreError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Update statistics for document stored
    async fn update_stats_document_stored(&self) {
        let mut stats = self.stats.write().await;
        stats.total_documents += 1;
        stats.total_dids = self.did_documents.len();
    }
    
    /// Update statistics for key stored
    async fn update_stats_key_stored(&self) {
        let mut stats = self.stats.write().await;
        stats.total_keys += 1;
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

impl Clone for DidStore {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            db: self.db.clone(),
            did_documents: self.did_documents.clone(),
            did_keys: self.did_keys.clone(),
            did_metadata: self.did_metadata.clone(),
            stats: Arc::clone(&self.stats),
            cleanup_task: None, // Cannot clone task handle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    async fn create_test_store() -> DidStore {
        let temp_dir = tempdir().unwrap();
        DidStore::new(temp_dir.path().to_path_buf()).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_store_creation() {
        let store = create_test_store().await;
        let stats = store.get_stats().await;
        
        assert_eq!(stats.total_dids, 0);
        assert_eq!(stats.total_keys, 0);
        assert_eq!(stats.total_documents, 0);
    }
    
    #[tokio::test]
    async fn test_store_did_document() {
        let store = create_test_store().await;
        
        let document = DidDocument {
            id: "did:polynet:test".to_string(),
            controller: None,
            verification_methods: Vec::new(),
            authentication: Vec::new(),
            assertion_method: Vec::new(),
            key_agreement: Vec::new(),
            key_encapsulation: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            services: Vec::new(),
            created: chrono::Utc::now(),
            updated: chrono::Utc::now(),
            version_id: "v1".to_string(),
            next_update: None,
            proof: None,
        };
        
        store.store_did_document("did:polynet:test", &document).await.unwrap();
        
        let stats = store.get_stats().await;
        assert_eq!(stats.total_documents, 1);
        assert_eq!(stats.total_dids, 1);
    }
    
    #[tokio::test]
    async fn test_retrieve_did_document() {
        let store = create_test_store().await;
        
        let document = DidDocument {
            id: "did:polynet:test".to_string(),
            controller: None,
            verification_methods: Vec::new(),
            authentication: Vec::new(),
            assertion_method: Vec::new(),
            key_agreement: Vec::new(),
            key_encapsulation: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            services: Vec::new(),
            created: chrono::Utc::now(),
            updated: chrono::Utc::now(),
            version_id: "v1".to_string(),
            next_update: None,
            proof: None,
        };
        
        store.store_did_document("did:polynet:test", &document).await.unwrap();
        
        let retrieved = store.get_did_document("did:polynet:test").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "did:polynet:test");
    }
    
    #[tokio::test]
    async fn test_store_did_key() {
        let store = create_test_store().await;
        
        let key = DidKey {
            id: "key1".to_string(),
            key_type: super::super::did::KeyType::Ed25519,
            public_key: vec![1, 2, 3, 4],
            private_key: Some(vec![5, 6, 7, 8]),
            created: chrono::Utc::now(),
            expires: None,
            revoked: false,
            purpose: vec![super::super::did::KeyPurpose::Authentication],
        };
        
        store.store_did_key("did:polynet:test", &key).await.unwrap();
        
        let stats = store.get_stats().await;
        assert_eq!(stats.total_keys, 1);
    }
    
    #[tokio::test]
    async fn test_retrieve_did_key() {
        let store = create_test_store().await;
        
        let key = DidKey {
            id: "key1".to_string(),
            key_type: super::super::did::KeyType::Ed25519,
            public_key: vec![1, 2, 3, 4],
            private_key: Some(vec![5, 6, 7, 8]),
            created: chrono::Utc::now(),
            expires: None,
            revoked: false,
            purpose: vec![super::super::did::KeyPurpose::Authentication],
        };
        
        store.store_did_key("did:polynet:test", &key).await.unwrap();
        
        let retrieved = store.get_did_key("did:polynet:test", "key1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "key1");
    }
    
    #[tokio::test]
    async fn test_revoke_did_key() {
        let store = create_test_store().await;
        
        let key = DidKey {
            id: "key1".to_string(),
            key_type: super::super::did::KeyType::Ed25519,
            public_key: vec![1, 2, 3, 4],
            private_key: Some(vec![5, 6, 7, 8]),
            created: chrono::Utc::now(),
            expires: None,
            revoked: false,
            purpose: vec![super::super::did::KeyPurpose::Authentication],
        };
        
        store.store_did_key("did:polynet:test", &key).await.unwrap();
        store.revoke_did_key("did:polynet:test", "key1").await.unwrap();
        
        let keys = store.get_did_keys("did:polynet:test").await.unwrap();
        assert!(keys.iter().all(|k| k.revoked));
    }
    
    #[tokio::test]
    async fn test_deactivate_did() {
        let store = create_test_store().await;
        
        let document = DidDocument {
            id: "did:polynet:test".to_string(),
            controller: None,
            verification_methods: Vec::new(),
            authentication: Vec::new(),
            assertion_method: Vec::new(),
            key_agreement: Vec::new(),
            key_encapsulation: Vec::new(),
            capability_invocation: Vec::new(),
            capability_delegation: Vec::new(),
            services: Vec::new(),
            created: chrono::Utc::now(),
            updated: chrono::Utc::now(),
            version_id: "v1".to_string(),
            next_update: None,
            proof: None,
        };
        
        store.store_did_document("did:polynet:test", &document).await.unwrap();
        store.deactivate_did("did:polynet:test", Some("Testing deactivation".to_string())).await.unwrap();
        
        let metadata = store.get_did_metadata("did:polynet:test").await.unwrap();
        assert!(metadata.unwrap().deactivated);
    }
}
