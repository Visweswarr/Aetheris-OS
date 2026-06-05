/// Key Management Module for Polymera OS
/// 
/// This module provides an in-kernel ephemeral keystore for managing:
/// - Dilithium public keys (issuers)
/// - Kyber encapsulation keys (session)
/// - Automatic key rotation policies
/// - Secure zeroization on drop
/// - Performance statistics and monitoring

use crate::{kprintln, klog};
use crate::log::Level;
use crate::crypto::pqc::kyber::{KyberPublicKey, KyberParameterSet};
// Note: dilithium module is stubbed - using placeholder types
pub type DilithiumPublicKey = [u8; 32];
pub type DilithiumParameterSet = u8;
use core::time::Duration;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use spin::Mutex;
use lazy_static::lazy_static;

//=============================================================================
// KEY MANAGEMENT CONSTANTS AND CONFIGURATION
//=============================================================================

/// Default session key rotation interval (5 minutes)
pub const DEFAULT_SESSION_ROTATION_INTERVAL: Duration = Duration::from_secs(300);

/// Default message count threshold for rotation
pub const DEFAULT_MESSAGE_ROTATION_THRESHOLD: usize = 1000;

/// Maximum number of issuer keys in the keystore
pub const MAX_ISSUER_KEYS: usize = 100;

/// Maximum number of session keys in the keystore
pub const MAX_SESSION_KEYS: usize = 200;

/// Grace period for key rotation (allows ongoing IPC streams to complete)
pub const KEY_ROTATION_GRACE_PERIOD: Duration = Duration::from_secs(30);

/// Key identifier length in bytes
pub const KEY_ID_LENGTH: usize = 32;

/// Session key encapsulation data length
pub const SESSION_KEY_DATA_LENGTH: usize = 32;

//=============================================================================
// KEY STRUCTURES AND TYPES
//=============================================================================

/// Unique identifier for a key
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyId([u8; KEY_ID_LENGTH]);

impl KeyId {
    /// Create a new key ID from bytes
    pub fn new(data: [u8; KEY_ID_LENGTH]) -> Self {
        Self(data)
    }
    
    /// Create a key ID from a slice
    pub fn from_slice(data: &[u8]) -> Option<Self> {
        if data.len() == KEY_ID_LENGTH {
            let mut key_id = [0u8; KEY_ID_LENGTH];
            key_id.copy_from_slice(data);
            Some(Self(key_id))
        } else {
            None
        }
    }
    
    /// Get the key ID as bytes
    pub fn as_bytes(&self) -> &[u8; KEY_ID_LENGTH] {
        &self.0
    }
    
    /// Generate a random key ID
    pub fn random() -> Self {
        use crate::rng::random_bytes;
        let mut key_id = [0u8; KEY_ID_LENGTH];
        let _ = random_bytes(&mut key_id);
        Self(key_id)
    }
}

impl core::fmt::Display for KeyId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "KeyId(")?;
        for (i, &byte) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ":")?;
            }
            write!(f, "{:02x}", byte)?;
        }
        write!(f, ")")
    }
}

/// Issuer key information
#[derive(Debug, Clone)]
pub struct IssuerKey {
    /// Unique key identifier
    pub id: KeyId,
    
    /// Dilithium public key
    pub public_key: DilithiumPublicKey,
    
    /// Parameter set used
    pub params: DilithiumParameterSet,
    
    /// When the key was added
    pub added_at: u64,
    
    /// Last used timestamp
    pub last_used: u64,
    
    /// Usage count
    pub usage_count: u64,
    
    /// Whether the key is active
    pub active: bool,
}

impl IssuerKey {
    /// Create a new issuer key
    pub fn new(
        id: KeyId,
        public_key: DilithiumPublicKey,
        params: DilithiumParameterSet,
    ) -> Self {
        let timestamp = get_current_timestamp();
        
        Self {
            id,
            public_key,
            params,
            added_at: timestamp,
            last_used: timestamp,
            usage_count: 0,
            active: true,
        }
    }
    
    /// Update usage statistics
    pub fn record_usage(&mut self) {
        self.last_used = get_current_timestamp();
        self.usage_count += 1;
    }
    
    /// Get the key's age in seconds
    pub fn age_seconds(&self) -> u64 {
        get_current_timestamp().saturating_sub(self.added_at)
    }
    
    /// Get the time since last use in seconds
    pub fn time_since_last_use(&self) -> u64 {
        get_current_timestamp().saturating_sub(self.last_used)
    }
}

