//! Huge Page Support for Polymera OS Memory Manager
//!
//! This module implements transparent huge page (THP) support for allocations
//! larger than 2MB. Huge pages reduce TLB misses for large memory allocations.
//!
//! Requirements: 1.3 - Transparent huge page promotion for allocations > 2MB

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

use super::{PhysicalAddress, VirtualAddress};

/// Standard page size (4KB)
pub const STANDARD_PAGE_SIZE: usize = 4096;

/// Huge page size (2MB)
pub const HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Threshold for huge page promotion (allocations >= 2MB automatically use huge pages)
pub const HUGE_PAGE_THRESHOLD: usize = HUGE_PAGE_SIZE;

/// Maximum huge pages in the pool
pub const MAX_HUGE_PAGES: usize = 256;

/// Alignment requirement for huge pages (2MB aligned)
pub const HUGE_PAGE_ALIGNMENT: usize = HUGE_PAGE_SIZE;

/// Huge page statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct HugePageStats {
    /// Total huge pages available
    pub total_pages: usize,
    /// Huge pages currently in use
    pub used_pages: usize,
    /// Huge pages in the free pool
    pub free_pages: usize,
    /// Total promotions to huge pages
    pub promotions: u64,
    /// Failed promotion attempts
    pub promotion_failures: u64,
    /// Demotions from huge pages
    pub demotions: u64,
    /// TLB entries saved (estimated)
    pub tlb_entries_saved: u64,
}

/// A huge page entry in the pool
#[derive(Debug, Clone, Copy)]
pub struct HugePage {
    /// Physical address of the huge page (2MB aligned)
    pub phys_addr: PhysicalAddress,
    /// Virtual address if mapped
    pub virt_addr: Option<VirtualAddress>,
    /// Owner process ID
    pub owner_pid: Option<u64>,
    /// Whether the page is in use
    pub in_use: bool,
    /// Reference count for shared mappings
    pub ref_count: usize,
}

impl HugePage {
    /// Create a new huge page entry
    pub const fn new(phys_addr: PhysicalAddress) -> Self {
        Self {
            phys_addr,
            virt_addr: None,
            owner_pid: None,
            in_use: false,
            ref_count: 0,
        }
    }
}

/// Huge Page Pool Manager
///
/// Manages a pool of 2MB huge pages for large allocations.
/// Implements automatic promotion for allocations > 2MB.
pub struct HugePagePool {
    /// Pool of huge pages indexed by physical address
    pages: BTreeMap<PhysicalAddress, HugePage>,
    /// Free huge page addresses (stack for fast allocation)
    free_stack: Vec<PhysicalAddress>,
    /// Statistics
    stats: HugePageStats,
    /// Next physical address to allocate from
    next_phys_addr: PhysicalAddress,
    /// Whether the pool is initialized
    initialized: bool,
}

