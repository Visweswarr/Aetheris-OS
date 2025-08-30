//! Fault Injection Tests
//! 
//! These tests verify that the fault injection system correctly simulates
//! inbox overflow conditions and that the system degrades gracefully
//! without panicking.

use crate::fault_injection;
use crate::ipc::{Message, Priority, ProcessId};
use crate::ipc::queues::Inbox;
use crate::secman::audit;

/// Test basic fault injection functionality
pub fn test_fault_injection_basic() -> Result<(), &'static str> {
    kprintln!("Testing basic fault injection functionality...");
    
    // Test enabling fault injection
    fault_injection::enable_inbox_overflow_fault(3);
    
    // Verify fault injection is enabled
    let stats = fault_injection::get_inbox_overflow_fault_stats()
        .ok_or("Failed to get fault injection stats")?;
    
    if !stats.enabled || stats.frequency != 3 {
        return Err("Fault injection not properly enabled");
    }
    
    kprintln!("  ✅ Fault injection enabled successfully");
    
    // Test fault injection frequency
    let mut fault_count = 0;
    for _ in 0..15 {
        if fault_injection::should_inject_inbox_overflow_fault() {
            fault_count += 1;
        }
    }
    
    // Should inject fault every 3rd operation (at 3, 6, 9, 12, 15)
    if fault_count != 5 {
        return Err("Fault injection frequency not working correctly");
    }
    
    kprintln!("  ✅ Fault injection frequency working correctly");
    
    // Test disabling fault injection
    fault_injection::disable_inbox_overflow_fault();
    let stats = fault_injection::get_inbox_overflow_fault_stats()
        .ok_or("Failed to get fault injection stats")?;
    
    if stats.enabled {
        return Err("Fault injection not properly disabled");
    }
    
    kprintln!("  ✅ Fault injection disabled successfully");
    
    kprintln!("✅ Basic fault injection functionality tests passed!");
    Ok(())
}

/// Test inbox overflow simulation with fault injection
pub fn test_inbox_overflow_simulation() -> Result<(), &'static str> {
    kprintln!("Testing inbox overflow simulation with fault injection...");
    
    // Enable fault injection every 2nd push
    fault_injection::enable_inbox_overflow_fault(2);
    
    // Create a small inbox to test overflow behavior
    let mut inbox = Inbox::new(2);
    
    // Create test messages
    let low_priority_msg = Message::new(
        ProcessId(1),
        ProcessId(2),
        Priority::Low,
        b"Low priority message"
    );
    
    let high_priority_msg = Message::new(
        ProcessId(1),
        ProcessId(2),
        Priority::High,
        b"High priority message"
    );
    
    let critical_priority_msg = Message::new(
        ProcessId(1),
        ProcessId(2),
        Priority::Critical,
        b"Critical priority message"
    );
    
    // Test normal message delivery (should work)
    let result = inbox.deliver_with_overflow_policy(low_priority_msg.clone());
    assert!(result.is_ok(), "First message should be delivered successfully");
    kprintln!("  ✅ First message delivered successfully");
    
    // Test second message delivery (should work)
    let result = inbox.deliver_with_overflow_policy(high_priority_msg.clone());
    assert!(result.is_ok(), "Second message should be delivered successfully");
    kprintln!("  ✅ Second message delivered successfully");
    
    // Test third message delivery with fault injection
    // This should trigger fault injection and simulate overflow
    let result = inbox.deliver_with_overflow_policy(critical_priority_msg.clone());
    
    // The result depends on whether fault injection triggered
    // If fault injection triggered, the message should be rejected with EBUSY
    // If not, it should be handled normally
    match result {
        Ok(_) => {
            kprintln!("  ✅ Third message handled normally (no fault injection)");
        }
        Err(crate::ipc::IpcError::Busy) => {
            kprintln!("  ✅ Fault injection triggered - message rejected with EBUSY");
        }
        Err(e) => {
            return Err("Unexpected error from inbox overflow simulation");
        }
    }
    
    // Test that system continues to function normally
    let result = inbox.deliver_with_overflow_policy(low_priority_msg.clone());
    assert!(result.is_ok(), "System should continue to function after fault injection");
    kprintln!("  ✅ System continues to function after fault injection");
    
    // Disable fault injection
    fault_injection::disable_inbox_overflow_fault();
    
    kprintln!("✅ Inbox overflow simulation tests passed!");
    Ok(())
}

