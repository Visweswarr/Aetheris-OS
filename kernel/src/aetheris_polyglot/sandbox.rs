//! Execution Sandbox
//!
//! Provides isolated execution environments with resource limits and capability
//! restrictions for running polyglot applications.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::security::CapToken;
use super::backend::{LanguageBackend, LanguageType, LoadedModule, RuntimeError, ResourceType, StackFrame, DebugInfo};

/// Unique identifier for a sandbox
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SandboxId(pub u64);

impl SandboxId {
    /// Generate a new unique sandbox ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        SandboxId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for SandboxId {
    fn default() -> Self {
        Self::new()
    }
}

/// Sandbox execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxState {
    /// Sandbox created but not yet running
    Created,
    /// Sandbox is actively executing
    Running,
    /// Sandbox is suspended (e.g., exceeded CPU quota)
    Suspended,
    /// Sandbox has been terminated
    Terminated,
}

/// Resource limits for a sandbox
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory in bytes
    pub max_memory_bytes: usize,
    /// Maximum CPU time in milliseconds
    pub max_cpu_time_ms: u64,
    /// Maximum file descriptors
    pub max_file_descriptors: u32,
    /// Maximum network connections
    pub max_network_connections: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64 MB
            max_cpu_time_ms: 30_000,             // 30 seconds
            max_file_descriptors: 64,
            max_network_connections: 16,
        }
    }
}

/// Current resource usage tracking
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory_bytes: usize,
    /// Peak memory usage in bytes (for reporting)
    pub peak_memory_bytes: usize,
    /// Cumulative CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// Open file descriptors
    pub file_descriptors: u32,
    /// Peak file descriptors (for reporting)
    pub peak_file_descriptors: u32,
    /// Active network connections
    pub network_connections: u32,
    /// Syscall count
    pub syscall_count: u64,
    /// Last update timestamp (milliseconds)
    pub last_update_ms: u64,
    /// Creation timestamp (milliseconds)
    pub created_at_ms: u64,
}

/// Resource usage report for DAO reporting
#[derive(Debug, Clone)]
pub struct ResourceReport {
    /// Sandbox identifier
    pub sandbox_id: SandboxId,
    /// Language type
    pub language: LanguageType,
    /// Final memory usage
    pub memory_bytes: usize,
    /// Peak memory usage
    pub peak_memory_bytes: usize,
    /// Total CPU time consumed
    pub cpu_time_ms: u64,
    /// Total syscalls made
    pub syscall_count: u64,
    /// Sandbox lifetime in milliseconds
    pub lifetime_ms: u64,
    /// Final state
    pub final_state: SandboxState,
    /// Resource limit violations count
    pub violations: u32,
}

// ============================================================================
// Debugging Support (Task 10)
// ============================================================================

/// Crash dump captured when an application crashes
#[derive(Debug, Clone)]
pub struct CrashDump {
    /// Sandbox ID where crash occurred
    pub sandbox_id: SandboxId,
    /// Timestamp of the crash (ms)
    pub timestamp_ms: u64,
    /// Stack frames at time of crash
    pub stack_frames: Vec<StackFrame>,
    /// Register state at time of crash
    pub register_state: RegisterState,
    /// Crash reason/message
    pub reason: String,
    /// Memory snapshot (limited to relevant regions)
    pub memory_snapshot: Vec<MemorySnapshot>,
}

/// CPU register state at time of crash
#[derive(Debug, Clone, Default)]
pub struct RegisterState {
    /// Program counter / instruction pointer
    pub pc: u64,
    /// Stack pointer
    pub sp: u64,
    /// Frame pointer
    pub fp: u64,
    /// General purpose registers (architecture-dependent)
    pub general: [u64; 16],
    /// Flags register
    pub flags: u64,
}

/// Memory snapshot for crash dump
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Start address
    pub address: u64,
    /// Memory contents
    pub data: Vec<u8>,
    /// Region name
    pub region_name: String,
}

/// Syscall trace event
#[derive(Debug, Clone)]
pub struct SyscallTraceEvent {
    /// Sandbox ID
    pub sandbox_id: SandboxId,
    /// Timestamp (ms)
    pub timestamp_ms: u64,
    /// Syscall number
    pub syscall_number: u32,
    /// Syscall name
    pub syscall_name: String,
    /// Arguments (as raw values)
    pub arguments: Vec<u64>,
    /// Return value (if completed)
    pub return_value: Option<i64>,
    /// Duration in microseconds
    pub duration_us: u64,
}

