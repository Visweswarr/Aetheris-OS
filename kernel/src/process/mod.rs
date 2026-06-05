//! Process Management Module for Polymera OS
//!
//! Provides a complete process management subsystem including:
//! - Process Control Block (PCB) with state, priority, capabilities
//! - Global process table with O(1) lookup by PID
//! - Full lifecycle: create → ready → running → blocked → terminated
//! - Parent/child relationships and process hierarchy
//! - Statistics tracking for observability

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

// ─────────────────────────────────────────────────────────────────────────────
// Process Identifier
// ─────────────────────────────────────────────────────────────────────────────

/// Process identifier type — thin wrapper around u64 for type safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ProcessId(pub u64);

impl ProcessId {
    /// Kernel process (PID 0)
    pub const KERNEL: Self = Self(0);

    /// Init process (PID 1)
    pub const INIT: Self = Self(1);

    /// Create a new ProcessId
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Check if this is the kernel process
    pub const fn is_kernel(&self) -> bool {
        self.0 == 0
    }
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PID({})", self.0)
    }
}

impl From<u64> for ProcessId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<ProcessId> for u64 {
    fn from(pid: ProcessId) -> Self {
        pid.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Process States
// ─────────────────────────────────────────────────────────────────────────────

/// Process lifecycle states following the standard OS model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Process has been created but not yet scheduled.
    Created,
    /// Process is in the ready queue, waiting for CPU time.
    Ready,
    /// Process is currently executing on a CPU core.
    Running,
    /// Process is waiting for an event (I/O, IPC, timer, etc.).
    Blocked(BlockReason),
    /// Process is suspended (paused by user or system).
    Suspended,
    /// Process has terminated and is awaiting cleanup.
    Zombie,
    /// Process has been fully terminated and resources freed.
    Terminated,
}

impl ProcessState {
    /// Returns true if the process is alive (not terminated or zombie).
    pub fn is_alive(&self) -> bool {
        !matches!(self, ProcessState::Terminated | ProcessState::Zombie)
    }

    /// Returns a human-readable status string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessState::Created => "CREATED",
            ProcessState::Ready => "READY",
            ProcessState::Running => "RUNNING",
            ProcessState::Blocked(_) => "BLOCKED",
            ProcessState::Suspended => "SUSPENDED",
            ProcessState::Zombie => "ZOMBIE",
            ProcessState::Terminated => "TERMINATED",
        }
    }
}

/// Reason why a process is blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockReason {
    /// Waiting for an IPC message.
    IpcReceive,
    /// Waiting for an IPC send to complete (channel full).
    IpcSend,
    /// Waiting for a timer/sleep to expire.
    Timer,
    /// Waiting for I/O completion.
    Io,
    /// Waiting for a child process to exit.
    WaitChild,
    /// Waiting for a mutex or lock.
    Mutex,
    /// Waiting for memory allocation.
    Memory,
}

// ─────────────────────────────────────────────────────────────────────────────
// Process Priority
// ─────────────────────────────────────────────────────────────────────────────

/// Process scheduling priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Idle priority — runs only when nothing else is runnable.
    Idle = 0,
    /// Low priority background tasks.
    Low = 1,
    /// Default priority for user applications.
    Normal = 2,
    /// Elevated priority for system services.
    High = 3,
    /// Real-time priority for latency-critical tasks.
    Realtime = 4,
    /// Kernel-level priority — highest.
    Kernel = 5,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Memory Region Descriptor
// ─────────────────────────────────────────────────────────────────────────────

/// Describes a memory region assigned to a process.
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    /// Virtual base address of the region.
    pub base: u64,
    /// Size of the region in bytes.
    pub size: u64,
    /// Access permissions for this region.
    pub permissions: MemoryPermissions,
}

/// Memory access permission flags.
#[derive(Debug, Clone, Copy)]
pub struct MemoryPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl MemoryPermissions {
    pub const READ_ONLY: Self = Self { read: true, write: false, execute: false };
    pub const READ_WRITE: Self = Self { read: true, write: true, execute: false };
    pub const READ_EXECUTE: Self = Self { read: true, write: false, execute: true };
    pub const NONE: Self = Self { read: false, write: false, execute: false };
}

// ─────────────────────────────────────────────────────────────────────────────
// Process Control Block (PCB)
// ─────────────────────────────────────────────────────────────────────────────

