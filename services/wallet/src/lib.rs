//! # Aetheris OS Wallet Service
//!
//! A DID-bound wallet and keyvault service providing multi-algorithm key support
//! with PDV-backed storage and polyglot bindings.

pub mod keystore;
pub mod did;
pub mod cap;
pub mod ffi;
pub mod error;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::WalletError;
use crate::keystore::{KeyStore, KeyType, KeyMaterial, KeyDerivationPath, Signature};
use crate::did::{DIDResolver, DIDDocument, DIDMethod};
use crate::cap::{CapabilityManager, WalletCapability};

/// Main wallet service providing DID-bound key management
pub struct WalletService {
    keystore: Arc<RwLock<KeyStore>>,
    did_resolver: Arc<RwLock<DIDResolver>>,
    capability_manager: Arc<CapabilityManager>,
    audit_log: Arc<RwLock<Vec<AuditEntry>>>,
}

/// Audit entry for tracking all wallet operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub subject: String,
    pub intent_id: Option<String>,
    pub capability: String,
    pub result: AuditResult,
    pub metadata: HashMap<String, String>,
}

/// Result of an audited operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuditResult {
    Success,
    Failure { reason: String },
    Denied { reason: String },
}

/// Wallet configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletConfig {
    pub pdv_path: String,
    pub max_keys_per_wallet: usize,
    pub keygen_rate_limit: u32, // keys per day
    pub audit_retention_days: u32,
    pub pqc_enabled: bool,
    pub hsm_enabled: bool,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            pdv_path: "/pdv/wallet".to_string(),
            max_keys_per_wallet: 1000,
            keygen_rate_limit: 10,
            audit_retention_days: 90,
            pqc_enabled: true,
            hsm_enabled: false,
        }
    }
}

/// Wallet initialization result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletInitResult {
    pub wallet_id: Uuid,
    pub did: String,
    pub pdv_location: String,
    pub created_at: DateTime<Utc>,
}

/// Key generation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyGenResult {
    pub key_id: Uuid,
    pub key_type: KeyType,
    pub public_key: Vec<u8>,
    pub did_binding: Option<String>,
    pub pdv_item_id: String,
    pub derivation_path: Option<KeyDerivationPath>,
}

/// Key derivation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyDeriveResult {
    pub key_id: Uuid,
    pub derived_key: Vec<u8>,
    pub address: Option<String>,
    pub derivation_path: KeyDerivationPath,
    pub persisted: bool,
}

/// Signature result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignResult {
    pub signature: Vec<u8>,
    pub recovery_id: Option<u8>,
    pub pqc_signature: Option<Vec<u8>>, // For hybrid signatures
    pub algorithm: String,
}

impl WalletService {
    /// Create a new wallet service instance
    pub async fn new(config: WalletConfig) -> Result<Self, WalletError> {
        let keystore = Arc::new(RwLock::new(KeyStore::new(config.pdv_path.clone()).await?));
        let did_resolver = Arc::new(RwLock::new(DIDResolver::new()));
        let capability_manager = Arc::new(CapabilityManager::new());
        let audit_log = Arc::new(RwLock::new(Vec::new()));

        Ok(Self {
            keystore,
            did_resolver,
            capability_manager,
            audit_log,
        })
    }

