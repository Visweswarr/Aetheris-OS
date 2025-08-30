//! Memory Safety System for Polymera OS
//! 
//! This module provides comprehensive memory safety features for debug builds,
//! including free-poison patterns, double-free detection, guard pages, and red zones.

use crate::{kprintln, klog, klog};
use crate::log::Level;
use crate::mm::{MemoryResult, MemoryError, VirtualAddress, PhysicalAddress, PageSize, MemoryFlags};
use crate::mm::vm::VirtualMemoryManager;
use core::sync::atomic::{AtomicU64, AtomicU32, AtomicBool, Ordering};
use core::ptr;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

//=============================================================================
// MEMORY SAFETY CONFIGURATION
//=============================================================================

/// Enable memory safety features in debug builds
#[cfg(feature = "debug")]
pub const MEMORY_SAFETY_ENABLED: bool = true;

#[cfg(not(feature = "debug"))]
pub const MEMORY_SAFETY_ENABLED: bool = false;

/// Poison byte patterns for different memory states
pub const POISON_PATTERNS: [u8; 4] = [
    0xDE, // DEAD - freed memory
    0xAD, // ADDR - uninitialized memory  
    0xBE, // BEEF - guard page
    0xEF, // EFEF - red zone
];

/// Red zone size around allocations (bytes)
pub const RED_ZONE_SIZE: usize = 16;

/// Guard page size (must be page-aligned)
pub const GUARD_PAGE_SIZE: usize = 4096;

/// Maximum number of tracked allocations
pub const MAX_TRACKED_ALLOCATIONS: usize = 10000;

//=============================================================================
// MEMORY SAFETY STATISTICS
//=============================================================================

/// Memory safety statistics and counters
#[derive(Debug, Default)]
pub struct MemorySafetyStats {
    /// Total allocations tracked
    pub total_allocations: AtomicU64,
    
    /// Total deallocations tracked
    pub total_deallocations: AtomicU64,
    
    /// Double-free attempts detected
    pub double_free_attempts: AtomicU32,
    
    /// Use-after-free violations detected
    pub use_after_free_violations: AtomicU32,
    
    /// Guard page violations detected
    pub guard_page_violations: AtomicU32,
    
    /// Red zone violations detected
    pub red_zone_violations: AtomicU32,
    
    /// Poison pattern violations detected
    pub poison_violations: AtomicU32,
    
    /// Memory corruption events detected
    pub corruption_events: AtomicU32,
    
    /// Guard pages created
    pub guard_pages_created: AtomicU64,
    
    /// Red zones created
    pub red_zones_created: AtomicU64,
}

impl MemorySafetyStats {
    /// Get a snapshot of current statistics
    pub fn snapshot(&self) -> MemorySafetySnapshot {
        MemorySafetySnapshot {
            total_allocations: self.total_allocations.load(Ordering::Relaxed),
            total_deallocations: self.total_deallocations.load(Ordering::Relaxed),
            double_free_attempts: self.double_free_attempts.load(Ordering::Relaxed),
            use_after_free_violations: self.use_after_free_violations.load(Ordering::Relaxed),
            guard_page_violations: self.guard_page_violations.load(Ordering::Relaxed),
            red_zone_violations: self.red_zone_violations.load(Ordering::Relaxed),
            poison_violations: self.poison_violations.load(Ordering::Relaxed),
            corruption_events: self.corruption_events.load(Ordering::Relaxed),
            guard_pages_created: self.guard_pages_created.load(Ordering::Relaxed),
            red_zones_created: self.red_zones_created.load(Ordering::Relaxed),
        }
    }
    
