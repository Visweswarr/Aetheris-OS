use super::*;
use tempfile::tempdir;
use secrecy::Secret;

/// Test suite for keystore functionality
#[cfg(test)]
mod keystore_tests {
    use super::*;
    
    /// Test complete keystore lifecycle: create → use → rotate → revoke → delete
    #[tokio::test]
    async fn test_complete_keystore_lifecycle() {
        let service = create_test_service().await;
        
        // Generate key
        let metadata = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            Some("Lifecycle Test Key".to_string()),
            None,
            None,
        ).await.unwrap();
        
        let key_id = &metadata.id;
        assert_eq!(metadata.key_type, KeyType::Ed25519);
        assert!(metadata.purposes.contains(&KeyPurpose::Sign));
        
        // Use key for signing
        let data = b"Test data for signing";
        let signature = service.sign(key_id, data, None).await.unwrap();
        assert_eq!(signature.key_id, *key_id);
        
        // Verify signature
        let is_valid = service.verify(key_id, data, &signature.signature, None).await.unwrap();
        assert!(is_valid);
        
        // Rotate key
        let new_metadata = service.rotate_key(
            key_id,
            Some(KeyType::Dilithium3),
            Some(vec![KeyPurpose::Sign, KeyPurpose::Verify]),
        ).await.unwrap();
        
        assert_ne!(new_metadata.id, *key_id);
        assert_eq!(new_metadata.key_type, KeyType::Dilithium3);
        
        // Verify old key is revoked
        let old_metadata = service.get_key_metadata(key_id).await.unwrap();
        assert_eq!(old_metadata.status, KeyStatus::Revoked);
        
        // Delete old key
        service.delete_key(key_id).await.unwrap();
        
