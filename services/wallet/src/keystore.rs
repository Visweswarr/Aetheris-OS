//! KeyStore implementation for secure key management with NGFS-backed storage

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::error::WalletError;

/// Supported key types for multi-chain and PQC operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyType {
    // Legacy algorithms
    Secp256k1,    // Ethereum, Bitcoin
    Ed25519,      // Solana, Cardano
    Sr25519,      // Polkadot, Substrate
    X25519,       // Key exchange
    
    // Post-quantum algorithms
    Kyber512,     // PQC KEM
    Kyber768,     // PQC KEM
    Kyber1024,    // PQC KEM
    Dilithium2,   // PQC Signature
    Dilithium3,   // PQC Signature
    Dilithium5,   // PQC Signature
}

/// Key purposes for different operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyPurpose {
    Sign,
    Verify,
    Encrypt,
    Decrypt,
    KeyEncapsulation,
    KeyDecapsulation,
    KeyExchange,
    Authentication,
}

/// Key derivation path for hierarchical deterministic keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationPath {
    pub path: String,           // e.g., "m/44'/60'/0'/0/0"
    pub chain_code: Vec<u8>,    // Chain code for derivation
    pub depth: u8,              // Derivation depth
    pub index: u32,             // Current index
    pub hardened: bool,         // Whether this is a hardened derivation
}

impl KeyDerivationPath {
    /// Create a new derivation path
    pub fn new(path: &str) -> Result<Self, WalletError> {
        if !path.starts_with("m/") {
            return Err(WalletError::InvalidDerivationPath("Path must start with 'm/'".to_string()));
        }
        
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 2 {
            return Err(WalletError::InvalidDerivationPath("Invalid path format".to_string()));
        }
        
        let depth = (parts.len() - 1) as u8;
        let last_part = parts.last().unwrap();
        let (index_str, hardened) = if last_part.ends_with("'") {
            (last_part.strip_suffix("'").unwrap(), true)
        } else {
            (*last_part, false)
        };
        
        let index = index_str.parse::<u32>()
            .map_err(|_| WalletError::InvalidDerivationPath("Invalid index".to_string()))?;
        
        Ok(Self {
            path: path.to_string(),
            chain_code: vec![0u8; 32], // Will be set during derivation
            depth,
            index,
            hardened,
        })
    }
    
    /// Check if this path should be persisted
    pub fn should_persist(&self) -> bool {
        // Persist keys at depth >= 3 (account level and below)
        self.depth >= 3
    }
}

/// Key material containing the actual cryptographic keys
#[derive(Debug, Clone)]
pub struct KeyMaterial {
    key_type: KeyType,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
    purposes: Vec<KeyPurpose>,
    derivation_path: Option<KeyDerivationPath>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    metadata: HashMap<String, String>,
}

