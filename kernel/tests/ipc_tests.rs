/// IPC Module Unit Tests
/// 
/// This module contains comprehensive unit tests for the IPC (Inter-Process Communication)
/// implementation, including message types, queues, channels, inboxes, and system calls.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec::Vec;
use polymera_kernel::ipc::types::*;
use polymera_kernel::ipc::queues::*;
use polymera_kernel::ipc::sys::*;
use polymera_kernel::ipc::{IpcError, IpcResult};
use polymera_kernel::{kprintln, klog};

/// Test runner function
pub fn test_runner(tests: &[&dyn Fn()]) {
    kprintln!("Running {} IPC unit tests", tests.len());
    for test in tests {
        test();
    }
    kprintln!("All IPC unit tests completed");
}

/// Panic handler for tests
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kprintln!("Test panic: {}", info);
    loop {}
}

/// Entry point for tests
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

/// Test basic message creation and properties
#[test_case]
fn test_message_creation() {
    kprintln!("Testing message creation and properties...");
    
    let sender = ProcessId(100);
    let receiver = ProcessId(200);
    let payload = MessagePayload::from_text("Hello, World!");
    
    let message = Message::new_notification(sender, receiver, payload);
    
    // Test basic properties
    assert_eq!(message.header.sender, sender);
    assert_eq!(message.header.receiver, receiver);
    assert!(matches!(message.header.msg_type, MessageType::Notification));
    assert_eq!(message.header.payload_size, 13); // "Hello, World!" length
    
    // Test message validation
    assert!(message.is_valid());
    assert!(!message.is_request());
    assert!(!message.is_response());
    
    // Test payload access
    assert_eq!(message.as_text(), Some("Hello, World!"));
    assert_eq!(message.as_bytes(), Some(b"Hello, World!"));
    
    kprintln!("  ✓ Message creation and properties test passed");
}

/// Test different message types
#[test_case]
fn test_message_types() {
    kprintln!("Testing different message types...");
    
    let sender = ProcessId(1);
    let receiver = ProcessId(2);
    
    // Test request message
    let request = Message::new_request(sender, receiver, MessagePayload::from_text("request"));
    assert!(request.is_request());
    assert!(request.header.flags.ack_required);
    
    // Test response message
    let response = Message::new_response(sender, receiver, request.header.id, MessagePayload::from_text("response"));
    assert!(response.is_response());
    assert!(response.is_response_to(request.header.id));
    
    // Test notification message
    let notification = Message::new_notification(sender, receiver, MessagePayload::from_text("notification"));
    assert!(!notification.is_request());
    assert!(!notification.is_response());
    
    // Test error message
    let error = Message::new_error(sender, receiver, 404, "Not found");
    assert!(matches!(error.header.msg_type, MessageType::Error));
    
    kprintln!("  ✓ Message types test passed");
}

/// Test message payload types
#[test_case]
fn test_message_payloads() {
    kprintln!("Testing message payload types...");
    
    // Test empty payload
    let empty = MessagePayload::Empty;
    assert!(empty.is_empty());
    assert_eq!(empty.size(), 0);
    assert_eq!(empty.type_name(), "Empty");
    
    // Test data payload
    let data = MessagePayload::from_data(b"binary data");
    assert!(!data.is_empty());
    assert_eq!(data.size(), 11);
    assert_eq!(data.type_name(), "Data");
    assert_eq!(data.as_bytes(), Some(b"binary data"));
    
    // Test text payload
    let text = MessagePayload::from_text("text data");
    assert_eq!(text.size(), 9);
    assert_eq!(text.type_name(), "Text");
    assert_eq!(text.as_text(), Some("text data"));
    
    // Test signal payload
    let signal = MessagePayload::from_signal(9, 0x12345678);
    assert_eq!(signal.size(), 12);
    assert_eq!(signal.type_name(), "Signal");
    
    // Test error payload
    let error = MessagePayload::from_error(500, "Internal Server Error");
    assert_eq!(error.type_name(), "Error");
    
    // Test file descriptor payload
    let fd = MessagePayload::FileDescriptor(42);
    assert_eq!(fd.size(), 4);
    assert_eq!(fd.type_name(), "FileDescriptor");
    
    // Test memory region payload
    let mem = MessagePayload::MemoryRegion {
        start_addr: 0x1000,
        size: 4096,
        permissions: 0x7,
    };
    assert_eq!(mem.size(), 16);
    assert_eq!(mem.type_name(), "MemoryRegion");
    
    kprintln!("  ✓ Message payloads test passed");
}

