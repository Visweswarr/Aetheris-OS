use crate::keystore::{
    KeyType, KeyPurpose, KeyStatus, KeyMetadata, Signature, EncryptedData,
    KeyEncapsulationResult, KeyDerivationParams, KeystoreStats,
};
use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use zeroize::Zeroizing;

pub mod software;
pub mod keyvault;

#[derive(Debug, Error)]
pub enum BackendError {
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
    
    #[error("Cryptographic error: {0}")]
    CryptographicError(String),
    
    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Internal key representation for backend implementations
#[derive(Debug, Clone)]
pub struct InternalKey {
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
    
    // Secure key material (encrypted in memory)
    pub encrypted_key_material: Zeroizing<Vec<u8>>,
    pub encryption_nonce: Vec<u8>,
    pub encryption_algorithm: String,
}

impl InternalKey {
    /// Create a new internal key
    pub fn new(
        id: String,
        key_type: KeyType,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
        backend: String,
        encrypted_key_material: Zeroizing<Vec<u8>>,
        encryption_nonce: Vec<u8>,
        encryption_algorithm: String,
    ) -> Self {
        Self {
            id,
            name,
            key_type,
            purposes,
            created: Utc::now(),
            expires,
            last_used: None,
            status: KeyStatus::Active,
            tags: tags.unwrap_or_default(),
            backend,
            encrypted_key_material,
            encryption_nonce,
            encryption_algorithm,
        }
    }
    
    /// Convert to public metadata
    pub fn to_metadata(&self) -> KeyMetadata {
        KeyMetadata {
            id: self.id.clone(),
            name: self.name.clone(),
            key_type: self.key_type,
            purposes: self.purposes.clone(),
            created: self.created,
            expires: self.expires,
            last_used: self.last_used,
            status: self.status,
            tags: self.tags.clone(),
            backend: self.backend.clone(),
        }
    }
    
    /// Check if key is usable
    pub fn is_usable(&self) -> bool {
        self.status == KeyStatus::Active && 
        (self.expires.is_none() || self.expires.unwrap() > Utc::now())
    }
    
    /// Check if key supports purpose
    pub fn supports_purpose(&self, purpose: KeyPurpose) -> bool {
        self.purposes.contains(&purpose)
    }
    
    /// Update last used timestamp
    pub fn update_last_used(&mut self) {
        self.last_used = Some(Utc::now());
    }
    
    /// Revoke key
    pub fn revoke(&mut self) {
        self.status = KeyStatus::Revoked;
    }
    
    /// Mark as compromised
    pub fn mark_compromised(&mut self) {
        self.status = KeyStatus::Compromised;
    }
}

/// Backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub backend_id: String,
    pub secure_erasure: bool,
    pub encryption_algorithm: String,
    pub key_rotation_interval: Option<u64>,
    pub max_key_lifetime: Option<u64>,
    pub storage_path: Option<String>,
    pub connection_string: Option<String>,
    pub authentication: Option<BackendAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendAuth {
    pub auth_type: String,
    pub credentials: HashMap<String, String>,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            backend_id: "software".to_string(),
            secure_erasure: true,
            encryption_algorithm: "AES256-GCM".to_string(),
            key_rotation_interval: Some(30 * 24 * 60 * 60), // 30 days
            max_key_lifetime: Some(365 * 24 * 60 * 60), // 1 year
            storage_path: Some("data/keystore".to_string()),
            connection_string: None,
            authentication: None,
        }
    }
}

/// Common backend interface for key operations
#[async_trait::async_trait]
pub trait KeyBackend: Send + Sync {
    /// Get backend identifier
    fn backend_id(&self) -> &str;
    
    /// Check if backend is available
    async fn is_available(&self) -> Result<bool, BackendError>;
    
    /// Initialize backend
    async fn initialize(&mut self) -> Result<(), BackendError>;
    
    /// Shutdown backend
    async fn shutdown(&mut self) -> Result<(), BackendError>;
    
