/// Capability Store v2 - In-Kernel Verifier with LRU Nonce Cache
/// 
/// This module implements the capability store for CapTokens v2 with:
/// - LRU nonce cache for replay protection
/// - O(1) lookups by (issuer, dst, nonce)
/// - PQC signature verification
/// - Performance monitoring

use crate::{kprintln, klog};
use crate::security::cap_v2::{
    CapTokenV2, CapTokenHeader, CapValidationResult, CapValidationFailure,
    SignatureAlgorithm, scope_v2,
};
use crate::crypto::pqc::{Dilithium, DilithiumParameterSet, DilithiumPublicKey};
use alloc::collections::{BTreeMap, BTreeSet, HashMap};
use alloc::vec::Vec;
use alloc::string::String;
use core::time::Duration;
use spin::Mutex;
use lru::LruCache;
use zeroize::Zeroizing;

use crate::security::cap::{CapTokenV2, CapVerifyError, CapVerifyOutcome};
use crate::secman::did::DidResolver;
use crate::time::Instant;
use crate::crypto::dilithium::DilithiumPubKey;

/// Replay protection error
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayError {
    /// Nonce has been used before
    NonceReused,
    /// Nonce is too old (outside window)
    NonceTooOld,
    /// Issuer not found
    IssuerNotFound,
    /// Internal error
    Internal,
}

/// Replay window configuration
#[derive(Debug, Clone)]
pub struct ReplayWindowConfig {
    /// Maximum number of nonces to track per issuer
    pub max_nonces: usize,
    /// Time-to-live for nonce entries
    pub nonce_ttl: Duration,
    /// Low watermark for nonce rejection
    pub low_watermark: u128,
}

impl Default for ReplayWindowConfig {
    fn default() -> Self {
        Self {
            max_nonces: 10000,
            nonce_ttl: Duration::from_secs(300), // 5 minutes
            low_watermark: 1000,
        }
    }
}

/// Nonce entry with timestamp
#[derive(Debug, Clone)]
struct NonceEntry {
    /// When the nonce was recorded
    recorded_at: Instant,
    /// Destination where nonce was used
    dst: String,
}

/// Replay window for a single issuer
pub struct ReplayWindow {
    /// LRU cache of recent nonces
    nonces: LruCache<u128, NonceEntry>,
    /// Low watermark for nonce rejection
    low_watermark: u128,
    /// Time-to-live for nonce entries
    ttl: Duration,
    /// Last cleanup time
    last_cleanup: Instant,
}

impl ReplayWindow {
    /// Create a new replay window
    pub fn new(config: &ReplayWindowConfig) -> Self {
        Self {
            nonces: LruCache::new(config.max_nonces),
            low_watermark: config.low_watermark,
            ttl: config.nonce_ttl,
            last_cleanup: Instant::now(),
        }
    }

    /// Verify and record a nonce
    pub fn verify_and_record(
        &mut self,
        nonce: u128,
        dst: &str,
        current_time: Instant,
    ) -> Result<(), ReplayError> {
        // Check if nonce is too old
        if nonce < self.low_watermark {
            return Err(ReplayError::NonceTooOld);
        }

        // Check if nonce has been used before
        if let Some(entry) = self.nonces.get(&nonce) {
            // Check if it's the same destination (allow reuse across different destinations)
            if entry.dst == dst {
                return Err(ReplayError::NonceReused);
            }
        }

        // Record the new nonce usage
        let entry = NonceEntry {
            recorded_at: current_time,
            dst: dst.to_string(),
        };
        
        self.nonces.put(nonce, entry);
        
        // Periodic cleanup of expired entries
        if current_time - self.last_cleanup > Duration::from_secs(60) {
            self.cleanup_expired(current_time);
            self.last_cleanup = current_time;
        }

        Ok(())
    }

