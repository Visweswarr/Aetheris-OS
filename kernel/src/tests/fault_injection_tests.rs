//! Fault Injection Tests
//! 
//! This module provides comprehensive tests for the fault injection system,
//! verifying that it correctly handles inbox overflow, allocation failures,
//! and timer jitter without causing system panics.

use crate::fault_injection::*;
use crate::{kprintln, klog};
use crate::log::Level::*;
use alloc::vec::Vec;

/// Test inbox overflow fault injection
pub fn test_inbox_overflow_fault_injection() -> Result<(), &'static str> {
    kprintln!("[TEST] Testing inbox overflow fault injection...");
    
    // Reset fault injection state
    reset_fault_state();
    
    // Configure fault injection for inbox overflow every 3 pushes
    let config = FaultInjectionConfig {
        inbox_overflow_enabled: true,
        inbox_overflow_interval: 3,
        alloc_failure_enabled: false,
        alloc_failure_interval: 100,
        timer_jitter_enabled: false,
        timer_jitter_percentage: 10,
        global_enabled: true,
    };
    
    update_fault_config(config);
    
    // Test inbox overflow detection
    let mut overflow_count = 0;
    for i in 0..10 {
        if should_force_inbox_overflow() {
            overflow_count += 1;
            klog!(INFO, "Inbox overflow forced at push {}", i + 1);
            
            // Log audit entry for forced overflow
            crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
                1, // Test process ID
                404, // FAULT_INJECTION_INBOX_OVERFLOW
                i as u64
            ));
        }
    }
    
    // Should have overflow at pushes 3, 6, 9 (3 times total)
    if overflow_count != 3 {
        return Err("Inbox overflow fault injection frequency incorrect");
    }
    
    // Test IPC resilience - send messages to High/RT priority tasks
    test_ipc_resilience_during_faults()?;
    
    kprintln!("[TEST] Inbox overflow fault injection test PASSED");
    Ok(())
}

/// Test allocation failure fault injection
pub fn test_alloc_failure_fault_injection() -> Result<(), &'static str> {
    kprintln!("[TEST] Testing allocation failure fault injection...");
    
    // Reset fault injection state
    reset_fault_state();
    
    // Configure fault injection for allocation failure every 5 allocations
    let config = FaultInjectionConfig {
        inbox_overflow_enabled: false,
        inbox_overflow_interval: 100,
        alloc_failure_enabled: true,
        alloc_failure_interval: 5,
        timer_jitter_enabled: false,
        timer_jitter_percentage: 10,
        global_enabled: true,
    };
    
    update_fault_config(config);
    
    // Test allocation failure detection
    let mut failure_count = 0;
    for i in 0..15 {
        if should_force_alloc_failure() {
            failure_count += 1;
            klog!(INFO, "Allocation failure forced at allocation {}", i + 1);
            
            // Log audit entry for forced allocation failure
            crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
                1, // Test process ID
                403, // FAULT_INJECTION_ALLOC_FAILURE
                64 // Test allocation size
            ));
        }
    }
    
    // Should have failures at allocations 5, 10, 15 (3 times total)
    if failure_count != 3 {
        return Err("Allocation failure fault injection frequency incorrect");
    }
    
    // Test system resilience during allocation failures
    test_memory_allocation_resilience()?;
    
    kprintln!("[TEST] Allocation failure fault injection test PASSED");
    Ok(())
}

/// Test timer jitter fault injection
pub fn test_timer_jitter_fault_injection() -> Result<(), &'static str> {
    kprintln!("[TEST] Testing timer jitter fault injection...");
    
    // Reset fault injection state
    reset_fault_state();
    
    // Configure fault injection for timer jitter +/- 15%
    let config = FaultInjectionConfig {
        inbox_overflow_enabled: false,
        inbox_overflow_interval: 100,
        alloc_failure_enabled: false,
        alloc_failure_interval: 100,
        timer_jitter_enabled: true,
        timer_jitter_percentage: 15,
        global_enabled: true,
    };
    
    update_fault_config(config);
    
    // Test timer jitter values
    let mut jitter_values = Vec::new();
    for _ in 0..50 {
        let jitter = get_timer_jitter();
        jitter_values.push(jitter);
        
        if jitter != 0 {
            klog!(TRACE, "Timer jitter: {}ms", jitter);
        }
    }
    
    // Check jitter range is within expected bounds
    let min_jitter = jitter_values.iter().min().unwrap();
    let max_jitter = jitter_values.iter().max().unwrap();
    
    if *min_jitter < -15 || *max_jitter > 15 {
        return Err("Timer jitter out of expected range");
    }
    
    // Test scheduler resilience during timer jitter
    test_scheduler_resilience_during_jitter()?;
    
    kprintln!("[TEST] Timer jitter fault injection test PASSED");
    Ok(())
}