    /// Generate a new key
    async fn generate_key(
        &mut self,
        key_type: KeyType,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, BackendError>;
    
    /// Import an existing key
    async fn import_key(
        &mut self,
        key_type: KeyType,
        key_data: Secret<Vec<u8>>,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, BackendError>;
    
    /// Get key metadata
    async fn get_key_metadata(&self, key_id: &str) -> Result<KeyMetadata, BackendError>;
    
    /// List all keys
    async fn list_keys(&self) -> Result<Vec<KeyMetadata>, BackendError>;
    
    /// Delete a key (with secure erasure)
    async fn delete_key(&mut self, key_id: &str) -> Result<(), BackendError>;
    
    /// Sign data
    async fn sign(
        &mut self,
        key_id: &str,
        data: &[u8],
        algorithm: Option<String>,
    ) -> Result<Signature, BackendError>;
    
    /// Verify signature
    async fn verify(
        &self,
        key_id: &str,
        data: &[u8],
        signature: &[u8],
        algorithm: Option<String>,
    ) -> Result<bool, BackendError>;
    
    /// Encrypt data
    async fn encrypt(
        &mut self,
        key_id: &str,
        data: &[u8],
        algorithm: Option<String>,
    ) -> Result<EncryptedData, BackendError>;
    
    /// Decrypt data
    async fn decrypt(
        &mut self,
        key_id: &str,
        encrypted_data: &EncryptedData,
    ) -> Result<Vec<u8>, BackendError>;
    
    /// Perform key encapsulation
    async fn encapsulate_key(
        &mut self,
        key_id: &str,
        algorithm: Option<String>,
    ) -> Result<KeyEncapsulationResult, BackendError>;
    
    /// Perform key decapsulation
    async fn decapsulate_key(
        &mut self,
        key_id: &str,
        encapsulated_key: &[u8],
        algorithm: Option<String>,
    ) -> Result<Vec<u8>, BackendError>;
    
    /// Derive a key
    async fn derive_key(
        &mut self,
        key_id: &str,
        params: &KeyDerivationParams,
    ) -> Result<Vec<u8>, BackendError>;
    
    /// Update key metadata
    async fn update_key_metadata(
        &mut self,
        key_id: &str,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, BackendError>;
    
    /// Revoke a key
    async fn revoke_key(&mut self, key_id: &str, reason: Option<String>) -> Result<(), BackendError>;
    
    /// Rotate a key
    async fn rotate_key(
        &mut self,
        key_id: &str,
        new_key_type: Option<KeyType>,
        new_purposes: Option<Vec<KeyPurpose>>,
    ) -> Result<KeyMetadata, BackendError>;
    
    /// Get backend statistics
    async fn get_stats(&self) -> Result<KeystoreStats, BackendError>;
    
    /// Perform secure cleanup
    async fn cleanup(&mut self) -> Result<(), BackendError>;
}

/// Backend statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub expired_keys: usize,
    pub revoked_keys: usize,
    pub total_operations: u64,
    pub last_operation: Option<DateTime<Utc>>,
    pub backend_id: String,
    pub backend_specific: HashMap<String, String>,
}

/// Utility functions for backend implementations
pub mod utils {
    use super::*;
    use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use aes_gcm::aead::{Aead, NewAead};
    use rand::Rng;
    
    /// Generate a secure random key ID
    pub fn generate_key_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Generate a secure random nonce
    pub fn generate_nonce() -> Vec<u8> {
        let mut nonce = vec![0u8; 12];
        rand::thread_rng().fill(&mut nonce);
        nonce
    }
    
    /// Generate a secure random salt
    pub fn generate_salt() -> Vec<u8> {
        let mut salt = vec![0u8; 32];
        rand::thread_rng().fill(&mut salt);
        salt
    }
    
    /// Encrypt key material with a master key
    pub fn encrypt_key_material(
        key_material: &[u8],
        master_key: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>), BackendError> {
        let key = Key::from_slice(master_key);
        let cipher = Aes256Gcm::new(key)
            .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
        
        let nonce = generate_nonce();
        let nonce_bytes = Nonce::from_slice(&nonce);
        
        let encrypted = cipher.encrypt(nonce_bytes, key_material)
            .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
        
        Ok((encrypted, nonce))
    }
    
    /// Decrypt key material with a master key
    pub fn decrypt_key_material(
        encrypted_material: &[u8],
        master_key: &[u8],
        nonce: &[u8],
    ) -> Result<Vec<u8>, BackendError> {
        let key = Key::from_slice(master_key);
        let cipher = Aes256Gcm::new(key)
            .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
        
        let nonce_bytes = Nonce::from_slice(nonce);
        
        let decrypted = cipher.decrypt(nonce_bytes, encrypted_material)
            .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
        
        Ok(decrypted)
    }
    
    /// Derive a key using Argon2
    pub fn derive_key_argon2(
        password: &[u8],
        salt: &[u8],
        iterations: u32,
        key_length: usize,
    ) -> Result<Vec<u8>, BackendError> {
        let salt_string = SaltString::b64_encode(salt)
            .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
        
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                iterations,
                1024, // memory cost
                1,    // parallelism
                Some(key_length as u32),
            ).map_err(|e| BackendError::CryptographicError(e.to_string()))?,
        );
        
        let hash = argon2.hash_password(password, &salt_string)
            .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
        
        let hash_bytes = hash.hash.unwrap().as_bytes().to_vec();
        Ok(hash_bytes[..key_length].to_vec())
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
    
    /// Validate key type and purpose compatibility
    pub fn validate_key_type_purpose(
        key_type: KeyType,
        purposes: &[KeyPurpose],
    ) -> Result<(), BackendError> {
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
                    return Err(BackendError::InvalidOperation(
                        format!("Key type {:?} is not compatible with purpose {:?}", key_type, purpose)
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_internal_key_creation() {
        let key = InternalKey::new(
            "test-key".to_string(),
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            Some("Test Key".to_string()),
            None,
            None,
            "software".to_string(),
            Zeroizing::new(vec![1, 2, 3, 4]),
            vec![5, 6, 7, 8],
            "AES256-GCM".to_string(),
        );
        
        assert_eq!(key.id, "test-key");
        assert_eq!(key.key_type, KeyType::Ed25519);
        assert!(key.is_usable());
        assert!(key.supports_purpose(KeyPurpose::Sign));
    }
    
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
        
        // Invalid combinations
        assert!(utils::validate_key_type_purpose(
            KeyType::Ed25519,
            &[KeyPurpose::Encrypt]
        ).is_err());
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
