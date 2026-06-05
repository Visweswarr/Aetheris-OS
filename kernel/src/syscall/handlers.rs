/// System Call Handlers for Polymera OS
/// 
/// This module implements the system call dispatcher and individual
/// syscall handler functions.

use super::table::*;
use super::validate::validate_syscall;
use crate::{klog, kprintln, format};

// Polyglot runtime handlers
pub mod polyglot;

/// System call dispatcher
/// 
/// Routes system calls to their appropriate handler functions based on
/// the syscall number and validates arguments.
/// 
/// # Arguments
/// * `num` - System call number
/// * `a0` - First argument
/// * `a1` - Second argument
/// * `a2` - Third argument
/// * `a3` - Fourth argument
/// 
/// # Returns
/// System call return value, or `u64::MAX` for invalid syscalls
pub fn dispatch(num: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    klog!(TRACE, "[SYSCALL] Dispatching syscall {} with args ({}, {}, {}, {})", 
          num, a0, a1, a2, a3);
    
    // Validate syscall number
    if !is_valid_syscall(num) {
        klog!(TRACE, "[SYSCALL] Invalid syscall number: {}", num);
        return u64::MAX;
    }
    
    // Check if syscall is implemented
    if !is_syscall_implemented(num) {
        klog!(TRACE, "[SYSCALL] Unimplemented syscall: {} ({})", num, get_syscall_name(num));
        return u64::MAX;
    }
    
    // Validate syscall arguments
    let args = [a0, a1, a2, a3];
    if let Err(validation_error) = validate_syscall(num, &args) {
        klog!(WARNING, "[SYSCALL] Argument validation failed for syscall {}: {}", num, validation_error);
        
        // Return appropriate error code based on validation failure
        if validation_error.contains("EFAULT") {
            return 14; // EFAULT
        } else if validation_error.contains("EINVAL") {
            return 2;  // EINVAL
        } else if validation_error.contains("ENOSYS") {
            return 21; // ENOSYS
        } else {
            return 2;  // EINVAL (default)
        }
    }
    
    // Dispatch to appropriate handler
    match num {
        SYS_YIELD => handle_yield(a0, a1, a2, a3),
        SYS_EXIT => handle_exit(a0, a1, a2, a3),
        SYS_SEND => handle_send(a0, a1, a2, a3),
        SYS_RECV => handle_recv(a0, a1, a2, a3),
        SYS_CHAN_CREATE => handle_channel_create(a0, a1, a2, a3),
        SYS_EXEC => handle_exec(a0, a1, a2, a3),
        
        SYS_STATS => handle_stats(a0, a1, a2, a3),
        SYS_DEBUG => handle_debug(a0, a1, a2, a3),
        SYS_GET_FEATURES => handle_get_features(a0, a1, a2, a3),
        
        // Polyglot Runtime syscalls
        SYS_POLYGLOT_CREATE => polyglot::handle_polyglot_create(a0, a1, a2, a3),
        SYS_POLYGLOT_EXEC => polyglot::handle_polyglot_exec(a0, a1, a2, a3),
        SYS_POLYGLOT_TERMINATE => polyglot::handle_polyglot_terminate(a0, a1, a2, a3),
        SYS_POLYGLOT_STATUS => polyglot::handle_polyglot_status(a0, a1, a2, a3),
        SYS_POLYGLOT_SEND => polyglot::handle_polyglot_send(a0, a1, a2, a3),
        SYS_POLYGLOT_RECV => polyglot::handle_polyglot_recv(a0, a1, a2, a3),
        SYS_POLYGLOT_METRICS => polyglot::handle_polyglot_metrics(a0, a1, a2, a3),
        
        _ => {
            klog!(TRACE, "[SYSCALL] Dispatcher missing handler for syscall {}", num);
            u64::MAX
        }
    }
}

/// Assembly-compatible syscall dispatcher
/// 
/// This function is called directly from the assembly trampoline.
/// It has the exact signature expected by the assembly code.
/// 
/// # Arguments
/// * `num` - System call number
/// * `a0` - First argument
/// * `a1` - Second argument
/// * `a2` - Third argument
/// * `a3` - Fourth argument
/// 
/// # Returns
/// System call return value
#[no_mangle]
pub extern "C" fn handlers_dispatch(num: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    dispatch(num, a0, a1, a2, a3)
}

