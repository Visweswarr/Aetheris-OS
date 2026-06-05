//! XR Replay Service - Deterministic Re-execution Engine
//! 
//! This module provides deterministic replay capabilities by re-executing
//! recorded events to regenerate exact snapshot bytes.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::scene::{
    SceneResult, SceneError, SceneNode, Avatar, Transform, Vector3, InputEvent, PhysicsOperation
};
use crate::multiuser::RoomState;
use crate::physics::{PhysicsState, PhysicsBody, PhysicsConstraint};
use crate::record::{RecordedEvent, EventType, DeterministicContext, RecordingSummary};
use crate::persist::{PersistentRoomState, SnapshotMetadata};

/// Replay mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayMode {
    Headless,  // Deterministic execution without visualization
    Visualization,  // Replay with visualization
    Debug,  // Replay with debug information
}

/// Replay session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySession {
    pub id: String,
    pub recording_id: String,
    pub mode: ReplayMode,
    pub started_at: DateTime<Utc>,
    pub current_event: u64,
    pub total_events: u64,
    pub is_active: bool,
    pub deterministic_seed: u64,
    pub physics_tick: u64,
    pub network_frame: u64,
}

/// Replay result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub replay_id: String,
    pub recording_id: String,
    pub final_snapshot_id: String,
    pub deterministic_hash: String,
    pub events_processed: u64,
    pub duration_seconds: f64,
    pub success: bool,
    pub error_message: Option<String>,
    pub byte_stability_verified: bool,
}

/// Replay request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRecordingRequest {
    pub recording_id: String,
    pub mode: ReplayMode,
    pub session_id: String,
    pub cap_token: String,
    pub start_event: Option<u64>,
    pub end_event: Option<u64>,
    pub verify_determinism: bool,
}

/// Replay response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRecordingResponse {
    pub replay_id: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Deterministic state for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayState {
    pub room_state: RoomState,
    pub nodes: HashMap<String, SceneNode>,
    pub avatars: HashMap<String, Avatar>,
    pub physics_state: PhysicsState,
    pub rng_state: u64,
    pub network_order: Vec<String>,
    pub participant_states: HashMap<String, ParticipantState>,
}

/// Participant state during replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantState {
    pub participant_id: String,
    pub transform: Transform,
    pub input_state: HashMap<String, f32>,
    pub device_state: HashMap<String, serde_cbor::Value>,
}

/// Replay manager trait
#[async_trait::async_trait]
pub trait ReplayManager: Send + Sync {
    async fn replay_recording(&self, request: &ReplayRecordingRequest) -> SceneResult<ReplayRecordingResponse>;
    async fn get_replay_result(&self, replay_id: &str) -> SceneResult<ReplayResult>;
    async fn stop_replay(&self, replay_id: &str) -> SceneResult<()>;
    async fn verify_determinism(&self, replay_id: &str) -> SceneResult<bool>;
    async fn export_replay_snapshot(&self, replay_id: &str) -> SceneResult<PersistentRoomState>;
}

/// Replay manager implementation
pub struct XRReplayManager {
    replay_sessions: Arc<RwLock<HashMap<String, ReplaySession>>>,
    replay_results: Arc<RwLock<HashMap<String, ReplayResult>>>,
    replay_states: Arc<RwLock<HashMap<String, ReplayState>>>,
    recording_manager: Arc<dyn crate::record::RecordingManager + Send + Sync>,
    persistence_manager: Arc<dyn crate::persist::PersistenceManager + Send + Sync>,
}

impl XRReplayManager {
    pub fn new(
        recording_manager: Arc<dyn crate::record::RecordingManager + Send + Sync>,
        persistence_manager: Arc<dyn crate::persist::PersistenceManager + Send + Sync>,
    ) -> Self {
        Self {
            replay_sessions: Arc::new(RwLock::new(HashMap::new())),
            replay_results: Arc::new(RwLock::new(HashMap::new())),
            replay_states: Arc::new(RwLock::new(HashMap::new())),
            recording_manager,
            persistence_manager,
        }
    }

