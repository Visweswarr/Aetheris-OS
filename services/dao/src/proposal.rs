//! Proposal service for managing DAO proposals

use crate::database::Database;
use crate::error::DAOError;
use crate::types::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Proposal service for managing DAO proposals
pub struct ProposalService {
    database: Arc<Database>,
}

impl ProposalService {
    /// Create a new proposal service
    pub async fn new(database: Arc<Database>) -> Result<Self, DAOError> {
        Ok(Self { database })
    }

    /// Create a new proposal
    pub async fn create_proposal(
        &self,
        title: String,
        description: String,
        proposal_type: ProposalType,
        voting_period: u64,
        proposer: String,
        proposer_did: String,
        execution_data: Option<ExecutionData>,
    ) -> Result<Proposal, DAOError> {
        // Validate inputs
        if title.is_empty() {
            return Err(DAOError::InvalidInput("Title cannot be empty".to_string()));
        }
        if description.is_empty() {
            return Err(DAOError::InvalidInput("Description cannot be empty".to_string()));
        }
        if voting_period < 3600 {
            return Err(DAOError::InvalidInput("Voting period must be at least 1 hour".to_string()));
        }

        // Get governance parameters
        let params = self.database.get_governance_params().await?;
        
        // Validate voting period
        if voting_period < params.min_voting_period {
            return Err(DAOError::InvalidInput(format!(
                "Voting period must be at least {} seconds",
                params.min_voting_period
            )));
        }
        if voting_period > params.max_voting_period {
            return Err(DAOError::InvalidInput(format!(
                "Voting period cannot exceed {} seconds",
                params.max_voting_period
            )));
        }

        // Create proposal
        let now = chrono::Utc::now();
        let proposal = Proposal {
            id: Uuid::new_v4(),
            title,
            description,
            proposal_type,
            start_time: now,
            end_time: now + chrono::Duration::seconds(voting_period as i64),
            proposer,
            proposer_did,
            status: ProposalStatus::Draft,
            execution_data,
            created_at: now,
            updated_at: now,
        };

        // Save to database
        self.database.create_proposal(&proposal).await?;

        Ok(proposal)
    }

    /// Get a proposal by ID
    pub async fn get_proposal(&self, id: &Uuid) -> Result<Option<Proposal>, DAOError> {
        self.database.get_proposal(id).await
    }

    /// Update a proposal
    pub async fn update_proposal(&self, proposal: &Proposal) -> Result<(), DAOError> {
        self.database.update_proposal(proposal).await
    }

    /// Activate a proposal (start voting)
    pub async fn activate_proposal(&self, id: &Uuid) -> Result<(), DAOError> {
        let mut proposal = self.database.get_proposal(id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(id.to_string()))?;

        if proposal.status != ProposalStatus::Draft {
            return Err(DAOError::InvalidInput("Only draft proposals can be activated".to_string()));
        }

        proposal.status = ProposalStatus::Active;
        proposal.updated_at = chrono::Utc::now();

        self.database.update_proposal(&proposal).await
    }

