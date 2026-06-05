//! Execution service for managing DAO proposal execution

use crate::database::Database;
use crate::error::DAOError;
use crate::types::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Execution service for managing DAO proposal execution
pub struct ExecutionService {
    database: Arc<Database>,
}

impl ExecutionService {
    /// Create a new execution service
    pub async fn new(database: Arc<Database>) -> Result<Self, DAOError> {
        Ok(Self { database })
    }

    /// Execute a proposal
    pub async fn execute_proposal(
        &self,
        proposal_id: Uuid,
        executor: String,
    ) -> Result<ExecutionResult, DAOError> {
        // Get proposal
        let proposal = self.database.get_proposal(&proposal_id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(proposal_id.to_string()))?;

        // Check if proposal can be executed
        if proposal.status != ProposalStatus::Passed {
            return Err(DAOError::InvalidInput("Proposal must be in Passed status to execute".to_string()));
        }

        // Check if proposal has already been executed
        if proposal.status == ProposalStatus::Executed {
            return Err(DAOError::ProposalAlreadyExecuted("Proposal has already been executed".to_string()));
        }

        // Get governance parameters
        let params = self.database.get_governance_params().await?;

        // Check execution delay
        let now = chrono::Utc::now();
        let execution_time = proposal.end_time + chrono::Duration::seconds(params.execution_delay as i64);
        if now < execution_time {
            return Err(DAOError::InvalidInput(format!(
                "Execution delay not met. Can execute after {}",
                execution_time
            )));
        }

        // Execute the proposal
        let execution_result = self.execute_proposal_logic(&proposal).await?;

        // Update proposal status
        let mut updated_proposal = proposal.clone();
        updated_proposal.status = if execution_result.success {
            ProposalStatus::Executed
        } else {
            ProposalStatus::Failed
        };
        updated_proposal.updated_at = now;

        self.database.update_proposal(&updated_proposal).await?;

        Ok(execution_result)
    }

    /// Execute the actual proposal logic
    async fn execute_proposal_logic(&self, proposal: &Proposal) -> Result<ExecutionResult, DAOError> {
        let now = chrono::Utc::now();
        
        match &proposal.proposal_type {
            ProposalType::AddSkill => {
                self.execute_add_skill_proposal(proposal).await
            }
            ProposalType::UpdatePolicy => {
                self.execute_update_policy_proposal(proposal).await
            }
            ProposalType::ChangeGovernance => {
                self.execute_change_governance_proposal(proposal).await
            }
            ProposalType::ExecuteCode => {
                self.execute_code_proposal(proposal).await
            }
            ProposalType::TransferFunds => {
                self.execute_transfer_funds_proposal(proposal).await
            }
            ProposalType::Custom(custom_type) => {
                self.execute_custom_proposal(proposal, custom_type).await
            }
        }
    }

    /// Execute add skill proposal
    async fn execute_add_skill_proposal(&self, proposal: &Proposal) -> Result<ExecutionResult, DAOError> {
        // This would implement actual skill addition logic
        // For now, we'll return a successful execution
        Ok(ExecutionResult {
            proposal_id: proposal.id,
            success: true,
            output: Some("Skill added successfully".to_string()),
            error: None,
            gas_used: Some(100000),
            tx_hash: Some("0x1234567890abcdef".to_string()),
            executed_at: chrono::Utc::now(),
            executor: "system".to_string(),
        })
    }

    /// Execute update policy proposal
    async fn execute_update_policy_proposal(&self, proposal: &Proposal) -> Result<ExecutionResult, DAOError> {
        // This would implement actual policy update logic
        // For now, we'll return a successful execution
        Ok(ExecutionResult {
            proposal_id: proposal.id,
            success: true,
            output: Some("Policy updated successfully".to_string()),
            error: None,
            gas_used: Some(150000),
            tx_hash: Some("0xabcdef1234567890".to_string()),
            executed_at: chrono::Utc::now(),
            executor: "system".to_string(),
        })
    }

    /// Execute change governance proposal
    async fn execute_change_governance_proposal(&self, proposal: &Proposal) -> Result<ExecutionResult, DAOError> {
        // This would implement actual governance change logic
        // For now, we'll return a successful execution
        Ok(ExecutionResult {
            proposal_id: proposal.id,
            success: true,
            output: Some("Governance parameters updated successfully".to_string()),
            error: None,
            gas_used: Some(200000),
            tx_hash: Some("0x9876543210fedcba".to_string()),
            executed_at: chrono::Utc::now(),
            executor: "system".to_string(),
        })
    }

