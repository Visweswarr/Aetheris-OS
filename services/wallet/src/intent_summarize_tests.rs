//! Snapshot Tests for Intent Summarizer
//!
//! Tests that verify the natural language output of the intent summarizer
//! by comparing generated summaries against expected snapshots.

#[cfg(test)]
mod tests {
    use super::super::intent_summarize::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use serde_json;

    /// Create a test USDC token
    fn create_usdc_token() -> TokenInfo {
        TokenInfo {
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            contract_address: Some("0xA0b86a33E6441D5F113C8f5C4Ba79A1a5c02CE5F".to_string()),
            chain_id: Some(1),
            logo_url: None,
        }
    }

    /// Create a test ETH token
    fn create_eth_token() -> TokenInfo {
        TokenInfo {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
            contract_address: None,
            chain_id: Some(1),
            logo_url: None,
        }
    }

    /// Create a test network (Ethereum mainnet)
    fn create_mainnet() -> NetworkInfo {
        NetworkInfo {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            currency: "ETH".to_string(),
            block_explorer_url: Some("https://etherscan.io".to_string()),
        }
    }

    /// Create a test gas estimate
    fn create_gas_estimate() -> GasEstimate {
        GasEstimate {
            gas_limit: 21000,
            gas_price: "20000000000".to_string(), // 20 gwei
            total_fee: "0.00042".to_string(),
            fee_token: create_eth_token(),
            usd_value: Some("1.05".to_string()),
        }
    }

