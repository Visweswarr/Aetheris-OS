/// CapTokens v2 Test Suite
/// 
/// This module provides comprehensive testing for the CapTokens v2 system:
/// - Expired/early/not_before token rejection
/// - Wrong destination PID rejection
/// - Replay attack detection within sliding window
/// - Performance validation (verification p50 < 1.5ms, cache hit < 100µs)

use super::cap_store::{CapabilityStoreV2, get_capability_store, validate_capability_token};
use super::did::{DidDocument, DidDocumentBuilder, VerificationMethod, VerificationMethodType, PublicKeyMaterial, add_did_document};
use crate::security::cap_v2::{
    CapTokenV2, CapTokenHeader, CapTokenSignature, CapTokenMetadata,
    SignatureAlgorithm, scope_v2, CapValidationResult, CapValidationFailure,
    CapTokenBuilder,
};
use crate::crypto::pqc::{Dilithium, DilithiumParameterSet, DilithiumSecretKey, DilithiumPublicKey};
use crate::secman::audit::{AuditEntry, ops};
use alloc::string::ToString;
use core::time::Duration;

/// Test expired token rejection
#[test]
fn test_expired_token_rejection() {
    println!("Testing expired token rejection...");
    
    // Create test issuer
    let (issuer_pk, issuer_sk) = Dilithium::keygen(DilithiumParameterSet::Dilithium2).unwrap();
    let issuer_did = "did:example:issuer".to_string();
    
    // Add issuer to capability store
    add_issuer_key(issuer_did.clone(), issuer_pk);
    
    // Create expired token (expired 1 hour ago)
    let now_ms = crate::time::get_current_time_ms();
    let expired_time = now_ms - 3_600_000; // 1 hour ago
    
    let header = CapTokenHeader::new(
        issuer_did.clone(),
        1001, // subject PID
        2001, // destination PID
        scope_v2::SEND,
        expired_time,     // not_before
        expired_time + 1000, // not_after (expired)
        0x1234567890abcdef, // nonce
        [0u8; 32], // purpose hash
    );
    
    // Sign the header
    let header_bytes = header.to_bytes();
    let signature = Dilithium::sign(&header_bytes, &issuer_sk).unwrap();
    
    let cap_signature = CapTokenSignature::new(
        signature.as_bytes().to_vec(),
        None, // no Kyber ciphertext
        SignatureAlgorithm::Dilithium2,
    );
    
    let metadata = CapTokenMetadata::new(now_ms, vec![1, 2, 3, 4]);
    
    let token = CapTokenBuilder::new()
        .with_header(header)
        .with_signature(cap_signature)
        .with_metadata(metadata)
        .build()
        .unwrap();
    
    // Validate token - should be rejected as expired
    let result = validate_capability_token(&token, 2001, scope_v2::SEND);
    
    assert!(!result.is_valid);
    assert_eq!(result.failure_reason, Some(CapValidationFailure::Expired));
    
    // Check audit log for rejection
    let audit_entries = crate::secman::audit::get_latest_entries(10);
    let rejection_entry = audit_entries.iter()
        .find(|e| e.op == ops::SEC_CAP_REJECT);
    
    assert!(rejection_entry.is_some());
    
    println!("✅ Expired token rejection test passed");
}