/// The Process Control Block holds all state for a single process.
///
/// This is the primary data structure for process management in PolymeraCore.
/// Each process gets a unique PCB when created, which tracks its lifecycle,
/// resource usage, and security context.
#[derive(Debug, Clone)]
pub struct ProcessControlBlock {
    /// Unique process identifier.
    pub pid: ProcessId,

    /// Human-readable name for this process.
    pub name: String,

    /// Current process state.
    pub state: ProcessState,

    /// Scheduling priority.
    pub priority: Priority,

    /// Parent process ID (kernel has no parent).
    pub parent: Option<ProcessId>,

    /// Child process IDs.
    pub children: Vec<ProcessId>,

    /// Capability tokens held by this process (cap token IDs).
    pub capabilities: Vec<u64>,

    /// Memory regions assigned to this process.
    pub memory_regions: Vec<MemoryRegion>,

    /// Total memory usage in bytes (approximate).
    pub memory_usage: u64,

    /// CPU time consumed in microseconds.
    pub cpu_time_us: u64,

    /// Number of context switches.
    pub context_switches: u64,

    /// Number of IPC messages sent.
    pub ipc_sent: u64,

    /// Number of IPC messages received.
    pub ipc_received: u64,

    /// Number of syscalls made.
    pub syscall_count: u64,

    /// Tick count when the process was created.
    pub created_at_tick: u64,

    /// Exit code (set when terminated).
    pub exit_code: Option<i32>,
}

impl ProcessControlBlock {
    /// Create a new PCB with default values.
    pub fn new(pid: ProcessId, name: String, parent: Option<ProcessId>, priority: Priority) -> Self {
        Self {
            pid,
            name,
            state: ProcessState::Created,
            priority,
            parent,
            children: Vec::new(),
            capabilities: Vec::new(),
            memory_regions: Vec::new(),
            memory_usage: 0,
            cpu_time_us: 0,
            context_switches: 0,
            ipc_sent: 0,
            ipc_received: 0,
            syscall_count: 0,
            created_at_tick: 0,
            exit_code: None,
        }
    }

    /// Transition to a new state. Returns Err if the transition is invalid.
    pub fn transition_to(&mut self, new_state: ProcessState) -> Result<(), ProcessError> {
        let valid = match (&self.state, &new_state) {
            (ProcessState::Created, ProcessState::Ready) => true,
            (ProcessState::Ready, ProcessState::Running) => true,
            (ProcessState::Running, ProcessState::Ready) => true,        // preempted
            (ProcessState::Running, ProcessState::Blocked(_)) => true,   // waiting
            (ProcessState::Running, ProcessState::Zombie) => true,       // exited
            (ProcessState::Running, ProcessState::Suspended) => true,    // paused
            (ProcessState::Blocked(_), ProcessState::Ready) => true,     // event arrived
            (ProcessState::Suspended, ProcessState::Ready) => true,      // resumed
            (ProcessState::Zombie, ProcessState::Terminated) => true,    // reaped
            _ => false,
        };

        if valid {
            self.state = new_state;
            Ok(())
        } else {
            Err(ProcessError::InvalidStateTransition {
                from: self.state.as_str(),
                to: new_state.as_str(),
            })
        }
    }

    /// Add a memory region to this process.
    pub fn add_memory_region(&mut self, region: MemoryRegion) {
        self.memory_usage += region.size;
        self.memory_regions.push(region);
    }

    /// Grant a capability token to this process.
    pub fn grant_capability(&mut self, cap_token: u64) {
        if !self.capabilities.contains(&cap_token) {
            self.capabilities.push(cap_token);
        }
    }