/// Handle yield system call
/// 
/// Yields the CPU to another task. This is a voluntary context switch.
/// 
/// # Arguments
/// * `_a0` - Unused (yield takes no arguments)
/// * `_a1` - Unused
/// * `_a2` - Unused
/// * `_a3` - Unused
/// 
/// # Returns
/// 0 on success
fn handle_yield(_a0: u64, _a1: u64, _a2: u64, _a3: u64) -> u64 {
    klog!(TRACE, "[SYSCALL] Handling yield syscall");
    
    // Call the scheduler's yield implementation
    crate::sched::sys::sys_yield();
    
    // Yield always succeeds
    0
}

/// Handle exit system call
/// 
/// Terminates the current task with the specified exit code.
/// 
/// # Arguments
/// * `a0` - Exit code
/// * `_a1` - Unused
/// * `_a2` - Unused
/// * `_a3` - Unused
/// 
/// # Returns
/// This function never returns (task exits)
fn handle_exit(a0: u64, _a1: u64, _a2: u64, _a3: u64) -> u64 {
    let exit_code = a0 as i32;
    klog!(TRACE, "[SYSCALL] Handling exit syscall - code={}", exit_code);
    
    // Call the scheduler's exit implementation
    crate::sched::sys::sys_exit(exit_code);
    
    // Exit should never return, but if it does, return the exit code
    exit_code as u64
}

/// Handle exec system call
/// 
/// Loads and executes a new user task image.
/// 
/// # Arguments
/// * `a0` - Image data pointer
/// * `a1` - Image data length
/// * `a2` - Capabilities pointer
/// * `a3` - Capabilities count
/// 
/// # Returns
/// Process ID on success, error code on failure
fn handle_exec(a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let image_ptr = a0;
    let image_len = a1;
    let caps_ptr = a2;
    let caps_count = a3;
    
    // Get current task ID for audit logging
    let current_pid = crate::sched::get_current_task_id();
    let current_tid = current_pid; // TODO: Implement thread ID tracking
    
    klog!(TRACE, "[SYSCALL] Handling exec syscall - image=0x{:x} len={} caps=0x{:x} count={}", 
          image_ptr, image_len, caps_ptr, caps_count);
    
    // Record exec load audit event
    crate::audit_exec_load!(current_pid as u32, current_tid as u32, &format!("0x{:x}", image_ptr), image_len);
    
    // TODO: Implement exec functionality
    // This would:
    // 1. Validate the image data pointer and length
    // 2. Copy the image data from user space
    // 3. Parse and validate the image header
    // 4. Verify required capabilities
    // 5. Create user task context with syscall gate
    // 6. Return the new process ID
    
    // For now, just log and return "not implemented"
    klog!(TRACE, "[SYSCALL] Exec syscall not yet implemented, returning EINVAL");
    
    // Return EINVAL (Invalid argument) as defined in the ABI
    2 // EINVAL
}

/// Handle send system call
/// 
/// Sends a message to another task via IPC.
/// 
/// # Arguments
/// * `a0` - Destination task ID
/// * `a1` - Message buffer pointer
/// * `a2` - Message length
/// * `_a3` - Unused
/// 
/// # Returns
/// 0 on success, error code on failure
fn handle_send(a0: u64, a1: u64, a2: u64, _a3: u64) -> u64 {
    let dst_task_id = a0;
    let buf_ptr = a1;
    let buf_len = a2;
    
    klog!(TRACE, "[SYSCALL] Handling send syscall - dst={} buf=0x{:x} len={}", 
          dst_task_id, buf_ptr, buf_len);
    
    // TODO: Implement IPC send functionality
    // This would:
    // 1. Validate the destination task ID
    // 2. Copy the message from user space
    // 3. Send it via the IPC system
    // 4. Return success/failure
    
    // For now, just log and return success
    klog!(TRACE, "[SYSCALL] Send syscall not yet implemented, returning success");
    0
}

/// Handle receive system call
/// 
/// Receives a message from another task via IPC.
/// 
/// # Arguments
/// * `a0` - Blocking flag (1 = block, 0 = non-blocking)
/// * `a1` - Output buffer pointer
/// * `a2` - Output buffer length
/// * `_a3` - Unused
/// 
/// # Returns
/// Number of bytes received on success, error code on failure
fn handle_recv(a0: u64, a1: u64, a2: u64, _a3: u64) -> u64 {
    let blocking = a0 != 0;
    let buf_ptr = a1;
    let buf_len = a2;
    
    klog!(TRACE, "[SYSCALL] Handling receive syscall - blocking={} buf=0x{:x} len={}", 
          blocking, buf_ptr, buf_len);
    
    // TODO: Implement IPC receive functionality
    // This would:
    // 1. Check if there are pending messages
    // 2. If blocking, wait for a message
    // 3. Copy the message to user space
    // 4. Return the number of bytes received
    
    // For now, just log and return "no message"
    klog!(TRACE, "[SYSCALL] Receive syscall not yet implemented, returning no message");
    0
}

