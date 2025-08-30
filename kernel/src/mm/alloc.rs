/// Memory Allocators for Polymera OS
/// 
/// This module provides various memory allocators for different use cases,
/// including a bump allocator, slab allocator, and buddy allocator.

use crate::{kprintln, klog};
use super::{MemoryResult, MemoryError, Allocator, AllocatorStats, constants::*};
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use spin::Mutex;

//=============================================================================
// SLAB ALLOCATOR IMPLEMENTATION
//=============================================================================

/// Slab allocator for fixed-size kernel objects
/// 
/// This allocator manages fixed-size objects efficiently using pre-allocated
/// slabs with free lists. Designed for frequent allocation/deallocation of
/// objects of the same size.

/// Supported slab sizes for kernel objects
pub const SLAB_SIZES: [usize; 5] = [32, 64, 128, 256, 512];

/// Poison byte pattern for debugging
#[cfg(test)]
const POISON_BYTE: u8 = 0xDE;

/// Individual slab for a specific object size
#[derive(Debug)]
pub struct KSlab {
    /// Size of objects in this slab
    pub size: usize,
    
    /// Free list containing available object pointers
    pub free: Vec<*mut u8>,
    
    /// Base address of the slab memory region
    base_addr: *mut u8,
    
    /// Total capacity (number of objects this slab can hold)
    capacity: usize,
    
    /// Number of objects currently allocated
    allocated_count: usize,
    
    /// Statistics
    allocation_count: u64,
    deallocation_count: u64,
    
    /// Total size of the slab in bytes
    total_size: usize,
}

impl KSlab {
    /// Create a new slab for objects of the given size
    /// 
    /// # Arguments
    /// * `size` - Size of objects this slab will manage
    /// * `base_addr` - Base address of memory region for this slab
    /// * `total_size` - Total size of memory available for this slab
    pub fn new(size: usize, base_addr: *mut u8, total_size: usize) -> Self {
        let capacity = total_size / size;
        let mut slab = KSlab {
            size,
            free: Vec::with_capacity(capacity),
            base_addr,
            capacity,
            allocated_count: 0,
            allocation_count: 0,
            deallocation_count: 0,
            total_size,
        };
        
        // Initialize free list with all available objects
        slab.init_free_list();
        slab
    }
    
    /// Initialize the free list with all available objects
    fn init_free_list(&mut self) {
        for i in 0..self.capacity {
            let object_ptr = unsafe { self.base_addr.add(i * self.size) };
            self.free.push(object_ptr);
        }
        
        klog!(DEBUG, "[SLAB] Initialized slab for {}-byte objects: {} objects, {} KB total",
              self.size, self.capacity, self.total_size / 1024);
    }
    
    /// Allocate an object from this slab
    /// 
    /// # Returns
    /// Pointer to allocated object or None if slab is full
    pub fn alloc_object(&mut self) -> Option<*mut u8> {
        if let Some(ptr) = self.free.pop() {
            self.allocated_count += 1;
            self.allocation_count += 1;
            
            // Track allocation for memory safety
            if let Err(e) = crate::mm::safety::track_allocation(ptr, self.size) {
                klog!(ERROR, "[SLAB] Failed to track allocation: {:?}", e);
            }
            
            klog!(TRACE, "[SLAB] Allocated {}-byte object at {:p} ({} free remaining)",
                  self.size, ptr, self.free.len());
            
            Some(ptr)
        } else {
            klog!(WARN, "[SLAB] Slab for {}-byte objects is full", self.size);
            None
        }
    }
    
    /// Free an object back to this slab
    /// 
    /// # Arguments
    /// * `ptr` - Pointer to object to free
    /// 
    /// # Returns
    /// True if object was successfully freed, false if invalid pointer
    pub fn free_object(&mut self, ptr: *mut u8) -> bool {
        // Validate that pointer belongs to this slab
        if !self.is_valid_pointer(ptr) {
            klog!(ERROR, "[SLAB] Invalid pointer {:p} for {}-byte slab", ptr, self.size);
            return false;
        }
        
        // Check for double-free (simplified check)
        for &free_ptr in &self.free {
            if free_ptr == ptr {
                klog!(ERROR, "[SLAB] Double-free detected for pointer {:p}", ptr);
                return false;
            }
        }
        
        // Track deallocation for memory safety
        if let Err(e) = crate::mm::safety::track_deallocation(ptr) {
            klog!(ERROR, "[SLAB] Failed to track deallocation: {:?}", e);
            return false;
        }
        
        // Add poison bytes in test configuration
        #[cfg(test)]
        unsafe {
            core::ptr::write_bytes(ptr, POISON_BYTE, self.size);
        }
        
        // Return object to free list
        self.free.push(ptr);
        self.allocated_count = self.allocated_count.saturating_sub(1);
        self.deallocation_count += 1;
        
        klog!(TRACE, "[SLAB] Freed {}-byte object at {:p} ({} free available)",
              self.size, ptr, self.free.len());
        
        true
    }
    
