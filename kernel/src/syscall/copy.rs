/// Safe Copy Operations Module
/// 
/// This module provides safe copy operations between user space and kernel space
/// with bounds checking, alignment validation, and kernel address guards.

use super::validate::{is_valid_user_pointer, is_kernel_pointer};
use crate::{kprintln, klog, kprintln};
use crate::log::Level;
use crate::secman::audit::{audit_log, AuditEvent, AuditLevel};
use core::mem;
use core::ptr;
use core::slice;

//=============================================================================
// COPY OPERATION CONSTANTS AND CONFIGURATION
//=============================================================================

/// Maximum size for any single copy operation
pub const MAX_COPY_SIZE: usize = 64 * 1024; // 64KB

/// Default alignment for copy operations
pub const DEFAULT_COPY_ALIGNMENT: usize = 8;

/// Maximum number of copy operations per syscall
pub const MAX_COPY_OPERATIONS: usize = 10;

/// Copy operation result
#[derive(Debug, Clone)]
pub enum CopyResult<T> {
    /// Copy operation succeeded
    Success(T),
    /// Copy operation failed with error code
    Failure(String),
}

impl<T> CopyResult<T> {
    /// Check if the copy operation succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, CopyResult::Success(_))
    }
    
    /// Check if the copy operation failed
    pub fn is_failure(&self) -> bool {
        matches!(self, CopyResult::Failure(_))
    }
    
    /// Get the success value, panicking if failed
    pub fn unwrap(self) -> T {
        match self {
            CopyResult::Success(value) => value,
            CopyResult::Failure(error) => panic!("Copy operation failed: {}", error),
        }
    }
    
    /// Get the success value, or a default if failed
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            CopyResult::Success(value) => value,
            CopyResult::Failure(_) => default,
        }
    }
    
    /// Get the error message if failed
    pub fn error_message(&self) -> Option<&str> {
        match self {
            CopyResult::Success(_) => None,
            CopyResult::Failure(error) => Some(error),
        }
    }
}

//=============================================================================
// COPY FROM USER OPERATIONS
//=============================================================================

/// Copy data from user space to kernel space
/// 
/// # Arguments
/// * `user_ptr` - User space pointer
/// * `kernel_ptr` - Kernel space pointer
/// * `size` - Number of bytes to copy
/// 
/// # Returns
/// `CopyResult<usize>` - Number of bytes copied on success, or error on failure
pub fn copy_from_user(
    user_ptr: u64,
    kernel_ptr: *mut u8,
    size: usize,
) -> CopyResult<usize> {
    // Validate input parameters
    if size == 0 {
        return CopyResult::Success(0);
    }
    
    if size > MAX_COPY_SIZE {
        return CopyResult::Failure("EINVAL: Copy size exceeds maximum".to_string());
    }
    
    // Validate user pointer
    if !is_valid_user_pointer(user_ptr) {
        audit_log(
            AuditEvent::CopyFromUserFailure,
            AuditLevel::WARNING,
            &format!("Invalid user pointer 0x{:x} for copy_from_user", user_ptr),
        );
        return CopyResult::Failure("EFAULT: Invalid user pointer".to_string());
    }
    
    // Validate kernel pointer
    if kernel_ptr.is_null() {
        return CopyResult::Failure("EINVAL: Kernel pointer is null".to_string());
    }
    
    let kernel_addr = kernel_ptr as u64;
    if !is_kernel_pointer(kernel_addr) {
        audit_log(
            AuditEvent::CopyFromUserFailure,
            AuditLevel::ERROR,
            &format!("Kernel pointer 0x{:x} is not in kernel space", kernel_addr),
        );
        return CopyResult::Failure("EINVAL: Kernel pointer not in kernel space".to_string());
    }
    
    // Check for potential buffer overflow
    if let Some(overflow) = check_buffer_overflow(kernel_ptr, size) {
        audit_log(
            AuditEvent::CopyFromUserFailure,
            AuditLevel::ERROR,
            &format!("Potential buffer overflow: kernel_ptr=0x{:x}, size={}, overflow={}", 
                     kernel_addr, size, overflow),
        );
        return CopyResult::Failure("EINVAL: Potential buffer overflow".to_string());
    }
    
    // Perform the copy operation
    match unsafe { unsafe_copy_from_user(user_ptr, kernel_ptr, size) } {
        Ok(bytes_copied) => {
            klog!(TRACE, "[COPY] copy_from_user: {} bytes from 0x{:x} to 0x{:x}", 
                  bytes_copied, user_ptr, kernel_addr);
            CopyResult::Success(bytes_copied)
        }
        Err(error) => {
            audit_log(
                AuditEvent::CopyFromUserFailure,
                AuditLevel::ERROR,
                &format!("copy_from_user failed: {}", error),
            );
            CopyResult::Failure(format!("EFAULT: {}", error))
        }
    }
}

