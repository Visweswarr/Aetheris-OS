//! Priority Inheritance Lock
//! 
//! This module implements a basic lock with priority inheritance to prevent
//! high-priority task starvation when waiting for locks held by low-priority tasks.

use core::sync::atomic::{AtomicU64, Ordering};
use crate::sched::task_id::TaskId;
use crate::sched::task::TaskPriority;
use crate::klog;
use crate::log::tags;

/// Lock with priority inheritance
pub struct PriorityInheritanceLock {
    /// Current owner of the lock (0 = unlocked)
    owner: AtomicU64,
    /// Original priority of the owner before inheritance
    original_priority: AtomicU64,
    /// Whether priority inheritance is enabled
    inheritance_enabled: bool,
}

impl PriorityInheritanceLock {
    /// Create a new lock
    pub fn new() -> Self {
        Self {
            owner: AtomicU64::new(0),
            original_priority: AtomicU64::new(0),
            inheritance_enabled: true,
        }
    }
    
    /// Create a new lock with inheritance disabled
    pub fn new_without_inheritance() -> Self {
        Self {
            owner: AtomicU64::new(0),
            original_priority: AtomicU64::new(0),
            inheritance_enabled: false,
        }
    }
    
    /// Try to acquire the lock
    pub fn try_lock(&self, task_id: TaskId, priority: TaskPriority) -> bool {
        let current_owner = self.owner.load(Ordering::Acquire);
        
        if current_owner == 0 {
            // Lock is free, acquire it
            if self.owner.compare_exchange(0, task_id.value(), Ordering::Acquire, Ordering::Relaxed).is_ok() {
                // Store original priority for restoration
                self.original_priority.store(priority as u64, Ordering::Relaxed);
                return true;
            }
        }
        
        false
    }
    
    /// Acquire the lock with priority inheritance
    pub fn lock(&self, task_id: TaskId, priority: TaskPriority) -> Result<(), LockError> {
        loop {
            if self.try_lock(task_id, priority) {
                return Ok(());
            }
            
            // Lock is held, check if we need priority inheritance
            if self.inheritance_enabled {
                self.handle_priority_inheritance(task_id, priority)?;
            }
            
            // Wait for lock to become available
            self.wait_for_lock();
        }
    }
    
    /// Release the lock and restore original priority
    pub fn unlock(&self, task_id: TaskId) -> Result<(), LockError> {
        let current_owner = self.owner.load(Ordering::Acquire);
        
        if current_owner != task_id.value() {
            return Err(LockError::NotOwner);
        }
        
        // Restore original priority if inheritance was used
        if self.inheritance_enabled {
            self.restore_original_priority(task_id);
        }
        
        // Release the lock
        self.owner.store(0, Ordering::Release);
        
        Ok(())
    }
    
    /// Handle priority inheritance when a high-priority task waits
    fn handle_priority_inheritance(&self, waiting_task_id: TaskId, waiting_priority: TaskPriority) -> Result<(), LockError> {
        let current_owner = self.owner.load(Ordering::Acquire);
        if current_owner == 0 {
            return Ok(());
        }
        
        let owner_task_id = TaskId::new_with_value(current_owner);
        let owner_priority = self.get_task_priority(owner_task_id);
        
        // Check if inheritance is needed (waiting task has higher priority)
        if waiting_priority > owner_priority {
            klog!(INFO, [tags::SCHED], 
                "Priority inheritance: boosting task {} from {:?} to {:?}",
                owner_task_id.value(), owner_priority, waiting_priority);
            
            // Boost owner's priority to match waiting task
            self.boost_task_priority(owner_task_id, waiting_priority);
        }
        
        Ok(())
    }
    
    /// Boost a task's priority
    fn boost_task_priority(&self, task_id: TaskId, new_priority: TaskPriority) {
        // This would integrate with the actual scheduler
        // For now, we just log the boost
        klog!(INFO, [tags::SCHED], 
            "Boosting task {} priority to {:?}", task_id.value(), new_priority);
    }
    
