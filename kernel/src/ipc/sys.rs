/// IPC System Call Interface
/// 
/// This module provides system call implementations for IPC operations,
/// including channel management, message sending/receiving, and inbox operations.

use super::types::{Message, MessageHeader, ProcessId, ChannelId, MessagePayload};
use super::queues::*;
use super::{IpcError, IpcResult};
use crate::{kprintln, klog, format};
use alloc::string::ToString;
use alloc::vec::Vec;

fn payload_size_u32(size: usize) -> u32 {
    if size > u32::MAX as usize {
        u32::MAX
    } else {
        size as u32
    }
}

fn evaluate_ipc_policy_or_deny(
    context: &crate::secman::policy::IpcPolicyContext,
) -> crate::secman::policy::PolicyResult {
    crate::secman::policy::evaluate_ipc_policy(context).unwrap_or_else(|| {
        crate::secman::policy::PolicyResult {
            allowed: false,
            decision: crate::secman::policy::POLICY_DECISION_DENY,
            reason: "Policy manager not initialized".to_string(),
            audit: true,
            metadata: Vec::new(),
        }
    })
}

/// IPC system call numbers
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcSyscall {
    /// Create a new communication channel
    CreateChannel = 100,
    
    /// Destroy a communication channel
    DestroyChannel = 101,
    
    /// Send a message through a channel
    SendMessage = 102,
    
    /// Receive a message from a channel
    ReceiveMessage = 103,
    
    /// Receive a message from process inbox
    ReceiveFromInbox = 104,
    
    /// Check if process has pending messages
    HasMessages = 105,
    
    /// Get number of pending messages
    MessageCount = 106,
    
    /// List channels for a process
    ListChannels = 107,
    
    /// Get channel information
    GetChannelInfo = 108,
    
    /// Set channel permissions
    SetChannelPermissions = 109,
    
    /// Cleanup expired messages
    CleanupExpired = 110,
    
    /// Get IPC statistics
    GetIpcStats = 111,
}

impl IpcSyscall {
    /// Convert from raw syscall number
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            100 => Some(IpcSyscall::CreateChannel),
            101 => Some(IpcSyscall::DestroyChannel),
            102 => Some(IpcSyscall::SendMessage),
            103 => Some(IpcSyscall::ReceiveMessage),
            104 => Some(IpcSyscall::ReceiveFromInbox),
            105 => Some(IpcSyscall::HasMessages),
            106 => Some(IpcSyscall::MessageCount),
            107 => Some(IpcSyscall::ListChannels),
            108 => Some(IpcSyscall::GetChannelInfo),
            109 => Some(IpcSyscall::SetChannelPermissions),
            110 => Some(IpcSyscall::CleanupExpired),
            111 => Some(IpcSyscall::GetIpcStats),
            _ => None,
        }
    }
}

/// System call arguments for IPC operations
#[derive(Debug, Clone)]
pub enum IpcSyscallArgs {
    /// Create channel arguments
    CreateChannel {
        sender: ProcessId,
        receiver: ProcessId,
        queue_size: usize,
    },
    
    /// Destroy channel arguments
    DestroyChannel {
        channel_id: ChannelId,
    },
    
    /// Send message arguments
    SendMessage {
        channel_id: ChannelId,
        message: Message,
    },
    
    /// Receive message arguments
    ReceiveMessage {
        channel_id: ChannelId,
    },
    
    /// Receive from inbox arguments
    ReceiveFromInbox {
        process_id: ProcessId,
    },
    
    /// Has messages arguments
    HasMessages {
        process_id: ProcessId,
    },
    
    /// Message count arguments
    MessageCount {
        process_id: ProcessId,
    },
    
    /// List channels arguments
    ListChannels {
        process_id: ProcessId,
    },
    
    /// Get channel info arguments
    GetChannelInfo {
        channel_id: ChannelId,
    },
    
    /// Set channel permissions arguments
    SetChannelPermissions {
        channel_id: ChannelId,
        permissions: ChannelPermissions,
    },
    
    /// No arguments needed
    NoArgs,
}

/// System call result types
#[derive(Debug, Clone)]
pub enum IpcSyscallResult {
    /// Channel ID result
    ChannelId(ChannelId),
    
    /// Message result
    Message(Message),
    
    /// Boolean result
    Bool(bool),
    
    /// Number result
    Number(u64),
    
