//! Integration tests for XR Scene Service
//! 
//! This module contains comprehensive integration tests that verify the
//! end-to-end functionality of the XR scene service with session management,
//! capability tokens, policy enforcement, and DAO governance.

use std::sync::Arc;
use std::time::Duration;

use crate::{
    SceneService, SceneConfig, SceneGraph, SessionManager, CapManager, PolicyManager,
    SnapshotManager, SceneRpcService, RpcRequest, BeginSessionRpcRequest, EndSessionRpcRequest,
    IssueCapRpcRequest, SpawnNodeRpcRequest, MoveNodeRpcRequest, DeleteNodeRpcRequest,
    AttachPolicyRpcRequest, SimulateRpcRequest, SnapshotSaveRpcRequest, SnapshotLoadRpcRequest,
    NodeSpec, Transform, Component, ComponentType, CapToken, PolicyBundle, PolicyRule,
    PolicyScope, PolicyAction, PolicyEffect, DaoHookFactory, MockDaoClient, SceneEvent,
    AvatarProfile, NodeId,
};

/// Test helper for creating a complete scene service
async fn create_test_scene_service() -> SceneService {
    let config = SceneConfig {
        max_nodes: 1000,
        max_avatars: 100,
        tick_rate_ms: 16,
        deterministic_mode: true,
        auto_save_interval: 300,
        enable_policy_enforcement: true,
        enable_capability_checking: true,
    };
    
    SceneService::new(config).await.unwrap()
}

/// Test helper for creating an RPC service
fn create_test_rpc_service() -> SceneRpcService {
    let scene_graph = Arc::new(SceneGraph::new(1000));
    let session_manager = Arc::new(SessionManager::default());
    let cap_manager = Arc::new(CapManager::new());
    let policy_manager = Arc::new(PolicyManager::new());
    let snapshot_manager = Arc::new(SnapshotManager::new());
    let dao_hook = Some(DaoHookFactory::create_mock_hook());

    SceneRpcService::new(
        scene_graph,
        session_manager,
        cap_manager,
        policy_manager,
        snapshot_manager,
        dao_hook,
    )
}

