/// IPC Stream Authentication and Key Rotation Module
/// 
/// This module provides an IPC stream abstraction that binds a sender↔receiver
/// with Kyber-derived session keys and optional rolling nonce windows.
/// Implements key rotation every N messages or T minutes with grace overlap
/// to enforce forward secrecy.

use super::types::{MessageId, ProcessId, MessageType, MessagePriority, MessageFlags};
use super::header::IpcHeaderV2;
use crate::crypto::pqc::kyber::{KyberKem, KyberParameterSet, KyberPublicKey, KyberSecretKey};
use crate::crypto::pqc::kyber::KyberParameterSet::Kyber768;
use crate::secman::keys::{KeyId, SessionKey, IssuerKey};
use crate::secman::audit::{audit_log, AuditEvent, AuditLevel};
use crate::{kprintln, klog, kprintln};
use crate::log::Level;
use core::time::Duration;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use alloc::collections::{HashMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

//=============================================================================
// STREAM CONFIGURATION CONSTANTS
//=============================================================================

/// Default session key rotation interval (5 minutes)
pub const DEFAULT_ROTATION_INTERVAL: Duration = Duration::from_secs(300);

/// Default message count threshold for rotation
pub const DEFAULT_MESSAGE_ROTATION_THRESHOLD: usize = 1000;

/// Grace period for key rotation (allows ongoing IPC streams to complete)
pub const KEY_ROTATION_GRACE_PERIOD: Duration = Duration::from_secs(30);

/// Rolling nonce window size (number of nonces to accept)
pub const ROLLING_NONCE_WINDOW_SIZE: usize = 64;

/// Maximum number of active streams per process pair
pub const MAX_STREAMS_PER_PAIR: usize = 4;

/// Session key derivation info length (32 bytes)
pub const SESSION_KEY_INFO_LENGTH: usize = 32;

/// MAC tag size for message authentication (16 bytes)
pub const MAC_TAG_SIZE: usize = 16;

//=============================================================================
// STREAM STRUCTURES AND TYPES
//=============================================================================

/// Unique identifier for an IPC stream
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamId {
    /// Sender process ID
    pub sender: ProcessId,
    /// Receiver process ID
    pub receiver: ProcessId,
    /// Stream sequence number
    pub sequence: u64,
}

impl StreamId {
    /// Create a new stream ID
    pub fn new(sender: ProcessId, receiver: ProcessId, sequence: u64) -> Self {
        Self {
            sender,
            receiver,
            sequence,
        }
    }
    
    /// Get the process pair key for this stream
    pub fn process_pair_key(&self) -> (ProcessId, ProcessId) {
        if self.sender < self.receiver {
            (self.sender, self.receiver)
        } else {
            (self.receiver, self.sender)
        }
    }
}

impl core::fmt::Display for StreamId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Stream({}->{}:{}", self.sender, self.receiver, self.sequence)
    }
}

/// Session key information for an IPC stream
#[derive(Debug, Clone)]
pub struct StreamSessionKey {
    /// Unique key identifier
    pub key_id: KeyId,
    /// Kyber public key for this session
    pub public_key: KyberPublicKey,
    /// Kyber secret key for this session
    pub secret_key: KyberSecretKey,
    /// Session key derivation info
    pub info: [u8; SESSION_KEY_INFO_LENGTH],
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Message count since creation
    pub message_count: usize,
    /// Whether this key is active
    pub is_active: bool,
}

impl StreamSessionKey {
    /// Create a new session key
    pub fn new(
        public_key: KyberPublicKey,
        secret_key: KyberSecretKey,
        info: [u8; SESSION_KEY_INFO_LENGTH],
        ttl: Duration,
    ) -> Self {
        let now = get_current_timestamp();
        let expires_at = now + ttl.as_millis() as u64;
        
        Self {
            key_id: KeyId::random(),
            public_key,
            secret_key,
            info,
            created_at: now,
            expires_at,
            message_count: 0,
            is_active: true,
        }
    }
    
