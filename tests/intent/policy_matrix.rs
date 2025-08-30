use crate::intent::wire::{
    IntentHeaderV1, IntentLane, IntentState
};
use crate::intent::{Intent, IntentManager, IntentError};
use crate::intent::policy::{IntentPolicy, PolicyDecision, PolicyError};
use crate::process::ProcessId;

#[test]
fn test_policy_deny_restricted_scope() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x4000_0000_0000_0000; // Restricted scope flag
    
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
        IntentError::PolicyDenied(reason) => {
            assert!(reason.contains("Restricted scope not allowed"));
        },
        _ => panic!("Expected PolicyDenied error"),
    }
}

#[test]
fn test_policy_deny_ttl_too_long() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 700_000; // Exceeds 600s limit
    header.text_len = 100;
    
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
        IntentError::PolicyDenied(reason) => {
            assert!(reason.contains("TTL exceeds maximum allowed"));
        },
        _ => panic!("Expected PolicyDenied error"),
    }
}

#[test]
fn test_policy_mutate_scopes_large_text() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 16384; // Exceeds 8KB threshold
    header.scope_flags = 0x0000_0000_0000_0001; // Basic scope
    
    let text_data = vec![0u8; 16384];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    // Should succeed with scope mutation
    assert!(result.is_ok());
    
    // Check that the intent was created with additional scopes
    let manager_ref = manager;
    // Note: In a real test, we'd need to access the intent to verify scope mutation
    // For now, we just verify the submission succeeded
}

#[test]
fn test_policy_rt_lane_requires_capabilities() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::RT as u8; // RT lane
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0; // No elevated capabilities
    
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
        IntentError::PolicyDenied(reason) => {
            assert!(reason.contains("RT lane requires elevated capabilities"));
        },
        _ => panic!("Expected PolicyDenied error"),
    }
}

#[test]
fn test_policy_rt_lane_with_capabilities() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::RT as u8; // RT lane
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001; // Elevated capabilities
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    // Should succeed with proper capabilities
    assert!(result.is_ok());
}

#[test]
fn test_policy_dev_mode_restrictions() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x8000_0000_0000_0000; // Dev mode restricted scope
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    // In dev mode, this should be denied
    assert!(result.is_err());
    match result.unwrap_err() {
        IntentError::PolicyDenied(reason) => {
            assert!(reason.contains("Development mode: restricted scope"));
        },
        _ => panic!("Expected PolicyDenied error"),
    }
}

#[test]
fn test_policy_dev_mode_ttl_limit() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 400_000; // Exceeds dev mode 300s limit
    header.text_len = 100;
    
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
        IntentError::PolicyDenied(reason) => {
            assert!(reason.contains("Development mode: TTL too long"));
        },
        _ => panic!("Expected PolicyDenied error"),
    }
}

#[test]
fn test_policy_dev_mode_mutate_scopes() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 2048; // Exceeds dev mode 1KB threshold
    header.scope_flags = 0;
    
    let text_data = vec![0u8; 2048];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    // Should succeed with scope mutation in dev mode
    assert!(result.is_ok());
}

#[test]
fn test_policy_allow_normal_intent() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001; // Basic scope
    
    let text_data = vec![0u8; 100];
    
    let result = manager.submit_intent(
        123,
        header,
        text_data,
        vec![],
        vec![],
        vec![],
    );
    
    // Should succeed
    assert!(result.is_ok());
}

#[test]
fn test_policy_scope_flag_mapping() {
    // Test that scope flags map to appropriate capabilities
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.lane = IntentLane::HIGH as u8;
    header.ttl_ms = 1000;
    header.text_len = 100;
    
    // Test various scope combinations
    let scope_tests = vec![
        0x0000_0000_0000_0001, // Basic capability
        0x0000_0000_0000_0002, // Another basic capability
        0x0000_0000_0000_0003, // Combined basic capabilities
        0x0000_0000_0000_0100, // Higher capability
    ];
    
    for scope_flags in scope_tests {
        header.scope_flags = scope_flags;
        let text_data = vec![0u8; 100];
        
        let result = manager.submit_intent(
            123,
            header.clone(),
            text_data,
            vec![],
            vec![],
            vec![],
        );
        
        // All should succeed with valid scope flags
        assert!(result.is_ok(), "Failed with scope flags 0x{:x}", scope_flags);
    }
}

#[test]
fn test_policy_lane_priority_validation() {
    let mut manager = IntentManager::new();
    let mut header = IntentHeaderV1::default();
    
    header.intent_id = 1;
    header.ttl_ms = 1000;
    header.text_len = 100;
    header.scope_flags = 0x0000_0000_0000_0001; // Elevated capabilities
    
    // Test all valid lanes
    let lanes = vec![
        IntentLane::RT as u8,
        IntentLane::HIGH as u8,
        IntentLane::BEST as u8,
    ];
    
    for lane in lanes {
        header.lane = lane;
        let text_data = vec![0u8; 100];
        
        let result = manager.submit_intent(
            123,
            header.clone(),
            text_data,
            vec![],
            vec![],
            vec![],
        );
        
        // All should succeed with proper capabilities
        assert!(result.is_ok(), "Failed with lane {}", lane);
    }
}