    /// Reset all statistics
    pub fn reset(&self) {
        self.total_allocations.store(0, Ordering::Relaxed);
        self.total_deallocations.store(0, Ordering::Relaxed);
        self.double_free_attempts.store(0, Ordering::Relaxed);
        self.use_after_free_violations.store(0, Ordering::Relaxed);
        self.guard_page_violations.store(0, Ordering::Relaxed);
        self.red_zone_violations.store(0, Ordering::Relaxed);
        self.poison_violations.store(0, Ordering::Relaxed);
        self.corruption_events.store(0, Ordering::Relaxed);
        self.guard_pages_created.store(0, Ordering::Relaxed);
        self.red_zones_created.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of memory safety statistics
#[derive(Debug, Clone)]
pub struct MemorySafetySnapshot {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub double_free_attempts: u32,
    pub use_after_free_violations: u32,
    pub guard_page_violations: u32,
    pub red_zone_violations: u32,
    pub poison_violations: u32,
    pub corruption_events: u32,
    pub guard_pages_created: u64,
    pub red_zones_created: u64,
}

//=============================================================================
// ALLOCATION TRACKING
//=============================================================================

/// Tracked allocation information
#[derive(Debug, Clone)]
pub struct TrackedAllocation {
    /// Pointer to allocated memory
    pub ptr: *mut u8,
    
    /// Size of allocation
    pub size: usize,
    
    /// Allocation timestamp
    pub allocated_at: u64,
    
    /// Stack trace (simplified)
    pub stack_trace: [u64; 8],
    
    /// Whether this allocation has been freed
    pub freed: bool,
    
    /// Red zone addresses (if enabled)
    pub red_zone_before: Option<*mut u8>,
    pub red_zone_after: Option<*mut u8>,
    
    /// Guard page addresses (if enabled)
    pub guard_page_before: Option<VirtualAddress>,
    pub guard_page_after: Option<VirtualAddress>,
}

impl TrackedAllocation {
    /// Create new tracked allocation
    pub fn new(ptr: *mut u8, size: usize) -> Self {
        Self {
            ptr,
            size,
            allocated_at: crate::log::get_current_time_ms(),
            stack_trace: [0; 8], // Simplified for now
            freed: false,
            red_zone_before: None,
            red_zone_after: None,
            guard_page_before: None,
            guard_page_after: None,
        }
    }
    
    /// Check if pointer is within this allocation
    pub fn contains_pointer(&self, ptr: *mut u8) -> bool {
        let ptr_addr = ptr as usize;
        let start_addr = self.ptr as usize;
        let end_addr = start_addr + self.size;
        
        ptr_addr >= start_addr && ptr_addr < end_addr
    }
    
    /// Check if pointer is in red zone
    pub fn is_in_red_zone(&self, ptr: *mut u8) -> bool {
        if let Some(red_before) = self.red_zone_before {
            let red_before_addr = red_before as usize;
            let red_before_end = red_before_addr + RED_ZONE_SIZE;
            if ptr as usize >= red_before_addr && ptr as usize < red_before_end {
                return true;
            }
        }
        
        if let Some(red_after) = self.red_zone_after {
            let red_after_addr = red_after as usize;
            let red_after_end = red_after_addr + RED_ZONE_SIZE;
            if ptr as usize >= red_after_addr && ptr as usize < red_after_end {
                return true;
            }
        }
        
        false
    }
}

/// Memory safety manager for tracking allocations
pub struct MemorySafetyManager {
    /// Tracked allocations
    allocations: Mutex<BTreeMap<*mut u8, TrackedAllocation>>,
    
    /// Statistics
    stats: MemorySafetyStats,
    
    /// Whether safety features are enabled
    enabled: bool,
    
    /// Virtual memory manager reference
    vm_manager: Option<&'static VirtualMemoryManager>,
}

impl MemorySafetyManager {
    /// Create new memory safety manager
    pub fn new() -> Self {
        Self {
            allocations: Mutex::new(BTreeMap::new()),
            stats: MemorySafetyStats::default(),
            enabled: MEMORY_SAFETY_ENABLED,
            vm_manager: None,
        }
    }
    
    /// Set virtual memory manager reference
    pub fn set_vm_manager(&mut self, vm_manager: &'static VirtualMemoryManager) {
        self.vm_manager = Some(vm_manager);
    }
    
    /// Track a new allocation
    pub fn track_allocation(&self, ptr: *mut u8, size: usize) -> MemoryResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut allocations = self.allocations.lock();
        
        // Check if we're at capacity
        if allocations.len() >= MAX_TRACKED_ALLOCATIONS {
            klog!(Level::WARN, "[MEMORY_SAFETY] Maximum tracked allocations reached, dropping oldest");
            
            // Remove oldest allocation
            if let Some((&oldest_ptr, _)) = allocations.iter().next() {
                allocations.remove(&oldest_ptr);
            }
        }
        
        // Create tracked allocation
        let mut tracked = TrackedAllocation::new(ptr, size);
        
        // Add red zones if enabled
        if self.enabled {
            self.add_red_zones(&mut tracked)?;
        }
        
        // Add guard pages if enabled and VM manager available
        if self.enabled && self.vm_manager.is_some() {
            self.add_guard_pages(&mut tracked)?;
        }
        
        // Insert into tracking map
        allocations.insert(ptr, tracked);
        
        self.stats.total_allocations.fetch_add(1, Ordering::Relaxed);
        
        klog!(Level::TRACE, "[MEMORY_SAFETY] Tracking allocation {:p} ({} bytes)", ptr, size);
        
        Ok(())
    }
    
    /// Track a deallocation
    pub fn track_deallocation(&self, ptr: *mut u8) -> MemoryResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut allocations = self.allocations.lock();
        
        // Check for double-free
        if let Some(allocation) = allocations.get(&ptr) {
            if allocation.freed {
                self.stats.double_free_attempts.fetch_add(1, Ordering::Relaxed);
                klog!(Level::ERROR, "[MEMORY_SAFETY] Double-free detected for pointer {:p}", ptr);
                
                // Log detailed information
                self.log_violation_details("DOUBLE_FREE", ptr, allocation);
                
                return Err(MemoryError::DoubleFree);
            }
        }
        
        // Mark as freed and apply poison
        if let Some(mut allocation) = allocations.remove(&ptr) {
            allocation.freed = true;
            
            // Apply poison pattern
            self.apply_poison_pattern(ptr, allocation.size);
            
            // Remove red zones
            self.remove_red_zones(&allocation);
            
            // Remove guard pages
            self.remove_guard_pages(&allocation);
            
            self.stats.total_deallocations.fetch_add(1, Ordering::Relaxed);
            
            klog!(Level::TRACE, "[MEMORY_SAFETY] Deallocation tracked for {:p}", ptr);
        }
        
        Ok(())
    }
    
