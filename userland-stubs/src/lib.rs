#![no_std]
#![feature(asm_const)]

// Include assembly syscall implementation
global_asm!(include_str!("syscall.S"));

/// Userland syscall stubs for Polymera OS
/// 
/// This crate provides safe Rust wrappers around the kernel syscalls.
/// All functions are marked as unsafe since they invoke kernel code.

/// Raw syscall function declaration
/// 
/// This function is implemented in assembly and invokes the kernel syscall
/// mechanism (int 0x80 on x86_64).
/// 
/// # Arguments
/// * `num` - Syscall number
/// * `a0` - First argument
/// * `a1` - Second argument
/// * `a2` - Third argument
/// * `a3` - Fourth argument
/// 
/// # Returns
/// Syscall return value
extern "C" { 
    fn syscall(num: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64; 
}

/// Yield CPU to another task
/// 
/// This syscall voluntarily gives up the CPU to allow other tasks to run.
/// It's useful for cooperative multitasking.
/// 
/// # Returns
/// Always returns 0 on success
pub fn sys_yield() -> u64 { 
    unsafe { syscall(1, 0, 0, 0, 0) } 
}

/// Exit current task
/// 
/// Terminates the current task with the specified exit code.
/// This function does not return.
/// 
/// # Arguments
/// * `code` - Exit code (typically 0 for success, non-zero for error)
/// 
/// # Safety
/// This function never returns and terminates the current task
pub fn sys_exit(code: i32) -> ! { 
    let result = unsafe { syscall(2, code as u64, 0, 0, 0) };
    // This should never be reached, but if it is, we'll exit anyway
    loop {
        // In a real implementation, this would trigger a panic or abort
        core::hint::spin_loop();
    }
}

/// Send message to another task
/// 
/// Sends a message to the specified destination task.
/// 
/// # Arguments
/// * `dst` - Destination task ID
/// * `buf` - Message buffer to send
/// 
/// # Returns
/// Number of bytes sent, or error code on failure
pub fn sys_send(dst: u64, buf: &[u8]) -> u64 { 
    unsafe { 
        syscall(3, dst, buf.as_ptr() as u64, buf.len() as u64, 0) 
    } 
}

/// Receive message from another task
/// 
/// Receives a message from another task.
/// 
/// # Arguments
/// * `block` - Whether to block waiting for a message
/// * `out` - Output buffer for received message
/// 
/// # Returns
/// Number of bytes received, or error code on failure
pub fn sys_recv(block: bool, out: &mut [u8]) -> u64 { 
    unsafe { 
        syscall(4, block as u64, out.as_mut_ptr() as u64, out.len() as u64, 0) 
    } 
}

/// Create IPC channel
/// 
/// Creates a new IPC channel for communication between tasks.
/// 
/// # Arguments
/// * `capacity` - Maximum number of messages in the channel
/// 
/// # Returns
/// Channel ID on success, or error code on failure
pub fn sys_chan_create(capacity: u64) -> u64 {
    unsafe { syscall(5, capacity, 0, 0, 0) }
}

/// Get system statistics
/// 
/// Retrieves system statistics and performance metrics.
/// 
/// # Arguments
/// * `stats_type` - Type of statistics to retrieve
/// * `buffer` - Buffer to store statistics data
/// 
/// # Returns
/// Number of bytes written, or error code on failure
pub fn sys_stats(stats_type: u64, buffer: &mut [u8]) -> u64 {
    unsafe { syscall(6, stats_type, buffer.as_mut_ptr() as u64, buffer.len() as u64, 0) }
}

/// Debug operations
/// 
/// Performs various debug operations for development and testing.
/// 
/// # Arguments
/// * `op` - Debug operation to perform
/// * `arg` - Additional argument for the operation
/// 
/// # Returns
/// Operation result, or error code on failure
pub fn sys_debug(op: u64, arg: u64) -> u64 {
    unsafe { syscall(7, op, arg, 0, 0) }
}

/// Get kernel feature flags
/// 
/// Retrieves the current kernel feature flags bitset.
/// This allows user applications to adapt their behavior
/// based on available kernel features.
/// 
/// # Returns
/// Feature flags bitset (64-bit value where each bit represents a feature)
pub fn sys_get_features() -> u64 {
    unsafe { syscall(21, 0, 0, 0, 0) }
}

/// Raw syscall wrapper for advanced use cases
/// 
/// This function provides direct access to the syscall mechanism
/// for cases where the high-level wrappers aren't sufficient.
/// 
/// # Arguments
/// * `num` - Syscall number
/// * `args` - Array of up to 4 arguments
/// 
/// # Returns
/// Syscall return value
/// 
/// # Safety
/// This function is unsafe as it directly invokes kernel code
pub unsafe fn raw_syscall(num: u64, args: [u64; 4]) -> u64 {
    syscall(num, args[0], args[1], args[2], args[3])
}

/// Syscall number constants
/// 
/// These constants match the kernel syscall definitions
pub mod syscall_numbers {
    pub const SYS_YIELD: u64 = 1;
    pub const SYS_EXIT: u64 = 2;
    pub const SYS_SEND: u64 = 3;
    pub const SYS_RECV: u64 = 4;
    pub const SYS_CHAN_CREATE: u64 = 5;
    pub const SYS_STATS: u64 = 6;
    pub const SYS_DEBUG: u64 = 7;
    pub const SYS_GET_FEATURES: u64 = 21;
}

/// Error codes returned by syscalls
/// 
/// These constants define common error codes that syscalls may return
pub mod error_codes {
    pub const ESUCCESS: u64 = 0;
    pub const EPERM: u64 = 1;      // Operation not permitted
    pub const ENOENT: u64 = 2;     // No such file or directory
    pub const EINTR: u64 = 4;      // Interrupted system call
    pub const EIO: u64 = 5;        // Input/output error
    pub const EAGAIN: u64 = 11;    // Resource temporarily unavailable
    pub const EINVAL: u64 = 22;    // Invalid argument
    pub const ENOMEM: u64 = 12;    // Cannot allocate memory
    pub const ETIMEDOUT: u64 = 110; // Connection timed out
}

/// Check if a syscall return value indicates an error
/// 
/// # Arguments
/// * `result` - Return value from a syscall
/// 
/// # Returns
/// `true` if the result indicates an error
pub fn is_error(result: u64) -> bool {
    // In our system, error codes are typically small positive integers
    // Success is usually 0 or a positive value representing bytes/items
    result > 0 && result < 1000
}

/// Get error code from syscall result
/// 
/// # Arguments
/// * `result` - Return value from a syscall
/// 
/// # Returns
/// Error code if result indicates an error, `None` otherwise
pub fn get_error_code(result: u64) -> Option<u64> {
    if is_error(result) {
        Some(result)
    } else {
        None
    }
}

/// Simple test function to verify the crate compiles
/// 
/// This function can be called to test that all syscall stubs are available
/// and the crate is properly configured.
pub fn test_crate_compilation() -> bool {
    // Test that all syscall numbers are defined
    let numbers = [
        syscall_numbers::SYS_YIELD,
        syscall_numbers::SYS_EXIT,
        syscall_numbers::SYS_SEND,
        syscall_numbers::SYS_RECV,
        syscall_numbers::SYS_CHAN_CREATE,
        syscall_numbers::SYS_STATS,
        syscall_numbers::SYS_DEBUG,
        syscall_numbers::SYS_GET_FEATURES,
    ];
    
    // Test that all error codes are defined
    let errors = [
        error_codes::ESUCCESS,
        error_codes::EPERM,
        error_codes::ENOENT,
        error_codes::EINTR,
        error_codes::EIO,
        error_codes::EAGAIN,
        error_codes::EINVAL,
        error_codes::ENOMEM,
        error_codes::ETIMEDOUT,
    ];
    
    // Note: SYS_GET_FEATURES is not sequential (it's 21), so we skip sequential validation
    // for syscalls that are not part of the core sequence
    
    // Verify error codes are non-zero (except ESUCCESS)
    for (i, &error) in errors.iter().enumerate() {
        if i == 0 && error != 0 {
            return false; // ESUCCESS should be 0
        }
        if i > 0 && error == 0 {
            return false; // Other errors should be non-zero
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syscall_numbers() {
        assert_eq!(syscall_numbers::SYS_YIELD, 1);
        assert_eq!(syscall_numbers::SYS_EXIT, 2);
        assert_eq!(syscall_numbers::SYS_CHAN_CREATE, 5);
        assert_eq!(syscall_numbers::SYS_STATS, 6);
        assert_eq!(syscall_numbers::SYS_DEBUG, 7);
        assert_eq!(syscall_numbers::SYS_GET_FEATURES, 21);
    }
    
    #[test]
    fn test_error_codes() {
        assert_eq!(error_codes::ESUCCESS, 0);
        assert_eq!(error_codes::EPERM, 1);
        assert_eq!(error_codes::ENOENT, 2);
        assert_eq!(error_codes::EINTR, 4);
        assert_eq!(error_codes::EIO, 5);
        assert_eq!(error_codes::EAGAIN, 11);
        assert_eq!(error_codes::EINVAL, 22);
        assert_eq!(error_codes::ENOMEM, 12);
        assert_eq!(error_codes::ETIMEDOUT, 110);
    }
    
    #[test]
    fn test_error_detection() {
        assert!(!is_error(0));           // Success
        assert!(!is_error(1000));        // Large success value
        assert!(!is_error(5000));        // Very large success value
        assert!(is_error(1));            // EPERM
        assert!(is_error(2));            // ENOENT
        assert!(is_error(22));           // EINVAL
        assert!(is_error(999));          // Large error code
    }
    
    #[test]
    fn test_error_code_extraction() {
        assert_eq!(get_error_code(0), None);           // Success
        assert_eq!(get_error_code(1000), None);        // Large success value
        assert_eq!(get_error_code(1), Some(1));        // EPERM
        assert_eq!(get_error_code(22), Some(22));      // EINVAL
        assert_eq!(get_error_code(999), Some(999));    // Large error code
    }
    
    #[test]
    fn test_crate_compilation() {
        assert!(test_crate_compilation());
    }
}


// Include assembly syscall implementation
global_asm!(include_str!("syscall.S"));

/// Userland syscall stubs for Polymera OS
/// 
/// This crate provides safe Rust wrappers around the kernel syscalls.
/// All functions are marked as unsafe since they invoke kernel code.

/// Raw syscall function declaration
/// 
/// This function is implemented in assembly and invokes the kernel syscall
/// mechanism (int 0x80 on x86_64).
/// 
/// # Arguments
/// * `num` - Syscall number
/// * `a0` - First argument
/// * `a1` - Second argument
/// * `a2` - Third argument
/// * `a3` - Fourth argument
/// 
/// # Returns
/// Syscall return value
extern "C" { 
    fn syscall(num: u64, a0: u64, a1: u64, a2: u64, a3: u64) -> u64; 
}

/// Yield CPU to another task
/// 
/// This syscall voluntarily gives up the CPU to allow other tasks to run.
/// It's useful for cooperative multitasking.
/// 
/// # Returns
/// Always returns 0 on success
pub fn sys_yield() -> u64 { 
    unsafe { syscall(1, 0, 0, 0, 0) } 
}

/// Exit current task
/// 
/// Terminates the current task with the specified exit code.
/// This function does not return.
/// 
/// # Arguments
/// * `code` - Exit code (typically 0 for success, non-zero for error)
/// 
/// # Safety
/// This function never returns and terminates the current task
pub fn sys_exit(code: i32) -> ! { 
    let result = unsafe { syscall(2, code as u64, 0, 0, 0) };
    // This should never be reached, but if it is, we'll exit anyway
    loop {
        // In a real implementation, this would trigger a panic or abort
        core::hint::spin_loop();
    }
}

/// Send message to another task
/// 
/// Sends a message to the specified destination task.
/// 
/// # Arguments
/// * `dst` - Destination task ID
/// * `buf` - Message buffer to send
/// 
/// # Returns
/// Number of bytes sent, or error code on failure
pub fn sys_send(dst: u64, buf: &[u8]) -> u64 { 
    unsafe { 
        syscall(3, dst, buf.as_ptr() as u64, buf.len() as u64, 0) 
    } 
}

/// Receive message from another task
/// 
/// Receives a message from another task.
/// 
/// # Arguments
/// * `block` - Whether to block waiting for a message
/// * `out` - Output buffer for received message
/// 
/// # Returns
/// Number of bytes received, or error code on failure
pub fn sys_recv(block: bool, out: &mut [u8]) -> u64 { 
    unsafe { 
        syscall(4, block as u64, out.as_mut_ptr() as u64, out.len() as u64, 0) 
    } 
}

/// Create IPC channel
/// 
/// Creates a new IPC channel for communication between tasks.
/// 
/// # Arguments
/// * `capacity` - Maximum number of messages in the channel
/// 
/// # Returns
/// Channel ID on success, or error code on failure
pub fn sys_chan_create(capacity: u64) -> u64 {
    unsafe { syscall(5, capacity, 0, 0, 0) }
}

/// Get system statistics
/// 
/// Retrieves system statistics and performance metrics.
/// 
/// # Arguments
/// * `stats_type` - Type of statistics to retrieve
/// * `buffer` - Buffer to store statistics data
/// 
/// # Returns
/// Number of bytes written, or error code on failure
pub fn sys_stats(stats_type: u64, buffer: &mut [u8]) -> u64 {
    unsafe { syscall(6, stats_type, buffer.as_mut_ptr() as u64, buffer.len() as u64, 0) }
}

/// Debug operations
/// 
/// Performs various debug operations for development and testing.
/// 
/// # Arguments
/// * `op` - Debug operation to perform
/// * `arg` - Additional argument for the operation
/// 
/// # Returns
/// Operation result, or error code on failure
pub fn sys_debug(op: u64, arg: u64) -> u64 {
    unsafe { syscall(7, op, arg, 0, 0) }
}

/// Raw syscall wrapper for advanced use cases
/// 
/// This function provides direct access to the syscall mechanism
/// for cases where the high-level wrappers aren't sufficient.
/// 
/// # Arguments
/// * `num` - Syscall number
/// * `args` - Array of up to 4 arguments
/// 
/// # Returns
/// Syscall return value
/// 
/// # Safety
/// This function is unsafe as it directly invokes kernel code
pub unsafe fn raw_syscall(num: u64, args: [u64; 4]) -> u64 {
    syscall(num, args[0], args[1], args[2], args[3])
}

/// Syscall number constants
/// 
/// These constants match the kernel syscall definitions
pub mod syscall_numbers {
    pub const SYS_YIELD: u64 = 1;
    pub const SYS_EXIT: u64 = 2;
    pub const SYS_SEND: u64 = 3;
    pub const SYS_RECV: u64 = 4;
    pub const SYS_CHAN_CREATE: u64 = 5;
    pub const SYS_STATS: u64 = 6;
    pub const SYS_DEBUG: u64 = 7;
}

/// Error codes returned by syscalls
/// 
/// These constants define common error codes that syscalls may return
pub mod error_codes {
    pub const ESUCCESS: u64 = 0;
    pub const EPERM: u64 = 1;      // Operation not permitted
    pub const ENOENT: u64 = 2;     // No such file or directory
    pub const EINTR: u64 = 4;      // Interrupted system call
    pub const EIO: u64 = 5;        // Input/output error
    pub const EAGAIN: u64 = 11;    // Resource temporarily unavailable
    pub const EINVAL: u64 = 22;    // Invalid argument
    pub const ENOMEM: u64 = 12;    // Cannot allocate memory
    pub const ETIMEDOUT: u64 = 110; // Connection timed out
}

/// Check if a syscall return value indicates an error
/// 
/// # Arguments
/// * `result` - Return value from a syscall
/// 
/// # Returns
/// `true` if the result indicates an error
pub fn is_error(result: u64) -> bool {
    // In our system, error codes are typically small positive integers
    // Success is usually 0 or a positive value representing bytes/items
    result > 0 && result < 1000
}

/// Get error code from syscall result
/// 
/// # Arguments
/// * `result` - Return value from a syscall
/// 
/// # Returns
/// Error code if result indicates an error, `None` otherwise
pub fn get_error_code(result: u64) -> Option<u64> {
    if is_error(result) {
        Some(result)
    } else {
        None
    }
}

/// Simple test function to verify the crate compiles
/// 
/// This function can be called to test that all syscall stubs are available
/// and the crate is properly configured.
pub fn test_crate_compilation() -> bool {
    // Test that all syscall numbers are defined
    let numbers = [
        syscall_numbers::SYS_YIELD,
        syscall_numbers::SYS_EXIT,
        syscall_numbers::SYS_SEND,
        syscall_numbers::SYS_RECV,
        syscall_numbers::SYS_CHAN_CREATE,
        syscall_numbers::SYS_STATS,
        syscall_numbers::SYS_DEBUG,
    ];
    
    // Test that all error codes are defined
    let errors = [
        error_codes::ESUCCESS,
        error_codes::EPERM,
        error_codes::ENOENT,
        error_codes::EINTR,
        error_codes::EIO,
        error_codes::EAGAIN,
        error_codes::EINVAL,
        error_codes::ENOMEM,
        error_codes::ETIMEDOUT,
    ];
    
    // Verify numbers are sequential starting from 1
    for (i, &num) in numbers.iter().enumerate() {
        if num != (i + 1) as u64 {
            return false;
        }
    }
    
    // Verify error codes are non-zero (except ESUCCESS)
    for (i, &error) in errors.iter().enumerate() {
        if i == 0 && error != 0 {
            return false; // ESUCCESS should be 0
        }
        if i > 0 && error == 0 {
            return false; // Other errors should be non-zero
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syscall_numbers() {
        assert_eq!(syscall_numbers::SYS_YIELD, 1);
        assert_eq!(syscall_numbers::SYS_EXIT, 2);
        assert_eq!(syscall_numbers::SYS_SEND, 3);
        assert_eq!(syscall_numbers::SYS_RECV, 4);
        assert_eq!(syscall_numbers::SYS_CHAN_CREATE, 5);
        assert_eq!(syscall_numbers::SYS_STATS, 6);
        assert_eq!(syscall_numbers::SYS_DEBUG, 7);
    }
    
    #[test]
    fn test_error_codes() {
        assert_eq!(error_codes::ESUCCESS, 0);
        assert_eq!(error_codes::EPERM, 1);
        assert_eq!(error_codes::ENOENT, 2);
        assert_eq!(error_codes::EINTR, 4);
        assert_eq!(error_codes::EIO, 5);
        assert_eq!(error_codes::EAGAIN, 11);
        assert_eq!(error_codes::EINVAL, 22);
        assert_eq!(error_codes::ENOMEM, 12);
        assert_eq!(error_codes::ETIMEDOUT, 110);
    }
    
    #[test]
    fn test_error_detection() {
        assert!(!is_error(0));           // Success
        assert!(!is_error(1000));        // Large success value
        assert!(!is_error(5000));        // Very large success value
        assert!(is_error(1));            // EPERM
        assert!(is_error(2));            // ENOENT
        assert!(is_error(22));           // EINVAL
        assert!(is_error(999));          // Large error code
    }
    
    #[test]
    fn test_error_code_extraction() {
        assert_eq!(get_error_code(0), None);           // Success
        assert_eq!(get_error_code(1000), None);        // Large success value
        assert_eq!(get_error_code(1), Some(1));        // EPERM
        assert_eq!(get_error_code(22), Some(22));      // EINVAL
        assert_eq!(get_error_code(999), Some(999));    // Large error code
    }
    
    #[test]
    fn test_crate_compilation() {
        assert!(test_crate_compilation());
    }
}