    /// Check if a pointer is valid for this slab
    fn is_valid_pointer(&self, ptr: *mut u8) -> bool {
        let ptr_addr = ptr as usize;
        let base_addr = self.base_addr as usize;
        let end_addr = base_addr + self.total_size;
        
        // Check if pointer is within slab bounds
        if ptr_addr < base_addr || ptr_addr >= end_addr {
            return false;
        }
        
        // Check if pointer is properly aligned for this object size
        let offset = ptr_addr - base_addr;
        offset % self.size == 0
    }
    
    /// Get slab statistics
    pub fn stats(&self) -> SlabStats {
        SlabStats {
            object_size: self.size,
            capacity: self.capacity,
            allocated_count: self.allocated_count,
            free_count: self.free.len(),
            allocation_count: self.allocation_count,
            deallocation_count: self.deallocation_count,
            total_size: self.total_size,
            utilization: if self.capacity > 0 {
                (self.allocated_count as f32 / self.capacity as f32) * 100.0
            } else {
                0.0
            },
        }
    }
    
    /// Check if slab is empty (no allocated objects)
    pub fn is_empty(&self) -> bool {
        self.allocated_count == 0
    }
    
    /// Check if slab is full (no free objects)
    pub fn is_full(&self) -> bool {
        self.free.is_empty()
    }
    
    /// Validate slab integrity
    pub fn validate(&self) -> bool {
        // Check that allocated + free = capacity
        let total_objects = self.allocated_count + self.free.len();
        if total_objects != self.capacity {
            klog!(ERROR, "[SLAB] Validation failed: allocated ({}) + free ({}) != capacity ({})",
                  self.allocated_count, self.free.len(), self.capacity);
            return false;
        }
        
        // Check that all free pointers are valid
        for &ptr in &self.free {
            if !self.is_valid_pointer(ptr) {
                klog!(ERROR, "[SLAB] Validation failed: invalid free pointer {:p}", ptr);
                return false;
            }
        }
        
        true
    }
    
    /// Print detailed slab state
    pub fn print_state(&self) {
        kprintln!("[SLAB] {}-byte objects:", self.size);
        kprintln!("  Capacity: {} objects", self.capacity);
        kprintln!("  Allocated: {} objects", self.allocated_count);
        kprintln!("  Free: {} objects", self.free.len());
        kprintln!("  Utilization: {:.1}%", self.stats().utilization);
        kprintln!("  Allocations: {}", self.allocation_count);
        kprintln!("  Deallocations: {}", self.deallocation_count);
        kprintln!("  Total size: {} KB", self.total_size / 1024);
    }
}

/// Statistics for a single slab
#[derive(Debug, Clone, Copy)]
pub struct SlabStats {
    pub object_size: usize,
    pub capacity: usize,
    pub allocated_count: usize,
    pub free_count: usize,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub total_size: usize,
    pub utilization: f32,
}

/// Slab allocator system managing multiple slab sizes
#[derive(Debug)]
pub struct SlabAllocator {
    /// Slabs for each supported size
    slabs: [Option<KSlab>; SLAB_SIZES.len()],
    
    /// Global statistics
    total_allocated_objects: u64,
    total_freed_objects: u64,
    total_allocated_bytes: u64,
    
    /// Memory region base address
    base_addr: *mut u8,
    
    /// Total memory size
    total_size: usize,
}

impl SlabAllocator {
    /// Create a new slab allocator
    /// 
    /// # Arguments
    /// * `base_addr` - Base address of memory region
    /// * `total_size` - Total size of memory region
    pub fn new(base_addr: *mut u8, total_size: usize) -> Self {
        let mut allocator = SlabAllocator {
            slabs: [None, None, None, None, None],
            total_allocated_objects: 0,
            total_freed_objects: 0,
            total_allocated_bytes: 0,
            base_addr,
            total_size,
        };
        
        allocator.init_slabs();
        allocator
    }
    
    /// Initialize slabs for all supported sizes
    fn init_slabs(&mut self) {
        let mut current_addr = self.base_addr;
        let size_per_slab = self.total_size / SLAB_SIZES.len();
        
        for (i, &size) in SLAB_SIZES.iter().enumerate() {
            let slab = KSlab::new(size, current_addr, size_per_slab);
            self.slabs[i] = Some(slab);
            
            current_addr = unsafe { current_addr.add(size_per_slab) };
        }
        
        klog!(INFO, "[SLAB] Initialized slab allocator with {} KB total",
              self.total_size / 1024);
        klog!(INFO, "[SLAB] Slab sizes: {:?}", SLAB_SIZES);
        klog!(INFO, "[SLAB] {} KB per slab", size_per_slab / 1024);
    }
    
    /// Find the appropriate slab index for a given size
    fn find_slab_index(&self, size: usize) -> Option<usize> {
        for (i, &slab_size) in SLAB_SIZES.iter().enumerate() {
            if size <= slab_size {
                return Some(i);
            }
        }
        None
    }
    
