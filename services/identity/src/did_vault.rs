use crate::did::{Did, DidDocument, DidVerificationMethod};
use crate::vault::{VaultEntryV1, EntryKind, CapTokenV2, VaultError, VaultErrorCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// DID-bound vault entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidVaultEntry {
    /// The vault entry itself
    pub entry: VaultEntryV1,
    /// DID document that owns this entry
    pub owner_did: Did,
    /// Verification method used for access control
    pub verification_method: String,
    /// Additional DID-specific metadata
    pub did_metadata: HashMap<String, serde_cbor::Value>,
}

/// DID vault service for managing DID-bound vault entries
pub struct DidVaultService {
    /// In-memory storage of DID-bound entries (in production, this would be persistent)
    entries: Arc<RwLock<HashMap<String, DidVaultEntry>>>,
    /// DID document registry
    did_registry: Arc<RwLock<HashMap<Did, DidDocument>>>,
}

impl DidVaultService {
    /// Create a new DID vault service
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            did_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new DID-bound vault entry
    pub async fn create_did_entry(
        &self,
        id: &str,
        kind: EntryKind,
        plaintext: Vec<u8>,
        meta: HashMap<String, serde_cbor::Value>,
        cap_token: &CapTokenV2,
        owner_did: &Did,
        verification_method: &str,
    ) -> Result<(), VaultError> {
        // Verify the CapToken allows this operation
        if !self.verify_capability(cap_token, "write", &format!("vault:{}", id)) {
            return Err(VaultError::new(
                VaultErrorCode::InsufficientCapabilities,
                "CapToken does not grant write access to this vault entry",
            ));
        }

        // Verify the DID exists and is valid
        let did_doc = self.get_did_document(owner_did).await?;
        
        // Verify the verification method exists
        if !did_doc.verification_methods.iter().any(|vm| vm.id == verification_method) {
            return Err(VaultError::new(
                VaultErrorCode::InvalidDid,
                "Verification method not found in DID document",
            ));
        }

        // Create the vault entry (this would integrate with the actual vault engine)
        let entry = VaultEntryV1 {
            id: id.to_string(),
            kind,
            subject_did: owner_did.clone(),
            enc_env: Default::default(), // Placeholder - would use actual encryption
            meta,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            capabilities: vec![], // Would be populated from CapToken
            tags: vec![],
            expires_at: None,
            revoked: false,
        };

        // Create the DID-bound entry
        let did_entry = DidVaultEntry {
            entry,
            owner_did: owner_did.clone(),
            verification_method: verification_method.to_string(),
            did_metadata: HashMap::new(),
        };

        // Store the entry
        let mut entries = self.entries.write().await;
        entries.insert(id.to_string(), did_entry);

        Ok(())
    }

    /// Retrieve a DID-bound vault entry
    pub async fn get_did_entry(
        &self,
        id: &str,
        cap_token: &CapTokenV2,
    ) -> Result<DidVaultEntry, VaultError> {
        // Verify the CapToken allows this operation
        if !self.verify_capability(cap_token, "read", &format!("vault:{}", id)) {
            return Err(VaultError::new(
                VaultErrorCode::InsufficientCapabilities,
                "CapToken does not grant read access to this vault entry",
            ));
        }

        // Retrieve the entry
        let entries = self.entries.read().await;
        entries.get(id).cloned().ok_or_else(|| {
            VaultError::new(
                VaultErrorCode::EntryNotFound,
                &format!("Vault entry '{}' not found", id),
            )
        })
    }

    /// List all entries for a specific DID
    pub async fn list_did_entries(
        &self,
        did: &Did,
        cap_token: &CapTokenV2,
    ) -> Result<Vec<DidVaultEntry>, VaultError> {
        // Verify the CapToken allows listing
        if !self.verify_capability(cap_token, "list", "vault:*") {
            return Err(VaultError::new(
                VaultErrorCode::InsufficientCapabilities,
                "CapToken does not grant list access to vault entries",
            ));
        }

        // Filter entries by DID
        let entries = self.entries.read().await;
        let did_entries: Vec<DidVaultEntry> = entries
            .values()
            .filter(|entry| entry.owner_did == *did)
            .cloned()
            .collect();

        Ok(did_entries)
    }

