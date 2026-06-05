/// System Call Number Table for Polymera OS
/// 
/// This module defines the system call numbers used by the kernel.
/// Each system call has a unique number that user space uses to invoke it.

/// System call: yield CPU to another task
pub const SYS_YIELD: u64 = 1;

/// System call: exit current task
pub const SYS_EXIT: u64 = 2;

// Future system calls for PolyBus IPC and other kernel services:

/// System call: send message to another task (future)
pub const SYS_SEND: u64 = 3;

/// System call: receive message from another task (future)
pub const SYS_RECV: u64 = 4;

/// System call: create IPC channel 
pub const SYS_CHAN_CREATE: u64 = 5;

/// System call: get system statistics (future)
pub const SYS_STATS: u64 = 6;

/// System call: debug operations (future)
pub const SYS_DEBUG: u64 = 7;

/// System call: memory map operation (future)
pub const SYS_MMAP: u64 = 8;

/// System call: memory unmap operation (future)
pub const SYS_MUNMAP: u64 = 9;

/// System call: create new task/process (future)
pub const SYS_FORK: u64 = 10;

/// System call: execute new program (future)
pub const SYS_EXEC: u64 = 11;

/// System call: wait for child process (future)
pub const SYS_WAIT: u64 = 12;

/// System call: get current task ID
pub const SYS_GETTID: u64 = 13;

/// System call: get process ID (future)
pub const SYS_GETPID: u64 = 14;

/// System call: sleep for specified time
pub const SYS_SLEEP: u64 = 15;

/// System call: file operations - open (future)
pub const SYS_OPEN: u64 = 16;

/// System call: file operations - read (future)
pub const SYS_READ: u64 = 17;

/// System call: file operations - write (future)
pub const SYS_WRITE: u64 = 18;

/// System call: file operations - close (future)
pub const SYS_CLOSE: u64 = 19;

/// System call: set task priority (future)
pub const SYS_SETPRIORITY: u64 = 20;

/// System call: get kernel feature flags
pub const SYS_GET_FEATURES: u64 = 21;

// ============================================================================
// Polyglot Runtime System Calls (100-109)
// ============================================================================

/// System call: create a new polyglot sandbox
/// Args: manifest_ptr, manifest_len, flags
/// Returns: sandbox_id on success, error code on failure
pub const SYS_POLYGLOT_CREATE: u64 = 100;

/// System call: execute code in a polyglot sandbox
/// Args: sandbox_id, entry_point_ptr, entry_point_len, args_ptr
/// Returns: 0 on success, error code on failure
pub const SYS_POLYGLOT_EXEC: u64 = 101;

/// System call: terminate a polyglot sandbox
/// Args: sandbox_id, flags
/// Returns: 0 on success, error code on failure
pub const SYS_POLYGLOT_TERMINATE: u64 = 102;

/// System call: get polyglot sandbox status
/// Args: sandbox_id, status_ptr, status_len
/// Returns: 0 on success, error code on failure
pub const SYS_POLYGLOT_STATUS: u64 = 103;

/// System call: send message via polyglot bridge
/// Args: sandbox_id, dest_id, msg_ptr, msg_len
/// Returns: 0 on success, error code on failure
pub const SYS_POLYGLOT_SEND: u64 = 104;

/// System call: receive message via polyglot bridge
/// Args: sandbox_id, buf_ptr, buf_len, timeout_ms
/// Returns: bytes received on success, error code on failure
pub const SYS_POLYGLOT_RECV: u64 = 105;

/// System call: get polyglot runtime metrics
/// Args: metrics_ptr, metrics_len
/// Returns: 0 on success, error code on failure
pub const SYS_POLYGLOT_METRICS: u64 = 106;

/// Maximum system call number (for validation)
pub const SYS_MAX: u64 = 106;

/// System call information structure
#[derive(Debug, Clone, Copy)]
pub struct SyscallInfo {
    /// System call number
    pub number: u64,
    
    /// System call name
    pub name: &'static str,
    
    /// Number of arguments
    pub arg_count: u8,
    
    /// Whether the syscall is implemented
    pub implemented: bool,
    
    /// Brief description
    pub description: &'static str,
}

