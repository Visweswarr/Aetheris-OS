use libp2p::{identity, PeerId};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use thiserror::Error;
use tracing::{debug, info, warn, error};

#[derive(Debug, Error)]
pub enum DidError {
    #[error("Invalid DID format: {0}")]
    InvalidDidFormat(String),
    
    #[error("Invalid DID key format: {0}")]
    InvalidDidKeyFormat(String),
    
    #[error("Failed to generate keypair: {0}")]
    KeypairGeneration(String),
    
    #[error("Failed to derive peer ID: {0}")]
    PeerIdDerivation(String),
    
    #[error("Invalid multibase encoding: {0}")]
    InvalidMultibase(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidPeerId {
    pub did: String,
    pub peer_id: PeerId,
    pub keypair: DidKeypair,
    pub metadata: DidMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidKeypair {
    pub public_key: Vec<u8>,
    pub private_key: Option<Vec<u8>>, // None for public-only keys
    pub key_type: DidKeyType,
    pub encoding: DidKeyEncoding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DidKeyType {
    Ed25519,
    X25519,
    Secp256k1,
    P256,
    Bls12381G1,
    Bls12381G2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DidKeyEncoding {
    Base58,
    Base64,
    Hex,
    Multibase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidMetadata {
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
    pub controller: Option<String>,
    pub verification_method: Option<String>,
    pub authentication: Option<Vec<String>>,
    pub assertion_method: Option<Vec<String>>,
    pub key_agreement: Option<Vec<String>>,
    pub capability_invocation: Option<Vec<String>>,
    pub capability_delegation: Option<Vec<String>>,
}

impl DidPeerId {
    /// Generate a new DID-based peer ID
    pub fn generate() -> Result<Self, DidError> {
        info!("Generating new DID-based peer ID");
        
        // Generate Ed25519 keypair
        let keypair = identity::Keypair::generate_ed25519();
        let public_key = keypair.public().encode_protobuf();
        
        // Create DID from public key hash
        let did = Self::create_did_from_public_key(&public_key)?;
        
        // Derive peer ID from public key
        let peer_id = PeerId::from_public_key(&keypair.public());
        
        // Create DID keypair
        let did_keypair = DidKeypair {
            public_key,
            private_key: Some(keypair.secret().encode_protobuf()),
            key_type: DidKeyType::Ed25519,
            encoding: DidKeyEncoding::Multibase,
        };
        
        // Create metadata
        let now = chrono::Utc::now();
        let metadata = DidMetadata {
            created: now,
            updated: now,
            controller: Some(did.clone()),
            verification_method: Some(format!("{}#{}", did, "key-1")),
            authentication: Some(vec![format!("{}#{}", did, "key-1")]),
            assertion_method: Some(vec![format!("{}#{}", did, "key-1")]),
            key_agreement: Some(vec![format!("{}#{}", did, "key-1")]),
            capability_invocation: Some(vec![format!("{}#{}", did, "key-1")]),
            capability_delegation: Some(vec![format!("{}#{}", did, "key-1")]),
        };
        
        let did_peer_id = Self {
            did,
            peer_id,
            keypair: did_keypair,
            metadata,
        };
        
        info!("Generated DID: {} -> Peer ID: {}", did_peer_id.did, did_peer_id.peer_id);
        
        Ok(did_peer_id)
    }
    
    /// Create DID-based peer ID from existing DID key
    pub fn from_did_key(did_key: &str) -> Result<Self, DidError> {
        info!("Creating DID-based peer ID from key: {}", did_key);
        
        // Parse DID key format
        let (did, key_id, public_key) = Self::parse_did_key(did_key)?;
        
        // Derive peer ID from public key
        let peer_id = Self::derive_peer_id_from_public_key(&public_key)?;
        
        // Create DID keypair
        let did_keypair = DidKeypair {
            public_key,
            private_key: None, // Public key only
            key_type: Self::detect_key_type(&public_key)?,
            encoding: DidKeyEncoding::Multibase,
        };
        
        // Create metadata
        let now = chrono::Utc::now();
        let metadata = DidMetadata {
            created: now,
            updated: now,
            controller: Some(did.clone()),
            verification_method: Some(format!("{}#{}", did, key_id)),
            authentication: Some(vec![format!("{}#{}", did, key_id)]),
            assertion_method: Some(vec![format!("{}#{}", did, key_id)]),
            key_agreement: Some(vec![format!("{}#{}", did, key_id)]),
            capability_invocation: Some(vec![format!("{}#{}", did, key_id)]),
            capability_delegation: Some(vec![format!("{}#{}", did, key_id)]),
        };
        
        let did_peer_id = Self {
            did,
            peer_id,
            keypair: did_keypair,
            metadata,
        };
        
        info!("Created DID: {} -> Peer ID: {}", did_peer_id.did, did_peer_id.peer_id);
        
        Ok(did_peer_id)
    }
    
    /// Create DID from public key hash
    fn create_did_from_public_key(public_key: &[u8]) -> Result<String, DidError> {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        
        // Use first 16 bytes of hash for DID
        let did_suffix = hex::encode(&hash[..16]);
        Ok(format!("did:polynet:{}", did_suffix))
    }
    
    /// Parse DID key format (did:method:identifier#key-id)
    fn parse_did_key(did_key: &str) -> Result<(String, String, Vec<u8>), DidError> {
        let parts: Vec<&str> = did_key.split('#').collect();
        if parts.len() != 2 {
            return Err(DidError::InvalidDidKeyFormat(
                "Expected format: did:method:identifier#key-id".to_string()
            ));
        }
        
        let did = parts[0].to_string();
        let key_id = parts[1].to_string();
        
        // Extract public key from DID (simplified - in practice this would be more complex)
        let public_key = Self::extract_public_key_from_did(&did)?;
        
        Ok((did, key_id, public_key))
    }
    
    /// Extract public key from DID (simplified implementation)
    fn extract_public_key_from_did(did: &str) -> Result<Vec<u8>, DidError> {
        // This is a simplified implementation
        // In practice, you would resolve the DID document and extract the public key
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return Err(DidError::InvalidDidFormat(
                "Invalid DID format".to_string()
            ));
        }
        
        let identifier = parts[2];
        
        // For now, generate a deterministic public key from the identifier
        let mut hasher = Sha256::new();
        hasher.update(identifier.as_bytes());
        let hash = hasher.finalize();
        
        // Use first 32 bytes as public key
        Ok(hash.to_vec())
    }
    
    /// Derive peer ID from public key
    fn derive_peer_id_from_public_key(public_key: &[u8]) -> Result<PeerId, DidError> {
        // Create Ed25519 public key from bytes
        let public_key = identity::PublicKey::try_decode_protobuf(public_key)
            .map_err(|e| DidError::PeerIdDerivation(e.to_string()))?;
        
        // Derive peer ID
        Ok(PeerId::from_public_key(&public_key))
    }
    
    /// Detect key type from public key
    fn detect_key_type(public_key: &[u8]) -> Result<DidKeyType, DidError> {
        if public_key.len() == 32 {
            Ok(DidKeyType::Ed25519)
        } else if public_key.len() == 33 {
            Ok(DidKeyType::Secp256k1)
        } else if public_key.len() == 65 {
            Ok(DidKeyType::P256)
        } else if public_key.len() == 48 {
            Ok(DidKeyType::Bls12381G1)
        } else if public_key.len() == 96 {
            Ok(DidKeyType::Bls12381G2)
        } else {
            Err(DidError::InvalidDidKeyFormat(
                format!("Unknown key type for length: {}", public_key.len())
            ))
        }
    }
    
    /// Get the DID string
    pub fn did(&self) -> &str {
        &self.did
    }
    
    /// Get the peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
    
    /// Get the public key
    pub fn public_key(&self) -> &[u8] {
        &self.keypair.public_key
    }
    
    /// Get the key type
    pub fn key_type(&self) -> &DidKeyType {
        &self.keypair.key_type
    }
    
    /// Check if this is a public-only key
    pub fn is_public_only(&self) -> bool {
        self.keypair.private_key.is_none()
    }
    
    /// Get the metadata
    pub fn metadata(&self) -> &DidMetadata {
        &self.metadata
    }
    
    /// Convert to libp2p identity keypair (if private key is available)
    pub fn to_libp2p_keypair(&self) -> Result<identity::Keypair, DidError> {
        if let Some(private_key_bytes) = &self.keypair.private_key {
            identity::Keypair::try_decode_protobuf(private_key_bytes)
                .map_err(|e| DidError::KeypairGeneration(e.to_string()))
        } else {
            Err(DidError::KeypairGeneration(
                "Private key not available".to_string()
            ))
        }
    }
    
    /// Export as DID document
    pub fn to_did_document(&self) -> DidDocument {
        DidDocument {
            id: self.did.clone(),
            controller: self.metadata.controller.clone(),
            verification_method: vec![VerificationMethod {
                id: format!("{}#{}", self.did, "key-1"),
                controller: self.did.clone(),
                key_type: format!("{}", self.key_type()),
                public_key_multibase: multibase::encode(
                    multibase::Base::Base58Btc,
                    &self.keypair.public_key
                ),
            }],
            authentication: self.metadata.authentication.clone(),
            assertion_method: self.metadata.assertion_method.clone(),
            key_agreement: self.metadata.key_agreement.clone(),
            capability_invocation: self.metadata.capability_invocation.clone(),
            capability_delegation: self.metadata.capability_delegation.clone(),
            created: self.metadata.created,
            updated: self.metadata.updated,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocument {
    pub id: String,
    pub controller: Option<String>,
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Option<Vec<String>>,
    pub assertion_method: Option<Vec<String>>,
    pub key_agreement: Option<Vec<String>>,
    pub capability_invocation: Option<Vec<String>>,
    pub capability_delegation: Option<Vec<String>>,
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub controller: String,
    pub key_type: String,
    pub public_key_multibase: String,
}

impl std::fmt::Display for DidPeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.did, self.peer_id)
    }
}

impl std::fmt::Display for DidKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DidKeyType::Ed25519 => write!(f, "Ed25519"),
            DidKeyType::X25519 => write!(f, "X25519"),
            DidKeyType::Secp256k1 => write!(f, "Secp256k1"),
            DidKeyType::P256 => write!(f, "P-256"),
            DidKeyType::Bls12381G1 => write!(f, "BLS12381G1"),
            DidKeyType::Bls12381G2 => write!(f, "BLS12381G2"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_did_peer_id_generation() {
        let did_peer_id = DidPeerId::generate().unwrap();
        
        assert!(!did_peer_id.did.is_empty());
        assert!(did_peer_id.did.starts_with("did:polynet:"));
        assert!(!did_peer_id.is_public_only());
        assert_eq!(did_peer_id.key_type(), &DidKeyType::Ed25519);
    }
    
    #[test]
    fn test_did_peer_id_from_key() {
        let did_key = "did:polynet:1234567890abcdef#key-1";
        let did_peer_id = DidPeerId::from_did_key(did_key).unwrap();
        
        assert_eq!(did_peer_id.did, "did:polynet:1234567890abcdef");
        assert!(did_peer_id.is_public_only());
    }
    
    #[test]
    fn test_did_document_creation() {
        let did_peer_id = DidPeerId::generate().unwrap();
        let document = did_peer_id.to_did_document();
        
        assert_eq!(document.id, did_peer_id.did);
        assert!(!document.verification_method.is_empty());
    }
    
    #[test]
    fn test_key_type_detection() {
        let ed25519_key = vec![0u8; 32];
        let key_type = DidPeerId::detect_key_type(&ed25519_key).unwrap();
        assert_eq!(key_type, DidKeyType::Ed25519);
        
        let secp256k1_key = vec![0u8; 33];
        let key_type = DidPeerId::detect_key_type(&secp256k1_key).unwrap();
        assert_eq!(key_type, DidKeyType::Secp256k1);
    }
}