    /// Allocate an object of the given size
    /// 
    /// # Arguments
    /// * `size` - Size of object to allocate
    /// 
    /// # Returns
    /// Pointer to allocated object or None if allocation failed
    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        if size == 0 {
            return None;
        }
        
        let slab_index = self.find_slab_index(size)?;
        
        if let Some(ref mut slab) = self.slabs[slab_index] {
            if let Some(ptr) = slab.alloc_object() {
                self.total_allocated_objects += 1;
                self.total_allocated_bytes += size as u64;
                
                klog!(TRACE, "[SLAB] Allocated {} bytes (slab size {}) at {:p}",
                      size, SLAB_SIZES[slab_index], ptr);
                
                return Some(ptr);
            }
        }
        
        klog!(WARN, "[SLAB] Failed to allocate {} bytes", size);
        None
    }
    
    /// Free an object
    /// 
    /// # Arguments
    /// * `ptr` - Pointer to object to free
    /// * `size` - Size of object being freed
    /// 
    /// # Returns
    /// True if object was successfully freed
    pub fn free(&mut self, ptr: *mut u8, size: usize) -> bool {
        if ptr.is_null() || size == 0 {
            return false;
        }
        
        let slab_index = match self.find_slab_index(size) {
            Some(index) => index,
            None => {
                klog!(ERROR, "[SLAB] No slab for size {} when freeing {:p}", size, ptr);
                return false;
            }
        };
        
        if let Some(ref mut slab) = self.slabs[slab_index] {
            if slab.free_object(ptr) {
                self.total_freed_objects += 1;
                
                klog!(TRACE, "[SLAB] Freed {} bytes (slab size {}) at {:p}",
                      size, SLAB_SIZES[slab_index], ptr);
                
                return true;
            }
        }
        
        klog!(ERROR, "[SLAB] Failed to free {} bytes at {:p}", size, ptr);
        false
    }
    
    /// Get allocator statistics
    pub fn stats(&self) -> SlabAllocatorStats {
        let mut slab_stats = Vec::new();
        let mut total_capacity = 0;
        let mut total_allocated = 0;
        let mut total_free = 0;
        
        for slab_opt in &self.slabs {
            if let Some(ref slab) = slab_opt {
                let stats = slab.stats();
                total_capacity += stats.capacity;
                total_allocated += stats.allocated_count;
                total_free += stats.free_count;
                slab_stats.push(stats);
            }
        }
        
        SlabAllocatorStats {
            slab_stats,
            total_capacity,
            total_allocated,
            total_free,
            total_allocated_objects: self.total_allocated_objects,
            total_freed_objects: self.total_freed_objects,
            total_allocated_bytes: self.total_allocated_bytes,
            total_size: self.total_size,
            overall_utilization: if total_capacity > 0 {
                (total_allocated as f32 / total_capacity as f32) * 100.0
            } else {
                0.0
            },
        }
    }
    
    /// Validate all slabs
    pub fn validate(&self) -> bool {
        for (i, slab_opt) in self.slabs.iter().enumerate() {
            if let Some(ref slab) = slab_opt {
                if !slab.validate() {
                    klog!(ERROR, "[SLAB] Validation failed for slab {}", i);
                    return false;
                }
            }
        }
        true
    }
    
    /// Print detailed allocator state
    pub fn print_state(&self) {
        kprintln!("");
        kprintln!("=== SLAB ALLOCATOR STATE ===");
        kprintln!("Total memory: {} KB", self.total_size / 1024);
        kprintln!("Total allocated objects: {}", self.total_allocated_objects);
        kprintln!("Total freed objects: {}", self.total_freed_objects);
        kprintln!("Total allocated bytes: {} KB", self.total_allocated_bytes / 1024);
        
        let stats = self.stats();
        kprintln!("Overall utilization: {:.1}%", stats.overall_utilization);
        
        kprintln!("");
        kprintln!("Per-slab statistics:");
        for (i, slab_opt) in self.slabs.iter().enumerate() {
            if let Some(ref slab) = slab_opt {
                slab.print_state();
            }
        }
        
        kprintln!("=== END SLAB ALLOCATOR STATE ===");
        kprintln!("");
    }
}

/// Statistics for the entire slab allocator system
#[derive(Debug, Clone)]
pub struct SlabAllocatorStats {
    pub slab_stats: Vec<SlabStats>,
    pub total_capacity: usize,
    pub total_allocated: usize,
    pub total_free: usize,
    pub total_allocated_objects: u64,
    pub total_freed_objects: u64,
    pub total_allocated_bytes: u64,
    pub total_size: usize,
    pub overall_utilization: f32,
}

/// Global slab allocator instance
static GLOBAL_SLAB_ALLOCATOR: Mutex<Option<SlabAllocator>> = Mutex::new(None);

//=============================================================================
// END SLAB ALLOCATOR IMPLEMENTATION
//=============================================================================