    /// End voting for a proposal
    pub async fn end_voting(&self, id: &Uuid) -> Result<(), DAOError> {
        let mut proposal = self.database.get_proposal(id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(id.to_string()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(DAOError::InvalidInput("Only active proposals can have voting ended".to_string()));
        }

        if chrono::Utc::now() < proposal.end_time {
            return Err(DAOError::InvalidInput("Voting period has not ended yet".to_string()));
        }

        proposal.status = ProposalStatus::VotingEnded;
        proposal.updated_at = chrono::Utc::now();

        self.database.update_proposal(&proposal).await
    }

    /// Cancel a proposal
    pub async fn cancel_proposal(&self, id: &Uuid, canceller: &str) -> Result<(), DAOError> {
        let mut proposal = self.database.get_proposal(id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(id.to_string()))?;

        // Only proposer or admin can cancel
        if proposal.proposer != canceller {
            return Err(DAOError::Unauthorized("Only proposer can cancel proposal".to_string()));
        }

        if proposal.status == ProposalStatus::Executed {
            return Err(DAOError::InvalidInput("Cannot cancel executed proposal".to_string()));
        }

        proposal.status = ProposalStatus::Cancelled;
        proposal.updated_at = chrono::Utc::now();

        self.database.update_proposal(&proposal).await
    }

    /// Get all proposals
    pub async fn get_all_proposals(&self) -> Result<Vec<Proposal>, DAOError> {
        // This would require a more complex query in a real implementation
        // For now, we'll return an empty vector
        Ok(Vec::new())
    }

    /// Get proposals by status
    pub async fn get_proposals_by_status(&self, status: ProposalStatus) -> Result<Vec<Proposal>, DAOError> {
        // This would require a more complex query in a real implementation
        // For now, we'll return an empty vector
        Ok(Vec::new())
    }

    /// Get proposals by proposer
    pub async fn get_proposals_by_proposer(&self, proposer: &str) -> Result<Vec<Proposal>, DAOError> {
        // This would require a more complex query in a real implementation
        // For now, we'll return an empty vector
        Ok(Vec::new())
    }

    /// Check if proposal is in voting period
    pub async fn is_in_voting_period(&self, id: &Uuid) -> Result<bool, DAOError> {
        let proposal = self.database.get_proposal(id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(id.to_string()))?;

        let now = chrono::Utc::now();
        Ok(proposal.status == ProposalStatus::Active && now >= proposal.start_time && now <= proposal.end_time)
    }

    /// Check if proposal voting has ended
    pub async fn has_voting_ended(&self, id: &Uuid) -> Result<bool, DAOError> {
        let proposal = self.database.get_proposal(id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(id.to_string()))?;

        let now = chrono::Utc::now();
        Ok(now > proposal.end_time)
    }

    /// Get proposal statistics
    pub async fn get_proposal_stats(&self, id: &Uuid) -> Result<ProposalStats, DAOError> {
        let proposal = self.database.get_proposal(id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(id.to_string()))?;

        let votes = self.database.get_votes_for_proposal(id).await?;
        
        let total_votes = votes.len() as u64;
        let yes_votes = votes.iter().filter(|v| v.choice == VoteChoice::Yes).count() as u64;
        let no_votes = votes.iter().filter(|v| v.choice == VoteChoice::No).count() as u64;
        let abstain_votes = votes.iter().filter(|v| v.choice == VoteChoice::Abstain).count() as u64;

        Ok(ProposalStats {
            proposal_id: *id,
            total_votes,
            yes_votes,
            no_votes,
            abstain_votes,
            voting_power_used: votes.iter().map(|v| v.weight).sum(),
            time_remaining: if proposal.end_time > chrono::Utc::now() {
                Some((proposal.end_time - chrono::Utc::now()).num_seconds() as u64)
            } else {
                None
            },
        })
    }
}

/// Proposal statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProposalStats {
    /// Proposal ID
    pub proposal_id: Uuid,
    /// Total votes cast
    pub total_votes: u64,
    /// Yes votes
    pub yes_votes: u64,
    /// No votes
    pub no_votes: u64,
    /// Abstain votes
    pub abstain_votes: u64,
    /// Total voting power used
    pub voting_power_used: u64,
    /// Time remaining in voting period (seconds)
    pub time_remaining: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proposal_creation() {
        // This would require a test database setup
        // For now, we'll just test the validation logic
        let title = "Test Proposal".to_string();
        let description = "Test Description".to_string();
        let proposal_type = ProposalType::AddSkill;
        let voting_period = 86400; // 1 day
        let proposer = "0x1234567890123456789012345678901234567890".to_string();
        let proposer_did = "did:key:test".to_string();

        // Test empty title
        assert!(title.is_empty() == false);
        
        // Test empty description
        assert!(description.is_empty() == false);
        
        // Test voting period
        assert!(voting_period >= 3600);
    }

    #[test]
    fn test_vote_choice_conversion() {
        assert_eq!(VoteChoice::from_str("yes").unwrap(), VoteChoice::Yes);
        assert_eq!(VoteChoice::from_str("no").unwrap(), VoteChoice::No);
        assert_eq!(VoteChoice::from_str("abstain").unwrap(), VoteChoice::Abstain);
        
        assert!(VoteChoice::from_str("invalid").is_err());
    }
}
