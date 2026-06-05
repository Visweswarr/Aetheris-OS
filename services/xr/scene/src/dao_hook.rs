//! DAO Hook Adapter for XR Scene Service
//! 
//! This module provides an adapter to integrate the XR scene service with the
//! DAO kernel for policy decisions and governance.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::{SceneError, SceneResult};
use crate::policy::{DaoHook, PolicyContext, PolicyResult, PolicyAction, PolicyScope};
use crate::session::SessionId;

/// DAO proposal for scene operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneProposal {
    /// Unique proposal ID
    pub id: String,
    /// Proposal title
    pub title: String,
    /// Proposal description
    pub description: String,
    /// Proposed action
    pub action: PolicyAction,
    /// Target scope
    pub scope: PolicyScope,
    /// Proposer DID
    pub proposer_did: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Current status
    pub status: ProposalStatus,
    /// Votes cast
    pub votes: Vec<ProposalVote>,
    /// Required threshold for approval
    pub approval_threshold: f64,
}

/// Proposal status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposal is active and accepting votes
    Active,
    /// Proposal has been approved
    Approved,
    /// Proposal has been rejected
    Rejected,
    /// Proposal has expired
    Expired,
    /// Proposal has been executed
    Executed,
}

/// Vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalVote {
    /// Voter DID
    pub voter_did: String,
    /// Vote choice
    pub choice: VoteChoice,
    /// Vote weight (based on stake/reputation)
    pub weight: f64,
    /// Vote timestamp
    pub timestamp: u64,
    /// Vote justification
    pub justification: Option<String>,
}

/// Vote choice
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteChoice {
    /// Approve the proposal
    Approve,
    /// Reject the proposal
    Reject,
    /// Abstain from voting
    Abstain,
}

/// DAO decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaoDecision {
    /// Whether the decision is approved
    pub approved: bool,
    /// Decision reason
    pub reason: String,
    /// Proposal ID (if applicable)
    pub proposal_id: Option<String>,
    /// Vote summary
    pub vote_summary: Option<VoteSummary>,
    /// Decision timestamp
    pub decided_at: u64,
}

/// Vote summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteSummary {
    /// Total votes cast
    pub total_votes: u32,
    /// Approve votes
    pub approve_votes: u32,
    /// Reject votes
    pub reject_votes: u32,
    /// Abstain votes
    pub abstain_votes: u32,
    /// Total voting power
    pub total_power: f64,
    /// Approval percentage
    pub approval_percentage: f64,
}

/// DAO client interface
#[async_trait::async_trait]
pub trait DaoClient {
    /// Create a new proposal
    async fn create_proposal(&self, proposal: SceneProposal) -> SceneResult<String>;
    
    /// Get a proposal by ID
    async fn get_proposal(&self, proposal_id: &str) -> SceneResult<SceneProposal>;
    
    /// Vote on a proposal
    async fn vote_on_proposal(&self, proposal_id: &str, vote: ProposalVote) -> SceneResult<()>;
    
    /// Execute a proposal
    async fn execute_proposal(&self, proposal_id: &str) -> SceneResult<DaoDecision>;
    
    /// Check if a user can perform an action
    async fn check_permission(&self, user_did: &str, action: &PolicyAction, scope: &PolicyScope) -> SceneResult<DaoDecision>;
    
    /// Get user's voting power
    async fn get_voting_power(&self, user_did: &str) -> SceneResult<f64>;
    
    /// Get active proposals
    async fn get_active_proposals(&self) -> SceneResult<Vec<SceneProposal>>;
}

/// Default DAO hook implementation
pub struct DefaultDaoHook {
    client: Arc<dyn DaoClient + Send + Sync>,
    /// Cache for recent decisions
    decision_cache: Arc<tokio::sync::RwLock<HashMap<String, (DaoDecision, u64)>>>,
    /// Cache TTL in seconds
    cache_ttl: u64,
}

