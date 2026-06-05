//! Policy management for XR scenes

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{SceneError, SceneResult};
use crate::scene::{SceneNode, NodeId};
use crate::avatar::Avatar;
use crate::cap::CapToken;
use crate::session::{SessionId, SessionManager};

/// Policy scope for applying rules
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyScope {
    /// Apply to entire scene
    Scene,
    /// Apply to specific node
    Node(NodeId),
    /// Apply to specific avatar
    Avatar(String), // DID
    /// Apply to specific user
    User(String), // DID
}

/// Policy action types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PolicyAction {
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
    /// Attach policy
    AttachPolicy,
    /// Simulate operations
    Simulate,
}

/// Policy effect
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEffect {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Allow with conditions
    AllowWithConditions(Vec<String>),
}

/// Policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub scope: PolicyScope,
    pub action: PolicyAction,
    pub effect: PolicyEffect,
    pub conditions: Vec<String>, // Rego conditions
    pub priority: u32,
    pub created_at: u64,
    pub created_by: String,
    pub version: u32,
}

impl PolicyRule {
    /// Create a new policy rule
    pub fn new(
        name: String,
        scope: PolicyScope,
        action: PolicyAction,
        effect: PolicyEffect,
        created_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            scope,
            action,
            effect,
            conditions: Vec::new(),
            priority: 100,
            created_at: chrono::Utc::now().timestamp() as u64,
            created_by,
            version: 1,
        }
    }

    /// Add a condition to the rule
    pub fn add_condition(&mut self, condition: String) {
        self.conditions.push(condition);
    }

    /// Set the priority of the rule
    pub fn set_priority(&mut self, priority: u32) {
        self.priority = priority;
    }

    /// Validate the rule
    pub fn validate(&self) -> SceneResult<()> {
        if self.name.trim().is_empty() {
            return Err(SceneError::InvalidInput("Policy rule name cannot be empty".to_string()));
        }

        if self.created_by.trim().is_empty() {
            return Err(SceneError::InvalidInput("Policy rule creator cannot be empty".to_string()));
        }

        if self.version == 0 {
            return Err(SceneError::InvalidInput("Policy rule version must be greater than 0".to_string()));
        }

        Ok(())
    }
}

/// Policy bundle containing multiple rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBundle {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<PolicyRule>,
    pub created_at: u64,
    pub created_by: String,
    pub version: u32,
}

impl PolicyBundle {
    /// Create a new policy bundle
    pub fn new(name: String, created_by: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: None,
            rules: Vec::new(),
            created_at: chrono::Utc::now().timestamp() as u64,
            created_by,
            version: 1,
        }
    }

    /// Add a rule to the bundle
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    /// Get rules by scope
    pub fn get_rules_by_scope(&self, scope: &PolicyScope) -> Vec<&PolicyRule> {
        self.rules.iter()
            .filter(|rule| &rule.scope == scope)
            .collect()
    }

    /// Get rules by action
    pub fn get_rules_by_action(&self, action: &PolicyAction) -> Vec<&PolicyRule> {
        self.rules.iter()
            .filter(|rule| &rule.action == action)
            .collect()
    }

    /// Validate the bundle
    pub fn validate(&self) -> SceneResult<()> {
        if self.name.trim().is_empty() {
            return Err(SceneError::InvalidInput("Policy bundle name cannot be empty".to_string()));
        }

        if self.created_by.trim().is_empty() {
            return Err(SceneError::InvalidInput("Policy bundle creator cannot be empty".to_string()));
        }

        if self.version == 0 {
            return Err(SceneError::InvalidInput("Policy bundle version must be greater than 0".to_string()));
        }

        // Validate all rules
        for rule in &self.rules {
            rule.validate()?;
        }

        // Check for duplicate rule IDs
        let mut rule_ids = std::collections::HashSet::new();
        for rule in &self.rules {
            if !rule_ids.insert(&rule.id) {
                return Err(SceneError::InvalidInput(format!("Duplicate rule ID: {}", rule.id)));
            }
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
}

/// Policy attachment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachPolicyRequest {
    /// Session ID making the request
    pub session_id: SessionId,
    /// Scope to attach policy to
    pub scope: PolicyScope,
    /// Rego policy bundle in CBOR format
    pub rego_bundle_cbor: Vec<u8>,
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: Option<String>,
}

/// Policy attachment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachPolicyResponse {
    /// Generated policy ID
    pub policy_id: String,
    /// Whether attachment was successful
    pub success: bool,
    /// Attachment timestamp
    pub attached_at: u64,
}

