//! Cryptographic operations for the development keyvault
//! 
//! This module provides AES-GCM encryption for persisting sensitive data
//! with a development master key. This is NOT for production use.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use thiserror::Error;

/// Cryptographic errors
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    
    #[error("Invalid nonce format: {0}")]
    InvalidNonceFormat(String),
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}

/// Encrypted data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Encrypted data
    pub ciphertext: Vec<u8>,
    /// Nonce used for encryption
    pub nonce: Vec<u8>,
    /// Associated data (if any)
    pub associated_data: Option<Vec<u8>>,
}

/// Development master key (NOT for production use)
#[derive(Debug, Clone)]
pub struct DevMasterKey {
    /// AES-256-GCM key
    key: Key<Aes256Gcm>,
    /// Key identifier
    id: String,
    /// Creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
}

impl DevMasterKey {
    /// Create a new development master key
    pub fn new() -> Result<Self, CryptoError> {
        let key = Aes256Gcm::generate_key(OsRng);
        let id = Self::generate_key_id();
        let created_at = chrono::Utc::now();
        
        Ok(Self {
            key,
            id,
            created_at,
        })
    }
    
    /// Load a master key from bytes
    pub fn from_bytes(key_bytes: &[u8]) -> Result<Self, CryptoError> {
        if key_bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyFormat(
                "Master key must be exactly 32 bytes".to_string()
            ));
        }
        
        let key = Key::<Aes256Gcm>::from_slice(key_bytes)
            .map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))?;
        
        let id = Self::generate_key_id();
        let created_at = chrono::Utc::now();
        
        Ok(Self {
            key,
            id,
            created_at,
        })
    }
    
    /// Generate a random key identifier
    fn generate_key_id() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
    
    /// Get the key identifier
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get the creation timestamp
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
    
    /// Export the key as bytes (for persistence)
    pub fn export(&self) -> Vec<u8> {
        self.key.to_vec()
    }
    
    /// Encrypt data with this master key
    pub fn encrypt<T>(&self, data: &T) -> Result<EncryptedData, CryptoError>
    where
        T: Serialize,
    {
        // Serialize the data
        let plaintext = serde_json::to_vec(data)
            .map_err(|e| CryptoError::SerializationFailed(e.to_string()))?;
        
        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the data
        let cipher = Aes256Gcm::new(&self.key);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
        
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            associated_data: None,
        })
    }
    
    /// Decrypt data with this master key
    pub fn decrypt<T>(&self, encrypted_data: &EncryptedData) -> Result<T, CryptoError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Reconstruct the nonce
        let nonce = Nonce::from_slice(&encrypted_data.nonce)
            .map_err(|e| CryptoError::InvalidNonceFormat(e.to_string()))?;
        
        // Decrypt the data
        let cipher = Aes256Gcm::new(&self.key);
        let plaintext = cipher
            .decrypt(nonce, encrypted_data.ciphertext.as_ref())
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
        
        // Deserialize the data
        let data: T = serde_json::from_slice(&plaintext)
            .map_err(|e| CryptoError::DeserializationFailed(e.to_string()))?;
        
        Ok(data)
    }
    
    /// Rotate the master key
    pub fn rotate(&mut self) -> Result<(), CryptoError> {
        let new_key = Aes256Gcm::generate_key(OsRng);
        let new_id = Self::generate_key_id();
        
        self.key = new_key;
        self.id = new_id;
        self.created_at = chrono::Utc::now();
        
        Ok(())
    }
}

/// Global development master key instance
static mut DEV_MASTER_KEY: Option<Mutex<DevMasterKey>> = None;

/// Initialize the development master key
pub fn init_dev_master_key() -> Result<(), CryptoError> {
    unsafe {
        if DEV_MASTER_KEY.is_none() {
            let master_key = DevMasterKey::new()?;
            DEV_MASTER_KEY = Some(Mutex::new(master_key));
        }
        Ok(())
    }
}