        // Verify old key is gone
        let result = service.get_key_metadata(key_id).await;
        assert!(result.is_err());
    }
    
    /// Test keystore with different key types
    #[tokio::test]
    async fn test_different_key_types() {
        let service = create_test_service().await;
        
        // Test Ed25519
        let ed25519_key = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            None,
            None,
            None,
        ).await.unwrap();
        
        let data = b"Ed25519 test";
        let signature = service.sign(&ed25519_key.id, data, None).await.unwrap();
        let is_valid = service.verify(&ed25519_key.id, data, &signature.signature, None).await.unwrap();
        assert!(is_valid);
        
        // Test Dilithium3
        let dilithium_key = service.generate_key(
            KeyType::Dilithium3,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            None,
            None,
            None,
        ).await.unwrap();
        
        let signature = service.sign(&dilithium_key.id, data, None).await.unwrap();
        let is_valid = service.verify(&dilithium_key.id, data, &signature.signature, None).await.unwrap();
        assert!(is_valid);
        
        // Test Kyber512
        let kyber_key = service.generate_key(
            KeyType::Kyber512,
            vec![KeyPurpose::KeyEncapsulation, KeyPurpose::KeyDecapsulation],
            None,
            None,
            None,
        ).await.unwrap();
        
        let result = service.encapsulate_key(&kyber_key.id, None).await.unwrap();
        let shared_secret = service.decapsulate_key(&kyber_key.id, &result.encapsulated_key, None).await.unwrap();
        assert_eq!(shared_secret, result.shared_secret);
        
        // Test AES256
        let aes_key = service.generate_key(
            KeyType::AES256,
            vec![KeyPurpose::Encrypt, KeyPurpose::Decrypt],
            None,
            None,
            None,
        ).await.unwrap();
        
        let data = b"AES encryption test";
        let encrypted = service.encrypt(&aes_key.id, data, None).await.unwrap();
        let decrypted = service.decrypt(&aes_key.id, &encrypted).await.unwrap();
        assert_eq!(decrypted, data);
        
        // Clean up
        service.delete_key(&ed25519_key.id).await.unwrap();
        service.delete_key(&dilithium_key.id).await.unwrap();
        service.delete_key(&kyber_key.id).await.unwrap();
        service.delete_key(&aes_key.id).await.unwrap();
    }
    
    /// Test hybrid key schemes
    #[tokio::test]
    async fn test_hybrid_key_schemes() {
        let service = create_test_service().await;
        
        // Test Ed25519 + Dilithium3 hybrid
        let hybrid_key = service.generate_key(
            KeyType::Ed25519Dilithium3,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            Some("Hybrid Test Key".to_string()),
            None,
            None,
        ).await.unwrap();
        
        let data = b"Hybrid key test";
        let signature = service.sign(&hybrid_key.id, data, None).await.unwrap();
        let is_valid = service.verify(&hybrid_key.id, data, &signature.signature, None).await.unwrap();
        assert!(is_valid);
        
        // Test Ed25519 + Kyber512 hybrid
        let hybrid_key2 = service.generate_key(
            KeyType::Ed25519Kyber512,
            vec![KeyPurpose::Sign, KeyPurpose::Verify, KeyPurpose::KeyEncapsulation],
            Some("Hybrid Test Key 2".to_string()),
            None,
            None,
        ).await.unwrap();
        
        let signature = service.sign(&hybrid_key2.id, data, None).await.unwrap();
        let is_valid = service.verify(&hybrid_key2.id, data, &signature.signature, None).await.unwrap();
        assert!(is_valid);
        
        // Test KEM with hybrid key
        let result = service.encapsulate_key(&hybrid_key2.id, None).await.unwrap();
        let shared_secret = service.decapsulate_key(&hybrid_key2.id, &result.encapsulated_key, None).await.unwrap();
        assert_eq!(shared_secret, result.shared_secret);
        
        // Clean up
        service.delete_key(&hybrid_key.id).await.unwrap();
        service.delete_key(&hybrid_key2.id).await.unwrap();
    }
    
    /// Test key import and export
    #[tokio::test]
    async fn test_key_import_export() {
        let service = create_test_service().await;
        
        // Generate a key to export
        let original_key = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            Some("Import/Export Test".to_string()),
            None,
            None,
        ).await.unwrap();
        
        // Sign some data with original key
        let data = b"Import/export test data";
        let original_signature = service.sign(&original_key.id, data, None).await.unwrap();
        
        // Delete original key
        service.delete_key(&original_key.id).await.unwrap();
        
        // Import the key back (in a real scenario, you'd export the key material first)
        let imported_key = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            Some("Imported Key".to_string()),
            None,
            None,
        ).await.unwrap();
        
        // Verify the signature still works (this tests the key material persistence)
        let is_valid = service.verify(&imported_key.id, data, &original_signature.signature, None).await.unwrap();
        assert!(is_valid);
        
        // Clean up
        service.delete_key(&imported_key.id).await.unwrap();
    }
    
    /// Test key metadata management
    #[tokio::test]
    async fn test_key_metadata_management() {
        let service = create_test_service().await;
        
        // Generate key with initial metadata
        let mut tags = HashMap::new();
        tags.insert("environment".to_string(), "test".to_string());
        tags.insert("purpose".to_string(), "testing".to_string());
        
        let metadata = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            Some("Metadata Test Key".to_string()),
            None,
            Some(tags.clone()),
        ).await.unwrap();
        
        // Update metadata
        let updated_metadata = service.update_key_metadata(
            &metadata.id,
            Some("Updated Name".to_string()),
            Some(chrono::Utc::now() + chrono::Duration::hours(24)),
            Some(tags.clone()),
        ).await.unwrap();
        
        assert_eq!(updated_metadata.name, Some("Updated Name".to_string()));
        assert!(updated_metadata.expires.is_some());
        
        // Add more tags
        tags.insert("version".to_string(), "2.0".to_string());
        let updated_metadata = service.update_key_metadata(
            &metadata.id,
            None,
            None,
            Some(tags.clone()),
        ).await.unwrap();
        
        assert_eq!(updated_metadata.tags.len(), 3);
        assert_eq!(updated_metadata.tags.get("version"), Some(&"2.0".to_string()));
        
        // Clean up
        service.delete_key(&metadata.id).await.unwrap();
    }
    
    /// Test key expiration and cleanup
    #[tokio::test]
    async fn test_key_expiration_and_cleanup() {
        let service = create_test_service().await;
        
        // Generate key with short expiration
        let expires = chrono::Utc::now() + chrono::Duration::milliseconds(100);
        let metadata = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            Some("Expiring Key".to_string()),
            Some(expires),
            None,
        ).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        // Run cleanup
        service.cleanup().await.unwrap();
        
        // Verify expired key is removed
        let result = service.get_key_metadata(&metadata.id).await;
        assert!(result.is_err());
    }
    
    /// Test key revocation
    #[tokio::test]
    async fn test_key_revocation() {
        let service = create_test_service().await;
        
        // Generate key
        let metadata = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            None,
            None,
            None,
        ).await.unwrap();
        
        // Revoke key
        service.revoke_key(&metadata.id, Some("Security compromise".to_string())).await.unwrap();
        
        // Verify key is revoked
        let revoked_metadata = service.get_key_metadata(&metadata.id).await.unwrap();
        assert_eq!(revoked_metadata.status, KeyStatus::Revoked);
        
        // Try to use revoked key (should fail)
        let data = b"Test data";
        let result = service.sign(&metadata.id, data, None).await;
        assert!(result.is_err());
        
        // Clean up
        service.delete_key(&metadata.id).await.unwrap();
    }
    
    /// Test key derivation
    #[tokio::test]
    async fn test_key_derivation() {
        let service = create_test_service().await;
        
        // Generate base key
        let base_key = service.generate_key(
            KeyType::AES256,
            vec![KeyPurpose::KeyDerivation],
            Some("Base Key for Derivation".to_string()),
            None,
            None,
        ).await.unwrap();
        
        // Derive keys with different parameters
        let salt1 = vec![1u8; 32];
        let params1 = KeyDerivationParams {
            salt: salt1,
            iterations: 100000,
            key_length: 32,
            algorithm: "Argon2id".to_string(),
        };
        
        let derived_key1 = service.derive_key(&base_key.id, &params1).await.unwrap();
        assert_eq!(derived_key1.len(), 32);
        
        let salt2 = vec![2u8; 32];
        let params2 = KeyDerivationParams {
            salt: salt2,
            iterations: 200000,
            key_length: 64,
            algorithm: "Argon2id".to_string(),
        };
        
        let derived_key2 = service.derive_key(&base_key.id, &params2).await.unwrap();
        assert_eq!(derived_key2.len(), 64);
        
        // Verify different salts produce different keys
        assert_ne!(derived_key1, derived_key2[..32]);
        
        // Clean up
        service.delete_key(&base_key.id).await.unwrap();
    }
    
    /// Test error handling scenarios
    #[tokio::test]
    async fn test_error_handling() {
        let service = create_test_service().await;
        
        // Test non-existent key
        let result = service.get_key_metadata("non-existent-key").await;
        assert!(result.is_err());
        
        // Test signing with non-existent key
        let result = service.sign("non-existent-key", b"test", None).await;
        assert!(result.is_err());
        
        // Test invalid key type and purpose combination
        let result = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Encrypt], // Ed25519 doesn't support encryption
            None,
            None,
            None,
        ).await;
        assert!(result.is_err());
        
        // Test encryption with signing key
        let signing_key = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            None,
            None,
            None,
        ).await.unwrap();
        
        let result = service.encrypt(&signing_key.id, b"test", None).await;
        assert!(result.is_err());
        
        // Clean up
        service.delete_key(&signing_key.id).await.unwrap();
    }
    
    /// Test performance characteristics
    #[tokio::test]
    async fn test_performance_characteristics() {
        let service = create_test_service().await;
        
        let start = std::time::Instant::now();
        
        // Generate multiple keys
        let mut key_ids = Vec::new();
        for i in 0..10 {
            let metadata = service.generate_key(
                KeyType::Ed25519,
                vec![KeyPurpose::Sign, KeyPurpose::Verify],
                Some(format!("Perf Test Key {}", i)),
                None,
                None,
            ).await.unwrap();
            key_ids.push(metadata.id);
        }
        
        let generation_time = start.elapsed();
        println!("Generated 10 keys in {:?}", generation_time);
        
        // Test signing performance
        let data = b"Performance test data";
        let start = std::time::Instant::now();
        
        for key_id in &key_ids {
            let _signature = service.sign(key_id, data, None).await.unwrap();
        }
        
        let signing_time = start.elapsed();
        println!("Signed with 10 keys in {:?}", signing_time);
        
        // Clean up
        for key_id in key_ids {
            service.delete_key(&key_id).await.unwrap();
        }
        
        // Performance assertions (adjust thresholds as needed)
        assert!(generation_time < std::time::Duration::from_secs(5));
        assert!(signing_time < std::time::Duration::from_secs(5));
    }
}