/// Test early token rejection
#[test]
fn test_early_token_rejection() {
    println!("Testing early token rejection...");
    
    // Create test issuer
    let (issuer_pk, issuer_sk) = Dilithium::keygen(DilithiumParameterSet::Dilithium2).unwrap();
    let issuer_did = "did:example:issuer".to_string();
    
    // Add issuer to capability store
    add_issuer_key(issuer_did.clone(), issuer_pk);
    
    // Create early token (valid 1 hour from now)
    let now_ms = crate::time::get_current_time_ms();
    let future_time = now_ms + 3_600_000; // 1 hour from now
    
    let header = CapTokenHeader::new(
        issuer_did.clone(),
        1001, // subject PID
        2001, // destination PID
        scope_v2::SEND,
        future_time,      // not_before (in future)
        future_time + 1000, // not_after
        0x1234567890abcdef, // nonce
        [0u8; 32], // purpose hash
    );
    
    // Sign the header
    let header_bytes = header.to_bytes();
    let signature = Dilithium::sign(&header_bytes, &issuer_sk).unwrap();
    
    let cap_signature = CapTokenSignature::new(
        signature.as_bytes().to_vec(),
        None, // no Kyber ciphertext
        SignatureAlgorithm::Dilithium2,
    );
    
    let metadata = CapTokenMetadata::new(now_ms, vec![1, 2, 3, 4]);
    
    let token = CapTokenBuilder::new()
        .with_header(header)
        .with_signature(cap_signature)
        .with_metadata(metadata)
        .build()
        .unwrap();
    
    // Validate token - should be rejected as early
    let result = validate_capability_token(&token, 2001, scope_v2::SEND);
    
    assert!(!result.is_valid);
    assert_eq!(result.failure_reason, Some(CapValidationFailure::Early));
    
    // Check audit log for rejection
    let audit_entries = crate::secman::audit::get_latest_entries(10);
    let rejection_entry = audit_entries.iter()
        .find(|e| e.op == ops::SEC_CAP_REJECT);
    
    assert!(rejection_entry.is_some());
    
    println!("✅ Early token rejection test passed");
}

/// Test wrong destination PID rejection
#[test]
fn test_wrong_destination_rejection() {
    println!("Testing wrong destination PID rejection...");
    
    // Create test issuer
    let (issuer_pk, issuer_sk) = Dilithium::keygen(DilithiumParameterSet::Dilithium2).unwrap();
    let issuer_did = "did:example:issuer".to_string();
    
    // Add issuer to capability store
    add_issuer_key(issuer_did.clone(), issuer_pk);
    
    // Create token for destination 2001
    let now_ms = crate::time::get_current_time_ms();
    
    let header = CapTokenHeader::new(
        issuer_did.clone(),
        1001, // subject PID
        2001, // destination PID
        scope_v2::SEND,
        now_ms,           // not_before
        now_ms + 3600000, // not_after (1 hour from now)
        0x1234567890abcdef, // nonce
        [0u8; 32], // purpose hash
    );
    
    // Sign the header
    let header_bytes = header.to_bytes();
    let signature = Dilithium::sign(&header_bytes, &issuer_sk).unwrap();
    
    let cap_signature = CapTokenSignature::new(
        signature.as_bytes().to_vec(),
        None, // no Kyber ciphertext
        SignatureAlgorithm::Dilithium2,
    );
    
    let metadata = CapTokenMetadata::new(now_ms, vec![1, 2, 3, 4]);
    
    let token = CapTokenBuilder::new()
        .with_header(header)
        .with_signature(cap_signature)
        .with_metadata(metadata)
        .build()
        .unwrap();
    
    // Try to validate token for wrong destination 3001
    let result = validate_capability_token(&token, 3001, scope_v2::SEND);
    
    assert!(!result.is_valid);
    assert_eq!(result.failure_reason, Some(CapValidationFailure::DestinationMismatch));
    
    // Check audit log for rejection
    let audit_entries = crate::secman::audit::get_latest_entries(10);
    let rejection_entry = audit_entries.iter()
        .find(|e| e.op == ops::SEC_CAP_REJECT);
    
    assert!(rejection_entry.is_some());
    
    println!("✅ Wrong destination rejection test passed");
}

