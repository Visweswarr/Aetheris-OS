//! Keystore Interface Service for Polymera OS
//! 
//! This module provides a unified keystore abstraction with:
//! - Post-quantum cryptography support (Dilithium, Kyber)
//! - Hybrid key schemes (Ed25519 + PQ)
//! - Multiple backend implementations (Software, KeyVault)
//! - Secure memory management with zeroization
//! - Comprehensive cryptographic operations

pub mod keystore;
pub mod backend;
pub mod session;
pub mod intent_summarize;

// Re-export main types
pub use keystore::{
    Keystore, KeystoreError, KeystoreConfig, KeystoreFactory,
    KeyType, KeyPurpose, KeyStatus, KeyMetadata,
    Signature, EncryptedData, KeyEncapsulationResult, KeyDerivationParams,
    KeystoreStats,
};

// Re-export session types
pub use session::{
    SessionKey, SessionType, SessionScope, TimeConstraints, AmountConstraints,
    GeographicConstraints, NetworkConstraints, DeviceConstraints, DataConstraints,
    SecurityInfo, SessionMetadata, SessionState, UsageStats,
    SessionKeyValidator, ValidationContext, SessionKeyError,
};

// Re-export intent summarize types
pub use intent_summarize::{
    IntentSummarizer, TransactionIntent, IntentOperation, IntentSummary,
    TokenInfo, TokenAmount, GasEstimate, NetworkInfo, RiskLevel,
    IntentSummaryError,
};
pub use backend::{
    KeyBackend, BackendError, BackendConfig, BackendAuth,
    software::SoftwareBackend,
    keyvault::KeyVaultBackend,
};

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Main keystore service that coordinates operations across backends
pub struct KeystoreService {
    backends: Arc<RwLock<Vec<Box<dyn Keystore>>>>,
    default_backend: String,
    config: KeystoreConfig,
}

impl KeystoreService {
    /// Create a new keystore service
    pub async fn new(config: KeystoreConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let mut backends = Vec::new();
        
        // Create default backend
        let default_backend = config.backend.clone();
        let backend = KeystoreFactory::create_keystore(&config).await?;
        backends.push(backend);
        
        let service = Self {
            backends: Arc::new(RwLock::new(backends)),
            default_backend,
            config,
        };
        
        info!("Keystore service initialized with backend: {}", default_backend);
        
        Ok(service)
    }
    
    /// Add a new backend
    pub async fn add_backend(&self, backend: Box<dyn Keystore>) -> Result<(), Box<dyn std::error::Error>> {
        let mut backends = self.backends.write().await;
        backends.push(backend);
        info!("Added new backend: {}", backends.last().unwrap().backend_id());
        Ok(())
    }
    
    /// Get the default backend
    pub async fn get_default_backend(&self) -> Result<Box<dyn Keystore>, Box<dyn std::error::Error>> {
        let backends = self.backends.read().await;
        if let Some(backend) = backends.first() {
            // Clone the backend (this would need to be implemented properly in a real scenario)
            Ok(KeystoreFactory::create_keystore(&self.config).await?)
        } else {
            Err("No backends available".into())
        }
    }
    
