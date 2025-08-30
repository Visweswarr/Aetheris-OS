use super::*;
use tempfile::tempdir;

/// Test suite for DID identity functionality
#[cfg(test)]
mod did_identity_tests {
    use super::*;
    
    /// Test complete DID lifecycle: create → resolve → rotate key
    #[tokio::test]
    async fn test_did_lifecycle_create_resolve_rotate() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Step 1: Create DID
        let (did, original_doc) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519, KeyType::Dilithium3],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion],
        ).await.unwrap();
        
        assert!(did.starts_with("did:polynet:"));
        assert_eq!(original_doc.verification_methods.len(), 2);
        assert_eq!(original_doc.authentication.len(), 2);
        assert_eq!(original_doc.assertion_method.len(), 2);
        
        // Step 2: Resolve DID
        let resolved_doc = identity.resolve_did(&did).await.unwrap();
        assert!(resolved_doc.is_some());
        
        let resolved = resolved_doc.unwrap();
        assert_eq!(resolved.id, did);
        assert_eq!(resolved.version_id, original_doc.version_id);
        assert_eq!(resolved.verification_methods.len(), 2);
        
        // Step 3: Rotate keys
        let original_keys = identity.get_did_keys(&did).await.unwrap();
        assert_eq!(original_keys.len(), 2);
        
        let key_ids: Vec<String> = original_keys.iter().map(|k| k.id.clone()).collect();
        let original_version = original_doc.version_id.clone();
        
        let updated_doc = identity.rotate_keys(
            &did,
            key_ids.clone(),
            vec![KeyType::Kyber512, KeyType::Dilithium5],
            vec![KeyPurpose::Authentication, KeyPurpose::KeyAgreement],
        ).await.unwrap();
        
        // Verify key rotation
        assert_ne!(updated_doc.version_id, original_version);
        assert_eq!(updated_doc.verification_methods.len(), 2);
        assert_eq!(updated_doc.authentication.len(), 2);
        assert_eq!(updated_doc.key_agreement.len(), 2);
        
        // Verify old keys are revoked
        let all_keys = identity.get_did_keys(&did).await.unwrap();
        let old_keys: Vec<&DidKey> = all_keys.iter()
            .filter(|k| key_ids.contains(&k.id))
            .collect();
        
        assert!(old_keys.iter().all(|k| k.revoked));
        
        // Verify new keys are active
        let new_keys: Vec<&DidKey> = all_keys.iter()
            .filter(|k| !key_ids.contains(&k.id))
            .collect();
        
        assert_eq!(new_keys.len(), 2);
        assert!(new_keys.iter().all(|k| !k.revoked));
        
        // Verify new key types
        let key_types: Vec<KeyType> = new_keys.iter().map(|k| k.key_type).collect();
        assert!(key_types.contains(&KeyType::Kyber512));
        assert!(key_types.contains(&KeyType::Dilithium5));
        
        // Verify new purposes
        for key in new_keys {
            assert!(key.purpose.contains(&KeyPurpose::Authentication) || 
                   key.purpose.contains(&KeyPurpose::KeyAgreement));
        }
        
        // Step 4: Resolve updated DID
        let final_resolved = identity.resolve_did(&did).await.unwrap();
        assert!(final_resolved.is_some());
        
        let final_doc = final_resolved.unwrap();
        assert_eq!(final_doc.version_id, updated_doc.version_id);
        assert_eq!(final_doc.verification_methods.len(), 2);
        assert_eq!(final_doc.authentication.len(), 2);
        assert_eq!(final_doc.key_agreement.len(), 2);
    }
    
    /// Test DID creation with different key types
    #[tokio::test]
    async fn test_did_creation_different_key_types() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Test Ed25519
        let (did1, doc1) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        assert!(did1.starts_with("did:polynet:"));
        assert_eq!(doc1.verification_methods.len(), 1);
        assert_eq!(doc1.authentication.len(), 1);
        
        // Test Dilithium3
        let (did2, doc2) = identity.create_did(
            "polynet",
            vec![KeyType::Dilithium3],
            vec![KeyPurpose::Assertion],
        ).await.unwrap();
        
        assert!(did2.starts_with("did:polynet:"));
        assert_eq!(doc2.verification_methods.len(), 1);
        assert_eq!(doc2.assertion_method.len(), 1);
        
        // Test Kyber512
        let (did3, doc3) = identity.create_did(
            "polynet",
            vec![KeyType::Kyber512],
            vec![KeyPurpose::KeyAgreement],
        ).await.unwrap();
        
        assert!(did3.starts_with("did:polynet:"));
        assert_eq!(doc3.verification_methods.len(), 1);
        assert_eq!(doc3.key_agreement.len(), 1);
        
        // Test hybrid Ed25519 + Dilithium3
        let (did4, doc4) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519Dilithium3],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion],
        ).await.unwrap();
        
        assert!(did4.starts_with("did:polynet:"));
        assert_eq!(doc4.verification_methods.len(), 1);
        assert_eq!(doc4.authentication.len(), 1);
        assert_eq!(doc4.assertion_method.len(), 1);
        
        // Verify all DIDs are unique
        let dids = vec![did1, did2, did3, did4];
        let unique_dids: std::collections::HashSet<_> = dids.iter().collect();
        assert_eq!(unique_dids.len(), 4);
    }
    
    /// Test key rotation scenarios
    #[tokio::test]
    async fn test_key_rotation_scenarios() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Create DID with multiple keys
        let (did, _) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519, KeyType::Dilithium3, KeyType::Kyber512],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion, KeyPurpose::KeyAgreement],
        ).await.unwrap();
        
        let original_keys = identity.get_did_keys(&did).await.unwrap();
        assert_eq!(original_keys.len(), 3);
        
        // Scenario 1: Rotate single key
        let single_key_id = vec![original_keys[0].id.clone()];
        let updated_doc1 = identity.rotate_keys(
            &did,
            single_key_id,
            vec![KeyType::Dilithium5],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        assert_eq!(updated_doc1.verification_methods.len(), 3);
        assert_eq!(updated_doc1.authentication.len(), 3);
        
        // Scenario 2: Rotate multiple keys
        let multiple_key_ids = vec![
            original_keys[1].id.clone(),
            original_keys[2].id.clone(),
        ];
        let updated_doc2 = identity.rotate_keys(
            &did,
            multiple_key_ids,
            vec![KeyType::Kyber768, KeyType::Ed25519],
            vec![KeyPurpose::Assertion, KeyPurpose::KeyAgreement],
        ).await.unwrap();
        
        assert_eq!(updated_doc2.verification_methods.len(), 3);
        assert_eq!(updated_doc2.assertion_method.len(), 3);
        assert_eq!(updated_doc2.key_agreement.len(), 3);
        
        // Scenario 3: Rotate all keys
        let all_keys = identity.get_did_keys(&did).await.unwrap();
        let all_key_ids: Vec<String> = all_keys.iter().map(|k| k.id.clone()).collect();
        
        let final_doc = identity.rotate_keys(
            &did,
            all_key_ids,
            vec![KeyType::Ed25519Dilithium3, KeyType::Ed25519Kyber512],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion],
        ).await.unwrap();
        
        assert_eq!(final_doc.verification_methods.len(), 2);
        assert_eq!(final_doc.authentication.len(), 2);
        assert_eq!(final_doc.assertion_method.len(), 2);
    }
    
    /// Test DID resolution with different states
    #[tokio::test]
    async fn test_did_resolution_different_states() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Create DID
        let (did, _) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await.unwrap();
        
        // Resolve active DID
        let resolved = identity.resolve_did(&did).await.unwrap();
        assert!(resolved.is_some());
        
        let doc = resolved.unwrap();
        assert_eq!(doc.id, did);
        assert!(!doc.verification_methods.is_empty());
        
        // Deactivate DID
        identity.deactivate_did(&did, Some("Testing deactivation".to_string())).await.unwrap();
        
        // Resolve deactivated DID (should still return document)
        let deactivated_resolved = identity.resolve_did(&did).await.unwrap();
        assert!(deactivated_resolved.is_some());
        
        // Verify metadata shows deactivation
        let metadata = store.get_did_metadata(&did).await.unwrap();
        assert!(metadata.unwrap().deactivated);
        
        // Resolve non-existent DID
        let non_existent = identity.resolve_did("did:polynet:nonexistent").await.unwrap();
        assert!(non_existent.is_none());
    }
    
    /// Test key management operations
    #[tokio::test]
    async fn test_key_management_operations() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Create DID with multiple keys
        let (did, _) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519, KeyType::Dilithium3],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion],
        ).await.unwrap();
        
        // Get all keys
        let keys = identity.get_did_keys(&did).await.unwrap();
        assert_eq!(keys.len(), 2);
        
        // Verify key properties
        for key in &keys {
            assert!(!key.id.is_empty());
            assert!(!key.public_key.is_empty());
            assert!(key.private_key.is_some());
            assert!(!key.revoked);
            assert!(!key.purpose.is_empty());
        }
        
        // Get specific key
        let key_id = &keys[0].id;
        let specific_key = store.get_did_key(&did, key_id).await.unwrap();
        assert!(specific_key.is_some());
        assert_eq!(specific_key.unwrap().id, *key_id);
        
        // Revoke key
        store.revoke_did_key(&did, key_id).await.unwrap();
        
        let updated_keys = identity.get_did_keys(&did).await.unwrap();
        let revoked_key = updated_keys.iter().find(|k| k.id == *key_id).unwrap();
        assert!(revoked_key.revoked);
    }
    
    /// Test DID document structure and verification methods
    #[tokio::test]
    async fn test_did_document_structure() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Create DID with comprehensive configuration
        let (did, doc) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519, KeyType::Dilithium3, KeyType::Kyber512],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion, KeyPurpose::KeyAgreement],
        ).await.unwrap();
        
        // Verify document structure
        assert_eq!(doc.id, did);
        assert!(doc.controller.is_none());
        assert_eq!(doc.verification_methods.len(), 3);
        assert!(!doc.created.is_null());
        assert!(!doc.updated.is_null());
        assert!(!doc.version_id.is_empty());
        assert!(doc.next_update.is_none());
        assert!(doc.proof.is_none());
        
        // Verify verification methods
        for vm in &doc.verification_methods {
            assert!(vm.id.starts_with(&did));
            assert!(!vm.key_type.is_empty());
            assert_eq!(vm.controller, did);
            assert!(vm.public_key_multibase.is_some());
        }
        
        // Verify key references
        assert_eq!(doc.authentication.len(), 3);
        assert_eq!(doc.assertion_method.len(), 3);
        assert_eq!(doc.key_agreement.len(), 3);
        assert_eq!(doc.key_encapsulation.len(), 0);
        assert_eq!(doc.capability_invocation.len(), 0);
        assert_eq!(doc.capability_delegation.len(), 0);
        
        // Verify services
        assert!(doc.services.is_empty());
    }
    
    /// Test hybrid key schemes
    #[tokio::test]
    async fn test_hybrid_key_schemes() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Test Ed25519 + Dilithium3 hybrid
        let (did1, doc1) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519Dilithium3],
            vec![KeyPurpose::Authentication, KeyPurpose::Assertion],
        ).await.unwrap();
        
        assert_eq!(doc1.verification_methods.len(), 1);
        let hybrid_vm = &doc1.verification_methods[0];
        assert_eq!(hybrid_vm.key_type, "Ed25519Dilithium3VerificationKey2020");
        
        // Test Ed25519 + Kyber512 hybrid
        let (did2, doc2) = identity.create_did(
            "polynet",
            vec![KeyType::Ed25519Kyber512],
            vec![KeyPurpose::Authentication, KeyPurpose::KeyAgreement],
        ).await.unwrap();
        
        assert_eq!(doc2.verification_methods.len(), 1);
        let hybrid_vm2 = &doc2.verification_methods[0];
        assert_eq!(hybrid_vm2.key_type, "Ed25519Kyber512VerificationKey2020");
        
        // Verify hybrid keys have appropriate purposes
        let keys1 = identity.get_did_keys(&did1).await.unwrap();
        let keys2 = identity.get_did_keys(&did2).await.unwrap();
        
        assert_eq!(keys1.len(), 1);
        assert_eq!(keys2.len(), 1);
        
        let hybrid_key1 = &keys1[0];
        let hybrid_key2 = &keys2[0];
        
        assert_eq!(hybrid_key1.key_type, KeyType::Ed25519Dilithium3);
        assert_eq!(hybrid_key2.key_type, KeyType::Ed25519Kyber512);
        
        // Verify hybrid public keys are longer (combined)
        assert!(hybrid_key1.public_key.len() > 32); // Longer than Ed25519 alone
        assert!(hybrid_key2.public_key.len() > 32); // Longer than Ed25519 alone
    }
    
    /// Test error handling scenarios
    #[tokio::test]
    async fn test_error_handling_scenarios() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Test resolving non-existent DID
        let non_existent = identity.resolve_did("did:polynet:nonexistent").await;
        assert!(non_existent.is_ok());
        assert!(non_existent.unwrap().is_none());
        
        // Test rotating keys for non-existent DID
        let rotate_result = identity.rotate_keys(
            "did:polynet:nonexistent",
            vec!["key1".to_string()],
            vec![KeyType::Ed25519],
            vec![KeyPurpose::Authentication],
        ).await;
        
        assert!(rotate_result.is_err());
        if let Err(DidError::DidNotFound(_)) = rotate_result {
            // Expected error
        } else {
            panic!("Expected DidNotFound error");
        }
        
        // Test getting keys for non-existent DID
        let keys_result = identity.get_did_keys("did:polynet:nonexistent").await;
        assert!(keys_result.is_ok());
        assert!(keys_result.unwrap().is_empty());
    }
    
    /// Test performance characteristics
    #[tokio::test]
    async fn test_performance_characteristics() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        let start = std::time::Instant::now();
        
        // Create multiple DIDs
        let mut dids = Vec::new();
        for i in 0..10 {
            let (did, _) = identity.create_did(
                "polynet",
                vec![KeyType::Ed25519],
                vec![KeyPurpose::Authentication],
            ).await.unwrap();
            dids.push(did);
        }
        
        let creation_time = start.elapsed();
        println!("Created 10 DIDs in {:?}", creation_time);
        assert!(creation_time.as_millis() < 5000); // Should be reasonably fast
        
        // Test resolution performance
        let resolve_start = std::time::Instant::now();
        
        for did in &dids {
            let _resolved = identity.resolve_did(did).await.unwrap();
        }
        
        let resolve_time = resolve_start.elapsed();
        println!("Resolved 10 DIDs in {:?}", resolve_time);
        assert!(resolve_time.as_millis() < 1000); // Should be very fast
        
        // Test key rotation performance
        let rotation_start = std::time::Instant::now();
        
        for did in &dids {
            let keys = identity.get_did_keys(did).await.unwrap();
            let key_ids: Vec<String> = keys.iter().map(|k| k.id.clone()).collect();
            
            let _updated = identity.rotate_keys(
                did,
                key_ids,
                vec![KeyType::Dilithium3],
                vec![KeyPurpose::Authentication],
            ).await.unwrap();
        }
        
        let rotation_time = rotation_start.elapsed();
        println!("Rotated keys for 10 DIDs in {:?}", rotation_time);
        assert!(rotation_time.as_millis() < 10000); // Should be reasonably fast
    }
}

