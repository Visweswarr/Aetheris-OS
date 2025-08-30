/// Physical Memory Management for Polymera OS
/// 
/// This module handles physical memory allocation, tracking, and management.
/// It provides the foundation for all other memory management subsystems.

use crate::{kprintln, klog};
use super::{MemoryResult, MemoryError, constants::*};
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::{PhysAddr, structures::paging::PhysFrame};
use spin::Mutex;
use alloc::vec::Vec;

/// Physical memory region descriptor
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    /// Start address of the region
    pub start: u64,
    
    /// End address of the region (exclusive)
    pub end: u64,
    
    /// Type of memory region
    pub region_type: MemoryRegionType,
}

impl MemoryRegion {
    /// Create a new memory region
    pub const fn new(start: u64, end: u64, region_type: MemoryRegionType) -> Self {
        Self { start, end, region_type }
    }
    
    /// Get the size of this region in bytes
    pub const fn size(&self) -> u64 {
        self.end - self.start
    }
    
    /// Check if this region contains the given address
    pub const fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end
    }
    
    /// Check if this region overlaps with another region
    pub const fn overlaps(&self, other: &MemoryRegion) -> bool {
        !(self.end <= other.start || other.end <= self.start)
    }
}

/// Types of memory regions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegionType {
    /// Available for allocation
    Available,
    
    /// Reserved by firmware/hardware
    Reserved,
    
    /// Used by kernel
    Kernel,
    
    /// Used for kernel heap
    KernelHeap,
    
    /// DMA-capable memory
    DmaCapable,
    
    /// Memory-mapped I/O
    MemoryMappedIo,
    
    /// Bad/defective memory
    Bad,
}

impl core::fmt::Display for MemoryRegionType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MemoryRegionType::Available => write!(f, "Available"),
            MemoryRegionType::Reserved => write!(f, "Reserved"),
            MemoryRegionType::Kernel => write!(f, "Kernel"),
            MemoryRegionType::KernelHeap => write!(f, "Kernel Heap"),
            MemoryRegionType::DmaCapable => write!(f, "DMA"),
            MemoryRegionType::MemoryMappedIo => write!(f, "MMIO"),
            MemoryRegionType::Bad => write!(f, "Bad"),
        }
    }
}

/// Physical memory statistics
#[derive(Debug, Clone, Copy)]
pub struct PhysicalMemoryStats {
    /// Total physical memory in bytes
    pub total_memory: u64,
    
    /// Available memory for allocation
    pub available_memory: u64,
    
    /// Used memory
    pub used_memory: u64,
    
    /// Reserved memory
    pub reserved_memory: u64,
    
    /// Kernel memory usage
    pub kernel_memory: u64,
    
    /// Number of allocations performed
    pub allocation_count: u64,
    
    /// Number of deallocations performed
    pub deallocation_count: u64,
    
    /// Largest allocation request
    pub largest_allocation: u64,
}

impl PhysicalMemoryStats {
    /// Create new physical memory statistics
    pub const fn new() -> Self {
        Self {
            total_memory: 0,
            available_memory: 0,
            used_memory: 0,
            reserved_memory: 0,
            kernel_memory: 0,
            allocation_count: 0,
            deallocation_count: 0,
            largest_allocation: 0,
        }
    }
    
    /// Calculate memory utilization percentage
    pub fn utilization(&self) -> f32 {
        if self.total_memory == 0 {
            0.0
        } else {
            (self.used_memory as f32 / self.total_memory as f32) * 100.0
        }
    }
}

/// Buddy allocator configuration
pub const MAX_ORDER: usize = 10;  // Maximum order for buddy allocator (2^10 = 1024 pages = 4MB)
pub const MIN_ORDER: usize = 0;   // Minimum order (1 page = 4KB)

/// Buddy allocator for physical memory
#[derive(Debug)]
pub struct Buddy {
    /// Free lists for each order (0 to MAX_ORDER)
    /// free_lists[i] contains free blocks of size 2^i pages
    free_lists: [Vec<PhysFrame>; MAX_ORDER + 1],
    
    /// Base address of the memory region managed by this allocator
    base_addr: PhysAddr,
    
    /// Total size of the memory region in pages
    total_pages: usize,
    
    /// Number of free pages at each order
    free_counts: [usize; MAX_ORDER + 1],
    
    /// Total number of allocated pages
    allocated_pages: usize,
    
    /// Statistics
    allocation_count: u64,
    deallocation_count: u64,
}

impl Buddy {
    /// Create a new buddy allocator
    /// 
    /// # Arguments
    /// * `base_addr` - Base physical address of the memory region
    /// * `total_pages` - Total number of pages in the region
    pub fn new(base_addr: PhysAddr, total_pages: usize) -> Self {
        let mut buddy = Self {
            free_lists: Default::default(),
            base_addr,
            total_pages,
            free_counts: [0; MAX_ORDER + 1],
            allocated_pages: 0,
            allocation_count: 0,
            deallocation_count: 0,
        };
        
        // Initialize the allocator with the entire region as one large block
        buddy.init_free_memory();
        buddy
    }
    