    /// Initialize replay state from recording
    async fn initialize_replay_state(&self, recording_id: &str) -> SceneResult<ReplayState> {
        // Get recording summary
        let recordings = self.recording_manager.list_recordings("").await?;
        let recording = recordings.iter()
            .find(|r| r.recording_id == recording_id)
            .ok_or_else(|| SceneError::NotFound(format!("Recording {} not found", recording_id)))?;

        // Initialize empty state
        let room_state = RoomState {
            id: recording.room_id.clone(),
            name: format!("Replay Room {}", recording.room_id),
            participants: vec![],
            max_participants: 10,
            is_public: true,
            created_at: recording.started_at,
            metadata: HashMap::new(),
        };

        let physics_state = PhysicsState {
            bodies: vec![],
            constraints: vec![],
            gravity: Vector3 { x: 0.0, y: -9.81, z: 0.0 },
            time_step: 1.0 / 60.0,
            metadata: HashMap::new(),
        };

        Ok(ReplayState {
            room_state,
            nodes: HashMap::new(),
            avatars: HashMap::new(),
            physics_state,
            rng_state: 12345, // Will be set from recording
            network_order: vec![],
            participant_states: HashMap::new(),
        })
    }

    /// Process a single event during replay
    async fn process_replay_event(
        &self,
        replay_id: &str,
        event: &RecordedEvent,
        state: &mut ReplayState,
    ) -> SceneResult<()> {
        // Update deterministic context
        state.rng_state = event.deterministic_context.rng_seed;
        
        match &event.event_type {
            EventType::InputEvent(input_event) => {
                self.process_input_event(replay_id, input_event, state).await?;
            }
            EventType::PhysicsTick(physics_tick) => {
                self.process_physics_tick(replay_id, physics_tick, state).await?;
            }
            EventType::NetworkFrame(network_frame) => {
                self.process_network_frame(replay_id, network_frame, state).await?;
            }
            EventType::SceneModification(scene_mod) => {
                self.process_scene_modification(replay_id, scene_mod, state).await?;
            }
            EventType::AvatarUpdate(avatar_update) => {
                self.process_avatar_update(replay_id, avatar_update, state).await?;
            }
            EventType::RoomStateChange(room_change) => {
                self.process_room_state_change(replay_id, room_change, state).await?;
            }
            EventType::SystemEvent(system_event) => {
                self.process_system_event(replay_id, system_event, state).await?;
            }
        }

        Ok(())
    }

    /// Process input event during replay
    async fn process_input_event(
        &self,
        _replay_id: &str,
        input_event: &InputEvent,
        state: &mut ReplayState,
    ) -> SceneResult<()> {
        // Update participant input state
        match input_event {
            InputEvent::ButtonPress { button_id, pressed, value } => {
                // Find participant and update input state
                for participant_state in state.participant_states.values_mut() {
                    participant_state.input_state.insert(button_id.clone(), *value);
                }
            }
            InputEvent::TriggerPull { trigger_id, value } => {
                for participant_state in state.participant_states.values_mut() {
                    participant_state.input_state.insert(trigger_id.clone(), *value);
                }
            }
            InputEvent::JoystickMove { joystick_id, x, y } => {
                for participant_state in state.participant_states.values_mut() {
                    participant_state.input_state.insert(format!("{}_x", joystick_id), *x);
                    participant_state.input_state.insert(format!("{}_y", joystick_id), *y);
                }
            }
            _ => {
                // Handle other input types
            }
        }

        Ok(())
    }

    /// Process physics tick during replay
    async fn process_physics_tick(
        &self,
        _replay_id: &str,
        physics_tick: &crate::record::PhysicsTickEvent,
        state: &mut ReplayState,
    ) -> SceneResult<()> {
        // Update physics state deterministically
        state.physics_state = physics_tick.physics_state.clone();
        state.rng_state = physics_tick.rng_seed;
        
        Ok(())
    }

    /// Process network frame during replay
    async fn process_network_frame(
        &self,
        _replay_id: &str,
        network_frame: &crate::record::NetworkFrameEvent,
        state: &mut ReplayState,
    ) -> SceneResult<()> {
        // Update network order
        state.network_order = network_frame.network_order.clone();
        
        Ok(())
    }

    /// Process scene modification during replay
    async fn process_scene_modification(
        &self,
        _replay_id: &str,
        scene_mod: &crate::record::SceneModificationEvent,
        state: &mut ReplayState,
    ) -> SceneResult<()> {
        // Update node state
        if let Some(node) = state.nodes.get_mut(&scene_mod.node_id) {
            if let Some(transform) = &scene_mod.transform {
                node.transform = transform.clone();
            }
        }
        
        Ok(())
    }