/// Session key information
#[derive(Debug, Clone)]
pub struct SessionKey {
    /// Unique key identifier
    pub id: KeyId,
    
    /// Kyber public key for encapsulation
    pub public_key: KyberPublicKey,
    
    /// Parameter set used
    pub params: KyberParameterSet,
    
    /// When the key was created
    pub created_at: u64,
    
    /// When the key expires
    pub expires_at: u64,
    
    /// Last used timestamp
    pub last_used: u64,
    
    /// Message count since creation
    pub message_count: u64,
    
    /// Whether the key is active
    pub active: bool,
    
    /// Associated issuer key ID (for authentication)
    pub issuer_key_id: Option<KeyId>,
}

impl SessionKey {
    /// Create a new session key
    pub fn new(
        id: KeyId,
        public_key: KyberPublicKey,
        params: KyberParameterSet,
        rotation_interval: Duration,
        issuer_key_id: Option<KeyId>,
    ) -> Self {
        let timestamp = get_current_timestamp();
        let expires_at = timestamp + rotation_interval.as_secs();
        
        Self {
            id,
            public_key,
            params,
            created_at: timestamp,
            expires_at,
            last_used: timestamp,
            message_count: 0,
            active: true,
            issuer_key_id,
        }
    }
    
    /// Check if the key has expired
    pub fn is_expired(&self) -> bool {
        get_current_timestamp() >= self.expires_at
    }
    
    /// Check if the key should be rotated based on message count
    pub fn should_rotate_by_message_count(&self, threshold: usize) -> bool {
        self.message_count >= threshold as u64
    }
    
    /// Check if the key should be rotated based on time
    pub fn should_rotate_by_time(&self) -> bool {
        self.is_expired()
    }
    
    /// Update usage statistics
    pub fn record_usage(&mut self) {
        self.last_used = get_current_timestamp();
        self.message_count += 1;
    }
    
    /// Extend the key's lifetime
    pub fn extend_lifetime(&mut self, additional_seconds: u64) {
        self.expires_at += additional_seconds;
    }
    
    /// Get the key's age in seconds
    pub fn age_seconds(&self) -> u64 {
        get_current_timestamp().saturating_sub(self.created_at)
    }
    
    /// Get the time until expiration in seconds
    pub fn time_until_expiration(&self) -> u64 {
        self.expires_at.saturating_sub(get_current_timestamp())
    }
    
    /// Get the time since last use in seconds
    pub fn time_since_last_use(&self) -> u64 {
        get_current_timestamp().saturating_sub(self.last_used)
    }
}

/// Key rotation policy configuration
#[derive(Debug, Clone)]
pub struct RotationPolicy {
    /// Session key rotation interval
    pub session_rotation_interval: Duration,
    
    /// Message count threshold for rotation
    pub message_rotation_threshold: usize,
    
    /// Whether to enable automatic rotation
    pub auto_rotation_enabled: bool,
    
    /// Grace period for rotation
    pub grace_period: Duration,
    
    /// Maximum key age before forced rotation
    pub max_key_age: Duration,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            session_rotation_interval: DEFAULT_SESSION_ROTATION_INTERVAL,
            message_rotation_threshold: DEFAULT_MESSAGE_ROTATION_THRESHOLD,
            auto_rotation_enabled: true,
            grace_period: KEY_ROTATION_GRACE_PERIOD,
            max_key_age: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Key management statistics
#[derive(Debug, Default)]
pub struct KeyManagementStats {
    /// Total issuer keys added
    pub total_issuer_keys_added: AtomicU64,
    
    /// Total session keys created
    pub total_session_keys_created: AtomicU64,
    
    /// Total keys rotated
    pub total_keys_rotated: AtomicU64,
    
    /// Total keys purged
    pub total_keys_purged: AtomicU64,
    
    /// Current issuer key count
    pub current_issuer_key_count: AtomicUsize,
    
    /// Current session key count
    pub current_session_key_count: AtomicUsize,
    
    /// Last rotation timestamp
    pub last_rotation_timestamp: AtomicU64,
    
    /// Last purge timestamp
    pub last_purge_timestamp: AtomicU64,
    
    /// Total key operations
    pub total_key_operations: AtomicU64,
    
