//! Development Keyvault Service for Polymera OS
//! 
//! This service provides development-only file-backed persistence for issuer anchors
//! and session metadata with encryption at rest using a development master key.
//! 
//! **WARNING: This is NOT for production use!**

pub mod crypto;
pub mod store;

pub use crypto::{
    DevMasterKey, EncryptedData, CryptoError,
    init_dev_master_key, get_dev_master_key, rotate_dev_master_key,
    export_master_key, import_master_key,
    encrypt_data, decrypt_data,
};

pub use store::{
    DevKeyVaultStore, IssuerAnchorData, SessionMetadataData, VaultMetadata, VaultStats, StoreError,
    init_dev_keyvault_store, get_dev_keyvault_store,
};

use std::path::PathBuf;
use thiserror::Error;

/// Development keyvault service errors
#[derive(Error, Debug)]
pub enum DevKeyVaultError {
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StoreError),
    
    #[error("Service not initialized")]
    NotInitialized,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Development keyvault service configuration
#[derive(Debug, Clone)]
pub struct DevKeyVaultConfig {
    /// Base directory for storage
    pub base_dir: PathBuf,
    /// Whether to enable automatic key rotation
    pub auto_key_rotation: bool,
    /// Key rotation interval in seconds
    pub key_rotation_interval: u64,
    /// Whether to enable compression
    pub enable_compression: bool,
    /// Maximum storage size in bytes
    pub max_storage_size: u64,
}

impl Default for DevKeyVaultConfig {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("/tmp/polymera-dev-keyvault"),
            auto_key_rotation: true,
            key_rotation_interval: 86400, // 24 hours
            enable_compression: false,
            max_storage_size: 100 * 1024 * 1024, // 100 MB
        }
    }
}

/// Development keyvault service
pub struct DevKeyVaultService {
    config: DevKeyVaultConfig,
    initialized: bool,
}

impl DevKeyVaultService {
    /// Create a new development keyvault service
    pub fn new(config: DevKeyVaultConfig) -> Self {
        Self {
            config,
            initialized: false,
        }
    }
    
    /// Initialize the service
    pub async fn init(&mut self) -> Result<(), DevKeyVaultError> {
        // Initialize the crypto system
        crypto::init_dev_master_key()?;
        
        // Initialize the storage system
        store::init_dev_keyvault_store(self.config.base_dir.clone()).await?;
        
        self.initialized = true;
        
        Ok(())
    }
    
    /// Check if the service is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Store an issuer anchor
    pub async fn store_issuer_anchor(&self, anchor: IssuerAnchorData) -> Result<(), DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        store.store_issuer_anchor(anchor).await?;
        
