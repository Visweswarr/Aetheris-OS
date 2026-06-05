//! Type definitions for the contracts service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Contract execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: Vec<u8>,
    pub gas_used: u64,
    pub execution_time: u64,
    pub events: Vec<ContractEvent>,
    pub error: Option<String>,
}

/// Contract event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub event_type: String,
    pub data: HashMap<String, String>,
    pub timestamp: u64,
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub wasm_hash: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Contract execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub contract_id: Uuid,
    pub method: String,
    pub args: Vec<u8>,
    pub gas_limit: u64,
    pub caller: String,
    pub value: u64,
}

/// Contract deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub wasm: Vec<u8>,
    pub metadata: ContractMetadata,
    pub gas_limit: u64,
    pub deployer: String,
    pub value: u64,
}

/// Resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub contract_id: Uuid,
    pub cpu_limit: u64,
    pub memory_limit: u64,
    pub storage_limit: u64,
    pub network_limit: u64,
    pub allocated_by: String,
    pub allocated_at: u64,
}

/// Contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractState {
    pub contract_id: Uuid,
    pub storage: HashMap<String, Vec<u8>>,
    pub balance: u64,
    pub last_execution: Option<u64>,
    pub execution_count: u64,
}
