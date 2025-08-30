/// Security Module for Polymera OS
/// 
/// This module provides security mechanisms including capability tokens,
/// access control, and permission validation for inter-process communication.

pub mod cap;
pub mod validator;

use crate::{kprintln, klog};
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

// Re-export key types and functions
pub use cap::*;

/// Security subsystem statistics
#[derive(Debug, Clone, Copy)]
pub struct SecurityStats {
    /// Total capability validations performed
    pub validations_performed: u64,
    
    /// Number of successful validations
    pub validations_passed: u64,
    
    /// Number of failed validations
    pub validations_failed: u64,
    
    /// Number of expired capabilities
    pub capabilities_expired: u64,
    
    /// Number of permission denied errors
    pub permission_denied: u64,
    
    /// Number of active capability tokens
    pub active_tokens: u64,
}

impl SecurityStats {
    pub const fn new() -> Self {
        Self {
            validations_performed: 0,
            validations_passed: 0,
            validations_failed: 0,
            capabilities_expired: 0,
            permission_denied: 0,
            active_tokens: 0,
        }
    }
}

/// Global security statistics
static SECURITY_STATS: Mutex<SecurityStats> = Mutex::new(SecurityStats::new());

/// Global timestamp counter for capability expiry
static CURRENT_TIME_MS: AtomicU64 = AtomicU64::new(0);

/// Update security statistics
pub fn update_security_stats<F>(updater: F) 
where
    F: FnOnce(&mut SecurityStats),
{
    let mut stats = SECURITY_STATS.lock();
    updater(&mut stats);
}

/// Get current security statistics
pub fn get_security_stats() -> SecurityStats {
    *SECURITY_STATS.lock()
}

/// Get current time in milliseconds
pub fn get_current_time_ms() -> u64 {
    CURRENT_TIME_MS.load(Ordering::Relaxed)
}

/// Advance current time (for testing and simulation)
pub fn advance_time_ms(delta_ms: u64) {
    CURRENT_TIME_MS.fetch_add(delta_ms, Ordering::Relaxed);
}

/// Set current time (for initialization)
pub fn set_current_time_ms(time_ms: u64) {
    CURRENT_TIME_MS.store(time_ms, Ordering::Relaxed);
}

/// Print security statistics
pub fn print_security_stats() {
    let stats = get_security_stats();
    
    kprintln!("");
    kprintln!("=== SECURITY STATISTICS ===");
    kprintln!("Validations performed: {}", stats.validations_performed);
    kprintln!("Validations passed: {}", stats.validations_passed);
    kprintln!("Validations failed: {}", stats.validations_failed);
    kprintln!("Capabilities expired: {}", stats.capabilities_expired);
    kprintln!("Permission denied: {}", stats.permission_denied);
    kprintln!("Active tokens: {}", stats.active_tokens);
    
    if stats.validations_performed > 0 {
        let success_rate = (stats.validations_passed * 100) / stats.validations_performed;
        kprintln!("Success rate: {}%", success_rate);
    }
    
    kprintln!("=== END SECURITY STATISTICS ===");
    kprintln!("");
}

/// Initialize security subsystem
pub fn init_security() {
    kprintln!("[SECURITY] Initializing security subsystem");
    
    // Initialize capability token storage
    cap::init_capability_storage();
    
    // Initialize capability validator
    validator::init_validator();
    
    // Set initial time
    set_current_time_ms(1000); // Start at 1 second
    
    // Setup demo capabilities
    cap::setup_demo_capabilities();
    
    klog!(INFO, "[SECURITY] Security subsystem initialized successfully");
}

/// Test security functionality
pub fn test_security() {
    kprintln!("");
    kprintln!("=== SECURITY FUNCTIONALITY TEST ===");
    
    let current_time = get_current_time_ms();
    kprintln!("Current time: {} ms", current_time);
    
    // Test capability token creation
    let token = CapToken {
        id: 0x123456789ABCDEFu128,
        dst: 42,
        scope: 0x1,
        expiry_ms: current_time + 5000, // Expires in 5 seconds
    };
    
    kprintln!("  ✓ Created capability token: ID=0x{:x}, dst={}, scope=0x{:x}, expires_at={}",
              token.id, token.dst, token.scope, token.expiry_ms);
    
    // Test validation (should pass)
    if validate_cap(&token, 42, current_time) {
        kprintln!("  ✓ Token validation passed for correct destination");
    } else {
        kprintln!("  ✗ Token validation failed unexpectedly");
    }
    
    // Test validation with wrong destination (should fail)
    if !validate_cap(&token, 99, current_time) {
        kprintln!("  ✓ Token validation correctly failed for wrong destination");
    } else {
        kprintln!("  ✗ Token validation should have failed for wrong destination");
    }
    
    // Test validation with expired token (should fail)
    advance_time_ms(6000); // Advance past expiry
    if !validate_cap(&token, 42, get_current_time_ms()) {
        kprintln!("  ✓ Token validation correctly failed for expired token");
    } else {
        kprintln!("  ✗ Token validation should have failed for expired token");
    }
    
    // Test capability storage
    cap::test_capability_storage();
    
    // Test capability validator
    validator::test_validator();
    
    // Print statistics
    print_security_stats();
    validator::print_validation_stats();
    
    kprintln!("=== SECURITY FUNCTIONALITY TEST COMPLETE ===");
    kprintln!("");
}

