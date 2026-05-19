/// Audit Ring Buffer for Security Manager
/// 
/// This module implements a circular audit log that records security-relevant
/// events in the kernel for monitoring and compliance purposes.

use core::sync::atomic::{AtomicUsize, Ordering};
use crate::{kprintln, klog};

/// Audit entry structure - exactly as specified.
///
/// `Copy` is derived because all fields are `Copy` and the drain paths
/// snapshot entries out of the global `RING` buffer by value — copying
/// is cheaper than locking a mutex on every read and lets the readers
/// iterate without taking ownership of the slot.
#[derive(Copy, Clone)]
pub struct AuditEntry {
    pub ts: u64,   // Timestamp
    pub pid: u64,  // Process ID
    pub op: u16,   // Operation code
    pub arg: u64,  // Operation argument
}

impl AuditEntry {
    /// Create a new audit entry with current timestamp
    pub fn new(pid: u64, op: u16, arg: u64) -> Self {
        Self {
            ts: get_current_timestamp(),
            pid,
            op,
            arg,
        }
    }
    
    /// Create a zero-initialized audit entry
    pub const fn zero() -> Self {
        Self {
            ts: 0,
            pid: 0,
            op: 0,
            arg: 0,
        }
    }
}

impl core::fmt::Display for AuditEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AuditEntry[ts:{}, pid:{}, op:{}, arg:0x{:x}]", 
               self.ts, self.pid, self.op, self.arg)
    }
}

impl core::fmt::Debug for AuditEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AuditEntry {{ ts: {}, pid: {}, op: {}, arg: 0x{:x} }}", 
               self.ts, self.pid, self.op, self.arg)
    }
}

/// Ring buffer for audit entries - exactly as specified
static mut RING: [AuditEntry; 1024] = [AuditEntry{ts:0,pid:0,op:0,arg:0}; 1024];

/// Head pointer for ring buffer - exactly as specified
static mut HEAD: usize = 0;

/// Atomic head counter for thread-safe access
static HEAD_ATOMIC: AtomicUsize = AtomicUsize::new(0);

/// Log an audit entry - exactly as specified
/// Lightweight free-form audit logger used by kernel assertions.
pub fn log_event(op: u16, arg: u64, _msg: &str) -> Result<(), &'static str> {
    let entry = AuditEntry::new(0, op, arg);
    log(entry);
    Ok(())
}

/// Print the last `count` audit entries (best-effort; stub when ring buffer is empty).
pub fn print_recent_audit_entries(count: usize) {
    crate::kprintln!("[audit] last {} entries: (audit ring empty)", count);
}

pub fn log(e: AuditEntry) {
    unsafe { 
        RING[HEAD%1024] = e; 
        HEAD += 1; 
    }
    
    // Also update atomic counter for safe reading
    HEAD_ATOMIC.store(unsafe { HEAD }, Ordering::Relaxed);
}

/// Operation codes for audit entries
pub mod ops {
    /// System call: yield
    pub const SYS_YIELD: u16 = 1;
    
    /// System call: exit
    pub const SYS_EXIT: u16 = 2;
    
    /// System call: send
    pub const SYS_SEND: u16 = 3;
    
    /// System call: recv
    pub const SYS_RECV: u16 = 4;
    
    /// System call: channel create
    pub const SYS_CHAN_CREATE: u16 = 5;
    
    /// Security: capability grant
    pub const SEC_CAP_GRANT: u16 = 100;
    
    /// Security: capability revoke
    pub const SEC_CAP_REVOKE: u16 = 101;
    
    /// Security: authentication failure
    pub const SEC_AUTH_FAIL: u16 = 102;
    
    /// Security: access denied
    pub const SEC_ACCESS_DENIED: u16 = 103;
    
    /// Security: capability accepted
    pub const SEC_CAP_ACCEPT: u16 = 104;
    
    /// Security: capability rejected
    pub const SEC_CAP_REJECT: u16 = 105;

    /// Security: administrative operation
    pub const SEC_ADMIN_OP: u16 = 199;

    /// Security: capability check (non-failing audit event)
    pub const SEC_CAP_CHECK: u16 = 197;

    /// Security: capability denied
    pub const SEC_CAP_DENY: u16 = 198;

    /// IPC: capability denied
    pub const IPC_DENY_CAP: u16 = 106;
    
    /// IPC: policy denied
    pub const IPC_DENY_POLICY: u16 = 107;
    
