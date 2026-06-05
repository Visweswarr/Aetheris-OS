//! Scene graph implementation for XR scenes

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{SceneError, SceneResult};

/// Unique identifier for a scene node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    /// Generate a new node ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a node ID from a string
    pub fn from_string(s: &str) -> SceneResult<Self> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| SceneError::InvalidInput(format!("Invalid node ID: {}", s)))?;
        Ok(Self(uuid))
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 3D transform for scene nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // quaternion (x, y, z, w)
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
            scale: [1.0, 1.0, 1.0],
        }
    }
}

impl Transform {
    /// Create a new transform
    pub fn new(position: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Create an identity transform
    pub fn identity() -> Self {
        Self::default()
    }

    /// Validate the transform
    pub fn validate(&self) -> SceneResult<()> {
        // Check for NaN or infinite values
        for &val in &self.position {
            if !val.is_finite() {
                return Err(SceneError::InvalidTransform(format!("Invalid position: {}", val)));
            }
        }

        for &val in &self.rotation {
            if !val.is_finite() {
                return Err(SceneError::InvalidTransform(format!("Invalid rotation: {}", val)));
            }
        }

        for &val in &self.scale {
            if !val.is_finite() || val <= 0.0 {
                return Err(SceneError::InvalidTransform(format!("Invalid scale: {}", val)));
            }
        }

        // Check quaternion normalization (approximately)
        let norm = self.rotation[0] * self.rotation[0] + 
                   self.rotation[1] * self.rotation[1] + 
                   self.rotation[2] * self.rotation[2] + 
                   self.rotation[3] * self.rotation[3];
        
        if (norm - 1.0).abs() > 0.01 {
            return Err(SceneError::InvalidTransform(format!("Quaternion not normalized: {}", norm)));
        }

        Ok(())
    }
}

/// Component types for scene nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Component {
    /// Mesh component with geometry data
    Mesh {
        mesh_id: String,
        material_id: Option<String>,
    },
    /// Light component
    Light {
        light_type: LightType,
        color: [f32; 3],
        intensity: f32,
        range: Option<f32>,
    },
    /// Collider component for physics
    Collider {
        collider_type: ColliderType,
        size: [f32; 3],
        is_trigger: bool,
    },
    /// Script component for behavior
    Script {
        script_id: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    /// Camera component
    Camera {
        fov: f32,
        near_plane: f32,
        far_plane: f32,
    },
}

/// Light types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

/// Collider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderType {
    Box,
    Sphere,
    Capsule,
    Mesh,
}

/// Scene node representing an object in the 3D scene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: NodeId,
    pub name: String,
    pub parent_id: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub transform: Transform,
    pub components: Vec<Component>,
    pub visible: bool,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

impl SceneNode {
    /// Create a new scene node
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().timestamp() as u64;
        Self {
            id: NodeId::new(),
            name,
            parent_id: None,
            children: Vec::new(),
            transform: Transform::identity(),
            components: Vec::new(),
            visible: true,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a component to the node
    pub fn add_component(&mut self, component: Component) -> SceneResult<()> {
        // Validate component
        match &component {
            Component::Light { intensity, range, .. } => {
                if !intensity.is_finite() || *intensity < 0.0 {
                    return Err(SceneError::InvalidComponent(format!("Invalid light intensity: {}", intensity)));
                }
                if let Some(range) = range {
                    if !range.is_finite() || *range <= 0.0 {
                        return Err(SceneError::InvalidComponent(format!("Invalid light range: {}", range)));
                    }
                }
            }
            Component::Camera { fov, near_plane, far_plane } => {
                if !fov.is_finite() || *fov <= 0.0 || *fov >= 180.0 {
                    return Err(SceneError::InvalidComponent(format!("Invalid camera FOV: {}", fov)));
                }
                if !near_plane.is_finite() || *near_plane <= 0.0 {
                    return Err(SceneError::InvalidComponent(format!("Invalid camera near plane: {}", near_plane)));
                }
                if !far_plane.is_finite() || *far_plane <= *near_plane {
                    return Err(SceneError::InvalidComponent(format!("Invalid camera far plane: {}", far_plane)));
                }
            }
            _ => {}
        }

        self.components.push(component);
        self.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(())
    }

    /// Remove a component by type
    pub fn remove_component(&mut self, component_type: &str) -> SceneResult<()> {
        let initial_len = self.components.len();
        self.components.retain(|c| {
            match c {
                Component::Mesh { .. } => component_type != "mesh",
                Component::Light { .. } => component_type != "light",
                Component::Collider { .. } => component_type != "collider",
                Component::Script { .. } => component_type != "script",
                Component::Camera { .. } => component_type != "camera",
            }
        });

        if self.components.len() == initial_len {
            return Err(SceneError::InvalidComponent(format!("Component type not found: {}", component_type)));
        }

        self.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(())
    }

    /// Get a component by type
    pub fn get_component(&self, component_type: &str) -> Option<&Component> {
        self.components.iter().find(|c| {
            match c {
                Component::Mesh { .. } => component_type == "mesh",
                Component::Light { .. } => component_type == "light",
                Component::Collider { .. } => component_type == "collider",
                Component::Script { .. } => component_type == "script",
                Component::Camera { .. } => component_type == "camera",
            }
        })
    }

    /// Update the transform
    pub fn update_transform(&mut self, transform: Transform) -> SceneResult<()> {
        transform.validate()?;
        self.transform = transform;
        self.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(())
    }

    /// Add a child node
    pub fn add_child(&mut self, child_id: NodeId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
            self.updated_at = chrono::Utc::now().timestamp() as u64;
        }
    }