    /// Delete a DID-bound vault entry (logical deletion)
    pub async fn delete_did_entry(
        &self,
        id: &str,
        cap_token: &CapTokenV2,
    ) -> Result<(), VaultError> {
        // Verify the CapToken allows this operation
        if !self.verify_capability(cap_token, "delete", &format!("vault:{}", id)) {
            return Err(VaultError::new(
                VaultErrorCode::InsufficientCapabilities,
                "CapToken does not grant delete access to this vault entry",
            ));
        }

        // Mark the entry as revoked
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(id) {
            entry.entry.revoked = true;
            entry.entry.updated_at = chrono::Utc::now();
        } else {
            return Err(VaultError::new(
                VaultErrorCode::EntryNotFound,
                &format!("Vault entry '{}' not found", id),
            ));
        }

        Ok(())
    }

    /// Issue a verifiable credential and store it in the vault
    pub async fn issue_credential(
        &self,
        subject_did: &Did,
        issuer_did: &Did,
        credential_type: &str,
        claims: HashMap<String, serde_cbor::Value>,
        cap_token: &CapTokenV2,
    ) -> Result<String, VaultError> {
        // Verify the CapToken allows credential issuance
        if !self.verify_capability(cap_token, "write", "credential:issue") {
            return Err(VaultError::new(
                VaultErrorCode::InsufficientCapabilities,
                "CapToken does not grant credential issuance capability",
            ));
        }

        // Verify both DIDs exist
        let _subject_doc = self.get_did_document(subject_did).await?;
        let _issuer_doc = self.get_did_document(issuer_did).await?;

        // Generate a unique credential ID
        let credential_id = format!("cred_{}_{}_{}", 
            issuer_did.to_string().replace(":", "_"),
            credential_type,
            chrono::Utc::now().timestamp()
        );

        // Create credential metadata
        let mut meta = HashMap::new();
        meta.insert("credential_type".to_string(), serde_cbor::Value::Text(credential_type.to_string()));
        meta.insert("issuer_did".to_string(), serde_cbor::Value::Text(issuer_did.to_string()));
        meta.insert("issued_at".to_string(), serde_cbor::Value::Text(chrono::Utc::now().to_rfc3339()));
        
        // Add claims
        for (key, value) in claims {
            meta.insert(format!("claim_{}", key), value);
        }

        // Create the vault entry for the credential
        self.create_did_entry(
            &credential_id,
            EntryKind::Credential,
            vec![], // Credential data would be encrypted here
            meta,
            cap_token,
            subject_did,
            "default", // Default verification method
        ).await?;

        Ok(credential_id)
    }

    /// Verify a verifiable credential
    pub async fn verify_credential(
        &self,
        credential_id: &str,
        cap_token: &CapTokenV2,
    ) -> Result<bool, VaultError> {
        // Retrieve the credential entry
        let entry = self.get_did_entry(credential_id, cap_token).await?;
        
        // Verify it's a credential
        if entry.entry.kind != EntryKind::Credential {
            return Err(VaultError::new(
                VaultErrorCode::InvalidEntryType,
                "Entry is not a credential",
            ));
        }

        // Verify the credential is not revoked
        if entry.entry.revoked {
            return Ok(false);
        }

        // Verify the credential has not expired
        if let Some(expires_at) = entry.entry.expires_at {
            if chrono::Utc::now() > expires_at {
                return Ok(false);
            }
        }

        // Additional verification logic would go here:
        // - Verify cryptographic signatures
        // - Check revocation lists
        // - Validate credential schema
        // - Verify issuer authority

        Ok(true)
    }