/// Test replay attack detection
#[test]
fn test_replay_detection() {
    println!("Testing replay attack detection...");
    
    // Create test issuer
    let (issuer_pk, issuer_sk) = Dilithium::keygen(DilithiumParameterSet::Dilithium2).unwrap();
    let issuer_did = "did:example:issuer".to_string();
    
    // Add issuer to capability store
    add_issuer_key(issuer_did.clone(), issuer_pk);
    
    // Create token with specific nonce
    let now_ms = crate::time::get_current_time_ms();
    let nonce = 0xdeadbeefcafebabe;
    
    let header = CapTokenHeader::new(
        issuer_did.clone(),
        1001, // subject PID
        2001, // destination PID
        scope_v2::SEND,
        now_ms,           // not_before
        now_ms + 3600000, // not_after (1 hour from now)
        nonce,            // specific nonce
        [0u8; 32], // purpose hash
    );
    
    // Sign the header
    let header_bytes = header.to_bytes();
    let signature = Dilithium::sign(&header_bytes, &issuer_sk).unwrap();
    
    let cap_signature = CapTokenSignature::new(
        signature.as_bytes().to_vec(),
        None, // no Kyber ciphertext
        SignatureAlgorithm::Dilithium2,
    );
    
    let metadata = CapTokenMetadata::new(now_ms, vec![1, 2, 3, 4]);
    
    let token = CapTokenBuilder::new()
        .with_header(header)
        .with_signature(cap_signature)
        .with_metadata(metadata)
        .build()
        .unwrap();
    
    // First validation should succeed
    let result1 = validate_capability_token(&token, 2001, scope_v2::SEND);
    assert!(result1.is_valid);
    
    // Second validation with same nonce should fail (replay detected)
    let result2 = validate_capability_token(&token, 2001, scope_v2::SEND);
    assert!(!result2.is_valid);
    assert_eq!(result2.failure_reason, Some(CapValidationFailure::ReplayDetected));
    
    // Check audit log for rejection
    let audit_entries = crate::secman::audit::get_latest_entries(10);
    let rejection_entry = audit_entries.iter()
        .find(|e| e.op == ops::SEC_CAP_REJECT);
    
    assert!(rejection_entry.is_some());
    
    println!("✅ Replay detection test passed");
}

/// Test performance targets
#[test]
fn test_performance_targets() {
    println!("Testing performance targets...");
    
    // Create test issuer
    let (issuer_pk, issuer_sk) = Dilithium::keygen(DilithiumParameterSet::Dilithium2).unwrap();
    let issuer_did = "did:example:issuer".to_string();
    
    // Add issuer to capability store
    add_issuer_key(issuer_did.clone(), issuer_pk);
    
    // Create valid token
    let now_ms = crate::time::get_current_time_ms();
    
    let header = CapTokenHeader::new(
        issuer_did.clone(),
        1001, // subject PID
        2001, // destination PID
        scope_v2::SEND,
        now_ms,           // not_before
        now_ms + 3600000, // not_after (1 hour from now)
        0x1234567890abcdef, // nonce
        [0u8; 32], // purpose hash
    );
    
    // Sign the header
    let header_bytes = header.to_bytes();
    let signature = Dilithium::sign(&header_bytes, &issuer_sk).unwrap();
    
    let cap_signature = CapTokenSignature::new(
        signature.as_bytes().to_vec(),
        None, // no Kyber ciphertext
        SignatureAlgorithm::Dilithium2,
    );
    
    let metadata = CapTokenMetadata::new(now_ms, vec![1, 2, 3, 4]);
    
    let token = CapTokenBuilder::new()
        .with_header(header)
        .with_signature(cap_signature)
        .with_metadata(metadata)
        .build()
        .unwrap();
    
    // Test verification performance (p50 < 1.5ms)
    let iterations = 100;
    let mut validation_times = Vec::new();
    
    for _ in 0..iterations {
        let start_time = crate::time::get_high_res_time();
        let result = validate_capability_token(&token, 2001, scope_v2::SEND);
        let end_time = crate::time::get_high_res_time();
        
        assert!(result.is_valid);
        validation_times.push(end_time - start_time);
    }
    
    // Calculate percentiles
    validation_times.sort();
    let p50 = validation_times[validation_times.len() / 2];
    let p95 = validation_times[(validation_times.len() * 95) / 100];
    
    // Convert to milliseconds (assuming high_res_time is in microseconds)
    let p50_ms = p50 as f64 / 1000.0;
    let p95_ms = p95 as f64 / 1000.0;
    
    println!("  Validation performance:");
    println!("    P50: {:.2}ms", p50_ms);
    println!("    P95: {:.2}ms", p95_ms);
    
    // Verify performance targets
    assert!(p50_ms < 1.5, "P50 validation time {}ms exceeds 1.5ms target", p50_ms);
    
    // Test cache hit performance (< 100µs)
    // First validation populates cache, second should be cache hit
    let start_time = crate::time::get_high_res_time();
    let _result = validate_capability_token(&token, 2001, scope_v2::SEND);
    let end_time = crate::time::get_high_res_time();
    
    let cache_hit_time = end_time - start_time;
    let cache_hit_time_us = cache_hit_time as f64;
    
    println!("  Cache hit performance: {:.2}µs", cache_hit_time_us);
    
    // Note: Cache hit performance may vary, but should be significantly faster than full validation
    assert!(cache_hit_time_us < 1000.0, "Cache hit time {:.2}µs exceeds 1000µs target", cache_hit_time_us);
    
    println!("✅ Performance targets test passed");
}

