//! Intent Summarizer
//!
//! Converts transaction intents into natural language explanations to help users
//! understand "what they are about to sign" before confirming wallet operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Errors that can occur during intent summarization
#[derive(Debug, Error)]
pub enum IntentSummaryError {
    #[error("Invalid intent structure: {0}")]
    InvalidIntent(String),

    #[error("Unsupported operation type: {0}")]
    UnsupportedOperation(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid amount format: {0}")]
    InvalidAmount(String),

    #[error("Unknown token contract: {0}")]
    UnknownToken(String),
}

/// Types of wallet operations that can be summarized
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IntentOperation {
    /// Simple token transfer
    Transfer {
        from: String,
        to: String,
        amount: String,
        token: TokenInfo,
        memo: Option<String>,
    },
    /// Token swap operation
    Swap {
        from_token: TokenInfo,
        to_token: TokenInfo,
        from_amount: String,
        min_to_amount: String,
        slippage_tolerance: Option<String>,
        dex: Option<String>,
    },
    /// Smart contract interaction
    ContractCall {
        contract_address: String,
        function_name: String,
        parameters: Vec<ContractParameter>,
        value: Option<String>,
    },
    /// Multi-signature transaction
    MultiSig {
        multisig_address: String,
        operations: Vec<IntentOperation>,
        required_signatures: u32,
        current_signatures: u32,
    },
    /// Staking operation
    Stake {
        validator: String,
        amount: String,
        token: TokenInfo,
        lock_period: Option<String>,
    },
    /// Unstaking operation
    Unstake {
        validator: String,
        amount: String,
        token: TokenInfo,
        withdrawal_delay: Option<String>,
    },
    /// DeFi protocol interaction
    DeFiOperation {
        protocol: String,
        operation_type: String,
        tokens: Vec<TokenAmount>,
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub contract_address: Option<String>,
    pub chain_id: Option<u64>,
    pub logo_url: Option<String>,
}

/// Token amount with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAmount {
    pub token: TokenInfo,
    pub amount: String,
    pub usd_value: Option<String>,
}

/// Smart contract parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractParameter {
    pub name: String,
    pub param_type: String,
    pub value: serde_json::Value,
}

/// Complete transaction intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionIntent {
    pub operations: Vec<IntentOperation>,
    pub gas_estimate: Option<GasEstimate>,
    pub network: NetworkInfo,
    pub timestamp: DateTime<Utc>,
    pub nonce: Option<u64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Gas fee estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub gas_limit: u64,
    pub gas_price: String,
    pub total_fee: String,
    pub fee_token: TokenInfo,
    pub usd_value: Option<String>,
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub chain_id: u64,
    pub name: String,
    pub currency: String,
    pub block_explorer_url: Option<String>,
}

/// Natural language summary of an intent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSummary {
    /// Primary action description (e.g., "Send 100 USDC to Alice")
    pub primary_action: String,
    /// Detailed breakdown of all operations
    pub details: Vec<String>,
    /// Risk warnings and important notices
    pub warnings: Vec<String>,
    /// Network and fee information
    pub network_info: String,
    /// Gas fee summary
    pub fee_summary: Option<String>,
    /// Overall risk level (Low, Medium, High, Critical)
    pub risk_level: RiskLevel,
    /// Confidence in the summary accuracy (0.0 - 1.0)
    pub confidence: f64,
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Intent summarizer service
pub struct IntentSummarizer {
    /// Known token registry for symbol resolution
    token_registry: HashMap<String, TokenInfo>,
    /// Known DeFi protocols for operation interpretation
    protocol_registry: HashMap<String, ProtocolInfo>,
    /// Contract ABI registry for function interpretation
    contract_registry: HashMap<String, ContractInfo>,
}

/// Protocol information for DeFi operations
#[derive(Debug, Clone)]
struct ProtocolInfo {
    name: String,
    description: String,
    risk_level: RiskLevel,
    known_functions: HashMap<String, String>,
}

/// Contract information for smart contract calls
#[derive(Debug, Clone)]
struct ContractInfo {
    name: String,
    description: String,
    verified: bool,
    functions: HashMap<String, FunctionInfo>,
}

