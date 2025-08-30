/// Memory Management Module for Polymera OS
/// 
/// This module provides the core memory management functionality including
/// physical memory management, virtual memory management, paging, and allocation.

pub mod paging;
pub mod phys;
pub mod virt;
pub mod alloc;
pub mod safety;
pub mod guard;
pub mod audit;
pub mod tlb;

use crate::{kprintln, klog};

/// Memory management statistics
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Total physical memory in bytes
    pub total_physical: u64,
    
    /// Available physical memory in bytes
    pub available_physical: u64,
    
    /// Used physical memory in bytes
    pub used_physical: u64,
    
    /// Total virtual memory in bytes
    pub total_virtual: u64,
    
    /// Used virtual memory in bytes
    pub used_virtual: u64,
    
    /// Number of allocated pages
    pub allocated_pages: u64,
    
    /// Number of free pages
    pub free_pages: u64,
    
    /// Page size in bytes
    pub page_size: u64,
}

impl MemoryStats {
    /// Create new memory statistics with default values
    pub const fn new() -> Self {
        Self {
            total_physical: 0,
            available_physical: 0,
            used_physical: 0,
            total_virtual: 0,
            used_virtual: 0,
            allocated_pages: 0,
            free_pages: 0,
            page_size: 4096, // 4KB default page size
        }
    }
    
    /// Calculate physical memory utilization percentage
    pub fn physical_utilization(&self) -> f32 {
        if self.total_physical == 0 {
            0.0
        } else {
            (self.used_physical as f32 / self.total_physical as f32) * 100.0
        }
    }
    
    /// Calculate virtual memory utilization percentage
    pub fn virtual_utilization(&self) -> f32 {
        if self.total_virtual == 0 {
            0.0
        } else {
            (self.used_virtual as f32 / self.total_virtual as f32) * 100.0
        }
    }
}

impl core::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Memory Stats: {:.1}% physical ({} MB), {:.1}% virtual ({} MB), {} pages",
            self.physical_utilization(),
            self.used_physical / (1024 * 1024),
            self.virtual_utilization(),
            self.used_virtual / (1024 * 1024),
            self.allocated_pages
        )
    }
}

/// Global memory statistics
static mut MEMORY_STATS: MemoryStats = MemoryStats::new();

/// Initialize the memory management subsystem
/// 
/// This function initializes all memory management components in the correct order:
/// 1. Physical memory manager
/// 2. Paging system 
/// 3. Virtual memory manager
/// 4. Memory allocators
/// 5. Stack protection system
pub fn init_mm() {
    kprintln!("[MM] Initializing memory management subsystem");
    
    // Initialize physical memory management first
    kprintln!("[MM] Initializing physical memory manager");
    phys::init();
    
    // Initialize paging system
    kprintln!("[MM] Initializing paging system");
    paging::init();
    
    // Test paging functionality
    kprintln!("[MM] Testing paging functionality");
    paging::test_paging();
    
    // Initialize virtual memory management
    kprintln!("[MM] Initializing virtual memory manager");
    virt::init();
    
    // Initialize memory allocators
    kprintln!("[MM] Initializing memory allocators");
    alloc::init();
    
    // Initialize stack protection system
    kprintln!("[MM] Initializing stack protection system");
    guard::init_stack_protection();
    
    // Initialize TLB management system
    kprintln!("[MM] Initializing TLB management system");
    if let Err(e) = tlb::init_tlb_system() {
        klog!(ERROR, "[MM] Failed to initialize TLB system: {}", e);
    }
    
    // Initialize page table audit system
    kprintln!("[MM] Initializing page table audit system");
    if let Err(e) = audit::init_audit_system() {
        klog!(ERROR, "[MM] Failed to initialize audit system: {}", e);
    }
    
    // Initialize memory statistics
    update_memory_stats();
    
    klog!(INFO, "[MM] Memory management initialization complete");
    print_memory_info();
}

/// Update global memory statistics
/// 
/// This function collects memory usage information from all MM subsystems
/// and updates the global statistics.
pub fn update_memory_stats() {
    unsafe {
        MEMORY_STATS.total_physical = phys::get_total_memory();
        MEMORY_STATS.available_physical = phys::get_available_memory();
        MEMORY_STATS.used_physical = MEMORY_STATS.total_physical - MEMORY_STATS.available_physical;
        
        MEMORY_STATS.allocated_pages = paging::get_allocated_page_count();
        MEMORY_STATS.free_pages = paging::get_free_page_count();
        
        MEMORY_STATS.total_virtual = virt::get_total_virtual_space();
        MEMORY_STATS.used_virtual = virt::get_used_virtual_space();
        
        MEMORY_STATS.page_size = paging::get_page_size();
    }
}

