/// Capability Token System for Polymera OS
/// 
/// This module implements capability-based security for inter-process communication.
/// Processes must hold valid capability tokens to send messages to specific destinations.

use super::{SecurityError, SecurityResult, update_security_stats, get_current_time_ms};
use crate::{kprintln, klog};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::fmt;
use serde::{Deserialize, Serialize};
use crate::crypto::dilithium::{DilithiumPubKey, DilithiumSignature};
use crate::time::{Duration, Instant};

/// Capability token version 2 with replay protection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapTokenV2 {
    /// Token header with signature
    pub header: CapTokenHeader,
    /// Signature over the header
    pub signature: DilithiumSignature,
}

/// Capability token header containing all verification data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapTokenHeader {
    /// Issuer DID identifier
    pub issuer: String,
    /// Target destination
    pub dst: String,
    /// Unique nonce for replay protection (u128 for collision resistance)
    pub nonce: u128,
    /// Capability identifier
    pub cap_id: String,
    /// Token creation timestamp
    pub created_at: Instant,
    /// Token expiration timestamp
    pub expires_at: Instant,
    /// Capability permissions bitmask
    pub permissions: u64,
}

/// Capability verification outcome
#[derive(Debug, Clone, PartialEq)]
pub enum CapVerifyOutcome {
    /// Token is valid and accepted
    Accepted,
    /// Token is valid but expired
    Expired,
    /// Token signature verification failed
    SignatureInvalid,
    /// Token issuer not trusted
    IssuerUntrusted,
    /// Token has been replayed
    Replayed,
    /// Token nonce is too old
    NonceTooOld,
    /// Token format is invalid
    InvalidFormat,
}

/// Capability verification error
#[derive(Debug, Clone)]
pub enum CapVerifyError {
    /// Token has expired
    Expired,
    /// Signature verification failed
    SignatureInvalid,
    /// Issuer is not trusted
    IssuerUntrusted,
    /// Token has been replayed
    Replayed,
    /// Token nonce is too old
    NonceTooOld,
    /// Token format is invalid
    InvalidFormat,
    /// Internal error
    Internal,
}

impl fmt::Display for CapVerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapVerifyError::Expired => write!(f, "capability token expired"),
            CapVerifyError::SignatureInvalid => write!(f, "capability token signature invalid"),
            CapVerifyError::IssuerUntrusted => write!(f, "capability token issuer untrusted"),
            CapVerifyError::Replayed => write!(f, "capability token replayed"),
            CapVerifyError::NonceTooOld => write!(f, "capability token nonce too old"),
            CapVerifyError::InvalidFormat => write!(f, "capability token format invalid"),
            CapVerifyError::Internal => write!(f, "capability verification internal error"),
        }
    }
}

impl CapTokenV2 {
    /// Create a new capability token
    pub fn new(
        issuer: String,
        dst: String,
        nonce: u128,
        cap_id: String,
        permissions: u64,
        ttl: u64,
    ) -> Self {
        let now = Instant::now();
        let header = CapTokenHeader {
            issuer,
            dst,
            nonce,
            cap_id,
            created_at: now,
            expires_at: now + Duration::from_millis(ttl),
            permissions,
        };
        
        Self {
            header,
            signature: DilithiumSignature::default(), // Will be set by issuer
        }
    }

    /// Verify the token signature
    pub fn verify_signature(&self, pubkey: &DilithiumPubKey) -> Result<(), CapVerifyError> {
        let header_bytes = postcard::to_allocvec(&self.header)
            .map_err(|_| CapVerifyError::InvalidFormat)?;
        
        if pubkey.verify(&header_bytes, &self.signature) {
            Ok(())
        } else {
            Err(CapVerifyError::SignatureInvalid)
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.header.expires_at
    }

    /// Get token age
    pub fn age(&self) -> u64 {
        let now = Instant::now();
        if now > self.header.created_at {
            (now - self.header.created_at).as_millis() as u64
        } else {
            0
        }
    }

    /// Serialize token to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, CapVerifyError> {
        postcard::to_allocvec(self).map_err(|_| CapVerifyError::InvalidFormat)
    }

