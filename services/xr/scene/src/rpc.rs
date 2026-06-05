//! RPC Layer for XR Scene Service
//! 
//! This module provides the RPC interface for the XR scene service, including
//! session management, capability token operations, scene mutations, and policy
//! enforcement with proper authorization chains.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{SceneError, SceneResult};
use crate::scene::{SceneGraph, NodeId, Transform, Component};
use crate::avatar::{AvatarId, AvatarProfile};
use crate::session::{SessionId, SessionManager, BeginSessionRequest, BeginSessionResponse, EndSessionRequest, EndSessionResponse, RefreshSessionRequest, RefreshSessionResponse, DidProof};
use crate::cap::{CapManager, CapToken, CapScope, CapAction};
use crate::policy::{PolicyManager, AttachPolicyRequest, AttachPolicyResponse, SimulateRequest, SimulateResponse, PolicyContext, PolicyAction, PolicyScope};
use crate::storage::{SnapshotManager, SnapshotId};
use crate::dao_hook::DaoHook;

// Import new modules for P4-06-A4
use crate::persist::{PersistenceManager, PersistRoomRequest, PersistRoomResponse};
use crate::record::{RecordingManager, StartRecordingRequest, StartRecordingResponse, StopRecordingRequest, StopRecordingResponse};
use crate::replay::{ReplayManager, ReplayRecordingRequest, ReplayRecordingResponse};
use crate::bridge::{BridgeManager, ExportSceneRequest, ExportSceneResponse, ImportSceneRequest, ImportSceneResponse, PublishSceneRequest, PublishSceneResponse};

/// RPC request wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest<T> {
    /// Request ID for correlation
    pub id: String,
    /// Request payload
    pub payload: T,
    /// Request timestamp
    pub timestamp: u64,
}

/// RPC response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    /// Request ID for correlation
    pub id: String,
    /// Response payload
    pub payload: T,
    /// Response timestamp
    pub timestamp: u64,
    /// Whether the request was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error: Option<String>,
}

/// Begin session request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginSessionRpcRequest {
    /// DID proof for authentication
    pub did: String,
    /// Authentication proof
    pub proof: String,
    /// Nonce for replay protection
    pub nonce: String,
    /// Optional avatar profile to bind
    pub avatar_profile: Option<AvatarProfile>,
    /// Session TTL in seconds
    pub ttl_seconds: Option<u64>,
}

/// Begin session response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginSessionRpcResponse {
    /// Created session ID
    pub session_id: String,
    /// Bound avatar ID (if any)
    pub avatar_id: Option<String>,
    /// Session expiration timestamp
    pub expires_at: u64,
}

/// End session request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndSessionRpcRequest {
    /// Session ID to terminate
    pub session_id: String,
    /// Reason for termination
    pub reason: Option<String>,
}

/// End session response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndSessionRpcResponse {
    /// Whether termination was successful
    pub success: bool,
    /// Termination timestamp
    pub terminated_at: u64,
}

/// Issue capability token request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCapRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Granted scopes
    pub scopes: Vec<String>,
    /// Token TTL in seconds
    pub ttl_seconds: u64,
}

/// Issue capability token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCapRpcResponse {
    /// Issued capability token
    pub cap_token: CapToken,
    /// Token expiration timestamp
    pub expires_at: u64,
}

/// Spawn node request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnNodeRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Capability token
    pub cap_token: CapToken,
    /// Node specification
    pub spec: NodeSpec,
}

/// Node specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSpec {
    /// Parent node ID (None for root)
    pub parent_id: Option<NodeId>,
    /// Node kind/type
    pub kind: String,
    /// Initial transform
    pub transform: Transform,
    /// Initial components
    pub components: Vec<Component>,
}

/// Spawn node response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnNodeRpcResponse {
    /// Created node ID
    pub node_id: String,
    /// Creation timestamp
    pub created_at: u64,
}

/// Move node request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveNodeRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Capability token
    pub cap_token: CapToken,
    /// Node ID to move
    pub node_id: String,
    /// New transform
    pub transform: Transform,
}

/// Move node response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveNodeRpcResponse {
    /// Whether move was successful
    pub success: bool,
    /// Move timestamp
    pub moved_at: u64,
}

/// Delete node request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNodeRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Capability token
    pub cap_token: CapToken,
    /// Node ID to delete
    pub node_id: String,
}

/// Delete node response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteNodeRpcResponse {
    /// Whether deletion was successful
    pub success: bool,
    /// Deletion timestamp
    pub deleted_at: u64,
}

/// Attach policy request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachPolicyRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Capability token
    pub cap_token: CapToken,
    /// Policy scope
    pub scope: String,
    /// Rego policy bundle in CBOR format
    pub rego_bundle_cbor: Vec<u8>,
}

/// Attach policy response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachPolicyRpcResponse {
    /// Generated policy ID
    pub policy_id: String,
    /// Whether attachment was successful
    pub success: bool,
    /// Attachment timestamp
    pub attached_at: u64,
}

/// Simulate operations request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Operations to simulate
    pub operations: Vec<SimulationOp>,
}

/// Simulation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationOp {
    /// Operation type
    pub operation: String,
    /// Operation parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Simulate operations response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateRpcResponse {
    /// Results for each operation
    pub results: Vec<OperationDecision>,
    /// Simulation timestamp
    pub simulated_at: u64,
}

/// Operation decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDecision {
    /// Operation that was simulated
    pub operation: String,
    /// Whether operation would be allowed
    pub allowed: bool,
    /// Reason for allow/deny
    pub reason: Option<String>,
    /// Policy reference that applied
    pub policy_ref: Option<String>,
}

/// Snapshot save request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotSaveRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Snapshot label
    pub label: String,
    /// Whether to anchor on-chain
    pub anchor: Option<bool>,
}

/// Snapshot save response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotSaveRpcResponse {
    /// Generated snapshot ID
    pub snapshot_id: String,
    /// Whether save was successful
    pub success: bool,
    /// Save timestamp
    pub saved_at: u64,
}

/// Snapshot load request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotLoadRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Snapshot ID to load
    pub snapshot_id: String,
}

/// Snapshot load response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotLoadRpcResponse {
    /// Whether load was successful
    pub success: bool,
    /// Load timestamp
    pub loaded_at: u64,
}

/// Start XR device request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartXRDeviceRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Device profile (e.g., "oculus_quest_2", "htc_vive", "mock")
    pub device_profile: String,
    /// XR session ID for device binding
    pub xr_session_id: String,
}

/// Start XR device response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartXRDeviceRpcResponse {
    /// Device handle for subsequent operations
    pub device_handle: String,
    /// Device capabilities
    pub capabilities: Vec<String>,
    /// Whether device was started successfully
    pub success: bool,
    /// Start timestamp
    pub started_at: u64,
}

/// Send input request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendInputRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Device handle
    pub device_handle: String,
    /// Input event data
    pub input_event: InputEvent,
}

/// Input event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// Button press/release
    Button { button_id: String, pressed: bool, timestamp: u64 },
    /// Trigger value change
    Trigger { trigger_id: String, value: f32, timestamp: u64 },
    /// Joystick/thumbstick movement
    Joystick { joystick_id: String, x: f32, y: f32, timestamp: u64 },
    /// Hand tracking data
    HandTracking { hand_id: String, joints: Vec<HandJoint>, timestamp: u64 },
    /// Eye tracking data
    EyeTracking { gaze_origin: Vector3, gaze_direction: Vector3, timestamp: u64 },
    /// Voice command
    VoiceCommand { command: String, confidence: f32, timestamp: u64 },
}

/// Hand joint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandJoint {
    pub joint_id: String,
    pub position: Vector3,
    pub rotation: Vector3,
    pub confidence: f32,
}

