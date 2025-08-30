/// System Call Module for Polymera OS
/// 
/// This module implements the system call interface, providing a bridge between
/// user space applications and kernel functionality.

pub mod table;
pub mod handlers;
pub mod validate;
pub mod copy;
pub mod schema_validation;
pub mod conformance_test;

#[cfg(debug_assertions)]
pub mod test;

use crate::{kprintln, klog};
use x86_64::structures::idt::InterruptStackFrame;

/// System call statistics for monitoring and debugging
static SYSCALL_COUNT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static INVALID_SYSCALL_COUNT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Initialize the system call subsystem
/// 
/// Sets up syscall handlers and any necessary state for system call processing.
pub fn init_syscalls() {
    kprintln!("[SYSCALL] Initializing system call subsystem");
    
    // System call infrastructure is initialized
    // The actual syscall entry point is installed in the IDT
    
    klog!(INFO, "[SYSCALL] System call subsystem initialized");
    klog!(INFO, "[SYSCALL] Available syscalls: {} (yield, exit)", table::get_syscall_count());
}

/// System call entry point from interrupt handler
/// 
/// This function is called by the syscall interrupt handler (int 0x80)
/// and dispatches to the appropriate system call implementation.
/// 
/// # Arguments
/// * `syscall_num` - System call number
/// * `arg0` - First argument
/// * `arg1` - Second argument  
/// * `arg2` - Third argument
/// * `arg3` - Fourth argument
/// 
/// # Returns
/// System call return value
pub fn syscall_entry(syscall_num: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    // Increment syscall counter
    SYSCALL_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    
    // Get current task ID for audit logging
    let current_pid = crate::sched::get_current_task_id();
    let current_tid = current_pid; // TODO: Implement thread ID tracking
    
    // Record syscall entry audit event
    crate::secman::audit_codes::audit_syscall_entry!(syscall_num, current_pid, current_tid);
    
    // Record start time for duration tracking
    let start_time = core::time::Instant::now();
    
    klog!(TRACE, "[SYSCALL] Entry: num={}, args=({}, {}, {}, {})", 
          syscall_num, arg0, arg1, arg2, arg3);
    
    // Dispatch to handler
    let result = handlers::dispatch(syscall_num, arg0, arg1, arg2, arg3);
    
    // Calculate duration
    let duration = start_time.elapsed();
    let duration_ns = duration.as_nanos() as u64;
    
    // Record syscall exit audit event
    crate::secman::audit_codes::audit_syscall_exit!(syscall_num, current_pid, current_tid, result, duration_ns);
    
    if result == u64::MAX {
        // Invalid syscall
        INVALID_SYSCALL_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        klog!(TRACE, "[SYSCALL] Invalid syscall number: {}", syscall_num);
    } else {
        klog!(TRACE, "[SYSCALL] Return: {}", result);
    }
    
    result
}

/// Rust entry point for assembly syscall wrapper
/// 
/// This function is called by the assembly syscall_entry_asm function
/// and provides the interface between assembly and Rust code.
#[no_mangle]
pub extern "C" fn syscall_entry_rust(syscall_num: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    syscall_entry(syscall_num, arg0, arg1, arg2, arg3)
}

/// System call interrupt handler (int 0x80)
/// 
/// This is the low-level interrupt handler that receives syscalls from user space.
/// It extracts arguments from registers and calls the syscall dispatcher.
pub extern "x86-interrupt" fn syscall_interrupt_handler(stack_frame: InterruptStackFrame) {
    // Extract syscall number and arguments from registers
    // The assembly trampoline has already saved all registers and set up arguments
    
    // Get syscall number from RAX (saved on stack by assembly)
    // Arguments are already in the correct registers for the dispatcher
    
    klog!(TRACE, "[SYSCALL] Interrupt handler at RIP: 0x{:016x}", stack_frame.instruction_pointer);
    
    // The assembly trampoline will handle the actual syscall dispatch
    // This function is now just a placeholder - the real work happens in assembly
    
    // Note: In the new implementation, the assembly trampoline:
    // 1. Saves all registers
    // 2. Sets up kernel data segments
    // 3. Calls handlers::dispatch directly
    // 4. Restores registers
    // 5. Returns to user space with iretq
    
    // This handler is kept for compatibility but is no longer the primary entry point
}

/// Get system call statistics
/// 
/// # Returns
/// (total_syscalls, invalid_syscalls)
pub fn get_syscall_stats() -> (u64, u64) {
    let total = SYSCALL_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    let invalid = INVALID_SYSCALL_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    (total, invalid)
}

/// Print system call statistics
pub fn print_syscall_stats() {
    let (total, invalid) = get_syscall_stats();
    let success = total - invalid;
    let success_rate = if total > 0 {
        (success as f32 / total as f32) * 100.0
    } else {
        100.0
    };
    
    kprintln!("");
    kprintln!("=== SYSCALL STATISTICS ===");
    kprintln!("Total syscalls: {}", total);
    kprintln!("Successful: {}", success);
    kprintln!("Invalid: {}", invalid);
    kprintln!("Success rate: {:.1}%", success_rate);
    kprintln!("=== END SYSCALL STATISTICS ===");
    kprintln!("");
}

/// User space syscall wrapper (would be in userland library)
/// 
/// This demonstrates how user space code would invoke system calls.
/// In a real system, this would be in a user space library (libc).
#[allow(dead_code)]
pub mod userspace {
    use super::table::*;
    
    /// User space wrapper for yield syscall
    pub fn yield_cpu() {
        unsafe {
            // In real implementation, this would use inline assembly
            // to invoke int 0x80 with syscall number in RAX
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_YIELD,
                in("rdi") 0u64,
                in("rsi") 0u64,
                in("rdx") 0u64,
                in("rcx") 0u64,
                options(nostack)
            );
        }
    }
    
    /// User space wrapper for exit syscall
    pub fn exit(code: i32) -> ! {
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_EXIT,
                in("rdi") code as u64,
                in("rsi") 0u64,
                in("rdx") 0u64,
                in("rcx") 0u64,
                options(nostack, noreturn)
            );
        }
    }
}

/// Test function for syscall functionality
#[allow(dead_code)]
pub fn test_syscalls() {
    kprintln!("Testing syscall functionality...");
    
    // Test direct syscall invocation
    kprintln!("  Testing yield syscall...");
    let result = syscall_entry(table::SYS_YIELD, 0, 0, 0, 0);
    kprintln!("    Yield result: {}", result);
    
    kprintln!("  Testing invalid syscall...");
    let result = syscall_entry(999, 0, 0, 0, 0);
    kprintln!("    Invalid result: {}", result);
    
    // Print statistics
    print_syscall_stats();
    
    kprintln!("Syscall test completed");
}

/// System call error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallError {
    /// Invalid system call number
    InvalidSyscall = 1,
    
    /// Invalid argument
    InvalidArgument = 2,
    
    /// Permission denied
    PermissionDenied = 3,
    
    /// Resource not found
    NotFound = 4,
    
    /// Resource busy
    Busy = 5,
    
    /// No memory available
    NoMemory = 6,
    
    /// Operation not supported
    NotSupported = 7,
}

impl SyscallError {
    /// Convert error to return value
    pub fn to_errno(self) -> u64 {
        (-(self as i32)) as u64
    }
}

/// System call return type
pub type SyscallResult = Result<u64, SyscallError>;