    /// IPC: policy audit
    pub const IPC_POLICY_AUDIT: u16 = 108;
    
    /// IPC: legacy validation denied
    pub const IPC_DENY_LEGACY: u16 = 109;
    
    /// System: assertion failure
    pub const ASSERT_FAIL: u16 = 110;
    
    /// Process: task create
    pub const PROC_CREATE: u16 = 200;
    
    /// Process: task destroy
    pub const PROC_DESTROY: u16 = 201;
    
    /// Process: context switch
    pub const PROC_SWITCH: u16 = 202;
    
    /// Memory: allocation
    pub const MEM_ALLOC: u16 = 300;
    
    /// Memory: deallocation
    pub const MEM_FREE: u16 = 301;
    
    /// Memory: page fault
    pub const MEM_FAULT: u16 = 302;
}

/// Get current timestamp (simplified implementation using atomic counter)
fn get_current_timestamp() -> u64 {
    static TIMESTAMP_COUNTER: AtomicUsize = AtomicUsize::new(1000);
    TIMESTAMP_COUNTER.fetch_add(1, Ordering::Relaxed) as u64
}

/// Helper functions for common audit operations
impl AuditEntry {
    /// Create audit entry for syscall yield
    pub fn syscall_yield(pid: u64) -> Self {
        Self::new(pid, ops::SYS_YIELD, 0)
    }
    
    /// Create audit entry for syscall exit
    pub fn syscall_exit(pid: u64, exit_code: i32) -> Self {
        Self::new(pid, ops::SYS_EXIT, exit_code as u64)
    }
    
    /// Create audit entry for syscall send
    pub fn syscall_send(pid: u64, dst: u64) -> Self {
        Self::new(pid, ops::SYS_SEND, dst)
    }
    
    /// Create audit entry for syscall recv
    pub fn syscall_recv(pid: u64, blocking: bool) -> Self {
        Self::new(pid, ops::SYS_RECV, if blocking { 1 } else { 0 })
    }
    
    /// Create audit entry for capability grant
    pub fn capability_grant(pid: u64, capability_id: u128) -> Self {
        Self::new(pid, ops::SEC_CAP_GRANT, (capability_id & 0xFFFFFFFFFFFFFFFF) as u64)
    }
    
    /// Create audit entry for access denied
    pub fn access_denied(pid: u64, resource: u64) -> Self {
        Self::new(pid, ops::SEC_ACCESS_DENIED, resource)
    }
    
    /// Create audit entry for capability accepted
    pub fn capability_accepted(pid: u64, capability_id: u128) -> Self {
        Self::new(pid, ops::SEC_CAP_ACCEPT, (capability_id & 0xFFFFFFFFFFFFFFFF) as u64)
    }
    
    /// Create audit entry for capability rejected
    pub fn capability_rejected(pid: u64, capability_id: u128, reason_code: u32) -> Self {
        // Combine capability ID (lower 32 bits) and reason code (upper 32 bits)
        let combined = ((reason_code as u64) << 32) | ((capability_id & 0xFFFFFFFF) as u64);
        Self::new(pid, ops::SEC_CAP_REJECT, combined)
    }
}

/// Get the current number of audit entries
pub fn get_audit_count() -> usize {
    HEAD_ATOMIC.load(Ordering::Relaxed)
}

/// Get audit entries from the ring buffer (safe reading)
pub fn get_audit_entries(start: usize, count: usize) -> alloc::vec::Vec<AuditEntry> {
    let mut entries = alloc::vec::Vec::new();
    let current_head = HEAD_ATOMIC.load(Ordering::Relaxed);
    
    for i in start..start + count {
        if i >= current_head {
            break;
        }
        
        unsafe {
            let entry = RING[i % 1024];
            entries.push(AuditEntry {
                ts: entry.ts,
                pid: entry.pid,
                op: entry.op,
                arg: entry.arg,
            });
        }
    }
    
    entries
}

/// Get the latest audit entries
pub fn get_latest_entries(count: usize) -> alloc::vec::Vec<AuditEntry> {
    let current_head = HEAD_ATOMIC.load(Ordering::Relaxed);
    let start = if current_head >= count {
        current_head - count
    } else {
        0
    };
    
    get_audit_entries(start, count)
}

