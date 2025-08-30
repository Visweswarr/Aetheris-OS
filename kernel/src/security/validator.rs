/// Capability Validator Module for Polymera OS Security System
/// 
/// This module provides comprehensive validation functions for capability tokens,
/// ensuring that only authorized processes can perform restricted operations.

use super::cap::{CapToken, SecurityError, SecurityResult};
use crate::{kprintln, klog};
use core::sync::atomic::{AtomicU64, Ordering};

/// Validation statistics
#[derive(Debug, Clone, Copy)]
pub struct ValidationStats {
    /// Total validations performed
    pub total_validations: u64,
    
    /// Successful validations
    pub successful_validations: u64,
    
    /// Failed validations due to expiry
    pub expiry_failures: u64,
    
    /// Failed validations due to wrong destination
    pub destination_failures: u64,
    
    /// Failed validations due to other reasons
    pub other_failures: u64,
}

impl ValidationStats {
    pub const fn new() -> Self {
        Self {
            total_validations: 0,
            successful_validations: 0,
            expiry_failures: 0,
            destination_failures: 0,
            other_failures: 0,
        }
    }
    
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f32 {
        if self.total_validations > 0 {
            (self.successful_validations as f32 / self.total_validations as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Global validation statistics
static VALIDATION_STATS: AtomicU64 = AtomicU64::new(0); // Packed stats for atomic operations

/// Pack validation statistics into a single u64
fn pack_validation_stats(total: u16, success: u16, expiry: u16, dest: u16) -> u64 {
    ((total as u64) << 48) | ((success as u64) << 32) | ((expiry as u64) << 16) | (dest as u64)
}

/// Unpack validation statistics from a single u64
fn unpack_validation_stats(packed: u64) -> (u16, u16, u16, u16) {
    let total = (packed >> 48) as u16;
    let success = (packed >> 32) as u16;
    let expiry = (packed >> 16) as u16;
    let dest = packed as u16;
    (total, success, expiry, dest)
}

/// Update validation statistics
fn update_validation_stats(success: bool, expiry_fail: bool, dest_fail: bool) {
    let current = VALIDATION_STATS.load(Ordering::Relaxed);
    let (mut total, mut success_count, mut expiry, mut dest) = unpack_validation_stats(current);
    
    total = total.saturating_add(1);
    
    if success {
        success_count = success_count.saturating_add(1);
    } else if expiry_fail {
        expiry = expiry.saturating_add(1);
    } else if dest_fail {
        dest = dest.saturating_add(1);
    }
    
    let new_packed = pack_validation_stats(total, success_count, expiry, dest);
    VALIDATION_STATS.store(new_packed, Ordering::Relaxed);
}

/// Validation failure reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationFailure {
    /// Token has expired
    Expired,
    
    /// Wrong destination
    WrongDestination,
    
    /// Invalid token format
    InvalidToken,
    
    /// Insufficient privileges
    InsufficientPrivileges,
    
    /// Token revoked
    Revoked,
    
    /// Unknown error
    Unknown,
}

impl ValidationFailure {
    /// Convert to SecurityError
    pub fn to_security_error(self) -> SecurityError {
        match self {
            ValidationFailure::Expired => SecurityError::CapabilityExpired,
            ValidationFailure::WrongDestination => SecurityError::PermissionDenied,
            ValidationFailure::InvalidToken => SecurityError::InvalidCapability,
            ValidationFailure::InsufficientPrivileges => SecurityError::PermissionDenied,
            ValidationFailure::Revoked => SecurityError::CapabilityNotFound,
            ValidationFailure::Unknown => SecurityError::PolicyViolation,
        }
    }
    
    /// Get audit operation code for this failure
    pub fn audit_op_code(self) -> u16 {
        match self {
            ValidationFailure::Expired => crate::secman::audit::ops::SEC_AUTH_FAIL + 10, // 112
            ValidationFailure::WrongDestination => crate::secman::audit::ops::SEC_AUTH_FAIL + 11, // 113
            ValidationFailure::InvalidToken => crate::secman::audit::ops::SEC_AUTH_FAIL + 12, // 114
            ValidationFailure::InsufficientPrivileges => crate::secman::audit::ops::SEC_AUTH_FAIL + 13, // 115
            ValidationFailure::Revoked => crate::secman::audit::ops::SEC_AUTH_FAIL + 14, // 116
            ValidationFailure::Unknown => crate::secman::audit::ops::SEC_AUTH_FAIL + 15, // 117
        }
    }
}

impl core::fmt::Display for ValidationFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValidationFailure::Expired => write!(f, "Token expired"),
            ValidationFailure::WrongDestination => write!(f, "Wrong destination"),
            ValidationFailure::InvalidToken => write!(f, "Invalid token"),
            ValidationFailure::InsufficientPrivileges => write!(f, "Insufficient privileges"),
            ValidationFailure::Revoked => write!(f, "Token revoked"),
            ValidationFailure::Unknown => write!(f, "Unknown error"),
        }
    }
}

