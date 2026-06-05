//! Storage and snapshot management for XR scenes

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{SceneError, SceneResult};
use crate::scene::{SceneNode, NodeId};
use crate::avatar::Avatar;

/// Unique identifier for a snapshot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotId(pub Uuid);

impl SnapshotId {
    /// Generate a new snapshot ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a snapshot ID from a string
    pub fn from_string(s: &str) -> SceneResult<Self> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| SceneError::InvalidInput(format!("Invalid snapshot ID: {}", s)))?;
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for SnapshotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Scene snapshot containing the complete state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSnapshot {
    pub id: SnapshotId,
    pub label: String,
    pub description: Option<String>,
    pub nodes: Vec<SceneNode>,
    pub avatars: Vec<Avatar>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: u64,
    pub created_by: String,
    pub version: u32,
}

impl SceneSnapshot {
    /// Create a new scene snapshot
    pub fn new(label: String, created_by: String) -> Self {
        Self {
            id: SnapshotId::new(),
            label,
            description: None,
            nodes: Vec::new(),
            avatars: Vec::new(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now().timestamp() as u64,
            created_by,
            version: 1,
        }
    }

    /// Add a node to the snapshot
    pub fn add_node(&mut self, node: SceneNode) {
        self.nodes.push(node);
    }

    /// Add an avatar to the snapshot
    pub fn add_avatar(&mut self, avatar: Avatar) {
        self.avatars.push(avatar);
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Serialize to CBOR bytes
    pub fn to_cbor(&self) -> SceneResult<Vec<u8>> {
        serde_cbor::to_vec(self)
            .map_err(|e| SceneError::SerializationError(e.into()))
    }

    /// Deserialize from CBOR bytes
    pub fn from_cbor(data: &[u8]) -> SceneResult<Self> {
        serde_cbor::from_slice(data)
            .map_err(|e| SceneError::SerializationError(e.into()))
    }

    /// Calculate the hash of the snapshot
    pub fn calculate_hash(&self) -> String {
        let data = self.to_cbor().unwrap_or_default();
        blake3::hash(&data).to_hex()
    }

    /// Validate the snapshot
    pub fn validate(&self) -> SceneResult<()> {
        if self.label.trim().is_empty() {
            return Err(SceneError::InvalidInput("Snapshot label cannot be empty".to_string()));
        }

        if self.created_by.trim().is_empty() {
            return Err(SceneError::InvalidInput("Snapshot creator cannot be empty".to_string()));
        }

        if self.version == 0 {
            return Err(SceneError::InvalidInput("Snapshot version must be greater than 0".to_string()));
        }

        // Validate all nodes
        for node in &self.nodes {
            node.transform.validate()?;
            
            // Check for duplicate node IDs
            let duplicate_count = self.nodes.iter()
                .filter(|n| n.id == node.id)
                .count();
            if duplicate_count > 1 {
                return Err(SceneError::InvalidInput(format!("Duplicate node ID: {}", node.id)));
            }
        }

        // Validate all avatars
        for avatar in &self.avatars {
            avatar.transform.validate()?;
            
            // Check for duplicate avatar IDs
            let duplicate_count = self.avatars.iter()
                .filter(|a| a.id == avatar.id)
                .count();
            if duplicate_count > 1 {
                return Err(SceneError::InvalidInput(format!("Duplicate avatar ID: {}", avatar.id)));
            }

            // Check for duplicate DIDs
            let duplicate_did_count = self.avatars.iter()
                .filter(|a| a.did == avatar.did)
                .count();
            if duplicate_did_count > 1 {
                return Err(SceneError::InvalidInput(format!("Duplicate avatar DID: {}", avatar.did)));
            }
        }

        Ok(())
    }
}

/// Snapshot manager handling scene snapshots
pub struct SnapshotManager {
    snapshots: Arc<RwLock<HashMap<SnapshotId, SceneSnapshot>>>,
    label_to_snapshot: Arc<RwLock<HashMap<String, SnapshotId>>>,
    ngfs_client: Option<Arc<ngfs::NGFSClient>>,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            label_to_snapshot: Arc::new(RwLock::new(HashMap::new())),
            ngfs_client: None,
        }
    }