    /// Check if the key is expired
    pub fn is_expired(&self) -> bool {
        get_current_timestamp() > self.expires_at
    }
    
    /// Check if the key should be rotated based on message count
    pub fn should_rotate_by_count(&self) -> bool {
        self.message_count >= DEFAULT_MESSAGE_ROTATION_THRESHOLD
    }
    
    /// Check if the key should be rotated based on time
    pub fn should_rotate_by_time(&self) -> bool {
        let now = get_current_timestamp();
        let rotation_time = self.created_at + DEFAULT_ROTATION_INTERVAL.as_millis() as u64;
        now >= rotation_time
    }
    
    /// Increment message count
    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
    }
    
    /// Derive MAC key from this session key
    pub fn derive_mac_key(&self, context: &[u8]) -> Result<[u8; 32], String> {
        // Use HKDF-like derivation from the session secret
        // This is a simplified implementation - in production, use proper HKDF
        let mut mac_key = [0u8; 32];
        let secret_data = self.secret_key.as_bytes();
        
        if secret_data.len() >= 32 {
            mac_key.copy_from_slice(&secret_data[..32]);
        } else {
            // Pad with zeros if secret is shorter than 32 bytes
            mac_key[..secret_data.len()].copy_from_slice(secret_data);
        }
        
        // XOR with context for additional entropy
        for (i, &context_byte) in context.iter().enumerate() {
            if i < 32 {
                mac_key[i] ^= context_byte;
            }
        }
        
        Ok(mac_key)
    }
}

/// Rolling nonce window for replay protection
#[derive(Debug, Clone)]
pub struct RollingNonceWindow {
    /// Nonce values in the window
    pub nonces: VecDeque<u64>,
    /// Maximum window size
    pub max_size: usize,
    /// Base nonce for this window
    pub base_nonce: u64,
}

impl RollingNonceWindow {
    /// Create a new rolling nonce window
    pub fn new(max_size: usize, base_nonce: u64) -> Self {
        Self {
            nonces: VecDeque::with_capacity(max_size),
            max_size,
            base_nonce,
        }
    }
    
    /// Check if a nonce is valid and add it to the window
    pub fn check_and_add_nonce(&mut self, nonce: u64) -> bool {
        // Check if nonce is within valid range
        if nonce < self.base_nonce {
            return false; // Nonce too old
        }
        
        // Check if nonce is already in the window
        if self.nonces.contains(&nonce) {
            return false; // Replay detected
        }
        
        // Add nonce to window
        self.nonces.push_back(nonce);
        
        // Maintain window size
        if self.nonces.len() > self.max_size {
            self.nonces.pop_front();
            self.base_nonce += 1;
        }
        
        true
    }
    
    /// Get the current window size
    pub fn len(&self) -> usize {
        self.nonces.len()
    }
    
    /// Check if the window is empty
    pub fn is_empty(&self) -> bool {
        self.nonces.is_empty()
    }
    
    /// Clear the window
    pub fn clear(&mut self) {
        self.nonces.clear();
    }
}

/// IPC stream state and authentication
#[derive(Debug)]
pub struct IpcStream {
    /// Unique stream identifier
    pub id: StreamId,
    /// Current active session key
    pub current_key: StreamSessionKey,
    /// Previous session key (for grace period)
    pub previous_key: Option<StreamSessionKey>,
    /// Rolling nonce window for replay protection
    pub nonce_window: RollingNonceWindow,
    /// Total messages sent on this stream
    pub messages_sent: u64,
    /// Total messages received on this stream
    pub messages_received: u64,
    /// Last activity timestamp
    pub last_activity: u64,
    /// Stream creation timestamp
    pub created_at: u64,
    /// Whether the stream is active
    pub is_active: bool,
}