/// Validation result
pub type ValidationResult<T> = Result<T, ValidationFailure>;

/// Validate capability token expiry
/// 
/// Checks if the capability token has expired based on the current time.
/// 
/// # Arguments
/// * `token` - The capability token to validate
/// * `current_time_ms` - Current time in milliseconds
/// 
/// # Returns
/// `Ok(())` if token is not expired, `Err(ValidationFailure::Expired)` if expired
pub fn validate_cap_expiry(token: &CapToken, current_time_ms: u64) -> ValidationResult<()> {
    klog!(TRACE, [crate::log::tags::VALIDATOR], "Checking expiry: token expires at {}, current time {}", 
          token.expiry_ms, current_time_ms);
    
    if current_time_ms > token.expiry_ms {
        klog!(TRACE, [crate::log::tags::VALIDATOR], "Token expired ({} > {})", current_time_ms, token.expiry_ms);
        
        // Log audit entry for expired token
        crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
            0, // System operation
            ValidationFailure::Expired.audit_op_code(),
            token.id as u64
        ));
        
        update_validation_stats(false, true, false);
        Err(ValidationFailure::Expired)
    } else {
        klog!(TRACE, "[VALIDATOR] Token expiry valid");
        Ok(())
    }
}

/// Validate capability token destination
/// 
/// Checks if the capability token grants access to the specified destination.
/// 
/// # Arguments
/// * `token` - The capability token to validate
/// * `target_dst` - The target destination to check access for
/// 
/// # Returns
/// `Ok(())` if destination matches, `Err(ValidationFailure::WrongDestination)` if not
pub fn validate_cap_dst(token: &CapToken, target_dst: u64) -> ValidationResult<()> {
    klog!(TRACE, "[VALIDATOR] Checking destination: token dst {}, target dst {}", 
          token.dst, target_dst);
    
    if token.dst != target_dst {
        klog!(TRACE, "[VALIDATOR] Destination mismatch ({} != {})", token.dst, target_dst);
        
        // Log audit entry for wrong destination
        crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
            0, // System operation
            ValidationFailure::WrongDestination.audit_op_code(),
            ((token.dst as u64) << 32) | target_dst
        ));
        
        update_validation_stats(false, false, true);
        Err(ValidationFailure::WrongDestination)
    } else {
        klog!(TRACE, "[VALIDATOR] Destination valid");
        Ok(())
    }
}

/// Validate capability token scope for specific operation
/// 
/// Checks if the capability token has the required scope permissions.
/// 
/// # Arguments
/// * `token` - The capability token to validate
/// * `required_scope` - The scope permissions required for the operation
/// 
/// # Returns
/// `Ok(())` if scope is sufficient, `Err(ValidationFailure::InsufficientPrivileges)` if not
pub fn validate_cap_scope(token: &CapToken, required_scope: u32) -> ValidationResult<()> {
    klog!(TRACE, "[VALIDATOR] Checking scope: token scope 0x{:x}, required 0x{:x}", 
          token.scope, required_scope);
    
    if (token.scope & required_scope) != required_scope {
        klog!(TRACE, "[VALIDATOR] Insufficient scope (0x{:x} & 0x{:x} != 0x{:x})", 
              token.scope, required_scope, required_scope);
        
        // Log audit entry for insufficient privileges
        crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
            0, // System operation
            ValidationFailure::InsufficientPrivileges.audit_op_code(),
            ((token.scope as u64) << 32) | (required_scope as u64)
        ));
        
        update_validation_stats(false, false, false);
        Err(ValidationFailure::InsufficientPrivileges)
    } else {
        klog!(TRACE, "[VALIDATOR] Scope valid");
        Ok(())
    }
}

/// Comprehensive capability validation
/// 
/// Performs all validation checks on a capability token for a specific operation.
/// 
/// # Arguments
/// * `token` - The capability token to validate
/// * `target_dst` - The target destination to check access for
/// * `required_scope` - The scope permissions required for the operation
/// * `current_time_ms` - Current time in milliseconds
/// 
/// # Returns
/// `Ok(())` if all validations pass, `Err(ValidationFailure)` on first failure
pub fn validate_all(
    token: &CapToken, 
    target_dst: u64, 
    required_scope: u32, 
    current_time_ms: u64
) -> ValidationResult<()> {
    klog!(TRACE, "[VALIDATOR] Starting comprehensive validation for token 0x{:x}", token.id);
    
    // Validate expiry first (most common failure)
    validate_cap_expiry(token, current_time_ms)?;
    
    // Validate destination
    validate_cap_dst(token, target_dst)?;
    
    // Validate scope
    validate_cap_scope(token, required_scope)?;
    
    // All validations passed
    update_validation_stats(true, false, false);
    
    // Log successful validation
    crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
        0, // System operation
        crate::secman::audit::ops::SEC_CAP_GRANT, // Successful validation
        token.id as u64
    ));
    
    klog!(TRACE, "[VALIDATOR] All validations passed for token 0x{:x}", token.id);
    Ok(())
}

