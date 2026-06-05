//! Kernel Macros
//! 
//! This module provides kernel-specific macros for assertions, logging,
//! and other common operations.

/// Kernel assertion macro
/// 
/// This macro checks a condition and, if it fails:
/// 1. Logs an audit entry with op=ASSERT_FAIL
/// 2. Logs the assertion failure message
/// 3. Halts the system
/// 
/// The macro can be enabled/disabled by build profile:
/// - Debug builds: Always enabled
/// - Release builds: Disabled by default (can be enabled with features)
/// 
/// # Examples
/// 
/// ```rust
/// kassert!(ptr != core::ptr::null(), "Pointer must not be null");
/// kassert!(size > 0, "Size must be positive, got {}", size);
/// ```
#[macro_export]
macro_rules! kassert {
    // Basic assertion with condition only
    ($cond:expr) => {
        $crate::macros::kassert_internal!($cond, "Assertion failed: {}", stringify!($cond))
    };
    
    // Assertion with condition and message
    ($cond:expr, $msg:expr) => {
        $crate::macros::kassert_internal!($cond, $msg)
    };
    
    // Assertion with condition, message, and format arguments
    ($cond:expr, $msg:expr, $($arg:tt)*) => {
        $crate::macros::kassert_internal!($cond, $msg, $($arg)*)
    };
}

/// Internal implementation of kernel assertion
/// 
/// This macro handles the actual assertion logic, including:
/// - Condition checking
/// - Audit logging
/// - Error message formatting
/// - System halting
#[macro_export]
macro_rules! kassert_internal {
    ($cond:expr, $msg:expr) => {
        if cfg!(debug_assertions) || cfg!(feature = "kassert-release") {
            if !$cond {
                $crate::macros::handle_assertion_failure!($msg);
            }
        }
    };
    
    ($cond:expr, $msg:expr, $($arg:tt)*) => {
        if cfg!(debug_assertions) || cfg!(feature = "kassert-release") {
            if !$cond {
                $crate::macros::handle_assertion_failure!($msg, $($arg)*);
            }
        }
    };
}

/// Handle assertion failure
/// 
/// This macro is called when an assertion fails and:
/// 1. Logs an audit entry with op=ASSERT_FAIL
/// 2. Logs the failure message
/// 3. Halts the system
#[macro_export]
macro_rules! handle_assertion_failure {
    ($msg:expr) => {
        $crate::macros::assertion_failure_internal!($msg, &[])
    };
    
    ($msg:expr, $($arg:tt)*) => {
        $crate::macros::assertion_failure_internal!($msg, &[$($arg)*])
    };
}

/// Internal assertion failure handler
/// 
/// This function performs the actual assertion failure handling:
/// - Creates audit entry with op=ASSERT_FAIL
/// - Logs the failure message
/// - Halts the system
pub fn assertion_failure_internal(msg: &str, args: &[&dyn core::fmt::Display]) {
    use crate::klog;
    use crate::log::{Level, tags};
    use crate::secman::audit;
    use alloc::string::ToString;
    use core::fmt::Write;
    
    let mut args_str = alloc::string::String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            args_str.push_str(", ");
        }
        let _ = write!(&mut args_str, "{}", arg);
    }
    
    // Format the message with arguments if provided
    let formatted_msg = if args.is_empty() {
        msg.to_string()
    } else {
        alloc::format!("{} with args: [{}]", msg, args_str)
    };
    
    // Log the assertion failure
    klog!(Level::ERROR, [tags::ASSERT], "KERNEL ASSERTION FAILED: {}", formatted_msg);
    
    // Create audit entry
    let audit_result = audit::log_event(
        audit::ops::ASSERT_FAIL,
        0, // No specific argument for assertion failures
        &formatted_msg
    );
    
    match audit_result {
        Ok(_) => {
            klog!(Level::INFO, [tags::ASSERT], "Assertion failure logged to audit trail");
        }
        Err(e) => {
            klog!(Level::ERROR, [tags::ASSERT], "Failed to log assertion failure to audit: {:?}", e);
        }
    }
    
    // Log additional context information
    klog!(Level::ERROR, [tags::ASSERT], "Assertion failure occurred at:");
    klog!(Level::ERROR, [tags::ASSERT], "  File: {}:{}", file!(), line!());
    klog!(Level::ERROR, [tags::ASSERT], "  Function: {}", crate::function_name!());
    
    // Log system state information
    let current_task = crate::sched::get_current_task_id();
    klog!(Level::ERROR, [tags::ASSERT], "  Current Task ID: {}", current_task);
    
    // Log memory and system statistics if available
    let sched_stats = crate::sched::get_scheduler_stats();
    klog!(Level::ERROR, [tags::ASSERT], "  Scheduler State: {} ready, {} running, {} blocked",
          sched_stats.ready_tasks, sched_stats.running_tasks, sched_stats.blocked_tasks);
    
    // Halt the system
    klog!(Level::ERROR, [tags::ASSERT], "System halted due to assertion failure");
    crate::macros::halt_system();
}

