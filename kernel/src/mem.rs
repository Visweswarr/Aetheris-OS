//! Memory management
//!
//! Provides memory allocation and management for the kernel.

use crate::{MemoryRegion, MemoryType, error::KernelResult};
use linked_list_allocator::LockedHeap;

/// Global heap allocator
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize memory management
pub fn init(memory_regions: &[MemoryRegion]) -> KernelResult<()> {
    crate::log::kprintln!("Initializing memory management...");
    
    // Find the largest usable memory region for heap
    let heap_region = find_heap_region(memory_regions)?;
    
    crate::log::kprintln!("Using memory region at 0x{:x} (size: {} MB) for heap", 
        heap_region.start, 
        heap_region.size / 1024 / 1024
    );
    
    // Initialize the heap
    unsafe {
        ALLOCATOR.lock().init(heap_region.start as *mut u8, heap_region.size as usize);
    }
    
    crate::log::kprintln!("Memory management initialized");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_heap_region() {
        let regions = [
            MemoryRegion {
                start: 0x1000,
                size: 0x1000, // Too small
                region_type: MemoryType::Usable,
            },
            MemoryRegion {
                start: 0x100000,
                size: 0x200000, // 2MB - good candidate
                region_type: MemoryType::Usable,
            },
            MemoryRegion {
                start: 0x400000,
                size: 0x100000, // 1MB - minimum size
                region_type: MemoryType::Usable,
            },
        ];

        let heap_region = find_heap_region(&regions).unwrap();
        assert_eq!(heap_region.start, 0x100000);
        assert_eq!(heap_region.size, 0x200000);
    }
}

