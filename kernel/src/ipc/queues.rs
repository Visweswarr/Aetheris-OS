/// IPC Message Queues and Inbox Implementation
/// 
/// This module provides message queue management, inbox functionality,
/// and channel-based communication for inter-process communication.

use super::types::*;
use super::{IpcError, IpcResult, MAX_CHANNELS, DEFAULT_QUEUE_SIZE, MAX_MESSAGE_SIZE};
use alloc::collections::{VecDeque, BTreeMap};
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// Message queue for storing messages
#[derive(Debug)]
pub struct MessageQueue {
    /// Queue of messages
    messages: VecDeque<Message>,
    
    /// Maximum queue size
    max_size: usize,
    
    /// Queue statistics
    total_enqueued: u64,
    total_dequeued: u64,
    max_size_reached: u64,
}

impl MessageQueue {
    /// Create a new message queue
    pub fn new(max_size: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_size.min(64)),
            max_size,
            total_enqueued: 0,
            total_dequeued: 0,
            max_size_reached: 0,
        }
    }
    
    /// Add a message to the queue
    pub fn enqueue(&mut self, message: Message) -> IpcResult<()> {
        if self.messages.len() >= self.max_size {
            self.max_size_reached += 1;
            return Err(IpcError::ChannelFull);
        }
        
        // Insert message based on priority
        let position = self.find_insert_position(&message);
        self.messages.insert(position, message);
        self.total_enqueued += 1;
        
        Ok(())
    }
    
    /// Add a message to the queue with inbox overflow policy
    /// 
    /// This implements the inbox overflow policy:
    /// - If inbox is full and incoming message is Low priority: drop-tail by priority (drop oldest Low priority first)
    /// - If inbox is full and incoming message is High/Critical priority: return EBUSY
    /// 
    /// # Arguments
    /// * `message` - The message to enqueue
    /// * `audit_dropped` - Callback to audit dropped messages
    /// 
    /// # Returns
    /// * `Ok(None)` - Message enqueued successfully
    /// * `Ok(Some(dropped))` - Message enqueued, but another message was dropped
    /// * `Err(IpcError::Busy)` - High/Critical priority message rejected due to full inbox
    pub fn enqueue_with_overflow_policy<F>(
        &mut self, 
        message: Message,
        mut audit_dropped: F
    ) -> IpcResult<Option<Message>>
    where
        F: FnMut(&Message),
    {
        if self.messages.len() < self.max_size {
            // Inbox has space, enqueue normally
            let position = self.find_insert_position(&message);
            self.messages.insert(position, message);
            self.total_enqueued += 1;
            return Ok(None);
        }
        
        // Inbox is full, apply overflow policy
        match message.header.priority {
            MessagePriority::Low => {
                // For Low priority messages, try to drop the oldest Low priority message
                if let Some(dropped_message) = self.drop_oldest_low_priority() {
                    // Audit the dropped message
                    audit_dropped(&dropped_message);
                    
                    // Now enqueue the new message
                    let position = self.find_insert_position(&message);
                    self.messages.insert(position, message);
                    self.total_enqueued += 1;
                    self.max_size_reached += 1; // Track overflow
                    
                    Ok(Some(dropped_message))
                } else {
                    // No Low priority messages to drop, reject the new one
                    self.max_size_reached += 1;
                    audit_dropped(&message);
                    Err(IpcError::ChannelFull)
                }
            }
            MessagePriority::Normal => {
                // For Normal priority messages, try to drop Low priority first, then oldest Normal
                if let Some(dropped_message) = self.drop_oldest_low_priority()
                    .or_else(|| self.drop_oldest_by_priority(MessagePriority::Normal)) {
                    
                    // Audit the dropped message
                    audit_dropped(&dropped_message);
                    
                    // Now enqueue the new message
                    let position = self.find_insert_position(&message);
                    self.messages.insert(position, message);
                    self.total_enqueued += 1;
                    self.max_size_reached += 1; // Track overflow
                    
                    Ok(Some(dropped_message))
                } else {
                    // Nothing to drop, reject the new message
                    self.max_size_reached += 1;
                    audit_dropped(&message);
                    Err(IpcError::ChannelFull)
                }
            }
            MessagePriority::High | MessagePriority::Critical => {
                // For High/Critical priority messages, return EBUSY when inbox is full
                self.max_size_reached += 1;
                Err(IpcError::Busy)
            }
        }
    }
    
    /// Drop the oldest Low priority message from the queue
    /// 
    /// # Returns
    /// The dropped message if one was found, None otherwise
    fn drop_oldest_low_priority(&mut self) -> Option<Message> {
        // Find the oldest (rightmost) Low priority message
        let position = self.messages.iter().rposition(|msg| {
            msg.header.priority == MessagePriority::Low
        })?;
        
        Some(self.messages.remove(position).unwrap())
    }
    
    /// Drop the oldest message with the specified priority
    /// 
    /// # Arguments
    /// * `priority` - The priority level to search for
    /// 
    /// # Returns
    /// The dropped message if one was found, None otherwise
    fn drop_oldest_by_priority(&mut self, priority: MessagePriority) -> Option<Message> {
        // Find the oldest (rightmost) message with the specified priority
        let position = self.messages.iter().rposition(|msg| {
            msg.header.priority == priority
        })?;
        
        Some(self.messages.remove(position).unwrap())
    }
    
    /// Remove and return the next message from the queue
    pub fn dequeue(&mut self) -> IpcResult<Message> {
        if let Some(message) = self.messages.pop_front() {
            self.total_dequeued += 1;
            Ok(message)
        } else {
            Err(IpcError::ChannelEmpty)
        }
    }
    
    /// Peek at the next message without removing it
    pub fn peek(&self) -> Option<&Message> {
        self.messages.front()
    }
    
    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    /// Check if queue is full
    pub fn is_full(&self) -> bool {
        self.messages.len() >= self.max_size
    }
    
    /// Get current queue length
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    /// Get queue capacity
    pub fn capacity(&self) -> usize {
        self.max_size
    }
    
    /// Find the correct position to insert a message based on priority
    fn find_insert_position(&self, message: &Message) -> usize {
        // Higher priority messages go to the front
        for (i, existing_msg) in self.messages.iter().enumerate() {
            if message.header.priority > existing_msg.header.priority {
                return i;
            }
        }
        self.messages.len()
    }
    
    /// Remove expired messages
    pub fn cleanup_expired(&mut self) -> usize {
        let initial_len = self.messages.len();
        self.messages.retain(|msg| !msg.header.is_expired());
        initial_len - self.messages.len()
    }
    
    /// Get queue statistics
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            current_size: self.messages.len(),
            max_size: self.max_size,
            total_enqueued: self.total_enqueued,
            total_dequeued: self.total_dequeued,
            max_size_reached: self.max_size_reached,
        }
    }
    
    /// Clear all messages from the queue
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