/// Halt the system
/// 
/// This function halts the system in a controlled manner.
/// In a real implementation, this would:
/// 1. Disable interrupts
/// 2. Save system state
/// 3. Enter a halt loop or trigger a watchdog reset
pub fn halt_system() -> ! {
    use crate::klog;
    use crate::log::{Level, tags};
    
    klog!(Level::ERROR, [tags::ASSERT], "Entering system halt state");
    
    // Disable interrupts
    unsafe {
        core::arch::asm!("cli");
    }
    
    // Enter infinite halt loop
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Get the current function name
/// 
/// This macro returns the name of the current function.
/// Note: This is a simplified implementation and may not work
/// in all cases or build configurations.
#[macro_export]
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3] // Remove "::f" suffix
    }}
}

/// Kernel assertion with custom audit operation
/// 
/// This macro allows specifying a custom audit operation code
/// when an assertion fails.
/// 
/// # Examples
/// 
/// ```rust
/// kassert_audit!(ptr != core::ptr::null(), audit::ops::NULL_POINTER, "Pointer must not be null");
/// ```
#[macro_export]
macro_rules! kassert_audit {
    ($cond:expr, $audit_op:expr, $msg:expr) => {
        if cfg!(debug_assertions) || cfg!(feature = "kassert-release") {
            if !$cond {
                $crate::macros::handle_assertion_failure_audit!($audit_op, $msg);
            }
        }
    };
    
    ($cond:expr, $audit_op:expr, $msg:expr, $($arg:tt)*) => {
        if cfg!(debug_assertions) || cfg!(feature = "kassert-release") {
            if !$cond {
                $crate::macros::handle_assertion_failure_audit!($audit_op, $msg, $($arg)*);
            }
        }
    };
}

/// Handle assertion failure with custom audit operation
#[macro_export]
macro_rules! handle_assertion_failure_audit {
    ($audit_op:expr, $msg:expr) => {
        $crate::macros::assertion_failure_audit_internal!($audit_op, $msg, &[])
    };
    
    ($audit_op:expr, $msg:expr, $($arg:tt)*) => {
        $crate::macros::assertion_failure_audit_internal!($audit_op, $msg, &[$($arg)*])
    };
}

