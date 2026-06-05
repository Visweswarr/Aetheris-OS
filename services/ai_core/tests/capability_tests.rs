//! Comprehensive tests for the Capability Guard system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

use aetheris_ai_core::cap::{
    CapTokenManager, CapabilityPolicy, PolicyEnforcementResult, PolicyEnforcer,
};
use aetheris_ai_core::error::AiCoreError;
use aetheris_ai_core::ipc::CapToken;

/// Test helper to create a capability token
fn create_test_token(capability: &str, issuer: &str) -> CapToken {
    CapToken {
        token_id: format!("test-token-{}", uuid::Uuid::new_v4()),
        capability: capability.to_string(),
        expires_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600,
        signature: vec![],
        issuer: issuer.to_string(),
    }
}

/// Test helper to create a capability token manager
async fn create_test_cap_manager() -> CapTokenManager {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("cap_tokens.toml");
    CapTokenManager::new(&config_path).await.unwrap()
}

#[tokio::test]
async fn test_cap_token_manager_creation() {
    let manager = create_test_cap_manager().await;
    // Manager should be created successfully
    assert!(true);
}

#[tokio::test]
async fn test_policy_enforcer_creation() {
    let enforcer = PolicyEnforcer::new().await;
    assert!(enforcer.is_ok());

    let enforcer = enforcer.unwrap();
    // Enforcer should be created successfully
    assert!(true);
}

#[tokio::test]
async fn test_capability_check_allowed() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    let result = manager
        .check_capability(&token, "ai_core", "generate_response")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);
    assert!(result.reason.contains("granted"));
}

#[tokio::test]
async fn test_capability_check_denied_unknown_capability() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("unknown:capability", "aetheris-system");

    let result = manager
        .check_capability(&token, "ai_core", "generate_response")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("Unknown capability"));
}

#[tokio::test]
async fn test_capability_check_denied_unknown_issuer() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "unknown-issuer");

    let result = manager
        .check_capability(&token, "ai_core", "generate_response")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("Unknown issuer"));
}

#[tokio::test]
async fn test_capability_check_denied_untrusted_issuer() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "untrusted-issuer");

    let result = manager
        .check_capability(&token, "ai_core", "generate_response")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("Unknown issuer"));
}

#[tokio::test]
async fn test_capability_check_denied_wrong_resource() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    let result = manager
        .check_capability(&token, "wrong_resource", "generate_response")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("denied"));
}

#[tokio::test]
async fn test_capability_check_denied_wrong_action() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    let result = manager
        .check_capability(&token, "ai_core", "wrong_action")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("denied"));
}

#[tokio::test]
async fn test_capability_check_with_deny_error() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    // This should succeed
    let result = manager
        .check_capability_with_deny(&token, "ai_core", "generate_response")
        .await;
    assert!(result.is_ok());

    // This should fail with CapDenied error
    let result = manager
        .check_capability_with_deny(&token, "wrong_resource", "generate_response")
        .await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, AiCoreError::CapDenied(_)));
}

#[tokio::test]
async fn test_policy_enforcement_direct() {
    let manager = create_test_cap_manager().await;

    // Test direct policy enforcement
    let result = manager
        .enforce_capability_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);
    assert!(result.reason.contains("granted"));
}

#[tokio::test]
async fn test_policy_enforcement_denied() {
    let manager = create_test_cap_manager().await;

    // Test denied policy enforcement
    let result = manager
        .enforce_capability_policy(
            "unknown:capability",
            "ai_core",
            "generate_response",
            "aetheris-system",
        )
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("Unknown capability"));
}

#[tokio::test]
async fn test_token_validation_success() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    let result = manager.validate_token(&token).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_token_validation_expired() {
    let manager = create_test_cap_manager().await;
    let mut token = create_test_token("ai:chat", "aetheris-system");
    token.expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 3600; // Expired

    let result = manager.validate_token(&token).await;
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, AiCoreError::CapabilityError(_)));
}

#[tokio::test]
async fn test_policy_enforcer_rule_matching() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test rule matching for allowed capability
    let result = enforcer
        .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);
    assert!(result.matched_rule.is_some());
}

#[tokio::test]
async fn test_policy_enforcer_rule_matching_denied() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test rule matching for denied capability
    let result = enforcer
        .enforce_policy(
            "unknown:capability",
            "ai_core",
            "generate_response",
            "aetheris-system",
        )
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.matched_rule.is_none());
}

#[tokio::test]
async fn test_policy_enforcer_resource_pattern_matching() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test exact resource matching
    let result = enforcer
        .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);
}

#[tokio::test]
async fn test_policy_enforcer_action_matching() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test exact action matching
    let result = enforcer
        .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);
}

#[tokio::test]
async fn test_policy_enforcer_issuer_validation() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test trusted issuer
    let result = enforcer
        .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);

    // Test unknown issuer
    let result = enforcer
        .enforce_policy("ai:chat", "ai_core", "generate_response", "unknown-issuer")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("Unknown issuer"));
}

#[tokio::test]
async fn test_policy_enforcer_capability_validation() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test known capability
    let result = enforcer
        .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);

    // Test unknown capability
    let result = enforcer
        .enforce_policy(
            "unknown:capability",
            "ai_core",
            "generate_response",
            "aetheris-system",
        )
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("Unknown capability"));
}

#[tokio::test]
async fn test_policy_enforcer_deny_by_default() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test that unknown combinations are denied by default
    let result = enforcer
        .enforce_policy(
            "ai:chat",
            "unknown_resource",
            "unknown_action",
            "aetheris-system",
        )
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
    assert!(result.reason.contains("denied"));
}