/// Validate capability with custom validation time
/// 
/// Same as validate_all but allows specifying a custom validation time.
/// Useful for testing and scenarios where current time might not be appropriate.
pub fn validate_all_at_time(
    token: &CapToken, 
    target_dst: u64, 
    required_scope: u32, 
    validation_time_ms: u64
) -> ValidationResult<()> {
    validate_all(token, target_dst, required_scope, validation_time_ms)
}

/// Quick validation for IPC send operations
/// 
/// Optimized validation specifically for IPC send operations.
/// Only checks expiry and destination, assumes SEND scope.
pub fn validate_for_ipc_send(token: &CapToken, target_dst: u64) -> ValidationResult<()> {
    let current_time = crate::security::get_current_time_ms();
    validate_all(token, target_dst, crate::security::cap::scope::SEND, current_time)
}

/// Batch validation for multiple tokens
/// 
/// Validates multiple capability tokens at once, returning the first valid one.
/// Useful when a process has multiple tokens that could grant access.
pub fn validate_batch(
    tokens: &[CapToken], 
    target_dst: u64, 
    required_scope: u32
) -> ValidationResult<CapToken> {
    let current_time = crate::security::get_current_time_ms();
    
    for token in tokens {
        if validate_all(token, target_dst, required_scope, current_time).is_ok() {
            return Ok(*token);
        }
    }
    
    // No valid token found
    update_validation_stats(false, false, false);
    Err(ValidationFailure::Unknown)
}

/// Get current validation statistics
pub fn get_validation_stats() -> ValidationStats {
    let packed = VALIDATION_STATS.load(Ordering::Relaxed);
    let (total, success, expiry, dest) = unpack_validation_stats(packed);
    
    ValidationStats {
        total_validations: total as u64,
        successful_validations: success as u64,
        expiry_failures: expiry as u64,
        destination_failures: dest as u64,
        other_failures: (total as u64).saturating_sub(success as u64 + expiry as u64 + dest as u64),
    }
}

/// Clear validation statistics
pub fn clear_validation_stats() {
    VALIDATION_STATS.store(0, Ordering::Relaxed);
    klog!(INFO, "[VALIDATOR] Validation statistics cleared");
}

/// Print validation statistics
pub fn print_validation_stats() {
    let stats = get_validation_stats();
    
    kprintln!("");
    kprintln!("=== CAPABILITY VALIDATION STATISTICS ===");
    kprintln!("Total validations: {}", stats.total_validations);
    kprintln!("Successful: {}", stats.successful_validations);
    kprintln!("Failed - Expired: {}", stats.expiry_failures);
    kprintln!("Failed - Wrong destination: {}", stats.destination_failures);
    kprintln!("Failed - Other: {}", stats.other_failures);
    
    if stats.total_validations > 0 {
        kprintln!("Success rate: {:.1}%", stats.success_rate());
    }
    
    kprintln!("=== END VALIDATION STATISTICS ===");
    kprintln!("");
}

/// Test helper to create an expired token
pub fn create_expired_token(dst: u64) -> CapToken {
    let past_time = crate::security::get_current_time_ms().saturating_sub(1000);
    CapToken {
        id: 0xDEADBEEF,
        dst,
        scope: crate::security::cap::scope::SEND,
        expiry_ms: past_time,
    }
}

/// Test helper to create a valid token
pub fn create_valid_token(dst: u64) -> CapToken {
    let future_time = crate::security::get_current_time_ms() + 10000;
    CapToken {
        id: 0xCAFEBABE,
        dst,
        scope: crate::security::cap::scope::ALL,
        expiry_ms: future_time,
    }
}