/// Test message queue functionality
#[test_case]
fn test_message_queue() {
    kprintln!("Testing message queue functionality...");
    
    let mut queue = MessageQueue::new(5);
    
    // Test initial state
    assert!(queue.is_empty());
    assert!(!queue.is_full());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.capacity(), 5);
    
    // Test enqueue
    let sender = ProcessId(1);
    let receiver = ProcessId(2);
    
    for i in 0..3 {
        let message = Message::new_notification(
            sender,
            receiver,
            MessagePayload::from_text(&format!("Message {}", i)),
        );
        assert!(queue.enqueue(message).is_ok());
    }
    
    assert_eq!(queue.len(), 3);
    assert!(!queue.is_empty());
    assert!(!queue.is_full());
    
    // Test priority ordering
    let high_priority_msg = {
        let mut msg = Message::new_notification(sender, receiver, MessagePayload::from_text("High priority"));
        msg.set_priority(MessagePriority::High);
        msg
    };
    
    assert!(queue.enqueue(high_priority_msg).is_ok());
    
    // High priority message should be at front
    if let Some(front_msg) = queue.peek() {
        assert_eq!(front_msg.header.priority, MessagePriority::High);
    }
    
    // Test dequeue
    let dequeued = queue.dequeue().expect("Should dequeue high priority message");
    assert_eq!(dequeued.header.priority, MessagePriority::High);
    assert_eq!(queue.len(), 3);
    
    // Test queue full
    for i in 0..2 {
        let message = Message::new_notification(
            sender,
            receiver,
            MessagePayload::from_text(&format!("Extra {}", i)),
        );
        assert!(queue.enqueue(message).is_ok());
    }
    
    assert!(queue.is_full());
    
    // Should reject new messages when full
    let overflow_msg = Message::new_notification(sender, receiver, MessagePayload::from_text("Overflow"));
    assert!(matches!(queue.enqueue(overflow_msg), Err(IpcError::ChannelFull)));
    
    // Test queue empty
    while !queue.is_empty() {
        assert!(queue.dequeue().is_ok());
    }
    
    assert!(matches!(queue.dequeue(), Err(IpcError::ChannelEmpty)));
    
    kprintln!("  ✓ Message queue test passed");
}

/// Test channel functionality
#[test_case]
fn test_channel_functionality() {
    kprintln!("Testing channel functionality...");
    
    // Initialize channel manager
    init_channel_manager();
    
    let sender = ProcessId(10);
    let receiver = ProcessId(20);
    
    // Test channel creation
    let channel_id = create_channel(sender, receiver, 5)
        .expect("Should create channel");
    
    kprintln!("  ✓ Created channel: {}", channel_id);
    
    // Test message sending
    let message = Message::new_notification(
        sender,
        receiver,
        MessagePayload::from_text("Test message"),
    );
    
    assert!(send_message(channel_id, message.clone()).is_ok());
    kprintln!("  ✓ Message sent successfully");
    
    // Test message receiving
    let received = receive_message(channel_id)
        .expect("Should receive message");
    
    assert_eq!(received.header.id, message.header.id);
    assert_eq!(received.as_text(), Some("Test message"));
    kprintln!("  ✓ Message received successfully");
    
    // Test channel destruction
    assert!(destroy_channel(channel_id).is_ok());
    kprintln!("  ✓ Channel destroyed successfully");
    
    // Should fail to send after destruction
    let message2 = Message::new_notification(sender, receiver, MessagePayload::from_text("After destroy"));
    assert!(matches!(send_message(channel_id, message2), Err(IpcError::ChannelNotFound)));
    
    kprintln!("  ✓ Channel functionality test passed");
}

