//! Unit tests for XR Scene Service modules
//! 
//! This module contains focused unit tests for individual components
//! of the XR scene service.

use std::collections::HashMap;
use std::time::Duration;

use crate::{
    SessionManager, SessionId, BeginSessionRequest, EndSessionRequest, DidProof,
    CapManager, CapToken, CapScope, CapAction, Permission,
    PolicyManager, PolicyBundle, PolicyRule, PolicyScope, PolicyAction, PolicyEffect,
    DaoHookFactory, MockDaoClient, SceneProposal, ProposalStatus, ProposalVote, VoteChoice,
    NodeId, Transform, Component, ComponentType,
};

#[tokio::test]
async fn test_session_manager_lifecycle() {
    let manager = SessionManager::new(Duration::from_secs(60), 3);
    
    let proof = DidProof {
        did: "did:aeth:test123".to_string(),
        proof: "jwt_proof_here".to_string(),
        nonce: "nonce123".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let request = BeginSessionRequest {
        proof,
        avatar_profile: None,
        ttl_seconds: Some(60),
    };

    // Begin session
    let response = manager.begin_session(request).unwrap();
    assert!(!response.session_id.0.is_empty());
    assert!(response.avatar_id.is_none());

    // Validate session
    let validation = manager.validate_session(&response.session_id).unwrap();
    assert!(validation.is_valid);
    assert!(validation.session.is_some());

    // End session
    let end_request = EndSessionRequest {
        session_id: response.session_id.clone(),
        reason: Some("Test completion".to_string()),
    };

    let end_response = manager.end_session(end_request).unwrap();
    assert!(end_response.success);

    // Validate session is no longer valid
    let validation = manager.validate_session(&response.session_id).unwrap();
    assert!(!validation.is_valid);
}

#[tokio::test]
async fn test_session_manager_limits() {
    let manager = SessionManager::new(Duration::from_secs(60), 2);
    
    let did = "did:aeth:test456";
    
    // Create first session
    let proof1 = DidProof {
        did: did.to_string(),
        proof: "jwt_proof_1".to_string(),
        nonce: "nonce1".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let request1 = BeginSessionRequest {
        proof: proof1,
        avatar_profile: None,
        ttl_seconds: Some(60),
    };

    let response1 = manager.begin_session(request1).unwrap();
    assert!(!response1.session_id.0.is_empty());

    // Create second session
    let proof2 = DidProof {
        did: did.to_string(),
        proof: "jwt_proof_2".to_string(),
        nonce: "nonce2".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let request2 = BeginSessionRequest {
        proof: proof2,
        avatar_profile: None,
        ttl_seconds: Some(60),
    };

    let response2 = manager.begin_session(request2).unwrap();
    assert!(!response2.session_id.0.is_empty());

    // Try to create third session (should fail)
    let proof3 = DidProof {
        did: did.to_string(),
        proof: "jwt_proof_3".to_string(),
        nonce: "nonce3".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let request3 = BeginSessionRequest {
        proof: proof3,
        avatar_profile: None,
        ttl_seconds: Some(60),
    };

    let result = manager.begin_session(request3);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_capability_manager_operations() {
    let manager = CapManager::new();
    
    // Create token
    let token_id = manager.create_token("test_token".to_string(), "test_user".to_string()).await.unwrap();
    assert_eq!(manager.get_token_count().await, 1);
    
    // Get token
    let token = manager.get_token(&token_id).await.unwrap();
    assert_eq!(token.name, "test_token");
    
    // Update token
    let permission = Permission::new(CapAction::Spawn, CapScope::Scene);
    manager.update_token(&token_id, crate::cap::CapTokenUpdate::AddPermission(permission)).await.unwrap();
    
    let updated_token = manager.get_token(&token_id).await.unwrap();
    assert_eq!(updated_token.permissions.len(), 1);
    
    // Assign token to user
    manager.assign_token_to_user(&token_id, "did:aeth:test").await.unwrap();
    
    let user_tokens = manager.get_user_tokens("did:aeth:test").await;
    assert_eq!(user_tokens.len(), 1);
    
    // Check permission
    let has_permission = manager.check_permission("did:aeth:test", &CapAction::Spawn, &CapScope::Scene).await.unwrap();
    assert!(has_permission);
    
    // Delete token
    manager.delete_token(&token_id).await.unwrap();
    assert_eq!(manager.get_token_count().await, 0);
}

#[tokio::test]
async fn test_capability_token_validation() {
    let mut token = CapToken::new("test_token".to_string());
    
    // Valid token
    assert!(token.validate().is_ok());
    
    // Invalid name
    token.name = "".to_string();
    assert!(token.validate().is_err());
    
    // Invalid creator
    token.name = "test_token".to_string();
    token.created_by = "".to_string();
    assert!(token.validate().is_err());
    
    // Invalid version
    token.created_by = "test_user".to_string();
    token.version = 0;
    assert!(token.validate().is_err());
}

#[tokio::test]
async fn test_capability_token_expiration() {
    let mut token = CapToken::new("test_token".to_string());
    
    // Token without expiration
    assert!(!token.is_expired());
    
    // Token with future expiration
    token.set_expiration(chrono::Utc::now() + chrono::Duration::hours(1));
    assert!(!token.is_expired());
    
    // Token with past expiration
    token.set_expiration(chrono::Utc::now() - chrono::Duration::hours(1));
    assert!(token.is_expired());
}

#[tokio::test]
async fn test_capability_token_permissions() {
    let mut token = CapToken::new("test_token".to_string());
    
    let permission = Permission::new(CapAction::Spawn, CapScope::Scene);
    token.add_permission(permission.clone());
    
    assert!(token.has_permission(&permission));
    assert!(token.has_action_permission(&CapAction::Spawn));
    assert!(!token.has_action_permission(&CapAction::Delete));
}

#[tokio::test]
async fn test_capability_token_serialization() {
    let token = CapToken::new("test_token".to_string());
    
    let cbor_data = token.to_cbor().unwrap();
    let deserialized = CapToken::from_cbor(&cbor_data).unwrap();
    
    assert_eq!(token.name, deserialized.name);
    assert_eq!(token.scope, deserialized.scope);
    assert_eq!(token.version, deserialized.version);
}

#[tokio::test]
async fn test_policy_manager_operations() {
    let manager = PolicyManager::new();
    
    // Create and add bundle
    let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
    let rule = PolicyRule::new(
        "test_rule".to_string(),
        PolicyScope::Scene,
        PolicyAction::Spawn,
        PolicyEffect::Allow,
        "test_user".to_string(),
    );
    bundle.add_rule(rule);
    
    manager.add_bundle(bundle.clone()).await.unwrap();
    
    // Get bundle
    let retrieved_bundle = manager.get_bundle(&bundle.id).await.unwrap();
    assert_eq!(retrieved_bundle.name, bundle.name);
    
    // Activate bundle
    manager.activate_bundle(&bundle.id).await.unwrap();
    
    // Get active bundles
    let active_bundles = manager.get_active_bundles().await;
    assert_eq!(active_bundles.len(), 1);
    
    // Deactivate bundle
    manager.deactivate_bundle(&bundle.id).await.unwrap();
    
    // Get active bundles
    let active_bundles = manager.get_active_bundles().await;
    assert_eq!(active_bundles.len(), 0);
    
    // Remove bundle
    manager.remove_bundle(&bundle.id).await.unwrap();
}

#[tokio::test]
async fn test_policy_evaluation() {
    let manager = PolicyManager::new();
    
    // Create and add bundle with allow rule
    let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
    let rule = PolicyRule::new(
        "allow_spawn".to_string(),
        PolicyScope::Scene,
        PolicyAction::Spawn,
        PolicyEffect::Allow,
        "test_user".to_string(),
    );
    bundle.add_rule(rule);
    
    manager.add_bundle(bundle.clone()).await.unwrap();
    manager.activate_bundle(&bundle.id).await.unwrap();
    
    // Create context
    let context = crate::policy::PolicyContext::new(
        "did:aeth:test".to_string(),
        PolicyAction::Spawn,
        PolicyScope::Scene,
    );
    
    // Evaluate policy
    let result = manager.evaluate(context).await.unwrap();
    assert!(result.allowed);
}

#[tokio::test]
async fn test_policy_evaluation_deny() {
    let manager = PolicyManager::new();
    
    // Create and add bundle with deny rule
    let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
    let rule = PolicyRule::new(
        "deny_spawn".to_string(),
        PolicyScope::Scene,
        PolicyAction::Spawn,
        PolicyEffect::Deny,
        "test_user".to_string(),
    );
    bundle.add_rule(rule);
    
    manager.add_bundle(bundle.clone()).await.unwrap();
    manager.activate_bundle(&bundle.id).await.unwrap();
    
    // Create context
    let context = crate::policy::PolicyContext::new(
        "did:aeth:test".to_string(),
        PolicyAction::Spawn,
        PolicyScope::Scene,
    );
    
    // Evaluate policy
    let result = manager.evaluate(context).await.unwrap();
    assert!(!result.allowed);
    assert!(result.reason.is_some());
}

#[tokio::test]
async fn test_policy_bundle_validation() {
    let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
    
    // Valid bundle
    assert!(bundle.validate().is_ok());

    // Invalid name
    bundle.name = "".to_string();
    assert!(bundle.validate().is_err());

    // Invalid creator
    bundle.name = "test_bundle".to_string();
    bundle.created_by = "".to_string();
    assert!(bundle.validate().is_err());

    // Invalid version
    bundle.created_by = "test_user".to_string();
    bundle.version = 0;
    assert!(bundle.validate().is_err());
}

#[tokio::test]
async fn test_policy_bundle_serialization() {
    let bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
    
    let cbor_data = bundle.to_cbor().unwrap();
    let deserialized = PolicyBundle::from_cbor(&cbor_data).unwrap();
    
    assert_eq!(bundle.name, deserialized.name);
    assert_eq!(bundle.created_by, deserialized.created_by);
    assert_eq!(bundle.version, deserialized.version);
}

#[tokio::test]
async fn test_dao_hook_with_mock_client() {
    let hook = DaoHookFactory::create_mock_hook();
    
    let context = crate::policy::PolicyContext::new(
        "did:aeth:test".to_string(),
        PolicyAction::Spawn,
        PolicyScope::Scene,
    );
    
    let result = hook.check_policy(&context).await.unwrap();
    assert!(result.allowed);
}

#[tokio::test]
async fn test_dao_hook_caching() {
    let hook = DaoHookFactory::create_mock_hook();
    
    let context = crate::policy::PolicyContext::new(
        "did:aeth:test".to_string(),
        PolicyAction::Spawn,
        PolicyScope::Scene,
    );
    
    // First call
    let result1 = hook.check_policy(&context).await.unwrap();
    assert!(result1.allowed);
    
    // Second call should use cache
    let result2 = hook.check_policy(&context).await.unwrap();
    assert!(result2.allowed);
    assert_eq!(result1.reason, result2.reason);
}

#[tokio::test]
async fn test_dao_hook_policy_management_deny() {
    let hook = DaoHookFactory::create_mock_hook();
    
    let context = crate::policy::PolicyContext::new(
        "did:aeth:test".to_string(),
        PolicyAction::AttachPolicy,
        PolicyScope::Scene,
    );
    
    let result = hook.check_policy(&context).await.unwrap();
    assert!(!result.allowed);
    assert!(result.reason.unwrap().contains("deny"));
}

#[tokio::test]
async fn test_mock_dao_client() {
    let client = MockDaoClient::new();
    
    // Set voting power
    client.set_voting_power("did:aeth:test".to_string(), 10.0).await;
    
    // Check voting power
    let power = client.get_voting_power("did:aeth:test").await.unwrap();
    assert_eq!(power, 10.0);
    
    // Check default permission
    let decision = client.check_permission(
        "did:aeth:test",
        &PolicyAction::Spawn,
        &PolicyScope::Scene,
    ).await.unwrap();
    
    assert!(decision.approved);
}

#[tokio::test]
async fn test_proposal_creation() {
    let client = MockDaoClient::new();
    
    let proposal = SceneProposal {
        id: uuid::Uuid::new_v4().to_string(),
        title: "Test Proposal".to_string(),
        description: "Test description".to_string(),
        action: PolicyAction::Spawn,
        scope: PolicyScope::Scene,
        proposer_did: "did:aeth:test".to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        expires_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() + 86400, // 24 hours
        status: ProposalStatus::Active,
        votes: Vec::new(),
        approval_threshold: 0.5,
    };
    
    let proposal_id = client.create_proposal(proposal).await.unwrap();
    assert!(!proposal_id.is_empty());
}

#[tokio::test]
async fn test_vote_creation() {
    let vote = ProposalVote {
        voter_did: "did:aeth:test".to_string(),
        choice: VoteChoice::Approve,
        weight: 10.0,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        justification: Some("Good proposal".to_string()),
    };
    
    assert_eq!(vote.voter_did, "did:aeth:test");
    assert_eq!(vote.choice, VoteChoice::Approve);
    assert_eq!(vote.weight, 10.0);
}

#[tokio::test]
async fn test_node_id_creation() {
    let node_id1 = NodeId::new();
    let node_id2 = NodeId::new();
    
    assert_ne!(node_id1.0, node_id2.0);
    assert!(!node_id1.0.is_empty());
    assert!(!node_id2.0.is_empty());
}

#[tokio::test]
async fn test_transform_operations() {
    let transform = Transform {
        position: [1.0, 2.0, 3.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: [2.0, 2.0, 2.0],
    };
    
    assert_eq!(transform.position, [1.0, 2.0, 3.0]);
    assert_eq!(transform.rotation, [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(transform.scale, [2.0, 2.0, 2.0]);
}

#[tokio::test]
async fn test_component_creation() {
    let component = Component {
        id: "test_component".to_string(),
        component_type: ComponentType::Mesh,
        data: serde_json::json!({
            "mesh_url": "test.glb",
            "material": "default"
        }),
    };
    
    assert_eq!(component.id, "test_component");
    assert_eq!(component.component_type, ComponentType::Mesh);
    assert!(component.data.is_object());
}

#[tokio::test]
async fn test_capability_scope_matching() {
    let scope1 = CapScope::Scene;
    let scope2 = CapScope::Node(NodeId::new());
    let scope3 = CapScope::Avatar("did:aeth:test".to_string());
    let scope4 = CapScope::User("did:aeth:test".to_string());
    
    assert_eq!(scope1, CapScope::Scene);
    assert_ne!(scope1, scope2);
    assert_ne!(scope2, scope3);
    assert_ne!(scope3, scope4);
}

#[tokio::test]
async fn test_capability_action_matching() {
    let action1 = CapAction::Spawn;
    let action2 = CapAction::Move;
    let action3 = CapAction::Delete;
    let action4 = CapAction::AttachPolicy;
    
    assert_eq!(action1, CapAction::Spawn);
    assert_ne!(action1, action2);
    assert_ne!(action2, action3);
    assert_ne!(action3, action4);
}

#[tokio::test]
async fn test_policy_effect_matching() {
    let effect1 = PolicyEffect::Allow;
    let effect2 = PolicyEffect::Deny;
    let effect3 = PolicyEffect::AllowWithConditions(vec!["condition1".to_string()]);
    
    assert_eq!(effect1, PolicyEffect::Allow);
    assert_eq!(effect2, PolicyEffect::Deny);
    assert_ne!(effect1, effect2);
    assert_ne!(effect2, effect3);
}

#[tokio::test]
async fn test_session_id_uniqueness() {
    let session_id1 = SessionId::new();
    let session_id2 = SessionId::new();
    
    assert_ne!(session_id1.0, session_id2.0);
    assert!(!session_id1.0.is_empty());
    assert!(!session_id2.0.is_empty());
}

#[tokio::test]
async fn test_did_proof_validation() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let proof = DidProof {
        did: "did:aeth:test".to_string(),
        proof: "jwt_proof".to_string(),
        nonce: "nonce123".to_string(),
        timestamp: now,
    };
    
    assert_eq!(proof.did, "did:aeth:test");
    assert_eq!(proof.proof, "jwt_proof");
    assert_eq!(proof.nonce, "nonce123");
    assert_eq!(proof.timestamp, now);
}

#[tokio::test]
async fn test_permission_creation() {
    let permission = Permission::new(CapAction::Spawn, CapScope::Scene);
    
    assert_eq!(permission.action, CapAction::Spawn);
    assert_eq!(permission.scope, CapScope::Scene);
    assert_eq!(permission.priority, 100);
    assert!(permission.conditions.is_empty());
}

#[tokio::test]
async fn test_permission_validation() {
    let mut permission = Permission::new(CapAction::Spawn, CapScope::Scene);
    
    // Valid permission
    assert!(permission.validate().is_ok());
    
    // Invalid priority
    permission.priority = 0;
    assert!(permission.validate().is_err());
}

#[tokio::test]
async fn test_policy_rule_creation() {
    let rule = PolicyRule::new(
        "test_rule".to_string(),
        PolicyScope::Scene,
        PolicyAction::Spawn,
        PolicyEffect::Allow,
        "test_user".to_string(),
    );

    assert_eq!(rule.name, "test_rule");
    assert_eq!(rule.scope, PolicyScope::Scene);
    assert_eq!(rule.action, PolicyAction::Spawn);
    assert_eq!(rule.effect, PolicyEffect::Allow);
    assert_eq!(rule.priority, 100);
    assert_eq!(rule.version, 1);
}

#[tokio::test]
async fn test_policy_rule_validation() {
    let mut rule = PolicyRule::new(
        "test_rule".to_string(),
        PolicyScope::Scene,
        PolicyAction::Spawn,
        PolicyEffect::Allow,
        "test_user".to_string(),
    );

    // Valid rule
    assert!(rule.validate().is_ok());

    // Invalid name
    rule.name = "".to_string();
    assert!(rule.validate().is_err());

    // Invalid creator
    rule.name = "test_rule".to_string();
    rule.created_by = "".to_string();
    assert!(rule.validate().is_err());

    // Invalid version
    rule.created_by = "test_user".to_string();
    rule.version = 0;
    assert!(rule.validate().is_err());
}
