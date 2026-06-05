//! Memory management
//!
//! Provides memory allocation and management for the kernel.
//! Note: The global allocator is defined in mm/mod.rs to avoid duplication.

use crate::error::KernelResult;

/// Memory region descriptor
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
    pub region_type: MemoryType,
}

/// Memory region type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    Usable,
    Reserved,
    AcpiReclaimable,
    AcpiNvs,
    BadMemory,
    Kernel,
    BootloaderReclaimable,
}

/// Initialize memory management
pub fn init(memory_regions: &[MemoryRegion]) -> KernelResult<()> {
    crate::kprintln!("Initializing memory management...");
    
    // Find the largest usable memory region for heap
    let heap_region = find_heap_region(memory_regions)?;
    
    crate::kprintln!("Using memory region at 0x{:x} (size: {} MB) for heap", 
        heap_region.start, 
        heap_region.size / 1024 / 1024
    );
    
    crate::kprintln!("Memory management initialized");
    Ok(())
}

/// Find suitable memory region for heap
fn find_heap_region(memory_regions: &[MemoryRegion]) -> KernelResult<&MemoryRegion> {
    memory_regions
        .iter()
        .filter(|region| region.region_type == MemoryType::Usable)
        .filter(|region| region.size >= 1024 * 1024) // At least 1MB
        .max_by_key(|region| region.size)
        .ok_or(crate::error::KernelError::MemoryError)
}