/// Function information for contract calls
#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    description: String,
    risk_level: RiskLevel,
    parameters: Vec<ParameterInfo>,
}

/// Parameter information for function calls
#[derive(Debug, Clone)]
struct ParameterInfo {
    name: String,
    param_type: String,
    description: String,
}

impl IntentSummarizer {
    /// Create a new intent summarizer with default registries
    pub fn new() -> Self {
        Self {
            token_registry: Self::default_token_registry(),
            protocol_registry: Self::default_protocol_registry(),
            contract_registry: Self::default_contract_registry(),
        }
    }

    /// Summarize a transaction intent into natural language
    pub fn summarize(&self, intent: &TransactionIntent) -> Result<IntentSummary, IntentSummaryError> {
        if intent.operations.is_empty() {
            return Err(IntentSummaryError::InvalidIntent(
                "Intent contains no operations".to_string()
            ));
        }

        let primary_action = self.generate_primary_action(&intent.operations)?;
        let details = self.generate_operation_details(&intent.operations)?;
        let warnings = self.generate_warnings(&intent.operations, &intent.network)?;
        let network_info = self.generate_network_info(&intent.network);
        let fee_summary = intent.gas_estimate.as_ref().map(|gas| self.generate_fee_summary(gas));
        let risk_level = self.assess_risk_level(&intent.operations);
        let confidence = self.calculate_confidence(&intent.operations);

        Ok(IntentSummary {
            primary_action,
            details,
            warnings,
            network_info,
            fee_summary,
            risk_level,
            confidence,
        })
    }

    /// Generate primary action description
    fn generate_primary_action(&self, operations: &[IntentOperation]) -> Result<String, IntentSummaryError> {
        if operations.len() == 1 {
            self.summarize_single_operation(&operations[0])
        } else {
            Ok(format!(
                "Execute {} operations: {}",
                operations.len(),
                operations.iter()
                    .take(3)
                    .map(|op| self.operation_type_name(op))
                    .collect::<Vec<_>>()
                    .join(", ")
                    + if operations.len() > 3 { ", ..." } else { "" }
            ))
        }
    }

    /// Summarize a single operation
    fn summarize_single_operation(&self, operation: &IntentOperation) -> Result<String, IntentSummaryError> {
        match operation {
            IntentOperation::Transfer { from: _, to, amount, token, memo } => {
                let formatted_amount = self.format_token_amount(amount, token)?;
                let recipient = self.format_address(to);
                let memo_text = memo.as_ref().map(|m| format!(" with memo: '{}'", m)).unwrap_or_default();
                Ok(format!("Send {} to {}{}", formatted_amount, recipient, memo_text))
            },
            
            IntentOperation::Swap { from_token, to_token, from_amount, min_to_amount, slippage_tolerance, dex } => {
                let from_formatted = self.format_token_amount(from_amount, from_token)?;
                let min_to_formatted = self.format_token_amount(min_to_amount, to_token)?;
                let dex_text = dex.as_ref().map(|d| format!(" on {}", d)).unwrap_or_default();
                let slippage_text = slippage_tolerance.as_ref()
                    .map(|s| format!(" (max slippage: {}%)", s))
                    .unwrap_or_default();
                Ok(format!(
                    "Swap {} for at least {}{}{}",
                    from_formatted, min_to_formatted, dex_text, slippage_text
                ))
            },
            
            IntentOperation::ContractCall { contract_address, function_name, parameters: _, value } => {
                let contract_name = self.resolve_contract_name(contract_address);
                let value_text = value.as_ref()
                    .map(|v| format!(" with {} ETH", self.format_decimal_amount(v)?))
                    .unwrap_or_default();
                Ok(format!("Call {} function on {}{}", function_name, contract_name, value_text))
            },
            
            IntentOperation::MultiSig { multisig_address, operations, required_signatures, current_signatures } => {
                let multisig_name = self.format_address(multisig_address);
                Ok(format!(
                    "Execute {}-of-{} multisig transaction on {} ({} operations)",
                    required_signatures, current_signatures, multisig_name, operations.len()
                ))
            },
            
            IntentOperation::Stake { validator, amount, token, lock_period } => {
                let formatted_amount = self.format_token_amount(amount, token)?;
                let validator_name = self.format_address(validator);
                let lock_text = lock_period.as_ref()
                    .map(|p| format!(" (locked for {})", p))
                    .unwrap_or_default();
                Ok(format!("Stake {} with validator {}{}", formatted_amount, validator_name, lock_text))
            },
            
            IntentOperation::Unstake { validator, amount, token, withdrawal_delay } => {
                let formatted_amount = self.format_token_amount(amount, token)?;
                let validator_name = self.format_address(validator);
                let delay_text = withdrawal_delay.as_ref()
                    .map(|d| format!(" (withdrawal delay: {})", d))
                    .unwrap_or_default();
                Ok(format!("Unstake {} from validator {}{}", formatted_amount, validator_name, delay_text))
            },
            
            IntentOperation::DeFiOperation { protocol, operation_type, tokens, parameters: _ } => {
                let protocol_name = self.resolve_protocol_name(protocol);
                let token_summary = if tokens.len() == 1 {
                    self.format_token_amount(&tokens[0].amount, &tokens[0].token)?
                } else {
                    format!("{} tokens", tokens.len())
                };
                Ok(format!("{} {} with {}", operation_type, token_summary, protocol_name))
            },
        }
    }