    /// Revoke a capability token from this process.
    pub fn revoke_capability(&mut self, cap_token: u64) -> bool {
        if let Some(pos) = self.capabilities.iter().position(|&c| c == cap_token) {
            self.capabilities.swap_remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if this process holds a specific capability.
    pub fn has_capability(&self, cap_token: u64) -> bool {
        self.capabilities.contains(&cap_token)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Process Error Types
// ─────────────────────────────────────────────────────────────────────────────

/// Errors that can occur during process operations.
#[derive(Debug, Clone)]
pub enum ProcessError {
    /// Process not found in the table.
    NotFound(ProcessId),
    /// Maximum number of processes reached.
    TableFull,
    /// Invalid state transition attempted.
    InvalidStateTransition {
        from: &'static str,
        to: &'static str,
    },
    /// Permission denied for the requested operation.
    PermissionDenied,
    /// Process is already terminated.
    AlreadyTerminated,
    /// Process name is empty or invalid.
    InvalidName,
}

impl core::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProcessError::NotFound(pid) => write!(f, "process {} not found", pid),
            ProcessError::TableFull => write!(f, "process table full (max {} processes)", MAX_PROCESSES),
            ProcessError::InvalidStateTransition { from, to } => {
                write!(f, "invalid state transition: {} → {}", from, to)
            }
            ProcessError::PermissionDenied => write!(f, "permission denied"),
            ProcessError::AlreadyTerminated => write!(f, "process already terminated"),
            ProcessError::InvalidName => write!(f, "invalid process name"),
        }
    }
}

pub type ProcessResult<T> = Result<T, ProcessError>;

// ─────────────────────────────────────────────────────────────────────────────
// Process Table
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum number of processes the system supports.
pub const MAX_PROCESSES: usize = 256;

/// Global process ID counter.
static NEXT_PID: AtomicU64 = AtomicU64::new(2); // 0=kernel, 1=init, start at 2

/// Process table statistics.
#[derive(Debug, Clone, Copy)]
pub struct ProcessStats {
    /// Total processes ever created.
    pub total_created: u64,
    /// Currently alive processes.
    pub active_count: u64,
    /// Total processes terminated.
    pub total_terminated: u64,
    /// Total context switches across all processes.
    pub total_context_switches: u64,
    /// Peak number of concurrent processes.
    pub peak_concurrent: u64,
}

impl ProcessStats {
    pub const fn new() -> Self {
        Self {
            total_created: 0,
            active_count: 0,
            total_terminated: 0,
            total_context_switches: 0,
            peak_concurrent: 0,
        }
    }
}

/// The global process table — holds all PCBs indexed by PID.
struct ProcessTable {
    processes: BTreeMap<u64, ProcessControlBlock>,
    stats: ProcessStats,
    current_pid: ProcessId,
}

impl ProcessTable {
    const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            stats: ProcessStats::new(),
            current_pid: ProcessId::KERNEL,
        }
    }
}

/// Global process table, protected by a spinlock for no_std safety.
static PROCESS_TABLE: Mutex<ProcessTable> = Mutex::new(ProcessTable::new());

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a new unique process ID.
pub fn generate_pid() -> ProcessId {
    ProcessId(NEXT_PID.fetch_add(1, Ordering::Relaxed))
}

/// Get the currently running process ID.
pub fn current_pid() -> ProcessId {
    PROCESS_TABLE.lock().current_pid
}

/// Set the currently running process (called by the scheduler).
pub fn set_current_pid(pid: ProcessId) {
    PROCESS_TABLE.lock().current_pid = pid;
}

/// Create a new process and add it to the process table.
///
/// The process starts in the `Created` state. Call `make_ready()` to
/// transition it to the `Ready` state for scheduling.
pub fn create_process(
    name: &str,
    parent: Option<ProcessId>,
    priority: Priority,
) -> ProcessResult<ProcessId> {
    if name.is_empty() {
        return Err(ProcessError::InvalidName);
    }

    let mut table = PROCESS_TABLE.lock();

    if table.processes.len() >= MAX_PROCESSES {
        return Err(ProcessError::TableFull);
    }

    let pid = generate_pid();
    let mut pcb = ProcessControlBlock::new(
        pid,
        String::from(name),
        parent,
        priority,
    );

    // Assign a default user-space memory region (4 MiB stack + 16 MiB heap).
    let stack_base = 0x0000_7000_0000_0000u64 + (pid.0 * 0x100_0000);
    pcb.add_memory_region(MemoryRegion {
        base: stack_base,
        size: 4 * 1024 * 1024, // 4 MiB stack
        permissions: MemoryPermissions::READ_WRITE,
    });
    pcb.add_memory_region(MemoryRegion {
        base: stack_base + 0x100_0000,
        size: 16 * 1024 * 1024, // 16 MiB heap
        permissions: MemoryPermissions::READ_WRITE,
    });

    // Register child with parent.
    if let Some(parent_pid) = parent {
        if let Some(parent_pcb) = table.processes.get_mut(&parent_pid.0) {
            parent_pcb.children.push(pid);
        }
    }

    table.processes.insert(pid.0, pcb);
    table.stats.total_created += 1;
    table.stats.active_count += 1;
    if table.stats.active_count > table.stats.peak_concurrent {
        table.stats.peak_concurrent = table.stats.active_count;
    }

    crate::kprintln!("[PROC] Created process {} \"{}\" (parent={:?}, priority={:?})",
        pid, name, parent, priority);

    Ok(pid)
}

