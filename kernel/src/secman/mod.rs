/// Security Manager (SecMan) Module for Polymera OS
/// 
/// This module provides security management functionality including audit logging,
/// identity mapping, message authentication, key management, and API operations for the kernel.

pub mod audit;
pub mod idmap;
pub mod msgauth;
pub mod keys;
pub mod api;
pub mod policy;
pub mod audit_codes;
#[cfg(feature = "dev-keyvault")]
pub mod dev_keyvault;

use crate::{kprintln, klog};

/// Security Manager initialization
pub fn init_secman() {
    kprintln!("[SECMAN] Initializing Security Manager");
    
    // Initialize audit subsystem
    audit::init_audit();
    
    // Initialize identity mapping
    idmap::init_idmap();
    
    // Initialize message authentication
    msgauth::init_msgauth();
    
    // Initialize key management system
    keys::init_keystore();
    
    // Initialize policy system
    match policy::init_policy_manager() {
        Ok(()) => klog!(INFO, "[SECMAN] Policy system initialized successfully"),
        Err(e) => klog!(WARN, "[SECMAN] Policy system initialization failed: {}", e),
    }
    
    // Initialize audit codes system
    audit_codes::init_audit_rate_limiter();
    klog!(INFO, "[SECMAN] Audit codes system initialized successfully");
    
    // Initialize development keyvault integration if enabled
    #[cfg(feature = "dev-keyvault")]
    {
        match dev_keyvault::init_dev_keyvault_integration() {
            Ok(()) => klog!(INFO, "[SECMAN] Development keyvault integration initialized successfully"),
            Err(e) => klog!(WARN, "[SECMAN] Development keyvault integration initialization failed: {}", e),
        }
    }
    
    klog!(INFO, "[SECMAN] Security Manager initialized successfully");
}

/// Security Manager test functionality
pub fn test_secman() {
    kprintln!("");
    kprintln!("=== SECURITY MANAGER TEST ===");
    
    // Test audit functionality
    audit::test_audit();
    
    // Test identity mapping
    idmap::test_idmap();
    
    // Test message authentication
    msgauth::test_msgauth();
    
    // Test key management system
    keys::test_keystore();
    
    // Test Security Manager API
    api::test_secman_api();
    
    // Test policy system
    policy::test_policy();
    
    // Test audit codes system
    audit_codes::test_audit_codes();
    
    // Test development keyvault integration if enabled
    #[cfg(feature = "dev-keyvault")]
    {
        dev_keyvault::test_dev_keyvault_integration();
    }
    
    kprintln!("=== SECURITY MANAGER TEST COMPLETE ===");
    kprintln!("");
}

/// Security Manager statistics
pub fn print_secman_stats() {
    kprintln!("");
    kprintln!("=== SECURITY MANAGER STATISTICS ===");
    
    audit::print_audit_stats();
    idmap::print_idmap_stats();
    msgauth::print_msgauth_stats();
    
    // Print key management statistics
    let key_stats = keys::get_keystore_stats();
    key_stats.print();
    
    // Print policy system statistics
    policy::print_policy_stats();
    
    // Print audit codes system statistics
    audit_codes::print_audit_codes_stats();
    
    // Print development keyvault integration statistics if enabled
    #[cfg(feature = "dev-keyvault")]
    {
        dev_keyvault::print_dev_keyvault_stats();
    }
    
    kprintln!("=== END SECURITY MANAGER STATISTICS ===");
    kprintln!("");
}