    /// Generate detailed operation descriptions
    fn generate_operation_details(&self, operations: &[IntentOperation]) -> Result<Vec<String>, IntentSummaryError> {
        operations.iter()
            .enumerate()
            .map(|(index, op)| {
                if operations.len() > 1 {
                    Ok(format!("{}. {}", index + 1, self.describe_operation_details(op)?))
                } else {
                    self.describe_operation_details(op)
                }
            })
            .collect()
    }

    /// Describe detailed operation information
    fn describe_operation_details(&self, operation: &IntentOperation) -> Result<String, IntentSummaryError> {
        match operation {
            IntentOperation::Transfer { from, to, amount, token, memo: _ } => {
                let formatted_amount = self.format_token_amount(amount, token)?;
                let from_addr = self.format_address(from);
                let to_addr = self.format_address(to);
                Ok(format!("Transfer {} from {} to {}", formatted_amount, from_addr, to_addr))
            },
            
            IntentOperation::Swap { from_token, to_token, from_amount, min_to_amount, slippage_tolerance, dex } => {
                let from_formatted = self.format_token_amount(from_amount, from_token)?;
                let min_to_formatted = self.format_token_amount(min_to_amount, to_token)?;
                
                let mut details = vec![
                    format!("You will give: {}", from_formatted),
                    format!("You will receive: at least {}", min_to_formatted),
                ];
                
                if let Some(slippage) = slippage_tolerance {
                    details.push(format!("Maximum slippage: {}%", slippage));
                }
                
                if let Some(dex_name) = dex {
                    details.push(format!("Exchange: {}", dex_name));
                }
                
                Ok(details.join(", "))
            },
            
            IntentOperation::ContractCall { contract_address, function_name, parameters, value } => {
                let contract_name = self.resolve_contract_name(contract_address);
                let mut details = vec![
                    format!("Contract: {}", contract_name),
                    format!("Function: {}", function_name),
                ];
                
                if let Some(val) = value {
                    if val != "0" {
                        details.push(format!("Value: {} ETH", self.format_decimal_amount(val)?));
                    }
                }
                
                if !parameters.is_empty() {
                    details.push(format!("Parameters: {} inputs", parameters.len()));
                }
                
                Ok(details.join(", "))
            },
            
            IntentOperation::MultiSig { multisig_address, operations, required_signatures, current_signatures } => {
                Ok(format!(
                    "Multisig wallet: {}, requires {}/{} signatures, contains {} operations",
                    self.format_address(multisig_address),
                    required_signatures,
                    current_signatures,
                    operations.len()
                ))
            },
            
            IntentOperation::Stake { validator, amount, token, lock_period } => {
                let formatted_amount = self.format_token_amount(amount, token)?;
                let mut details = vec![
                    format!("Amount: {}", formatted_amount),
                    format!("Validator: {}", self.format_address(validator)),
                ];
                
                if let Some(lock) = lock_period {
                    details.push(format!("Lock period: {}", lock));
                }
                
                Ok(details.join(", "))
            },
            
            IntentOperation::Unstake { validator, amount, token, withdrawal_delay } => {
                let formatted_amount = self.format_token_amount(amount, token)?;
                let mut details = vec![
                    format!("Amount: {}", formatted_amount),
                    format!("Validator: {}", self.format_address(validator)),
                ];
                
                if let Some(delay) = withdrawal_delay {
                    details.push(format!("Withdrawal delay: {}", delay));
                }
                
                Ok(details.join(", "))
            },
            
            IntentOperation::DeFiOperation { protocol, operation_type, tokens, parameters } => {
                let mut details = vec![
                    format!("Protocol: {}", self.resolve_protocol_name(protocol)),
                    format!("Operation: {}", operation_type),
                ];
                
                if !tokens.is_empty() {
                    let token_list = tokens.iter()
                        .map(|ta| self.format_token_amount(&ta.amount, &ta.token))
                        .collect::<Result<Vec<_>, _>>()?;
                    details.push(format!("Tokens: {}", token_list.join(", ")));
                }
                
                if !parameters.is_empty() {
                    details.push(format!("Additional parameters: {}", parameters.len()));
                }
                
                Ok(details.join(", "))
            },
        }
    }