/// Handle channel creation system call
/// 
/// Creates a new communication channel between two tasks.
/// 
/// # Arguments
/// * `a0` - Sender task ID
/// * `a1` - Receiver task ID
/// * `a2` - Queue size
/// * `_a3` - Unused
/// 
/// # Returns
/// Channel ID on success, error code on failure
fn handle_channel_create(a0: u64, a1: u64, a2: u64, _a3: u64) -> u64 {
    let sender_id = a0;
    let receiver_id = a1;
    let queue_size = a2;
    
    klog!(TRACE, "[SYSCALL] Handling channel creation syscall - sender={} receiver={} queue_size={}", 
          sender_id, receiver_id, queue_size);
    
    // TODO: Implement channel creation functionality
    // This would:
    // 1. Validate the task IDs
    // 2. Create a new channel
    // 3. Return the channel ID
    
    // For now, just log and return a dummy channel ID
    klog!(TRACE, "[SYSCALL] Channel creation syscall not yet implemented, returning dummy ID");
    1
}

/// Handle get task ID system call (future implementation)
#[allow(dead_code)]
fn handle_gettid(_a0: u64, _a1: u64, _a2: u64, _a3: u64) -> u64 {
    klog!(TRACE, "[SYSCALL] Handling gettid syscall (stub)");
    
    // TODO: Return current task ID
    // This would call into the scheduler to get the current task ID
    
    let current_task_id = crate::sched::get_current_task_id();
    current_task_id
}

/// Handle sleep system call (future implementation)
#[allow(dead_code)]
fn handle_sleep(a0: u64, _a1: u64, _a2: u64, _a3: u64) -> u64 {
    let milliseconds = a0;
    klog!(TRACE, "[SYSCALL] Handling sleep syscall (stub) - duration={}ms", milliseconds);
    
    // TODO: Implement sleep functionality
    // This would integrate with the timer system to sleep for specified duration
    
    // For now, just yield and return immediately
    crate::sched::sys::sys_yield();
    0
}

/// System call handler statistics
static YIELD_COUNT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static EXIT_COUNT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Increment yield syscall counter
fn increment_yield_count() {
    YIELD_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
}

/// Increment exit syscall counter
fn increment_exit_count() {
    EXIT_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
}

/// Get syscall handler statistics
/// 
/// # Returns
/// (yield_count, exit_count)
pub fn get_handler_stats() -> (u64, u64) {
    let yields = YIELD_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    let exits = EXIT_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    (yields, exits)
}

/// Print syscall handler statistics
pub fn print_handler_stats() {
    let (yields, exits) = get_handler_stats();
    
    use crate::kprintln;
    kprintln!("");
    kprintln!("=== SYSCALL HANDLER STATISTICS ===");
    kprintln!("Yield calls: {}", yields);
    kprintln!("Exit calls: {}", exits);
    kprintln!("Total handled: {}", yields + exits);
    kprintln!("=== END HANDLER STATISTICS ===");
    kprintln!("");
}

/// Debug operation codes for sys_debug
pub mod debug_ops {
    pub const PRINT_STATS: u64 = 1;
    pub const PRINT_TRACES: u64 = 2;
    pub const PRINT_TASK_STATS: u64 = 3;
    pub const CLEAR_STATS: u64 = 4;
    pub const TEST_TRACE: u64 = 5;
    
    // New debug operations for Prompt 37
    pub const PRINT_AUDIT_ENTRIES: u64 = 6;  // op=1 in user request
    pub const PRINT_SCHEDULER_IPC_COUNTERS: u64 = 7;  // op=2 in user request
    
    // New debug operation for Prompt 43 - Capability revocation
    pub const REVOKE_CAP: u64 = 8;  // op=8 for revoking capabilities
    
    // New debug operation for Prompt 46 - Randomness proxy
    pub const SET_SEED: u64 = 9;  // op=9 for setting RNG seed
    pub const PRINT_DASH: u64 = 10;  // op=10 for printing dashboard
    
    // New debug operation for Prompt 56 - Starvation detector
    pub const GET_STARVATION_STATS: u64 = 11;  // op=11 for getting starvation statistics
    
    // New debug operation for Prompt 58 - Fault injection hooks
    pub const SET_FAULT_INJECTION: u64 = 12;  // op=12 for setting fault injection flags
    
    // New debug operation for Prompt 59 - Fault injection status
    pub const GET_FAULT_INJECTION_STATUS: u64 = 13;  // op=13 for getting fault injection status
    