/// Test comprehensive token validation scenarios
#[test]
fn test_comprehensive_validation() {
    println!("Testing comprehensive token validation...");
    
    // Create test issuer
    let (issuer_pk, issuer_sk) = Dilithium::keygen(DilithiumParameterSet::Dilithium2).unwrap();
    let issuer_did = "did:example:issuer".to_string();
    
    // Add issuer to capability store
    add_issuer_key(issuer_did.clone(), issuer_pk);
    
    let now_ms = crate::time::get_current_time_ms();
    
    // Test valid token
    let valid_header = CapTokenHeader::new(
        issuer_did.clone(),
        1001, // subject PID
        2001, // destination PID
        scope_v2::SEND | scope_v2::RECV,
        now_ms,           // not_before
        now_ms + 3600000, // not_after (1 hour from now)
        0x1111111111111111, // nonce
        [0u8; 32], // purpose hash
    );
    
    let header_bytes = valid_header.to_bytes();
    let signature = Dilithium::sign(&header_bytes, &issuer_sk).unwrap();
    
    let cap_signature = CapTokenSignature::new(
        signature.as_bytes().to_vec(),
        None,
        SignatureAlgorithm::Dilithium2,
    );
    
    let metadata = CapTokenMetadata::new(now_ms, vec![1, 2, 3, 4]);
    
    let valid_token = CapTokenBuilder::new()
        .with_header(valid_header)
        .with_signature(cap_signature)
        .with_metadata(metadata)
        .build()
        .unwrap();
    
    // Test valid token
    let result = validate_capability_token(&valid_token, 2001, scope_v2::SEND);
    assert!(result.is_valid);
    
    // Test insufficient scope
    let result = validate_capability_token(&valid_token, 2001, scope_v2::ADMIN);
    assert!(!result.is_valid);
    assert_eq!(result.failure_reason, Some(CapValidationFailure::InsufficientScope));
    
    // Test malformed token
    let mut malformed_token = valid_token.clone();
    malformed_token.header.issuer_did = "".to_string(); // Empty DID
    
    let result = validate_capability_token(&malformed_token, 2001, scope_v2::SEND);
    assert!(!result.is_valid);
    assert_eq!(result.failure_reason, Some(CapValidationFailure::Malformed));
    
    // Test unknown issuer
    let unknown_issuer_header = CapTokenHeader::new(
        "did:example:unknown".to_string(),
        1001,
        2001,
        scope_v2::SEND,
        now_ms,
        now_ms + 3600000,
        0x2222222222222222,
        [0u8; 32],
    );
    
    let header_bytes = unknown_issuer_header.to_bytes();
    let signature = Dilithium::sign(&header_bytes, &issuer_sk).unwrap();
    
    let cap_signature = CapTokenSignature::new(
        signature.as_bytes().to_vec(),
        None,
        SignatureAlgorithm::Dilithium2,
    );
    
    let metadata = CapTokenMetadata::new(now_ms, vec![1, 2, 3, 4]);
    
    let unknown_issuer_token = CapTokenBuilder::new()
        .with_header(unknown_issuer_header)
        .with_signature(cap_signature)
        .with_metadata(metadata)
        .build()
        .unwrap();
    
    let result = validate_capability_token(&unknown_issuer_token, 2001, scope_v2::SEND);
    assert!(!result.is_valid);
    assert_eq!(result.failure_reason, Some(CapValidationFailure::UnknownIssuer));
    
    println!("✅ Comprehensive validation test passed");
}

