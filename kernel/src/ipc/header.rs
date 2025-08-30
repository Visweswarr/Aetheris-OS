/// IPC Header v2: PQC-Authenticated Message Headers
/// 
/// This module defines the enhanced IPC message header structure that includes
/// PQC authentication fields for secure inter-process communication.

use super::types::{MessageId, ProcessId, MessageType, MessagePriority, MessageFlags};
use crate::security::cap_v2::CapTokenV2;
use alloc::string::ToString;

/// MAC tag size for message authentication (16 bytes)
pub const MAC_TAG_SIZE: usize = 16;

/// Capability ID size (128 bits)
pub const CAP_ID_SIZE: usize = 16;

/// Authentication mode for IPC messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    /// Capability-only authentication (fast path)
    CapabilityOnly,
    
    /// Full authentication with message MAC
    FullAuth,
}

/// IPC Header v2 with PQC authentication support
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcHeaderV2 {
    /// Unique message identifier
    pub msg_id: MessageId,
    
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
    
    /// Message authentication code tag (16 bytes)
    pub mac_tag: [u8; MAC_TAG_SIZE],
    
    /// Capability token ID for authentication
    pub cap_id: [u8; CAP_ID_SIZE],
    
    /// Authentication mode
    pub auth_mode: AuthMode,
    
    /// Ephemeral session key for Kyber KEM (if using full auth)
    pub session_key: Option<[u8; 32]>,
}

impl IpcHeaderV2 {
    /// Create a new IPC header v2 with capability-only authentication
    pub fn new_capability_only(
        msg_id: MessageId,
        sender: ProcessId,
        receiver: ProcessId,
        msg_type: MessageType,
        payload_size: usize,
        cap_token: &CapTokenV2,
    ) -> Self {
        let now_ms = get_current_timestamp();
        
        Self {
            msg_id,
            sender,
            receiver,
            msg_type,
            priority: MessagePriority::default(),
            flags: MessageFlags::default(),
            payload_size,
            timestamp: now_ms,
            send_timestamp: None,
            request_id: None,
            timeout: None,
            sequence: 0,
            checksum: 0,
            mac_tag: [0u8; MAC_TAG_SIZE],
            cap_id: Self::extract_cap_id(cap_token),
            auth_mode: AuthMode::CapabilityOnly,
            session_key: None,
        }
    }
    
    /// Create a new IPC header v2 with full authentication
    pub fn new_full_auth(
        msg_id: MessageId,
        sender: ProcessId,
        receiver: ProcessId,
        msg_type: MessageType,
        payload_size: usize,
        cap_token: &CapTokenV2,
        session_key: [u8; 32],
    ) -> Self {
        let now_ms = get_current_timestamp();
        
        Self {
            msg_id,
            sender,
            receiver,
            msg_type,
            priority: MessagePriority::default(),
            flags: MessageFlags::default(),
            payload_size,
            timestamp: now_ms,
            send_timestamp: None,
            request_id: None,
            timeout: None,
            sequence: 0,
            checksum: 0,
            mac_tag: [0u8; MAC_TAG_SIZE], // Will be computed later
            cap_id: Self::extract_cap_id(cap_token),
            auth_mode: AuthMode::FullAuth,
            session_key: Some(session_key),
        }
    }
    
