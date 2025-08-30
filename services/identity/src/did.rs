use crate::dstore::DidStore;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

pub mod dstore;

#[derive(Debug, Error)]
pub enum DidError {
    #[error("Invalid DID: {0}")]
    InvalidDid(String),
    
    #[error("DID not found: {0}")]
    DidNotFound(String),
    
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),
    
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidKey {
    pub id: String,
    pub key_type: KeyType,
    pub public_key: Vec<u8>,
    pub private_key: Option<Vec<u8>>,
    pub created: DateTime<Utc>,
    pub expires: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub purpose: Vec<KeyPurpose>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    Authentication,
    Assertion,
    KeyAgreement,
    KeyEncapsulation,
    CapabilityInvocation,
    CapabilityDelegation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    pub id: String,
    pub controller: Option<String>,
    pub verification_methods: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub key_agreement: Vec<String>,
    pub key_encapsulation: Vec<String>,
    pub capability_invocation: Vec<String>,
    pub capability_delegation: Vec<String>,
    pub services: Vec<Service>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub version_id: String,
    pub next_update: Option<DateTime<Utc>>,
    pub proof: Option<Proof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub key_type: String,
    pub controller: String,
    pub public_key_multibase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    pub service_type: String,
    pub service_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub proof_type: String,
    pub created: DateTime<Utc>,
    pub verification_method: String,
    pub proof_purpose: String,
    pub proof_value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidMetadata {
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub version_id: String,
    pub next_update: Option<DateTime<Utc>>,
    pub deactivated: bool,
    pub deactivated_reason: Option<String>,
}

pub struct DidIdentity {
    store: DidStore,
}

impl DidIdentity {
    /// Create a new DID identity service
    pub fn new(store: DidStore) -> Self {
        Self { store }
    }
    
    /// Create a new DID with post-quantum keys
    pub async fn create_did(
        &self,
        method: &str,
        key_types: Vec<KeyType>,
        purposes: Vec<KeyPurpose>,
    ) -> Result<(String, DidDocument), DidError> {
        info!("Creating new DID with method: {}, key types: {:?}", method, key_types);
        
        // Generate unique DID identifier
        let did_id = self.generate_did_id(method);
        let did_string = format!("did:{}:{}", method, did_id);
        
        // Generate keys for each specified type
        let mut keys = Vec::new();
        let mut verification_methods = Vec::new();
        
        for key_type in key_types {
            let key = self.generate_key(key_type, &purposes).await?;
            keys.push(key.clone());
            
            // Create verification method
            let verification_method = self.create_verification_method(&did_string, &key)?;
            verification_methods.push(verification_method);
        }
        
        // Create DID document
        let did_doc = DidDocument {
            id: did_string.clone(),
            controller: None,
            verification_methods,
            authentication: keys.iter()
                .filter(|k| k.purpose.contains(&KeyPurpose::Authentication))
                .map(|k| k.id.clone())
                .collect(),
            assertion_method: keys.iter()
                .filter(|k| k.purpose.contains(&KeyPurpose::Assertion))
                .map(|k| k.id.clone())
                .collect(),
            key_agreement: keys.iter()
                .filter(|k| k.purpose.contains(&KeyPurpose::KeyAgreement))
                .map(|k| k.id.clone())
                .collect(),
            key_encapsulation: keys.iter()
                .filter(|k| k.purpose.contains(&KeyPurpose::KeyEncapsulation))
                .map(|k| k.id.clone())
                .collect(),
            capability_invocation: keys.iter()
                .filter(|k| k.purpose.contains(&KeyPurpose::CapabilityInvocation))
                .map(|k| k.id.clone())
                .collect(),
            capability_delegation: keys.iter()
                .filter(|k| k.purpose.contains(&KeyPurpose::CapabilityDelegation))
                .map(|k| k.id.clone())
                .collect(),
            services: Vec::new(),
            created: Utc::now(),
            updated: Utc::now(),
            version_id: Uuid::new_v4().to_string(),
            next_update: None,
            proof: None,
        };
        
        // Store DID document and keys
        self.store.store_did_document(&did_string, &did_doc).await?;
        for key in keys {
            self.store.store_did_key(&did_string, &key).await?;
        }
        
        info!("Created DID: {} with {} keys", did_string, keys.len());
        
        Ok((did_string, did_doc))
    }
    
    /// Resolve a DID to its document
    pub async fn resolve_did(&self, did: &str) -> Result<Option<DidDocument>, DidError> {
        debug!("Resolving DID: {}", did);
        
        // Retrieve from store
        let did_doc = self.store.get_did_document(did).await?;
        
        if let Some(doc) = &did_doc {
            info!("Resolved DID: {} (version: {})", did, doc.version_id);
        } else {
            debug!("DID not found: {}", did);
        }
        
        Ok(did_doc)
    }
    
    /// Rotate keys for a DID
    pub async fn rotate_keys(
        &self,
        did: &str,
        key_ids: Vec<String>,
        new_key_types: Vec<KeyType>,
        new_purposes: Vec<KeyPurpose>,
    ) -> Result<DidDocument, DidError> {
        info!("Rotating keys for DID: {}, keys: {:?}", did, key_ids);
        
        // Get current DID document
        let current_doc = self.resolve_did(did).await?
            .ok_or_else(|| DidError::DidNotFound(did.to_string()))?;
        
        // Revoke old keys
        for key_id in &key_ids {
            self.store.revoke_did_key(did, key_id).await?;
        }
        
        // Generate new keys
        let mut new_keys = Vec::new();
        let mut new_verification_methods = Vec::new();
        
        for key_type in new_key_types {
            let key = self.generate_key(key_type, &new_purposes).await?;
            new_keys.push(key.clone());
            
            let verification_method = self.create_verification_method(did, &key)?;
            new_verification_methods.push(verification_method);
        }
        
        // Create updated DID document
        let mut updated_doc = current_doc.clone();
        updated_doc.updated = Utc::now();
        updated_doc.version_id = Uuid::new_v4().to_string();
        
        // Update verification methods
        updated_doc.verification_methods.retain(|vm| !key_ids.contains(&vm.id));
        updated_doc.verification_methods.extend(new_verification_methods);
        
        // Update key references
        self.update_key_references(&mut updated_doc, &key_ids, &new_keys);
        
        // Store updated document and new keys
        self.store.store_did_document(did, &updated_doc).await?;
        for key in new_keys {
            self.store.store_did_key(did, &key).await?;
        }
        
        info!("Successfully rotated keys for DID: {}", did);
        
        Ok(updated_doc)
    }
    
    /// Get all keys for a DID
    pub async fn get_did_keys(&self, did: &str) -> Result<Vec<DidKey>, DidError> {
        self.store.get_did_keys(did).await
    }
    
    /// Deactivate a DID
    pub async fn deactivate_did(&self, did: &str, reason: Option<String>) -> Result<(), DidError> {
        info!("Deactivating DID: {} (reason: {:?})", did, reason);
        
        let mut doc = self.resolve_did(did).await?
            .ok_or_else(|| DidError::DidNotFound(did.to_string()))?;
        
        doc.updated = Utc::now();
        doc.version_id = Uuid::new_v4().to_string();
        
        // Store deactivated document
        self.store.store_did_document(did, &doc).await?;
        
        // Mark as deactivated in metadata
        self.store.deactivate_did(did, reason).await?;
        
        info!("DID deactivated: {}", did);
        Ok(())
    }
    
    /// Generate a unique DID identifier
    fn generate_did_id(&self, method: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(method.as_bytes());
        hasher.update(Uuid::new_v4().as_bytes());
        hasher.update(Utc::now().timestamp().to_le_bytes());
        
        let hash = hasher.finalize();
        hex::encode(&hash[..16]) // Use first 16 bytes for shorter ID
    }
    
    /// Generate a new key of specified type
    async fn generate_key(
        &self,
        key_type: KeyType,
        purposes: &[KeyPurpose],
    ) -> Result<DidKey, DidError> {
        let key_id = Uuid::new_v4().to_string();
        let created = Utc::now();
        
        let (public_key, private_key) = match key_type {
            KeyType::Ed25519 => self.generate_ed25519_key().await?,
            KeyType::Dilithium3 => self.generate_dilithium3_key().await?,
            KeyType::Dilithium5 => self.generate_dilithium5_key().await?,
            KeyType::Kyber512 => self.generate_kyber512_key().await?,
            KeyType::Kyber768 => self.generate_kyber768_key().await?,
            KeyType::Kyber1024 => self.generate_kyber1024_key().await?,
            KeyType::Ed25519Dilithium3 => self.generate_hybrid_ed25519_dilithium3_key().await?,
            KeyType::Ed25519Kyber512 => self.generate_hybrid_ed25519_kyber512_key().await?,
        };
        
        Ok(DidKey {
            id: key_id,
            key_type,
            public_key,
            private_key: Some(private_key),
            created,
            expires: None,
            revoked: false,
            purpose: purposes.to_vec(),
        })
    }
    
    /// Generate Ed25519 key pair
    async fn generate_ed25519_key(&self) -> Result<(Vec<u8>, Vec<u8>), DidError> {
        use ed25519_dalek::{SigningKey, VerifyingKey};
        
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        
        Ok((
            verifying_key.to_bytes().to_vec(),
            signing_key.to_bytes().to_vec(),
        ))
    }
    
    /// Generate Dilithium3 key pair
    async fn generate_dilithium3_key(&self) -> Result<(Vec<u8>, Vec<u8>), DidError> {
        use dilithium::Dilithium3;
        
        let (public_key, private_key) = Dilithium3::generate_keypair();
        
        Ok((
            public_key.to_bytes().to_vec(),
            private_key.to_bytes().to_vec(),
        ))
    }
    
    /// Generate Dilithium5 key pair
    async fn generate_dilithium5_key(&self) -> Result<(Vec<u8>, Vec<u8>), DidError> {
        use dilithium::Dilithium5;
        
        let (public_key, private_key) = Dilithium5::generate_keypair();
        
        Ok((
            public_key.to_bytes().to_vec(),
            private_key.to_bytes().to_vec(),
        ))
    }
    
    /// Generate Kyber512 key pair
    async fn generate_kyber512_key(&self) -> Result<(Vec<u8>, Vec<u8>), DidError> {
        use kyber::Kyber512;
        
        let (public_key, private_key) = Kyber512::generate_keypair();
        
        Ok((
            public_key.to_bytes().to_vec(),
            private_key.to_bytes().to_vec(),
        ))
    }
    
    /// Generate Kyber768 key pair
    async fn generate_kyber768_key(&self) -> Result<(Vec<u8>, Vec<u8>), DidError> {
        use kyber::Kyber768;
        
        let (public_key, private_key) = Kyber768::generate_keypair();
        
        Ok((
            public_key.to_bytes().to_vec(),
            private_key.to_bytes().to_vec(),
        ))
    }
    
    /// Generate Kyber1024 key pair
    async fn generate_kyber1024_key(&self) -> Result<(Vec<u8>, Vec<u8>), DidError> {
        use kyber::Kyber1024;
        
        let (public_key, private_key) = Kyber1024::generate_keypair();
        
        Ok((
            public_key.to_bytes().to_vec(),
            private_key.to_bytes().to_vec(),
        ))
    }
    
    /// Generate hybrid Ed25519 + Dilithium3 key pair
    async fn generate_hybrid_ed25519_dilithium3_key(&self) -> Result<(Vec<u8>, Vec<u8>), DidError> {
        let (ed25519_pub, ed25519_priv) = self.generate_ed25519_key().await?;
        let (dilithium_pub, dilithium_priv) = self.generate_dilithium3_key().await?;
        
        // Combine public keys
        let mut combined_pub = Vec::new();
        combined_pub.extend_from_slice(&ed25519_pub);
        combined_pub.extend_from_slice(&dilithium_pub);
        
        // Combine private keys
        let mut combined_priv = Vec::new();
        combined_priv.extend_from_slice(&ed25519_priv);
        combined_priv.extend_from_slice(&dilithium_priv);
        
        Ok((combined_pub, combined_priv))
    }
    
    /// Generate hybrid Ed25519 + Kyber512 key pair
    async fn generate_hybrid_ed25519_kyber512_key(&self) -> Result<(Vec<u8>, Vec<u8>), DidError> {
        let (ed25519_pub, ed25519_priv) = self.generate_ed25519_key().await?;
        let (kyber_pub, kyber_priv) = self.generate_kyber512_key().await?;
        
        // Combine public keys
        let mut combined_pub = Vec::new();
        combined_pub.extend_from_slice(&ed25519_pub);
        combined_pub.extend_from_slice(&kyber_pub);
        
        // Combine private keys
        let mut combined_priv = Vec::new();
        combined_priv.extend_from_slice(&ed25519_priv);
        combined_priv.extend_from_slice(&kyber_priv);
        
        Ok((combined_pub, combined_priv))
    }
    
    /// Create verification method from key
    fn create_verification_method(
        &self,
        did: &str,
        key: &DidKey,
    ) -> Result<VerificationMethod, DidError> {
        let key_type = match key.key_type {
            KeyType::Ed25519 => "Ed25519VerificationKey2020",
            KeyType::Dilithium3 => "Dilithium3VerificationKey2020",
            KeyType::Dilithium5 => "Dilithium5VerificationKey2020",
            KeyType::Kyber512 => "Kyber512VerificationKey2020",
            KeyType::Kyber768 => "Kyber768VerificationKey2020",
            KeyType::Kyber1024 => "Kyber1024VerificationKey2020",
            KeyType::Ed25519Dilithium3 => "Ed25519Dilithium3VerificationKey2020",
            KeyType::Ed25519Kyber512 => "Ed25519Kyber512VerificationKey2020",
        };
        
        Ok(VerificationMethod {
            id: format!("{}#{}", did, key.id),
            key_type: key_type.to_string(),
            controller: did.to_string(),
            public_key_multibase: Some(base64::encode(&key.public_key)),
        })
    }
    
    /// Update key references in DID document
    fn update_key_references(
        &self,
        doc: &mut DidDocument,
        old_key_ids: &[String],
        new_keys: &[DidKey],
    ) {
        // Remove old key references
        doc.authentication.retain(|id| !old_key_ids.contains(id));
        doc.assertion_method.retain(|id| !old_key_ids.contains(id));
        doc.key_agreement.retain(|id| !old_key_ids.contains(id));
        doc.key_encapsulation.retain(|id| !old_key_ids.contains(id));
        doc.capability_invocation.retain(|id| !old_key_ids.contains(id));
        doc.capability_delegation.retain(|id| !old_key_ids.contains(id));
        
        // Add new key references
        for key in new_keys {
            if key.purpose.contains(&KeyPurpose::Authentication) {
                doc.authentication.push(key.id.clone());
            }
            if key.purpose.contains(&KeyPurpose::Assertion) {
                doc.assertion_method.push(key.id.clone());
            }
            if key.purpose.contains(&KeyPurpose::KeyAgreement) {
                doc.key_agreement.push(key.id.clone());
            }
            if key.purpose.contains(&KeyPurpose::KeyEncapsulation) {
                doc.key_encapsulation.push(key.id.clone());
            }
            if key.purpose.contains(&KeyPurpose::CapabilityInvocation) {
                doc.capability_invocation.push(key.id.clone());
            }
            if key.purpose.contains(&KeyPurpose::CapabilityDelegation) {
                doc.capability_delegation.push(key.id.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dstore::DidStore;
    use tempfile::tempdir;
    
    async fn create_test_store() -> DidStore {
        let temp_dir = tempdir().unwrap();
        DidStore::new(temp_dir.path().to_path_buf()).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_create_did() {
        let store = create_test_store().await;
        let identity = DidIdentity::new(store);
        
        let (did, doc) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519, KeyType::Dilithium3],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion],
        ).await.unwrap();
        
        assert!(did.starts_with("did:polynet:"));
        assert_eq!(doc.verification_methods.len(), 2);
        assert_eq!(doc.authentication.len(), 2);
        assert_eq!(doc.assertion_method.len(), 2);
    }
    
    #[tokio::test]
    async fn test_resolve_did() {
        let store = create_test_store().await;
        let identity = DidIdentity::new(store);
        
        let (did, _) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        let resolved = identity.resolve_did(&did).await.unwrap();
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().id, did);
    }
    
    #[tokio::test]
    async fn test_rotate_keys() {
        let store = create_test_store().await;
        let identity = DidIdentity::new(store);
        
        let (did, original_doc) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        let original_version = original_doc.version_id.clone();
        
        // Get original keys
        let original_keys = identity.get_did_keys(&did).await.unwrap();
        let key_ids: Vec<String> = original_keys.iter().map(|k| k.id.clone()).collect();
        
        // Rotate keys
        let updated_doc = identity.rotate_keys(
            &did,
            key_ids,
            vec![KeyType::Dilithium3],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        assert_ne!(updated_doc.version_id, original_version);
        assert_eq!(updated_doc.verification_methods.len(), 1);
        
        // Verify old keys are revoked
        let keys = identity.get_did_keys(&did).await.unwrap();
        assert!(keys.iter().all(|k| k.revoked));
    }
}
