//! NGFS v1 Envelope Encryption Module
//! 
//! This module provides envelope encryption for CAS chunks using:
//! - XChaCha20-Poly1305 AEAD from libpolycrypto
//! - KEK derivation from KeyVault (X25519 default, Kyber KEM with "pqc" feature)
//! - Virtual clock-based nonce generation
//! - Deterministic associated data construction

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::schema::{EncHeaderV1, EncryptionAlg, ContentId, ContentType};
use polycrypto_sys::{xchacha20_seal, xchacha20_open, memzero, AETH_KEY_SIZE, AETH_NONCE_SIZE, AETH_TAG_SIZE};

/// Key ID type for KeyVault
pub type Kid = String;

/// Data encryption key (DEK) - 32 bytes
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
#[zeroize(drop)]
pub struct Dek([u8; AETH_KEY_SIZE]);

/// Key encryption key (KEK) - 32 bytes
#[derive(Debug, Clone, PartialEq, Eq, Zeroize)]
#[zeroize(drop)]
pub struct Kek([u8; AETH_KEY_SIZE]);

/// Nonce for encryption - 24 bytes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nonce([u8; AETH_NONCE_SIZE]);

/// Encrypted envelope containing header and data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncEnvelopeV1 {
    /// CBOR-serialized encryption header
    pub header: Vec<u8>,
    /// Encrypted data
    pub enc_data: Vec<u8>,
}

/// Associated data for encryption
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssociatedData {
    /// Schema hash for determinism
    pub schema_hash: [u8; 32],
    /// Key ID
    pub key_id: String,
    /// Algorithm identifier
    pub algorithm: EncryptionAlg,
    /// Chunk length
    pub chunk_len: u64,
    /// Additional metadata (sorted keys for determinism)
    pub metadata: BTreeMap<String, String>,
}

/// Encryption errors
#[derive(Debug, thiserror::Error)]
pub enum EncError {
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },
    
    #[error("Invalid nonce size: expected {expected}, got {actual}")]
    InvalidNonceSize { expected: usize, actual: usize },
    
    #[error("Invalid tag size: expected {expected}, got {actual}")]
    InvalidTagSize { expected: usize, actual: usize },
    
    #[error("Payload too large: {size} > {max}")]
    PayloadTooLarge { size: usize, max: usize },
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("KeyVault error: {0}")]
    KeyVaultError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid associated data: {0}")]
    InvalidAssociatedData(String),
}

/// Virtual clock for nonce generation
pub trait VirtualClock {
    /// Get current virtual clock value
    fn now(&self) -> u64;
    
    /// Get mount-specific salt
    fn get_mount_salt(&self) -> &[u8; 16];
}

/// KeyVault client for KEK derivation
pub trait KeyVaultClient {
    /// Derive KEK for a given key ID
    fn derive_kek_for_kid(&self, kid: &Kid) -> Result<Kek, EncError>;
}

/// Default virtual clock implementation
pub struct DefaultVirtualClock {
    counter: u64,
    mount_salt: [u8; 16],
}

impl DefaultVirtualClock {
    /// Create a new virtual clock with mount salt
    pub fn new(mount_salt: [u8; 16]) -> Self {
        Self {
            counter: 0,
            mount_salt,
        }
    }
    
    /// Increment the counter
    pub fn increment(&mut self) {
        self.counter += 1;
    }
}

impl VirtualClock for DefaultVirtualClock {
    fn now(&self) -> u64 {
        self.counter
    }
    
    fn get_mount_salt(&self) -> &[u8; 16] {
        &self.mount_salt
    }
}

/// NGFS encryption service
pub struct NgfsEncryption {
    virtual_clock: Box<dyn VirtualClock>,
    keyvault: Box<dyn KeyVaultClient>,
}

impl NgfsEncryption {
    /// Create a new NGFS encryption service
    pub fn new(
        virtual_clock: Box<dyn VirtualClock>,
        keyvault: Box<dyn KeyVaultClient>,
    ) -> Self {
        Self {
            virtual_clock,
            keyvault,
        }
    }
    