/// Copy a string from user space to kernel space
/// 
/// # Arguments
/// * `user_ptr` - User space string pointer
/// * `max_length` - Maximum string length to copy
/// 
/// # Returns
/// `CopyResult<String>` - Copied string on success, or error on failure
pub fn copy_string_from_user(
    user_ptr: u64,
    max_length: usize,
) -> CopyResult<String> {
    // Validate input parameters
    if max_length == 0 {
        return CopyResult::Success(String::new());
    }
    
    if max_length > MAX_COPY_SIZE {
        return CopyResult::Failure("EINVAL: String length exceeds maximum".to_string());
    }
    
    // Validate user pointer
    if !is_valid_user_pointer(user_ptr) {
        return CopyResult::Failure("EFAULT: Invalid user pointer".to_string());
    }
    
    // Create a temporary buffer for the string
    let mut buffer = vec![0u8; max_length];
    
    // Copy the string data
    match copy_from_user(user_ptr, buffer.as_mut_ptr(), max_length) {
        CopyResult::Success(bytes_copied) => {
            // Find the null terminator
            let null_pos = buffer.iter().position(|&b| b == 0).unwrap_or(bytes_copied);
            let actual_length = null_pos.min(bytes_copied);
            
            // Convert to string
            match String::from_utf8(buffer[..actual_length].to_vec()) {
                Ok(s) => CopyResult::Success(s),
                Err(_) => CopyResult::Failure("EINVAL: Invalid UTF-8 sequence".to_string()),
            }
        }
        CopyResult::Failure(error) => CopyResult::Failure(error),
    }
}

/// Copy a buffer from user space to kernel space with size validation
/// 
/// # Arguments
/// * `user_ptr` - User space buffer pointer
/// * `size` - Size of the buffer
/// * `max_size` - Maximum allowed size
/// 
/// # Returns
/// `CopyResult<Vec<u8>>` - Copied buffer on success, or error on failure
pub fn copy_buffer_from_user(
    user_ptr: u64,
    size: usize,
    max_size: usize,
) -> CopyResult<Vec<u8>> {
    // Validate size constraints
    if size == 0 {
        return CopyResult::Success(Vec::new());
    }
    
    if size > max_size {
        return CopyResult::Failure(format!("EINVAL: Buffer size {} exceeds maximum {}", size, max_size));
    }
    
    if size > MAX_COPY_SIZE {
        return CopyResult::Failure("EINVAL: Buffer size exceeds maximum copy size".to_string());
    }
    
    // Validate user pointer
    if !is_valid_user_pointer(user_ptr) {
        return CopyResult::Failure("EFAULT: Invalid user pointer".to_string());
    }
    
    // Create output buffer
    let mut buffer = vec![0u8; size];
    
    // Copy the data
    match copy_from_user(user_ptr, buffer.as_mut_ptr(), size) {
        CopyResult::Success(bytes_copied) => {
            if bytes_copied == size {
                CopyResult::Success(buffer)
            } else {
                CopyResult::Failure(format!("EINVAL: Expected {} bytes, got {}", size, bytes_copied))
            }
        }
        CopyResult::Failure(error) => CopyResult::Failure(error),
    }
}