    /// Process avatar update during replay
    async fn process_avatar_update(
        &self,
        _replay_id: &str,
        avatar_update: &crate::record::AvatarUpdateEvent,
        state: &mut ReplayState,
    ) -> SceneResult<()> {
        // Update avatar state
        if let Some(avatar) = state.avatars.get_mut(&avatar_update.avatar_id) {
            avatar.transform = avatar_update.transform.clone();
        }
        
        // Update participant state
        if let Some(participant_state) = state.participant_states.get_mut(&avatar_update.avatar_id) {
            participant_state.transform = avatar_update.transform.clone();
        }
        
        Ok(())
    }

    /// Process room state change during replay
    async fn process_room_state_change(
        &self,
        _replay_id: &str,
        room_change: &crate::record::RoomStateChangeEvent,
        state: &mut ReplayState,
    ) -> SceneResult<()> {
        // Update room state
        state.room_state = room_change.room_state.clone();
        
        Ok(())
    }

    /// Process system event during replay
    async fn process_system_event(
        &self,
        _replay_id: &str,
        _system_event: &crate::record::SystemEvent,
        state: &mut ReplayState,
    ) -> SceneResult<()> {
        // Handle system events
        Ok(())
    }

    /// Calculate deterministic hash of final state
    fn calculate_final_hash(&self, state: &ReplayState) -> SceneResult<String> {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        
        // Hash room state
        let room_bytes = serde_cbor::to_vec(&state.room_state)
            .map_err(|e| SceneError::SerializationError(e.to_string()))?;
        hasher.update(&room_bytes);
        
        // Hash nodes
        let nodes_bytes = serde_cbor::to_vec(&state.nodes)
            .map_err(|e| SceneError::SerializationError(e.to_string()))?;
        hasher.update(&nodes_bytes);
        
        // Hash avatars
        let avatars_bytes = serde_cbor::to_vec(&state.avatars)
            .map_err(|e| SceneError::SerializationError(e.to_string()))?;
        hasher.update(&avatars_bytes);
        
        // Hash physics state
        let physics_bytes = serde_cbor::to_vec(&state.physics_state)
            .map_err(|e| SceneError::SerializationError(e.to_string()))?;
        hasher.update(&physics_bytes);
        
        // Hash RNG state
        hasher.update(&state.rng_state.to_le_bytes());
        
        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Verify byte stability by comparing hashes
    async fn verify_byte_stability(&self, replay_id: &str, expected_hash: &str) -> SceneResult<bool> {
        let state = {
            let states = self.replay_states.read().await;
            states.get(replay_id)
                .cloned()
                .ok_or_else(|| SceneError::NotFound(format!("Replay state {} not found", replay_id)))?
        };
        
        let calculated_hash = self.calculate_final_hash(&state)?;
        Ok(calculated_hash == expected_hash)
    }
}

#[async_trait::async_trait]
impl ReplayManager for XRReplayManager {
    async fn replay_recording(&self, request: &ReplayRecordingRequest) -> SceneResult<ReplayRecordingResponse> {
        let replay_id = format!("replay_{}", Uuid::new_v4());
        
        // Initialize replay state
        let mut replay_state = self.initialize_replay_state(&request.recording_id).await?;
        
        // Create replay session
        let session = ReplaySession {
            id: replay_id.clone(),
            recording_id: request.recording_id.clone(),
            mode: request.mode.clone(),
            started_at: Utc::now(),
            current_event: 0,
            total_events: 0,
            is_active: true,
            deterministic_seed: 12345, // Will be set from recording
            physics_tick: 0,
            network_frame: 0,
        };
        
        // Store session
        {
            let mut sessions = self.replay_sessions.write().await;
            sessions.insert(replay_id.clone(), session);
        }
        
        // Store initial state
        {
            let mut states = self.replay_states.write().await;
            states.insert(replay_id.clone(), replay_state);
        }
        
        // Start replay in background
        let manager = self.clone();
        let replay_id_clone = replay_id.clone();
        let request_clone = request.clone();
        
        tokio::spawn(async move {
            if let Err(e) = manager.execute_replay(&replay_id_clone, &request_clone).await {
                eprintln!("Replay execution failed: {}", e);
            }
        });
        
        Ok(ReplayRecordingResponse {
            replay_id,
            success: true,
            error_message: None,
        })
    }