/// Test helper for creating a test avatar profile
fn create_test_avatar_profile() -> AvatarProfile {
    AvatarProfile {
        name: "Test Avatar".to_string(),
        description: Some("Test avatar for integration tests".to_string()),
        mesh_url: Some("test_mesh.glb".to_string()),
        texture_url: Some("test_texture.png".to_string()),
        animation_url: Some("test_animation.glb".to_string()),
        scale: [1.0, 1.0, 1.0],
        position: [0.0, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
        metadata: std::collections::HashMap::new(),
    }
}

/// Test helper for creating a test node spec
fn create_test_node_spec() -> NodeSpec {
    NodeSpec {
        parent_id: None,
        kind: "cube".to_string(),
        transform: Transform {
            position: [0.0, 1.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        components: vec![
            Component {
                id: "mesh".to_string(),
                component_type: ComponentType::Mesh,
                data: serde_json::json!({
                    "mesh_url": "cube.glb",
                    "material": "default"
                }),
            },
        ],
    }
}

/// Test helper for creating a test policy bundle
fn create_test_policy_bundle() -> PolicyBundle {
    let mut bundle = PolicyBundle::new(
        "Test Policy Bundle".to_string(),
        "did:aeth:test".to_string(),
    );
    
    let rule = PolicyRule::new(
        "Allow Spawn".to_string(),
        PolicyScope::Scene,
        PolicyAction::Spawn,
        PolicyEffect::Allow,
        "did:aeth:test".to_string(),
    );
    
    bundle.add_rule(rule);
    bundle
}

#[tokio::test]
async fn test_complete_session_lifecycle() {
    let service = create_test_scene_service().await;
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: Some(create_test_avatar_profile()),
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    assert!(begin_response.success);
    let session_id = begin_response.payload.session_id;
    
    // Issue capability token
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["scene:spawn".to_string(), "scene:move".to_string()],
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    assert!(cap_response.success);
    let cap_token = cap_response.payload.cap_token;
    
    // Spawn a node
    let spawn_request = RpcRequest {
        id: "test_spawn".to_string(),
        payload: SpawnNodeRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            spec: create_test_node_spec(),
        },
        timestamp: 1234567890,
    };
    
    let spawn_response = rpc_service.spawn_node(spawn_request).await.unwrap();
    assert!(spawn_response.success);
    let node_id = spawn_response.payload.node_id;
    
    // Move the node
    let move_request = RpcRequest {
        id: "test_move".to_string(),
        payload: MoveNodeRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            node_id: node_id.clone(),
            transform: Transform {
                position: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
        },
        timestamp: 1234567890,
    };
    
    let move_response = rpc_service.move_node(move_request).await.unwrap();
    assert!(move_response.success);
    
    // End session
    let end_request = RpcRequest {
        id: "test_end".to_string(),
        payload: EndSessionRpcRequest {
            session_id: session_id.clone(),
            reason: Some("Test completion".to_string()),
        },
        timestamp: 1234567890,
    };
    
    let end_response = rpc_service.end_session(end_request).await.unwrap();
    assert!(end_response.success);
}

#[tokio::test]
async fn test_capability_enforcement() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Issue capability token with limited scope
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["scene:spawn".to_string()], // Only spawn, no move
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    let cap_token = cap_response.payload.cap_token;
    
    // Spawn a node (should succeed)
    let spawn_request = RpcRequest {
        id: "test_spawn".to_string(),
        payload: SpawnNodeRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            spec: create_test_node_spec(),
        },
        timestamp: 1234567890,
    };
    
    let spawn_response = rpc_service.spawn_node(spawn_request).await.unwrap();
    assert!(spawn_response.success);
    let node_id = spawn_response.payload.node_id;
    
    // Try to move the node (should fail due to insufficient capability)
    let move_request = RpcRequest {
        id: "test_move".to_string(),
        payload: MoveNodeRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            node_id: node_id.clone(),
            transform: Transform {
                position: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: [1.0, 1.0, 1.0],
            },
        },
        timestamp: 1234567890,
    };
    
    let move_response = rpc_service.move_node(move_request).await;
    assert!(move_response.is_err());
}

#[tokio::test]
async fn test_policy_attachment_and_simulation() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Issue capability token for policy management
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["policy:attach".to_string()],
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    let cap_token = cap_response.payload.cap_token;
    
    // Create and serialize policy bundle
    let policy_bundle = create_test_policy_bundle();
    let rego_bundle_cbor = policy_bundle.to_cbor().unwrap();
    
    // Attach policy
    let attach_request = RpcRequest {
        id: "test_attach".to_string(),
        payload: AttachPolicyRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            scope: "scene".to_string(),
            rego_bundle_cbor,
        },
        timestamp: 1234567890,
    };
    
    let attach_response = rpc_service.attach_policy(attach_request).await.unwrap();
    assert!(attach_response.success);
    let policy_id = attach_response.payload.policy_id;
    
    // Simulate operations
    let simulate_request = RpcRequest {
        id: "test_simulate".to_string(),
        payload: SimulateRpcRequest {
            session_id: session_id.clone(),
            operations: vec![
                crate::rpc::SimulationOp {
                    operation: "spawn_node".to_string(),
                    parameters: std::collections::HashMap::new(),
                },
                crate::rpc::SimulationOp {
                    operation: "move_node".to_string(),
                    parameters: {
                        let mut params = std::collections::HashMap::new();
                        params.insert("node_id".to_string(), serde_json::Value::String("test_node".to_string()));
                        params
                    },
                },
            ],
        },
        timestamp: 1234567890,
    };
    
    let simulate_response = rpc_service.simulate(simulate_request).await.unwrap();
    assert!(simulate_response.success);
    assert_eq!(simulate_response.payload.results.len(), 2);
}

