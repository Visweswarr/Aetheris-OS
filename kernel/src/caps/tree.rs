//! Capability Tree for tracking parent-child relationships
//!
//! This module implements the capability derivation tree used for
//! cascading revocation. When a parent capability is revoked, all
//! derived children are also invalidated.
//!
//! Requirements: 7.3 - Capability revocation cascade

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};

use super::{CapabilityId, CapError, CapResult};

/// Node in the capability derivation tree
#[derive(Debug, Clone)]
pub struct CapabilityNode {
    /// This node's capability ID
    pub id: CapabilityId,
    /// Parent node ID (None for root capabilities)
    pub parent: Option<CapabilityId>,
    /// Child node IDs
    pub children: Vec<CapabilityId>,
    /// Current generation (incremented on revocation)
    pub generation: u32,
    /// Whether this capability has been revoked
    pub revoked: bool,
    /// Timestamp when created
    pub created_at: u64,
    /// Timestamp when revoked (if applicable)
    pub revoked_at: Option<u64>,
}

impl CapabilityNode {
    /// Create a new capability node
    pub fn new(id: CapabilityId, parent: Option<CapabilityId>) -> Self {
        static TIMESTAMP: AtomicU64 = AtomicU64::new(1);
        Self {
            id,
            parent,
            children: Vec::new(),
            generation: 1,
            revoked: false,
            created_at: TIMESTAMP.fetch_add(1, Ordering::Relaxed),
            revoked_at: None,
        }
    }

    /// Check if this node is valid (not revoked)
    pub fn is_valid(&self) -> bool {
        !self.revoked
    }

    /// Add a child to this node
    pub fn add_child(&mut self, child_id: CapabilityId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove a child from this node
    pub fn remove_child(&mut self, child_id: CapabilityId) {
        self.children.retain(|&id| id != child_id);
    }
}

/// Capability derivation tree
///
/// Tracks parent-child relationships between capabilities for
/// cascading revocation.
pub struct CapabilityTree {
    /// All nodes indexed by capability ID
    nodes: BTreeMap<CapabilityId, CapabilityNode>,
    /// Next capability ID to assign
    next_id: AtomicU64,
    /// Global generation counter
    global_generation: AtomicU32,
    /// Revocation log for audit
    revocation_log: Vec<RevocationEntry>,
}

/// Entry in the revocation log
#[derive(Debug, Clone)]
pub struct RevocationEntry {
    /// Capability that was revoked
    pub capability_id: CapabilityId,
    /// Number of derived capabilities also revoked
    pub cascade_count: usize,
    /// Timestamp of revocation
    pub timestamp: u64,
    /// Reason for revocation
    pub reason: RevocationReason,
}

/// Reason for capability revocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationReason {
    /// Explicit revocation by owner
    Explicit,
    /// Parent was revoked (cascade)
    ParentRevoked,
    /// Process terminated
    ProcessTerminated,
    /// Security violation detected
    SecurityViolation,
    /// Resource cleanup
    ResourceCleanup,
}