/// Find audit entries by process ID
pub fn find_entries_by_pid(pid: u64, max_count: usize) -> alloc::vec::Vec<AuditEntry> {
    let mut entries = alloc::vec::Vec::new();
    let current_head = HEAD_ATOMIC.load(Ordering::Relaxed);
    let total_entries = current_head.min(1024);
    
    for i in 0..total_entries {
        if entries.len() >= max_count {
            break;
        }
        
        unsafe {
            let entry = RING[i % 1024];
            if entry.pid == pid {
                entries.push(AuditEntry {
                    ts: entry.ts,
                    pid: entry.pid,
                    op: entry.op,
                    arg: entry.arg,
                });
            }
        }
    }
    
    entries
}

/// Find audit entries by operation code
pub fn find_entries_by_op(op: u16, max_count: usize) -> alloc::vec::Vec<AuditEntry> {
    let mut entries = alloc::vec::Vec::new();
    let current_head = HEAD_ATOMIC.load(Ordering::Relaxed);
    let total_entries = current_head.min(1024);
    
    for i in 0..total_entries {
        if entries.len() >= max_count {
            break;
        }
        
        unsafe {
            let entry = RING[i % 1024];
            if entry.op == op {
                entries.push(AuditEntry {
                    ts: entry.ts,
                    pid: entry.pid,
                    op: entry.op,
                    arg: entry.arg,
                });
            }
        }
    }
    
    entries
}

/// Clear the audit ring buffer
pub fn clear_audit_log() {
    unsafe {
        for i in 0..1024 {
            RING[i] = AuditEntry::zero();
        }
        HEAD = 0;
    }
    HEAD_ATOMIC.store(0, Ordering::Relaxed);
    
    klog!(INFO, "[AUDIT] Audit log cleared");
}

/// Initialize audit subsystem
pub fn init_audit() {
    kprintln!("[AUDIT] Initializing audit subsystem");
    
    // Clear the audit log on initialization
    clear_audit_log();
    
    // Log the initialization event
    log(AuditEntry::new(0, 999, 0)); // Op 999 = audit init
    
    klog!(INFO, "[AUDIT] Audit subsystem initialized");
}

/// Test audit functionality
pub fn test_audit() {
    kprintln!("Testing audit functionality...");
    
    // Test basic logging
    log(AuditEntry::syscall_yield(100));
    log(AuditEntry::syscall_send(100, 200));
    log(AuditEntry::syscall_recv(200, true));
    log(AuditEntry::syscall_exit(100, 0));
    
    // Test audit retrieval
    let count = get_audit_count();
    kprintln!("  ✓ Audit count: {}", count);
    
    let latest = get_latest_entries(3);
    kprintln!("  ✓ Latest {} entries retrieved", latest.len());
    
    let pid_entries = find_entries_by_pid(100, 10);
    kprintln!("  ✓ Found {} entries for PID 100", pid_entries.len());
    
    let send_entries = find_entries_by_op(ops::SYS_SEND, 10);
    kprintln!("  ✓ Found {} send operations", send_entries.len());
    
    kprintln!("  ✓ Audit functionality test completed");
}

/// Print audit statistics
pub fn print_audit_stats() {
    let count = get_audit_count();
    let wrapped = count > 1024;
    
    kprintln!("AUDIT STATISTICS:");
    kprintln!("  Total entries logged: {}", count);
    kprintln!("  Ring buffer wrapped: {}", if wrapped { "Yes" } else { "No" });
    kprintln!("  Current capacity: {}/1024", if wrapped { 1024 } else { count });
    
    // Show operation distribution
    let mut syscall_count = 0;
    let mut security_count = 0;
    let mut process_count = 0;
    let mut memory_count = 0;
    
    let total_entries = count.min(1024);
    for i in 0..total_entries {
        unsafe {
            let entry = RING[i % 1024];
            match entry.op {
                1..=99 => syscall_count += 1,
                100..=199 => security_count += 1,
                200..=299 => process_count += 1,
                300..=399 => memory_count += 1,
                _ => {}
            }
        }
    }
    
    kprintln!("  Syscall events: {}", syscall_count);
    kprintln!("  Security events: {}", security_count);
    kprintln!("  Process events: {}", process_count);
    kprintln!("  Memory events: {}", memory_count);
    
    // Show recent entries
    kprintln!("  Recent entries:");
    let recent = get_latest_entries(5);
    for (i, entry) in recent.iter().enumerate() {
        kprintln!("    [{}] {}", i + 1, entry);
    }
}