/// Policy simulation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateRequest {
    /// Session ID making the request
    pub session_id: SessionId,
    /// Operations to simulate
    pub operations: Vec<SimulationOperation>,
}

/// Simulation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationOperation {
    /// Operation type
    pub operation: String,
    /// Operation parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Policy simulation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulateResponse {
    /// Results for each operation
    pub results: Vec<OperationResult>,
    /// Simulation timestamp
    pub simulated_at: u64,
}

/// Operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    /// Operation that was simulated
    pub operation: String,
    /// Whether operation would be allowed
    pub allowed: bool,
    /// Reason for allow/deny
    pub reason: Option<String>,
    /// Policy reference that applied
    pub policy_ref: Option<String>,
}

/// Policy evaluation context
#[derive(Debug, Clone)]
pub struct PolicyContext {
    pub user_did: String,
    pub avatar_id: Option<String>,
    pub node_id: Option<NodeId>,
    pub action: PolicyAction,
    pub scope: PolicyScope,
    pub metadata: HashMap<String, serde_json::Value>,
    pub cap_token: Option<CapToken>,
    pub session_id: Option<SessionId>,
}

impl PolicyContext {
    /// Create a new policy context
    pub fn new(user_did: String, action: PolicyAction, scope: PolicyScope) -> Self {
        Self {
            user_did,
            avatar_id: None,
            node_id: None,
            action,
            scope,
            metadata: HashMap::new(),
            cap_token: None,
            session_id: None,
        }
    }

    /// Set the avatar ID
    pub fn set_avatar_id(&mut self, avatar_id: String) {
        self.avatar_id = Some(avatar_id);
    }

    /// Set the node ID
    pub fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = Some(node_id);
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Set the capability token
    pub fn set_cap_token(&mut self, cap_token: CapToken) {
        self.cap_token = Some(cap_token);
    }

    /// Set the session ID
    pub fn set_session_id(&mut self, session_id: SessionId) {
        self.session_id = Some(session_id);
    }
}

/// Policy evaluation result
#[derive(Debug, Clone)]
pub struct PolicyResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub conditions: Vec<String>,
    pub rule_id: Option<String>,
}

impl PolicyResult {
    /// Create an allowed result
    pub fn allow() -> Self {
        Self {
            allowed: true,
            reason: None,
            conditions: Vec::new(),
            rule_id: None,
        }
    }

    /// Create a denied result
    pub fn deny(reason: String) -> Self {
        Self {
            allowed: false,
            reason: Some(reason),
            conditions: Vec::new(),
            rule_id: None,
        }
    }

    /// Create a conditional result
    pub fn conditional(conditions: Vec<String>, rule_id: String) -> Self {
        Self {
            allowed: true,
            reason: None,
            conditions,
            rule_id: Some(rule_id),
        }
    }
}

/// Policy manager for handling scene policies
pub struct PolicyManager {
    bundles: Arc<RwLock<HashMap<String, PolicyBundle>>>,
    active_bundles: Arc<RwLock<Vec<String>>>,
    attached_policies: Arc<RwLock<HashMap<String, PolicyBundle>>>, // policy_id -> bundle
    dao_hook: Option<Arc<dyn DaoHook + Send + Sync>>,
}

impl PolicyManager {
    /// Create a new policy manager
    pub fn new() -> Self {
        Self {
            bundles: Arc::new(RwLock::new(HashMap::new())),
            active_bundles: Arc::new(RwLock::new(Vec::new())),
            attached_policies: Arc::new(RwLock::new(HashMap::new())),
            dao_hook: None,
        }
    }

    /// Create a new policy manager with DAO hook
    pub fn with_dao_hook(dao_hook: Arc<dyn DaoHook + Send + Sync>) -> Self {
        Self {
            bundles: Arc::new(RwLock::new(HashMap::new())),
            active_bundles: Arc::new(RwLock::new(Vec::new())),
            attached_policies: Arc::new(RwLock::new(HashMap::new())),
            dao_hook: Some(dao_hook),
        }
    }

    /// Add a policy bundle
    pub async fn add_bundle(&self, bundle: PolicyBundle) -> SceneResult<()> {
        bundle.validate()?;

        let mut bundles = self.bundles.write().await;
        bundles.insert(bundle.id.clone(), bundle);
        Ok(())
    }