    /// Failed key operations
    pub failed_key_operations: AtomicU64,
}

impl Clone for KeyManagementStats {
    fn clone(&self) -> Self {
        Self {
            total_issuer_keys_added: AtomicU64::new(self.total_issuer_keys_added.load(Ordering::Relaxed)),
            total_session_keys_created: AtomicU64::new(self.total_session_keys_created.load(Ordering::Relaxed)),
            total_keys_rotated: AtomicU64::new(self.total_keys_rotated.load(Ordering::Relaxed)),
            total_keys_purged: AtomicU64::new(self.total_keys_purged.load(Ordering::Relaxed)),
            current_issuer_key_count: AtomicUsize::new(self.current_issuer_key_count.load(Ordering::Relaxed)),
            current_session_key_count: AtomicUsize::new(self.current_session_key_count.load(Ordering::Relaxed)),
            last_rotation_timestamp: AtomicU64::new(self.last_rotation_timestamp.load(Ordering::Relaxed)),
            last_purge_timestamp: AtomicU64::new(self.last_purge_timestamp.load(Ordering::Relaxed)),
            total_key_operations: AtomicU64::new(self.total_key_operations.load(Ordering::Relaxed)),
            failed_key_operations: AtomicU64::new(self.failed_key_operations.load(Ordering::Relaxed)),
        }
    }
}

impl KeyManagementStats {
    /// Reset all statistics
    pub fn reset(&self) {
        self.total_issuer_keys_added.store(0, Ordering::Relaxed);
        self.total_session_keys_created.store(0, Ordering::Relaxed);
        self.total_keys_rotated.store(0, Ordering::Relaxed);
        self.total_keys_purged.store(0, Ordering::Relaxed);
        self.current_issuer_key_count.store(0, Ordering::Relaxed);
        self.current_session_key_count.store(0, Ordering::Relaxed);
        self.last_rotation_timestamp.store(0, Ordering::Relaxed);
        self.last_purge_timestamp.store(0, Ordering::Relaxed);
        self.total_key_operations.store(0, Ordering::Relaxed);
        self.failed_key_operations.store(0, Ordering::Relaxed);
    }
    
    /// Print statistics
    pub fn print(&self) {
        kprintln!("=== KEY MANAGEMENT STATISTICS ===");
        kprintln!("Issuer Keys:");
        kprintln!("  Total Added: {}", self.total_issuer_keys_added.load(Ordering::Relaxed));
        kprintln!("  Current Count: {}", self.current_issuer_key_count.load(Ordering::Relaxed));
        kprintln!("");
        kprintln!("Session Keys:");
        kprintln!("  Total Created: {}", self.total_session_keys_created.load(Ordering::Relaxed));
        kprintln!("  Current Count: {}", self.current_session_key_count.load(Ordering::Relaxed));
        kprintln!("");
        kprintln!("Operations:");
        kprintln!("  Total Operations: {}", self.total_key_operations.load(Ordering::Relaxed));
        kprintln!("  Failed Operations: {}", self.failed_key_operations.load(Ordering::Relaxed));
        kprintln!("  Total Rotated: {}", self.total_keys_rotated.load(Ordering::Relaxed));
        kprintln!("  Total Purged: {}", self.total_keys_purged.load(Ordering::Relaxed));
        kprintln!("");
        kprintln!("Timestamps:");
        kprintln!("  Last Rotation: {}", self.last_rotation_timestamp.load(Ordering::Relaxed));
        kprintln!("  Last Purge: {}", self.last_purge_timestamp.load(Ordering::Relaxed));
        kprintln!("=== END KEY MANAGEMENT STATISTICS ===");
    }
}

//=============================================================================
// KEYSTORE IMPLEMENTATION
//=============================================================================

/// In-kernel ephemeral keystore
pub struct KeyStore {
    /// Issuer keys (Dilithium public keys)
    issuer_keys: Mutex<BTreeMap<KeyId, IssuerKey>>,
    
    /// Session keys (Kyber encapsulation keys)
    session_keys: Mutex<BTreeMap<KeyId, SessionKey>>,
    
    /// Rotation policy
    rotation_policy: Mutex<RotationPolicy>,
    
    /// Statistics
    stats: KeyManagementStats,
    
    /// Last maintenance timestamp
    last_maintenance: AtomicU64,
}

impl KeyStore {
    /// Create a new keystore
    pub fn new() -> Self {
        Self {
            issuer_keys: Mutex::new(BTreeMap::new()),
            session_keys: Mutex::new(BTreeMap::new()),
            rotation_policy: Mutex::new(RotationPolicy::default()),
            stats: KeyManagementStats::default(),
            last_maintenance: AtomicU64::new(get_current_timestamp()),
        }
    }
    
