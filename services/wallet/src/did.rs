//! DID (Decentralized Identifier) implementation for wallet service

use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::WalletError;

/// Supported DID methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DIDMethod {
    Key,    // did:key
    Pkh,    // did:pkh (Ethereum)
    Web,    // did:web (placeholder)
}

/// DID Document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    pub id: String,
    pub context: Vec<String>,
    pub verification_methods: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub key_agreement: Vec<String>,
    pub service_endpoints: Vec<ServiceEndpoint>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

/// Verification method in DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub type_: String,
    pub controller: String,
    pub public_key_multibase: Option<String>,
    pub public_key_jwk: Option<serde_json::Value>,
}

/// Service endpoint in DID document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub id: String,
    pub type_: String,
    pub service_endpoint: String,
}

/// DID resolver for handling different DID methods
pub struct DIDResolver {
    key_dids: HashMap<String, DIDDocument>,
    pkh_dids: HashMap<String, DIDDocument>,
    web_dids: HashMap<String, DIDDocument>,
}

impl DIDResolver {
    /// Create a new DID resolver
    pub fn new() -> Self {
        Self {
            key_dids: HashMap::new(),
            pkh_dids: HashMap::new(),
            web_dids: HashMap::new(),
        }
    }

    /// Create a new DID document
    pub async fn create_did(
        &mut self,
        method: DIDMethod,
        key_id: Option<&Uuid>,
    ) -> Result<DIDDocument, WalletError> {
        let did = match method {
            DIDMethod::Key => {
                let key_identifier = key_id
                    .map(|id| hex::encode(id.as_bytes()))
                    .unwrap_or_else(|| hex::encode(blake3::hash(Uuid::new_v4().as_bytes()).as_bytes()));
                format!("did:key:{}", key_identifier)
            }
            DIDMethod::Pkh => {
                // Mock Ethereum address
                let address = format!("0x{}", hex::encode(&[0u8; 20]));
                format!("did:pkh:eip155:1:{}", address)
            }
            DIDMethod::Web => {
                format!("did:web:example.com:{}", Uuid::new_v4())
            }
        };

        let did_doc = self.create_did_document(&did, method, key_id).await?;

        // Store in appropriate registry
        match method {
            DIDMethod::Key => {
                self.key_dids.insert(did.clone(), did_doc.clone());
            }
            DIDMethod::Pkh => {
                self.pkh_dids.insert(did.clone(), did_doc.clone());
            }
            DIDMethod::Web => {
                self.web_dids.insert(did.clone(), did_doc.clone());
            }
        }

        Ok(did_doc)
    }

    /// Resolve a DID to its document
    pub async fn resolve(&self, did: &str) -> Result<DIDDocument, WalletError> {
        if let Some(doc) = self.key_dids.get(did) {
            return Ok(doc.clone());
        }
        if let Some(doc) = self.pkh_dids.get(did) {
            return Ok(doc.clone());
        }
        if let Some(doc) = self.web_dids.get(did) {
            return Ok(doc.clone());
        }

        // For offline mode, try to resolve did:key deterministically
        if did.starts_with("did:key:") {
            return self.resolve_key_did(did).await;
        }

        Err(WalletError::DIDResolutionFailed(format!("DID not found: {}", did)))
    }

    /// Create a DID document structure
    async fn create_did_document(
        &self,
        did: &str,
        method: DIDMethod,
        key_id: Option<&Uuid>,
    ) -> Result<DIDDocument, WalletError> {
        let now = Utc::now();
        let verification_method_id = format!("{}#key-1", did);

        let verification_method = VerificationMethod {
            id: verification_method_id.clone(),
            type_: match method {
                DIDMethod::Key => "Ed25519VerificationKey2020".to_string(),
                DIDMethod::Pkh => "EcdsaSecp256k1RecoveryMethod2020".to_string(),
                DIDMethod::Web => "JsonWebKey2020".to_string(),
            },
            controller: did.to_string(),
            public_key_multibase: key_id.map(|id| {
                let key_bytes = id.as_bytes();
                format!("z{}", bs58::encode(key_bytes).into_string())
            }),
            public_key_jwk: None,
        };

        Ok(DIDDocument {
            id: did.to_string(),
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
            ],
            verification_methods: vec![verification_method.clone()],
            authentication: vec![verification_method_id.clone()],
            assertion_method: vec![verification_method_id.clone()],
            key_agreement: vec![],
            service_endpoints: vec![],
            created: now,
            updated: now,
        })
    }

    /// Resolve a did:key deterministically
    async fn resolve_key_did(&self, did: &str) -> Result<DIDDocument, WalletError> {
        // Extract the key identifier
        let key_identifier = did.strip_prefix("did:key:")
            .ok_or_else(|| WalletError::DIDResolutionFailed("Invalid did:key format".to_string()))?;

        // For offline mode, create a deterministic DID document
        let verification_method_id = format!("{}#key-1", did);
        let now = Utc::now();

        let verification_method = VerificationMethod {
            id: verification_method_id.clone(),
            type_: "Ed25519VerificationKey2020".to_string(),
            controller: did.to_string(),
            public_key_multibase: Some(format!("z{}", key_identifier)),
            public_key_jwk: None,
        };

        Ok(DIDDocument {
            id: did.to_string(),
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
            ],
            verification_methods: vec![verification_method.clone()],
            authentication: vec![verification_method_id.clone()],
            assertion_method: vec![verification_method_id.clone()],
            key_agreement: vec![],
            service_endpoints: vec![],
            created: now,
            updated: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_did_resolver_creation() {
        let resolver = DIDResolver::new();
        assert!(resolver.key_dids.is_empty());
    }

    #[tokio::test]
    async fn test_did_creation() {
        let mut resolver = DIDResolver::new();
        let did_doc = resolver.create_did(DIDMethod::Key, None).await;
        assert!(did_doc.is_ok());
        
        let doc = did_doc.unwrap();
        assert!(doc.id.starts_with("did:key:"));
        assert!(!doc.verification_methods.is_empty());
    }

    #[tokio::test]
    async fn test_did_resolution() {
        let mut resolver = DIDResolver::new();
        let did_doc = resolver.create_did(DIDMethod::Key, None).await.unwrap();
        let resolved = resolver.resolve(&did_doc.id).await;
        assert!(resolved.is_ok());
        
        let resolved_doc = resolved.unwrap();
        assert_eq!(resolved_doc.id, did_doc.id);
    }
}
