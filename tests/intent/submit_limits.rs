use crate::intent::wire::{
    IntentHeaderV1, IntentLane, IntentState
};
use crate::intent::{Intent, IntentManager, IntentError};
use crate::process::ProcessId;

#[test]
fn test_intent_oversize_text() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 70000; // Exceeds 64KB limit
    
    let text_data = vec![0u8; 70000];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::InvalidInput => {},
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_intent_oversize_plan() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.plan_len = 50000; // Exceeds 48KB limit
    
    let text_data = vec![0u8; 100];
    let plan_data = vec![0u8; 50000];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        plan_data,
        vec![],
        vec![],
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::InvalidInput => {},
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_intent_oversize_preview() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.preview_len = 20000; // Exceeds 16KB limit
    
    let text_data = vec![0u8; 100];
    let preview_data = vec![0u8; 20000];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        preview_data,
        vec![],
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::InvalidInput => {},
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_intent_oversize_whylog() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.whylog_len = 10000; // Exceeds 8KB limit
    
    let text_data = vec![0u8; 100];
    let whylog_data = vec![0u8; 10000];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        whylog_data,
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::InvalidInput => {},
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_intent_ttl_zero() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 0; // Invalid TTL
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::InvalidInput => {},
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_intent_invalid_lane() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = 3; // Invalid lane (only 0, 1, 2 are valid)
    header.ttl_ms = 1000;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::InvalidInput => {},
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_intent_invalid_version() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.version = 2; // Invalid version (only 1 is valid)
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::InvalidInput => {},
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_intent_valid_limits() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 65536; // Exactly at limit
    header.plan_len = 49152; // Exactly at limit
    header.preview_len = 16384; // Exactly at limit
    header.whylog_len = 8192; // Exactly at limit
    
    let text_data = vec![0u8; 65536];
    let plan_data = vec![0u8; 49152];
    let preview_data = vec![0u8; 16384];
    let whylog_data = vec![0u8; 8192];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        plan_data,
        preview_data,
        whylog_data,
    );
    
    // This should succeed in dev mode, but may fail due to policy
    // We just check that it doesn't fail with InvalidInput
    match result {
        Ok(()) => {},
        Err(IntentError::InvalidInput) => panic!("Should not fail with InvalidInput"),
        Err(_) => {}, // Other errors are acceptable
    }
}

#[test]
fn test_intent_total_payload_size() {
    let mut header = IntentHeaderV1::default();
    
    header.text_len = 1000;
    header.plan_len = 2000;
    header.preview_len = 500;
    header.whylog_len = 250;
    
    let total_size = header.total_payload_size();
    assert_eq!(total_size, 3750);
}

#[test]
fn test_intent_header_validation() {
    let mut header = IntentHeaderV1::default();
    
    // Valid header
    header.version = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    
    assert!(header.is_valid());
    
    // Invalid version
    header.version = 2;
    assert!(!header.is_valid());
    
    // Invalid lane
    header.version = 1;
    header.lane = 3;
    assert!(!header.is_valid());
    
    // Invalid TTL
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 0;
    assert!(!header.is_valid());
    
    // Text too long
    header.ttl_ms = 1000;
    header.text_len = 70000;
    assert!(!header.is_valid());
    
    // Plan too long
    header.text_len = 100;
    header.plan_len = 50000;
    assert!(!header.is_valid());
    
    // Preview too long
    header.plan_len = 100;
    header.preview_len = 20000;
    assert!(!header.is_valid());
    
    // WhyLog too long
    header.preview_len = 100;
    header.whylog_len = 10000;
    assert!(!header.is_valid());
}
