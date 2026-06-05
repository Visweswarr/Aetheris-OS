/// IPC Message Types and Data Structures
/// 
/// This module defines the core data structures for inter-process communication,
/// including message headers, payloads, process identifiers, and message types.

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::time::Duration;

/// Process identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(pub u64);

impl ProcessId {
    /// Create a new process ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub const fn id(&self) -> u64 {
        self.0
    }
    
    /// Check if this is a valid process ID
    pub const fn is_valid(&self) -> bool {
        self.0 > 0
    }
    
    /// Kernel process ID (reserved)
    pub const KERNEL: ProcessId = ProcessId(0);
    
    /// Init process ID
    pub const INIT: ProcessId = ProcessId(1);
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PID:{}", self.0)
    }
}

/// Message identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageId(pub u64);

impl MessageId {
    /// Create a new message ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub const fn id(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for MessageId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MSG:{}", self.0)
    }
}

/// Channel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelId(pub u64);

impl ChannelId {
    /// Create a new channel ID
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub const fn id(&self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CH:{}", self.0)
    }
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Low priority message
    Low = 0,
    
    /// Normal priority message
    Normal = 1,
    
    /// High priority message
    High = 2,
    
    /// Critical priority message (system messages)
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

impl core::fmt::Display for MessagePriority {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MessagePriority::Low => write!(f, "LOW"),
            MessagePriority::Normal => write!(f, "NORMAL"),
            MessagePriority::High => write!(f, "HIGH"),
            MessagePriority::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl MessagePriority {
    /// Numeric priority used by policy and trace layers.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Request message (expects response)
    Request,
    
    /// Response message (to a request)
    Response,
    
    /// Notification message (no response expected)
    Notification,
    
    /// Signal message (system signals)
    Signal,
    
    /// Data transfer message
    Data,
    
    /// Control message (system control)
    Control,
    
    /// Error message
    Error,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Notification
    }
}

impl core::fmt::Display for MessageType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MessageType::Request => write!(f, "REQUEST"),
            MessageType::Response => write!(f, "RESPONSE"),
            MessageType::Notification => write!(f, "NOTIFICATION"),
            MessageType::Signal => write!(f, "SIGNAL"),
            MessageType::Data => write!(f, "DATA"),
            MessageType::Control => write!(f, "CONTROL"),
            MessageType::Error => write!(f, "ERROR"),
        }
    }
}

/// Message flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageFlags {
    /// Message requires acknowledgment
    pub ack_required: bool,
    
    /// Message is urgent
    pub urgent: bool,
    
    /// Message should be delivered reliably
    pub reliable: bool,
    
    /// Message contains sensitive data
    pub secure: bool,
    
    /// Message is a broadcast
    pub broadcast: bool,
    
    /// Message should be compressed
    pub compressed: bool,
}

impl Default for MessageFlags {
    fn default() -> Self {
        Self {
            ack_required: false,
            urgent: false,
            reliable: true,
            secure: false,
            broadcast: false,
            compressed: false,
        }
    }
}

impl MessageFlags {
    pub const ACK_REQUIRED: u32 = 1 << 0;
    pub const URGENT: u32 = 1 << 1;
    pub const RELIABLE: u32 = 1 << 2;
    pub const SECURE: u32 = 1 << 3;
    pub const BROADCAST: u32 = 1 << 4;
    pub const COMPRESSED: u32 = 1 << 5;

    /// Compatibility alias used by authenticated IPC fast paths.
    pub const REALTIME: Self = Self {
        ack_required: false,
        urgent: true,
        reliable: false,
        secure: false,
        broadcast: false,
        compressed: false,
    };

    /// Create flags for a simple notification
    pub const fn notification() -> Self {
        Self {
            ack_required: false,
            urgent: false,
            reliable: true,
            secure: false,
            broadcast: false,
            compressed: false,
        }
    }
    
    /// Create flags for a request message
    pub const fn request() -> Self {
        Self {
            ack_required: true,
            urgent: false,
            reliable: true,
            secure: false,
            broadcast: false,
            compressed: false,
        }
    }
    
    /// Create flags for an urgent message
    pub const fn urgent() -> Self {
        Self {
            ack_required: true,
            urgent: true,
            reliable: true,
            secure: false,
            broadcast: false,
            compressed: false,
        }
    }
    