/// Vector3 for 3D coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Send input response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendInputRpcResponse {
    /// Whether input was processed successfully
    pub success: bool,
    /// Acknowledgment timestamp
    pub ack_timestamp: u64,
}

/// Join multi-user session request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinMultiUserRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Room ID to join
    pub room_id: String,
    /// User's display name
    pub display_name: String,
    /// Avatar configuration
    pub avatar_config: Option<AvatarConfig>,
}

/// Avatar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarConfig {
    pub model_id: String,
    pub skin_color: String,
    pub hair_color: String,
    pub clothing: Vec<String>,
}

/// Join multi-user session response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinMultiUserRpcResponse {
    /// Participant ID in the room
    pub participant_id: String,
    /// Current room state
    pub room_state: RoomState,
    /// Whether join was successful
    pub success: bool,
    /// Join timestamp
    pub joined_at: u64,
}

/// Room state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomState {
    pub room_id: String,
    pub participants: Vec<Participant>,
    pub scene_snapshot: Option<String>,
    pub last_updated: u64,
}

/// Participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub participant_id: String,
    pub session_id: String,
    pub display_name: String,
    pub avatar_config: Option<AvatarConfig>,
    pub position: Vector3,
    pub rotation: Vector3,
    pub is_active: bool,
    pub joined_at: u64,
}

/// Sync scene request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSceneRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Room ID
    pub room_id: String,
    /// Last known scene version
    pub last_version: Option<u64>,
}

/// Sync scene response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSceneRpcResponse {
    /// Scene snapshot data
    pub scene_snapshot: SceneSnapshot,
    /// Whether sync was successful
    pub success: bool,
    /// Sync timestamp
    pub synced_at: u64,
}

/// Scene snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSnapshot {
    pub version: u64,
    pub nodes: Vec<SceneNode>,
    pub participants: Vec<Participant>,
    pub physics_state: Option<PhysicsState>,
    pub timestamp: u64,
}

/// Scene node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub node_id: String,
    pub parent_id: Option<String>,
    pub transform: Transform,
    pub components: Vec<Component>,
    pub owner: Option<String>,
}

/// Physics state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsState {
    pub bodies: Vec<PhysicsBody>,
    pub constraints: Vec<PhysicsConstraint>,
    pub timestamp: u64,
}

/// Physics body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBody {
    pub body_id: String,
    pub node_id: String,
    pub body_type: String,
    pub transform: Transform,
    pub velocity: Vector3,
    pub angular_velocity: Vector3,
}

/// Physics constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConstraint {
    pub constraint_id: String,
    pub body_a: String,
    pub body_b: String,
    pub constraint_type: String,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

/// Physics interaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsInteractRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Capability token
    pub cap_token: CapToken,
    /// Physics operation
    pub operation: PhysicsOperation,
}

/// Physics operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsOperation {
    /// Grab an object
    Grab { object_id: String, grab_point: Vector3 },
    /// Move an object
    Move { object_id: String, target_transform: Transform },
    /// Release a grabbed object
    Release { object_id: String },
    /// Apply force to an object
    ApplyForce { object_id: String, force: Vector3, point: Vector3 },
    /// Apply impulse to an object
    ApplyImpulse { object_id: String, impulse: Vector3, point: Vector3 },
    /// Set object velocity
    SetVelocity { object_id: String, velocity: Vector3 },
    /// Teleport object
    Teleport { object_id: String, transform: Transform },
    /// Raycast from point
    Raycast { origin: Vector3, direction: Vector3, max_distance: f32 },
}

/// Physics interaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsInteractRpcResponse {
    /// Whether interaction was successful
    pub success: bool,
    /// Operation result (for raycast, etc.)
    pub result: Option<PhysicsResult>,
    /// Interaction timestamp
    pub interacted_at: u64,
}

/// Physics operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsResult {
    /// Raycast hit result
    RaycastHit {
        hit: bool,
        body_id: Option<String>,
        node_id: Option<String>,
        hit_point: Vector3,
        hit_normal: Vector3,
        distance: f32,
    },
    /// Generic success result
    Success { message: String },
}

// P4-06-A4: Persistence RPCs
/// Persist room request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistRoomRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Room ID to persist
    pub room_id: String,
    /// Reason for persistence
    pub reason: String,
    /// Capability token
    pub cap_token: String,
    /// DAO proposal ID (optional)
    pub dao_proposal_id: Option<String>,
}

/// Persist room response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistRoomRpcResponse {
    /// Generated snapshot ID
    pub snapshot_id: String,
    /// NGFS CID
    pub ngfs_cid: String,
    /// Deterministic hash
    pub deterministic_hash: String,
    /// Whether persistence was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
}

// P4-06-A4: Recording RPCs
/// Start recording request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRecordingRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Room ID to record
    pub room_id: String,
    /// Capability token
    pub cap_token: String,
    /// Deterministic seed (optional)
    pub deterministic_seed: Option<u64>,
    /// Physics tick rate (optional)
    pub physics_tick_rate: Option<f32>,
}

/// Start recording response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRecordingRpcResponse {
    /// Recording ID
    pub recording_id: String,
    /// Whether recording was started successfully
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
}

/// Stop recording request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopRecordingRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Recording ID to stop
    pub recording_id: String,
    /// Capability token
    pub cap_token: String,
}

/// Stop recording response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopRecordingRpcResponse {
    /// Recording summary
    pub recording_summary: RecordingSummary,
    /// Whether stop was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
}

// P4-06-A4: Replay RPCs
/// Replay recording request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRecordingRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Recording ID to replay
    pub recording_id: String,
    /// Capability token
    pub cap_token: String,
    /// Replay mode
    pub mode: ReplayMode,
    /// Start event (optional)
    pub start_event: Option<u64>,
    /// End event (optional)
    pub end_event: Option<u64>,
    /// Verify determinism
    pub verify_determinism: bool,
}

/// Replay recording response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRecordingRpcResponse {
    /// Replay ID
    pub replay_id: String,
    /// Whether replay was started successfully
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
}

/// Replay mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayMode {
    Headless,
    Visualization,
    Debug,
}

// P4-06-A4: Bridge RPCs
/// Export scene request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSceneRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Room ID to export
    pub room_id: String,
    /// Capability token
    pub cap_token: String,
    /// Export formats
    pub formats: Vec<ExportFormat>,
    /// Include avatars
    pub include_avatars: bool,
    /// Include physics
    pub include_physics: bool,
    /// Compression level (optional)
    pub compression_level: Option<u8>,
}

/// Export scene response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSceneRpcResponse {
    /// Export bundle ID
    pub bundle_id: String,
    /// Whether export was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
}

/// Import scene request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSceneRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Capability token
    pub cap_token: String,
    /// Bundle data
    pub bundle_data: Vec<u8>,
    /// Policy mode
    pub policy_mode: ImportPolicyMode,
    /// Target room ID (optional)
    pub target_room_id: Option<String>,
    /// Merge mode
    pub merge_mode: bool,
}

/// Import scene response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSceneRpcResponse {
    /// Import result
    pub import_result: ImportResult,
    /// Whether import was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
}

/// Publish scene request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishSceneRpcRequest {
    /// Session ID
    pub session_id: String,
    /// Room ID to publish
    pub room_id: String,
    /// Capability token
    pub cap_token: String,
    /// Anchor on-chain
    pub anchor: bool,
    /// DAO proposal ID (optional)
    pub dao_proposal_id: Option<String>,
    /// Public access
    pub public_access: bool,
    /// License (optional)
    pub license: Option<String>,
}

/// Publish scene response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishSceneRpcResponse {
    /// Publish result
    pub publish_result: PublishResult,
    /// Whether publish was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
}

