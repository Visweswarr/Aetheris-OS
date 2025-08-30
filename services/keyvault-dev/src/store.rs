//! Persistent storage for the development keyvault
//! 
//! This module provides file-backed persistence for issuer anchors and session metadata
//! with encryption at rest using the development master key.

use crate::crypto::{CryptoError, EncryptedData, encrypt_data, decrypt_data};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Storage errors
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),
    
    #[error("Storage not initialized")]
    NotInitialized,
}

/// Issuer anchor data for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerAnchorData {
    /// DID identifier
    pub did: String,
    /// Dilithium public key (serialized)
    pub public_key_data: Vec<u8>,
    /// Parameter set identifier
    pub parameter_set: String,
    /// Trust anchor identifier
    pub trust_anchor: String,
    /// TTL in seconds
    pub ttl_seconds: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed: u64,
}

/// Session metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadataData {
    /// Session key identifier
    pub session_id: String,
    /// Kyber public key (serialized)
    pub public_key_data: Vec<u8>,
    /// Parameter set identifier
    pub parameter_set: String,
    /// Associated issuer key ID
    pub issuer_key_id: Option<String>,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Last used timestamp
    pub last_used: u64,
    /// Message count
    pub message_count: u64,
    /// Whether the session is active
    pub active: bool,
}

/// Vault metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    /// Vault version
    pub version: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modification timestamp
    pub last_modified: u64,
    /// Master key identifier
    pub master_key_id: String,
    /// Number of issuer anchors
    pub issuer_anchor_count: usize,
    /// Number of session metadata entries
    pub session_metadata_count: usize,
}

/// Development keyvault store
pub struct DevKeyVaultStore {
    /// Base directory for storage
    base_dir: PathBuf,
    /// Vault metadata
    metadata: Mutex<VaultMetadata>,
    /// Issuer anchors
    issuer_anchors: Mutex<HashMap<String, IssuerAnchorData>>,
    /// Session metadata
    session_metadata: Mutex<HashMap<String, SessionMetadataData>>,
    /// Whether the store is initialized
    initialized: Mutex<bool>,
}

impl DevKeyVaultStore {
    /// Create a new development keyvault store
    pub fn new(base_dir: PathBuf) -> Self {
        let metadata = VaultMetadata {
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now().timestamp() as u64,
            last_modified: chrono::Utc::now().timestamp() as u64,
            master_key_id: String::new(),
            issuer_anchor_count: 0,
            session_metadata_count: 0,
        };
        
        Self {
            base_dir,
            metadata: Mutex::new(metadata),
            issuer_anchors: Mutex::new(HashMap::new()),
            session_metadata: Mutex::new(HashMap::new()),
            initialized: Mutex::new(false),
        }
    }
    
    /// Initialize the store
    pub async fn init(&self) -> Result<(), StoreError> {
        // Create base directory if it doesn't exist
        fs::create_dir_all(&self.base_dir).await?;
        
        // Create subdirectories
        let anchors_dir = self.base_dir.join("anchors");
        let sessions_dir = self.base_dir.join("sessions");
        let metadata_dir = self.base_dir.join("metadata");
        
        fs::create_dir_all(&anchors_dir).await?;
        fs::create_dir_all(&sessions_dir).await?;
        fs::create_dir_all(&metadata_dir).await?;
        
        // Load existing data if available
        self.load_vault_data().await?;
        
        // Update metadata
        {
            let mut metadata = self.metadata.lock().unwrap();
            metadata.last_modified = chrono::Utc::now().timestamp() as u64;
        }
        
        *self.initialized.lock().unwrap() = true;
        
        Ok(())
    }
    