    /// Remove a child node
    pub fn remove_child(&mut self, child_id: NodeId) -> SceneResult<()> {
        let initial_len = self.children.len();
        self.children.retain(|&id| id != child_id);
        
        if self.children.len() == initial_len {
            return Err(SceneError::NodeNotFound(format!("Child node not found: {}", child_id)));
        }

        self.updated_at = chrono::Utc::now().timestamp() as u64;
        Ok(())
    }
}

/// Scene graph managing all nodes in the 3D scene
pub struct SceneGraph {
    nodes: Arc<RwLock<HashMap<NodeId, SceneNode>>>,
    root_nodes: Arc<RwLock<Vec<NodeId>>>,
    max_nodes: usize,
    tick_count: Arc<RwLock<u64>>,
}

impl SceneGraph {
    /// Create a new scene graph
    pub fn new(max_nodes: usize) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            root_nodes: Arc::new(RwLock::new(Vec::new())),
            max_nodes,
            tick_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new node
    pub async fn create_node(&self, name: String, parent_id: Option<NodeId>) -> SceneResult<NodeId> {
        let mut nodes = self.nodes.write().await;
        
        if nodes.len() >= self.max_nodes {
            return Err(SceneError::SceneGraphFull);
        }

        let mut node = SceneNode::new(name);
        let node_id = node.id;

        // Set parent if specified
        if let Some(parent_id) = parent_id {
            if let Some(parent) = nodes.get_mut(&parent_id) {
                parent.add_child(node_id);
                node.parent_id = Some(parent_id);
            } else {
                return Err(SceneError::NodeNotFound(format!("Parent node not found: {}", parent_id)));
            }
        } else {
            // Add to root nodes
            let mut root_nodes = self.root_nodes.write().await;
            root_nodes.push(node_id);
        }

        nodes.insert(node_id, node);
        Ok(node_id)
    }

    /// Get a node by ID
    pub async fn get_node(&self, node_id: NodeId) -> SceneResult<SceneNode> {
        let nodes = self.nodes.read().await;
        nodes.get(&node_id)
            .cloned()
            .ok_or_else(|| SceneError::NodeNotFound(format!("Node not found: {}", node_id)))
    }

    /// Update a node
    pub async fn update_node(&self, node_id: NodeId, updates: NodeUpdate) -> SceneResult<()> {
        let mut nodes = self.nodes.write().await;
        
        if let Some(node) = nodes.get_mut(&node_id) {
            match updates {
                NodeUpdate::Transform(transform) => {
                    node.update_transform(transform)?;
                }
                NodeUpdate::Components(components) => {
                    node.components = components;
                    node.updated_at = chrono::Utc::now().timestamp() as u64;
                }
                NodeUpdate::Visibility(visible) => {
                    node.visible = visible;
                    node.updated_at = chrono::Utc::now().timestamp() as u64;
                }
                NodeUpdate::Active(active) => {
                    node.active = active;
                    node.updated_at = chrono::Utc::now().timestamp() as u64;
                }
            }
        } else {
            return Err(SceneError::NodeNotFound(format!("Node not found: {}", node_id)));
        }

        Ok(())
    }

    /// Delete a node and all its children
    pub async fn delete_node(&self, node_id: NodeId) -> SceneResult<()> {
        let mut nodes = self.nodes.write().await;
        let mut root_nodes = self.root_nodes.write().await;

        // Recursively delete children
        if let Some(node) = nodes.get(&node_id) {
            for &child_id in &node.children {
                self.delete_node_recursive(child_id, &mut nodes).await?;
            }
        }

        // Remove from parent's children list
        if let Some(node) = nodes.get(&node_id) {
            if let Some(parent_id) = node.parent_id {
                if let Some(parent) = nodes.get_mut(&parent_id) {
                    parent.remove_child(node_id)?;
                }
            } else {
                // Remove from root nodes
                root_nodes.retain(|&id| id != node_id);
            }
        }

        // Remove the node
        nodes.remove(&node_id)
            .ok_or_else(|| SceneError::NodeNotFound(format!("Node not found: {}", node_id)))?;

        Ok(())
    }