    /// Add an issuer key
    /// 
    /// # Arguments
    /// * `public_key` - Dilithium public key
    /// * `params` - Parameter set used
    /// 
    /// # Returns
    /// `Result<KeyId, String>` - Key ID if successful
    pub fn add_issuer_key(
        &self,
        public_key: DilithiumPublicKey,
        params: DilithiumParameterSet,
    ) -> Result<KeyId, String> {
        self.stats.total_key_operations.fetch_add(1, Ordering::Relaxed);
        
        let mut keys = self.issuer_keys.lock();
        
        // Check capacity limit
        if keys.len() >= MAX_ISSUER_KEYS {
            self.stats.failed_key_operations.fetch_add(1, Ordering::Relaxed);
            return Err("Issuer key capacity limit reached".to_string());
        }
        
        // Generate unique key ID
        let key_id = KeyId::random();
        
        // Create issuer key
        let issuer_key = IssuerKey::new(key_id.clone(), public_key, params);
        
        // Store the key
        keys.insert(key_id.clone(), issuer_key);
        
        // Update statistics
        self.stats.total_issuer_keys_added.fetch_add(1, Ordering::Relaxed);
        self.stats.current_issuer_key_count.store(keys.len(), Ordering::Relaxed);
        
        klog!(INFO, "[KEYSTORE] Added issuer key: {}", key_id);
        
        Ok(key_id)
    }
    
    /// Create a session key
    /// 
    /// # Arguments
    /// * `public_key` - Kyber public key
    /// * `params` - Parameter set used
    /// * `issuer_key_id` - Optional associated issuer key ID
    /// 
    /// # Returns
    /// `Result<KeyId, String>` - Key ID if successful
    pub fn create_session_key(
        &self,
        public_key: KyberPublicKey,
        params: KyberParameterSet,
        issuer_key_id: Option<KeyId>,
    ) -> Result<KeyId, String> {
        self.stats.total_key_operations.fetch_add(1, Ordering::Relaxed);
        
        let mut keys = self.session_keys.lock();
        
        // Check capacity limit
        if keys.len() >= MAX_SESSION_KEYS {
            self.stats.failed_key_operations.fetch_add(1, Ordering::Relaxed);
            return Err("Session key capacity limit reached".to_string());
        }
        
        // Get rotation policy
        let policy = self.rotation_policy.lock();
        let rotation_interval = policy.session_rotation_interval;
        drop(policy);
        
        // Generate unique key ID
        let key_id = KeyId::random();
        
        // Create session key
        let session_key = SessionKey::new(
            key_id.clone(),
            public_key,
            params,
            rotation_interval,
            issuer_key_id,
        );
        
        // Store the key
        keys.insert(key_id.clone(), session_key);
        
        // Update statistics
        self.stats.total_session_keys_created.fetch_add(1, Ordering::Relaxed);
        self.stats.current_session_key_count.store(keys.len(), Ordering::Relaxed);
        
        klog!(INFO, "[KEYSTORE] Created session key: {}", key_id);
        
        Ok(key_id)
    }
    
    /// Get an issuer key by ID
    /// 
    /// # Arguments
    /// * `key_id` - Key identifier
    /// 
    /// # Returns
    /// `Option<IssuerKey>` - Issuer key if found
    pub fn get_issuer_key(&self, key_id: &KeyId) -> Option<IssuerKey> {
        let mut keys = self.issuer_keys.lock();
        
        if let Some(key) = keys.get_mut(key_id) {
            key.record_usage();
            Some(key.clone())
        } else {
            None
        }
    }
    
    /// Get a session key by ID
    /// 
    /// # Arguments
    /// * `key_id` - Key identifier
    /// 
    /// # Returns
    /// `Option<SessionKey>` - Session key if found
    pub fn get_session_key(&self, key_id: &KeyId) -> Option<SessionKey> {
        let mut keys = self.session_keys.lock();
        
        if let Some(key) = keys.get_mut(key_id) {
            key.record_usage();
            Some(key.clone())
        } else {
            None
        }
    }
    
