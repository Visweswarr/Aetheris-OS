//! Capability management for XR scenes

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::error::{SceneError, SceneResult};
use crate::scene::{SceneNode, NodeId};
use crate::avatar::Avatar;
use crate::session::{SessionId, SessionManager};

/// Capability token for access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapToken {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
    pub scope: CapScope,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub version: u32,
    pub signature: Option<String>,
    /// Session ID that issued this token
    pub session_id: Option<SessionId>,
    /// Scopes granted by this token
    pub granted_scopes: Vec<String>,
}

impl CapToken {
    /// Create a new capability token
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            permissions: Vec::new(),
            scope: CapScope::Scene,
            expires_at: None,
            created_at: Utc::now(),
            created_by: String::new(),
            version: 1,
            signature: None,
            session_id: None,
            granted_scopes: Vec::new(),
        }
    }

    /// Add a permission to the token
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }

    /// Set the scope of the token
    pub fn set_scope(&mut self, scope: CapScope) {
        self.scope = scope;
    }

    /// Set the expiration time
    pub fn set_expiration(&mut self, expires_at: DateTime<Utc>) {
        self.expires_at = Some(expires_at);
    }

    /// Set the creator
    pub fn set_creator(&mut self, created_by: String) {
        self.created_by = created_by;
    }

    /// Set the session ID that issued this token
    pub fn set_session_id(&mut self, session_id: SessionId) {
        self.session_id = Some(session_id);
    }

    /// Add a granted scope
    pub fn add_granted_scope(&mut self, scope: String) {
        if !self.granted_scopes.contains(&scope) {
            self.granted_scopes.push(scope);
        }
    }

    /// Check if token has a specific granted scope
    pub fn has_granted_scope(&self, scope: &str) -> bool {
        self.granted_scopes.contains(&scope.to_string())
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if the token has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// Check if the token has permission for a specific action
    pub fn has_action_permission(&self, action: &CapAction) -> bool {
        self.permissions.iter().any(|p| p.action == *action)
    }

    /// Validate the token
    pub fn validate(&self) -> SceneResult<()> {
        if self.name.trim().is_empty() {
            return Err(SceneError::InvalidInput("Capability token name cannot be empty".to_string()));
        }

        if self.created_by.trim().is_empty() {
            return Err(SceneError::InvalidInput("Capability token creator cannot be empty".to_string()));
        }

        if self.version == 0 {
            return Err(SceneError::InvalidInput("Capability token version must be greater than 0".to_string()));
        }

        if self.is_expired() {
            return Err(SceneError::CapabilityError("Capability token has expired".to_string()));
        }

        // Validate all permissions
        for permission in &self.permissions {
            permission.validate()?;
        }

        Ok(())
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

    /// Calculate the hash of the token
    pub fn calculate_hash(&self) -> String {
        let data = self.to_cbor().unwrap_or_default();
        blake3::hash(&data).to_hex()
    }
}

/// Capability scope
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapScope {
    /// Apply to entire scene
    Scene,
    /// Apply to specific node
    Node(NodeId),
    /// Apply to specific avatar
    Avatar(String), // DID
    /// Apply to specific user
    User(String), // DID
    /// Apply to specific component
    Component(String),
}

/// Capability action
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapAction {
    /// Enter the scene
    Enter,
    /// Exit the scene
    Exit,
    /// Spawn a new node
    Spawn,
    /// Modify an existing node
    Modify,
    /// Delete a node
    Delete,
    /// Move a node
    Move,
    /// Attach a component
    AttachComponent,
    /// Detach a component
    DetachComponent,
    /// Bind an avatar
    BindAvatar,
    /// Unbind an avatar
    UnbindAvatar,
    /// Create a snapshot
    CreateSnapshot,
    /// Load a snapshot
    LoadSnapshot,
    /// Export scene
    Export,
    /// Import scene
    Import,
    /// Read scene data
    Read,
    /// Write scene data
    Write,
    /// Execute scripts
    Execute,
    /// Manage policies
    ManagePolicies,
    /// Manage capabilities
    ManageCapabilities,
    /// Begin a session
    BeginSession,
    /// End a session
    EndSession,
    /// Issue capability tokens
    IssueCap,
    /// Attach policies
    AttachPolicy,
    /// Simulate operations
    Simulate,
}

/// Permission for a capability token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub action: CapAction,
    pub scope: CapScope,
    pub conditions: Vec<String>, // Rego conditions
    pub priority: u32,
}