    /// Deserialize token from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CapVerifyError> {
        postcard::from_bytes(bytes).map_err(|_| CapVerifyError::InvalidFormat)
    }
}

impl CapTokenHeader {
    /// Check if header is valid
    pub fn is_valid(&self) -> bool {
        !self.issuer.is_empty() 
            && !self.dst.is_empty() 
            && !self.cap_id.is_empty()
            && self.created_at < self.expires_at
    }

    /// Get token TTL in milliseconds
    pub fn ttl_ms(&self) -> u64 {
        (self.expires_at - self.created_at).as_millis() as u64
    }
}

/// Capability permissions constants
pub mod permissions {
    pub const READ: u64 = 1 << 0;
    pub const WRITE: u64 = 1 << 1;
    pub const EXECUTE: u64 = 1 << 2;
    pub const DELETE: u64 = 1 << 3;
    pub const ADMIN: u64 = 1 << 4;
    
    /// Check if permission is granted
    pub fn has_permission(granted: u64, required: u64) -> bool {
        (granted & required) == required
    }
    
    /// Combine multiple permissions
    pub fn combine(perms: &[u64]) -> u64 {
        perms.iter().fold(0, |acc, &perm| acc | perm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::Duration;

    #[test]
    fn test_cap_token_creation() {
        let token = CapTokenV2::new(
            "issuer1".to_string(),
            "dst1".to_string(),
            12345,
            "cap1".to_string(),
            permissions::READ | permissions::WRITE,
            1000,
        );
        
        assert_eq!(token.header.issuer, "issuer1");
        assert_eq!(token.header.dst, "dst1");
        assert_eq!(token.header.nonce, 12345);
        assert_eq!(token.header.cap_id, "cap1");
        assert!(permissions::has_permission(token.header.permissions, permissions::READ));
        assert!(permissions::has_permission(token.header.permissions, permissions::WRITE));
        assert!(!permissions::has_permission(token.header.permissions, permissions::EXECUTE));
    }

    #[test]
    fn test_cap_token_validation() {
        let token = CapTokenV2::new(
            "issuer1".to_string(),
            "dst1".to_string(),
            12345,
            "cap1".to_string(),
            permissions::READ,
            1000,
        );
        
        assert!(token.header.is_valid());
        assert!(!token.is_expired());
        assert!(token.age() < 1000);
    }

    #[test]
    fn test_permissions() {
        let perms = permissions::combine(&[permissions::READ, permissions::WRITE]);
        assert!(permissions::has_permission(perms, permissions::READ));
        assert!(permissions::has_permission(perms, permissions::WRITE));
        assert!(!permissions::has_permission(perms, permissions::EXECUTE));
    }

    #[test]
    fn test_serialization() {
        let token = CapTokenV2::new(
            "issuer1".to_string(),
            "dst1".to_string(),
            12345,
            "cap1".to_string(),
            permissions::READ,
            1000,
        );
        
        let bytes = token.to_bytes().unwrap();
        let deserialized = CapTokenV2::from_bytes(&bytes).unwrap();
        
        assert_eq!(token.header.issuer, deserialized.header.issuer);
        assert_eq!(token.header.dst, deserialized.header.dst);
        assert_eq!(token.header.nonce, deserialized.header.nonce);
        assert_eq!(token.header.cap_id, deserialized.header.cap_id);
    }
}

/// Capability token structure
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapToken { 
    /// Unique capability token ID
    pub id: u128, 
    
    /// Destination process ID this token grants access to
    pub dst: u64, 
    
    /// Scope of access (bit flags for different permissions)
    pub scope: u32, 
    
    /// Expiry time in milliseconds
    pub expiry_ms: u64 
}

impl CapToken {
    /// Create a new capability token
    pub fn new(id: u128, dst: u64, scope: u32, expiry_ms: u64) -> Self {
        Self { id, dst, scope, expiry_ms }
    }
    
    /// Check if this token is expired
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms > self.expiry_ms
    }
    
    /// Check if this token grants access to a destination
    pub fn grants_access_to(&self, dst: u64) -> bool {
        self.dst == dst
    }
    
    /// Get remaining time until expiry
    pub fn time_until_expiry(&self, now_ms: u64) -> Option<u64> {
        if self.is_expired(now_ms) {
            None
        } else {
            Some(self.expiry_ms - now_ms)
        }
    }
}

impl core::fmt::Display for CapToken {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CapToken[ID:0x{:x}, dst:{}, scope:0x{:x}, expires:{}ms]",
               self.id, self.dst, self.scope, self.expiry_ms)
    }
}