    /// Get a policy bundle
    pub async fn get_bundle(&self, bundle_id: &str) -> SceneResult<PolicyBundle> {
        let bundles = self.bundles.read().await;
        bundles.get(bundle_id)
            .cloned()
            .ok_or_else(|| SceneError::PolicyError(format!("Policy bundle not found: {}", bundle_id)))
    }

    /// Remove a policy bundle
    pub async fn remove_bundle(&self, bundle_id: &str) -> SceneResult<()> {
        let mut bundles = self.bundles.write().await;
        let mut active_bundles = self.active_bundles.write().await;

        if bundles.remove(bundle_id).is_some() {
            active_bundles.retain(|id| id != bundle_id);
            Ok(())
        } else {
            Err(SceneError::PolicyError(format!("Policy bundle not found: {}", bundle_id)))
        }
    }

    /// Activate a policy bundle
    pub async fn activate_bundle(&self, bundle_id: &str) -> SceneResult<()> {
        let bundles = self.bundles.read().await;
        if !bundles.contains_key(bundle_id) {
            return Err(SceneError::PolicyError(format!("Policy bundle not found: {}", bundle_id)));
        }

        let mut active_bundles = self.active_bundles.write().await;
        if !active_bundles.contains(&bundle_id.to_string()) {
            active_bundles.push(bundle_id.to_string());
        }

        Ok(())
    }

    /// Deactivate a policy bundle
    pub async fn deactivate_bundle(&self, bundle_id: &str) -> SceneResult<()> {
        let mut active_bundles = self.active_bundles.write().await;
        active_bundles.retain(|id| id != bundle_id);
        Ok(())
    }

    /// Get all active bundles
    pub async fn get_active_bundles(&self) -> Vec<PolicyBundle> {
        let bundles = self.bundles.read().await;
        let active_bundles = self.active_bundles.read().await;

        active_bundles.iter()
            .filter_map(|id| bundles.get(id).cloned())
            .collect()
    }

    /// Evaluate a policy context
    pub async fn evaluate(&self, context: PolicyContext) -> SceneResult<PolicyResult> {
        let active_bundles = self.get_active_bundles().await;

        // Collect all applicable rules
        let mut applicable_rules = Vec::new();
        for bundle in active_bundles {
            for rule in &bundle.rules {
                if self.is_rule_applicable(rule, &context) {
                    applicable_rules.push(rule);
                }
            }
        }

        // Sort by priority (higher priority first)
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Evaluate rules in order
        for rule in applicable_rules {
            let result = self.evaluate_rule(rule, &context).await?;
            if !result.allowed {
                return Ok(result);
            }
            if !result.conditions.is_empty() {
                return Ok(result);
            }
        }

        // Default deny if no rules match
        Ok(PolicyResult::deny("No applicable policy rules found".to_string()))
    }

    /// Check if a rule is applicable to the context
    fn is_rule_applicable(&self, rule: &PolicyRule, context: &PolicyContext) -> bool {
        // Check scope match
        match &rule.scope {
            PolicyScope::Scene => true,
            PolicyScope::Node(node_id) => {
                context.node_id.map_or(false, |id| id == *node_id)
            }
            PolicyScope::Avatar(did) => {
                context.avatar_id.as_ref().map_or(false, |id| id == did)
            }
            PolicyScope::User(did) => {
                context.user_did == *did
            }
        } && rule.action == context.action
    }

    /// Evaluate a single rule
    async fn evaluate_rule(&self, rule: &PolicyRule, context: &PolicyContext) -> SceneResult<PolicyResult> {
        match &rule.effect {
            PolicyEffect::Allow => Ok(PolicyResult::allow()),
            PolicyEffect::Deny => Ok(PolicyResult::deny(format!("Denied by rule: {}", rule.name))),
            PolicyEffect::AllowWithConditions(conditions) => {
                // Check if conditions are met
                let mut met_conditions = Vec::new();
                for condition in conditions {
                    if self.evaluate_condition(condition, context).await? {
                        met_conditions.push(condition.clone());
                    }
                }

                if met_conditions.len() == conditions.len() {
                    Ok(PolicyResult::conditional(met_conditions, rule.id.clone()))
                } else {
                    Ok(PolicyResult::deny(format!("Conditions not met for rule: {}", rule.name)))
                }
            }
        }
    }

