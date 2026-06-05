/// Inter-Process Communication (IPC) Module for Polymera OS
/// 
/// This module provides comprehensive IPC functionality including message types,
/// queues, and system call interfaces for secure inter-process communication.

pub mod types;
pub mod header;
pub mod auth;
pub mod pqc_helpers;
pub mod queues;
pub mod sys;
pub mod stream;
pub mod shmem;

#[cfg(test)]
pub mod pqc_ipc_tests;

use crate::{kprintln, klog};
use alloc::collections::VecDeque;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

// Re-export key types and functions
pub use types::*;
pub use queues::*;
pub use sys::*;

/// Global IPC statistics
#[derive(Debug, Clone, Copy)]
pub struct IpcStats {
    /// Total messages sent
    pub messages_sent: u64,
    
    /// Total messages received
    pub messages_received: u64,
    
    /// Messages currently in queues
    pub messages_queued: u64,
    
    /// Number of active channels
    pub active_channels: u64,
    
    /// Number of failed sends
    pub failed_sends: u64,
    
    /// Number of failed receives
    pub failed_receives: u64,
    
    /// Total bytes transferred
    pub bytes_transferred: u64,
}

impl IpcStats {
    pub const fn new() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            messages_queued: 0,
            active_channels: 0,
            failed_sends: 0,
            failed_receives: 0,
            bytes_transferred: 0,
        }
    }
}

impl Default for IpcStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Shortcut alias for the dashboard / external callers.
pub fn get_stats() -> IpcStats {
    get_ipc_stats()
}

/// Global IPC statistics
static IPC_STATS: Mutex<IpcStats> = Mutex::new(IpcStats::new());