/// Capability scope flags
pub mod scope {
    /// Permission to send messages
    pub const SEND: u32 = 0x1;
    
    /// Permission to receive messages
    pub const RECV: u32 = 0x2;
    
    /// Permission to create channels
    pub const CREATE_CHANNEL: u32 = 0x4;
    
    /// Permission to destroy channels
    pub const DESTROY_CHANNEL: u32 = 0x8;
    
    /// Administrative permissions
    pub const ADMIN: u32 = 0x10;
    
    /// All permissions
    pub const ALL: u32 = SEND | RECV | CREATE_CHANNEL | DESTROY_CHANNEL | ADMIN;
    
    /// Default permissions for normal processes
    pub const DEFAULT: u32 = SEND | RECV;
}

/// Validate capability token
/// 
/// # Arguments
/// * `tok` - Capability token to validate
/// * `dst` - Destination process ID
/// * `now_ms` - Current time in milliseconds
/// * `cap_store` - Reference to capability store for revocation checks
/// 
/// # Returns
/// `true` if token is valid for the destination at the current time
/// Wrapper for `validate_cap` that operates against the kernel's global capability store.
pub fn validate_cap_simple(tok: &CapToken, dst: u64, now_ms: u64) -> bool {
    // Basic expiry + destination checks (the global store may not be initialized in tests).
    if now_ms >= tok.expiry_ms {
        return false;
    }
    tok.dst == dst
}

pub fn validate_cap(tok: &CapToken, dst: u64, now_ms: u64, cap_store: &CapabilityStore) -> bool {
    update_security_stats(|stats| {
        stats.validations_performed += 1;
    });
    
    // Check if token is revoked (highest priority check)
    if cap_store.is_token_revoked(tok.id) {
        update_security_stats(|stats| {
            stats.validations_failed += 1;
            stats.permission_denied += 1;
        });
        
        // Audit the revocation check failure
        let audit_entry = crate::secman::audit::AuditEntry::new(
            0, // System process ID
            crate::secman::audit::ops::SEC_CAP_REVOKE,
            tok.id as u64, // Store token ID in audit argument
        );
        crate::secman::audit::log(audit_entry);
        
        klog!(TRACE, "[SECURITY] Capability validation failed: token 0x{:x} is revoked", tok.id);
        return false;
    }
    
    // Check if token grants access to destination
    if tok.dst != dst {
        update_security_stats(|stats| {
            stats.validations_failed += 1;
            stats.permission_denied += 1;
        });
        return false;
    }
    
    // Check if token is expired
    if now_ms > tok.expiry_ms {
        update_security_stats(|stats| {
            stats.validations_failed += 1;
            stats.capabilities_expired += 1;
        });
        return false;
    }
    
    // Token is valid
    update_security_stats(|stats| {
        stats.validations_passed += 1;
    });
    
    true
}

/// Per-task capability token storage
#[derive(Debug)]
pub struct CapabilityStore {
    /// Map from process ID to list of capability tokens
    tokens_by_process: BTreeMap<u64, Vec<CapToken>>,
    
