/**
 * Multi-User Session Manager
 * 
 * This module provides multi-user XR session management with:
 * - Room-based session organization
 * - Real-time scene synchronization
 * - Participant management and authentication
 * - Transport layer for reliable communication
 * - Conflict resolution and state reconciliation
 */

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

use crate::error::{SceneError, SceneResult};
use crate::xr::scene::{SceneNode, SceneSnapshot, NodeId, Transform};

/// Multi-user room identifier
pub type RoomId = String;

/// Participant identifier
pub type ParticipantId = String;

/// Multi-user room configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    /// Maximum number of participants
    pub max_participants: usize,
    /// Scene synchronization interval
    pub sync_interval: Duration,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
    /// Enable voice chat
    pub enable_voice: bool,
    /// Enable spatial audio
    pub enable_spatial_audio: bool,
    /// Room visibility (public/private)
    pub visibility: RoomVisibility,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolution {
    /// Last writer wins
    LastWriterWins,
    /// Authority-based (room owner has priority)
    AuthorityBased,
    /// Consensus-based (majority vote)
    ConsensusBased,
    /// Time-based (earliest timestamp wins)
    TimeBased,
}

/// Room visibility levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoomVisibility {
    /// Public room (discoverable)
    Public,
    /// Private room (invite only)
    Private,
    /// Friends only
    FriendsOnly,
}

/// Multi-user room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiUserRoom {
    pub id: RoomId,
    pub name: String,
    pub description: Option<String>,
    pub owner_did: String,
    pub config: RoomConfig,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub participant_count: usize,
    pub scene_snapshot: Option<SceneSnapshot>,
}

/// Participant in a multi-user session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: ParticipantId,
    pub did: String,
    pub session_id: String,
    pub room_id: RoomId,
    pub avatar_id: Option<String>,
    pub device_handle: Option<String>,
    pub joined_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_active: bool,
    pub permissions: ParticipantPermissions,
}

/// Participant permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantPermissions {
    /// Can modify scene
    pub can_modify_scene: bool,
    /// Can invite others
    pub can_invite: bool,
    /// Can kick participants
    pub can_kick: bool,
    /// Can change room settings
    pub can_change_settings: bool,
    /// Can use voice chat
    pub can_use_voice: bool,
}

/// Scene synchronization event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEvent {
    /// Node added to scene
    NodeAdded { node: SceneNode, participant_id: ParticipantId },
    /// Node updated
    NodeUpdated { node_id: NodeId, transform: Transform, participant_id: ParticipantId },
    /// Node removed
    NodeRemoved { node_id: NodeId, participant_id: ParticipantId },
    /// Participant joined
    ParticipantJoined { participant: Participant },
    /// Participant left
    ParticipantLeft { participant_id: ParticipantId },
    /// Scene snapshot update
    SceneSnapshot { snapshot: SceneSnapshot, participant_id: ParticipantId },
}

/// Transport message for reliable communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
    pub id: String,
    pub room_id: RoomId,
    pub from_participant: ParticipantId,
    pub to_participant: Option<ParticipantId>, // None for broadcast
    pub message_type: TransportMessageType,
    pub payload: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub sequence_number: u64,
    pub requires_ack: bool,
}

/// Transport message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportMessageType {
    /// Scene synchronization
    SceneSync,
    /// Voice data
    VoiceData,
    /// Control message
    Control,
    /// Acknowledgment
    Acknowledgment,
    /// Heartbeat
    Heartbeat,
}

/// Multi-user session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiUserStats {
    pub room_id: RoomId,
    pub participant_count: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub sync_events_processed: u64,
    pub average_latency_ms: f32,
    pub uptime: Duration,
}

/// Multi-user session manager
pub struct MultiUserManager {
    rooms: Arc<RwLock<HashMap<RoomId, MultiUserRoom>>>,
    participants: Arc<RwLock<HashMap<ParticipantId, Participant>>>,
    room_participants: Arc<RwLock<HashMap<RoomId, HashSet<ParticipantId>>>>,
    sync_events: Arc<RwLock<Vec<SyncEvent>>>,
    transport_messages: Arc<RwLock<Vec<TransportMessage>>>,
    event_sender: broadcast::Sender<SyncEvent>,
    stats: Arc<RwLock<HashMap<RoomId, MultiUserStats>>>,
}

