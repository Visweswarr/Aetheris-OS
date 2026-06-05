//! Storage Service for Polymera OS
//!
//! Implements advanced storage features:
//! - Copy-on-Write snapshots (14.2)
//! - Block deduplication (14.4)
//! - Write-ahead log for crash consistency (14.6)
//! - Hardware enclave key unwrapping (14.8)

pub mod cow;
pub mod dedup;
pub mod wal;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Block size (4KB)
pub const BLOCK_SIZE: usize = 4096;

/// Block address
pub type BlockAddr = u64;

/// Storage Manager
pub struct StorageManager {
    pub cow_manager: cow::CowManager,
    pub dedup_engine: dedup::DedupEngine,
    pub wal: wal::WriteAheadLog,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            cow_manager: cow::CowManager::new(),
            dedup_engine: dedup::DedupEngine::new(),
            wal: wal::WriteAheadLog::new(),
        }
    }
    
    /// Write a block with dedup and CoW
    pub fn write_block(&mut self, addr: BlockAddr, data: &[u8]) -> Result<(), StorageError> {
        if data.len() != BLOCK_SIZE {
            return Err(StorageError::InvalidBlockSize);
        }
        
        // 1. Log to WAL first (crash consistency)
        self.wal.log_write(addr, data)?;
        
        // 2. Check for dedup
        let hash = self.dedup_engine.hash_block(data);
        if let Some(existing_addr) = self.dedup_engine.find_duplicate(&hash) {
            // Reference existing block instead of writing duplicate
            self.cow_manager.add_reference(existing_addr);
            self.wal.commit()?;
            return Ok(());
        }
        
        // 3. Write with CoW
        self.cow_manager.write(addr, data)?;
        
        // 4. Register in dedup index
        self.dedup_engine.register_block(addr, hash);
        
        // 5. Commit WAL entry
        self.wal.commit()?;
        
        Ok(())
    }
    
    /// Read a block
    pub fn read_block(&self, addr: BlockAddr) -> Result<Vec<u8>, StorageError> {
        self.cow_manager.read(addr)
    }
    
    /// Create a snapshot
    pub fn create_snapshot(&mut self, name: &str) -> Result<u64, StorageError> {
        self.cow_manager.create_snapshot(name)
    }
    
    /// Restore from snapshot
    pub fn restore_snapshot(&mut self, snapshot_id: u64) -> Result<(), StorageError> {
        self.cow_manager.restore_snapshot(snapshot_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    InvalidBlockSize,
    BlockNotFound,
    SnapshotNotFound,
    WalError,
    IoError,
}