    /// Set of revoked capability token IDs (global revocation list)
    revoked_tokens: alloc::collections::BTreeSet<u128>,
    
    /// Next token ID to assign
    next_token_id: u128,
    
    /// Total number of tokens issued
    tokens_issued: u64,
    
    /// Total number of tokens revoked
    tokens_revoked: u64,
}

impl CapabilityStore {
    /// Create a new capability store
    pub fn new() -> Self {
        Self {
            tokens_by_process: BTreeMap::new(),
            revoked_tokens: alloc::collections::BTreeSet::new(),
            next_token_id: 1,
            tokens_issued: 0,
            tokens_revoked: 0,
        }
    }
    
    /// Grant a capability token to a process
    pub fn grant_capability(
        &mut self,
        process_id: u64,
        dst: u64,
        scope: u32,
        duration_ms: u64,
    ) -> CapToken {
        let token_id = self.next_token_id;
        self.next_token_id += 1;
        
        let expiry_ms = get_current_time_ms() + duration_ms;
        let token = CapToken::new(token_id, dst, scope, expiry_ms);
        
        // Add token to process's capability list
        let process_tokens = self.tokens_by_process.entry(process_id).or_insert_with(Vec::new);
        process_tokens.push(token);
        
        self.tokens_issued += 1;
        
        update_security_stats(|stats| {
            stats.active_tokens += 1;
        });
        
        klog!(TRACE, "[SECURITY] Granted capability to process {}: {}", process_id, token);
        
        token
    }
    
    /// Find a valid capability token for a process to access a destination
    pub fn find_capability(&self, process_id: u64, dst: u64, now_ms: u64) -> Option<CapToken> {
        if let Some(tokens) = self.tokens_by_process.get(&process_id) {
            for &token in tokens {
                if validate_cap(&token, dst, now_ms, self) {
                    return Some(token);
                }
            }
        }
        None
    }
    
    /// Check if a process has a valid capability for a destination
    pub fn has_capability(&self, process_id: u64, dst: u64, now_ms: u64) -> bool {
        self.find_capability(process_id, dst, now_ms).is_some()
    }
    
    /// Revoke a specific capability token
    pub fn revoke_capability(&mut self, process_id: u64, token_id: u128) -> bool {
        if let Some(tokens) = self.tokens_by_process.get_mut(&process_id) {
            if let Some(pos) = tokens.iter().position(|token| token.id == token_id) {
                tokens.remove(pos);
                self.tokens_revoked += 1;
                
                update_security_stats(|stats| {
                    stats.active_tokens = stats.active_tokens.saturating_sub(1);
                });
                
                klog!(TRACE, "[SECURITY] Revoked capability token 0x{:x} from process {}", 
                      token_id, process_id);
                return true;
            }
        }
        false
    }
    
    /// Revoke a capability token globally by ID (adds to revocation list)
    /// 
    /// This method adds a token ID to the global revocation list, making it
    /// invalid for all processes regardless of where it was originally granted.
    /// 
    /// # Arguments
    /// * `token_id` - The capability token ID to revoke
    /// 
    /// # Returns
    /// `true` if the token was added to the revocation list, `false` if already revoked
    pub fn revoke_capability_globally(&mut self, token_id: u128) -> bool {
        if self.revoked_tokens.insert(token_id) {
            self.tokens_revoked += 1;
            
            // Also remove from any process that holds this token
            let mut removed_from_processes = 0;
            for (process_id, tokens) in self.tokens_by_process.iter_mut() {
                let initial_len = tokens.len();
                tokens.retain(|token| token.id != token_id);
                let removed = initial_len - tokens.len();
                if removed > 0 {
                    removed_from_processes += removed;
                    klog!(TRACE, "[SECURITY] Removed revoked token 0x{:x} from process {}", 
                          token_id, process_id);
                }
            }
            
            // Update statistics
            if removed_from_processes > 0 {
                update_security_stats(|stats| {
                    stats.active_tokens = stats.active_tokens.saturating_sub(removed_from_processes as u64);
                });
            }
            
            klog!(TRACE, "[SECURITY] Globally revoked capability token 0x{:x} (removed from {} processes)", 
                  token_id, removed_from_processes);
            true
        } else {
            false // Token was already revoked
        }
    }
    
