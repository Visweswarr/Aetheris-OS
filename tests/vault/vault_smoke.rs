use ngfs::vault::{VaultEngine, EntryKind, CapTokenV2, Capability, VaultError, VaultErrorCode};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Mock CAS index for testing
struct MockCasIndex;

impl ngfs::cas::CasIndex for MockCasIndex {
    fn store(&self, _data: &[u8]) -> Result<String, ngfs::cas::CasError> {
        Ok("mock_cid_123".to_string())
    }
    
    fn retrieve(&self, _cid: &str) -> Result<Vec<u8>, ngfs::cas::CasError> {
        Ok(b"mock_encrypted_data".to_vec())
    }
    
    fn exists(&self, _cid: &str) -> bool {
        true
    }
}

#[tokio::test]
async fn test_vault_engine_creation() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    assert_eq!(engine.vault_id, "test-vault");
    assert_eq!(engine.owner_did, "did:aetheris:test:user");
}

#[tokio::test]
async fn test_create_entry_success() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![Capability {
            operation: "write".to_string(),
            resource: "vault:test-entry".to_string(),
            conditions: None,
            expires_at: None,
        }],
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        signature: "test_signature".to_string(),
    };
    
    let mut meta = HashMap::new();
    meta.insert("description".to_string(), serde_cbor::Value::Text("Test entry".to_string()));
    
    let result = engine.create_entry(
        "test-entry",
        EntryKind::Key,
        b"secret_key_data".to_vec(),
        meta,
        &cap_token,
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_entry_insufficient_capabilities() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![Capability {
            operation: "read".to_string(),
            resource: "vault:test-entry".to_string(),
            conditions: None,
            expires_at: None,
        }],
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        signature: "test_signature".to_string(),
    };
    
    let mut meta = HashMap::new();
    meta.insert("description".to_string(), serde_cbor::Value::Text("Test entry".to_string()));
    
    let result = engine.create_entry(
        "test-entry",
        EntryKind::Key,
        b"secret_key_data".to_vec(),
        meta,
        &cap_token,
    ).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, VaultErrorCode::InsufficientCapabilities);
}

#[tokio::test]
async fn test_get_entry_success() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    // First create an entry
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![
            Capability {
                operation: "write".to_string(),
                resource: "vault:test-entry".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "read".to_string(),
                resource: "vault:test-entry".to_string(),
                conditions: None,
                expires_at: None,
            },
        ],
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        signature: "test_signature".to_string(),
    };
    
    let mut meta = HashMap::new();
    meta.insert("description".to_string(), serde_cbor::Value::Text("Test entry".to_string()));
    
    engine.create_entry(
        "test-entry",
        EntryKind::Key,
        b"secret_key_data".to_vec(),
        meta,
        &cap_token,
    ).await.unwrap();
    
    // Now retrieve it
    let result = engine.get_entry("test-entry", &cap_token).await;
    assert!(result.is_ok());
    
    let content = result.unwrap();
    assert_eq!(content, b"secret_key_data");
}

#[tokio::test]
async fn test_get_entry_not_found() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![Capability {
            operation: "read".to_string(),
            resource: "vault:test-entry".to_string(),
            conditions: None,
            expires_at: None,
        }],
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        signature: "test_signature".to_string(),
    };
    
    let result = engine.get_entry("non-existent-entry", &cap_token).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, VaultErrorCode::EntryNotFound);
}

#[tokio::test]
async fn test_list_entries() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![
            Capability {
                operation: "write".to_string(),
                resource: "vault:test-entry-1".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "write".to_string(),
                resource: "vault:test-entry-2".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "list".to_string(),
                resource: "vault:*".to_string(),
                conditions: None,
                expires_at: None,
            },
        ],
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        signature: "test_signature".to_string(),
    };
    
    // Create two entries
    let mut meta1 = HashMap::new();
    meta1.insert("description".to_string(), serde_cbor::Value::Text("First entry".to_string()));
    
    let mut meta2 = HashMap::new();
    meta2.insert("description".to_string(), serde_cbor::Value::Text("Second entry".to_string()));
    
    engine.create_entry(
        "test-entry-1",
        EntryKind::Key,
        b"first_data".to_vec(),
        meta1,
        &cap_token,
    ).await.unwrap();
    
    engine.create_entry(
        "test-entry-2",
        EntryKind::Document,
        b"second_data".to_vec(),
        meta2,
        &cap_token,
    ).await.unwrap();
    
    // List entries
    let result = engine.list_entries(&cap_token).await;
    assert!(result.is_ok());
    
    let entries = result.unwrap();
    assert_eq!(entries.len(), 2);
    
    // Verify entry metadata
    let entry1 = entries.iter().find(|e| e.id == "test-entry-1").unwrap();
    assert_eq!(entry1.kind, EntryKind::Key);
    
    let entry2 = entries.iter().find(|e| e.id == "test-entry-2").unwrap();
    assert_eq!(entry2.kind, EntryKind::Document);
}

#[tokio::test]
async fn test_delete_entry() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![
            Capability {
                operation: "write".to_string(),
                resource: "vault:test-entry".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "delete".to_string(),
                resource: "vault:test-entry".to_string(),
                conditions: None,
                expires_at: None,
            },
        ],
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        signature: "test_signature".to_string(),
    };
    
    let mut meta = HashMap::new();
    meta.insert("description".to_string(), serde_cbor::Value::Text("Test entry".to_string()));
    
    // Create entry
    engine.create_entry(
        "test-entry",
        EntryKind::Key,
        b"secret_key_data".to_vec(),
        meta,
        &cap_token,
    ).await.unwrap();
    
    // Delete entry
    let result = engine.delete_entry("test-entry", &cap_token).await;
    assert!(result.is_ok());
    
    // Verify entry is marked as revoked
    let entries = engine.list_entries(&cap_token).await.unwrap();
    let deleted_entry = entries.iter().find(|e| e.id == "test-entry").unwrap();
    assert!(deleted_entry.revoked);
}