    /// Initialize a new wallet with DID binding
    pub async fn init_wallet(
        &self,
        subject: &str,
        intent_id: Option<&str>,
    ) -> Result<WalletInitResult, WalletError> {
        let capability = WalletCapability::WalletCreate;
        
        // Check capability
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "wallet_init",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        let wallet_id = Uuid::new_v4();
        let did = self.generate_did(&wallet_id).await?;
        let pdv_location = format!("{}/{}", self.keystore.read().await.pdv_path(), wallet_id);

        // Create PDV directory structure
        self.keystore.write().await.init_wallet(&wallet_id).await?;

        let result = WalletInitResult {
            wallet_id,
            did,
            pdv_location,
            created_at: Utc::now(),
        };

        self.audit_operation(
            subject,
            "wallet_init",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("wallet_id".to_string(), result.wallet_id.to_string()),
                ("did".to_string(), result.did.clone()),
            ]),
        ).await;

        Ok(result)
    }

    /// Generate a new key with specified algorithm
    pub async fn generate_key(
        &self,
        subject: &str,
        wallet_id: &Uuid,
        key_type: KeyType,
        intent_id: Option<&str>,
    ) -> Result<KeyGenResult, WalletError> {
        let capability = WalletCapability::KeyGenerate;
        
        // Check capability and rate limits
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "key_generate",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        // Check rate limits
        if !self.capability_manager.check_rate_limit(subject, "keygen").await? {
            return Err(WalletError::RateLimitExceeded);
        }

        let key_id = Uuid::new_v4();
        let key_material = self.keystore.write().await.generate_key(
            wallet_id,
            &key_id,
            key_type,
        ).await?;

        let did_binding = self.create_did_binding(&key_material).await?;
        let pdv_item_id = self.keystore.read().await.store_key(wallet_id, &key_id, &key_material).await?;

        let result = KeyGenResult {
            key_id,
            key_type,
            public_key: key_material.public_key().to_vec(),
            did_binding,
            pdv_item_id,
            derivation_path: key_material.derivation_path().clone(),
        };

        self.audit_operation(
            subject,
            "key_generate",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("key_id".to_string(), result.key_id.to_string()),
                ("key_type".to_string(), format!("{:?}", result.key_type)),
            ]),
        ).await;

        Ok(result)
    }

    /// Derive a key from an existing key using hierarchical derivation
    pub async fn derive_key(
        &self,
        subject: &str,
        wallet_id: &Uuid,
        parent_key_id: &Uuid,
        derivation_path: KeyDerivationPath,
        intent_id: Option<&str>,
    ) -> Result<KeyDeriveResult, WalletError> {
        let capability = WalletCapability::KeyDerive;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "key_derive",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        let parent_key = self.keystore.read().await.get_key(wallet_id, parent_key_id).await?;
        let derived_key = self.keystore.write().await.derive_key(
            &parent_key,
            &derivation_path,
        ).await?;

        let key_id = Uuid::new_v4();
        let address = self.compute_address(&derived_key).await?;

        // Optionally persist the derived key
        let persisted = if derivation_path.should_persist() {
            self.keystore.write().await.store_key(wallet_id, &key_id, &derived_key).await?;
            true
        } else {
            false
        };

        let result = KeyDeriveResult {
            key_id,
            derived_key: derived_key.public_key().to_vec(),
            address,
            derivation_path,
            persisted,
        };

        self.audit_operation(
            subject,
            "key_derive",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("parent_key_id".to_string(), parent_key_id.to_string()),
                ("derivation_path".to_string(), format!("{:?}", result.derivation_path)),
            ]),
        ).await;

        Ok(result)
    }

    /// Sign data with a key
    pub async fn sign(
        &self,
        subject: &str,
        wallet_id: &Uuid,
        key_id: &Uuid,
        data: &[u8],
        intent_id: Option<&str>,
        hybrid_pqc: bool,
    ) -> Result<SignResult, WalletError> {
        let capability = WalletCapability::KeySign;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "key_sign",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        let key_material = self.keystore.read().await.get_key(wallet_id, key_id).await?;
        let signature = self.keystore.read().await.sign(&key_material, data).await?;

        let mut result = SignResult {
            signature: signature.signature,
            recovery_id: signature.recovery_id,
            pqc_signature: None,
            algorithm: format!("{:?}", key_material.key_type()),
        };

        // Add PQC signature if hybrid mode is enabled
        if hybrid_pqc && self.is_pqc_eligible(&key_material) {
            let pqc_sig = self.keystore.read().await.sign_pqc(&key_material, data).await?;
            result.pqc_signature = Some(pqc_sig);
            result.algorithm = format!("{:?}+PQC", key_material.key_type());
        }

        self.audit_operation(
            subject,
            "key_sign",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("key_id".to_string(), key_id.to_string()),
                ("data_hash".to_string(), hex::encode(blake3::hash(data).as_bytes())),
                ("hybrid_pqc".to_string(), hybrid_pqc.to_string()),
            ]),
        ).await;

        Ok(result)
    }

    /// Export public key (only public material allowed)
    pub async fn export_public_key(
        &self,
        subject: &str,
        wallet_id: &Uuid,
        key_id: &Uuid,
        intent_id: Option<&str>,
    ) -> Result<Vec<u8>, WalletError> {
        let capability = WalletCapability::KeyExport;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "key_export",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        let key_material = self.keystore.read().await.get_key(wallet_id, key_id).await?;
        let public_key = key_material.public_key().to_vec();

        self.audit_operation(
            subject,
            "key_export",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("key_id".to_string(), key_id.to_string()),
                ("public_key_hash".to_string(), hex::encode(blake3::hash(&public_key).as_bytes())),
            ]),
        ).await;

        Ok(public_key)
    }

    /// Import an encrypted key
    pub async fn import_key(
        &self,
        subject: &str,
        wallet_id: &Uuid,
        encrypted_key: &[u8],
        intent_id: Option<&str>,
    ) -> Result<KeyGenResult, WalletError> {
        let capability = WalletCapability::KeyImport;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "key_import",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        let key_material = self.keystore.write().await.import_encrypted_key(encrypted_key).await?;
        let key_id = Uuid::new_v4();
        let pdv_item_id = self.keystore.write().await.store_key(wallet_id, &key_id, &key_material).await?;

        let result = KeyGenResult {
            key_id,
            key_type: key_material.key_type(),
            public_key: key_material.public_key().to_vec(),
            did_binding: None,
            pdv_item_id,
            derivation_path: key_material.derivation_path().clone(),
        };

        self.audit_operation(
            subject,
            "key_import",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("key_id".to_string(), result.key_id.to_string()),
                ("key_type".to_string(), format!("{:?}", result.key_type)),
            ]),
        ).await;

        Ok(result)
    }

    /// Revoke a key
    pub async fn revoke_key(
        &self,
        subject: &str,
        wallet_id: &Uuid,
        key_id: &Uuid,
        intent_id: Option<&str>,
    ) -> Result<(), WalletError> {
        let capability = WalletCapability::KeyRevoke;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "key_revoke",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        self.keystore.write().await.revoke_key(wallet_id, key_id).await?;

        self.audit_operation(
            subject,
            "key_revoke",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("key_id".to_string(), key_id.to_string()),
            ]),
        ).await;

        Ok(())
    }

    /// Create a new DID
    pub async fn create_did(
        &self,
        subject: &str,
        method: DIDMethod,
        key_id: Option<&Uuid>,
        intent_id: Option<&str>,
    ) -> Result<DIDDocument, WalletError> {
        let capability = WalletCapability::DIDCreate;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "did_create",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        let did_doc = self.did_resolver.write().await.create_did(method, key_id).await?;

        self.audit_operation(
            subject,
            "did_create",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("did".to_string(), did_doc.id.clone()),
                ("method".to_string(), format!("{:?}", method)),
            ]),
        ).await;

        Ok(did_doc)
    }

    /// Resolve a DID to its document
    pub async fn resolve_did(
        &self,
        subject: &str,
        did: &str,
        intent_id: Option<&str>,
    ) -> Result<DIDDocument, WalletError> {
        let capability = WalletCapability::DIDResolve;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "did_resolve",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        let did_doc = self.did_resolver.read().await.resolve(did).await?;

        self.audit_operation(
            subject,
            "did_resolve",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("did".to_string(), did.to_string()),
            ]),
        ).await;

        Ok(did_doc)
    }

    /// List vault items
    pub async fn list_vault_items(
        &self,
        subject: &str,
        wallet_id: &Uuid,
        intent_id: Option<&str>,
    ) -> Result<Vec<String>, WalletError> {
        let capability = WalletCapability::VaultList;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            self.audit_operation(
                subject,
                "vault_list",
                intent_id,
                &capability,
                AuditResult::Denied { reason: "Insufficient capabilities".to_string() },
                HashMap::new(),
            ).await;
            return Err(WalletError::CapabilityDenied);
        }

        let items = self.keystore.read().await.list_keys(wallet_id).await?;

        self.audit_operation(
            subject,
            "vault_list",
            intent_id,
            &capability,
            AuditResult::Success,
            HashMap::from([
                ("item_count".to_string(), items.len().to_string()),
            ]),
        ).await;

        Ok(items)
    }

    /// Get audit log entries
    pub async fn get_audit_log(
        &self,
        subject: &str,
        limit: Option<usize>,
    ) -> Result<Vec<AuditEntry>, WalletError> {
        let capability = WalletCapability::AuditRead;
        
        if !self.capability_manager.check_capability(subject, &capability).await? {
            return Err(WalletError::CapabilityDenied);
        }

        let log = self.audit_log.read().await;
        let entries = if let Some(limit) = limit {
            log.iter().rev().take(limit).cloned().collect()
        } else {
            log.clone()
        };

        Ok(entries)
    }

    // Private helper methods

    async fn generate_did(&self, wallet_id: &Uuid) -> Result<String, WalletError> {
        // Generate a did:key for the wallet
        let did = format!("did:key:{}", hex::encode(wallet_id.as_bytes()));
        Ok(did)
    }

    async fn create_did_binding(&self, key_material: &KeyMaterial) -> Result<Option<String>, WalletError> {
        // Create a did:key binding for the key
        let key_id = hex::encode(blake3::hash(key_material.public_key()).as_bytes());
        let did = format!("did:key:{}", key_id);
        Ok(Some(did))
    }

    async fn compute_address(&self, key_material: &KeyMaterial) -> Result<Option<String>, WalletError> {
        match key_material.key_type() {
            KeyType::Secp256k1 => {
                // Compute Ethereum address
                let pubkey = key_material.public_key();
                let address = self.keystore.read().await.compute_ethereum_address(pubkey).await?;
                Ok(Some(format!("0x{}", hex::encode(address))))
            }
            KeyType::Ed25519 => {
                // Compute Solana address
                let pubkey = key_material.public_key();
                let address = self.keystore.read().await.compute_solana_address(pubkey).await?;
                Ok(Some(bs58::encode(address).into_string()))
            }
            _ => Ok(None),
        }
    }

    fn is_pqc_eligible(&self, key_material: &KeyMaterial) -> bool {
        matches!(key_material.key_type(), KeyType::Secp256k1 | KeyType::Ed25519)
    }

    async fn audit_operation(
        &self,
        subject: &str,
        operation: &str,
        intent_id: Option<&str>,
        capability: &WalletCapability,
        result: AuditResult,
        metadata: HashMap<String, String>,
    ) {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            operation: operation.to_string(),
            subject: subject.to_string(),
            intent_id: intent_id.map(|s| s.to_string()),
            capability: format!("{:?}", capability),
            result,
            metadata,
        };

        self.audit_log.write().await.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet_service_creation() {
        let config = WalletConfig::default();
        let service = WalletService::new(config).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_wallet_initialization() {
        let config = WalletConfig::default();
        let service = WalletService::new(config).await.unwrap();
        
        let result = service.init_wallet("test_user", Some("test_intent")).await;
        assert!(result.is_ok());
        
        let wallet = result.unwrap();
        assert!(wallet.did.starts_with("did:key:"));
        assert!(!wallet.pdv_location.is_empty());
    }
}
