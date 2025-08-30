//! DID Document Resolver Module
//! 
//! Provides in-kernel minimal DID document cache mapping issuer_did → Dilithium public key,
//! with trust anchors and TTL management. Supports soft-fail for local tests and hard-fail
//! for strict resolver mode.

use alloc::collections::HashMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;
use spin::Mutex;
use serde::{Deserialize, Serialize};

use crate::crypto::dilithium::DilithiumPubKey;
use crate::time::Instant;

/// DID resolution error
#[derive(Debug, Clone, PartialEq)]
pub enum DidResolveError {
    /// DID not found
    NotFound,
    /// DID anchor has expired
    Expired,
    /// DID anchor is invalid
    Invalid,
    /// Internal error
    Internal,
}

/// DID anchor record with trust information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidAnchor {
    /// Public key for the DID
    pub pubkey: DilithiumPubKey,
    /// Time-to-live for the anchor
    pub ttl: Duration,
    /// When the anchor was created
    pub created_at: Instant,
    /// When the anchor expires
    pub expires_at: Instant,
    /// When the anchor was rotated (if applicable)
    pub rotated_at: Option<Instant>,
    /// Previous public key (for rotation grace)
    pub previous_pubkey: Option<DilithiumPubKey>,
    /// Rotation grace period
    pub rotation_grace: Duration,
}

impl DidAnchor {
    /// Create a new DID anchor
    pub fn new(
        pubkey: DilithiumPubKey,
        ttl: Duration,
        rotation_grace: Duration,
    ) -> Self {
        let now = Instant::now();
        Self {
            pubkey,
            ttl,
            created_at: now,
            expires_at: now + ttl,
            rotated_at: None,
            previous_pubkey: None,
            rotation_grace,
        }
    }

    /// Check if anchor is expired
    pub fn is_expired(&self, current_time: Instant) -> bool {
        current_time > self.expires_at
    }

    /// Check if anchor is in rotation grace period
    pub fn is_in_rotation_grace(&self, current_time: Instant) -> bool {
        if let Some(rotated_at) = self.rotated_at {
            current_time <= rotated_at + self.rotation_grace
        } else {
            false
        }
    }

    /// Rotate the anchor to a new public key
    pub fn rotate(&mut self, new_pubkey: DilithiumPubKey) {
        self.previous_pubkey = Some(self.pubkey.clone());
        self.pubkey = new_pubkey;
        self.rotated_at = Some(Instant::now());
    }

    /// Get the current valid public key(s)
    pub fn get_valid_keys(&self, current_time: Instant) -> Vec<DilithiumPubKey> {
        let mut keys = Vec::new();
        
        // Add current key if not expired
        if !self.is_expired(current_time) {
            keys.push(self.pubkey.clone());
        }
        
        // Add previous key if in rotation grace period
        if let Some(prev_key) = &self.previous_pubkey {
            if self.is_in_rotation_grace(current_time) {
                keys.push(prev_key.clone());
            }
        }
        
        keys
    }

    /// Check if a public key is valid for this anchor
    pub fn is_key_valid(&self, pubkey: &DilithiumPubKey, current_time: Instant) -> bool {
        if self.is_expired(current_time) {
            return false;
        }
        
        // Check current key
        if self.pubkey == *pubkey {
            return true;
        }
        
        // Check previous key if in rotation grace
        if let Some(prev_key) = &self.previous_pubkey {
            if self.is_in_rotation_grace(current_time) && *prev_key == *pubkey {
                return true;
            }
        }
        
        false
    }

    /// Get anchor age
    pub fn age(&self, current_time: Instant) -> Duration {
        if current_time > self.created_at {
            current_time - self.created_at
        } else {
            Duration::from_secs(0)
        }
    }

    /// Get time until expiration
    pub fn time_until_expiry(&self, current_time: Instant) -> Duration {
        if current_time < self.expires_at {
            self.expires_at - current_time
        } else {
            Duration::from_secs(0)
        }
    }
}