/// Get the development master key
pub fn get_dev_master_key() -> Result<std::sync::MutexGuard<DevMasterKey>, CryptoError> {
    unsafe {
        if let Some(key_mutex) = &DEV_MASTER_KEY {
            key_mutex.lock().map_err(|_| {
                CryptoError::KeyDerivationFailed("Failed to acquire master key lock".to_string())
            })
        } else {
            Err(CryptoError::KeyDerivationFailed(
                "Development master key not initialized".to_string()
            ))
        }
    }
}

/// Rotate the development master key
pub fn rotate_dev_master_key() -> Result<(), CryptoError> {
    let mut master_key = get_dev_master_key()?;
    master_key.rotate()
}

/// Export the current master key
pub fn export_master_key() -> Result<Vec<u8>, CryptoError> {
    let master_key = get_dev_master_key()?;
    Ok(master_key.export())
}

/// Import a master key
pub fn import_master_key(key_bytes: &[u8]) -> Result<(), CryptoError> {
    let new_master_key = DevMasterKey::from_bytes(key_bytes)?;
    
    unsafe {
        if let Some(key_mutex) = &DEV_MASTER_KEY {
            if let Ok(mut key_guard) = key_mutex.lock() {
                *key_guard = new_master_key;
                Ok(())
            } else {
                Err(CryptoError::KeyDerivationFailed(
                    "Failed to acquire master key lock".to_string()
                ))
            }
        } else {
            Err(CryptoError::KeyDerivationFailed(
                "Development master key not initialized".to_string()
            ))
        }
    }
}

/// Encrypt data using the development master key
pub fn encrypt_data<T>(data: &T) -> Result<EncryptedData, CryptoError>
where
    T: Serialize,
{
    let master_key = get_dev_master_key()?;
    master_key.encrypt(data)
}

/// Decrypt data using the development master key
pub fn decrypt_data<T>(encrypted_data: &EncryptedData) -> Result<T, CryptoError>
where
    T: for<'de> Deserialize<'de>,
{
    let master_key = get_dev_master_key()?;
    master_key.decrypt(encrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        message: String,
        number: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    }
    
    #[test]
    fn test_master_key_creation() {
        let master_key = DevMasterKey::new().unwrap();
        assert_eq!(master_key.key.to_vec().len(), 32);
        assert!(!master_key.id().is_empty());
    }
    
    #[test]
    fn test_master_key_import_export() {
        let master_key = DevMasterKey::new().unwrap();
        let exported = master_key.export();
        
        let imported = DevMasterKey::from_bytes(&exported).unwrap();
        assert_eq!(exported, imported.export());
    }
    
    #[test]
    fn test_encryption_decryption() {
        let master_key = DevMasterKey::new().unwrap();
        
        let test_data = TestData {
            message: "Hello, World!".to_string(),
            number: 42,
            timestamp: chrono::Utc::now(),
        };
        
        let encrypted = master_key.encrypt(&test_data).unwrap();
        let decrypted: TestData = master_key.decrypt(&encrypted).unwrap();
        
        assert_eq!(test_data, decrypted);
    }
    
    #[test]
    fn test_key_rotation() {
        let mut master_key = DevMasterKey::new().unwrap();
        let original_id = master_key.id().to_string();
        let original_created = master_key.created_at();
        
        master_key.rotate().unwrap();
        
        assert_ne!(master_key.id(), original_id);
        assert!(master_key.created_at() > original_created);
    }
    
    #[test]
    fn test_global_master_key() {
        init_dev_master_key().unwrap();
        
        let test_data = TestData {
            message: "Global test".to_string(),
            number: 123,
            timestamp: chrono::Utc::now(),
        };
        
        let encrypted = encrypt_data(&test_data).unwrap();
        let decrypted: TestData = decrypt_data(&encrypted).unwrap();
        
        assert_eq!(test_data, decrypted);
    }
}