/// Simple bump allocator for early boot allocation
/// 
/// This allocator simply bumps a pointer forward for each allocation
/// and never deallocates. Suitable for early kernel initialization.
pub struct BumpAllocator {
    /// Current allocation pointer
    current: u64,
    
    /// End of allocation region
    end: u64,
    
    /// Statistics
    stats: AllocatorStats,
}

impl BumpAllocator {
    /// Create a new bump allocator
    /// 
    /// # Arguments
    /// * `start` - Start address of allocation region
    /// * `size` - Size of allocation region
    pub const fn new(start: u64, size: u64) -> Self {
        Self {
            current: start,
            end: start + size,
            stats: AllocatorStats::new(),
        }
    }
    
    /// Check if allocator is out of memory
    pub fn is_exhausted(&self) -> bool {
        self.current >= self.end
    }
    
    /// Get remaining space
    pub fn remaining(&self) -> u64 {
        if self.current < self.end {
            self.end - self.current
        } else {
            0
        }
    }
}

impl Allocator for BumpAllocator {
    fn allocate(&mut self, size: usize, align: usize) -> MemoryResult<*mut u8> {
        if size == 0 {
            return Err(MemoryError::InvalidAddress);
        }
        
        if !align.is_power_of_two() {
            return Err(MemoryError::AlignmentError);
        }
        
        // Align current pointer
        let aligned_current = (self.current + align as u64 - 1) & !(align as u64 - 1);
        let new_current = aligned_current + size as u64;
        
        if new_current > self.end {
            return Err(MemoryError::OutOfMemory);
        }
        
        // Update pointer and statistics
        self.current = new_current;
        self.stats.total_allocated += size;
        self.stats.bytes_in_use += size;
        self.stats.active_allocations += 1;
        self.stats.allocation_count += 1;
        
        if size > self.stats.largest_allocation {
            self.stats.largest_allocation = size;
        }
        
        Ok(aligned_current as *mut u8)
    }
    
    fn deallocate(&mut self, _ptr: *mut u8, size: usize, _align: usize) -> MemoryResult<()> {
        // Bump allocator doesn't actually deallocate, just update stats
        self.stats.total_deallocated += size;
        self.stats.bytes_in_use = self.stats.bytes_in_use.saturating_sub(size);
        self.stats.active_allocations = self.stats.active_allocations.saturating_sub(1);
        self.stats.deallocation_count += 1;
        
        Ok(())
    }
    
    fn stats(&self) -> AllocatorStats {
        self.stats
    }
}

/// Fixed-size block allocator (simplified slab allocator)
/// 
/// This allocator manages fixed-size blocks efficiently.
pub struct BlockAllocator {
    /// Block size
    block_size: usize,
    
    /// Total number of blocks
    total_blocks: usize,
    
    /// Free block list head
    free_head: Option<usize>,
    
    /// Base address of allocation region
    base_addr: u64,
    
    /// Allocation bitmap (simplified for Phase 1)
    allocated_blocks: u64,
    
    /// Statistics
    stats: AllocatorStats,
}

impl BlockAllocator {
    /// Create a new block allocator
    /// 
    /// # Arguments
    /// * `base_addr` - Base address of allocation region
    /// * `total_size` - Total size of allocation region
    /// * `block_size` - Size of each block
    pub const fn new(base_addr: u64, total_size: usize, block_size: usize) -> Self {
        let total_blocks = total_size / block_size;
        
        Self {
            block_size,
            total_blocks,
            free_head: Some(0),
            base_addr,
            allocated_blocks: 0,
            stats: AllocatorStats::new(),
        }
    }
    
    /// Get address of a block by index
    const fn block_address(&self, index: usize) -> u64 {
        self.base_addr + (index * self.block_size) as u64
    }
    