/// DID resolver configuration
#[derive(Debug, Clone)]
pub struct DidResolverConfig {
    /// Default TTL for new anchors
    pub default_ttl: Duration,
    /// Default rotation grace period
    pub default_rotation_grace: Duration,
    /// Maximum number of anchors to cache
    pub max_anchors: usize,
    /// Cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for DidResolverConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(86400), // 24 hours
            default_rotation_grace: Duration::from_secs(3600), // 1 hour
            max_anchors: 1000,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// DID resolver with anchor management
pub struct DidResolver {
    /// Anchors by DID
    anchors: Mutex<HashMap<String, DidAnchor>>,
    /// Configuration
    config: DidResolverConfig,
    /// Last cleanup time
    last_cleanup: Mutex<Instant>,
}

impl DidResolver {
    /// Create a new DID resolver
    pub fn new(config: DidResolverConfig) -> Self {
        Self {
            anchors: Mutex::new(HashMap::new()),
            config,
            last_cleanup: Mutex::new(Instant::now()),
        }
    }

    /// Add a DID anchor
    pub fn add_anchor(
        &self,
        did: String,
        pubkey: DilithiumPubKey,
        ttl: Option<Duration>,
        rotation_grace: Option<Duration>,
    ) -> Result<(), DidResolveError> {
        let ttl = ttl.unwrap_or(self.config.default_ttl);
        let rotation_grace = rotation_grace.unwrap_or(self.config.default_rotation_grace);
        
        let anchor = DidAnchor::new(pubkey, ttl, rotation_grace);
        
        let mut anchors = self.anchors.lock();
        
        // Check if we need to clean up old anchors
        if anchors.len() >= self.config.max_anchors {
            self.cleanup_expired_internal(&mut anchors);
        }
        
        anchors.insert(did, anchor);
        Ok(())
    }

    /// Rotate a DID anchor
    pub fn rotate_anchor(
        &self,
        did: &str,
        new_pubkey: DilithiumPubKey,
    ) -> Result<(), DidResolveError> {
        let mut anchors = self.anchors.lock();
        
        if let Some(anchor) = anchors.get_mut(did) {
            anchor.rotate(new_pubkey);
            Ok(())
        } else {
            Err(DidResolveError::NotFound)
        }
    }

    /// Remove a DID anchor
    pub fn remove_anchor(&self, did: &str) -> Result<(), DidResolveError> {
        let mut anchors = self.anchors.lock();
        
        if anchors.remove(did).is_some() {
            Ok(())
        } else {
            Err(DidResolveError::NotFound)
        }
    }

    /// Resolve a DID to a public key
    pub fn resolve_did(&self, did: &str) -> Result<DilithiumPubKey, DidResolveError> {
        let current_time = Instant::now();
        
        // Periodic cleanup
        self.maybe_cleanup(current_time);
        
        let anchors = self.anchors.lock();
        
        if let Some(anchor) = anchors.get(did) {
            if anchor.is_expired(current_time) {
                return Err(DidResolveError::Expired);
            }
            
            Ok(anchor.pubkey.clone())
        } else {
            Err(DidResolveError::NotFound)
        }
    }

    /// Resolve a DID to all valid public keys (including rotation grace)
    pub fn resolve_did_all_keys(&self, did: &str) -> Result<Vec<DilithiumPubKey>, DidResolveError> {
        let current_time = Instant::now();
        
        // Periodic cleanup
        self.maybe_cleanup(current_time);
        
        let anchors = self.anchors.lock();
        
        if let Some(anchor) = anchors.get(did) {
            let keys = anchor.get_valid_keys(current_time);
            if keys.is_empty() {
                Err(DidResolveError::Expired)
            } else {
                Ok(keys)
            }
        } else {
            Err(DidResolveError::NotFound)
        }
    }

    /// Check if a public key is valid for a DID
    pub fn verify_did_key(
        &self,
        did: &str,
        pubkey: &DilithiumPubKey,
    ) -> Result<bool, DidResolveError> {
        let current_time = Instant::now();
        
        // Periodic cleanup
        self.maybe_cleanup(current_time);
        
        let anchors = self.anchors.lock();
        
        if let Some(anchor) = anchors.get(did) {
            Ok(anchor.is_key_valid(pubkey, current_time))
        } else {
            Err(DidResolveError::NotFound)
        }
    }