impl Permission {
    /// Create a new permission
    pub fn new(action: CapAction, scope: CapScope) -> Self {
        Self {
            action,
            scope,
            conditions: Vec::new(),
            priority: 100,
        }
    }

    /// Add a condition to the permission
    pub fn add_condition(&mut self, condition: String) {
        self.conditions.push(condition);
    }

    /// Set the priority of the permission
    pub fn set_priority(&mut self, priority: u32) {
        self.priority = priority;
    }

    /// Validate the permission
    pub fn validate(&self) -> SceneResult<()> {
        if self.priority == 0 {
            return Err(SceneError::InvalidInput("Permission priority must be greater than 0".to_string()));
        }

        Ok(())
    }
}

/// Capability manager for handling scene capabilities
pub struct CapManager {
    tokens: Arc<RwLock<HashMap<String, CapToken>>>,
    user_tokens: Arc<RwLock<HashMap<String, Vec<String>>>>, // DID -> token IDs
    node_tokens: Arc<RwLock<HashMap<NodeId, Vec<String>>>>, // Node ID -> token IDs
    avatar_tokens: Arc<RwLock<HashMap<String, Vec<String>>>>, // Avatar ID -> token IDs
    session_tokens: Arc<RwLock<HashMap<SessionId, Vec<String>>>>, // Session ID -> token IDs
}

impl CapManager {
    /// Create a new capability manager
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            user_tokens: Arc::new(RwLock::new(HashMap::new())),
            node_tokens: Arc::new(RwLock::new(HashMap::new())),
            avatar_tokens: Arc::new(RwLock::new(HashMap::new())),
            session_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new capability token
    pub async fn create_token(&self, name: String, created_by: String) -> SceneResult<String> {
        let mut token = CapToken::new(name);
        token.set_creator(created_by);
        token.validate()?;

        let token_id = token.id.clone();
        let mut tokens = self.tokens.write().await;
        tokens.insert(token_id.clone(), token);

        Ok(token_id)
    }

    /// Get a capability token
    pub async fn get_token(&self, token_id: &str) -> SceneResult<CapToken> {
        let tokens = self.tokens.read().await;
        tokens.get(token_id)
            .cloned()
            .ok_or_else(|| SceneError::CapabilityError(format!("Capability token not found: {}", token_id)))
    }

    /// Update a capability token
    pub async fn update_token(&self, token_id: &str, updates: CapTokenUpdate) -> SceneResult<()> {
        let mut tokens = self.tokens.write().await;

        if let Some(token) = tokens.get_mut(token_id) {
            match updates {
                CapTokenUpdate::AddPermission(permission) => {
                    token.add_permission(permission);
                }
                CapTokenUpdate::SetScope(scope) => {
                    token.set_scope(scope);
                }
                CapTokenUpdate::SetExpiration(expires_at) => {
                    token.set_expiration(expires_at);
                }
                CapTokenUpdate::SetDescription(description) => {
                    token.description = Some(description);
                }
                CapTokenUpdate::IncrementVersion => {
                    token.version += 1;
                }
            }

            token.validate()?;
        } else {
            return Err(SceneError::CapabilityError(format!("Capability token not found: {}", token_id)));
        }

        Ok(())
    }