impl MultiUserManager {
    /// Create a new multi-user manager
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            participants: Arc::new(RwLock::new(HashMap::new())),
            room_participants: Arc::new(RwLock::new(HashMap::new())),
            sync_events: Arc::new(RwLock::new(Vec::new())),
            transport_messages: Arc::new(RwLock::new(Vec::new())),
            event_sender,
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new multi-user room
    pub async fn create_room(
        &self,
        name: String,
        description: Option<String>,
        owner_did: String,
        owner_session_id: String,
        config: RoomConfig,
    ) -> SceneResult<MultiUserRoom> {
        let room_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let room = MultiUserRoom {
            id: room_id.clone(),
            name,
            description,
            owner_did: owner_did.clone(),
            config,
            created_at: now,
            last_activity: now,
            participant_count: 0,
            scene_snapshot: None,
        };

        // Store room
        self.rooms.write().unwrap().insert(room_id.clone(), room.clone());

        // Initialize room stats
        let room_stats = MultiUserStats {
            room_id: room_id.clone(),
            participant_count: 0,
            messages_sent: 0,
            messages_received: 0,
            sync_events_processed: 0,
            average_latency_ms: 0.0,
            uptime: Duration::from_secs(0),
        };
        self.stats.write().unwrap().insert(room_id, room_stats);

        // Auto-join the room owner
        self.join_room(room_id.clone(), owner_did, owner_session_id, None).await?;

        Ok(room)
    }

    /// Join a multi-user room
    pub async fn join_room(
        &self,
        room_id: RoomId,
        did: String,
        session_id: String,
        avatar_id: Option<String>,
    ) -> SceneResult<ParticipantId> {
        // Check if room exists
        let room = self.rooms.read().unwrap()
            .get(&room_id)
            .ok_or_else(|| SceneError::RoomNotFound(room_id.clone()))?
            .clone();

        // Check participant limit
        if room.participant_count >= room.config.max_participants {
            return Err(SceneError::RoomFull(room_id));
        }

        // Check if already in room
        let participants = self.participants.read().unwrap();
        for participant in participants.values() {
            if participant.did == did && participant.room_id == room_id {
                return Err(SceneError::AlreadyInRoom(room_id));
            }
        }
        drop(participants);

        let participant_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Determine permissions based on room ownership
        let permissions = if did == room.owner_did {
            ParticipantPermissions {
                can_modify_scene: true,
                can_invite: true,
                can_kick: true,
                can_change_settings: true,
                can_use_voice: true,
            }
        } else {
            ParticipantPermissions {
                can_modify_scene: true,
                can_invite: false,
                can_kick: false,
                can_change_settings: false,
                can_use_voice: room.config.enable_voice,
            }
        };

        let participant = Participant {
            id: participant_id.clone(),
            did,
            session_id,
            room_id: room_id.clone(),
            avatar_id,
            device_handle: None,
            joined_at: now,
            last_seen: now,
            is_active: true,
            permissions,
        };

        // Store participant
        self.participants.write().unwrap().insert(participant_id.clone(), participant.clone());

        // Add to room participants
        self.room_participants.write().unwrap()
            .entry(room_id.clone())
            .or_insert_with(HashSet::new)
            .insert(participant_id.clone());

        // Update room participant count
        if let Some(room) = self.rooms.write().unwrap().get_mut(&room_id) {
            room.participant_count += 1;
            room.last_activity = now;
        }

        // Emit participant joined event
        let sync_event = SyncEvent::ParticipantJoined { participant };
        self.emit_sync_event(sync_event).await?;

        Ok(participant_id)
    }