/// Transition a process to the Ready state (eligible for scheduling).
pub fn make_ready(pid: ProcessId) -> ProcessResult<()> {
    let mut table = PROCESS_TABLE.lock();
    let pcb = table.processes.get_mut(&pid.0)
        .ok_or(ProcessError::NotFound(pid))?;
    pcb.transition_to(ProcessState::Ready)
}

/// Mark a process as running (called by the scheduler on context switch).
pub fn mark_running(pid: ProcessId) -> ProcessResult<()> {
    let mut table = PROCESS_TABLE.lock();
    let result = {
        let pcb = table.processes.get_mut(&pid.0)
            .ok_or(ProcessError::NotFound(pid))?;
        pcb.context_switches += 1;
        pcb.transition_to(ProcessState::Running)
    };
    if result.is_ok() {
        table.stats.total_context_switches += 1;
    }
    result
}

/// Block a process with a specific reason.
pub fn block_process(pid: ProcessId, reason: BlockReason) -> ProcessResult<()> {
    let mut table = PROCESS_TABLE.lock();
    let pcb = table.processes.get_mut(&pid.0)
        .ok_or(ProcessError::NotFound(pid))?;
    pcb.transition_to(ProcessState::Blocked(reason))
}

/// Unblock a process (move back to Ready).
pub fn unblock_process(pid: ProcessId) -> ProcessResult<()> {
    let mut table = PROCESS_TABLE.lock();
    let pcb = table.processes.get_mut(&pid.0)
        .ok_or(ProcessError::NotFound(pid))?;
    pcb.transition_to(ProcessState::Ready)
}

/// Suspend a running process.
pub fn suspend_process(pid: ProcessId) -> ProcessResult<()> {
    let mut table = PROCESS_TABLE.lock();
    let pcb = table.processes.get_mut(&pid.0)
        .ok_or(ProcessError::NotFound(pid))?;
    pcb.transition_to(ProcessState::Suspended)
}

/// Resume a suspended process.
pub fn resume_process(pid: ProcessId) -> ProcessResult<()> {
    let mut table = PROCESS_TABLE.lock();
    let pcb = table.processes.get_mut(&pid.0)
        .ok_or(ProcessError::NotFound(pid))?;
    pcb.transition_to(ProcessState::Ready)
}

/// Terminate a process with an exit code.
pub fn terminate_process(pid: ProcessId, exit_code: i32) -> ProcessResult<()> {
    // Don't allow killing the kernel process.
    if pid.is_kernel() {
        return Err(ProcessError::PermissionDenied);
    }

    let mut table = PROCESS_TABLE.lock();
    let pcb = table.processes.get_mut(&pid.0)
        .ok_or(ProcessError::NotFound(pid))?;

    if !pcb.state.is_alive() {
        return Err(ProcessError::AlreadyTerminated);
    }

    // Force transition to Zombie (bypassing normal validation for kill).
    pcb.state = ProcessState::Zombie;
    pcb.exit_code = Some(exit_code);
    let process_name = pcb.name.clone();
    drop(pcb);

    table.stats.active_count = table.stats.active_count.saturating_sub(1);
    table.stats.total_terminated += 1;

    crate::kprintln!("[PROC] Terminated process {} \"{}\" (exit_code={})",
        pid, process_name, exit_code);

    Ok(())
}

/// Reap a zombie process — fully removes it from the process table.
pub fn reap_process(pid: ProcessId) -> ProcessResult<i32> {
    let mut table = PROCESS_TABLE.lock();
    let pcb = table.processes.get(&pid.0)
        .ok_or(ProcessError::NotFound(pid))?;

    if pcb.state != ProcessState::Zombie {
        return Err(ProcessError::InvalidStateTransition {
            from: pcb.state.as_str(),
            to: "TERMINATED",
        });
    }

    let exit_code = pcb.exit_code.unwrap_or(-1);
    table.processes.remove(&pid.0);
    Ok(exit_code)
}

/// Get a snapshot of a process's PCB (clone for safe reading).
pub fn get_process(pid: ProcessId) -> ProcessResult<ProcessControlBlock> {
    let table = PROCESS_TABLE.lock();
    table.processes.get(&pid.0)
        .cloned()
        .ok_or(ProcessError::NotFound(pid))
}

/// List all processes in the system. Returns cloned PCBs sorted by PID.
pub fn list_processes() -> Vec<ProcessControlBlock> {
    let table = PROCESS_TABLE.lock();
    table.processes.values().cloned().collect()
}

/// Get the number of active (alive) processes.
pub fn active_process_count() -> usize {
    let table = PROCESS_TABLE.lock();
    table.processes.values().filter(|p| p.state.is_alive()).count()
}