    /// Create flags for a secure message
    pub const fn secure() -> Self {
        Self {
            ack_required: true,
            urgent: false,
            reliable: true,
            secure: true,
            broadcast: false,
            compressed: false,
        }
    }

    pub fn bits(&self) -> u32 {
        let mut bits = 0;
        if self.ack_required { bits |= Self::ACK_REQUIRED; }
        if self.urgent { bits |= Self::URGENT; }
        if self.reliable { bits |= Self::RELIABLE; }
        if self.secure { bits |= Self::SECURE; }
        if self.broadcast { bits |= Self::BROADCAST; }
        if self.compressed { bits |= Self::COMPRESSED; }
        bits
    }

    pub fn from_bits(bits: u32) -> Option<Self> {
        let known = Self::ACK_REQUIRED
            | Self::URGENT
            | Self::RELIABLE
            | Self::SECURE
            | Self::BROADCAST
            | Self::COMPRESSED;
        if bits & !known != 0 {
            return None;
        }

        Some(Self {
            ack_required: bits & Self::ACK_REQUIRED != 0,
            urgent: bits & Self::URGENT != 0,
            reliable: bits & Self::RELIABLE != 0,
            secure: bits & Self::SECURE != 0,
            broadcast: bits & Self::BROADCAST != 0,
            compressed: bits & Self::COMPRESSED != 0,
        })
    }

    pub fn contains(&self, required: Self) -> bool {
        self.bits() & required.bits() == required.bits()
    }
}

impl core::fmt::Display for MessageFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut flags = Vec::new();
        
        if self.ack_required { flags.push("ACK"); }
        if self.urgent { flags.push("URG"); }
        if self.reliable { flags.push("REL"); }
        if self.secure { flags.push("SEC"); }
        if self.broadcast { flags.push("BRD"); }
        if self.compressed { flags.push("CMP"); }
        
        if flags.is_empty() {
            write!(f, "NONE")
        } else {
            write!(f, "{}", flags.join("|"))
        }
    }
}

/// Message payload types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessagePayload {
    /// Empty payload
    Empty,
    
    /// Raw data payload
    Data(Vec<u8>),
    
    /// Text payload
    Text(Vec<u8>),
    
    /// Structured data payload (key-value pairs)
    Structured(BTreeMap<Vec<u8>, Vec<u8>>),
    
    /// File descriptor payload
    FileDescriptor(u32),
    
    /// Memory region payload
    MemoryRegion {
        /// Start address
        start_addr: u64,
        /// Size in bytes
        size: usize,
        /// Access permissions
        permissions: u32,
    },
    
    /// Signal payload
    Signal {
        /// Signal number
        signal: u32,
        /// Additional data
        data: u64,
    },
    
    /// Error payload
    Error {
        /// Error code
        code: u32,
        /// Error message
        message: Vec<u8>,
    },
}

impl Default for MessagePayload {
    fn default() -> Self {
        MessagePayload::Empty
    }
}

impl MessagePayload {
    /// Get the size of the payload in bytes
    pub fn size(&self) -> usize {
        match self {
            MessagePayload::Empty => 0,
            MessagePayload::Data(data) => data.len(),
            MessagePayload::Text(text) => text.len(),
            MessagePayload::Structured(map) => {
                map.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() + 
                map.len() * 16 // Approximate overhead per entry
            }
            MessagePayload::FileDescriptor(_) => 4,
            MessagePayload::MemoryRegion { .. } => 16,
            MessagePayload::Signal { .. } => 12,
            MessagePayload::Error { message, .. } => 4 + message.len(),
        }
    }

    pub fn len(&self) -> usize {
        self.size()
    }