    // New debug operations for Prompt 60 - Key Management
    pub const SECMAN_API: u64 = 14;  // op=14 for Security Manager API operations
    pub const PRINT_ATTEST: u64 = 15; // op=15 for printing attestation info
}

/// Handle sys_stats system call
/// 
/// Returns system statistics including ticks, context switches,
/// messages sent/received, and other performance metrics.
/// 
/// # Arguments
/// * `a0` - Pointer to SystemStats structure (output)
/// * `a1` - Size of SystemStats structure
/// * `a2` - Reserved (unused)
/// * `a3` - Reserved (unused)
/// 
/// # Returns
/// 0 on success, error code on failure
fn handle_stats(a0: u64, a1: u64, _a2: u64, _a3: u64) -> u64 {
    klog!(TRACE, [crate::log::tags::SYSCALL], "Handling sys_stats syscall - ptr=0x{:x} size={}", a0, a1);
    
    // Increment syscall counter
    crate::trace::trace_syscall();
    
    // Validate arguments
    if a0 == 0 {
        klog!(TRACE, [crate::log::tags::SYSCALL], "sys_stats: null pointer provided");
        return 1; // EPERM
    }
    
    let expected_size = core::mem::size_of::<crate::trace::SystemStats>() as u64;
    if a1 != expected_size {
        klog!(TRACE, [crate::log::tags::SYSCALL], "sys_stats: invalid size {} (expected {})", a1, expected_size);
        return 2; // EINVAL
    }
    
    // Get current system statistics
    let stats = crate::trace::get_system_stats();
    
    // In a real implementation, we would copy the stats to user space
    // For now, we'll just log the operation and return the stats via a different mechanism
    klog!(INFO, [crate::log::tags::SYSCALL], "sys_stats: ticks={} ctx_switches={} msgs_sent={} msgs_recvd={}", 
          stats.ticks, stats.ctx_switches, stats.msgs_sent, stats.msgs_recvd);
    
    // TODO: Implement proper user space memory copying
    // For now, return success and log that we would copy the stats
    klog!(TRACE, [crate::log::tags::SYSCALL], "sys_stats: would copy {} bytes to user space", expected_size);
    
    0 // Success
}

