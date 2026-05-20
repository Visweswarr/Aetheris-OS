//! WASM Sandbox Implementation
//!
//! Provides memory-limited execution environment for WASM drivers.
//! Requirement: 12.4 - 64MB memory limit

use super::SANDBOX_MEMORY_LIMIT;

/// Sandbox configuration
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub memory_limit: usize,
    pub stack_size: usize,
    pub execution_timeout_ms: u64,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit: SANDBOX_MEMORY_LIMIT,
            stack_size: 1024 * 1024,    // 1MB stack
            execution_timeout_ms: 5000, // 5 second timeout
        }
    }
}

/// Sandbox instance for a single driver
pub struct Sandbox {
    config: SandboxConfig,
    allocated_memory: usize,
}

impl Sandbox {
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            allocated_memory: 0,
        }
    }

    /// Allocate memory within sandbox limits
    pub fn allocate(&mut self, size: usize) -> Result<usize, SandboxError> {
        if self.allocated_memory + size > self.config.memory_limit {
            return Err(SandboxError::MemoryLimitExceeded);
        }

        let offset = self.allocated_memory;
        self.allocated_memory += size;
        Ok(offset)
    }

    /// Free memory
    pub fn free(&mut self, size: usize) {
        self.allocated_memory = self.allocated_memory.saturating_sub(size);
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.allocated_memory
    }

    /// Check if allocation would exceed limit
    pub fn can_allocate(&self, size: usize) -> bool {
        self.allocated_memory + size <= self.config.memory_limit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxError {
    MemoryLimitExceeded,
    StackOverflow,
    ExecutionTimeout,
}