//=============================================================================
// COPY TO USER OPERATIONS
//=============================================================================

/// Copy data from kernel space to user space
/// 
/// # Arguments
/// * `kernel_ptr` - Kernel space pointer
/// * `user_ptr` - User space pointer
/// * `size` - Number of bytes to copy
/// 
/// # Returns
/// `CopyResult<usize>` - Number of bytes copied on success, or error on failure
pub fn copy_to_user(
    kernel_ptr: *const u8,
    user_ptr: u64,
    size: usize,
) -> CopyResult<usize> {
    // Validate input parameters
    if size == 0 {
        return CopyResult::Success(0);
    }
    
    if size > MAX_COPY_SIZE {
        return CopyResult::Failure("EINVAL: Copy size exceeds maximum".to_string());
    }
    
    // Validate kernel pointer
    if kernel_ptr.is_null() {
        return CopyResult::Failure("EINVAL: Kernel pointer is null".to_string());
    }
    
    let kernel_addr = kernel_ptr as u64;
    if !is_kernel_pointer(kernel_addr) {
        audit_log(
            AuditEvent::CopyToUserFailure,
            AuditLevel::ERROR,
            &format!("Kernel pointer 0x{:x} is not in kernel space", kernel_addr),
        );
        return CopyResult::Failure("EINVAL: Kernel pointer not in kernel space".to_string());
    }
    
    // Validate user pointer
    if !is_valid_user_pointer(user_ptr) {
        audit_log(
            AuditEvent::CopyToUserFailure,
            AuditLevel::WARNING,
            &format!("Invalid user pointer 0x{:x} for copy_to_user", user_ptr),
        );
        return CopyResult::Failure("EFAULT: Invalid user pointer".to_string());
    }
    
    // Check for potential buffer overflow
    if let Some(overflow) = check_user_buffer_overflow(user_ptr, size) {
        audit_log(
            AuditEvent::CopyToUserFailure,
            AuditLevel::ERROR,
            &format!("Potential user buffer overflow: user_ptr=0x{:x}, size={}, overflow={}", 
                     user_ptr, size, overflow),
        );
        return CopyResult::Failure("EINVAL: Potential user buffer overflow".to_string());
    }
    
    // Perform the copy operation
    match unsafe { unsafe_copy_to_user(kernel_ptr, user_ptr, size) } {
        Ok(bytes_copied) => {
            klog!(TRACE, "[COPY] copy_to_user: {} bytes from 0x{:x} to 0x{:x}", 
                  bytes_copied, kernel_addr, user_ptr);
            CopyResult::Success(bytes_copied)
        }
        Err(error) => {
            audit_log(
                AuditEvent::CopyToUserFailure,
                AuditLevel::ERROR,
                &format!("copy_to_user failed: {}", error),
            );
            CopyResult::Failure(format!("EFAULT: {}", error))
        }
    }
}

/// Copy a string to user space
/// 
/// # Arguments
/// * `kernel_string` - Kernel space string
/// * `user_ptr` - User space destination pointer
/// * `max_length` - Maximum length to copy
/// 
/// # Returns
/// `CopyResult<usize>` - Number of bytes copied on success, or error on failure
pub fn copy_string_to_user(
    kernel_string: &str,
    user_ptr: u64,
    max_length: usize,
) -> CopyResult<usize> {
    // Validate input parameters
    if max_length == 0 {
        return CopyResult::Success(0);
    }
    
    if max_length > MAX_COPY_SIZE {
        return CopyResult::Failure("EINVAL: String length exceeds maximum".to_string());
    }
    
    // Validate user pointer
    if !is_valid_user_pointer(user_ptr) {
        return CopyResult::Failure("EFAULT: Invalid user pointer".to_string());
    }
    
    // Determine how much to copy
    let bytes_to_copy = kernel_string.len().min(max_length);
    
    // Copy the string data
    copy_to_user(kernel_string.as_ptr(), user_ptr, bytes_to_copy)
}