/// Global message ID counter
static MESSAGE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate unique message ID
pub fn generate_message_id() -> MessageId {
    MessageId(MESSAGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Update IPC statistics
pub fn update_stats<F>(updater: F) 
where
    F: FnOnce(&mut IpcStats),
{
    let mut stats = IPC_STATS.lock();
    updater(&mut stats);
}

/// Get current IPC statistics
pub fn get_ipc_stats() -> IpcStats {
    *IPC_STATS.lock()
}

/// Print IPC statistics
pub fn print_ipc_stats() {
    let stats = get_ipc_stats();
    
    kprintln!("");
    kprintln!("=== IPC STATISTICS ===");
    kprintln!("Messages sent: {}", stats.messages_sent);
    kprintln!("Messages received: {}", stats.messages_received);
    kprintln!("Messages queued: {}", stats.messages_queued);
    kprintln!("Active channels: {}", stats.active_channels);
    kprintln!("Failed sends: {}", stats.failed_sends);
    kprintln!("Failed receives: {}", stats.failed_receives);
    kprintln!("Bytes transferred: {} KB", stats.bytes_transferred / 1024);
    kprintln!("=== END IPC STATISTICS ===");
    kprintln!("");
}

/// Initialize IPC subsystem
pub fn init_ipc() {
    kprintln!("[IPC] Initializing Inter-Process Communication subsystem");
    
    // Initialize the channel manager
    queues::init_channel_manager();
    
    // Initialize system call handlers
    sys::init_ipc_syscalls();
    
    klog!(INFO, "[IPC] IPC subsystem initialized successfully");
}

/// Test IPC functionality
pub fn test_ipc() {
    kprintln!("");
    kprintln!("=== IPC FUNCTIONALITY TEST ===");
    
    // Test basic message creation
    let sender_id = ProcessId(1);
    let receiver_id = ProcessId(2);
    let payload = MessagePayload::Data(b"Hello, IPC!".to_vec());
    
    let message = Message::new(
        sender_id,
        receiver_id,
        MessageType::Request,
        payload,
    );
    
    kprintln!("  ✓ Created message: ID={}, Type={:?}, Size={} bytes", 
              message.header.id.0, message.header.msg_type, message.header.payload_size);
    
    // Test channel creation and messaging
    match queues::create_channel(sender_id, receiver_id, 10) {
        Ok(channel_id) => {
            kprintln!("  ✓ Created channel: {}", channel_id.0);
            
            // Test sending message
            match queues::send_message(channel_id, message.clone()) {
                Ok(()) => {
                    kprintln!("  ✓ Message sent successfully");
                    
                    // Test receiving message
                    match queues::receive_message(channel_id) {
                        Ok(received_msg) => {
                            kprintln!("  ✓ Message received: ID={}", received_msg.header.id.0);
                            
                            // Verify message content
                            if received_msg.header.id == message.header.id {
                                kprintln!("  ✓ Message content verified");
                            } else {
                                kprintln!("  ✗ Message content mismatch");
                            }
                        }
                        Err(e) => kprintln!("  ✗ Failed to receive message: {:?}", e),
                    }
                }
                Err(e) => kprintln!("  ✗ Failed to send message: {:?}", e),
            }
            
            // Clean up
            let _ = queues::destroy_channel(channel_id);
        }
        Err(e) => kprintln!("  ✗ Failed to create channel: {:?}", e),
    }
    
    // Print statistics
    print_ipc_stats();
    
    // Test capability-enforced sending
    kprintln!("");
    kprintln!("Testing capability-enforced sending:");
    
    // Grant capability for process 1 to send to process 2
    match crate::security::cap::grant_capability(1, 2, 
        crate::security::cap::scope::SEND, 5000) {
        Ok(token) => {
            kprintln!("  ✓ Granted capability: {}", token);
            
            // Test authorized send
            let auth_message = Message::new(
                ProcessId(1),
                ProcessId(2),
                MessageType::Notification,
                MessagePayload::from_text("Authorized message"),
            );
            
            match sys::sys_send(2, &auth_message) {
                Ok(()) => kprintln!("  ✓ Authorized send succeeded"),
                Err(e) => kprintln!("  ✗ Authorized send failed: {}", e),
            }
        }
        Err(e) => kprintln!("  ✗ Failed to grant capability: {}", e),
    }
    
    // Test unauthorized send (no capability)
    let unauth_message = Message::new(
        ProcessId(3),
        ProcessId(4),
        MessageType::Notification,
        MessagePayload::from_text("Unauthorized message"),
    );
    
    match sys::sys_send(4, &unauth_message) {
        Ok(()) => kprintln!("  ✗ Unauthorized send should have failed"),
        Err(e) => kprintln!("  ✓ Unauthorized send correctly failed with error: {}", e),
    }
    
    // Test blocking receive and wakeup functionality
    kprintln!("");
    kprintln!("Testing blocking receive and priority-based preemption:");
    
    // Test non-blocking receive on empty inbox (should return EAGAIN)
    match sys::sys_recv(false) {
        Ok(_) => kprintln!("  ✗ Non-blocking recv should have failed on empty inbox"),
        Err(e) => kprintln!("  ✓ Non-blocking recv correctly failed: {}", e),
    }
    
    // Grant capability for process 10 to send to process 20
    match crate::security::cap::grant_capability(10, 20, 
        crate::security::cap::scope::SEND, 5000) {
        Ok(token) => {
            kprintln!("  ✓ Granted capability for process 10 -> 20: {}", token);
            
            // Create a high priority message
            let high_prio_message = Message::new_with_priority(
                ProcessId(10),
                ProcessId(20),
                MessageType::Notification,
                MessagePriority::High,
                MessagePayload::from_text("High priority message"),
            );
            
            // Send high priority message (should trigger preemption logic)
            match sys::sys_send(20, &high_prio_message) {
                Ok(()) => kprintln!("  ✓ High priority message sent successfully"),
                Err(e) => kprintln!("  ✗ High priority message send failed: {}", e),
            }
            
            // Create a normal priority message
            let normal_prio_message = Message::new_with_priority(
                ProcessId(10),
                ProcessId(20),
                MessageType::Request,
                MessagePriority::Normal,
                MessagePayload::from_text("Normal priority message"),
            );
            
            // Send normal priority message
            match sys::sys_send(20, &normal_prio_message) {
                Ok(()) => kprintln!("  ✓ Normal priority message sent successfully"),
                Err(e) => kprintln!("  ✗ Normal priority message send failed: {}", e),
            }
        }
        Err(e) => kprintln!("  ✗ Failed to grant capability: {}", e),
    }
    
    kprintln!("=== IPC FUNCTIONALITY TEST COMPLETE ===");
    kprintln!("");
}

/// IPC error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    /// Channel not found
    ChannelNotFound,
    
    /// Channel is full
    ChannelFull,
    
    /// Channel is empty
    ChannelEmpty,
    
    /// Invalid process ID
    InvalidProcessId,
    
    /// Permission denied
    PermissionDenied,
    
    /// Invalid message
    InvalidMessage,
    
    /// Message too large
    MessageTooLarge,
    
    /// Channel already exists
    ChannelExists,
    
    /// Out of memory
    OutOfMemory,
    
    /// Operation timeout
    Timeout,
    
    /// Resource busy (for High/RT priority messages when inbox is full)
    Busy,
}

impl core::fmt::Display for IpcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IpcError::ChannelNotFound => write!(f, "Channel not found"),
            IpcError::ChannelFull => write!(f, "Channel is full"),
            IpcError::ChannelEmpty => write!(f, "Channel is empty"),
            IpcError::InvalidProcessId => write!(f, "Invalid process ID"),
            IpcError::PermissionDenied => write!(f, "Permission denied"),
            IpcError::InvalidMessage => write!(f, "Invalid message"),
            IpcError::MessageTooLarge => write!(f, "Message too large"),
            IpcError::ChannelExists => write!(f, "Channel already exists"),
            IpcError::OutOfMemory => write!(f, "Out of memory"),
            IpcError::Timeout => write!(f, "Operation timeout"),
            IpcError::Busy => write!(f, "Resource busy"),
        }
    }
}

/// IPC result type
pub type IpcResult<T> = Result<T, IpcError>;

/// Constants
pub const MAX_MESSAGE_SIZE: usize = 4096;  // 4KB max message size
pub const MAX_CHANNELS: usize = 1024;      // Maximum number of channels
pub const DEFAULT_QUEUE_SIZE: usize = 32;  // Default queue size