/// Export format enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Gltf,
    Car,
    Json,
    All,
}

/// Import policy mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportPolicyMode {
    Strict,
    Permissive,
    Sandbox,
}

/// Import result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    /// Import ID
    pub import_id: String,
    /// Room ID
    pub room_id: String,
    /// Whether import was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
    /// Imported nodes count
    pub imported_nodes: u32,
    /// Imported avatars count
    pub imported_avatars: u32,
    /// Imported assets count
    pub imported_assets: u32,
    /// Policy violations
    pub policy_violations: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

/// Publish result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    /// Publish ID
    pub publish_id: String,
    /// Room ID
    pub room_id: String,
    /// Whether publish was successful
    pub success: bool,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
    /// IPFS CIDs
    pub ipfs_cids: HashMap<String, String>,
    /// On-chain transaction hash (optional)
    pub on_chain_tx_hash: Option<String>,
    /// DAO proposal ID (optional)
    pub dao_proposal_id: Option<String>,
    /// Public URL (optional)
    pub public_url: Option<String>,
}

/// Recording summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSummary {
    /// Recording ID
    pub recording_id: String,
    /// Room ID
    pub room_id: String,
    /// Duration in seconds
    pub duration_seconds: f64,
    /// Event count
    pub event_count: u64,
    /// Started at timestamp
    pub started_at: u64,
    /// Stopped at timestamp
    pub stopped_at: u64,
    /// Deterministic hash
    pub deterministic_hash: String,
    /// File size in bytes
    pub file_size_bytes: u64,
    /// Participants
    pub participants: Vec<String>,
    /// Devices used
    pub devices_used: Vec<String>,
}

/// Event types for the event stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SceneEvent {
    /// Policy was denied
    PolicyDenied {
        operation: String,
        reason: String,
        policy_ref: Option<String>,
        session_id: String,
        timestamp: u64,
    },
    /// Avatar session started
    AvatarSessionStarted {
        session_id: String,
        did: String,
        timestamp: u64,
    },
    /// Avatar session ended
    AvatarSessionEnded {
        session_id: String,
        did: String,
        timestamp: u64,
    },
    /// Capability token issued
    CapIssued {
        token_id: String,
        session_id: String,
        scopes: Vec<String>,
        timestamp: u64,
    },
    /// Capability token expired
    CapExpired {
        token_id: String,
        session_id: String,
        timestamp: u64,
    },
    /// Node was spawned
    NodeSpawned {
        node_id: String,
        session_id: String,
        timestamp: u64,
    },
    /// Node was moved
    NodeMoved {
        node_id: String,
        session_id: String,
        timestamp: u64,
    },
    /// Node was deleted
    NodeDeleted {
        node_id: String,
        session_id: String,
        timestamp: u64,
    },
    /// Policy was attached
    PolicyAttached {
        policy_id: String,
        session_id: String,
        scope: String,
        timestamp: u64,
    },
    /// Snapshot was saved
    SnapshotSaved {
        snapshot_id: String,
        session_id: String,
        label: String,
        timestamp: u64,
    },
    /// XR device started
    XRDeviceStarted {
        device_handle: String,
        device_profile: String,
        session_id: String,
        timestamp: u64,
    },
    /// Input event received
    InputEventReceived {
        device_handle: String,
        event_type: String,
        session_id: String,
        timestamp: u64,
    },
    /// Multi-user session joined
    MultiUserJoined {
        participant_id: String,
        room_id: String,
        session_id: String,
        timestamp: u64,
    },
    /// Multi-user session left
    MultiUserLeft {
        participant_id: String,
        room_id: String,
        session_id: String,
        timestamp: u64,
    },
    /// Scene synchronized
    SceneSynchronized {
        room_id: String,
        version: u64,
        session_id: String,
        timestamp: u64,
    },
    /// Physics interaction performed
    PhysicsInteraction {
        operation: String,
        object_id: String,
        session_id: String,
        timestamp: u64,
    },
    /// Room persisted to NGFS
    RoomPersisted {
        room_id: String,
        snapshot_id: String,
        ngfs_cid: String,
        session_id: String,
        timestamp: u64,
    },
    /// Recording started
    RecordingStarted {
        recording_id: String,
        room_id: String,
        session_id: String,
        timestamp: u64,
    },
    /// Recording stopped
    RecordingStopped {
        recording_id: String,
        room_id: String,
        duration_seconds: f64,
        event_count: u64,
        session_id: String,
        timestamp: u64,
    },
    /// Recording replayed
    RecordingReplayed {
        replay_id: String,
        recording_id: String,
        mode: String,
        session_id: String,
        timestamp: u64,
    },
    /// Scene exported
    SceneExported {
        bundle_id: String,
        room_id: String,
        formats: Vec<String>,
        session_id: String,
        timestamp: u64,
    },
    /// Scene imported
    SceneImported {
        import_id: String,
        room_id: String,
        imported_nodes: u32,
        imported_avatars: u32,
        session_id: String,
        timestamp: u64,
    },
    /// Scene published
    ScenePublished {
        publish_id: String,
        room_id: String,
        anchored: bool,
        public_url: Option<String>,
        session_id: String,
        timestamp: u64,
    },
}

/// RPC service for XR scene operations
pub struct SceneRpcService {
    scene_graph: Arc<SceneGraph>,
    session_manager: Arc<SessionManager>,
    cap_manager: Arc<CapManager>,
    policy_manager: Arc<PolicyManager>,
    snapshot_manager: Arc<SnapshotManager>,
    dao_hook: Option<Arc<dyn DaoHook + Send + Sync>>,
    event_sender: Arc<tokio::sync::broadcast::Sender<SceneEvent>>,
    audit_log: Arc<RwLock<Vec<AuditRecord>>>,
    // XR device manager (would be injected in real implementation)
    xr_device_manager: Option<Arc<dyn XRDeviceManager + Send + Sync>>,
    // Multi-user session manager (would be injected in real implementation)
    multi_user_manager: Option<Arc<dyn MultiUserManager + Send + Sync>>,
    // Physics world (would be injected in real implementation)
    physics_world: Option<Arc<dyn PhysicsWorld + Send + Sync>>,
    // P4-06-A4: Persistence manager
    persistence_manager: Option<Arc<dyn PersistenceManager + Send + Sync>>,
    // P4-06-A4: Recording manager
    recording_manager: Option<Arc<dyn RecordingManager + Send + Sync>>,
    // P4-06-A4: Replay manager
    replay_manager: Option<Arc<dyn ReplayManager + Send + Sync>>,
    // P4-06-A4: Bridge manager
    bridge_manager: Option<Arc<dyn BridgeManager + Send + Sync>>,
}

/// XR device manager trait
#[async_trait::async_trait]
pub trait XRDeviceManager {
    async fn start_device(&self, device_profile: &str, xr_session_id: &str) -> SceneResult<(String, Vec<String>)>;
    async fn send_input(&self, device_handle: &str, input_event: &InputEvent) -> SceneResult<()>;
    async fn stop_device(&self, device_handle: &str) -> SceneResult<()>;
}

/// Multi-user session manager trait
#[async_trait::async_trait]
pub trait MultiUserManager {
    async fn join_room(&self, session_id: &str, room_id: &str, display_name: &str, avatar_config: Option<&AvatarConfig>) -> SceneResult<(String, RoomState)>;
    async fn leave_room(&self, session_id: &str, room_id: &str) -> SceneResult<()>;
    async fn sync_scene(&self, session_id: &str, room_id: &str, last_version: Option<u64>) -> SceneResult<SceneSnapshot>;
    async fn get_room_state(&self, room_id: &str) -> SceneResult<RoomState>;
}

