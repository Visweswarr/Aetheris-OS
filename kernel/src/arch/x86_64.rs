//! x86_64 architecture-specific code

use crate::{KernelConfig, error::KernelResult};
use x86_64::instructions::{hlt, interrupts};

/// x86_64-specific early initialization
pub fn early_init(config: &KernelConfig) -> KernelResult<()> {
    crate::log::kprintln!("Initializing x86_64 architecture...");
    
    // Initialize GDT
    init_gdt();
    
    // Initialize IDT
    init_idt();
    
    // Initialize paging
    init_paging(config)?;
    
    crate::log::kprintln!("x86_64 initialization complete");
    Ok(())
}

/// Initialize Global Descriptor Table
fn init_gdt() {
    crate::log::kprintln!("Initializing GDT...");
    // TODO: Implement GDT setup
}

/// Initialize Interrupt Descriptor Table
fn init_idt() {
    crate::log::kprintln!("Initializing IDT...");
    // TODO: Implement IDT setup
}

/// Initialize paging
fn init_paging(config: &KernelConfig) -> KernelResult<()> {
    crate::log::kprintln!("Initializing paging...");
    // TODO: Implement paging setup
    Ok(())
}

/// Halt the CPU
pub fn halt() -> ! {
    loop {
        hlt();
    }
}

/// Shutdown the system
pub fn shutdown() -> ! {
    crate::log::kprintln!("x86_64 shutdown");
    
    // TODO: Implement proper ACPI shutdown
    // For now, just halt
    halt();
}

/// Disable interrupts
pub unsafe fn disable_interrupts() {
    interrupts::disable();
}

/// Enable interrupts
pub unsafe fn enable_interrupts() {
    interrupts::enable();
}