    /// Evaluate a condition
    async fn evaluate_condition(&self, condition: &str, context: &PolicyContext) -> SceneResult<bool> {
        // Simple condition evaluation - in a real implementation, this would use a proper policy engine
        match condition {
            "has_cap_token" => Ok(context.cap_token.is_some()),
            "is_owner" => {
                // Check if user is the owner of the resource
                if let Some(node_id) = context.node_id {
                    // This would check ownership in the actual implementation
                    Ok(true) // Placeholder
                } else {
                    Ok(false)
                }
            }
            "is_online" => {
                // Check if avatar is online
                if let Some(avatar_id) = &context.avatar_id {
                    // This would check avatar status in the actual implementation
                    Ok(true) // Placeholder
                } else {
                    Ok(false)
                }
            }
            _ => {
                // Default to false for unknown conditions
                Ok(false)
            }
        }
    }

    /// Attach a policy to a scope
    pub async fn attach_policy(
        &self,
        request: AttachPolicyRequest,
        session_manager: &SessionManager,
    ) -> SceneResult<AttachPolicyResponse> {
        // Validate session
        let session_validation = session_manager.validate_session(&request.session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        let session = session_validation.session.unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| SceneError::InternalError("System time error".to_string()))?
            .as_secs();

        // Parse Rego bundle from CBOR
        let bundle = PolicyBundle::from_cbor(&request.rego_bundle_cbor)?;
        
        // Validate bundle
        bundle.validate()?;

        let policy_id = Uuid::new_v4().to_string();
        
        // Store attached policy
        {
            let mut attached_policies = self.attached_policies.write().await;
            attached_policies.insert(policy_id.clone(), bundle);
        }

        // Emit policy attached event
        self.emit_policy_event("policy_attached", &policy_id, &request.session_id, &session.did).await?;

        Ok(AttachPolicyResponse {
            policy_id,
            success: true,
            attached_at: now,
        })
    }

    /// Simulate policy decisions for operations
    pub async fn simulate_policy_decisions(
        &self,
        request: SimulateRequest,
        session_manager: &SessionManager,
    ) -> SceneResult<SimulateResponse> {
        // Validate session
        let session_validation = session_manager.validate_session(&request.session_id)?;
        if !session_validation.is_valid {
            return Err(SceneError::InvalidState("Session is not valid".to_string()));
        }

        let session = session_validation.session.unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| SceneError::InternalError("System time error".to_string()))?
            .as_secs();

        let mut results = Vec::new();

        for operation in request.operations {
            let result = self.simulate_operation(&operation, &session.did, &request.session_id).await?;
            results.push(result);
        }

        Ok(SimulateResponse {
            results,
            simulated_at: now,
        })
    }

    /// Simulate a single operation
    async fn simulate_operation(
        &self,
        operation: &SimulationOperation,
        user_did: &str,
        session_id: &SessionId,
    ) -> SceneResult<OperationResult> {
        // Convert operation to policy action and scope
        let (action, scope) = self.parse_operation(operation)?;

        // Create policy context
        let mut context = PolicyContext::new(user_did.to_string(), action, scope);
        context.set_session_id(session_id.clone());

        // Evaluate policy
        let policy_result = self.evaluate(context).await?;

        Ok(OperationResult {
            operation: operation.operation.clone(),
            allowed: policy_result.allowed,
            reason: policy_result.reason,
            policy_ref: policy_result.rule_id,
        })
    }

    /// Parse operation string to policy action and scope
    fn parse_operation(&self, operation: &SimulationOperation) -> SceneResult<(PolicyAction, PolicyScope)> {
        match operation.operation.as_str() {
            "spawn_node" => Ok((PolicyAction::Spawn, PolicyScope::Scene)),
            "move_node" => {
                if let Some(node_id_str) = operation.parameters.get("node_id") {
                    if let Some(node_id) = node_id_str.as_str() {
                        // Parse node ID (simplified)
                        let node_id = NodeId::new(); // In real implementation, parse from string
                        Ok((PolicyAction::Move, PolicyScope::Node(node_id)))
                    } else {
                        Err(SceneError::InvalidInput("Invalid node_id parameter".to_string()))
                    }
                } else {
                    Err(SceneError::InvalidInput("Missing node_id parameter".to_string()))
                }
            }
            "delete_node" => {
                if let Some(node_id_str) = operation.parameters.get("node_id") {
                    if let Some(node_id) = node_id_str.as_str() {
                        // Parse node ID (simplified)
                        let node_id = NodeId::new(); // In real implementation, parse from string
                        Ok((PolicyAction::Delete, PolicyScope::Node(node_id)))
                    } else {
                        Err(SceneError::InvalidInput("Invalid node_id parameter".to_string()))
                    }
                } else {
                    Err(SceneError::InvalidInput("Missing node_id parameter".to_string()))
                }
            }
            "attach_policy" => Ok((PolicyAction::AttachPolicy, PolicyScope::Scene)),
            _ => Err(SceneError::InvalidInput(format!("Unknown operation: {}", operation.operation)))
        }
    }

    /// Check with DAO for policy decisions
    pub async fn check_dao_policy(&self, context: &PolicyContext) -> SceneResult<PolicyResult> {
        if let Some(dao_hook) = &self.dao_hook {
            dao_hook.check_policy(context).await
        } else {
            Ok(PolicyResult::deny("DAO hook not available".to_string()))
        }
    }

    /// Emit policy event (placeholder implementation)
    async fn emit_policy_event(
        &self,
        event_type: &str,
        policy_id: &str,
        session_id: &SessionId,
        did: &str,
    ) -> SceneResult<()> {
        // TODO: Implement proper event emission
        log::info!(
            "Policy event: {} for policy {} (Session: {}, DID: {})",
            event_type,
            policy_id,
            session_id.0,
            did
        );
        Ok(())
    }

    /// Get policy statistics
    pub async fn get_policy_stats(&self) -> PolicyStats {
        let bundles = self.bundles.read().await;
        let active_bundles = self.active_bundles.read().await;
        let attached_policies = self.attached_policies.read().await;

        let total_rules: usize = bundles.values().map(|b| b.rules.len()).sum();
        let active_rules: usize = active_bundles.iter()
            .filter_map(|id| bundles.get(id))
            .map(|b| b.rules.len())
            .sum();

        PolicyStats {
            total_bundles: bundles.len(),
            active_bundles: active_bundles.len(),
            total_rules,
            active_rules,
            attached_policies: attached_policies.len(),
        }
    }
}

