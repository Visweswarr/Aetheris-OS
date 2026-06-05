//! XR Persistence Service - NGFS Snapshot Pipelines
//! 
//! This module provides durable multi-user persistence by committing room state
//! to NGFS snapshots with DAO/Audit linkage.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::scene::{
    SceneResult, SceneNode, Avatar, Snapshot, Transform, Vector3, Component,
    AvatarProfile, CapToken, PolicyContext, PolicyResult
};
use crate::multiuser::{RoomState, Participant};
use crate::physics::{PhysicsState, PhysicsBody, PhysicsConstraint};

/// Snapshot metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub id: String,
    pub room_id: String,
    pub version: u64,
    pub created_at: u64,
    pub created_by: String,
    pub reason: String,
    pub dao_proposal_id: Option<String>,
    pub audit_trail: Vec<AuditEntry>,
    pub deterministic_hash: String,
    pub ngfs_cid: Option<String>,
}

/// Audit entry for tracking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub details: HashMap<String, String>,
    pub signature: String,
}

/// Persistence request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistRoomRequest {
    pub room_id: String,
    pub reason: String,
    pub session_id: String,
    pub cap_token: String,
    pub dao_proposal_id: Option<String>,
}

/// Persistence response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistRoomResponse {
    pub snapshot_id: String,
    pub ngfs_cid: String,
    pub deterministic_hash: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Room state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentRoomState {
    pub room: RoomState,
    pub nodes: Vec<SceneNode>,
    pub avatars: Vec<Avatar>,
    pub physics_state: PhysicsState,
    pub policies: Vec<PolicySnapshot>,
    pub capabilities: Vec<CapToken>,
}

/// Policy snapshot for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySnapshot {
    pub id: String,
    pub scope: String,
    pub rego_bundle: String,
    pub created_at: u64,
    pub created_by: String,
    pub dao_approved: bool,
}

/// Persistence manager trait
#[async_trait::async_trait]
pub trait PersistenceManager: Send + Sync {
    async fn persist_room(&self, request: &PersistRoomRequest) -> SceneResult<PersistRoomResponse>;
    async fn load_snapshot(&self, snapshot_id: &str) -> SceneResult<PersistentRoomState>;
    async fn get_snapshot_metadata(&self, snapshot_id: &str) -> SceneResult<SnapshotMetadata>;
    async fn list_snapshots(&self, room_id: &str) -> SceneResult<Vec<SnapshotMetadata>>;
    async fn verify_determinism(&self, snapshot_id: &str) -> SceneResult<bool>;
}

/// NGFS persistence manager implementation
pub struct NGFS persistenceManager {
    ngfs_client: Arc<dyn NgfsClient + Send + Sync>,
    dao_client: Arc<dyn DaoClient + Send + Sync>,
    audit_client: Arc<dyn AuditClient + Send + Sync>,
    snapshots: Arc<RwLock<HashMap<String, SnapshotMetadata>>>,
    room_states: Arc<RwLock<HashMap<String, PersistentRoomState>>>,
}

/// NGFS client trait for snapshot storage
#[async_trait::async_trait]
pub trait NgfsClient: Send + Sync {
    async fn store_snapshot(&self, data: &[u8]) -> SceneResult<String>;
    async fn retrieve_snapshot(&self, cid: &str) -> SceneResult<Vec<u8>>;
    async fn verify_snapshot(&self, cid: &str, data: &[u8]) -> SceneResult<bool>;
}

/// DAO client trait for governance
#[async_trait::async_trait]
pub trait DaoClient: Send + Sync {
    async fn check_proposal_approval(&self, proposal_id: &str) -> SceneResult<bool>;
    async fn get_proposal_details(&self, proposal_id: &str) -> SceneResult<HashMap<String, String>>;
}

/// Audit client trait for audit trails
#[async_trait::async_trait]
pub trait AuditClient: Send + Sync {
    async fn log_action(&self, entry: &AuditEntry) -> SceneResult<()>;
    async fn get_audit_trail(&self, target: &str) -> SceneResult<Vec<AuditEntry>>;
}