    /// Recursively delete a node and its children
    async fn delete_node_recursive(&self, node_id: NodeId, nodes: &mut HashMap<NodeId, SceneNode>) -> SceneResult<()> {
        if let Some(node) = nodes.get(&node_id) {
            for &child_id in &node.children {
                self.delete_node_recursive(child_id, nodes).await?;
            }
        }
        nodes.remove(&node_id);
        Ok(())
    }

    /// Get all nodes
    pub async fn get_all_nodes(&self) -> Vec<SceneNode> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    /// Get root nodes
    pub async fn get_root_nodes(&self) -> Vec<NodeId> {
        let root_nodes = self.root_nodes.read().await;
        root_nodes.clone()
    }

    /// Get children of a node
    pub async fn get_children(&self, node_id: NodeId) -> SceneResult<Vec<NodeId>> {
        let nodes = self.nodes.read().await;
        let node = nodes.get(&node_id)
            .ok_or_else(|| SceneError::NodeNotFound(format!("Node not found: {}", node_id)))?;
        Ok(node.children.clone())
    }

    /// Perform a scene graph tick
    pub async fn tick(&self) {
        let mut tick_count = self.tick_count.write().await;
        *tick_count += 1;
        
        // Update all active nodes
        let mut nodes = self.nodes.write().await;
        for node in nodes.values_mut() {
            if node.active {
                // Update node state (e.g., run scripts, update physics)
                // This is where node-specific update logic would go
            }
        }
    }

    /// Get the current tick count
    pub async fn get_tick_count(&self) -> u64 {
        let tick_count = self.tick_count.read().await;
        *tick_count
    }

    /// Get the number of nodes
    pub async fn get_node_count(&self) -> usize {
        let nodes = self.nodes.read().await;
        nodes.len()
    }
}

/// Node update operations
#[derive(Debug, Clone)]
pub enum NodeUpdate {
    Transform(Transform),
    Components(Vec<Component>),
    Visibility(bool),
    Active(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scene_node_creation() {
        let node = SceneNode::new("TestNode".to_string());
        assert_eq!(node.name, "TestNode");
        assert!(node.visible);
        assert!(node.active);
        assert!(node.children.is_empty());
        assert!(node.components.is_empty());
    }

    #[tokio::test]
    async fn test_scene_node_transform() {
        let mut node = SceneNode::new("TestNode".to_string());
        let transform = Transform::new(
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0],
            [1.0, 1.0, 1.0]
        );
        
        let result = node.update_transform(transform.clone());
        assert!(result.is_ok());
        assert_eq!(node.transform.position, [1.0, 2.0, 3.0]);
    }

    #[tokio::test]
    async fn test_scene_node_components() {
        let mut node = SceneNode::new("TestNode".to_string());
        
        let light_component = Component::Light {
            light_type: LightType::Point,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: Some(10.0),
        };
        
        let result = node.add_component(light_component.clone());
        assert!(result.is_ok());
        assert_eq!(node.components.len(), 1);
        
        let retrieved = node.get_component("light");
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_scene_graph_creation() {
        let scene_graph = SceneGraph::new(1000);
        let node_count = scene_graph.get_node_count().await;
        assert_eq!(node_count, 0);
    }

    #[tokio::test]
    async fn test_scene_graph_node_operations() {
        let scene_graph = SceneGraph::new(1000);
        
        // Create a root node
        let node_id = scene_graph.create_node("RootNode".to_string(), None).await.unwrap();
        assert_eq!(scene_graph.get_node_count().await, 1);
        
        // Create a child node
        let child_id = scene_graph.create_node("ChildNode".to_string(), Some(node_id)).await.unwrap();
        assert_eq!(scene_graph.get_node_count().await, 2);
        
        // Get the root node and check it has a child
        let root_node = scene_graph.get_node(node_id).await.unwrap();
        assert_eq!(root_node.children.len(), 1);
        assert_eq!(root_node.children[0], child_id);
        
        // Delete the root node (should delete both nodes)
        scene_graph.delete_node(node_id).await.unwrap();
        assert_eq!(scene_graph.get_node_count().await, 0);
    }

    #[tokio::test]
    async fn test_scene_graph_tick() {
        let scene_graph = SceneGraph::new(1000);
        let initial_tick = scene_graph.get_tick_count().await;
        
        scene_graph.tick().await;
        let new_tick = scene_graph.get_tick_count().await;
        
        assert_eq!(new_tick, initial_tick + 1);
    }
}