/// DAO hook trait for policy decisions
#[async_trait::async_trait]
pub trait DaoHook {
    /// Check policy with DAO
    async fn check_policy(&self, context: &PolicyContext) -> SceneResult<PolicyResult>;
}

/// Policy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStats {
    pub total_bundles: usize,
    pub active_bundles: usize,
    pub total_rules: usize,
    pub active_rules: usize,
    pub attached_policies: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_rule_creation() {
        let rule = PolicyRule::new(
            "test_rule".to_string(),
            PolicyScope::Scene,
            PolicyAction::Spawn,
            PolicyEffect::Allow,
            "test_user".to_string(),
        );

        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.scope, PolicyScope::Scene);
        assert_eq!(rule.action, PolicyAction::Spawn);
        assert_eq!(rule.effect, PolicyEffect::Allow);
        assert_eq!(rule.priority, 100);
        assert_eq!(rule.version, 1);
    }

    #[test]
    fn test_policy_rule_validation() {
        let mut rule = PolicyRule::new(
            "test_rule".to_string(),
            PolicyScope::Scene,
            PolicyAction::Spawn,
            PolicyEffect::Allow,
            "test_user".to_string(),
        );

        // Valid rule
        assert!(rule.validate().is_ok());

        // Invalid name
        rule.name = "".to_string();
        assert!(rule.validate().is_err());

        // Invalid creator
        rule.name = "test_rule".to_string();
        rule.created_by = "".to_string();
        assert!(rule.validate().is_err());

        // Invalid version
        rule.created_by = "test_user".to_string();
        rule.version = 0;
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_policy_bundle_creation() {
        let bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
        assert_eq!(bundle.name, "test_bundle");
        assert_eq!(bundle.created_by, "test_user");
        assert_eq!(bundle.version, 1);
        assert!(bundle.rules.is_empty());
    }

    #[test]
    fn test_policy_bundle_validation() {
        let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
        
        // Valid bundle
        assert!(bundle.validate().is_ok());

        // Invalid name
        bundle.name = "".to_string();
        assert!(bundle.validate().is_err());

        // Invalid creator
        bundle.name = "test_bundle".to_string();
        bundle.created_by = "".to_string();
        assert!(bundle.validate().is_err());

        // Invalid version
        bundle.created_by = "test_user".to_string();
        bundle.version = 0;
        assert!(bundle.validate().is_err());
    }

    #[test]
    fn test_policy_bundle_serialization() {
        let bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
        
        let cbor_data = bundle.to_cbor().unwrap();
        let deserialized = PolicyBundle::from_cbor(&cbor_data).unwrap();
        
        assert_eq!(bundle.name, deserialized.name);
        assert_eq!(bundle.created_by, deserialized.created_by);
        assert_eq!(bundle.version, deserialized.version);
    }

    #[tokio::test]
    async fn test_policy_manager_operations() {
        let manager = PolicyManager::new();
        
        // Create and add bundle
        let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
        let rule = PolicyRule::new(
            "test_rule".to_string(),
            PolicyScope::Scene,
            PolicyAction::Spawn,
            PolicyEffect::Allow,
            "test_user".to_string(),
        );
        bundle.add_rule(rule);
        
        manager.add_bundle(bundle.clone()).await.unwrap();
        
        // Get bundle
        let retrieved_bundle = manager.get_bundle(&bundle.id).await.unwrap();
        assert_eq!(retrieved_bundle.name, bundle.name);
        
        // Activate bundle
        manager.activate_bundle(&bundle.id).await.unwrap();
        
        // Get active bundles
        let active_bundles = manager.get_active_bundles().await;
        assert_eq!(active_bundles.len(), 1);
        
        // Deactivate bundle
        manager.deactivate_bundle(&bundle.id).await.unwrap();
        
        // Get active bundles
        let active_bundles = manager.get_active_bundles().await;
        assert_eq!(active_bundles.len(), 0);
        
        // Remove bundle
        manager.remove_bundle(&bundle.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_policy_evaluation() {
        let manager = PolicyManager::new();
        
        // Create and add bundle with allow rule
        let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
        let rule = PolicyRule::new(
            "allow_spawn".to_string(),
            PolicyScope::Scene,
            PolicyAction::Spawn,
            PolicyEffect::Allow,
            "test_user".to_string(),
        );
        bundle.add_rule(rule);
        
        manager.add_bundle(bundle.clone()).await.unwrap();
        manager.activate_bundle(&bundle.id).await.unwrap();
        
        // Create context
        let context = PolicyContext::new(
            "did:aeth:test".to_string(),
            PolicyAction::Spawn,
            PolicyScope::Scene,
        );
        
        // Evaluate policy
        let result = manager.evaluate(context).await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_policy_evaluation_deny() {
        let manager = PolicyManager::new();
        
        // Create and add bundle with deny rule
        let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
        let rule = PolicyRule::new(
            "deny_spawn".to_string(),
            PolicyScope::Scene,
            PolicyAction::Spawn,
            PolicyEffect::Deny,
            "test_user".to_string(),
        );
        bundle.add_rule(rule);
        
        manager.add_bundle(bundle.clone()).await.unwrap();
        manager.activate_bundle(&bundle.id).await.unwrap();
        
        // Create context
        let context = PolicyContext::new(
            "did:aeth:test".to_string(),
            PolicyAction::Spawn,
            PolicyScope::Scene,
        );
        
        // Evaluate policy
        let result = manager.evaluate(context).await.unwrap();
        assert!(!result.allowed);
        assert!(result.reason.is_some());
    }

    #[tokio::test]
    async fn test_policy_evaluation_conditional() {
        let manager = PolicyManager::new();
        
        // Create and add bundle with conditional rule
        let mut bundle = PolicyBundle::new("test_bundle".to_string(), "test_user".to_string());
        let mut rule = PolicyRule::new(
            "conditional_spawn".to_string(),
            PolicyScope::Scene,
            PolicyAction::Spawn,
            PolicyEffect::AllowWithConditions(vec!["has_cap_token".to_string()]),
            "test_user".to_string(),
        );
        bundle.add_rule(rule);
        
        manager.add_bundle(bundle.clone()).await.unwrap();
        manager.activate_bundle(&bundle.id).await.unwrap();
        
        // Create context without cap token
        let context = PolicyContext::new(
            "did:aeth:test".to_string(),
            PolicyAction::Spawn,
            PolicyScope::Scene,
        );
        
        // Evaluate policy
        let result = manager.evaluate(context).await.unwrap();
        assert!(!result.allowed);
        
        // Create context with cap token
        let mut context = PolicyContext::new(
            "did:aeth:test".to_string(),
            PolicyAction::Spawn,
            PolicyScope::Scene,
        );
        context.set_cap_token(CapToken::new("test_token".to_string()));
        
        // Evaluate policy
        let result = manager.evaluate(context).await.unwrap();
        assert!(result.allowed);
        assert!(!result.conditions.is_empty());
    }
}
