//! TLS Integration Tests for Aetheris OS
//! 
//! This module contains comprehensive tests for the TLS/mTLS implementation
//! with Post-Quantum Cryptography support.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use posixnet::tls::{
    TLSManager, TLSConfig, TLSSession, TLSError, TLSResult,
    TLSProfile, PQCAlgorithm, CipherSuite, TLSVersion
};

#[tokio::test]
async fn test_tls_manager_initialization() {
    let manager = TLSManager::new().await;
    assert!(manager.is_ok());
    
    let manager = manager.unwrap();
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_sessions, 0);
    assert_eq!(stats.total_handshakes, 0);
}

#[tokio::test]
async fn test_tls_config_creation() {
    let config = TLSConfig {
        profile: TLSProfile::TLS13Modern,
        pqc_algorithms: vec![PQCAlgorithm::Kyber512, PQCAlgorithm::Dilithium2],
        zero_copy: true,
        max_early_data: 16384,
        alpn_protocols: vec!["h2".to_string(), "http/1.1".to_string()],
        session_resumption: true,
        ocsp_stapling: true,
        ..Default::default()
    };
    
    assert_eq!(config.profile, TLSProfile::TLS13Modern);
    assert!(config.pqc_algorithms.contains(&PQCAlgorithm::Kyber512));
    assert!(config.zero_copy);
}

#[tokio::test]
async fn test_pqc_key_generation() {
    let manager = TLSManager::new().await.unwrap();
    
    // Test Kyber key generation
    let kyber_keys = manager.generate_kyber_keys(PQCAlgorithm::Kyber512).await;
    assert!(kyber_keys.is_ok());
    
    let kyber_keys = kyber_keys.unwrap();
    assert!(!kyber_keys.public_key.is_empty());
    assert!(!kyber_keys.private_key_id.is_empty());
    
    // Test Dilithium key generation
    let dilithium_keys = manager.generate_dilithium_keys(PQCAlgorithm::Dilithium2).await;
    assert!(dilithium_keys.is_ok());
    
    let dilithium_keys = dilithium_keys.unwrap();
    assert!(!dilithium_keys.public_key.is_empty());
    assert!(!dilithium_keys.private_key_id.is_empty());
}

#[tokio::test]
async fn test_did_certificate_creation() {
    let manager = TLSManager::new().await.unwrap();
    
    let kyber_keys = manager.generate_kyber_keys(PQCAlgorithm::Kyber512).await.unwrap();
    let dilithium_keys = manager.generate_dilithium_keys(PQCAlgorithm::Dilithium2).await.unwrap();
    
    let cert = manager.create_did_certificate(
        "did:aetheris:test:client",
        "Test Client",
        365,
        &kyber_keys,
        &dilithium_keys
    ).await;
    
    assert!(cert.is_ok());
    
    let cert = cert.unwrap();
    assert_eq!(cert.did, "did:aetheris:test:client");
    assert!(!cert.certificate_data.is_empty());
    assert!(!cert.pqc_signature.is_empty());
}

#[tokio::test]
async fn test_tls_session_creation() {
    let manager = TLSManager::new().await.unwrap();
    
    let config = TLSConfig {
        profile: TLSProfile::PQCHybrid,
        pqc_algorithms: vec![PQCAlgorithm::Kyber768, PQCAlgorithm::Dilithium3],
        ..Default::default()
    };
    
    let session = manager.create_session("test_session", &config).await;
    assert!(session.is_ok());
    
    let session = session.unwrap();
    assert_eq!(session.session_id, "test_session");
    assert!(!session.handshake_completed);
}

#[tokio::test]
async fn test_tls_handshake_simulation() {
    let manager = TLSManager::new().await.unwrap();
    
    // Create client and server configurations
    let client_config = TLSConfig {
        profile: TLSProfile::TLS13Modern,
        pqc_algorithms: vec![PQCAlgorithm::Kyber512, PQCAlgorithm::Dilithium2],
        ..Default::default()
    };
    
    let server_config = TLSConfig {
        profile: TLSProfile::TLS13Modern,
        pqc_algorithms: vec![PQCAlgorithm::Kyber512, PQCAlgorithm::Dilithium2],
        ..Default::default()
    };
    
    // Create sessions
    let client_session = manager.create_session("client_session", &client_config).await.unwrap();
    let server_session = manager.create_session("server_session", &server_config).await.unwrap();
    
    // Simulate handshake
    let handshake_result = manager.perform_handshake(&client_session, &server_session).await;
    assert!(handshake_result.is_ok());
    
    let result = handshake_result.unwrap();
    assert!(result.success);
    assert!(result.handshake_time_ms > 0);
    assert!(result.cipher_suite.is_some());
    assert!(result.tls_version.is_some());
}

