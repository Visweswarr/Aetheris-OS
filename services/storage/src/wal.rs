//! Write-Ahead Log for Crash Consistency
//!
//! Requirement: 14.6 - WAL for crash consistency

use std::collections::VecDeque;
use super::{BlockAddr, StorageError};

/// WAL Entry
#[derive(Debug, Clone)]
pub struct WalEntry {
    pub sequence: u64,
    pub block_addr: BlockAddr,
    pub data: Vec<u8>,
    pub committed: bool,
}

/// Write-Ahead Log
pub struct WriteAheadLog {
    entries: VecDeque<WalEntry>,
    next_sequence: u64,
    /// Maximum uncommitted entries before forcing flush
    max_pending: usize,
}

impl WriteAheadLog {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            next_sequence: 1,
            max_pending: 1024,
        }
    }
    
    /// Log a write operation
    pub fn log_write(&mut self, addr: BlockAddr, data: &[u8]) -> Result<u64, StorageError> {
        let seq = self.next_sequence;
        self.next_sequence += 1;
        
        let entry = WalEntry {
            sequence: seq,
            block_addr: addr,
            data: data.to_vec(),
            committed: false,
        };
        
        self.entries.push_back(entry);
        
        // Check if we need to force flush
        if self.pending_count() >= self.max_pending {
            self.flush()?;
        }
        
        Ok(seq)
    }
    
    /// Commit the most recent entry
    pub fn commit(&mut self) -> Result<(), StorageError> {
        if let Some(entry) = self.entries.back_mut() {
            entry.committed = true;
        }
        Ok(())
    }
    
    /// Commit a specific sequence number
    pub fn commit_sequence(&mut self, seq: u64) -> Result<(), StorageError> {
        for entry in self.entries.iter_mut() {
            if entry.sequence == seq {
                entry.committed = true;
                return Ok(());
            }
        }
        Err(StorageError::WalError)
    }
    
    /// Flush committed entries (persist and remove)
    pub fn flush(&mut self) -> Result<(), StorageError> {
        // In production: fsync to disk
        // Remove committed entries from front
        while let Some(entry) = self.entries.front() {
            if entry.committed {
                self.entries.pop_front();
            } else {
                break;
            }
        }
        Ok(())
    }
    
    /// Recover from crash - replay uncommitted entries
    pub fn recover(&self) -> Vec<WalEntry> {
        self.entries
            .iter()
            .filter(|e| !e.committed)
            .cloned()
            .collect()
    }
    
    /// Count pending (uncommitted) entries
    pub fn pending_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.committed).count()
    }
    
    /// Total entries
    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }
}