/// Get current memory statistics
/// 
/// # Returns
/// Copy of current memory statistics
pub fn get_memory_stats() -> MemoryStats {
    unsafe { MEMORY_STATS }
}

/// Print comprehensive memory information
pub fn print_memory_info() {
    let stats = get_memory_stats();
    
    kprintln!("");
    kprintln!("=== MEMORY MANAGEMENT STATUS ===");
    kprintln!("Physical Memory:");
    kprintln!("  Total: {} KB ({} MB)", stats.total_physical / 1024, stats.total_physical / (1024 * 1024));
    kprintln!("  Available: {} KB ({} MB)", stats.available_physical / 1024, stats.available_physical / (1024 * 1024));
    kprintln!("  Used: {} KB ({} MB) - {:.1}%", 
              stats.used_physical / 1024, 
              stats.used_physical / (1024 * 1024),
              stats.physical_utilization());
    
    kprintln!("Virtual Memory:");
    kprintln!("  Total: {} KB ({} MB)", stats.total_virtual / 1024, stats.total_virtual / (1024 * 1024));
    kprintln!("  Used: {} KB ({} MB) - {:.1}%",
              stats.used_virtual / 1024,
              stats.used_virtual / (1024 * 1024), 
              stats.virtual_utilization());
    
    kprintln!("Paging:");
    kprintln!("  Page size: {} KB", stats.page_size / 1024);
    kprintln!("  Allocated pages: {}", stats.allocated_pages);
    kprintln!("  Free pages: {}", stats.free_pages);
    kprintln!("  Total pages: {}", stats.allocated_pages + stats.free_pages);
    
    kprintln!("=== END MEMORY STATUS ===");
    kprintln!("");
}

/// Memory management error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    /// Out of memory
    OutOfMemory,
    
    /// Invalid address
    InvalidAddress,
    
    /// Page not found
    PageNotFound,
    
    /// Permission denied
    PermissionDenied,
    
    /// Alignment error
    AlignmentError,
    
    /// Double free attempt
    DoubleFree,
    
    /// Use after free violation
    UseAfterFree,
    
    /// Buffer overflow detected
    BufferOverflow,
    
    /// Red zone violation
    RedZoneViolation,
    
    /// Guard page violation
    GuardPageViolation,
    
    /// Allocation too large
    AllocationTooLarge,
    
    /// Fragmentation prevents allocation
    Fragmentation,
    
    /// Page already mapped
    AlreadyMapped,
    
    /// Page not mapped
    NotMapped,
}

impl core::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MemoryError::OutOfMemory => write!(f, "Out of memory"),
            MemoryError::InvalidAddress => write!(f, "Invalid address"),
            MemoryError::PageNotFound => write!(f, "Page not found"),
            MemoryError::PermissionDenied => write!(f, "Permission denied"),
            MemoryError::AlignmentError => write!(f, "Alignment error"),
            MemoryError::DoubleFree => write!(f, "Double free attempt"),
            MemoryError::UseAfterFree => write!(f, "Use after free violation"),
            MemoryError::BufferOverflow => write!(f, "Buffer overflow detected"),
            MemoryError::RedZoneViolation => write!(f, "Red zone violation"),
            MemoryError::GuardPageViolation => write!(f, "Guard page violation"),
            MemoryError::AllocationTooLarge => write!(f, "Allocation too large"),
            MemoryError::Fragmentation => write!(f, "Memory fragmentation"),
            MemoryError::AlreadyMapped => write!(f, "Page already mapped"),
            MemoryError::NotMapped => write!(f, "Page not mapped"),
        }
    }
}

/// Memory management result type
pub type MemoryResult<T> = Result<T, MemoryError>;