impl DefaultDaoHook {
    /// Create a new default DAO hook
    pub fn new(client: Arc<dyn DaoClient + Send + Sync>, cache_ttl: u64) -> Self {
        Self {
            client,
            decision_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    /// Generate cache key for a policy context
    fn cache_key(&self, context: &PolicyContext) -> String {
        format!(
            "{}:{}:{}:{}",
            context.user_did,
            format!("{:?}", context.action),
            format!("{:?}", context.scope),
            context.session_id.as_ref().map(|s| s.0.as_str()).unwrap_or("none")
        )
    }

    /// Check if cached decision is still valid
    fn is_cache_valid(&self, timestamp: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now - timestamp < self.cache_ttl
    }
}

#[async_trait::async_trait]
impl DaoHook for DefaultDaoHook {
    async fn check_policy(&self, context: &PolicyContext) -> SceneResult<PolicyResult> {
        // Check cache first
        let cache_key = self.cache_key(context);
        {
            let cache = self.decision_cache.read().await;
            if let Some((decision, timestamp)) = cache.get(&cache_key) {
                if self.is_cache_valid(*timestamp) {
                    return Ok(PolicyResult {
                        allowed: decision.approved,
                        reason: Some(decision.reason.clone()),
                        conditions: Vec::new(),
                        rule_id: decision.proposal_id.clone(),
                    });
                }
            }
        }

        // Make DAO decision
        let decision = self.client.check_permission(
            &context.user_did,
            &context.action,
            &context.scope,
        ).await?;

        // Cache the decision
        {
            let mut cache = self.decision_cache.write().await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            cache.insert(cache_key, (decision.clone(), now));
        }

        Ok(PolicyResult {
            allowed: decision.approved,
            reason: Some(decision.reason),
            conditions: Vec::new(),
            rule_id: decision.proposal_id,
        })
    }
}

/// Mock DAO client for testing
pub struct MockDaoClient {
    /// Mock decisions
    decisions: Arc<tokio::sync::RwLock<HashMap<String, DaoDecision>>>,
    /// Mock voting power
    voting_power: Arc<tokio::sync::RwLock<HashMap<String, f64>>>,
}

impl MockDaoClient {
    /// Create a new mock DAO client
    pub fn new() -> Self {
        Self {
            decisions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            voting_power: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Set a mock decision
    pub async fn set_decision(&self, key: String, decision: DaoDecision) {
        let mut decisions = self.decisions.write().await;
        decisions.insert(key, decision);
    }

    /// Set mock voting power
    pub async fn set_voting_power(&self, user_did: String, power: f64) {
        let mut voting_power = self.voting_power.write().await;
        voting_power.insert(user_did, power);
    }

    /// Generate decision key
    fn decision_key(&self, user_did: &str, action: &PolicyAction, scope: &PolicyScope) -> String {
        format!("{}:{}:{}", user_did, format!("{:?}", action), format!("{:?}", scope))
    }
}

#[async_trait::async_trait]
impl DaoClient for MockDaoClient {
    async fn create_proposal(&self, _proposal: SceneProposal) -> SceneResult<String> {
        Ok(uuid::Uuid::new_v4().to_string())
    }

    async fn get_proposal(&self, _proposal_id: &str) -> SceneResult<SceneProposal> {
        Err(SceneError::NotFound("Proposal not found".to_string()))
    }

    async fn vote_on_proposal(&self, _proposal_id: &str, _vote: ProposalVote) -> SceneResult<()> {
        Ok(())
    }

    async fn execute_proposal(&self, _proposal_id: &str) -> SceneResult<DaoDecision> {
        Ok(DaoDecision {
            approved: true,
            reason: "Mock execution".to_string(),
            proposal_id: None,
            vote_summary: None,
            decided_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    async fn check_permission(&self, user_did: &str, action: &PolicyAction, scope: &PolicyScope) -> SceneResult<DaoDecision> {
        let key = self.decision_key(user_did, action, scope);
        let decisions = self.decisions.read().await;
        
        if let Some(decision) = decisions.get(&key) {
            Ok(decision.clone())
        } else {
            // Default decision based on action type
            let approved = match action {
                PolicyAction::Spawn | PolicyAction::Move | PolicyAction::Delete => {
                    // Allow basic scene operations by default
                    true
                }
                PolicyAction::AttachPolicy | PolicyAction::ManagePolicies => {
                    // Require higher privileges for policy management
                    false
                }
                _ => {
                    // Default to allow for other actions
                    true
                }
            };

            Ok(DaoDecision {
                approved,
                reason: if approved {
                    "Default allow".to_string()
                } else {
                    "Default deny for policy management".to_string()
                },
                proposal_id: None,
                vote_summary: None,
                decided_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            })
        }
    }

    async fn get_voting_power(&self, user_did: &str) -> SceneResult<f64> {
        let voting_power = self.voting_power.read().await;
        Ok(voting_power.get(user_did).copied().unwrap_or(1.0))
    }

    async fn get_active_proposals(&self) -> SceneResult<Vec<SceneProposal>> {
        Ok(Vec::new())
    }
}

/// DAO hook factory
pub struct DaoHookFactory;

impl DaoHookFactory {
    /// Create a default DAO hook with mock client
    pub fn create_mock_hook() -> Arc<dyn DaoHook + Send + Sync> {
        let mock_client = Arc::new(MockDaoClient::new());
        Arc::new(DefaultDaoHook::new(mock_client, 300)) // 5 minute cache
    }

    /// Create a DAO hook with custom client
    pub fn create_hook(client: Arc<dyn DaoClient + Send + Sync>, cache_ttl: u64) -> Arc<dyn DaoHook + Send + Sync> {
        Arc::new(DefaultDaoHook::new(client, cache_ttl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::NodeId;

    #[tokio::test]
    async fn test_mock_dao_client() {
        let client = MockDaoClient::new();
        
        // Set voting power
        client.set_voting_power("did:aeth:test".to_string(), 10.0).await;
        
        // Check voting power
        let power = client.get_voting_power("did:aeth:test").await.unwrap();
        assert_eq!(power, 10.0);
        
        // Check default permission
        let decision = client.check_permission(
            "did:aeth:test",
            &PolicyAction::Spawn,
            &PolicyScope::Scene,
        ).await.unwrap();
        
        assert!(decision.approved);
    }

    #[tokio::test]
    async fn test_dao_hook_with_mock_client() {
        let hook = DaoHookFactory::create_mock_hook();
        
        let context = PolicyContext::new(
            "did:aeth:test".to_string(),
            PolicyAction::Spawn,
            PolicyScope::Scene,
        );
        
        let result = hook.check_policy(&context).await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_dao_hook_caching() {
        let hook = DaoHookFactory::create_mock_hook();
        
        let context = PolicyContext::new(
            "did:aeth:test".to_string(),
            PolicyAction::Spawn,
            PolicyScope::Scene,
        );
        
        // First call
        let result1 = hook.check_policy(&context).await.unwrap();
        assert!(result1.allowed);
        
        // Second call should use cache
        let result2 = hook.check_policy(&context).await.unwrap();
        assert!(result2.allowed);
        assert_eq!(result1.reason, result2.reason);
    }

    #[tokio::test]
    async fn test_dao_hook_policy_management_deny() {
        let hook = DaoHookFactory::create_mock_hook();
        
        let context = PolicyContext::new(
            "did:aeth:test".to_string(),
            PolicyAction::AttachPolicy,
            PolicyScope::Scene,
        );
        
        let result = hook.check_policy(&context).await.unwrap();
        assert!(!result.allowed);
        assert!(result.reason.unwrap().contains("deny"));
    }

    #[tokio::test]
    async fn test_proposal_creation() {
        let client = MockDaoClient::new();
        
        let proposal = SceneProposal {
            id: uuid::Uuid::new_v4().to_string(),
            title: "Test Proposal".to_string(),
            description: "Test description".to_string(),
            action: PolicyAction::Spawn,
            scope: PolicyScope::Scene,
            proposer_did: "did:aeth:test".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() + 86400, // 24 hours
            status: ProposalStatus::Active,
            votes: Vec::new(),
            approval_threshold: 0.5,
        };
        
        let proposal_id = client.create_proposal(proposal).await.unwrap();
        assert!(!proposal_id.is_empty());
    }

    #[tokio::test]
    async fn test_vote_creation() {
        let vote = ProposalVote {
            voter_did: "did:aeth:test".to_string(),
            choice: VoteChoice::Approve,
            weight: 10.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            justification: Some("Good proposal".to_string()),
        };
        
        assert_eq!(vote.voter_did, "did:aeth:test");
        assert_eq!(vote.choice, VoteChoice::Approve);
        assert_eq!(vote.weight, 10.0);
    }
}