/// Queue statistics
#[derive(Debug, Clone, Copy)]
pub struct QueueStats {
    pub current_size: usize,
    pub max_size: usize,
    pub total_enqueued: u64,
    pub total_dequeued: u64,
    pub max_size_reached: u64,
}

/// Communication channel between two processes
#[derive(Debug)]
pub struct Channel {
    /// Channel identifier
    pub id: ChannelId,
    
    /// Sender process ID
    pub sender: ProcessId,
    
    /// Receiver process ID
    pub receiver: ProcessId,
    
    /// Message queue
    pub queue: MessageQueue,
    
    /// Channel creation timestamp
    pub created_at: u64,
    
    /// Channel state
    pub state: ChannelState,
    
    /// Channel access permissions
    pub permissions: ChannelPermissions,
}

impl Channel {
    /// Create a new channel
    pub fn new(
        id: ChannelId,
        sender: ProcessId,
        receiver: ProcessId,
        queue_size: usize,
    ) -> Self {
        Self {
            id,
            sender,
            receiver,
            queue: MessageQueue::new(queue_size),
            created_at: get_current_timestamp(),
            state: ChannelState::Active,
            permissions: ChannelPermissions::default(),
        }
    }
    
    /// Send a message through this channel
    pub fn send(&mut self, message: Message) -> IpcResult<()> {
        // Validate message
        if !self.can_send(&message)? {
            return Err(IpcError::PermissionDenied);
        }
        
        // Check message size
        if message.total_size() > MAX_MESSAGE_SIZE {
            return Err(IpcError::MessageTooLarge);
        }
        
        // Check channel state
        match self.state {
            ChannelState::Active => {},
            ChannelState::Closed => return Err(IpcError::ChannelNotFound),
            ChannelState::Suspended => return Err(IpcError::PermissionDenied),
        }
        
        self.queue.enqueue(message)
    }
    