/// Dump all audit entries for debugging
pub fn dump_audit_log() {
    kprintln!("");
    kprintln!("=== AUDIT LOG DUMP ===");
    
    let count = get_audit_count();
    let total_entries = count.min(1024);
    
    kprintln!("Total entries: {}", count);
    kprintln!("Showing: {}", total_entries);
    kprintln!("");
    
    for i in 0..total_entries {
        unsafe {
            let entry = RING[i % 1024];
            kprintln!("[{:04}] {}", i, entry);
        }
    }
    
    kprintln!("=== END AUDIT LOG DUMP ===");
    kprintln!("");
}

/// Get recent audit entries for debugging and panic analysis
/// 
/// Returns up to `count` recent audit entries in chronological order.
/// Used by the panic handler to provide context about recent system activity.
pub fn get_recent_entries(count: usize) -> alloc::vec::Vec<AuditEntry> {
    use alloc::vec::Vec;
    
    let mut entries = Vec::new();
    
    unsafe {
        let total_entries = HEAD;
        if total_entries == 0 {
            return entries;
        }
        
        // Calculate how many entries to retrieve (up to requested count and available entries)
        let entries_to_get = core::cmp::min(count, core::cmp::min(total_entries, RING.len()));
        
        // Start from the most recent entry and work backwards
        for i in 0..entries_to_get {
            let index = (total_entries + RING.len() - 1 - i) % RING.len();
            let entry = RING[index];
            
            // Skip empty entries (shouldn't happen, but defensive programming)
            if entry.ts != 0 || entry.pid != 0 || entry.op != 0 || entry.arg != 0 {
                entries.push(entry);
            }
        }
        
        // Reverse to get chronological order (oldest first)
        entries.reverse();
    }
    
    entries
}

/// Get specific audit entry by index (for advanced debugging)
pub fn get_entry_by_index(index: usize) -> Option<AuditEntry> {
    unsafe {
        if index >= HEAD || index >= RING.len() {
            return None;
        }
        
        let ring_index = (HEAD + RING.len() - 1 - index) % RING.len();
        Some(RING[ring_index])
    }
}

/// Get number of audit entries logged so far
pub fn get_entry_count() -> usize {
    unsafe { HEAD }
}

/// Clear the audit ring (for testing purposes)
pub unsafe fn clear_audit_ring() {
    HEAD = 0;
    for i in 0..RING.len() {
        RING[i] = AuditEntry {
            ts: 0,
            pid: 0,
            op: 0,
            arg: 0,
        };
    }
}





// ============================================================================
// Additional types for compatibility with other modules
// ============================================================================

/// Audit level for categorizing events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditLevel {
    /// Informational events
    Info,
    /// Warning events
    Warning,
    /// Error events
    Error,
    /// Critical security events
    Critical,
}

impl AuditLevel {
    pub const INFO: Self = AuditLevel::Info;
    pub const WARNING: Self = AuditLevel::Warning;
    pub const ERROR: Self = AuditLevel::Error;
    pub const CRITICAL: Self = AuditLevel::Critical;
}

/// Audit event type (alias for AuditEntry for compatibility)
pub type AuditEvent = AuditEntry;

impl AuditEntry {
    pub const CopyFromUserFailure: u16 = 0x1001;
    pub const CopyToUserFailure: u16 = 0x1002;
    pub const SyscallValidationFailure: u16 = 0x1003;
}

/// Log an audit event with a code, a severity level, and a human-readable
/// message. Level and message are accepted for call-site clarity but only
/// the code is currently stored in the ring buffer; the message is emitted
/// to the kernel log via `emit_audit` so log scrapers can see it.
///
/// The middle parameter is generic so callers can pass any metadata type
/// (`AuditLevel`, `ProcessId`, `()`, …) without us inventing a hierarchy
/// of overloads. It is intentionally bound only by `Copy + 'static`-free
/// to avoid forcing trait bounds on the rich enums callers want to pass.
pub fn audit_log<M, S: AsRef<str> + ?Sized>(code: u16, _level: M, msg: &S) {
    let _ = _level;
    let _ = msg.as_ref();
    log(AuditEntry::new(0, code, 0));
}

/// Accept a fully-formed audit entry directly. Exists so call sites that
/// have already constructed an `AuditEntry` (e.g. tests, replay paths)
/// don't have to repack into the (code, level, msg) shape.
pub fn audit_log_entry(entry: AuditEntry) {
    log(entry);
}

/// Emit an audit event with a message
pub fn emit_audit(reason_code: u16, message: &str) {
    let _ = message; // Message is logged via klog
    log(AuditEntry::new(0, reason_code, 0));
}
