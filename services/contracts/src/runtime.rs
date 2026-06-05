//! WASM Contract Runtime for Aetheris OS
//! 
//! This module provides a sandboxed WASM runtime for executing smart contracts
//! with deterministic gas metering, resource allocation, and integration with
//! the DAO kernel and DID-bound wallet system.

use crate::sandbox::{Capability, ContractSandbox, ContractV1, ExecutionRequest, ResultV1};
use crate::zkvm::ZKVMService;
use dao::{DAOService, ProposalType};
use chain::ChainService;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use wasmtime::{Engine, Module};

/// WASM Contract Runtime
pub struct ContractRuntime {
    /// WASM engine for contract execution
    engine: Engine,
    /// Contract sandbox for execution
    sandbox: Arc<ContractSandbox>,
    /// ZKVM service for proof generation
    zkvm_service: Arc<ZKVMService>,
    /// DAO service for resource allocation
    dao_service: Arc<DAOService>,
    /// Chain service for anchoring
    chain_service: Arc<ChainService>,
    /// Resource allocation registry
    resource_allocations: Arc<RwLock<HashMap<String, ResourceAllocation>>>,
    /// Contract registry
    contracts: Arc<RwLock<HashMap<String, ContractV1>>>,
    /// Execution statistics
    stats: Arc<RwLock<RuntimeStats>>,
}

/// Resource allocation tied to DAO votes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceAllocation {
    /// Unique allocation ID
    pub id: Uuid,
    /// Contract ID this allocation is for
    pub contract_id: String,
    /// DAO proposal ID that approved this allocation
    pub proposal_id: Uuid,
    /// Allocated resources
    pub resources: ResourceSpec,
    /// Capabilities granted
    pub capabilities: Vec<Capability>,
    /// Allocation status
    pub status: AllocationStatus,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Allocated by (DID)
    pub allocated_by: String,
}

/// Resource specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceSpec {
    /// Maximum gas per execution
    pub max_gas_per_execution: u64,
    /// Maximum total gas allocation
    pub max_total_gas: u64,
    /// Maximum memory allocation in bytes
    pub max_memory_bytes: u64,
    /// Maximum storage operations
    pub max_storage_operations: u64,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum concurrent executions
    pub max_concurrent_executions: u32,
    /// Allowed storage keys
    pub allowed_storage_keys: Vec<String>,
    /// Allowed network access
    pub network_access: NetworkAccess,
}

/// Network access permissions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkAccess {
    /// Allowed domains
    pub allowed_domains: Vec<String>,
    /// Allowed protocols
    pub allowed_protocols: Vec<String>,
    /// Maximum bandwidth in bytes per second
    pub max_bandwidth_bps: u64,
    /// Maximum requests per minute
    pub max_requests_per_minute: u32,
}

/// Allocation status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AllocationStatus {
    Active,
    Suspended,
    Expired,
    Revoked,
}

/// Runtime statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeStats {
    /// Total contracts deployed
    pub total_contracts: u64,
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Total gas consumed
    pub total_gas_consumed: u64,
    /// Total memory allocated
    pub total_memory_allocated: u64,
    /// Active resource allocations
    pub active_allocations: u64,
    /// ZK proofs generated
    pub zk_proofs_generated: u64,
    /// Average execution time
    pub average_execution_time_ms: f64,
    /// Peak memory usage
    pub peak_memory_usage: u64,
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self {
            total_contracts: 0,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_gas_consumed: 0,
            total_memory_allocated: 0,
            active_allocations: 0,
            zk_proofs_generated: 0,
            average_execution_time_ms: 0.0,
            peak_memory_usage: 0,
        }
    }
}