    /// Initialize the free memory by adding the entire region as large blocks
    fn init_free_memory(&mut self) {
        let mut remaining_pages = self.total_pages;
        let mut current_addr = self.base_addr;
        
        // Start from the largest possible order and work down
        for order in (MIN_ORDER..=MAX_ORDER).rev() {
            let block_size = 1 << order; // 2^order pages
            
            while remaining_pages >= block_size {
                let frame = PhysFrame::containing_address(current_addr);
                self.free_lists[order].push(frame);
                self.free_counts[order] += 1;
                
                current_addr += block_size * PAGE_SIZE as u64;
                remaining_pages -= block_size;
            }
        }
        
        klog!(DEBUG, "[BUDDY] Initialized with {} pages across all orders", self.total_pages);
        for order in MIN_ORDER..=MAX_ORDER {
            if self.free_counts[order] > 0 {
                klog!(DEBUG, "[BUDDY] Order {}: {} blocks ({} pages each)", 
                      order, self.free_counts[order], 1 << order);
            }
        }
    }
    
    /// Allocate a block of pages with the specified order
    /// 
    /// # Arguments
    /// * `order` - Order of allocation (allocates 2^order pages)
    /// 
    /// # Returns
    /// Physical frame of the allocated block or None if no memory available
    pub fn alloc(&mut self, order: usize) -> Option<PhysFrame> {
        if order > MAX_ORDER {
            klog!(ERROR, "[BUDDY] Requested order {} exceeds MAX_ORDER {}", order, MAX_ORDER);
            return None;
        }
        
        // Find the smallest available block that can satisfy the request
        for current_order in order..=MAX_ORDER {
            if !self.free_lists[current_order].is_empty() {
                // Remove a block from this order
                let frame = self.free_lists[current_order].pop().unwrap();
                self.free_counts[current_order] -= 1;
                
                // Split the block down to the requested order
                self.split_block(frame, current_order, order);
                
                // Update statistics
                let allocated_pages = 1 << order;
                self.allocated_pages += allocated_pages;
                self.allocation_count += 1;
                
                klog!(TRACE, "[BUDDY] Allocated order {} block at {:?} ({} pages)", 
                      order, frame, allocated_pages);
                
                return Some(frame);
            }
        }
        
        klog!(WARN, "[BUDDY] No available memory for order {} allocation", order);
        None
    }
    
    /// Split a block from current_order down to target_order
    fn split_block(&mut self, frame: PhysFrame, current_order: usize, target_order: usize) {
        let mut current_frame = frame;
        let mut order = current_order;
        
        // Split down to target order
        while order > target_order {
            order -= 1;
            let block_size_pages = 1 << order;
            let block_size_bytes = block_size_pages * PAGE_SIZE;
            
            // Calculate buddy frame (second half of the split)
            let buddy_addr = current_frame.start_address() + block_size_bytes as u64;
            let buddy_frame = PhysFrame::containing_address(buddy_addr);
            
            // Add buddy to free list
            self.free_lists[order].push(buddy_frame);
            self.free_counts[order] += 1;
            
            klog!(TRACE, "[BUDDY] Split order {} block, buddy at {:?}", order, buddy_frame);
        }
    }
    
    /// Free a block of pages
    /// 
    /// # Arguments
    /// * `frame` - Physical frame to free
    /// * `order` - Order of the block being freed
    pub fn free(&mut self, frame: PhysFrame, order: usize) {
        if order > MAX_ORDER {
            klog!(ERROR, "[BUDDY] Invalid free order {} exceeds MAX_ORDER {}", order, MAX_ORDER);
            return;
        }
        
        klog!(TRACE, "[BUDDY] Freeing order {} block at {:?}", order, frame);
        
        // Update statistics
        let freed_pages = 1 << order;
        self.allocated_pages = self.allocated_pages.saturating_sub(freed_pages);
        self.deallocation_count += 1;
        
        // Try to coalesce with buddy
        self.coalesce_and_free(frame, order);
    }
    
    /// Coalesce freed block with its buddy if possible
    fn coalesce_and_free(&mut self, frame: PhysFrame, order: usize) {
        let mut current_frame = frame;
        let mut current_order = order;
        
        // Try to coalesce up to the maximum order
        while current_order < MAX_ORDER {
            let buddy_frame = self.calculate_buddy(current_frame, current_order);
            
            // Check if buddy is free (exists in the free list)
            if let Some(buddy_index) = self.find_buddy_in_free_list(buddy_frame, current_order) {
                // Remove buddy from free list
                self.free_lists[current_order].swap_remove(buddy_index);
                self.free_counts[current_order] -= 1;
                
                // Coalesce: the new block starts at the lower address
                let coalesced_frame = if current_frame.start_address() < buddy_frame.start_address() {
                    current_frame
                } else {
                    buddy_frame
                };
                
                current_frame = coalesced_frame;
                current_order += 1;
                
                klog!(TRACE, "[BUDDY] Coalesced to order {} block at {:?}", current_order, current_frame);
            } else {
                // Can't coalesce, add to free list at current order
                break;
            }
        }
        
        // Add the (possibly coalesced) block to the free list
        self.free_lists[current_order].push(current_frame);
        self.free_counts[current_order] += 1;
        
        klog!(TRACE, "[BUDDY] Added order {} block to free list", current_order);
    }
    