impl IpcStream {
    /// Create a new IPC stream
    pub fn new(
        sender: ProcessId,
        receiver: ProcessId,
        sequence: u64,
        initial_key: StreamSessionKey,
    ) -> Self {
        let now = get_current_timestamp();
        let base_nonce = get_random_nonce();
        
        Self {
            id: StreamId::new(sender, receiver, sequence),
            current_key: initial_key,
            previous_key: None,
            nonce_window: RollingNonceWindow::new(ROLLING_NONCE_WINDOW_SIZE, base_nonce),
            messages_sent: 0,
            messages_received: 0,
            last_activity: now,
            created_at: now,
            is_active: true,
        }
    }
    
    /// Check if the stream needs key rotation
    pub fn needs_rotation(&self) -> bool {
        self.current_key.should_rotate_by_count() || 
        self.current_key.should_rotate_by_time() ||
        self.current_key.is_expired()
    }
    
    /// Rotate the session key
    pub fn rotate_key(&mut self, new_key: StreamSessionKey) -> Result<(), String> {
        // Move current key to previous key for grace period
        let old_key = core::mem::replace(&mut self.current_key, new_key);
        self.previous_key = Some(old_key);
        
        // Update activity timestamp
        self.last_activity = get_current_timestamp();
        
        klog!(INFO, "[IPC-STREAM] Rotated key for stream {}: new_key_id={}", 
              self.id, self.current_key.key_id);
        
        Ok(())
    }
    
    /// Clean up expired keys
    pub fn cleanup_expired_keys(&mut self) {
        // Check if previous key has expired grace period
        if let Some(ref mut prev_key) = self.previous_key {
            let grace_expiry = prev_key.expires_at + KEY_ROTATION_GRACE_PERIOD.as_millis() as u64;
            if get_current_timestamp() > grace_expiry {
                klog!(INFO, "[IPC-STREAM] Removing expired previous key {} from stream {}", 
                      prev_key.key_id, self.id);
                self.previous_key = None;
            }
        }
    }
    
    /// Validate message nonce and update window
    pub fn validate_nonce(&mut self, nonce: u64) -> bool {
        let valid = self.nonce_window.check_and_add_nonce(nonce);
        
        if !valid {
            audit_log(
                AuditEvent::StreamReplayAttempt,
                AuditLevel::WARNING,
                &format!("Replay attempt detected on stream {} with nonce {}", self.id, nonce),
            );
        }
        
        valid
    }
    
    /// Generate MAC for a message
    pub fn generate_mac(&self, message: &[u8], nonce: u64) -> Result<[u8; MAC_TAG_SIZE], String> {
        let context = format!("{}:{}:{}", self.id.sender, self.id.receiver, nonce);
        let mac_key = self.current_key.derive_mac_key(context.as_bytes())?;
        
        // Use Blake3 for MAC calculation (stub implementation)
        // In production, this would use a proper MAC function
        let mut mac_tag = [0u8; MAC_TAG_SIZE];
        let hash_input = [message, &mac_key, &nonce.to_le_bytes()].concat();
        
        // Simple hash-based MAC (stub)
        for (i, &byte) in hash_input.iter().enumerate() {
            if i < MAC_TAG_SIZE {
                mac_tag[i] = byte.wrapping_add(i as u8);
            }
        }
        
        Ok(mac_tag)
    }
    
    /// Verify MAC for a message
    pub fn verify_mac(&self, message: &[u8], nonce: u64, mac_tag: &[u8; MAC_TAG_SIZE]) -> bool {
        match self.generate_mac(message, nonce) {
            Ok(expected_mac) => {
                let valid = expected_mac == *mac_tag;
                if !valid {
                    audit_log(
                        AuditEvent::StreamMacFailure,
                        AuditLevel::WARNING,
                        &format!("MAC verification failed on stream {} with nonce {}", self.id, nonce),
                    );
                }
                valid
            }
            Err(_) => {
                audit_log(
                    AuditEvent::StreamMacFailure,
                    AuditLevel::ERROR,
                    &format!("Failed to generate MAC for verification on stream {}", self.id),
                );
                false
            }
        }
    }
    