    /// Generate risk warnings
    fn generate_warnings(&self, operations: &[IntentOperation], network: &NetworkInfo) -> Result<Vec<String>, IntentSummaryError> {
        let mut warnings = Vec::new();

        // Check for high-value transactions
        for operation in operations {
            if let Some(warning) = self.check_high_value_warning(operation)? {
                warnings.push(warning);
            }
        }

        // Check for unknown contracts
        for operation in operations {
            if let IntentOperation::ContractCall { contract_address, .. } = operation {
                if !self.is_verified_contract(contract_address) {
                    warnings.push(format!(
                        "⚠️  Interacting with unverified contract: {}",
                        self.format_address(contract_address)
                    ));
                }
            }
        }

        // Check for risky DeFi operations
        for operation in operations {
            if let IntentOperation::DeFiOperation { protocol, operation_type, .. } = operation {
                if let Some(protocol_info) = self.protocol_registry.get(protocol) {
                    match protocol_info.risk_level {
                        RiskLevel::High | RiskLevel::Critical => {
                            warnings.push(format!(
                                "⚠️  High-risk DeFi operation: {} on {}",
                                operation_type, protocol_info.name
                            ));
                        },
                        _ => {}
                    }
                }
            }
        }

        // Check for testnet transactions
        if self.is_testnet(network.chain_id) {
            warnings.push("ℹ️  This transaction will execute on a test network".to_string());
        }

        // Check for multisig complexity
        for operation in operations {
            if let IntentOperation::MultiSig { operations: inner_ops, .. } = operation {
                if inner_ops.len() > 5 {
                    warnings.push("⚠️  Complex multisig transaction with many operations".to_string());
                }
            }
        }

        Ok(warnings)
    }

    /// Generate network information
    fn generate_network_info(&self, network: &NetworkInfo) -> String {
        format!("Network: {} (Chain ID: {})", network.name, network.chain_id)
    }

    /// Generate gas fee summary
    fn generate_fee_summary(&self, gas: &GasEstimate) -> String {
        if let Some(usd_value) = &gas.usd_value {
            format!(
                "Estimated fee: {} {} (~${})",
                self.format_decimal_amount(&gas.total_fee).unwrap_or_else(|_| gas.total_fee.clone()),
                gas.fee_token.symbol,
                usd_value
            )
        } else {
            format!(
                "Estimated fee: {} {}",
                self.format_decimal_amount(&gas.total_fee).unwrap_or_else(|_| gas.total_fee.clone()),
                gas.fee_token.symbol
            )
        }
    }