/// Test helper to create a wrong destination token
pub fn create_wrong_dst_token(actual_dst: u64, target_dst: u64) -> CapToken {
    let future_time = crate::security::get_current_time_ms() + 10000;
    CapToken {
        id: 0xBAADF00D,
        dst: actual_dst,
        scope: crate::security::cap::scope::ALL,
        expiry_ms: future_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test that expired tokens are rejected
    #[test]
    fn test_expired_token_rejected() {
        // Create an expired token
        let expired_token = create_expired_token(42);
        let current_time = crate::security::get_current_time_ms();
        
        // Validation should fail due to expiry
        let result = validate_cap_expiry(&expired_token, current_time);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationFailure::Expired);
        
        // Full validation should also fail
        let full_result = validate_all(&expired_token, 42, crate::security::cap::scope::SEND, current_time);
        assert!(full_result.is_err());
        assert_eq!(full_result.unwrap_err(), ValidationFailure::Expired);
    }
    
    /// Test that wrong destination tokens are rejected
    #[test]
    fn test_wrong_destination_rejected() {
        // Create a token for destination 42, but try to use for destination 24
        let wrong_dst_token = create_wrong_dst_token(42, 24);
        
        // Destination validation should fail
        let result = validate_cap_dst(&wrong_dst_token, 24);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationFailure::WrongDestination);
        
        // Full validation should also fail
        let current_time = crate::security::get_current_time_ms();
        let full_result = validate_all(&wrong_dst_token, 24, crate::security::cap::scope::SEND, current_time);
        assert!(full_result.is_err());
        assert_eq!(full_result.unwrap_err(), ValidationFailure::WrongDestination);
    }
    
    /// Test that valid tokens are accepted
    #[test]
    fn test_valid_token_accepted() {
        let valid_token = create_valid_token(42);
        let current_time = crate::security::get_current_time_ms();
        
        // All individual validations should pass
        assert!(validate_cap_expiry(&valid_token, current_time).is_ok());
        assert!(validate_cap_dst(&valid_token, 42).is_ok());
        assert!(validate_cap_scope(&valid_token, crate::security::cap::scope::SEND).is_ok());
        
        // Full validation should pass
        let full_result = validate_all(&valid_token, 42, crate::security::cap::scope::SEND, current_time);
        assert!(full_result.is_ok());
    }
    
    /// Test scope validation
    #[test]
    fn test_scope_validation() {
        let mut token = create_valid_token(42);
        token.scope = crate::security::cap::scope::RECV; // Only receive permission
        
        // Should fail for send operation
        let result = validate_cap_scope(&token, crate::security::cap::scope::SEND);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationFailure::InsufficientPrivileges);
        
        // Should pass for receive operation
        let result = validate_cap_scope(&token, crate::security::cap::scope::RECV);
        assert!(result.is_ok());
    }
    
    /// Test batch validation
    #[test]
    fn test_batch_validation() {
        let expired_token = create_expired_token(42);
        let wrong_dst_token = create_wrong_dst_token(43, 42);
        let valid_token = create_valid_token(42);
        
        let tokens = [expired_token, wrong_dst_token, valid_token];
        
        // Should find the valid token
        let result = validate_batch(&tokens, 42, crate::security::cap::scope::SEND);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 0xCAFEBABE); // Valid token ID
    }
}

/// Initialize the validator subsystem
pub fn init_validator() {
    kprintln!("[VALIDATOR] Initializing capability validator");
    
    // Clear statistics
    clear_validation_stats();
    
    klog!(INFO, "[VALIDATOR] Capability validator initialized");
}

/// Test the validator functionality
pub fn test_validator() {
    kprintln!("");
    kprintln!("=== CAPABILITY VALIDATOR TEST ===");
    
    let current_time = crate::security::get_current_time_ms();
    
    // Test expired token rejection
    kprintln!("Testing expired token rejection...");
    let expired_token = create_expired_token(42);
    match validate_cap_expiry(&expired_token, current_time) {
        Ok(()) => kprintln!("  ✗ Expired token should have been rejected"),
        Err(ValidationFailure::Expired) => kprintln!("  ✓ Expired token correctly rejected"),
        Err(e) => kprintln!("  ✗ Unexpected error: {}", e),
    }
    
    // Test wrong destination rejection
    kprintln!("Testing wrong destination rejection...");
    let wrong_dst_token = create_wrong_dst_token(42, 24);
    match validate_cap_dst(&wrong_dst_token, 24) {
        Ok(()) => kprintln!("  ✗ Wrong destination should have been rejected"),
        Err(ValidationFailure::WrongDestination) => kprintln!("  ✓ Wrong destination correctly rejected"),
        Err(e) => kprintln!("  ✗ Unexpected error: {}", e),
    }
    
    // Test valid token acceptance
    kprintln!("Testing valid token acceptance...");
    let valid_token = create_valid_token(42);
    match validate_all(&valid_token, 42, crate::security::cap::scope::SEND, current_time) {
        Ok(()) => kprintln!("  ✓ Valid token correctly accepted"),
        Err(e) => kprintln!("  ✗ Valid token should have been accepted: {}", e),
    }
    
    // Test IPC send validation
    kprintln!("Testing IPC send validation...");
    match validate_for_ipc_send(&valid_token, 42) {
        Ok(()) => kprintln!("  ✓ IPC send validation passed"),
        Err(e) => kprintln!("  ✗ IPC send validation failed: {}", e),
    }
    
    // Print statistics
    print_validation_stats();
    
    kprintln!("=== CAPABILITY VALIDATOR TEST COMPLETE ===");
    kprintln!("");
}

