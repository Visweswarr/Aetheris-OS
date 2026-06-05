//! Anchor service for on-chain NGFS snapshot anchoring

use crate::error::ChainError;
use crate::types::*;
use crate::ChainConfig;
use polymera_crypto::{DilithiumKeyPair, KyberKeyPair, SignatureAlgorithm};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use uuid::Uuid;
use web3::contract::{Contract, Options};
use web3::transports::Http;
use web3::types::{Address, BlockNumber, H256, U256};
use web3::Web3;

/// Anchor service for managing on-chain anchoring
pub struct AnchorService {
    config: ChainConfig,
    web3: Web3<Http>,
    contract: Contract<Http>,
    state: Arc<RwLock<ChainState>>,
    request_queue: Arc<RwLock<VecDeque<AnchorRequest>>>,
    batch_timer: Arc<RwLock<Option<Instant>>>,
}

impl AnchorService {
    /// Create a new anchor service
    pub fn new(config: ChainConfig) -> Result<Self, ChainError> {
        let transport = Http::new(&config.ethereum_rpc)?;
        let web3 = Web3::new(transport);
        
        // Parse contract address
        let contract_address: Address = config.anchor_contract.parse()
            .map_err(|_| ChainError::ConfigError("Invalid contract address".to_string()))?;
        
        // Load contract ABI (simplified for this implementation)
        let contract = Contract::from_json(
            web3.eth(),
            contract_address,
            include_bytes!("../../contracts/AnchorDAO.json")
        ).map_err(|e| ChainError::ContractError(format!("Failed to load contract: {}", e)))?;

        Ok(Self {
            config,
            web3,
            contract,
            state: Arc::new(RwLock::new(ChainState::default())),
            request_queue: Arc::new(RwLock::new(VecDeque::new())),
            batch_timer: Arc::new(RwLock::new(None)),
        })
    }

    /// Queue a snapshot hash for anchoring
    pub async fn queue_anchor(&self, snapshot_hash: SnapshotHash, priority: u8) -> Result<Uuid, ChainError> {
        let request = AnchorRequest::new(snapshot_hash, priority);
        let request_id = request.request_id;
        
        let mut queue = self.request_queue.write().await;
        queue.push_back(request);
        
        // Sort by priority (higher priority first)
        queue.make_contiguous().sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(request_id)
    }

    /// Process the next batch of anchor requests
    pub async fn process_batch(&self) -> Result<AnchorResult, ChainError> {
        let mut queue = self.request_queue.write().await;
        let mut batch_snapshots = Vec::new();
        
        // Collect snapshots for batch (up to max_batch_size)
        while batch_snapshots.len() < self.config.max_batch_size && !queue.is_empty() {
            if let Some(request) = queue.pop_front() {
                batch_snapshots.push(request.snapshot_hash);
            }
        }
        
        if batch_snapshots.is_empty() {
            return Err(ChainError::InvalidInput("No snapshots to anchor".to_string()));
        }

        // Create batch
        let batch = AnchorBatch {
            batch_id: Uuid::new_v4(),
            snapshots: batch_snapshots,
            submitter_did: "did:key:example".to_string(), // TODO: Get from wallet service
            created_at: chrono::Utc::now(),
            signature: self.create_batch_signature(&batch_snapshots).await?,
        };

        // Submit batch to contract
        let result = self.submit_batch(&batch).await?;
        
        // Update state
        let mut state = self.state.write().await;
        state.anchored_batches.insert(batch.batch_id, result.clone());
        state.total_transactions += 1;
        state.total_gas_used += result.gas_used;
        
        Ok(result)
    }

    /// Create a hybrid signature for the batch
    async fn create_batch_signature(&self, snapshots: &[SnapshotHash]) -> Result<BatchSignature, ChainError> {
        // Serialize snapshots for signing
        let data = serde_json::to_vec(snapshots)
            .map_err(|e| ChainError::SerializationError(e))?;
        
        // Create hybrid signature (PQC + legacy)
        let dilithium_keypair = DilithiumKeyPair::generate(SignatureAlgorithm::Dilithium3)
            .map_err(|e| ChainError::CryptoError(format!("Failed to generate Dilithium keypair: {}", e)))?;
        
        let legacy_signature = self.create_legacy_signature(&data).await?;
        let pqc_signature = dilithium_keypair.sign(&data)
            .map_err(|e| ChainError::CryptoError(format!("Failed to create PQC signature: {}", e)))?;
        
        Ok(BatchSignature {
            legacy_signature: hex::encode(&legacy_signature),
            pqc_signature: hex::encode(&pqc_signature),
            algorithm: "hybrid-dilithium3-secp256k1".to_string(),
            public_key: hex::encode(dilithium_keypair.public_key()),
        })
    }

    /// Create legacy signature (ECDSA)
    async fn create_legacy_signature(&self, data: &[u8]) -> Result<Vec<u8>, ChainError> {
        // TODO: Integrate with wallet service for actual signing
        // For now, return a mock signature
        Ok(vec![0u8; 64]) // Mock ECDSA signature
    }