    /// Process a message (validate nonce, verify MAC, update counters)
    pub fn process_message(&mut self, message: &[u8], nonce: u64, mac_tag: &[u8; MAC_TAG_SIZE]) -> bool {
        // Validate nonce first
        if !self.validate_nonce(nonce) {
            return false;
        }
        
        // Verify MAC
        if !self.verify_mac(message, nonce, mac_tag) {
            return false;
        }
        
        // Update counters and activity
        self.messages_received += 1;
        self.current_key.increment_message_count();
        self.last_activity = get_current_timestamp();
        
        true
    }
    
    /// Send a message (generate MAC, update counters)
    pub fn send_message(&mut self, message: &[u8], nonce: u64) -> Result<[u8; MAC_TAG_SIZE], String> {
        // Generate MAC
        let mac_tag = self.generate_mac(message, nonce)?;
        
        // Update counters and activity
        self.messages_sent += 1;
        self.current_key.increment_message_count();
        self.last_activity = get_current_timestamp();
        
        Ok(mac_tag)
    }
}

//=============================================================================
// STREAM MANAGEMENT
//=============================================================================

/// Stream manager for handling multiple IPC streams
pub struct StreamManager {
    /// Active streams indexed by stream ID
    streams: HashMap<StreamId, IpcStream>,
    /// Streams indexed by process pair for quick lookup
    process_pair_streams: HashMap<(ProcessId, ProcessId), Vec<StreamId>>,
    /// Global stream counter
    stream_counter: AtomicU64,
    /// Statistics
    stats: StreamManagerStats,
}

impl StreamManager {
    /// Create a new stream manager
    pub fn new() -> Self {
        Self {
            streams: HashMap::new(),
            process_pair_streams: HashMap::new(),
            stream_counter: AtomicU64::new(1),
            stats: StreamManagerStats::new(),
        }
    }
    
    /// Create a new stream between two processes
    pub fn create_stream(
        &mut self,
        sender: ProcessId,
        receiver: ProcessId,
    ) -> Result<StreamId, String> {
        // Check if we've reached the limit for this process pair
        let pair_key = if sender < receiver {
            (sender, receiver)
        } else {
            (receiver, sender)
        };
        
        let existing_streams = self.process_pair_streams.get(&pair_key).map(|v| v.len()).unwrap_or(0);
        if existing_streams >= MAX_STREAMS_PER_PAIR {
            return Err(format!("Maximum streams ({}) reached for process pair ({}, {})", 
                             MAX_STREAMS_PER_PAIR, sender, receiver));
        }
        
        // Generate new sequence number
        let sequence = self.stream_counter.fetch_add(1, Ordering::Relaxed);
        
        // Generate new session key
        let session_key = self.generate_session_key()?;
        
        // Create stream
        let stream_id = StreamId::new(sender, receiver, sequence);
        let stream = IpcStream::new(sender, receiver, sequence, session_key);
        
        // Store stream
        self.streams.insert(stream_id.clone(), stream);
        
        // Update process pair index
        self.process_pair_streams
            .entry(pair_key)
            .or_insert_with(Vec::new)
            .push(stream_id.clone());
        
        // Update statistics
        self.stats.streams_created += 1;
        self.stats.active_streams += 1;
        
        klog!(INFO, "[STREAM-MANAGER] Created stream {} between processes {} and {}", 
              stream_id, sender, receiver);
        
        Ok(stream_id)
    }
    
    /// Get a stream by ID
    pub fn get_stream(&self, stream_id: &StreamId) -> Option<&IpcStream> {
        self.streams.get(stream_id)
    }
    
    /// Get a mutable reference to a stream
    pub fn get_stream_mut(&mut self, stream_id: &StreamId) -> Option<&mut IpcStream> {
        self.streams.get_mut(stream_id)
    }
    