/// This module provides comprehensive validation functions for capability tokens,
/// ensuring that only authorized processes can perform restricted operations.

use super::cap::{CapToken, SecurityError, SecurityResult};
use crate::{kprintln, klog};
use core::sync::atomic::{AtomicU64, Ordering};

/// Validation statistics
#[derive(Debug, Clone, Copy)]
pub struct ValidationStats {
    /// Total validations performed
    pub total_validations: u64,
    
    /// Successful validations
    pub successful_validations: u64,
    
    /// Failed validations due to expiry
    pub expiry_failures: u64,
    
    /// Failed validations due to wrong destination
    pub destination_failures: u64,
    
    /// Failed validations due to other reasons
    pub other_failures: u64,
}

impl ValidationStats {
    pub const fn new() -> Self {
        Self {
            total_validations: 0,
            successful_validations: 0,
            expiry_failures: 0,
            destination_failures: 0,
            other_failures: 0,
        }
    }
    
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f32 {
        if self.total_validations > 0 {
            (self.successful_validations as f32 / self.total_validations as f32) * 100.0
        } else {
            0.0
        }
    }
}

/// Global validation statistics
static VALIDATION_STATS: AtomicU64 = AtomicU64::new(0); // Packed stats for atomic operations

/// Pack validation statistics into a single u64
fn pack_validation_stats(total: u16, success: u16, expiry: u16, dest: u16) -> u64 {
    ((total as u64) << 48) | ((success as u64) << 32) | ((expiry as u64) << 16) | (dest as u64)
}

/// Unpack validation statistics from a single u64
fn unpack_validation_stats(packed: u64) -> (u16, u16, u16, u16) {
    let total = (packed >> 48) as u16;
    let success = (packed >> 32) as u16;
    let expiry = (packed >> 16) as u16;
    let dest = packed as u16;
    (total, success, expiry, dest)
}

/// Update validation statistics
fn update_validation_stats(success: bool, expiry_fail: bool, dest_fail: bool) {
    let current = VALIDATION_STATS.load(Ordering::Relaxed);
    let (mut total, mut success_count, mut expiry, mut dest) = unpack_validation_stats(current);
    
    total = total.saturating_add(1);
    
    if success {
        success_count = success_count.saturating_add(1);
    } else if expiry_fail {
        expiry = expiry.saturating_add(1);
    } else if dest_fail {
        dest = dest.saturating_add(1);
    }
    
    let new_packed = pack_validation_stats(total, success_count, expiry, dest);
    VALIDATION_STATS.store(new_packed, Ordering::Relaxed);
}

/// Validation failure reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationFailure {
    /// Token has expired
    Expired,
    
    /// Wrong destination
    WrongDestination,
    
    /// Invalid token format
    InvalidToken,
    
    /// Insufficient privileges
    InsufficientPrivileges,
    
    /// Token revoked
    Revoked,
    
    /// Unknown error
    Unknown,
}

impl ValidationFailure {
    /// Convert to SecurityError
    pub fn to_security_error(self) -> SecurityError {
        match self {
            ValidationFailure::Expired => SecurityError::CapabilityExpired,
            ValidationFailure::WrongDestination => SecurityError::PermissionDenied,
            ValidationFailure::InvalidToken => SecurityError::InvalidCapability,
            ValidationFailure::InsufficientPrivileges => SecurityError::PermissionDenied,
            ValidationFailure::Revoked => SecurityError::CapabilityNotFound,
            ValidationFailure::Unknown => SecurityError::PolicyViolation,
        }
    }
    
    /// Get audit operation code for this failure
    pub fn audit_op_code(self) -> u16 {
        match self {
            ValidationFailure::Expired => crate::secman::audit::ops::SEC_AUTH_FAIL + 10, // 112
            ValidationFailure::WrongDestination => crate::secman::audit::ops::SEC_AUTH_FAIL + 11, // 113
            ValidationFailure::InvalidToken => crate::secman::audit::ops::SEC_AUTH_FAIL + 12, // 114
            ValidationFailure::InsufficientPrivileges => crate::secman::audit::ops::SEC_AUTH_FAIL + 13, // 115
            ValidationFailure::Revoked => crate::secman::audit::ops::SEC_AUTH_FAIL + 14, // 116
            ValidationFailure::Unknown => crate::secman::audit::ops::SEC_AUTH_FAIL + 15, // 117
        }
    }
}

impl core::fmt::Display for ValidationFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValidationFailure::Expired => write!(f, "Token expired"),
            ValidationFailure::WrongDestination => write!(f, "Wrong destination"),
            ValidationFailure::InvalidToken => write!(f, "Invalid token"),
            ValidationFailure::InsufficientPrivileges => write!(f, "Insufficient privileges"),
            ValidationFailure::Revoked => write!(f, "Token revoked"),
            ValidationFailure::Unknown => write!(f, "Unknown error"),
        }
    }
}