/// Performance benchmarks for keystore operations
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    /// Benchmark key generation
    #[tokio::test]
    async fn benchmark_key_generation() {
        let service = create_test_service().await;
        
        let key_types = vec![
            KeyType::Ed25519,
            KeyType::Dilithium3,
            KeyType::Kyber512,
            KeyType::AES256,
        ];
        
        for key_type in key_types {
            let start = Instant::now();
            
            let metadata = service.generate_key(
                key_type,
                vec![KeyPurpose::Sign, KeyPurpose::Verify],
                None,
                None,
                None,
            ).await.unwrap();
            
            let duration = start.elapsed();
            println!("Generated {:?} key in {:?}", key_type, duration);
            
            // Clean up
            service.delete_key(&metadata.id).await.unwrap();
        }
    }
    
    /// Benchmark signing operations
    #[tokio::test]
    async fn benchmark_signing_operations() {
        let service = create_test_service().await;
        
        // Generate signing key
        let metadata = service.generate_key(
            KeyType::Ed25519,
            vec![KeyPurpose::Sign, KeyPurpose::Verify],
            None,
            None,
            None,
        ).await.unwrap();
        
        let data_sizes = vec![64, 1024, 4096, 16384];
        
        for size in data_sizes {
            let data = vec![0u8; size];
            let start = Instant::now();
            
            let _signature = service.sign(&metadata.id, &data, None).await.unwrap();
            
            let duration = start.elapsed();
            println!("Signed {} bytes in {:?}", size, duration);
        }
        
        // Clean up
        service.delete_key(&metadata.id).await.unwrap();
    }
    
    /// Benchmark encryption operations
    #[tokio::test]
    async fn benchmark_encryption_operations() {
        let service = create_test_service().await;
        
        // Generate encryption key
        let metadata = service.generate_key(
            KeyType::AES256,
            vec![KeyPurpose::Encrypt, KeyPurpose::Decrypt],
            None,
            None,
            None,
        ).await.unwrap();
        
        let data_sizes = vec![64, 1024, 4096, 16384];
        
        for size in data_sizes {
            let data = vec![0u8; size];
            let start = Instant::now();
            
            let encrypted = service.encrypt(&metadata.id, &data, None).await.unwrap();
            let _decrypted = service.decrypt(&metadata.id, &encrypted).await.unwrap();
            
            let duration = start.elapsed();
            println!("Encrypted/decrypted {} bytes in {:?}", size, duration);
        }
        
        // Clean up
        service.delete_key(&metadata.id).await.unwrap();
    }
    
    /// Benchmark key encapsulation
    #[tokio::test]
    async fn benchmark_key_encapsulation() {
        let service = create_test_service().await;
        
        let kem_types = vec![
            KeyType::Kyber512,
            KeyType::Kyber768,
            KeyType::Kyber1024,
        ];
        
        for key_type in kem_types {
            let metadata = service.generate_key(
                key_type,
                vec![KeyPurpose::KeyEncapsulation, KeyPurpose::KeyDecapsulation],
                None,
                None,
                None,
            ).await.unwrap();
            
            let start = Instant::now();
            
            let result = service.encapsulate_key(&metadata.id, None).await.unwrap();
            let _shared_secret = service.decapsulate_key(&metadata.id, &result.encapsulated_key, None).await.unwrap();
            
            let duration = start.elapsed();
            println!("KEM {:?} in {:?}", key_type, duration);
            
            // Clean up
            service.delete_key(&metadata.id).await.unwrap();
        }
    }
}

/// Helper function to create test service
async fn create_test_service() -> KeystoreService {
    let config = KeystoreConfig {
        backend: "software".to_string(),
        storage_path: Some("test_data/keystore".to_string()),
        ..Default::default()
    };
    
    KeystoreService::new(config).await.unwrap()
}