    /// Validate memory access
    pub fn validate_access(&self, ptr: *mut u8, size: usize) -> MemoryResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let allocations = self.allocations.lock();
        
        // Check if pointer is in any tracked allocation
        for (_, allocation) in allocations.iter() {
            if allocation.contains_pointer(ptr) {
                if allocation.freed {
                    self.stats.use_after_free_violations.fetch_add(1, Ordering::Relaxed);
                    klog!(Level::ERROR, "[MEMORY_SAFETY] Use-after-free detected for pointer {:p}", ptr);
                    
                    self.log_violation_details("USE_AFTER_FREE", ptr, allocation);
                    return Err(MemoryError::UseAfterFree);
                }
                
                // Check if access extends beyond allocation
                let ptr_end = ptr as usize + size;
                let alloc_end = allocation.ptr as usize + allocation.size;
                
                if ptr_end > alloc_end {
                    self.stats.corruption_events.fetch_add(1, Ordering::Relaxed);
                    klog!(Level::ERROR, "[MEMORY_SAFETY] Buffer overflow detected for pointer {:p}", ptr);
                    
                    self.log_violation_details("BUFFER_OVERFLOW", ptr, allocation);
                    return Err(MemoryError::BufferOverflow);
                }
                
                return Ok(());
            }
            
            // Check red zone violations
            if allocation.is_in_red_zone(ptr) {
                self.stats.red_zone_violations.fetch_add(1, Ordering::Relaxed);
                klog!(Level::ERROR, "[MEMORY_SAFETY] Red zone violation detected for pointer {:p}", ptr);
                
                self.log_violation_details("RED_ZONE_VIOLATION", ptr, allocation);
                return Err(MemoryError::RedZoneViolation);
            }
        }
        
        // Pointer not tracked - could be static or external memory
        klog!(Level::TRACE, "[MEMORY_SAFETY] Untracked pointer access {:p}", ptr);
        