    /// Receive a message from this channel
    pub fn receive(&mut self) -> IpcResult<Message> {
        // Check channel state
        match self.state {
            ChannelState::Active => {},
            ChannelState::Closed => return Err(IpcError::ChannelNotFound),
            ChannelState::Suspended => return Err(IpcError::PermissionDenied),
        }
        
        self.queue.dequeue()
    }
    
    /// Peek at the next message without removing it
    pub fn peek(&self) -> Option<&Message> {
        if matches!(self.state, ChannelState::Active) {
            self.queue.peek()
        } else {
            None
        }
    }
    
    /// Check if the message can be sent through this channel
    fn can_send(&self, message: &Message) -> IpcResult<bool> {
        // Check sender authorization
        if message.header.sender != self.sender && message.header.sender != ProcessId::KERNEL {
            return Ok(false);
        }
        
        // Check receiver
        if message.header.receiver != self.receiver {
            return Ok(false);
        }
        
        // Check permissions
        if !self.permissions.can_send {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Close the channel
    pub fn close(&mut self) {
        self.state = ChannelState::Closed;
        self.queue.clear();
    }
    
    /// Suspend the channel
    pub fn suspend(&mut self) {
        self.state = ChannelState::Suspended;
    }
    
    /// Resume the channel
    pub fn resume(&mut self) {
        if matches!(self.state, ChannelState::Suspended) {
            self.state = ChannelState::Active;
        }
    }
    
    /// Cleanup expired messages
    pub fn cleanup(&mut self) -> usize {
        self.queue.cleanup_expired()
    }
}

/// Channel state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelState {
    /// Channel is active and can send/receive messages
    Active,
    
    /// Channel is suspended (temporary)
    Suspended,
    
    /// Channel is closed and cannot be used
    Closed,
}

/// Channel permissions
#[derive(Debug, Clone, Copy)]
pub struct ChannelPermissions {
    /// Can send messages
    pub can_send: bool,
    
    /// Can receive messages
    pub can_receive: bool,
    
    /// Can close channel
    pub can_close: bool,
    
    /// Can modify permissions
    pub can_modify: bool,
}

impl Default for ChannelPermissions {
    fn default() -> Self {
        Self {
            can_send: true,
            can_receive: true,
            can_close: true,
            can_modify: false,
        }
    }
}

/// Process inbox for receiving messages from multiple channels
#[derive(Debug)]
pub struct Inbox {
    /// Process ID that owns this inbox
    pub owner: ProcessId,
    
    /// Channels that deliver to this inbox
    pub channels: Vec<ChannelId>,
    
    /// Incoming message queue
    pub queue: MessageQueue,
    
    /// Inbox statistics
    pub stats: InboxStats,
}

impl Inbox {
    /// Create a new inbox
    pub fn new(owner: ProcessId, max_size: usize) -> Self {
        Self {
            owner,
            channels: Vec::new(),
            queue: MessageQueue::new(max_size),
            stats: InboxStats::new(),
        }
    }
    
    /// Add a channel to this inbox
    pub fn add_channel(&mut self, channel_id: ChannelId) {
        if !self.channels.contains(&channel_id) {
            self.channels.push(channel_id);
        }
    }
    
    /// Remove a channel from this inbox
    pub fn remove_channel(&mut self, channel_id: ChannelId) {
        self.channels.retain(|&id| id != channel_id);
    }
    
    /// Deliver a message to this inbox
    pub fn deliver(&mut self, message: Message) -> IpcResult<()> {
        // Validate that message is for this inbox owner
        if message.header.receiver != self.owner {
            return Err(IpcError::InvalidMessage);
        }
        
        self.stats.messages_received += 1;
        self.queue.enqueue(message)
    }
    