    /// Delete a capability token
    pub async fn delete_token(&self, token_id: &str) -> SceneResult<()> {
        let mut tokens = self.tokens.write().await;
        let mut user_tokens = self.user_tokens.write().await;
        let mut node_tokens = self.node_tokens.write().await;
        let mut avatar_tokens = self.avatar_tokens.write().await;

        if let Some(token) = tokens.remove(token_id) {
            // Remove from user tokens
            if let Some(user_token_list) = user_tokens.get_mut(&token.created_by) {
                user_token_list.retain(|id| id != token_id);
            }

            // Remove from node tokens
            for (node_id, node_token_list) in node_tokens.iter_mut() {
                node_token_list.retain(|id| id != token_id);
            }

            // Remove from avatar tokens
            for (avatar_id, avatar_token_list) in avatar_tokens.iter_mut() {
                avatar_token_list.retain(|id| id != token_id);
            }

            Ok(())
        } else {
            Err(SceneError::CapabilityError(format!("Capability token not found: {}", token_id)))
        }
    }

    /// Assign a token to a user
    pub async fn assign_token_to_user(&self, token_id: &str, user_did: &str) -> SceneResult<()> {
        let tokens = self.tokens.read().await;
        if !tokens.contains_key(token_id) {
            return Err(SceneError::CapabilityError(format!("Capability token not found: {}", token_id)));
        }

        let mut user_tokens = self.user_tokens.write().await;
        user_tokens.entry(user_did.to_string())
            .or_insert_with(Vec::new)
            .push(token_id.to_string());

        Ok(())
    }

    /// Assign a token to a node
    pub async fn assign_token_to_node(&self, token_id: &str, node_id: NodeId) -> SceneResult<()> {
        let tokens = self.tokens.read().await;
        if !tokens.contains_key(token_id) {
            return Err(SceneError::CapabilityError(format!("Capability token not found: {}", token_id)));
        }

        let mut node_tokens = self.node_tokens.write().await;
        node_tokens.entry(node_id)
            .or_insert_with(Vec::new)
            .push(token_id.to_string());

        Ok(())
    }

    /// Assign a token to an avatar
    pub async fn assign_token_to_avatar(&self, token_id: &str, avatar_id: &str) -> SceneResult<()> {
        let tokens = self.tokens.read().await;
        if !tokens.contains_key(token_id) {
            return Err(SceneError::CapabilityError(format!("Capability token not found: {}", token_id)));
        }

        let mut avatar_tokens = self.avatar_tokens.write().await;
        avatar_tokens.entry(avatar_id.to_string())
            .or_insert_with(Vec::new)
            .push(token_id.to_string());

        Ok(())
    }