    /// Deterministic byte representation used by authentication paths.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            MessagePayload::Empty => out.push(0),
            MessagePayload::Data(data) => {
                out.push(1);
                out.extend_from_slice(data);
            }
            MessagePayload::Text(text) => {
                out.push(2);
                out.extend_from_slice(text);
            }
            MessagePayload::Structured(map) => {
                out.push(3);
                out.extend_from_slice(&(map.len() as u32).to_le_bytes());
                for (key, value) in map {
                    out.extend_from_slice(&(key.len() as u32).to_le_bytes());
                    out.extend_from_slice(key);
                    out.extend_from_slice(&(value.len() as u32).to_le_bytes());
                    out.extend_from_slice(value);
                }
            }
            MessagePayload::FileDescriptor(fd) => {
                out.push(4);
                out.extend_from_slice(&fd.to_le_bytes());
            }
            MessagePayload::MemoryRegion { start_addr, size, permissions } => {
                out.push(5);
                out.extend_from_slice(&start_addr.to_le_bytes());
                out.extend_from_slice(&(*size as u64).to_le_bytes());
                out.extend_from_slice(&permissions.to_le_bytes());
            }
            MessagePayload::Signal { signal, data } => {
                out.push(6);
                out.extend_from_slice(&signal.to_le_bytes());
                out.extend_from_slice(&data.to_le_bytes());
            }
            MessagePayload::Error { code, message } => {
                out.push(7);
                out.extend_from_slice(&code.to_le_bytes());
                out.extend_from_slice(&(message.len() as u32).to_le_bytes());
                out.extend_from_slice(message);
            }
        }
        out
    }
    
    /// Check if payload is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, MessagePayload::Empty) || self.size() == 0
    }
    
    /// Get payload type name
    pub fn type_name(&self) -> &'static str {
        match self {
            MessagePayload::Empty => "Empty",
            MessagePayload::Data(_) => "Data",
            MessagePayload::Text(_) => "Text",
            MessagePayload::Structured(_) => "Structured",
            MessagePayload::FileDescriptor(_) => "FileDescriptor",
            MessagePayload::MemoryRegion { .. } => "MemoryRegion",
            MessagePayload::Signal { .. } => "Signal",
            MessagePayload::Error { .. } => "Error",
        }
    }
    
    /// Create a text payload from a string
    pub fn from_text(text: &str) -> Self {
        MessagePayload::Text(text.as_bytes().to_vec())
    }
    
    /// Create a data payload from bytes
    pub fn from_data(data: &[u8]) -> Self {
        MessagePayload::Data(data.to_vec())
    }
    
    /// Create an error payload
    pub fn from_error(code: u32, message: &str) -> Self {
        MessagePayload::Error {
            code,
            message: message.as_bytes().to_vec(),
        }
    }
    
    /// Create a signal payload
    pub fn from_signal(signal: u32, data: u64) -> Self {
        MessagePayload::Signal { signal, data }
    }
    
    /// Extract data as bytes if possible
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            MessagePayload::Data(data) | MessagePayload::Text(data) => Some(data),
            _ => None,
        }
    }
    
    /// Extract text as string if possible
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessagePayload::Text(data) => core::str::from_utf8(data).ok(),
            _ => None,
        }
    }
}

impl core::fmt::Display for MessagePayload {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MessagePayload::Empty => write!(f, "Empty"),
            MessagePayload::Data(data) => write!(f, "Data({} bytes)", data.len()),
            MessagePayload::Text(text) => {
                if let Ok(s) = core::str::from_utf8(text) {
                    write!(f, "Text(\"{}\")", s)
                } else {
                    write!(f, "Text({} bytes, invalid UTF-8)", text.len())
                }
            }
            MessagePayload::Structured(map) => write!(f, "Structured({} entries)", map.len()),
            MessagePayload::FileDescriptor(fd) => write!(f, "FileDescriptor({})", fd),
            MessagePayload::MemoryRegion { start_addr, size, permissions } => {
                write!(f, "MemoryRegion(0x{:x}, {} bytes, perm=0x{:x})", start_addr, size, permissions)
            }
            MessagePayload::Signal { signal, data } => write!(f, "Signal({}, 0x{:x})", signal, data),
            MessagePayload::Error { code, message } => {
                if let Ok(s) = core::str::from_utf8(message) {
                    write!(f, "Error({}, \"{}\")", code, s)
                } else {
                    write!(f, "Error({}, {} bytes)", code, message.len())
                }
            }
        }
    }
}

/// Message header containing metadata about the message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeader {
    /// Unique message identifier
    pub id: MessageId,
    
    /// Sender process ID
    pub sender: ProcessId,
    
    /// Receiver process ID
    pub receiver: ProcessId,
    
    /// Message type
    pub msg_type: MessageType,
    
    /// Message priority
    pub priority: MessagePriority,
    
    /// Message flags
    pub flags: MessageFlags,
    
    /// Payload size in bytes
    pub payload_size: usize,
    
    /// Timestamp when message was created (in milliseconds)
    pub timestamp: u64,
    
    /// Timestamp when message was sent (in milliseconds, for latency tracking)
    pub send_timestamp: Option<u64>,
    
    /// Request ID for request-response correlation
    pub request_id: Option<MessageId>,
    
    /// Timeout for the message (in milliseconds)
    pub timeout: Option<u64>,
    
    /// Sequence number for ordering
    pub sequence: u64,
    
    /// Checksum for integrity verification
    pub checksum: u32,
}

