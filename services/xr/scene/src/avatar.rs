//! Avatar management for XR scenes

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{SceneError, SceneResult};
use crate::scene::{NodeId, Transform};

/// Unique identifier for an avatar
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AvatarId(pub Uuid);

impl AvatarId {
    /// Generate a new avatar ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an avatar ID from a string
    pub fn from_string(s: &str) -> SceneResult<Self> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| SceneError::InvalidInput(format!("Invalid avatar ID: {}", s)))?;
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for AvatarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Avatar profile containing appearance and behavior data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvatarProfile {
    pub name: String,
    pub appearance: AppearanceData,
    pub behavior: BehaviorData,
    pub permissions: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Appearance data for avatars
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceData {
    pub mesh_id: String,
    pub material_id: String,
    pub texture_id: Option<String>,
    pub color: [f32; 4], // RGBA
    pub scale: [f32; 3],
    pub animations: Vec<AnimationData>,
}

/// Animation data for avatars
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationData {
    pub name: String,
    pub animation_id: String,
    pub loop_count: Option<u32>,
    pub speed: f32,
}

/// Behavior data for avatars
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorData {
    pub movement_speed: f32,
    pub jump_height: f32,
    pub interaction_range: f32,
    pub ai_enabled: bool,
    pub ai_script: Option<String>,
    pub custom_behaviors: HashMap<String, serde_json::Value>,
}

/// Avatar representing a user in the XR scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Avatar {
    pub id: AvatarId,
    pub did: String,
    pub profile: AvatarProfile,
    pub node_id: Option<NodeId>,
    pub transform: Transform,
    pub is_online: bool,
    pub last_seen: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Avatar {
    /// Create a new avatar
    pub fn new(did: String, profile: AvatarProfile) -> Self {
        let now = chrono::Utc::now().timestamp() as u64;
        Self {
            id: AvatarId::new(),
            did,
            profile,
            node_id: None,
            transform: Transform::identity(),
            is_online: false,
            last_seen: now,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the avatar's transform
    pub fn update_transform(&mut self, transform: Transform) -> SceneResult<()> {
        transform.validate()?;
        self.transform = transform;
        self.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(())
    }

    /// Set the avatar's online status
    pub fn set_online(&mut self, online: bool) {
        self.is_online = online;
        if online {
            self.last_seen = chrono::Utc::now().timestamp() as u64;
        }
        self.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    /// Update the avatar's profile
    pub fn update_profile(&mut self, profile: AvatarProfile) {
        self.profile = profile;
        self.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    /// Bind the avatar to a scene node
    pub fn bind_to_node(&mut self, node_id: NodeId) {
        self.node_id = Some(node_id);
        self.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    /// Unbind the avatar from its scene node
    pub fn unbind_from_node(&mut self) {
        self.node_id = None;
        self.updated_at = chrono::Utc::now().timestamp() as u64;
    }

    /// Check if the avatar has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.profile.permissions.contains(&permission.to_string())
    }

    /// Add a permission to the avatar
    pub fn add_permission(&mut self, permission: String) {
        if !self.profile.permissions.contains(&permission) {
            self.profile.permissions.push(permission);
            self.updated_at = chrono::Utc::now().timestamp() as u64;
        }
    }

    /// Remove a permission from the avatar
    pub fn remove_permission(&mut self, permission: &str) {
        self.profile.permissions.retain(|p| p != permission);
        self.updated_at = chrono::Utc::now().timestamp() as u64;
    }
}

/// Avatar manager handling all avatars in the scene
pub struct AvatarManager {
    avatars: Arc<RwLock<HashMap<AvatarId, Avatar>>>,
    did_to_avatar: Arc<RwLock<HashMap<String, AvatarId>>>,
    max_avatars: usize,
}

impl AvatarManager {
    /// Create a new avatar manager
    pub fn new(max_avatars: usize) -> Self {
        Self {
            avatars: Arc::new(RwLock::new(HashMap::new())),
            did_to_avatar: Arc::new(RwLock::new(HashMap::new())),
            max_avatars,
        }
    }

    /// Create a new avatar
    pub async fn create_avatar(&self, did: String, profile: AvatarProfile) -> SceneResult<AvatarId> {
        let mut avatars = self.avatars.write().await;
        let mut did_to_avatar = self.did_to_avatar.write().await;

        if avatars.len() >= self.max_avatars {
            return Err(SceneError::AvatarLimitReached);
        }

        // Check if avatar with this DID already exists
        if did_to_avatar.contains_key(&did) {
            return Err(SceneError::InvalidDID(format!("Avatar with DID already exists: {}", did)));
        }

        let avatar = Avatar::new(did.clone(), profile);
        let avatar_id = avatar.id;

        did_to_avatar.insert(did, avatar_id);
        avatars.insert(avatar_id, avatar);

        Ok(avatar_id)
    }

    /// Get an avatar by ID
    pub async fn get_avatar(&self, avatar_id: AvatarId) -> SceneResult<Avatar> {
        let avatars = self.avatars.read().await;
        avatars.get(&avatar_id)
            .cloned()
            .ok_or_else(|| SceneError::AvatarNotFound(format!("Avatar not found: {}", avatar_id)))
    }

    /// Get an avatar by DID
    pub async fn get_avatar_by_did(&self, did: &str) -> SceneResult<Avatar> {
        let did_to_avatar = self.did_to_avatar.read().await;
        let avatar_id = did_to_avatar.get(did)
            .ok_or_else(|| SceneError::AvatarNotFound(format!("Avatar with DID not found: {}", did)))?;

        self.get_avatar(*avatar_id).await
    }

    /// Update an avatar
    pub async fn update_avatar(&self, avatar_id: AvatarId, updates: AvatarUpdate) -> SceneResult<()> {
        let mut avatars = self.avatars.write().await;

        if let Some(avatar) = avatars.get_mut(&avatar_id) {
            match updates {
                AvatarUpdate::Transform(transform) => {
                    avatar.update_transform(transform)?;
                }
                AvatarUpdate::Profile(profile) => {
                    avatar.update_profile(profile);
                }
                AvatarUpdate::OnlineStatus(online) => {
                    avatar.set_online(online);
                }
                AvatarUpdate::BindNode(node_id) => {
                    avatar.bind_to_node(node_id);
                }
                AvatarUpdate::UnbindNode => {
                    avatar.unbind_from_node();
                }
                AvatarUpdate::AddPermission(permission) => {
                    avatar.add_permission(permission);
                }
                AvatarUpdate::RemovePermission(permission) => {
                    avatar.remove_permission(&permission);
                }
            }
        } else {
            return Err(SceneError::AvatarNotFound(format!("Avatar not found: {}", avatar_id)));
        }

        Ok(())
    }

    /// Delete an avatar
    pub async fn delete_avatar(&self, avatar_id: AvatarId) -> SceneResult<()> {
        let mut avatars = self.avatars.write().await;
        let mut did_to_avatar = self.did_to_avatar.write().await;

        if let Some(avatar) = avatars.remove(&avatar_id) {
            did_to_avatar.remove(&avatar.did);
            Ok(())
        } else {
            Err(SceneError::AvatarNotFound(format!("Avatar not found: {}", avatar_id)))
        }
    }

    /// Get all avatars
    pub async fn get_all_avatars(&self) -> Vec<Avatar> {
        let avatars = self.avatars.read().await;
        avatars.values().cloned().collect()
    }

    /// Get online avatars
    pub async fn get_online_avatars(&self) -> Vec<Avatar> {
        let avatars = self.avatars.read().await;
        avatars.values()
            .filter(|avatar| avatar.is_online)
            .cloned()
            .collect()
    }

    /// Get avatar count
    pub async fn get_avatar_count(&self) -> usize {
        let avatars = self.avatars.read().await;
        avatars.len()
    }

    /// Get online avatar count
    pub async fn get_online_avatar_count(&self) -> usize {
        let avatars = self.avatars.read().await;
        avatars.values().filter(|avatar| avatar.is_online).count()
    }

    /// Check if an avatar exists by DID
    pub async fn avatar_exists_by_did(&self, did: &str) -> bool {
        let did_to_avatar = self.did_to_avatar.read().await;
        did_to_avatar.contains_key(did)
    }

    /// Get avatar by scene node ID
    pub async fn get_avatar_by_node(&self, node_id: NodeId) -> SceneResult<Avatar> {
        let avatars = self.avatars.read().await;
        avatars.values()
            .find(|avatar| avatar.node_id == Some(node_id))
            .cloned()
            .ok_or_else(|| SceneError::AvatarNotFound(format!("Avatar with node ID not found: {}", node_id)))
    }

    /// Update avatar last seen timestamp
    pub async fn update_last_seen(&self, avatar_id: AvatarId) -> SceneResult<()> {
        let mut avatars = self.avatars.write().await;

        if let Some(avatar) = avatars.get_mut(&avatar_id) {
            avatar.last_seen = chrono::Utc::now().timestamp() as u64;
            avatar.updated_at = avatar.last_seen;
        } else {
            return Err(SceneError::AvatarNotFound(format!("Avatar not found: {}", avatar_id)));
        }

        Ok(())
    }

    /// Clean up offline avatars older than specified seconds
    pub async fn cleanup_offline_avatars(&self, max_age_seconds: u64) -> SceneResult<u32> {
        let mut avatars = self.avatars.write().await;
        let mut did_to_avatar = self.did_to_avatar.write().await;
        let current_time = chrono::Utc::now().timestamp() as u64;
        let mut removed_count = 0;

        let mut to_remove = Vec::new();
        for (avatar_id, avatar) in avatars.iter() {
            if !avatar.is_online && (current_time - avatar.last_seen) > max_age_seconds {
                to_remove.push(*avatar_id);
            }
        }

        for avatar_id in to_remove {
            if let Some(avatar) = avatars.remove(&avatar_id) {
                did_to_avatar.remove(&avatar.did);
                removed_count += 1;
            }
        }

        Ok(removed_count)
    }
}

/// Avatar update operations
#[derive(Debug, Clone)]
pub enum AvatarUpdate {
    Transform(Transform),
    Profile(AvatarProfile),
    OnlineStatus(bool),
    BindNode(NodeId),
    UnbindNode,
    AddPermission(String),
    RemovePermission(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile() -> AvatarProfile {
        AvatarProfile {
            name: "TestAvatar".to_string(),
            appearance: AppearanceData {
                mesh_id: "test_mesh".to_string(),
                material_id: "test_material".to_string(),
                texture_id: None,
                color: [1.0, 1.0, 1.0, 1.0],
                scale: [1.0, 1.0, 1.0],
                animations: Vec::new(),
            },
            behavior: BehaviorData {
                movement_speed: 5.0,
                jump_height: 2.0,
                interaction_range: 3.0,
                ai_enabled: false,
                ai_script: None,
                custom_behaviors: HashMap::new(),
            },
            permissions: vec!["move".to_string(), "interact".to_string()],
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_avatar_creation() {
        let avatar = Avatar::new("did:aeth:test".to_string(), create_test_profile());
        assert_eq!(avatar.did, "did:aeth:test");
        assert_eq!(avatar.profile.name, "TestAvatar");
        assert!(!avatar.is_online);
        assert!(avatar.node_id.is_none());
    }

    #[tokio::test]
    async fn test_avatar_permissions() {
        let mut avatar = Avatar::new("did:aeth:test".to_string(), create_test_profile());
        
        assert!(avatar.has_permission("move"));
        assert!(avatar.has_permission("interact"));
        assert!(!avatar.has_permission("admin"));

        avatar.add_permission("admin".to_string());
        assert!(avatar.has_permission("admin"));

        avatar.remove_permission("move");
        assert!(!avatar.has_permission("move"));
    }

    #[tokio::test]
    async fn test_avatar_manager_creation() {
        let manager = AvatarManager::new(100);
        let count = manager.get_avatar_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_avatar_manager_operations() {
        let manager = AvatarManager::new(100);
        let profile = create_test_profile();
        
        // Create avatar
        let avatar_id = manager.create_avatar("did:aeth:test".to_string(), profile).await.unwrap();
        assert_eq!(manager.get_avatar_count().await, 1);
        
        // Get avatar
        let avatar = manager.get_avatar(avatar_id).await.unwrap();
        assert_eq!(avatar.did, "did:aeth:test");
        
        // Get avatar by DID
        let avatar_by_did = manager.get_avatar_by_did("did:aeth:test").await.unwrap();
        assert_eq!(avatar_by_did.id, avatar_id);
        
        // Update avatar
        let transform = Transform::new([1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0]);
        manager.update_avatar(avatar_id, AvatarUpdate::Transform(transform)).await.unwrap();
        
        let updated_avatar = manager.get_avatar(avatar_id).await.unwrap();
        assert_eq!(updated_avatar.transform.position, [1.0, 2.0, 3.0]);
        
        // Delete avatar
        manager.delete_avatar(avatar_id).await.unwrap();
        assert_eq!(manager.get_avatar_count().await, 0);
    }

    #[tokio::test]
    async fn test_avatar_manager_duplicate_did() {
        let manager = AvatarManager::new(100);
        let profile = create_test_profile();
        
        // Create first avatar
        manager.create_avatar("did:aeth:test".to_string(), profile.clone()).await.unwrap();
        
        // Try to create second avatar with same DID
        let result = manager.create_avatar("did:aeth:test".to_string(), profile).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_avatar_manager_online_status() {
        let manager = AvatarManager::new(100);
        let profile = create_test_profile();
        
        let avatar_id = manager.create_avatar("did:aeth:test".to_string(), profile).await.unwrap();
        
        // Set online
        manager.update_avatar(avatar_id, AvatarUpdate::OnlineStatus(true)).await.unwrap();
        assert_eq!(manager.get_online_avatar_count().await, 1);
        
        // Set offline
        manager.update_avatar(avatar_id, AvatarUpdate::OnlineStatus(false)).await.unwrap();
        assert_eq!(manager.get_online_avatar_count().await, 0);
    }
}