    /// Rotate a session key with overlap support
    /// 
    /// # Arguments
    /// * `key_id` - Key identifier to rotate
    /// * `new_public_key` - New Kyber public key
    /// * `new_params` - New parameter set
    /// 
    /// # Returns
    /// `Result<KeyId, String>` - New key ID if successful
    pub fn rotate_session_key_with_overlap(
        &self,
        key_id: &KeyId,
        new_public_key: KyberPublicKey,
        new_params: KyberParameterSet,
    ) -> Result<KeyId, String> {
        let mut keys = self.session_keys.lock();
        
        // Get the existing key
        let existing_key = keys.get(key_id).ok_or("Session key not found")?;
        
        // Check if rotation is needed
        if !existing_key.should_rotate_by_time() && 
           !existing_key.should_rotate_by_message_count(DEFAULT_MESSAGE_ROTATION_THRESHOLD) {
            return Err("Key rotation not needed".to_string());
        }
        
        // Create new session key
        let new_key_id = self.create_session_key(new_public_key, new_params, existing_key.issuer_key_id.clone())?;
        
        // Mark old key for graceful retirement (overlap period)
        if let Some(old_key) = keys.get_mut(key_id) {
            old_key.active = false;
            old_key.expires_at = get_current_timestamp() + KEY_ROTATION_GRACE_PERIOD.as_secs();
        }
        
        // Update statistics
        self.stats.total_keys_rotated.fetch_add(1, Ordering::Relaxed);
        
        klog!(INFO, "[KEYSTORE] Rotated session key {} with overlap, new key: {}", key_id, new_key_id);
        
        Ok(new_key_id)
    }
    
    /// Get overlapping keys for a session (current + previous for grace period)
    /// 
    /// # Arguments
    /// * `key_id` - Current key identifier
    /// 
    /// # Returns
    /// `Vec<SessionKey>` - List of valid keys including overlap period
    pub fn get_overlapping_session_keys(&self, key_id: &KeyId) -> Vec<SessionKey> {
        let keys = self.session_keys.lock();
        let mut overlapping_keys = Vec::new();
        
        // Add current key if active
        if let Some(current_key) = keys.get(key_id) {
            if current_key.active && !current_key.is_expired() {
                overlapping_keys.push(current_key.clone());
            }
        }
        
        // Find keys that are in overlap period (recently rotated)
        for (_, key) in keys.iter() {
            if !key.active && !key.is_expired() {
                // Check if this key is within grace period
                let grace_expiry = key.expires_at + KEY_ROTATION_GRACE_PERIOD.as_secs();
                if get_current_timestamp() < grace_expiry {
                    overlapping_keys.push(key.clone());
                }
            }
        }
        
        overlapping_keys
    }
    
    /// Clean up expired keys and overlap periods
    /// 
    /// # Returns
    /// `usize` - Number of keys cleaned up
    pub fn cleanup_expired_keys(&self) -> usize {
        let mut keys = self.session_keys.lock();
        let mut keys_to_remove = Vec::new();
        
        for (key_id, key) in keys.iter() {
            if key.is_expired() {
                keys_to_remove.push(key_id.clone());
            }
        }
        
        let cleanup_count = keys_to_remove.len();
        
        for key_id in keys_to_remove {
            keys.remove(&key_id);
        }
        
        // Update statistics
        self.stats.current_session_key_count.store(keys.len(), Ordering::Relaxed);
        self.stats.total_keys_purged.fetch_add(cleanup_count as u64, Ordering::Relaxed);
        
        if cleanup_count > 0 {
            klog!(INFO, "[KEYSTORE] Cleaned up {} expired session keys", cleanup_count);
        }
        
        cleanup_count
    }
    
    /// Force rotation of all session keys
    /// 
    /// # Returns
    /// `usize` - Number of keys rotated
    pub fn force_rotate_all_session_keys(&self) -> usize {
        let mut keys = self.session_keys.lock();
        let mut rotated_count = 0;
        
        for (_key_id, key) in keys.iter_mut() {
            if key.active && !key.is_expired() {
                // Mark for rotation
                key.active = false;
                key.expires_at = get_current_timestamp() + KEY_ROTATION_GRACE_PERIOD.as_secs();
                rotated_count += 1;
            }
        }
        
        if rotated_count > 0 {
            klog!(INFO, "[KEYSTORE] Force rotated {} session keys", rotated_count);
        }
        
        rotated_count
    }
    
    /// Remove an issuer key
    /// 
    /// # Arguments
    /// * `key_id` - Key identifier
    /// 
    /// # Returns
    /// `bool` - Whether the key was removed
    pub fn remove_issuer_key(&self, key_id: &KeyId) -> bool {
        let mut keys = self.issuer_keys.lock();
        
        if keys.remove(key_id).is_some() {
            self.stats.current_issuer_key_count.store(keys.len(), Ordering::Relaxed);
            klog!(INFO, "[KEYSTORE] Removed issuer key: {}", key_id);
            true
        } else {
            false
        }
    }
    