    /// Channel list result
    ChannelList(Vec<ChannelId>),
    
    /// Channel info result
    ChannelInfo {
        id: ChannelId,
        sender: ProcessId,
        receiver: ProcessId,
        state: ChannelState,
        queue_stats: QueueStats,
    },
    
    /// IPC statistics result
    IpcStats(super::IpcStats),
    
    /// Success with no return value
    Success,
}

/// IPC system call handler
pub struct IpcSyscallHandler {
    /// Handler statistics
    calls_handled: u64,
    calls_failed: u64,
}

impl IpcSyscallHandler {
    /// Create a new IPC syscall handler
    pub fn new() -> Self {
        Self {
            calls_handled: 0,
            calls_failed: 0,
        }
    }
    
    /// Handle an IPC system call
    pub fn handle_syscall(
        &mut self,
        syscall: IpcSyscall,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        klog!(TRACE, "[IPC] Handling syscall {:?} from {}", syscall, caller_pid);
        
        let result = match syscall {
            IpcSyscall::CreateChannel => self.handle_create_channel(args, caller_pid),
            IpcSyscall::DestroyChannel => self.handle_destroy_channel(args, caller_pid),
            IpcSyscall::SendMessage => self.handle_send_message(args, caller_pid),
            IpcSyscall::ReceiveMessage => self.handle_receive_message(args, caller_pid),
            IpcSyscall::ReceiveFromInbox => self.handle_receive_from_inbox(args, caller_pid),
            IpcSyscall::HasMessages => self.handle_has_messages(args, caller_pid),
            IpcSyscall::MessageCount => self.handle_message_count(args, caller_pid),
            IpcSyscall::ListChannels => self.handle_list_channels(args, caller_pid),
            IpcSyscall::GetChannelInfo => self.handle_get_channel_info(args, caller_pid),
            IpcSyscall::SetChannelPermissions => self.handle_set_channel_permissions(args, caller_pid),
            IpcSyscall::CleanupExpired => self.handle_cleanup_expired(args, caller_pid),
            IpcSyscall::GetIpcStats => self.handle_get_ipc_stats(args, caller_pid),
        };
        
        match &result {
            Ok(_) => {
                self.calls_handled += 1;
                klog!(TRACE, "[IPC] Syscall {:?} completed successfully", syscall);
            }
            Err(e) => {
                self.calls_failed += 1;
                klog!(WARN, "[IPC] Syscall {:?} failed: {}", syscall, e);
            }
        }
        
        result
    }
    