impl MessageHeader {
    /// Create a new message header
    pub fn new(
        id: MessageId,
        sender: ProcessId,
        receiver: ProcessId,
        msg_type: MessageType,
        payload_size: usize,
    ) -> Self {
        Self {
            id,
            sender,
            receiver,
            msg_type,
            priority: MessagePriority::default(),
            flags: MessageFlags::default(),
            payload_size,
            timestamp: get_current_timestamp(),
            send_timestamp: None,
            request_id: None,
            timeout: None,
            sequence: 0,
            checksum: 0,
        }
    }
    
    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set message flags
    pub fn with_flags(mut self, flags: MessageFlags) -> Self {
        self.flags = flags;
        self
    }
    
    /// Set request ID for responses
    pub fn with_request_id(mut self, request_id: MessageId) -> Self {
        self.request_id = Some(request_id);
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout = Some(timeout_ms);
        self
    }
    
    /// Set sequence number
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = sequence;
        self
    }
    
    /// Set send timestamp (called when message is actually sent)
    pub fn set_send_timestamp(&mut self) {
        self.send_timestamp = Some(get_current_timestamp());
    }
    
    /// Calculate latency from send to current time (in milliseconds)
    pub fn calculate_latency_ms(&self) -> Option<u64> {
        self.send_timestamp.map(|send_ts| {
            let current_time = get_current_timestamp();
            if current_time > send_ts {
                current_time - send_ts
            } else {
                0
            }
        })
    }
    
    /// Calculate latency from send to current time (in microseconds)
    pub fn calculate_latency_us(&self) -> Option<u32> {
        self.calculate_latency_ms().map(|ms| (ms * 1000) as u32)
    }
    
    /// Check if message has expired
    pub fn is_expired(&self) -> bool {
        if let Some(timeout) = self.timeout {
            let current_time = get_current_timestamp();
            current_time > self.timestamp + timeout
        } else {
            false
        }
    }
    
    /// Get message age in milliseconds
    pub fn age_ms(&self) -> u64 {
        let current_time = get_current_timestamp();
        current_time.saturating_sub(self.timestamp)
    }
    
    /// Calculate checksum for the header
    pub fn calculate_checksum(&self) -> u32 {
        // Simple checksum calculation (in production, use a proper algorithm)
        let mut checksum = 0u32;
        checksum ^= self.id.0 as u32;
        checksum ^= self.sender.0 as u32;
        checksum ^= self.receiver.0 as u32;
        checksum ^= self.payload_size as u32;
        checksum ^= self.timestamp as u32;
        checksum ^= self.sequence as u32;
        checksum
    }
    
    /// Update and verify checksum
    pub fn update_checksum(&mut self) {
        self.checksum = self.calculate_checksum();
    }
    
    /// Verify checksum integrity
    pub fn verify_checksum(&self) -> bool {
        self.checksum == self.calculate_checksum()
    }
}

impl core::fmt::Display for MessageHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Header[{}] {} -> {} | Type: {} | Priority: {} | Size: {} bytes | Age: {}ms",
               self.id,
               self.sender,
               self.receiver,
               self.msg_type,
               self.priority,
               self.payload_size,
               self.age_ms())
    }
}

/// Complete message structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Message header
    pub header: MessageHeader,
    
    /// Message payload
    pub payload: MessagePayload,
}

impl Message {
    /// Create a new message
    pub fn new(
        sender: ProcessId,
        receiver: ProcessId,
        msg_type: MessageType,
        payload: MessagePayload,
    ) -> Self {
        let id = super::generate_message_id();
        let payload_size = payload.size();
        
        let mut header = MessageHeader::new(id, sender, receiver, msg_type, payload_size);
        header.update_checksum();
        
        Self { header, payload }
    }
    
    /// Create a new message with specific priority
    pub fn new_with_priority(
        sender: ProcessId,
        receiver: ProcessId,
        msg_type: MessageType,
        priority: MessagePriority,
        payload: MessagePayload,
    ) -> Self {
        let mut msg = Self::new(sender, receiver, msg_type, payload);
        msg.header.priority = priority;
        msg.header.update_checksum();
        msg
    }
    