/// Memory inspection result
#[derive(Debug, Clone)]
pub struct MemoryInspection {
    /// Start address
    pub address: u64,
    /// Requested size
    pub size: usize,
    /// Actual data read
    pub data: Vec<u8>,
    /// Whether the region is readable
    pub readable: bool,
}

/// Memory inspection error
#[derive(Debug, Clone)]
pub enum MemoryInspectionError {
    /// Address out of bounds
    OutOfBounds { address: u64, size: usize },
    /// Region not readable
    NotReadable { address: u64 },
    /// Write attempt rejected (read-only inspection)
    WriteRejected { address: u64 },
    /// Sandbox not found
    SandboxNotFound,
}

/// Execution sandbox - isolated environment for application execution
pub struct ExecutionSandbox {
    /// Unique sandbox identifier
    pub id: SandboxId,
    /// Language type for this sandbox
    pub language: LanguageType,
    /// Loaded application module
    pub module: Option<LoadedModule>,
    /// Resource limits
    pub limits: ResourceLimits,
    /// Current resource usage
    pub usage: ResourceUsage,
    /// Capability tokens held by this sandbox
    pub capabilities: Vec<CapToken>,
    /// Sandbox state
    pub state: SandboxState,
    /// Creation timestamp (milliseconds)
    pub created_at: u64,
    /// Resource limit violation count
    pub violations: u32,
    // ========================================================================
    // Debugging Support (Task 10)
    // ========================================================================
    /// Whether syscall tracing is enabled
    pub tracing_enabled: bool,
    /// Syscall trace events (when tracing is enabled)
    pub syscall_traces: Vec<SyscallTraceEvent>,
    /// Crash dump (if sandbox crashed)
    pub crash_dump: Option<CrashDump>,
    /// Simulated memory regions for inspection (address -> data)
    pub memory_regions: BTreeMap<u64, Vec<u8>>,
}

impl ExecutionSandbox {
    /// Create a new execution sandbox with the given limits
    pub fn new(language: LanguageType, limits: ResourceLimits) -> Self {
        let now = crate::security::get_current_time_ms();
        Self {
            id: SandboxId::new(),
            language,
            module: None,
            limits,
            usage: ResourceUsage {
                last_update_ms: now,
                created_at_ms: now,
                ..Default::default()
            },
            capabilities: Vec::new(),
            state: SandboxState::Created,
            created_at: now,
            violations: 0,
            // Debugging support
            tracing_enabled: false,
            syscall_traces: Vec::new(),
            crash_dump: None,
            memory_regions: BTreeMap::new(),
        }
    }

    /// Create a sandbox with default resource limits
    pub fn with_defaults(language: LanguageType) -> Self {
        Self::new(language, ResourceLimits::default())
    }

    /// Load a module into this sandbox
    pub fn load_module(&mut self, module: LoadedModule) {
        self.module = Some(module);
    }

    /// Start execution of the sandbox
    pub fn start(&mut self) -> Result<(), RuntimeError> {
        if self.state != SandboxState::Created {
            return Err(RuntimeError::SandboxCreationFailed(
                "Sandbox not in Created state".into()
            ));
        }
        self.state = SandboxState::Running;
        Ok(())
    }

    /// Suspend the sandbox
    pub fn suspend(&mut self) {
        if self.state == SandboxState::Running {
            self.state = SandboxState::Suspended;
        }
    }

    /// Resume a suspended sandbox
    pub fn resume(&mut self) -> Result<(), RuntimeError> {
        if self.state != SandboxState::Suspended {
            return Err(RuntimeError::SandboxCreationFailed(
                "Sandbox not in Suspended state".into()
            ));
        }
        self.state = SandboxState::Running;
        Ok(())
    }

    /// Terminate the sandbox
    pub fn terminate(&mut self) -> ResourceUsage {
        self.state = SandboxState::Terminated;
        self.usage.clone()
    }

    /// Check if a memory allocation would exceed limits
    pub fn check_memory_limit(&self, additional_bytes: usize) -> Result<(), RuntimeError> {
        let new_total = self.usage.memory_bytes.saturating_add(additional_bytes);
        if new_total > self.limits.max_memory_bytes {
            return Err(RuntimeError::ResourceLimitExceeded {
                resource: ResourceType::Memory,
                limit: self.limits.max_memory_bytes as u64,
                requested: new_total as u64,
            });
        }
        Ok(())
    }