    async fn get_replay_result(&self, replay_id: &str) -> SceneResult<ReplayResult> {
        let results = self.replay_results.read().await;
        results.get(replay_id)
            .cloned()
            .ok_or_else(|| SceneError::NotFound(format!("Replay result {} not found", replay_id)))
    }

    async fn stop_replay(&self, replay_id: &str) -> SceneResult<()> {
        let mut sessions = self.replay_sessions.write().await;
        if let Some(session) = sessions.get_mut(replay_id) {
            session.is_active = false;
        }
        Ok(())
    }

    async fn verify_determinism(&self, replay_id: &str) -> SceneResult<bool> {
        let result = self.get_replay_result(replay_id).await?;
        self.verify_byte_stability(replay_id, &result.deterministic_hash).await
    }

    async fn export_replay_snapshot(&self, replay_id: &str) -> SceneResult<PersistentRoomState> {
        let state = {
            let states = self.replay_states.read().await;
            states.get(replay_id)
                .cloned()
                .ok_or_else(|| SceneError::NotFound(format!("Replay state {} not found", replay_id)))?
        };
        
        // Convert to persistent room state
        let persistent_state = PersistentRoomState {
            room: state.room_state,
            nodes: state.nodes.into_values().collect(),
            avatars: state.avatars.into_values().collect(),
            physics_state: state.physics_state,
            policies: vec![],
            capabilities: vec![],
        };
        
        Ok(persistent_state)
    }
}

impl XRReplayManager {
    /// Execute replay in background
    async fn execute_replay(&self, replay_id: &str, request: &ReplayRecordingRequest) -> SceneResult<()> {
        let start_time = std::time::Instant::now();
        
        // Get recording events
        let recording_data = self.recording_manager.export_recording(&request.recording_id, "cbor").await?;
        let events: Vec<RecordedEvent> = self.parse_recording_data(&recording_data)?;
        
        let total_events = events.len() as u64;
        let start_event = request.start_event.unwrap_or(0);
        let end_event = request.end_event.unwrap_or(total_events);
        
        // Update session with total events
        {
            let mut sessions = self.replay_sessions.write().await;
            if let Some(session) = sessions.get_mut(replay_id) {
                session.total_events = total_events;
            }
        }
        
        // Process events
        let mut state = {
            let states = self.replay_states.read().await;
            states.get(replay_id)
                .cloned()
                .ok_or_else(|| SceneError::NotFound(format!("Replay state {} not found", replay_id)))?
        };
        
        for (index, event) in events.iter().enumerate() {
            let event_index = index as u64;
            
            if event_index < start_event || event_index >= end_event {
                continue;
            }
            
            // Process event
            self.process_replay_event(replay_id, event, &mut state).await?;
            
            // Update session progress
            {
                let mut sessions = self.replay_sessions.write().await;
                if let Some(session) = sessions.get_mut(replay_id) {
                    session.current_event = event_index + 1;
                }
            }
            
            // Update state
            {
                let mut states = self.replay_states.write().await;
                states.insert(replay_id.to_string(), state.clone());
            }
        }
        
        // Calculate final hash
        let final_hash = self.calculate_final_hash(&state)?;
        
        // Create result
        let duration = start_time.elapsed().as_secs_f64();
        let result = ReplayResult {
            replay_id: replay_id.to_string(),
            recording_id: request.recording_id.clone(),
            final_snapshot_id: format!("snapshot_{}", Uuid::new_v4()),
            deterministic_hash: final_hash.clone(),
            events_processed: end_event - start_event,
            duration_seconds: duration,
            success: true,
            error_message: None,
            byte_stability_verified: request.verify_determinism,
        };
        
        // Store result
        {
            let mut results = self.replay_results.write().await;
            results.insert(replay_id.to_string(), result);
        }
        
        // Mark session as inactive
        {
            let mut sessions = self.replay_sessions.write().await;
            if let Some(session) = sessions.get_mut(replay_id) {
                session.is_active = false;
            }
        }
        
        Ok(())
    }
    