    /// Deliver a message to this inbox with overflow policy
    /// 
    /// This implements the inbox overflow policy:
    /// - If inbox is full and incoming message is Low priority: drop-tail by priority
    /// - If inbox is full and incoming message is High/Critical priority: return EBUSY
    /// 
    /// # Arguments
    /// * `message` - The message to deliver
    /// 
    /// # Returns
    /// * `Ok(None)` - Message delivered successfully
    /// * `Ok(Some(dropped))` - Message delivered, but another message was dropped
    /// * `Err(IpcError::Busy)` - High/Critical priority message rejected due to full inbox
    pub fn deliver_with_overflow_policy(&mut self, message: Message) -> IpcResult<Option<Message>> {
        // Check for fault injection - simulate inbox overflow every Nth push
        if crate::fault_injection::should_force_inbox_overflow() {
            crate::log::klog!(crate::log::Level::WARN, [crate::log::tags::FAULT_INJECTION], 
                "FAULT INJECTION: Simulating inbox overflow for message from {} to {} (len={}, prio={:?})",
                message.header.sender.0, message.header.receiver.0, 
                message.payload.size(), message.header.priority);
            
            // Simulate inbox overflow by treating this as a full inbox
            // This will trigger the normal overflow policy logic
            if message.header.priority == Priority::Low {
                // For Low priority messages, audit the drop and return success
                self.audit_dropped_message(&message);
                self.stats.messages_expired += 1;
                return Ok(Some(message)); // Return dropped message
            } else {
                // For High/Critical priority messages, return EBUSY
                self.stats.messages_received = self.stats.messages_received.saturating_sub(1);
                return Err(IpcError::Busy);
            }
        }
        
        // Validate that message is for this inbox owner
        if message.header.receiver != self.owner {
            return Err(IpcError::InvalidMessage);
        }
        
        self.stats.messages_received += 1;
        
        // Use the queue's overflow policy implementation
        let mut dropped_for_audit = None;
        let result = self.queue.enqueue_with_overflow_policy(message, |dropped_msg| {
            dropped_for_audit = Some(dropped_msg.clone());
        });
        if let Some(dropped_msg) = dropped_for_audit.as_ref() {
            self.audit_dropped_message(dropped_msg);
        }
        
        // Update statistics based on result
        match &result {
            Ok(Some(_)) => {
                // A message was dropped - this is tracked in the audit
                self.stats.messages_expired += 1; // Reuse this counter for drops
            }
            Err(_) => {
                // Message was rejected
                self.stats.messages_received = self.stats.messages_received.saturating_sub(1);
            }
            _ => {
                // Normal delivery, no additional stats needed
            }
        }
        
        result
    }
    
    /// Audit a dropped message
    /// 
    /// # Arguments
    /// * `message` - The message that was dropped
    fn audit_dropped_message(&self, message: &Message) {
        // Create an audit entry for the dropped message
        // For now, we'll use a simple approach - in the full implementation,
        // we would add the IPC_MSG_DROPPED operation code
        let audit_entry = crate::secman::audit::AuditEntry::new(
            message.header.sender.0,        // pid (sender)
            104 + 1,                        // op (use 105 for dropped message)
            self.encode_drop_audit_arg(message) // arg (encode message info)
        );
        
        crate::secman::audit::log(audit_entry);
        
        crate::klog!(TRACE, "[INBOX] DROPPED message: sender={} dst={} len={} prio={:?}",
                    message.header.sender.0,
                    message.header.receiver.0,
                    message.payload.size(),
                    message.header.priority);
    }
    
    /// Encode message information for audit argument
    /// 
    /// Encodes: priority (2 bits) | payload_size (30 bits) | receiver_id (32 bits)
    /// 
    /// # Arguments
    /// * `message` - The message to encode
    /// 
    /// # Returns
    /// Encoded audit argument containing message metadata
    fn encode_drop_audit_arg(&self, message: &Message) -> u64 {
        let prio = message.header.priority as u64;
        let len = (message.payload.size() as u64) & 0x3FFFFFFF; // 30 bits max
        let dst = message.header.receiver.0 & 0xFFFFFFFF; // 32 bits max
        
        (prio << 62) | (len << 32) | dst
    }
    