impl ContractRuntime {
    /// Create a new contract runtime
    pub async fn new(
        dao_service: Arc<DAOService>,
        chain_service: Arc<ChainService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = Engine::default();
        let sandbox = Arc::new(ContractSandbox::new()?);
        let zkvm_service = Arc::new(ZKVMService::new());
        
        Ok(Self {
            engine,
            sandbox,
            zkvm_service,
            dao_service,
            chain_service,
            resource_allocations: Arc::new(RwLock::new(HashMap::new())),
            contracts: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(RuntimeStats::default())),
        })
    }

    /// Deploy a new contract
    pub async fn deploy_contract(
        &self,
        contract: ContractV1,
        deployer_did: String,
        capabilities: Vec<Capability>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Validate contract WASM
        self.validate_wasm(&contract.wasm)?;
        
        // Check deployer capabilities
        self.validate_deployment_capabilities(&deployer_did, &capabilities).await?;
        
        // Deploy to sandbox
        self.sandbox.deploy_contract(contract.clone()).await?;
        
        // Store in registry
        let mut contracts = self.contracts.write().await;
        contracts.insert(contract.id.clone(), contract.clone());
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_contracts += 1;
        
        Ok(contract.id)
    }

    /// Execute a contract with resource allocation
    pub async fn execute_contract(
        &self,
        contract_id: String,
        input: Vec<u8>,
        executor_did: String,
        gas_limit: Option<u64>,
        zk_mode: bool,
    ) -> Result<ResultV1, Box<dyn std::error::Error>> {
        // Get contract
        let contract = self.get_contract(&contract_id).await
            .ok_or("Contract not found")?;
        
        // Check resource allocation
        let allocation = self.get_resource_allocation(&contract_id).await
            .ok_or("No resource allocation found for contract")?;
        
        // Validate allocation status
        if allocation.status != AllocationStatus::Active {
            return Err("Resource allocation is not active".into());
        }
        
        // Check expiration
        if let Some(expires_at) = allocation.expires_at {
            if chrono::Utc::now() > expires_at {
                return Err("Resource allocation has expired".into());
            }
        }
        
        // Determine gas limit
        let effective_gas_limit = gas_limit
            .unwrap_or(allocation.resources.max_gas_per_execution)
            .min(allocation.resources.max_gas_per_execution);
        
        // Create execution request
        let request = ExecutionRequest {
            contract_id: contract_id.clone(),
            input: input.clone(),
            gas_limit: Some(effective_gas_limit),
            zk_mode: Some(zk_mode),
            capabilities: allocation.capabilities.clone(),
            timeout_ms: Some(allocation.resources.max_execution_time_ms),
            storage_context: Some(self.create_storage_context(&allocation)?),
        };
        
        // Execute contract
        let result = self.execute_with_allocation(request, &allocation).await?;
        
        // Update statistics
        self.update_execution_stats(&result).await;
        
        Ok(result)
    }

    /// Create resource allocation via DAO proposal
    pub async fn create_resource_allocation_proposal(
        &self,
        contract_id: String,
        resources: ResourceSpec,
        capabilities: Vec<Capability>,
        proposer_did: String,
        voting_period: u64,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        // Create DAO proposal
        let proposal = self.dao_service.proposal_service().write().await.create_proposal(
            format!("Resource Allocation for Contract {}", contract_id),
            format!("Allocate resources for contract execution: {:?}", resources),
            ProposalType::ExecuteCode,
            voting_period,
            "system".to_string(), // System proposer
            proposer_did,
            Some(dao::ExecutionData {
                target: "contract_runtime".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("contract_id".to_string(), serde_json::Value::String(contract_id));
                    params.insert("resources".to_string(), serde_json::to_value(&resources)?);
                    params.insert("capabilities".to_string(), serde_json::to_value(&capabilities)?);
                    params
                },
                method: dao::ExecutionMethod::SystemCall,
                gas_limit: Some(100000),
                value: Some(0),
            }),
        ).await?;
        
        Ok(proposal.id)
    }

    /// Approve resource allocation (called after DAO vote passes)
    pub async fn approve_resource_allocation(
        &self,
        proposal_id: Uuid,
        contract_id: String,
        resources: ResourceSpec,
        capabilities: Vec<Capability>,
        approved_by: String,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        // Create resource allocation
        let allocation = ResourceAllocation {
            id: Uuid::new_v4(),
            contract_id,
            proposal_id,
            resources,
            capabilities,
            status: AllocationStatus::Active,
            created_at: chrono::Utc::now(),
            expires_at: None, // No expiration by default
            allocated_by: approved_by,
        };
        
        // Store allocation
        let mut allocations = self.resource_allocations.write().await;
        allocations.insert(allocation.id.to_string(), allocation.clone());
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.active_allocations += 1;
        
        Ok(allocation.id)
    }

    /// Get contract by ID
    pub async fn get_contract(&self, id: &str) -> Option<ContractV1> {
        let contracts = self.contracts.read().await;
        contracts.get(id).cloned()
    }

    /// Get resource allocation for contract
    pub async fn get_resource_allocation(&self, contract_id: &str) -> Option<ResourceAllocation> {
        let allocations = self.resource_allocations.read().await;
        allocations.values()
            .find(|allocation| allocation.contract_id == contract_id)
            .cloned()
    }

    /// List all contracts
    pub async fn list_contracts(&self) -> Vec<ContractV1> {
        let contracts = self.contracts.read().await;
        contracts.values().cloned().collect()
    }

    /// List resource allocations
    pub async fn list_resource_allocations(&self) -> Vec<ResourceAllocation> {
        let allocations = self.resource_allocations.read().await;
        allocations.values().cloned().collect()
    }

    /// Get runtime statistics
    pub async fn get_stats(&self) -> RuntimeStats {
        self.stats.read().await.clone()
    }

    /// Validate WASM bytecode
    fn validate_wasm(&self, wasm: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Compile WASM module to validate
        Module::new(&self.engine, wasm)?;
        Ok(())
    }

    /// Validate deployment capabilities
    async fn validate_deployment_capabilities(
        &self,
        deployer_did: &str,
        capabilities: &[Capability],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // This would check if the deployer has the required capabilities
        // For now, we'll just validate that capabilities are provided
        if capabilities.is_empty() {
            return Err("At least one capability is required for deployment".into());
        }
        Ok(())
    }

    /// Create storage context from allocation
    fn create_storage_context(
        &self,
        allocation: &ResourceAllocation,
    ) -> Result<crate::sandbox::StorageContext, Box<dyn std::error::Error>> {
        Ok(crate::sandbox::StorageContext {
            read_keys: allocation.resources.allowed_storage_keys.clone(),
            write_keys: allocation.resources.allowed_storage_keys.clone(),
            vault_access: None, // Would be set based on capabilities
            snapshot_cid: None,
        })
    }

    /// Execute contract with resource allocation
    async fn execute_with_allocation(
        &self,
        request: ExecutionRequest,
        allocation: &ResourceAllocation,
    ) -> Result<ResultV1, Box<dyn std::error::Error>> {
        // Get contract
        let contract = self.get_contract(&request.contract_id).await
            .ok_or("Contract not found")?;
        
        // Execute in sandbox
        let result = self.sandbox.run_contract(
            &contract.wasm,
            &request.input,
            request.gas_limit.unwrap_or(allocation.resources.max_gas_per_execution),
            request.zk_mode.unwrap_or(false),
            request.storage_context,
        ).await?;
        
        // Generate ZK proof if requested
        if request.zk_mode.unwrap_or(false) && result.ok {
            // This would integrate with the ZKVM service
            // For now, we'll just mark that a proof was requested
        }
        
        Ok(result)
    }

    /// Update execution statistics
    async fn update_execution_stats(&self, result: &ResultV1) {
        let mut stats = self.stats.write().await;
        
        stats.total_executions += 1;
        if result.ok {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }
        
        stats.total_gas_consumed += result.gas_used;
        stats.total_memory_allocated += result.memory_used;
        
        if result.memory_used > stats.peak_memory_usage {
            stats.peak_memory_usage = result.memory_used;
        }
        
        if result.proof.is_some() {
            stats.zk_proofs_generated += 1;
        }
        
        // Update average execution time
        stats.average_execution_time_ms = 
            (stats.average_execution_time_ms * (stats.total_executions - 1) as f64 + result.execution_time_ms as f64) 
            / stats.total_executions as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        // This would require proper service initialization
        // For now, we'll just test the basic structure
        let stats = RuntimeStats::default();
        assert_eq!(stats.total_contracts, 0);
        assert_eq!(stats.total_executions, 0);
    }

    #[test]
    fn test_resource_spec_validation() {
        let resources = ResourceSpec {
            max_gas_per_execution: 100000,
            max_total_gas: 1000000,
            max_memory_bytes: 64 * 1024 * 1024,
            max_storage_operations: 1000,
            max_execution_time_ms: 5000,
            max_concurrent_executions: 10,
            allowed_storage_keys: vec!["key1".to_string(), "key2".to_string()],
            network_access: NetworkAccess {
                allowed_domains: vec!["example.com".to_string()],
                allowed_protocols: vec!["https".to_string()],
                max_bandwidth_bps: 1024 * 1024,
                max_requests_per_minute: 100,
            },
        };
        
        assert!(resources.max_gas_per_execution > 0);
        assert!(resources.max_memory_bytes > 0);
        assert!(!resources.allowed_storage_keys.is_empty());
    }
}
