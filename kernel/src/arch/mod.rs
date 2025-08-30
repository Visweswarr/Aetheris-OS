//! Architecture-specific code
//!
//! Provides abstractions for different CPU architectures.

use crate::{KernelConfig, error::KernelResult};

// Architecture-specific modules
#[cfg(target_arch = "x86_64")]
pub mod x86_64;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

// Re-export architecture-specific implementations
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;

/// Architecture-specific early initialization
pub fn early_init(config: &KernelConfig) -> KernelResult<()> {
    crate::log::kprintln!("Initializing architecture-specific components...");
    
    #[cfg(target_arch = "x86_64")]
    x86_64::early_init(config)?;
    
    #[cfg(target_arch = "aarch64")]
    aarch64::early_init(config)?;
    
    Ok(())
}

/// Halt the CPU
pub fn halt() -> ! {
    #[cfg(target_arch = "x86_64")]
    x86_64::halt();
    
    #[cfg(target_arch = "aarch64")]
    aarch64::halt();
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    loop {}
}

/// Shutdown the system
pub fn shutdown() -> ! {
    crate::log::kprintln!("System shutdown requested");
    
    #[cfg(target_arch = "x86_64")]
    x86_64::shutdown();
    
    #[cfg(target_arch = "aarch64")]
    aarch64::shutdown();
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    loop {}
}

/// Disable interrupts
pub unsafe fn disable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    x86_64::disable_interrupts();
    
    #[cfg(target_arch = "aarch64")]
    aarch64::disable_interrupts();
}

/// Enable interrupts
pub unsafe fn enable_interrupts() {
    #[cfg(target_arch = "x86_64")]
    x86_64::enable_interrupts();
    
    #[cfg(target_arch = "aarch64")]
    aarch64::enable_interrupts();
}