    /// Receive the next message from the inbox
    pub fn receive(&mut self) -> IpcResult<Message> {
        let message = self.queue.dequeue()?;
        self.stats.messages_delivered += 1;
        Ok(message)
    }
    
    /// Peek at the next message without removing it
    pub fn peek(&self) -> Option<&Message> {
        self.queue.peek()
    }
    
    /// Check if inbox has messages
    pub fn has_messages(&self) -> bool {
        !self.queue.is_empty()
    }
    
    /// Get number of pending messages
    pub fn message_count(&self) -> usize {
        self.queue.len()
    }
    
    /// Cleanup expired messages
    pub fn cleanup(&mut self) -> usize {
        self.queue.cleanup_expired()
    }
}

/// Inbox statistics
#[derive(Debug, Clone, Copy)]
pub struct InboxStats {
    pub messages_received: u64,
    pub messages_delivered: u64,
    pub messages_expired: u64,
}

impl InboxStats {
    pub fn new() -> Self {
        Self {
            messages_received: 0,
            messages_delivered: 0,
            messages_expired: 0,
        }
    }
}

/// Global channel manager
#[derive(Debug)]
pub struct ChannelManager {
    /// All channels indexed by ID
    channels: BTreeMap<ChannelId, Channel>,
    
    /// Process inboxes indexed by process ID
    inboxes: BTreeMap<ProcessId, Inbox>,
    
    /// Channel ID counter
    next_channel_id: u64,
    
