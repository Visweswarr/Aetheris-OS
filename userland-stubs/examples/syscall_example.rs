#![no_std]
#![no_main]

use userland_stubs::*;

/// Simple example demonstrating userland syscall usage
/// 
/// This example shows how to use the syscall stubs to interact with the kernel.
/// In a real system, this would be a userland program running under the kernel.

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // This is the entry point for the userland program
    
    // Example 1: Yield CPU to demonstrate cooperative multitasking
    kprintln("[USERLAND] Yielding CPU...");
    let result = sys_yield();
    kprintln("[USERLAND] sys_yield returned");
    
    // Example 2: Create an IPC channel
    kprintln("[USERLAND] Creating IPC channel...");
    let channel_id = sys_chan_create(10); // 10 message capacity
    kprintln("[USERLAND] Channel created");
    
    // Example 3: Send a message through the channel
    if channel_id > 0 && channel_id < 1000 { // Valid channel ID
        let message = b"Hello from userland!";
        kprintln("[USERLAND] Sending message");
        let bytes_sent = sys_send(channel_id, message);
        kprintln("[USERLAND] Message sent");
        
        // Example 4: Receive a message (non-blocking)
        let mut receive_buffer = [0u8; 64];
        kprintln("[USERLAND] Receiving message...");
        let bytes_received = sys_recv(false, &mut receive_buffer);
        kprintln("[USERLAND] Message received");
        
        if bytes_received > 0 && bytes_received < 1000 {
            kprintln("[USERLAND] Message content received");
        }
    }
    
    // Example 5: Get system statistics
    let mut stats_buffer = [0u8; 128];
    kprintln("[USERLAND] Getting system statistics...");
    let stats_result = sys_stats(0, &mut stats_buffer); // 0 = general stats
    kprintln("[USERLAND] Stats retrieved");
    
    // Example 6: Debug operation
    kprintln("[USERLAND] Performing debug operation...");
    let debug_result = sys_debug(1, 42); // op=1, arg=42
    kprintln("[USERLAND] Debug operation completed");
    
    // Example 7: Demonstrate error handling
    kprintln("[USERLAND] Testing error handling...");
    let invalid_result = sys_send(99999, b"test"); // Invalid destination
    if is_error(invalid_result) {
        if let Some(error_code) = get_error_code(invalid_result) {
            kprintln("[USERLAND] Got expected error");
        }
    }
    
    // Example 8: Raw syscall usage
    kprintln("[USERLAND] Testing raw syscall...");
    let raw_result = unsafe { 
        raw_syscall(syscall_numbers::SYS_YIELD, [0, 0, 0, 0]) 
    };
    kprintln("[USERLAND] Raw syscall completed");
    
    // Example 9: Multiple yield operations to demonstrate scheduling
    kprintln("[USERLAND] Performing multiple yields...");
    for i in 0..5 {
        kprintln("[USERLAND] Yield iteration");
        let _ = sys_yield();
    }
    
    // Example 10: Exit the program
    kprintln("[USERLAND] Example completed, exiting...");
    sys_exit(0);
}

/// Simple print function for userland programs
/// 
/// In a real system, this would use proper I/O mechanisms
fn kprintln(fmt: &str) {
    // This is a simplified version - in reality, this would use
    // proper kernel I/O syscalls or be provided by a runtime library
    
    // For demonstration purposes, we'll use a debug syscall
    let message = fmt.as_bytes();
    let _ = sys_debug(100, message.len() as u64); // op=100 for print
    
    // In a real implementation, you might have:
    // - A write syscall for stdout
    // - A logging syscall for debug output
    // - A message passing mechanism to a logging service
}

/// Panic handler for userland programs
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Log the panic information
    let message = format!("[USERLAND PANIC] {}", info);
    let _ = sys_debug(200, message.len() as u64); // op=200 for panic
    
    // Exit with error code
    sys_exit(1);
}

/// Global allocator for userland programs
/// 
/// This provides basic memory allocation capabilities
#[global_allocator]
static ALLOCATOR: userland_stubs::alloc::UserlandAllocator = 
    userland_stubs::alloc::UserlandAllocator::new();

/// Module for userland memory allocation
/// 
/// This provides basic memory management for userland programs
pub mod alloc {
    use core::alloc::{GlobalAlloc, Layout};
    
    /// Simple userland allocator
    /// 
    /// This is a basic implementation that would be replaced
    /// by a proper memory management system in production
    pub struct UserlandAllocator;
    