/// Validation result
pub type ValidationResult<T> = Result<T, ValidationFailure>;

/// Validate capability token expiry
/// 
/// Checks if the capability token has expired based on the current time.
/// 
/// # Arguments
/// * `token` - The capability token to validate
/// * `current_time_ms` - Current time in milliseconds
/// 
/// # Returns
/// `Ok(())` if token is not expired, `Err(ValidationFailure::Expired)` if expired
pub fn validate_cap_expiry(token: &CapToken, current_time_ms: u64) -> ValidationResult<()> {
    klog!(TRACE, [crate::log::tags::VALIDATOR], "Checking expiry: token expires at {}, current time {}", 
          token.expiry_ms, current_time_ms);
    
    if current_time_ms > token.expiry_ms {
        klog!(TRACE, [crate::log::tags::VALIDATOR], "Token expired ({} > {})", current_time_ms, token.expiry_ms);
        
        // Log audit entry for expired token
        crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
            0, // System operation
            ValidationFailure::Expired.audit_op_code(),
            token.id as u64
        ));
        
        update_validation_stats(false, true, false);
        Err(ValidationFailure::Expired)
    } else {
        klog!(TRACE, "[VALIDATOR] Token expiry valid");
        Ok(())
    }
}

/// Validate capability token destination
/// 
/// Checks if the capability token grants access to the specified destination.
/// 
/// # Arguments
/// * `token` - The capability token to validate
/// * `target_dst` - The target destination to check access for
/// 
/// # Returns
/// `Ok(())` if destination matches, `Err(ValidationFailure::WrongDestination)` if not
pub fn validate_cap_dst(token: &CapToken, target_dst: u64) -> ValidationResult<()> {
    klog!(TRACE, "[VALIDATOR] Checking destination: token dst {}, target dst {}", 
          token.dst, target_dst);
    
    if token.dst != target_dst {
        klog!(TRACE, "[VALIDATOR] Destination mismatch ({} != {})", token.dst, target_dst);
        
        // Log audit entry for wrong destination
        crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
            0, // System operation
            ValidationFailure::WrongDestination.audit_op_code(),
            ((token.dst as u64) << 32) | target_dst
        ));
        
        update_validation_stats(false, false, true);
        Err(ValidationFailure::WrongDestination)
    } else {
        klog!(TRACE, "[VALIDATOR] Destination valid");
        Ok(())
    }
}

/// Validate capability token scope for specific operation
/// 
/// Checks if the capability token has the required scope permissions.
/// 
/// # Arguments
/// * `token` - The capability token to validate
/// * `required_scope` - The scope permissions required for the operation
/// 
/// # Returns
/// `Ok(())` if scope is sufficient, `Err(ValidationFailure::InsufficientPrivileges)` if not
pub fn validate_cap_scope(token: &CapToken, required_scope: u32) -> ValidationResult<()> {
    klog!(TRACE, "[VALIDATOR] Checking scope: token scope 0x{:x}, required 0x{:x}", 
          token.scope, required_scope);
    
    if (token.scope & required_scope) != required_scope {
        klog!(TRACE, "[VALIDATOR] Insufficient scope (0x{:x} & 0x{:x} != 0x{:x})", 
              token.scope, required_scope, required_scope);
        
        // Log audit entry for insufficient privileges
        crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
            0, // System operation
            ValidationFailure::InsufficientPrivileges.audit_op_code(),
            ((token.scope as u64) << 32) | (required_scope as u64)
        ));
        
        update_validation_stats(false, false, false);
        Err(ValidationFailure::InsufficientPrivileges)
    } else {
        klog!(TRACE, "[VALIDATOR] Scope valid");
        Ok(())
    }
}

/// Comprehensive capability validation
/// 
/// Performs all validation checks on a capability token for a specific operation.
/// 
/// # Arguments
/// * `token` - The capability token to validate
/// * `target_dst` - The target destination to check access for
/// * `required_scope` - The scope permissions required for the operation
/// * `current_time_ms` - Current time in milliseconds
/// 
/// # Returns
/// `Ok(())` if all validations pass, `Err(ValidationFailure)` on first failure
pub fn validate_all(
    token: &CapToken, 
    target_dst: u64, 
    required_scope: u32, 
    current_time_ms: u64
) -> ValidationResult<()> {
    klog!(TRACE, "[VALIDATOR] Starting comprehensive validation for token 0x{:x}", token.id);
    
    // Validate expiry first (most common failure)
    validate_cap_expiry(token, current_time_ms)?;
    
    // Validate destination
    validate_cap_dst(token, target_dst)?;
    
    // Validate scope
    validate_cap_scope(token, required_scope)?;
    
    // All validations passed
    update_validation_stats(true, false, false);
    
    // Log successful validation
    crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
        0, // System operation
        crate::secman::audit::ops::SEC_CAP_GRANT, // Successful validation
        token.id as u64
    ));
    
    klog!(TRACE, "[VALIDATOR] All validations passed for token 0x{:x}", token.id);
    Ok(())
}

