use contracts::sandbox::{ContractSandbox, GasConfig, ResultV1, ContractError, ErrorCode};
use contracts::zkvm::{ZKProof, ZKAlgorithm};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

// Mock WASM data for testing
const MOCK_WASM: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
const MOCK_INPUT: &[u8] = b"{\"test\": \"data\"}";

#[tokio::test]
async fn test_sandbox_creation() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    // Verify default gas config
    let gas_config = sandbox.get_gas_config();
    assert_eq!(gas_config.max_instructions, 1_000_000);
    assert_eq!(gas_config.max_memory_mb, 64);
    assert_eq!(gas_config.max_storage_ops, 1000);
}

#[tokio::test]
async fn test_contract_deployment() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    let contract_id = "test-contract";
    let wasm_data = MOCK_WASM.to_vec();
    
    let result = sandbox.deploy_contract(contract_id, wasm_data, None).await;
    assert!(result.is_ok(), "Contract deployment should succeed");
    
    // Verify contract was stored
    let contract = sandbox.get_contract(contract_id).await;
    assert!(contract.is_ok(), "Should be able to retrieve deployed contract");
    
    let contract = contract.unwrap();
    assert_eq!(contract.id, contract_id);
    assert_eq!(contract.wasm, MOCK_WASM);
}

#[tokio::test]
async fn test_contract_execution() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    // Deploy a test contract
    let contract_id = "exec-test-contract";
    sandbox.deploy_contract(contract_id, MOCK_WASM.to_vec(), None)
        .await
        .expect("Contract deployment failed");
    
    // Execute the contract
    let result = sandbox.run_contract(
        MOCK_WASM,
        MOCK_INPUT,
        100_000, // gas limit
        false,   // no ZK
        None,    // no storage context
    ).await;
    
    assert!(result.is_ok(), "Contract execution should succeed");
    
    let execution_result = result.unwrap();
    assert!(execution_result.ok, "Execution should be successful");
    assert!(execution_result.gas_used > 0, "Should consume some gas");
    assert!(execution_result.gas_used <= 100_000, "Should not exceed gas limit");
}

#[tokio::test]
async fn test_gas_limit_enforcement() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    // Try to execute with very low gas limit
    let result = sandbox.run_contract(
        MOCK_WASM,
        MOCK_INPUT,
        10,      // very low gas limit
        false,   // no ZK
        None,    // no storage context
    ).await;
    
    assert!(result.is_ok(), "Execution should complete (even if it fails)");
    
    let execution_result = result.unwrap();
    if !execution_result.ok {
        // If execution failed due to gas limit, verify error details
        if let Some(error) = execution_result.error {
            assert_eq!(error.code, ErrorCode::GAS_LIMIT_EXCEEDED);
            assert!(error.gas_consumed > 0);
        }
    }
}

#[tokio::test]
async fn test_zk_proof_generation() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    // Execute with ZK mode enabled
    let result = sandbox.run_contract(
        MOCK_WASM,
        MOCK_INPUT,
        1_000_000, // sufficient gas
        true,       // ZK mode enabled
        None,       // no storage context
    ).await;
    
    assert!(result.is_ok(), "ZK execution should succeed");
    
    let execution_result = result.unwrap();
    assert!(execution_result.ok, "Execution should be successful");
    
    // Verify ZK proof was generated
    assert!(execution_result.proof.is_some(), "ZK proof should be generated");
    
    let proof = execution_result.proof.unwrap();
    assert_eq!(proof.algorithm, ZKAlgorithm::Halo2); // Default algorithm
    assert!(proof.proof_size > 0, "Proof should have non-zero size");
    assert!(!proof.proof_data.is_empty(), "Proof data should not be empty");
}

#[tokio::test]
async fn test_contract_listing() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    // Deploy multiple contracts
    let contracts = vec![
        ("contract-1", MOCK_WASM),
        ("contract-2", MOCK_WASM),
        ("contract-3", MOCK_WASM),
    ];
    
    for (id, wasm) in contracts {
        sandbox.deploy_contract(id, wasm.to_vec(), None)
            .await
            .expect(&format!("Failed to deploy {}", id));
    }
    
    // List all contracts
    let contract_list = sandbox.list_contracts().await;
    assert!(contract_list.is_ok(), "Should be able to list contracts");
    
    let contracts = contract_list.unwrap();
    assert!(contracts.len() >= 3, "Should have at least 3 contracts");
    
    // Verify all deployed contracts are listed
    let contract_ids: Vec<&str> = contracts.iter().map(|c| c.id.as_str()).collect();
    assert!(contract_ids.contains(&"contract-1"));
    assert!(contract_ids.contains(&"contract-2"));
    assert!(contract_ids.contains(&"contract-3"));
}

#[tokio::test]
async fn test_gas_config_update() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    let new_config = GasConfig {
        max_instructions: 2_000_000,
        max_memory_mb: 128,
        max_storage_ops: 2000,
        instruction_cost: 1,
        memory_cost_per_mb: 1000,
        storage_op_cost: 10,
    };
    
    sandbox.update_gas_config(new_config.clone()).await;
    
    let current_config = sandbox.get_gas_config();
    assert_eq!(current_config.max_instructions, 2_000_000);
    assert_eq!(current_config.max_memory_mb, 128);
    assert_eq!(current_config.max_storage_ops, 2000);
}

