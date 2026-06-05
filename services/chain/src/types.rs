//! Type definitions for the chain service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// NGFS snapshot hash for anchoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SnapshotHash {
    /// The content hash of the snapshot
    pub hash: String,
    /// Timestamp when the snapshot was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Size of the snapshot in bytes
    pub size: u64,
    /// DID of the creator
    pub creator_did: String,
}

/// Anchor batch for batch submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorBatch {
    /// Unique batch ID
    pub batch_id: Uuid,
    /// List of snapshot hashes to anchor
    pub snapshots: Vec<SnapshotHash>,
    /// DID of the submitter
    pub submitter_did: String,
    /// Timestamp when batch was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Batch signature (hybrid PQC + legacy)
    pub signature: BatchSignature,
}

/// Hybrid signature for batch verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSignature {
    /// Legacy signature (ECDSA/Ed25519)
    pub legacy_signature: String,
    /// PQC signature (Dilithium)
    pub pqc_signature: String,
    /// Signature algorithm used
    pub algorithm: String,
    /// Public key for verification
    pub public_key: String,
}

/// Anchor transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorResult {
    /// Transaction hash
    pub tx_hash: String,
    /// Block number
    pub block_number: u64,
    /// Gas used
    pub gas_used: u64,
    /// Timestamp of the transaction
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Status of the transaction
    pub status: TransactionStatus,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

/// Anchor verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the anchor is valid
    pub is_valid: bool,
    /// Block number where anchored
    pub block_number: Option<u64>,
    /// Transaction hash
    pub tx_hash: Option<String>,
    /// Verification timestamp
    pub verified_at: chrono::DateTime<chrono::Utc>,
    /// Error message if verification failed
    pub error: Option<String>,
}

/// Chain state for tracking anchored snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainState {
    /// Map of snapshot hash to anchor result
    pub anchored_snapshots: HashMap<String, AnchorResult>,
    /// Map of batch ID to batch result
    pub anchored_batches: HashMap<Uuid, AnchorResult>,
    /// Last processed block number
    pub last_block_number: u64,
    /// Total gas used
    pub total_gas_used: u64,
    /// Total transactions submitted
    pub total_transactions: u64,
}

impl Default for ChainState {
    fn default() -> Self {
        Self {
            anchored_snapshots: HashMap::new(),
            anchored_batches: HashMap::new(),
            last_block_number: 0,
            total_gas_used: 0,
            total_transactions: 0,
        }
    }
}

/// Anchor request for queuing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorRequest {
    /// Unique request ID
    pub request_id: Uuid,
    /// Snapshot hash to anchor
    pub snapshot_hash: SnapshotHash,
    /// Priority level (higher = more urgent)
    pub priority: u8,
    /// Timestamp when request was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Retry count
    pub retry_count: u32,
    /// Maximum retries
    pub max_retries: u32,
}

impl AnchorRequest {
    /// Create a new anchor request
    pub fn new(snapshot_hash: SnapshotHash, priority: u8) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            snapshot_hash,
            priority,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
        }
    }

    /// Check if request can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}