/// Validate capability with custom validation time
/// 
/// Same as validate_all but allows specifying a custom validation time.
/// Useful for testing and scenarios where current time might not be appropriate.
pub fn validate_all_at_time(
    token: &CapToken, 
    target_dst: u64, 
    required_scope: u32, 
    validation_time_ms: u64
) -> ValidationResult<()> {
    validate_all(token, target_dst, required_scope, validation_time_ms)
}

/// Quick validation for IPC send operations
/// 
/// Optimized validation specifically for IPC send operations.
/// Only checks expiry and destination, assumes SEND scope.
pub fn validate_for_ipc_send(token: &CapToken, target_dst: u64) -> ValidationResult<()> {
    let current_time = crate::security::get_current_time_ms();
    validate_all(token, target_dst, crate::security::cap::scope::SEND, current_time)
}

/// Batch validation for multiple tokens
/// 
/// Validates multiple capability tokens at once, returning the first valid one.
/// Useful when a process has multiple tokens that could grant access.
pub fn validate_batch(
    tokens: &[CapToken], 
    target_dst: u64, 
    required_scope: u32
) -> ValidationResult<CapToken> {
    let current_time = crate::security::get_current_time_ms();
    
    for token in tokens {
        if validate_all(token, target_dst, required_scope, current_time).is_ok() {
            return Ok(*token);
        }
    }
    
    // No valid token found
    update_validation_stats(false, false, false);
    Err(ValidationFailure::Unknown)
}

/// Get current validation statistics
pub fn get_validation_stats() -> ValidationStats {
    let packed = VALIDATION_STATS.load(Ordering::Relaxed);
    let (total, success, expiry, dest) = unpack_validation_stats(packed);
    
    ValidationStats {
        total_validations: total as u64,
        successful_validations: success as u64,
        expiry_failures: expiry as u64,
        destination_failures: dest as u64,
        other_failures: (total as u64).saturating_sub(success as u64 + expiry as u64 + dest as u64),
    }
}

/// Clear validation statistics
pub fn clear_validation_stats() {
    VALIDATION_STATS.store(0, Ordering::Relaxed);
    klog!(INFO, "[VALIDATOR] Validation statistics cleared");
}

/// Print validation statistics
pub fn print_validation_stats() {
    let stats = get_validation_stats();
    
    kprintln!("");
    kprintln!("=== CAPABILITY VALIDATION STATISTICS ===");
    kprintln!("Total validations: {}", stats.total_validations);
    kprintln!("Successful: {}", stats.successful_validations);
    kprintln!("Failed - Expired: {}", stats.expiry_failures);
    kprintln!("Failed - Wrong destination: {}", stats.destination_failures);
    kprintln!("Failed - Other: {}", stats.other_failures);
    
    if stats.total_validations > 0 {
        kprintln!("Success rate: {:.1}%", stats.success_rate());
    }
    
    kprintln!("=== END VALIDATION STATISTICS ===");
    kprintln!("");
}

/// Test helper to create an expired token
pub fn create_expired_token(dst: u64) -> CapToken {
    let past_time = crate::security::get_current_time_ms().saturating_sub(1000);
    CapToken {
        id: 0xDEADBEEF,
        dst,
        scope: crate::security::cap::scope::SEND,
        expiry_ms: past_time,
    }
}

/// Test helper to create a valid token
pub fn create_valid_token(dst: u64) -> CapToken {
    let future_time = crate::security::get_current_time_ms() + 10000;
    CapToken {
        id: 0xCAFEBABE,
        dst,
        scope: crate::security::cap::scope::ALL,
        expiry_ms: future_time,
    }
}