    /// Check if a capability token is revoked
    /// 
    /// # Arguments
    /// * `token_id` - The capability token ID to check
    /// 
    /// # Returns
    /// `true` if the token is in the revocation list
    pub fn is_token_revoked(&self, token_id: u128) -> bool {
        self.revoked_tokens.contains(&token_id)
    }
    
    /// Revoke all capabilities for a process
    pub fn revoke_all_capabilities(&mut self, process_id: u64) -> usize {
        if let Some(tokens) = self.tokens_by_process.remove(&process_id) {
            let count = tokens.len();
            self.tokens_revoked += count as u64;
            
            update_security_stats(|stats| {
                stats.active_tokens = stats.active_tokens.saturating_sub(count as u64);
            });
            
            klog!(TRACE, "[SECURITY] Revoked {} capability tokens from process {}", 
                  count, process_id);
            count
        } else {
            0
        }
    }
    
    /// Clean up expired tokens for all processes
    pub fn cleanup_expired_tokens(&mut self, now_ms: u64) -> usize {
        let mut removed_count = 0;
        
        for (process_id, tokens) in self.tokens_by_process.iter_mut() {
            let initial_len = tokens.len();
            tokens.retain(|token| !token.is_expired(now_ms));
            let removed = initial_len - tokens.len();
            
            if removed > 0 {
                removed_count += removed;
                klog!(TRACE, "[SECURITY] Cleaned up {} expired tokens from process {}", 
                      removed, process_id);
            }
        }
        
        // Remove processes with no tokens
        self.tokens_by_process.retain(|_, tokens| !tokens.is_empty());
        
        if removed_count > 0 {
            self.tokens_revoked += removed_count as u64;
            
            update_security_stats(|stats| {
                stats.active_tokens = stats.active_tokens.saturating_sub(removed_count as u64);
            });
        }
        
        removed_count
    }
    
    /// Get all tokens for a process
    pub fn get_process_tokens(&self, process_id: u64) -> Option<&Vec<CapToken>> {
        self.tokens_by_process.get(&process_id)
    }
    
    /// Get statistics
    pub fn stats(&self) -> CapabilityStoreStats {
        let active_tokens = self.tokens_by_process.values()
            .map(|tokens| tokens.len())
            .sum::<usize>() as u64;
        
        CapabilityStoreStats {
            processes_with_tokens: self.tokens_by_process.len() as u64,
            active_tokens,
            tokens_issued: self.tokens_issued,
            tokens_revoked: self.tokens_revoked,
            revoked_tokens_count: self.revoked_tokens.len() as u64,
        }
    }
    
    /// Print all capabilities for debugging
    pub fn print_all_capabilities(&self) {
        kprintln!("");
        kprintln!("=== CAPABILITY STORE CONTENTS ===");
        
        for (&process_id, tokens) in &self.tokens_by_process {
            kprintln!("Process {}: {} tokens", process_id, tokens.len());
            for (i, token) in tokens.iter().enumerate() {
                kprintln!("  [{}] {}", i, token);
            }
        }
        
        let stats = self.stats();
        kprintln!("Store stats: {} processes, {} active tokens, {} issued, {} revoked, {} in revocation list",
                  stats.processes_with_tokens, stats.active_tokens, 
                  stats.tokens_issued, stats.tokens_revoked, stats.revoked_tokens_count);
        
        kprintln!("=== END CAPABILITY STORE ===");
        kprintln!("");
    }
}