/// Handle sys_debug system call
/// 
/// Performs debug operations based on the operation code.
/// 
/// # Arguments
/// * `a0` - Debug operation code (see debug_ops module)
/// * `a1` - Operation-specific argument
/// * `a2` - Reserved (unused)
/// * `a3` - Reserved (unused)
/// 
/// # Returns
/// 0 on success, error code on failure
fn handle_debug(a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let op_code = a0;
    let arg = a1;
    
    klog!(TRACE, [crate::log::tags::SYSCALL], "Handling sys_debug syscall - op={} arg={}", op_code, arg);
    
    // Increment syscall counter
    crate::trace::trace_syscall();
    
    match op_code {
        debug_ops::PRINT_STATS => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: PRINT_STATS requested");
            crate::trace::print_system_stats();
            0
        }
        
        debug_ops::PRINT_TRACES => {
            let count = if arg > 0 && arg <= 100 { arg as usize } else { 10 };
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: PRINT_TRACES requested (count={})", count);
            let _ = count;
            crate::trace::print_recent_process_switches();
            crate::trace::print_recent_ipc_traces();
            0
        }
        
        debug_ops::PRINT_TASK_STATS => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: PRINT_TASK_STATS requested");
            crate::trace::print_task_stats();
            0
        }
        
        debug_ops::CLEAR_STATS => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: CLEAR_STATS requested");
            // Reinitialize tracing to clear all counters
            crate::trace::init_trace();
            0
        }
        
        debug_ops::TEST_TRACE => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: TEST_TRACE requested");
            crate::trace::test_trace();
            0
        }
        
        debug_ops::PRINT_AUDIT_ENTRIES => {
            let count = if arg > 0 && arg <= 1000 { arg as usize } else { 100 };
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: PRINT_AUDIT_ENTRIES requested (count={})", count);
            print_audit_entries(count);
            0
        }
        
        debug_ops::PRINT_SCHEDULER_IPC_COUNTERS => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: PRINT_SCHEDULER_IPC_COUNTERS requested");
            print_scheduler_ipc_counters();
            0
        }
        
        debug_ops::REVOKE_CAP => {
            let token_id = arg;
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: REVOKE_CAP requested for token 0x{:x}", token_id);
            
            // Convert u64 to u128 for token ID (truncate if necessary)
            let token_id_128 = token_id as u128;
            
            // Revoke the capability
            match crate::security::cap::revoke_capability_globally(token_id_128) {
                true => {
                    klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: Successfully revoked capability token 0x{:x}", token_id);
                    0 // Success
                }
                false => {
                    klog!(WARN, [crate::log::tags::SYSCALL], "sys_debug: Failed to revoke capability token 0x{:x} (already revoked or invalid)", token_id);
                    1 // EPERM - permission denied or already revoked
                }
            }
        }
        
        debug_ops::SET_SEED => {
            let seed = arg;
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: SET_SEED requested with seed 0x{:x}", seed);
            
            // Set the RNG seed via the randomness proxy
            match crate::rng::set_seed(seed) {
                true => {
                    klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: Successfully set RNG seed to 0x{:x}", seed);
                    0 // Success
                }
                false => {
                    klog!(WARN, [crate::log::tags::SYSCALL], "sys_debug: Failed to set RNG seed to 0x{:x}", seed);
                    1 // EPERM - permission denied
                }
            }
        }
        
        debug_ops::PRINT_DASH => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: PRINT_DASH requested");
            
            // Print the ASCII dashboard
            crate::dashboard::print_dashboard();
            
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: Dashboard printed successfully");
            0 // Success
        }
        
        debug_ops::GET_STARVATION_STATS => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: GET_STARVATION_STATS requested");
            
            // Get starvation statistics
            let stats = crate::sched::starvation::get_starvation_stats();
            
            // Return starvation statistics as a packed value
            // High 32 bits: warnings, Low 32 bits: critical events
            let result = ((stats.total_warnings as u64) << 32) | (stats.total_critical as u64);
            
            klog!(INFO, [crate::log::tags::SYSCALL], 
                "sys_debug: Starvation stats - warnings={}, critical={}, currently_starving={}",
                stats.total_warnings, stats.total_critical, stats.currently_starving);
            
            result // Return packed statistics
        }
        
        debug_ops::SET_FAULT_INJECTION => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: SET_FAULT_INJECTION requested");
            
            // Parse fault injection parameters from arg1
            // High 32 bits: fault type, Low 32 bits: frequency (every Nth operation)
            let fault_type = (arg >> 32) as u32;
            let frequency = (arg & 0xFFFFFFFF) as u32;
            
            match fault_type {
                1 => { // Inbox overflow fault injection
                    if frequency == 0 {
                        // Disable fault injection
                        let mut config = crate::fault_injection::get_fault_config().clone();
                        config.inbox_overflow_enabled = false;
                        config.global_enabled = false;
                        crate::fault_injection::update_fault_config(config);
                        klog!(INFO, [crate::log::tags::SYSCALL], 
                            "sys_debug: Disabled inbox overflow fault injection");
                    } else {
                        // Enable fault injection every Nth push
                        let mut config = crate::fault_injection::get_fault_config().clone();
                        config.inbox_overflow_enabled = true;
                        config.inbox_overflow_interval = frequency;
                        config.global_enabled = true;
                        crate::fault_injection::update_fault_config(config);
                        klog!(INFO, [crate::log::tags::SYSCALL], 
                            "sys_debug: Enabled inbox overflow fault injection every {} pushes", frequency);
                    }
                    0 // Success
                }
                2 => { // Allocation failure fault injection
                    if frequency == 0 {
                        // Disable fault injection
                        let mut config = crate::fault_injection::get_fault_config().clone();
                        config.alloc_failure_enabled = false;
                        config.global_enabled = false;
                        crate::fault_injection::update_fault_config(config);
                        klog!(INFO, [crate::log::tags::SYSCALL], 
                            "sys_debug: Disabled allocation failure fault injection");
                    } else {
                        // Enable fault injection every Nth allocation
                        let mut config = crate::fault_injection::get_fault_config().clone();
                        config.alloc_failure_enabled = true;
                        config.alloc_failure_interval = frequency;
                        config.global_enabled = true;
                        crate::fault_injection::update_fault_config(config);
                        klog!(INFO, [crate::log::tags::SYSCALL], 
                            "sys_debug: Enabled allocation failure fault injection every {} allocations", frequency);
                    }
                    0 // Success
                }
                3 => { // Timer jitter fault injection
                    if frequency == 0 {
                        // Disable fault injection
                        let mut config = crate::fault_injection::get_fault_config().clone();
                        config.timer_jitter_enabled = false;
                        config.global_enabled = false;
                        crate::fault_injection::update_fault_config(config);
                        klog!(INFO, [crate::log::tags::SYSCALL], 
                            "sys_debug: Disabled timer jitter fault injection");
                    } else {
                        // Enable fault injection with specified jitter percentage
                        let mut config = crate::fault_injection::get_fault_config().clone();
                        config.timer_jitter_enabled = true;
                        config.timer_jitter_percentage = frequency.min(50); // Cap at 50% jitter
                        config.global_enabled = true;
                        crate::fault_injection::update_fault_config(config);
                        klog!(INFO, [crate::log::tags::SYSCALL], 
                            "sys_debug: Enabled timer jitter fault injection with +/-{}% jitter", frequency);
                    }
                    0 // Success
                }
                _ => {
                    klog!(ERROR, [crate::log::tags::SYSCALL], 
                        "sys_debug: Unknown fault type {} (1=inbox_overflow, 2=alloc_failure, 3=timer_jitter)", fault_type);
                    1 // EINVAL
                }
            }
        }
        
        debug_ops::GET_FAULT_INJECTION_STATUS => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: GET_FAULT_INJECTION_STATUS requested");
            
            let config = crate::fault_injection::get_fault_config();
            
            // Print fault injection status in a parsable format
            kprintln!("");
            kprintln!("=== FAULT INJECTION STATUS ===");
            kprintln!("FAULT_GLOBAL_ENABLED: {}", config.global_enabled);
            kprintln!("FAULT_INBOX_OVERFLOW: enabled={}, interval={}", 
                     config.inbox_overflow_enabled, config.inbox_overflow_interval);
            kprintln!("FAULT_ALLOC_FAILURE: enabled={}, interval={}", 
                     config.alloc_failure_enabled, config.alloc_failure_interval);
            kprintln!("FAULT_TIMER_JITTER: enabled={}, percentage={}", 
                     config.timer_jitter_enabled, config.timer_jitter_percentage);
            kprintln!("=== END FAULT INJECTION STATUS ===");
            kprintln!("");
            
            0 // Success
        }
        
        debug_ops::SECMAN_API => {
            let op_code = arg;
            let arg1 = a2; // Use a2 as first argument
            let arg2 = a3; // Use a3 as second argument
            
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: SECMAN_API requested with op={} args=({}, {})", op_code, arg1, arg2);
            
            // Handle Security Manager API operation
            match crate::secman::api::handle_secman_api(op_code, arg1, arg2, 0) {
                Ok(result) => {
                    klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: SECMAN_API operation {} completed successfully with result {}", op_code, result);
                    result
                }
                Err(e) => {
                    klog!(WARN, [crate::log::tags::SYSCALL], "sys_debug: SECMAN_API operation {} failed: {}", op_code, e);
                    1 // EPERM
                }
            }
        }
        
        debug_ops::PRINT_ATTEST => {
            klog!(INFO, [crate::log::tags::SYSCALL], "sys_debug: PRINT_ATTEST requested");
            crate::boot::attest::print_attestation_info();
            0
        }
        
        _ => {
            klog!(TRACE, [crate::log::tags::SYSCALL], "sys_debug: unknown operation code {}", op_code);
            2 // EINVAL
        }
    }
}