/// Security error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    /// Permission denied
    PermissionDenied,
    
    /// Invalid capability token
    InvalidCapability,
    
    /// Capability token expired
    CapabilityExpired,
    
    /// Capability token not found
    CapabilityNotFound,
    
    /// Invalid process ID
    InvalidProcessId,
    
    /// Access denied
    AccessDenied,
    
    /// Security policy violation
    PolicyViolation,
}

impl core::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SecurityError::PermissionDenied => write!(f, "Permission denied"),
            SecurityError::InvalidCapability => write!(f, "Invalid capability token"),
            SecurityError::CapabilityExpired => write!(f, "Capability token expired"),
            SecurityError::CapabilityNotFound => write!(f, "Capability token not found"),
            SecurityError::InvalidProcessId => write!(f, "Invalid process ID"),
            SecurityError::AccessDenied => write!(f, "Access denied"),
            SecurityError::PolicyViolation => write!(f, "Security policy violation"),
        }
    }
}

/// Security result type
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Convert SecurityError to POSIX-style error codes
impl SecurityError {
    pub fn to_errno(self) -> i32 {
        match self {
            SecurityError::PermissionDenied => 1,  // EPERM
            SecurityError::InvalidCapability => 22, // EINVAL
            SecurityError::CapabilityExpired => 110, // ETIMEDOUT
            SecurityError::CapabilityNotFound => 2, // ENOENT
            SecurityError::InvalidProcessId => 3,  // ESRCH
            SecurityError::AccessDenied => 13,     // EACCES
            SecurityError::PolicyViolation => 1,   // EPERM
        }
    }
}

/// This module provides security mechanisms including capability tokens,
/// access control, and permission validation for inter-process communication.

pub mod cap;
pub mod validator;

use crate::{kprintln, klog};
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

// Re-export key types and functions
pub use cap::*;

/// Security subsystem statistics
#[derive(Debug, Clone, Copy)]
pub struct SecurityStats {
    /// Total capability validations performed
    pub validations_performed: u64,
    
    /// Number of successful validations
    pub validations_passed: u64,
    
    /// Number of failed validations
    pub validations_failed: u64,
    
    /// Number of expired capabilities
    pub capabilities_expired: u64,
    
    /// Number of permission denied errors
    pub permission_denied: u64,
    
    /// Number of active capability tokens
    pub active_tokens: u64,
}

impl SecurityStats {
    pub const fn new() -> Self {
        Self {
            validations_performed: 0,
            validations_passed: 0,
            validations_failed: 0,
            capabilities_expired: 0,
            permission_denied: 0,
            active_tokens: 0,
        }
    }
}

/// Global security statistics
static SECURITY_STATS: Mutex<SecurityStats> = Mutex::new(SecurityStats::new());

/// Global timestamp counter for capability expiry
static CURRENT_TIME_MS: AtomicU64 = AtomicU64::new(0);

/// Update security statistics
pub fn update_security_stats<F>(updater: F) 
where
    F: FnOnce(&mut SecurityStats),
{
    let mut stats = SECURITY_STATS.lock();
    updater(&mut stats);
}

/// Get current security statistics
pub fn get_security_stats() -> SecurityStats {
    *SECURITY_STATS.lock()
}

/// Get current time in milliseconds
pub fn get_current_time_ms() -> u64 {
    CURRENT_TIME_MS.load(Ordering::Relaxed)
}

/// Advance current time (for testing and simulation)
pub fn advance_time_ms(delta_ms: u64) {
    CURRENT_TIME_MS.fetch_add(delta_ms, Ordering::Relaxed);
}

/// Set current time (for initialization)
pub fn set_current_time_ms(time_ms: u64) {
    CURRENT_TIME_MS.store(time_ms, Ordering::Relaxed);
}

