//! Task ID module for Polymera OS scheduler
//!
//! Provides task identification types.

/// Task identifier type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TaskId(pub u64);

impl TaskId {
    /// Create a new TaskId
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Alias for `as_u64`. Retained for call-site compatibility.
    pub const fn value(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for TaskId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Task({})", self.0)
    }
}

impl From<u64> for TaskId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<TaskId> for u64 {
    fn from(tid: TaskId) -> Self {
        tid.0
    }
}