    /// Assess overall risk level
    fn assess_risk_level(&self, operations: &[IntentOperation]) -> RiskLevel {
        let mut max_risk = RiskLevel::Low;

        for operation in operations {
            let risk = match operation {
                IntentOperation::Transfer { .. } => RiskLevel::Low,
                IntentOperation::Swap { .. } => RiskLevel::Medium,
                IntentOperation::ContractCall { contract_address, .. } => {
                    if self.is_verified_contract(contract_address) {
                        RiskLevel::Medium
                    } else {
                        RiskLevel::High
                    }
                },
                IntentOperation::MultiSig { operations: inner_ops, .. } => {
                    if inner_ops.len() > 3 {
                        RiskLevel::High
                    } else {
                        RiskLevel::Medium
                    }
                },
                IntentOperation::Stake { .. } | IntentOperation::Unstake { .. } => RiskLevel::Medium,
                IntentOperation::DeFiOperation { protocol, .. } => {
                    self.protocol_registry.get(protocol)
                        .map(|p| p.risk_level.clone())
                        .unwrap_or(RiskLevel::High)
                },
            };

            max_risk = match (&max_risk, &risk) {
                (_, RiskLevel::Critical) => RiskLevel::Critical,
                (RiskLevel::Critical, _) => RiskLevel::Critical,
                (_, RiskLevel::High) => RiskLevel::High,
                (RiskLevel::High, _) => RiskLevel::High,
                (_, RiskLevel::Medium) => RiskLevel::Medium,
                (RiskLevel::Medium, _) => RiskLevel::Medium,
                _ => RiskLevel::Low,
            };
        }

        max_risk
    }

    /// Calculate confidence in summary accuracy
    fn calculate_confidence(&self, operations: &[IntentOperation]) -> f64 {
        let mut total_confidence = 0.0;
        let mut count = 0;

        for operation in operations {
            let confidence = match operation {
                IntentOperation::Transfer { .. } => 0.95,
                IntentOperation::Swap { .. } => 0.90,
                IntentOperation::ContractCall { contract_address, .. } => {
                    if self.contract_registry.contains_key(contract_address) {
                        0.85
                    } else {
                        0.60
                    }
                },
                IntentOperation::MultiSig { .. } => 0.80,
                IntentOperation::Stake { .. } | IntentOperation::Unstake { .. } => 0.85,
                IntentOperation::DeFiOperation { protocol, .. } => {
                    if self.protocol_registry.contains_key(protocol) {
                        0.80
                    } else {
                        0.50
                    }
                },
            };

            total_confidence += confidence;
            count += 1;
        }

        if count > 0 {
            total_confidence / count as f64
        } else {
            0.0
        }
    }

    /// Helper methods
    fn format_token_amount(&self, amount: &str, token: &TokenInfo) -> Result<String, IntentSummaryError> {
        let formatted_amount = self.format_decimal_amount(amount)?;
        Ok(format!("{} {}", formatted_amount, token.symbol))
    }

    fn format_decimal_amount(&self, amount: &str) -> Result<String, IntentSummaryError> {
        let decimal = amount.parse::<Decimal>()
            .map_err(|_| IntentSummaryError::InvalidAmount(amount.to_string()))?;
        
        // Format with appropriate precision
        if decimal >= Decimal::from(1000000) {
            Ok(format!("{:.2}M", decimal / Decimal::from(1000000)))
        } else if decimal >= Decimal::from(1000) {
            Ok(format!("{:.2}K", decimal / Decimal::from(1000)))
        } else if decimal >= Decimal::from(1) {
            Ok(format!("{:.2}", decimal))
        } else {
            Ok(format!("{:.6}", decimal))
        }
    }

    fn format_address(&self, address: &str) -> String {
        if address.len() > 10 {
            format!("{}...{}", &address[0..6], &address[address.len()-4..])
        } else {
            address.to_string()
        }
    }