/// System call table with metadata
pub const SYSCALL_TABLE: &[SyscallInfo] = &[
    SyscallInfo {
        number: SYS_YIELD,
        name: "yield",
        arg_count: 0,
        implemented: true,
        description: "Yield CPU to another task",
    },
    SyscallInfo {
        number: SYS_EXIT,
        name: "exit",
        arg_count: 1,
        implemented: true,
        description: "Exit current task with status code",
    },
    SyscallInfo {
        number: SYS_SEND,
        name: "send",
        arg_count: 3,
        implemented: true,
        description: "Send message to another task",
    },
    SyscallInfo {
        number: SYS_RECV,
        name: "recv",
        arg_count: 2,
        implemented: true,
        description: "Receive message from another task",
    },
    SyscallInfo {
        number: SYS_CHAN_CREATE,
        name: "chan_create",
        arg_count: 1,
        implemented: true,
        description: "Create IPC channel",
    },
    SyscallInfo {
        number: SYS_EXEC,
        name: "exec",
        arg_count: 4,
        implemented: true,
        description: "Load and execute user task image",
    },
    SyscallInfo {
        number: SYS_STATS,
        name: "stats",
        arg_count: 2,
        implemented: true,
        description: "Get system statistics",
    },
    SyscallInfo {
        number: SYS_DEBUG,
        name: "debug",
        arg_count: 2,
        implemented: true,
        description: "Debug operations: op=1 (audit entries), op=2 (scheduler/IPC counters)",
    },
    SyscallInfo {
        number: SYS_GETTID,
        name: "gettid",
        arg_count: 0,
        implemented: false,
        description: "Get current task ID",
    },
    SyscallInfo {
        number: SYS_SLEEP,
        name: "sleep",
        arg_count: 1,
        implemented: false,
        description: "Sleep for specified milliseconds",
    },
    SyscallInfo {
        number: SYS_GET_FEATURES,
        name: "get_features",
        arg_count: 0,
        implemented: true,
        description: "Get kernel feature flags bitset",
    },
    // Polyglot Runtime System Calls
    SyscallInfo {
        number: SYS_POLYGLOT_CREATE,
        name: "polyglot_create",
        arg_count: 3,
        implemented: true,
        description: "Create a new polyglot sandbox from manifest",
    },
    SyscallInfo {
        number: SYS_POLYGLOT_EXEC,
        name: "polyglot_exec",
        arg_count: 4,
        implemented: true,
        description: "Execute code in a polyglot sandbox",
    },
    SyscallInfo {
        number: SYS_POLYGLOT_TERMINATE,
        name: "polyglot_terminate",
        arg_count: 2,
        implemented: true,
        description: "Terminate a polyglot sandbox",
    },
    SyscallInfo {
        number: SYS_POLYGLOT_STATUS,
        name: "polyglot_status",
        arg_count: 3,
        implemented: true,
        description: "Get polyglot sandbox status",
    },
    SyscallInfo {
        number: SYS_POLYGLOT_SEND,
        name: "polyglot_send",
        arg_count: 4,
        implemented: true,
        description: "Send message via polyglot bridge",
    },
    SyscallInfo {
        number: SYS_POLYGLOT_RECV,
        name: "polyglot_recv",
        arg_count: 4,
        implemented: true,
        description: "Receive message via polyglot bridge",
    },
    SyscallInfo {
        number: SYS_POLYGLOT_METRICS,
        name: "polyglot_metrics",
        arg_count: 2,
        implemented: true,
        description: "Get polyglot runtime metrics",
    },
];

/// Get system call information by number
/// 
/// # Arguments
/// * `syscall_num` - System call number to look up
/// 
/// # Returns
/// `Some(SyscallInfo)` if found, `None` if invalid
pub fn get_syscall_info(syscall_num: u64) -> Option<&'static SyscallInfo> {
    SYSCALL_TABLE.iter().find(|info| info.number == syscall_num)
}

/// Get system call name by number
/// 
/// # Arguments
/// * `syscall_num` - System call number to look up
/// 
/// # Returns
/// System call name or "unknown" if invalid
pub fn get_syscall_name(syscall_num: u64) -> &'static str {
    get_syscall_info(syscall_num)
        .map(|info| info.name)
        .unwrap_or("unknown")
}