    /// Remove a stream
    pub fn remove_stream(&mut self, stream_id: &StreamId) -> bool {
        if let Some(stream) = self.streams.remove(stream_id) {
            // Remove from process pair index
            let pair_key = stream.id.process_pair_key();
            if let Some(streams) = self.process_pair_streams.get_mut(&pair_key) {
                streams.retain(|id| id != stream_id);
            }
            
            // Update statistics
            self.stats.active_streams -= 1;
            self.stats.streams_destroyed += 1;
            
            klog!(INFO, "[STREAM-MANAGER] Removed stream {}", stream_id);
            true
        } else {
            false
        }
    }
    
    /// Rotate keys for all streams that need rotation
    pub fn rotate_keys(&mut self) -> usize {
        let mut rotated_count = 0;
        let mut streams_to_remove = Vec::new();
        
        for (stream_id, stream) in &mut self.streams {
            if stream.needs_rotation() {
                match self.generate_session_key() {
                    Ok(new_key) => {
                        if let Err(e) = stream.rotate_key(new_key) {
                            klog!(ERROR, "[STREAM-MANAGER] Failed to rotate key for stream {}: {}", 
                                  stream_id, e);
                            streams_to_remove.push(stream_id.clone());
                        } else {
                            rotated_count += 1;
                        }
                    }
                    Err(e) => {
                        klog!(ERROR, "[STREAM-MANAGER] Failed to generate new key for stream {}: {}", 
                              stream_id, e);
                        streams_to_remove.push(stream_id.clone());
                    }
                }
            }
            
            // Clean up expired keys
            stream.cleanup_expired_keys();
        }
        
        // Remove streams that failed key rotation
        for stream_id in streams_to_remove {
            self.remove_stream(&stream_id);
        }
        
        if rotated_count > 0 {
            klog!(INFO, "[STREAM-MANAGER] Rotated keys for {} streams", rotated_count);
        }
        
        rotated_count
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> &StreamManagerStats {
        &self.stats
    }
    
    /// Generate a new session key
    fn generate_session_key(&self) -> Result<StreamSessionKey, String> {
        // Generate Kyber keypair
        let (public_key, secret_key) = KyberKem::generate_keypair(Kyber768)
            .map_err(|e| format!("Failed to generate Kyber keypair: {}", e))?;
        
        // Generate random info for key derivation
        let mut info = [0u8; SESSION_KEY_INFO_LENGTH];
        let random_data = get_random_bytes(SESSION_KEY_INFO_LENGTH);
        info.copy_from_slice(&random_data[..SESSION_KEY_INFO_LENGTH]);
        
        // Create session key with default TTL
        let session_key = StreamSessionKey::new(
            public_key,
            secret_key,
            info,
            DEFAULT_ROTATION_INTERVAL,
        );
        
        Ok(session_key)
    }
}

/// Stream manager statistics
#[derive(Debug, Clone)]
pub struct StreamManagerStats {
    /// Total streams created
    pub streams_created: u64,
    /// Total streams destroyed
    pub streams_destroyed: u64,
    /// Currently active streams
    pub active_streams: usize,
    /// Total key rotations performed
    pub key_rotations: u64,
    /// Total messages processed
    pub messages_processed: u64,
    /// Total MAC failures
    pub mac_failures: u64,
    /// Total replay attempts
    pub replay_attempts: u64,
}

impl StreamManagerStats {
    /// Create new statistics
    pub const fn new() -> Self {
        Self {
            streams_created: 0,
            streams_destroyed: 0,
            active_streams: 0,
            key_rotations: 0,
            messages_processed: 0,
            mac_failures: 0,
            replay_attempts: 0,
        }
    }
}

//=============================================================================
// GLOBAL STREAM MANAGER
//=============================================================================

lazy_static! {
    /// Global stream manager instance
    static ref STREAM_MANAGER: Mutex<StreamManager> = Mutex::new(StreamManager::new());
}

/// Get the global stream manager
pub fn get_stream_manager() -> &'static Mutex<StreamManager> {
    &STREAM_MANAGER
}