    /// Search for vault entries by various criteria
    pub async fn search_entries(
        &self,
        query: &VaultSearchQuery,
        cap_token: &CapTokenV2,
    ) -> Result<Vec<DidVaultEntry>, VaultError> {
        // Verify the CapToken allows search
        if !self.verify_capability(cap_token, "search", "vault:*") {
            return Err(VaultError::new(
                VaultErrorCode::InsufficientCapabilities,
                "CapToken does not grant search access to vault entries",
            ));
        }

        let entries = self.entries.read().await;
        let mut results = Vec::new();

        for entry in entries.values() {
            if self.matches_search_query(entry, query) {
                results.push(entry.clone());
            }
        }

        // Apply sorting
        if let Some(sort_field) = &query.sort_by {
            match sort_field.as_str() {
                "created_at" => {
                    results.sort_by(|a, b| a.entry.created_at.cmp(&b.entry.created_at));
                }
                "updated_at" => {
                    results.sort_by(|a, b| a.entry.updated_at.cmp(&b.entry.updated_at));
                }
                "id" => {
                    results.sort_by(|a, b| a.entry.id.cmp(&b.entry.id));
                }
                "kind" => {
                    results.sort_by(|a, b| format!("{:?}", a.entry.kind).cmp(&format!("{:?}", b.entry.kind)));
                }
                _ => {}
            }
        }

        // Apply sort order
        if query.sort_order == Some("desc".to_string()) {
            results.reverse();
        }

        // Apply pagination
        if let Some(limit) = query.limit {
            results.truncate(limit as usize);
        }

        Ok(results)
    }

    /// Get vault statistics for a DID
    pub async fn get_vault_stats(
        &self,
        did: &Did,
        cap_token: &CapTokenV2,
    ) -> Result<VaultStats, VaultError> {
        // Verify the CapToken allows access to stats
        if !self.verify_capability(cap_token, "read", "vault:stats") {
            return Err(VaultError::new(
                VaultErrorCode::InsufficientCapabilities,
                "CapToken does not grant access to vault statistics",
            ));
        }

        let entries = self.entries.read().await;
        let did_entries: Vec<&DidVaultEntry> = entries
            .values()
            .filter(|entry| entry.owner_did == *did)
            .collect();

        let mut stats = VaultStats {
            total_entries: did_entries.len() as u32,
            entries_by_kind: HashMap::new(),
            total_size_bytes: 0,
            revoked_entries: 0,
            expired_entries: 0,
        };

        for entry in did_entries {
            // Count by kind
            let kind_str = format!("{:?}", entry.entry.kind);
            *stats.entries_by_kind.entry(kind_str).or_insert(0) += 1;

            // Count revoked entries
            if entry.entry.revoked {
                stats.revoked_entries += 1;
            }

            // Count expired entries
            if let Some(expires_at) = entry.entry.expires_at {
                if chrono::Utc::now() > expires_at {
                    stats.expired_entries += 1;
                }
            }

            // Estimate size (in production, this would be actual encrypted size)
            stats.total_size_bytes += 1024; // Placeholder
        }

        Ok(stats)
    }

    // Private helper methods

    /// Verify that a CapToken grants a specific capability
    fn verify_capability(&self, cap_token: &CapTokenV2, operation: &str, resource: &str) -> bool {
        // This is a simplified capability check
        // In production, this would verify cryptographic signatures and check against the DID document
        
        // Check if the token has expired
        if let Some(expires_at) = cap_token.expires_at {
            if chrono::Utc::now() > expires_at {
                return false;
            }
        }

        // Check if the operation is allowed
        cap_token.capabilities.iter().any(|cap| {
            cap.operation == operation && 
            (cap.resource == resource || cap.resource == "*" || resource.starts_with(&cap.resource[..cap.resource.len()-1]))
        })
    }

    /// Get a DID document from the registry
    async fn get_did_document(&self, did: &Did) -> Result<DidDocument, VaultError> {
        let registry = self.did_registry.read().await;
        registry.get(did).cloned().ok_or_else(|| {
            VaultError::new(
                VaultErrorCode::DidNotFound,
                &format!("DID '{}' not found in registry", did),
            )
        })
    }

