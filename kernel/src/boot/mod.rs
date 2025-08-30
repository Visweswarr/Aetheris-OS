//! Kernel Boot Module
//! 
//! Handles early kernel boot process including attestation and verification.

pub mod attest;

/// Initialize the boot system
pub fn init_boot() {
    // Initialize attestation verifier with development bypass
    // In production, this would be false
    let dev_bypass = true; // TODO: Make this configurable
    
    attest::init_attestation_verifier(dev_bypass);
    
    crate::println!("[BOOT] Boot system initialized");
    crate::println!("[BOOT] Development bypass: {}", dev_bypass);
}

/// Get boot information
pub fn get_boot_info() -> &'static str {
    "Polymera OS - Development Build"
}