    /// Create a new snapshot manager with NGFS client
    pub fn with_ngfs_client(ngfs_client: Arc<ngfs::NGFSClient>) -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            label_to_snapshot: Arc::new(RwLock::new(HashMap::new())),
            ngfs_client: Some(ngfs_client),
        }
    }

    /// Create a new snapshot
    pub async fn create_snapshot(&self, label: String, created_by: String) -> SceneResult<SnapshotId> {
        let mut snapshots = self.snapshots.write().await;
        let mut label_to_snapshot = self.label_to_snapshot.write().await;

        // Check if snapshot with this label already exists
        if label_to_snapshot.contains_key(&label) {
            return Err(SceneError::InvalidInput(format!("Snapshot with label already exists: {}", label)));
        }

        let snapshot = SceneSnapshot::new(label.clone(), created_by);
        let snapshot_id = snapshot.id;

        label_to_snapshot.insert(label, snapshot_id);
        snapshots.insert(snapshot_id, snapshot);

        Ok(snapshot_id)
    }

    /// Get a snapshot by ID
    pub async fn get_snapshot(&self, snapshot_id: SnapshotId) -> SceneResult<SceneSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.get(&snapshot_id)
            .cloned()
            .ok_or_else(|| SceneError::SnapshotNotFound(format!("Snapshot not found: {}", snapshot_id)))
    }

    /// Get a snapshot by label
    pub async fn get_snapshot_by_label(&self, label: &str) -> SceneResult<SceneSnapshot> {
        let label_to_snapshot = self.label_to_snapshot.read().await;
        let snapshot_id = label_to_snapshot.get(label)
            .ok_or_else(|| SceneError::SnapshotNotFound(format!("Snapshot with label not found: {}", label)))?;

        self.get_snapshot(*snapshot_id).await
    }

    /// Update a snapshot
    pub async fn update_snapshot(&self, snapshot_id: SnapshotId, updates: SnapshotUpdate) -> SceneResult<()> {
        let mut snapshots = self.snapshots.write().await;

        if let Some(snapshot) = snapshots.get_mut(&snapshot_id) {
            match updates {
                SnapshotUpdate::AddNode(node) => {
                    snapshot.add_node(node);
                }
                SnapshotUpdate::AddAvatar(avatar) => {
                    snapshot.add_avatar(avatar);
                }
                SnapshotUpdate::SetMetadata(key, value) => {
                    snapshot.set_metadata(key, value);
                }
                SnapshotUpdate::SetDescription(description) => {
                    snapshot.description = Some(description);
                }
                SnapshotUpdate::IncrementVersion => {
                    snapshot.version += 1;
                }
            }
        } else {
            return Err(SceneError::SnapshotNotFound(format!("Snapshot not found: {}", snapshot_id)));
        }

        Ok(())
    }

    /// Delete a snapshot
    pub async fn delete_snapshot(&self, snapshot_id: SnapshotId) -> SceneResult<()> {
        let mut snapshots = self.snapshots.write().await;
        let mut label_to_snapshot = self.label_to_snapshot.write().await;

        if let Some(snapshot) = snapshots.remove(&snapshot_id) {
            label_to_snapshot.remove(&snapshot.label);
            Ok(())
        } else {
            Err(SceneError::SnapshotNotFound(format!("Snapshot not found: {}", snapshot_id)))
        }
    }

    /// Get all snapshots
    pub async fn get_all_snapshots(&self) -> Vec<SceneSnapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.values().cloned().collect()
    }

    /// Get snapshot count
    pub async fn get_snapshot_count(&self) -> usize {
        let snapshots = self.snapshots.read().await;
        snapshots.len()
    }

    /// Save snapshot to NGFS
    pub async fn save_to_ngfs(&self, snapshot_id: SnapshotId) -> SceneResult<String> {
        let snapshot = self.get_snapshot(snapshot_id).await?;
        snapshot.validate()?;

        if let Some(ngfs_client) = &self.ngfs_client {
            let cbor_data = snapshot.to_cbor()?;
            let hash = ngfs_client.store(&cbor_data).await?;
            Ok(hash)
        } else {
            Err(SceneError::NGFSError(ngfs::NGFSError::ClientNotInitialized))
        }
    }

    /// Load snapshot from NGFS
    pub async fn load_from_ngfs(&self, hash: &str) -> SceneResult<SnapshotId> {
        if let Some(ngfs_client) = &self.ngfs_client {
            let cbor_data = ngfs_client.retrieve(hash).await?;
            let snapshot = SceneSnapshot::from_cbor(&cbor_data)?;
            snapshot.validate()?;

            let snapshot_id = snapshot.id;
            let label = snapshot.label.clone();

            // Store in local cache
            let mut snapshots = self.snapshots.write().await;
            let mut label_to_snapshot = self.label_to_snapshot.write().await;

            // Check for conflicts
            if snapshots.contains_key(&snapshot_id) {
                return Err(SceneError::InvalidInput(format!("Snapshot with ID already exists: {}", snapshot_id)));
            }

            if label_to_snapshot.contains_key(&label) {
                return Err(SceneError::InvalidInput(format!("Snapshot with label already exists: {}", label)));
            }

            label_to_snapshot.insert(label, snapshot_id);
            snapshots.insert(snapshot_id, snapshot);

            Ok(snapshot_id)
        } else {
            Err(SceneError::NGFSError(ngfs::NGFSError::ClientNotInitialized))
        }
    }

    /// Export snapshot to file
    pub async fn export_snapshot(&self, snapshot_id: SnapshotId, file_path: &str) -> SceneResult<()> {
        let snapshot = self.get_snapshot(snapshot_id).await?;
        let cbor_data = snapshot.to_cbor()?;
        
        tokio::fs::write(file_path, cbor_data).await?;
        Ok(())
    }

    /// Import snapshot from file
    pub async fn import_snapshot(&self, file_path: &str) -> SceneResult<SnapshotId> {
        let cbor_data = tokio::fs::read(file_path).await?;
        let snapshot = SceneSnapshot::from_cbor(&cbor_data)?;
        snapshot.validate()?;

        let snapshot_id = snapshot.id;
        let label = snapshot.label.clone();

        // Store in local cache
        let mut snapshots = self.snapshots.write().await;
        let mut label_to_snapshot = self.label_to_snapshot.write().await;

        // Check for conflicts
        if snapshots.contains_key(&snapshot_id) {
            return Err(SceneError::InvalidInput(format!("Snapshot with ID already exists: {}", snapshot_id)));
        }

        if label_to_snapshot.contains_key(&label) {
            return Err(SceneError::InvalidInput(format!("Snapshot with label already exists: {}", label)));
        }

        label_to_snapshot.insert(label, snapshot_id);
        snapshots.insert(snapshot_id, snapshot);

        Ok(snapshot_id)
    }

    /// Get snapshot statistics
    pub async fn get_snapshot_stats(&self, snapshot_id: SnapshotId) -> SceneResult<SnapshotStats> {
        let snapshot = self.get_snapshot(snapshot_id).await?;
        
        Ok(SnapshotStats {
            id: snapshot.id,
            label: snapshot.label,
            node_count: snapshot.nodes.len(),
            avatar_count: snapshot.avatars.len(),
            online_avatar_count: snapshot.avatars.iter().filter(|a| a.is_online).count(),
            created_at: snapshot.created_at,
            version: snapshot.version,
            size_bytes: snapshot.to_cbor()?.len(),
        })
    }
}