impl CapabilityTree {
    /// Create a new capability tree
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            next_id: AtomicU64::new(1),
            global_generation: AtomicU32::new(1),
            revocation_log: Vec::new(),
        }
    }

    /// Create a root capability (no parent)
    pub fn create_root(&mut self) -> CapabilityId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let node = CapabilityNode::new(id, None);
        self.nodes.insert(id, node);
        id
    }

    /// Derive a new capability from a parent
    ///
    /// The derived capability will be invalidated if the parent is revoked.
    pub fn derive(&mut self, parent_id: CapabilityId) -> CapResult<CapabilityId> {
        // Check parent exists and is valid
        let parent = self.nodes.get(&parent_id)
            .ok_or(CapError::InvalidHandle { handle: parent_id, generation: 0 })?;
        
        if parent.revoked {
            return Err(CapError::HandleRevoked { handle: parent_id });
        }

        // Create child node
        let child_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let child = CapabilityNode::new(child_id, Some(parent_id));
        
        // Add child to parent
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.add_child(child_id);
        }
        
        self.nodes.insert(child_id, child);
        Ok(child_id)
    }

    /// Revoke a capability and all its descendants
    ///
    /// **Property 23: Capability Revocation Cascade**
    /// For any capability revocation, all handles derived from the revoked
    /// capability shall become invalid immediately.
    ///
    /// # Returns
    /// List of process IDs affected by the revocation
    pub fn revoke(&mut self, cap_id: CapabilityId, reason: RevocationReason) -> CapResult<Vec<CapabilityId>> {
        let mut revoked = Vec::new();
        self.revoke_recursive(cap_id, reason, &mut revoked)?;
        
        // Log the revocation
        static TIMESTAMP: AtomicU64 = AtomicU64::new(1);
        self.revocation_log.push(RevocationEntry {
            capability_id: cap_id,
            cascade_count: revoked.len().saturating_sub(1),
            timestamp: TIMESTAMP.fetch_add(1, Ordering::Relaxed),
            reason,
        });
        
        Ok(revoked)
    }

    /// Recursive revocation helper
    fn revoke_recursive(
        &mut self, 
        cap_id: CapabilityId, 
        reason: RevocationReason,
        revoked: &mut Vec<CapabilityId>
    ) -> CapResult<()> {
        // Get the node (we need to collect children first to avoid borrow issues)
        let children: Vec<CapabilityId> = {
            let node = self.nodes.get(&cap_id)
                .ok_or(CapError::InvalidHandle { handle: cap_id, generation: 0 })?;
            
            if node.revoked {
                return Ok(()); // Already revoked
            }
            
            node.children.clone()
        };

        // Revoke all children first (depth-first)
        for child_id in children {
            self.revoke_recursive(child_id, RevocationReason::ParentRevoked, revoked)?;
        }

        // Now revoke this node
        if let Some(node) = self.nodes.get_mut(&cap_id) {
            node.revoked = true;
            node.generation = self.global_generation.fetch_add(1, Ordering::Relaxed);
            static TIMESTAMP: AtomicU64 = AtomicU64::new(1);
            node.revoked_at = Some(TIMESTAMP.fetch_add(1, Ordering::Relaxed));
            revoked.push(cap_id);
        }

        // Remove from parent's children list
        if let Some(node) = self.nodes.get(&cap_id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.remove_child(cap_id);
                }
            }
        }

        Ok(())
    }

    /// Validate that a capability is still valid
    pub fn validate(&self, cap_id: CapabilityId) -> CapResult<()> {
        let node = self.nodes.get(&cap_id)
            .ok_or(CapError::InvalidHandle { handle: cap_id, generation: 0 })?;
        
        if node.revoked {
            return Err(CapError::HandleRevoked { handle: cap_id });
        }
        
        // Also check all ancestors
        let mut current = node.parent;
        while let Some(parent_id) = current {
            let parent = self.nodes.get(&parent_id)
                .ok_or(CapError::InvalidHandle { handle: parent_id, generation: 0 })?;
            
            if parent.revoked {
                return Err(CapError::HandleRevoked { handle: parent_id });
            }
            
            current = parent.parent;
        }
        
        Ok(())
    }

    /// Get the current generation of a capability
    pub fn get_generation(&self, cap_id: CapabilityId) -> Option<u32> {
        self.nodes.get(&cap_id).map(|n| n.generation)
    }

    /// Get statistics about the tree
    pub fn stats(&self) -> TreeStats {
        let total = self.nodes.len();
        let revoked = self.nodes.values().filter(|n| n.revoked).count();
        let roots = self.nodes.values().filter(|n| n.parent.is_none()).count();
        
        TreeStats {
            total_capabilities: total,
            active_capabilities: total - revoked,
            revoked_capabilities: revoked,
            root_capabilities: roots,
            revocation_log_size: self.revocation_log.len(),
        }
    }

    /// Get revocation log entries
    pub fn get_revocation_log(&self) -> &[RevocationEntry] {
        &self.revocation_log
    }

    /// Clear old revocation log entries
    pub fn trim_revocation_log(&mut self, keep_last: usize) {
        if self.revocation_log.len() > keep_last {
            let start = self.revocation_log.len() - keep_last;
            self.revocation_log = self.revocation_log[start..].to_vec();
        }
    }
}

/// Statistics about the capability tree
#[derive(Debug, Clone, Copy)]
pub struct TreeStats {
    pub total_capabilities: usize,
    pub active_capabilities: usize,
    pub revoked_capabilities: usize,
    pub root_capabilities: usize,
    pub revocation_log_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_derive() {
        let mut tree = CapabilityTree::new();
        
        let root = tree.create_root();
        assert!(tree.validate(root).is_ok());
        
        let child = tree.derive(root).unwrap();
        assert!(tree.validate(child).is_ok());
    }

    #[test]
    fn test_revocation_cascade() {
        let mut tree = CapabilityTree::new();
        
        // Create a tree: root -> child1 -> grandchild
        //                    -> child2
        let root = tree.create_root();
        let child1 = tree.derive(root).unwrap();
        let child2 = tree.derive(root).unwrap();
        let grandchild = tree.derive(child1).unwrap();
        
        // All should be valid
        assert!(tree.validate(root).is_ok());
        assert!(tree.validate(child1).is_ok());
        assert!(tree.validate(child2).is_ok());
        assert!(tree.validate(grandchild).is_ok());
        
        // Revoke root
        let revoked = tree.revoke(root, RevocationReason::Explicit).unwrap();
        assert_eq!(revoked.len(), 4); // root, child1, child2, grandchild
        
        // All should now be invalid
        assert!(tree.validate(root).is_err());
        assert!(tree.validate(child1).is_err());
        assert!(tree.validate(child2).is_err());
        assert!(tree.validate(grandchild).is_err());
    }
}