    /// Clean up expired nonce entries
    fn cleanup_expired(&mut self, current_time: Instant) {
        let mut expired_nonces = Vec::new();
        
        for (nonce, entry) in self.nonces.iter() {
            if current_time - entry.recorded_at > self.ttl {
                expired_nonces.push(*nonce);
            }
        }
        
        for nonce in expired_nonces {
            self.nonces.pop(&nonce);
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> ReplayWindowStats {
        ReplayWindowStats {
            total_nonces: self.nonces.len(),
            low_watermark: self.low_watermark,
            ttl_seconds: self.ttl.as_secs(),
        }
    }

    /// Reset the replay window
    pub fn reset(&mut self) {
        self.nonces.clear();
        self.last_cleanup = Instant::now();
    }
}

/// Replay window statistics
#[derive(Debug, Clone)]
pub struct ReplayWindowStats {
    /// Total number of nonces being tracked
    pub total_nonces: usize,
    /// Low watermark for nonce rejection
    pub low_watermark: u128,
    /// TTL in seconds
    pub ttl_seconds: u64,
}

/// Capability store with replay protection
pub struct CapStore {
    /// Replay windows per issuer
    replay_windows: HashMap<String, Mutex<ReplayWindow>>,
    /// DID resolver for issuer verification
    did_resolver: DidResolver,
    /// Configuration
    config: ReplayWindowConfig,
    /// Statistics counters
    stats: Mutex<CapStoreStats>,
}

/// Capability store statistics
#[derive(Debug, Clone, Default)]
pub struct CapStoreStats {
    /// Number of successful verifications
    pub auth_ok: u64,
    /// Number of failed verifications
    pub auth_fail: u64,
    /// Number of replay attempts dropped
    pub replay_drops: u64,
    /// Number of DID verification failures
    pub did_failures: u64,
}

impl CapStore {
    /// Create a new capability store
    pub fn new(did_resolver: DidResolver, config: ReplayWindowConfig) -> Self {
        Self {
            replay_windows: HashMap::new(),
            did_resolver,
            config,
            stats: Mutex::new(CapStoreStats::default()),
        }
    }

    /// Verify a capability token with replay protection
    pub fn verify_capability(
        &self,
        token: &CapTokenV2,
        current_time: Instant,
    ) -> Result<CapVerifyOutcome, CapVerifyError> {
        // Check if token is expired
        if token.is_expired() {
            return Err(CapVerifyError::Expired);
        }

        // Resolve issuer DID
        let issuer_pubkey = match self.did_resolver.resolve_did(&token.header.issuer) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                self.increment_stat(|stats| stats.did_failures += 1);
                return Err(CapVerifyError::IssuerUntrusted);
            }
        };

        // Verify token signature
        if let Err(e) = token.verify_signature(&issuer_pubkey) {
            self.increment_stat(|stats| stats.auth_fail += 1);
            return Err(e);
        }

        // Check replay protection
        let replay_result = self.check_replay_protection(
            &token.header.issuer,
            token.header.nonce,
            &token.header.dst,
            current_time,
        );

        match replay_result {
            Ok(()) => {
                self.increment_stat(|stats| stats.auth_ok += 1);
                Ok(CapVerifyOutcome::Accepted)
            }
            Err(ReplayError::NonceReused) => {
                self.increment_stat(|stats| stats.replay_drops += 1);
                Err(CapVerifyError::Replayed)
            }
            Err(ReplayError::NonceTooOld) => {
                self.increment_stat(|stats| stats.replay_drops += 1);
                Err(CapVerifyError::NonceTooOld)
            }
            Err(_) => {
                self.increment_stat(|stats| stats.auth_fail += 1);
                Err(CapVerifyError::Internal)
            }
        }
    }

    /// Check replay protection for a specific issuer
    fn check_replay_protection(
        &self,
        issuer: &str,
        nonce: u128,
        dst: &str,
        current_time: Instant,
    ) -> Result<(), ReplayError> {
        let window = self.get_or_create_replay_window(issuer);
        let mut window_guard = window.lock();
        window_guard.verify_and_record(nonce, dst, current_time)
    }

