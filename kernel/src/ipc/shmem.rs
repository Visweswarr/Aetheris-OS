//! Shared Memory Support for Zero-Copy IPC
//!
//! This module provides mechanisms for creating and managing shared memory regions
//! that can be mapped into multiple process address spaces, enabling zero-copy
//! message passing.
//!
//! Requirement: 6.1 - Zero-Copy Message Passing

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::ipc::{IpcError, IpcResult, ProcessId};

/// Unique identifier for a shared memory region
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShmId(pub u64);

impl core::fmt::Display for ShmId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SHM:{}", self.0)
    }
}

/// Handle to a shared memory region for a specific process
#[derive(Debug, Clone)]
pub struct ShmHandle {
    /// ID of the shared memory region
    pub id: ShmId,
    
    /// Size of the region in bytes
    pub size: usize,
    
    /// Permissions (Read/Write/Exec)
    pub permissions: ShmPermissions,
    
    /// Reference to the underlying region implementation
    region: Arc<Mutex<ShmRegion>>,
}

/// Physical backing for a shared memory region
/// In a real implementation, this would hold the PhysicalFrame objects
#[derive(Debug)]
pub struct ShmRegion {
    pub id: ShmId,
    pub size: usize,
    pub creator: ProcessId,
    /// Reference count of mappings
    pub mappings: usize,
    /// Vector of physical addresses (simulated for compilation)
    pub frames: Vec<usize>, 
    /// Flags
    pub constant: bool, // If true, read-only after creation
}

/// Shared memory permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShmPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Default for ShmPermissions {
    fn default() -> Self {
        Self {
            read: true,
            write: true, 
            execute: false,
        }
    }
}

impl ShmPermissions {
    pub fn readonly() -> Self {
        Self { read: true, write: false, execute: false }
    }
}

/// Global Shared Memory Manager
pub struct ShmManager {
    regions: BTreeMap<ShmId, Arc<Mutex<ShmRegion>>>,
    next_id: AtomicU64,
}

static SHM_MANAGER: Mutex<ShmManager> = Mutex::new(ShmManager::new());

impl ShmManager {
    pub const fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            next_id: AtomicU64::new(1),
        }
    }
    
    fn generate_id(&self) -> ShmId {
        ShmId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

/// Create a new shared memory region
pub fn create_shm(pid: ProcessId, size: usize) -> IpcResult<ShmHandle> {
    if size == 0 {
        return Err(IpcError::InvalidMessage); // Reuse error code for invalid size
    }
    
    // Align size to page boundary (4KB)
    let aligned_size = (size + 4095) & !4095;
    
    let mut manager = SHM_MANAGER.lock();
    let id = manager.generate_id();
    
    let region = ShmRegion {
        id,
        size: aligned_size,
        creator: pid,
        mappings: 0,
        frames: Vec::new(), // In real impl, allocate frames here
        constant: false,
    };
    
    let region_ref = Arc::new(Mutex::new(region));
    manager.regions.insert(id, region_ref.clone());
    
    crate::kprintln!("[IPC] SHM region {} created ({} bytes) by PID {}", id, aligned_size, pid.0);
    
    Ok(ShmHandle {
        id,
        size: aligned_size,
        permissions: ShmPermissions::default(),
        region: region_ref,
    })
}

/// Map shared memory into current process
/// Returns the virtual address where it was mapped
pub fn map_shm(handle: &ShmHandle, pid: ProcessId) -> IpcResult<usize> {
    let mut region = handle.region.lock();
    
    // In a real OS, we would Find a free range in PID's page table
    // and map the physical frames in 'region.frames' to that range.
    // Here we simulate it.
    
    region.mappings += 1;
    let mock_vaddr = 0x8000_0000 + (handle.id.0 as usize * 0x100000);
    
    crate::kprintln!("[IPC] SHM region {} mapped to PID {} at 0x{:x}", handle.id, pid.0, mock_vaddr);
    
    Ok(mock_vaddr)
}

/// Unmap shared memory
pub fn unmap_shm(handle: &ShmHandle, pid: ProcessId, vaddr: usize) -> IpcResult<()> {
    let mut region = handle.region.lock();
    
    if region.mappings > 0 {
        region.mappings -= 1;
    }
    
    crate::kprintln!("[IPC] SHM region {} unmapped from PID {} at 0x{:x}", handle.id, pid.0, vaddr);
    
    // If mappings == 0 and no handles remain, we would free frames.
    // However, handles keep Arc alive, so memory is safe.
    
    Ok(())
}

/// destroy/close handle
pub fn close_shm(handle: ShmHandle) {
    // Drop logic handles Arc decrement
    // When last Arc is dropped, region is removed from map if we had a mechanism to remove it
    // Currently map holds a strong reference, so explicit destroy is needed.
}

/// Explicitly destroy shared memory object (e.g. by creator)
pub fn destroy_shm(id: ShmId, pid: ProcessId) -> IpcResult<()> {
    let mut manager = SHM_MANAGER.lock();
    
    if let Some(region_arc) = manager.regions.get(&id) {
        let region = region_arc.lock();
        if region.creator != pid {
            return Err(IpcError::PermissionDenied);
        }
        // Force unmap or fail if mapped? Standard behavior is mark for deletion.
        crate::kprintln!("[IPC] SHM region {} destroyed by PID {}", id, pid.0);
    } else {
        return Err(IpcError::InvalidMessage); // Not found
    }
    
    manager.regions.remove(&id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shm_creation() {
        let pid = ProcessId(1);
        let handle = create_shm(pid, 1024).expect("Failed to create SHM");
        
        assert!(handle.size >= 1024);
        assert_eq!(handle.id.0, 1);
        
        let vaddr = map_shm(&handle, pid).expect("Failed to map");
        assert!(vaddr > 0);
        
        unmap_shm(&handle, pid, vaddr).expect("Failed to unmap");
    }
}