// Re-export key functions for external use
pub use paging::{flush_tlb, translate_address, PageFlags};
pub use phys::{buddy_alloc, buddy_free, get_buddy_stats, print_buddy_state, Buddy, BuddyStats, MAX_ORDER, MIN_ORDER};
pub use alloc::{kmalloc, kfree, kcalloc, krealloc, KSlab, SlabAllocator, SLAB_SIZES, test_slab_allocator};
pub use virt::{
    map_page, unmap_page, translate_va, is_mapped, get_page_flags, check_access,
    update_flags, get_vm_stats, validate_vm, print_vm_mappings, test_vm_api,
    VmFlags, VmStats, PageMapping, VirtualMemoryManager
};
pub use safety::{
    init as init_memory_safety, get_memory_safety, track_allocation, track_deallocation,
    validate_access, print_report as print_memory_safety_report, MemorySafetyManager,
    MemorySafetySnapshot, POISON_PATTERNS, RED_ZONE_SIZE, GUARD_PAGE_SIZE
};
pub use audit::{run_page_table_audit, get_audit_stats, print_audit_stats, print_suspicious_pages};
pub use tlb::{flush_page, flush_all, flush_range, flush_multiple_pages, get_tlb_stats, print_tlb_stats};
pub use x86_64::{VirtAddr, PhysAddr};

// Global allocator support
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// Simple global allocator using kernel heap
pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // For now, use the existing kmalloc function
        match crate::mm::alloc::kmalloc(layout.size(), layout.align()) {
            Ok(ptr) => {
                // Track allocation for memory safety
                let _ = track_allocation(ptr, layout.size());
                ptr
            },
            Err(_) => null_mut(),
        }
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Track deallocation for memory safety
        let _ = track_deallocation(ptr);
        let _ = crate::mm::alloc::kfree(ptr, layout.size(), layout.align());
    }
}

#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator;

/// Common memory management constants
pub mod constants {
    /// Standard page size (4KB)
    pub const PAGE_SIZE: usize = 4096;
    
    /// Large page size (2MB)
    pub const LARGE_PAGE_SIZE: usize = 2 * 1024 * 1024;
    
    /// Huge page size (1GB)
    pub const HUGE_PAGE_SIZE: usize = 1024 * 1024 * 1024;
    
    /// Page alignment mask
    pub const PAGE_MASK: usize = PAGE_SIZE - 1;
    
    /// Kernel virtual address space start
    pub const KERNEL_VIRT_START: u64 = 0xFFFF_8000_0000_0000;
    
    /// User virtual address space end
    pub const USER_VIRT_END: u64 = 0x0000_7FFF_FFFF_FFFF;
    
    /// Default heap size (64MB)
    pub const DEFAULT_HEAP_SIZE: usize = 64 * 1024 * 1024;
    
    /// Maximum allocation size (16MB)
    pub const MAX_ALLOCATION_SIZE: usize = 16 * 1024 * 1024;
}

/// Utility functions for memory management
pub mod utils {
    use super::constants::*;
    
    /// Round up to next page boundary
    pub const fn round_up_to_page(addr: u64) -> u64 {
        (addr + PAGE_SIZE as u64 - 1) & !(PAGE_SIZE as u64 - 1)
    }
    
    /// Round down to page boundary
    pub const fn round_down_to_page(addr: u64) -> u64 {
        addr & !(PAGE_SIZE as u64 - 1)
    }
    
    /// Check if address is page aligned
    pub const fn is_page_aligned(addr: u64) -> bool {
        (addr & PAGE_MASK as u64) == 0
    }
    
    /// Convert bytes to number of pages (rounded up)
    pub const fn bytes_to_pages(bytes: u64) -> u64 {
        (bytes + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64
    }
    
    /// Convert pages to bytes
    pub const fn pages_to_bytes(pages: u64) -> u64 {
        pages * PAGE_SIZE as u64
    }
    
    /// Check if virtual address is in kernel space
    pub const fn is_kernel_address(addr: u64) -> bool {
        addr >= KERNEL_VIRT_START
    }
    
    /// Check if virtual address is in user space
    pub const fn is_user_address(addr: u64) -> bool {
        addr <= USER_VIRT_END
    }
}

/// Memory test functions for validation
pub mod test {
    use super::*;
    use crate::kprintln;
    
    /// Run basic memory management tests
    pub fn run_mm_tests() {
        kprintln!("");
        kprintln!("=== MEMORY MANAGEMENT TESTS ===");
        
        test_utils();
        test_memory_stats();
        
        kprintln!("=== MM TESTS COMPLETE ===");
        kprintln!("");
    }
    