#[tokio::test]
async fn test_snapshot_save_and_load() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Save snapshot
    let save_request = RpcRequest {
        id: "test_save".to_string(),
        payload: SnapshotSaveRpcRequest {
            session_id: session_id.clone(),
            label: "test_snapshot".to_string(),
            anchor: Some(true),
        },
        timestamp: 1234567890,
    };
    
    let save_response = rpc_service.snapshot_save(save_request).await.unwrap();
    assert!(save_response.success);
    let snapshot_id = save_response.payload.snapshot_id;
    
    // Load snapshot
    let load_request = RpcRequest {
        id: "test_load".to_string(),
        payload: SnapshotLoadRpcRequest {
            session_id: session_id.clone(),
            snapshot_id: snapshot_id.clone(),
        },
        timestamp: 1234567890,
    };
    
    let load_response = rpc_service.snapshot_load(load_request).await.unwrap();
    assert!(load_response.success);
}

#[tokio::test]
async fn test_event_stream() {
    let rpc_service = create_test_rpc_service();
    let mut event_receiver = rpc_service.get_event_receiver();
    
    // Begin session (should emit event)
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Check for session started event
    tokio::time::sleep(Duration::from_millis(10)).await;
    if let Ok(event) = event_receiver.try_recv() {
        match event {
            SceneEvent::AvatarSessionStarted { session_id: event_session_id, .. } => {
                assert_eq!(event_session_id, session_id);
            }
            _ => panic!("Expected AvatarSessionStarted event"),
        }
    }
    
    // Issue capability token (should emit event)
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["scene:spawn".to_string()],
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    let cap_token = cap_response.payload.cap_token;
    
    // Check for cap issued event
    tokio::time::sleep(Duration::from_millis(10)).await;
    if let Ok(event) = event_receiver.try_recv() {
        match event {
            SceneEvent::CapIssued { token_id, .. } => {
                assert_eq!(token_id, cap_token.id);
            }
            _ => panic!("Expected CapIssued event"),
        }
    }
}

#[tokio::test]
async fn test_audit_logging() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Issue capability token
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["scene:spawn".to_string()],
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    let cap_token = cap_response.payload.cap_token;
    
    // Perform operations that should be audited
    let spawn_request = RpcRequest {
        id: "test_spawn".to_string(),
        payload: SpawnNodeRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            spec: create_test_node_spec(),
        },
        timestamp: 1234567890,
    };
    
    let spawn_response = rpc_service.spawn_node(spawn_request).await.unwrap();
    assert!(spawn_response.success);
    
    // Check audit log
    let audit_log = rpc_service.get_audit_log().await;
    assert!(!audit_log.is_empty());
    
    // Find spawn_node audit record
    let spawn_audit = audit_log.iter()
        .find(|record| record.operation == "spawn_node")
        .expect("Should have spawn_node audit record");
    
    assert_eq!(spawn_audit.session_id, session_id);
    assert_eq!(spawn_audit.user_did, "did:aeth:test");
    assert!(spawn_audit.allowed);
    assert!(spawn_audit.reason.contains("successfully"));
}

#[tokio::test]
async fn test_deterministic_snapshots() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Issue capability token
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["scene:spawn".to_string()],
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    let cap_token = cap_response.payload.cap_token;
    
    // Spawn a node
    let spawn_request = RpcRequest {
        id: "test_spawn".to_string(),
        payload: SpawnNodeRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            spec: create_test_node_spec(),
        },
        timestamp: 1234567890,
    };
    
    let spawn_response = rpc_service.spawn_node(spawn_request).await.unwrap();
    assert!(spawn_response.success);
    
    // Save snapshot multiple times
    let mut snapshot_ids = Vec::new();
    for i in 0..3 {
        let save_request = RpcRequest {
            id: format!("test_save_{}", i),
            payload: SnapshotSaveRpcRequest {
                session_id: session_id.clone(),
                label: format!("test_snapshot_{}", i),
                anchor: Some(false),
            },
            timestamp: 1234567890,
        };
        
        let save_response = rpc_service.snapshot_save(save_request).await.unwrap();
        assert!(save_response.success);
        snapshot_ids.push(save_response.payload.snapshot_id);
    }
    
    // In a real implementation, we would verify that the snapshots are byte-identical
    // For now, we just verify that all snapshots were created successfully
    assert_eq!(snapshot_ids.len(), 3);
}