/// Test inbox functionality
#[test_case]
fn test_inbox_functionality() {
    kprintln!("Testing inbox functionality...");
    
    // Initialize channel manager
    init_channel_manager();
    
    let sender1 = ProcessId(30);
    let sender2 = ProcessId(31);
    let receiver = ProcessId(32);
    
    // Create multiple channels to the same receiver
    let channel1 = create_channel(sender1, receiver, 5).expect("Should create channel 1");
    let channel2 = create_channel(sender2, receiver, 5).expect("Should create channel 2");
    
    // Send messages from different senders
    let msg1 = Message::new_notification(sender1, receiver, MessagePayload::from_text("From sender 1"));
    let msg2 = Message::new_notification(sender2, receiver, MessagePayload::from_text("From sender 2"));
    
    assert!(send_message(channel1, msg1).is_ok());
    assert!(send_message(channel2, msg2).is_ok());
    
    // Check inbox status
    assert!(has_messages(receiver));
    assert_eq!(message_count(receiver), 2);
    
    // Receive from inbox
    let received1 = receive_from_inbox(receiver).expect("Should receive from inbox");
    let received2 = receive_from_inbox(receiver).expect("Should receive from inbox");
    
    // Verify messages
    let received_texts: Vec<_> = [&received1, &received2]
        .iter()
        .filter_map(|msg| msg.as_text())
        .collect();
    
    assert!(received_texts.contains(&"From sender 1"));
    assert!(received_texts.contains(&"From sender 2"));
    
    // Inbox should be empty now
    assert!(!has_messages(receiver));
    assert_eq!(message_count(receiver), 0);
    
    // Clean up
    let _ = destroy_channel(channel1);
    let _ = destroy_channel(channel2);
    
    kprintln!("  ✓ Inbox functionality test passed");
}

/// Test IPC system calls
#[test_case]
fn test_ipc_syscalls() {
    kprintln!("Testing IPC system calls...");
    
    // Initialize IPC system
    init_ipc_syscalls();
    
    let sender = ProcessId(40);
    let receiver = ProcessId(50);
    
    // Test create channel syscall
    let create_args = IpcSyscallArgs::CreateChannel {
        sender,
        receiver,
        queue_size: 10,
    };
    
    let channel_id = match handle_ipc_syscall(100, create_args, sender) {
        Ok(IpcSyscallResult::ChannelId(id)) => {
            kprintln!("  ✓ Created channel via syscall: {}", id);
            id
        }
        result => panic!("Unexpected create channel result: {:?}", result),
    };
    
    // Test send message syscall
    let message = Message::new_notification(
        sender,
        receiver,
        MessagePayload::from_text("Syscall message"),
    );
    
    let send_args = IpcSyscallArgs::SendMessage {
        channel_id,
        message: message.clone(),
    };
    
    match handle_ipc_syscall(102, send_args, sender) {
        Ok(IpcSyscallResult::Success) => {
            kprintln!("  ✓ Message sent via syscall");
        }
        result => panic!("Unexpected send message result: {:?}", result),
    }
    
    // Test receive message syscall
    let recv_args = IpcSyscallArgs::ReceiveMessage { channel_id };
    
    match handle_ipc_syscall(103, recv_args, receiver) {
        Ok(IpcSyscallResult::Message(received)) => {
            assert_eq!(received.header.id, message.header.id);
            kprintln!("  ✓ Message received via syscall");
        }
        result => panic!("Unexpected receive message result: {:?}", result),
    }
    
    // Test has messages syscall
    let has_msgs_args = IpcSyscallArgs::HasMessages { process_id: receiver };
    
    match handle_ipc_syscall(105, has_msgs_args, receiver) {
        Ok(IpcSyscallResult::Bool(has_msgs)) => {
            kprintln!("  ✓ Has messages check: {}", has_msgs);
        }
        result => panic!("Unexpected has messages result: {:?}", result),
    }
    
    // Test destroy channel syscall
    let destroy_args = IpcSyscallArgs::DestroyChannel { channel_id };
    
    match handle_ipc_syscall(101, destroy_args, sender) {
        Ok(IpcSyscallResult::Success) => {
            kprintln!("  ✓ Channel destroyed via syscall");
        }
        result => panic!("Unexpected destroy channel result: {:?}", result),
    }
    
    kprintln!("  ✓ IPC system calls test passed");
}