    /// Manager statistics
    stats: ChannelManagerStats,
}

impl ChannelManager {
    /// Create a new channel manager
    pub fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
            inboxes: BTreeMap::new(),
            next_channel_id: 1,
            stats: ChannelManagerStats::new(),
        }
    }
    
    /// Create a new channel
    pub fn create_channel(
        &mut self,
        sender: ProcessId,
        receiver: ProcessId,
        queue_size: usize,
    ) -> IpcResult<ChannelId> {
        if self.channels.len() >= MAX_CHANNELS {
            return Err(IpcError::OutOfMemory);
        }
        
        let channel_id = ChannelId(self.next_channel_id);
        self.next_channel_id += 1;
        
        let channel = Channel::new(channel_id, sender, receiver, queue_size);
        self.channels.insert(channel_id, channel);
        
        // Ensure receiver has an inbox
        self.ensure_inbox(receiver);
        
        // Add channel to receiver's inbox
        if let Some(inbox) = self.inboxes.get_mut(&receiver) {
            inbox.add_channel(channel_id);
        }
        
        self.stats.channels_created += 1;
        self.stats.active_channels += 1;
        
        Ok(channel_id)
    }
    
    /// Destroy a channel
    pub fn destroy_channel(&mut self, channel_id: ChannelId) -> IpcResult<()> {
        if let Some(mut channel) = self.channels.remove(&channel_id) {
            channel.close();
            
            // Remove from receiver's inbox
            if let Some(inbox) = self.inboxes.get_mut(&channel.receiver) {
                inbox.remove_channel(channel_id);
            }
            
            self.stats.channels_destroyed += 1;
            self.stats.active_channels = self.stats.active_channels.saturating_sub(1);
            
            Ok(())
        } else {
            Err(IpcError::ChannelNotFound)
        }
    }
    
    /// Send a message through a channel
    pub fn send_message(&mut self, channel_id: ChannelId, message: Message) -> IpcResult<()> {
        let channel = self.channels.get_mut(&channel_id)
            .ok_or(IpcError::ChannelNotFound)?;
        
        // Set send timestamp for latency tracking
        let mut message_with_timestamp = message.clone();
        message_with_timestamp.header.set_send_timestamp();
        
        channel.send(message_with_timestamp.clone())?;
        
        // Deliver to receiver's inbox
        if let Some(inbox) = self.inboxes.get_mut(&channel.receiver) {
            inbox.deliver(message_with_timestamp)?;
        }
        
        self.stats.messages_sent += 1;
        super::update_stats(|stats| {
            stats.messages_sent += 1;
            stats.messages_queued += 1;
            stats.bytes_transferred += message.total_size() as u64;
        });
        
        Ok(())
    }
    
    /// Receive a message from a channel
    pub fn receive_message(&mut self, channel_id: ChannelId) -> IpcResult<Message> {
        let channel = self.channels.get_mut(&channel_id)
            .ok_or(IpcError::ChannelNotFound)?;
        
        let message = channel.receive()?;
        
        // Calculate and record IPC latency
        if let Some(latency_us) = message.header.calculate_latency_us() {
            crate::trace::record_ipc_latency(latency_us);
        }
        
        self.stats.messages_received += 1;
        super::update_stats(|stats| {
            stats.messages_received += 1;
            stats.messages_queued = stats.messages_queued.saturating_sub(1);
        });
        
        Ok(message)
    }
    
    /// Receive a message from a process inbox
    pub fn receive_from_inbox(&mut self, process_id: ProcessId) -> IpcResult<Message> {
        let inbox = self.inboxes.get_mut(&process_id)
            .ok_or(IpcError::InvalidProcessId)?;
        
        let message = inbox.receive()?;
        
        // Calculate and record IPC latency
        if let Some(latency_us) = message.header.calculate_latency_us() {
            crate::trace::record_ipc_latency(latency_us);
        }
        
        self.stats.messages_received += 1;
        super::update_stats(|stats| {
            stats.messages_received += 1;
            stats.messages_queued = stats.messages_queued.saturating_sub(1);
        });
        
        Ok(message)
    }
    
    /// Enqueue a message to a process inbox
    pub fn enqueue_to_inbox(&mut self, process_id: ProcessId, message: Message) -> IpcResult<()> {
        // Ensure the process has an inbox
        self.ensure_inbox(process_id);
        
        let inbox = self.inboxes.get_mut(&process_id)
            .ok_or(IpcError::InvalidProcessId)?;
        
        inbox.deliver(message)?;
        
        self.stats.messages_sent += 1;
        super::update_stats(|stats| {
            stats.messages_sent += 1;
            stats.messages_queued += 1;
        });
        
        Ok(())
    }
    
    /// Enqueue a message to a process inbox with overflow policy
    /// 
    /// This implements the inbox overflow policy for message delivery.
    /// 
    /// # Arguments
    /// * `process_id` - The target process ID
    /// * `message` - The message to enqueue
    /// 
    /// # Returns
    /// * `Ok(None)` - Message delivered successfully
    /// * `Ok(Some(dropped))` - Message delivered, but another message was dropped
    /// * `Err(IpcError::Busy)` - High/Critical priority message rejected due to full inbox
    pub fn enqueue_to_inbox_with_overflow_policy(
        &mut self, 
        process_id: ProcessId, 
        message: Message
    ) -> IpcResult<Option<Message>> {
        // Ensure the process has an inbox
        self.ensure_inbox(process_id);
        
        let inbox = self.inboxes.get_mut(&process_id)
            .ok_or(IpcError::InvalidProcessId)?;
        
        let result = inbox.deliver_with_overflow_policy(message)?;
        
        self.stats.messages_sent += 1;
        super::update_stats(|stats| {
            stats.messages_sent += 1;
            match &result {
                Some(_) => {
                    // A message was dropped, queue size remains the same
                    // (one removed, one added)
                }
                None => {
                    // Normal enqueue, increase queue size
                    stats.messages_queued += 1;
                }
            }
        });
        
        Ok(result)
    }
    
    /// Check if a process has pending messages
    pub fn has_messages(&self, process_id: ProcessId) -> bool {
        self.inboxes.get(&process_id)
            .map(|inbox| inbox.has_messages())
            .unwrap_or(false)
    }
    
    /// Get number of pending messages for a process
    pub fn message_count(&self, process_id: ProcessId) -> usize {
        self.inboxes.get(&process_id)
            .map(|inbox| inbox.message_count())
            .unwrap_or(0)
    }
    
    /// Ensure a process has an inbox
    fn ensure_inbox(&mut self, process_id: ProcessId) {
        if !self.inboxes.contains_key(&process_id) {
            let inbox = Inbox::new(process_id, DEFAULT_QUEUE_SIZE * 4); // Larger inbox
            self.inboxes.insert(process_id, inbox);
        }
    }
    
    /// Get channel information
    pub fn get_channel(&self, channel_id: ChannelId) -> Option<&Channel> {
        self.channels.get(&channel_id)
    }
    
    /// Get inbox information
    pub fn get_inbox(&self, process_id: ProcessId) -> Option<&Inbox> {
        self.inboxes.get(&process_id)
    }
    
    /// List all channels for a process
    pub fn list_channels(&self, process_id: ProcessId) -> Vec<ChannelId> {
        self.channels.iter()
            .filter_map(|(id, channel)| {
                if channel.sender == process_id || channel.receiver == process_id {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Cleanup expired messages across all channels and inboxes
    pub fn cleanup_expired(&mut self) -> usize {
        let mut cleaned = 0;
        
        for channel in self.channels.values_mut() {
            cleaned += channel.cleanup();
        }
        
        for inbox in self.inboxes.values_mut() {
            cleaned += inbox.cleanup();
        }
        
        cleaned
    }
    
    /// Get manager statistics
    pub fn stats(&self) -> ChannelManagerStats {
        self.stats
    }
}

/// Channel manager statistics
#[derive(Debug, Clone, Copy)]
pub struct ChannelManagerStats {
    pub channels_created: u64,
    pub channels_destroyed: u64,
    pub active_channels: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

impl ChannelManagerStats {
    pub fn new() -> Self {
        Self {
            channels_created: 0,
            channels_destroyed: 0,
            active_channels: 0,
            messages_sent: 0,
            messages_received: 0,
        }
    }
}

/// Global channel manager instance
static CHANNEL_MANAGER: Mutex<Option<ChannelManager>> = Mutex::new(None);

/// Initialize the channel manager
pub fn init_channel_manager() {
    let mut manager = CHANNEL_MANAGER.lock();
    *manager = Some(ChannelManager::new());
}

/// Create a new channel
pub fn create_channel(
    sender: ProcessId,
    receiver: ProcessId,
    queue_size: usize,
) -> IpcResult<ChannelId> {
    let mut manager = CHANNEL_MANAGER.lock();
    if let Some(ref mut mgr) = *manager {
        mgr.create_channel(sender, receiver, queue_size)
    } else {
        Err(IpcError::OutOfMemory)
    }
}

/// Destroy a channel
pub fn destroy_channel(channel_id: ChannelId) -> IpcResult<()> {
    let mut manager = CHANNEL_MANAGER.lock();
    if let Some(ref mut mgr) = *manager {
        mgr.destroy_channel(channel_id)
    } else {
        Err(IpcError::ChannelNotFound)
    }
}

/// Send a message through a channel
pub fn send_message(channel_id: ChannelId, message: Message) -> IpcResult<()> {
    let mut manager = CHANNEL_MANAGER.lock();
    if let Some(ref mut mgr) = *manager {
        mgr.send_message(channel_id, message)
    } else {
        Err(IpcError::ChannelNotFound)
    }
}

/// Receive a message from a channel
pub fn receive_message(channel_id: ChannelId) -> IpcResult<Message> {
    let mut manager = CHANNEL_MANAGER.lock();
    if let Some(ref mut mgr) = *manager {
        mgr.receive_message(channel_id)
    } else {
        Err(IpcError::ChannelNotFound)
    }
}

/// Receive a message from a process inbox
pub fn receive_from_inbox(process_id: ProcessId) -> IpcResult<Message> {
    let mut manager = CHANNEL_MANAGER.lock();
    if let Some(ref mut mgr) = *manager {
        mgr.receive_from_inbox(process_id)
    } else {
        Err(IpcError::InvalidProcessId)
    }
}

/// Check if a process has pending messages
pub fn has_messages(process_id: ProcessId) -> bool {
    let manager = CHANNEL_MANAGER.lock();
    if let Some(ref mgr) = *manager {
        mgr.has_messages(process_id)
    } else {
        false
    }
}

/// Get number of pending messages for a process
pub fn message_count(process_id: ProcessId) -> usize {
    let manager = CHANNEL_MANAGER.lock();
    if let Some(ref mgr) = *manager {
        mgr.message_count(process_id)
    } else {
        0
    }
}

/// List all channels for a process
pub fn list_channels(process_id: ProcessId) -> Vec<ChannelId> {
    let manager = CHANNEL_MANAGER.lock();
    if let Some(ref mgr) = *manager {
        mgr.list_channels(process_id)
    } else {
        Vec::new()
    }
}

/// Cleanup expired messages
pub fn cleanup_expired() -> usize {
    let mut manager = CHANNEL_MANAGER.lock();
    if let Some(ref mut mgr) = *manager {
        mgr.cleanup_expired()
    } else {
        0
    }
}

/// Get channel manager statistics
pub fn get_manager_stats() -> ChannelManagerStats {
    let manager = CHANNEL_MANAGER.lock();
    if let Some(ref mgr) = *manager {
        mgr.stats()
    } else {
        ChannelManagerStats::new()
    }
}

/// Get current timestamp (simplified implementation)
fn get_current_timestamp() -> u64 {
    static TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(0);
    TIMESTAMP_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Try to receive a message from a process inbox without blocking
pub fn try_receive_message(process_id: u64) -> Option<Message> {
    let process_id = ProcessId(process_id);
    match receive_from_inbox(process_id) {
        Ok(message) => Some(message),
        Err(_) => None,
    }
}

/// Send a message to a process inbox and wake up the task if blocked
pub fn send_to_inbox_and_wake(dst_process_id: u64, message: Message) -> IpcResult<()> {
    let dst_process = ProcessId(dst_process_id);
    
    // Check message priority for RT preemption decision
    let is_rt_message = message.header.priority >= MessagePriority::High;
    
    // Enqueue the message to the destination inbox with overflow policy
    let mut manager = CHANNEL_MANAGER.lock();
    if let Some(ref mut mgr) = *manager {
        // Add message to destination inbox using the overflow policy
        match mgr.enqueue_to_inbox_with_overflow_policy(dst_process, message) {
            Ok(dropped_message) => {
                if let Some(dropped) = dropped_message {
                    crate::klog!(TRACE, "[IPC] Message enqueued to process {} inbox, dropped message from sender {}", 
                               dst_process_id, dropped.header.sender.0);
                } else {
                    crate::klog!(TRACE, "[IPC] Message enqueued to process {} inbox", dst_process_id);
                }
                
                // Release the lock before scheduler operations
                drop(manager);
                
                // Wake up the destination task if it's blocked
                let dst_task_id = crate::sched::TaskId(dst_process_id);
                
                // Check if the task is currently blocked (waiting for messages)
                if let Some(task) = crate::sched::get_task(dst_task_id) {
                    if task.state == crate::sched::task::TaskState::Blocked {
                        crate::klog!(TRACE, "[IPC] Waking up blocked task {}", dst_process_id);
                        crate::sched::wake(dst_task_id);
                        
                        // For RT priority messages, request preemption of current task
                        if is_rt_message {
                            let current_task_id = crate::sched::get_current_task_id();
                            if current_task_id != 0 { // Don't preempt idle task
                                crate::klog!(TRACE, "[IPC] RT message for blocked task, requesting preemption of task {}", current_task_id);
                                crate::sched::request_task_preemption(crate::sched::TaskId(current_task_id));
                            }
                        }
                    }
                }
                
                Ok(())
            }
            Err(IpcError::Busy) => {
                crate::klog!(TRACE, "[IPC] High/Critical priority message rejected - inbox full for process {}", dst_process_id);
                Err(IpcError::Busy)
            }
            Err(e) => Err(e),
        }
    } else {
        Err(IpcError::InvalidProcessId)
    }
}

/// Enhanced send message function that handles wakeup and preemption
pub fn send_message_with_wakeup(
    sender_process_id: u64,
    dst_process_id: u64,
    message: Message,
) -> IpcResult<()> {
    crate::klog!(TRACE, "[IPC] Sending message from {} to {} (priority: {:?})",
               sender_process_id, dst_process_id, message.header.priority);
    
    // Send to inbox and handle wakeup/preemption
    send_to_inbox_and_wake(dst_process_id, message)
}