    /// Leave a multi-user room
    pub async fn leave_room(&self, participant_id: ParticipantId) -> SceneResult<()> {
        // Get participant
        let participant = self.participants.read().unwrap()
            .get(&participant_id)
            .ok_or_else(|| SceneError::ParticipantNotFound(participant_id.clone()))?
            .clone();

        let room_id = participant.room_id.clone();

        // Remove participant
        self.participants.write().unwrap().remove(&participant_id);

        // Remove from room participants
        if let Some(participants) = self.room_participants.write().unwrap().get_mut(&room_id) {
            participants.remove(&participant_id);
        }

        // Update room participant count
        if let Some(room) = self.rooms.write().unwrap().get_mut(&room_id) {
            room.participant_count = room.participant_count.saturating_sub(1);
            room.last_activity = Utc::now();
        }

        // Emit participant left event
        let sync_event = SyncEvent::ParticipantLeft { participant_id };
        self.emit_sync_event(sync_event).await?;

        Ok(())
    }

    /// Synchronize scene with room participants
    pub async fn sync_scene(
        &self,
        room_id: RoomId,
        participant_id: ParticipantId,
        scene_snapshot: SceneSnapshot,
    ) -> SceneResult<()> {
        // Verify participant is in room
        let participant = self.participants.read().unwrap()
            .get(&participant_id)
            .ok_or_else(|| SceneError::ParticipantNotFound(participant_id.clone()))?;

        if participant.room_id != room_id {
            return Err(SceneError::ParticipantNotInRoom(participant_id));
        }

        // Check permissions
        if !participant.permissions.can_modify_scene {
            return Err(SceneError::InsufficientPermissions("scene_modify".to_string()));
        }

        // Update room scene snapshot
        if let Some(room) = self.rooms.write().unwrap().get_mut(&room_id) {
            room.scene_snapshot = Some(scene_snapshot.clone());
            room.last_activity = Utc::now();
        }

        // Emit scene snapshot event
        let sync_event = SyncEvent::SceneSnapshot {
            snapshot: scene_snapshot,
            participant_id,
        };
        self.emit_sync_event(sync_event).await?;

        Ok(())
    }

    /// Get room scene snapshot
    pub fn get_room_scene(&self, room_id: &RoomId) -> SceneResult<Option<SceneSnapshot>> {
        let rooms = self.rooms.read().unwrap();
        Ok(rooms.get(room_id).and_then(|room| room.scene_snapshot.clone()))
    }

    /// Get room participants
    pub fn get_room_participants(&self, room_id: &RoomId) -> SceneResult<Vec<Participant>> {
        let room_participants = self.room_participants.read().unwrap();
        let participant_ids = room_participants.get(room_id)
            .ok_or_else(|| SceneError::RoomNotFound(room_id.clone()))?;

        let participants = self.participants.read().unwrap();
        let mut result = Vec::new();
        
        for participant_id in participant_ids {
            if let Some(participant) = participants.get(participant_id) {
                result.push(participant.clone());
            }
        }

        Ok(result)
    }

    /// Get all rooms
    pub fn get_rooms(&self) -> SceneResult<Vec<MultiUserRoom>> {
        let rooms = self.rooms.read().unwrap();
        Ok(rooms.values().cloned().collect())
    }

    /// Get room by ID
    pub fn get_room(&self, room_id: &RoomId) -> SceneResult<MultiUserRoom> {
        let rooms = self.rooms.read().unwrap();
        rooms.get(room_id)
            .cloned()
            .ok_or_else(|| SceneError::RoomNotFound(room_id.clone()))
    }

    /// Send transport message
    pub async fn send_message(
        &self,
        room_id: RoomId,
        from_participant: ParticipantId,
        to_participant: Option<ParticipantId>,
        message_type: TransportMessageType,
        payload: Vec<u8>,
    ) -> SceneResult<String> {
        let message_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Get sequence number (in real implementation, this would be per-participant)
        let sequence_number = self.transport_messages.read().unwrap().len() as u64;

        let message = TransportMessage {
            id: message_id.clone(),
            room_id: room_id.clone(),
            from_participant: from_participant.clone(),
            to_participant,
            message_type,
            payload,
            timestamp: now,
            sequence_number,
            requires_ack: true,
        };

        // Store message
        self.transport_messages.write().unwrap().push(message);

        // Update stats
        if let Some(stats) = self.stats.write().unwrap().get_mut(&room_id) {
            stats.messages_sent += 1;
        }

        Ok(message_id)
    }