    impl UserlandAllocator {
        pub const fn new() -> Self {
            Self
        }
    }
    
    unsafe impl GlobalAlloc for UserlandAllocator {
        unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
            // In a real system, this would use kernel memory allocation syscalls
            // For now, return null to indicate allocation failure
            core::ptr::null_mut()
        }
        
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
            // In a real system, this would free memory via kernel syscalls
        }
    }
}

/// Module for userland I/O operations
/// 
/// This provides basic input/output capabilities for userland programs
pub mod io {
    use userland_stubs::*;
    
    /// Write bytes to stdout
    /// 
    /// # Arguments
    /// * `bytes` - Bytes to write
    /// 
    /// # Returns
    /// Number of bytes written
    pub fn write_stdout(bytes: &[u8]) -> u64 {
        // In a real system, this would use a write syscall
        // For now, we'll use the debug syscall as a placeholder
        sys_debug(300, bytes.len() as u64) // op=300 for stdout write
    }
    
    /// Read bytes from stdin
    /// 
    /// # Arguments
    /// * `buffer` - Buffer to read into
    /// 
    /// # Returns
    /// Number of bytes read
    pub fn read_stdin(buffer: &mut [u8]) -> u64 {
        // In a real system, this would use a read syscall
        // For now, we'll use the debug syscall as a placeholder
        sys_debug(301, buffer.len() as u64) // op=301 for stdin read
    }
}

/// Module for userland process management
/// 
/// This provides basic process control capabilities
pub mod process {
    use userland_stubs::*;
    
    /// Get current task ID
    /// 
    /// # Returns
    /// Current task ID
    pub fn get_current_task_id() -> u64 {
        // In a real system, this would use a gettid syscall
        // For now, we'll use the debug syscall as a placeholder
        sys_debug(400, 0) // op=400 for get current task ID
    }
    
    /// Sleep for specified milliseconds
    /// 
    /// # Arguments
    /// * `milliseconds` - Time to sleep in milliseconds
    pub fn sleep(milliseconds: u64) {
        // In a real system, this would use a sleep syscall
        // For now, we'll use the debug syscall as a placeholder
        let _ = sys_debug(401, milliseconds); // op=401 for sleep
    }
}


use userland_stubs::*;

/// Simple example demonstrating userland syscall usage
/// 
/// This example shows how to use the syscall stubs to interact with the kernel.
/// In a real system, this would be a userland program running under the kernel.

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // This is the entry point for the userland program
    
    // Example 1: Yield CPU to demonstrate cooperative multitasking
    kprintln("[USERLAND] Yielding CPU...");
    let result = sys_yield();
    kprintln("[USERLAND] sys_yield returned");
    
    // Example 2: Create an IPC channel
    kprintln("[USERLAND] Creating IPC channel...");
    let channel_id = sys_chan_create(10); // 10 message capacity
    kprintln("[USERLAND] Channel created");
    
    // Example 3: Send a message through the channel
    if channel_id > 0 && channel_id < 1000 { // Valid channel ID
        let message = b"Hello from userland!";
        kprintln("[USERLAND] Sending message");
        let bytes_sent = sys_send(channel_id, message);
        kprintln("[USERLAND] Message sent");
        
        // Example 4: Receive a message (non-blocking)
        let mut receive_buffer = [0u8; 64];
        kprintln("[USERLAND] Receiving message...");
        let bytes_received = sys_recv(false, &mut receive_buffer);
        kprintln("[USERLAND] Message received");
        
        if bytes_received > 0 && bytes_received < 1000 {
            kprintln("[USERLAND] Message content received");
        }
    }
    
    // Example 5: Get system statistics
    let mut stats_buffer = [0u8; 128];
    kprintln("[USERLAND] Getting system statistics...");
    let stats_result = sys_stats(0, &mut stats_buffer); // 0 = general stats
    kprintln("[USERLAND] Stats retrieved");
    
    // Example 6: Debug operation
    kprintln("[USERLAND] Performing debug operation...");
    let debug_result = sys_debug(1, 42); // op=1, arg=42
    kprintln("[USERLAND] Debug operation completed");
    
    // Example 7: Demonstrate error handling
    kprintln("[USERLAND] Testing error handling...");
    let invalid_result = sys_send(99999, b"test"); // Invalid destination
    if is_error(invalid_result) {
        if let Some(error_code) = get_error_code(invalid_result) {
            kprintln("[USERLAND] Got expected error");
        }
    }
    
    // Example 8: Raw syscall usage
    kprintln("[USERLAND] Testing raw syscall...");
    let raw_result = unsafe { 
        raw_syscall(syscall_numbers::SYS_YIELD, [0, 0, 0, 0]) 
    };
    kprintln("[USERLAND] Raw syscall completed");
    
    // Example 9: Multiple yield operations to demonstrate scheduling
    kprintln("[USERLAND] Performing multiple yields...");
    for i in 0..5 {
        kprintln("[USERLAND] Yield iteration");
        let _ = sys_yield();
    }
    
    // Example 10: Exit the program
    kprintln("[USERLAND] Example completed, exiting...");
    sys_exit(0);
}