impl HugePagePool {
    /// Create a new huge page pool
    pub const fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
            free_stack: Vec::new(),
            stats: HugePageStats {
                total_pages: 0,
                used_pages: 0,
                free_pages: 0,
                promotions: 0,
                promotion_failures: 0,
                demotions: 0,
                tlb_entries_saved: 0,
            },
            next_phys_addr: 0,
            initialized: false,
        }
    }

    /// Initialize the huge page pool with a reserved memory region
    ///
    /// # Arguments
    /// * `base_addr` - Base physical address for the huge page pool (must be 2MB aligned)
    /// * `size` - Total size of the memory region for huge pages
    ///
    /// # Returns
    /// Number of huge pages initialized
    pub fn init(&mut self, base_addr: PhysicalAddress, size: usize) -> usize {
        if self.initialized {
            return 0;
        }

        // Ensure base address is 2MB aligned
        let aligned_base = (base_addr + HUGE_PAGE_ALIGNMENT as u64 - 1) & !(HUGE_PAGE_ALIGNMENT as u64 - 1);
        let aligned_end = (base_addr + size as u64) & !(HUGE_PAGE_ALIGNMENT as u64 - 1);
        
        if aligned_end <= aligned_base {
            return 0;
        }

        let num_pages = ((aligned_end - aligned_base) as usize / HUGE_PAGE_SIZE).min(MAX_HUGE_PAGES);
        
        for i in 0..num_pages {
            let phys_addr = aligned_base + (i * HUGE_PAGE_SIZE) as u64;
            let page = HugePage::new(phys_addr);
            self.pages.insert(phys_addr, page);
            self.free_stack.push(phys_addr);
        }

        self.stats.total_pages = num_pages;
        self.stats.free_pages = num_pages;
        self.next_phys_addr = aligned_base + (num_pages * HUGE_PAGE_SIZE) as u64;
        self.initialized = true;

        num_pages
    }

    /// Allocate a huge page for a process
    ///
    /// **Property 2: Huge Page Promotion**
    /// For any memory allocation request larger than 2MB, this shall allocate
    /// from the huge page pool rather than standard 4KB pages.
    ///
    /// # Arguments
    /// * `size` - Requested allocation size (must be >= 2MB)
    /// * `pid` - Process ID requesting the allocation
    ///
    /// # Returns
    /// * `Ok((phys_addr, virt_addr))` on success
    /// * `Err(HugePageError)` on failure
    pub fn allocate(&mut self, size: usize, pid: u64) -> Result<PhysicalAddress, HugePageError> {
        if size < HUGE_PAGE_THRESHOLD {
            return Err(HugePageError::SizeTooSmall);
        }

        if !self.initialized {
            return Err(HugePageError::NotInitialized);
        }

        let phys_addr = self.free_stack.pop()
            .ok_or(HugePageError::NoFreePages)?;

        if let Some(page) = self.pages.get_mut(&phys_addr) {
            page.in_use = true;
            page.owner_pid = Some(pid);
            page.ref_count = 1;
        }

        self.stats.used_pages += 1;
        self.stats.free_pages = self.stats.free_pages.saturating_sub(1);
        self.stats.promotions += 1;
        // Each huge page saves 511 TLB entries (512 small pages - 1 huge page entry)
        self.stats.tlb_entries_saved += 511;

        Ok(phys_addr)
    }

    /// Free a huge page
    ///
    /// # Arguments
    /// * `phys_addr` - Physical address of the huge page to free
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(HugePageError)` on failure
    pub fn free(&mut self, phys_addr: PhysicalAddress) -> Result<(), HugePageError> {
        let page = self.pages.get_mut(&phys_addr)
            .ok_or(HugePageError::PageNotFound)?;

        if !page.in_use {
            return Err(HugePageError::PageNotAllocated);
        }

        page.ref_count = page.ref_count.saturating_sub(1);
        
        if page.ref_count == 0 {
            page.in_use = false;
            page.owner_pid = None;
            page.virt_addr = None;
            
            self.free_stack.push(phys_addr);
            self.stats.used_pages = self.stats.used_pages.saturating_sub(1);
            self.stats.free_pages += 1;
            self.stats.demotions += 1;
        }

        Ok(())
    }

    /// Check if an allocation should use huge pages
    ///
    /// # Arguments
    /// * `size` - Requested allocation size
    ///
    /// # Returns
    /// true if the allocation should use huge pages
    pub fn should_use_huge_page(size: usize) -> bool {
        size >= HUGE_PAGE_THRESHOLD
    }

    /// Map a huge page to a virtual address
    pub fn map(&mut self, phys_addr: PhysicalAddress, virt_addr: VirtualAddress) -> Result<(), HugePageError> {
        let page = self.pages.get_mut(&phys_addr)
            .ok_or(HugePageError::PageNotFound)?;

        if !page.in_use {
            return Err(HugePageError::PageNotAllocated);
        }

        page.virt_addr = Some(virt_addr);
        Ok(())
    }

    /// Get the physical address from a virtual address in a huge page
    pub fn translate(&self, virt_addr: VirtualAddress) -> Option<PhysicalAddress> {
        for (phys, page) in &self.pages {
            if let Some(va) = page.virt_addr {
                if virt_addr >= va && virt_addr < va + HUGE_PAGE_SIZE as u64 {
                    let offset = virt_addr - va;
                    return Some(phys + offset);
                }
            }
        }
        None
    }

    /// Check if an address is within a huge page
    pub fn is_huge_page(&self, phys_addr: PhysicalAddress) -> bool {
        let aligned = phys_addr & !(HUGE_PAGE_SIZE as u64 - 1);
        self.pages.contains_key(&aligned)
    }

    /// Get statistics
    pub fn stats(&self) -> HugePageStats {
        self.stats
    }

    /// Get the number of free huge pages
    pub fn free_count(&self) -> usize {
        self.stats.free_pages
    }

    /// Get the number of used huge pages
    pub fn used_count(&self) -> usize {
        self.stats.used_pages
    }

    /// Clear all huge pages owned by a process
    pub fn clear_process_pages(&mut self, pid: u64) {
        let to_free: Vec<_> = self.pages
            .iter()
            .filter(|(_, page)| page.owner_pid == Some(pid))
            .map(|(addr, _)| *addr)
            .collect();

        for addr in to_free {
            let _ = self.free(addr);
        }
    }
}

/// Huge page error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HugePageError {
    /// Pool not initialized
    NotInitialized,
    /// Allocation size too small for huge page
    SizeTooSmall,
    /// No free huge pages available
    NoFreePages,
    /// Page not found in pool
    PageNotFound,
    /// Page not allocated
    PageNotAllocated,
    /// Invalid alignment
    InvalidAlignment,
    /// Memory region too small
    RegionTooSmall,
}

impl core::fmt::Display for HugePageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "Huge page pool not initialized"),
            Self::SizeTooSmall => write!(f, "Allocation size too small for huge page"),
            Self::NoFreePages => write!(f, "No free huge pages available"),
            Self::PageNotFound => write!(f, "Huge page not found in pool"),
            Self::PageNotAllocated => write!(f, "Huge page not allocated"),
            Self::InvalidAlignment => write!(f, "Invalid alignment for huge page"),
            Self::RegionTooSmall => write!(f, "Memory region too small for huge pages"),
        }
    }
}

/// Global huge page pool
pub static HUGE_PAGE_POOL: Mutex<HugePagePool> = Mutex::new(HugePagePool::new());

/// Initialize the global huge page pool
pub fn init_huge_pages(base_addr: PhysicalAddress, size: usize) -> usize {
    HUGE_PAGE_POOL.lock().init(base_addr, size)
}

/// Allocate memory, automatically using huge pages for large allocations
///
/// **Property 2: Huge Page Promotion**
/// This function ensures allocations > 2MB use huge pages.
pub fn allocate_with_promotion(size: usize, pid: u64) -> Result<PhysicalAddress, HugePageError> {
    if HugePagePool::should_use_huge_page(size) {
        HUGE_PAGE_POOL.lock().allocate(size, pid)
    } else {
        // Fall back to regular allocation (handled by main allocator)
        Err(HugePageError::SizeTooSmall)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_use_huge_page() {
        assert!(!HugePagePool::should_use_huge_page(1024));
        assert!(!HugePagePool::should_use_huge_page(4096));
        assert!(!HugePagePool::should_use_huge_page(1024 * 1024));
        assert!(HugePagePool::should_use_huge_page(2 * 1024 * 1024));
        assert!(HugePagePool::should_use_huge_page(4 * 1024 * 1024));
    }

    #[test]
    fn test_huge_page_pool_init() {
        let mut pool = HugePagePool::new();
        
        // Initialize with 16MB (8 huge pages)
        let base = 0x20_0000; // 2MB aligned
        let size = 16 * 1024 * 1024;
        let count = pool.init(base, size);
        
        assert_eq!(count, 8);
        assert_eq!(pool.stats().total_pages, 8);
        assert_eq!(pool.stats().free_pages, 8);
    }

    #[test]
    fn test_huge_page_allocate_free() {
        let mut pool = HugePagePool::new();
        let base = 0x20_0000;
        pool.init(base, 16 * 1024 * 1024);

        // Allocate
        let result = pool.allocate(HUGE_PAGE_SIZE, 1);
        assert!(result.is_ok());
        assert_eq!(pool.stats().used_pages, 1);
        
        // Free
        let phys = result.unwrap();
        let free_result = pool.free(phys);
        assert!(free_result.is_ok());
        assert_eq!(pool.stats().used_pages, 0);
    }
}
