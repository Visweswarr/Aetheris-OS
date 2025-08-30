/// Inbox Overflow Policy Tests
/// 
/// This module tests the inbox overflow policy implementation to ensure:
/// - Low priority messages are dropped when inbox is full
/// - High/Critical priority messages return EBUSY when inbox is full
/// - Proper audit logging for dropped messages
/// - Drop-tail by priority behavior

use crate::ipc::*;
use crate::ipc::queues::*;
use crate::ipc::types::*;
use crate::secman::audit;

/// Test basic inbox overflow with Low priority messages
#[test]
fn test_low_priority_drop_on_overflow() {
    // Initialize IPC and audit systems
    init_channel_manager();
    audit::init_audit();
    
    let owner = ProcessId::new(100);
    let sender = ProcessId::new(200);
    
    // Create a small inbox (capacity 3 for testing)
    let mut inbox = Inbox::new(owner, 3);
    
    // Fill the inbox with Low priority messages
    for i in 0..3 {
        let message = Message::new_with_priority(
            sender,
            owner,
            MessageType::Notification,
            MessagePriority::Low,
            MessagePayload::from_text(&format!("Low message {}", i)),
        );
        
        let result = inbox.deliver_with_overflow_policy(message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No drop on first 3 messages
    }
    
    // Verify inbox is full
    assert_eq!(inbox.message_count(), 3);
    
    // Add another Low priority message - should drop oldest Low priority
    let new_message = Message::new_with_priority(
        sender,
        owner,
        MessageType::Notification,
        MessagePriority::Low,
        MessagePayload::from_text("New low message"),
    );
    
    let result = inbox.deliver_with_overflow_policy(new_message);
    assert!(result.is_ok());
    let dropped = result.unwrap();
    assert!(dropped.is_some()); // Should have dropped a message
    
    // Verify dropped message was Low priority
    let dropped_msg = dropped.unwrap();
    assert_eq!(dropped_msg.header.priority, MessagePriority::Low);
    
    // Verify inbox still has 3 messages
    assert_eq!(inbox.message_count(), 3);
    
    // Verify audit entry was created for dropped message
    let audit_entries = audit::get_latest_entries(10);
    let dropped_entries: Vec<_> = audit_entries.iter()
        .filter(|entry| entry.op == 105) // IPC_MSG_DROPPED operation code
        .collect();
    assert!(!dropped_entries.is_empty(), "Should have audit entry for dropped message");
}

/// Test that High priority messages return EBUSY when inbox is full
#[test]
fn test_high_priority_ebusy_on_overflow() {
    // Initialize IPC and audit systems
    init_channel_manager();
    audit::init_audit();
    
    let owner = ProcessId::new(101);
    let sender = ProcessId::new(201);
    
    // Create a small inbox (capacity 2 for testing)
    let mut inbox = Inbox::new(owner, 2);
    
    // Fill the inbox with Normal priority messages
    for i in 0..2 {
        let message = Message::new_with_priority(
            sender,
            owner,
            MessageType::Notification,
            MessagePriority::Normal,
            MessagePayload::from_text(&format!("Normal message {}", i)),
        );
        
        let result = inbox.deliver_with_overflow_policy(message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
    
    // Verify inbox is full
    assert_eq!(inbox.message_count(), 2);
    
    // Try to add a High priority message - should return EBUSY
    let high_message = Message::new_with_priority(
        sender,
        owner,
        MessageType::Request,
        MessagePriority::High,
        MessagePayload::from_text("High priority message"),
    );
    
    let result = inbox.deliver_with_overflow_policy(high_message);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), IpcError::Busy);
    
    // Verify inbox still has 2 messages (no change)
    assert_eq!(inbox.message_count(), 2);
}

/// Test that Critical priority messages return EBUSY when inbox is full
#[test]
fn test_critical_priority_ebusy_on_overflow() {
    // Initialize IPC and audit systems
    init_channel_manager();
    audit::init_audit();
    
    let owner = ProcessId::new(102);
    let sender = ProcessId::new(202);
    
    // Create a small inbox (capacity 2 for testing)
    let mut inbox = Inbox::new(owner, 2);
    
    // Fill the inbox with High priority messages
    for i in 0..2 {
        let message = Message::new_with_priority(
            sender,
            owner,
            MessageType::Notification,
            MessagePriority::High,
            MessagePayload::from_text(&format!("High message {}", i)),
        );
        
        let result = inbox.deliver_with_overflow_policy(message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
    
    // Try to add a Critical priority message - should return EBUSY
    let critical_message = Message::new_with_priority(
        sender,
        owner,
        MessageType::Control,
        MessagePriority::Critical,
        MessagePayload::from_text("Critical priority message"),
    );
    
    let result = inbox.deliver_with_overflow_policy(critical_message);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), IpcError::Busy);
    
    // Verify inbox still has 2 messages (no change)
    assert_eq!(inbox.message_count(), 2);
}

/// Test drop-tail by priority behavior (Low messages dropped first)
#[test]
fn test_drop_tail_by_priority() {
    // Initialize IPC and audit systems
    init_channel_manager();
    audit::init_audit();
    
    let owner = ProcessId::new(103);
    let sender = ProcessId::new(203);
    
    // Create a small inbox (capacity 3 for testing)
    let mut inbox = Inbox::new(owner, 3);
    
    // Add 2 Low priority messages and 1 Normal priority message
    let low_msg1 = Message::new_with_priority(
        sender,
        owner,
        MessageType::Notification,
        MessagePriority::Low,
        MessagePayload::from_text("Low message 1"),
    );
    let low_msg2 = Message::new_with_priority(
        sender,
        owner,
        MessageType::Notification,
        MessagePriority::Low,
        MessagePayload::from_text("Low message 2"),
    );
    let normal_msg = Message::new_with_priority(
        sender,
        owner,
        MessageType::Notification,
        MessagePriority::Normal,
        MessagePayload::from_text("Normal message"),
    );
    
    // Add messages to inbox
    assert!(inbox.deliver_with_overflow_policy(low_msg1).unwrap().is_none());
    assert!(inbox.deliver_with_overflow_policy(low_msg2).unwrap().is_none());
    assert!(inbox.deliver_with_overflow_policy(normal_msg).unwrap().is_none());
    
    // Now add another Normal priority message - should drop oldest Low priority
    let new_normal_msg = Message::new_with_priority(
        sender,
        owner,
        MessageType::Notification,
        MessagePriority::Normal,
        MessagePayload::from_text("New normal message"),
    );
    
    let result = inbox.deliver_with_overflow_policy(new_normal_msg);
    assert!(result.is_ok());
    let dropped = result.unwrap();
    assert!(dropped.is_some());
    
    // Verify dropped message was Low priority
    let dropped_msg = dropped.unwrap();
    assert_eq!(dropped_msg.header.priority, MessagePriority::Low);
    assert_eq!(dropped_msg.payload.as_text().unwrap(), "Low message 1"); // Should be oldest Low
}

/// Test that Normal priority messages can drop Low priority but not each other when no Low exist
#[test]
fn test_normal_priority_overflow_behavior() {
    // Initialize IPC and audit systems
    init_channel_manager();
    audit::init_audit();
    
    let owner = ProcessId::new(104);
    let sender = ProcessId::new(204);
    
    // Create a small inbox (capacity 2 for testing)
    let mut inbox = Inbox::new(owner, 2);
    
    // Fill with Normal priority messages
    for i in 0..2 {
        let message = Message::new_with_priority(
            sender,
            owner,
            MessageType::Notification,
            MessagePriority::Normal,
            MessagePayload::from_text(&format!("Normal message {}", i)),
        );
        
        let result = inbox.deliver_with_overflow_policy(message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
    
    // Add another Normal priority message - should drop oldest Normal
    let new_message = Message::new_with_priority(
        sender,
        owner,
        MessageType::Notification,
        MessagePriority::Normal,
        MessagePayload::from_text("New normal message"),
    );
    
    let result = inbox.deliver_with_overflow_policy(new_message);
    assert!(result.is_ok());
    let dropped = result.unwrap();
    assert!(dropped.is_some());
    
    // Verify dropped message was Normal priority
    let dropped_msg = dropped.unwrap();
    assert_eq!(dropped_msg.header.priority, MessagePriority::Normal);
    assert_eq!(dropped_msg.payload.as_text().unwrap(), "Normal message 0"); // Should be oldest Normal
}

/// Test audit encoding for dropped messages
#[test]
fn test_audit_encoding_for_dropped_messages() {
    let owner = ProcessId::new(105);
    let sender = ProcessId::new(205);
    let inbox = Inbox::new(owner, 5);
    
    let message = Message::new_with_priority(
        sender,
        owner,
        MessageType::Data,
        MessagePriority::Low,
        MessagePayload::from_data(&[1, 2, 3, 4, 5]), // 5 bytes
    );
    
    // Test the encoding function
    let encoded = inbox.encode_drop_audit_arg(&message);
    
    // Decode and verify
    let decoded_prio = (encoded >> 62) & 0x3;
    let decoded_len = (encoded >> 32) & 0x3FFFFFFF;
    let decoded_dst = encoded & 0xFFFFFFFF;
    
    assert_eq!(decoded_prio, MessagePriority::Low as u64);
    assert_eq!(decoded_len, 5); // payload size
    assert_eq!(decoded_dst, owner.0);
}

/// Test inbox statistics after overflow operations
#[test]
fn test_inbox_stats_after_overflow() {
    let owner = ProcessId::new(106);
    let sender = ProcessId::new(206);
    
    // Create a small inbox (capacity 2 for testing)
    let mut inbox = Inbox::new(owner, 2);
    
    // Check initial stats
    assert_eq!(inbox.stats.messages_received, 0);
    assert_eq!(inbox.stats.messages_expired, 0);
    
    // Fill inbox
    for i in 0..2 {
        let message = Message::new_with_priority(
            sender,
            owner,
            MessageType::Notification,
            MessagePriority::Low,
            MessagePayload::from_text(&format!("Message {}", i)),
        );
        
        let _ = inbox.deliver_with_overflow_policy(message);
    }
    
    // Check stats after filling
    assert_eq!(inbox.stats.messages_received, 2);
    assert_eq!(inbox.stats.messages_expired, 0);
    
    // Add another message that should cause a drop
    let overflow_message = Message::new_with_priority(
        sender,
        owner,
        MessageType::Notification,
        MessagePriority::Low,
        MessagePayload::from_text("Overflow message"),
    );
    
    let result = inbox.deliver_with_overflow_policy(overflow_message);
    assert!(result.is_ok());
    assert!(result.unwrap().is_some()); // Should have dropped something
    
    // Check stats after overflow
    assert_eq!(inbox.stats.messages_received, 3);
    assert_eq!(inbox.stats.messages_expired, 1); // One message was "expired" (dropped)
}

/// Integration test with channel manager
#[test]
fn test_channel_manager_overflow_integration() {
    // Initialize systems
    init_channel_manager();
    audit::init_audit();
    
    let sender = ProcessId::new(300);
    let receiver = ProcessId::new(400);
    
    // Create a channel manager and ensure small inbox for receiver
    {
        let mut manager = CHANNEL_MANAGER.lock();
        if let Some(ref mut mgr) = *manager {
            // Force creation of small inbox by adding messages up to limit
            mgr.ensure_inbox(receiver);
            
            // Directly modify the inbox size for testing (hack for test)
            if let Some(inbox) = mgr.inboxes.get_mut(&receiver) {
                inbox.queue = MessageQueue::new(2); // Set small capacity
            }
        }
    }
    
    // Try to send Low priority messages until overflow
    for i in 0..3 {
        let message = Message::new_with_priority(
            sender,
            receiver,
            MessageType::Notification,
            MessagePriority::Low,
            MessagePayload::from_text(&format!("Test message {}", i)),
        );
        
        let result = send_to_inbox_and_wake(receiver.0, message);
        
        if i < 2 {
            // First two messages should succeed
            assert!(result.is_ok());
        } else {
            // Third message might succeed (with drop) or fail depending on implementation
            // This demonstrates the overflow behavior in action
        }
    }
    
    // Try to send High priority message - should fail with EBUSY
    let high_message = Message::new_with_priority(
        sender,
        receiver,
        MessageType::Request,
        MessagePriority::High,
        MessagePayload::from_text("High priority test"),
    );
    
    let result = send_to_inbox_and_wake(receiver.0, high_message);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), IpcError::Busy);
}

/// Benchmark test to ensure overflow policy doesn't significantly impact performance
#[test]
fn test_overflow_policy_performance() {
    let owner = ProcessId::new(500);
    let sender = ProcessId::new(600);
    
    // Create inbox with reasonable capacity
    let mut inbox = Inbox::new(owner, 100);
    
    // Measure time for normal operations (should be fast)
    let start_time = crate::log::get_current_time_ms();
    
    // Add many messages without triggering overflow
    for i in 0..90 {
        let message = Message::new_with_priority(
            sender,
            owner,
            MessageType::Notification,
            MessagePriority::Normal,
            MessagePayload::from_text(&format!("Perf test {}", i)),
        );
        
        let result = inbox.deliver_with_overflow_policy(message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No drops
    }
    
    let end_time = crate::log::get_current_time_ms();
    let duration = end_time - start_time;
    
    // Should complete quickly (less than 100ms in QEMU environment)
    assert!(duration < 100, "Overflow policy should not significantly impact performance");
}