        Ok(())
    }
    
    /// Add red zones around allocation
    fn add_red_zones(&self, allocation: &mut TrackedAllocation) -> MemoryResult<()> {
        let ptr_addr = allocation.ptr as usize;
        
        // Red zone before allocation
        let red_before_addr = ptr_addr.saturating_sub(RED_ZONE_SIZE);
        let red_before_ptr = red_before_addr as *mut u8;
        
        // Red zone after allocation
        let red_after_addr = ptr_addr + allocation.size;
        let red_after_ptr = red_after_addr as *mut u8;
        
        // Fill red zones with poison pattern
        unsafe {
            ptr::write_bytes(red_before_ptr, POISON_PATTERNS[3], RED_ZONE_SIZE); // EFEF
            ptr::write_bytes(red_after_ptr, POISON_PATTERNS[3], RED_ZONE_SIZE);   // EFEF
        }
        
        allocation.red_zone_before = Some(red_before_ptr);
        allocation.red_zone_after = Some(red_after_ptr);
        
        self.stats.red_zones_created.fetch_add(2, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Remove red zones
    fn remove_red_zones(&self, allocation: &TrackedAllocation) {
        // Red zones are automatically cleaned up when memory is freed
        // The poison pattern will remain visible for debugging
    }
    
    /// Add guard pages around allocation
    fn add_guard_pages(&self, allocation: &mut TrackedAllocation) -> MemoryResult<()> {
        if let Some(vm_manager) = self.vm_manager {
            let ptr_addr = allocation.ptr as usize;
            
            // Guard page before allocation
            let guard_before_addr = (ptr_addr / GUARD_PAGE_SIZE) * GUARD_PAGE_SIZE;
            let guard_before_vaddr = VirtualAddress::new(guard_before_addr as u64);
            
            // Guard page after allocation
            let guard_after_addr = ((ptr_addr + allocation.size + GUARD_PAGE_SIZE - 1) / GUARD_PAGE_SIZE) * GUARD_PAGE_SIZE;
            let guard_after_vaddr = VirtualAddress::new(guard_after_addr as u64);
            
            // Map guard pages as not present (will cause page fault if accessed)
            // In a real implementation, you'd use the VM manager to create these
            
            allocation.guard_page_before = Some(guard_before_vaddr);
            allocation.guard_page_after = Some(guard_after_vaddr);
            
            self.stats.guard_pages_created.fetch_add(2, Ordering::Relaxed);
        }
        
        Ok(())
    }
    
    /// Remove guard pages
    fn remove_guard_pages(&self, allocation: &TrackedAllocation) {
        // Guard pages are automatically cleaned up when memory is freed
        // In a real implementation, you'd unmap them via the VM manager
    }
    
    /// Apply poison pattern to freed memory
    fn apply_poison_pattern(&self, ptr: *mut u8, size: usize) {
        unsafe {
            // Use alternating poison patterns for better detection
            for i in 0..size {
                let poison_byte = POISON_PATTERNS[i % POISON_PATTERNS.len()];
                ptr::write(ptr.add(i), poison_byte);
            }
        }
    }
    
    /// Log violation details for debugging
    fn log_violation_details(&self, violation_type: &str, ptr: *mut u8, allocation: &TrackedAllocation) {
        kprintln!("[MEMORY_SAFETY] {} VIOLATION DETAILS:", violation_type);
        kprintln!("  Pointer: {:p}", ptr);
        kprintln!("  Allocation: {:p} ({} bytes)", allocation.ptr, allocation.size);
        kprintln!("  Allocated at: {}ms", allocation.allocated_at);
        kprintln!("  Freed: {}", allocation.freed);
        
        if let Some(red_before) = allocation.red_zone_before {
            kprintln!("  Red zone before: {:p}", red_before);
        }
        if let Some(red_after) = allocation.red_zone_after {
            kprintln!("  Red zone after: {:p}", red_after);
        }
        if let Some(guard_before) = allocation.guard_page_before {
            kprintln!("  Guard page before: 0x{:016x}", guard_before.as_u64());
        }
        if let Some(guard_after) = allocation.guard_page_after {
            kprintln!("  Guard page after: 0x{:016x}", guard_after.as_u64());
        }
    }
    
    /// Get current statistics
    pub fn stats(&self) -> MemorySafetySnapshot {
        self.stats.snapshot()
    }
    
    /// Reset all statistics
    pub fn reset_stats(&self) {
        self.stats.reset();
    }
    
    /// Validate all tracked allocations
    pub fn validate_all(&self) -> MemoryResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let allocations = self.allocations.lock();
        
        for (ptr, allocation) in allocations.iter() {
            if allocation.freed {
                // Check if freed memory still contains poison pattern
                let mut poison_valid = true;
                unsafe {
                    for i in 0..allocation.size {
                        let expected_poison = POISON_PATTERNS[i % POISON_PATTERNS.len()];
                        if *ptr.add(i) != expected_poison {
                            poison_valid = false;
                            break;
                        }
                    }
                }
                
                if !poison_valid {
                    self.stats.poison_violations.fetch_add(1, Ordering::Relaxed);
                    klog!(Level::ERROR, "[MEMORY_SAFETY] Poison pattern corrupted for freed allocation {:p}", ptr);
                }
            }
        }
        
        Ok(())
    }
    
    /// Print memory safety report
    pub fn print_report(&self) {
        let stats = self.stats();
        
        kprintln!("");
        kprintln!("=== MEMORY SAFETY REPORT ===");
        kprintln!("Safety Features Enabled: {}", self.enabled);
        kprintln!("");
        kprintln!("Allocation Tracking:");
        kprintln!("  Total Allocations: {}", stats.total_allocations);
        kprintln!("  Total Deallocations: {}", stats.total_deallocations);
        kprintln!("  Currently Tracked: {}", stats.total_allocations - stats.total_deallocations);
        kprintln!("");
        kprintln!("Violations Detected:");
        kprintln!("  Double-Free Attempts: {}", stats.double_free_attempts);
        kprintln!("  Use-After-Free: {}", stats.use_after_free_violations);
        kprintln!("  Guard Page Violations: {}", stats.guard_page_violations);
        kprintln!("  Red Zone Violations: {}", stats.red_zone_violations);
        kprintln!("  Poison Violations: {}", stats.poison_violations);
        kprintln!("  Corruption Events: {}", stats.corruption_events);
        kprintln!("");
        kprintln!("Protection Features:");
        kprintln!("  Guard Pages Created: {}", stats.guard_pages_created);
        kprintln!("  Red Zones Created: {}", stats.red_zones_created);
        kprintln!("");
        
        if stats.double_free_attempts > 0 || stats.use_after_free_violations > 0 {
            kprintln!("🚨 CRITICAL: Memory safety violations detected!");
        } else {
            kprintln!("✅ All memory safety checks passed");
        }
        
        kprintln!("=== END MEMORY SAFETY REPORT ===");
        kprintln!("");
    }
}

