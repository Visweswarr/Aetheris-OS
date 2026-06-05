//! Kernel Boot Module
//! 
//! Handles early kernel boot process including attestation and verification.

pub mod attest;
pub mod serial;
pub mod uefi_main;

use crate::{kprintln, klog};
use crate::hal::Hal;

#[cfg(target_arch = "x86_64")]
use crate::hal::x86_64::X64Hal;

#[cfg(target_arch = "aarch64")]
use crate::hal::aarch64::AArch64Hal;

/// Initialize the boot system
pub fn init_boot() {
    // Initialize attestation verifier with development bypass
    // In production, this would be false
    let dev_bypass = true; // TODO: Make this configurable
    
    attest::init_attestation_verifier(dev_bypass);
    
    kprintln!("[BOOT] Boot system initialized");
    kprintln!("[BOOT] Development bypass: {}", dev_bypass);
}

/// Get boot information
pub fn get_boot_info() -> &'static str {
    "Polymera OS - Development Build"
}

/// Initialize boot components and full kernel subsystems
pub fn init() {
    kprintln!("[PolymeraCore] boot::init()");

    #[cfg(target_arch = "x86_64")]
    {
        let _ = X64Hal::init_cpu();
        let _ = X64Hal::init_timer();
        let _ = X64Hal::enable_interrupts();
    }

    #[cfg(target_arch = "aarch64")]
    {
        let _ = AArch64Hal::init_cpu();
        let _ = AArch64Hal::init_timer();
        let _ = AArch64Hal::enable_interrupts();
    }

    // Initialize Memory Management early in boot sequence
    kprintln!("[PolymeraCore] Initializing Memory Management");
    crate::mm::init_mm();

    // Initialize the scheduler
    kprintln!("[PolymeraCore] Initializing scheduler");
    crate::sched::init_sched();
    
    // Initialize syscalls
    kprintln!("[PolymeraCore] Initializing system calls");
    crate::syscall::init_syscalls();
    
    // Initialize ABI feature manager
    kprintln!("[PolymeraCore] Initializing ABI feature manager");
    crate::abi::init_feature_manager();
    
    // Initialize security subsystem
    kprintln!("[PolymeraCore] Initializing security subsystem");
    crate::security::init_security();
    
    // Initialize security manager
    kprintln!("[PolymeraCore] Initializing security manager");
    crate::secman::init_secman();
    
    // Initialize IPC subsystem
    kprintln!("[PolymeraCore] Initializing IPC subsystem");
    crate::ipc::init_ipc();
    
    // Initialize determinism system
    kprintln!("[PolymeraCore] Initializing determinism system");
    crate::determinism::init();
    
    // Initialize randomness proxy
    kprintln!("[PolymeraCore] Initializing randomness proxy");
    crate::rng::init();
    
    // Initialize formatting system
    kprintln!("[PolymeraCore] Initializing formatting system");
    let _ = crate::format::test_format_stability();
    
    // Initialize dashboard system
    kprintln!("[PolymeraCore] Initializing dashboard system");
    crate::dashboard::init();
    
    // Initialize fault injection system
    kprintln!("[PolymeraCore] Initializing fault injection system");
    crate::fault_injection::init();
    
    // Initialize Phase 1.5 components
    kprintln!("[PolymeraCore] Initializing Phase 1.5 - Enhanced Crash Dump System");
    crate::crash_dump::init();
    
    kprintln!("[PolymeraCore] Initializing Phase 1.5 - Enhanced Fuzzing System");
    crate::fuzzing::init();
    
    kprintln!("[PolymeraCore] Initializing Phase 1.5 - Scheduler Fairness Analysis");
    crate::sched::fairness::init();
    
    kprintln!("[PolymeraCore] Initializing Phase 1.5 - Memory Safety System");
    crate::mm::safety::init();
    
    kprintln!("[PolymeraCore] Initializing Phase 1.5 - Shell System");
    crate::shell::init();
    
    kprintln!("[PolymeraCore] Initializing Phase 1.5 - Flaky Detector");
    crate::flaky_detector::init();
    
    kprintln!("[PolymeraCore] Initializing tracing subsystem");
    crate::trace::init_trace();
    
    // Initialize boot attestation
    init_boot();
    
    klog!(INFO, "[PHASE1.5 PASS] boot init sequence completed");
}