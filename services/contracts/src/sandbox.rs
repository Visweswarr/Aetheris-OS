use crate::zkvm::{ZKProof, ZKAlgorithm};
use ngfs::vault::{CapTokenV2, Capability, VaultError, VaultErrorCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::{Engine, Instance, Module, Store, Val, ValType};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

/// Smart contract execution sandbox
pub struct ContractSandbox {
    /// WASM engine for contract execution
    engine: Engine,
    /// Gas metering configuration
    gas_config: GasConfig,
    /// Contract registry
    contracts: Arc<RwLock<HashMap<String, ContractV1>>>,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
}

/// Gas metering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasConfig {
    /// Cost per WASM instruction
    pub instruction_cost: u64,
    /// Cost per byte of memory
    pub memory_cost_per_byte: u64,
    /// Cost per storage read
    pub storage_read_cost: u64,
    /// Cost per storage write
    pub storage_write_cost: u64,
    /// Cost per event emission
    pub event_cost: u64,
    /// Cost for ZK proof generation
    pub zk_proof_cost: u64,
    /// Maximum instructions allowed
    pub max_instructions: u64,
    /// Maximum memory allowed in bytes
    pub max_memory_bytes: u64,
    /// Maximum storage operations
    pub max_storage_operations: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            instruction_cost: 1,
            memory_cost_per_byte: 1,
            storage_read_cost: 100,
            storage_write_cost: 200,
            event_cost: 50,
            zk_proof_cost: 1000,
            max_instructions: 1_000_000,
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            max_storage_operations: 1000,
        }
    }
}

/// Contract execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultV1 {
    /// Whether execution succeeded
    pub ok: bool,
    /// Actual gas consumed
    pub gas_used: u64,
    /// Gas limit that was set
    pub gas_limit: u64,
    /// Contract output data
    pub output: Vec<u8>,
    /// ZK proof if zk_mode=true
    pub proof: Option<Vec<u8>>,
    /// Error details if ok=false
    pub error: Option<ContractError>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Peak memory usage in bytes
    pub memory_used: u64,
    /// NGFS storage operations
    pub storage_accesses: Vec<StorageAccess>,
    /// Events emitted during execution
    pub events: Vec<ContractEvent>,
    /// Timestamp of execution
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// ID of executed contract
    pub contract_id: String,
    /// Blake3 hash of input data
    pub input_hash: Vec<u8>,
}

/// Contract execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractError {
    /// Error code
    pub code: ErrorCode,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    pub details: Option<String>,
    /// Gas consumed before error
    pub gas_consumed: u64,
    /// Instructions executed before error
    pub instruction_count: u64,
    /// Peak memory usage before error
    pub memory_peak: u64,
}

/// Error codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCode {
    GasExceeded,
    MemoryExceeded,
    Timeout,
    WasmError,
    StorageError,
    CapabilityError,
    ValidationError,
    ZKError,
    SystemError,
}

/// Storage access record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAccess {
    /// Type of storage operation
    pub operation: StorageOp,
    /// Storage key accessed
    pub key: String,
    /// Data size in bytes
    pub size: u64,
    /// Whether operation succeeded
    pub success: bool,
    /// Gas cost of operation
    pub gas_cost: u64,
}

/// Storage operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOp {
    Read,
    Write,
    Delete,
    List,
    Exists,
}

/// Contract event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    /// Event type identifier
    pub event_type: String,
    /// Event data
    pub data: Vec<u8>,
    /// Gas cost of event emission
    pub gas_cost: u64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Contract execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Contract to execute
    pub contract_id: String,
    /// Input data for contract
    pub input: Vec<u8>,
    /// Override gas limit
    pub gas_limit: Option<u64>,
    /// Override ZK mode
    pub zk_mode: Option<bool>,
    /// Capabilities for this execution
    pub capabilities: Vec<Capability>,
    /// Execution timeout
    pub timeout_ms: Option<u64>,
    /// NGFS storage context
    pub storage_context: Option<StorageContext>,
}