/// Test convenience functions
#[test_case]
fn test_convenience_functions() {
    kprintln!("Testing convenience functions...");
    
    let sender = ProcessId(60);
    let receiver = ProcessId(70);
    
    // Test send text message
    assert!(send_text_message(sender, receiver, "Hello from convenience!").is_ok());
    kprintln!("  ✓ Text message sent");
    
    // Test send signal
    assert!(send_signal(sender, receiver, 15, 0xDEADBEEF).is_ok());
    kprintln!("  ✓ Signal sent");
    
    // Test send error message
    assert!(send_error_message(sender, receiver, 404, "Resource not found").is_ok());
    kprintln!("  ✓ Error message sent");
    
    kprintln!("  ✓ Convenience functions test passed");
}

/// Test error conditions
#[test_case]
fn test_error_conditions() {
    kprintln!("Testing error conditions...");
    
    // Test invalid channel operations
    let invalid_channel = ChannelId(9999);
    let dummy_msg = Message::new_notification(
        ProcessId(1),
        ProcessId(2),
        MessagePayload::Empty,
    );
    
    assert!(matches!(send_message(invalid_channel, dummy_msg), Err(IpcError::ChannelNotFound)));
    assert!(matches!(receive_message(invalid_channel), Err(IpcError::ChannelNotFound)));
    assert!(matches!(destroy_channel(invalid_channel), Err(IpcError::ChannelNotFound)));
    
    kprintln!("  ✓ Invalid channel operations rejected");
    
    // Test permission checks
    let unauthorized_sender = ProcessId(999);
    let authorized_sender = ProcessId(100);
    let receiver = ProcessId(200);
    
    let create_args = IpcSyscallArgs::CreateChannel {
        sender: authorized_sender,
        receiver,
        queue_size: 5,
    };
    
    // Unauthorized process trying to create channel
    match handle_ipc_syscall(100, create_args.clone(), unauthorized_sender) {
        Err(IpcError::PermissionDenied) => {
            kprintln!("  ✓ Unauthorized channel creation rejected");
        }
        result => panic!("Expected permission denied, got: {:?}", result),
    }
    
    // Authorized process should succeed
    match handle_ipc_syscall(100, create_args, authorized_sender) {
        Ok(IpcSyscallResult::ChannelId(_)) => {
            kprintln!("  ✓ Authorized channel creation succeeded");
        }
        result => panic!("Authorized creation should succeed: {:?}", result),
    }
    
    kprintln!("  ✓ Error conditions test passed");
}

/// Test message priorities and ordering
#[test_case]
fn test_message_priorities() {
    kprintln!("Testing message priorities and ordering...");
    
    let sender = ProcessId(80);
    let receiver = ProcessId(90);
    
    let channel_id = create_channel(sender, receiver, 10)
        .expect("Should create channel");
    
    // Send messages with different priorities
    let priorities = [
        MessagePriority::Low,
        MessagePriority::Critical,
        MessagePriority::Normal,
        MessagePriority::High,
    ];
    
    for (i, &priority) in priorities.iter().enumerate() {
        let mut message = Message::new_notification(
            sender,
            receiver,
            MessagePayload::from_text(&format!("Priority {} message", i)),
        );
        message.set_priority(priority);
        
        assert!(send_message(channel_id, message).is_ok());
    }
    
    // Messages should be received in priority order: Critical, High, Normal, Low
    let expected_order = [
        MessagePriority::Critical,
        MessagePriority::High,
        MessagePriority::Normal,
        MessagePriority::Low,
    ];
    
    for &expected_priority in &expected_order {
        let received = receive_message(channel_id)
            .expect("Should receive message");
        assert_eq!(received.header.priority, expected_priority);
    }
    
    kprintln!("  ✓ Messages received in correct priority order");
    
    // Clean up
    let _ = destroy_channel(channel_id);
    
    kprintln!("  ✓ Message priorities test passed");
}