/// Check if a system call is implemented
/// 
/// # Arguments
/// * `syscall_num` - System call number to check
/// 
/// # Returns
/// `true` if implemented, `false` otherwise
pub fn is_syscall_implemented(syscall_num: u64) -> bool {
    get_syscall_info(syscall_num)
        .map(|info| info.implemented)
        .unwrap_or(false)
}

/// Validate system call number
/// 
/// # Arguments
/// * `syscall_num` - System call number to validate
/// 
/// # Returns
/// `true` if valid, `false` otherwise
pub fn is_valid_syscall(syscall_num: u64) -> bool {
    syscall_num > 0 && syscall_num <= SYS_MAX
}

/// Get the number of implemented system calls
/// 
/// # Returns
/// Count of implemented system calls
pub fn get_syscall_count() -> usize {
    SYSCALL_TABLE.iter().filter(|info| info.implemented).count()
}

/// Get the total number of defined system calls
/// 
/// # Returns
/// Total count of defined system calls
pub fn get_total_syscall_count() -> usize {
    SYSCALL_TABLE.len()
}

/// Print system call table for debugging
pub fn print_syscall_table() {
    use crate::kprintln;
    
    kprintln!("");
    kprintln!("=== SYSTEM CALL TABLE ===");
    kprintln!("Implemented syscalls:");
    
    for info in SYSCALL_TABLE.iter().filter(|info| info.implemented) {
        kprintln!("  {} ({}): {} - {}", 
                  info.number, info.name, info.arg_count, info.description);
    }
    
    kprintln!("");
    kprintln!("Future syscalls:");
    
    for info in SYSCALL_TABLE.iter().filter(|info| !info.implemented) {
        kprintln!("  {} ({}): {} - {} [NOT IMPLEMENTED]", 
                  info.number, info.name, info.arg_count, info.description);
    }
    
    kprintln!("");
    kprintln!("Total: {} syscalls ({} implemented, {} planned)", 
              get_total_syscall_count(), get_syscall_count(), 
              get_total_syscall_count() - get_syscall_count());
    kprintln!("=== END SYSTEM CALL TABLE ===");
    kprintln!("");
}

/// System call argument validation
pub struct SyscallArgs {
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub arg3: u64,
}

impl SyscallArgs {
    /// Create new syscall arguments
    pub const fn new(arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> Self {
        Self { arg0, arg1, arg2, arg3 }
    }
    
    /// Get argument by index
    pub fn get(&self, index: usize) -> Option<u64> {
        match index {
            0 => Some(self.arg0),
            1 => Some(self.arg1),
            2 => Some(self.arg2),
            3 => Some(self.arg3),
            _ => None,
        }
    }
    
    /// Validate argument count for syscall
    pub fn validate_count(&self, syscall_num: u64) -> bool {
        if let Some(info) = get_syscall_info(syscall_num) {
            // For now, we accept any number of arguments
            // In a full implementation, we would validate the exact count
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syscall_lookup() {
        assert_eq!(get_syscall_name(SYS_YIELD), "yield");
        assert_eq!(get_syscall_name(SYS_EXIT), "exit");
        assert_eq!(get_syscall_name(999), "unknown");
    }
    
    #[test]
    fn test_syscall_validation() {
        assert!(is_valid_syscall(SYS_YIELD));
        assert!(is_valid_syscall(SYS_EXIT));
        assert!(!is_valid_syscall(0));
        assert!(!is_valid_syscall(999));
    }
    
    #[test]
    fn test_implementation_status() {
        assert!(is_syscall_implemented(SYS_YIELD));
        assert!(is_syscall_implemented(SYS_EXIT));
        assert!(!is_syscall_implemented(SYS_SEND));
    }
    
    #[test]
    fn test_syscall_args() {
        let args = SyscallArgs::new(1, 2, 3, 4);
        assert_eq!(args.get(0), Some(1));
        assert_eq!(args.get(1), Some(2));
        assert_eq!(args.get(4), None);
        assert!(args.validate_count(SYS_YIELD));
    }
}