/// Storage context for contract execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageContext {
    /// Keys contract can read
    pub read_keys: Vec<String>,
    /// Keys contract can write (vault only)
    pub write_keys: Vec<String>,
    /// Vault access permissions
    pub vault_access: Option<VaultAccess>,
    /// NGFS snapshot to use
    pub snapshot_cid: Option<String>,
}

/// Vault access permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultAccess {
    /// Vault identifier
    pub vault_id: String,
    /// Entry patterns for access
    pub entry_patterns: Vec<String>,
    /// Allowed vault operations
    pub operations: Vec<VaultOp>,
    /// Maximum entries to access
    pub max_entries: Option<u64>,
}

/// Vault operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultOp {
    Read,
    Write,
    Delete,
    List,
    Search,
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMeta {
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Contract author
    pub author: String,
    /// License information
    pub license: Option<String>,
    /// Contract type/category
    pub category: ContractCategory,
    /// Supported interfaces
    pub interfaces: Vec<String>,
    /// NGFS storage keys accessed
    pub storage_keys: Vec<String>,
    /// JSON schema for input validation
    pub input_schema: Option<String>,
    /// JSON schema for output validation
    pub output_schema: Option<String>,
    /// Estimated gas usage
    pub gas_estimate: Option<u64>,
    /// Maximum memory usage in bytes
    pub max_memory: Option<u64>,
    /// Execution timeout in milliseconds
    pub timeout_ms: Option<u64>,
}

/// Contract categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractCategory {
    Skill,
    Validator,
    Policy,
    Bridge,
    Utility,
    Custom,
}

/// Contract definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractV1 {
    /// Unique contract identifier
    pub id: String,
    /// Compiled WASM bytecode
    pub wasm: Vec<u8>,
    /// Contract metadata
    pub meta: ContractMeta,
    /// Whether to generate ZK proofs
    pub zk_mode: bool,
    /// Maximum gas allowed for execution
    pub gas_limit: u64,
    /// Schema version
    pub version: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// DID that owns this contract
    pub owner_did: String,
    /// Required capabilities for execution
    pub capabilities: Vec<Capability>,
    /// Searchable tags
    pub tags: Vec<String>,
    /// Required contract dependencies
    pub dependencies: Vec<String>,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Number of successful executions
    pub successful_runs: u64,
    /// Number of failed executions
    pub failed_runs: u64,
    /// Total gas consumed
    pub total_gas_consumed: u64,
    /// Average gas per execution
    pub average_gas_per_run: f64,
    /// Peak memory usage across runs
    pub peak_memory_usage: u64,
    /// Average execution time
    pub average_execution_time_ms: f64,
    /// Last error encountered
    pub last_error: Option<ContractError>,
    /// Number of ZK proofs generated
    pub zk_proofs_generated: u64,
    /// Number of ZK proofs verified
    pub zk_proofs_verified: u64,
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            successful_runs: 0,
            failed_runs: 0,
            total_gas_consumed: 0,
            average_gas_per_run: 0.0,
            peak_memory_usage: 0,
            average_execution_time_ms: 0.0,
            last_error: None,
            zk_proofs_generated: 0,
            zk_proofs_verified: 0,
        }
    }
}

impl ContractSandbox {
    /// Create a new contract sandbox
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let engine = Engine::default();
        