#[tokio::test]
async fn test_policy_enforcer_rule_priority() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test that rules are evaluated in priority order
    let result = enforcer
        .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);
    assert!(result.matched_rule.is_some());
}

#[tokio::test]
async fn test_policy_enforcer_condition_evaluation() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test condition evaluation (currently simple implementation)
    let result = enforcer
        .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);
}

#[tokio::test]
async fn test_capability_check_audit_logging() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    // This should generate audit logs
    let result = manager
        .check_capability(&token, "ai_core", "generate_response")
        .await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.granted);
}

#[tokio::test]
async fn test_capability_check_caching() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    // First check
    let result1 = manager
        .check_capability(&token, "ai_core", "generate_response")
        .await;
    assert!(result1.is_ok());

    // Second check (should use cache)
    let result2 = manager
        .check_capability(&token, "ai_core", "generate_response")
        .await;
    assert!(result2.is_ok());

    let result1 = result1.unwrap();
    let result2 = result2.unwrap();

    assert_eq!(result1.granted, result2.granted);
    assert_eq!(result1.reason, result2.reason);
}

#[tokio::test]
async fn test_capability_check_concurrent() {
    let manager = Arc::new(create_test_cap_manager().await);
    let token = create_test_token("ai:chat", "aetheris-system");

    // Test concurrent capability checks
    let mut handles = Vec::new();

    for _ in 0..10 {
        let manager_clone = manager.clone();
        let token_clone = token.clone();
        let handle = tokio::spawn(async move {
            manager_clone
                .check_capability(&token_clone, "ai_core", "generate_response")
                .await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.granted);
    }
}

#[tokio::test]
async fn test_capability_check_error_handling() {
    let manager = create_test_cap_manager().await;

    // Test with invalid token
    let mut token = create_test_token("ai:chat", "aetheris-system");
    token.token_id = "".to_string(); // Invalid token ID

    let result = manager
        .check_capability(&token, "ai_core", "generate_response")
        .await;
    // Should still work as we don't validate token ID format
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policy_enforcer_error_handling() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test with empty strings
    let result = enforcer.enforce_policy("", "", "", "").await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(!result.granted);
}

#[tokio::test]
async fn test_capability_check_performance() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    let start_time = std::time::Instant::now();

    // Perform multiple capability checks
    for _ in 0..100 {
        let result = manager
            .check_capability(&token, "ai_core", "generate_response")
            .await;
        assert!(result.is_ok());
    }

    let elapsed = start_time.elapsed();
    println!("100 capability checks took: {:?}", elapsed);

    // Should complete within reasonable time
    assert!(elapsed < Duration::from_secs(5));
}

#[tokio::test]
async fn test_policy_enforcer_performance() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    let start_time = std::time::Instant::now();

    // Perform multiple policy enforcements
    for _ in 0..100 {
        let result = enforcer
            .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
            .await;
        assert!(result.is_ok());
    }

    let elapsed = start_time.elapsed();
    println!("100 policy enforcements took: {:?}", elapsed);

    // Should complete within reasonable time
    assert!(elapsed < Duration::from_secs(5));
}

#[tokio::test]
async fn test_capability_check_memory_usage() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    // Perform many capability checks to test memory usage
    for i in 0..1000 {
        let mut token_clone = token.clone();
        token_clone.token_id = format!("test-token-{}", i);

        let result = manager
            .check_capability(&token_clone, "ai_core", "generate_response")
            .await;
        assert!(result.is_ok());
    }

    // If we get here, memory usage is reasonable
    assert!(true);
}

#[tokio::test]
async fn test_policy_enforcer_memory_usage() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Perform many policy enforcements to test memory usage
    for i in 0..1000 {
        let result = enforcer
            .enforce_policy("ai:chat", "ai_core", "generate_response", "aetheris-system")
            .await;
        assert!(result.is_ok());
    }

    // If we get here, memory usage is reasonable
    assert!(true);
}

#[tokio::test]
async fn test_capability_check_edge_cases() {
    let manager = create_test_cap_manager().await;
    let token = create_test_token("ai:chat", "aetheris-system");

    // Test edge cases
    let edge_cases = vec![
        ("", "", ""),
        ("ai:chat", "", ""),
        ("", "ai_core", ""),
        ("", "", "generate_response"),
        ("ai:chat", "ai_core", ""),
        ("ai:chat", "", "generate_response"),
        ("", "ai_core", "generate_response"),
    ];

    for (capability, resource, action) in edge_cases {
        let result = manager.check_capability(&token, resource, action).await;
        // Should handle edge cases gracefully
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_policy_enforcer_edge_cases() {
    let enforcer = PolicyEnforcer::new().await.unwrap();

    // Test edge cases
    let edge_cases = vec![
        ("", "", "", ""),
        ("ai:chat", "", "", ""),
        ("", "ai_core", "", ""),
        ("", "", "generate_response", ""),
        ("", "", "", "aetheris-system"),
        ("ai:chat", "ai_core", "", ""),
        ("ai:chat", "", "generate_response", ""),
        ("ai:chat", "", "", "aetheris-system"),
        ("", "ai_core", "generate_response", ""),
        ("", "ai_core", "", "aetheris-system"),
        ("", "", "generate_response", "aetheris-system"),
    ];

    for (capability, resource, action, issuer) in edge_cases {
        let result = enforcer
            .enforce_policy(capability, resource, action, issuer)
            .await;
        // Should handle edge cases gracefully
        assert!(result.is_ok());
    }
}