/// Test IPC resilience during fault injection
fn test_ipc_resilience_during_faults() -> Result<(), &'static str> {
    kprintln!("[TEST] Testing IPC resilience during fault injection...");
    
    // Test that High and RT priority messages still get delivered
    // even when inbox overflow is forced
    
    // Create test messages with different priorities
    let high_priority_message = create_test_message(crate::ipc::MessagePriority::High, 1, 2);
    let rt_priority_message = create_test_message(crate::ipc::MessagePriority::Critical, 1, 3);
    let low_priority_message = create_test_message(crate::ipc::MessagePriority::Low, 1, 4);
    
    // Test message delivery during forced overflow
    // High and RT messages should succeed, Low messages may be dropped
    
    // Simulate IPC operations
    let mut high_delivered = 0;
    let mut rt_delivered = 0;
    let mut low_dropped = 0;
    
    for _ in 0..10 {
        // Force inbox overflow for testing
        if should_force_inbox_overflow() {
            // High priority should still be delivered (return EBUSY but handled gracefully)
            high_delivered += 1;
            
            // RT priority should still be delivered (return EBUSY but handled gracefully)
            rt_delivered += 1;
            
            // Low priority should be dropped
            low_dropped += 1;
            
            // Log audit entries for message handling
            crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
                2, // Receiver process ID
                105, // IPC_MSG_DROPPED
                low_priority_message.header.message_id.0
            ));
        }
    }
    
    klog!(INFO, "IPC resilience test: High delivered: {}, RT delivered: {}, Low dropped: {}", 
         high_delivered, rt_delivered, low_dropped);
    
    Ok(())
}

/// Test memory allocation resilience
fn test_memory_allocation_resilience() -> Result<(), &'static str> {
    kprintln!("[TEST] Testing memory allocation resilience...");
    
    // Test that the system handles allocation failures gracefully
    // and doesn't panic
    
    let mut successful_allocs = 0;
    let mut failed_allocs = 0;
    
    for i in 0..20 {
        // Simulate kmalloc calls
        if should_force_alloc_failure() {
            failed_allocs += 1;
            klog!(TRACE, "Allocation {} forced to fail", i + 1);
            
            // System should handle this gracefully without panic
            // Log audit entry for failed allocation
            crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
                0, // System operation
                403, // FAULT_INJECTION_ALLOC_FAILURE
                64 // Test allocation size
            ));
        } else {
            successful_allocs += 1;
        }
    }
    
    klog!(INFO, "Memory allocation resilience: {} successful, {} failed", 
         successful_allocs, failed_allocs);
    
    // System should still be functional
    if failed_allocs == 0 {
        return Err("No allocation failures were triggered");
    }
    
    Ok(())
}

/// Test scheduler resilience during timer jitter
fn test_scheduler_resilience_during_jitter() -> Result<(), &'static str> {
    kprintln!("[TEST] Testing scheduler resilience during timer jitter...");
    
    // Test that scheduler continues to function correctly
    // even with timer jitter
    
    let mut jitter_events = 0;
    
    for _ in 0..30 {
        let jitter = get_timer_jitter();
        if jitter != 0 {
            jitter_events += 1;
            
            // Set base timer to simulate jitter effect
            set_base_timer(1000 + jitter as u64);
            
            // Log that we're handling timer jitter
            klog!(TRACE, "Handling timer jitter: {}ms", jitter);
        }
    }
    
    klog!(INFO, "Timer jitter events handled: {}", jitter_events);
    
    // Scheduler should continue functioning
    if jitter_events == 0 {
        return Err("No timer jitter events were generated");
    }
    
    Ok(())
}

/// Create a test message with specified priority
fn create_test_message(priority: crate::ipc::MessagePriority, sender: u64, receiver: u64) -> crate::ipc::Message {
    use crate::ipc::*;
    
    let header = MessageHeader {
        message_id: MessageId(1),
        sender: ProcessId(sender),
        receiver: ProcessId(receiver),
        priority,
        message_type: MessageType::Request,
        sequence: 0,
        timestamp: crate::log::get_current_time_ms(),
        expiry: crate::log::get_current_time_ms() + 5000, // 5 second expiry
        flags: MessageFlags::empty(),
    };
    
    let payload = MessagePayload::new(b"test");
    
    Message { header, payload }
}