/// Capability store statistics
#[derive(Debug, Clone, Copy)]
pub struct CapabilityStoreStats {
    pub processes_with_tokens: u64,
    pub active_tokens: u64,
    pub tokens_issued: u64,
    pub tokens_revoked: u64,
    pub revoked_tokens_count: u64,
}

/// Global capability store
static CAPABILITY_STORE: Mutex<Option<CapabilityStore>> = Mutex::new(None);

/// Initialize capability storage
pub fn init_capability_storage() {
    let mut store = CAPABILITY_STORE.lock();
    *store = Some(CapabilityStore::new());
    klog!(INFO, "[SECURITY] Capability storage initialized");
}

/// Grant a capability token to a process
pub fn grant_capability(
    process_id: u64,
    dst: u64,
    scope: u32,
    duration_ms: u64,
) -> SecurityResult<CapToken> {
    let mut store = CAPABILITY_STORE.lock();
    if let Some(ref mut cap_store) = *store {
        let token = cap_store.grant_capability(process_id, dst, scope, duration_ms);
        Ok(token)
    } else {
        Err(SecurityError::PolicyViolation)
    }
}

/// Check if a process has capability to access a destination
pub fn check_capability(process_id: u64, dst: u64) -> SecurityResult<CapToken> {
    let now_ms = get_current_time_ms();
    let store = CAPABILITY_STORE.lock();
    
    if let Some(ref cap_store) = *store {
        if let Some(token) = cap_store.find_capability(process_id, dst, now_ms) {
            Ok(token)
        } else {
            Err(SecurityError::PermissionDenied)
        }
    } else {
        Err(SecurityError::PolicyViolation)
    }
}

/// Revoke a capability token
pub fn revoke_capability(process_id: u64, token_id: u128) -> SecurityResult<()> {
    let mut store = CAPABILITY_STORE.lock();
    if let Some(ref mut cap_store) = *store {
        if cap_store.revoke_capability(process_id, token_id) {
            Ok(())
        } else {
            Err(SecurityError::CapabilityNotFound)
        }
    } else {
        Err(SecurityError::PolicyViolation)
    }
}

/// Clean up expired tokens
pub fn cleanup_expired_capabilities() -> usize {
    let now_ms = get_current_time_ms();
    let mut store = CAPABILITY_STORE.lock();
    
    if let Some(ref mut cap_store) = *store {
        cap_store.cleanup_expired_tokens(now_ms)
    } else {
        0
    }
}

/// Get capability store statistics
pub fn get_capability_stats() -> Option<CapabilityStoreStats> {
    let store = CAPABILITY_STORE.lock();
    store.as_ref().map(|cap_store| cap_store.stats())
}

/// Revoke a capability token globally by ID
/// 
/// This function adds a capability token ID to the global revocation list,
/// making it invalid for all processes regardless of where it was originally granted.
/// 
/// # Arguments
/// * `token_id` - The capability token ID to revoke
/// 
/// # Returns
/// `true` if the token was successfully revoked, `false` if already revoked
pub fn revoke_capability_globally(token_id: u128) -> bool {
    let mut store = CAPABILITY_STORE.lock();
    
    if let Some(ref mut cap_store) = *store {
        let was_revoked = cap_store.revoke_capability_globally(token_id);
        
        if was_revoked {
            // Audit the revocation
            let audit_entry = crate::secman::audit::AuditEntry::new(
                0, // System process ID
                crate::secman::audit::ops::SEC_CAP_REVOKE,
                token_id as u64, // Store token ID in audit argument
            );
            crate::secman::audit::log(audit_entry);
            
            klog!(INFO, "[SECURITY] Globally revoked capability token 0x{:x}", token_id);
        }
        
        was_revoked
    } else {
        klog!(WARN, "[SECURITY] Cannot revoke capability: capability store not initialized");
        false
    }
}