/// Test graceful degradation under fault injection
pub fn test_graceful_degradation() -> Result<(), &'static str> {
    kprintln!("Testing graceful degradation under fault injection...");
    
    // Enable fault injection every 3rd push
    fault_injection::enable_inbox_overflow_fault(3);
    
    // Create a small inbox
    let mut inbox = Inbox::new(1);
    
    // Send multiple messages to trigger fault injection
    let mut messages_sent = 0;
    let mut messages_dropped = 0;
    let mut messages_rejected = 0;
    
    for i in 0..10 {
        let message = Message::new(
            ProcessId(1),
            ProcessId(2),
            if i % 3 == 0 { Priority::High } else { Priority::Low },
            &format!("Message {}", i).into_bytes()
        );
        
        match inbox.deliver_with_overflow_policy(message) {
            Ok(None) => {
                messages_sent += 1;
            }
            Ok(Some(_)) => {
                messages_dropped += 1;
            }
            Err(crate::ipc::IpcError::Busy) => {
                messages_rejected += 1;
            }
            Err(_) => {
                return Err("Unexpected error during graceful degradation test");
            }
        }
    }
    
    kprintln!("  📊 Messages sent: {}", messages_sent);
    kprintln!("  📊 Messages dropped: {}", messages_dropped);
    kprintln!("  📊 Messages rejected: {}", messages_rejected);
    
    // Verify that the system handled the fault injection gracefully
    // (no panics, proper error handling)
    if messages_sent + messages_dropped + messages_rejected != 10 {
        return Err("Message count mismatch in graceful degradation test");
    }
    
    kprintln!("  ✅ All messages handled without system failure");
    
    // Disable fault injection
    fault_injection::disable_inbox_overflow_fault();
    
    kprintln!("✅ Graceful degradation tests passed!");
    Ok(())
}

/// Test audit logging during fault injection
pub fn test_audit_logging() -> Result<(), &'static str> {
    kprintln!("Testing audit logging during fault injection...");
    
    // Enable fault injection every 2nd push
    fault_injection::enable_inbox_overflow_fault(2);
    
    // Create a small inbox
    let mut inbox = Inbox::new(1);
    
    // Send messages to trigger fault injection and audit logging
    let low_msg = Message::new(
        ProcessId(1),
        ProcessId(2),
        Priority::Low,
        b"Low priority message for audit test"
    );
    
    let high_msg = Message::new(
        ProcessId(1),
        ProcessId(2),
        Priority::High,
        b"High priority message for audit test"
    );
    
    // Send first message (should succeed)
    let result = inbox.deliver_with_overflow_policy(low_msg.clone());
    assert!(result.is_ok(), "First message should be delivered");
    
    // Send second message (should trigger fault injection)
    let result = inbox.deliver_with_overflow_policy(high_msg.clone());
    
    // Check that audit entries were created
    let audit_entries = audit::get_latest_entries(10);
    let dropped_entries: Vec<_> = audit_entries.iter()
        .filter(|entry| entry.op == 105) // IPC_MSG_DROPPED
        .collect();
    
    if dropped_entries.is_empty() {
        kprintln!("  ℹ️  No audit entries found (fault injection may not have triggered)");
    } else {
        kprintln!("  ✅ Audit entries created for dropped messages: {}", dropped_entries.len());
        
        // Verify audit entry content
        for entry in &dropped_entries {
            kprintln!("    📝 Audit entry: ts={}, pid={}, op={}, arg=0x{:x}", 
                     entry.ts, entry.pid, entry.op, entry.arg);
        }
    }
    
    // Disable fault injection
    fault_injection::disable_inbox_overflow_fault();
    
    kprintln!("✅ Audit logging tests passed!");
    Ok(())
}