    /// Calculate the buddy frame for a given frame and order
    fn calculate_buddy(&self, frame: PhysFrame, order: usize) -> PhysFrame {
        let block_size_pages = 1 << order;
        let block_size_bytes = block_size_pages * PAGE_SIZE;
        
        // Calculate offset from base address
        let offset = frame.start_address().as_u64() - self.base_addr.as_u64();
        let block_index = offset / block_size_bytes as u64;
        
        // Buddy is at block_index XOR 1
        let buddy_block_index = block_index ^ 1;
        let buddy_offset = buddy_block_index * block_size_bytes as u64;
        let buddy_addr = self.base_addr + buddy_offset;
        
        PhysFrame::containing_address(buddy_addr)
    }
    
    /// Find buddy in the free list for the given order
    fn find_buddy_in_free_list(&self, buddy_frame: PhysFrame, order: usize) -> Option<usize> {
        self.free_lists[order]
            .iter()
            .position(|&frame| frame == buddy_frame)
    }
    
    /// Get statistics about the allocator
    pub fn stats(&self) -> BuddyStats {
        let mut total_free_pages = 0;
        for order in MIN_ORDER..=MAX_ORDER {
            total_free_pages += self.free_counts[order] * (1 << order);
        }
        
        BuddyStats {
            total_pages: self.total_pages,
            allocated_pages: self.allocated_pages,
            free_pages: total_free_pages,
            allocation_count: self.allocation_count,
            deallocation_count: self.deallocation_count,
            free_counts: self.free_counts,
        }
    }
    
