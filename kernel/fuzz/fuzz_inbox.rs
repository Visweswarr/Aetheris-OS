#![no_main]
use libfuzzer_sys::fuzz_target;
use polymera_os_kernel::ipc::{Message, MessageHeader, MessagePayload, MessagePriority, MessageType, MessageFlags, ProcessId, MessageId};
use polymera_os_kernel::ipc::queues::Inbox;
use polymera_os_kernel::secman::audit;
use alloc::vec::Vec;
use core::mem;

// Fuzzing target for inbox operations
// Tests push/pop with random sizes and message properties
fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return; // Need minimum data for meaningful fuzzing
    }
    
    // Initialize kernel components for fuzzing
    if let Err(_) = polymera_os_kernel::init_fuzzing_environment() {
        return; // Skip if initialization fails
    }
    
    // Parse fuzz input to determine operations
    let mut inbox = Inbox::new(ProcessId(1), 1000); // Large capacity for fuzzing
    
    // Extract operation parameters from fuzz data
    let mut offset = 0;
    
    // First byte: operation type (0=push, 1=pop, 2=mixed)
    let op_type = if data.len() > offset { data[offset] % 3 } else { 0 };
    offset += 1;
    
    // Second byte: number of operations (1-50)
    let num_ops = if data.len() > offset { 
        (data[offset] % 50) + 1 
    } else { 
        10 
    };
    offset += 1;
    
    // Third byte: message size range (1-1024 bytes)
    let max_msg_size = if data.len() > offset { 
        ((data[offset] as usize) % 1024) + 1 
    } else { 
        64 
    };
    offset += 1;
    
    // Fourth byte: priority distribution
    let priority_dist = if data.len() > offset { data[offset] } else { 0 };
    offset += 1;
    
    // Fifth byte: message type distribution
    let msg_type_dist = if data.len() > offset { data[offset] } else { 0 };
    offset += 1;
    
    // Sixth byte: flags distribution
    let flags_dist = if data.len() > offset { data[offset] } else { 0 };
    offset += 1;
    
    // Seventh byte: expiry time range (0-60 seconds)
    let max_expiry = if data.len() > offset { 
        ((data[offset] as u64) % 60) + 1 
    } else { 
        30 
    };
    offset += 1;
    
    // Eighth byte: sender/receiver ID range
    let max_id = if data.len() > offset { 
        ((data[offset] as u64) % 100) + 1 
    } else { 
        10 
    };
    offset += 1;
    
    let mut messages = Vec::new();
    let mut successful_pushes = 0;
    let mut successful_pops = 0;
    let mut overflow_events = 0;
    
    // Generate and process messages based on operation type
    match op_type {
        0 => { // Push-only operations
            for i in 0..num_ops {
                let msg = generate_random_message(
                    i, max_msg_size, priority_dist, msg_type_dist, 
                    flags_dist, max_expiry, max_id, &data, offset
                );
                
                match inbox.deliver_with_overflow_policy(msg.clone()) {
                    Ok(_) => {
                        successful_pushes += 1;
                        messages.push(msg);
                    }
                    Err(_) => {
                        overflow_events += 1;
                        // Log overflow event for audit
                        audit::log(audit::AuditEntry::new(
                            msg.header.sender.0,
                            105, // IPC_MSG_DROPPED
                            msg.header.message_id.0
                        ));
                    }
                }
                
                offset = (offset + 1) % (data.len().max(1));
            }
        }
        
        1 => { // Pop-only operations
            // First push some messages to have something to pop
            for i in 0..(num_ops / 2) {
                let msg = generate_random_message(
                    i, max_msg_size, priority_dist, msg_type_dist,
                    flags_dist, max_expiry, max_id, &data, offset
                );
                
                if let Ok(_) = inbox.deliver_with_overflow_policy(msg.clone()) {
                    messages.push(msg);
                }
                offset = (offset + 1) % (data.len().max(1));
            }
            
            // Now pop messages
            for _ in 0..num_ops {
                if let Some(msg) = inbox.try_pop() {
                    successful_pops += 1;
                    // Verify message integrity
                    assert!(msg.header.message_id.0 > 0, "Invalid message ID");
                    assert!(msg.payload.len() <= max_msg_size, "Message size exceeds limit");
                }
                offset = (offset + 1) % (data.len().max(1));
            }
        }
        
        2 => { // Mixed push/pop operations
            for i in 0..num_ops {
                if i % 3 == 0 { // Pop every 3rd operation
                    if let Some(msg) = inbox.try_pop() {
                        successful_pops += 1;
                        // Verify message integrity
                        assert!(msg.header.message_id.0 > 0, "Invalid message ID");
                        assert!(msg.payload.len() <= max_msg_size, "Message size exceeds limit");
                    }
                } else { // Push operation
                    let msg = generate_random_message(
                        i, max_msg_size, priority_dist, msg_type_dist,
                        flags_dist, max_expiry, max_id, &data, offset
                    );
                    
                    match inbox.deliver_with_overflow_policy(msg.clone()) {
                        Ok(_) => {
                            successful_pushes += 1;
                            messages.push(msg);
                        }
                        Err(_) => {
                            overflow_events += 1;
                            // Log overflow event for audit
                            audit::log(audit::AuditEntry::new(
                                msg.header.sender.0,
                                105, // IPC_MSG_DROPPED
                                msg.header.message_id.0
                            ));
                        }
                    }
                }
                
                offset = (offset + 1) % (data.len().max(1));
            }
        }
        
        _ => unreachable!()
    }
    
    // Verify inbox state consistency
    let remaining_count = inbox.len();
    let expected_remaining = successful_pushes - successful_pops;
    
    // Allow for some tolerance due to overflow handling
    assert!(remaining_count <= expected_remaining, 
            "Inbox has more messages than expected: {} vs {}", 
            remaining_count, expected_remaining);
    
    // Verify no messages were corrupted
    for msg in &messages {
        assert!(msg.header.message_id.0 > 0, "Corrupted message ID");
        assert!(msg.header.sender.0 > 0, "Corrupted sender ID");
        assert!(msg.header.receiver.0 > 0, "Corrupted receiver ID");
        assert!(msg.payload.len() <= max_msg_size, "Message size corrupted");
    }
    
    // Log fuzzing results for analysis
    audit::log(audit::AuditEntry::new(
        0, // System operation
        999, // FUZZ_TEST_COMPLETED
        (successful_pushes << 32) | successful_pops as u64
    ));
});