/// Create a new IPC stream
pub fn create_stream(sender: ProcessId, receiver: ProcessId) -> Result<StreamId, String> {
    let mut manager = get_stream_manager().lock();
    manager.create_stream(sender, receiver)
}

/// Get a stream by ID
pub fn get_stream(stream_id: &StreamId) -> Option<IpcStream> {
    let manager = get_stream_manager().lock();
    manager.get_stream(stream_id).cloned()
}

/// Remove a stream
pub fn remove_stream(stream_id: &StreamId) -> bool {
    let mut manager = get_stream_manager().lock();
    manager.remove_stream(stream_id)
}

/// Rotate keys for all streams
pub fn rotate_all_keys() -> usize {
    let mut manager = get_stream_manager().lock();
    manager.rotate_keys()
}

/// Get stream manager statistics
pub fn get_stream_stats() -> StreamManagerStats {
    let manager = get_stream_manager().lock();
    manager.get_stats().clone()
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

/// Get current timestamp in milliseconds
fn get_current_timestamp() -> u64 {
    // This would use the system clock in a real implementation
    // For now, use a simple counter
    static TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(0);
    TIMESTAMP_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Get random bytes
fn get_random_bytes(length: usize) -> Vec<u8> {
    // This would use the system RNG in a real implementation
    // For now, return zeros
    vec![0u8; length]
}

/// Get random nonce
fn get_random_nonce() -> u64 {
    // This would use the system RNG in a real implementation
    // For now, use timestamp-based nonce
    get_current_timestamp()
}

//=============================================================================
// TESTS
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stream_creation() {
        let mut manager = StreamManager::new();
        let stream_id = manager.create_stream(1, 2).unwrap();
        
        assert_eq!(stream_id.sender, 1);
        assert_eq!(stream_id.receiver, 2);
        assert_eq!(stream_id.sequence, 1);
        assert_eq!(manager.get_stats().active_streams, 1);
    }
    
    #[test]
    fn test_stream_key_rotation() {
        let mut manager = StreamManager::new();
        let stream_id = manager.create_stream(1, 2).unwrap();
        
        // Force key rotation by setting message count
        if let Some(stream) = manager.get_stream_mut(&stream_id) {
            stream.current_key.message_count = DEFAULT_MESSAGE_ROTATION_THRESHOLD;
        }
        
        let rotated_count = manager.rotate_keys();
        assert_eq!(rotated_count, 1);
    }
    
    #[test]
    fn test_nonce_window() {
        let mut window = RollingNonceWindow::new(4, 100);
        
        // Valid nonces
        assert!(window.check_and_add_nonce(100));
        assert!(window.check_and_add_nonce(101));
        assert!(window.check_and_add_nonce(102));
        assert!(window.check_and_add_nonce(103));
        
        // Window should be full
        assert_eq!(window.len(), 4);
        
        // Adding another nonce should shift the window
        assert!(window.check_and_add_nonce(104));
        assert_eq!(window.len(), 4);
        assert_eq!(window.base_nonce, 101);
        
        // Old nonce should be rejected
        assert!(!window.check_and_add_nonce(100));
    }
    
    #[test]
    fn test_mac_generation_and_verification() {
        let mut manager = StreamManager::new();
        let stream_id = manager.create_stream(1, 2).unwrap();
        
        if let Some(stream) = manager.get_stream_mut(&stream_id) {
            let message = b"test message";
            let nonce = 12345;
            
            // Generate MAC
            let mac_tag = stream.generate_mac(message, nonce).unwrap();
            
            // Verify MAC
            assert!(stream.verify_mac(message, nonce, &mac_tag));
            
            // Verify with wrong MAC
            let wrong_mac = [0u8; MAC_TAG_SIZE];
            assert!(!stream.verify_mac(message, nonce, &wrong_mac));
        }
    }
}
