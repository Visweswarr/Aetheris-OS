//! Security Adapter for Polyglot Runtime
//!
//! This module provides integration between the Polyglot Runtime's capability
//! management and the kernel's global security module. It ensures:
//!
//! - Token revocation propagates from kernel security to all sandboxes
//! - Capability operations are logged via the kernel audit module
//! - The PolyglotBridge's cap_store stays synchronized with the global store

use alloc::vec::Vec;
use crate::klog;
use crate::security::{
    CapToken, CapabilityStore, grant_capability, check_capability, 
    revoke_capability_globally, get_capability_stats,
};
use crate::secman::audit::{self, AuditEntry, ops};
use super::bridge::PolyglotBridge;
use super::sandbox::SandboxId;
use super::backend::RuntimeError;

/// Security adapter for polyglot runtime integration
pub struct SecurityAdapter {
    /// Revoked tokens cache (for fast lookup)
    revoked_tokens: Vec<u128>,
    /// Audit logging enabled
    audit_enabled: bool,
}

impl SecurityAdapter {
    /// Create a new security adapter
    pub fn new() -> Self {
        Self {
            revoked_tokens: Vec::new(),
            audit_enabled: true,
        }
    }

    /// Connect a PolyglotBridge to the kernel's global CapabilityStore
    /// 
    /// This synchronizes the bridge's revocation list with the kernel's
    /// global revocation list.
    pub fn connect_bridge(&mut self, bridge: &mut PolyglotBridge) {
        // Sync revoked tokens from kernel to bridge
        for &token_id in &self.revoked_tokens {
            bridge.revoke_token(token_id);
        }
        
        klog!(INFO, "[SECURITY_ADAPTER] Connected bridge to kernel security module");
    }

    /// Revoke a capability token globally and propagate to all sandboxes
    /// 
    /// This method:
    /// 1. Revokes the token in the kernel's global CapabilityStore
    /// 2. Adds the token to the local revocation cache
    /// 3. Logs the revocation via the audit module
    pub fn revoke_token_globally(&mut self, token_id: u128) -> bool {
        // Revoke in kernel's global store
        let was_revoked = revoke_capability_globally(token_id);
        
        if was_revoked {
            // Add to local cache
            if !self.revoked_tokens.contains(&token_id) {
                self.revoked_tokens.push(token_id);
            }
            
            // Audit log
            if self.audit_enabled {
                self.log_capability_revocation(token_id);
            }
            
            klog!(INFO, "[SECURITY_ADAPTER] Globally revoked token 0x{:x}", token_id);
        }
        
        was_revoked
    }

    /// Propagate token revocation to a specific bridge
    pub fn propagate_revocation(&self, bridge: &mut PolyglotBridge, token_id: u128) {
        bridge.revoke_token(token_id);
        klog!(TRACE, "[SECURITY_ADAPTER] Propagated revocation of token 0x{:x} to bridge", token_id);
    }

    /// Propagate all revocations to a bridge
    pub fn propagate_all_revocations(&self, bridge: &mut PolyglotBridge) {
        for &token_id in &self.revoked_tokens {
            bridge.revoke_token(token_id);
        }
        klog!(TRACE, "[SECURITY_ADAPTER] Propagated {} revocations to bridge", 
              self.revoked_tokens.len());
    }

    /// Check if a token is revoked
    pub fn is_token_revoked(&self, token_id: u128) -> bool {
        self.revoked_tokens.contains(&token_id)
    }

    /// Verify a capability for a sandbox operation
    /// 
    /// This method checks the kernel's global CapabilityStore to verify
    /// that a sandbox has the required capability.
    pub fn verify_capability(
        &self,
        sandbox_id: SandboxId,
        destination: u64,
        operation: &str,
    ) -> Result<CapToken, RuntimeError> {
        // Use sandbox ID as process ID for capability lookup
        let process_id = sandbox_id.0;
        
        match check_capability(process_id, destination) {
            Ok(token) => {
                // Log successful verification
                if self.audit_enabled {
                    self.log_capability_check(sandbox_id, destination, true);
                }
                Ok(token)
            }
            Err(e) => {
                // Log failed verification
                if self.audit_enabled {
                    self.log_capability_check(sandbox_id, destination, false);
                }
                
                Err(RuntimeError::CapabilityDenied {
                    operation: operation.into(),
                    reason: crate::kformat!("Capability check failed: {}", e),
                })
            }
        }
    }

    /// Grant a capability to a sandbox
    pub fn grant_sandbox_capability(
        &self,
        sandbox_id: SandboxId,
        destination: u64,
        scope: u32,
        duration_ms: u64,
    ) -> Result<CapToken, RuntimeError> {
        let process_id = sandbox_id.0;
        
        match grant_capability(process_id, destination, scope, duration_ms) {
            Ok(token) => {
                if self.audit_enabled {
                    self.log_capability_grant(sandbox_id, destination, token.id);
                }
                Ok(token)
            }
            Err(e) => {
                Err(RuntimeError::CapabilityDenied {
                    operation: "grant".into(),
                    reason: crate::kformat!("Failed to grant capability: {}", e),
                })
            }
        }
    }

    /// Log a capability revocation to the audit module
    fn log_capability_revocation(&self, token_id: u128) {
        let entry = AuditEntry::new(
            0, // System process
            ops::SEC_CAP_REVOKE,
            token_id as u64,
        );
        audit::log(entry);
    }

    /// Log a capability check to the audit module
    fn log_capability_check(&self, sandbox_id: SandboxId, destination: u64, success: bool) {
        let op = if success {
            ops::SEC_CAP_CHECK
        } else {
            ops::SEC_CAP_DENY
        };
        
        let entry = AuditEntry::new(
            sandbox_id.0,
            op,
            destination,
        );
        audit::log(entry);
    }

    /// Log a capability grant to the audit module
    fn log_capability_grant(&self, sandbox_id: SandboxId, destination: u64, token_id: u128) {
        let entry = AuditEntry::new(
            sandbox_id.0,
            ops::SEC_CAP_GRANT,
            token_id as u64,
        );
        audit::log(entry);
    }

    /// Enable or disable audit logging
    pub fn set_audit_enabled(&mut self, enabled: bool) {
        self.audit_enabled = enabled;
    }

    /// Get the number of revoked tokens
    pub fn revoked_token_count(&self) -> usize {
        self.revoked_tokens.len()
    }

    /// Clear the revocation cache (for testing)
    #[cfg(test)]
    pub fn clear_revocation_cache(&mut self) {
        self.revoked_tokens.clear();
    }
}

impl Default for SecurityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Global security adapter instance
static SECURITY_ADAPTER: spin::Once<spin::Mutex<SecurityAdapter>> = spin::Once::new();

/// Initialize the global security adapter
pub fn init_security_adapter() {
    SECURITY_ADAPTER.call_once(|| spin::Mutex::new(SecurityAdapter::new()));
    klog!(INFO, "[SECURITY_ADAPTER] Security adapter initialized");
}

/// Get the global security adapter
pub fn get_security_adapter() -> &'static spin::Mutex<SecurityAdapter> {
    SECURITY_ADAPTER.get().expect("Security adapter not initialized")
}

/// Revoke a token globally via the security adapter
pub fn revoke_token(token_id: u128) -> bool {
    let adapter = get_security_adapter();
    adapter.lock().revoke_token_globally(token_id)
}

/// Check if a token is revoked
pub fn is_revoked(token_id: u128) -> bool {
    let adapter = get_security_adapter();
    adapter.lock().is_token_revoked(token_id)
}