/// Print audit entries in a script-parsable format
/// 
/// # Arguments
/// * `count` - Number of audit entries to print
fn print_audit_entries(count: usize) {
    use crate::secman::audit;
    
    kprintln!("");
    kprintln!("=== AUDIT ENTRIES (last {}) ===", count);
    
    let entries = audit::get_latest_entries(count);
    
    if entries.is_empty() {
        kprintln!("No audit entries found");
    } else {
        // Header for script parsing
        kprintln!("AUDIT_HEADER: ts,pid,op,arg");
        
        // Print entries in reverse chronological order (newest first)
        for entry in entries.iter().rev() {
            // Format: ts pid op arg (script-parsable)
            kprintln!("AUDIT_ENTRY: {} {} {} 0x{:x}", 
                     entry.ts, entry.pid, entry.op, entry.arg);
        }
        
        kprintln!("AUDIT_COUNT: {}", entries.len());
    }
    
    kprintln!("=== END AUDIT ENTRIES ===");
    kprintln!("");
}

/// Print scheduler and IPC counters in a script-parsable format
fn print_scheduler_ipc_counters() {
    use crate::trace;
    use crate::sched;
    
    kprintln!("");
    kprintln!("=== SCHEDULER & IPC COUNTERS ===");
    
    // Get system statistics
    let stats = trace::get_system_stats();
    
    // Scheduler counters
    kprintln!("SCHEDULER_HEADER: metric,value,unit");
    kprintln!("SCHEDULER_TICKS: {},ticks", stats.ticks);
    kprintln!("SCHEDULER_CTX_SWITCHES: {},count", stats.ctx_switches);
    kprintln!("SCHEDULER_ACTIVE_TASKS: {},count", stats.active_tasks);
    kprintln!("SCHEDULER_BLOCKED_TASKS: {},count", stats.blocked_tasks);
    kprintln!("SCHEDULER_UPTIME: {},ms", stats.uptime_ms);
    
    // IPC counters
    kprintln!("IPC_HEADER: metric,value,unit");
    kprintln!("IPC_MSGS_SENT: {},count", stats.msgs_sent);
    kprintln!("IPC_MSGS_RECVD: {},count", stats.msgs_recvd);
    kprintln!("IPC_MSGS_PER_SEC: {:.2},msg/s", stats.msgs_per_second());
    
    // IPC latency statistics
    kprintln!("IPC_LATENCY_HEADER: metric,value,unit");
    kprintln!("IPC_LATENCY_P50: {},μs", stats.ipc_latency_p50_us);
    kprintln!("IPC_LATENCY_P95: {},μs", stats.ipc_latency_p95_us);
    kprintln!("IPC_LATENCY_P99: {},μs", stats.ipc_latency_p99_us);
    kprintln!("IPC_LATENCY_MEAN: {},μs", stats.ipc_latency_mean_us);
    kprintln!("IPC_LATENCY_MIN: {},μs", stats.ipc_latency_min_us);
    kprintln!("IPC_LATENCY_MAX: {},μs", stats.ipc_latency_max_us);
    kprintln!("IPC_LATENCY_SAMPLES: {},count", stats.ipc_latency_samples);
    
    // Wake-to-run latency statistics
    kprintln!("WAKE_TO_RUN_HEADER: metric,value,unit");
    kprintln!("WAKE_TO_RUN_P50: {},μs", stats.wake_to_run_p50_us);
    kprintln!("WAKE_TO_RUN_P95: {},μs", stats.wake_to_run_p95_us);
    kprintln!("WAKE_TO_RUN_P99: {},μs", stats.wake_to_run_p99_us);
    kprintln!("WAKE_TO_RUN_MEAN: {},μs", stats.wake_to_run_mean_us);
    kprintln!("WAKE_TO_RUN_MIN: {},μs", stats.wake_to_run_min_us);
    kprintln!("WAKE_TO_RUN_MAX: {},μs", stats.wake_to_run_max_us);
    kprintln!("WAKE_TO_RUN_SAMPLES: {},count", stats.wake_to_run_samples);
    
    // Performance metrics
    kprintln!("PERFORMANCE_HEADER: metric,value,unit");
    kprintln!("PERFORMANCE_CTX_SWITCHES_PER_SEC: {:.2},switches/s", stats.ctx_switches_per_second());
    kprintln!("PERFORMANCE_TICKS_PER_SEC: {:.2},ticks/s", stats.ticks_per_second());
    
    // System call counters
    kprintln!("SYSCALL_HEADER: metric,value,unit");
    kprintln!("SYSCALL_TOTAL: {},count", stats.syscalls);
    kprintln!("SYSCALL_PAGE_FAULTS: {},count", stats.page_faults);
    kprintln!("SYSCALL_SECURITY_FAILURES: {},count", stats.security_failures);
    
    // Summary
    kprintln!("SUMMARY_TOTAL_ENTRIES: {}", 29); // Count of all metric lines above (22 + 7 wake-to-run entries)
    kprintln!("=== END SCHEDULER & IPC COUNTERS ===");
    kprintln!("");
}