    /// Extract capability ID from a capability token
    fn extract_cap_id(cap_token: &CapTokenV2) -> [u8; CAP_ID_SIZE] {
        // For now, use a hash of the token header as the capability ID
        // In a real implementation, this would be a proper cryptographic hash
        let mut cap_id = [0u8; CAP_ID_SIZE];
        
        // Simple hash function for demonstration
        let header_bytes = cap_token.header.to_bytes();
        for (i, &byte) in header_bytes.iter().take(CAP_ID_SIZE).enumerate() {
            cap_id[i] = byte;
        }
        
        cap_id
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
    
    /// Set checksum
    pub fn with_checksum(mut self, checksum: u32) -> Self {
        self.checksum = checksum;
        self
    }
    
    /// Set MAC tag
    pub fn with_mac_tag(mut self, mac_tag: [u8; MAC_TAG_SIZE]) -> Self {
        self.mac_tag = mac_tag;
        self
    }
    
    /// Set send timestamp
    pub fn with_send_timestamp(mut self, timestamp: u64) -> Self {
        self.send_timestamp = Some(timestamp);
        self
    }
    
    /// Check if message requires full authentication
    pub fn requires_full_auth(&self) -> bool {
        self.auth_mode == AuthMode::FullAuth
    }
    
    /// Check if message can skip MAC (RT messages < 64B with appropriate scope)
    pub fn can_skip_mac(&self) -> bool {
        self.payload_size < 64 && self.flags.contains(MessageFlags::REALTIME)
    }
    
    /// Get the header size in bytes
    pub fn header_size(&self) -> usize {
        let base_size = core::mem::size_of::<Self>();
        if self.session_key.is_some() {
            base_size
        } else {
            base_size - 32 // Exclude session key if not present
        }
    }
    
    /// Convert to bytes for serialization
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Add basic fields
        bytes.extend_from_slice(&self.msg_id.0.to_le_bytes());
        bytes.extend_from_slice(&self.sender.0.to_le_bytes());
        bytes.extend_from_slice(&self.receiver.0.to_le_bytes());
        bytes.extend_from_slice(&(self.msg_type as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.priority as u32).to_le_bytes());
        bytes.extend_from_slice(&self.flags.bits().to_le_bytes());
        bytes.extend_from_slice(&self.payload_size.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        
        // Add optional fields
        if let Some(send_ts) = self.send_timestamp {
            bytes.extend_from_slice(&send_ts.to_le_bytes());
        } else {
            bytes.extend_from_slice(&0u64.to_le_bytes());
        }
        
        if let Some(req_id) = self.request_id {
            bytes.extend_from_slice(&req_id.0.to_le_bytes());
        } else {
            bytes.extend_from_slice(&0u64.to_le_bytes());
        }
        
        if let Some(timeout) = self.timeout {
            bytes.extend_from_slice(&timeout.to_le_bytes());
        } else {
            bytes.extend_from_slice(&0u64.to_le_bytes());
        }
        
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        
        // Add authentication fields
        bytes.extend_from_slice(&self.mac_tag);
        bytes.extend_from_slice(&self.cap_id);
        bytes.extend_from_slice(&(self.auth_mode as u32).to_le_bytes());
        
        if let Some(session_key) = self.session_key {
            bytes.extend_from_slice(&session_key);
        }
        
        bytes
    }
    
    /// Create header from bytes (deserialization)
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < core::mem::size_of::<Self>() - 32 {
            return None; // Too short
        }
        
        let mut offset = 0;
        
        // Read basic fields
        let msg_id = MessageId::new(u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?));
        offset += 8;
        
        let sender = ProcessId::new(u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?));
        offset += 8;
        
        let receiver = ProcessId::new(u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?));
        offset += 8;
        
        let msg_type = MessageType::from_u32(u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?))?;
        offset += 4;
        
        let priority = MessagePriority::from_u32(u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?))?;
        offset += 4;
        
        let flags = MessageFlags::from_bits(u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?))?;
        offset += 4;
        
        let payload_size = usize::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;
        
        let timestamp = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;
        
        let send_timestamp = {
            let ts = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
            offset += 8;
            if ts > 0 { Some(ts) } else { None }
        };
        
        let request_id = {
            let req_id = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
            offset += 8;
            if req_id > 0 { Some(MessageId::new(req_id)) } else { None }
        };
        
        let timeout = {
            let to = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
            offset += 8;
            if to > 0 { Some(to) } else { None }
        };
        
        let sequence = u64::from_le_bytes(bytes[offset..offset + 8].try_into().ok()?);
        offset += 8;
        
        let checksum = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?);
        offset += 4;
        
        // Read authentication fields
        let mac_tag = bytes[offset..offset + MAC_TAG_SIZE].try_into().ok()?;
        offset += MAC_TAG_SIZE;
        
        let cap_id = bytes[offset..offset + CAP_ID_SIZE].try_into().ok()?;
        offset += CAP_ID_SIZE;
        
        let auth_mode = AuthMode::from_u32(u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?))?;
        offset += 4;
        
        // Read session key if present
        let session_key = if offset + 32 <= bytes.len() {
            let key = bytes[offset..offset + 32].try_into().ok()?;
            offset += 32;
            Some(key)
        } else {
            None
        };
        
        Some(Self {
            msg_id,
            sender,
            receiver,
            msg_type,
            priority,
            flags,
            payload_size,
            timestamp,
            send_timestamp,
            request_id,
            timeout,
            sequence,
            checksum,
            mac_tag,
            cap_id,
            auth_mode,
            session_key,
        })
    }
}