    /// Get transport messages for participant
    pub fn get_messages(
        &self,
        room_id: &RoomId,
        participant_id: &ParticipantId,
    ) -> SceneResult<Vec<TransportMessage>> {
        let messages = self.transport_messages.read().unwrap();
        let participant_messages: Vec<TransportMessage> = messages
            .iter()
            .filter(|msg| {
                msg.room_id == *room_id && (
                    msg.to_participant.is_none() || // Broadcast
                    msg.to_participant.as_ref() == Some(participant_id) || // Direct
                    msg.from_participant == *participant_id // From participant
                )
            })
            .cloned()
            .collect();

        Ok(participant_messages)
    }

    /// Emit synchronization event
    async fn emit_sync_event(&self, event: SyncEvent) -> SceneResult<()> {
        // Store event
        self.sync_events.write().unwrap().push(event.clone());

        // Broadcast to subscribers
        let _ = self.event_sender.send(event);

        Ok(())
    }

    /// Subscribe to synchronization events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_sender.subscribe()
    }

    /// Get room statistics
    pub fn get_room_stats(&self, room_id: &RoomId) -> SceneResult<MultiUserStats> {
        let stats = self.stats.read().unwrap();
        stats.get(room_id)
            .cloned()
            .ok_or_else(|| SceneError::RoomNotFound(room_id.clone()))
    }

    /// Update participant activity
    pub fn update_participant_activity(&self, participant_id: &ParticipantId) -> SceneResult<()> {
        if let Some(participant) = self.participants.write().unwrap().get_mut(participant_id) {
            participant.last_seen = Utc::now();
            participant.is_active = true;
        }
        Ok(())
    }

    /// Clean up inactive participants
    pub async fn cleanup_inactive_participants(&self, timeout: Duration) -> SceneResult<()> {
        let now = Utc::now();
        let mut inactive_participants = Vec::new();

        // Find inactive participants
        {
            let participants = self.participants.read().unwrap();
            for (participant_id, participant) in participants.iter() {
                let time_since_last_seen = now.signed_duration_since(participant.last_seen);
                if time_since_last_seen.to_std().unwrap_or_default() > timeout {
                    inactive_participants.push(participant_id.clone());
                }
            }
        }

        // Remove inactive participants
        for participant_id in inactive_participants {
            self.leave_room(participant_id).await?;
        }

        Ok(())
    }
}

impl Default for MultiUserManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_room() {
        let manager = MultiUserManager::new();
        
        let config = RoomConfig {
            max_participants: 10,
            sync_interval: Duration::from_millis(100),
            conflict_resolution: ConflictResolution::LastWriterWins,
            enable_voice: true,
            enable_spatial_audio: true,
            visibility: RoomVisibility::Public,
        };

        let room = manager.create_room(
            "Test Room".to_string(),
            Some("Test Description".to_string()),
            "did:aeth:owner".to_string(),
            "session_123".to_string(),
            config,
        ).await.unwrap();

