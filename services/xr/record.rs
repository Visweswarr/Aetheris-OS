//! XR Recording Service - Event Capture and Log Writer
//! 
//! This module provides session recording capabilities by capturing time-ordered
//! inputs/events and exporting them as deterministic replay logs.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::scene::{
    SceneResult, SceneError, InputEvent, Vector3, Transform, PhysicsOperation
};
use crate::multiuser::RoomState;
use crate::physics::PhysicsState;

/// Recording session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: String,
    pub room_id: String,
    pub started_at: DateTime<Utc>,
    pub started_by: String,
    pub session_id: String,
    pub is_active: bool,
    pub event_count: u64,
    pub deterministic_seed: u64,
    pub physics_tick_rate: f32,
}

/// Recorded event with full context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    pub sequence: u64,
    pub timestamp: u64,
    pub source: EventSource,
    pub event_type: EventType,
    pub data: serde_cbor::Value,
    pub signature: String,
    pub deterministic_context: DeterministicContext,
}

/// Event source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    pub participant_id: Option<String>,
    pub device_handle: Option<String>,
    pub session_id: String,
    pub actor_did: String,
}

/// Event type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    InputEvent(InputEvent),
    PhysicsTick(PhysicsTickEvent),
    NetworkFrame(NetworkFrameEvent),
    SceneModification(SceneModificationEvent),
    AvatarUpdate(AvatarUpdateEvent),
    RoomStateChange(RoomStateChangeEvent),
    SystemEvent(SystemEvent),
}

/// Physics tick event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsTickEvent {
    pub tick_number: u64,
    pub delta_time: f32,
    pub physics_state: PhysicsState,
    pub rng_seed: u64,
}

/// Network frame event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFrameEvent {
    pub frame_number: u64,
    pub participants: Vec<String>,
    pub network_order: Vec<String>,
    pub latency_measurements: HashMap<String, f32>,
}

/// Scene modification event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneModificationEvent {
    pub node_id: String,
    pub operation: String,
    pub transform: Option<Transform>,
    pub component_data: Option<serde_cbor::Value>,
}

/// Avatar update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarUpdateEvent {
    pub avatar_id: String,
    pub transform: Transform,
    pub animation_state: Option<String>,
    pub interaction_state: Option<String>,
}

/// Room state change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomStateChangeEvent {
    pub change_type: String,
    pub participant_id: Option<String>,
    pub room_state: RoomState,
}

/// System event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_name: String,
    pub parameters: HashMap<String, serde_cbor::Value>,
}

/// Deterministic context for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterministicContext {
    pub physics_tick: u64,
    pub network_frame: u64,
    pub rng_seed: u64,
    pub server_timestamp: u64,
    pub client_timestamp: u64,
    pub network_delay: f32,
}

/// Recording summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSummary {
    pub recording_id: String,
    pub room_id: String,
    pub duration_seconds: f64,
    pub event_count: u64,
    pub started_at: DateTime<Utc>,
    pub stopped_at: DateTime<Utc>,
    pub deterministic_hash: String,
    pub file_size_bytes: u64,
    pub participants: Vec<String>,
    pub devices_used: Vec<String>,
}

/// Recording request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRecordingRequest {
    pub room_id: String,
    pub session_id: String,
    pub cap_token: String,
    pub deterministic_seed: Option<u64>,
    pub physics_tick_rate: Option<f32>,
}

/// Recording response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRecordingResponse {
    pub recording_id: String,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Stop recording request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopRecordingRequest {
    pub recording_id: String,
    pub session_id: String,
    pub cap_token: String,
}

/// Stop recording response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopRecordingResponse {
    pub recording_summary: RecordingSummary,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Recording manager trait
#[async_trait::async_trait]
pub trait RecordingManager: Send + Sync {
    async fn start_recording(&self, request: &StartRecordingRequest) -> SceneResult<StartRecordingResponse>;
    async fn stop_recording(&self, request: &StopRecordingRequest) -> SceneResult<StopRecordingResponse>;
    async fn record_event(&self, recording_id: &str, event: &RecordedEvent) -> SceneResult<()>;
    async fn get_recording_session(&self, recording_id: &str) -> SceneResult<RecordingSession>;
    async fn export_recording(&self, recording_id: &str, format: &str) -> SceneResult<Vec<u8>>;
    async fn list_recordings(&self, room_id: &str) -> SceneResult<Vec<RecordingSummary>>;
}