    /// Check if the store is initialized
    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }
    
    /// Store an issuer anchor
    pub async fn store_issuer_anchor(&self, anchor: IssuerAnchorData) -> Result<(), StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        // Encrypt the anchor data
        let encrypted = encrypt_data(&anchor)?;
        
        // Serialize the encrypted data
        let serialized = serde_json::to_vec(&encrypted)?;
        
        // Write to file
        let filename = format!("{}.enc", anchor.did.replace(":", "_"));
        let filepath = self.base_dir.join("anchors").join(filename);
        
        let mut file = fs::File::create(&filepath).await?;
        file.write_all(&serialized).await?;
        file.flush().await?;
        
        // Update in-memory storage
        {
            let mut anchors = self.issuer_anchors.lock().unwrap();
            anchors.insert(anchor.did.clone(), anchor);
        }
        
        // Update metadata
        self.update_metadata().await?;
        
        Ok(())
    }
    
    /// Load an issuer anchor
    pub async fn load_issuer_anchor(&self, did: &str) -> Result<Option<IssuerAnchorData>, StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        // Check in-memory storage first
        {
            let anchors = self.issuer_anchors.lock().unwrap();
            if let Some(anchor) = anchors.get(did) {
                return Ok(Some(anchor.clone()));
            }
        }
        
        // Try to load from file
        let filename = format!("{}.enc", did.replace(":", "_"));
        let filepath = self.base_dir.join("anchors").join(filename);
        
        if !filepath.exists() {
            return Ok(None);
        }
        
        // Read and decrypt the data
        let mut file = fs::File::open(&filepath).await?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).await?;
        
        let encrypted: EncryptedData = serde_json::from_slice(&encrypted_data)?;
        let anchor: IssuerAnchorData = decrypt_data(&encrypted)?;
        
        // Update in-memory storage
        {
            let mut anchors = self.issuer_anchors.lock().unwrap();
            anchors.insert(did.to_string(), anchor.clone());
        }
        
        Ok(Some(anchor))
    }
    
    /// Store session metadata
    pub async fn store_session_metadata(&self, metadata: SessionMetadataData) -> Result<(), StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        // Encrypt the metadata
        let encrypted = encrypt_data(&metadata)?;
        
        // Serialize the encrypted data
        let serialized = serde_json::to_vec(&encrypted)?;
        
        // Write to file
        let filename = format!("{}.enc", metadata.session_id);
        let filepath = self.base_dir.join("sessions").join(filename);
        
        let mut file = fs::File::create(&filepath).await?;
        file.write_all(&serialized).await?;
        file.flush().await?;
        
        // Update in-memory storage
        {
            let mut sessions = self.session_metadata.lock().unwrap();
            sessions.insert(metadata.session_id.clone(), metadata);
        }
        
        // Update metadata
        self.update_metadata().await?;
        
        Ok(())
    }
    
    /// Load session metadata
    pub async fn load_session_metadata(&self, session_id: &str) -> Result<Option<SessionMetadataData>, StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        // Check in-memory storage first
        {
            let sessions = self.session_metadata.lock().unwrap();
            if let Some(metadata) = sessions.get(session_id) {
                return Ok(Some(metadata.clone()));
            }
        }
        
        // Try to load from file
        let filename = format!("{}.enc", session_id);
        let filepath = self.base_dir.join("sessions").join(filename);
        
        if !filepath.exists() {
            return Ok(None);
        }
        
        // Read and decrypt the data
        let mut file = fs::File::open(&filepath).await?;
        let mut encrypted_data = Vec::new();
        file.read_to_end(&mut encrypted_data).await?;
        
        let encrypted: EncryptedData = serde_json::from_slice(&encrypted_data)?;
        let metadata: SessionMetadataData = decrypt_data(&encrypted)?;
        
        // Update in-memory storage
        {
            let mut sessions = self.session_metadata.lock().unwrap();
            sessions.insert(session_id.to_string(), metadata.clone());
        }
        
        Ok(Some(metadata))
    }
    
    /// List all issuer anchor DIDs
    pub async fn list_issuer_anchors(&self) -> Result<Vec<String>, StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        let anchors_dir = self.base_dir.join("anchors");
        let mut entries = fs::read_dir(anchors_dir).await?;
        let mut dids = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "enc" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(stem_str) = stem.to_str() {
                            // Convert filename back to DID format
                            let did = stem_str.replace("_", ":");
                            dids.push(did);
                        }
                    }
                }
            }
        }
        
        Ok(dids)
    }
    
    /// List all session IDs
    pub async fn list_sessions(&self) -> Result<Vec<String>, StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        let sessions_dir = self.base_dir.join("sessions");
        let mut entries = fs::read_dir(sessions_dir).await?;
        let mut session_ids = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension == "enc" {
                    if let Some(stem) = path.file_stem() {
                        if let Some(stem_str) = stem.to_str() {
                            session_ids.push(stem_str.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(session_ids)
    }
    
    /// Delete an issuer anchor
    pub async fn delete_issuer_anchor(&self, did: &str) -> Result<(), StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        // Remove from in-memory storage
        {
            let mut anchors = self.issuer_anchors.lock().unwrap();
            anchors.remove(did);
        }
        
        // Remove file
        let filename = format!("{}.enc", did.replace(":", "_"));
        let filepath = self.base_dir.join("anchors").join(filename);
        
        if filepath.exists() {
            fs::remove_file(filepath).await?;
        }
        
        // Update metadata
        self.update_metadata().await?;
        
        Ok(())
    }
    
    /// Delete session metadata
    pub async fn delete_session_metadata(&self, session_id: &str) -> Result<(), StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        // Remove from in-memory storage
        {
            let mut sessions = self.session_metadata.lock().unwrap();
            sessions.remove(session_id);
        }
        
        // Remove file
        let filename = format!("{}.enc", session_id);
        let filepath = self.base_dir.join("sessions").join(filename);
        
        if filepath.exists() {
            fs::remove_file(filepath).await?;
        }
        
        // Update metadata
        self.update_metadata().await?;
        
        Ok(())
    }
    
    /// Load all vault data
    async fn load_vault_data(&self) -> Result<(), StoreError> {
        // Load issuer anchors
        let anchor_dids = self.list_issuer_anchors().await?;
        for did in anchor_dids {
            if let Ok(Some(anchor)) = self.load_issuer_anchor(&did).await {
                let mut anchors = self.issuer_anchors.lock().unwrap();
                anchors.insert(did, anchor);
            }
        }
        
        // Load session metadata
        let session_ids = self.list_sessions().await?;
        for session_id in session_ids {
            if let Ok(Some(metadata)) = self.load_session_metadata(&session_id).await {
                let mut sessions = self.session_metadata.lock().unwrap();
                sessions.insert(session_id, metadata);
            }
        }
        
        Ok(())
    }
    
    /// Update vault metadata
    async fn update_metadata(&self) -> Result<(), StoreError> {
        let mut metadata = self.metadata.lock().unwrap();
        metadata.last_modified = chrono::Utc::now().timestamp() as u64;
        metadata.issuer_anchor_count = self.issuer_anchors.lock().unwrap().len();
        metadata.session_metadata_count = self.session_metadata.lock().unwrap().len();
        
        // Store updated metadata
        let metadata_file = self.base_dir.join("metadata").join("vault.json");
        let serialized = serde_json::to_vec_pretty(&*metadata)?;
        
        let mut file = fs::File::create(metadata_file).await?;
        file.write_all(&serialized).await?;
        file.flush().await?;
        
        Ok(())
    }
    
    /// Get vault statistics
    pub fn get_stats(&self) -> Result<VaultStats, StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        let anchors = self.issuer_anchors.lock().unwrap();
        let sessions = self.session_metadata.lock().unwrap();
        
        Ok(VaultStats {
            issuer_anchor_count: anchors.len(),
            session_metadata_count: sessions.len(),
            total_storage_size: self.calculate_storage_size(),
        })
    }
    
    /// Calculate total storage size
    fn calculate_storage_size(&self) -> u64 {
        // This is a simplified calculation
        // In a real implementation, you'd want to actually measure file sizes
        let anchors = self.issuer_anchors.lock().unwrap();
        let sessions = self.session_metadata.lock().unwrap();
        
        (anchors.len() + sessions.len()) as u64 * 1024 // Estimate 1KB per entry
    }
    
    /// Clear all data (for testing)
    pub async fn clear_all(&self) -> Result<(), StoreError> {
        if !self.is_initialized() {
            return Err(StoreError::NotInitialized);
        }
        
        // Clear in-memory storage
        {
            let mut anchors = self.issuer_anchors.lock().unwrap();
            anchors.clear();
        }
        {
            let mut sessions = self.session_metadata.lock().unwrap();
            sessions.clear();
        }
        
        // Remove all files
        let anchors_dir = self.base_dir.join("anchors");
        let sessions_dir = self.base_dir.join("sessions");
        
        if anchors_dir.exists() {
            fs::remove_dir_all(&anchors_dir).await?;
            fs::create_dir_all(&anchors_dir).await?;
        }
        
        if sessions_dir.exists() {
            fs::remove_dir_all(&sessions_dir).await?;
            fs::create_dir_all(&sessions_dir).await?;
        }
        
        // Update metadata
        self.update_metadata().await?;
        
        Ok(())
    }
}