    /// Create a request message
    pub fn new_request(
        sender: ProcessId,
        receiver: ProcessId,
        payload: MessagePayload,
    ) -> Self {
        let mut msg = Self::new(sender, receiver, MessageType::Request, payload);
        msg.header.flags = MessageFlags::request();
        msg.header.update_checksum();
        msg
    }
    
    /// Create a response message
    pub fn new_response(
        sender: ProcessId,
        receiver: ProcessId,
        request_id: MessageId,
        payload: MessagePayload,
    ) -> Self {
        let mut msg = Self::new(sender, receiver, MessageType::Response, payload);
        msg.header.request_id = Some(request_id);
        msg.header.update_checksum();
        msg
    }
    
    /// Create a notification message
    pub fn new_notification(
        sender: ProcessId,
        receiver: ProcessId,
        payload: MessagePayload,
    ) -> Self {
        let mut msg = Self::new(sender, receiver, MessageType::Notification, payload);
        msg.header.flags = MessageFlags::notification();
        msg.header.update_checksum();
        msg
    }
    
    /// Create an error message
    pub fn new_error(
        sender: ProcessId,
        receiver: ProcessId,
        error_code: u32,
        error_message: &str,
    ) -> Self {
        let payload = MessagePayload::from_error(error_code, error_message);
        Self::new(sender, receiver, MessageType::Error, payload)
    }
    
    /// Get total message size (header + payload)
    pub fn total_size(&self) -> usize {
        core::mem::size_of::<MessageHeader>() + self.payload.size()
    }
    
    /// Check if message is valid
    pub fn is_valid(&self) -> bool {
        // Basic validation checks
        self.header.sender.is_valid() &&
        self.header.receiver.is_valid() &&
        self.header.payload_size == self.payload.size() &&
        self.header.verify_checksum() &&
        !self.header.is_expired()
    }
    
    /// Set message priority
    pub fn set_priority(&mut self, priority: MessagePriority) {
        self.header.priority = priority;
        self.header.update_checksum();
    }
    
    /// Set message flags
    pub fn set_flags(&mut self, flags: MessageFlags) {
        self.header.flags = flags;
        self.header.update_checksum();
    }
    
    /// Set timeout
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.header.timeout = Some(timeout_ms);
        self.header.update_checksum();
    }
    
    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        matches!(self.header.msg_type, MessageType::Request)
    }
    
    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        matches!(self.header.msg_type, MessageType::Response)
    }
    
    /// Check if this is a response to a specific request
    pub fn is_response_to(&self, request_id: MessageId) -> bool {
        self.is_response() && self.header.request_id == Some(request_id)
    }
    
    /// Get the payload as text if possible
    pub fn as_text(&self) -> Option<&str> {
        self.payload.as_text()
    }
    
    /// Get the payload as bytes if possible
    pub fn as_bytes(&self) -> Option<&[u8]> {
        self.payload.as_bytes()
    }
}

impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Message[{}]\n  Header: {}\n  Payload: {}",
               self.header.id,
               self.header,
               self.payload)
    }
}

/// Get current timestamp in milliseconds
/// This is a simplified implementation - in production, use proper time source
/// When determinism mode is enabled, uses virtualized time source
fn get_current_timestamp() -> u64 {
    // Check if determinism mode is enabled
    if crate::determinism::is_determinism_enabled() {
        // Use virtualized time source for deterministic behavior
        crate::determinism::get_virtual_time_ms()
    } else {
        // Use real time source (simulated counter for now)
        static TIMESTAMP_COUNTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
        TIMESTAMP_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
    }
}

/// Convenience functions for creating common message types

/// Create a simple text message
pub fn create_text_message(
    sender: ProcessId,
    receiver: ProcessId,
    text: &str,
) -> Message {
    Message::new_notification(sender, receiver, MessagePayload::from_text(text))
}

/// Create a data message
pub fn create_data_message(
    sender: ProcessId,
    receiver: ProcessId,
    data: &[u8],
) -> Message {
    Message::new_notification(sender, receiver, MessagePayload::from_data(data))
}

/// Create a signal message
pub fn create_signal_message(
    sender: ProcessId,
    receiver: ProcessId,
    signal: u32,
    data: u64,
) -> Message {
    Message::new_notification(sender, receiver, MessagePayload::from_signal(signal, data))
}

/// Create an error message
pub fn create_error_message(
    sender: ProcessId,
    receiver: ProcessId,
    error_code: u32,
    error_text: &str,
) -> Message {
    Message::new_error(sender, receiver, error_code, error_text)
}