    /// Check if an entry matches a search query
    fn matches_search_query(&self, entry: &DidVaultEntry, query: &VaultSearchQuery) -> bool {
        // Check filters
        if let Some(filters) = &query.filters {
            for filter in filters {
                match filter {
                    SearchFilter::Kind(kind) => {
                        if entry.entry.kind != *kind {
                            return false;
                        }
                    }
                    SearchFilter::Tag(tag) => {
                        if !entry.entry.tags.contains(tag) {
                            return false;
                        }
                    }
                    SearchFilter::DateRange { start, end } => {
                        if let Some(start_date) = start {
                            if entry.entry.created_at < *start_date {
                                return false;
                            }
                        }
                        if let Some(end_date) = end {
                            if entry.entry.created_at > *end_date {
                                return false;
                            }
                        }
                    }
                    SearchFilter::SizeRange { min, max } => {
                        let size = entry.entry.enc_env.len() as u64;
                        if let Some(min_size) = min {
                            if size < *min_size {
                                return false;
                            }
                        }
                        if let Some(max_size) = max {
                            if size > *max_size {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        // Check search query
        if let Some(search) = &query.search {
            let search_lower = search.to_lowercase();
            if !entry.entry.id.to_lowercase().contains(&search_lower) &&
               !entry.entry.tags.iter().any(|tag| tag.to_lowercase().contains(&search_lower)) {
                return false;
            }
        }

        true
    }
}

/// Search query for vault entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSearchQuery {
    /// Text search query
    pub search: Option<String>,
    /// Filters to apply
    pub filters: Option<Vec<SearchFilter>>,
    /// Field to sort by
    pub sort_by: Option<String>,
    /// Sort order (asc/desc)
    pub sort_order: Option<String>,
    /// Maximum number of results
    pub limit: Option<u32>,
}

/// Search filter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchFilter {
    /// Filter by entry kind
    Kind(EntryKind),
    /// Filter by tag
    Tag(String),
    /// Filter by date range
    DateRange {
        start: Option<chrono::DateTime<chrono::Utc>>,
        end: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// Filter by size range
    SizeRange {
        min: Option<u64>,
        max: Option<u64>,
    },
}

/// Vault statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStats {
    /// Total number of entries
    pub total_entries: u32,
    /// Number of entries by kind
    pub entries_by_kind: HashMap<String, u32>,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Number of revoked entries
    pub revoked_entries: u32,
    /// Number of expired entries
    pub expired_entries: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::{CapTokenV2, Capability, Operation};

    fn create_test_cap_token(operation: &str, resource: &str) -> CapTokenV2 {
        CapTokenV2 {
            version: "2.0".to_string(),
            issuer_did: "did:aetheris:test:issuer".to_string(),
            subject_did: "did:aetheris:test:subject".to_string(),
            capabilities: vec![Capability {
                operation: operation.to_string(),
                resource: resource.to_string(),
                conditions: None,
                expires_at: None,
            }],
            issued_at: chrono::Utc::now(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            signature: "test_signature".to_string(),
        }
    }

    #[tokio::test]
    async fn test_create_did_entry() {
        let service = DidVaultService::new();
        let cap_token = create_test_cap_token("write", "vault:test-entry");
        let did = Did::new("did:aetheris:test:user").unwrap();
        
        let result = service.create_did_entry(
            "test-entry",
            EntryKind::Key,
            vec![1, 2, 3],
            HashMap::new(),
            &cap_token,
            &did,
            "default",
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_insufficient_capabilities() {
        let service = DidVaultService::new();
        let cap_token = create_test_cap_token("read", "vault:test-entry");
        let did = Did::new("did:aetheris:test:user").unwrap();
        
        let result = service.create_did_entry(
            "test-entry",
            EntryKind::Key,
            vec![1, 2, 3],
            HashMap::new(),
            &cap_token,
            &did,
            "default",
        ).await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, VaultErrorCode::InsufficientCapabilities);
    }

    #[tokio::test]
    async fn test_issue_credential() {
        let service = DidVaultService::new();
        let cap_token = create_test_cap_token("write", "credential:issue");
        let subject_did = Did::new("did:aetheris:test:subject").unwrap();
        let issuer_did = Did::new("did:aetheris:test:issuer").unwrap();
        
        let mut claims = HashMap::new();
        claims.insert("name".to_string(), serde_cbor::Value::Text("John Doe".to_string()));
        
        let result = service.issue_credential(
            &subject_did,
            &issuer_did,
            "Person",
            claims,
            &cap_token,
        ).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("cred_"));
    }
}