    /// Get block index from address
    fn address_to_block(&self, addr: u64) -> Option<usize> {
        if addr >= self.base_addr {
            let offset = addr - self.base_addr;
            let index = (offset / self.block_size as u64) as usize;
            if index < self.total_blocks {
                Some(index)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Check if a block is allocated
    fn is_block_allocated(&self, index: usize) -> bool {
        if index < 64 {
            (self.allocated_blocks & (1 << index)) != 0
        } else {
            false // Simplified for Phase 1
        }
    }
    
    /// Mark a block as allocated
    fn mark_block_allocated(&mut self, index: usize) {
        if index < 64 {
            self.allocated_blocks |= 1 << index;
        }
    }
    
    /// Mark a block as free
    fn mark_block_free(&mut self, index: usize) {
        if index < 64 {
            self.allocated_blocks &= !(1 << index);
        }
    }
}

impl Allocator for BlockAllocator {
    fn allocate(&mut self, size: usize, align: usize) -> MemoryResult<*mut u8> {
        if size == 0 {
            return Err(MemoryError::InvalidAddress);
        }
        
        if size > self.block_size {
            return Err(MemoryError::AllocationTooLarge);
        }
        
        if !align.is_power_of_two() {
            return Err(MemoryError::AlignmentError);
        }
        
        // Find a free block
        for i in 0..self.total_blocks.min(64) { // Simplified for Phase 1
            if !self.is_block_allocated(i) {
                let block_addr = self.block_address(i);
                
                // Check alignment
                if (block_addr % align as u64) != 0 {
                    continue;
                }
                
                // Allocate this block
                self.mark_block_allocated(i);
                
                // Update statistics
                self.stats.total_allocated += self.block_size;
                self.stats.bytes_in_use += self.block_size;
                self.stats.active_allocations += 1;
                self.stats.allocation_count += 1;
                
                if self.block_size > self.stats.largest_allocation {
                    self.stats.largest_allocation = self.block_size;
                }
                
                return Ok(block_addr as *mut u8);
            }
        }
        
        Err(MemoryError::OutOfMemory)
    }
    
    fn deallocate(&mut self, ptr: *mut u8, _size: usize, _align: usize) -> MemoryResult<()> {
        let addr = ptr as u64;
        
        if let Some(index) = self.address_to_block(addr) {
            if self.is_block_allocated(index) {
                self.mark_block_free(index);
                
                // Update statistics
                self.stats.total_deallocated += self.block_size;
                self.stats.bytes_in_use = self.stats.bytes_in_use.saturating_sub(self.block_size);
                self.stats.active_allocations = self.stats.active_allocations.saturating_sub(1);
                self.stats.deallocation_count += 1;
                
                Ok(())
            } else {
                Err(MemoryError::DoubleFree)
            }
        } else {
            Err(MemoryError::InvalidAddress)
        }
    }
    
    fn stats(&self) -> AllocatorStats {
        self.stats
    }
}

/// Global allocator instance (simplified for Phase 1)
static mut GLOBAL_BUMP_ALLOCATOR: Option<BumpAllocator> = None;
static mut GLOBAL_BLOCK_ALLOCATOR: Option<BlockAllocator> = None;

/// Allocation counters
static TOTAL_ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static TOTAL_DEALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static TOTAL_ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

/// Initialize memory allocators
pub fn init() {
    kprintln!("[ALLOC] Initializing memory allocators");
    
    // Initialize slab allocator for kernel objects
    let slab_start = 0x5000000; // Start at 80MB
    let slab_size = 0x2000000;  // 32MB for slab allocator
    
    {
        let mut slab_allocator = GLOBAL_SLAB_ALLOCATOR.lock();
        *slab_allocator = Some(SlabAllocator::new(slab_start as *mut u8, slab_size));
    }
    
    unsafe {
        // Initialize bump allocator for early allocations
        let bump_start = 0x2000000; // Start at 32MB
        let bump_size = 0x1000000;  // 16MB for bump allocator
        GLOBAL_BUMP_ALLOCATOR = Some(BumpAllocator::new(bump_start, bump_size));
        
        // Initialize block allocator for fixed-size allocations
        let block_start = 0x3000000; // Start at 48MB
        let block_total_size = 0x1000000; // 16MB for block allocator
        let block_size = 64; // 64-byte blocks
        GLOBAL_BLOCK_ALLOCATOR = Some(BlockAllocator::new(block_start, block_total_size, block_size));
    }
    
    klog!(INFO, "[ALLOC] Memory allocators initialized");
    klog!(INFO, "[ALLOC] Slab allocator: 32MB at 0x5000000 (sizes: {:?})", SLAB_SIZES);
    klog!(INFO, "[ALLOC] Bump allocator: 16MB at 0x2000000");
    klog!(INFO, "[ALLOC] Block allocator: 16MB at 0x3000000 (64-byte blocks)");
    
    // Test the slab allocator
    test_slab_allocator();
}

/// Allocate memory using the appropriate allocator
/// 
/// # Arguments
/// * `size` - Size to allocate
/// * `align` - Alignment requirement
/// 
/// # Returns
/// Pointer to allocated memory or error
pub fn kmalloc(size: usize, align: usize) -> MemoryResult<*mut u8> {
    if size == 0 {
        return Err(MemoryError::InvalidAddress);
    }
    
    TOTAL_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    TOTAL_ALLOCATED_BYTES.fetch_add(size as u64, Ordering::Relaxed);
    
    // Check for fault injection - force allocation failure every Nth allocation
    if crate::fault_injection::should_force_alloc_failure() {
        crate::kprintln!("[FAULT_INJECTION] Forcing allocation failure for {} bytes", size);
        
        // Log the fault injection for audit purposes
        crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
            0, // System operation
            403, // FAULT_INJECTION_ALLOC_FAILURE
            size as u64
        ));
        
        return Err(MemoryError::OutOfMemory);
    }
    
    // Try slab allocator first for supported sizes
    if size <= 512 {
        let mut slab_allocator = GLOBAL_SLAB_ALLOCATOR.lock();
        if let Some(ref mut allocator) = *slab_allocator {
            if let Some(ptr) = allocator.alloc(size) {
                klog!(TRACE, "[ALLOC] Slab allocated {} bytes at {:p}", size, ptr);
                return Ok(ptr);
            }
        }
    }
    
    // Fall back to other allocators for larger sizes or when slab is full
    
    // Use block allocator for small allocations
    if size <= 64 {
        unsafe {
            if let Some(ref mut allocator) = GLOBAL_BLOCK_ALLOCATOR {
                let result = allocator.allocate(size, align);
                if result.is_ok() {
                    klog!(TRACE, "[ALLOC] Block allocated {} bytes", size);
                }
                return result;
            }
        }
    }
    
    // Use bump allocator for larger allocations
    unsafe {
        if let Some(ref mut allocator) = GLOBAL_BUMP_ALLOCATOR {
            let result = allocator.allocate(size, align);
            if result.is_ok() {
                klog!(TRACE, "[ALLOC] Bump allocated {} bytes", size);
            }
            return result;
        }
    }
    
    Err(MemoryError::OutOfMemory)
}

/// Deallocate memory
/// 
/// # Arguments
/// * `ptr` - Pointer to memory to deallocate
/// * `size` - Size of allocation
/// * `align` - Alignment of allocation
/// 
/// # Returns
/// Result indicating success or error
pub fn kfree(ptr: *mut u8, size: usize, align: usize) -> MemoryResult<()> {
    if ptr.is_null() {
        return Err(MemoryError::InvalidAddress);
    }
    
    TOTAL_DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    
    let addr = ptr as u64;
    
    // Try slab allocator first for supported sizes
    if size <= 512 {
        let mut slab_allocator = GLOBAL_SLAB_ALLOCATOR.lock();
        if let Some(ref mut allocator) = *slab_allocator {
            // Check if address is in slab range
            if addr >= 0x5000000 && addr < 0x7000000 {
                if allocator.free(ptr, size) {
                    klog!(TRACE, "[ALLOC] Slab freed {} bytes at {:p}", size, ptr);
                    return Ok(());
                }
            }
        }
    }
    
    // Try other allocators based on address ranges
    
    // Try block allocator
    if size <= 64 {
        unsafe {
            if let Some(ref mut allocator) = GLOBAL_BLOCK_ALLOCATOR {
                if addr >= 0x3000000 && addr < 0x4000000 {
                    klog!(TRACE, "[ALLOC] Block freed {} bytes", size);
                    return allocator.deallocate(ptr, size, align);
                }
            }
        }
    }
    
    // Try bump allocator (though it doesn't actually free)
    unsafe {
        if let Some(ref mut allocator) = GLOBAL_BUMP_ALLOCATOR {
            if addr >= 0x2000000 && addr < 0x3000000 {
                klog!(TRACE, "[ALLOC] Bump freed {} bytes (no-op)", size);
                return allocator.deallocate(ptr, size, align);
            }
        }
    }
    
    Err(MemoryError::InvalidAddress)
}

/// Allocate zeroed memory
/// 
/// # Arguments
/// * `size` - Size to allocate
/// * `align` - Alignment requirement
/// 
/// # Returns
/// Pointer to zeroed memory or error
pub fn kcalloc(size: usize, align: usize) -> MemoryResult<*mut u8> {
    let ptr = kmalloc(size, align)?;
    
    // Zero the memory
    unsafe {
        core::ptr::write_bytes(ptr, 0, size);
    }
    
    Ok(ptr)
}

/// Reallocate memory
/// 
/// # Arguments
/// * `ptr` - Existing pointer
/// * `old_size` - Current size
/// * `new_size` - Desired new size
/// * `align` - Alignment requirement
/// 
/// # Returns
/// Pointer to reallocated memory or error
pub fn krealloc(ptr: *mut u8, old_size: usize, new_size: usize, align: usize) -> MemoryResult<*mut u8> {
    if ptr.is_null() {
        return kmalloc(new_size, align);
    }
    
    if new_size == 0 {
        kfree(ptr, old_size, align)?;
        return Ok(core::ptr::null_mut());
    }
    
    // Allocate new memory
    let new_ptr = kmalloc(new_size, align)?;
    
    // Copy data
    unsafe {
        let copy_size = core::cmp::min(old_size, new_size);
        core::ptr::copy_nonoverlapping(ptr, new_ptr, copy_size);
    }
    
    // Free old memory
    kfree(ptr, old_size, align)?;
    
    Ok(new_ptr)
}

/// Get allocator statistics
pub fn get_allocator_stats() -> (AllocatorStats, AllocatorStats) {
    unsafe {
        let bump_stats = GLOBAL_BUMP_ALLOCATOR
            .as_ref()
            .map(|a| a.stats())
            .unwrap_or(AllocatorStats::new());
        
        let block_stats = GLOBAL_BLOCK_ALLOCATOR
            .as_ref()
            .map(|a| a.stats())
            .unwrap_or(AllocatorStats::new());
        
        (bump_stats, block_stats)
    }
}

/// Get global allocation statistics
pub fn get_global_alloc_stats() -> (u64, u64, u64) {
    (
        TOTAL_ALLOCATIONS.load(Ordering::Relaxed),
        TOTAL_DEALLOCATIONS.load(Ordering::Relaxed),
        TOTAL_ALLOCATED_BYTES.load(Ordering::Relaxed),
    )
}

/// Print allocator statistics
pub fn print_allocator_stats() {
    let (bump_stats, block_stats) = get_allocator_stats();
    let (total_allocs, total_deallocs, total_bytes) = get_global_alloc_stats();
    
    kprintln!("");
    kprintln!("=== ALLOCATOR STATISTICS ===");
    kprintln!("Global:");
    kprintln!("  Total allocations: {}", total_allocs);
    kprintln!("  Total deallocations: {}", total_deallocs);
    kprintln!("  Total bytes allocated: {} KB", total_bytes / 1024);
    
    kprintln!("Bump Allocator:");
    kprintln!("  {}", bump_stats);
    kprintln!("  Allocations: {}, Deallocations: {}", bump_stats.allocation_count, bump_stats.deallocation_count);
    
    kprintln!("Block Allocator:");
    kprintln!("  {}", block_stats);
    kprintln!("  Allocations: {}, Deallocations: {}", block_stats.allocation_count, block_stats.deallocation_count);
    
    unsafe {
        if let Some(ref allocator) = GLOBAL_BUMP_ALLOCATOR {
            kprintln!("  Remaining space: {} KB", allocator.remaining() / 1024);
        }
    }
    
    kprintln!("=== END ALLOCATOR STATISTICS ===");
    kprintln!("");
}

/// Test slab allocator functionality
pub fn test_slab_allocator() {
    kprintln!("");
    kprintln!("=== SLAB ALLOCATOR TEST ===");
    
    let initial_stats = {
        let slab_allocator = GLOBAL_SLAB_ALLOCATOR.lock();
        if let Some(ref allocator) = *slab_allocator {
            allocator.stats()
        } else {
            kprintln!("  ✗ Slab allocator not initialized");
            return;
        }
    };
    
    kprintln!("Initial state: {} total capacity across all slabs", initial_stats.total_capacity);
    
    // Test allocation across all slab sizes
    let mut allocated_objects = Vec::new();
    
    kprintln!("Testing allocation across all slab sizes:");
    for &size in &SLAB_SIZES {
        match kmalloc(size, 1) {
            Ok(ptr) => {
                kprintln!("  ✓ Allocated {}-byte object at {:p}", size, ptr);
                allocated_objects.push((ptr, size));
                
                // Test writing to allocated memory
                unsafe {
                    *ptr = (size & 0xFF) as u8;
                    if *ptr == (size & 0xFF) as u8 {
                        kprintln!("    ✓ Memory write/read successful");
                    } else {
                        kprintln!("    ✗ Memory write/read failed");
                    }
                }
            }
            Err(e) => kprintln!("  ✗ Failed to allocate {}-byte object: {}", size, e),
        }
    }
    
    // Test allocating many objects to verify capacity
    kprintln!("Testing bulk allocation (1000 objects of each size):");
    const BULK_COUNT: usize = 1000;
    let mut bulk_objects = Vec::new();
    
    for &size in &SLAB_SIZES {
        let mut count = 0;
        for i in 0..BULK_COUNT {
            match kmalloc(size, 1) {
                Ok(ptr) => {
                    bulk_objects.push((ptr, size));
                    count += 1;
                    
                    // Write a pattern to verify memory integrity
                    unsafe {
                        *ptr = ((i + size) & 0xFF) as u8;
                    }
                }
                Err(_) => break,
            }
        }
        kprintln!("  ✓ Allocated {} objects of size {} bytes", count, size);
    }
    
    // Verify memory integrity
    kprintln!("Verifying memory integrity:");
    let mut integrity_ok = true;
    for (i, &(ptr, size)) in bulk_objects.iter().enumerate() {
        unsafe {
            let expected = ((i + size) & 0xFF) as u8;
            if *ptr != expected {
                kprintln!("  ✗ Memory corruption at {:p}: expected {}, got {}", ptr, expected, *ptr);
                integrity_ok = false;
                break;
            }
        }
    }
    
    if integrity_ok {
        kprintln!("  ✓ All allocated memory has correct data");
    }
    
    // Test freeing objects
    kprintln!("Testing deallocation:");
    
    // Free initial test objects
    for (ptr, size) in allocated_objects {
        match kfree(ptr, size, 1) {
            Ok(()) => kprintln!("  ✓ Freed {}-byte object at {:p}", size, ptr),
            Err(e) => kprintln!("  ✗ Failed to free {}-byte object: {}", size, e),
        }
    }
    
    // Free bulk objects
    let bulk_count = bulk_objects.len();
    for (ptr, size) in bulk_objects {
        let _ = kfree(ptr, size, 1);
    }
    kprintln!("  ✓ Freed {} bulk objects", bulk_count);
    
    // Test allocating 10k objects as specified
    kprintln!("Testing 10k object allocation/deallocation:");
    const TEST_COUNT: usize = 10000;
    let mut test_objects = Vec::new();
    
    // Simple pseudo-random size selection
    let mut seed = 42u64;
    let mut next_size = || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        SLAB_SIZES[(seed as usize) % SLAB_SIZES.len()]
    };
    
