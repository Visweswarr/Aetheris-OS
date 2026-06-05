//! DAO kernel service for Aetheris OS
//! 
//! This module provides decentralized autonomous organization (DAO) functionality
//! including proposal creation, voting, and execution with capability-based access control.

pub mod proposal;
pub mod vote;
pub mod execution;
pub mod governance;
pub mod error;
pub mod types;
pub mod database;

pub use proposal::ProposalService;
pub use vote::VoteService;
pub use execution::ExecutionService;
pub use governance::GovernanceService;
pub use error::DAOError;
pub use types::*;

use std::sync::Arc;
use tokio::sync::RwLock;

/// DAO service configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DAOConfig {
    /// Database connection string
    pub database_url: String,
    /// Minimum voting period (seconds)
    pub min_voting_period: u64,
    /// Maximum voting period (seconds)
    pub max_voting_period: u64,
    /// Minimum votes required for execution
    pub min_votes_required: u64,
    /// Majority threshold (percentage)
    pub majority_threshold: u8,
    /// Quorum threshold (percentage)
    pub quorum_threshold: u8,
    /// Chain service configuration
    pub chain_config: chain::ChainServiceConfig,
}

impl Default for DAOConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite:dao.db".to_string(),
            min_voting_period: 86400, // 1 day
            max_voting_period: 604800, // 7 days
            min_votes_required: 3,
            majority_threshold: 51,
            quorum_threshold: 25,
            chain_config: chain::ChainServiceConfig::default(),
        }
    }
}

/// Main DAO service
pub struct DAOService {
    config: DAOConfig,
    proposal_service: Arc<RwLock<ProposalService>>,
    vote_service: Arc<RwLock<VoteService>>,
    execution_service: Arc<RwLock<ExecutionService>>,
    governance_service: Arc<RwLock<GovernanceService>>,
}

impl DAOService {
    /// Create a new DAO service
    pub async fn new(config: DAOConfig) -> Result<Self, DAOError> {
        // Initialize database
        let database = Arc::new(database::Database::new(&config.database_url).await?);
        
        // Initialize services
        let proposal_service = Arc::new(RwLock::new(ProposalService::new(database.clone()).await?));
        let vote_service = Arc::new(RwLock::new(VoteService::new(database.clone()).await?));
        let execution_service = Arc::new(RwLock::new(ExecutionService::new(database.clone()).await?));
        let governance_service = Arc::new(RwLock::new(GovernanceService::new(database.clone()).await?));
        
        Ok(Self {
            config,
            proposal_service,
            vote_service,
            execution_service,
            governance_service,
        })
    }

    /// Get the proposal service
    pub fn proposal_service(&self) -> Arc<RwLock<ProposalService>> {
        self.proposal_service.clone()
    }

    /// Get the vote service
    pub fn vote_service(&self) -> Arc<RwLock<VoteService>> {
        self.vote_service.clone()
    }

    /// Get the execution service
    pub fn execution_service(&self) -> Arc<RwLock<ExecutionService>> {
        self.execution_service.clone()
    }

    /// Get the governance service
    pub fn governance_service(&self) -> Arc<RwLock<GovernanceService>> {
        self.governance_service.clone()
    }

    /// Get the configuration
    pub fn config(&self) -> &DAOConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dao_config_default() {
        let config = DAOConfig::default();
        assert_eq!(config.min_voting_period, 86400);
        assert_eq!(config.max_voting_period, 604800);
        assert_eq!(config.min_votes_required, 3);
        assert_eq!(config.majority_threshold, 51);
        assert_eq!(config.quorum_threshold, 25);
    }

    #[tokio::test]
    async fn test_dao_service_creation() {
        let config = DAOConfig::default();
        let service = DAOService::new(config).await;
        assert!(service.is_ok());
    }
}