/// Simple print function for userland programs
/// 
/// In a real system, this would use proper I/O mechanisms
fn kprintln(fmt: &str) {
    // This is a simplified version - in reality, this would use
    // proper kernel I/O syscalls or be provided by a runtime library
    
    // For demonstration purposes, we'll use a debug syscall
    let message = fmt.as_bytes();
    let _ = sys_debug(100, message.len() as u64); // op=100 for print
    
    // In a real implementation, you might have:
    // - A write syscall for stdout
    // - A logging syscall for debug output
    // - A message passing mechanism to a logging service
}

/// Panic handler for userland programs
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // Log the panic information
    let message = format!("[USERLAND PANIC] {}", info);
    let _ = sys_debug(200, message.len() as u64); // op=200 for panic
    
    // Exit with error code
    sys_exit(1);
}

/// Global allocator for userland programs
/// 
/// This provides basic memory allocation capabilities
#[global_allocator]
static ALLOCATOR: userland_stubs::alloc::UserlandAllocator = 
    userland_stubs::alloc::UserlandAllocator::new();

/// Module for userland memory allocation
/// 
/// This provides basic memory management for userland programs
pub mod alloc {
    use core::alloc::{GlobalAlloc, Layout};
    
    /// Simple userland allocator
    /// 
    /// This is a basic implementation that would be replaced
    /// by a proper memory management system in production
    pub struct UserlandAllocator;
    
    impl UserlandAllocator {
        pub const fn new() -> Self {
            Self
        }
    }
    
    unsafe impl GlobalAlloc for UserlandAllocator {
        unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
            // In a real system, this would use kernel memory allocation syscalls
            // For now, return null to indicate allocation failure
            core::ptr::null_mut()
        }
        
        unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
            // In a real system, this would free memory via kernel syscalls
        }
    }
}

/// Module for userland I/O operations
/// 
/// This provides basic input/output capabilities for userland programs
pub mod io {
    use userland_stubs::*;
    
    /// Write bytes to stdout
    /// 
    /// # Arguments
    /// * `bytes` - Bytes to write
    /// 
    /// # Returns
    /// Number of bytes written
    pub fn write_stdout(bytes: &[u8]) -> u64 {
        // In a real system, this would use a write syscall
        // For now, we'll use the debug syscall as a placeholder
        sys_debug(300, bytes.len() as u64) // op=300 for stdout write
    }
    
    /// Read bytes from stdin
    /// 
    /// # Arguments
    /// * `buffer` - Buffer to read into
    /// 
    /// # Returns
    /// Number of bytes read
    pub fn read_stdin(buffer: &mut [u8]) -> u64 {
        // In a real system, this would use a read syscall
        // For now, we'll use the debug syscall as a placeholder
        sys_debug(301, buffer.len() as u64) // op=301 for stdin read
    }
}

/// Module for userland process management
/// 
/// This provides basic process control capabilities
pub mod process {
    use userland_stubs::*;
    
    /// Get current task ID
    /// 
    /// # Returns
    /// Current task ID
    pub fn get_current_task_id() -> u64 {
        // In a real system, this would use a gettid syscall
        // For now, we'll use the debug syscall as a placeholder
        sys_debug(400, 0) // op=400 for get current task ID
    }
    
    /// Sleep for specified milliseconds
    /// 
    /// # Arguments
    /// * `milliseconds` - Time to sleep in milliseconds
    pub fn sleep(milliseconds: u64) {
        // In a real system, this would use a sleep syscall
        // For now, we'll use the debug syscall as a placeholder
        let _ = sys_debug(401, milliseconds); // op=401 for sleep
    }
}