/// Recording manager implementation
pub struct XRRecordingManager {
    sessions: Arc<RwLock<HashMap<String, RecordingSession>>>,
    events: Arc<RwLock<HashMap<String, Vec<RecordedEvent>>>>,
    summaries: Arc<RwLock<HashMap<String, RecordingSummary>>>,
    rng_seed: u64,
}

impl XRRecordingManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(HashMap::new())),
            summaries: Arc::new(RwLock::new(HashMap::new())),
            rng_seed: 12345, // In real implementation, this would be configurable
        }
    }

    /// Generate deterministic seed
    fn generate_deterministic_seed(&self, room_id: &str, timestamp: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        room_id.hash(&mut hasher);
        timestamp.hash(&mut hasher);
        self.rng_seed.hash(&mut hasher);
        hasher.finish()
    }

    /// Create event signature
    fn create_event_signature(&self, event: &RecordedEvent) -> String {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        hasher.update(&event.sequence.to_le_bytes());
        hasher.update(&event.timestamp.to_le_bytes());
        hasher.update(event.source.session_id.as_bytes());
        hasher.update(event.source.actor_did.as_bytes());
        
        // In real implementation, this would be signed with DID key
        format!("sig_{}", hex::encode(&hasher.finalize().as_bytes()[..8]))
    }

    /// Validate recording request
    async fn validate_recording_request(&self, request: &StartRecordingRequest) -> SceneResult<()> {
        if request.session_id.is_empty() {
            return Err(SceneError::InvalidSession("Empty session ID".to_string()));
        }

        if request.cap_token.is_empty() {
            return Err(SceneError::InsufficientCapabilities("Empty capability token".to_string()));
        }

        // Check if room is already being recorded
        let sessions = self.sessions.read().await;
        for session in sessions.values() {
            if session.room_id == request.room_id && session.is_active {
                return Err(SceneError::Conflict("Room is already being recorded".to_string()));
            }
        }

        Ok(())
    }

    /// Calculate recording hash for determinism verification
    fn calculate_recording_hash(&self, events: &[RecordedEvent]) -> SceneResult<String> {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        
        for event in events {
            hasher.update(&event.sequence.to_le_bytes());
            hasher.update(&event.timestamp.to_le_bytes());
            hasher.update(&event.deterministic_context.physics_tick.to_le_bytes());
            hasher.update(&event.deterministic_context.network_frame.to_le_bytes());
            hasher.update(&event.deterministic_context.rng_seed.to_le_bytes());
        }
        
        Ok(hasher.finalize().to_hex().to_string())
    }
}

#[async_trait::async_trait]
impl RecordingManager for XRRecordingManager {
    async fn start_recording(&self, request: &StartRecordingRequest) -> SceneResult<StartRecordingResponse> {
        // Validate request
        self.validate_recording_request(request).await?;

        let recording_id = format!("recording_{}", Uuid::new_v4());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let deterministic_seed = request.deterministic_seed
            .unwrap_or_else(|| self.generate_deterministic_seed(&request.room_id, timestamp));

        let physics_tick_rate = request.physics_tick_rate.unwrap_or(60.0);

        let session = RecordingSession {
            id: recording_id.clone(),
            room_id: request.room_id.clone(),
            started_at: Utc::now(),
            started_by: request.session_id.clone(),
            session_id: request.session_id.clone(),
            is_active: true,
            event_count: 0,
            deterministic_seed,
            physics_tick_rate,
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(recording_id.clone(), session);
        }

        // Initialize events list
        {
            let mut events = self.events.write().await;
            events.insert(recording_id.clone(), Vec::new());
        }

        Ok(StartRecordingResponse {
            recording_id,
            success: true,
            error_message: None,
        })
    }