    /// Seal (encrypt) data into an envelope
    pub fn seal_envelope(
        &mut self,
        kid: Kid,
        dek: Dek,
        ad: &AssociatedData,
        plaintext: &[u8],
    ) -> Result<EncEnvelopeV1, EncError> {
        // Validate payload size
        if plaintext.len() > 256 * 1024 {
            return Err(EncError::PayloadTooLarge {
                size: plaintext.len(),
                max: 256 * 1024,
            });
        }
        
        // Generate nonce from virtual clock and mount salt
        let nonce = self.generate_nonce();
        
        // Derive KEK from KeyVault
        let kek = self.keyvault.derive_kek_for_kid(&kid)?;
        
        // Encrypt the DEK with KEK
        let (enc_dek, dek_tag) = xchacha20_seal(
            &kek.0,
            &nonce.0,
            &[],
            &dek.0,
        ).map_err(|e| EncError::EncryptionFailed(format!("DEK encryption failed: {:?}", e)))?;
        
        // Create encryption header
        let header = EncHeaderV1 {
            key_id: kid,
            algorithm: EncryptionAlg::XChaCha20Poly1305,
            nonce: nonce.0,
            tag: dek_tag,
            aad: Some(ad.serialize_to_cbor()?),
        };
        
        // Encrypt the plaintext with DEK
        let (enc_data, data_tag) = xchacha20_seal(
            &dek.0,
            &nonce.0,
            &ad.serialize_to_cbor()?,
            plaintext,
        ).map_err(|e| EncError::EncryptionFailed(format!("Data encryption failed: {:?}", e)))?;
        
        // Combine encrypted data with tag
        let mut final_enc_data = enc_data;
        final_enc_data.extend_from_slice(&data_tag);
        
        // Serialize header to CBOR
        let header_bytes = serde_cbor::to_vec(&header)
            .map_err(|e| EncError::SerializationError(format!("Header serialization failed: {}", e)))?;
        
        // Increment virtual clock
        self.virtual_clock.increment();
        
        Ok(EncEnvelopeV1 {
            header: header_bytes,
            enc_data: final_enc_data,
        })
    }
    
    /// Open (decrypt) data from an envelope
    pub fn open_envelope(
        &self,
        kid: &Kid,
        envelope: &EncEnvelopeV1,
        ad: &AssociatedData,
    ) -> Result<Vec<u8>, EncError> {
        // Deserialize header
        let header: EncHeaderV1 = serde_cbor::from_slice(&envelope.header)
            .map_err(|e| EncError::SerializationError(format!("Header deserialization failed: {}", e)))?;
        
        // Verify key ID matches
        if &header.key_id != kid {
            return Err(EncError::InvalidAssociatedData("Key ID mismatch".to_string()));
        }
        
        // Derive KEK from KeyVault
        let kek = self.keyvault.derive_kek_for_kid(kid)?;
        
        // Decrypt DEK with KEK
        let dek = xchacha20_open(
            &kek.0,
            &header.nonce,
            &[],
            &envelope.enc_data[..envelope.enc_data.len() - AETH_TAG_SIZE],
            &envelope.enc_data[envelope.enc_data.len() - AETH_TAG_SIZE..],
        ).map_err(|e| EncError::DecryptionFailed(format!("DEK decryption failed: {:?}", e)))?;
        
        // Verify DEK size
        if dek.len() != AETH_KEY_SIZE {
            return Err(EncError::InvalidKeySize {
                expected: AETH_KEY_SIZE,
                actual: dek.len(),
            });
        }
        
        let dek_array: [u8; AETH_KEY_SIZE] = dek.try_into()
            .map_err(|_| EncError::InvalidKeySize {
                expected: AETH_KEY_SIZE,
                actual: dek.len(),
            })?;
        
        let dek = Dek(dek_array);
        
        // Decrypt data with DEK
        let plaintext = xchacha20_open(
            &dek.0,
            &header.nonce,
            &ad.serialize_to_cbor()?,
            &envelope.enc_data[..envelope.enc_data.len() - AETH_TAG_SIZE],
            &envelope.enc_data[envelope.enc_data.len() - AETH_TAG_SIZE..],
        ).map_err(|e| EncError::DecryptionFailed(format!("Data decryption failed: {:?}", e)))?;
        
        Ok(plaintext)
    }
    