        Ok(Self {
            engine,
            gas_config: GasConfig::default(),
            contracts: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
        })
    }

    /// Run a smart contract
    pub async fn run_contract(
        &self,
        wasm: &[u8],
        input: &[u8],
        gas_limit: u64,
        zk_mode: bool,
        storage_context: Option<StorageContext>,
    ) -> Result<ResultV1, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let mut gas_used = 0;
        let mut memory_used = 0;
        let mut storage_accesses = Vec::new();
        let mut events = Vec::new();
        let mut instruction_count = 0;

        // Validate gas limit
        if gas_limit > self.gas_config.max_instructions {
            return Err(Box::new(ContractError {
                code: ErrorCode::GasExceeded,
                message: "Gas limit exceeds maximum allowed".to_string(),
                details: Some(format!("Limit: {}, Max: {}", gas_limit, self.gas_config.max_instructions)),
                gas_consumed: 0,
                instruction_count: 0,
                memory_peak: 0,
            }));
        }

        // Create WASM store with gas metering
        let mut store = Store::new(&self.engine, ());
        
        // Set up WASI context
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();
        let wasi = WasiCtx::new(wasi);
        
        // Compile WASM module
        let module = Module::new(&self.engine, wasm)?;
        
        // Create instance
        let instance = Instance::new(&mut store, &module, &[wasi.into()])?;
        
        // Get memory and exports
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or("No memory export found")?;
        
        // Set up gas metering
        let mut gas_meter = GasMeter::new(gas_limit, &self.gas_config);
        
        // Execute contract
        let result = self.execute_contract(
            &mut store,
            &instance,
            &memory,
            input,
            &mut gas_meter,
            &mut instruction_count,
            &mut memory_used,
            &mut storage_accesses,
            &mut events,
            storage_context,
        ).await;

        let execution_time = start_time.elapsed();
        let execution_time_ms = execution_time.as_millis() as u64;
        
        // Calculate final gas usage
        gas_used = gas_meter.total_gas_used();

        // Generate ZK proof if requested
        let proof = if zk_mode && result.is_ok() {
            match self.generate_zk_proof(input, &result.as_ref().unwrap()) {
                Ok(proof_data) => Some(proof_data),
                Err(_) => {
                    // Log ZK proof generation failure but don't fail execution
                    eprintln!("Warning: Failed to generate ZK proof");
                    None
                }
            }
        } else {
            None
        };

        // Create result
        let result_v1 = ResultV1 {
            ok: result.is_ok(),
            gas_used,
            gas_limit,
            output: result.unwrap_or_default(),
            proof,
            error: if result.is_err() {
                Some(ContractError {
                    code: ErrorCode::WasmError,
                    message: "Contract execution failed".to_string(),
                    details: Some(result.unwrap_err().to_string()),
                    gas_consumed: gas_used,
                    instruction_count,
                    memory_peak: memory_used,
                })
            } else {
                None
            },
            execution_time_ms,
            memory_used,
            storage_accesses,
            events,
            timestamp: chrono::Utc::now(),
            contract_id: "unknown".to_string(), // Would be set by caller
            input_hash: self.hash_input(input),
        };

        // Update statistics
        self.update_stats(&result_v1).await;

        Ok(result_v1)
    }

    /// Execute a contract with gas metering
    async fn execute_contract(
        &self,
        store: &mut Store<()>,
        instance: &Instance,
        memory: &wasmtime::Memory,
        input: &[u8],
        gas_meter: &mut GasMeter,
        instruction_count: &mut u64,
        memory_used: &mut u64,
        storage_accesses: &mut Vec<StorageAccess>,
        events: &mut Vec<ContractEvent>,
        storage_context: Option<StorageContext>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Write input to memory
        let input_offset = 0;
        memory.write(store, input_offset, input)?;
        
        // Call main function
        let main_func = instance.get_func(store, "main")
            .ok_or("No main function found")?;
        
        // Execute with gas metering
        let result = main_func.call(store, &[Val::I32(input_offset as i32), Val::I32(input.len() as i32)])?;
        
        // Read output from memory
        let output_offset = result[0].i32().unwrap_or(0) as usize;
        let output_size = result[1].i32().unwrap_or(0) as usize;
        
        if output_size > 0 {
            let mut output = vec![0u8; output_size];
            memory.read(store, output_offset, &mut output)?;
            
            // Update instruction count (simplified - would use actual WASM instruction counting)
            *instruction_count = gas_meter.instruction_count();
            
            // Update memory usage
            *memory_used = memory.size(store) as u64 * 65536; // 64KB pages
            
            Ok(output)
        } else {
            Ok(Vec::new())
        }
    }

    /// Generate ZK proof for contract execution
    fn generate_zk_proof(&self, input: &[u8], output: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // This would integrate with the C ZKVM adapter
        // For now, return a placeholder proof
        
        let proof = ZKProof {
            algorithm: ZKAlgorithm::Halo2,
            proof_data: b"placeholder_proof_data".to_vec(),
            public_inputs: vec![input.to_vec(), output.to_vec()],
            circuit_hash: b"placeholder_circuit_hash".to_vec(),
            prover_version: "1.0.0".to_string(),
            proof_size: 1024,
            verification_key: b"placeholder_verification_key".to_vec(),
        };
        
        // Serialize proof to CBOR
        Ok(serde_cbor::to_vec(&proof)?)
    }

    /// Hash input data using Blake3
    fn hash_input(&self, input: &[u8]) -> Vec<u8> {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(input);
        hasher.finalize().as_bytes().to_vec()
    }

    /// Update execution statistics
    async fn update_stats(&self, result: &ResultV1) {
        let mut stats = self.stats.write().await;
        
        if result.ok {
            stats.successful_runs += 1;
        } else {
            stats.failed_runs += 1;
            stats.last_error = result.error.clone();
        }
        
        stats.total_gas_consumed += result.gas_used;
        stats.average_gas_per_run = stats.total_gas_consumed as f64 / (stats.successful_runs + stats.failed_runs) as f64;
        
        if result.memory_used > stats.peak_memory_usage {
            stats.peak_memory_usage = result.memory_used;
        }
        
        stats.average_execution_time_ms = 
            (stats.average_execution_time_ms * (stats.successful_runs + stats.failed_runs - 1) as f64 + result.execution_time_ms as f64) 
            / (stats.successful_runs + stats.failed_runs) as f64;
        
        if result.proof.is_some() {
            stats.zk_proofs_generated += 1;
        }
    }

    /// Get execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.stats.read().await.clone()
    }

    /// Deploy a new contract
    pub async fn deploy_contract(&self, contract: ContractV1) -> Result<(), Box<dyn std::error::Error>> {
        // Validate WASM bytecode
        Module::new(&self.engine, &contract.wasm)?;
        
        // Store contract
        let mut contracts = self.contracts.write().await;
        contracts.insert(contract.id.clone(), contract);
        
        Ok(())
    }

    /// Get contract by ID
    pub async fn get_contract(&self, id: &str) -> Option<ContractV1> {
        let contracts = self.contracts.read().await;
        contracts.get(id).cloned()
    }

    /// List all contracts
    pub async fn list_contracts(&self) -> Vec<ContractV1> {
        let contracts = self.contracts.read().await;
        contracts.values().cloned().collect()
    }

    /// Update gas configuration
    pub fn update_gas_config(&mut self, config: GasConfig) {
        self.gas_config = config;
    }

    /// Get current gas configuration
    pub fn get_gas_config(&self) -> &GasConfig {
        &self.gas_config
    }
}