impl Default for IpcHeaderV2 {
    fn default() -> Self {
        Self {
            msg_id: MessageId::new(0),
            sender: ProcessId::new(0),
            receiver: ProcessId::new(0),
            msg_type: MessageType::Data,
            priority: MessagePriority::Normal,
            flags: MessageFlags::default(),
            payload_size: 0,
            timestamp: 0,
            send_timestamp: None,
            request_id: None,
            timeout: None,
            sequence: 0,
            checksum: 0,
            mac_tag: [0u8; MAC_TAG_SIZE],
            cap_id: [0u8; CAP_ID_SIZE],
            auth_mode: AuthMode::CapabilityOnly,
            session_key: None,
        }
    }
}

impl core::fmt::Display for IpcHeaderV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "IPC Header v2 [{} -> {}, type={:?}, size={}, auth={:?}]",
                self.sender, self.receiver, self.msg_type, self.payload_size, self.auth_mode)
    }
}

/// Helper function to get current timestamp
fn get_current_timestamp() -> u64 {
    // This would call the actual time function in a real implementation
    // For now, return a placeholder
    0
}

/// Helper function to convert MessageType to u32
impl MessageType {
    fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(MessageType::Data),
            1 => Some(MessageType::Control),
            2 => Some(MessageType::Request),
            3 => Some(MessageType::Response),
            4 => Some(MessageType::Error),
            _ => None,
        }
    }
}

/// Helper function to convert MessagePriority to u32
impl MessagePriority {
    fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(MessagePriority::Low),
            1 => Some(MessagePriority::Normal),
            2 => Some(MessagePriority::High),
            3 => Some(MessagePriority::Critical),
            _ => None,
        }
    }
}

/// Helper function to convert AuthMode to u32
impl AuthMode {
    fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(AuthMode::CapabilityOnly),
            1 => Some(AuthMode::FullAuth),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::cap_v2::{CapTokenV2, CapTokenHeader, CapTokenSignature, CapTokenMetadata, SignatureAlgorithm, scope_v2};
    
    #[test]
    fn test_header_creation() {
        let cap_token = create_test_cap_token();
        let header = IpcHeaderV2::new_capability_only(
            MessageId::new(123),
            ProcessId::new(100),
            ProcessId::new(200),
            MessageType::Data,
            1024,
            &cap_token,
        );
        
        assert_eq!(header.auth_mode, AuthMode::CapabilityOnly);
        assert_eq!(header.session_key, None);
        assert_eq!(header.payload_size, 1024);
    }
    
    #[test]
    fn test_header_serialization() {
        let cap_token = create_test_cap_token();
        let header = IpcHeaderV2::new_capability_only(
            MessageId::new(123),
            ProcessId::new(100),
            ProcessId::new(200),
            MessageType::Data,
            1024,
            &cap_token,
        );
        
        let bytes = header.to_bytes();
        let deserialized = IpcHeaderV2::from_bytes(&bytes);
        
        assert!(deserialized.is_some());
        let deserialized = deserialized.unwrap();
        assert_eq!(header.msg_id, deserialized.msg_id);
        assert_eq!(header.sender, deserialized.sender);
        assert_eq!(header.receiver, deserialized.receiver);
    }
    
    #[test]
    fn test_mac_skip_logic() {
        let cap_token = create_test_cap_token();
        let mut header = IpcHeaderV2::new_capability_only(
            MessageId::new(123),
            ProcessId::new(100),
            ProcessId::new(200),
            MessageType::Data,
            32, // Small payload
            &cap_token,
        );
        
        // Add REALTIME flag
        header.flags = MessageFlags::REALTIME;
        
        assert!(header.can_skip_mac());
        
        // Large payload should not skip MAC even with REALTIME
        header.payload_size = 128;
        assert!(!header.can_skip_mac());
    }
    
    fn create_test_cap_token() -> CapTokenV2 {
        let header = CapTokenHeader::new(
            "did:example:test".to_string(),
            100,
            200,
            scope_v2::SEND,
            0,
            3600000,
            0x1234567890abcdef,
            [0u8; 32],
        );
        
        let signature = CapTokenSignature::new(
            vec![0u8; 64], // Placeholder signature
            None,
            SignatureAlgorithm::Dilithium2,
        );
        
        let metadata = CapTokenMetadata::new(0, vec![]);
        
        CapTokenV2 {
            header,
            signature,
            metadata,
        }
    }
}
