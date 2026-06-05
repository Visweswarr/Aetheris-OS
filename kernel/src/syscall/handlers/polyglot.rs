//! Polyglot Runtime System Call Handlers
//!
//! This module implements syscall handlers for the Aetheris Polyglot Runtime,
//! enabling multi-language application execution with capability-based security.
//!
//! # Syscalls
//!
//! - `SYS_POLYGLOT_CREATE` (100): Create a new polyglot sandbox
//! - `SYS_POLYGLOT_EXEC` (101): Execute code in a sandbox
//! - `SYS_POLYGLOT_TERMINATE` (102): Terminate a sandbox
//! - `SYS_POLYGLOT_STATUS` (103): Get sandbox status
//! - `SYS_POLYGLOT_SEND` (104): Send message via bridge
//! - `SYS_POLYGLOT_RECV` (105): Receive message via bridge
//! - `SYS_POLYGLOT_METRICS` (106): Get runtime metrics

use crate::klog;
use crate::aetheris_polyglot::{
    self, LanguageType, ResourceLimits, SandboxState,
    Message, Destination,
};
use crate::aetheris_polyglot::sandbox::SandboxId;
use alloc::string::String;
use alloc::vec::Vec;

/// Error codes for polyglot syscalls
pub mod errno {
    pub const SUCCESS: u64 = 0;
    pub const EINVAL: u64 = 22;    // Invalid argument
    pub const ENOENT: u64 = 2;     // No such sandbox
    pub const ENOMEM: u64 = 12;    // Out of memory
    pub const EPERM: u64 = 1;      // Permission denied
    pub const EBUSY: u64 = 16;     // Sandbox busy
    pub const EAGAIN: u64 = 11;    // Try again (no message)
    pub const EFAULT: u64 = 14;    // Bad address
    pub const ENOSYS: u64 = 38;    // Function not implemented
}

/// Handle SYS_POLYGLOT_CREATE syscall
///
/// Creates a new polyglot sandbox from a manifest.
///
/// # Arguments
/// * `a0` - Pointer to manifest data
/// * `a1` - Length of manifest data
/// * `a2` - Flags (reserved, must be 0)
/// * `_a3` - Unused
///
/// # Returns
/// Sandbox ID on success, error code on failure
pub fn handle_polyglot_create(a0: u64, a1: u64, a2: u64, _a3: u64) -> u64 {
    let manifest_ptr = a0;
    let manifest_len = a1;
    let flags = a2;

    klog!(TRACE, "[POLYGLOT_SYSCALL] polyglot_create: ptr=0x{:x} len={} flags={}",
          manifest_ptr, manifest_len, flags);

    // Validate arguments
    if manifest_ptr == 0 || manifest_len == 0 {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_create: invalid arguments");
        return errno::EINVAL;
    }

    if flags != 0 {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_create: unsupported flags");
        return errno::EINVAL;
    }

    // Get the runtime manager
    let manager = aetheris_polyglot::get_runtime_manager();
    let mut manager_guard = manager.lock();

    // For now, create a sandbox with default settings
    // In a full implementation, we would parse the manifest from user memory
    let default_limits = ResourceLimits {
        max_memory_bytes: 64 * 1024 * 1024, // 64 MB
        max_cpu_time_ms: 30000,              // 30 seconds
        max_file_descriptors: 64,
        max_network_connections: 8,
    };

    // Create sandbox with WASM backend (default)
    match manager_guard.create_sandbox_with_limits(LanguageType::Wasm, default_limits) {
        Ok(sandbox_id) => {
            klog!(INFO, "[POLYGLOT_SYSCALL] Created sandbox {:?}", sandbox_id);
            sandbox_id.0
        }
        Err(e) => {
            klog!(WARN, "[POLYGLOT_SYSCALL] Failed to create sandbox: {:?}", e);
            errno::ENOMEM
        }
    }
}

