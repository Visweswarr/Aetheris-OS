use super::{
    BackendError, BackendConfig, InternalKey, KeyBackend, BackendStats,
    utils::{generate_key_id, generate_nonce, encrypt_key_material, decrypt_key_material},
};
use crate::keystore::{
    KeyType, KeyPurpose, KeyStatus, KeyMetadata, Signature, EncryptedData,
    KeyEncapsulationResult, KeyDerivationParams, KeystoreStats,
};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use secrecy::{ExposeSecret, Secret};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn, error};
use uuid::Uuid;
use zeroize::Zeroizing;

/// Software-based keystore backend
pub struct SoftwareBackend {
    config: BackendConfig,
    keys: DashMap<String, InternalKey>,
    master_key: Zeroizing<Vec<u8>>,
    stats: Arc<dashmap::DashMap<String, u64>>,
    storage_path: PathBuf,
    initialized: bool,
}

impl SoftwareBackend {
    /// Create a new software backend
    pub async fn new(config: BackendConfig) -> Result<Self, BackendError> {
        let storage_path = config.storage_path
            .as_ref()
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|| PathBuf::from("data/keystore"));
        
        // Create storage directory if it doesn't exist
        std::fs::create_dir_all(&storage_path)
            .map_err(|e| BackendError::StorageError(e.to_string()))?;
        
        // Generate or load master key
        let master_key = Self::get_or_generate_master_key(&storage_path).await?;
        
        let backend = Self {
            config,
            keys: DashMap::new(),
            master_key,
            stats: Arc::new(DashMap::new()),
            storage_path,
            initialized: false,
        };
        
