//! Capability Manager - seL4-style capability-based security
//!
//! This module implements the main CapabilityManager struct that coordinates
//! capability spaces, the derivation tree, and IPC validation.
//!
//! Requirements: 7.1, 7.2, 7.3, 7.4

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use super::tree::{CapabilityTree, RevocationReason};
use super::cspace::CapabilitySpace;
use super::{
    Capability, CapabilityHandle, CapabilityId, CapError, CapResult, 
    DeviceOps, IpcHandle, Permissions, ProcessId
};

/// Capability Manager - seL4-style capability-based security
pub struct CapabilityManager {
    /// Capability derivation tree
    cap_tree: CapabilityTree,
    /// Per-process capability spaces
    cspaces: BTreeMap<ProcessId, CapabilitySpace>,
    /// Handle ID counter
    next_handle_id: AtomicU64,
    /// Revocation audit log
    revocation_log: Vec<RevocationLogEntry>,
}

/// Entry in the revocation audit log
#[derive(Debug, Clone)]
pub struct RevocationLogEntry {
    pub timestamp: u64,
    pub revoker_pid: ProcessId,
    pub revoked_cap: CapabilityId,
    pub affected_pids: Vec<ProcessId>,
    pub reason: RevocationReason,
}

impl CapabilityManager {
    /// Create a new capability manager
    pub const fn new() -> Self {
        Self {
            cap_tree: CapabilityTree::new(),
            cspaces: BTreeMap::new(),
            next_handle_id: AtomicU64::new(1),
            revocation_log: Vec::new(),
        }
    }

    /// Create initial empty capability space for new process
    ///
    /// **Property 21: Zero Capability Spawn**
    /// For any newly spawned service, its capability space shall be empty
    /// until the Service_Manager explicitly grants capabilities.
    pub fn create_cspace(&mut self, pid: ProcessId) -> &CapabilitySpace {
        let cspace = CapabilitySpace::new(pid);
        self.cspaces.insert(pid, cspace);
        self.cspaces.get(&pid).unwrap()
    }

    /// Create root capability space for Service_Manager (PID 1)
    pub fn create_root_cspace(&mut self, pid: ProcessId) -> &CapabilitySpace {
        let cspace = CapabilitySpace::new_root(pid);
        self.cspaces.insert(pid, cspace);
        self.cspaces.get(&pid).unwrap()
    }

    /// Grant capability from one process to another
    ///
    /// # Arguments
    /// * `from` - Source process (must own the capability or be root)
    /// * `to` - Destination process
    /// * `cap` - The capability to grant
    ///
    /// # Returns
    /// The new capability handle for the destination process
    pub fn grant(
        &mut self,
        from: ProcessId,
        to: ProcessId,
        cap: Capability,
    ) -> CapResult<CapabilityHandle> {
        // Verify source process exists
        let from_cspace = self.cspaces.get(&from)
            .ok_or(CapError::ProcessNotFound { pid: from })?;
        
        // Only root process (Service_Manager) can grant capabilities directly
        // Or process must already have a compatible capability to derive
        if !from_cspace.is_root_process() {
            // For non-root, they must already have a matching capability
            let has_compatible = match &cap {
                Capability::Memory { base, size, perms } => {
                    from_cspace.find_memory_cap(*base, *perms).is_some()
                }
                Capability::Ipc { target } => {
                    from_cspace.find_ipc_cap(*target).is_some()
                }
                Capability::Device { device_id, .. } => {
                    from_cspace.find_device_cap(*device_id).is_some()
                }
                _ => false,
            };
            
            if !has_compatible {
                return Err(CapError::InsufficientPermissions {
                    required: Permissions::ALL,
                    actual: Permissions::NONE,
                });
            }
        }

        // Verify destination process exists
        if !self.cspaces.contains_key(&to) {
            return Err(CapError::ProcessNotFound { pid: to });
        }

        // Create capability tree node
        let tree_node_id = self.cap_tree.create_root();

        // Create capability handle
        let handle_id = self.next_handle_id.fetch_add(1, Ordering::Relaxed);
        let handle = CapabilityHandle {
            id: handle_id,
            capability: cap,
            parent: None,
            generation: 1,
            owner: to,
        };

        // Insert into destination's cspace
        let to_cspace = self.cspaces.get_mut(&to)
            .ok_or(CapError::ProcessNotFound { pid: to })?;
        to_cspace.insert(handle.clone(), tree_node_id)?;

        Ok(handle)
    }