#[tokio::test]
async fn test_expired_cap_token() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![Capability {
            operation: "write".to_string(),
            resource: "vault:test-entry".to_string(),
            conditions: None,
            expires_at: None,
        }],
        issued_at: chrono::Utc::now() - chrono::Duration::hours(2),
        expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)), // Expired
        signature: "test_signature".to_string(),
    };
    
    let mut meta = HashMap::new();
    meta.insert("description".to_string(), serde_cbor::Value::Text("Test entry".to_string()));
    
    let result = engine.create_entry(
        "test-entry",
        EntryKind::Key,
        b"secret_key_data".to_vec(),
        meta,
        &cap_token,
    ).await;
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, VaultErrorCode::InsufficientCapabilities);
}

#[tokio::test]
async fn test_entry_kinds() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![
            Capability {
                operation: "write".to_string(),
                resource: "vault:test-key".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "write".to_string(),
                resource: "vault:test-doc".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "write".to_string(),
                resource: "vault:test-cred".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "list".to_string(),
                resource: "vault:*".to_string(),
                conditions: None,
                expires_at: None,
            },
        ],
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        signature: "test_signature".to_string(),
    };
    
    let mut meta = HashMap::new();
    meta.insert("description".to_string(), serde_cbor::Value::Text("Test entry".to_string()));
    
    // Test different entry kinds
    engine.create_entry(
        "test-key",
        EntryKind::Key,
        b"key_data".to_vec(),
        meta.clone(),
        &cap_token,
    ).await.unwrap();
    
    engine.create_entry(
        "test-doc",
        EntryKind::Document,
        b"document_data".to_vec(),
        meta.clone(),
        &cap_token,
    ).await.unwrap();
    
    engine.create_entry(
        "test-cred",
        EntryKind::Credential,
        b"credential_data".to_vec(),
        meta,
        &cap_token,
    ).await.unwrap();
    
    // List and verify kinds
    let entries = engine.list_entries(&cap_token).await.unwrap();
    assert_eq!(entries.len(), 3);
    
    let key_entry = entries.iter().find(|e| e.id == "test-key").unwrap();
    assert_eq!(key_entry.kind, EntryKind::Key);
    
    let doc_entry = entries.iter().find(|e| e.id == "test-doc").unwrap();
    assert_eq!(doc_entry.kind, EntryKind::Document);
    
    let cred_entry = entries.iter().find(|e| e.id == "test-cred").unwrap();
    assert_eq!(cred_entry.kind, EntryKind::Credential);
}

#[tokio::test]
async fn test_metadata_persistence() {
    let cas = Arc::new(MockCasIndex);
    let engine = VaultEngine::new(cas, "test-vault".to_string(), "did:aetheris:test:user".to_string());
    
    let cap_token = CapTokenV2 {
        version: "2.0".to_string(),
        issuer_did: "did:aetheris:test:issuer".to_string(),
        subject_did: "did:aetheris:test:user".to_string(),
        capabilities: vec![
            Capability {
                operation: "write".to_string(),
                resource: "vault:test-entry".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "read".to_string(),
                resource: "vault:test-entry".to_string(),
                conditions: None,
                expires_at: None,
            },
            Capability {
                operation: "list".to_string(),
                resource: "vault:*".to_string(),
                conditions: None,
                expires_at: None,
            },
        ],
        issued_at: chrono::Utc::now(),
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        signature: "test_signature".to_string(),
    };
    
    let mut meta = HashMap::new();
    meta.insert("description".to_string(), serde_cbor::Value::Text("Test entry with metadata".to_string()));
    meta.insert("algorithm".to_string(), serde_cbor::Value::Text("ed25519".to_string()));
    meta.insert("key_size".to_string(), serde_cbor::Value::Integer(256));
    meta.insert("tags".to_string(), serde_cbor::Value::Array(vec![
        serde_cbor::Value::Text("key".to_string()),
        serde_cbor::Value::Text("secret".to_string()),
        serde_cbor::Value::Text("ed25519".to_string()),
    ]));
    
    engine.create_entry(
        "test-entry",
        EntryKind::Key,
        b"secret_key_data".to_vec(),
        meta.clone(),
        &cap_token,
    ).await.unwrap();
    
    // Verify metadata is preserved
    let entries = engine.list_entries(&cap_token).await.unwrap();
    let entry = entries.iter().find(|e| e.id == "test-entry").unwrap();
    
    // Check that metadata contains expected values
    assert_eq!(entry.meta.get("description").unwrap(), &serde_cbor::Value::Text("Test entry with metadata".to_string()));
    assert_eq!(entry.meta.get("algorithm").unwrap(), &serde_cbor::Value::Text("ed25519".to_string()));
    assert_eq!(entry.meta.get("key_size").unwrap(), &serde_cbor::Value::Integer(256));
    
    // Check tags array
    if let serde_cbor::Value::Array(tags) = entry.meta.get("tags").unwrap() {
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&serde_cbor::Value::Text("key".to_string())));
        assert!(tags.contains(&serde_cbor::Value::Text("secret".to_string())));
        assert!(tags.contains(&serde_cbor::Value::Text("ed25519".to_string())));
    } else {
        panic!("Tags should be an array");
    }
}