        Ok(backend)
    }
    
    /// Get or generate master key for encrypting key material
    async fn get_or_generate_master_key(storage_path: &PathBuf) -> Result<Zeroizing<Vec<u8>>, BackendError> {
        let master_key_path = storage_path.join("master.key");
        
        if master_key_path.exists() {
            // Load existing master key
            let encrypted_master = std::fs::read(&master_key_path)
                .map_err(|e| BackendError::StorageError(e.to_string()))?;
            
            // For now, use a simple approach - in production, this should use
            // hardware-backed key derivation or secure enclave
            let salt = b"polymera-keystore-master";
            let derived_key = utils::derive_key_argon2(salt, salt, 100000, 32)?;
            
            // Decrypt master key (simplified - in production use proper key derivation)
            Ok(Zeroizing::new(derived_key))
        } else {
            // Generate new master key
            let mut master_key = vec![0u8; 32];
            rand::thread_rng().fill(&mut master_key);
            
            // Encrypt and store master key
            let salt = b"polymera-keystore-master";
            let derived_key = utils::derive_key_argon2(salt, salt, 100000, 32)?;
            
            let encrypted_master = encrypt_key_material(&master_key, &derived_key)?;
            std::fs::write(&master_key_path, encrypted_master.0)
                .map_err(|e| BackendError::StorageError(e.to_string()))?;
            
            Ok(Zeroizing::new(master_key))
        }
    }
    
    /// Generate cryptographic key material based on type
    fn generate_key_material(
        key_type: KeyType,
    ) -> Result<Vec<u8>, BackendError> {
        match key_type {
            KeyType::Ed25519 => {
                use ed25519_dalek::SigningKey;
                let mut rng = rand::thread_rng();
                let signing_key = SigningKey::generate(&mut rng);
                Ok(signing_key.to_bytes().to_vec())
            }
            
            KeyType::Dilithium3 => {
                use dilithium::Dilithium3;
                let (public_key, private_key) = Dilithium3::generate_keypair();
                let mut combined = Vec::new();
                combined.extend_from_slice(&public_key.to_bytes());
                combined.extend_from_slice(&private_key.to_bytes());
                Ok(combined)
            }
            
            KeyType::Dilithium5 => {
                use dilithium::Dilithium5;
                let (public_key, private_key) = Dilithium5::generate_keypair();
                let mut combined = Vec::new();
                combined.extend_from_slice(&public_key.to_bytes());
                combined.extend_from_slice(&private_key.to_bytes());
                Ok(combined)
            }
            
            KeyType::Kyber512 => {
                use kyber::Kyber512;
                let (public_key, private_key) = Kyber512::generate_keypair();
                let mut combined = Vec::new();
                combined.extend_from_slice(&public_key.to_bytes());
                combined.extend_from_slice(&private_key.to_bytes());
                Ok(combined)
            }
            
            KeyType::Kyber768 => {
                use kyber::Kyber768;
                let (public_key, private_key) = Kyber768::generate_keypair();
                let mut combined = Vec::new();
                combined.extend_from_slice(&public_key.to_bytes());
                combined.extend_from_slice(&private_key.to_bytes());
                Ok(combined)
            }
            
            KeyType::Kyber1024 => {
                use kyber::Kyber1024;
                let (public_key, private_key) = Kyber1024::generate_keypair();
                let mut combined = Vec::new();
                combined.extend_from_slice(&public_key.to_bytes());
                combined.extend_from_slice(&private_key.to_bytes());
                Ok(combined)
            }
            
            KeyType::Ed25519Dilithium3 => {
                // Generate hybrid key
                let ed25519_key = Self::generate_key_material(KeyType::Ed25519)?;
                let dilithium_key = Self::generate_key_material(KeyType::Dilithium3)?;
                let mut combined = Vec::new();
                combined.extend_from_slice(&ed25519_key);
                combined.extend_from_slice(&dilithium_key);
                Ok(combined)
            }
            
            KeyType::Ed25519Kyber512 => {
                // Generate hybrid key
                let ed25519_key = Self::generate_key_material(KeyType::Ed25519)?;
                let kyber_key = Self::generate_key_material(KeyType::Kyber512)?;
                let mut combined = Vec::new();
                combined.extend_from_slice(&ed25519_key);
                combined.extend_from_slice(&kyber_key);
                Ok(combined)
            }
            
            KeyType::AES256 => {
                let mut key = vec![0u8; 32];
                rand::thread_rng().fill(&mut key);
                Ok(key)
            }
            
            KeyType::ChaCha20Poly1305 => {
                let mut key = vec![0u8; 32];
                rand::thread_rng().fill(&mut key);
                Ok(key)
            }
        }
    }
    
    /// Get key material (decrypted)
    fn get_key_material(&self, key_id: &str) -> Result<Vec<u8>, BackendError> {
        let key = self.keys.get(key_id)
            .ok_or_else(|| BackendError::KeyNotFound(key_id.to_string()))?;
        
        if !key.is_usable() {
            return Err(BackendError::KeyExpired(key_id.to_string()));
        }
        
        let decrypted = decrypt_key_material(
            key.encrypted_key_material.expose_secret(),
            &self.master_key,
            &key.encryption_nonce,
        )?;
        
        Ok(decrypted)
    }
    
    /// Update operation statistics
    fn update_stats(&self, operation: &str) {
        let count = self.stats.entry(operation.to_string()).or_insert(0);
        *count += 1;
    }
    
    /// Perform secure erasure of key material
    fn secure_erase_key(&self, key_id: &str) -> Result<(), BackendError> {
        if let Some(mut key) = self.keys.get_mut(key_id) {
            // Zeroize encrypted key material
            key.encrypted_key_material = Zeroizing::new(vec![0u8; key.encrypted_key_material.len()]);
            key.encryption_nonce = vec![0u8; key.encryption_nonce.len()];
            
            // Mark as revoked
            key.revoke();
            
            info!("Securely erased key: {}", key_id);
        }
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl KeyBackend for SoftwareBackend {
    fn backend_id(&self) -> &str {
        &self.config.backend_id
    }
    
    async fn is_available(&self) -> Result<bool, BackendError> {
        Ok(self.initialized)
    }
    
    async fn initialize(&mut self) -> Result<(), BackendError> {
        if self.initialized {
            return Ok(());
        }
        
        // Load existing keys from storage
        let keys_path = self.storage_path.join("keys");
        if keys_path.exists() {
            // In a real implementation, load keys from persistent storage
            debug!("Loading existing keys from storage");
        }
        
        self.initialized = true;
        info!("Software backend initialized");
        
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), BackendError> {
        if !self.initialized {
            return Ok(());
        }
        
        // Perform secure cleanup
        self.cleanup().await?;
        
        self.initialized = false;
        info!("Software backend shutdown");
        
        Ok(())
    }
    
    async fn generate_key(
        &mut self,
        key_type: KeyType,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, BackendError> {
        // Validate key type and purpose compatibility
        utils::validate_key_type_purpose(key_type, &purposes)?;
        
        // Generate key material
        let key_material = Self::generate_key_material(key_type)?;
        
        // Encrypt key material with master key
        let (encrypted_material, nonce) = encrypt_key_material(
            &key_material,
            &self.master_key,
        )?;
        
        // Create internal key
        let key_id = generate_key_id();
        let internal_key = InternalKey::new(
            key_id.clone(),
            key_type,
            purposes,
            name,
            expires,
            tags,
            self.backend_id().to_string(),
            Zeroizing::new(encrypted_material),
            nonce,
            self.config.encryption_algorithm.clone(),
        );
        
        // Store key
        self.keys.insert(key_id.clone(), internal_key.clone());
        
        // Update statistics
        self.update_stats("generate_key");
        
        info!("Generated key: {} (type: {:?})", key_id, key_type);
        
        Ok(internal_key.to_metadata())
    }
    
    async fn import_key(
        &mut self,
        key_type: KeyType,
        key_data: Secret<Vec<u8>>,
        purposes: Vec<KeyPurpose>,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, BackendError> {
        // Validate key type and purpose compatibility
        utils::validate_key_type_purpose(key_type, &purposes)?;
        
        // Encrypt imported key material
        let (encrypted_material, nonce) = encrypt_key_material(
            key_data.expose_secret(),
            &self.master_key,
        )?;
        
        // Create internal key
        let key_id = generate_key_id();
        let internal_key = InternalKey::new(
            key_id.clone(),
            key_type,
            purposes,
            name,
            expires,
            tags,
            self.backend_id().to_string(),
            Zeroizing::new(encrypted_material),
            nonce,
            self.config.encryption_algorithm.clone(),
        );
        
        // Store key
        self.keys.insert(key_id.clone(), internal_key.clone());
        
        // Update statistics
        self.update_stats("import_key");
        
        info!("Imported key: {} (type: {:?})", key_id, key_type);
        
        Ok(internal_key.to_metadata())
    }
    
    async fn get_key_metadata(&self, key_id: &str) -> Result<KeyMetadata, BackendError> {
        let key = self.keys.get(key_id)
            .ok_or_else(|| BackendError::KeyNotFound(key_id.to_string()))?;
        
        Ok(key.to_metadata())
    }
    
    async fn list_keys(&self) -> Result<Vec<KeyMetadata>, BackendError> {
        let keys: Vec<KeyMetadata> = self.keys.iter()
            .map(|entry| entry.to_metadata())
            .collect();
        
        Ok(keys)
    }
    
    async fn delete_key(&mut self, key_id: &str) -> Result<(), BackendError> {
        // Perform secure erasure
        self.secure_erase_key(key_id)?;
        
        // Remove from storage
        self.keys.remove(key_id);
        
        // Update statistics
        self.update_stats("delete_key");
        
        info!("Deleted key: {}", key_id);
        
        Ok(())
    }
    
    async fn sign(
        &mut self,
        key_id: &str,
        data: &[u8],
        algorithm: Option<String>,
    ) -> Result<Signature, BackendError> {
        let key_material = self.get_key_material(key_id)?;
        let key = self.keys.get(key_id).unwrap();
        
        // Update last used timestamp
        if let Some(mut key) = self.keys.get_mut(key_id) {
            key.update_last_used();
        }
        
        let signature = match key.key_type {
            KeyType::Ed25519 => {
                use ed25519_dalek::{SigningKey, Signer};
                let signing_key = SigningKey::from_bytes(&key_material)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                signing_key.sign(data).to_bytes().to_vec()
            }
            
            KeyType::Dilithium3 => {
                use dilithium::Dilithium3;
                let private_key = Dilithium3::PrivateKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                private_key.sign(data).to_bytes().to_vec()
            }
            
            KeyType::Dilithium5 => {
                use dilithium::Dilithium5;
                let private_key = Dilithium5::PrivateKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                private_key.sign(data).to_bytes().to_vec()
            }
            
            _ => return Err(BackendError::InvalidOperation(
                format!("Key type {:?} does not support signing", key.key_type)
            )),
        };
        
        // Update statistics
        self.update_stats("sign");
        
        Ok(Signature {
            key_id: key_id.to_string(),
            signature,
            algorithm: algorithm.unwrap_or_else(|| format!("{:?}", key.key_type)),
            created: Utc::now(),
        })
    }
    
    async fn verify(
        &self,
        key_id: &str,
        data: &[u8],
        signature: &[u8],
        algorithm: Option<String>,
    ) -> Result<bool, BackendError> {
        let key_material = self.get_key_material(key_id)?;
        let key = self.keys.get(key_id).unwrap();
        
        let is_valid = match key.key_type {
            KeyType::Ed25519 => {
                use ed25519_dalek::{VerifyingKey, Verifier};
                let verifying_key = VerifyingKey::from_bytes(&key_material)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let sig = ed25519_dalek::Signature::from_bytes(signature)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                verifying_key.verify(data, &sig).is_ok()
            }
            
            KeyType::Dilithium3 => {
                use dilithium::Dilithium3;
                let public_key = Dilithium3::PublicKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let sig = Dilithium3::Signature::from_bytes(signature)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                public_key.verify(data, &sig)
            }
            
            KeyType::Dilithium5 => {
                use dilithium::Dilithium5;
                let public_key = Dilithium5::PublicKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let sig = Dilithium5::Signature::from_bytes(signature)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                public_key.verify(data, &sig)
            }
            
            _ => return Err(BackendError::InvalidOperation(
                format!("Key type {:?} does not support verification", key.key_type)
            )),
        };
        
        // Update statistics
        self.update_stats("verify");
        
        Ok(is_valid)
    }
    
    async fn encrypt(
        &mut self,
        key_id: &str,
        data: &[u8],
        algorithm: Option<String>,
    ) -> Result<EncryptedData, BackendError> {
        let key_material = self.get_key_material(key_id)?;
        let key = self.keys.get(key_id).unwrap();
        
        // Update last used timestamp
        if let Some(mut key) = self.keys.get_mut(key_id) {
            key.update_last_used();
        }
        
        let (encrypted_data, nonce) = match key.key_type {
            KeyType::AES256 => {
                use aes_gcm::{Aes256Gcm, Key, Nonce};
                use aes_gcm::aead::{Aead, NewAead};
                
                let cipher = Aes256Gcm::new(Key::from_slice(&key_material));
                let nonce_bytes = Nonce::from_slice(&generate_nonce());
                let encrypted = cipher.encrypt(nonce_bytes, data)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                
                (encrypted, nonce_bytes.to_vec())
            }
            
            KeyType::ChaCha20Poly1305 => {
                use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
                use chacha20poly1305::aead::{Aead, NewAead};
                
                let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_material));
                let nonce_bytes = Nonce::from_slice(&generate_nonce());
                let encrypted = cipher.encrypt(nonce_bytes, data)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                
                (encrypted, nonce_bytes.to_vec())
            }
            
            _ => return Err(BackendError::InvalidOperation(
                format!("Key type {:?} does not support encryption", key.key_type)
            )),
        };
        
        // Update statistics
        self.update_stats("encrypt");
        
        Ok(EncryptedData {
            key_id: key_id.to_string(),
            encrypted_data,
            nonce,
            algorithm: algorithm.unwrap_or_else(|| format!("{:?}", key.key_type)),
            created: Utc::now(),
        })
    }
    
    async fn decrypt(
        &mut self,
        key_id: &str,
        encrypted_data: &EncryptedData,
    ) -> Result<Vec<u8>, BackendError> {
        let key_material = self.get_key_material(key_id)?;
        let key = self.keys.get(key_id).unwrap();
        
        // Update last used timestamp
        if let Some(mut key) = self.keys.get_mut(key_id) {
            key.update_last_used();
        }
        
        let decrypted = match key.key_type {
            KeyType::AES256 => {
                use aes_gcm::{Aes256Gcm, Key, Nonce};
                use aes_gcm::aead::{Aead, NewAead};
                
                let cipher = Aes256Gcm::new(Key::from_slice(&key_material));
                let nonce_bytes = Nonce::from_slice(&encrypted_data.nonce);
                cipher.decrypt(nonce_bytes, &encrypted_data.encrypted_data)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?
            }
            
            KeyType::ChaCha20Poly1305 => {
                use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
                use chacha20poly1305::aead::{Aead, NewAead};
                
                let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_material));
                let nonce_bytes = Nonce::from_slice(&encrypted_data.nonce);
                cipher.decrypt(nonce_bytes, &encrypted_data.encrypted_data)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?
            }
            
            _ => return Err(BackendError::InvalidOperation(
                format!("Key type {:?} does not support decryption", key.key_type)
            )),
        };
        
        // Update statistics
        self.update_stats("decrypt");
        
        Ok(decrypted)
    }
    
    async fn encapsulate_key(
        &mut self,
        key_id: &str,
        algorithm: Option<String>,
    ) -> Result<KeyEncapsulationResult, BackendError> {
        let key_material = self.get_key_material(key_id)?;
        let key = self.keys.get(key_id).unwrap();
        
        // Update last used timestamp
        if let Some(mut key) = self.keys.get_mut(key_id) {
            key.update_last_used();
        }
        
        let (encapsulated_key, shared_secret) = match key.key_type {
            KeyType::Kyber512 => {
                use kyber::Kyber512;
                let public_key = Kyber512::PublicKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let (encapsulated, shared) = Kyber512::encapsulate(&public_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                (encapsulated.to_bytes().to_vec(), shared.to_bytes().to_vec())
            }
            
            KeyType::Kyber768 => {
                use kyber::Kyber768;
                let public_key = Kyber768::PublicKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let (encapsulated, shared) = Kyber768::encapsulate(&public_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                (encapsulated.to_bytes().to_vec(), shared.to_bytes().to_vec())
            }
            
            KeyType::Kyber1024 => {
                use kyber::Kyber1024;
                let public_key = Kyber1024::PublicKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let (encapsulated, shared) = Kyber1024::encapsulate(&public_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                (encapsulated.to_bytes().to_vec(), shared.to_bytes().to_vec())
            }
            
            _ => return Err(BackendError::InvalidOperation(
                format!("Key type {:?} does not support key encapsulation", key.key_type)
            )),
        };
        
        // Update statistics
        self.update_stats("encapsulate_key");
        
        Ok(KeyEncapsulationResult {
            key_id: key_id.to_string(),
            encapsulated_key,
            shared_secret,
            algorithm: algorithm.unwrap_or_else(|| format!("{:?}", key.key_type)),
            created: Utc::now(),
        })
    }
    
    async fn decapsulate_key(
        &mut self,
        key_id: &str,
        encapsulated_key: &[u8],
        algorithm: Option<String>,
    ) -> Result<Vec<u8>, BackendError> {
        let key_material = self.get_key_material(key_id)?;
        let key = self.keys.get(key_id).unwrap();
        
        // Update last used timestamp
        if let Some(mut key) = self.keys.get_mut(key_id) {
            key.update_last_used();
        }
        
        let shared_secret = match key.key_type {
            KeyType::Kyber512 => {
                use kyber::Kyber512;
                let private_key = Kyber512::PrivateKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let encapsulated = Kyber512::Ciphertext::from_bytes(encapsulated_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                Kyber512::decapsulate(&encapsulated, &private_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?
                    .to_bytes()
                    .to_vec()
            }
            
            KeyType::Kyber768 => {
                use kyber::Kyber768;
                let private_key = Kyber768::PrivateKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let encapsulated = Kyber768::Ciphertext::from_bytes(encapsulated_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                Kyber768::decapsulate(&encapsulated, &private_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?
                    .to_bytes()
                    .to_vec()
            }
            
            KeyType::Kyber1024 => {
                use kyber::Kyber1024;
                let private_key = Kyber1024::PrivateKey::from_bytes(&key_material[..])
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                let encapsulated = Kyber1024::Ciphertext::from_bytes(encapsulated_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?;
                Kyber1024::decapsulate(&encapsulated, &private_key)
                    .map_err(|e| BackendError::CryptographicError(e.to_string()))?
                    .to_bytes()
                    .to_vec()
            }
            
            _ => return Err(BackendError::InvalidOperation(
                format!("Key type {:?} does not support key decapsulation", key.key_type)
            )),
        };
        
        // Update statistics
        self.update_stats("decapsulate_key");
        
        Ok(shared_secret)
    }
    
    async fn derive_key(
        &mut self,
        key_id: &str,
        params: &KeyDerivationParams,
    ) -> Result<Vec<u8>, BackendError> {
        let key_material = self.get_key_material(key_id)?;
        let key = self.keys.get(key_id).unwrap();
        
        // Update last used timestamp
        if let Some(mut key) = self.keys.get_mut(key_id) {
            key.update_last_used();
        }
        
        // Use Argon2 for key derivation
        let derived_key = utils::derive_key_argon2(
            &key_material,
            &params.salt,
            params.iterations,
            params.key_length,
        )?;
        
        // Update statistics
        self.update_stats("derive_key");
        
        Ok(derived_key)
    }
    
    async fn update_key_metadata(
        &mut self,
        key_id: &str,
        name: Option<String>,
        expires: Option<DateTime<Utc>>,
        tags: Option<HashMap<String, String>>,
    ) -> Result<KeyMetadata, BackendError> {
        let mut key = self.keys.get_mut(key_id)
            .ok_or_else(|| BackendError::KeyNotFound(key_id.to_string()))?;
        
        if let Some(name) = name {
            key.name = Some(name);
        }
        
        if let Some(expires) = expires {
            key.expires = Some(expires);
        }
        
        if let Some(tags) = tags {
            key.tags = tags;
        }
        
        // Update statistics
        self.update_stats("update_key_metadata");
        
        Ok(key.to_metadata())
    }
    
    async fn revoke_key(&mut self, key_id: &str, reason: Option<String>) -> Result<(), BackendError> {
        if let Some(mut key) = self.keys.get_mut(key_id) {
            key.revoke();
            info!("Revoked key: {} (reason: {:?})", key_id, reason);
        }
        
        // Update statistics
        self.update_stats("revoke_key");
        
        Ok(())
    }
    
    async fn rotate_key(
        &mut self,
        key_id: &str,
        new_key_type: Option<KeyType>,
        new_purposes: Option<Vec<KeyPurpose>>,
    ) -> Result<KeyMetadata, BackendError> {
        let old_key = self.keys.get(key_id)
            .ok_or_else(|| BackendError::KeyNotFound(key_id.to_string()))?;
        
        // Generate new key with same or new type
        let key_type = new_key_type.unwrap_or(old_key.key_type);
        let purposes = new_purposes.unwrap_or(old_key.purposes.clone());
        
        let new_key_metadata = self.generate_key(
            key_type,
            purposes,
            old_key.name.clone(),
            old_key.expires,
            Some(old_key.tags.clone()),
        ).await?;
        
        // Revoke old key
        self.revoke_key(key_id, Some("Key rotation".to_string())).await?;
        
        // Update statistics
        self.update_stats("rotate_key");
        
        info!("Rotated key: {} -> {}", key_id, new_key_metadata.id);
        
        Ok(new_key_metadata)
    }
    
    async fn get_stats(&self) -> Result<KeystoreStats, BackendError> {
        let total_keys = self.keys.len();
        let active_keys = self.keys.iter()
            .filter(|entry| entry.is_usable())
            .count();
        let expired_keys = self.keys.iter()
            .filter(|entry| entry.expires.is_some() && entry.expires.unwrap() < Utc::now())
            .count();
        let revoked_keys = self.keys.iter()
            .filter(|entry| entry.status == KeyStatus::Revoked)
            .count();
        
        let total_operations: u64 = self.stats.values().sum();
        let last_operation = self.stats.iter()
            .max_by_key(|entry| *entry.value())
            .map(|_| Utc::now());
        
        Ok(KeystoreStats {
            total_keys,
            active_keys,
            expired_keys,
            revoked_keys,
            total_operations,
            last_operation,
            backend_id: self.backend_id().to_string(),
        })
    }
    
    async fn cleanup(&mut self) -> Result<(), BackendError> {
        // Remove expired keys
        let expired_keys: Vec<String> = self.keys.iter()
            .filter(|entry| entry.expires.is_some() && entry.expires.unwrap() < Utc::now())
            .map(|entry| entry.id.clone())
            .collect();
        
        for key_id in expired_keys {
            self.delete_key(&key_id).await?;
        }
        
        // Update statistics
        self.update_stats("cleanup");
        
        info!("Cleanup completed");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    async fn create_test_backend() -> SoftwareBackend {
        let config = BackendConfig {
            backend_id: "test-software".to_string(),
            storage_path: Some("test_data/keystore".to_string()),
            ..Default::default()
        };
        
        SoftwareBackend::new(config).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_backend_creation() {
        let backend = create_test_backend().await;
        assert_eq!(backend.backend_id(), "test-software");
    }
    
    #[tokio::test]
    async fn test_key_generation() {
        let mut backend = create_test_backend().await;
        backend.initialize().await.unwrap();
        
        let metadata = backend.generate_key(
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
        let mut backend = create_test_backend().await;
        backend.initialize().await.unwrap();
        
        // Generate signing key
        let metadata = backend.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            None,
            None,
            None,
        ).await.unwrap();
        
        let data = b"Hello, World!";
        
        // Sign data
        let signature = backend.sign(&metadata.id, data, None).await.unwrap();
        assert_eq!(signature.key_id, metadata.id);
        
        // Verify signature
        let is_valid = backend.verify(&metadata.id, data, &signature.signature, None).await.unwrap();
        assert!(is_valid);
    }
    
    #[tokio::test]
    async fn test_encrypt_and_decrypt() {
        let mut backend = create_test_backend().await;
        backend.initialize().await.unwrap();
        
        // Generate encryption key
        let metadata = backend.generate_key(
            KeyType::AES256,
            vec![KeyPurpose::Encrypt, KeyPurpose::Decrypt],
            None,
            None,
            None,
        ).await.unwrap();
        
        let data = b"Secret message";
        
        // Encrypt data
        let encrypted = backend.encrypt(&metadata.id, data, None).await.unwrap();
        assert_eq!(encrypted.key_id, metadata.id);
        
        // Decrypt data
        let decrypted = backend.decrypt(&metadata.id, &encrypted).await.unwrap();
        assert_eq!(decrypted, data);
    }
    
    #[tokio::test]
    async fn test_key_encapsulation() {
        let mut backend = create_test_backend().await;
        backend.initialize().await.unwrap();
        
        // Generate KEM key
        let metadata = backend.generate_key(
            KeyType::Kyber512,
            vec![KeyPurpose::KeyEncapsulation, KeyPurpose::KeyDecapsulation],
            None,
            None,
            None,
        ).await.unwrap();
        
        // Encapsulate key
        let result = backend.encapsulate_key(&metadata.id, None).await.unwrap();
        assert_eq!(result.key_id, metadata.id);
        
        // Decapsulate key
        let shared_secret = backend.decapsulate_key(&metadata.id, &result.encapsulated_key, None).await.unwrap();
        assert_eq!(shared_secret, result.shared_secret);
    }
    
    #[tokio::test]
    async fn test_secure_erasure() {
        let mut backend = create_test_backend().await;
        backend.initialize().await.unwrap();
        
        // Generate key
        let metadata = backend.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign],
            None,
            None,
            None,
        ).await.unwrap();
        
        // Delete key (should perform secure erasure)
        backend.delete_key(&metadata.id).await.unwrap();
        
        // Verify key is gone
        let result = backend.get_key_metadata(&metadata.id).await;
        assert!(result.is_err());
    }
}