    fn operation_type_name(&self, operation: &IntentOperation) -> &'static str {
        match operation {
            IntentOperation::Transfer { .. } => "Transfer",
            IntentOperation::Swap { .. } => "Swap",
            IntentOperation::ContractCall { .. } => "Contract Call",
            IntentOperation::MultiSig { .. } => "MultiSig",
            IntentOperation::Stake { .. } => "Stake",
            IntentOperation::Unstake { .. } => "Unstake",
            IntentOperation::DeFiOperation { .. } => "DeFi",
        }
    }

    fn resolve_contract_name(&self, address: &str) -> String {
        self.contract_registry.get(address)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| self.format_address(address))
    }

    fn resolve_protocol_name(&self, protocol: &str) -> String {
        self.protocol_registry.get(protocol)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| protocol.to_string())
    }

    fn is_verified_contract(&self, address: &str) -> bool {
        self.contract_registry.get(address)
            .map(|c| c.verified)
            .unwrap_or(false)
    }

    fn is_testnet(&self, chain_id: u64) -> bool {
        matches!(chain_id, 5 | 11155111 | 80001 | 97) // Goerli, Sepolia, Mumbai, BSC Testnet
    }

    fn check_high_value_warning(&self, operation: &IntentOperation) -> Result<Option<String>, IntentSummaryError> {
        match operation {
            IntentOperation::Transfer { amount, token, .. } => {
                let decimal_amount = amount.parse::<Decimal>()
                    .map_err(|_| IntentSummaryError::InvalidAmount(amount.to_string()))?;
                
                if decimal_amount >= Decimal::from(10000) {
                    Ok(Some(format!("💰 High-value transfer: {} {}", amount, token.symbol)))
                } else {
                    Ok(None)
                }
            },
            IntentOperation::Swap { from_amount, from_token, .. } => {
                let decimal_amount = from_amount.parse::<Decimal>()
                    .map_err(|_| IntentSummaryError::InvalidAmount(from_amount.to_string()))?;
                
                if decimal_amount >= Decimal::from(10000) {
                    Ok(Some(format!("💰 High-value swap: {} {}", from_amount, from_token.symbol)))
                } else {
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }

    /// Default token registry with common tokens
    fn default_token_registry() -> HashMap<String, TokenInfo> {
        let mut registry = HashMap::new();
        
        registry.insert("ETH".to_string(), TokenInfo {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
            contract_address: None,
            chain_id: Some(1),
            logo_url: None,
        });
        
        registry.insert("USDC".to_string(), TokenInfo {
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            contract_address: Some("0xA0b86a33E6441D5F113C8f5C4Ba79A1a5c02CE5F".to_string()),
            chain_id: Some(1),
            logo_url: None,
        });
        
        registry.insert("POLY".to_string(), TokenInfo {
            symbol: "POLY".to_string(),
            name: "Polymera Token".to_string(),
            decimals: 18,
            contract_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            chain_id: Some(1),
            logo_url: None,
        });
        
        registry
    }

    /// Default protocol registry with known DeFi protocols
    fn default_protocol_registry() -> HashMap<String, ProtocolInfo> {
        let mut registry = HashMap::new();
        
        registry.insert("uniswap".to_string(), ProtocolInfo {
            name: "Uniswap".to_string(),
            description: "Decentralized exchange".to_string(),
            risk_level: RiskLevel::Medium,
            known_functions: HashMap::new(),
        });
        
        registry.insert("compound".to_string(), ProtocolInfo {
            name: "Compound".to_string(),
            description: "Lending protocol".to_string(),
            risk_level: RiskLevel::Medium,
            known_functions: HashMap::new(),
        });
        
        registry
    }

    /// Default contract registry with known smart contracts
    fn default_contract_registry() -> HashMap<String, ContractInfo> {
        let mut registry = HashMap::new();
        
        registry.insert("0xA0b86a33E6441D5F113C8f5C4Ba79A1a5c02CE5F".to_string(), ContractInfo {
            name: "USDC Token".to_string(),
            description: "USD Coin smart contract".to_string(),
            verified: true,
            functions: HashMap::new(),
        });
        
        registry
    }
}

impl Default for IntentSummarizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // Include comprehensive snapshot tests
    #[path = "intent_summarize_tests.rs"]
    mod snapshot_tests;

    fn create_test_token() -> TokenInfo {
        TokenInfo {
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            decimals: 6,
            contract_address: Some("0xA0b86a33E6441D5F113C8f5C4Ba79A1a5c02CE5F".to_string()),
            chain_id: Some(1),
            logo_url: None,
        }
    }

    fn create_test_network() -> NetworkInfo {
        NetworkInfo {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            currency: "ETH".to_string(),
            block_explorer_url: Some("https://etherscan.io".to_string()),
        }
    }

    #[test]
    fn test_transfer_summary() {
        let summarizer = IntentSummarizer::new();
        let token = create_test_token();
        
        let operation = IntentOperation::Transfer {
            from: "0x1234567890123456789012345678901234567890".to_string(),
            to: "0x0987654321098765432109876543210987654321".to_string(),
            amount: "1000.50".to_string(),
            token,
            memo: Some("Payment for services".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: None,
            network: create_test_network(),
            timestamp: Utc::now(),
            nonce: Some(42),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        assert!(summary.primary_action.contains("Send 1000.50 USDC"));
        assert!(summary.primary_action.contains("0x0987...4321"));
        assert!(summary.primary_action.contains("Payment for services"));
        assert_eq!(summary.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_swap_summary() {
        let summarizer = IntentSummarizer::new();
        let usdc = create_test_token();
        let eth = TokenInfo {
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            decimals: 18,
            contract_address: None,
            chain_id: Some(1),
            logo_url: None,
        };
        
        let operation = IntentOperation::Swap {
            from_token: usdc,
            to_token: eth,
            from_amount: "1000.0".to_string(),
            min_to_amount: "0.5".to_string(),
            slippage_tolerance: Some("2.0".to_string()),
            dex: Some("Uniswap".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: None,
            network: create_test_network(),
            timestamp: Utc::now(),
            nonce: Some(42),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        assert!(summary.primary_action.contains("Swap 1000.00 USDC"));
        assert!(summary.primary_action.contains("at least 0.50 ETH"));
        assert!(summary.primary_action.contains("Uniswap"));
        assert!(summary.primary_action.contains("2.0%"));
        assert_eq!(summary.risk_level, RiskLevel::Medium);
    }

    #[test]
    fn test_contract_call_summary() {
        let summarizer = IntentSummarizer::new();
        
        let operation = IntentOperation::ContractCall {
            contract_address: "0xUnknownContract123456789012345678901234567890".to_string(),
            function_name: "complexFunction".to_string(),
            parameters: vec![
                ContractParameter {
                    name: "amount".to_string(),
                    param_type: "uint256".to_string(),
                    value: serde_json::Value::String("1000".to_string()),
                }
            ],
            value: Some("0.1".to_string()),
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: None,
            network: create_test_network(),
            timestamp: Utc::now(),
            nonce: Some(42),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        assert!(summary.primary_action.contains("Call complexFunction"));
        assert!(summary.primary_action.contains("0.10 ETH"));
        assert!(summary.warnings.iter().any(|w| w.contains("unverified contract")));
        assert_eq!(summary.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_high_value_warning() {
        let summarizer = IntentSummarizer::new();
        let token = create_test_token();
        
        let operation = IntentOperation::Transfer {
            from: "0x1234567890123456789012345678901234567890".to_string(),
            to: "0x0987654321098765432109876543210987654321".to_string(),
            amount: "50000.0".to_string(), // High value
            token,
            memo: None,
        };
        
        let intent = TransactionIntent {
            operations: vec![operation],
            gas_estimate: None,
            network: create_test_network(),
            timestamp: Utc::now(),
            nonce: Some(42),
            metadata: HashMap::new(),
        };
        
        let summary = summarizer.summarize(&intent).unwrap();
        assert!(summary.warnings.iter().any(|w| w.contains("High-value transfer")));
    }

    #[test]
    fn test_format_token_amount() {
        let summarizer = IntentSummarizer::new();
        let token = create_test_token();
        
        assert_eq!(
            summarizer.format_token_amount("1000.50", &token).unwrap(),
            "1000.50 USDC"
        );
        
        assert_eq!(
            summarizer.format_token_amount("1500000.0", &token).unwrap(),
            "1.50M USDC"
        );
        
        assert_eq!(
            summarizer.format_token_amount("2500.0", &token).unwrap(),
            "2.50K USDC"
        );
    }

    #[test]
    fn test_address_formatting() {
        let summarizer = IntentSummarizer::new();
        
        let long_address = "0x1234567890123456789012345678901234567890";
        assert_eq!(
            summarizer.format_address(long_address),
            "0x1234...7890"
        );
        
        let short_address = "0x1234";
        assert_eq!(
            summarizer.format_address(short_address),
            "0x1234"
        );
    }
}
