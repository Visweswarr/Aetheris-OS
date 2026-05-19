//! Type definitions for the DAO service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// DAO proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    /// Unique proposal ID
    pub id: Uuid,
    /// Proposal title
    pub title: String,
    /// Proposal description
    pub description: String,
    /// Proposal type
    pub proposal_type: ProposalType,
    /// Voting start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Voting end time
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// Proposal creator
    pub proposer: String,
    /// Proposal creator DID
    pub proposer_did: String,
    /// Current status
    pub status: ProposalStatus,
    /// Execution data
    pub execution_data: Option<ExecutionData>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Proposal type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    /// Add a new skill to the system
    AddSkill,
    /// Update system policy
    UpdatePolicy,
    /// Change governance parameters
    ChangeGovernance,
    /// Execute arbitrary code
    ExecuteCode,
    /// Transfer funds
    TransferFunds,
    /// Other custom proposal
    Custom(String),
}

/// Proposal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    /// Proposal is being created
    Draft,
    /// Proposal is active and accepting votes
    Active,
    /// Voting period has ended
    VotingEnded,
    /// Proposal passed and is ready for execution
    Passed,
    /// Proposal failed to pass
    Failed,
    /// Proposal has been executed
    Executed,
    /// Proposal was cancelled
    Cancelled,
}

/// Execution data for proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionData {
    /// Execution target (contract address, function name, etc.)
    pub target: String,
    /// Execution parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Execution method
    pub method: ExecutionMethod,
    /// Gas limit for execution
    pub gas_limit: Option<u64>,
    /// Value to send with execution
    pub value: Option<u64>,
}

/// Execution method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionMethod {
    /// Smart contract call
    ContractCall,
    /// System function call
    SystemCall,
    /// Policy update
    PolicyUpdate,
    /// Configuration change
    ConfigChange,
    /// Custom execution
    Custom(String),
}

/// Vote on a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    /// Unique vote ID
    pub id: Uuid,
    /// Proposal ID
    pub proposal_id: Uuid,
    /// Voter address
    pub voter: String,
    /// Voter DID
    pub voter_did: String,
    /// Vote choice
    pub choice: VoteChoice,
    /// Voting weight
    pub weight: u64,
    /// Vote reason (optional)
    pub reason: Option<String>,
    /// Vote signature
    pub signature: String,
    /// Vote timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Vote choice
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteChoice {
    /// Vote in favor
    Yes,
    /// Vote against
    No,
    /// Abstain from voting
    Abstain,
}

impl VoteChoice {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            VoteChoice::Yes => "yes",
            VoteChoice::No => "no",
            VoteChoice::Abstain => "abstain",
        }
    }

    /// Convert from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "yes" | "y" | "1" => Ok(VoteChoice::Yes),
            "no" | "n" | "0" => Ok(VoteChoice::No),
            "abstain" | "a" | "-" => Ok(VoteChoice::Abstain),
            _ => Err(format!("Invalid vote choice: {}", s)),
        }
    }
}

/// Vote result for a proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResult {
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
    /// Total voting power
    pub total_power: u64,
    /// Quorum achieved
    pub quorum_achieved: bool,
    /// Majority achieved
    pub majority_achieved: bool,
    /// Final result
    pub result: VoteResultType,
    /// Calculated timestamp
    pub calculated_at: chrono::DateTime<chrono::Utc>,
}

/// Vote result type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteResultType {
    /// Proposal passed
    Passed,
    /// Proposal failed
    Failed,
    /// Quorum not met
    NoQuorum,
    /// No majority
    NoMajority,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Proposal ID
    pub proposal_id: Uuid,
    /// Execution success
    pub success: bool,
    /// Execution output
    pub output: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Gas used
    pub gas_used: Option<u64>,
    /// Transaction hash
    pub tx_hash: Option<String>,
    /// Execution timestamp
    pub executed_at: chrono::DateTime<chrono::Utc>,
    /// Executor
    pub executor: String,
}

/// Governance parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceParams {
    /// Minimum voting period (seconds)
    pub min_voting_period: u64,
    /// Maximum voting period (seconds)
    pub max_voting_period: u64,
    /// Minimum votes required
    pub min_votes_required: u64,
    /// Majority threshold (percentage)
    pub majority_threshold: u8,
    /// Quorum threshold (percentage)
    pub quorum_threshold: u8,
    /// Execution delay (seconds)
    pub execution_delay: u64,
    /// Proposal deposit required
    pub proposal_deposit: u64,
    /// Voting power calculation method
    pub voting_power_method: VotingPowerMethod,
}

/// Voting power calculation method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VotingPowerMethod {
    /// One vote per address
    OneVotePerAddress,
    /// Based on token balance
    TokenBalance,
    /// Based on reputation score
    ReputationScore,
    /// Based on stake amount
    StakeAmount,
    /// Custom calculation
    Custom(String),
}

/// DAO member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAOMember {
    /// Member address
    pub address: String,
    /// Member DID
    pub did: String,
    /// Member reputation score
    pub reputation: u64,
    /// Member stake amount
    pub stake: u64,
    /// Member voting power
    pub voting_power: u64,
    /// Member status
    pub status: MemberStatus,
    /// Joined timestamp
    pub joined_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// Member status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberStatus {
    /// Active member
    Active,
    /// Suspended member
    Suspended,
    /// Banned member
    Banned,
    /// Inactive member
    Inactive,
}

/// DAO statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAOStats {
    /// Total number of proposals
    pub total_proposals: u64,
    /// Active proposals
    pub active_proposals: u64,
    /// Passed proposals
    pub passed_proposals: u64,
    /// Failed proposals
    pub failed_proposals: u64,
    /// Executed proposals
    pub executed_proposals: u64,
    /// Total number of votes
    pub total_votes: u64,
    /// Total number of members
    pub total_members: u64,
    /// Active members
    pub active_members: u64,
    /// Total voting power
    pub total_voting_power: u64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for GovernanceParams {
    fn default() -> Self {
        Self {
            min_voting_period: 86400, // 1 day
            max_voting_period: 604800, // 7 days
            min_votes_required: 3,
            majority_threshold: 51,
            quorum_threshold: 25,
            execution_delay: 3600, // 1 hour
            proposal_deposit: 0,
            voting_power_method: VotingPowerMethod::OneVotePerAddress,
        }
    }
}