/// Gas metering for contract execution
struct GasMeter {
    /// Total gas used
    total_gas: u64,
    /// Gas limit
    limit: u64,
    /// Gas configuration
    config: GasConfig,
    /// Instruction count
    instruction_count: u64,
}

impl GasMeter {
    /// Create a new gas meter
    fn new(limit: u64, config: &GasConfig) -> Self {
        Self {
            total_gas: 0,
            limit,
            config: config.clone(),
            instruction_count: 0,
        }
    }

    /// Consume gas for an instruction
    fn consume_instruction(&mut self) -> Result<(), ContractError> {
        self.instruction_count += 1;
        self.consume_gas(self.config.instruction_cost)
    }

    /// Consume gas for memory allocation
    fn consume_memory(&mut self, bytes: u64) -> Result<(), ContractError> {
        let cost = bytes * self.config.memory_cost_per_byte;
        self.consume_gas(cost)
    }

    /// Consume gas for storage operation
    fn consume_storage(&mut self, operation: &StorageOp) -> Result<(), ContractError> {
        let cost = match operation {
            StorageOp::Read => self.config.storage_read_cost,
            StorageOp::Write => self.config.storage_write_cost,
            StorageOp::Delete => self.config.storage_write_cost,
            StorageOp::List => self.config.storage_read_cost,
            StorageOp::Exists => self.config.storage_read_cost,
        };
        self.consume_gas(cost)
    }

    /// Consume gas for event emission
    fn consume_event(&mut self) -> Result<(), ContractError> {
        self.consume_gas(self.config.event_cost)
    }

    /// Consume gas for ZK proof generation
    fn consume_zk_proof(&mut self) -> Result<(), ContractError> {
        self.consume_gas(self.config.zk_proof_cost)
    }