    /// Get or create replay window for an issuer
    fn get_or_create_replay_window(&self, issuer: &str) -> &Mutex<ReplayWindow> {
        if !self.replay_windows.contains_key(issuer) {
            let new_window = ReplayWindow::new(&self.config);
            self.replay_windows.insert(issuer.to_string(), Mutex::new(new_window));
        }
        
        self.replay_windows.get(issuer).unwrap()
    }

    /// Get replay window statistics for an issuer
    pub fn get_replay_stats(&self, issuer: &str) -> Option<ReplayWindowStats> {
        self.replay_windows
            .get(issuer)
            .map(|window| window.lock().stats())
    }

    /// Get overall capability store statistics
    pub fn get_stats(&self) -> CapStoreStats {
        self.stats.lock().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        let mut stats = self.stats.lock();
        *stats = CapStoreStats::default();
    }

    /// Reset replay window for a specific issuer
    pub fn reset_issuer_window(&self, issuer: &str) {
        if let Some(window) = self.replay_windows.get(issuer) {
            window.lock().reset();
        }
    }

    /// Reset all replay windows
    pub fn reset_all_windows(&self) {
        for window in self.replay_windows.values() {
            window.lock().reset();
        }
    }

    /// Increment a statistic
    fn increment_stat<F>(&self, f: F)
    where
        F: FnOnce(&mut CapStoreStats),
    {
        let mut stats = self.stats.lock();
        f(&mut stats);
    }

    /// Clean up expired entries across all windows
    pub fn cleanup_expired(&self, current_time: Instant) {
        for window in self.replay_windows.values() {
            let mut window_guard = window.lock();
            window_guard.cleanup_expired(current_time);
        }
    }
}

impl Default for CapStore {
    fn default() -> Self {
        Self::new(DidResolver::default(), ReplayWindowConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secman::did::DidResolver;
    use crate::security::cap::{CapTokenV2, permissions};

    fn create_test_token(issuer: &str, nonce: u128) -> CapTokenV2 {
        CapTokenV2::new(
            issuer.to_string(),
            "test_dst".to_string(),
            nonce,
            "test_cap".to_string(),
            permissions::READ,
            1000,
        )
    }

    #[test]
    fn test_replay_window_creation() {
        let config = ReplayWindowConfig::default();
        let window = ReplayWindow::new(&config);
        let stats = window.stats();
        
        assert_eq!(stats.total_nonces, 0);
        assert_eq!(stats.low_watermark, 1000);
    }

    #[test]
    fn test_replay_protection() {
        let config = ReplayWindowConfig::default();
        let mut window = ReplayWindow::new(&config);
        let current_time = Instant::now();
        
        // First use should succeed
        assert!(window.verify_and_record(1000, "dst1", current_time).is_ok());
        
        // Same nonce, same destination should fail
        assert_eq!(
            window.verify_and_record(1000, "dst1", current_time),
            Err(ReplayError::NonceReused)
        );
        
        // Same nonce, different destination should succeed
        assert!(window.verify_and_record(1000, "dst2", current_time).is_ok());
        
        // Nonce too old should fail
        assert_eq!(
            window.verify_and_record(500, "dst1", current_time),
            Err(ReplayError::NonceTooOld)
        );
    }

    #[test]
    fn test_cap_store_verification() {
        let did_resolver = DidResolver::default();
        let config = ReplayWindowConfig::default();
        let cap_store = CapStore::new(did_resolver, config);
        
        let token = create_test_token("test_issuer", 1000);
        let current_time = Instant::now();
        
        // Note: This test will fail signature verification since we don't have a real key
        // In a real test, we'd need to properly sign the token
        let result = cap_store.verify_capability(&token, current_time);
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics_tracking() {
        let did_resolver = DidResolver::default();
        let config = ReplayWindowConfig::default();
        let cap_store = CapStore::new(did_resolver, config);
        
        let stats = cap_store.get_stats();
        assert_eq!(stats.auth_ok, 0);
        assert_eq!(stats.auth_fail, 0);
        assert_eq!(stats.replay_drops, 0);
        assert_eq!(stats.did_failures, 0);
    }
}