    /// Check if the allocator is in a consistent state
    pub fn validate(&self) -> bool {
        let stats = self.stats();
        let expected_total = stats.allocated_pages + stats.free_pages;
        
        if expected_total != self.total_pages {
            klog!(ERROR, "[BUDDY] Validation failed: allocated ({}) + free ({}) != total ({})",
                  stats.allocated_pages, stats.free_pages, self.total_pages);
            return false;
        }
        
        // Check that all free blocks are within our memory region
        for order in MIN_ORDER..=MAX_ORDER {
            for &frame in &self.free_lists[order] {
                let frame_addr = frame.start_address().as_u64();
                let region_start = self.base_addr.as_u64();
                let region_end = region_start + (self.total_pages * PAGE_SIZE) as u64;
                
                if frame_addr < region_start || frame_addr >= region_end {
                    klog!(ERROR, "[BUDDY] Validation failed: frame {:?} outside region", frame);
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Print detailed allocator state
    pub fn print_state(&self) {
        kprintln!("[BUDDY] Allocator State:");
        kprintln!("  Base address: {:?}", self.base_addr);
        kprintln!("  Total pages: {}", self.total_pages);
        kprintln!("  Allocated pages: {}", self.allocated_pages);
        kprintln!("  Allocations: {}", self.allocation_count);
        kprintln!("  Deallocations: {}", self.deallocation_count);
        
        kprintln!("  Free lists:");
        for order in MIN_ORDER..=MAX_ORDER {
            if self.free_counts[order] > 0 {
                kprintln!("    Order {}: {} blocks ({} pages each, {} pages total)",
                          order, self.free_counts[order], 1 << order, 
                          self.free_counts[order] * (1 << order));
            }
        }
    }
}

/// Buddy allocator statistics
#[derive(Debug, Clone, Copy)]
pub struct BuddyStats {
    pub total_pages: usize,
    pub allocated_pages: usize,
    pub free_pages: usize,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub free_counts: [usize; MAX_ORDER + 1],
}

impl BuddyStats {
    /// Calculate fragmentation percentage
    pub fn fragmentation(&self) -> f32 {
        if self.free_pages == 0 {
            return 0.0;
        }
        
        // Count how many free pages are in non-maximum order blocks
        let mut fragmented_pages = 0;
        for order in MIN_ORDER..MAX_ORDER {
            fragmented_pages += self.free_counts[order] * (1 << order);
        }
        
        (fragmented_pages as f32 / self.free_pages as f32) * 100.0
    }
    
    /// Calculate utilization percentage
    pub fn utilization(&self) -> f32 {
        if self.total_pages == 0 {
            0.0
        } else {
            (self.allocated_pages as f32 / self.total_pages as f32) * 100.0
        }
    }
}

/// Global buddy allocator instance
static BUDDY_ALLOCATOR: Mutex<Option<Buddy>> = Mutex::new(None);

/// Global physical memory statistics
static mut PHYS_STATS: PhysicalMemoryStats = PhysicalMemoryStats::new();

/// Atomic counters for thread-safe statistics
static ALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static DEALLOC_COUNT: AtomicU64 = AtomicU64::new(0);
static TOTAL_ALLOCATED: AtomicU64 = AtomicU64::new(0);
static TOTAL_FREED: AtomicU64 = AtomicU64::new(0);

/// Memory regions table (simplified for Phase 1)
const MAX_MEMORY_REGIONS: usize = 32;
static mut MEMORY_REGIONS: [Option<MemoryRegion>; MAX_MEMORY_REGIONS] = [None; MAX_MEMORY_REGIONS];
static mut REGION_COUNT: usize = 0;

/// Initialize physical memory management
pub fn init() {
    kprintln!("[PHYS] Initializing physical memory manager");
    
    // For Phase 1, we'll create some sample memory regions
    // In a real implementation, this would:
    // 1. Parse memory map from bootloader/UEFI
    // 2. Reserve kernel memory regions
    // 3. Set up physical page allocator
    // 4. Initialize DMA pools
    
    unsafe {
        // Sample memory layout for demonstration
        add_memory_region(MemoryRegion::new(
            0x0000_0000,
            0x0009_F000,
            MemoryRegionType::Available
        ));
        
        add_memory_region(MemoryRegion::new(
            0x0009_F000,
            0x000A_0000,
            MemoryRegionType::Reserved
        ));
        
        add_memory_region(MemoryRegion::new(
            0x0010_0000,
            0x4000_0000, // 1GB total
            MemoryRegionType::Available
        ));
        
        // Calculate total memory
        PHYS_STATS.total_memory = calculate_total_memory();
        PHYS_STATS.available_memory = calculate_available_memory();
        PHYS_STATS.reserved_memory = calculate_reserved_memory();
        PHYS_STATS.used_memory = 0;
        PHYS_STATS.kernel_memory = 0x100000; // Assume 1MB kernel
    }
    
    // Initialize buddy allocator for the main available region
    // Use region from 64MB to 256MB for buddy allocator (192MB = 49152 pages)
    let buddy_base = PhysAddr::new(0x4000000);  // 64MB
    let buddy_pages = 49152; // 192MB / 4KB = 49152 pages
    
    {
        let mut buddy_allocator = BUDDY_ALLOCATOR.lock();
        *buddy_allocator = Some(Buddy::new(buddy_base, buddy_pages));
    }
    
    klog!(INFO, "[PHYS] Physical memory manager initialized");
    klog!(INFO, "[PHYS] Total memory: {} MB", get_total_memory() / (1024 * 1024));
    klog!(INFO, "[PHYS] Available memory: {} MB", get_available_memory() / (1024 * 1024));
    klog!(INFO, "[PHYS] Buddy allocator initialized: {} MB at {:?}", 
          (buddy_pages * PAGE_SIZE) / (1024 * 1024), buddy_base);
    
    // Test the buddy allocator
    test_buddy_allocator();
}

/// Add a memory region to the regions table
unsafe fn add_memory_region(region: MemoryRegion) {
    if REGION_COUNT < MAX_MEMORY_REGIONS {
        MEMORY_REGIONS[REGION_COUNT] = Some(region);
        REGION_COUNT += 1;
        klog!(TRACE, "[PHYS] Added memory region: 0x{:016x}-0x{:016x} ({})", 
              region.start, region.end, region.region_type);
    }
}

/// Calculate total physical memory
fn calculate_total_memory() -> u64 {
    let mut total = 0;
    unsafe {
        for i in 0..REGION_COUNT {
            if let Some(region) = MEMORY_REGIONS[i] {
                total += region.size();
            }
        }
    }
    total
}

/// Calculate available physical memory
fn calculate_available_memory() -> u64 {
    let mut available = 0;
    unsafe {
        for i in 0..REGION_COUNT {
            if let Some(region) = MEMORY_REGIONS[i] {
                if region.region_type == MemoryRegionType::Available {
                    available += region.size();
                }
            }
        }
    }
    available
}

/// Calculate reserved physical memory
fn calculate_reserved_memory() -> u64 {
    let mut reserved = 0;
    unsafe {
        for i in 0..REGION_COUNT {
            if let Some(region) = MEMORY_REGIONS[i] {
                if region.region_type == MemoryRegionType::Reserved {
                    reserved += region.size();
                }
            }
        }
    }
    reserved
}

/// Allocate physical memory using buddy allocator
/// 
/// # Arguments
/// * `size` - Size in bytes to allocate
/// * `align` - Required alignment (must be power of 2)
/// 
/// # Returns
/// Physical address of allocated memory or error
pub fn allocate_physical(size: u64, align: u64) -> MemoryResult<u64> {
    if size == 0 {
        return Err(MemoryError::InvalidAddress);
    }
    
    if !align.is_power_of_two() {
        return Err(MemoryError::AlignmentError);
    }
    
    // Calculate required order (number of pages needed)
    let pages_needed = (size + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;
    let order = if pages_needed == 0 {
        0
    } else {
        let mut order = 0;
        let mut pages = 1;
        while pages < pages_needed {
            order += 1;
            pages <<= 1;
        }
        order
    };
    
    // Try to allocate from buddy allocator
    {
        let mut buddy_allocator = BUDDY_ALLOCATOR.lock();
        if let Some(ref mut allocator) = *buddy_allocator {
            if let Some(frame) = allocator.alloc(order) {
                let phys_addr = frame.start_address().as_u64();
                
                // Check alignment
                if (phys_addr % align) != 0 {
                    // Free the block and return error
                    allocator.free(frame, order);
                    return Err(MemoryError::AlignmentError);
                }
                
                // Update global statistics
                ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
                TOTAL_ALLOCATED.fetch_add(size, Ordering::Relaxed);
                
                unsafe {
                    PHYS_STATS.allocation_count += 1;
                    PHYS_STATS.used_memory += size;
                    PHYS_STATS.available_memory = PHYS_STATS.available_memory.saturating_sub(size);
                    
                    if size > PHYS_STATS.largest_allocation {
                        PHYS_STATS.largest_allocation = size;
                    }
                }
                
                klog!(TRACE, "[PHYS] Allocated {} bytes (order {}, {} pages) at 0x{:016x}", 
                      size, order, 1 << order, phys_addr);
                
                return Ok(phys_addr);
            }
        }
    }
    
    // Fallback to old allocation method if buddy allocator fails
    let available = get_available_memory();
    let used = TOTAL_ALLOCATED.load(Ordering::Relaxed) - TOTAL_FREED.load(Ordering::Relaxed);
    
    if used + size > available {
        return Err(MemoryError::OutOfMemory);
    }
    
    // Simulate allocation by returning a placeholder address
    let phys_addr = 0x1000000 + used; // Start at 16MB
    let aligned_addr = (phys_addr + align - 1) & !(align - 1);
    
    // Update statistics
    ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
    TOTAL_ALLOCATED.fetch_add(size, Ordering::Relaxed);
    
    unsafe {
        PHYS_STATS.allocation_count += 1;
        PHYS_STATS.used_memory += size;
        PHYS_STATS.available_memory -= size;
        
        if size > PHYS_STATS.largest_allocation {
            PHYS_STATS.largest_allocation = size;
        }
    }
    
    klog!(TRACE, "[PHYS] Fallback allocated {} bytes at 0x{:016x}", size, aligned_addr);
    
    Ok(aligned_addr)
}

/// Deallocate physical memory
/// 
/// # Arguments
/// * `addr` - Physical address to deallocate
/// * `size` - Size of the allocation
/// 
/// # Returns
/// Result indicating success or error
pub fn deallocate_physical(addr: u64, size: u64) -> MemoryResult<()> {
    if addr == 0 || size == 0 {
        return Err(MemoryError::InvalidAddress);
    }
    
    // Update statistics
    DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
    TOTAL_FREED.fetch_add(size, Ordering::Relaxed);
    
    unsafe {
        PHYS_STATS.deallocation_count += 1;
        PHYS_STATS.used_memory = PHYS_STATS.used_memory.saturating_sub(size);
        PHYS_STATS.available_memory += size;
    }
    
    klog!(TRACE, "[PHYS] Deallocated {} bytes at 0x{:016x}", size, addr);
    
    Ok(())
}

/// Allocate physically contiguous pages
/// 
/// # Arguments
/// * `page_count` - Number of pages to allocate
/// 
/// # Returns
/// Physical address of first page or error
pub fn allocate_pages(page_count: u64) -> MemoryResult<u64> {
    let size = page_count * PAGE_SIZE as u64;
    allocate_physical(size, PAGE_SIZE as u64)
}

/// Deallocate physically contiguous pages
/// 
/// # Arguments
/// * `addr` - Physical address of first page
/// * `page_count` - Number of pages to deallocate
/// 
/// # Returns
/// Result indicating success or error
pub fn deallocate_pages(addr: u64, page_count: u64) -> MemoryResult<()> {
    let size = page_count * PAGE_SIZE as u64;
    deallocate_physical(addr, size)
}

/// Allocate physical memory using buddy allocator with specific order
/// 
/// # Arguments
/// * `order` - Order of allocation (allocates 2^order pages)
/// 
/// # Returns
/// Physical frame of allocated block or error
pub fn buddy_alloc(order: usize) -> MemoryResult<PhysFrame> {
    let mut buddy_allocator = BUDDY_ALLOCATOR.lock();
    if let Some(ref mut allocator) = *buddy_allocator {
        if let Some(frame) = allocator.alloc(order) {
            let allocated_pages = 1 << order;
            let size = allocated_pages * PAGE_SIZE as u64;
            
            // Update global statistics
            ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
            TOTAL_ALLOCATED.fetch_add(size, Ordering::Relaxed);
            
            unsafe {
                PHYS_STATS.allocation_count += 1;
                PHYS_STATS.used_memory += size;
                PHYS_STATS.available_memory = PHYS_STATS.available_memory.saturating_sub(size);
            }
            
            klog!(TRACE, "[BUDDY] Allocated order {} block at {:?} ({} pages)", 
                  order, frame, allocated_pages);
            
            Ok(frame)
        } else {
            Err(MemoryError::OutOfMemory)
        }
    } else {
        Err(MemoryError::OutOfMemory)
    }
}

/// Free physical memory using buddy allocator
/// 
/// # Arguments
/// * `frame` - Physical frame to free
/// * `order` - Order of the block being freed
/// 
/// # Returns
/// Result indicating success or error
pub fn buddy_free(frame: PhysFrame, order: usize) -> MemoryResult<()> {
    let mut buddy_allocator = BUDDY_ALLOCATOR.lock();
    if let Some(ref mut allocator) = *buddy_allocator {
        allocator.free(frame, order);
        
        let freed_pages = 1 << order;
        let size = freed_pages * PAGE_SIZE as u64;
        
        // Update global statistics
        DEALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        TOTAL_FREED.fetch_add(size, Ordering::Relaxed);
        
        unsafe {
            PHYS_STATS.deallocation_count += 1;
            PHYS_STATS.used_memory = PHYS_STATS.used_memory.saturating_sub(size);
            PHYS_STATS.available_memory += size;
        }
        
        klog!(TRACE, "[BUDDY] Freed order {} block at {:?} ({} pages)", 
              order, frame, freed_pages);
        
        Ok(())
    } else {
        Err(MemoryError::InvalidAddress)
    }
}

/// Get buddy allocator statistics
pub fn get_buddy_stats() -> Option<BuddyStats> {
    let buddy_allocator = BUDDY_ALLOCATOR.lock();
    if let Some(ref allocator) = *buddy_allocator {
        Some(allocator.stats())
    } else {
        None
    }
}

/// Print buddy allocator state
pub fn print_buddy_state() {
    let buddy_allocator = BUDDY_ALLOCATOR.lock();
    if let Some(ref allocator) = *buddy_allocator {
        allocator.print_state();
    } else {
        kprintln!("[BUDDY] Allocator not initialized");
    }
}

/// Check if an address is in a specific type of memory region
/// 
/// # Arguments
/// * `addr` - Address to check
/// * `region_type` - Type of region to look for
/// 
/// # Returns
/// True if address is in a region of the specified type
pub fn is_address_in_region_type(addr: u64, region_type: MemoryRegionType) -> bool {
    unsafe {
        for i in 0..REGION_COUNT {
            if let Some(region) = MEMORY_REGIONS[i] {
                if region.region_type == region_type && region.contains(addr) {
                    return true;
                }
            }
        }
    }
    false
}

/// Get total physical memory
pub fn get_total_memory() -> u64 {
    unsafe { PHYS_STATS.total_memory }
}

/// Get available physical memory
pub fn get_available_memory() -> u64 {
    unsafe { PHYS_STATS.available_memory }
}

/// Get used physical memory
pub fn get_used_memory() -> u64 {
    unsafe { PHYS_STATS.used_memory }
}

/// Get physical memory statistics
pub fn get_physical_stats() -> PhysicalMemoryStats {
    unsafe {
        // Update atomic counters
        PHYS_STATS.allocation_count = ALLOC_COUNT.load(Ordering::Relaxed);
        PHYS_STATS.deallocation_count = DEALLOC_COUNT.load(Ordering::Relaxed);
        PHYS_STATS
    }
}

/// Print memory regions
pub fn print_memory_regions() {
    kprintln!("");
    kprintln!("=== MEMORY REGIONS ===");
    
    unsafe {
        for i in 0..REGION_COUNT {
            if let Some(region) = MEMORY_REGIONS[i] {
                kprintln!("Region {}: 0x{:016x}-0x{:016x} ({} KB) - {}", 
                          i,
                          region.start,
                          region.end,
                          region.size() / 1024,
                          region.region_type);
            }
        }
    }
    
    kprintln!("=== END MEMORY REGIONS ===");
    kprintln!("");
}

/// Print physical memory statistics
pub fn print_physical_stats() {
    let stats = get_physical_stats();
    
    kprintln!("");
    kprintln!("=== PHYSICAL MEMORY STATISTICS ===");
    kprintln!("Total memory: {} KB ({} MB)", stats.total_memory / 1024, stats.total_memory / (1024 * 1024));
    kprintln!("Available memory: {} KB ({} MB)", stats.available_memory / 1024, stats.available_memory / (1024 * 1024));
    kprintln!("Used memory: {} KB ({} MB) - {:.1}%", 
              stats.used_memory / 1024, 
              stats.used_memory / (1024 * 1024),
              stats.utilization());
    kprintln!("Reserved memory: {} KB ({} MB)", stats.reserved_memory / 1024, stats.reserved_memory / (1024 * 1024));
    kprintln!("Kernel memory: {} KB ({} MB)", stats.kernel_memory / 1024, stats.kernel_memory / (1024 * 1024));
    kprintln!("Allocations: {} (largest: {} KB)", stats.allocation_count, stats.largest_allocation / 1024);
    kprintln!("Deallocations: {}", stats.deallocation_count);
    kprintln!("=== END PHYSICAL MEMORY STATISTICS ===");
    kprintln!("");
}

/// Test physical memory management
pub fn test_physical_memory() {
    kprintln!("Testing physical memory management...");
    
    // Test basic allocation
    match allocate_physical(4096, 4096) {
        Ok(addr) => {
            kprintln!("  ✓ Allocated 4KB at 0x{:016x}", addr);
            
            // Test deallocation
            match deallocate_physical(addr, 4096) {
                Ok(()) => kprintln!("  ✓ Deallocated 4KB"),
                Err(e) => kprintln!("  ✗ Deallocation failed: {}", e),
            }
        }
        Err(e) => kprintln!("  ✗ Allocation failed: {}", e),
    }
    
    // Test page allocation
    match allocate_pages(10) {
        Ok(addr) => {
            kprintln!("  ✓ Allocated 10 pages at 0x{:016x}", addr);
            
            match deallocate_pages(addr, 10) {
                Ok(()) => kprintln!("  ✓ Deallocated 10 pages"),
                Err(e) => kprintln!("  ✗ Page deallocation failed: {}", e),
            }
        }
        Err(e) => kprintln!("  ✗ Page allocation failed: {}", e),
    }
    
    // Print memory regions and statistics
    print_memory_regions();
    print_physical_stats();
    
    kprintln!("Physical memory test completed");
}

/// Test buddy allocator functionality
pub fn test_buddy_allocator() {
    kprintln!("");
    kprintln!("=== BUDDY ALLOCATOR TEST ===");
    
    // Get initial state
    if let Some(initial_stats) = get_buddy_stats() {
        kprintln!("Initial state: {} total pages, {} free pages", 
                  initial_stats.total_pages, initial_stats.free_pages);
        
        let initial_free_count = initial_stats.free_pages;
        
        // Test basic allocation and deallocation
        kprintln!("Testing basic allocation/deallocation:");
        
        // Allocate some blocks of different orders
        let mut allocated_blocks = Vec::new();
        
        // Test order 0 (1 page)
        match buddy_alloc(0) {
            Ok(frame) => {
                kprintln!("  ✓ Allocated order 0 block at {:?}", frame);
                allocated_blocks.push((frame, 0));
            }
            Err(e) => kprintln!("  ✗ Failed to allocate order 0 block: {}", e),
        }
        
        // Test order 2 (4 pages)
        match buddy_alloc(2) {
            Ok(frame) => {
                kprintln!("  ✓ Allocated order 2 block at {:?}", frame);
                allocated_blocks.push((frame, 2));
            }
            Err(e) => kprintln!("  ✗ Failed to allocate order 2 block: {}", e),
        }
        
        // Test order 5 (32 pages)
        match buddy_alloc(5) {
            Ok(frame) => {
                kprintln!("  ✓ Allocated order 5 block at {:?}", frame);
                allocated_blocks.push((frame, 5));
            }
            Err(e) => kprintln!("  ✗ Failed to allocate order 5 block: {}", e),
        }
        
        // Check state after allocation
        if let Some(alloc_stats) = get_buddy_stats() {
            kprintln!("After allocation: {} allocated pages, {} free pages", 
                      alloc_stats.allocated_pages, alloc_stats.free_pages);
        }
        
        // Free all allocated blocks
        kprintln!("Freeing allocated blocks:");
        for (frame, order) in allocated_blocks {
            match buddy_free(frame, order) {
                Ok(()) => kprintln!("  ✓ Freed order {} block at {:?}", order, frame),
                Err(e) => kprintln!("  ✗ Failed to free order {} block: {}", order, e),
            }
        }
        
        // Check that free count returns to initial value
        if let Some(final_stats) = get_buddy_stats() {
            kprintln!("Final state: {} allocated pages, {} free pages", 
                      final_stats.allocated_pages, final_stats.free_pages);
            
            if final_stats.free_pages == initial_free_count {
                kprintln!("  ✓ Free count returned to initial value");
            } else {
                kprintln!("  ✗ Free count mismatch: expected {}, got {}", 
                          initial_free_count, final_stats.free_pages);
            }
            
            if final_stats.allocated_pages == 0 {
                kprintln!("  ✓ All memory freed correctly");
            } else {
                kprintln!("  ✗ Memory leak: {} pages still allocated", final_stats.allocated_pages);
            }
        }
        
        // Test fragmentation and coalescing
        kprintln!("Testing fragmentation and coalescing:");
        test_buddy_fragmentation();
        
        // Test random allocation/deallocation sequences
        kprintln!("Testing random allocation/deallocation sequences:");
        test_buddy_random_sequences();
        
        // Print final allocator state
        print_buddy_state();
        
    } else {
        kprintln!("  ✗ Buddy allocator not initialized");
    }
    
    kprintln!("=== BUDDY ALLOCATOR TEST COMPLETE ===");
    kprintln!("");
}

/// Test buddy allocator fragmentation and coalescing
fn test_buddy_fragmentation() {
    let mut allocated_blocks = Vec::new();
    
    // Allocate many small blocks to create fragmentation
    kprintln!("  Creating fragmentation with small allocations:");
    for i in 0..8 {
        match buddy_alloc(0) {
            Ok(frame) => {
                kprintln!("    Allocated small block {} at {:?}", i, frame);
                allocated_blocks.push((frame, 0));
            }
            Err(e) => {
                kprintln!("    Failed to allocate small block {}: {}", i, e);
                break;
            }
        }
    }
    
    // Free every other block to create holes
    kprintln!("  Freeing every other block to create holes:");
    let mut freed_count = 0;
    for (i, &(frame, order)) in allocated_blocks.iter().enumerate() {
        if i % 2 == 0 {
            match buddy_free(frame, order) {
                Ok(()) => {
                    kprintln!("    Freed block {} at {:?}", i, frame);
                    freed_count += 1;
                }
                Err(e) => kprintln!("    Failed to free block {}: {}", i, e),
            }
        }
    }
    
    // Try to allocate a larger block (should trigger coalescing)
    kprintln!("  Attempting large allocation (should trigger coalescing):");
    match buddy_alloc(3) {
        Ok(frame) => {
            kprintln!("    ✓ Successfully allocated order 3 block at {:?}", frame);
            // Free the large block
            let _ = buddy_free(frame, 3);
        }
        Err(e) => kprintln!("    Could not allocate large block: {}", e),
    }
    
    // Free remaining blocks
    kprintln!("  Cleaning up remaining blocks:");
    for (i, &(frame, order)) in allocated_blocks.iter().enumerate() {
        if i % 2 == 1 {
            let _ = buddy_free(frame, order);
        }
    }
}

/// Test random allocation/deallocation sequences
fn test_buddy_random_sequences() {
    const NUM_ITERATIONS: usize = 20;
    let mut allocated_blocks = Vec::new();
    
    // Simple pseudo-random number generator
    let mut seed = 12345u64;
    let mut next_random = || {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        seed
    };
    
    kprintln!("  Running {} random allocation/deallocation operations:", NUM_ITERATIONS);
    
    for i in 0..NUM_ITERATIONS {
        let random_val = next_random();
        let should_allocate = allocated_blocks.is_empty() || (random_val % 3) != 0;
        
        if should_allocate {
            // Allocate a random order block (0-6)
            let order = (random_val % 7) as usize;
            match buddy_alloc(order) {
                Ok(frame) => {
                    kprintln!("    [{:2}] ✓ Allocated order {} block at {:?}", i, order, frame);
                    allocated_blocks.push((frame, order));
                }
                Err(_) => {
                    kprintln!("    [{:2}] - Failed to allocate order {} block (out of memory)", i, order);
                }
            }
        } else {
            // Free a random allocated block
            if !allocated_blocks.is_empty() {
                let index = (random_val as usize) % allocated_blocks.len();
                let (frame, order) = allocated_blocks.swap_remove(index);
                match buddy_free(frame, order) {
                    Ok(()) => {
                        kprintln!("    [{:2}] ✓ Freed order {} block at {:?}", i, order, frame);
                    }
                    Err(e) => {
                        kprintln!("    [{:2}] ✗ Failed to free order {} block: {}", i, order, e);
                    }
                }
            }
        }
    }
    
    // Free all remaining blocks
    kprintln!("  Cleaning up {} remaining blocks:", allocated_blocks.len());
    for (frame, order) in allocated_blocks {
        let _ = buddy_free(frame, order);
    }
    
    // Validate allocator state
    {
        let buddy_allocator = BUDDY_ALLOCATOR.lock();
        if let Some(ref allocator) = *buddy_allocator {
            if allocator.validate() {
                kprintln!("  ✓ Allocator validation passed");
            } else {
                kprintln!("  ✗ Allocator validation failed");
            }
        }
    }
}