/// Vault statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStats {
    /// Number of issuer anchors
    pub issuer_anchor_count: usize,
    /// Number of session metadata entries
    pub session_metadata_count: usize,
    /// Total storage size in bytes (estimated)
    pub total_storage_size: u64,
}

/// Global development keyvault store instance
static mut DEV_KEYVAULT_STORE: Option<Mutex<DevKeyVaultStore>> = None;

/// Initialize the global development keyvault store
pub async fn init_dev_keyvault_store(base_dir: PathBuf) -> Result<(), StoreError> {
    unsafe {
        if DEV_KEYVAULT_STORE.is_none() {
            let store = DevKeyVaultStore::new(base_dir);
            DEV_KEYVAULT_STORE = Some(Mutex::new(store));
        }
        
        if let Some(store_mutex) = &DEV_KEYVAULT_STORE {
            let store = store_mutex.lock().unwrap();
            store.init().await
        } else {
            Err(StoreError::NotInitialized)
        }
    }
}

/// Get the global development keyvault store
pub fn get_dev_keyvault_store() -> Result<std::sync::MutexGuard<DevKeyVaultStore>, StoreError> {
    unsafe {
        if let Some(store_mutex) = &DEV_KEYVAULT_STORE {
            store_mutex.lock().map_err(|_| {
                StoreError::NotInitialized
            })
        } else {
            Err(StoreError::NotInitialized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_store_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = DevKeyVaultStore::new(temp_dir.path().to_path_buf());
        
        assert!(!store.is_initialized());
        store.init().await.unwrap();
        assert!(store.is_initialized());
    }
    
    #[tokio::test]
    async fn test_issuer_anchor_storage() {
        let temp_dir = TempDir::new().unwrap();
        let store = DevKeyVaultStore::new(temp_dir.path().to_path_buf());
        store.init().await.unwrap();
        
        let anchor = IssuerAnchorData {
            did: "did:polymera:dev:test".to_string(),
            public_key_data: vec![1, 2, 3, 4],
            parameter_set: "Dilithium2".to_string(),
            trust_anchor: "dev".to_string(),
            ttl_seconds: 3600,
            created_at: 1234567890,
            last_accessed: 1234567890,
        };
        
        store.store_issuer_anchor(anchor.clone()).await.unwrap();
        
        let loaded = store.load_issuer_anchor("did:polymera:dev:test").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().did, anchor.did);
    }
    
    #[tokio::test]
    async fn test_session_metadata_storage() {
        let temp_dir = TempDir::new().unwrap();
        let store = DevKeyVaultStore::new(temp_dir.path().to_path_buf());
        store.init().await.unwrap();
        
        let metadata = SessionMetadataData {
            session_id: "session_123".to_string(),
            public_key_data: vec![5, 6, 7, 8],
            parameter_set: "Kyber512".to_string(),
            issuer_key_id: Some("issuer_456".to_string()),
            created_at: 1234567890,
            expires_at: 1234567890 + 3600,
            last_used: 1234567890,
            message_count: 42,
            active: true,
        };
        
        store.store_session_metadata(metadata.clone()).await.unwrap();
        
        let loaded = store.load_session_metadata("session_123").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().session_id, metadata.session_id);
    }
}