/// Handle SYS_POLYGLOT_EXEC syscall
///
/// Executes code in a polyglot sandbox.
///
/// # Arguments
/// * `a0` - Sandbox ID
/// * `a1` - Pointer to entry point name
/// * `a2` - Length of entry point name
/// * `a3` - Pointer to arguments (CBOR-encoded)
///
/// # Returns
/// 0 on success, error code on failure
pub fn handle_polyglot_exec(a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let sandbox_id = SandboxId(a0);
    let entry_ptr = a1;
    let entry_len = a2;
    let args_ptr = a3;

    klog!(TRACE, "[POLYGLOT_SYSCALL] polyglot_exec: sandbox={:?} entry=0x{:x} len={} args=0x{:x}",
          sandbox_id, entry_ptr, entry_len, args_ptr);

    // Validate arguments
    if entry_ptr == 0 || entry_len == 0 {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_exec: invalid entry point");
        return errno::EINVAL;
    }

    // Get the runtime manager
    let manager = aetheris_polyglot::get_runtime_manager();
    let manager_guard = manager.lock();

    // Check if sandbox exists
    if !manager_guard.has_sandbox(sandbox_id) {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_exec: sandbox {:?} not found", sandbox_id);
        return errno::ENOENT;
    }

    // In a full implementation, we would:
    // 1. Copy entry point name from user memory
    // 2. Copy arguments from user memory
    // 3. Execute the function in the sandbox
    // For now, return success as a stub
    klog!(INFO, "[POLYGLOT_SYSCALL] Executing in sandbox {:?}", sandbox_id);
    errno::SUCCESS
}

/// Handle SYS_POLYGLOT_TERMINATE syscall
///
/// Terminates a polyglot sandbox and releases its resources.
///
/// # Arguments
/// * `a0` - Sandbox ID
/// * `a1` - Flags (0 = graceful, 1 = force)
/// * `_a2` - Unused
/// * `_a3` - Unused
///
/// # Returns
/// 0 on success, error code on failure
pub fn handle_polyglot_terminate(a0: u64, a1: u64, _a2: u64, _a3: u64) -> u64 {
    let sandbox_id = SandboxId(a0);
    let flags = a1;

    klog!(TRACE, "[POLYGLOT_SYSCALL] polyglot_terminate: sandbox={:?} flags={}",
          sandbox_id, flags);

    // Get the runtime manager
    let manager = aetheris_polyglot::get_runtime_manager();
    let mut manager_guard = manager.lock();

    // Check if sandbox exists
    if !manager_guard.has_sandbox(sandbox_id) {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_terminate: sandbox {:?} not found", sandbox_id);
        return errno::ENOENT;
    }

    // Terminate the sandbox
    match manager_guard.terminate_sandbox(sandbox_id) {
        Ok(usage) => {
            klog!(INFO, "[POLYGLOT_SYSCALL] Terminated sandbox {:?}, final usage: {:?}", 
                  sandbox_id, usage);
            errno::SUCCESS
        }
        Err(e) => {
            klog!(WARN, "[POLYGLOT_SYSCALL] Failed to terminate sandbox {:?}: {:?}", 
                  sandbox_id, e);
            errno::EBUSY
        }
    }
}

/// Handle SYS_POLYGLOT_STATUS syscall
///
/// Gets the status of a polyglot sandbox.
///
/// # Arguments
/// * `a0` - Sandbox ID
/// * `a1` - Pointer to status buffer
/// * `a2` - Length of status buffer
/// * `_a3` - Unused
///
/// # Returns
/// 0 on success, error code on failure
pub fn handle_polyglot_status(a0: u64, a1: u64, a2: u64, _a3: u64) -> u64 {
    let sandbox_id = SandboxId(a0);
    let status_ptr = a1;
    let status_len = a2;

    klog!(TRACE, "[POLYGLOT_SYSCALL] polyglot_status: sandbox={:?} ptr=0x{:x} len={}",
          sandbox_id, status_ptr, status_len);

    // Validate arguments
    if status_ptr == 0 || status_len == 0 {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_status: invalid buffer");
        return errno::EINVAL;
    }

    // Get the runtime manager
    let manager = aetheris_polyglot::get_runtime_manager();
    let manager_guard = manager.lock();

    // Get sandbox status
    match manager_guard.get_sandbox_state(sandbox_id) {
        Some(state) => {
            klog!(TRACE, "[POLYGLOT_SYSCALL] Sandbox {:?} state: {:?}", sandbox_id, state);
            // In a full implementation, we would copy the status to user memory
            errno::SUCCESS
        }
        None => {
            klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_status: sandbox {:?} not found", sandbox_id);
            errno::ENOENT
        }
    }
}

