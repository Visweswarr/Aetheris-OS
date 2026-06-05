//! Capability Space (CSpace) for per-process capability management
//!
//! Each process has its own capability space that starts empty.
//! Capabilities must be explicitly granted by the Service_Manager.
//!
//! Requirements: 7.1 - Zero capability spawn

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::{Capability, CapabilityId, CapabilityHandle, CapError, CapResult, ProcessId, Permissions};

/// Maximum capabilities per process
pub const MAX_CAPS_PER_PROCESS: usize = 1024;

/// Capability Space for a single process
///
/// **Property 21: Zero Capability Spawn**
/// When created via new(), the capability space is empty.
/// Capabilities must be explicitly granted.
#[derive(Debug, Clone)]
pub struct CapabilitySpace {
    /// Owner process ID
    pub owner_pid: ProcessId,
    /// Capabilities indexed by ID
    capabilities: BTreeMap<CapabilityId, CapabilityHandle>,
    /// Mapping from capability to tree node ID
    tree_mapping: BTreeMap<CapabilityId, CapabilityId>,
    /// Whether this is the root/init process (has special privileges)
    is_root: bool,
}

/// Error types specific to CSpace operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CSpaceError {
    /// CSpace is full
    Full,
    /// Capability not found
    NotFound,
    /// Duplicate capability
    Duplicate,
    /// Invalid operation
    InvalidOperation,
}

impl CapabilitySpace {
    /// Create a new empty capability space for a process
    ///
    /// **Property 21: Zero Capability Spawn**
    /// The new process starts with zero capabilities.
    pub fn new(owner_pid: ProcessId) -> Self {
        Self {
            owner_pid,
            capabilities: BTreeMap::new(),
            tree_mapping: BTreeMap::new(),
            is_root: false,
        }
    }

    /// Create a root capability space (for Service_Manager)
    pub fn new_root(owner_pid: ProcessId) -> Self {
        Self {
            owner_pid,
            capabilities: BTreeMap::new(),
            tree_mapping: BTreeMap::new(),
            is_root: true,
        }
    }

    /// Check if this is an empty capability space
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Get the number of capabilities
    pub fn count(&self) -> usize {
        self.capabilities.len()
    }

    /// Insert a capability into the space
    pub fn insert(&mut self, handle: CapabilityHandle, tree_node_id: CapabilityId) -> CapResult<()> {
        if self.capabilities.len() >= MAX_CAPS_PER_PROCESS {
            return Err(CapError::CSpaceFull { pid: self.owner_pid });
        }

        if self.capabilities.contains_key(&handle.id) {
            return Err(CapError::AlreadyExists);
        }

        self.tree_mapping.insert(handle.id, tree_node_id);
        self.capabilities.insert(handle.id, handle);
        Ok(())
    }

    /// Remove a capability from the space
    pub fn remove(&mut self, cap_id: CapabilityId) -> Option<CapabilityHandle> {
        self.tree_mapping.remove(&cap_id);
        self.capabilities.remove(&cap_id)
    }

    /// Get a capability by ID
    pub fn get(&self, cap_id: CapabilityId) -> Option<&CapabilityHandle> {
        self.capabilities.get(&cap_id)
    }

    /// Check if the process has a specific capability
    pub fn has_capability(&self, cap_id: CapabilityId) -> bool {
        self.capabilities.contains_key(&cap_id)
    }

    /// Find a capability that grants access to a memory address
    pub fn find_memory_cap(&self, addr: u64, required_perms: Permissions) -> Option<&CapabilityHandle> {
        self.capabilities.values().find(|cap| {
            if let Capability::Memory { base, size, perms } = &cap.capability {
                addr >= *base && addr < base + *size as u64 && perms.contains(required_perms)
            } else {
                false
            }
        })
    }

    /// Find an IPC capability for a target process
    ///
    /// **Property 22: IPC Handle Enforcement**
    /// Returns the IPC handle only if it exists.
    pub fn find_ipc_cap(&self, target: ProcessId) -> Option<&CapabilityHandle> {
        self.capabilities.values().find(|cap| {
            if let Capability::Ipc { target: t } = &cap.capability {
                *t == target
            } else {
                false
            }
        })
    }

    /// Find a device capability
    pub fn find_device_cap(&self, device_id: super::DeviceId) -> Option<&CapabilityHandle> {
        self.capabilities.values().find(|cap| {
            if let Capability::Device { device_id: d, .. } = &cap.capability {
                *d == device_id
            } else {
                false
            }
        })
    }

    /// Get all capabilities in this space
    pub fn all_capabilities(&self) -> Vec<&CapabilityHandle> {
        self.capabilities.values().collect()
    }

    /// Get the tree node ID for a capability
    pub fn get_tree_node(&self, cap_id: CapabilityId) -> Option<CapabilityId> {
        self.tree_mapping.get(&cap_id).copied()
    }

    /// Invalidate capabilities based on revoked tree nodes
    pub fn invalidate_from_revocation(&mut self, revoked_tree_ids: &[CapabilityId]) {
        let to_remove: Vec<CapabilityId> = self.tree_mapping
            .iter()
            .filter(|(_, tree_id)| revoked_tree_ids.contains(tree_id))
            .map(|(cap_id, _)| *cap_id)
            .collect();

        for cap_id in to_remove {
            self.remove(cap_id);
        }
    }

    /// Clear all capabilities (for process termination)
    pub fn clear(&mut self) {
        self.capabilities.clear();
        self.tree_mapping.clear();
    }

    /// Check if this is the root process
    pub fn is_root_process(&self) -> bool {
        self.is_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_capability_spawn() {
        // Property 21: New processes start with zero capabilities
        let cspace = CapabilitySpace::new(1);
        assert!(cspace.is_empty());
        assert_eq!(cspace.count(), 0);
    }

    #[test]
    fn test_insert_and_find() {
        let mut cspace = CapabilitySpace::new(1);
        
        let handle = CapabilityHandle {
            id: 1,
            capability: Capability::Memory { 
                base: 0x1000, 
                size: 4096, 
                perms: Permissions::READ_WRITE 
            },
            parent: None,
            generation: 1,
            owner: 1,
        };
        
        assert!(cspace.insert(handle.clone(), 100).is_ok());
        assert!(!cspace.is_empty());
        
        // Should find the memory cap
        let found = cspace.find_memory_cap(0x1000, Permissions::READ);
        assert!(found.is_some());
        
        // Should not find with wrong permissions
        let not_found = cspace.find_memory_cap(0x1000, Permissions::EXECUTE);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_ipc_cap_enforcement() {
        let mut cspace = CapabilitySpace::new(1);
        
        // No IPC cap initially
        assert!(cspace.find_ipc_cap(2).is_none());
        
        // Add IPC cap for process 2
        let handle = CapabilityHandle {
            id: 1,
            capability: Capability::Ipc { target: 2 },
            parent: None,
            generation: 1,
            owner: 1,
        };
        cspace.insert(handle, 100).unwrap();
        
        // Now we can find it
        assert!(cspace.find_ipc_cap(2).is_some());
        // But not for other processes
        assert!(cspace.find_ipc_cap(3).is_none());
    }
}