    async fn stop_recording(&self, request: &StopRecordingRequest) -> SceneResult<StopRecordingResponse> {
        // Get session
        let session = self.get_recording_session(&request.recording_id).await?;
        
        if !session.is_active {
            return Err(SceneError::InvalidState("Recording is not active".to_string()));
        }

        // Get events
        let events = {
            let events_map = self.events.read().await;
            events_map.get(&request.recording_id)
                .cloned()
                .unwrap_or_default()
        };

        // Calculate hash
        let deterministic_hash = self.calculate_recording_hash(&events)?;

        // Create summary
        let stopped_at = Utc::now();
        let duration = (stopped_at - session.started_at).num_milliseconds() as f64 / 1000.0;
        
        let summary = RecordingSummary {
            recording_id: request.recording_id.clone(),
            room_id: session.room_id.clone(),
            duration_seconds: duration,
            event_count: events.len() as u64,
            started_at: session.started_at,
            stopped_at,
            deterministic_hash,
            file_size_bytes: 0, // Will be calculated when exporting
            participants: vec![], // Will be populated from events
            devices_used: vec![], // Will be populated from events
        };

        // Update session to inactive
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&request.recording_id) {
                session.is_active = false;
            }
        }

        // Store summary
        {
            let mut summaries = self.summaries.write().await;
            summaries.insert(request.recording_id.clone(), summary.clone());
        }

        Ok(StopRecordingResponse {
            recording_summary: summary,
            success: true,
            error_message: None,
        })
    }

    async fn record_event(&self, recording_id: &str, event: &RecordedEvent) -> SceneResult<()> {
        // Check if recording is active
        let session = self.get_recording_session(recording_id).await?;
        if !session.is_active {
            return Err(SceneError::InvalidState("Recording is not active".to_string()));
        }

        // Create signature
        let signature = self.create_event_signature(event);
        let mut event_with_sig = event.clone();
        event_with_sig.signature = signature;

        // Store event
        {
            let mut events = self.events.write().await;
            if let Some(event_list) = events.get_mut(recording_id) {
                event_list.push(event_with_sig);
            } else {
                return Err(SceneError::NotFound(format!("Recording {} not found", recording_id)));
            }
        }

        // Update event count
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(recording_id) {
                session.event_count += 1;
            }
        }

        Ok(())
    }

    async fn get_recording_session(&self, recording_id: &str) -> SceneResult<RecordingSession> {
        let sessions = self.sessions.read().await;
        sessions.get(recording_id)
            .cloned()
            .ok_or_else(|| SceneError::NotFound(format!("Recording {} not found", recording_id)))
    }

    async fn export_recording(&self, recording_id: &str, format: &str) -> SceneResult<Vec<u8>> {
        let events = {
            let events_map = self.events.read().await;
            events_map.get(recording_id)
                .cloned()
                .ok_or_else(|| SceneError::NotFound(format!("Recording {} not found", recording_id)))?
        };

        match format {
            "cbor" => {
                // Export as newline-delimited CBOR
                let mut output = Vec::new();
                for event in events {
                    let cbor_data = serde_cbor::to_vec(&event)
                        .map_err(|e| SceneError::SerializationError(e.to_string()))?;
                    output.extend_from_slice(&cbor_data);
                    output.push(b'\n');
                }
                Ok(output)
            }
            "json" => {
                // Export as JSON array
                let json_data = serde_json::to_vec(&events)
                    .map_err(|e| SceneError::SerializationError(e.to_string()))?;
                Ok(json_data)
            }
            _ => Err(SceneError::InvalidInput(format!("Unsupported format: {}", format)))
        }
    }

    async fn list_recordings(&self, room_id: &str) -> SceneResult<Vec<RecordingSummary>> {
        let summaries = self.summaries.read().await;
        let room_recordings: Vec<RecordingSummary> = summaries
            .values()
            .filter(|summary| summary.room_id == room_id)
            .cloned()
            .collect();
        
        Ok(room_recordings)
    }
}

/// Helper functions for creating events
impl XRRecordingManager {
    /// Create input event
    pub fn create_input_event(
        &self,
        sequence: u64,
        source: EventSource,
        input_event: InputEvent,
        deterministic_context: DeterministicContext,
    ) -> RecordedEvent {
        RecordedEvent {
            sequence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source,
            event_type: EventType::InputEvent(input_event),
            data: serde_cbor::Value::Null, // Will be populated from input_event
            signature: String::new(), // Will be set when recording
            deterministic_context,
        }
    }