/// Get global process statistics.
pub fn get_stats() -> ProcessStats {
    PROCESS_TABLE.lock().stats
}

/// Record a syscall for the specified process.
pub fn record_syscall(pid: ProcessId) {
    let mut table = PROCESS_TABLE.lock();
    if let Some(pcb) = table.processes.get_mut(&pid.0) {
        pcb.syscall_count += 1;
    }
}

/// Record IPC activity for a process.
pub fn record_ipc_sent(pid: ProcessId) {
    let mut table = PROCESS_TABLE.lock();
    if let Some(pcb) = table.processes.get_mut(&pid.0) {
        pcb.ipc_sent += 1;
    }
}

/// Record IPC receive for a process.
pub fn record_ipc_received(pid: ProcessId) {
    let mut table = PROCESS_TABLE.lock();
    if let Some(pcb) = table.processes.get_mut(&pid.0) {
        pcb.ipc_received += 1;
    }
}

/// Add CPU time to a process (in microseconds).
pub fn add_cpu_time(pid: ProcessId, microseconds: u64) {
    let mut table = PROCESS_TABLE.lock();
    if let Some(pcb) = table.processes.get_mut(&pid.0) {
        pcb.cpu_time_us += microseconds;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Initialization
// ─────────────────────────────────────────────────────────────────────────────

/// Initialize the process subsystem.
///
/// Creates PID 0 (kernel) and PID 1 (init) processes.
pub fn init_processes() {
    let mut table = PROCESS_TABLE.lock();

    // Create kernel process (PID 0).
    let mut kernel_pcb = ProcessControlBlock::new(
        ProcessId::KERNEL,
        String::from("kernel"),
        None,
        Priority::Kernel,
    );
    kernel_pcb.state = ProcessState::Running;
    kernel_pcb.add_memory_region(MemoryRegion {
        base: 0xFFFF_8000_0000_0000,
        size: 256 * 1024 * 1024, // 256 MiB kernel space
        permissions: MemoryPermissions::READ_WRITE,
    });
    table.processes.insert(0, kernel_pcb);
    table.stats.total_created += 1;
    table.stats.active_count += 1;

    // Create init process (PID 1).
    let mut init_pcb = ProcessControlBlock::new(
        ProcessId::INIT,
        String::from("init"),
        Some(ProcessId::KERNEL),
        Priority::High,
    );
    init_pcb.state = ProcessState::Ready;
    init_pcb.add_memory_region(MemoryRegion {
        base: 0x0000_1000_0000_0000,
        size: 8 * 1024 * 1024, // 8 MiB for init
        permissions: MemoryPermissions::READ_WRITE,
    });
    table.processes.insert(1, init_pcb);
    table.stats.total_created += 1;
    table.stats.active_count += 1;
    table.stats.peak_concurrent = 2;

    crate::kprintln!("[PROC] Process subsystem initialized (kernel=PID0, init=PID1)");
}

/// Print a summary of all processes (for kernel dashboard / debugging).
pub fn print_process_table() {
    let table = PROCESS_TABLE.lock();
    crate::kprintln!("");
    crate::kprintln!("╔══════════════════════════════════════════════════════════════╗");
    crate::kprintln!("║                    PROCESS TABLE                            ║");
    crate::kprintln!("╠══════╦══════════════════╦════════════╦══════════╦════════════╣");
    crate::kprintln!("║  PID ║ Name             ║ State      ║ Priority ║ Memory     ║");
    crate::kprintln!("╠══════╬══════════════════╬════════════╬══════════╬════════════╣");
    for pcb in table.processes.values() {
        let mem_kb = pcb.memory_usage / 1024;
        let prio = match pcb.priority {
            Priority::Idle => "IDLE",
            Priority::Low => "LOW",
            Priority::Normal => "NORMAL",
            Priority::High => "HIGH",
            Priority::Realtime => "RT",
            Priority::Kernel => "KERNEL",
        };
        crate::kprintln!("║ {:>4} ║ {:>16} ║ {:>10} ║ {:>8} ║ {:>7} KB ║",
            pcb.pid.0, pcb.name, pcb.state.as_str(), prio, mem_kb);
    }
    crate::kprintln!("╚══════╩══════════════════╩════════════╩══════════╩════════════╝");
    crate::kprintln!("  Total: {} | Active: {} | Peak: {}",
        table.stats.total_created, table.stats.active_count, table.stats.peak_concurrent);
    crate::kprintln!("");
}