#[tokio::test]
async fn test_tls_encryption_decryption() {
    let manager = TLSManager::new().await.unwrap();
    
    let config = TLSConfig {
        profile: TLSProfile::PQCHybrid,
        pqc_algorithms: vec![PQCAlgorithm::Kyber768, PQCAlgorithm::Dilithium3],
        ..Default::default()
    };
    
    let session = manager.create_session("encryption_test", &config).await.unwrap();
    
    // Test data
    let original_data = b"Hello, Post-Quantum TLS!";
    
    // Encrypt data
    let encrypted = manager.encrypt_data(&session, original_data).await;
    assert!(encrypted.is_ok());
    
    let encrypted_data = encrypted.unwrap();
    assert_ne!(encrypted_data, original_data);
    
    // Decrypt data
    let decrypted = manager.decrypt_data(&session, &encrypted_data).await;
    assert!(decrypted.is_ok());
    
    let decrypted_data = decrypted.unwrap();
    assert_eq!(decrypted_data, original_data);
}

#[tokio::test]
async fn test_tls_rekeying() {
    let manager = TLSManager::new().await.unwrap();
    
    let config = TLSConfig {
        profile: TLSProfile::PQCHybrid,
        pqc_algorithms: vec![PQCAlgorithm::Kyber512, PQCAlgorithm::Dilithium2],
        ..Default::default()
    };
    
    let session = manager.create_session("rekey_test", &config).await.unwrap();
    
    // Perform rekeying
    let rekey_result = manager.rekey_session(&session, PQCAlgorithm::Kyber768).await;
    assert!(rekey_result.is_ok());
    
    // Verify session is still functional after rekeying
    let test_data = b"Test data after rekeying";
    let encrypted = manager.encrypt_data(&session, test_data).await.unwrap();
    let decrypted = manager.decrypt_data(&session, &encrypted).await.unwrap();
    assert_eq!(decrypted, test_data);
}

#[tokio::test]
async fn test_tls_session_cleanup() {
    let manager = TLSManager::new().await.unwrap();
    
    let config = TLSConfig {
        profile: TLSProfile::TLS13Modern,
        ..Default::default()
    };
    
    let session = manager.create_session("cleanup_test", &config).await.unwrap();
    
    // Close session
    let close_result = manager.close_session(&session).await;
    assert!(close_result.is_ok());
    
    // Verify session is closed
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_sessions, 0);
}

#[tokio::test]
async fn test_tls_error_handling() {
    let manager = TLSManager::new().await.unwrap();
    
    // Test invalid session ID
    let invalid_session = TLSSession {
        session_id: "invalid_session".to_string(),
        ..Default::default()
    };
    
    let result = manager.encrypt_data(&invalid_session, b"test").await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        TLSError::SessionError(_) => {}, // Expected
        _ => panic!("Expected SessionError"),
    }
}

#[tokio::test]
async fn test_tls_performance_benchmark() {
    let manager = TLSManager::new().await.unwrap();
    
    let config = TLSConfig {
        profile: TLSProfile::PQCHybrid,
        pqc_algorithms: vec![PQCAlgorithm::Kyber768, PQCAlgorithm::Dilithium3],
        zero_copy: true,
        ..Default::default()
    };
    
    let session = manager.create_session("perf_test", &config).await.unwrap();
    
    // Benchmark encryption/decryption performance
    let test_data = vec![0u8; 1024]; // 1KB test data
    let iterations = 1000;
    
    let start = std::time::Instant::now();
    
    for _ in 0..iterations {
        let encrypted = manager.encrypt_data(&session, &test_data).await.unwrap();
        let _decrypted = manager.decrypt_data(&session, &encrypted).await.unwrap();
    }
    
    let duration = start.elapsed();
    let throughput = (test_data.len() * iterations * 2) as f64 / duration.as_secs_f64();
    
    println!("TLS Performance: {:.2} MB/s", throughput / 1_000_000.0);
    
    // Verify performance is within acceptable bounds (> 100 MB/s)
    assert!(throughput > 100_000_000.0);
}

#[tokio::test]
async fn test_tls_concurrent_sessions() {
    let manager = Arc::new(TLSManager::new().await.unwrap());
    
    let config = TLSConfig {
        profile: TLSProfile::TLS13Modern,
        pqc_algorithms: vec![PQCAlgorithm::Kyber512, PQCAlgorithm::Dilithium2],
        ..Default::default()
    };
    
    let num_sessions = 100;
    let mut handles = vec![];
    
    // Create multiple concurrent sessions
    for i in 0..num_sessions {
        let manager_clone = manager.clone();
        let config_clone = config.clone();
        
        let handle = tokio::spawn(async move {
            let session_id = format!("concurrent_session_{}", i);
            let session = manager_clone.create_session(&session_id, &config_clone).await.unwrap();
            
            // Perform some operations
            let test_data = format!("Test data for session {}", i);
            let encrypted = manager_clone.encrypt_data(&session, test_data.as_bytes()).await.unwrap();
            let decrypted = manager_clone.decrypt_data(&session, &encrypted).await.unwrap();
            
            assert_eq!(decrypted, test_data.as_bytes());
            
            manager_clone.close_session(&session).await.unwrap();
        });
        
        handles.push(handle);
    }
    
    // Wait for all sessions to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all sessions are closed
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_sessions, 0);
}