/// Internal assertion failure handler with custom audit operation
pub fn assertion_failure_audit_internal(audit_op: u32, msg: &str, args: &[&dyn core::fmt::Display]) {
    use crate::klog;
    use crate::log::{Level, tags};
    use crate::secman::audit;
    use alloc::string::ToString;
    use core::fmt::Write;
    
    let mut args_str = alloc::string::String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            args_str.push_str(", ");
        }
        let _ = write!(&mut args_str, "{}", arg);
    }
    
    // Format the message with arguments if provided
    let formatted_msg = if args.is_empty() {
        msg.to_string()
    } else {
        alloc::format!("{} with args: [{}]", msg, args_str)
    };
    
    // Log the assertion failure
    klog!(Level::ERROR, [tags::ASSERT], "KERNEL ASSERTION FAILED: {}", formatted_msg);
    
    // Create audit entry with custom operation
    let audit_result = audit::log_event(
        audit_op as u16,
        0, // No specific argument
        &formatted_msg
    );
    
    match audit_result {
        Ok(_) => {
            klog!(Level::INFO, [tags::ASSERT], "Assertion failure logged to audit trail with op={}", audit_op);
        }
        Err(e) => {
            klog!(Level::ERROR, [tags::ASSERT], "Failed to log assertion failure to audit: {:?}", e);
        }
    }
    
    // Log additional context information
    klog!(Level::ERROR, [tags::ASSERT], "Assertion failure occurred at:");
    klog!(Level::ERROR, [tags::ASSERT], "  File: {}:{}", file!(), line!());
    klog!(Level::ERROR, [tags::ASSERT], "  Function: {}", crate::function_name!());
    klog!(Level::ERROR, [tags::ASSERT], "  Audit Operation: {}", audit_op);
    
    // Log system state information
    let current_task = crate::sched::get_current_task_id();
    klog!(Level::ERROR, [tags::ASSERT], "  Current Task ID: {}", current_task);
    
    // Halt the system
    klog!(Level::ERROR, [tags::ASSERT], "System halted due to assertion failure");
    halt_system();
}

/// Kernel assertion that only runs in debug builds
/// 
/// This macro is similar to kassert! but only runs in debug builds.
/// It's useful for expensive checks that shouldn't run in release builds.
/// 
/// # Examples
/// 
/// ```rust
/// kassert_debug!(expensive_validation_function(), "Validation failed");
/// ```
#[macro_export]
macro_rules! kassert_debug {
    ($cond:expr) => {
        if cfg!(debug_assertions) {
            $crate::macros::kassert_internal!($cond, "Debug assertion failed: {}", stringify!($cond))
        }
    };
    
    ($cond:expr, $msg:expr) => {
        if cfg!(debug_assertions) {
            $crate::macros::kassert_internal!($cond, $msg)
        }
    };
    
    ($cond:expr, $msg:expr, $($arg:tt)*) => {
        if cfg!(debug_assertions) {
            $crate::macros::kassert_internal!($cond, $msg, $($arg)*)
        }
    };
}

/// Kernel assertion that only runs in release builds
/// 
/// This macro is similar to kassert! but only runs in release builds.
/// It's useful for critical checks that should always run.
/// 
/// # Examples
/// 
/// ```rust
/// kassert_release!(critical_system_check(), "Critical check failed");
/// ```
#[macro_export]
macro_rules! kassert_release {
    ($cond:expr) => {
        if !cfg!(debug_assertions) {
            $crate::macros::kassert_internal!($cond, "Release assertion failed: {}", stringify!($cond))
        }
    };
    
    ($cond:expr, $msg:expr) => {
        if !cfg!(debug_assertions) {
            $crate::macros::kassert_internal!($cond, $msg)
        }
    };
    
    ($cond:expr, $msg:expr, $($arg:tt)*) => {
        if !cfg!(debug_assertions) {
            $crate::macros::kassert_internal!($cond, $msg, $($arg)*)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_kassert_basic() {
        // This should not panic
        kassert!(true, "This should not fail");
        
        // This should not panic in release builds (disabled by default)
        kassert!(false, "This should not fail in release builds");
    }
    
    #[test]
    fn test_kassert_debug() {
        // This should not panic in debug builds
        kassert_debug!(true, "Debug assertion should pass");
        
        // This should not panic in release builds
        kassert_debug!(false, "Debug assertion should not run in release");
    }
    
    #[test]
    fn test_kassert_release() {
        // This should not panic in debug builds
        kassert_release!(true, "Release assertion should not run in debug");
        
        // This should not panic in release builds
        kassert_release!(false, "Release assertion should not run in debug");
    }
    
    #[test]
    fn test_function_name() {
        let name = function_name!();
        assert!(!name.is_empty(), "Function name should not be empty");
    }
}