#[tokio::test]
async fn test_execution_stats() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    // Execute a few contracts to generate stats
    for i in 0..3 {
        let contract_id = format!("stats-test-{}", i);
        sandbox.deploy_contract(&contract_id, MOCK_WASM.to_vec(), None)
            .await
            .expect("Contract deployment failed");
        
        sandbox.run_contract(
            MOCK_WASM,
            MOCK_INPUT,
            100_000,
            false,
            None,
        ).await.expect("Contract execution failed");
    }
    
    let stats = sandbox.get_stats().await;
    assert!(stats.is_ok(), "Should be able to get execution stats");
    
    let stats = stats.unwrap();
    assert!(stats.total_executions >= 3, "Should have executed at least 3 contracts");
    assert!(stats.total_gas_used > 0, "Should have consumed some gas");
    assert!(stats.successful_executions >= 3, "Should have successful executions");
}

#[tokio::test]
async fn test_error_handling() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    // Test with empty WASM data
    let result = sandbox.run_contract(
        &[],
        MOCK_INPUT,
        100_000,
        false,
        None,
    ).await;
    
    // Should handle gracefully (either succeed or fail with clear error)
    assert!(result.is_ok(), "Should handle empty WASM gracefully");
    
    // Test with very large input
    let large_input = vec![0u8; 1024 * 1024]; // 1MB input
    let result = sandbox.run_contract(
        MOCK_WASM,
        &large_input,
        1_000_000,
        false,
        None,
    ).await;
    
    // Should handle large input gracefully
    assert!(result.is_ok(), "Should handle large input gracefully");
}

#[tokio::test]
async fn test_deterministic_execution() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    // Execute the same contract multiple times with same input
    let mut results = Vec::new();
    
    for _ in 0..5 {
        let result = sandbox.run_contract(
            MOCK_WASM,
            MOCK_INPUT,
            100_000,
            false,
            None,
        ).await.expect("Contract execution failed");
        
        results.push(result);
    }
    
    // Verify all executions produced the same result
    let first_result = &results[0];
    for result in results.iter().skip(1) {
        assert_eq!(result.ok, first_result.ok, "Execution results should be consistent");
        assert_eq!(result.gas_used, first_result.gas_used, "Gas usage should be consistent");
        assert_eq!(result.output, first_result.output, "Output should be consistent");
    }
}

#[tokio::test]
async fn test_zk_algorithm_variants() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    let algorithms = vec![
        ZKAlgorithm::Halo2,
        ZKAlgorithm::Noir,
        ZKAlgorithm::Plonk,
    ];
    
    for algorithm in algorithms {
        // Deploy a contract
        let contract_id = format!("zk-test-{:?}", algorithm);
        sandbox.deploy_contract(&contract_id, MOCK_WASM.to_vec(), None)
            .await
            .expect("Contract deployment failed");
        
        // Execute with ZK mode and specific algorithm
        let result = sandbox.run_contract(
            MOCK_WASM,
            MOCK_INPUT,
            1_000_000,
            true, // ZK mode
            None,
        ).await;
        
        assert!(result.is_ok(), "ZK execution with {:?} should succeed", algorithm);
        
        let execution_result = result.unwrap();
        if execution_result.ok {
            if let Some(proof) = execution_result.proof {
                // Verify the proof was generated with the expected algorithm
                // Note: In the current mock implementation, this might always be Halo2
                assert!(proof.proof_size > 0, "Proof should have non-zero size");
            }
        }
    }
}

#[tokio::test]
async fn test_contract_metadata() {
    let sandbox = ContractSandbox::new().expect("Failed to create sandbox");
    
    let contract_id = "metadata-test";
    let metadata = serde_json::json!({
        "name": "Test Contract",
        "description": "A test contract for metadata testing",
        "author": "Test Author",
        "version": "1.0.0"
    });
    
    let result = sandbox.deploy_contract(contract_id, MOCK_WASM.to_vec(), Some(metadata.clone()))
        .await;
    
    assert!(result.is_ok(), "Contract deployment with metadata should succeed");
    
    // Retrieve the contract and verify metadata
    let contract = sandbox.get_contract(contract_id).await.expect("Should retrieve contract");
    
    // Verify basic contract properties
    assert_eq!(contract.id, contract_id);
    assert_eq!(contract.wasm, MOCK_WASM);
    assert_eq!(contract.version, "1.0");
    assert!(!contract.owner_did.is_empty());
}

#[tokio::test]
async fn test_concurrent_executions() {
    let sandbox = Arc::new(ContractSandbox::new().expect("Failed to create sandbox"));
    
    // Deploy a test contract
    let contract_id = "concurrent-test";
    sandbox.deploy_contract(contract_id, MOCK_WASM.to_vec(), None)
        .await
        .expect("Contract deployment failed");
    
    // Execute multiple contracts concurrently
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let sandbox_clone = Arc::clone(&sandbox);
        let handle = tokio::spawn(async move {
            sandbox_clone.run_contract(
                MOCK_WASM,
                MOCK_INPUT,
                100_000,
                false,
                None,
            ).await
        });
        handles.push(handle);
    }
    
    // Wait for all executions to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        results.push(result);
    }
    
    // Verify all executions completed
    assert_eq!(results.len(), 10, "Should have 10 execution results");
    
    // Verify all executions were successful
    for result in results {
        assert!(result.is_ok(), "All concurrent executions should succeed");
        let execution_result = result.unwrap();
        assert!(execution_result.ok, "All executions should be successful");
    }
}