    /// Handle create channel syscall
    fn handle_create_channel(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::CreateChannel { sender, receiver, queue_size } = args {
            // Validate that caller is authorized to create channel
            if sender != caller_pid && caller_pid != ProcessId::KERNEL {
                return Err(IpcError::PermissionDenied);
            }
            
            let channel_id = create_channel(sender, receiver, queue_size)?;
            Ok(IpcSyscallResult::ChannelId(channel_id))
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle destroy channel syscall
    fn handle_destroy_channel(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::DestroyChannel { channel_id } = args {
            // TODO: Add authorization check - only channel participants can destroy
            destroy_channel(channel_id)?;
            Ok(IpcSyscallResult::Success)
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle send message syscall
    fn handle_send_message(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::SendMessage { channel_id, mut message } = args {
            // Validate that caller is authorized to send
            if message.header.sender != caller_pid && caller_pid != ProcessId::KERNEL {
                return Err(IpcError::PermissionDenied);
            }
            
            // Update message timestamp and checksum
            message.header.timestamp = get_current_timestamp();
            message.header.update_checksum();
            
            send_message(channel_id, message)?;
            Ok(IpcSyscallResult::Success)
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle receive message syscall
    fn handle_receive_message(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::ReceiveMessage { channel_id } = args {
            // TODO: Add authorization check - only channel participants can receive
            let message = receive_message(channel_id)?;
            Ok(IpcSyscallResult::Message(message))
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle receive from inbox syscall
    fn handle_receive_from_inbox(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::ReceiveFromInbox { process_id } = args {
            // Only allow receiving from own inbox
            if process_id != caller_pid && caller_pid != ProcessId::KERNEL {
                return Err(IpcError::PermissionDenied);
            }
            
            let message = receive_from_inbox(process_id)?;
            Ok(IpcSyscallResult::Message(message))
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle has messages syscall
    fn handle_has_messages(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::HasMessages { process_id } = args {
            // Only allow checking own messages
            if process_id != caller_pid && caller_pid != ProcessId::KERNEL {
                return Err(IpcError::PermissionDenied);
            }
            
            let has_msgs = has_messages(process_id);
            Ok(IpcSyscallResult::Bool(has_msgs))
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle message count syscall
    fn handle_message_count(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::MessageCount { process_id } = args {
            // Only allow checking own message count
            if process_id != caller_pid && caller_pid != ProcessId::KERNEL {
                return Err(IpcError::PermissionDenied);
            }
            
            let count = message_count(process_id);
            Ok(IpcSyscallResult::Number(count as u64))
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle list channels syscall
    fn handle_list_channels(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::ListChannels { process_id } = args {
            // Only allow listing own channels
            if process_id != caller_pid && caller_pid != ProcessId::KERNEL {
                return Err(IpcError::PermissionDenied);
            }
            
            let channels = list_channels(process_id);
            Ok(IpcSyscallResult::ChannelList(channels))
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle get channel info syscall
    fn handle_get_channel_info(
        &mut self,
        args: IpcSyscallArgs,
        _caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::GetChannelInfo { channel_id } = args {
            // TODO: Add authorization check based on channel participants
            let manager = super::queues::CHANNEL_MANAGER.lock();
            if let Some(ref mgr) = *manager {
                if let Some(channel) = mgr.get_channel(channel_id) {
                    let info = IpcSyscallResult::ChannelInfo {
                        id: channel.id,
                        sender: channel.sender,
                        receiver: channel.receiver,
                        state: channel.state,
                        queue_stats: channel.queue.stats(),
                    };
                    Ok(info)
                } else {
                    Err(IpcError::ChannelNotFound)
                }
            } else {
                Err(IpcError::ChannelNotFound)
            }
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle set channel permissions syscall
    fn handle_set_channel_permissions(
        &mut self,
        args: IpcSyscallArgs,
        caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        if let IpcSyscallArgs::SetChannelPermissions { channel_id: _, permissions: _ } = args {
            // TODO: Implement channel permission modification
            // For now, only kernel can modify permissions
            if caller_pid != ProcessId::KERNEL {
                return Err(IpcError::PermissionDenied);
            }
            
            // TODO: Implement actual permission setting
            Ok(IpcSyscallResult::Success)
        } else {
            Err(IpcError::InvalidMessage)
        }
    }
    
    /// Handle cleanup expired syscall
    fn handle_cleanup_expired(
        &mut self,
        _args: IpcSyscallArgs,
        _caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        let cleaned = cleanup_expired();
        Ok(IpcSyscallResult::Number(cleaned as u64))
    }
    
    /// Handle get IPC stats syscall
    fn handle_get_ipc_stats(
        &mut self,
        _args: IpcSyscallArgs,
        _caller_pid: ProcessId,
    ) -> IpcResult<IpcSyscallResult> {
        let stats = super::get_ipc_stats();
        Ok(IpcSyscallResult::IpcStats(stats))
    }
    
    /// Get handler statistics
    pub fn stats(&self) -> (u64, u64) {
        (self.calls_handled, self.calls_failed)
    }
}

/// Global IPC syscall handler
static mut IPC_HANDLER: Option<IpcSyscallHandler> = None;

/// Initialize IPC system call handlers
pub fn init_ipc_syscalls() {
    unsafe {
        IPC_HANDLER = Some(IpcSyscallHandler::new());
    }
    
    klog!(INFO, "[IPC] System call handlers initialized");
}

/// Handle an IPC system call (entry point from kernel syscall dispatcher)
pub fn handle_ipc_syscall(
    syscall_num: u64,
    args: IpcSyscallArgs,
    caller_pid: ProcessId,
) -> IpcResult<IpcSyscallResult> {
    let syscall = IpcSyscall::from_u64(syscall_num)
        .ok_or(IpcError::InvalidMessage)?;
    
    unsafe {
        if let Some(ref mut handler) = IPC_HANDLER {
            handler.handle_syscall(syscall, args, caller_pid)
        } else {
            Err(IpcError::OutOfMemory)
        }
    }
}

/// Convenience functions for common IPC operations

/// Send a text message to a process
pub fn send_text_message(
    sender: ProcessId,
    receiver: ProcessId,
    text: &str,
) -> IpcResult<()> {
    // Create a temporary channel for one-time communication
    let channel_id = create_channel(sender, receiver, 1)?;
    
    let message = Message::new_notification(
        sender,
        receiver,
        MessagePayload::from_text(text),
    );
    
    let result = send_message(channel_id, message);
    
    // Clean up the temporary channel
    let _ = destroy_channel(channel_id);
    
    result
}

/// Send a signal to a process
pub fn send_signal(
    sender: ProcessId,
    receiver: ProcessId,
    signal: u32,
    data: u64,
) -> IpcResult<()> {
    let channel_id = create_channel(sender, receiver, 1)?;
    
    let message = Message::new_notification(
        sender,
        receiver,
        MessagePayload::from_signal(signal, data),
    );
    
    let result = send_message(channel_id, message);
    let _ = destroy_channel(channel_id);
    
    result
}

/// Send an error message to a process
pub fn send_error_message(
    sender: ProcessId,
    receiver: ProcessId,
    error_code: u32,
    error_text: &str,
) -> IpcResult<()> {
    let channel_id = create_channel(sender, receiver, 1)?;
    
    let message = Message::new_error(sender, receiver, error_code, error_text);
    
    let result = send_message(channel_id, message);
    let _ = destroy_channel(channel_id);
    
    result
}

/// Request-response pattern helper
pub fn send_request_and_wait_response(
    sender: ProcessId,
    receiver: ProcessId,
    request_payload: MessagePayload,
    timeout_ms: u64,
) -> IpcResult<Message> {
    // Create a bi-directional channel
    let request_channel = create_channel(sender, receiver, 1)?;
    let response_channel = create_channel(receiver, sender, 1)?;
    
    // Send the request
    let request = Message::new_request(sender, receiver, request_payload);
    let request_id = request.header.id;
    
    send_message(request_channel, request)?;
    
    // Wait for response (simplified - in real implementation, use proper timeout)
    let mut attempts = 0;
    let max_attempts = timeout_ms / 10; // Poll every 10ms
    
    loop {
        if let Ok(response) = receive_message(response_channel) {
            if response.is_response_to(request_id) {
                // Clean up channels
                let _ = destroy_channel(request_channel);
                let _ = destroy_channel(response_channel);
                return Ok(response);
            }
        }
        
        attempts += 1;
        if attempts >= max_attempts {
            // Clean up channels
            let _ = destroy_channel(request_channel);
            let _ = destroy_channel(response_channel);
            return Err(IpcError::Timeout);
        }
        
        // Simple delay (in real implementation, use proper sleep/yield)
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
    }
}

/// Get current timestamp (simplified implementation)
fn get_current_timestamp() -> u64 {
    static TIMESTAMP_COUNTER: core::sync::atomic::AtomicU64 = 
        core::sync::atomic::AtomicU64::new(0);
    TIMESTAMP_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}

/// Test IPC system calls
pub fn test_ipc_syscalls() {
    kprintln!("");
    kprintln!("=== IPC SYSTEM CALLS TEST ===");
    
    let sender = ProcessId(10);
    let receiver = ProcessId(20);
    
    // Test channel creation
    let args = IpcSyscallArgs::CreateChannel {
        sender,
        receiver,
        queue_size: 5,
    };
    
    match handle_ipc_syscall(100, args, sender) {
        Ok(IpcSyscallResult::ChannelId(channel_id)) => {
            kprintln!("  ✓ Created channel: {}", channel_id);
            
            // Test message sending
            let message = Message::new_notification(
                sender,
                receiver,
                MessagePayload::from_text("Hello from syscall!"),
            );
            
            let send_args = IpcSyscallArgs::SendMessage {
                channel_id,
                message: message.clone(),
            };
            
            match handle_ipc_syscall(102, send_args, sender) {
                Ok(IpcSyscallResult::Success) => {
                    kprintln!("  ✓ Message sent successfully");
                    
                    // Test message receiving
                    let recv_args = IpcSyscallArgs::ReceiveMessage { channel_id };
                    
                    match handle_ipc_syscall(103, recv_args, receiver) {
                        Ok(IpcSyscallResult::Message(received_msg)) => {
                            kprintln!("  ✓ Message received: {}", received_msg.header.id);
                            
                            if received_msg.header.id == message.header.id {
                                kprintln!("  ✓ Message content verified");
                            } else {
                                kprintln!("  ✗ Message content mismatch");
                            }
                        }
                        Ok(_) => kprintln!("  ✗ Unexpected syscall result type"),
                        Err(e) => kprintln!("  ✗ Failed to receive message: {}", e),
                    }
                }
                Ok(_) => kprintln!("  ✗ Unexpected syscall result type"),
                Err(e) => kprintln!("  ✗ Failed to send message: {}", e),
            }
            
            // Test channel destruction
            let destroy_args = IpcSyscallArgs::DestroyChannel { channel_id };
            
            match handle_ipc_syscall(101, destroy_args, sender) {
                Ok(IpcSyscallResult::Success) => {
                    kprintln!("  ✓ Channel destroyed successfully");
                }
                Ok(_) => kprintln!("  ✗ Unexpected syscall result type"),
                Err(e) => kprintln!("  ✗ Failed to destroy channel: {}", e),
            }
        }
        Ok(_) => kprintln!("  ✗ Unexpected syscall result type"),
        Err(e) => kprintln!("  ✗ Failed to create channel: {}", e),
    }
    
    // Test convenience functions
    kprintln!("Testing convenience functions:");
    
    match send_text_message(sender, receiver, "Hello from convenience function!") {
        Ok(()) => kprintln!("  ✓ Text message sent successfully"),
        Err(e) => kprintln!("  ✗ Failed to send text message: {}", e),
    }
    
    match send_signal(sender, receiver, 9, 0x12345678) {
        Ok(()) => kprintln!("  ✓ Signal sent successfully"),
        Err(e) => kprintln!("  ✗ Failed to send signal: {}", e),
    }
    
    // Get handler statistics
    unsafe {
        if let Some(ref handler) = IPC_HANDLER {
            let (handled, failed) = handler.stats();
            kprintln!("  Handler stats: {} handled, {} failed", handled, failed);
        }
    }
    
    kprintln!("=== IPC SYSTEM CALLS TEST COMPLETE ===");
    kprintln!("");
}

//=============================================================================
// SIMPLIFIED IPC SYSCALLS (AS PER PROMPT 17)
//=============================================================================

/// Create a new communication channel and return capability ID
pub fn sys_channel_create(_peer: u64) -> Result<u128, i32> {
    // Return a dummy capability id for now; real caps in security module
    kprintln!("[ipc] sys_channel_create -> peer={}, returning dummy cap", _peer);
    Ok(0xC0DEC0DEu128)
}

/// Send a message to a destination process with PQC authentication
pub fn sys_send(dst: u64, msg: &Message) -> Result<(), i32> {
    // Get the sender's process ID (for now, use the sender from message header)
    let sender_pid = msg.header.sender.0;
    
    // Log audit entry
    crate::secman::audit::log(crate::secman::audit::AuditEntry::syscall_send(sender_pid, dst));
    
    // Look up the sender's capability token for (dst)
    match crate::security::cap::check_capability(sender_pid, dst) {
        Ok(token) => {
            // Convert to CapTokenV2 format for PQC authentication
            let cap_token_v2 = super::pqc_helpers::convert_to_cap_token_v2(&token);
            
            // Create IPC header v2 with authentication
            let header_v2 = super::pqc_helpers::create_ipc_header_v2(msg, &cap_token_v2);
            let payload_bytes = msg.payload.to_bytes();
            
            // Authenticate the message using PQC authentication
            let auth_result = super::auth::authenticate_ipc_message(
                &header_v2,
                &payload_bytes,
                &cap_token_v2,
            );
            
            // Evaluate IPC policy based on authentication result
            let policy_context = crate::secman::policy::IpcPolicyContext {
                sender_pid,
                destination_pid: dst,
                has_capability: true, // We have a capability token
                has_valid_mac: auth_result.as_ref().map(|r| r.authenticated && r.mac_valid.unwrap_or(false)).unwrap_or(false),
                message_size: msg.header.payload_size,
                priority: msg.header.priority.as_u8(),
                auth_mode: format!("{:?}", auth_result.as_ref().map(|r| r.auth_mode).unwrap_or(super::header::AuthMode::CapabilityOnly)),
                timestamp: 0, // TODO: Get actual timestamp
            };
            
            let policy_result = evaluate_ipc_policy_or_deny(&policy_context);
            
            // Check if policy allows the operation
            if !policy_result.allowed {
                kprintln!("[ipc] send -> {} POLICY DENIED: {}", dst, policy_result.reason);
                
                // Log policy denial audit entry
                let audit_entry = crate::secman::audit::AuditEntry::new(
                    sender_pid,
                    crate::secman::audit::ops::IPC_DENY_POLICY,
                    dst
                );
                crate::secman::audit::log(audit_entry);
                
                // Return EPERM for policy denial
                return Err(1); // EPERM
            }
            
            // If policy requires audit, log the decision
            if policy_result.audit {
                let audit_entry = crate::secman::audit::AuditEntry::new(
                    sender_pid,
                    crate::secman::audit::ops::IPC_POLICY_AUDIT,
                    dst
                );
                crate::secman::audit::log(audit_entry);
            }
            
            match auth_result {
                Some(auth_result) if auth_result.authenticated => {
                    kprintln!("[ipc] send -> {}, len={} (PQC authenticated, mode={:?})", 
                              dst, msg.header.payload_size, auth_result.auth_mode);
                    
                    // Trace the IPC operation
                    crate::trace::trace_ipc(sender_pid, dst, payload_size_u32(msg.header.payload_size), msg.header.priority);
                    
                    // Send message to destination inbox with wakeup and preemption handling
                    match super::queues::send_message_with_wakeup(sender_pid, dst, msg.clone()) {
                        Ok(()) => {
                            kprintln!("[ipc] Message successfully delivered to process {}", dst);
                            Ok(())
                        }
                        Err(ipc_error) => {
                            kprintln!("[ipc] Failed to deliver message to process {}: {:?}", dst, ipc_error);
                            Err(-28) // ENOSPC - No space left on device
                        }
                    }
                }
                Some(auth_result) => {
                    // Authentication failed
                    kprintln!("[ipc] send -> {} AUTH FAILED: {:?}", dst, auth_result.failure_reason);
                    
                    // Log appropriate audit entry based on failure reason
                    let audit_entry = match auth_result.failure_reason {
                        Some(super::auth::IpcAuthFailure::CapabilityFailed(_)) => {
                            crate::secman::audit::AuditEntry::new(
                                sender_pid,
                                crate::secman::audit::ops::IPC_DENY_CAP,
                                dst
                            )
                        }
                        Some(super::auth::IpcAuthFailure::MacFailed) => {
                            crate::secman::audit::AuditEntry::capability_rejected(
                                sender_pid,
                                u128::from_le_bytes(header_v2.cap_id.try_into().unwrap_or([0u8; 16])),
                                0x1001, // MAC_FAIL reason code
                            )
                        }
                        _ => {
                            crate::secman::audit::AuditEntry::new(
                                sender_pid,
                                crate::secman::audit::ops::SEC_CAP_REJECT,
                                dst
                            )
                        }
                    };
                    
                    crate::secman::audit::log(audit_entry);
                    
                    // Return EPERM for authentication failure
                    Err(1) // EPERM
                }
                None => {
                    // Authentication manager not available
                    kprintln!("[ipc] send -> {} AUTH MANAGER UNAVAILABLE", dst);
                    
                    // Fall back to legacy validation
                    match crate::security::validator::validate_for_ipc_send(&token, dst) {
                        Ok(()) => {
                            kprintln!("[ipc] send -> {}, len={} (legacy validated with token 0x{:x})", 
                                      dst, msg.header.payload_size, token.id);
                            
                            // Trace the IPC operation
                            crate::trace::trace_ipc(sender_pid, dst, payload_size_u32(msg.header.payload_size), msg.header.priority);
                            
                            // Send message to destination inbox
                            match super::queues::send_message_with_wakeup(sender_pid, dst, msg.clone()) {
                                Ok(()) => {
                                    kprintln!("[ipc] Message successfully delivered to process {}", dst);
                                    Ok(())
                                }
                                Err(ipc_error) => {
                                    kprintln!("[ipc] Failed to deliver message to process {}: {:?}", dst, ipc_error);
                                    Err(-28) // ENOSPC - No space left on device
                                }
                            }
                        }
                        Err(validation_error) => {
                            kprintln!("[ipc] send -> {} LEGACY VALIDATION FAILED: {:?}", dst, validation_error);
                            
                            // Log legacy validation failure audit entry
                            let audit_entry = crate::secman::audit::AuditEntry::new(
                                sender_pid,
                                crate::secman::audit::ops::IPC_DENY_LEGACY,
                                dst
                            );
                            crate::secman::audit::log(audit_entry);
                            
                            Err(1) // EPERM
                        }
                    }
                }
            }
        }
        Err(cap_error) => {
            // No capability token available
            kprintln!("[ipc] send -> {} NO CAPABILITY: {:?}", dst, cap_error);
            
            // Evaluate policy for operation without capability
            let policy_context = crate::secman::policy::IpcPolicyContext {
                sender_pid,
                destination_pid: dst,
                has_capability: false,
                has_valid_mac: false,
                message_size: msg.header.payload_size,
                priority: msg.header.priority.as_u8(),
                auth_mode: "none".to_string(),
                timestamp: 0, // TODO: Get actual timestamp
            };
            
            let policy_result = evaluate_ipc_policy_or_deny(&policy_context);
            
            // Check if policy allows the operation without capability
            if !policy_result.allowed {
                kprintln!("[ipc] send -> {} POLICY DENIED (NO CAP): {}", dst, policy_result.reason);
                
                // Log policy denial audit entry
                let audit_entry = crate::secman::audit::AuditEntry::new(
                    sender_pid,
                    crate::secman::audit::ops::IPC_DENY_POLICY,
                    dst
                );
                crate::secman::audit::log(audit_entry);
                
                // Return EPERM for policy denial
                return Err(1); // EPERM
            }
            
            // If policy allows but requires audit, log the decision
            if policy_result.audit {
                let audit_entry = crate::secman::audit::AuditEntry::new(
                    sender_pid,
                    crate::secman::audit::ops::IPC_POLICY_AUDIT,
                    dst
                );
                crate::secman::audit::log(audit_entry);
            }
            
            // Policy allows operation without capability (dev mode)
            kprintln!("[ipc] send -> {}, len={} (policy allowed, no capability)", 
                      dst, msg.header.payload_size);
            
            // Trace the IPC operation
            crate::trace::trace_ipc(sender_pid, dst, payload_size_u32(msg.header.payload_size), msg.header.priority);
            
            // Send message to destination inbox
            match super::queues::send_message_with_wakeup(sender_pid, dst, msg.clone()) {
                Ok(()) => {
                    kprintln!("[ipc] Message successfully delivered to process {} (policy allowed)", dst);
                    Ok(())
                }
                Err(ipc_error) => {
                    kprintln!("[ipc] Failed to deliver message to process {}: {:?}", dst, ipc_error);
                    Err(-28) // ENOSPC - No space left on device
                }
            }
        }
    }
}

/// Receive a message from current process inbox
pub fn sys_recv(blocking: bool) -> Result<Message, i32> {
    let current_task_id = crate::sched::get_current_task_id();
    
    // Log audit entry
    crate::secman::audit::log(crate::secman::audit::AuditEntry::syscall_recv(current_task_id, blocking));
    
    klog!(TRACE, [crate::log::tags::IPC], "sys_recv blocking={} (task={})", blocking, current_task_id);
    
    // Try to get a message from the current process inbox
    match super::queues::try_receive_message(current_task_id) {
        Some(message) => {
            klog!(INFO, [crate::log::tags::IPC], "sys_recv: received message from {}", message.header.sender.0);
            
            // Trace the IPC receive operation
            crate::trace::trace_ipc_operation(
                crate::trace::IpcOperation::Receive,
                message.header.sender.0,
                current_task_id,
                payload_size_u32(message.header.payload_size),
                message.header.priority,
            );
            
            Ok(message)
        }
        None => {
            // Inbox is empty
            if blocking {
                // Mark current task as blocked and call schedule()
                kprintln!("[ipc] sys_recv: inbox empty, blocking task {}", current_task_id);
                crate::sched::block_current_and_schedule();
                
                // When we get here, we've been woken up by a message arrival
                // Try to receive the message again
                match super::queues::try_receive_message(current_task_id) {
                    Some(message) => {
                        kprintln!("[ipc] sys_recv: received message after wakeup from {}", 
                                  message.header.sender.0);
                        Ok(message)
                    }
                    None => {
                        // This shouldn't happen if we were woken up correctly
                        kprintln!("[ipc] sys_recv: WARNING - woken up but no message available");
                        Err(-11) // EAGAIN
                    }
                }
            } else {
                // Non-blocking mode - return immediately
                kprintln!("[ipc] sys_recv: inbox empty, returning EAGAIN (non-blocking)");
                Err(-11) // EAGAIN
            }
        }
    }
}