    /// Check if opening a file descriptor would exceed limits
    pub fn check_fd_limit(&self) -> Result<(), RuntimeError> {
        if self.usage.file_descriptors >= self.limits.max_file_descriptors {
            return Err(RuntimeError::ResourceLimitExceeded {
                resource: ResourceType::FileDescriptors,
                limit: self.limits.max_file_descriptors as u64,
                requested: (self.usage.file_descriptors + 1) as u64,
            });
        }
        Ok(())
    }

    /// Check if CPU time limit has been exceeded
    pub fn check_cpu_limit(&self) -> Result<(), RuntimeError> {
        if self.usage.cpu_time_ms >= self.limits.max_cpu_time_ms {
            return Err(RuntimeError::ResourceLimitExceeded {
                resource: ResourceType::CpuTime,
                limit: self.limits.max_cpu_time_ms,
                requested: self.usage.cpu_time_ms,
            });
        }
        Ok(())
    }

    /// Allocate memory within the sandbox
    pub fn allocate_memory(&mut self, bytes: usize) -> Result<(), RuntimeError> {
        if let Err(e) = self.check_memory_limit(bytes) {
            self.violations += 1;
            return Err(e);
        }
        self.usage.memory_bytes = self.usage.memory_bytes.saturating_add(bytes);
        // Track peak memory for reporting
        if self.usage.memory_bytes > self.usage.peak_memory_bytes {
            self.usage.peak_memory_bytes = self.usage.memory_bytes;
        }
        self.update_timestamp();
        Ok(())
    }

    /// Free memory within the sandbox
    pub fn free_memory(&mut self, bytes: usize) {
        self.usage.memory_bytes = self.usage.memory_bytes.saturating_sub(bytes);
        self.update_timestamp();
    }

    /// Open a file descriptor
    pub fn open_fd(&mut self) -> Result<(), RuntimeError> {
        if let Err(e) = self.check_fd_limit() {
            self.violations += 1;
            return Err(e);
        }
        self.usage.file_descriptors += 1;
        // Track peak FDs for reporting
        if self.usage.file_descriptors > self.usage.peak_file_descriptors {
            self.usage.peak_file_descriptors = self.usage.file_descriptors;
        }
        self.update_timestamp();
        Ok(())
    }

    /// Close a file descriptor
    pub fn close_fd(&mut self) {
        if self.usage.file_descriptors > 0 {
            self.usage.file_descriptors -= 1;
        }
        self.update_timestamp();
    }

    /// Record CPU time usage
    pub fn record_cpu_time(&mut self, ms: u64) {
        self.usage.cpu_time_ms = self.usage.cpu_time_ms.saturating_add(ms);
        self.update_timestamp();
    }

    /// Record a syscall
    pub fn record_syscall(&mut self) {
        self.usage.syscall_count += 1;
        self.update_timestamp();
    }

    /// Add a capability token to this sandbox
    pub fn add_capability(&mut self, token: CapToken) {
        self.capabilities.push(token);
    }

    /// Remove a capability token from this sandbox
    pub fn revoke_capability(&mut self, token_id: u128) {
        self.capabilities.retain(|t| t.id != token_id);
    }

    /// Check if sandbox has a specific capability
    pub fn has_capability(&self, token_id: u128) -> bool {
        self.capabilities.iter().any(|t| t.id == token_id)
    }

    /// Update the last update timestamp
    fn update_timestamp(&mut self) {
        self.usage.last_update_ms = crate::security::get_current_time_ms();
    }

    /// Generate a resource usage report for DAO reporting
    pub fn generate_report(&self) -> ResourceReport {
        let now = crate::security::get_current_time_ms();
        ResourceReport {
            sandbox_id: self.id,
            language: self.language,
            memory_bytes: self.usage.memory_bytes,
            peak_memory_bytes: self.usage.peak_memory_bytes,
            cpu_time_ms: self.usage.cpu_time_ms,
            syscall_count: self.usage.syscall_count,
            lifetime_ms: now.saturating_sub(self.created_at),
            final_state: self.state,
            violations: self.violations,
        }
    }

    /// Get the current state
    pub fn get_state(&self) -> SandboxState {
        self.state
    }

    /// Check if sandbox is in a runnable state
    pub fn is_runnable(&self) -> bool {
        matches!(self.state, SandboxState::Created | SandboxState::Running)
    }

    /// Check if sandbox has been terminated
    pub fn is_terminated(&self) -> bool {
        self.state == SandboxState::Terminated
    }

    /// Get sandbox lifetime in milliseconds
    pub fn lifetime_ms(&self) -> u64 {
        crate::security::get_current_time_ms().saturating_sub(self.created_at)
    }