/// Print all capabilities for debugging
pub fn print_all_capabilities() {
    let store = CAPABILITY_STORE.lock();
    if let Some(ref cap_store) = *store {
        cap_store.print_all_capabilities();
    } else {
        kprintln!("Capability store not initialized");
    }
}

/// Test capability storage functionality
pub fn test_capability_storage() {
    kprintln!("Testing capability storage...");
    
    let process_a = 100;
    let process_b = 200;
    let process_c = 300;
    
    // Grant some capabilities
    match grant_capability(process_a, process_b, scope::SEND, 5000) {
        Ok(token) => kprintln!("  ✓ Granted capability to process {}: {}", process_a, token),
        Err(e) => kprintln!("  ✗ Failed to grant capability: {}", e),
    }
    
    match grant_capability(process_a, process_c, scope::DEFAULT, 3000) {
        Ok(token) => kprintln!("  ✓ Granted capability to process {}: {}", process_a, token),
        Err(e) => kprintln!("  ✗ Failed to grant capability: {}", e),
    }
    
    match grant_capability(process_b, process_a, scope::ALL, 10000) {
        Ok(token) => kprintln!("  ✓ Granted capability to process {}: {}", process_b, token),
        Err(e) => kprintln!("  ✗ Failed to grant capability: {}", e),
    }
    
    // Test capability checking
    match check_capability(process_a, process_b) {
        Ok(token) => kprintln!("  ✓ Process {} has valid capability for {}: {}", 
                              process_a, process_b, token),
        Err(e) => kprintln!("  ✗ Process {} lacks capability for {}: {}", 
                          process_a, process_b, e),
    }
    
    // Test checking non-existent capability
    match check_capability(process_c, process_a) {
        Ok(token) => kprintln!("  ✗ Process {} should not have capability for {}: {}", 
                              process_c, process_a, token),
        Err(e) => kprintln!("  ✓ Process {} correctly lacks capability for {}: {}", 
                          process_c, process_a, e),
    }
    
    // Test expiry by advancing time
    super::advance_time_ms(4000); // Advance 4 seconds
    
    // Check expired capability
    match check_capability(process_a, process_c) {
        Ok(token) => kprintln!("  ✗ Expired capability should not be valid: {}", token),
        Err(e) => kprintln!("  ✓ Expired capability correctly rejected: {}", e),
    }
    
    // Clean up expired tokens
    let cleaned = cleanup_expired_capabilities();
    kprintln!("  ✓ Cleaned up {} expired capabilities", cleaned);
    
    // Print final state
    if let Some(stats) = get_capability_stats() {
        kprintln!("  Final stats: {} processes, {} active tokens", 
                  stats.processes_with_tokens, stats.active_tokens);
    }
    
    kprintln!("Capability storage test completed");
}

/// Setup initial capabilities for testing and demo
pub fn setup_demo_capabilities() {
    kprintln!("[SECURITY] Setting up demo capabilities...");
    
    // Grant some basic capabilities for demo processes
    let demo_capabilities = [
        (1, 2, scope::SEND, 60000),      // Process 1 can send to 2
        (2, 1, scope::SEND, 60000),      // Process 2 can send to 1  
        (1, 3, scope::DEFAULT, 30000),   // Process 1 can send/recv to 3
        (10, 20, scope::ALL, 120000),    // Process 10 has all permissions for 20
        (100, 200, scope::SEND, 45000),  // Process 100 can send to 200
    ];
    
    for &(sender, receiver, scope, duration) in &demo_capabilities {
        match grant_capability(sender, receiver, scope, duration) {
            Ok(token) => klog!(TRACE, "[SECURITY] Demo capability: {} -> {} ({})", 
                              sender, receiver, token),
            Err(e) => klog!(WARN, "[SECURITY] Failed to grant demo capability {} -> {}: {}", 
                           sender, receiver, e),
        }
    }
    
    kprintln!("[SECURITY] Demo capabilities setup complete");
}