/// Handle SYS_POLYGLOT_SEND syscall
///
/// Sends a message via the polyglot bridge.
///
/// # Arguments
/// * `a0` - Source sandbox ID
/// * `a1` - Destination sandbox ID (or topic ID)
/// * `a2` - Pointer to message data
/// * `a3` - Length of message data
///
/// # Returns
/// 0 on success, error code on failure
pub fn handle_polyglot_send(a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let src_sandbox = SandboxId(a0);
    let dst_sandbox = a1;
    let msg_ptr = a2;
    let msg_len = a3;

    klog!(TRACE, "[POLYGLOT_SYSCALL] polyglot_send: src={:?} dst={} ptr=0x{:x} len={}",
          src_sandbox, dst_sandbox, msg_ptr, msg_len);

    // Validate arguments
    if msg_ptr == 0 || msg_len == 0 {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_send: invalid message");
        return errno::EINVAL;
    }

    // Get the runtime manager
    let manager = aetheris_polyglot::get_runtime_manager();
    let manager_guard = manager.lock();

    // Check if source sandbox exists
    if !manager_guard.has_sandbox(src_sandbox) {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_send: source sandbox {:?} not found", src_sandbox);
        return errno::ENOENT;
    }

    // In a full implementation, we would:
    // 1. Copy message from user memory
    // 2. Verify capability to send to destination
    // 3. Route message via Intent Bus
    klog!(INFO, "[POLYGLOT_SYSCALL] Message sent from {:?} to {}", src_sandbox, dst_sandbox);
    errno::SUCCESS
}

/// Handle SYS_POLYGLOT_RECV syscall
///
/// Receives a message via the polyglot bridge.
///
/// # Arguments
/// * `a0` - Sandbox ID
/// * `a1` - Pointer to receive buffer
/// * `a2` - Length of receive buffer
/// * `a3` - Timeout in milliseconds (0 = non-blocking)
///
/// # Returns
/// Number of bytes received on success, error code on failure
pub fn handle_polyglot_recv(a0: u64, a1: u64, a2: u64, a3: u64) -> u64 {
    let sandbox_id = SandboxId(a0);
    let buf_ptr = a1;
    let buf_len = a2;
    let timeout_ms = a3;

    klog!(TRACE, "[POLYGLOT_SYSCALL] polyglot_recv: sandbox={:?} ptr=0x{:x} len={} timeout={}",
          sandbox_id, buf_ptr, buf_len, timeout_ms);

    // Validate arguments
    if buf_ptr == 0 || buf_len == 0 {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_recv: invalid buffer");
        return errno::EINVAL;
    }

    // Get the runtime manager
    let manager = aetheris_polyglot::get_runtime_manager();
    let manager_guard = manager.lock();

    // Check if sandbox exists
    if !manager_guard.has_sandbox(sandbox_id) {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_recv: sandbox {:?} not found", sandbox_id);
        return errno::ENOENT;
    }

    // In a full implementation, we would:
    // 1. Check message queue for sandbox
    // 2. If blocking, wait for message or timeout
    // 3. Copy message to user buffer
    // For now, return "no message" (EAGAIN)
    klog!(TRACE, "[POLYGLOT_SYSCALL] No message available for {:?}", sandbox_id);
    errno::EAGAIN
}

/// Handle SYS_POLYGLOT_METRICS syscall
///
/// Gets runtime metrics for the polyglot subsystem.
///
/// # Arguments
/// * `a0` - Pointer to metrics buffer
/// * `a1` - Length of metrics buffer
/// * `_a2` - Unused
/// * `_a3` - Unused
///
/// # Returns
/// 0 on success, error code on failure
pub fn handle_polyglot_metrics(a0: u64, a1: u64, _a2: u64, _a3: u64) -> u64 {
    let metrics_ptr = a0;
    let metrics_len = a1;

    klog!(TRACE, "[POLYGLOT_SYSCALL] polyglot_metrics: ptr=0x{:x} len={}",
          metrics_ptr, metrics_len);

    // Validate arguments
    if metrics_ptr == 0 || metrics_len == 0 {
        klog!(WARN, "[POLYGLOT_SYSCALL] polyglot_metrics: invalid buffer");
        return errno::EINVAL;
    }

    // Get the runtime manager
    let manager = aetheris_polyglot::get_runtime_manager();
    let manager_guard = manager.lock();

    // Get metrics
    let stats = manager_guard.get_stats();
    klog!(TRACE, "[POLYGLOT_SYSCALL] Runtime stats: {:?}", stats);

    // In a full implementation, we would copy metrics to user memory
    errno::SUCCESS
}