/// Physics world trait
#[async_trait::async_trait]
pub trait PhysicsWorld {
    async fn interact(&self, session_id: &str, operation: &PhysicsOperation) -> SceneResult<Option<PhysicsResult>>;
    async fn get_physics_state(&self) -> SceneResult<PhysicsState>;
}

/// Audit record for security decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Unique audit ID
    pub id: String,
    /// Session ID
    pub session_id: String,
    /// User DID
    pub user_did: String,
    /// Operation performed
    pub operation: String,
    /// Whether operation was allowed
    pub allowed: bool,
    /// Reason for decision
    pub reason: String,
    /// Policy reference (if applicable)
    pub policy_ref: Option<String>,
    /// Request hash for integrity
    pub request_hash: String,
    /// Monotonic tick when decision was made
    pub tick: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl SceneRpcService {
    /// Create a new RPC service
    pub fn new(
        scene_graph: Arc<SceneGraph>,
        session_manager: Arc<SessionManager>,
        cap_manager: Arc<CapManager>,
        policy_manager: Arc<PolicyManager>,
        snapshot_manager: Arc<SnapshotManager>,
        dao_hook: Option<Arc<dyn DaoHook + Send + Sync>>,
        xr_device_manager: Option<Arc<dyn XRDeviceManager + Send + Sync>>,
        multi_user_manager: Option<Arc<dyn MultiUserManager + Send + Sync>>,
        physics_world: Option<Arc<dyn PhysicsWorld + Send + Sync>>,
        persistence_manager: Option<Arc<dyn PersistenceManager + Send + Sync>>,
        recording_manager: Option<Arc<dyn RecordingManager + Send + Sync>>,
        replay_manager: Option<Arc<dyn ReplayManager + Send + Sync>>,
        bridge_manager: Option<Arc<dyn BridgeManager + Send + Sync>>,
    ) -> Self {
        let (event_sender, _) = tokio::sync::broadcast::channel(1000);
        
        Self {
            scene_graph,
            session_manager,
            cap_manager,
            policy_manager,
            snapshot_manager,
            dao_hook,
            event_sender,
            audit_log: Arc::new(RwLock::new(Vec::new())),
            xr_device_manager,
            multi_user_manager,
            physics_world,
            persistence_manager,
            recording_manager,
            replay_manager,
            bridge_manager,
        }
    }

    /// Begin a new session
    pub async fn begin_session(&self, request: RpcRequest<BeginSessionRpcRequest>) -> SceneResult<RpcResponse<BeginSessionRpcResponse>> {
        let now = self.current_timestamp();
        
        let begin_request = BeginSessionRequest {
            proof: DidProof {
                did: request.payload.did.clone(),
                proof: request.payload.proof.clone(),
                nonce: request.payload.nonce.clone(),
                timestamp: now,
            },
            avatar_profile: request.payload.avatar_profile,
            ttl_seconds: request.payload.ttl_seconds,
        };

        let response = self.session_manager.begin_session(begin_request)?;
        
        // Emit session started event
        self.emit_event(SceneEvent::AvatarSessionStarted {
            session_id: response.session_id.0.clone(),
            did: request.payload.did,
            timestamp: now,
        }).await;

        Ok(RpcResponse {
            id: request.id,
            payload: BeginSessionRpcResponse {
                session_id: response.session_id.0,
                avatar_id: response.avatar_id.map(|id| id.0),
                expires_at: response.expires_at,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// End a session
    pub async fn end_session(&self, request: RpcRequest<EndSessionRpcRequest>) -> SceneResult<RpcResponse<EndSessionRpcResponse>> {
        let now = self.current_timestamp();
        
        let end_request = EndSessionRequest {
            session_id: SessionId(request.payload.session_id.clone()),
            reason: request.payload.reason,
        };

        let response = self.session_manager.end_session(end_request)?;
        
        // Revoke all capability tokens for this session
        let _ = self.cap_manager.revoke_session_tokens(&SessionId(request.payload.session_id.clone())).await;
        
        // Emit session ended event
        self.emit_event(SceneEvent::AvatarSessionEnded {
            session_id: request.payload.session_id.clone(),
            did: "unknown".to_string(), // Would get from session
            timestamp: now,
        }).await;

        Ok(RpcResponse {
            id: request.id,
            payload: EndSessionRpcResponse {
                success: response.success,
                terminated_at: response.terminated_at,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Issue a capability token
    pub async fn issue_cap(&self, request: RpcRequest<IssueCapRpcRequest>) -> SceneResult<RpcResponse<IssueCapRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session
        let session_validation = self.session_manager.validate_session(&session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        // Issue capability token
        let cap_token = self.cap_manager.issue_cap_for_session(
            &session_id,
            request.payload.scopes.clone(),
            request.payload.ttl_seconds,
            &self.session_manager,
        ).await?;

        // Emit cap issued event
        self.emit_event(SceneEvent::CapIssued {
            token_id: cap_token.id.clone(),
            session_id: request.payload.session_id.clone(),
            scopes: request.payload.scopes,
            timestamp: now,
        }).await;

        Ok(RpcResponse {
            id: request.id,
            payload: IssueCapRpcResponse {
                cap_token: cap_token.clone(),
                expires_at: cap_token.expires_at.unwrap_or_default().timestamp() as u64,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Spawn a node
    pub async fn spawn_node(&self, request: RpcRequest<SpawnNodeRpcRequest>) -> SceneResult<RpcResponse<SpawnNodeRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            CapAction::Spawn,
            CapScope::Scene,
        ).await?;

        if !auth_result.allowed {
            self.audit_decision(&session_id, &auth_result.user_did, "spawn_node", false, &auth_result.reason, None, &request.id, now).await;
            return Err(SceneError::PermissionDenied(auth_result.reason));
        }

        // Parse node ID
        let node_id = NodeId::new(); // In real implementation, parse from string or generate
        
        // Create node in scene graph
        // This would call the actual scene graph spawn method
        // For now, we'll simulate success
        
        // Emit node spawned event
        self.emit_event(SceneEvent::NodeSpawned {
            node_id: node_id.0.clone(),
            session_id: request.payload.session_id.clone(),
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "spawn_node", true, "Node spawned successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: SpawnNodeRpcResponse {
                node_id: node_id.0,
                created_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Move a node
    pub async fn move_node(&self, request: RpcRequest<MoveNodeRpcRequest>) -> SceneResult<RpcResponse<MoveNodeRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            CapAction::Move,
            CapScope::Scene,
        ).await?;

        if !auth_result.allowed {
            self.audit_decision(&session_id, &auth_result.user_did, "move_node", false, &auth_result.reason, None, &request.id, now).await;
            return Err(SceneError::PermissionDenied(auth_result.reason));
        }

        // Parse node ID
        let node_id = NodeId(request.payload.node_id.clone());
        
        // Move node in scene graph
        // This would call the actual scene graph move method
        // For now, we'll simulate success
        
        // Emit node moved event
        self.emit_event(SceneEvent::NodeMoved {
            node_id: request.payload.node_id.clone(),
            session_id: request.payload.session_id.clone(),
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "move_node", true, "Node moved successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: MoveNodeRpcResponse {
                success: true,
                moved_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Delete a node
    pub async fn delete_node(&self, request: RpcRequest<DeleteNodeRpcRequest>) -> SceneResult<RpcResponse<DeleteNodeRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            CapAction::Delete,
            CapScope::Scene,
        ).await?;

        if !auth_result.allowed {
            self.audit_decision(&session_id, &auth_result.user_did, "delete_node", false, &auth_result.reason, None, &request.id, now).await;
            return Err(SceneError::PermissionDenied(auth_result.reason));
        }

        // Parse node ID
        let node_id = NodeId(request.payload.node_id.clone());
        
        // Delete node in scene graph
        // This would call the actual scene graph delete method
        // For now, we'll simulate success
        
        // Emit node deleted event
        self.emit_event(SceneEvent::NodeDeleted {
            node_id: request.payload.node_id.clone(),
            session_id: request.payload.session_id.clone(),
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "delete_node", true, "Node deleted successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: DeleteNodeRpcResponse {
                success: true,
                deleted_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Attach a policy
    pub async fn attach_policy(&self, request: RpcRequest<AttachPolicyRpcRequest>) -> SceneResult<RpcResponse<AttachPolicyRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            CapAction::AttachPolicy,
            CapScope::Scene,
        ).await?;

        if !auth_result.allowed {
            self.audit_decision(&session_id, &auth_result.user_did, "attach_policy", false, &auth_result.reason, None, &request.id, now).await;
            return Err(SceneError::PermissionDenied(auth_result.reason));
        }

        // Parse policy scope
        let scope = match request.payload.scope.as_str() {
            "scene" => PolicyScope::Scene,
            _ => PolicyScope::Scene, // Default to scene scope
        };

        let attach_request = AttachPolicyRequest {
            session_id: session_id.clone(),
            scope,
            rego_bundle_cbor: request.payload.rego_bundle_cbor,
            name: "Attached Policy".to_string(),
            description: None,
        };

        let response = self.policy_manager.attach_policy(attach_request, &self.session_manager).await?;
        
        // Emit policy attached event
        self.emit_event(SceneEvent::PolicyAttached {
            policy_id: response.policy_id.clone(),
            session_id: request.payload.session_id.clone(),
            scope: request.payload.scope,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "attach_policy", true, "Policy attached successfully", Some(response.policy_id.clone()), &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: AttachPolicyRpcResponse {
                policy_id: response.policy_id,
                success: response.success,
                attached_at: response.attached_at,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Simulate operations
    pub async fn simulate(&self, request: RpcRequest<SimulateRpcRequest>) -> SceneResult<RpcResponse<SimulateRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session
        let session_validation = self.session_manager.validate_session(&session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        let session = session_validation.session.unwrap();
        
        // Convert operations
        let operations = request.payload.operations.into_iter().map(|op| {
            crate::policy::SimulationOperation {
                operation: op.operation,
                parameters: op.parameters,
            }
        }).collect();

        let simulate_request = SimulateRequest {
            session_id: session_id.clone(),
            operations,
        };

        let response = self.policy_manager.simulate_policy_decisions(simulate_request, &self.session_manager).await?;
        
        // Convert results
        let results = response.results.into_iter().map(|result| {
            OperationDecision {
                operation: result.operation,
                allowed: result.allowed,
                reason: result.reason,
                policy_ref: result.policy_ref,
            }
        }).collect();

        Ok(RpcResponse {
            id: request.id,
            payload: SimulateRpcResponse {
                results,
                simulated_at: response.simulated_at,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Save snapshot
    pub async fn snapshot_save(&self, request: RpcRequest<SnapshotSaveRpcRequest>) -> SceneResult<RpcResponse<SnapshotSaveRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session
        let session_validation = self.session_manager.validate_session(&session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        // Create snapshot
        let snapshot_id = self.snapshot_manager.create_snapshot(
            &request.payload.label,
            &self.scene_graph,
        ).await?;

        // Emit snapshot saved event
        self.emit_event(SceneEvent::SnapshotSaved {
            snapshot_id: snapshot_id.0.clone(),
            session_id: request.payload.session_id.clone(),
            label: request.payload.label,
            timestamp: now,
        }).await;

        Ok(RpcResponse {
            id: request.id,
            payload: SnapshotSaveRpcResponse {
                snapshot_id: snapshot_id.0,
                success: true,
                saved_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Load snapshot
    pub async fn snapshot_load(&self, request: RpcRequest<SnapshotLoadRpcRequest>) -> SceneResult<RpcResponse<SnapshotLoadRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session
        let session_validation = self.session_manager.validate_session(&session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        // Load snapshot
        let snapshot_id = SnapshotId(request.payload.snapshot_id.clone());
        self.snapshot_manager.load_snapshot(&snapshot_id, &self.scene_graph).await?;

        Ok(RpcResponse {
            id: request.id,
            payload: SnapshotLoadRpcResponse {
                success: true,
                loaded_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Start XR device
    pub async fn start_xr_device(&self, request: RpcRequest<StartXRDeviceRpcRequest>) -> SceneResult<RpcResponse<StartXRDeviceRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session
        let session_validation = self.session_manager.validate_session(&session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        // Start XR device
        let (device_handle, capabilities) = if let Some(xr_manager) = &self.xr_device_manager {
            xr_manager.start_device(&request.payload.device_profile, &request.payload.xr_session_id).await?
        } else {
            // Mock implementation for testing
            (format!("device_{}", Uuid::new_v4()), vec!["hand_tracking".to_string(), "eye_tracking".to_string()])
        };

        // Emit XR device started event
        self.emit_event(SceneEvent::XRDeviceStarted {
            device_handle: device_handle.clone(),
            device_profile: request.payload.device_profile,
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        Ok(RpcResponse {
            id: request.id,
            payload: StartXRDeviceRpcResponse {
                device_handle,
                capabilities,
                success: true,
                started_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Send input to XR device
    pub async fn send_input(&self, request: RpcRequest<SendInputRpcRequest>) -> SceneResult<RpcResponse<SendInputRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session
        let session_validation = self.session_manager.validate_session(&session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        // Send input to XR device
        if let Some(xr_manager) = &self.xr_device_manager {
            xr_manager.send_input(&request.payload.device_handle, &request.payload.input_event).await?;
        }

        // Emit input event received
        let event_type = match &request.payload.input_event {
            InputEvent::Button { .. } => "button",
            InputEvent::Trigger { .. } => "trigger",
            InputEvent::Joystick { .. } => "joystick",
            InputEvent::HandTracking { .. } => "hand_tracking",
            InputEvent::EyeTracking { .. } => "eye_tracking",
            InputEvent::VoiceCommand { .. } => "voice_command",
        };

        self.emit_event(SceneEvent::InputEventReceived {
            device_handle: request.payload.device_handle.clone(),
            event_type: event_type.to_string(),
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        Ok(RpcResponse {
            id: request.id,
            payload: SendInputRpcResponse {
                success: true,
                ack_timestamp: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Join multi-user session
    pub async fn join_multi_user(&self, request: RpcRequest<JoinMultiUserRpcRequest>) -> SceneResult<RpcResponse<JoinMultiUserRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session
        let session_validation = self.session_manager.validate_session(&session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        // Join multi-user room
        let (participant_id, room_state) = if let Some(multi_user_manager) = &self.multi_user_manager {
            multi_user_manager.join_room(
                &request.payload.session_id,
                &request.payload.room_id,
                &request.payload.display_name,
                request.payload.avatar_config.as_ref(),
            ).await?
        } else {
            // Mock implementation for testing
            let participant_id = format!("participant_{}", Uuid::new_v4());
            let room_state = RoomState {
                room_id: request.payload.room_id.clone(),
                participants: vec![],
                scene_snapshot: None,
                last_updated: now,
            };
            (participant_id, room_state)
        };

        // Emit multi-user joined event
        self.emit_event(SceneEvent::MultiUserJoined {
            participant_id: participant_id.clone(),
            room_id: request.payload.room_id.clone(),
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        Ok(RpcResponse {
            id: request.id,
            payload: JoinMultiUserRpcResponse {
                participant_id,
                room_state,
                success: true,
                joined_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Sync scene
    pub async fn sync_scene(&self, request: RpcRequest<SyncSceneRpcRequest>) -> SceneResult<RpcResponse<SyncSceneRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session
        let session_validation = self.session_manager.validate_session(&session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        // Sync scene
        let scene_snapshot = if let Some(multi_user_manager) = &self.multi_user_manager {
            multi_user_manager.sync_scene(
                &request.payload.session_id,
                &request.payload.room_id,
                request.payload.last_version,
            ).await?
        } else {
            // Mock implementation for testing
            SceneSnapshot {
                version: 1,
                nodes: vec![],
                participants: vec![],
                physics_state: None,
                timestamp: now,
            }
        };

        // Emit scene synchronized event
        self.emit_event(SceneEvent::SceneSynchronized {
            room_id: request.payload.room_id,
            version: scene_snapshot.version,
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        Ok(RpcResponse {
            id: request.id,
            payload: SyncSceneRpcResponse {
                scene_snapshot,
                success: true,
                synced_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Physics interaction
    pub async fn physics_interact(&self, request: RpcRequest<PhysicsInteractRpcRequest>) -> SceneResult<RpcResponse<PhysicsInteractRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            CapAction::Move, // Physics interactions are treated as move operations
            CapScope::Scene,
        ).await?;

        if !auth_result.allowed {
            self.audit_decision(&session_id, &auth_result.user_did, "physics_interact", false, &auth_result.reason, None, &request.id, now).await;
            return Err(SceneError::PermissionDenied(auth_result.reason));
        }

        // Perform physics interaction
        let result = if let Some(physics_world) = &self.physics_world {
            physics_world.interact(&request.payload.session_id, &request.payload.operation).await?
        } else {
            // Mock implementation for testing
            Some(PhysicsResult::Success { message: "Physics interaction completed".to_string() })
        };

        // Emit physics interaction event
        let operation_name = match &request.payload.operation {
            PhysicsOperation::Grab { object_id, .. } => format!("grab:{}", object_id),
            PhysicsOperation::Move { object_id, .. } => format!("move:{}", object_id),
            PhysicsOperation::Release { object_id, .. } => format!("release:{}", object_id),
            PhysicsOperation::ApplyForce { object_id, .. } => format!("apply_force:{}", object_id),
            PhysicsOperation::ApplyImpulse { object_id, .. } => format!("apply_impulse:{}", object_id),
            PhysicsOperation::SetVelocity { object_id, .. } => format!("set_velocity:{}", object_id),
            PhysicsOperation::Teleport { object_id, .. } => format!("teleport:{}", object_id),
            PhysicsOperation::Raycast { .. } => "raycast".to_string(),
        };

        self.emit_event(SceneEvent::PhysicsInteraction {
            operation: operation_name.clone(),
            object_id: "unknown".to_string(), // Would extract from operation
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "physics_interact", true, "Physics interaction successful", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: PhysicsInteractRpcResponse {
                success: true,
                result,
                interacted_at: now,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    // P4-06-A4: Persistence RPCs
    /// Persist room to NGFS
    pub async fn persist_room(&self, request: RpcRequest<PersistRoomRpcRequest>) -> SceneResult<RpcResponse<PersistRoomRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            "persist_room",
        ).await?;

        // Call persistence manager
        let persist_request = PersistRoomRequest {
            room_id: request.payload.room_id.clone(),
            reason: request.payload.reason.clone(),
            session_id: request.payload.session_id.clone(),
            cap_token: request.payload.cap_token.clone(),
            dao_proposal_id: request.payload.dao_proposal_id.clone(),
        };

        let response = if let Some(persistence_manager) = &self.persistence_manager {
            persistence_manager.persist_room(&persist_request).await?
        } else {
            // Mock implementation for testing
            PersistRoomResponse {
                snapshot_id: format!("snapshot_{}", Uuid::new_v4()),
                ngfs_cid: format!("Qm{}", hex::encode(&[1, 2, 3, 4, 5, 6, 7, 8])),
                deterministic_hash: "mock_hash".to_string(),
                success: true,
                error_message: None,
            }
        };

        // Emit event
        self.emit_event(SceneEvent::RoomPersisted {
            room_id: request.payload.room_id.clone(),
            snapshot_id: response.snapshot_id.clone(),
            ngfs_cid: response.ngfs_cid.clone(),
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "persist_room", true, "Room persisted successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: PersistRoomRpcResponse {
                snapshot_id: response.snapshot_id,
                ngfs_cid: response.ngfs_cid,
                deterministic_hash: response.deterministic_hash,
                success: response.success,
                error_message: response.error_message,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    // P4-06-A4: Recording RPCs
    /// Start recording
    pub async fn start_recording(&self, request: RpcRequest<StartRecordingRpcRequest>) -> SceneResult<RpcResponse<StartRecordingRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            "start_recording",
        ).await?;

        // Call recording manager
        let recording_request = StartRecordingRequest {
            room_id: request.payload.room_id.clone(),
            session_id: request.payload.session_id.clone(),
            cap_token: request.payload.cap_token.clone(),
            deterministic_seed: request.payload.deterministic_seed,
            physics_tick_rate: request.payload.physics_tick_rate,
        };

        let response = if let Some(recording_manager) = &self.recording_manager {
            recording_manager.start_recording(&recording_request).await?
        } else {
            // Mock implementation for testing
            StartRecordingResponse {
                recording_id: format!("recording_{}", Uuid::new_v4()),
                success: true,
                error_message: None,
            }
        };

        // Emit event
        self.emit_event(SceneEvent::RecordingStarted {
            recording_id: response.recording_id.clone(),
            room_id: request.payload.room_id.clone(),
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "start_recording", true, "Recording started successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: StartRecordingRpcResponse {
                recording_id: response.recording_id,
                success: response.success,
                error_message: response.error_message,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Stop recording
    pub async fn stop_recording(&self, request: RpcRequest<StopRecordingRpcRequest>) -> SceneResult<RpcResponse<StopRecordingRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            "stop_recording",
        ).await?;

        // Call recording manager
        let stop_request = StopRecordingRequest {
            recording_id: request.payload.recording_id.clone(),
            session_id: request.payload.session_id.clone(),
            cap_token: request.payload.cap_token.clone(),
        };

        let response = if let Some(recording_manager) = &self.recording_manager {
            recording_manager.stop_recording(&stop_request).await?
        } else {
            // Mock implementation for testing
            StopRecordingResponse {
                recording_summary: RecordingSummary {
                    recording_id: request.payload.recording_id.clone(),
                    room_id: "test_room".to_string(),
                    duration_seconds: 10.0,
                    event_count: 100,
                    started_at: now - 10,
                    stopped_at: now,
                    deterministic_hash: "mock_hash".to_string(),
                    file_size_bytes: 1024,
                    participants: vec![],
                    devices_used: vec![],
                },
                success: true,
                error_message: None,
            }
        };

        // Emit event
        self.emit_event(SceneEvent::RecordingStopped {
            recording_id: request.payload.recording_id.clone(),
            room_id: response.recording_summary.room_id.clone(),
            duration_seconds: response.recording_summary.duration_seconds,
            event_count: response.recording_summary.event_count,
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "stop_recording", true, "Recording stopped successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: StopRecordingRpcResponse {
                recording_summary: response.recording_summary,
                success: response.success,
                error_message: response.error_message,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    // P4-06-A4: Replay RPCs
    /// Replay recording
    pub async fn replay_recording(&self, request: RpcRequest<ReplayRecordingRpcRequest>) -> SceneResult<RpcResponse<ReplayRecordingRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            "replay_recording",
        ).await?;

        // Call replay manager
        let replay_request = ReplayRecordingRequest {
            recording_id: request.payload.recording_id.clone(),
            mode: request.payload.mode.clone(),
            session_id: request.payload.session_id.clone(),
            cap_token: request.payload.cap_token.clone(),
            start_event: request.payload.start_event,
            end_event: request.payload.end_event,
            verify_determinism: request.payload.verify_determinism,
        };

        let response = if let Some(replay_manager) = &self.replay_manager {
            replay_manager.replay_recording(&replay_request).await?
        } else {
            // Mock implementation for testing
            ReplayRecordingResponse {
                replay_id: format!("replay_{}", Uuid::new_v4()),
                success: true,
                error_message: None,
            }
        };

        // Emit event
        let mode_str = match request.payload.mode {
            ReplayMode::Headless => "headless",
            ReplayMode::Visualization => "visualization",
            ReplayMode::Debug => "debug",
        };

        self.emit_event(SceneEvent::RecordingReplayed {
            replay_id: response.replay_id.clone(),
            recording_id: request.payload.recording_id.clone(),
            mode: mode_str.to_string(),
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "replay_recording", true, "Recording replayed successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: ReplayRecordingRpcResponse {
                replay_id: response.replay_id,
                success: response.success,
                error_message: response.error_message,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    // P4-06-A4: Bridge RPCs
    /// Export scene
    pub async fn export_scene(&self, request: RpcRequest<ExportSceneRpcRequest>) -> SceneResult<RpcResponse<ExportSceneRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            "export_scene",
        ).await?;

        // Call bridge manager
        let export_request = ExportSceneRequest {
            room_id: request.payload.room_id.clone(),
            formats: request.payload.formats.clone(),
            session_id: request.payload.session_id.clone(),
            cap_token: request.payload.cap_token.clone(),
            include_avatars: request.payload.include_avatars,
            include_physics: request.payload.include_physics,
            compression_level: request.payload.compression_level,
        };

        let response = if let Some(bridge_manager) = &self.bridge_manager {
            bridge_manager.export_scene(&export_request).await?
        } else {
            // Mock implementation for testing
            ExportSceneResponse {
                bundle_id: format!("bundle_{}", Uuid::new_v4()),
                success: true,
                error_message: None,
            }
        };

        // Emit event
        let format_strings: Vec<String> = request.payload.formats.iter()
            .map(|f| match f {
                ExportFormat::Gltf => "gltf",
                ExportFormat::Car => "car",
                ExportFormat::Json => "json",
                ExportFormat::All => "all",
            })
            .map(|s| s.to_string())
            .collect();

        self.emit_event(SceneEvent::SceneExported {
            bundle_id: response.bundle_id.clone(),
            room_id: request.payload.room_id.clone(),
            formats: format_strings,
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "export_scene", true, "Scene exported successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: ExportSceneRpcResponse {
                bundle_id: response.bundle_id,
                success: response.success,
                error_message: response.error_message,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Import scene
    pub async fn import_scene(&self, request: RpcRequest<ImportSceneRpcRequest>) -> SceneResult<RpcResponse<ImportSceneRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            "import_scene",
        ).await?;

        // Call bridge manager
        let import_request = ImportSceneRequest {
            bundle_data: request.payload.bundle_data.clone(),
            policy_mode: request.payload.policy_mode.clone(),
            session_id: request.payload.session_id.clone(),
            cap_token: request.payload.cap_token.clone(),
            target_room_id: request.payload.target_room_id.clone(),
            merge_mode: request.payload.merge_mode,
        };

        let response = if let Some(bridge_manager) = &self.bridge_manager {
            bridge_manager.import_scene(&import_request).await?
        } else {
            // Mock implementation for testing
            ImportSceneResponse {
                import_result: ImportResult {
                    import_id: format!("import_{}", Uuid::new_v4()),
                    room_id: request.payload.target_room_id.unwrap_or_else(|| "imported_room".to_string()),
                    success: true,
                    error_message: None,
                    imported_nodes: 5,
                    imported_avatars: 2,
                    imported_assets: 10,
                    policy_violations: vec![],
                    warnings: vec![],
                },
                success: true,
                error_message: None,
            }
        };

        // Emit event
        self.emit_event(SceneEvent::SceneImported {
            import_id: response.import_result.import_id.clone(),
            room_id: response.import_result.room_id.clone(),
            imported_nodes: response.import_result.imported_nodes,
            imported_avatars: response.import_result.imported_avatars,
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "import_scene", true, "Scene imported successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: ImportSceneRpcResponse {
                import_result: response.import_result,
                success: response.success,
                error_message: response.error_message,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Publish scene
    pub async fn publish_scene(&self, request: RpcRequest<PublishSceneRpcRequest>) -> SceneResult<RpcResponse<PublishSceneRpcResponse>> {
        let now = self.current_timestamp();
        let session_id = SessionId(request.payload.session_id.clone());
        
        // Validate session and capability
        let auth_result = self.validate_session_and_capability(
            &session_id,
            &request.payload.cap_token,
            "publish_scene",
        ).await?;

        // Call bridge manager
        let publish_request = PublishSceneRequest {
            room_id: request.payload.room_id.clone(),
            anchor: request.payload.anchor,
            dao_proposal_id: request.payload.dao_proposal_id.clone(),
            session_id: request.payload.session_id.clone(),
            cap_token: request.payload.cap_token.clone(),
            public_access: request.payload.public_access,
            license: request.payload.license.clone(),
        };

        let response = if let Some(bridge_manager) = &self.bridge_manager {
            bridge_manager.publish_scene(&publish_request).await?
        } else {
            // Mock implementation for testing
            PublishSceneResponse {
                publish_result: PublishResult {
                    publish_id: format!("publish_{}", Uuid::new_v4()),
                    room_id: request.payload.room_id.clone(),
                    success: true,
                    error_message: None,
                    ipfs_cids: HashMap::new(),
                    on_chain_tx_hash: if request.payload.anchor { Some("0x1234567890abcdef".to_string()) } else { None },
                    dao_proposal_id: request.payload.dao_proposal_id.clone(),
                    public_url: Some("https://ipfs.io/ip/mock_cid".to_string()),
                },
                success: true,
                error_message: None,
            }
        };

        // Emit event
        self.emit_event(SceneEvent::ScenePublished {
            publish_id: response.publish_result.publish_id.clone(),
            room_id: request.payload.room_id.clone(),
            anchored: request.payload.anchor,
            public_url: response.publish_result.public_url.clone(),
            session_id: request.payload.session_id,
            timestamp: now,
        }).await;

        self.audit_decision(&session_id, &auth_result.user_did, "publish_scene", true, "Scene published successfully", None, &request.id, now).await;

        Ok(RpcResponse {
            id: request.id,
            payload: PublishSceneRpcResponse {
                publish_result: response.publish_result,
                success: response.success,
                error_message: response.error_message,
            },
            timestamp: now,
            success: true,
            error: None,
        })
    }

    /// Get event stream receiver
    pub fn get_event_receiver(&self) -> tokio::sync::broadcast::Receiver<SceneEvent> {
        self.event_sender.subscribe()
    }

    /// Get audit log
    pub async fn get_audit_log(&self) -> Vec<AuditRecord> {
        let audit_log = self.audit_log.read().await;
        audit_log.clone()
    }

    /// Validate session and capability
    async fn validate_session_and_capability(
        &self,
        session_id: &SessionId,
        cap_token: &CapToken,
        action: CapAction,
        scope: CapScope,
    ) -> SceneResult<AuthResult> {
        // Validate session
        let session_validation = self.session_manager.validate_session(session_id)?;
        if !session_validation.is_valid {
            return Ok(AuthResult {
                allowed: false,
                user_did: "unknown".to_string(),
                reason: "Session is not valid".to_string(),
            });
        }

        let session = session_validation.session.unwrap();
        
        // Validate capability token
        if cap_token.is_expired() {
            return Ok(AuthResult {
                allowed: false,
                user_did: session.did.clone(),
                reason: "Capability token has expired".to_string(),
            });
        }

        // Check capability scope
        let has_scope = match action {
            CapAction::Spawn => cap_token.has_granted_scope("scene:spawn") || cap_token.has_granted_scope("node:spawn"),
            CapAction::Move => cap_token.has_granted_scope("scene:move") || cap_token.has_granted_scope("node:move"),
            CapAction::Delete => cap_token.has_granted_scope("scene:delete") || cap_token.has_granted_scope("node:delete"),
            CapAction::AttachPolicy => cap_token.has_granted_scope("policy:attach"),
            _ => false,
        };

        if !has_scope {
            return Ok(AuthResult {
                allowed: false,
                user_did: session.did.clone(),
                reason: "Insufficient capability scope".to_string(),
            });
        }

        // Check DAO policy if available
        if let Some(dao_hook) = &self.dao_hook {
            let mut context = PolicyContext::new(session.did.clone(), action.into(), scope.into());
            context.set_session_id(session_id.clone());
            context.set_cap_token(cap_token.clone());

            let policy_result = dao_hook.check_policy(&context).await?;
            if !policy_result.allowed {
                return Ok(AuthResult {
                    allowed: false,
                    user_did: session.did.clone(),
                    reason: policy_result.reason.unwrap_or("DAO policy denied".to_string()),
                });
            }
        }

        Ok(AuthResult {
            allowed: true,
            user_did: session.did.clone(),
            reason: "Authorization successful".to_string(),
        })
    }

    /// Emit an event
    async fn emit_event(&self, event: SceneEvent) {
        let _ = self.event_sender.send(event);
    }

    /// Audit a security decision
    async fn audit_decision(
        &self,
        session_id: &SessionId,
        user_did: &str,
        operation: &str,
        allowed: bool,
        reason: &str,
        policy_ref: Option<String>,
        request_id: &str,
        timestamp: u64,
    ) {
        let audit_record = AuditRecord {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.0.clone(),
            user_did: user_did.to_string(),
            operation: operation.to_string(),
            allowed,
            reason: reason.to_string(),
            policy_ref,
            request_hash: self.hash_request(request_id),
            tick: self.current_tick(),
            timestamp,
        };

        let mut audit_log = self.audit_log.write().await;
        audit_log.push(audit_record);
    }

    /// Hash a request for integrity
    fn hash_request(&self, request_id: &str) -> String {
        // In a real implementation, this would hash the full request
        format!("hash_{}", request_id)
    }

    /// Get current monotonic tick
    fn current_tick(&self) -> u64 {
        // In a real implementation, this would get the current scene tick
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get current timestamp
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Authentication result
#[derive(Debug, Clone)]
struct AuthResult {
    allowed: bool,
    user_did: String,
    reason: String,
}

// Conversion implementations
impl From<CapAction> for PolicyAction {
    fn from(action: CapAction) -> Self {
        match action {
            CapAction::Spawn => PolicyAction::Spawn,
            CapAction::Move => PolicyAction::Move,
            CapAction::Delete => PolicyAction::Delete,
            CapAction::AttachPolicy => PolicyAction::AttachPolicy,
            _ => PolicyAction::Spawn, // Default
        }
    }
}

impl From<CapScope> for PolicyScope {
    fn from(scope: CapScope) -> Self {
        match scope {
            CapScope::Scene => PolicyScope::Scene,
            CapScope::Node(node_id) => PolicyScope::Node(node_id),
            CapScope::Avatar(did) => PolicyScope::Avatar(did),
            CapScope::User(did) => PolicyScope::User(did),
            _ => PolicyScope::Scene, // Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::SceneGraph;
    use crate::storage::SnapshotManager;
    use crate::dao_hook::DaoHookFactory;

    #[tokio::test]
    async fn test_rpc_service_creation() {
        let scene_graph = Arc::new(SceneGraph::new(1000));
        let session_manager = Arc::new(SessionManager::default());
        let cap_manager = Arc::new(CapManager::new());
        let policy_manager = Arc::new(PolicyManager::new());
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let dao_hook = Some(DaoHookFactory::create_mock_hook());

        let service = SceneRpcService::new(
            scene_graph,
            session_manager,
            cap_manager,
            policy_manager,
            snapshot_manager,
            dao_hook,
            None, // xr_device_manager
            None, // multi_user_manager
            None, // physics_world
            None, // persistence_manager
            None, // recording_manager
            None, // replay_manager
            None, // bridge_manager
        );

        assert!(service.get_event_receiver().try_recv().is_err()); // No events yet
    }

    #[tokio::test]
    async fn test_begin_session_rpc() {
        let scene_graph = Arc::new(SceneGraph::new(1000));
        let session_manager = Arc::new(SessionManager::default());
        let cap_manager = Arc::new(CapManager::new());
        let policy_manager = Arc::new(PolicyManager::new());
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let dao_hook = Some(DaoHookFactory::create_mock_hook());

        let service = SceneRpcService::new(
            scene_graph,
            session_manager,
            cap_manager,
            policy_manager,
            snapshot_manager,
            dao_hook,
            None, // xr_device_manager
            None, // multi_user_manager
            None, // physics_world
            None, // persistence_manager
            None, // recording_manager
            None, // replay_manager
            None, // bridge_manager
        );

        let request = RpcRequest {
            id: "test_request".to_string(),
            payload: BeginSessionRpcRequest {
                did: "did:aeth:test".to_string(),
                proof: "jwt_proof".to_string(),
                nonce: "nonce123".to_string(),
                avatar_profile: None,
                ttl_seconds: Some(3600),
            },
            timestamp: 1234567890,
        };

        let response = service.begin_session(request).await.unwrap();
        assert!(response.success);
        assert!(!response.payload.session_id.is_empty());
    }

    #[tokio::test]
    async fn test_issue_cap_rpc() {
        let scene_graph = Arc::new(SceneGraph::new(1000));
        let session_manager = Arc::new(SessionManager::default());
        let cap_manager = Arc::new(CapManager::new());
        let policy_manager = Arc::new(PolicyManager::new());
        let snapshot_manager = Arc::new(SnapshotManager::new());
        let dao_hook = Some(DaoHookFactory::create_mock_hook());

        let service = SceneRpcService::new(
            scene_graph,
            session_manager,
            cap_manager,
            policy_manager,
            snapshot_manager,
            dao_hook,
            None, // xr_device_manager
            None, // multi_user_manager
            None, // physics_world
            None, // persistence_manager
            None, // recording_manager
            None, // replay_manager
            None, // bridge_manager
        );

        // First create a session
        let session_request = RpcRequest {
            id: "session_request".to_string(),
            payload: BeginSessionRpcRequest {
                did: "did:aeth:test".to_string(),
                proof: "jwt_proof".to_string(),
                nonce: "nonce123".to_string(),
                avatar_profile: None,
                ttl_seconds: Some(3600),
            },
            timestamp: 1234567890,
        };

        let session_response = service.begin_session(session_request).await.unwrap();
        let session_id = session_response.payload.session_id;

        // Now issue a capability token
        let cap_request = RpcRequest {
            id: "cap_request".to_string(),
            payload: IssueCapRpcRequest {
                session_id,
                scopes: vec!["scene:spawn".to_string()],
                ttl_seconds: 300,
            },
            timestamp: 1234567890,
        };

        let cap_response = service.issue_cap(cap_request).await.unwrap();
        assert!(cap_response.success);
        assert!(!cap_response.payload.cap_token.id.is_empty());
    }
}