    /// Generate nonce from virtual clock and mount salt
    fn generate_nonce(&self) -> Nonce {
        let mut nonce = [0u8; AETH_NONCE_SIZE];
        
        // First 8 bytes: virtual clock counter (little-endian)
        let clock_bytes = self.virtual_clock.now().to_le_bytes();
        nonce[..8].copy_from_slice(&clock_bytes);
        
        // Next 16 bytes: mount salt
        let salt = self.virtual_clock.get_mount_salt();
        nonce[8..].copy_from_slice(salt);
        
        Nonce(nonce)
    }
}

impl AssociatedData {
    /// Serialize to CBOR for deterministic encryption
    pub fn serialize_to_cbor(&self) -> Result<Vec<u8>, EncError> {
        serde_cbor::to_vec(self)
            .map_err(|e| EncError::SerializationError(format!("Associated data serialization failed: {}", e)))
    }
    
    /// Create associated data for a chunk
    pub fn for_chunk(
        schema_hash: [u8; 32],
        key_id: String,
        algorithm: EncryptionAlg,
        chunk_len: u64,
        metadata: BTreeMap<String, String>,
    ) -> Self {
        Self {
            schema_hash,
            key_id,
            algorithm,
            chunk_len,
            metadata,
        }
    }
}

impl Dek {
    /// Create a new DEK from bytes
    pub fn new(key: [u8; AETH_KEY_SIZE]) -> Self {
        Self(key)
    }
    
    /// Generate a random DEK
    pub fn random() -> Self {
        use rand::RngCore;
        let mut key = [0u8; AETH_KEY_SIZE];
        rand::thread_rng().fill_bytes(&mut key);
        Self(key)
    }
    
    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; AETH_KEY_SIZE] {
        &self.0
    }
}

impl Kek {
    /// Create a new KEK from bytes
    pub fn new(key: [u8; AETH_KEY_SIZE]) -> Self {
        Self(key)
    }
    
    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; AETH_KEY_SIZE] {
        &self.0
    }
}

impl Nonce {
    /// Create a new nonce from bytes
    pub fn new(nonce: [u8; AETH_NONCE_SIZE]) -> Self {
        Self(nonce)
    }
    
    /// Get the nonce bytes
    pub fn as_bytes(&self) -> &[u8; AETH_NONCE_SIZE] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ngfs_schema_hash;
    
    struct MockKeyVault;
    impl KeyVaultClient for MockKeyVault {
        fn derive_kek_for_kid(&self, _kid: &Kid) -> Result<Kek, EncError> {
            // Return a fixed test key
            Ok(Kek([42u8; AETH_KEY_SIZE]))
        }
    }
    
    #[test]
    fn test_associated_data_serialization() {
        let mut metadata = BTreeMap::new();
        metadata.insert("test".to_string(), "value".to_string());
        
        let ad = AssociatedData::for_chunk(
            [1u8; 32],
            "test-key".to_string(),
            EncryptionAlg::XChaCha20Poly1305,
            1024,
            metadata,
        );
        
        let cbor = ad.serialize_to_cbor().unwrap();
        assert!(!cbor.is_empty());
    }
    
    #[test]
    fn test_nonce_generation() {
        let salt = [1u8; 16];
        let mut clock = DefaultVirtualClock::new(salt);
        
        let nonce1 = clock.generate_nonce();
        clock.increment();
        let nonce2 = clock.generate_nonce();
        
        assert_ne!(nonce1.0, nonce2.0);
        assert_eq!(nonce1.0[8..], salt);
        assert_eq!(nonce2.0[8..], salt);
    }
    
    #[test]
    fn test_dek_creation() {
        let key = [42u8; AETH_KEY_SIZE];
        let dek = Dek::new(key);
        assert_eq!(dek.as_bytes(), &key);
    }
    
    #[test]
    fn test_kek_creation() {
        let key = [42u8; AETH_KEY_SIZE];
        let kek = Kek::new(key);
        assert_eq!(kek.as_bytes(), &key);
    }
}
