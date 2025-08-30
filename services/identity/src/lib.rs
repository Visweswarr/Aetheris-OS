//! DID Identity Service for Polymera OS
//! 
//! This module provides Decentralized Identifier (DID) functionality with:
//! - Post-quantum cryptography support (Dilithium, Kyber)
//! - Hybrid key schemes (Ed25519 + PQ)
//! - Local DID document storage
//! - Key rotation and management
//! - DID resolution and verification

pub mod did;
pub mod dstore;

// Re-export main types
pub use did::{
    DidIdentity, DidKey, DidDocument, DidMetadata,
    KeyType, KeyPurpose, VerificationMethod, Service, Proof,
    DidError,
};
pub use dstore::{DidStore, StoreConfig, StoreStats, StoreError};

use std::sync::Arc;
use tracing::{info, warn, error};

/// Main DID identity service that coordinates all operations
pub struct IdentityService {
    store: Arc<DidStore>,
    identity: Arc<DidIdentity>,
}

impl IdentityService {
    /// Create a new identity service
    pub async fn new(store_path: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let store = Arc::new(DidStore::new(store_path).await?);
        let identity = Arc::new(DidIdentity::new(store.clone()));
        
        info!("DID Identity service initialized");
        
        Ok(Self {
            store,
            identity,
        })
    }
    
    /// Create a new DID
    pub async fn create_did(
        &self,
        method: &str,
        key_types: Vec<KeyType>,
        purposes: Vec<KeyPurpose>,
    ) -> Result<(String, DidDocument), Box<dyn std::error::Error>> {
        self.identity.create_did(method, key_types, purposes).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    
    /// Resolve a DID
    pub async fn resolve_did(&self, did: &str) -> Result<Option<DidDocument>, Box<dyn std::error::Error>> {
        self.identity.resolve_did(did).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    
    /// Rotate keys for a DID
    pub async fn rotate_keys(
        &self,
        did: &str,
        key_ids: Vec<String>,
        new_key_types: Vec<KeyType>,
        new_purposes: Vec<KeyPurpose>,
    ) -> Result<DidDocument, Box<dyn std::error::Error>> {
        self.identity.rotate_keys(did, key_ids, new_key_types, new_purposes).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    
    /// Get all keys for a DID
    pub async fn get_did_keys(&self, did: &str) -> Result<Vec<DidKey>, Box<dyn std::error::Error>> {
        self.identity.get_did_keys(did).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    
    /// Deactivate a DID
    pub async fn deactivate_did(&self, did: &str, reason: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        self.identity.deactivate_did(did, reason).await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    
    /// Get service statistics
    pub async fn get_stats(&self) -> Result<ServiceStats, Box<dyn std::error::Error>> {
        let store_stats = self.store.get_stats().await;
        
        let stats = ServiceStats {
            store: store_stats,
        };
        
        Ok(stats)
    }
    
    /// List all DIDs
    pub async fn list_dids(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.store.list_dids().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
    
    /// Clean up expired keys
    pub async fn cleanup_expired(&self) -> Result<usize, Box<dyn std::error::Error>> {
        self.store.cleanup_expired().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// Combined service statistics
#[derive(Debug, Clone)]
pub struct ServiceStats {
    pub store: StoreStats,
}

/// Configuration for the identity service
#[derive(Debug, Clone)]
pub struct IdentityServiceConfig {
    pub store_path: std::path::PathBuf,
    pub max_dids: usize,
    pub max_keys_per_did: usize,
    pub cleanup_interval: u64,
}

impl Default for IdentityServiceConfig {
    fn default() -> Self {
        Self {
            store_path: std::path::PathBuf::from("data/identity"),
            max_dids: 10000,
            max_keys_per_did: 100,
            cleanup_interval: 3600, // 1 hour
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    async fn create_test_service() -> IdentityService {
        let temp_dir = tempdir().unwrap();
        IdentityService::new(temp_dir.path().to_path_buf()).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_service_creation() {
        let service = create_test_service().await;
        
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_dids, 0);
        assert_eq!(stats.store.total_keys, 0);
        assert_eq!(stats.store.total_documents, 0);
    }
    
    #[tokio::test]
    async fn test_create_and_resolve_did() {
        let service = create_test_service().await;
        
        // Create DID
        let (did, doc) = service.create_did(
            "polynet",
            vec![KeyType::Ed25519, KeyType::Dilithium3],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion],
        ).await.unwrap();
        
        assert!(did.starts_with("did:polynet:"));
        assert_eq!(doc.verification_methods.len(), 2);
        
        // Resolve DID
        let resolved = service.resolve_did(&did).await.unwrap();
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().id, did);
    }
    
    #[tokio::test]
    async fn test_key_rotation() {
        let service = create_test_service().await;
        
        // Create DID
        let (did, _) = service.create_did(
            "polynet",
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        // Get original keys
        let original_keys = service.get_did_keys(&did).await.unwrap();
        let key_ids: Vec<String> = original_keys.iter().map(|k| k.id.clone()).collect();
        
        // Rotate keys
        let updated_doc = service.rotate_keys(
            &did,
            key_ids,
            vec![KeyType::Dilithium3],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        assert_eq!(updated_doc.verification_methods.len(), 1);
        
        // Verify old keys are revoked
        let keys = service.get_did_keys(&did).await.unwrap();
        assert!(keys.iter().all(|k| k.revoked));
    }
    
    #[tokio::test]
    async fn test_did_deactivation() {
        let service = create_test_service().await;
        
        // Create DID
        let (did, _) = service.create_did(
            "polynet",
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        // Deactivate DID
        service.deactivate_did(&did, Some("Testing deactivation".to_string())).await.unwrap();
        
        // Verify deactivation
        let metadata = service.store.get_did_metadata(&did).await.unwrap();
        assert!(metadata.unwrap().deactivated);
    }
    
    #[tokio::test]
    async fn test_list_dids() {
        let service = create_test_service().await;
        
        // Create multiple DIDs
        let (did1, _) = service.create_did(
            "polynet",
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        let (did2, _) = service.create_did(
            "polynet",
            vec![KeyType::Dilithium3],
            vec![KeyPurpose::Assertion],
        ).await.unwrap();
        
        // List DIDs
        let dids = service.list_dids().await.unwrap();
        assert_eq!(dids.len(), 2);
        assert!(dids.contains(&did1));
        assert!(dids.contains(&did2));
    }
    
    #[tokio::test]
    async fn test_cleanup_expired() {
        let service = create_test_service().await;
        
        // Create DID with expiring keys
        let (did, _) = service.create_did(
            "polynet",
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        // Cleanup should not remove any keys (none expired)
        let cleaned = service.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 0);
        
        let stats = service.get_stats().await.unwrap();
        assert_eq!(stats.store.total_keys, 1);
    }
}