    /// Generate a new key using the default backend
    pub async fn generate_key(
        &self,
        key_type: KeyType,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<chrono::DateTime<chrono::Utc>>,
        tags: Option<std::collections::HashMap<String, String>>,
    ) -> Result<KeyMetadata, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let metadata = backend.generate_key(key_type, purposes, name, expires, tags).await?;
        info!("Generated key: {} (type: {:?})", metadata.id, metadata.key_type);
        Ok(metadata)
    }
    
    /// Import an existing key using the default backend
    pub async fn import_key(
        &self,
        key_type: KeyType,
        key_data: secrecy::Secret<Vec<u8>>,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<chrono::DateTime<chrono::Utc>>,
        tags: Option<std::collections::HashMap<String, String>>,
    ) -> Result<KeyMetadata, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let metadata = backend.import_key(key_type, key_data, purposes, name, expires, tags).await?;
        info!("Imported key: {} (type: {:?})", metadata.id, metadata.key_type);
        Ok(metadata)
    }
    
    /// Sign data using the default backend
    pub async fn sign(
        &self,
        key_id: &str,
        data: &[u8],
        algorithm: Option<String>,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let signature = backend.sign(key_id, data, algorithm).await?;
        info!("Signed data with key: {}", key_id);
        Ok(signature)
    }
    
    /// Verify signature using the default backend
    pub async fn verify(
        &self,
        key_id: &str,
        data: &[u8],
        signature: &[u8],
        algorithm: Option<String>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let is_valid = backend.verify(key_id, data, signature, algorithm).await?;
        info!("Verified signature with key: {} - valid: {}", key_id, is_valid);
        Ok(is_valid)
    }
    
    /// Encrypt data using the default backend
    pub async fn encrypt(
        &self,
        key_id: &str,
        data: &[u8],
        algorithm: Option<String>,
    ) -> Result<EncryptedData, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let encrypted = backend.encrypt(key_id, data, algorithm).await?;
        info!("Encrypted data with key: {}", key_id);
        Ok(encrypted)
    }
    
    /// Decrypt data using the default backend
    pub async fn decrypt(
        &self,
        key_id: &str,
        encrypted_data: &EncryptedData,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let decrypted = backend.decrypt(key_id, encrypted_data).await?;
        info!("Decrypted data with key: {}", key_id);
        Ok(decrypted)
    }
    
    /// Perform key encapsulation using the default backend
    pub async fn encapsulate_key(
        &self,
        key_id: &str,
        algorithm: Option<String>,
    ) -> Result<KeyEncapsulationResult, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let result = backend.encapsulate_key(key_id, algorithm).await?;
        info!("Encapsulated key: {}", key_id);
        Ok(result)
    }
    
    /// Perform key decapsulation using the default backend
    pub async fn decapsulate_key(
        &self,
        key_id: &str,
        encapsulated_key: &[u8],
        algorithm: Option<String>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let shared_secret = backend.decapsulate_key(key_id, encapsulated_key, algorithm).await?;
        info!("Decapsulated key: {}", key_id);
        Ok(shared_secret)
    }
    
    /// Derive a key using the default backend
    pub async fn derive_key(
        &self,
        key_id: &str,
        params: &KeyDerivationParams,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let derived_key = backend.derive_key(key_id, params).await?;
        info!("Derived key from: {}", key_id);
        Ok(derived_key)
    }
    
    /// Get key metadata using the default backend
    pub async fn get_key_metadata(&self, key_id: &str) -> Result<KeyMetadata, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let metadata = backend.get_key_metadata(key_id).await?;
        Ok(metadata)
    }
    
    /// List all keys using the default backend
    pub async fn list_keys(&self) -> Result<Vec<KeyMetadata>, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let keys = backend.list_keys().await?;
        Ok(keys)
    }
    
    /// Delete a key using the default backend
    pub async fn delete_key(&self, key_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        backend.delete_key(key_id).await?;
        info!("Deleted key: {}", key_id);
        Ok(())
    }
    
    /// Update key metadata using the default backend
    pub async fn update_key_metadata(
        &self,
        key_id: &str,
        name: Option<String>,
        expires: Option<chrono::DateTime<chrono::Utc>>,
        tags: Option<std::collections::HashMap<String, String>>,
    ) -> Result<KeyMetadata, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let metadata = backend.update_key_metadata(key_id, name, expires, tags).await?;
        info!("Updated metadata for key: {}", key_id);
        Ok(metadata)
    }
    
    /// Revoke a key using the default backend
    pub async fn revoke_key(&self, key_id: &str, reason: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        backend.revoke_key(key_id, reason).await?;
        info!("Revoked key: {} (reason: {:?})", key_id, reason);
        Ok(())
    }
    
    /// Rotate a key using the default backend
    pub async fn rotate_key(
        &self,
        key_id: &str,
        new_key_type: Option<KeyType>,
        new_purposes: Option<Vec<KeyPurpose>>,
    ) -> Result<KeyMetadata, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let new_metadata = backend.rotate_key(key_id, new_key_type, new_purposes).await?;
        info!("Rotated key: {} -> {}", key_id, new_metadata.id);
        Ok(new_metadata)
    }
    
    /// Get service statistics
    pub async fn get_stats(&self) -> Result<KeystoreStats, Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        let stats = backend.get_stats().await?;
        Ok(stats)
    }
    
    /// Perform cleanup using the default backend
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let backend = self.get_default_backend().await?;
        backend.cleanup().await?;
        info!("Keystore cleanup completed");
        Ok(())
    }
    
    /// Get service configuration
    pub fn get_config(&self) -> &KeystoreConfig {
        &self.config
    }
    
    /// Get default backend identifier
    pub fn get_default_backend_id(&self) -> &str {
        &self.default_backend
    }
}

/// Utility functions for keystore operations
pub mod utils {
    use super::*;
    
    /// Create a software keystore service
    pub async fn create_software_keystore(
        config: Option<KeystoreConfig>,
    ) -> Result<KeystoreService, Box<dyn std::error::Error>> {
        let mut config = config.unwrap_or_default();
        config.backend = "software".to_string();
        
        KeystoreService::new(config).await
    }
    
    /// Create a KeyVault keystore service
    pub async fn create_keyvault_keystore(
        vault_url: String,
        tenant_id: String,
        client_id: String,
        config: Option<KeystoreConfig>,
    ) -> Result<KeystoreService, Box<dyn std::error::Error>> {
        let mut config = config.unwrap_or_default();
        config.backend = "keyvault".to_string();
        config.connection_string = Some(vault_url);
        
        let mut auth = std::collections::HashMap::new();
        auth.insert("tenant_id".to_string(), tenant_id);
        auth.insert("client_id".to_string(), client_id);
        config.authentication = Some(BackendAuth {
            auth_type: "service_principal".to_string(),
            credentials: auth,
        });
        
        KeystoreService::new(config).await
    }
    