    // Allocate 10k objects
    let mut allocated_count = 0;
    for i in 0..TEST_COUNT {
        let size = next_size();
        match kmalloc(size, 1) {
            Ok(ptr) => {
                test_objects.push((ptr, size));
                allocated_count += 1;
                
                // Write test pattern with poison detection
                unsafe {
                    *ptr = ((i ^ size) & 0xFF) as u8;
                }
            }
            Err(_) => break,
        }
    }
    
    kprintln!("  ✓ Allocated {} out of {} requested objects", allocated_count, TEST_COUNT);
    
    // Verify no memory leaks by checking all objects are freed
    let pre_free_stats = {
        let slab_allocator = GLOBAL_SLAB_ALLOCATOR.lock();
        if let Some(ref allocator) = *slab_allocator {
            allocator.stats()
        } else {
            return;
        }
    };
    
    // Free all test objects
    for (ptr, size) in test_objects {
        let _ = kfree(ptr, size, 1);
    }
    
    let post_free_stats = {
        let slab_allocator = GLOBAL_SLAB_ALLOCATOR.lock();
        if let Some(ref allocator) = *slab_allocator {
            allocator.stats()
        } else {
            return;
        }
    };
    
    // Check for memory leaks
    if post_free_stats.total_allocated == initial_stats.total_allocated {
        kprintln!("  ✓ No memory leaks detected - all objects freed");
    } else {
        kprintln!("  ✗ Memory leak detected: {} objects still allocated",
                  post_free_stats.total_allocated - initial_stats.total_allocated);
    }
    