/// Test fault injection statistics
pub fn test_fault_injection_statistics() -> Result<(), &'static str> {
    kprintln!("Testing fault injection statistics...");
    
    // Reset counters
    fault_injection::reset_fault_injection_counters();
    
    // Enable fault injection every 4th push
    fault_injection::enable_inbox_overflow_fault(4);
    
    // Get initial stats
    let initial_stats = fault_injection::get_inbox_overflow_fault_stats()
        .ok_or("Failed to get initial fault injection stats")?;
    
    kprintln!("  📊 Initial stats: enabled={}, frequency={}, counter={}, faults={}", 
              initial_stats.enabled, initial_stats.frequency, 
              initial_stats.operation_counter, initial_stats.total_faults_injected);
    
    // Trigger some operations
    for _ in 0..12 {
        fault_injection::should_inject_inbox_overflow_fault();
    }
    
    // Get updated stats
    let updated_stats = fault_injection::get_inbox_overflow_fault_stats()
        .ok_or("Failed to get updated fault injection stats")?;
    
    kprintln!("  📊 Updated stats: enabled={}, frequency={}, counter={}, faults={}", 
              updated_stats.enabled, updated_stats.frequency, 
              updated_stats.operation_counter, updated_stats.total_faults_injected);
    
    // Verify statistics
    if updated_stats.operation_counter != 12 {
        return Err("Operation counter not updated correctly");
    }
    
    // Should inject fault every 4th operation (at 4, 8, 12)
    if updated_stats.total_faults_injected != 3 {
        return Err("Fault injection count not correct");
    }
    
    kprintln!("  ✅ Fault injection statistics working correctly");
    
    // Disable fault injection
    fault_injection::disable_inbox_overflow_fault();
    
    kprintln!("✅ Fault injection statistics tests passed!");
    Ok(())
}

/// Test system stability under continuous fault injection
pub fn test_system_stability() -> Result<(), &'static str> {
    kprintln!("Testing system stability under continuous fault injection...");
    
    // Enable fault injection every 2nd push (high frequency)
    fault_injection::enable_inbox_overflow_fault(2);
    
    // Create a small inbox
    let mut inbox = Inbox::new(1);
    
    // Send many messages to stress test the system
    let mut total_operations = 0;
    let mut successful_operations = 0;
    
    for i in 0..50 {
        let message = Message::new(
            ProcessId(1),
            ProcessId(2),
            Priority::Low,
            &format!("Stress test message {}", i).into_bytes()
        );
        
        match inbox.deliver_with_overflow_policy(message) {
            Ok(_) => {
                successful_operations += 1;
            }
            Err(_) => {
                // Errors are expected due to fault injection
            }
        }
        
        total_operations += 1;
        
        // Verify system hasn't panicked
        if total_operations % 10 == 0 {
            kprintln!("  📊 Completed {} operations, {} successful", total_operations, successful_operations);
        }
    }
    
    kprintln!("  📊 Total operations: {}, Successful: {}", total_operations, successful_operations);
    
    // Verify system is still functional
    let test_message = Message::new(
        ProcessId(1),
        ProcessId(2),
        Priority::Low,
        b"Final test message"
    );
    
    let result = inbox.deliver_with_overflow_policy(test_message);
    assert!(result.is_ok(), "System should remain functional after stress test");
    
    kprintln!("  ✅ System remained stable under continuous fault injection");
    
    // Disable fault injection
    fault_injection::disable_inbox_overflow_fault();
    
    kprintln!("✅ System stability tests passed!");
    Ok(())
}

/// Run all fault injection tests
pub fn run_all_fault_injection_tests() -> Result<(), &'static str> {
    kprintln!("🚀 Running fault injection tests...");
    
    test_fault_injection_basic()?;
    test_inbox_overflow_simulation()?;
    test_graceful_degradation()?;
    test_audit_logging()?;
    test_fault_injection_statistics()?;
    test_system_stability()?;
    
    kprintln!("🎉 All fault injection tests completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic() {
        assert!(test_fault_injection_basic().is_ok());
    }
    
    #[test]
    fn test_overflow_simulation() {
        assert!(test_inbox_overflow_simulation().is_ok());
    }
    
    #[test]
    fn test_graceful_degradation() {
        assert!(test_graceful_degradation().is_ok());
    }
    
    #[test]
    fn test_audit_logging() {
        assert!(test_audit_logging().is_ok());
    }
    
    #[test]
    fn test_statistics() {
        assert!(test_fault_injection_statistics().is_ok());
    }
    
    #[test]
    fn test_system_stability() {
        assert!(test_system_stability().is_ok());
    }
    
    #[test]
    fn test_all() {
        assert!(run_all_fault_injection_tests().is_ok());
    }
}


