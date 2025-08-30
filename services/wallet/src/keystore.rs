use crate::backend::{BackendError, KeyBackend};
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use zeroize::Zeroizing;

pub mod backend;

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error("Backend error: {0}")]
    BackendError(#[from] BackendError),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Invalid key type: {0}")]
    InvalidKeyType(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Key expired: {0}")]
    KeyExpired(String),
    
    #[error("Key revoked: {0}")]
    KeyRevoked(String),
    
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    Ed25519,
    Dilithium3,
    Dilithium5,
    Kyber512,
    Kyber768,
    Kyber1024,
    Ed25519Dilithium3,  // Hybrid
    Ed25519Kyber512,    // Hybrid
    AES256,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    Sign,
    Verify,
    Encrypt,
    Decrypt,
    KeyEncapsulation,
    KeyDecapsulation,
    KeyAgreement,
    KeyDerivation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStatus {
    Active,
    Expired,
    Revoked,
    Compromised,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub id: String,
    pub name: Option<String>,
    pub key_type: KeyType,
    pub purposes: Vec<KeyPurpose>,
    pub created: DateTime<Utc>,
    pub expires: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub status: KeyStatus,
    pub tags: HashMap<String, String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub key_id: String,
    pub signature: Vec<u8>,
    pub algorithm: String,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub key_id: String,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub algorithm: String,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEncapsulationResult {
    pub key_id: String,
    pub encapsulated_key: Vec<u8>,
    pub shared_secret: Vec<u8>,
    pub algorithm: String,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationParams {
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub key_length: usize,
    pub algorithm: String,
}

/// Core keystore interface trait that defines unified cryptographic operations
#[async_trait::async_trait]
pub trait Keystore: Send + Sync {
    /// Get the backend identifier
    fn backend_id(&self) -> &str;
    
    /// Check if the keystore is available
    async fn is_available(&self) -> Result<bool, KeystoreError>;
    
    /// Generate a new key
    async fn generate_key(
        &self,
        key_type: KeyType,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, KeystoreError>;
    
    /// Import an existing key
    async fn import_key(
        &self,
        key_type: KeyType,
        key_data: Secret<Vec<u8>>,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, KeystoreError>;
    
    /// Get key metadata
    async fn get_key_metadata(&self, key_id: &str) -> Result<KeyMetadata, KeystoreError>;
    
    /// List all keys
    async fn list_keys(&self) -> Result<Vec<KeyMetadata>, KeystoreError>;
    
    /// Delete a key (with secure erasure)
    async fn delete_key(&self, key_id: &str) -> Result<(), KeystoreError>;
    
    /// Sign data
    async fn sign(
        &self,
        key_id: &str,
        data: &[u8],
        algorithm: Option<String>,
    ) -> Result<Signature, KeystoreError>;
    
    /// Verify signature
    async fn verify(
        &self,
        key_id: &str,
        data: &[u8],
        signature: &[u8],
        algorithm: Option<String>,
    ) -> Result<bool, KeystoreError>;
    
    /// Encrypt data
    async fn encrypt(
        &self,
        key_id: &str,
        data: &[u8],
        algorithm: Option<String>,
    ) -> Result<EncryptedData, KeystoreError>;
    
    /// Decrypt data
    async fn decrypt(
        &self,
        key_id: &str,
        encrypted_data: &EncryptedData,
    ) -> Result<Vec<u8>, KeystoreError>;
    
    /// Perform key encapsulation
    async fn encapsulate_key(
        &self,
        key_id: &str,
        algorithm: Option<String>,
    ) -> Result<KeyEncapsulationResult, KeystoreError>;
    
    /// Perform key decapsulation
    async fn decapsulate_key(
        &self,
        key_id: &str,
        encapsulated_key: &[u8],
        algorithm: Option<String>,
    ) -> Result<Vec<u8>, KeystoreError>;
    
    /// Derive a key
    async fn derive_key(
        &self,
        key_id: &str,
        params: &KeyDerivationParams,
    ) -> Result<Vec<u8>, KeystoreError>;
    
    /// Update key metadata
    async fn update_key_metadata(
        &self,
        key_id: &str,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, KeystoreError>;
    
    /// Revoke a key
    async fn revoke_key(&self, key_id: &str, reason: Option<String>) -> Result<(), KeystoreError>;
    
    /// Rotate a key
    async fn rotate_key(
        &self,
        key_id: &str,
        new_key_type: Option<KeyType>,
        new_purposes: Option<Vec<KeyPurpose>>,
    ) -> Result<KeyMetadata, KeystoreError>;
    
    /// Get keystore statistics
    async fn get_stats(&self) -> Result<KeystoreStats, KeystoreError>;
    
    /// Perform secure cleanup
    async fn cleanup(&self) -> Result<(), KeystoreError>;
}

/// Keystore statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub expired_keys: usize,
    pub revoked_keys: usize,
    pub total_operations: u64,
    pub last_operation: Option<DateTime<Utc>>,
    pub backend_id: String,
}

/// Keystore configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreConfig {
    pub backend: String,
    pub secure_erasure: bool,
    pub key_rotation_interval: Option<u64>,
    pub max_key_lifetime: Option<u64>,
    pub encryption_algorithm: String,
    pub signature_algorithm: String,
    pub key_derivation_algorithm: String,
}

impl Default for KeystoreConfig {
    fn default() -> Self {
        Self {
            backend: "software".to_string(),
            secure_erasure: true,
            key_rotation_interval: Some(30 * 24 * 60 * 60), // 30 days
            max_key_lifetime: Some(365 * 24 * 60 * 60), // 1 year
            encryption_algorithm: "AES256-GCM".to_string(),
            signature_algorithm: "Ed25519".to_string(),
            key_derivation_algorithm: "Argon2id".to_string(),
        }
    }
}

/// Keystore factory for creating different backend implementations
pub struct KeystoreFactory;

impl KeystoreFactory {
    /// Create a keystore with the specified backend
    pub async fn create_keystore(
        config: &KeystoreConfig,
    ) -> Result<Box<dyn Keystore>, KeystoreError> {
        match config.backend.as_str() {
            "software" => {
                let backend = crate::backend::software::SoftwareBackend::new(config.clone()).await?;
                Ok(Box::new(backend))
            }
            "keyvault" => {
                let backend = crate::backend::keyvault::KeyVaultBackend::new(config.clone()).await?;
                Ok(Box::new(backend))
            }
            _ => Err(KeystoreError::InvalidOperation(
                format!("Unsupported backend: {}", config.backend)
            )),
        }
    }
    
    /// Create a software keystore
    pub async fn create_software_keystore(
        config: Option<KeystoreConfig>,
    ) -> Result<Box<dyn Keystore>, KeystoreError> {
        let config = config.unwrap_or_default();
        let backend = crate::backend::software::SoftwareBackend::new(config).await?;
        Ok(Box::new(backend))
    }
    
    /// Create a KeyVault keystore
    pub async fn create_keyvault_keystore(
        config: Option<KeystoreConfig>,
    ) -> Result<Box<dyn Keystore>, KeystoreError> {
        let config = config.unwrap_or_default();
        let backend = crate::backend::keyvault::KeyVaultBackend::new(config).await?;
        Ok(Box::new(backend))
    }
}

/// Utility functions for keystore operations
pub mod utils {
    use super::*;
    
    /// Generate a secure random key ID
    pub fn generate_key_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Check if a key supports the specified purpose
    pub fn key_supports_purpose(key: &KeyMetadata, purpose: KeyPurpose) -> bool {
        key.purposes.contains(&purpose)
    }
    
    /// Check if a key is usable for the specified operation
    pub fn is_key_usable(key: &KeyMetadata) -> bool {
        key.status == KeyStatus::Active && 
        (key.expires.is_none() || key.expires.unwrap() > Utc::now())
    }
    
    /// Validate key type and purpose compatibility
    pub fn validate_key_type_purpose(
        key_type: KeyType,
        purposes: &[KeyPurpose],
    ) -> Result<(), KeystoreError> {
        for purpose in purposes {
            match (key_type, purpose) {
                // Sign/Verify compatible types
                (KeyType::Ed25519 | KeyType::Dilithium3 | KeyType::Dilithium5, 
                 KeyPurpose::Sign | KeyPurpose::Verify) => {},
                
                // Encrypt/Decrypt compatible types
                (KeyType::AES256 | KeyType::ChaCha20Poly1305, 
                 KeyPurpose::Encrypt | KeyPurpose::Decrypt) => {},
                
                // Key encapsulation compatible types
                (KeyType::Kyber512 | KeyType::Kyber768 | KeyType::Kyber1024, 
                 KeyPurpose::KeyEncapsulation | KeyPurpose::KeyDecapsulation) => {},
                
                // Hybrid types support multiple purposes
                (KeyType::Ed25519Dilithium3 | KeyType::Ed25519Kyber512, _) => {},
                
                // Incompatible combinations
                _ => {
                    return Err(KeystoreError::InvalidOperation(
                        format!("Key type {:?} is not compatible with purpose {:?}", key_type, purpose)
                    ));
                }
            }
        }
        Ok(())
    }
    
    /// Secure memory allocation with zeroization
    pub fn secure_alloc(size: usize) -> Zeroizing<Vec<u8>> {
        let mut data = vec![0u8; size];
        Zeroizing::new(data)
    }
    
    /// Secure memory deallocation with zeroization
    pub fn secure_dealloc(mut data: Zeroizing<Vec<u8>>) {
        // The Zeroizing wrapper will automatically zero the memory when dropped
        drop(data);
    }
    
    /// Constant-time comparison for security
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        
        result == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_type_purpose_validation() {
        // Valid combinations
        assert!(utils::validate_key_type_purpose(
            KeyType::Ed25519,
            &[KeyPurpose::Sign, KeyPurpose::Verify]
        ).is_ok());
        
        assert!(utils::validate_key_type_purpose(
            KeyType::AES256,
            &[KeyPurpose::Encrypt, KeyPurpose::Decrypt]
        ).is_ok());
        
        assert!(utils::validate_key_type_purpose(
            KeyType::Kyber512,
            &[KeyPurpose::KeyEncapsulation, KeyPurpose::KeyDecapsulation]
        ).is_ok());
        
        // Invalid combinations
        assert!(utils::validate_key_type_purpose(
            KeyType::Ed25519,
            &[KeyPurpose::Encrypt]
        ).is_err());
        
        assert!(utils::validate_key_type_purpose(
            KeyType::AES256,
            &[KeyPurpose::Sign]
        ).is_err());
    }
    
    #[test]
    fn test_key_metadata_utilities() {
        let key = KeyMetadata {
            id: "test-key".to_string(),
            name: Some("Test Key".to_string()),
            key_type: KeyType::Ed25519,
            purposes: vec![KeyPurpose::Sign, KeyPurpose::Verify],
            created: Utc::now(),
            expires: None,
            last_used: None,
            status: KeyStatus::Active,
            tags: HashMap::new(),
            backend: "software".to_string(),
        };
        
        assert!(utils::is_key_usable(&key));
        assert!(utils::key_supports_purpose(&key, KeyPurpose::Sign));
        assert!(!utils::key_supports_purpose(&key, KeyPurpose::Encrypt));
    }
    
    #[test]
    fn test_secure_memory_utilities() {
        let data = utils::secure_alloc(32);
        assert_eq!(data.len(), 32);
        
        // Test constant-time comparison
        let a = b"hello";
        let b = b"hello";
        let c = b"world";
        
        assert!(utils::constant_time_eq(a, b));
        assert!(!utils::constant_time_eq(a, c));
    }
}