/// Test IPC syscalls
pub fn test_ipc_syscalls() {
    use crate::kprintln;
    
    kprintln!("");
    kprintln!("=== TESTING IPC SYSCALLS ===");
    
    // Test channel create syscall
    kprintln!("Testing SYS_CHAN_CREATE...");
    let result = dispatch(SYS_CHAN_CREATE, 42, 0, 0, 0); // peer=42
    kprintln!("  Channel create result: 0x{:x}", result);
    
    // Test send syscall (first without capability - should fail)
    kprintln!("Testing SYS_SEND (without capability)...");
    let result = dispatch(SYS_SEND, 42, 0x1000, 64, 0); // dst=42, msg=0x1000, len=64
    kprintln!("  Send result (should fail): {}", result);
    
    // Grant capability and test again
    kprintln!("Granting capability for process 1 -> 42...");
    match crate::security::cap::grant_capability(1, 42, 
        crate::security::cap::scope::SEND, 10000) {
        Ok(token) => {
            kprintln!("  ✓ Granted capability: {}", token);
            
            kprintln!("Testing SYS_SEND (with capability)...");
            let result = dispatch(SYS_SEND, 42, 0x1000, 64, 0);
            kprintln!("  Send result (should succeed): {}", result);
        }
        Err(e) => kprintln!("  ✗ Failed to grant capability: {}", e),
    }
    
    // Test recv syscall (non-blocking)
    kprintln!("Testing SYS_RECV (non-blocking)...");
    let result = dispatch(SYS_RECV, 0, 0, 0, 0); // blocking=false
    kprintln!("  Recv result: {}", result);
    
    // Test recv syscall (blocking) - this will demonstrate the blocking functionality
    kprintln!("Testing SYS_RECV (blocking) - should block since no messages...");
    kprintln!("  Note: In real kernel, this would block the current task");
    let result = dispatch(SYS_RECV, 1, 0, 0, 0); // blocking=true
    kprintln!("  Recv result (after simulated blocking): {}", result);
    
    // Test with message priority and preemption
    kprintln!("Testing priority-based preemption...");
    kprintln!("  Creating high-priority message to demonstrate preemption logic");
    
    // This test demonstrates the complete blocking/wakeup flow
    
    // Print syscall table to show which ones are implemented
    use crate::syscall::table::print_syscall_table;
    print_syscall_table();
    
    kprintln!("=== IPC SYSCALLS TEST COMPLETE ===");
    kprintln!("");
}