        assert_eq!(room.name, "Test Room");
        assert_eq!(room.owner_did, "did:aeth:owner");
        assert_eq!(room.participant_count, 1); // Owner auto-joined
    }

    #[tokio::test]
    async fn test_join_leave_room() {
        let manager = MultiUserManager::new();
        
        let config = RoomConfig {
            max_participants: 10,
            sync_interval: Duration::from_millis(100),
            conflict_resolution: ConflictResolution::LastWriterWins,
            enable_voice: true,
            enable_spatial_audio: true,
            visibility: RoomVisibility::Public,
        };

        let room = manager.create_room(
            "Test Room".to_string(),
            None,
            "did:aeth:owner".to_string(),
            "session_123".to_string(),
            config,
        ).await.unwrap();

        // Join room
        let participant_id = manager.join_room(
            room.id.clone(),
            "did:aeth:user1".to_string(),
            "session_456".to_string(),
            Some("avatar_123".to_string()),
        ).await.unwrap();

        // Check participant count
        let room = manager.get_room(&room.id).unwrap();
        assert_eq!(room.participant_count, 2);

        // Get participants
        let participants = manager.get_room_participants(&room.id).unwrap();
        assert_eq!(participants.len(), 2);

        // Leave room
        manager.leave_room(participant_id).await.unwrap();

        // Check participant count
        let room = manager.get_room(&room.id).unwrap();
        assert_eq!(room.participant_count, 1);
    }

    #[tokio::test]
    async fn test_room_participant_limit() {
        let manager = MultiUserManager::new();
        
        let mut config = RoomConfig {
            max_participants: 2,
            sync_interval: Duration::from_millis(100),
            conflict_resolution: ConflictResolution::LastWriterWins,
            enable_voice: true,
            enable_spatial_audio: true,
            visibility: RoomVisibility::Public,
        };

        let room = manager.create_room(
            "Test Room".to_string(),
            None,
            "did:aeth:owner".to_string(),
            "session_123".to_string(),
            config,
        ).await.unwrap();

        // Join second participant
        let _participant2 = manager.join_room(
            room.id.clone(),
            "did:aeth:user1".to_string(),
            "session_456".to_string(),
            None,
        ).await.unwrap();

        // Try to join third participant (should fail)
        let result = manager.join_room(
            room.id.clone(),
            "did:aeth:user2".to_string(),
            "session_789".to_string(),
            None,
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scene_synchronization() {
        let manager = MultiUserManager::new();
        
        let config = RoomConfig {
            max_participants: 10,
            sync_interval: Duration::from_millis(100),
            conflict_resolution: ConflictResolution::LastWriterWins,
            enable_voice: true,
            enable_spatial_audio: true,
            visibility: RoomVisibility::Public,
        };

        let room = manager.create_room(
            "Test Room".to_string(),
            None,
            "did:aeth:owner".to_string(),
            "session_123".to_string(),
            config,
        ).await.unwrap();

        // Get owner participant ID
        let participants = manager.get_room_participants(&room.id).unwrap();
        let owner_participant = participants.iter()
            .find(|p| p.did == "did:aeth:owner")
            .unwrap();

        // Create scene snapshot
        let snapshot = SceneSnapshot {
            id: "snapshot_123".to_string(),
            label: "Test Snapshot".to_string(),
            description: None,
            nodes: vec![],
            avatars: vec![],
            metadata: HashMap::new(),
            created_at: Utc::now(),
            created_by: "did:aeth:owner".to_string(),
            version: 1,
        };

        // Sync scene
        manager.sync_scene(
            room.id.clone(),
            owner_participant.id.clone(),
            snapshot.clone(),
        ).await.unwrap();

        // Check scene was stored
        let stored_scene = manager.get_room_scene(&room.id).unwrap();
        assert!(stored_scene.is_some());
        assert_eq!(stored_scene.unwrap().id, "snapshot_123");
    }

    #[tokio::test]
    async fn test_transport_messages() {
        let manager = MultiUserManager::new();
        
        let config = RoomConfig {
            max_participants: 10,
            sync_interval: Duration::from_millis(100),
            conflict_resolution: ConflictResolution::LastWriterWins,
            enable_voice: true,
            enable_spatial_audio: true,
            visibility: RoomVisibility::Public,
        };

        let room = manager.create_room(
            "Test Room".to_string(),
            None,
            "did:aeth:owner".to_string(),
            "session_123".to_string(),
            config,
        ).await.unwrap();

        // Get owner participant ID
        let participants = manager.get_room_participants(&room.id).unwrap();
        let owner_participant = participants.iter()
            .find(|p| p.did == "did:aeth:owner")
            .unwrap();

        // Send message
        let message_id = manager.send_message(
            room.id.clone(),
            owner_participant.id.clone(),
            None, // Broadcast
            TransportMessageType::Control,
            b"test payload".to_vec(),
        ).await.unwrap();

        // Get messages
        let messages = manager.get_messages(&room.id, &owner_participant.id).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, message_id);
        assert_eq!(messages[0].payload, b"test payload");
    }
}