    /// Test utility functions
    fn test_utils() {
        use crate::mm::utils::*;
        
        kprintln!("Testing MM utility functions:");
        
        // Test page alignment
        assert_eq!(round_up_to_page(0x1000), 0x1000);
        assert_eq!(round_up_to_page(0x1001), 0x2000);
        assert_eq!(round_down_to_page(0x1FFF), 0x1000);
        
        assert!(is_page_aligned(0x1000));
        assert!(!is_page_aligned(0x1001));
        
        // Test page calculations
        assert_eq!(bytes_to_pages(4096), 1);
        assert_eq!(bytes_to_pages(4097), 2);
        assert_eq!(pages_to_bytes(2), 8192);
        
        // Test address space checks
        assert!(is_kernel_address(0xFFFF_8000_0000_0000));
        assert!(is_user_address(0x0000_4000_0000_0000));
        
        kprintln!("  ✓ Utility functions working correctly");
    }
    
    /// Test memory statistics
    fn test_memory_stats() {
        kprintln!("Testing memory statistics:");
        
        let stats = get_memory_stats();
        kprintln!("  Current stats: {}", stats);
        
        // Basic validation
        assert!(stats.page_size > 0, "Page size should be positive");
        
        kprintln!("  ✓ Memory statistics functional");
    }
}

/// Memory allocator interface
pub trait Allocator {
    /// Allocate memory of specified size
    /// 
    /// # Arguments
    /// * `size` - Number of bytes to allocate
    /// * `align` - Required alignment
    /// 
    /// # Returns
    /// Pointer to allocated memory or error
    fn allocate(&mut self, size: usize, align: usize) -> MemoryResult<*mut u8>;
    
    /// Deallocate previously allocated memory
    /// 
    /// # Arguments
    /// * `ptr` - Pointer to memory to deallocate
    /// * `size` - Size of the allocation
    /// * `align` - Alignment of the allocation
    fn deallocate(&mut self, ptr: *mut u8, size: usize, align: usize) -> MemoryResult<()>;
    
    /// Reallocate memory to a new size
    /// 
    /// # Arguments
    /// * `ptr` - Pointer to existing allocation
    /// * `old_size` - Current size of allocation
    /// * `new_size` - Desired new size
    /// * `align` - Required alignment
    /// 
    /// # Returns
    /// Pointer to reallocated memory or error
    fn reallocate(&mut self, ptr: *mut u8, old_size: usize, new_size: usize, align: usize) -> MemoryResult<*mut u8> {
        // Default implementation: allocate new, copy, deallocate old
        let new_ptr = self.allocate(new_size, align)?;
        
        if !ptr.is_null() && old_size > 0 {
            unsafe {
                let copy_size = core::cmp::min(old_size, new_size);
                core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
            }
            self.deallocate(ptr, old_size, align)?;
        }
        
        Ok(new_ptr)
    }
    
    /// Get allocator statistics
    fn stats(&self) -> AllocatorStats;
}

/// Allocator statistics
#[derive(Debug, Clone, Copy)]
pub struct AllocatorStats {
    /// Total bytes allocated
    pub total_allocated: usize,
    
    /// Total bytes deallocated
    pub total_deallocated: usize,
    
    /// Current bytes in use
    pub bytes_in_use: usize,
    
    /// Number of active allocations
    pub active_allocations: usize,
    
    /// Largest allocation size
    pub largest_allocation: usize,
    
    /// Total number of allocations performed
    pub allocation_count: u64,
    
    /// Total number of deallocations performed
    pub deallocation_count: u64,
}

impl AllocatorStats {
    /// Create new allocator statistics
    pub const fn new() -> Self {
        Self {
            total_allocated: 0,
            total_deallocated: 0,
            bytes_in_use: 0,
            active_allocations: 0,
            largest_allocation: 0,
            allocation_count: 0,
            deallocation_count: 0,
        }
    }
    
    /// Calculate allocation efficiency
    pub fn efficiency(&self) -> f32 {
        if self.total_allocated == 0 {
            100.0
        } else {
            (self.bytes_in_use as f32 / self.total_allocated as f32) * 100.0
        }
    }
}

impl core::fmt::Display for AllocatorStats {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Allocator: {} KB in use, {} allocations, {:.1}% efficiency",
            self.bytes_in_use / 1024,
            self.active_allocations,
            self.efficiency()
        )
    }
}