/// Generate a random message based on fuzz input parameters
fn generate_random_message(
    index: usize,
    max_size: usize,
    priority_dist: u8,
    msg_type_dist: u8,
    flags_dist: u8,
    max_expiry: u64,
    max_id: u64,
    data: &[u8],
    offset: usize
) -> Message {
    // Generate message size (1 to max_size)
    let msg_size = if data.len() > offset {
        ((data[offset] as usize) % max_size) + 1
    } else {
        1
    };
    
    // Generate priority based on distribution
    let priority = match priority_dist % 4 {
        0 => MessagePriority::Low,
        1 => MessagePriority::Normal,
        2 => MessagePriority::High,
        3 => MessagePriority::Critical,
        _ => MessagePriority::Normal,
    };
    
    // Generate message type based on distribution
    let msg_type = match msg_type_dist % 3 {
        0 => MessageType::Request,
        1 => MessageType::Response,
        2 => MessageType::Notification,
        _ => MessageType::Request,
    };
    
    // Generate flags based on distribution
    let mut flags = MessageFlags::empty();
    if (flags_dist & 0x01) != 0 { flags.insert(MessageFlags::URGENT); }
    if (flags_dist & 0x02) != 0 { flags.insert(MessageFlags::RELIABLE); }
    if (flags_dist & 0x04) != 0 { flags.insert(MessageFlags::BROADCAST); }
    
    // Generate sender and receiver IDs
    let sender_id = (index as u64 % max_id) + 1;
    let receiver_id = ((index + 1) as u64 % max_id) + 1;
    
    // Generate expiry time (current time + random offset)
    let current_time = polymera_os_kernel::log::get_current_time_ms();
    let expiry_offset = if data.len() > offset {
        ((data[offset] as u64) % max_expiry) * 1000 // Convert to milliseconds
    } else {
        5000 // Default 5 seconds
    };
    
    // Create message header
    let header = MessageHeader {
        message_id: MessageId(index as u64 + 1),
        sender: ProcessId(sender_id),
        receiver: ProcessId(receiver_id),
        priority,
        message_type: msg_type,
        sequence: index as u64,
        timestamp: current_time,
        expiry: current_time + expiry_offset,
        flags,
    };
    
    // Create message payload with random data
    let payload_data = if data.len() > offset + msg_size {
        &data[offset..offset + msg_size]
    } else {
        // Generate deterministic payload if not enough fuzz data
        let mut payload = Vec::new();
        for i in 0..msg_size {
            payload.push((index + i) as u8);
        }
        payload.as_slice()
    };
    
    let payload = MessagePayload::new(payload_data);
    
    Message { header, payload }
}