impl NGFS persistenceManager {
    pub fn new(
        ngfs_client: Arc<dyn NgfsClient + Send + Sync>,
        dao_client: Arc<dyn DaoClient + Send + Sync>,
        audit_client: Arc<dyn AuditClient + Send + Sync>,
    ) -> Self {
        Self {
            ngfs_client,
            dao_client,
            audit_client,
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            room_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Calculate deterministic hash for snapshot
    fn calculate_deterministic_hash(&self, state: &PersistentRoomState) -> SceneResult<String> {
        use blake3::Hasher;
        
        // Serialize state deterministically
        let serialized = serde_cbor::to_vec(state)
            .map_err(|e| SceneError::SerializationError(e.to_string()))?;
        
        // Calculate BLAKE3 hash
        let mut hasher = Hasher::new();
        hasher.update(&serialized);
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Create audit entry for persistence action
    fn create_audit_entry(
        &self,
        actor: &str,
        action: &str,
        target: &str,
        details: HashMap<String, String>,
    ) -> AuditEntry {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // In real implementation, this would be signed with DID key
        let signature = format!("sig_{}_{}_{}", actor, action, timestamp);
        
        AuditEntry {
            timestamp,
            actor: actor.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            details,
            signature,
        }
    }

    /// Validate persistence request
    async fn validate_persist_request(&self, request: &PersistRoomRequest) -> SceneResult<()> {
        // Check session validity
        if request.session_id.is_empty() {
            return Err(SceneError::InvalidSession("Empty session ID".to_string()));
        }

        // Check capability token
        if request.cap_token.is_empty() {
            return Err(SceneError::InsufficientCapabilities("Empty capability token".to_string()));
        }

        // Check DAO proposal if provided
        if let Some(proposal_id) = &request.dao_proposal_id {
            let approved = self.dao_client.check_proposal_approval(proposal_id).await?;
            if !approved {
                return Err(SceneError::DaoRejected(format!("Proposal {} not approved", proposal_id)));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl PersistenceManager for NGFS persistenceManager {
    async fn persist_room(&self, request: &PersistRoomRequest) -> SceneResult<PersistRoomResponse> {
        // Validate request
        self.validate_persist_request(request).await?;

        // Get current room state (in real implementation, this would come from the scene service)
        let room_state = self.get_current_room_state(&request.room_id).await?;

        // Calculate deterministic hash
        let deterministic_hash = self.calculate_deterministic_hash(&room_state)?;

        // Create snapshot metadata
        let snapshot_id = format!("snapshot_{}", Uuid::new_v4());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create audit entry
        let mut details = HashMap::new();
        details.insert("reason".to_string(), request.reason.clone());
        details.insert("room_id".to_string(), request.room_id.clone());
        if let Some(proposal_id) = &request.dao_proposal_id {
            details.insert("dao_proposal_id".to_string(), proposal_id.clone());
        }

        let audit_entry = self.create_audit_entry(
            &request.session_id,
            "persist_room",
            &request.room_id,
            details,
        );

        // Store in NGFS
        let serialized_state = serde_cbor::to_vec(&room_state)
            .map_err(|e| SceneError::SerializationError(e.to_string()))?;
        
        let ngfs_cid = self.ngfs_client.store_snapshot(&serialized_state).await?;

        // Create snapshot metadata
        let metadata = SnapshotMetadata {
            id: snapshot_id.clone(),
            room_id: request.room_id.clone(),
            version: 1, // In real implementation, this would be incremented
            created_at: timestamp,
            created_by: request.session_id.clone(),
            reason: request.reason.clone(),
            dao_proposal_id: request.dao_proposal_id.clone(),
            audit_trail: vec![audit_entry],
            deterministic_hash: deterministic_hash.clone(),
            ngfs_cid: Some(ngfs_cid.clone()),
        };

        // Store metadata and state locally
        {
            let mut snapshots = self.snapshots.write().await;
            snapshots.insert(snapshot_id.clone(), metadata);
        }

        {
            let mut room_states = self.room_states.write().await;
            room_states.insert(snapshot_id.clone(), room_state);
        }

        // Log audit entry
        self.audit_client.log_action(&audit_entry).await?;

        Ok(PersistRoomResponse {
            snapshot_id,
            ngfs_cid,
            deterministic_hash,
            success: true,
            error_message: None,
        })
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> SceneResult<PersistentRoomState> {
        // Check local cache first
        {
            let room_states = self.room_states.read().await;
            if let Some(state) = room_states.get(snapshot_id) {
                return Ok(state.clone());
            }
        }

        // Get metadata
        let metadata = self.get_snapshot_metadata(snapshot_id).await?;
        
        // Retrieve from NGFS
        let ngfs_cid = metadata.ngfs_cid
            .ok_or_else(|| SceneError::NotFound("No NGFS CID for snapshot".to_string()))?;
        
        let serialized_state = self.ngfs_client.retrieve_snapshot(&ngfs_cid).await?;
        
        // Deserialize state
        let state: PersistentRoomState = serde_cbor::from_slice(&serialized_state)
            .map_err(|e| SceneError::DeserializationError(e.to_string()))?;

        // Verify deterministic hash
        let calculated_hash = self.calculate_deterministic_hash(&state)?;
        if calculated_hash != metadata.deterministic_hash {
            return Err(SceneError::IntegrityError("Hash mismatch".to_string()));
        }

        // Cache the state
        {
            let mut room_states = self.room_states.write().await;
            room_states.insert(snapshot_id.to_string(), state.clone());
        }

        Ok(state)
    }

    async fn get_snapshot_metadata(&self, snapshot_id: &str) -> SceneResult<SnapshotMetadata> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(snapshot_id)
            .cloned()
            .ok_or_else(|| SceneError::NotFound(format!("Snapshot {} not found", snapshot_id)))
    }

    async fn list_snapshots(&self, room_id: &str) -> SceneResult<Vec<SnapshotMetadata>> {
        let snapshots = self.snapshots.read().await;
        let room_snapshots: Vec<SnapshotMetadata> = snapshots
            .values()
            .filter(|metadata| metadata.room_id == room_id)
            .cloned()
            .collect();
        
        Ok(room_snapshots)
    }

    async fn verify_determinism(&self, snapshot_id: &str) -> SceneResult<bool> {
        let state = self.load_snapshot(snapshot_id).await?;
        let metadata = self.get_snapshot_metadata(snapshot_id).await?;
        
        let calculated_hash = self.calculate_deterministic_hash(&state)?;
        Ok(calculated_hash == metadata.deterministic_hash)
    }
}

impl NGFS persistenceManager {
    /// Get current room state (mock implementation)
    async fn get_current_room_state(&self, room_id: &str) -> SceneResult<PersistentRoomState> {
        // In real implementation, this would fetch from the scene service
        let room = RoomState {
            id: room_id.to_string(),
            name: format!("Room {}", room_id),
            participants: vec![],
            max_participants: 10,
            is_public: true,
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };

        let nodes = vec![
            SceneNode {
                id: "node_1".to_string(),
                name: "Test Cube".to_string(),
                parent_id: None,
                transform: Transform {
                    position: Vector3 { x: 0.0, y: 1.0, z: 0.0 },
                    rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                    scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
                },
                components: vec![],
                metadata: HashMap::new(),
            }
        ];

        let avatars = vec![
            Avatar {
                id: "avatar_1".to_string(),
                did: "did:aeth:test".to_string(),
                profile: AvatarProfile {
                    name: "Test Avatar".to_string(),
                    description: Some("Test avatar for persistence".to_string()),
                    appearance: HashMap::new(),
                    preferences: HashMap::new(),
                },
                transform: Transform {
                    position: Vector3 { x: 0.0, y: 1.0, z: 2.0 },
                    rotation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                    scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
                },
                is_online: true,
                metadata: HashMap::new(),
            }
        ];

        let physics_state = PhysicsState {
            bodies: vec![],
            constraints: vec![],
            gravity: Vector3 { x: 0.0, y: -9.81, z: 0.0 },
            time_step: 1.0 / 60.0,
            metadata: HashMap::new(),
        };

        let policies = vec![];
        let capabilities = vec![];

        Ok(PersistentRoomState {
            room,
            nodes,
            avatars,
            physics_state,
            policies,
            capabilities,
        })
    }
}

/// Mock implementations for testing
pub struct MockNgfsClient;

#[async_trait::async_trait]
impl NgfsClient for MockNgfsClient {
    async fn store_snapshot(&self, data: &[u8]) -> SceneResult<String> {
        // Mock implementation - return a fake CID
        Ok(format!("Qm{}", hex::encode(&data[..8])))
    }

    async fn retrieve_snapshot(&self, _cid: &str) -> SceneResult<Vec<u8>> {
        // Mock implementation - return empty data
        Ok(vec![])
    }

    async fn verify_snapshot(&self, _cid: &str, _data: &[u8]) -> SceneResult<bool> {
        Ok(true)
    }
}

pub struct MockDaoClient;

#[async_trait::async_trait]
impl DaoClient for MockDaoClient {
    async fn check_proposal_approval(&self, _proposal_id: &str) -> SceneResult<bool> {
        Ok(true)
    }

    async fn get_proposal_details(&self, _proposal_id: &str) -> SceneResult<HashMap<String, String>> {
        Ok(HashMap::new())
    }
}

pub struct MockAuditClient;

#[async_trait::async_trait]
impl AuditClient for MockAuditClient {
    async fn log_action(&self, _entry: &AuditEntry) -> SceneResult<()> {
        Ok(())
    }

    async fn get_audit_trail(&self, _target: &str) -> SceneResult<Vec<AuditEntry>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_persist_room() {
        let ngfs_client = Arc::new(MockNgfsClient);
        let dao_client = Arc::new(MockDaoClient);
        let audit_client = Arc::new(MockAuditClient);
        
        let manager = NGFS persistenceManager::new(ngfs_client, dao_client, audit_client);
        
        let request = PersistRoomRequest {
            room_id: "test_room".to_string(),
            reason: "Test persistence".to_string(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
            dao_proposal_id: None,
        };
        
        let response = manager.persist_room(&request).await.unwrap();
        assert!(response.success);
        assert!(!response.snapshot_id.is_empty());
        assert!(!response.ngfs_cid.is_empty());
        assert!(!response.deterministic_hash.is_empty());
    }

    #[tokio::test]
    async fn test_deterministic_hash() {
        let ngfs_client = Arc::new(MockNgfsClient);
        let dao_client = Arc::new(MockDaoClient);
        let audit_client = Arc::new(MockAuditClient);
        
        let manager = NGFS persistenceManager::new(ngfs_client, dao_client, audit_client);
        
        let state = manager.get_current_room_state("test_room").await.unwrap();
        let hash1 = manager.calculate_deterministic_hash(&state).unwrap();
        let hash2 = manager.calculate_deterministic_hash(&state).unwrap();
        
        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_load_snapshot() {
        let ngfs_client = Arc::new(MockNgfsClient);
        let dao_client = Arc::new(MockDaoClient);
        let audit_client = Arc::new(MockAuditClient);
        
        let manager = NGFS persistenceManager::new(ngfs_client, dao_client, audit_client);
        
        let request = PersistRoomRequest {
            room_id: "test_room".to_string(),
            reason: "Test persistence".to_string(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
            dao_proposal_id: None,
        };
        
        let response = manager.persist_room(&request).await.unwrap();
        let loaded_state = manager.load_snapshot(&response.snapshot_id).await.unwrap();
        
        assert_eq!(loaded_state.room.id, "test_room");
        assert!(!loaded_state.nodes.is_empty());
        assert!(!loaded_state.avatars.is_empty());
    }
}