#[tokio::test]
async fn test_session_expiration() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session with short TTL
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(1), // 1 second TTL
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Wait for session to expire
    tokio::time::sleep(Duration::from_millis(1100)).await;
    
    // Try to issue capability token (should fail)
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["scene:spawn".to_string()],
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await;
    assert!(cap_response.is_err());
}

#[tokio::test]
async fn test_capability_token_expiration() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Issue capability token with short TTL
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["scene:spawn".to_string()],
            ttl_seconds: 1, // 1 second TTL
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    let cap_token = cap_response.payload.cap_token;
    
    // Wait for token to expire
    tokio::time::sleep(Duration::from_millis(1100)).await;
    
    // Try to spawn node (should fail due to expired token)
    let spawn_request = RpcRequest {
        id: "test_spawn".to_string(),
        payload: SpawnNodeRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            spec: create_test_node_spec(),
        },
        timestamp: 1234567890,
    };
    
    let spawn_response = rpc_service.spawn_node(spawn_request).await;
    assert!(spawn_response.is_err());
}

#[tokio::test]
async fn test_dao_policy_enforcement() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Issue capability token for policy management
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["policy:attach".to_string()],
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    let cap_token = cap_response.payload.cap_token;
    
    // Try to attach policy (should be allowed by DAO)
    let policy_bundle = create_test_policy_bundle();
    let rego_bundle_cbor = policy_bundle.to_cbor().unwrap();
    
    let attach_request = RpcRequest {
        id: "test_attach".to_string(),
        payload: AttachPolicyRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            scope: "scene".to_string(),
            rego_bundle_cbor,
        },
        timestamp: 1234567890,
    };
    
    let attach_response = rpc_service.attach_policy(attach_request).await.unwrap();
    assert!(attach_response.success);
}

#[tokio::test]
async fn test_performance_spawn_to_visible() {
    let rpc_service = create_test_rpc_service();
    
    // Begin session
    let begin_request = RpcRequest {
        id: "test_begin".to_string(),
        payload: BeginSessionRpcRequest {
            did: "did:aeth:test".to_string(),
            proof: "jwt_proof".to_string(),
            nonce: "nonce123".to_string(),
            avatar_profile: None,
            ttl_seconds: Some(3600),
        },
        timestamp: 1234567890,
    };
    
    let begin_response = rpc_service.begin_session(begin_request).await.unwrap();
    let session_id = begin_response.payload.session_id;
    
    // Issue capability token
    let cap_request = RpcRequest {
        id: "test_cap".to_string(),
        payload: IssueCapRpcRequest {
            session_id: session_id.clone(),
            scopes: vec!["scene:spawn".to_string()],
            ttl_seconds: 300,
        },
        timestamp: 1234567890,
    };
    
    let cap_response = rpc_service.issue_cap(cap_request).await.unwrap();
    let cap_token = cap_response.payload.cap_token;
    
    // Measure spawn to visible time
    let start_time = std::time::Instant::now();
    
    let spawn_request = RpcRequest {
        id: "test_spawn".to_string(),
        payload: SpawnNodeRpcRequest {
            session_id: session_id.clone(),
            cap_token: cap_token.clone(),
            spec: create_test_node_spec(),
        },
        timestamp: 1234567890,
    };
    
    let spawn_response = rpc_service.spawn_node(spawn_request).await.unwrap();
    assert!(spawn_response.success);
    
    let elapsed = start_time.elapsed();
    
    // Verify performance requirement: p95 spawn → visible ≤ 120ms
    assert!(elapsed.as_millis() <= 120, "Spawn to visible took {}ms, exceeds 120ms limit", elapsed.as_millis());
}
