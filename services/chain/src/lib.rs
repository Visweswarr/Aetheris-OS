//! Chain service for Aetheris OS
//! 
//! This module provides blockchain interaction capabilities including
//! transaction relay, state management, and cross-chain communication.

pub mod relay;

pub use relay::{
    TransactionRelay, ChainType, ChainConfig, TransactionRequest, 
    TransactionResult, TransactionStatus, RelayStats, ChainStats
};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Chain service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainServiceConfig {
    /// Default gas price in wei
    pub default_gas_price: u64,
    /// Default gas limit
    pub default_gas_limit: u64,
    /// Transaction timeout in milliseconds
    pub transaction_timeout_ms: u64,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
    /// Enable PQC signatures
    pub enable_pqc_signatures: bool,
    /// Chain configurations
    pub chain_configs: Vec<ChainConfig>,
}

impl Default for ChainServiceConfig {
    fn default() -> Self {
        Self {
            default_gas_price: 20_000_000_000, // 20 gwei
            default_gas_limit: 21000,
            transaction_timeout_ms: 30000,
            max_retry_attempts: 3,
            enable_pqc_signatures: true,
            chain_configs: vec![],
        }
    }
}

/// Chain service error
#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Chain not supported: {0}")]
    ChainNotSupported(String),
    
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Main chain service
pub struct ChainService {
    config: ChainServiceConfig,
    relay: Arc<TransactionRelay>,
    stats: Arc<RwLock<ChainServiceStats>>,
}

/// Chain service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainServiceStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: u64,
    pub active_connections: u32,
    pub last_updated: u64,
}

impl ChainService {
    /// Create a new chain service
    pub fn new(config: ChainServiceConfig) -> Result<Self, ChainError> {
        let relay = Arc::new(TransactionRelay::new(config.chain_configs.clone()));
        
        let stats = Arc::new(RwLock::new(ChainServiceStats {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: 0,
            active_connections: 0,
            last_updated: chrono::Utc::now().timestamp() as u64,
        }));

        Ok(Self {
            config,
            relay,
            stats,
        })
    }

    /// Get the transaction relay
    pub fn relay(&self) -> Arc<TransactionRelay> {
        self.relay.clone()
    }

    /// Get the configuration
    pub fn config(&self) -> &ChainServiceConfig {
        &self.config
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> ChainServiceStats {
        self.stats.read().await.clone()
    }

    /// Update service statistics
    async fn update_stats(&self, success: bool, response_time: u64) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
        
        // Update average response time
        let total = stats.successful_requests + stats.failed_requests;
        if total > 0 {
            stats.average_response_time = 
                (stats.average_response_time * (total - 1) + response_time) / total;
        }
        
        stats.last_updated = chrono::Utc::now().timestamp() as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_service_config_default() {
        let config = ChainServiceConfig::default();
        assert_eq!(config.default_gas_price, 20_000_000_000);
        assert_eq!(config.default_gas_limit, 21000);
        assert_eq!(config.transaction_timeout_ms, 30000);
        assert_eq!(config.max_retry_attempts, 3);
        assert!(config.enable_pqc_signatures);
    }

    #[test]
    fn test_chain_service_creation() {
        let config = ChainServiceConfig::default();
        let service = ChainService::new(config);
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_chain_service_stats() {
        let config = ChainServiceConfig::default();
        let service = ChainService::new(config).unwrap();
        
        let stats = service.get_stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
    }
}