#[tokio::test]
async fn test_tls_mtls_authentication() {
    let manager = TLSManager::new().await.unwrap();
    
    // Create client and server certificates
    let client_kyber = manager.generate_kyber_keys(PQCAlgorithm::Kyber512).await.unwrap();
    let client_dilithium = manager.generate_dilithium_keys(PQCAlgorithm::Dilithium2).await.unwrap();
    let server_kyber = manager.generate_kyber_keys(PQCAlgorithm::Kyber512).await.unwrap();
    let server_dilithium = manager.generate_dilithium_keys(PQCAlgorithm::Dilithium2).await.unwrap();
    
    let client_cert = manager.create_did_certificate(
        "did:aetheris:test:client",
        "Test Client",
        365,
        &client_kyber,
        &client_dilithium
    ).await.unwrap();
    
    let server_cert = manager.create_did_certificate(
        "did:aetheris:test:server",
        "Test Server",
        365,
        &server_kyber,
        &server_dilithium
    ).await.unwrap();
    
    // Create mTLS configuration
    let mtls_config = TLSConfig {
        profile: TLSProfile::PQCHybrid,
        pqc_algorithms: vec![PQCAlgorithm::Kyber512, PQCAlgorithm::Dilithium2],
        require_client_cert: true,
        ..Default::default()
    };
    
    let client_session = manager.create_session("mtls_client", &mtls_config).await.unwrap();
    let server_session = manager.create_session("mtls_server", &mtls_config).await.unwrap();
    
    // Perform mTLS handshake
    let handshake_result = manager.perform_handshake(&client_session, &server_session).await;
    assert!(handshake_result.is_ok());
    
    let result = handshake_result.unwrap();
    assert!(result.success);
    assert!(result.is_mtls);
}

#[tokio::test]
async fn test_tls_zero_copy_io() {
    let manager = TLSManager::new().await.unwrap();
    
    let config = TLSConfig {
        profile: TLSProfile::PQCHybrid,
        pqc_algorithms: vec![PQCAlgorithm::Kyber768, PQCAlgorithm::Dilithium3],
        zero_copy: true,
        ..Default::default()
    };
    
    let session = manager.create_session("zero_copy_test", &config).await.unwrap();
    
    // Test zero-copy operations
    let large_data = vec![0u8; 64 * 1024]; // 64KB test data
    
    let start = std::time::Instant::now();
    let encrypted = manager.encrypt_data(&session, &large_data).await.unwrap();
    let decrypted = manager.decrypt_data(&session, &encrypted).await.unwrap();
    let duration = start.elapsed();
    
    assert_eq!(decrypted, large_data);
    
    // Zero-copy should be faster than regular copy
    let throughput = (large_data.len() * 2) as f64 / duration.as_secs_f64();
    println!("Zero-copy TLS throughput: {:.2} MB/s", throughput / 1_000_000.0);
    
    // Should achieve high throughput with zero-copy
    assert!(throughput > 500_000_000.0); // > 500 MB/s
}

#[tokio::test]
async fn test_tls_algorithm_negotiation() {
    let manager = TLSManager::new().await.unwrap();
    
    // Client supports multiple algorithms
    let client_config = TLSConfig {
        profile: TLSProfile::PQCHybrid,
        pqc_algorithms: vec![
            PQCAlgorithm::Kyber1024,
            PQCAlgorithm::Kyber768,
            PQCAlgorithm::Kyber512,
            PQCAlgorithm::Dilithium5,
            PQCAlgorithm::Dilithium3,
            PQCAlgorithm::Dilithium2,
        ],
        ..Default::default()
    };
    
    // Server supports fewer algorithms
    let server_config = TLSConfig {
        profile: TLSProfile::PQCHybrid,
        pqc_algorithms: vec![
            PQCAlgorithm::Kyber768,
            PQCAlgorithm::Dilithium3,
        ],
        ..Default::default()
    };
    
    let client_session = manager.create_session("algo_client", &client_config).await.unwrap();
    let server_session = manager.create_session("algo_server", &server_config).await.unwrap();
    
    // Perform handshake and verify algorithm negotiation
    let handshake_result = manager.perform_handshake(&client_session, &server_session).await;
    assert!(handshake_result.is_ok());
    
    let result = handshake_result.unwrap();
    assert!(result.success);
    
    // Should negotiate to the strongest common algorithm
    assert_eq!(result.negotiated_kyber, Some(PQCAlgorithm::Kyber768));
    assert_eq!(result.negotiated_dilithium, Some(PQCAlgorithm::Dilithium3));
}