    /// Validate keystore configuration
    pub fn validate_config(config: &KeystoreConfig) -> Result<(), Box<dyn std::error::Error>> {
        if config.backend.is_empty() {
            return Err("Backend identifier cannot be empty".into());
        }
        
        if config.encryption_algorithm.is_empty() {
            return Err("Encryption algorithm cannot be empty".into());
        }
        
        if config.signature_algorithm.is_empty() {
            return Err("Signature algorithm cannot be empty".into());
        }
        
        if config.key_derivation_algorithm.is_empty() {
            return Err("Key derivation algorithm cannot be empty".into());
        }
        
        Ok(())
    }
    
    /// Generate a secure random key ID
    pub fn generate_key_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    
    /// Check if a key supports the specified purpose
    pub fn key_supports_purpose(key: &KeyMetadata, purpose: KeyPurpose) -> bool {
        key.purposes.contains(&purpose)
    }
    
    /// Check if a key is usable
    pub fn is_key_usable(key: &KeyMetadata) -> bool {
        key.status == KeyStatus::Active && 
        (key.expires.is_none() || key.expires.unwrap() > chrono::Utc::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    async fn create_test_service() -> KeystoreService {
        let config = KeystoreConfig {
            backend: "software".to_string(),
            storage_path: Some("test_data/keystore".to_string()),
            ..Default::default()
        };
        
        KeystoreService::new(config).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_service_creation() {
        let service = create_test_service().await;
        assert_eq!(service.get_default_backend_id(), "software");
    }
    
    #[tokio::test]
    async fn test_key_generation() {
        let service = create_test_service().await;
        
        let metadata = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            Some("Test Key".to_string()),
            None,
            None,
        ).await.unwrap();
        
        assert_eq!(metadata.key_type, KeyType::Ed25519);
        assert!(metadata.purposes.contains(&KeyPurpose::Sign));
        assert!(metadata.purposes.contains(&KeyPurpose::Verify));
    }
    
    #[tokio::test]
    async fn test_sign_and_verify() {
        let service = create_test_service().await;
        
        // Generate signing key
        let metadata = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            None,
            None,
            None,
        ).await.unwrap();
        
        let data = b"Hello, World!";
        
        // Sign data
        let signature = service.sign(&metadata.id, data, None).await.unwrap();
        assert_eq!(signature.key_id, metadata.id);
        
        // Verify signature
        let is_valid = service.verify(&metadata.id, data, &signature.signature, None).await.unwrap();
        assert!(is_valid);
    }
    
    #[tokio::test]
    async fn test_encrypt_and_decrypt() {
        let service = create_test_service().await;
        
        // Generate encryption key
        let metadata = service.generate_key(
            KeyType::AES256,
            vec![KeyPurpose::Encrypt, KeyPurpose::Decrypt],
            None,
            None,
            None,
        ).await.unwrap();
        
        let data = b"Secret message";
        
        // Encrypt data
        let encrypted = service.encrypt(&metadata.id, data, None).await.unwrap();
        assert_eq!(encrypted.key_id, metadata.id);
        
        // Decrypt data
        let decrypted = service.decrypt(&metadata.id, &encrypted).await.unwrap();
        assert_eq!(decrypted, data);
    }
    
    #[tokio::test]
    async fn test_key_encapsulation() {
        let service = create_test_service().await;
        
        // Generate KEM key
        let metadata = service.generate_key(
            KeyType::Kyber512,
            vec![KeyPurpose::KeyEncapsulation, KeyPurpose::KeyDecapsulation],
            None,
            None,
            None,
        ).await.unwrap();
        
        // Encapsulate key
        let result = service.encapsulate_key(&metadata.id, None).await.unwrap();
        assert_eq!(result.key_id, metadata.id);
        
        // Decapsulate key
        let shared_secret = service.decapsulate_key(&metadata.id, &result.encapsulated_key, None).await.unwrap();
        assert_eq!(shared_secret, result.shared_secret);
    }
    
    #[test]
    fn test_utility_functions() {
        let key_id = utils::generate_key_id();
        assert!(!key_id.is_empty());
        
        let key = KeyMetadata {
            id: "test-key".to_string(),
            name: Some("Test Key".to_string()),
            key_type: KeyType::Ed25519,
            purposes: vec![KeyPurpose::Sign, KeyPurpose::Verify],
            created: chrono::Utc::now(),
            expires: None,
            last_used: None,
            status: KeyStatus::Active,
            tags: std::collections::HashMap::new(),
            backend: "software".to_string(),
        };
        
        assert!(utils::is_key_usable(&key));
        assert!(utils::key_supports_purpose(&key, KeyPurpose::Sign));
        assert!(!utils::key_supports_purpose(&key, KeyPurpose::Encrypt));
    }
}