    /// Revoke capability and all derived handles
    ///
    /// **Property 23: Capability Revocation Cascade**
    /// For any capability revocation, all handles derived from the revoked
    /// capability shall become invalid immediately.
    pub fn revoke(&mut self, cap: CapabilityHandle) -> CapResult<Vec<ProcessId>> {
        // Get the tree node for this capability
        let cspace = self.cspaces.get(&cap.owner)
            .ok_or(CapError::ProcessNotFound { pid: cap.owner })?;
        
        let tree_node_id = cspace.get_tree_node(cap.id)
            .ok_or(CapError::InvalidHandle { handle: cap.id, generation: cap.generation })?;

        // Revoke in the tree (cascading)
        let revoked_nodes = self.cap_tree.revoke(tree_node_id, RevocationReason::Explicit)?;

        // Propagate to all cspaces
        let mut affected_pids = Vec::new();
        for (pid, cspace) in self.cspaces.iter_mut() {
            let count_before = cspace.count();
            cspace.invalidate_from_revocation(&revoked_nodes);
            if cspace.count() < count_before {
                affected_pids.push(*pid);
            }
        }

        // Log the revocation
        static TIMESTAMP: AtomicU64 = AtomicU64::new(1);
        self.revocation_log.push(RevocationLogEntry {
            timestamp: TIMESTAMP.fetch_add(1, Ordering::Relaxed),
            revoker_pid: cap.owner,
            revoked_cap: cap.id,
            affected_pids: affected_pids.clone(),
            reason: RevocationReason::Explicit,
        });

        Ok(affected_pids)
    }

    /// Validate IPC handle for inter-service communication
    ///
    /// **Property 22: IPC Handle Enforcement**
    /// For any IPC send operation, the sender must possess a valid IPC handle
    /// for the destination, and sends without valid handles shall be rejected.
    pub fn validate_ipc(
        &self,
        from: ProcessId,
        to: ProcessId,
        handle: &IpcHandle,
    ) -> CapResult<()> {
        // Get source cspace
        let from_cspace = self.cspaces.get(&from)
            .ok_or(CapError::ProcessNotFound { pid: from })?;

        // Check that handle matches the IPC request
        if handle.from != from || handle.to != to {
            return Err(CapError::IpcDenied { from, to });
        }

        // Find IPC capability for the target
        let ipc_cap = from_cspace.find_ipc_cap(to)
            .ok_or(CapError::IpcDenied { from, to })?;

        // Verify the handle ID matches
        if ipc_cap.id != handle.id {
            return Err(CapError::InvalidHandle { 
                handle: handle.id, 
                generation: handle.generation 
            });
        }

        // Verify in the tree that the capability is still valid
        if let Some(tree_node) = from_cspace.get_tree_node(ipc_cap.id) {
            self.cap_tree.validate(tree_node)?;
        }

        Ok(())
    }

    /// Check W^X memory protection before mapping
    ///
    /// **Property 24: W^X Memory Protection**
    /// For any memory region, reject attempts to make the region both
    /// writable and executable simultaneously.
    pub fn check_wxorx(
        &self,
        pid: ProcessId,
        _addr: u64,
        prot: Permissions,
    ) -> CapResult<()> {
        // W^X check: cannot be both writable and executable
        if !prot.is_wxorx() {
            return Err(CapError::WxViolation { address: _addr });
        }

        // Get cspace
        let cspace = self.cspaces.get(&pid)
            .ok_or(CapError::ProcessNotFound { pid })?;

        // Check if any existing capability for overlapping region would create W^X violation
        // This is a simplified check - real implementation would check page table mappings
        for cap in cspace.all_capabilities() {
            if let Capability::Memory { base, size, perms } = &cap.capability {
                // Check overlap
                if _addr >= *base && _addr < base + *size as u64 {
                    // Combined permissions would violate W^X?
                    let combined = Permissions(perms.0 | prot.0);
                    if !combined.is_wxorx() {
                        return Err(CapError::WxViolation { address: _addr });
                    }
                }
            }
        }

        Ok(())
    }

    /// Clean up when a process terminates
    pub fn cleanup_process(&mut self, pid: ProcessId) {
        if let Some(cspace) = self.cspaces.get(&pid) {
            // Get all tree nodes for this process's capabilities
            let tree_nodes: Vec<CapabilityId> = cspace.all_capabilities()
                .iter()
                .filter_map(|cap| cspace.get_tree_node(cap.id))
                .collect();

            // Revoke all (this will cascade to derived capabilities)
            for node_id in tree_nodes {
                let _ = self.cap_tree.revoke(node_id, RevocationReason::ProcessTerminated);
            }
        }

        // Remove the cspace
        self.cspaces.remove(&pid);
    }