    /// Get anchor information
    pub fn get_anchor_info(&self, did: &str) -> Option<DidAnchor> {
        let current_time = Instant::now();
        
        // Periodic cleanup
        self.maybe_cleanup(current_time);
        
        let anchors = self.anchors.lock();
        anchors.get(did).cloned()
    }

    /// Get all DID identifiers
    pub fn get_all_dids(&self) -> Vec<String> {
        let current_time = Instant::now();
        
        // Periodic cleanup
        self.maybe_cleanup(current_time);
        
        let anchors = self.anchors.lock();
        anchors.keys().cloned().collect()
    }

    /// Get resolver statistics
    pub fn get_stats(&self) -> DidResolverStats {
        let current_time = Instant::now();
        let anchors = self.anchors.lock();
        
        let mut total_anchors = 0;
        let mut expired_anchors = 0;
        let mut active_anchors = 0;
        let mut rotating_anchors = 0;
        
        for anchor in anchors.values() {
            total_anchors += 1;
            
            if anchor.is_expired(current_time) {
                expired_anchors += 1;
            } else {
                active_anchors += 1;
            }
            
            if anchor.rotated_at.is_some() {
                rotating_anchors += 1;
            }
        }
        
        DidResolverStats {
            total_anchors,
            expired_anchors,
            active_anchors,
            rotating_anchors,
            max_anchors: self.config.max_anchors,
        }
    }

    /// Clean up expired anchors
    pub fn cleanup_expired(&self) {
        let current_time = Instant::now();
        let mut anchors = self.anchors.lock();
        self.cleanup_expired_internal(&mut anchors);
        *self.last_cleanup.lock() = current_time;
    }

    /// Internal cleanup method
    fn cleanup_expired_internal(&self, anchors: &mut HashMap<String, DidAnchor>) {
        let current_time = Instant::now();
        let expired_dids: Vec<String> = anchors
            .iter()
            .filter_map(|(did, anchor)| {
                if anchor.is_expired(current_time) {
                    Some(did.clone())
                } else {
                    None
                }
            })
            .collect();
        
        for did in expired_dids {
            anchors.remove(&did);
        }
    }

    /// Check if cleanup is needed and perform it
    fn maybe_cleanup(&self, current_time: Instant) {
        let mut last_cleanup = self.last_cleanup.lock();
        if current_time - *last_cleanup > self.config.cleanup_interval {
            drop(last_cleanup); // Release lock before calling cleanup
            self.cleanup_expired();
        }
    }

    /// Load anchors from test fixtures
    #[cfg(test)]
    pub fn load_test_fixtures(&self) -> Result<(), DidResolveError> {
        // Load test DID anchors for testing
        let test_anchors = vec![
            ("did:test:issuer1", DilithiumPubKey::default()),
            ("did:test:issuer2", DilithiumPubKey::default()),
            ("did:test:issuer3", DilithiumPubKey::default()),
        ];
        
        for (did, pubkey) in test_anchors {
            self.add_anchor(did.to_string(), pubkey, None, None)?;
        }
        
        Ok(())
    }
}

impl Default for DidResolver {
    fn default() -> Self {
        Self::new(DidResolverConfig::default())
    }
}

/// DID resolver statistics
#[derive(Debug, Clone)]
pub struct DidResolverStats {
    /// Total number of anchors
    pub total_anchors: usize,
    /// Number of expired anchors
    pub expired_anchors: usize,
    /// Number of active anchors
    pub active_anchors: usize,
    /// Number of anchors in rotation
    pub rotating_anchors: usize,
    /// Maximum number of anchors allowed
    pub max_anchors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::dilithium::DilithiumPubKey;

    fn create_test_pubkey() -> DilithiumPubKey {
        DilithiumPubKey::default()
    }