    /// Create physics tick event
    pub fn create_physics_tick_event(
        &self,
        sequence: u64,
        tick_number: u64,
        delta_time: f32,
        physics_state: PhysicsState,
        rng_seed: u64,
        deterministic_context: DeterministicContext,
    ) -> RecordedEvent {
        let physics_tick = PhysicsTickEvent {
            tick_number,
            delta_time,
            physics_state,
            rng_seed,
        };

        RecordedEvent {
            sequence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: EventSource {
                participant_id: None,
                device_handle: None,
                session_id: "system".to_string(),
                actor_did: "system".to_string(),
            },
            event_type: EventType::PhysicsTick(physics_tick),
            data: serde_cbor::Value::Null,
            signature: String::new(),
            deterministic_context,
        }
    }

    /// Create network frame event
    pub fn create_network_frame_event(
        &self,
        sequence: u64,
        frame_number: u64,
        participants: Vec<String>,
        network_order: Vec<String>,
        latency_measurements: HashMap<String, f32>,
        deterministic_context: DeterministicContext,
    ) -> RecordedEvent {
        let network_frame = NetworkFrameEvent {
            frame_number,
            participants,
            network_order,
            latency_measurements,
        };

        RecordedEvent {
            sequence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: EventSource {
                participant_id: None,
                device_handle: None,
                session_id: "system".to_string(),
                actor_did: "system".to_string(),
            },
            event_type: EventType::NetworkFrame(network_frame),
            data: serde_cbor::Value::Null,
            signature: String::new(),
            deterministic_context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_stop_recording() {
        let manager = XRRecordingManager::new();
        
        let start_request = StartRecordingRequest {
            room_id: "test_room".to_string(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
            deterministic_seed: Some(12345),
            physics_tick_rate: Some(60.0),
        };
        
        let start_response = manager.start_recording(&start_request).await.unwrap();
        assert!(start_response.success);
        assert!(!start_response.recording_id.is_empty());
        
        let stop_request = StopRecordingRequest {
            recording_id: start_response.recording_id.clone(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
        };
        
        let stop_response = manager.stop_recording(&stop_request).await.unwrap();
        assert!(stop_response.success);
        assert_eq!(stop_response.recording_summary.recording_id, start_response.recording_id);
    }

    #[tokio::test]
    async fn test_record_events() {
        let manager = XRRecordingManager::new();
        
        let start_request = StartRecordingRequest {
            room_id: "test_room".to_string(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
            deterministic_seed: Some(12345),
            physics_tick_rate: Some(60.0),
        };
        
        let start_response = manager.start_recording(&start_request).await.unwrap();
        
        // Create test event
        let source = EventSource {
            participant_id: Some("participant_1".to_string()),
            device_handle: Some("device_1".to_string()),
            session_id: "session_123".to_string(),
            actor_did: "did:aeth:test".to_string(),
        };
        
        let input_event = InputEvent::ButtonPress {
            button_id: "trigger".to_string(),
            pressed: true,
            value: 1.0,
        };
        
        let deterministic_context = DeterministicContext {
            physics_tick: 1,
            network_frame: 1,
            rng_seed: 12345,
            server_timestamp: 1000,
            client_timestamp: 1000,
            network_delay: 0.0,
        };
        
        let event = manager.create_input_event(1, source, input_event, deterministic_context);
        manager.record_event(&start_response.recording_id, &event).await.unwrap();
        
        // Stop recording
        let stop_request = StopRecordingRequest {
            recording_id: start_response.recording_id.clone(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
        };
        
        let stop_response = manager.stop_recording(&stop_request).await.unwrap();
        assert_eq!(stop_response.recording_summary.event_count, 1);
    }

    #[tokio::test]
    async fn test_export_recording() {
        let manager = XRRecordingManager::new();
        
        let start_request = StartRecordingRequest {
            room_id: "test_room".to_string(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
            deterministic_seed: Some(12345),
            physics_tick_rate: Some(60.0),
        };
        
        let start_response = manager.start_recording(&start_request).await.unwrap();
        
        // Stop recording
        let stop_request = StopRecordingRequest {
            recording_id: start_response.recording_id.clone(),
            session_id: "session_123".to_string(),
            cap_token: "cap_123".to_string(),
        };
        
        manager.stop_recording(&stop_request).await.unwrap();
        
        // Export recording
        let cbor_data = manager.export_recording(&start_response.recording_id, "cbor").await.unwrap();
        assert!(!cbor_data.is_empty());
        
        let json_data = manager.export_recording(&start_response.recording_id, "json").await.unwrap();
        assert!(!json_data.is_empty());
    }
}
