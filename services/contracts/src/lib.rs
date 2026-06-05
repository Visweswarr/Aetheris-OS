//! Contracts service for Aetheris OS
//! 
//! This module provides smart contract execution capabilities with WASM runtime,
//! resource allocation tied to DAO votes, and multi-chain transaction relay.

pub mod sandbox;
pub mod zkvm;
pub mod runtime;
pub mod error;
pub mod types;

pub use sandbox::{ContractSandbox, ContractV1, ResultV1, ExecutionRequest, GasConfig};
pub use zkvm::{ZKVMService, ZKProof, ZKAlgorithm};
pub use runtime::{ContractRuntime, ResourceAllocation, ResourceSpec, AllocationStatus, RuntimeStats};
pub use error::ContractError;
pub use types::*;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Contract service configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractConfig {
    /// Maximum gas per execution
    pub max_gas_per_execution: u64,
    /// Maximum memory allocation in bytes
    pub max_memory_bytes: u64,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum concurrent executions
    pub max_concurrent_executions: u32,
    /// Enable ZK proof generation
    pub enable_zk_proofs: bool,
    /// Gas configuration
    pub gas_config: GasConfig,
    /// DAO service configuration
    pub dao_config: dao::DAOConfig,
    /// Chain service configuration
    pub chain_config: chain::ChainServiceConfig,
}

impl Default for ContractConfig {
    fn default() -> Self {
        Self {
            max_gas_per_execution: 1_000_000,
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            max_execution_time_ms: 30_000, // 30 seconds
            max_concurrent_executions: 100,
            enable_zk_proofs: true,
            gas_config: GasConfig::default(),
            dao_config: dao::DAOConfig::default(),
            chain_config: chain::ChainServiceConfig::default(),
        }
    }
}

/// Main contract service
pub struct ContractService {
    config: ContractConfig,
    runtime: Arc<ContractRuntime>,
    sandbox: Arc<ContractSandbox>,
    zkvm_service: Arc<ZKVMService>,
}

impl ContractService {
    /// Create a new contract service
    pub async fn new(config: ContractConfig) -> Result<Self, ContractError> {
        // Initialize DAO service
        let dao_service = Arc::new(dao::DAOService::new(config.dao_config.clone()).await?);
        
        // Initialize chain service
        let chain_service = Arc::new(chain::ChainService::new(config.chain_config.clone())?);
        
        // Initialize runtime
        let runtime = Arc::new(
            ContractRuntime::new(dao_service, chain_service)
                .await
                .map_err(|e| ContractError::WasmError(e.to_string()))?,
        );
        
        // Initialize sandbox
        let sandbox = Arc::new(
            ContractSandbox::new()
                .map_err(|e| ContractError::WasmError(e.to_string()))?,
        );
        
        // Initialize ZKVM service
        let zkvm_service = Arc::new(ZKVMService::new());
        
        Ok(Self {
            config,
            runtime,
            sandbox,
            zkvm_service,
        })
    }

    /// Get the runtime
    pub fn runtime(&self) -> Arc<ContractRuntime> {
        self.runtime.clone()
    }

    /// Get the sandbox
    pub fn sandbox(&self) -> Arc<ContractSandbox> {
        self.sandbox.clone()
    }

    /// Get the ZKVM service
    pub fn zkvm_service(&self) -> Arc<ZKVMService> {
        self.zkvm_service.clone()
    }

    /// Get the configuration
    pub fn config(&self) -> &ContractConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_config_default() {
        let config = ContractConfig::default();
        assert_eq!(config.max_gas_per_execution, 1_000_000);
        assert_eq!(config.max_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(config.max_execution_time_ms, 30_000);
        assert_eq!(config.max_concurrent_executions, 100);
        assert!(config.enable_zk_proofs);
    }

    #[tokio::test]
    async fn test_contract_service_creation() {
        let config = ContractConfig::default();
        let service = ContractService::new(config).await;
        assert!(service.is_ok());
    }
}