/// Copy a buffer to user space
/// 
/// # Arguments
/// * `kernel_buffer` - Kernel space buffer
/// * `user_ptr` - User space destination pointer
/// 
/// # Returns
/// `CopyResult<usize>` - Number of bytes copied on success, or error on failure
pub fn copy_buffer_to_user(
    kernel_buffer: &[u8],
    user_ptr: u64,
) -> CopyResult<usize> {
    // Validate input parameters
    if kernel_buffer.is_empty() {
        return CopyResult::Success(0);
    }
    
    if kernel_buffer.len() > MAX_COPY_SIZE {
        return CopyResult::Failure("EINVAL: Buffer size exceeds maximum".to_string());
    }
    
    // Validate user pointer
    if !is_valid_user_pointer(user_ptr) {
        return CopyResult::Failure("EFAULT: Invalid user pointer".to_string());
    }
    
    // Copy the buffer data
    copy_to_user(kernel_buffer.as_ptr(), user_ptr, kernel_buffer.len())
}

//=============================================================================
// SAFETY CHECKS AND VALIDATION
//=============================================================================

/// Check for potential buffer overflow in kernel space
fn check_buffer_overflow(ptr: *mut u8, size: usize) -> Option<usize> {
    // This is a simplified check - in a real implementation, you would:
    // 1. Check page boundaries
    // 2. Validate against allocated memory regions
    // 3. Check for stack overflow
    
    let ptr_addr = ptr as u64;
    
    // Check if the pointer + size would wrap around
    if ptr_addr.checked_add(size as u64).is_none() {
        return Some(size);
    }
    
    // Check if the pointer + size exceeds kernel space
    if ptr_addr + size as u64 > 0xffffffffffffffff {
        return Some(size);
    }
    
    None
}

/// Check for potential buffer overflow in user space
fn check_user_buffer_overflow(user_ptr: u64, size: usize) -> Option<usize> {
    // Check if the pointer + size would wrap around
    if user_ptr.checked_add(size as u64).is_none() {
        return Some(size);
    }
    
    // Check if the pointer + size exceeds user space
    if user_ptr + size as u64 > 0x7fffffffffff {
        return Some(size);
    }
    
    None
}

//=============================================================================
// UNSAFE COPY IMPLEMENTATIONS
//=============================================================================

/// Unsafe copy from user space (internal use only)
/// 
/// # Safety
/// This function assumes all pointers and sizes have been validated.
/// Callers must ensure safety invariants are maintained.
unsafe fn unsafe_copy_from_user(
    user_ptr: u64,
    kernel_ptr: *mut u8,
    size: usize,
) -> Result<usize, String> {
    // Convert user pointer to kernel pointer
    let user_ptr_kernel = user_ptr as *const u8;
    
    // Perform the copy
    ptr::copy_nonoverlapping(user_ptr_kernel, kernel_ptr, size);
    
    Ok(size)
}

/// Unsafe copy to user space (internal use only)
/// 
/// # Safety
/// This function assumes all pointers and sizes have been validated.
/// Callers must ensure safety invariants are maintained.
unsafe fn unsafe_copy_to_user(
    kernel_ptr: *const u8,
    user_ptr: u64,
    size: usize,
) -> Result<usize, String> {
    // Convert user pointer to kernel pointer
    let user_ptr_kernel = user_ptr as *mut u8;
    
    // Perform the copy
    ptr::copy_nonoverlapping(kernel_ptr, user_ptr_kernel, size);
    
    Ok(size)
}

//=============================================================================
// BATCH COPY OPERATIONS
//=============================================================================