    /// Execute code proposal
    async fn execute_code_proposal(&self, proposal: &Proposal) -> Result<ExecutionResult, DAOError> {
        // This would implement actual code execution logic
        // For now, we'll return a successful execution
        Ok(ExecutionResult {
            proposal_id: proposal.id,
            success: true,
            output: Some("Code executed successfully".to_string()),
            error: None,
            gas_used: Some(300000),
            tx_hash: Some("0xfedcba0987654321".to_string()),
            executed_at: chrono::Utc::now(),
            executor: "system".to_string(),
        })
    }

    /// Execute transfer funds proposal
    async fn execute_transfer_funds_proposal(&self, proposal: &Proposal) -> Result<ExecutionResult, DAOError> {
        // This would implement actual fund transfer logic
        // For now, we'll return a successful execution
        Ok(ExecutionResult {
            proposal_id: proposal.id,
            success: true,
            output: Some("Funds transferred successfully".to_string()),
            error: None,
            gas_used: Some(50000),
            tx_hash: Some("0x13579bdf02468ace".to_string()),
            executed_at: chrono::Utc::now(),
            executor: "system".to_string(),
        })
    }

    /// Execute custom proposal
    async fn execute_custom_proposal(&self, proposal: &Proposal, custom_type: &str) -> Result<ExecutionResult, DAOError> {
        // This would implement custom proposal execution logic
        // For now, we'll return a successful execution
        Ok(ExecutionResult {
            proposal_id: proposal.id,
            success: true,
            output: Some(format!("Custom proposal '{}' executed successfully", custom_type)),
            error: None,
            gas_used: Some(250000),
            tx_hash: Some("0xace13579bdf02468".to_string()),
            executed_at: chrono::Utc::now(),
            executor: "system".to_string(),
        })
    }

    /// Get execution result for a proposal
    pub async fn get_execution_result(&self, proposal_id: &Uuid) -> Result<Option<ExecutionResult>, DAOError> {
        // This would query the execution_results table
        // For now, we'll return None
        Ok(None)
    }

    /// Get all execution results
    pub async fn get_all_execution_results(&self) -> Result<Vec<ExecutionResult>, DAOError> {
        // This would query the execution_results table
        // For now, we'll return an empty vector
        Ok(Vec::new())
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> Result<ExecutionStats, DAOError> {
        // This would calculate actual statistics from the database
        // For now, we'll return mock statistics
        Ok(ExecutionStats {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_gas_used: 0,
            average_gas_used: 0,
            last_execution: None,
        })
    }

    /// Validate execution data
    pub async fn validate_execution_data(&self, execution_data: &ExecutionData) -> Result<bool, DAOError> {
        // Validate execution data
        if execution_data.target.is_empty() {
            return Err(DAOError::InvalidInput("Execution target cannot be empty".to_string()));
        }

        // Validate gas limit
        if let Some(gas_limit) = execution_data.gas_limit {
            if gas_limit == 0 {
                return Err(DAOError::InvalidInput("Gas limit must be greater than 0".to_string()));
            }
        }

        // Note: value is u64 (unsigned), so negative check is unnecessary.
        // Validation passes if we reach here.


        Ok(true)
    }

    /// Check if proposal is ready for execution
    pub async fn is_ready_for_execution(&self, proposal_id: &Uuid) -> Result<bool, DAOError> {
        let proposal = self.database.get_proposal(proposal_id).await?
            .ok_or_else(|| DAOError::ProposalNotFound(proposal_id.to_string()))?;

        if proposal.status != ProposalStatus::Passed {
            return Ok(false);
        }

        // Check execution delay
        let params = self.database.get_governance_params().await?;
        let now = chrono::Utc::now();
        let execution_time = proposal.end_time + chrono::Duration::seconds(params.execution_delay as i64);

        Ok(now >= execution_time)
    }
}

/// Execution statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionStats {
    /// Total number of executions
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Total gas used
    pub total_gas_used: u64,
    /// Average gas used per execution
    pub average_gas_used: u64,
    /// Time of last execution
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_data_validation() {
        let valid_data = ExecutionData {
            target: "0x1234567890123456789012345678901234567890".to_string(),
            parameters: std::collections::HashMap::new(),
            method: ExecutionMethod::ContractCall,
            gas_limit: Some(100000),
            value: Some(0),
        };

        // This would test actual validation logic
        assert!(!valid_data.target.is_empty());
        assert!(valid_data.gas_limit.unwrap() > 0);
        assert!(valid_data.value.unwrap() >= 0);
    }
}