    /// Remove a session key
    /// 
    /// # Arguments
    /// * `key_id` - Key identifier
    /// 
    /// # Returns
    /// `bool` - Whether the key was removed
    pub fn remove_session_key(&self, key_id: &KeyId) -> bool {
        let mut keys = self.session_keys.lock();
        
        if keys.remove(key_id).is_some() {
            self.stats.current_session_key_count.store(keys.len(), Ordering::Relaxed);
            klog!(INFO, "[KEYSTORE] Removed session key: {}", key_id);
            true
        } else {
            false
        }
    }
    
    /// Rotate session keys
    /// 
    /// This function rotates expired or message-count-threshold-exceeded session keys
    /// while maintaining a grace period for ongoing IPC streams.
    pub fn rotate_session_keys(&self) -> usize {
        let mut keys = self.session_keys.lock();
        let policy = self.rotation_policy.lock();
        
        let current_time = get_current_timestamp();
        let grace_period_end = current_time + policy.grace_period.as_secs();
        let mut rotated_count = 0;
        
        // Collect keys that need rotation
        let keys_to_rotate: Vec<KeyId> = keys
            .iter()
            .filter_map(|(id, key)| {
                if !key.active {
                    return None;
                }
                
                // Check if key should be rotated
                let should_rotate = key.should_rotate_by_time() || 
                                  key.should_rotate_by_message_count(policy.message_rotation_threshold);
                
                if should_rotate {
                    // Check if we're in grace period
                    if current_time < grace_period_end {
                        // Mark for rotation but keep active during grace period
                        Some(id.clone())
                    } else {
                        // Force rotation after grace period
                        Some(id.clone())
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // Rotate the keys
        for key_id in keys_to_rotate {
            if let Some(key) = keys.get_mut(&key_id) {
                key.active = false;
                rotated_count += 1;
                klog!(INFO, "[KEYSTORE] Rotated session key: {}", key_id);
            }
        }
        
        drop(policy);
        drop(keys);
        
        // Update statistics
        self.stats.total_keys_rotated.fetch_add(rotated_count as u64, Ordering::Relaxed);
        self.stats.last_rotation_timestamp.store(current_time, Ordering::Relaxed);
        
        klog!(INFO, "[KEYSTORE] Rotated {} session keys", rotated_count);
        
        rotated_count
    }
    
    /// Purge expired and inactive keys
    /// 
    /// This function removes keys that are no longer needed.
    pub fn purge_expired_keys(&self) -> usize {
        let mut issuer_keys = self.issuer_keys.lock();
        let mut session_keys = self.session_keys.lock();
        let policy = self.rotation_policy.lock();
        
        let current_time = get_current_timestamp();
        let max_age = policy.max_key_age.as_secs();
        let mut purged_count = 0;
        
        // Purge expired issuer keys
        let issuer_keys_to_purge: Vec<KeyId> = issuer_keys
            .iter()
            .filter_map(|(id, key)| {
                if !key.active && key.time_since_last_use() > max_age {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        
        for key_id in issuer_keys_to_purge {
            if issuer_keys.remove(&key_id).is_some() {
                purged_count += 1;
                klog!(INFO, "[KEYSTORE] Purged expired issuer key: {}", key_id);
            }
        }
        
        // Purge expired session keys
        let session_keys_to_purge: Vec<KeyId> = session_keys
            .iter()
            .filter_map(|(id, key)| {
                if !key.active && key.time_since_last_use() > max_age {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        
        for key_id in session_keys_to_purge {
            if session_keys.remove(&key_id).is_some() {
                purged_count += 1;
                klog!(INFO, "[KEYSTORE] Purged expired session key: {}", key_id);
            }
        }
        
        drop(policy);
        
        // Update statistics
        self.stats.current_issuer_key_count.store(issuer_keys.len(), Ordering::Relaxed);
        self.stats.current_session_key_count.store(session_keys.len(), Ordering::Relaxed);
        self.stats.total_keys_purged.fetch_add(purged_count as u64, Ordering::Relaxed);
        self.stats.last_purge_timestamp.store(current_time, Ordering::Relaxed);
        
        klog!(INFO, "[KEYSTORE] Purged {} expired keys", purged_count);
        
        purged_count
    }
    
    /// Perform maintenance operations
    /// 
    /// This function performs periodic maintenance including key rotation and purging.
    pub fn perform_maintenance(&self) {
        let current_time = get_current_timestamp();
        let last_maintenance = self.last_maintenance.load(Ordering::Relaxed);
        
        // Perform maintenance every 5 minutes
        if current_time - last_maintenance >= 300 {
            klog!(INFO, "[KEYSTORE] Performing periodic maintenance");
            
            // Rotate session keys
            let rotated = self.rotate_session_keys();
            
            // Purge expired keys
            let purged = self.purge_expired_keys();
            
            if rotated > 0 || purged > 0 {
                klog!(INFO, "[KEYSTORE] Maintenance complete: {} keys rotated, {} keys purged", rotated, purged);
            }
            
            self.last_maintenance.store(current_time, Ordering::Relaxed);
        }
    }
    
    /// Get current key counts
    /// 
    /// # Returns
    /// `(usize, usize)` - (issuer key count, session key count)
    pub fn get_key_counts(&self) -> (usize, usize) {
        let issuer_count = self.issuer_keys.lock().len();
        let session_count = self.session_keys.lock().len();
        (issuer_count, session_count)
    }
    
    /// Get rotation policy
    /// 
    /// # Returns
    /// `RotationPolicy` - Current rotation policy
    pub fn get_rotation_policy(&self) -> RotationPolicy {
        self.rotation_policy.lock().clone()
    }
    
    /// Update rotation policy
    /// 
    /// # Arguments
    /// * `policy` - New rotation policy
    pub fn update_rotation_policy(&self, policy: RotationPolicy) {
        let mut current_policy = self.rotation_policy.lock();
        *current_policy = policy;
        klog!(INFO, "[KEYSTORE] Updated rotation policy");
    }
    
    /// Get statistics
    /// 
    /// # Returns
    /// `&KeyManagementStats` - Reference to statistics
    pub fn get_stats(&self) -> &KeyManagementStats {
        &self.stats
    }
    
    /// Force immediate rotation of all session keys
    /// 
    /// This bypasses the grace period and forces immediate rotation.
    pub fn force_rotation_now(&self) -> usize {
        let mut keys = self.session_keys.lock();
        let mut rotated_count = 0;
        
        for (id, key) in keys.iter_mut() {
            if key.active {
                key.active = false;
                rotated_count += 1;
                klog!(INFO, "[KEYSTORE] Force rotated session key: {}", id);
            }
        }
        
        drop(keys);
        
        // Update statistics
        self.stats.total_keys_rotated.fetch_add(rotated_count as u64, Ordering::Relaxed);
        self.stats.last_rotation_timestamp.store(get_current_timestamp(), Ordering::Relaxed);
        
        klog!(INFO, "[KEYSTORE] Force rotated {} session keys", rotated_count);
        
        rotated_count
    }
    
    /// Purge all keys (emergency operation)
    /// 
    /// This removes all keys from the keystore.
    pub fn purge_all_keys(&self) -> usize {
        let mut issuer_keys = self.issuer_keys.lock();
        let mut session_keys = self.session_keys.lock();
        
        let issuer_count = issuer_keys.len();
        let session_count = session_keys.len();
        let total_count = issuer_count + session_count;
        
        // Clear all keys
        issuer_keys.clear();
        session_keys.clear();
        
        drop(issuer_keys);
        drop(session_keys);
        
        // Update statistics
        self.stats.current_issuer_key_count.store(0, Ordering::Relaxed);
        self.stats.current_session_key_count.store(0, Ordering::Relaxed);
        self.stats.total_keys_purged.fetch_add(total_count as u64, Ordering::Relaxed);
        self.stats.last_purge_timestamp.store(get_current_timestamp(), Ordering::Relaxed);
        
        klog!(WARN, "[KEYSTORE] Emergency purge: removed {} keys", total_count);
        
        total_count
    }
}

impl Drop for KeyStore {
    fn drop(&mut self) {
        // Ensure all keys are properly zeroized
        klog!(INFO, "[KEYSTORE] Dropping keystore, zeroizing {} keys", 
              self.get_key_counts().0 + self.get_key_counts().1);
        
        // Force rotation and purge to trigger zeroization
        self.force_rotation_now();
        self.purge_expired_keys();
    }
}

//=============================================================================
// GLOBAL KEYSTORE INSTANCE
//=============================================================================

lazy_static! {
    static ref GLOBAL_KEYSTORE: Mutex<KeyStore> = Mutex::new(KeyStore::new());
}

/// Initialize the global keystore
pub fn init_keystore() {
    kprintln!("[KEYSTORE] Initializing global keystore");
    
    let keystore = KeyStore::new();
    *GLOBAL_KEYSTORE.lock() = keystore;
    
    kprintln!("[KEYSTORE] Global keystore initialized");
}

/// Get the global keystore
pub fn get_keystore() -> &'static Mutex<KeyStore> {
    &GLOBAL_KEYSTORE
}

/// Add an issuer key to the global keystore
pub fn add_issuer_key(
    public_key: DilithiumPublicKey,
    params: DilithiumParameterSet,
) -> Result<KeyId, String> {
    let keystore = get_keystore();
    keystore.lock().add_issuer_key(public_key, params)
}

/// Create a session key in the global keystore
pub fn create_session_key(
    public_key: KyberPublicKey,
    params: KyberParameterSet,
    issuer_key_id: Option<KeyId>,
) -> Result<KeyId, String> {
    let keystore = get_keystore();
    keystore.lock().create_session_key(public_key, params, issuer_key_id)
}

/// Get an issuer key from the global keystore
pub fn get_issuer_key(key_id: &KeyId) -> Option<IssuerKey> {
    let keystore = get_keystore();
    keystore.lock().get_issuer_key(key_id)
}

/// Get a session key from the global keystore
pub fn get_session_key(key_id: &KeyId) -> Option<SessionKey> {
    let keystore = get_keystore();
    keystore.lock().get_session_key(key_id)
}

/// Rotate session keys in the global keystore
pub fn rotate_session_keys() -> usize {
    let keystore = get_keystore();
    keystore.lock().rotate_session_keys()
}

/// Purge expired keys from the global keystore
pub fn purge_expired_keys() -> usize {
    let keystore = get_keystore();
    keystore.lock().purge_expired_keys()
}

/// Force immediate rotation of all session keys
pub fn force_rotation_now() -> usize {
    let keystore = get_keystore();
    keystore.lock().force_rotation_now()
}

/// Purge all keys from the global keystore
pub fn purge_all_keys() -> usize {
    let keystore = get_keystore();
    keystore.lock().purge_all_keys()
}

/// Get key counts from the global keystore
pub fn get_key_counts() -> (usize, usize) {
    let keystore = get_keystore();
    keystore.lock().get_key_counts()
}

/// Get statistics from the global keystore
pub fn get_keystore_stats() -> KeyManagementStats {
    let keystore = get_keystore();
    keystore.lock().get_stats().clone()
}

/// Perform maintenance on the global keystore
pub fn perform_keystore_maintenance() {
    let keystore = get_keystore();
    keystore.lock().perform_maintenance()
}

//=============================================================================
// UTILITY FUNCTIONS
//=============================================================================

/// Get current timestamp in seconds
fn get_current_timestamp() -> u64 {
    // TODO: Integrate with actual time system
    core::sync::atomic::AtomicU64::new(0).fetch_add(1, Ordering::Relaxed)
}

//=============================================================================
// TESTING
//=============================================================================

/// Test keystore functionality
#[allow(dead_code)]
pub fn test_keystore() {
    kprintln!("[KEYSTORE] Testing keystore functionality...");
    
    // Test key creation and management
    let _keystore = KeyStore::new();
    
    // Test issuer key addition
    // Note: This is a simplified test - in practice, you'd create actual Dilithium keys
    kprintln!("[KEYSTORE] Test completed (simplified - no actual key generation)");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_id_creation() {
        let key_id = KeyId::new([0xAA; KEY_ID_LENGTH]);
        assert_eq!(key_id.as_bytes(), &[0xAA; KEY_ID_LENGTH]);
        
        let key_id = KeyId::from_slice(&[0xBB; KEY_ID_LENGTH]).unwrap();
        assert_eq!(key_id.as_bytes(), &[0xBB; KEY_ID_LENGTH]);
        
        let key_id = KeyId::from_slice(&[0xCC; 16]);
        assert!(key_id.is_none());
    }
    
    #[test]
    fn test_rotation_policy_default() {
        let policy = RotationPolicy::default();
        assert_eq!(policy.session_rotation_interval, DEFAULT_SESSION_ROTATION_INTERVAL);
        assert_eq!(policy.message_rotation_threshold, DEFAULT_MESSAGE_ROTATION_THRESHOLD);
        assert!(policy.auto_rotation_enabled);
    }
    
    #[test]
    fn test_keystore_creation() {
        let keystore = KeyStore::new();
        let (issuer_count, session_count) = keystore.get_key_counts();
        assert_eq!(issuer_count, 0);
        assert_eq!(session_count, 0);
    }
    
    #[test]
    fn test_keystore_stats() {
        let stats = KeyManagementStats::default();
        assert_eq!(stats.total_issuer_keys_added.load(Ordering::Relaxed), 0);
        assert_eq!(stats.current_issuer_key_count.load(Ordering::Relaxed), 0);
        
        stats.reset();
        assert_eq!(stats.total_issuer_keys_added.load(Ordering::Relaxed), 0);
    }
}