    /// Check and suspend if CPU limit exceeded
    pub fn check_and_suspend_if_exceeded(&mut self) -> bool {
        if self.usage.cpu_time_ms >= self.limits.max_cpu_time_ms {
            self.suspend();
            self.violations += 1;
            true
        } else {
            false
        }
    }

    // ========================================================================
    // Debugging Support Methods (Task 10)
    // ========================================================================

    /// Enable syscall tracing for this sandbox
    pub fn enable_tracing(&mut self) {
        self.tracing_enabled = true;
    }

    /// Disable syscall tracing for this sandbox
    pub fn disable_tracing(&mut self) {
        self.tracing_enabled = false;
    }

    /// Check if tracing is enabled
    pub fn is_tracing_enabled(&self) -> bool {
        self.tracing_enabled
    }

    /// Record a syscall trace event (only if tracing is enabled)
    pub fn trace_syscall(
        &mut self,
        syscall_number: u32,
        syscall_name: &str,
        arguments: &[u64],
        return_value: Option<i64>,
        duration_us: u64,
    ) {
        if !self.tracing_enabled {
            return;
        }

        let event = SyscallTraceEvent {
            sandbox_id: self.id,
            timestamp_ms: crate::security::get_current_time_ms(),
            syscall_number,
            syscall_name: syscall_name.into(),
            arguments: arguments.to_vec(),
            return_value,
            duration_us,
        };

        self.syscall_traces.push(event);
    }

    /// Get all syscall trace events
    pub fn get_syscall_traces(&self) -> &[SyscallTraceEvent] {
        &self.syscall_traces
    }

    /// Clear syscall trace events
    pub fn clear_syscall_traces(&mut self) {
        self.syscall_traces.clear();
    }

    /// Capture a crash dump when the sandbox crashes
    pub fn capture_crash_dump(&mut self, reason: &str, stack_frames: Vec<StackFrame>, register_state: RegisterState) {
        let mut memory_snapshots = Vec::new();

        // Capture memory snapshots from registered regions
        for (address, data) in &self.memory_regions {
            memory_snapshots.push(MemorySnapshot {
                address: *address,
                data: data.clone(),
                region_name: crate::kformat!("region_0x{:x}", address),
            });
        }

        self.crash_dump = Some(CrashDump {
            sandbox_id: self.id,
            timestamp_ms: crate::security::get_current_time_ms(),
            stack_frames,
            register_state,
            reason: reason.into(),
            memory_snapshot: memory_snapshots,
        });

        self.state = SandboxState::Terminated;
    }

    /// Get the crash dump if one was captured
    pub fn get_crash_dump(&self) -> Option<&CrashDump> {
        self.crash_dump.as_ref()
    }

    /// Register a memory region for inspection
    pub fn register_memory_region(&mut self, address: u64, data: Vec<u8>) {
        self.memory_regions.insert(address, data);
    }

    /// Inspect memory at the given address (read-only)
    pub fn inspect_memory(&self, address: u64, size: usize) -> Result<MemoryInspection, MemoryInspectionError> {
        // Find the region containing this address
        for (&region_start, region_data) in &self.memory_regions {
            let region_end = region_start + region_data.len() as u64;
            
            if address >= region_start && address < region_end {
                let offset = (address - region_start) as usize;
                let available = region_data.len() - offset;
                let read_size = size.min(available);
                
                let data = region_data[offset..offset + read_size].to_vec();
                
                return Ok(MemoryInspection {
                    address,
                    size: read_size,
                    data,
                    readable: true,
                });
            }
        }

        // Address not in any registered region
        Err(MemoryInspectionError::OutOfBounds { address, size })
    }

    /// Attempt to write memory (always rejected - read-only inspection)
    pub fn write_memory(&self, address: u64, _data: &[u8]) -> Result<(), MemoryInspectionError> {
        // Memory inspection is read-only - always reject writes
        Err(MemoryInspectionError::WriteRejected { address })
    }

    /// Get debug info for this sandbox
    pub fn get_debug_info(&self) -> DebugInfo {
        let mut info = DebugInfo::default();

        // Add memory regions
        for (&address, data) in &self.memory_regions {
            info.memory_regions.push(super::backend::MemoryRegion {
                start: address,
                size: data.len(),
                permissions: super::backend::permissions::READ,
                name: crate::kformat!("region_0x{:x}", address),
            });
        }

        // If we have a crash dump, include its stack frames
        if let Some(ref crash) = self.crash_dump {
            info.stack_frames = crash.stack_frames.clone();
        }

        info
    }
}