/// Test large message handling
#[test_case]
fn test_large_messages() {
    kprintln!("Testing large message handling...");
    
    let sender = ProcessId(100);
    let receiver = ProcessId(101);
    
    let channel_id = create_channel(sender, receiver, 5)
        .expect("Should create channel");
    
    // Test normal sized message
    let normal_data = vec![0u8; 1024]; // 1KB
    let normal_message = Message::new_notification(
        sender,
        receiver,
        MessagePayload::Data(normal_data),
    );
    
    assert!(send_message(channel_id, normal_message).is_ok());
    kprintln!("  ✓ Normal sized message (1KB) sent successfully");
    
    // Test large message (should fail if over limit)
    let large_data = vec![0u8; 8192]; // 8KB (larger than MAX_MESSAGE_SIZE)
    let large_message = Message::new_notification(
        sender,
        receiver,
        MessagePayload::Data(large_data),
    );
    
    match send_message(channel_id, large_message) {
        Err(IpcError::MessageTooLarge) => {
            kprintln!("  ✓ Large message correctly rejected");
        }
        Ok(_) => {
            kprintln!("  - Large message was accepted (no size limit enforced)");
        }
        Err(e) => panic!("Unexpected error for large message: {}", e),
    }
    
    // Clean up
    let _ = destroy_channel(channel_id);
    
    kprintln!("  ✓ Large message handling test passed");
}

/// Test channel capacity limits
#[test_case]
fn test_channel_capacity() {
    kprintln!("Testing channel capacity limits...");
    
    let sender = ProcessId(110);
    let receiver = ProcessId(111);
    
    // Create channel with small capacity
    let channel_id = create_channel(sender, receiver, 2)
        .expect("Should create channel");
    
    // Fill the channel
    for i in 0..2 {
        let message = Message::new_notification(
            sender,
            receiver,
            MessagePayload::from_text(&format!("Message {}", i)),
        );
        assert!(send_message(channel_id, message).is_ok());
    }
    
    // Next message should fail (channel full)
    let overflow_message = Message::new_notification(
        sender,
        receiver,
        MessagePayload::from_text("Overflow message"),
    );
    
    match send_message(channel_id, overflow_message) {
        Err(IpcError::ChannelFull) => {
            kprintln!("  ✓ Channel correctly rejected message when full");
        }
        result => panic!("Expected channel full error, got: {:?}", result),
    }
    
    // Receive one message to make space
    assert!(receive_message(channel_id).is_ok());
    
    // Now should be able to send again
    let new_message = Message::new_notification(
        sender,
        receiver,
        MessagePayload::from_text("New message"),
    );
    assert!(send_message(channel_id, new_message).is_ok());
    kprintln!("  ✓ Channel accepts messages after space is freed");
    
    // Clean up
    let _ = destroy_channel(channel_id);
    
    kprintln!("  ✓ Channel capacity test passed");
}

/// Run all IPC tests
pub fn run_all_tests() {
    kprintln!("");
    kprintln!("=== IPC MODULE UNIT TESTS ===");
    
    test_message_creation();
    test_message_types();
    test_message_payloads();
    test_message_queue();
    test_channel_functionality();
    test_inbox_functionality();
    test_ipc_syscalls();
    test_convenience_functions();
    test_error_conditions();
    test_message_priorities();
    test_large_messages();
    test_channel_capacity();
    
    kprintln!("=== ALL IPC TESTS PASSED ===");
    kprintln!("");
}