/// Test helper to create a wrong destination token
pub fn create_wrong_dst_token(actual_dst: u64, target_dst: u64) -> CapToken {
    let future_time = crate::security::get_current_time_ms() + 10000;
    CapToken {
        id: 0xBAADF00D,
        dst: actual_dst,
        scope: crate::security::cap::scope::ALL,
        expiry_ms: future_time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test that expired tokens are rejected
    #[test]
    fn test_expired_token_rejected() {
        // Create an expired token
        let expired_token = create_expired_token(42);
        let current_time = crate::security::get_current_time_ms();
        
        // Validation should fail due to expiry
        let result = validate_cap_expiry(&expired_token, current_time);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationFailure::Expired);
        
        // Full validation should also fail
        let full_result = validate_all(&expired_token, 42, crate::security::cap::scope::SEND, current_time);
        assert!(full_result.is_err());
        assert_eq!(full_result.unwrap_err(), ValidationFailure::Expired);
    }
    
    /// Test that wrong destination tokens are rejected
    #[test]
    fn test_wrong_destination_rejected() {
        // Create a token for destination 42, but try to use for destination 24
        let wrong_dst_token = create_wrong_dst_token(42, 24);
        
        // Destination validation should fail
        let result = validate_cap_dst(&wrong_dst_token, 24);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationFailure::WrongDestination);
        
        // Full validation should also fail
        let current_time = crate::security::get_current_time_ms();
        let full_result = validate_all(&wrong_dst_token, 24, crate::security::cap::scope::SEND, current_time);
        assert!(full_result.is_err());
        assert_eq!(full_result.unwrap_err(), ValidationFailure::WrongDestination);
    }
    
    /// Test that valid tokens are accepted
    #[test]
    fn test_valid_token_accepted() {
        let valid_token = create_valid_token(42);
        let current_time = crate::security::get_current_time_ms();
        
        // All individual validations should pass
        assert!(validate_cap_expiry(&valid_token, current_time).is_ok());
        assert!(validate_cap_dst(&valid_token, 42).is_ok());
        assert!(validate_cap_scope(&valid_token, crate::security::cap::scope::SEND).is_ok());
        
        // Full validation should pass
        let full_result = validate_all(&valid_token, 42, crate::security::cap::scope::SEND, current_time);
        assert!(full_result.is_ok());
    }
    
    /// Test scope validation
    #[test]
    fn test_scope_validation() {
        let mut token = create_valid_token(42);
        token.scope = crate::security::cap::scope::RECV; // Only receive permission
        
        // Should fail for send operation
        let result = validate_cap_scope(&token, crate::security::cap::scope::SEND);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationFailure::InsufficientPrivileges);
        
        // Should pass for receive operation
        let result = validate_cap_scope(&token, crate::security::cap::scope::RECV);
        assert!(result.is_ok());
    }
    
    /// Test batch validation
    #[test]
    fn test_batch_validation() {
        let expired_token = create_expired_token(42);
        let wrong_dst_token = create_wrong_dst_token(43, 42);
        let valid_token = create_valid_token(42);
        
        let tokens = [expired_token, wrong_dst_token, valid_token];
        
        // Should find the valid token
        let result = validate_batch(&tokens, 42, crate::security::cap::scope::SEND);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 0xCAFEBABE); // Valid token ID
    }
}

/// Initialize the validator subsystem
pub fn init_validator() {
    kprintln!("[VALIDATOR] Initializing capability validator");
    
    // Clear statistics
    clear_validation_stats();
    
    klog!(INFO, "[VALIDATOR] Capability validator initialized");
}

/// Test the validator functionality
pub fn test_validator() {
    kprintln!("");
    kprintln!("=== CAPABILITY VALIDATOR TEST ===");
    
    let current_time = crate::security::get_current_time_ms();
    
    // Test expired token rejection
    kprintln!("Testing expired token rejection...");
    let expired_token = create_expired_token(42);
    match validate_cap_expiry(&expired_token, current_time) {
        Ok(()) => kprintln!("  ✗ Expired token should have been rejected"),
        Err(ValidationFailure::Expired) => kprintln!("  ✓ Expired token correctly rejected"),
        Err(e) => kprintln!("  ✗ Unexpected error: {}", e),
    }
    
    // Test wrong destination rejection
    kprintln!("Testing wrong destination rejection...");
    let wrong_dst_token = create_wrong_dst_token(42, 24);
    match validate_cap_dst(&wrong_dst_token, 24) {
        Ok(()) => kprintln!("  ✗ Wrong destination should have been rejected"),
        Err(ValidationFailure::WrongDestination) => kprintln!("  ✓ Wrong destination correctly rejected"),
        Err(e) => kprintln!("  ✗ Unexpected error: {}", e),
    }
    
    // Test valid token acceptance
    kprintln!("Testing valid token acceptance...");
    let valid_token = create_valid_token(42);
    match validate_all(&valid_token, 42, crate::security::cap::scope::SEND, current_time) {
        Ok(()) => kprintln!("  ✓ Valid token correctly accepted"),
        Err(e) => kprintln!("  ✗ Valid token should have been accepted: {}", e),
    }
    
    // Test IPC send validation
    kprintln!("Testing IPC send validation...");
    match validate_for_ipc_send(&valid_token, 42) {
        Ok(()) => kprintln!("  ✓ IPC send validation passed"),
        Err(e) => kprintln!("  ✗ IPC send validation failed: {}", e),
    }
    
    // Print statistics
    print_validation_stats();
    
    kprintln!("=== CAPABILITY VALIDATOR TEST COMPLETE ===");
    kprintln!("");
}