/// Print security statistics
pub fn print_security_stats() {
    let stats = get_security_stats();
    
    kprintln!("");
    kprintln!("=== SECURITY STATISTICS ===");
    kprintln!("Validations performed: {}", stats.validations_performed);
    kprintln!("Validations passed: {}", stats.validations_passed);
    kprintln!("Validations failed: {}", stats.validations_failed);
    kprintln!("Capabilities expired: {}", stats.capabilities_expired);
    kprintln!("Permission denied: {}", stats.permission_denied);
    kprintln!("Active tokens: {}", stats.active_tokens);
    
    if stats.validations_performed > 0 {
        let success_rate = (stats.validations_passed * 100) / stats.validations_performed;
        kprintln!("Success rate: {}%", success_rate);
    }
    
    kprintln!("=== END SECURITY STATISTICS ===");
    kprintln!("");
}

/// Initialize security subsystem
pub fn init_security() {
    kprintln!("[SECURITY] Initializing security subsystem");
    
    // Initialize capability token storage
    cap::init_capability_storage();
    
    // Initialize capability validator
    validator::init_validator();
    
    // Set initial time
    set_current_time_ms(1000); // Start at 1 second
    
    // Setup demo capabilities
    cap::setup_demo_capabilities();
    
    klog!(INFO, "[SECURITY] Security subsystem initialized successfully");
}

/// Test security functionality
pub fn test_security() {
    kprintln!("");
    kprintln!("=== SECURITY FUNCTIONALITY TEST ===");
    
    let current_time = get_current_time_ms();
    kprintln!("Current time: {} ms", current_time);
    
    // Test capability token creation
    let token = CapToken {
        id: 0x123456789ABCDEFu128,
        dst: 42,
        scope: 0x1,
        expiry_ms: current_time + 5000, // Expires in 5 seconds
    };
    
    kprintln!("  ✓ Created capability token: ID=0x{:x}, dst={}, scope=0x{:x}, expires_at={}",
              token.id, token.dst, token.scope, token.expiry_ms);
    
    // Test validation (should pass)
    if validate_cap(&token, 42, current_time) {
        kprintln!("  ✓ Token validation passed for correct destination");
    } else {
        kprintln!("  ✗ Token validation failed unexpectedly");
    }
    
    // Test validation with wrong destination (should fail)
    if !validate_cap(&token, 99, current_time) {
        kprintln!("  ✓ Token validation correctly failed for wrong destination");
    } else {
        kprintln!("  ✗ Token validation should have failed for wrong destination");
    }
    
    // Test validation with expired token (should fail)
    advance_time_ms(6000); // Advance past expiry
    if !validate_cap(&token, 42, get_current_time_ms()) {
        kprintln!("  ✓ Token validation correctly failed for expired token");
    } else {
        kprintln!("  ✗ Token validation should have failed for expired token");
    }
    
    // Test capability storage
    cap::test_capability_storage();
    
    // Test capability validator
    validator::test_validator();
    
    // Print statistics
    print_security_stats();
    validator::print_validation_stats();
    
    kprintln!("=== SECURITY FUNCTIONALITY TEST COMPLETE ===");
    kprintln!("");
}

/// Security error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    /// Permission denied
    PermissionDenied,
    
    /// Invalid capability token
    InvalidCapability,
    
    /// Capability token expired
    CapabilityExpired,
    
    /// Capability token not found
    CapabilityNotFound,
    
    /// Invalid process ID
    InvalidProcessId,
    
    /// Access denied
    AccessDenied,
    
    /// Security policy violation
    PolicyViolation,
}

impl core::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SecurityError::PermissionDenied => write!(f, "Permission denied"),
            SecurityError::InvalidCapability => write!(f, "Invalid capability token"),
            SecurityError::CapabilityExpired => write!(f, "Capability token expired"),
            SecurityError::CapabilityNotFound => write!(f, "Capability token not found"),
            SecurityError::InvalidProcessId => write!(f, "Invalid process ID"),
            SecurityError::AccessDenied => write!(f, "Access denied"),
            SecurityError::PolicyViolation => write!(f, "Security policy violation"),
        }
    }
}

/// Security result type
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Convert SecurityError to POSIX-style error codes
impl SecurityError {
    pub fn to_errno(self) -> i32 {
        match self {
            SecurityError::PermissionDenied => 1,  // EPERM
            SecurityError::InvalidCapability => 22, // EINVAL
            SecurityError::CapabilityExpired => 110, // ETIMEDOUT
            SecurityError::CapabilityNotFound => 2, // ENOENT
            SecurityError::InvalidProcessId => 3,  // ESRCH
            SecurityError::AccessDenied => 13,     // EACCES
            SecurityError::PolicyViolation => 1,   // EPERM
        }
    }
}