        Ok(())
    }
    
    /// Load an issuer anchor
    pub async fn load_issuer_anchor(&self, did: &str) -> Result<Option<IssuerAnchorData>, DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        let anchor = store.load_issuer_anchor(did).await?;
        
        Ok(anchor)
    }
    
    /// Store session metadata
    pub async fn store_session_metadata(&self, metadata: SessionMetadataData) -> Result<(), DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        store.store_session_metadata(metadata).await?;
        
        Ok(())
    }
    
    /// Load session metadata
    pub async fn load_session_metadata(&self, session_id: &str) -> Result<Option<SessionMetadataData>, DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        let metadata = store.load_session_metadata(session_id).await?;
        
        Ok(metadata)
    }
    
    /// List all issuer anchors
    pub async fn list_issuer_anchors(&self) -> Result<Vec<String>, DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        let anchors = store.list_issuer_anchors().await?;
        
        Ok(anchors)
    }
    
    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<String>, DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        let sessions = store.list_sessions().await?;
        
        Ok(sessions)
    }
    
    /// Delete an issuer anchor
    pub async fn delete_issuer_anchor(&self, did: &str) -> Result<(), DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        store.delete_issuer_anchor(did).await?;
        
        Ok(())
    }
    
    /// Delete session metadata
    pub async fn delete_session_metadata(&self, session_id: &str) -> Result<(), DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        store.delete_session_metadata(session_id).await?;
        
        Ok(())
    }
    
    /// Rotate the master key
    pub async fn rotate_master_key(&self) -> Result<(), DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        // Rotate the crypto master key
        crypto::rotate_dev_master_key()?;
        
        // Re-encrypt all stored data with the new key
        self.re_encrypt_all_data().await?;
        
        Ok(())
    }
    
    /// Export the master key
    pub fn export_master_key(&self) -> Result<Vec<u8>, DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let key_data = crypto::export_master_key()?;
        Ok(key_data)
    }
    
    /// Import a master key
    pub async fn import_master_key(&self, key_data: &[u8]) -> Result<(), DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        // Import the new master key
        crypto::import_master_key(key_data)?;
        
        // Re-encrypt all stored data with the new key
        self.re_encrypt_all_data().await?;
        
        Ok(())
    }
    
    /// Get service statistics
    pub async fn get_stats(&self) -> Result<VaultStats, DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        let stats = store.get_stats()?;
        
        Ok(stats)
    }
    
    /// Clear all data (for testing)
    pub async fn clear_all(&self) -> Result<(), DevKeyVaultError> {
        if !self.initialized {
            return Err(DevKeyVaultError::NotInitialized);
        }
        
        let store = store::get_dev_keyvault_store()?;
        store.clear_all().await?;
        
        Ok(())
    }
    
    /// Re-encrypt all data with the current master key
    async fn re_encrypt_all_data(&self) -> Result<(), DevKeyVaultError> {
        // This is a simplified implementation
        // In a real system, you'd want to handle this more carefully
        
        let store = store::get_dev_keyvault_store()?;
        
        // Re-encrypt issuer anchors
        let anchor_dids = store.list_issuer_anchors().await?;
        for did in anchor_dids {
            if let Ok(Some(anchor)) = store.load_issuer_anchor(&did).await {
                store.store_issuer_anchor(anchor).await?;
            }
        }
        
        // Re-encrypt session metadata
        let session_ids = store.list_sessions().await?;
        for session_id in session_ids {
            if let Ok(Some(metadata)) = store.load_session_metadata(&session_id).await {
                store.store_session_metadata(metadata).await?;
            }
        }
        
        Ok(())
    }
}

/// Global development keyvault service instance
static mut DEV_KEYVAULT_SERVICE: Option<DevKeyVaultService> = None;

/// Initialize the global development keyvault service
pub async fn init_dev_keyvault_service(config: DevKeyVaultConfig) -> Result<(), DevKeyVaultError> {
    unsafe {
        if DEV_KEYVAULT_SERVICE.is_none() {
            let mut service = DevKeyVaultService::new(config);
            service.init().await?;
            DEV_KEYVAULT_SERVICE = Some(service);
        }
        Ok(())
    }
}

/// Get the global development keyvault service
pub fn get_dev_keyvault_service() -> Result<&'static DevKeyVaultService, DevKeyVaultError> {
    unsafe {
        if let Some(service) = &DEV_KEYVAULT_SERVICE {
            Ok(service)
        } else {
            Err(DevKeyVaultError::NotInitialized)
        }
    }
}

/// Get a mutable reference to the global development keyvault service
pub fn get_dev_keyvault_service_mut() -> Result<&'static mut DevKeyVaultService, DevKeyVaultError> {
    unsafe {
        if let Some(service) = &mut DEV_KEYVAULT_SERVICE {
            Ok(service)
        } else {
            Err(DevKeyVaultError::NotInitialized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_service_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = DevKeyVaultConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let mut service = DevKeyVaultService::new(config);
        assert!(!service.is_initialized());
        
        service.init().await.unwrap();
        assert!(service.is_initialized());
    }
    
    #[tokio::test]
    async fn test_issuer_anchor_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = DevKeyVaultConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let mut service = DevKeyVaultService::new(config);
        service.init().await.unwrap();
        
        let anchor = IssuerAnchorData {
            did: "did:polymera:dev:test".to_string(),
            public_key_data: vec![1, 2, 3, 4],
            parameter_set: "Dilithium2".to_string(),
            trust_anchor: "dev".to_string(),
            ttl_seconds: 3600,
            created_at: 1234567890,
            last_accessed: 1234567890,
        };
        
        service.store_issuer_anchor(anchor.clone()).await.unwrap();
        
        let loaded = service.load_issuer_anchor("did:polymera:dev:test").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().did, anchor.did);
    }
    
    #[tokio::test]
    async fn test_session_metadata_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = DevKeyVaultConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let mut service = DevKeyVaultService::new(config);
        service.init().await.unwrap();
        
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
        
        service.store_session_metadata(metadata.clone()).await.unwrap();
        
        let loaded = service.load_session_metadata("session_123").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().session_id, metadata.session_id);
    }
}

