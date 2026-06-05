//! Copy-on-Write Snapshot Management
//!
//! Requirement: 14.2 - CoW snapshots

use std::collections::{HashMap, BTreeMap};
use super::{BlockAddr, BLOCK_SIZE, StorageError};

/// Snapshot metadata
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: u64,
    pub name: String,
    pub timestamp: u64,
    pub block_map: HashMap<BlockAddr, BlockAddr>, // Virtual -> Physical
}

/// CoW Manager
pub struct CowManager {
    /// Current block mapping (virtual -> physical)
    current_map: HashMap<BlockAddr, BlockAddr>,
    /// Block data storage (physical addr -> data)
    block_store: HashMap<BlockAddr, Vec<u8>>,
    /// Reference counts for blocks
    ref_counts: HashMap<BlockAddr, u32>,
    /// Snapshots
    snapshots: BTreeMap<u64, Snapshot>,
    /// Next physical block address
    next_phys_addr: BlockAddr,
    /// Next snapshot ID
    next_snapshot_id: u64,
}

impl CowManager {
    pub fn new() -> Self {
        Self {
            current_map: HashMap::new(),
            block_store: HashMap::new(),
            ref_counts: HashMap::new(),
            snapshots: BTreeMap::new(),
            next_phys_addr: 1,
            next_snapshot_id: 1,
        }
    }
    
    /// Write a block with CoW semantics
    pub fn write(&mut self, vaddr: BlockAddr, data: &[u8]) -> Result<(), StorageError> {
        // Allocate new physical block (CoW - never overwrite)
        let paddr = self.next_phys_addr;
        self.next_phys_addr += 1;
        
        // Store data
        self.block_store.insert(paddr, data.to_vec());
        self.ref_counts.insert(paddr, 1);
        
        // Decrement old block ref if exists
        if let Some(&old_paddr) = self.current_map.get(&vaddr) {
            self.decrement_ref(old_paddr);
        }
        
        // Update mapping
        self.current_map.insert(vaddr, paddr);
        
        Ok(())
    }
    
    /// Read a block
    pub fn read(&self, vaddr: BlockAddr) -> Result<Vec<u8>, StorageError> {
        let paddr = self.current_map.get(&vaddr).ok_or(StorageError::BlockNotFound)?;
        self.block_store.get(paddr).cloned().ok_or(StorageError::BlockNotFound)
    }
    
    /// Add reference to a block (for dedup)
    pub fn add_reference(&mut self, paddr: BlockAddr) {
        *self.ref_counts.entry(paddr).or_insert(0) += 1;
    }
    
    /// Decrement reference count
    fn decrement_ref(&mut self, paddr: BlockAddr) {
        if let Some(count) = self.ref_counts.get_mut(&paddr) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                // Free block
                self.block_store.remove(&paddr);
                self.ref_counts.remove(&paddr);
            }
        }
    }
    
    /// Create a snapshot
    pub fn create_snapshot(&mut self, name: &str) -> Result<u64, StorageError> {
        let id = self.next_snapshot_id;
        self.next_snapshot_id += 1;
        
        // Clone current mapping
        let block_map = self.current_map.clone();
        
        // Increment ref counts for all blocks in snapshot
        for &paddr in block_map.values() {
            self.add_reference(paddr);
        }
        
        let snapshot = Snapshot {
            id,
            name: name.to_string(),
            timestamp: 0, // Would use real time
            block_map,
        };
        
        self.snapshots.insert(id, snapshot);
        println!("[STORAGE] Created snapshot {} '{}'", id, name);
        
        Ok(id)
    }
    
    /// Restore from snapshot
    pub fn restore_snapshot(&mut self, snapshot_id: u64) -> Result<(), StorageError> {
        let snapshot = self.snapshots.get(&snapshot_id)
            .ok_or(StorageError::SnapshotNotFound)?
            .clone();
        
        // Decrement refs for current blocks
        for &paddr in self.current_map.values() {
            self.decrement_ref(paddr);
        }
        
        // Restore mapping from snapshot
        self.current_map = snapshot.block_map.clone();
        
        // Increment refs for restored blocks
        for &paddr in self.current_map.values() {
            self.add_reference(paddr);
        }
        
        println!("[STORAGE] Restored snapshot {}", snapshot_id);
        Ok(())
    }
    
    /// List snapshots
    pub fn list_snapshots(&self) -> Vec<&Snapshot> {
        self.snapshots.values().collect()
    }
}