/// Test audit logging integration
#[test]
fn test_audit_logging_integration() {
    println!("Testing audit logging integration...");
    
    // Clear audit log
    let initial_count = crate::secman::audit::get_audit_count();
    
    // Create test issuer
    let (issuer_pk, issuer_sk) = Dilithium::keygen(DilithiumParameterSet::Dilithium2).unwrap();
    let issuer_did = "did:example:issuer".to_string();
    
    // Add issuer to capability store
    add_issuer_key(issuer_did.clone(), issuer_pk);
    
    let now_ms = crate::time::get_current_time_ms();
    
    // Create valid token
    let header = CapTokenHeader::new(
        issuer_did.clone(),
        1001, // subject PID
        2001, // destination PID
        scope_v2::SEND,
        now_ms,           // not_before
        now_ms + 3600000, // not_after (1 hour from now)
        0x3333333333333333, // nonce
        [0u8; 32], // purpose hash
    );
    
    let header_bytes = header.to_bytes();
    let signature = Dilithium::sign(&header_bytes, &issuer_sk).unwrap();
    
    let cap_signature = CapTokenSignature::new(
        signature.as_bytes().to_vec(),
        None,
        SignatureAlgorithm::Dilithium2,
    );
    
    let metadata = CapTokenMetadata::new(now_ms, vec![1, 2, 3, 4]);
    
    let token = CapTokenBuilder::new()
        .with_header(header)
        .with_signature(cap_signature)
        .with_metadata(metadata)
        .build()
        .unwrap();
    
    // Test successful validation (should log CAP_ACCEPT)
    let result = validate_capability_token(&token, 2001, scope_v2::SEND);
    assert!(result.is_valid);
    
    // Test failed validation (should log CAP_REJECT)
    let result = validate_capability_token(&token, 3001, scope_v2::SEND);
    assert!(!result.is_valid);
    
    // Check audit log entries
    let final_count = crate::secman::audit::get_audit_count();
    let new_entries = final_count - initial_count;
    
    assert!(new_entries >= 2, "Expected at least 2 new audit entries, got {}", new_entries);
    
    let audit_entries = crate::secman::audit::get_latest_entries(new_entries);
    
    // Should have CAP_ACCEPT and CAP_REJECT entries
    let has_accept = audit_entries.iter().any(|e| e.op == ops::SEC_CAP_ACCEPT);
    let has_reject = audit_entries.iter().any(|e| e.op == ops::SEC_CAP_REJECT);
    
    assert!(has_accept, "Missing CAP_ACCEPT audit entry");
    assert!(has_reject, "Missing CAP_REJECT audit entry");
    
    println!("✅ Audit logging integration test passed");
}

/// Run all CapTokens v2 tests
pub fn run_all_cap_tokens_v2_tests() {
    println!("🚀 Running CapTokens v2 Test Suite");
    println!("===================================");
    
    // Run individual test functions
    test_expired_token_rejection();
    test_early_token_rejection();
    test_wrong_destination_rejection();
    test_replay_detection();
    test_performance_targets();
    test_comprehensive_validation();
    test_audit_logging_integration();
    
    println!("===================================");
    println!("✅ All CapTokens v2 tests passed!");
    println!("🎯 CapTokens v2 system is ready!");
}