    /// Test snapshot: Simple USDC transfer
    #[test]
    fn test_snapshot_simple_transfer() {
        let summarizer = IntentSummarizer::new();
        
        let operation = IntentOperation::Transfer {
            from: "0x1234567890123456789012345678901234567890".to_string(),
            to: "0x0987654321098765432109876543210987654321".to_string(),
            amount: "1000.50".to_string(),
            token: create_usdc_token(),
            memo: Some("Payment for services".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: Some(create_gas_estimate()),
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(42),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Send 1000.50 USDC to 0x0987...4321 with memo: 'Payment for services'");
        assert_eq!(summary.details.len(), 1);
        assert_eq!(summary.details[0], "Transfer 1000.50 USDC from 0x1234...7890 to 0x0987...4321");
        assert_eq!(summary.risk_level, RiskLevel::Low);
        assert!(summary.confidence > 0.9);
        assert_eq!(summary.warnings.len(), 0);
        assert_eq!(summary.network_info, "Network: Ethereum Mainnet (Chain ID: 1)");
        assert_eq!(summary.fee_summary, Some("Estimated fee: 0.00042 ETH (~$1.05)".to_string()));
        
        println!("✅ Simple Transfer Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Details: {:?}", summary.details);
        println!("Risk: {:?}, Confidence: {:.2}", summary.risk_level, summary.confidence);
    }

    /// Test snapshot: High-value transfer with warning
    #[test]
    fn test_snapshot_high_value_transfer() {
        let summarizer = IntentSummarizer::new();
        
        let operation = IntentOperation::Transfer {
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: "0x2222222222222222222222222222222222222222".to_string(),
            amount: "50000.0".to_string(), // High value
            token: create_usdc_token(),
            memo: None,
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: Some(create_gas_estimate()),
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(43),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Send 50000.00 USDC to 0x2222...2222");
        assert!(summary.warnings.len() > 0);
        assert!(summary.warnings[0].contains("High-value transfer"));
        assert_eq!(summary.risk_level, RiskLevel::Low); // Still low for simple transfer
        
        println!("✅ High-Value Transfer Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Warnings: {:?}", summary.warnings);
        println!("Risk: {:?}", summary.risk_level);
    }

    /// Test snapshot: Token swap on Uniswap
    #[test]
    fn test_snapshot_token_swap() {
        let summarizer = IntentSummarizer::new();
        
        let operation = IntentOperation::Swap {
            from_token: create_usdc_token(),
            to_token: create_eth_token(),
            from_amount: "1000.0".to_string(),
            min_to_amount: "0.5".to_string(),
            slippage_tolerance: Some("2.0".to_string()),
            dex: Some("Uniswap".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: Some(create_gas_estimate()),
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(44),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Swap 1000.00 USDC for at least 0.50 ETH on Uniswap (max slippage: 2.0%)");
        assert_eq!(summary.details.len(), 1);
        assert!(summary.details[0].contains("You will give: 1000.00 USDC"));
        assert!(summary.details[0].contains("You will receive: at least 0.50 ETH"));
        assert!(summary.details[0].contains("Maximum slippage: 2.0%"));
        assert!(summary.details[0].contains("Exchange: Uniswap"));
        assert_eq!(summary.risk_level, RiskLevel::Medium);
        assert!(summary.confidence > 0.85);
        
        println!("✅ Token Swap Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Details: {:?}", summary.details);
        println!("Risk: {:?}, Confidence: {:.2}", summary.risk_level, summary.confidence);
    }

    /// Test snapshot: Smart contract call (unverified)
    #[test]
    fn test_snapshot_unverified_contract_call() {
        let summarizer = IntentSummarizer::new();
        
        let operation = IntentOperation::ContractCall {
            contract_address: "0xUnknownContract123456789012345678901234567890".to_string(),
            function_name: "claimRewards".to_string(),
            parameters: vec![
                ContractParameter {
                    name: "amount".to_string(),
                    param_type: "uint256".to_string(),
                    value: serde_json::Value::String("1000000000000000000".to_string()),
                }
            ],
            value: Some("0.1".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: Some(create_gas_estimate()),
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(45),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Call claimRewards function on 0xUnkn...7890 with 0.10 ETH");
        assert!(summary.warnings.len() > 0);
        assert!(summary.warnings.iter().any(|w| w.contains("unverified contract")));
        assert_eq!(summary.risk_level, RiskLevel::High);
        assert!(summary.confidence < 0.7);
        
        println!("✅ Unverified Contract Call Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Warnings: {:?}", summary.warnings);
        println!("Risk: {:?}, Confidence: {:.2}", summary.risk_level, summary.confidence);
    }

    /// Test snapshot: Multi-signature transaction
    #[test]
    fn test_snapshot_multisig_transaction() {
        let summarizer = IntentSummarizer::new();
        
        let inner_operation = IntentOperation::Transfer {
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: "0x2222222222222222222222222222222222222222".to_string(),
            amount: "500.0".to_string(),
            token: create_usdc_token(),
            memo: None,
        };
        
        let operation = IntentOperation::MultiSig {
            multisig_address: "0x3333333333333333333333333333333333333333".to_string(),
            operations: vec![inner_operation],
            required_signatures: 3,
            current_signatures: 2,
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: Some(create_gas_estimate()),
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(46),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Execute 3-of-2 multisig transaction on 0x3333...3333 (1 operations)");
        assert_eq!(summary.details.len(), 1);
        assert!(summary.details[0].contains("Multisig wallet: 0x3333...3333"));
        assert!(summary.details[0].contains("requires 3/2 signatures"));
        assert!(summary.details[0].contains("contains 1 operations"));
        assert_eq!(summary.risk_level, RiskLevel::Medium);
        
        println!("✅ MultiSig Transaction Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Details: {:?}", summary.details);
        println!("Risk: {:?}", summary.risk_level);
    }

    /// Test snapshot: Staking operation
    #[test]
    fn test_snapshot_staking_operation() {
        let summarizer = IntentSummarizer::new();
        
        let operation = IntentOperation::Stake {
            validator: "0x4444444444444444444444444444444444444444".to_string(),
            amount: "32.0".to_string(),
            token: create_eth_token(),
            lock_period: Some("365 days".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: Some(create_gas_estimate()),
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(47),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Stake 32.00 ETH with validator 0x4444...4444 (locked for 365 days)");
        assert_eq!(summary.details.len(), 1);
        assert!(summary.details[0].contains("Amount: 32.00 ETH"));
        assert!(summary.details[0].contains("Validator: 0x4444...4444"));
        assert!(summary.details[0].contains("Lock period: 365 days"));
        assert_eq!(summary.risk_level, RiskLevel::Medium);
        
        println!("✅ Staking Operation Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Details: {:?}", summary.details);
        println!("Risk: {:?}", summary.risk_level);
    }

    /// Test snapshot: DeFi operation on known protocol
    #[test]
    fn test_snapshot_defi_operation() {
        let summarizer = IntentSummarizer::new();
        
        let token_amount = TokenAmount {
            token: create_usdc_token(),
            amount: "1000.0".to_string(),
            usd_value: Some("1000.00".to_string()),
        };
        
        let operation = IntentOperation::DeFiOperation {
            protocol: "uniswap".to_string(),
            operation_type: "Add Liquidity".to_string(),
            tokens: vec![token_amount],
            parameters: HashMap::new(),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: Some(create_gas_estimate()),
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(48),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Add Liquidity 1000.00 USDC with Uniswap");
        assert_eq!(summary.details.len(), 1);
        assert!(summary.details[0].contains("Protocol: Uniswap"));
        assert!(summary.details[0].contains("Operation: Add Liquidity"));
        assert!(summary.details[0].contains("Tokens: 1000.00 USDC"));
        assert_eq!(summary.risk_level, RiskLevel::Medium);
        
        println!("✅ DeFi Operation Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Details: {:?}", summary.details);
        println!("Risk: {:?}", summary.risk_level);
    }

    /// Test snapshot: Multiple operations
    #[test]
    fn test_snapshot_multiple_operations() {
        let summarizer = IntentSummarizer::new();
        
        let transfer_op = IntentOperation::Transfer {
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: "0x2222222222222222222222222222222222222222".to_string(),
            amount: "100.0".to_string(),
            token: create_usdc_token(),
            memo: None,
        };
        
        let swap_op = IntentOperation::Swap {
            from_token: create_usdc_token(),
            to_token: create_eth_token(),
            from_amount: "500.0".to_string(),
            min_to_amount: "0.25".to_string(),
            slippage_tolerance: Some("1.0".to_string()),
            dex: Some("Uniswap".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![transfer_op, swap_op],
            gas_estimate: Some(create_gas_estimate()),
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(49),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Execute 2 operations: Transfer, Swap");
        assert_eq!(summary.details.len(), 2);
        assert!(summary.details[0].contains("1. Transfer 100.00 USDC"));
        assert!(summary.details[1].contains("2. You will give: 500.00 USDC"));
        assert_eq!(summary.risk_level, RiskLevel::Medium); // Higher risk due to complexity
        
        println!("✅ Multiple Operations Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Details: {:?}", summary.details);
        println!("Risk: {:?}", summary.risk_level);
    }

    /// Test snapshot: Testnet transaction warning
    #[test]
    fn test_snapshot_testnet_warning() {
        let summarizer = IntentSummarizer::new();
        
        let operation = IntentOperation::Transfer {
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: "0x2222222222222222222222222222222222222222".to_string(),
            amount: "10.0".to_string(),
            token: create_usdc_token(),
            memo: None,
        };
        
        let testnet = NetworkInfo {
            chain_id: 5, // Goerli testnet
            name: "Goerli Testnet".to_string(),
            currency: "ETH".to_string(),
            block_explorer_url: Some("https://goerli.etherscan.io".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: Some(create_gas_estimate()),
            network: testnet,
            timestamp: Utc::now(),
            nonce: Some(50),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        
        // Snapshot assertions
        assert_eq!(summary.primary_action, "Send 10.00 USDC to 0x2222...2222");
        assert!(summary.warnings.len() > 0);
        assert!(summary.warnings.iter().any(|w| w.contains("test network")));
        assert_eq!(summary.network_info, "Network: Goerli Testnet (Chain ID: 5)");
        
        println!("✅ Testnet Warning Snapshot:");
        println!("Primary: {}", summary.primary_action);
        println!("Warnings: {:?}", summary.warnings);
        println!("Network: {}", summary.network_info);
    }

    /// Test snapshot: Confidence calculation
    #[test]
    fn test_snapshot_confidence_levels() {
        let summarizer = IntentSummarizer::new();
        
        // Known contract (high confidence)
        let known_contract = IntentOperation::ContractCall {
            contract_address: "0xA0b86a33E6441D5F113C8f5C4Ba79A1a5c02CE5F".to_string(), // USDC contract
            function_name: "transfer".to_string(),
            parameters: vec![],
            value: None,
        };
        
        let intent1 = TransactionIntent {
            operations: vec![known_contract],
            gas_estimate: None,
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(51),
            metadata: HashMap::new(),
        };
        
        let summary1 = summarizer.summarize(&intent1).unwrap();
        
        // Unknown contract (lower confidence)
        let unknown_contract = IntentOperation::ContractCall {
            contract_address: "0xUnknownContract123456789012345678901234567890".to_string(),
            function_name: "unknownFunction".to_string(),
            parameters: vec![],
            value: None,
        };
        
        let intent2 = TransactionIntent {
            operations: vec![unknown_contract],
            gas_estimate: None,
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(52),
            metadata: HashMap::new(),
        };
        
        let summary2 = summarizer.summarize(&intent2).unwrap();
        
        // Confidence assertions
        assert!(summary1.confidence > summary2.confidence);
        assert!(summary1.confidence > 0.8); // Known contract should have high confidence
        assert!(summary2.confidence < 0.7); // Unknown contract should have lower confidence
        
        println!("✅ Confidence Levels Snapshot:");
        println!("Known contract confidence: {:.2}", summary1.confidence);
        println!("Unknown contract confidence: {:.2}", summary2.confidence);
    }

    /// Test snapshot: Risk level progression
    #[test]
    fn test_snapshot_risk_levels() {
        let summarizer = IntentSummarizer::new();
        
        // Low risk: Simple transfer
        let low_risk = IntentOperation::Transfer {
            from: "0x1111111111111111111111111111111111111111".to_string(),
            to: "0x2222222222222222222222222222222222222222".to_string(),
            amount: "10.0".to_string(),
            token: create_usdc_token(),
            memo: None,
        };
        
        // Medium risk: Token swap
        let medium_risk = IntentOperation::Swap {
            from_token: create_usdc_token(),
            to_token: create_eth_token(),
            from_amount: "100.0".to_string(),
            min_to_amount: "0.05".to_string(),
            slippage_tolerance: None,
            dex: None,
        };
        
        // High risk: Unknown contract call
        let high_risk = IntentOperation::ContractCall {
            contract_address: "0xUnknownContract123456789012345678901234567890".to_string(),
            function_name: "suspiciousFunction".to_string(),
            parameters: vec![],
            value: Some("1.0".to_string()),
        };
        
        let scenarios = vec![
            ("Low Risk", low_risk, RiskLevel::Low),
            ("Medium Risk", medium_risk, RiskLevel::Medium),
            ("High Risk", high_risk, RiskLevel::High),
        ];
        
        for (name, operation, expected_risk) in scenarios {
            let intent = TransactionIntent {
                operations: vec![operation],
                gas_estimate: None,
                network: create_mainnet(),
                timestamp: Utc::now(),
                nonce: Some(53),
                metadata: HashMap::new(),
            };
            
            let summary = summarizer.summarize(&intent).unwrap();
            assert_eq!(summary.risk_level, expected_risk);
            
            println!("✅ {} Snapshot: {:?}", name, summary.risk_level);
        }
    }

    /// Test snapshot: Error handling for invalid input
    #[test]
    fn test_snapshot_error_handling() {
        let summarizer = IntentSummarizer::new();
        
        // Empty operations
        let intent = TransactionIntent {
            operations: vec![],
            gas_estimate: None,
            network: create_mainnet(),
            timestamp: Utc::now(),
            nonce: Some(54),
            metadata: HashMap::new(),
        };
        
        let result = summarizer.summarize(&intent);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            IntentSummaryError::InvalidIntent(msg) => {
                assert_eq!(msg, "Intent contains no operations");
                println!("✅ Error Handling Snapshot: {}", msg);
            },
            _ => panic!("Expected InvalidIntent error"),
        }
    }

    /// Helper function to run all snapshot tests
    #[test]
    fn run_all_snapshots() {
        println!("\n🔍 Running Intent Summarizer Snapshot Tests\n");
        
        test_snapshot_simple_transfer();
        test_snapshot_high_value_transfer();
        test_snapshot_token_swap();
        test_snapshot_unverified_contract_call();
        test_snapshot_multisig_transaction();
        test_snapshot_staking_operation();
        test_snapshot_defi_operation();
        test_snapshot_multiple_operations();
        test_snapshot_testnet_warning();
        test_snapshot_confidence_levels();
        test_snapshot_risk_levels();
        test_snapshot_error_handling();
        
        println!("\n✅ All snapshot tests completed successfully!");
    }
}