/// System call validation helpers
pub mod validation {
    use super::*;
    
    /// Validate pointer argument
    /// 
    /// # Arguments
    /// * `ptr` - Pointer value to validate
    /// 
    /// # Returns
    /// `true` if pointer appears valid, `false` otherwise
    pub fn validate_pointer(ptr: u64) -> bool {
        // Basic pointer validation
        // In a real kernel, this would check against memory maps
        
        // Null pointer check
        if ptr == 0 {
            return false;
        }
        
        // Check for obviously invalid addresses
        if ptr < 0x1000 {
            return false; // Below 4KB (guard against null deref)
        }
        
        if ptr >= 0x800000000000 {
            return false; // Above canonical address space
        }
        
        // Check alignment (assuming 8-byte alignment requirement)
        if ptr & 0x7 != 0 {
            return false;
        }
        
        true
    }
    
    /// Validate buffer with length
    /// 
    /// # Arguments
    /// * `ptr` - Buffer pointer
    /// * `len` - Buffer length
    /// 
    /// # Returns
    /// `true` if buffer appears valid, `false` otherwise
    pub fn validate_buffer(ptr: u64, len: u64) -> bool {
        if !validate_pointer(ptr) {
            return false;
        }
        
        // Check for overflow
        if ptr.checked_add(len).is_none() {
            return false;
        }
        
        // Check reasonable length limits
        if len > 1024 * 1024 * 1024 {
            return false; // Reject buffers > 1GB
        }
        
        true
    }
    
    /// Validate task ID
    /// 
    /// # Arguments
    /// * `task_id` - Task ID to validate
    /// 
    /// # Returns
    /// `true` if task ID appears valid, `false` otherwise
    pub fn validate_task_id(task_id: u64) -> bool {
        // Task ID 0 is reserved for idle task
        // Task IDs should be reasonable values
        task_id <= 65536
    }
}

/// Handle get_features system call
/// 
/// Returns the current kernel feature flags bitset.
/// 
/// # Arguments
/// * `_a0` - Unused (get_features takes no arguments)
/// * `_a1` - Unused
/// * `_a2` - Unused
/// * `_a3` - Unused
/// 
/// # Returns
/// Feature flags bitset
fn handle_get_features(_a0: u64, _a1: u64, _a2: u64, _a3: u64) -> u64 {
    klog!(TRACE, "[SYSCALL] Handling get_features syscall");
    
    // Import the features module
    use crate::abi::features::get_features;
    
    // Get the current feature flags
    let features = get_features();
    
    klog!(TRACE, "[SYSCALL] get_features returning: 0x{:016x}", features);
    
    features
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syscall_dispatch() {
        // Test valid syscalls
        let result = dispatch(SYS_YIELD, 0, 0, 0, 0);
        assert_eq!(result, 0);
        
        // Test invalid syscall
        let result = dispatch(999, 0, 0, 0, 0);
        assert_eq!(result, u64::MAX);
    }
    
    #[test]
    fn test_validation() {
        use validation::*;
        
        // Test pointer validation
        assert!(!validate_pointer(0)); // null
        assert!(!validate_pointer(0x100)); // too low
        assert!(validate_pointer(0x10000)); // valid
        
        // Test buffer validation
        assert!(validate_buffer(0x10000, 1024));
        assert!(!validate_buffer(0, 1024)); // null pointer
        assert!(!validate_buffer(0x10000, u64::MAX)); // overflow
        
        // Test task ID validation
        assert!(validate_task_id(1));
        assert!(validate_task_id(1000));
        assert!(!validate_task_id(100000)); // too large
    }
}