    // Validate slab integrity
    let validation_ok = {
        let slab_allocator = GLOBAL_SLAB_ALLOCATOR.lock();
        if let Some(ref allocator) = *slab_allocator {
            allocator.validate()
        } else {
            false
        }
    };
    
    if validation_ok {
        kprintln!("  ✓ Slab allocator validation passed");
    } else {
        kprintln!("  ✗ Slab allocator validation failed");
    }
    
    // Print final statistics
    {
        let slab_allocator = GLOBAL_SLAB_ALLOCATOR.lock();
        if let Some(ref allocator) = *slab_allocator {
            allocator.print_state();
        }
    }
    
    kprintln!("=== SLAB ALLOCATOR TEST COMPLETE ===");
    kprintln!("");
}

/// Test memory allocators
pub fn test_allocators() {
    kprintln!("Testing memory allocators...");
    
    // Test basic allocation and deallocation
    match kmalloc(1024, 8) {
        Ok(ptr) => {
            kprintln!("  ✓ Allocated 1024 bytes at {:p}", ptr);
            
            // Test writing to allocated memory
            unsafe {
                *ptr = 0x42;
                if *ptr == 0x42 {
                    kprintln!("  ✓ Memory write/read successful");
                } else {
                    kprintln!("  ✗ Memory write/read failed");
                }
            }
            
            // Test deallocation
            match kfree(ptr, 1024, 8) {
                Ok(()) => kprintln!("  ✓ Deallocated memory"),
                Err(e) => kprintln!("  ✗ Deallocation failed: {}", e),
            }
        }
        Err(e) => kprintln!("  ✗ Allocation failed: {}", e),
    }
    
    // Test small block allocation
    match kmalloc(32, 4) {
        Ok(ptr) => {
            kprintln!("  ✓ Allocated 32-byte block at {:p}", ptr);
            
            match kfree(ptr, 32, 4) {
                Ok(()) => kprintln!("  ✓ Freed 32-byte block"),
                Err(e) => kprintln!("  ✗ Block deallocation failed: {}", e),
            }
        }
        Err(e) => kprintln!("  ✗ Block allocation failed: {}", e),
    }
    
    // Test zeroed allocation
    match kcalloc(256, 8) {
        Ok(ptr) => {
            kprintln!("  ✓ Allocated 256 zeroed bytes at {:p}", ptr);
            
            // Verify memory is zeroed
            unsafe {
                let mut all_zero = true;
                for i in 0..256 {
                    if *ptr.add(i) != 0 {
                        all_zero = false;
                        break;
                    }
                }
                
                if all_zero {
                    kprintln!("  ✓ Memory is properly zeroed");
                } else {
                    kprintln!("  ✗ Memory is not zeroed");
                }
            }
            
            let _ = kfree(ptr, 256, 8);
        }
        Err(e) => kprintln!("  ✗ Zeroed allocation failed: {}", e),
    }
    
    // Print allocator statistics
    print_allocator_stats();
    
    kprintln!("Allocator test completed");
}