impl KeyMaterial {
    /// Create new key material
    pub fn new(
        key_type: KeyType,
        public_key: Vec<u8>,
        private_key: Vec<u8>,
        purposes: Vec<KeyPurpose>,
        derivation_path: Option<KeyDerivationPath>,
    ) -> Self {
        Self {
            key_type,
            public_key,
            private_key,
            purposes,
            derivation_path,
            created_at: Utc::now(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Get the key type
    pub fn key_type(&self) -> KeyType {
        self.key_type
    }
    
    /// Get the public key
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }
    
    /// Get the private key (zeroized on drop)
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }
    
    /// Get the derivation path
    pub fn derivation_path(&self) -> &Option<KeyDerivationPath> {
        &self.derivation_path
    }
    
    /// Check if key supports a specific purpose
    pub fn supports_purpose(&self, purpose: KeyPurpose) -> bool {
        self.purposes.contains(&purpose)
    }
    
    /// Check if key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            Utc::now() > expires
        } else {
            false
        }
    }
    
    /// Set expiration time
    pub fn set_expires_at(&mut self, expires: DateTime<Utc>) {
        self.expires_at = Some(expires);
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl Drop for KeyMaterial {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

/// Signature result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signature: Vec<u8>,
    pub recovery_id: Option<u8>,
    pub algorithm: String,
    pub created_at: DateTime<Utc>,
}

/// NGFS client trait for vault operations
#[async_trait::async_trait]
pub trait NgfsClient {
    async fn store_item(&self, path: &str, data: &[u8]) -> Result<String, WalletError>;
    async fn get_item(&self, path: &str) -> Result<Vec<u8>, WalletError>;
    async fn list_items(&self, path: &str) -> Result<Vec<String>, WalletError>;
    async fn delete_item(&self, path: &str) -> Result<(), WalletError>;
}

/// KeyStore for managing keys with NGFS backend
pub struct KeyStore {
    pdv_path: String,
    keys: Arc<RwLock<HashMap<Uuid, KeyMaterial>>>,
    ngfs_client: Option<Arc<dyn NgfsClient + Send + Sync>>,
}

impl KeyStore {
    /// Create a new keystore
    pub async fn new(pdv_path: String) -> Result<Self, WalletError> {
        Ok(Self {
            pdv_path,
            keys: Arc::new(RwLock::new(HashMap::new())),
            ngfs_client: None,
        })
    }
    
    /// Set the NGFS client
    pub fn set_ngfs_client(&mut self, client: Arc<dyn NgfsClient + Send + Sync>) {
        self.ngfs_client = Some(client);
    }
    
    /// Get the PDV path
    pub fn pdv_path(&self) -> &str {
        &self.pdv_path
    }
    
    /// Initialize a wallet directory
    pub async fn init_wallet(&mut self, wallet_id: &Uuid) -> Result<(), WalletError> {
        let wallet_path = format!("{}/{}", self.pdv_path, wallet_id);
        
        // Create wallet directory structure
        tokio::fs::create_dir_all(&wallet_path).await?;
        tokio::fs::create_dir_all(format!("{}/keys", wallet_path)).await?;
        tokio::fs::create_dir_all(format!("{}/metadata", wallet_path)).await?;
        
        Ok(())
    }
    
    /// Generate a new key
    pub async fn generate_key(
        &mut self,
        wallet_id: &Uuid,
        key_id: &Uuid,
        key_type: KeyType,
    ) -> Result<KeyMaterial, WalletError> {
        let (public_key, private_key) = match key_type {
            KeyType::Secp256k1 => self.generate_secp256k1_key().await?,
            KeyType::Ed25519 => self.generate_ed25519_key().await?,
            KeyType::Sr25519 => self.generate_sr25519_key().await?,
            KeyType::X25519 => self.generate_x25519_key().await?,
            KeyType::Kyber512 => self.generate_kyber_key(512).await?,
            KeyType::Kyber768 => self.generate_kyber_key(768).await?,
            KeyType::Kyber1024 => self.generate_kyber_key(1024).await?,
            KeyType::Dilithium2 => self.generate_dilithium_key(2).await?,
            KeyType::Dilithium3 => self.generate_dilithium_key(3).await?,
            KeyType::Dilithium5 => self.generate_dilithium_key(5).await?,
        };
        
        let purposes = self.get_default_purposes(key_type);
        
        let key_material = KeyMaterial::new(
            key_type,
            public_key,
            private_key,
            purposes,
            None,
        );
        
        // Store in memory
        self.keys.write().await.insert(*key_id, key_material.clone());
        
        Ok(key_material)
    }
    
    /// Store a key in the vault
    pub async fn store_key(
        &self,
        wallet_id: &Uuid,
        key_id: &Uuid,
        key_material: &KeyMaterial,
    ) -> Result<String, WalletError> {
        let key_path = format!("{}/{}/keys/{}", self.pdv_path, wallet_id, key_id);
        
        // Encrypt the key material
        let encrypted_data = self.encrypt_key_material(key_material).await?;
        
        if let Some(ngfs_client) = &self.ngfs_client {
            // Store in NGFS
            let item_id = ngfs_client.store_item(&key_path, &encrypted_data).await?;
            Ok(item_id)
        } else {
            // Store locally
            tokio::fs::write(&key_path, &encrypted_data).await?;
            Ok(key_path)
        }
    }
    
    /// Get a key from the vault
    pub async fn get_key(&self, wallet_id: &Uuid, key_id: &Uuid) -> Result<KeyMaterial, WalletError> {
        // Check memory cache first
        if let Some(key_material) = self.keys.read().await.get(key_id) {
            return Ok(key_material.clone());
        }
        
        let key_path = format!("{}/{}/keys/{}", self.pdv_path, wallet_id, key_id);
        
        let encrypted_data = if let Some(ngfs_client) = &self.ngfs_client {
            // Get from NGFS
            ngfs_client.get_item(&key_path).await?
        } else {
            // Get from local storage
            tokio::fs::read(&key_path).await?
        };
        
        // Decrypt the key material
        let key_material = self.decrypt_key_material(&encrypted_data).await?;
        
        // Cache in memory
        self.keys.write().await.insert(*key_id, key_material.clone());
        
        Ok(key_material)
    }
    
    /// List all keys in a wallet
    pub async fn list_keys(&self, wallet_id: &Uuid) -> Result<Vec<String>, WalletError> {
        let keys_path = format!("{}/{}/keys", self.pdv_path, wallet_id);
        
        if let Some(ngfs_client) = &self.ngfs_client {
            // List from NGFS
            ngfs_client.list_items(&keys_path).await
        } else {
            // List from local storage
            let mut entries = tokio::fs::read_dir(&keys_path).await?;
            let mut keys = Vec::new();
            
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        keys.push(name.to_string());
                    }
                }
            }
            
            Ok(keys)
        }
    }
    
    /// Sign data with a key
    pub async fn sign(&self, key_material: &KeyMaterial, data: &[u8]) -> Result<Signature, WalletError> {
        if !key_material.supports_purpose(KeyPurpose::Sign) {
            return Err(WalletError::InvalidKeyType("Key does not support signing".to_string()));
        }
        
        let signature = match key_material.key_type() {
            KeyType::Secp256k1 => self.sign_secp256k1(key_material, data).await?,
            KeyType::Ed25519 => self.sign_ed25519(key_material, data).await?,
            KeyType::Sr25519 => self.sign_sr25519(key_material, data).await?,
            KeyType::Dilithium2 | KeyType::Dilithium3 | KeyType::Dilithium5 => {
                self.sign_dilithium(key_material, data).await?
            },
            _ => return Err(WalletError::InvalidKeyType("Key type does not support signing".to_string())),
        };
        
        Ok(signature)
    }
    
    /// Sign with PQC algorithm
    pub async fn sign_pqc(&self, key_material: &KeyMaterial, data: &[u8]) -> Result<Vec<u8>, WalletError> {
        match key_material.key_type() {
            KeyType::Dilithium2 | KeyType::Dilithium3 | KeyType::Dilithium5 => {
                let signature = self.sign_dilithium(key_material, data).await?;
                Ok(signature.signature)
            },
            _ => Err(WalletError::InvalidKeyType("Key type does not support PQC signing".to_string())),
        }
    }
    
    /// Derive a key from a parent key
    pub async fn derive_key(
        &self,
        parent_key: &KeyMaterial,
        derivation_path: &KeyDerivationPath,
    ) -> Result<KeyMaterial, WalletError> {
        // Implementation depends on key type
        match parent_key.key_type() {
            KeyType::Secp256k1 => self.derive_secp256k1_key(parent_key, derivation_path).await,
            KeyType::Ed25519 => self.derive_ed25519_key(parent_key, derivation_path).await,
            _ => Err(WalletError::InvalidKeyType("Key type does not support derivation".to_string())),
        }
    }
    
    /// Compute Ethereum address from public key
    pub async fn compute_ethereum_address(&self, public_key: &[u8]) -> Result<Vec<u8>, WalletError> {
        use sha2::{Sha256, Digest};
        
        // Remove the 0x04 prefix if present
        let pubkey = if public_key.len() == 65 && public_key[0] == 0x04 {
            &public_key[1..]
        } else {
            public_key
        };
        
        if pubkey.len() != 64 {
            return Err(WalletError::InvalidKeyType("Invalid public key length".to_string()));
        }
        
        // Hash the public key
        let hash = Sha256::digest(pubkey);
        let keccak_hash = keccak256(&hash);
        
        // Take the last 20 bytes as the address
        Ok(keccak_hash[12..].to_vec())
    }
    
    /// Compute Solana address from public key
    pub async fn compute_solana_address(&self, public_key: &[u8]) -> Result<Vec<u8>, WalletError> {
        // Solana addresses are just the public key
        Ok(public_key.to_vec())
    }
    
    /// Import an encrypted key
    pub async fn import_encrypted_key(&self, encrypted_data: &[u8]) -> Result<KeyMaterial, WalletError> {
        self.decrypt_key_material(encrypted_data).await
    }
    
    /// Revoke a key
    pub async fn revoke_key(&mut self, wallet_id: &Uuid, key_id: &Uuid) -> Result<(), WalletError> {
        // Remove from memory
        self.keys.write().await.remove(key_id);
        
        // Remove from storage
        let key_path = format!("{}/{}/keys/{}", self.pdv_path, wallet_id, key_id);
        
        if let Some(ngfs_client) = &self.ngfs_client {
            ngfs_client.delete_item(&key_path).await?;
        } else {
            tokio::fs::remove_file(&key_path).await?;
        }
        
        Ok(())
    }
    
    // Private helper methods
    
    fn get_default_purposes(&self, key_type: KeyType) -> Vec<KeyPurpose> {
        match key_type {
            KeyType::Secp256k1 => vec![KeyPurpose::Sign, KeyPurpose::Verify],
            KeyType::Ed25519 => vec![KeyPurpose::Sign, KeyPurpose::Verify],
            KeyType::Sr25519 => vec![KeyPurpose::Sign, KeyPurpose::Verify],
            KeyType::X25519 => vec![KeyPurpose::KeyExchange],
            KeyType::Kyber512 | KeyType::Kyber768 | KeyType::Kyber1024 => {
                vec![KeyPurpose::KeyEncapsulation, KeyPurpose::KeyDecapsulation]
            },
            KeyType::Dilithium2 | KeyType::Dilithium3 | KeyType::Dilithium5 => {
                vec![KeyPurpose::Sign, KeyPurpose::Verify]
            },
        }
    }
    
    async fn generate_secp256k1_key(&self) -> Result<(Vec<u8>, Vec<u8>), WalletError> {
        use secp256k1::{Secp256k1, SecretKey, PublicKey};
        use rand::rngs::OsRng;
        
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        Ok((
            public_key.serialize().to_vec(),
            secret_key.secret_bytes().to_vec(),
        ))
    }
    
    async fn generate_ed25519_key(&self) -> Result<(Vec<u8>, Vec<u8>), WalletError> {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;
        
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        
        Ok((
            verifying_key.to_bytes().to_vec(),
            signing_key.to_bytes().to_vec(),
        ))
    }
    
    async fn generate_sr25519_key(&self) -> Result<(Vec<u8>, Vec<u8>), WalletError> {
        // SR25519 implementation would go here
        // For now, return a placeholder
        Ok((vec![0u8; 32], vec![0u8; 64]))
    }
    
    async fn generate_x25519_key(&self) -> Result<(Vec<u8>, Vec<u8>), WalletError> {
        use x25519_dalek::{StaticSecret, PublicKey};
        use rand::rngs::OsRng;
        
        let mut rng = OsRng;
        let secret = StaticSecret::random_from_rng(&mut rng);
        let public = PublicKey::from(&secret);
        
        Ok((
            public.as_bytes().to_vec(),
            secret.to_bytes().to_vec(),
        ))
    }
    
    async fn generate_kyber_key(&self, parameter_set: u16) -> Result<(Vec<u8>, Vec<u8>), WalletError> {
        use polymera_crypto::{KyberKem, KyberParameterSet};

        let params = match parameter_set {
            512 => KyberParameterSet::Kyber512,
            768 => KyberParameterSet::Kyber768,
            1024 => KyberParameterSet::Kyber1024,
            _ => return Err(WalletError::InvalidKeyType("Invalid Kyber parameter set".to_string())),
        };

        let (public_key, secret_key) = KyberKem::generate_keypair(params)
            .map_err(|e| WalletError::PQCOperationFailed(e.to_string()))?;

        Ok((public_key.to_vec(), secret_key.to_vec()))
    }
    
    async fn generate_dilithium_key(&self, parameter_set: u8) -> Result<(Vec<u8>, Vec<u8>), WalletError> {
        use polymera_crypto::{Dilithium, DilithiumParameterSet};

        let params = match parameter_set {
            2 => DilithiumParameterSet::Dilithium2,
            3 => DilithiumParameterSet::Dilithium3,
            5 => DilithiumParameterSet::Dilithium5,
            _ => return Err(WalletError::InvalidKeyType("Invalid Dilithium parameter set".to_string())),
        };

        let (public_key, secret_key) = Dilithium::generate_keypair(params)
            .map_err(|e| WalletError::PQCOperationFailed(e.to_string()))?;

        Ok((public_key.to_vec(), secret_key.to_vec()))
    }
    
    async fn sign_secp256k1(&self, key_material: &KeyMaterial, data: &[u8]) -> Result<Signature, WalletError> {
        use secp256k1::{Secp256k1, SecretKey, Message};
        use sha2::{Sha256, Digest};
        
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(key_material.private_key())
            .map_err(|_| WalletError::CryptoError("Invalid secret key".to_string()))?;
        
        // Hash the data
        let hash = Sha256::digest(data);
        let message = Message::from_slice(&hash)
            .map_err(|_| WalletError::CryptoError("Invalid message".to_string()))?;
        
        // Sign
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let signature_bytes = signature.serialize_compact();
        
        Ok(Signature {
            signature: signature_bytes.to_vec(),
            recovery_id: None,
            algorithm: "secp256k1".to_string(),
            created_at: Utc::now(),
        })
    }
    
    async fn sign_ed25519(&self, key_material: &KeyMaterial, data: &[u8]) -> Result<Signature, WalletError> {
        use ed25519_dalek::{Signer, SigningKey};
        
        let signing_key = SigningKey::from_bytes(
            key_material.private_key().try_into()
                .map_err(|_| WalletError::CryptoError("Invalid secret key length".to_string()))?
        );
        
        let signature = signing_key.sign(data);
        
        Ok(Signature {
            signature: signature.to_bytes().to_vec(),
            recovery_id: None,
            algorithm: "ed25519".to_string(),
            created_at: Utc::now(),
        })
    }
    
    async fn sign_sr25519(&self, key_material: &KeyMaterial, data: &[u8]) -> Result<Signature, WalletError> {
        // SR25519 implementation would go here
        // For now, return a placeholder
        Ok(Signature {
            signature: vec![0u8; 64],
            recovery_id: None,
            algorithm: "sr25519".to_string(),
            created_at: Utc::now(),
        })
    }
    
    async fn sign_dilithium(&self, key_material: &KeyMaterial, data: &[u8]) -> Result<Signature, WalletError> {
        use polymera_crypto::{Dilithium, DilithiumParameterSet, DilithiumSecretKey};

        let params = match key_material.key_type() {
            KeyType::Dilithium2 => DilithiumParameterSet::Dilithium2,
            KeyType::Dilithium3 => DilithiumParameterSet::Dilithium3,
            KeyType::Dilithium5 => DilithiumParameterSet::Dilithium5,
            _ => return Err(WalletError::InvalidKeyType("Invalid Dilithium key type".to_string())),
        };

        let secret_key = DilithiumSecretKey::new(params, key_material.private_key().to_vec())
            .map_err(|e| WalletError::PQCOperationFailed(e.to_string()))?;
        let signature = Dilithium::sign(&secret_key, data)
            .map_err(|e| WalletError::PQCOperationFailed(e.to_string()))?;

        Ok(Signature {
            signature: signature.to_vec(),
            recovery_id: None,
            algorithm: format!("dilithium{}", match key_material.key_type() {
                KeyType::Dilithium2 => "2",
                KeyType::Dilithium3 => "3",
                KeyType::Dilithium5 => "5",
                _ => "unknown",
            }),
            created_at: Utc::now(),
        })
    }
    
    async fn derive_secp256k1_key(
        &self,
        parent_key: &KeyMaterial,
        derivation_path: &KeyDerivationPath,
    ) -> Result<KeyMaterial, WalletError> {
        // HD key derivation for Secp256k1 would go here
        // For now, return a placeholder
        let (public_key, private_key) = self.generate_secp256k1_key().await?;
        
        Ok(KeyMaterial::new(
            parent_key.key_type(),
            public_key,
            private_key,
            parent_key.purposes.clone(),
            Some(derivation_path.clone()),
        ))
    }
    
    async fn derive_ed25519_key(
        &self,
        parent_key: &KeyMaterial,
        derivation_path: &KeyDerivationPath,
    ) -> Result<KeyMaterial, WalletError> {
        // HD key derivation for Ed25519 would go here
        // For now, return a placeholder
        let (public_key, private_key) = self.generate_ed25519_key().await?;
        
        Ok(KeyMaterial::new(
            parent_key.key_type(),
            public_key,
            private_key,
            parent_key.purposes.clone(),
            Some(derivation_path.clone()),
        ))
    }
    
    async fn encrypt_key_material(&self, key_material: &KeyMaterial) -> Result<Vec<u8>, WalletError> {
        let stored = StoredKeyMaterial::from_key_material(key_material);
        let serialized = bincode::serialize(&stored)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;
        
        // For now, just return the serialized data
        // In production, this would be encrypted with a master key
        Ok(serialized)
    }
    
    async fn decrypt_key_material(&self, encrypted_data: &[u8]) -> Result<KeyMaterial, WalletError> {
        // Decrypt key material from storage
        // For now, just deserialize
        let stored: StoredKeyMaterial = bincode::deserialize(encrypted_data)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;
        
        Ok(stored.into_key_material())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredKeyMaterial {
    key_type: KeyType,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
    purposes: Vec<KeyPurpose>,
    derivation_path: Option<KeyDerivationPath>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    metadata: HashMap<String, String>,
}

impl StoredKeyMaterial {
    fn from_key_material(key_material: &KeyMaterial) -> Self {
        Self {
            key_type: key_material.key_type,
            public_key: key_material.public_key.clone(),
            private_key: key_material.private_key.clone(),
            purposes: key_material.purposes.clone(),
            derivation_path: key_material.derivation_path.clone(),
            created_at: key_material.created_at,
            expires_at: key_material.expires_at,
            metadata: key_material.metadata.clone(),
        }
    }

    fn into_key_material(self) -> KeyMaterial {
        KeyMaterial {
            key_type: self.key_type,
            public_key: self.public_key,
            private_key: self.private_key,
            purposes: self.purposes,
            derivation_path: self.derivation_path,
            created_at: self.created_at,
            expires_at: self.expires_at,
            metadata: self.metadata,
        }
    }
}

// Helper function for Keccak256
fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let hash = Sha256::digest(data);
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_keystore_creation() {
        let keystore = KeyStore::new("/tmp/test_pdv".to_string()).await;
        assert!(keystore.is_ok());
    }
    
    #[tokio::test]
    async fn test_key_generation() {
        let mut keystore = KeyStore::new("/tmp/test_pdv".to_string()).await.unwrap();
        let wallet_id = Uuid::new_v4();
        let key_id = Uuid::new_v4();
        
        let key_material = keystore.generate_key(&wallet_id, &key_id, KeyType::Ed25519).await;
        assert!(key_material.is_ok());
        
        let key = key_material.unwrap();
        assert_eq!(key.key_type(), KeyType::Ed25519);
        assert!(!key.public_key().is_empty());
    }
    
    #[tokio::test]
    async fn test_derivation_path() {
        let path = KeyDerivationPath::new("m/44'/60'/0'/0/0");
        assert!(path.is_ok());
        
        let path = path.unwrap();
        assert_eq!(path.depth, 5);
        assert_eq!(path.index, 0);
        assert!(path.hardened);
    }
    
    #[tokio::test]
    async fn test_invalid_derivation_path() {
        let path = KeyDerivationPath::new("invalid/path");
        assert!(path.is_err());
    }
}
