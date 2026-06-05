//! Vote service for managing DAO votes

use crate::database::Database;
use crate::error::DAOError;
use crate::types::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Vote service for managing DAO votes
pub struct VoteService {
    database: Arc<Database>,
}

impl VoteService {
    /// Create a new vote service
    pub async fn new(database: Arc<Database>) -> Result<Self, DAOError> {
        Ok(Self { database })
    }

    /// Cast a vote on a proposal
    pub async fn cast_vote(
        &self,
        proposal_id: Uuid,
        voter: String,
        voter_did: String,
        choice: VoteChoice,
        weight: u64,
        reason: Option<String>,
        signature: String,
    ) -> Result<Vote, DAOError> {
        // Validate inputs
        if weight == 0 {
            return Err(DAOError::InvalidInput("Vote weight must be greater than 0".to_string()));
        }

        // Check if proposal exists and is in voting period
        let proposal = self.database.get_proposal(&proposal_id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.status != ProposalStatus::Active {
            return Err(DAOError::InvalidInput("Proposal is not active for voting".to_string()));
        }

        let now = chrono::Utc::now();
        if now < proposal.start_time {
            return Err(DAOError::VotingPeriodNotStarted("Voting has not started yet".to_string()));
        }
        if now > proposal.end_time {
            return Err(DAOError::VotingPeriodEnded("Voting period has ended".to_string()));
        }

        // Check if voter has already voted
        let existing_votes = self.database.get_votes_for_proposal(&proposal_id).await?;
        if existing_votes.iter().any(|v| v.voter == voter) {
            return Err(DAOError::AlreadyVoted("Voter has already voted on this proposal".to_string()));
        }

        // Create vote
        let vote = Vote {
            id: Uuid::new_v4(),
            proposal_id,
            voter,
            voter_did,
            choice,
            weight,
            reason,
            signature,
            timestamp: now,
        };

        // Save vote to database
        self.database.create_vote(&vote).await?;

        Ok(vote)
    }

    /// Get votes for a proposal
    pub async fn get_votes_for_proposal(&self, proposal_id: &Uuid) -> Result<Vec<Vote>, DAOError> {
        self.database.get_votes_for_proposal(proposal_id).await
    }

    /// Get vote by ID
    pub async fn get_vote(&self, vote_id: &Uuid) -> Result<Option<Vote>, DAOError> {
        // This would require a more complex query in a real implementation
        // For now, we'll return None
        Ok(None)
    }

    /// Get votes by voter
    pub async fn get_votes_by_voter(&self, voter: &str) -> Result<Vec<Vote>, DAOError> {
        // This would require a more complex query in a real implementation
        // For now, we'll return an empty vector
        Ok(Vec::new())
    }

    /// Calculate vote result for a proposal
    pub async fn calculate_vote_result(&self, proposal_id: &Uuid) -> Result<VoteResult, DAOError> {
        let proposal = self.database.get_proposal(proposal_id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(proposal_id.to_string()))?;

        let votes = self.database.get_votes_for_proposal(proposal_id).await?;
        
        let total_votes = votes.len() as u64;
        let yes_votes = votes.iter().filter(|v| v.choice == VoteChoice::Yes).count() as u64;
        let no_votes = votes.iter().filter(|v| v.choice == VoteChoice::No).count() as u64;
        let abstain_votes = votes.iter().filter(|v| v.choice == VoteChoice::Abstain).count() as u64;
        let total_power = votes.iter().map(|v| v.weight).sum();

        // Get governance parameters
        let params = self.database.get_governance_params().await?;

        // Calculate quorum and majority
        let quorum_threshold = (total_power * params.quorum_threshold as u64) / 100;
        let quorum_achieved = total_power >= quorum_threshold;
        
        let majority_threshold = (total_power * params.majority_threshold as u64) / 100;
        let majority_achieved = yes_votes > majority_threshold;

        // Determine result
        let result = if !quorum_achieved {
            VoteResultType::NoQuorum
        } else if !majority_achieved {
            VoteResultType::NoMajority
        } else if yes_votes > no_votes {
            VoteResultType::Passed
        } else {
            VoteResultType::Failed
        };

        Ok(VoteResult {
            proposal_id: *proposal_id,
            total_votes,
            yes_votes,
            no_votes,
            abstain_votes,
            total_power,
            quorum_achieved,
            majority_achieved,
            result,
            calculated_at: chrono::Utc::now(),
        })
    }

    /// Get voting statistics
    pub async fn get_voting_stats(&self, proposal_id: &Uuid) -> Result<VotingStats, DAOError> {
        let votes = self.database.get_votes_for_proposal(proposal_id).await?;
        
        let total_votes = votes.len() as u64;
        let unique_voters = votes.iter().map(|v| &v.voter).collect::<std::collections::HashSet<_>>().len() as u64;
        let total_weight = votes.iter().map(|v| v.weight).sum();
        let average_weight = if total_votes > 0 { total_weight / total_votes } else { 0 };

        // Calculate participation rate (simplified)
        let participation_rate = if total_votes > 0 { (total_votes * 100) / 100 } else { 0 }; // Placeholder

        Ok(VotingStats {
            proposal_id: *proposal_id,
            total_votes,
            unique_voters,
            total_weight,
            average_weight,
            participation_rate,
            last_vote_time: votes.iter().map(|v| v.timestamp).max(),
        })
    }

    /// Verify vote signature
    pub async fn verify_vote_signature(&self, vote: &Vote) -> Result<bool, DAOError> {
        // This would implement actual signature verification
        // For now, we'll return true as a placeholder
        Ok(true)
    }

    /// Get vote distribution
    pub async fn get_vote_distribution(&self, proposal_id: &Uuid) -> Result<VoteDistribution, DAOError> {
        let votes = self.database.get_votes_for_proposal(proposal_id).await?;
        
        let mut distribution = VoteDistribution {
            proposal_id: *proposal_id,
            yes_votes: 0,
            no_votes: 0,
            abstain_votes: 0,
            yes_weight: 0,
            no_weight: 0,
            abstain_weight: 0,
            total_votes: 0,
            total_weight: 0,
        };

        for vote in votes {
            distribution.total_votes += 1;
            distribution.total_weight += vote.weight;

            match vote.choice {
                VoteChoice::Yes => {
                    distribution.yes_votes += 1;
                    distribution.yes_weight += vote.weight;
                }
                VoteChoice::No => {
                    distribution.no_votes += 1;
                    distribution.no_weight += vote.weight;
                }
                VoteChoice::Abstain => {
                    distribution.abstain_votes += 1;
                    distribution.abstain_weight += vote.weight;
                }
            }
        }

        Ok(distribution)
    }
}

/// Voting statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VotingStats {
    /// Proposal ID
    pub proposal_id: Uuid,
    /// Total votes cast
    pub total_votes: u64,
    /// Number of unique voters
    pub unique_voters: u64,
    /// Total voting weight
    pub total_weight: u64,
    /// Average voting weight
    pub average_weight: u64,
    /// Participation rate (percentage)
    pub participation_rate: u64,
    /// Time of last vote
    pub last_vote_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Vote distribution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VoteDistribution {
    /// Proposal ID
    pub proposal_id: Uuid,
    /// Number of yes votes
    pub yes_votes: u64,
    /// Number of no votes
    pub no_votes: u64,
    /// Number of abstain votes
    pub abstain_votes: u64,
    /// Weight of yes votes
    pub yes_weight: u64,
    /// Weight of no votes
    pub no_weight: u64,
    /// Weight of abstain votes
    pub abstain_weight: u64,
    /// Total votes
    pub total_votes: u64,
    /// Total weight
    pub total_weight: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_choice_validation() {
        assert!(VoteChoice::from_str("yes").is_ok());
        assert!(VoteChoice::from_str("no").is_ok());
        assert!(VoteChoice::from_str("abstain").is_ok());
        assert!(VoteChoice::from_str("invalid").is_err());
    }

    #[test]
    fn test_vote_choice_string_conversion() {
        assert_eq!(VoteChoice::Yes.as_str(), "yes");
        assert_eq!(VoteChoice::No.as_str(), "no");
        assert_eq!(VoteChoice::Abstain.as_str(), "abstain");
    }
}