    /// Submit batch to smart contract
    async fn submit_batch(&self, batch: &AnchorBatch) -> Result<AnchorResult, ChainError> {
        // Prepare contract call data
        let snapshot_hashes: Vec<String> = batch.snapshots.iter()
            .map(|s| s.hash.clone())
            .collect();
        
        let timestamps: Vec<u64> = batch.snapshots.iter()
            .map(|s| s.timestamp.timestamp() as u64)
            .collect();
        
        let sizes: Vec<u64> = batch.snapshots.iter()
            .map(|s| s.size)
            .collect();
        
        let creator_dids: Vec<String> = batch.snapshots.iter()
            .map(|s| s.creator_did.clone())
            .collect();

        // Call contract method
        let tx_hash = self.contract
            .call(
                "anchorBatch",
                (
                    snapshot_hashes,
                    timestamps,
                    sizes,
                    creator_dids,
                    batch.batch_id.to_string(),
                    batch.submitter_did.clone(),
                    batch.signature.legacy_signature.clone(),
                    batch.signature.pqc_signature.clone(),
                    batch.signature.algorithm.clone(),
                    batch.signature.public_key.clone(),
                ),
                self.get_account(),
                Options::with(|opt| {
                    opt.gas = Some(U256::from(self.config.gas_limit));
                    opt.gas_price = Some(U256::from(self.config.gas_price));
                }),
            )
            .await
            .map_err(|e| ChainError::ContractError(format!("Contract call failed: {}", e)))?;

        // Wait for transaction confirmation
        let receipt = self.wait_for_transaction(tx_hash).await?;
        
        Ok(AnchorResult {
            tx_hash: format!("{:?}", tx_hash),
            block_number: receipt.block_number.unwrap_or_default().as_u64(),
            gas_used: receipt.gas_used.unwrap_or_default().as_u64(),
            timestamp: chrono::Utc::now(),
            status: if receipt.status.unwrap_or_default() == U256::from(1) {
                TransactionStatus::Confirmed
            } else {
                TransactionStatus::Failed
            },
        })
    }

    /// Wait for transaction confirmation
    async fn wait_for_transaction(&self, tx_hash: H256) -> Result<web3::types::TransactionReceipt, ChainError> {
        let mut attempts = 0;
        let max_attempts = 30; // 5 minutes with 10s intervals
        
        while attempts < max_attempts {
            if let Some(receipt) = self.web3.eth().transaction_receipt(tx_hash).await? {
                return Ok(receipt);
            }
            
            tokio::time::sleep(Duration::from_secs(10)).await;
            attempts += 1;
        }
        
        Err(ChainError::TransactionTimeout("Transaction confirmation timeout".to_string()))
    }

    /// Get the account address for transactions
    fn get_account(&self) -> Address {
        // TODO: Get from wallet service
        Address::from([0u8; 20]) // Mock address
    }

    /// Verify an anchored snapshot
    pub async fn verify_anchor(&self, snapshot_hash: &str) -> Result<VerificationResult, ChainError> {
        // Call contract to verify anchor
        let result: (bool, u64, String) = self.contract
            .query(
                "verifyAnchor",
                snapshot_hash,
                None,
                Options::default(),
                None,
            )
            .await
            .map_err(|e| ChainError::ContractError(format!("Verification query failed: {}", e)))?;

        Ok(VerificationResult {
            is_valid: result.0,
            block_number: if result.1 > 0 { Some(result.1) } else { None },
            tx_hash: if !result.2.is_empty() { Some(result.2) } else { None },
            verified_at: chrono::Utc::now(),
            error: if !result.0 { Some("Anchor not found or invalid".to_string()) } else { None },
        })
    }

    /// Get chain state
    pub async fn get_state(&self) -> ChainState {
        self.state.read().await.clone()
    }

    /// Start batch processing timer
    pub async fn start_batch_timer(&self) -> Result<(), ChainError> {
        let mut timer = self.batch_timer.write().await;
        *timer = Some(Instant::now());
        Ok(())
    }

    /// Check if batch timer has expired
    pub async fn should_process_batch(&self) -> bool {
        let timer = self.batch_timer.read().await;
        if let Some(start_time) = *timer {
            start_time.elapsed() >= Duration::from_secs(self.config.batch_interval)
        } else {
            false
        }
    }

    /// Reset batch timer
    pub async fn reset_batch_timer(&self) {
        let mut timer = self.batch_timer.write().await;
        *timer = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_anchor_request_creation() {
        let snapshot_hash = SnapshotHash {
            hash: "test_hash".to_string(),
            timestamp: chrono::Utc::now(),
            size: 1024,
            creator_did: "did:key:test".to_string(),
        };
        
        let request = AnchorRequest::new(snapshot_hash, 5);
        assert_eq!(request.priority, 5);
        assert_eq!(request.retry_count, 0);
        assert!(request.can_retry());
    }

    #[tokio::test]
    async fn test_anchor_request_retry() {
        let snapshot_hash = SnapshotHash {
            hash: "test_hash".to_string(),
            timestamp: chrono::Utc::now(),
            size: 1024,
            creator_did: "did:key:test".to_string(),
        };
        
        let mut request = AnchorRequest::new(snapshot_hash, 5);
        request.max_retries = 2;
        
        assert!(request.can_retry());
        request.increment_retry();
        assert!(request.can_retry());
        request.increment_retry();
        assert!(!request.can_retry());
    }
}