/// Snapshot update operations
#[derive(Debug, Clone)]
pub enum SnapshotUpdate {
    AddNode(SceneNode),
    AddAvatar(Avatar),
    SetMetadata(String, serde_json::Value),
    SetDescription(String),
    IncrementVersion,
}

/// Snapshot statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotStats {
    pub id: SnapshotId,
    pub label: String,
    pub node_count: usize,
    pub avatar_count: usize,
    pub online_avatar_count: usize,
    pub created_at: u64,
    pub version: u32,
    pub size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::SceneNode;

    #[tokio::test]
    async fn test_snapshot_creation() {
        let snapshot = SceneSnapshot::new("test_snapshot".to_string(), "test_user".to_string());
        assert_eq!(snapshot.label, "test_snapshot");
        assert_eq!(snapshot.created_by, "test_user");
        assert_eq!(snapshot.version, 1);
        assert!(snapshot.nodes.is_empty());
        assert!(snapshot.avatars.is_empty());
    }

    #[tokio::test]
    async fn test_snapshot_validation() {
        let mut snapshot = SceneSnapshot::new("test_snapshot".to_string(), "test_user".to_string());
        
        // Valid snapshot
        assert!(snapshot.validate().is_ok());
        
        // Invalid label
        snapshot.label = "".to_string();
        assert!(snapshot.validate().is_err());
        
        // Invalid creator
        snapshot.label = "test_snapshot".to_string();
        snapshot.created_by = "".to_string();
        assert!(snapshot.validate().is_err());
        
        // Invalid version
        snapshot.created_by = "test_user".to_string();
        snapshot.version = 0;
        assert!(snapshot.validate().is_err());
    }

    #[tokio::test]
    async fn test_snapshot_serialization() {
        let snapshot = SceneSnapshot::new("test_snapshot".to_string(), "test_user".to_string());
        
        let cbor_data = snapshot.to_cbor().unwrap();
        let deserialized = SceneSnapshot::from_cbor(&cbor_data).unwrap();
        
        assert_eq!(snapshot.label, deserialized.label);
        assert_eq!(snapshot.created_by, deserialized.created_by);
        assert_eq!(snapshot.version, deserialized.version);
    }

    #[tokio::test]
    async fn test_snapshot_manager_creation() {
        let manager = SnapshotManager::new();
        let count = manager.get_snapshot_count().await;
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_snapshot_manager_operations() {
        let manager = SnapshotManager::new();
        
        // Create snapshot
        let snapshot_id = manager.create_snapshot("test_snapshot".to_string(), "test_user".to_string()).await.unwrap();
        assert_eq!(manager.get_snapshot_count().await, 1);
        
        // Get snapshot
        let snapshot = manager.get_snapshot(snapshot_id).await.unwrap();
        assert_eq!(snapshot.label, "test_snapshot");
        
        // Get snapshot by label
        let snapshot_by_label = manager.get_snapshot_by_label("test_snapshot").await.unwrap();
        assert_eq!(snapshot_by_label.id, snapshot_id);
        
        // Update snapshot
        let node = SceneNode::new("test_node".to_string());
        manager.update_snapshot(snapshot_id, SnapshotUpdate::AddNode(node)).await.unwrap();
        
        let updated_snapshot = manager.get_snapshot(snapshot_id).await.unwrap();
        assert_eq!(updated_snapshot.nodes.len(), 1);
        
        // Delete snapshot
        manager.delete_snapshot(snapshot_id).await.unwrap();
        assert_eq!(manager.get_snapshot_count().await, 0);
    }

    #[tokio::test]
    async fn test_snapshot_manager_duplicate_label() {
        let manager = SnapshotManager::new();
        
        // Create first snapshot
        manager.create_snapshot("test_snapshot".to_string(), "test_user".to_string()).await.unwrap();
        
        // Try to create second snapshot with same label
        let result = manager.create_snapshot("test_snapshot".to_string(), "test_user2".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_snapshot_export_import() {
        let manager = SnapshotManager::new();
        
        // Create snapshot
        let snapshot_id = manager.create_snapshot("test_snapshot".to_string(), "test_user".to_string()).await.unwrap();
        
        // Export snapshot
        let temp_file = "test_snapshot.cbor";
        manager.export_snapshot(snapshot_id, temp_file).await.unwrap();
        
        // Create new manager and import snapshot
        let new_manager = SnapshotManager::new();
        let imported_id = new_manager.import_snapshot(temp_file).await.unwrap();
        
        // Verify imported snapshot
        let imported_snapshot = new_manager.get_snapshot(imported_id).await.unwrap();
        assert_eq!(imported_snapshot.label, "test_snapshot");
        assert_eq!(imported_snapshot.created_by, "test_user");
        
        // Clean up
        let _ = tokio::fs::remove_file(temp_file).await;
    }
}