    /// Restore a task's original priority
    fn restore_original_priority(&self, task_id: TaskId) {
        let original_priority = self.original_priority.load(Ordering::Relaxed);
        
        klog!(INFO, [tags::SCHED], 
            "Restoring task {} to original priority {:?}", 
            task_id.value(), 
            TaskPriority::from_u64(original_priority));
        
        // This would integrate with the actual scheduler
        // For now, we just log the restoration
    }
    
    /// Get the current priority of a task
    fn get_task_priority(&self, task_id: TaskId) -> TaskPriority {
        // This would query the actual scheduler
        // For now, return a default priority
        TaskPriority::Normal
    }
    
    /// Wait for the lock to become available
    fn wait_for_lock(&self) {
        // Simple busy wait for now
        // In a real implementation, this would block the task
        core::hint::spin_loop();
    }
    
    /// Check if the lock is currently held
    pub fn is_locked(&self) -> bool {
        self.owner.load(Ordering::Relaxed) != 0
    }
    
    /// Get the current owner of the lock
    pub fn get_owner(&self) -> Option<TaskId> {
        let owner = self.owner.load(Ordering::Relaxed);
        if owner != 0 {
            Some(TaskId::new_with_value(owner))
        } else {
            None
        }
    }
    
    /// Enable or disable priority inheritance
    pub fn set_inheritance_enabled(&mut self, enabled: bool) {
        self.inheritance_enabled = enabled;
    }
}

/// Lock-related errors
#[derive(Debug, Clone, PartialEq)]
pub enum LockError {
    /// Task is not the owner of the lock
    NotOwner,
    /// Lock operation failed
    OperationFailed,
    /// Priority inheritance failed
    InheritanceFailed,
}

impl core::fmt::Display for LockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LockError::NotOwner => write!(f, "Task is not the owner of the lock"),
            LockError::OperationFailed => write!(f, "Lock operation failed"),
            LockError::InheritanceFailed => write!(f, "Priority inheritance failed"),
        }
    }
}

/// Extension trait for TaskId to support creation with specific values
impl TaskId {
    /// Create a TaskId with a specific value (for testing)
    pub fn new_with_value(value: u64) -> Self {
        Self(value)
    }
}

/// Extension trait for TaskPriority to support conversion from u64
impl TaskPriority {
    /// Convert from u64 to TaskPriority
    pub fn from_u64(value: u64) -> Self {
        match value {
            0 => TaskPriority::Low,
            1 => TaskPriority::Normal,
            2 => TaskPriority::High,
            3 => TaskPriority::RealTime,
            _ => TaskPriority::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lock_creation() {
        let lock = PriorityInheritanceLock::new();
        assert!(!lock.is_locked());
        assert!(lock.get_owner().is_none());
    }
    
    #[test]
    fn test_basic_lock_acquire() {
        let lock = PriorityInheritanceLock::new();
        let task_id = TaskId::new();
        let priority = TaskPriority::Normal;
        
        assert!(lock.try_lock(task_id, priority));
        assert!(lock.is_locked());
        assert_eq!(lock.get_owner(), Some(task_id));
    }
    
    #[test]
    fn test_lock_release() {
        let lock = PriorityInheritanceLock::new();
        let task_id = TaskId::new();
        let priority = TaskPriority::Normal;
        
        lock.try_lock(task_id, priority).unwrap();
        assert!(lock.unlock(task_id).is_ok());
        assert!(!lock.is_locked());
        assert!(lock.get_owner().is_none());
    }
    
    #[test]
    fn test_priority_inheritance_enabled() {
        let lock = PriorityInheritanceLock::new();
        assert!(lock.inheritance_enabled);
        
        let mut lock_disabled = PriorityInheritanceLock::new();
        lock_disabled.set_inheritance_enabled(false);
        assert!(!lock_disabled.inheritance_enabled);
    }
}