    /// Get capability space for a process
    pub fn get_cspace(&self, pid: ProcessId) -> Option<&CapabilitySpace> {
        self.cspaces.get(&pid)
    }

    /// Get mutable capability space for a process
    pub fn get_cspace_mut(&mut self, pid: ProcessId) -> Option<&mut CapabilitySpace> {
        self.cspaces.get_mut(&pid)
    }

    /// Get statistics
    pub fn stats(&self) -> CapabilityManagerStats {
        let tree_stats = self.cap_tree.stats();
        CapabilityManagerStats {
            process_count: self.cspaces.len(),
            total_capabilities: tree_stats.total_capabilities,
            active_capabilities: tree_stats.active_capabilities,
            revoked_capabilities: tree_stats.revoked_capabilities,
            revocation_log_size: self.revocation_log.len(),
        }
    }
}

/// Statistics for the capability manager
#[derive(Debug, Clone, Copy)]
pub struct CapabilityManagerStats {
    pub process_count: usize,
    pub total_capabilities: usize,
    pub active_capabilities: usize,
    pub revoked_capabilities: usize,
    pub revocation_log_size: usize,
}

/// Global capability manager instance
pub static CAPABILITY_MANAGER: Mutex<CapabilityManager> = Mutex::new(CapabilityManager::new());

/// Create capability space for a new process
pub fn create_cspace(pid: ProcessId) {
    CAPABILITY_MANAGER.lock().create_cspace(pid);
}

/// Grant a capability from one process to another
pub fn grant(from: ProcessId, to: ProcessId, cap: Capability) -> CapResult<CapabilityHandle> {
    CAPABILITY_MANAGER.lock().grant(from, to, cap)
}

/// Validate an IPC operation
pub fn validate_ipc(from: ProcessId, to: ProcessId, handle: &IpcHandle) -> CapResult<()> {
    CAPABILITY_MANAGER.lock().validate_ipc(from, to, handle)
}

/// Check W^X for a memory operation
pub fn check_wxorx(pid: ProcessId, addr: u64, prot: Permissions) -> CapResult<()> {
    CAPABILITY_MANAGER.lock().check_wxorx(pid, addr, prot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_capability_spawn() {
        let mut mgr = CapabilityManager::new();
        
        // Create cspace for new process
        let cspace = mgr.create_cspace(1);
        
        // Property 21: Should be empty
        assert!(cspace.is_empty());
    }

    #[test]
    fn test_grant_from_root() {
        let mut mgr = CapabilityManager::new();
        
        // Create root (Service_Manager) and regular process
        mgr.create_root_cspace(1);
        mgr.create_cspace(2);
        
        // Root can grant capabilities
        let cap = Capability::Memory {
            base: 0x1000,
            size: 4096,
            perms: Permissions::READ_WRITE,
        };
        
        let result = mgr.grant(1, 2, cap);
        assert!(result.is_ok());
        
        // Process 2 should now have the capability
        let cspace = mgr.get_cspace(2).unwrap();
        assert!(!cspace.is_empty());
    }

    #[test]
    fn test_ipc_validation() {
        let mut mgr = CapabilityManager::new();
        
        mgr.create_root_cspace(1);
        mgr.create_cspace(2);
        mgr.create_cspace(3);
        
        // Grant IPC cap from 2 to 3
        let ipc_cap = Capability::Ipc { target: 3 };
        let handle = mgr.grant(1, 2, ipc_cap).unwrap();
        
        // Valid IPC
        let ipc_handle = IpcHandle {
            id: handle.id,
            from: 2,
            to: 3,
            generation: 1,
        };
        assert!(mgr.validate_ipc(2, 3, &ipc_handle).is_ok());
        
        // Invalid IPC (wrong target)
        let bad_handle = IpcHandle {
            id: handle.id,
            from: 2,
            to: 4, // Wrong target
            generation: 1,
        };
        assert!(mgr.validate_ipc(2, 4, &bad_handle).is_err());
    }

    #[test]
    fn test_wxorx_enforcement() {
        let mut mgr = CapabilityManager::new();
        mgr.create_cspace(1);
        
        // Valid: Read + Write
        assert!(mgr.check_wxorx(1, 0x1000, Permissions::READ_WRITE).is_ok());
        
        // Valid: Read + Execute
        assert!(mgr.check_wxorx(1, 0x1000, Permissions::READ_EXECUTE).is_ok());
        
        // Invalid: Write + Execute
        let wx = Permissions(Permissions::WRITE.0 | Permissions::EXECUTE.0);
        assert!(mgr.check_wxorx(1, 0x1000, wx).is_err());
    }
}