/// Performance benchmarks for DID operations
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    /// Benchmark DID creation
    #[tokio::test]
    async fn benchmark_did_creation() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        let start = Instant::now();
        
        for _ in 0..100 {
            let _ = identity.create_did(
                "polynet",
                vec![KeyType::Ed25519],
                vec![KeyPurpose::Authentication],
            ).await.unwrap();
        }
        
        let duration = start.elapsed();
        println!("Created 100 DIDs in {:?}", duration);
        assert!(duration.as_millis() < 30000); // Should be reasonably fast
    }
    
    /// Benchmark key generation
    #[tokio::test]
    async fn benchmark_key_generation() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        let start = Instant::now();
        
        for _ in 0..50 {
            let _ = identity.create_did(
                "polynet",
                vec![KeyType::Dilithium3, KeyType::Kyber512],
                vec![KeyPurpose::Authentication, KeyPurpose::KeyAgreement],
            ).await.unwrap();
        }
        
        let duration = start.elapsed();
        println!("Generated 100 post-quantum keys in {:?}", duration);
        assert!(duration.as_millis() < 60000); // Should be reasonably fast
    }
    
    /// Benchmark DID resolution
    #[tokio::test]
    async fn benchmark_did_resolution() {
        let temp_dir = tempdir().unwrap();
        let store = DidStore::new(temp_dir.path().to_path_buf()).await.unwrap();
        let identity = DidIdentity::new(store);
        
        // Create DIDs first
        let mut dids = Vec::new();
        for _ in 0..50 {
            let (did, _) = identity.create_did(
                "polynet",
                vec![KeyType::Ed25519],
                vec![KeyPurpose::Authentication],
            ).await.unwrap();
            dids.push(did);
        }
        
        let start = Instant::now();
        
        for did in &dids {
            let _resolved = identity.resolve_did(did).await.unwrap();
        }
        
        let duration = start.elapsed();
        println!("Resolved 50 DIDs in {:?}", duration);
        assert!(duration.as_millis() < 1000); // Should be very fast
    }
}