    #[test]
    fn test_did_anchor_creation() {
        let pubkey = create_test_pubkey();
        let ttl = Duration::from_secs(3600);
        let rotation_grace = Duration::from_secs(300);
        
        let anchor = DidAnchor::new(pubkey.clone(), ttl, rotation_grace);
        
        assert_eq!(anchor.ttl, ttl);
        assert_eq!(anchor.rotation_grace, rotation_grace);
        assert!(anchor.rotated_at.is_none());
        assert!(anchor.previous_pubkey.is_none());
    }

    #[test]
    fn test_did_anchor_expiry() {
        let pubkey = create_test_pubkey();
        let ttl = Duration::from_secs(1);
        let rotation_grace = Duration::from_secs(300);
        
        let anchor = DidAnchor::new(pubkey, ttl, rotation_grace);
        let current_time = Instant::now();
        
        // Should not be expired immediately
        assert!(!anchor.is_expired(current_time));
        
        // Wait for expiry (in real test, we'd use a mock time)
        // For now, just test the logic
        let future_time = current_time + Duration::from_secs(2);
        assert!(anchor.is_expired(future_time));
    }

    #[test]
    fn test_did_anchor_rotation() {
        let pubkey1 = create_test_pubkey();
        let pubkey2 = create_test_pubkey();
        let ttl = Duration::from_secs(3600);
        let rotation_grace = Duration::from_secs(300);
        
        let mut anchor = DidAnchor::new(pubkey1.clone(), ttl, rotation_grace);
        
        // Before rotation
        assert_eq!(anchor.pubkey, pubkey1);
        assert!(anchor.rotated_at.is_none());
        assert!(anchor.previous_pubkey.is_none());
        
        // After rotation
        anchor.rotate(pubkey2.clone());
        
        assert_eq!(anchor.pubkey, pubkey2);
        assert!(anchor.rotated_at.is_some());
        assert_eq!(anchor.previous_pubkey, Some(pubkey1));
    }

    #[test]
    fn test_did_resolver_creation() {
        let config = DidResolverConfig::default();
        let resolver = DidResolver::new(config);
        
        let stats = resolver.get_stats();
        assert_eq!(stats.total_anchors, 0);
        assert_eq!(stats.active_anchors, 0);
    }

    #[test]
    fn test_did_resolver_add_anchor() {
        let resolver = DidResolver::default();
        let pubkey = create_test_pubkey();
        
        let result = resolver.add_anchor(
            "did:test:issuer1".to_string(),
            pubkey.clone(),
            None,
            None,
        );
        
        assert!(result.is_ok());
        
        let stats = resolver.get_stats();
        assert_eq!(stats.total_anchors, 1);
        assert_eq!(stats.active_anchors, 1);
    }

    #[test]
    fn test_did_resolver_resolve() {
        let resolver = DidResolver::default();
        let pubkey = create_test_pubkey();
        
        // Add anchor
        resolver.add_anchor(
            "did:test:issuer1".to_string(),
            pubkey.clone(),
            None,
            None,
        ).unwrap();
        
        // Resolve DID
        let resolved_pubkey = resolver.resolve_did("did:test:issuer1").unwrap();
        assert_eq!(resolved_pubkey, pubkey);
        
        // Resolve non-existent DID
        let result = resolver.resolve_did("did:test:nonexistent");
        assert_eq!(result, Err(DidResolveError::NotFound));
    }

    #[test]
    fn test_did_resolver_rotation() {
        let resolver = DidResolver::default();
        let pubkey1 = create_test_pubkey();
        let pubkey2 = create_test_pubkey();
        
        // Add initial anchor
        resolver.add_anchor(
            "did:test:issuer1".to_string(),
            pubkey1.clone(),
            None,
            None,
        ).unwrap();
        
        // Rotate anchor
        resolver.rotate_anchor("did:test:issuer1", pubkey2.clone()).unwrap();
        
        // Both keys should be valid during rotation grace
        let keys = resolver.resolve_did_all_keys("did:test:issuer1").unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&pubkey1));
        assert!(keys.contains(&pubkey2));
    }
}
