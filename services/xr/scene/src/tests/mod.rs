//! Test modules for XR Scene Service
//! 
//! This module organizes the test suites for the XR scene service.

pub mod unit;
pub mod integration;

/// Test utilities and helpers
pub mod utils {
    use std::time::Duration;
    use crate::{
        SessionManager, CapManager, PolicyManager, SnapshotManager,
        SceneGraph, SceneService, SceneConfig, SceneRpcService,
        DaoHookFactory, AvatarProfile, NodeSpec, Transform, Component, ComponentType,
        PolicyBundle, PolicyRule, PolicyScope, PolicyAction, PolicyEffect,
    };

    /// Create a test scene service with default configuration
    pub async fn create_test_scene_service() -> SceneService {
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

    /// Create a test RPC service with all components
    pub fn create_test_rpc_service() -> SceneRpcService {
        let scene_graph = std::sync::Arc::new(SceneGraph::new(1000));
        let session_manager = std::sync::Arc::new(SessionManager::default());
        let cap_manager = std::sync::Arc::new(CapManager::new());
        let policy_manager = std::sync::Arc::new(PolicyManager::new());
        let snapshot_manager = std::sync::Arc::new(SnapshotManager::new());
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

    /// Create a test avatar profile
    pub fn create_test_avatar_profile() -> AvatarProfile {
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

    /// Create a test node specification
    pub fn create_test_node_spec() -> NodeSpec {
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

    /// Create a test policy bundle
    pub fn create_test_policy_bundle() -> PolicyBundle {
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

    /// Create a test DID proof
    pub fn create_test_did_proof(did: &str) -> crate::session::DidProof {
        crate::session::DidProof {
            did: did.to_string(),
            proof: "jwt_proof".to_string(),
            nonce: uuid::Uuid::new_v4().to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a test capability token
    pub fn create_test_cap_token(name: &str, scopes: Vec<String>) -> crate::cap::CapToken {
        let mut token = crate::cap::CapToken::new(name.to_string());
        token.set_creator("did:aeth:test".to_string());
        
        for scope in scopes {
            token.add_granted_scope(scope);
        }
        
        token
    }

    /// Create a test transform
    pub fn create_test_transform() -> Transform {
        Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    /// Create a test component
    pub fn create_test_component(id: &str, component_type: ComponentType) -> Component {
        Component {
            id: id.to_string(),
            component_type,
            data: serde_json::json!({
                "test": "data"
            }),
        }
    }

    /// Wait for a short duration (useful for async tests)
    pub async fn wait_short() {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    /// Wait for a medium duration (useful for async tests)
    pub async fn wait_medium() {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    /// Wait for a long duration (useful for async tests)
    pub async fn wait_long() {
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    /// Assert that two values are approximately equal (for floating point comparisons)
    pub fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!((a - b).abs() < epsilon, "{} is not approximately equal to {} (epsilon: {})", a, b, epsilon);
    }

    /// Assert that a duration is within expected bounds
    pub fn assert_duration_within(duration: Duration, min: Duration, max: Duration) {
        assert!(duration >= min, "Duration {:?} is less than minimum {:?}", duration, min);
        assert!(duration <= max, "Duration {:?} is greater than maximum {:?}", duration, max);
    }

    /// Create a test session manager with custom settings
    pub fn create_test_session_manager(ttl: Duration, max_sessions: usize) -> SessionManager {
        SessionManager::new(ttl, max_sessions)
    }

    /// Create a test capability manager
    pub fn create_test_cap_manager() -> CapManager {
        CapManager::new()
    }

    /// Create a test policy manager
    pub fn create_test_policy_manager() -> PolicyManager {
        PolicyManager::new()
    }

    /// Create a test snapshot manager
    pub fn create_test_snapshot_manager() -> SnapshotManager {
        SnapshotManager::new()
    }

    /// Create a test scene graph
    pub fn create_test_scene_graph(max_nodes: usize) -> SceneGraph {
        SceneGraph::new(max_nodes)
    }
}