    /// Get tokens for a user
    pub async fn get_user_tokens(&self, user_did: &str) -> Vec<CapToken> {
        let tokens = self.tokens.read().await;
        let user_tokens = self.user_tokens.read().await;

        user_tokens.get(user_did)
            .map(|token_ids| {
                token_ids.iter()
                    .filter_map(|id| tokens.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get tokens for a node
    pub async fn get_node_tokens(&self, node_id: NodeId) -> Vec<CapToken> {
        let tokens = self.tokens.read().await;
        let node_tokens = self.node_tokens.read().await;

        node_tokens.get(&node_id)
            .map(|token_ids| {
                token_ids.iter()
                    .filter_map(|id| tokens.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get tokens for an avatar
    pub async fn get_avatar_tokens(&self, avatar_id: &str) -> Vec<CapToken> {
        let tokens = self.tokens.read().await;
        let avatar_tokens = self.avatar_tokens.read().await;

        avatar_tokens.get(avatar_id)
            .map(|token_ids| {
                token_ids.iter()
                    .filter_map(|id| tokens.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a user has permission for an action
    pub async fn check_permission(&self, user_did: &str, action: &CapAction, scope: &CapScope) -> SceneResult<bool> {
        let user_tokens = self.get_user_tokens(user_did).await;

        for token in user_tokens {
            if token.is_expired() {
                continue;
            }

            for permission in &token.permissions {
                if permission.action == *action && permission.scope == *scope {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if a user has permission for an action with conditions
    pub async fn check_permission_with_conditions(
        &self,
        user_did: &str,
        action: &CapAction,
        scope: &CapScope,
        context: &HashMap<String, serde_json::Value>,
    ) -> SceneResult<CapCheckResult> {
        let user_tokens = self.get_user_tokens(user_did).await;

        for token in user_tokens {
            if token.is_expired() {
                continue;
            }

            for permission in &token.permissions {
                if permission.action == *action && permission.scope == *scope {
                    // Check conditions
                    let mut met_conditions = Vec::new();
                    for condition in &permission.conditions {
                        if self.evaluate_condition(condition, context).await? {
                            met_conditions.push(condition.clone());
                        }
                    }

                    if met_conditions.len() == permission.conditions.len() {
                        return Ok(CapCheckResult {
                            allowed: true,
                            token_id: Some(token.id.clone()),
                            conditions: met_conditions,
                            reason: None,
                        });
                    }
                }
            }
        }

        Ok(CapCheckResult {
            allowed: false,
            token_id: None,
            conditions: Vec::new(),
            reason: Some("No valid capability token found".to_string()),
        })
    }

    /// Evaluate a condition
    async fn evaluate_condition(&self, condition: &str, context: &HashMap<String, serde_json::Value>) -> SceneResult<bool> {
        // Simple condition evaluation - in a real implementation, this would use a proper policy engine
        match condition {
            "is_owner" => {
                // Check if user is the owner of the resource
                if let Some(owner) = context.get("owner") {
                    if let Some(owner_str) = owner.as_str() {
                        if let Some(user_did) = context.get("user_did") {
                            if let Some(user_did_str) = user_did.as_str() {
                                return Ok(owner_str == user_did_str);
                            }
                        }
                    }
                }
                Ok(false)
            }
            "is_online" => {
                // Check if avatar is online
                if let Some(online) = context.get("online") {
                    if let Some(online_bool) = online.as_bool() {
                        return Ok(online_bool);
                    }
                }
                Ok(false)
            }
            "has_resource" => {
                // Check if user has a specific resource
                if let Some(resource) = context.get("resource") {
                    if let Some(resource_str) = resource.as_str() {
                        if let Some(user_resources) = context.get("user_resources") {
                            if let Some(resources_array) = user_resources.as_array() {
                                return Ok(resources_array.iter().any(|r| r.as_str() == Some(resource_str)));
                            }
                        }
                    }
                }
                Ok(false)
            }
            _ => {
                // Default to false for unknown conditions
                Ok(false)
            }
        }
    }

    /// Get all tokens
    pub async fn get_all_tokens(&self) -> Vec<CapToken> {
        let tokens = self.tokens.read().await;
        tokens.values().cloned().collect()
    }

    /// Get token count
    pub async fn get_token_count(&self) -> usize {
        let tokens = self.tokens.read().await;
        tokens.len()
    }

    /// Issue a capability token for a session
    pub async fn issue_cap_for_session(
        &self,
        session_id: &SessionId,
        scopes: Vec<String>,
        ttl_seconds: u64,
        session_manager: &SessionManager,
    ) -> SceneResult<CapToken> {
        // Validate session
        let session_validation = session_manager.validate_session(session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        let session = session_validation.session.unwrap();
        let expires_at = Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);

        let mut token = CapToken::new(format!("session_cap_{}", session_id.0));
        token.set_creator(session.did.clone());
        token.set_session_id(session_id.clone());
        token.set_expiration(expires_at);

        // Add granted scopes
        for scope in scopes {
            token.add_granted_scope(scope);
        }

        // Add permissions based on scopes
        for scope in &token.granted_scopes {
            match scope.as_str() {
                "scene:spawn" => {
                    let permission = Permission::new(CapAction::Spawn, CapScope::Scene);
                    token.add_permission(permission);
                }
                "scene:move" => {
                    let permission = Permission::new(CapAction::Move, CapScope::Scene);
                    token.add_permission(permission);
                }
                "scene:delete" => {
                    let permission = Permission::new(CapAction::Delete, CapScope::Scene);
                    token.add_permission(permission);
                }
                "node:spawn" => {
                    let permission = Permission::new(CapAction::Spawn, CapScope::Scene);
                    token.add_permission(permission);
                }
                "node:move" => {
                    let permission = Permission::new(CapAction::Move, CapScope::Scene);
                    token.add_permission(permission);
                }
                "node:delete" => {
                    let permission = Permission::new(CapAction::Delete, CapScope::Scene);
                    token.add_permission(permission);
                }
                "policy:attach" => {
                    let permission = Permission::new(CapAction::AttachPolicy, CapScope::Scene);
                    token.add_permission(permission);
                }
                "policy:manage" => {
                    let permission = Permission::new(CapAction::ManagePolicies, CapScope::Scene);
                    token.add_permission(permission);
                }
                "cap:issue" => {
                    let permission = Permission::new(CapAction::IssueCap, CapScope::Scene);
                    token.add_permission(permission);
                }
                "simulate" => {
                    let permission = Permission::new(CapAction::Simulate, CapScope::Scene);
                    token.add_permission(permission);
                }
                _ => {
                    // Unknown scope, skip
                }
            }
        }

        token.validate()?;

        let token_id = token.id.clone();
        let mut tokens = self.tokens.write().await;
        tokens.insert(token_id.clone(), token.clone());

        // Update session tokens index
        let mut session_tokens = self.session_tokens.write().await;
        session_tokens
            .entry(session_id.clone())
            .or_insert_with(Vec::new)
            .push(token_id);

        // Emit cap issued event
        self.emit_cap_event("cap_issued", &token.id, session_id, &session.did).await?;

        Ok(token)
    }

    /// Get tokens for a session
    pub async fn get_session_tokens(&self, session_id: &SessionId) -> Vec<CapToken> {
        let tokens = self.tokens.read().await;
        let session_tokens = self.session_tokens.read().await;

        session_tokens
            .get(session_id)
            .map(|token_ids| {
                token_ids
                    .iter()
                    .filter_map(|id| tokens.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a session has permission for an action
    pub async fn check_session_permission(
        &self,
        session_id: &SessionId,
        action: &CapAction,
        scope: &CapScope,
    ) -> SceneResult<bool> {
        let session_tokens = self.get_session_tokens(session_id).await;

        for token in session_tokens {
            if token.is_expired() {
                continue;
            }

            for permission in &token.permissions {
                if permission.action == *action && permission.scope == *scope {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if a session has a specific granted scope
    pub async fn check_session_scope(&self, session_id: &SessionId, scope: &str) -> SceneResult<bool> {
        let session_tokens = self.get_session_tokens(session_id).await;

        for token in session_tokens {
            if token.is_expired() {
                continue;
            }

            if token.has_granted_scope(scope) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Revoke all tokens for a session
    pub async fn revoke_session_tokens(&self, session_id: &SessionId) -> SceneResult<usize> {
        let mut tokens = self.tokens.write().await;
        let mut session_tokens = self.session_tokens.write().await;

        if let Some(token_ids) = session_tokens.remove(session_id) {
            let mut revoked_count = 0;
            for token_id in token_ids {
                if tokens.remove(&token_id).is_some() {
                    revoked_count += 1;
                }
            }

            // Emit cap expired event
            self.emit_cap_event("cap_expired", "", session_id, "").await?;

            Ok(revoked_count)
        } else {
            Ok(0)
        }
    }

    /// Emit capability event (placeholder implementation)
    async fn emit_cap_event(
        &self,
        event_type: &str,
        token_id: &str,
        session_id: &SessionId,
        did: &str,
    ) -> SceneResult<()> {
        // TODO: Implement proper event emission
        log::info!(
            "Capability event: {} for token {} (Session: {}, DID: {})",
            event_type,
            token_id,
            session_id.0,
            did
        );
        Ok(())
    }

    /// Get capability statistics
    pub async fn get_capability_stats(&self) -> CapabilityStats {
        let tokens = self.tokens.read().await;
        let user_tokens = self.user_tokens.read().await;
        let node_tokens = self.node_tokens.read().await;
        let avatar_tokens = self.avatar_tokens.read().await;
        let session_tokens = self.session_tokens.read().await;

        let total_permissions: usize = tokens.values().map(|t| t.permissions.len()).sum();
        let expired_tokens: usize = tokens.values().filter(|t| t.is_expired()).count();

        CapabilityStats {
            total_tokens: tokens.len(),
            total_permissions,
            expired_tokens,
            user_assignments: user_tokens.len(),
            node_assignments: node_tokens.len(),
            avatar_assignments: avatar_tokens.len(),
            session_assignments: session_tokens.len(),
        }
    }
}

/// Capability token update operations
#[derive(Debug, Clone)]
pub enum CapTokenUpdate {
    AddPermission(Permission),
    SetScope(CapScope),
    SetExpiration(DateTime<Utc>),
    SetDescription(String),
    IncrementVersion,
}

/// Capability check result
#[derive(Debug, Clone)]
pub struct CapCheckResult {
    pub allowed: bool,
    pub token_id: Option<String>,
    pub conditions: Vec<String>,
    pub reason: Option<String>,
}

/// Capability statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStats {
    pub total_tokens: usize,
    pub total_permissions: usize,
    pub expired_tokens: usize,
    pub user_assignments: usize,
    pub node_assignments: usize,
    pub avatar_assignments: usize,
    pub session_assignments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cap_token_creation() {
        let token = CapToken::new("test_token".to_string());
        assert_eq!(token.name, "test_token");
        assert_eq!(token.scope, CapScope::Scene);
        assert_eq!(token.version, 1);
        assert!(token.permissions.is_empty());
        assert!(!token.is_expired());
    }

    #[test]
    fn test_cap_token_validation() {
        let mut token = CapToken::new("test_token".to_string());
        
        // Valid token
        assert!(token.validate().is_ok());
        
        // Invalid name
        token.name = "".to_string();
        assert!(token.validate().is_err());
        
        // Invalid creator
        token.name = "test_token".to_string();
        token.created_by = "".to_string();
        assert!(token.validate().is_err());
        
        // Invalid version
        token.created_by = "test_user".to_string();
        token.version = 0;
        assert!(token.validate().is_err());
    }

    #[test]
    fn test_cap_token_expiration() {
        let mut token = CapToken::new("test_token".to_string());
        
        // Token without expiration
        assert!(!token.is_expired());
        
        // Token with future expiration
        token.set_expiration(Utc::now() + chrono::Duration::hours(1));
        assert!(!token.is_expired());
        
        // Token with past expiration
        token.set_expiration(Utc::now() - chrono::Duration::hours(1));
        assert!(token.is_expired());
    }

    #[test]
    fn test_cap_token_permissions() {
        let mut token = CapToken::new("test_token".to_string());
        
        let permission = Permission::new(CapAction::Spawn, CapScope::Scene);
        token.add_permission(permission.clone());
        
        assert!(token.has_permission(&permission));
        assert!(token.has_action_permission(&CapAction::Spawn));
        assert!(!token.has_action_permission(&CapAction::Delete));
    }

    #[test]
    fn test_cap_token_serialization() {
        let token = CapToken::new("test_token".to_string());
        
        let cbor_data = token.to_cbor().unwrap();
        let deserialized = CapToken::from_cbor(&cbor_data).unwrap();
        
        assert_eq!(token.name, deserialized.name);
        assert_eq!(token.scope, deserialized.scope);
        assert_eq!(token.version, deserialized.version);
    }

    #[tokio::test]
    async fn test_cap_manager_operations() {
        let manager = CapManager::new();
        
        // Create token
        let token_id = manager.create_token("test_token".to_string(), "test_user".to_string()).await.unwrap();
        assert_eq!(manager.get_token_count().await, 1);
        
        // Get token
        let token = manager.get_token(&token_id).await.unwrap();
        assert_eq!(token.name, "test_token");
        
        // Update token
        let permission = Permission::new(CapAction::Spawn, CapScope::Scene);
        manager.update_token(&token_id, CapTokenUpdate::AddPermission(permission)).await.unwrap();
        
        let updated_token = manager.get_token(&token_id).await.unwrap();
        assert_eq!(updated_token.permissions.len(), 1);
        
        // Assign token to user
        manager.assign_token_to_user(&token_id, "did:aeth:test").await.unwrap();
        
        let user_tokens = manager.get_user_tokens("did:aeth:test").await;
        assert_eq!(user_tokens.len(), 1);
        
        // Check permission
        let has_permission = manager.check_permission("did:aeth:test", &CapAction::Spawn, &CapScope::Scene).await.unwrap();
        assert!(has_permission);
        
        // Delete token
        manager.delete_token(&token_id).await.unwrap();
        assert_eq!(manager.get_token_count().await, 0);
    }

    #[tokio::test]
    async fn test_cap_manager_permission_check() {
        let manager = CapManager::new();
        
        // Create token with permission
        let token_id = manager.create_token("test_token".to_string(), "test_user".to_string()).await.unwrap();
        let permission = Permission::new(CapAction::Spawn, CapScope::Scene);
        manager.update_token(&token_id, CapTokenUpdate::AddPermission(permission)).await.unwrap();
        
        // Assign token to user
        manager.assign_token_to_user(&token_id, "did:aeth:test").await.unwrap();
        
        // Check permission
        let has_permission = manager.check_permission("did:aeth:test", &CapAction::Spawn, &CapScope::Scene).await.unwrap();
        assert!(has_permission);
        
        // Check permission for different action
        let has_permission = manager.check_permission("did:aeth:test", &CapAction::Delete, &CapScope::Scene).await.unwrap();
        assert!(!has_permission);
        
        // Check permission for different scope
        let has_permission = manager.check_permission("did:aeth:test", &CapAction::Spawn, &CapScope::Node(NodeId::new())).await.unwrap();
        assert!(!has_permission);
    }

    #[tokio::test]
    async fn test_cap_manager_conditional_permission() {
        let manager = CapManager::new();
        
        // Create token with conditional permission
        let token_id = manager.create_token("test_token".to_string(), "test_user".to_string()).await.unwrap();
        let mut permission = Permission::new(CapAction::Spawn, CapScope::Scene);
        permission.add_condition("is_owner".to_string());
        manager.update_token(&token_id, CapTokenUpdate::AddPermission(permission)).await.unwrap();
        
        // Assign token to user
        manager.assign_token_to_user(&token_id, "did:aeth:test").await.unwrap();
        
        // Check permission without context
        let mut context = HashMap::new();
        let result = manager.check_permission_with_conditions("did:aeth:test", &CapAction::Spawn, &CapScope::Scene, &context).await.unwrap();
        assert!(!result.allowed);
        
        // Check permission with context
        context.insert("owner".to_string(), serde_json::Value::String("did:aeth:test".to_string()));
        context.insert("user_did".to_string(), serde_json::Value::String("did:aeth:test".to_string()));
        let result = manager.check_permission_with_conditions("did:aeth:test", &CapAction::Spawn, &CapScope::Scene, &context).await.unwrap();
        assert!(result.allowed);
        assert!(!result.conditions.is_empty());
    }
}