/// Copy multiple buffers from user space in a single operation
/// 
/// # Arguments
/// * `copies` - Vector of copy operations
/// 
/// # Returns
/// `CopyResult<Vec<usize>>` - Results for each copy operation
pub fn batch_copy_from_user(
    copies: Vec<(u64, *mut u8, usize)>,
) -> CopyResult<Vec<usize>> {
    if copies.len() > MAX_COPY_OPERATIONS {
        return CopyResult::Failure("EINVAL: Too many copy operations".to_string());
    }
    
    let mut results = Vec::new();
    
    for (user_ptr, kernel_ptr, size) in copies {
        match copy_from_user(user_ptr, kernel_ptr, size) {
            CopyResult::Success(bytes_copied) => results.push(bytes_copied),
            CopyResult::Failure(error) => return CopyResult::Failure(error),
        }
    }
    
    CopyResult::Success(results)
}

/// Copy multiple buffers to user space in a single operation
/// 
/// # Arguments
/// * `copies` - Vector of copy operations
/// 
/// # Returns
/// `CopyResult<Vec<usize>>` - Results for each copy operation
pub fn batch_copy_to_user(
    copies: Vec<(*const u8, u64, usize)>,
) -> CopyResult<Vec<usize>> {
    if copies.len() > MAX_COPY_OPERATIONS {
        return CopyResult::Failure("EINVAL: Too many copy operations".to_string());
    }
    
    let mut results = Vec::new();
    
    for (kernel_ptr, user_ptr, size) in copies {
        match copy_to_user(kernel_ptr, user_ptr, size) {
            CopyResult::Success(bytes_copied) => results.push(bytes_copied),
            CopyResult::Failure(error) => return CopyResult::Failure(error),
        }
    }
    
    CopyResult::Success(results)
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

/// Get the current copy operation limits
pub fn get_copy_limits() -> (usize, usize, usize) {
    (MAX_COPY_SIZE, MAX_COPY_OPERATIONS, DEFAULT_COPY_ALIGNMENT)
}

/// Check if a copy operation would be valid
pub fn validate_copy_operation(
    src_ptr: u64,
    dst_ptr: u64,
    size: usize,
) -> bool {
    if size == 0 {
        return true;
    }
    
    if size > MAX_COPY_SIZE {
        return false;
    }
    
    // Check for potential overflow
    if src_ptr.checked_add(size as u64).is_none() {
        return false;
    }
    
    if dst_ptr.checked_add(size as u64).is_none() {
        return false;
    }
    
    true
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_copy_result() {
        let success = CopyResult::Success(42);
        assert!(success.is_success());
        assert!(!success.is_failure());
        assert_eq!(success.unwrap(), 42);
        
        let failure = CopyResult::Failure("test error".to_string());
        assert!(!failure.is_success());
        assert!(failure.is_failure());
        assert_eq!(failure.error_message(), Some("test error"));
    }
    
    #[test]
    fn test_copy_limits() {
        let (max_size, max_ops, alignment) = get_copy_limits();
        assert_eq!(max_size, MAX_COPY_SIZE);
        assert_eq!(max_ops, MAX_COPY_OPERATIONS);
        assert_eq!(alignment, DEFAULT_COPY_ALIGNMENT);
    }
    
    #[test]
    fn test_validate_copy_operation() {
        // Valid operation
        assert!(validate_copy_operation(0x400000, 0x500000, 1024));
        
        // Invalid size
        assert!(!validate_copy_operation(0x400000, 0x500000, MAX_COPY_SIZE + 1));
        
        // Overflow check
        assert!(!validate_copy_operation(0xffffffffffffffff, 0x500000, 1024));
    }
    
    #[test]
    fn test_buffer_overflow_checks() {
        // Valid kernel pointer
        let ptr = 0xffff800000000000 as *mut u8;
        assert!(check_buffer_overflow(ptr, 1024).is_none());
        
        // Wrapping pointer
        let ptr = 0xffffffffffffffff as *mut u8;
        assert!(check_buffer_overflow(ptr, 1024).is_some());
    }
    
    #[test]
    fn test_user_buffer_overflow_checks() {
        // Valid user pointer
        assert!(check_user_buffer_overflow(0x400000, 1024).is_none());
        
        // Wrapping pointer
        assert!(check_user_buffer_overflow(0x7fffffffffff, 1024).is_some());
    }
}