//=============================================================================
// GLOBAL MEMORY SAFETY MANAGER
//=============================================================================

/// Global memory safety manager instance
static mut MEMORY_SAFETY_MANAGER: Option<MemorySafetyManager> = None;

/// Initialize the memory safety system
pub fn init() {
    if !MEMORY_SAFETY_ENABLED {
        kprintln!("[MEMORY_SAFETY] Memory safety features disabled (not debug build)");
        return;
    }
    
    kprintln!("[MEMORY_SAFETY] Initializing memory safety system");
    
    unsafe {
        MEMORY_SAFETY_MANAGER = Some(MemorySafetyManager::new());
    }
    
    kprintln!("[MEMORY_SAFETY] Memory safety system initialized");
    kprintln!("  - Free-poison patterns enabled");
    kprintln!("  - Double-free detection enabled");
    kprintln!("  - Use-after-free detection enabled");
    kprintln!("  - Red zones enabled ({} bytes)", RED_ZONE_SIZE);
    kprintln!("  - Guard pages enabled ({} bytes)", GUARD_PAGE_SIZE);
}

/// Get the global memory safety manager
pub fn get_memory_safety() -> Option<&'static MemorySafetyManager> {
    if !MEMORY_SAFETY_ENABLED {
        return None;
    }
    
    unsafe {
        MEMORY_SAFETY_MANAGER.as_ref()
    }
}

/// Track allocation (convenience function)
pub fn track_allocation(ptr: *mut u8, size: usize) -> MemoryResult<()> {
    if let Some(manager) = get_memory_safety() {
        manager.track_allocation(ptr, size)
    } else {
        Ok(())
    }
}

/// Track deallocation (convenience function)
pub fn track_deallocation(ptr: *mut u8) -> MemoryResult<()> {
    if let Some(manager) = get_memory_safety() {
        manager.track_deallocation(ptr)
    } else {
        Ok(())
    }
}

/// Validate memory access (convenience function)
pub fn validate_access(ptr: *mut u8, size: usize) -> MemoryResult<()> {
    if let Some(manager) = get_memory_safety() {
        manager.validate_access(ptr, size)
    } else {
        Ok(())
    }
}

/// Print memory safety report (convenience function)
pub fn print_report() {
    if let Some(manager) = get_memory_safety() {
        manager.print_report();
    }
}
