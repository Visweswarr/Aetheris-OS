//! Block Deduplication Engine
//!
//! Requirement: 14.4 - Block deduplication

use std::collections::HashMap;
use super::{BlockAddr, BLOCK_SIZE};

/// Block hash (SHA-256)
pub type BlockHash = [u8; 32];

/// Deduplication Engine
pub struct DedupEngine {
    /// Hash -> Physical address mapping
    hash_index: HashMap<BlockHash, BlockAddr>,
    /// Statistics
    stats: DedupStats,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DedupStats {
    pub total_blocks: u64,
    pub unique_blocks: u64,
    pub deduplicated_blocks: u64,
    pub bytes_saved: u64,
}

impl DedupEngine {
    pub fn new() -> Self {
        Self {
            hash_index: HashMap::new(),
            stats: DedupStats::default(),
        }
    }
    
    /// Hash a block using SHA-256
    pub fn hash_block(&self, data: &[u8]) -> BlockHash {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    /// Find duplicate block by hash
    pub fn find_duplicate(&self, hash: &BlockHash) -> Option<BlockAddr> {
        self.hash_index.get(hash).copied()
    }
    
    /// Register a new block in the dedup index
    pub fn register_block(&mut self, addr: BlockAddr, hash: BlockHash) {
        self.hash_index.insert(hash, addr);
        self.stats.total_blocks += 1;
        self.stats.unique_blocks += 1;
    }
    
    /// Record a deduplication hit
    pub fn record_dedup_hit(&mut self) {
        self.stats.total_blocks += 1;
        self.stats.deduplicated_blocks += 1;
        self.stats.bytes_saved += BLOCK_SIZE as u64;
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> DedupStats {
        self.stats
    }
    
    /// Deduplication ratio
    pub fn dedup_ratio(&self) -> f64 {
        if self.stats.total_blocks == 0 {
            1.0
        } else {
            self.stats.unique_blocks as f64 / self.stats.total_blocks as f64
        }
    }
}