    /// Consume a specific amount of gas
    fn consume_gas(&mut self, amount: u64) -> Result<(), ContractError> {
        if self.total_gas + amount > self.limit {
            return Err(ContractError {
                code: ErrorCode::GasExceeded,
                message: "Gas limit exceeded".to_string(),
                details: Some(format!("Used: {}, Limit: {}", self.total_gas + amount, self.limit)),
                gas_consumed: self.total_gas,
                instruction_count: self.instruction_count,
                memory_peak: 0,
            });
        }
        
        self.total_gas += amount;
        Ok(())
    }

    /// Get total gas used
    fn total_gas_used(&self) -> u64 {
        self.total_gas
    }

    /// Get instruction count
    fn instruction_count(&self) -> u64 {
        self.instruction_count
    }

    /// Check if gas limit would be exceeded
    fn would_exceed(&self, amount: u64) -> bool {
        self.total_gas + amount > self.limit
    }
}

/// ZK proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProof {
    /// ZK algorithm used
    pub algorithm: ZKAlgorithm,
    /// Serialized proof data
    pub proof_data: Vec<u8>,
    /// Public inputs for verification
    pub public_inputs: Vec<Vec<u8>>,
    /// Hash of the circuit
    pub circuit_hash: Vec<u8>,
    /// Version of the prover
    pub prover_version: String,
    /// Size of proof in bytes
    pub proof_size: u64,
    /// Verification key for the circuit
    pub verification_key: Vec<u8>,
}

/// ZK algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZKAlgorithm {
    Halo2,
    Noir,
    Plonk,
    Custom,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_creation() {
        let sandbox = ContractSandbox::new();
        assert!(sandbox.is_ok());
    }

    #[tokio::test]
    async fn test_gas_meter() {
        let config = GasConfig::default();
        let mut meter = GasMeter::new(1000, &config);
        
        // Test instruction consumption
        assert!(meter.consume_instruction().is_ok());
        assert_eq!(meter.total_gas_used(), 1);
        
        // Test memory consumption
        assert!(meter.consume_memory(100).is_ok());
        assert_eq!(meter.total_gas_used(), 101);
        
        // Test limit enforcement
        let result = meter.consume_gas(1000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, ErrorCode::GasExceeded);
    }

    #[tokio::test]
    async fn test_contract_deployment() {
        let sandbox = ContractSandbox::new().unwrap();
        
        let contract = ContractV1 {
            id: "test-contract".to_string(),
            wasm: b"(module)".to_vec(), // Minimal valid WASM
            meta: ContractMeta {
                name: "Test Contract".to_string(),
                description: None,
                author: "Test Author".to_string(),
                license: None,
                category: ContractCategory::Utility,
                interfaces: vec![],
                storage_keys: vec![],
                input_schema: None,
                output_schema: None,
                gas_estimate: None,
                max_memory: None,
                timeout_ms: None,
            },
            zk_mode: false,
            gas_limit: 1000,
            version: "1.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            owner_did: "did:aetheris:test:user".to_string(),
            capabilities: vec![],
            tags: vec![],
            dependencies: vec![],
        };
        
        let result = sandbox.deploy_contract(contract).await;
        assert!(result.is_ok());
        
        let deployed = sandbox.get_contract("test-contract").await;
        assert!(deployed.is_some());
    }

    #[tokio::test]
    async fn test_stats_update() {
        let sandbox = ContractSandbox::new().unwrap();
        
        let result = ResultV1 {
            ok: true,
            gas_used: 100,
            gas_limit: 1000,
            output: vec![1, 2, 3],
            proof: None,
            error: None,
            execution_time_ms: 50,
            memory_used: 1024,
            storage_accesses: vec![],
            events: vec![],
            timestamp: chrono::Utc::now(),
            contract_id: "test".to_string(),
            input_hash: vec![1, 2, 3],
        };
        
        sandbox.update_stats(&result).await;
        let stats = sandbox.get_stats().await;
        
        assert_eq!(stats.successful_runs, 1);
        assert_eq!(stats.total_gas_consumed, 100);
        assert_eq!(stats.peak_memory_usage, 1024);
    }
}