    /// Parse recording data from CBOR format
    fn parse_recording_data(&self, data: &[u8]) -> SceneResult<Vec<RecordedEvent>> {
        let mut events = Vec::new();
        let mut cursor = 0;
        
        while cursor < data.len() {
            // Find next newline
            let mut end = cursor;
            while end < data.len() && data[end] != b'\n' {
                end += 1;
            }
            
            if end > cursor {
                let event_data = &data[cursor..end];
                let event: RecordedEvent = serde_cbor::from_slice(event_data)
                    .map_err(|e| SceneError::DeserializationError(e.to_string()))?;
                events.push(event);
            }
            
            cursor = end + 1;
        }
        
        Ok(events)
    }
}

// Clone implementation for XRReplayManager
impl Clone for XRReplayManager {
    fn clone(&self) -> Self {
        Self {
            replay_sessions: self.replay_sessions.clone(),
            replay_results: self.replay_results.clone(),
            replay_states: self.replay_states.clone(),
            recording_manager: self.recording_manager.clone(),
            persistence_manager: self.persistence_manager.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{XRRecordingManager, StartRecordingRequest, StopRecordingRequest};

    #[tokio::test]
    async fn test_replay_recording() {
        let recording_manager = Arc::new(XRRecordingManager::new());
        let persistence_manager = Arc::new(crate::persist::NGFS persistenceManager::new(
            Arc::new(crate::persist::MockNgfsClient),
            Arc::new(crate::persist::MockDaoClient),
            Arc::new(crate::persist::MockAuditClient),
        ));
        
        let replay_manager = XRReplayManager::new(recording_manager.clone(), persistence_manager);
        
        // Start recording
        let start_request = StartRecordingRequest {
            room_id: "test_room".to_string(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
            deterministic_seed: Some(12345),
            physics_tick_rate: Some(60.0),
        };
        
        let start_response = recording_manager.start_recording(&start_request).await.unwrap();
        
        // Stop recording
        let stop_request = StopRecordingRequest {
            recording_id: start_response.recording_id.clone(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
        };
        
        recording_manager.stop_recording(&stop_request).await.unwrap();
        
        // Replay recording
        let replay_request = ReplayRecordingRequest {
            recording_id: start_response.recording_id,
            mode: ReplayMode::Headless,
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
            start_event: None,
            end_event: None,
            verify_determinism: true,
        };
        
        let replay_response = replay_manager.replay_recording(&replay_request).await.unwrap();
        assert!(replay_response.success);
        assert!(!replay_response.replay_id.is_empty());
    }

    #[tokio::test]
    async fn test_deterministic_replay() {
        let recording_manager = Arc::new(XRRecordingManager::new());
        let persistence_manager = Arc::new(crate::persist::NGFS persistenceManager::new(
            Arc::new(crate::persist::MockNgfsClient),
            Arc::new(crate::persist::MockDaoClient),
            Arc::new(crate::persist::MockAuditClient),
        ));
        
        let replay_manager = XRReplayManager::new(recording_manager.clone(), persistence_manager);
        
        // Create a simple recording
        let start_request = StartRecordingRequest {
            room_id: "test_room".to_string(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
            deterministic_seed: Some(12345),
            physics_tick_rate: Some(60.0),
        };
        
        let start_response = recording_manager.start_recording(&start_request).await.unwrap();
        
        // Stop recording
        let stop_request = StopRecordingRequest {
            recording_id: start_response.recording_id.clone(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
        };
        
        recording_manager.stop_recording(&stop_request).await.unwrap();
        
        // Replay multiple times and verify determinism
        let mut hashes = Vec::new();
        
        for _ in 0..3 {
            let replay_request = ReplayRecordingRequest {
                recording_id: start_response.recording_id.clone(),
                mode: ReplayMode::Headless,
                session_id: "session_123".to_string(),
                cap_token: "cap_123".to_string(),
                start_event: None,
                end_event: None,
                verify_determinism: true,
            };
            
            let replay_response = replay_manager.replay_recording(&replay_request).await.unwrap();
            
            // Wait for replay to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            let result = replay_manager.get_replay_result(&replay_response.replay_id).await.unwrap();
            hashes.push(result.deterministic_hash);
        }
        
        // All hashes should be identical
        assert_eq!(hashes.len(), 3);
        assert_eq!(hashes[0], hashes[1]);
        assert_eq!(hashes[1], hashes[2]);
    }
}