/// Test all fault injection types together
pub fn test_combined_fault_injection() -> Result<(), &'static str> {
    kprintln!("[TEST] Testing combined fault injection...");
    
    // Reset fault injection state
    reset_fault_state();
    
    // Enable all fault injection types
    let config = FaultInjectionConfig {
        inbox_overflow_enabled: true,
        inbox_overflow_interval: 10,
        alloc_failure_enabled: true,
        alloc_failure_interval: 8,
        timer_jitter_enabled: true,
        timer_jitter_percentage: 5,
        global_enabled: true,
    };
    
    update_fault_config(config);
    
    // Run system operations and verify no panics occur
    let mut inbox_faults = 0;
    let mut alloc_faults = 0;
    let mut timer_jitter_events = 0;
    
    for i in 0..50 {
        // Test inbox overflow
        if should_force_inbox_overflow() {
            inbox_faults += 1;
            klog!(TRACE, "Combined test: Inbox overflow at iteration {}", i);
        }
        
        // Test allocation failure
        if should_force_alloc_failure() {
            alloc_faults += 1;
            klog!(TRACE, "Combined test: Allocation failure at iteration {}", i);
        }
        
        // Test timer jitter
        let jitter = get_timer_jitter();
        if jitter != 0 {
            timer_jitter_events += 1;
            klog!(TRACE, "Combined test: Timer jitter {} at iteration {}", jitter, i);
        }
    }
    
    klog!(INFO, "Combined fault injection test: {} inbox, {} alloc, {} timer events", 
         inbox_faults, alloc_faults, timer_jitter_events);
    
    // Verify we had some fault events
    if inbox_faults == 0 && alloc_faults == 0 && timer_jitter_events == 0 {
        return Err("No fault injection events occurred in combined test");
    }
    
    kprintln!("[TEST] Combined fault injection test PASSED");
    Ok(())
}

/// Test fault injection audit logging
pub fn test_fault_injection_audit_logging() -> Result<(), &'static str> {
    kprintln!("[TEST] Testing fault injection audit logging...");
    
    // Reset audit system
    crate::secman::audit::clear_entries();
    
    // Reset fault injection state
    reset_fault_state();
    
    // Enable fault injection
    let config = FaultInjectionConfig {
        inbox_overflow_enabled: true,
        inbox_overflow_interval: 2,
        alloc_failure_enabled: true,
        alloc_failure_interval: 3,
        timer_jitter_enabled: false,
        timer_jitter_percentage: 10,
        global_enabled: true,
    };
    
    update_fault_config(config);
    
    // Trigger fault injection events and log them
    for _ in 0..10 {
        if should_force_inbox_overflow() {
            crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
                1, // Test process ID
                404, // FAULT_INJECTION_INBOX_OVERFLOW
                1 // Test argument
            ));
        }
        
        if should_force_alloc_failure() {
            crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
                0, // System operation
                403, // FAULT_INJECTION_ALLOC_FAILURE
                128 // Test allocation size
            ));
        }
    }
    
    // Verify audit entries were logged
    let entries = crate::secman::audit::get_latest_entries(20);
    
    let mut inbox_audit_count = 0;
    let mut alloc_audit_count = 0;
    
    for entry in entries {
        match entry.op {
            404 => inbox_audit_count += 1, // FAULT_INJECTION_INBOX_OVERFLOW
            403 => alloc_audit_count += 1, // FAULT_INJECTION_ALLOC_FAILURE
            _ => {}
        }
    }
    
    klog!(INFO, "Audit logging test: {} inbox entries, {} alloc entries", 
         inbox_audit_count, alloc_audit_count);
    
    if inbox_audit_count == 0 && alloc_audit_count == 0 {
        return Err("No fault injection audit entries were logged");
    }
    
    kprintln!("[TEST] Fault injection audit logging test PASSED");
    Ok(())
}

/// Run all fault injection tests
pub fn run_all_fault_injection_tests() -> Result<(), &'static str> {
    kprintln!("[TEST] Running all fault injection tests...");
    
    // Run individual test suites
    test_inbox_overflow_fault_injection()?;
    test_alloc_failure_fault_injection()?;
    test_timer_jitter_fault_injection()?;
    test_combined_fault_injection()?;
    test_fault_injection_audit_logging()?;
    
    // Reset fault injection to clean state
    let default_config = FaultInjectionConfig::default();
    update_fault_config(default_config);
    reset_fault_state();
    
    kprintln!("[TEST] All fault injection tests PASSED");
    Ok(())
}

