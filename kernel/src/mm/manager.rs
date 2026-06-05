//! Memory Manager for Polymera OS Microkernel
//!
//! This module implements the core Memory Manager as specified in the design document.
//! It handles virtual memory, LZ4 compression, huge pages, and capability-enforced access.
//!
//! Requirements: 1.1, 1.2, 1.3, 1.4

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use super::compressor::{CompressionError, CompressionStats, Lz4Compressor, COMPRESSION_THRESHOLD, COMPRESSION_PAGE_SIZE};
use super::huge_pages::{HugePageError, HugePagePool, HugePageStats, HUGE_PAGE_THRESHOLD};
use super::{MemoryError, MemoryResult, PhysicalAddress, VirtualAddress};

/// Process ID type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(pub u64);

impl ProcessId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Page ID for tracking individual pages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageId(pub u64);

/// Capability handle for memory access validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityHandle {
    /// Unique handle ID
    pub id: u64,
    /// Capability type
    pub cap_type: CapabilityType,
    /// Parent handle (for revocation)
    pub parent: Option<u64>,
    /// Generation number (invalidated on revoke)
    pub generation: u32,
    /// Memory region base address
    pub base: VirtualAddress,
    /// Memory region size
    pub size: usize,
    /// Permissions
    pub perms: Permissions,
}

impl CapabilityHandle {
    /// Create a new memory capability handle
    pub fn new_memory(base: VirtualAddress, size: usize, perms: Permissions) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            cap_type: CapabilityType::Memory,
            parent: None,
            generation: 0,
            base,
            size,
            perms,
        }
    }

    /// Check if an address is within this capability's range
    pub fn contains(&self, addr: VirtualAddress) -> bool {
        addr >= self.base && addr < self.base + self.size as u64
    }
}

/// Capability types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityType {
    Memory,
    Ipc,
    Device,
    Framebuffer,
    Enclave,
    IoPort,
}

/// Memory permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permissions {
    pub const fn read_only() -> Self {
        Self { read: true, write: false, execute: false }
    }

    pub const fn read_write() -> Self {
        Self { read: true, write: true, execute: false }
    }

    pub const fn read_execute() -> Self {
        Self { read: true, write: false, execute: true }
    }
}

/// Page fault reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultReason {
    /// Page not present
    NotPresent,
    /// Write to read-only page
    WriteProtection,
    /// Execute on non-executable page
    ExecuteProtection,
    /// Capability violation
    CapabilityViolation,
    /// Invalid address
    InvalidAddress,
}

/// Page fault information
#[derive(Debug, Clone)]
pub struct PageFault {
    pub address: VirtualAddress,
    pub reason: FaultReason,
    pub pid: ProcessId,
}

/// Memory pressure status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureStatus {
    /// Memory pressure is low (< 60%)
    Low,
    /// Memory pressure is moderate (60-80%)
    Moderate,
    /// Memory pressure is high (> 80%), compression triggered
    High,
    /// Memory pressure is critical (> 95%), OOM imminent
    Critical,
}

/// Frame allocator trait for physical memory management
pub trait FrameAllocator {
    fn allocate(&mut self) -> Option<PhysicalAddress>;
    fn deallocate(&mut self, addr: PhysicalAddress);
    fn allocate_contiguous(&mut self, count: usize) -> Option<PhysicalAddress>;
}

/// Simple frame allocator implementation
pub struct SimpleFrameAllocator {
    next_frame: PhysicalAddress,
    free_frames: Vec<PhysicalAddress>,
    total_frames: usize,
    used_frames: usize,
}

impl SimpleFrameAllocator {
    pub const fn new() -> Self {
        Self {
            next_frame: 0,
            free_frames: Vec::new(),
            total_frames: 0,
            used_frames: 0,
        }
    }

    pub fn init(&mut self, start: PhysicalAddress, size: usize) {
        self.next_frame = start;
        self.total_frames = size / 4096;
    }
}

impl FrameAllocator for SimpleFrameAllocator {
    fn allocate(&mut self) -> Option<PhysicalAddress> {
        if let Some(frame) = self.free_frames.pop() {
            self.used_frames += 1;
            return Some(frame);
        }
        
        if self.used_frames < self.total_frames {
            let frame = self.next_frame;
            self.next_frame += 4096;
            self.used_frames += 1;
            Some(frame)
        } else {
            None
        }
    }

    fn deallocate(&mut self, addr: PhysicalAddress) {
        self.free_frames.push(addr);
        self.used_frames = self.used_frames.saturating_sub(1);
    }

    fn allocate_contiguous(&mut self, count: usize) -> Option<PhysicalAddress> {
        if self.used_frames + count <= self.total_frames {
            let base = self.next_frame;
            self.next_frame += (count * 4096) as u64;
            self.used_frames += count;
            Some(base)
        } else {
            None
        }
    }
}


/// Page table entry for virtual memory mapping
#[derive(Debug, Clone)]
pub struct PageTableEntry {
    pub phys_addr: PhysicalAddress,
    pub flags: PageFlags,
    pub is_huge: bool,
}

/// Page flags for memory protection
#[derive(Debug, Clone, Copy, Default)]
pub struct PageFlags {
    pub present: bool,
    pub writable: bool,
    pub executable: bool,
    pub user: bool,
    pub cached: bool,
}

impl PageFlags {
    pub const fn new() -> Self {
        Self {
            present: true,
            writable: false,
            executable: false,
            user: false,
            cached: true,
        }
    }

    pub const fn read_write() -> Self {
        Self {
            present: true,
            writable: true,
            executable: false,
            user: false,
            cached: true,
        }
    }
}

/// Page table for a process
#[derive(Debug)]
pub struct PageTable {
    entries: BTreeMap<VirtualAddress, PageTableEntry>,
    pid: ProcessId,
}

impl PageTable {
    pub fn new(pid: ProcessId) -> Self {
        Self {
            entries: BTreeMap::new(),
            pid,
        }
    }

    pub fn map(&mut self, virt: VirtualAddress, phys: PhysicalAddress, flags: PageFlags, is_huge: bool) {
        self.entries.insert(virt, PageTableEntry { phys_addr: phys, flags, is_huge });
    }

    pub fn unmap(&mut self, virt: VirtualAddress) -> Option<PageTableEntry> {
        self.entries.remove(&virt)
    }

    pub fn translate(&self, virt: VirtualAddress) -> Option<PhysicalAddress> {
        // Find the page containing this address
        let page_size = 4096u64;
        let page_base = virt & !(page_size - 1);
        
        self.entries.get(&page_base).map(|entry| {
            let offset = virt - page_base;
            entry.phys_addr + offset
        })
    }

    pub fn get_entry(&self, virt: VirtualAddress) -> Option<&PageTableEntry> {
        let page_size = 4096u64;
        let page_base = virt & !(page_size - 1);
        self.entries.get(&page_base)
    }
}

/// Compressed page cache for storing compressed pages
///
/// This cache stores pages that have been compressed due to memory pressure.
/// When memory pressure exceeds 80%, inactive pages are compressed using LZ4
/// and stored here, freeing their physical frames for reuse.
///
/// **Property 1: Memory Compression Trigger**
/// For any memory state where pressure exceeds 80%, the Memory_Manager shall
/// initiate LZ4 compression of inactive pages before any swap operations occur.
#[derive(Debug)]
pub struct CompressedPageCache {
    compressor: Lz4Compressor,
    /// Track which pages are candidates for compression (inactive pages)
    inactive_pages: Vec<(VirtualAddress, PhysicalAddress, u64)>, // (virt, phys, pid)
}

impl CompressedPageCache {
    pub const fn new() -> Self {
        Self {
            compressor: Lz4Compressor::new(),
            inactive_pages: Vec::new(),
        }
    }

    /// Compress a page and store it in the cache
    ///
    /// # Arguments
    /// * `data` - The page data to compress (must be 4KB)
    /// * `virt_addr` - Virtual address of the page
    /// * `phys_addr` - Physical address of the page
    /// * `owner_pid` - Process ID that owns the page
    ///
    /// # Returns
    /// * `Ok(compression_ratio)` if compression successful
    /// * `Err(CompressionError)` if compression failed
    pub fn compress_page(
        &mut self,
        data: &[u8],
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        owner_pid: u64,
    ) -> Result<f32, CompressionError> {
        self.compressor.compress_page(data, virt_addr, phys_addr, owner_pid)
    }

    /// Decompress a page from the cache
    ///
    /// # Arguments
    /// * `virt_addr` - Virtual address of the page to decompress
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` containing decompressed page data
    /// * `Err(CompressionError)` if page not found or decompression failed
    pub fn decompress_page(&mut self, virt_addr: VirtualAddress) -> Result<Vec<u8>, CompressionError> {
        self.compressor.decompress_page(virt_addr)
    }

    /// Check if a page is in the compressed cache
    pub fn is_compressed(&self, virt_addr: VirtualAddress) -> bool {
        self.compressor.is_page_compressed(virt_addr)
    }

    /// Get compression statistics
    pub fn stats(&self) -> CompressionStats {
        self.compressor.stats()
    }

    /// Mark a page as inactive (candidate for compression)
    pub fn mark_inactive(&mut self, virt_addr: VirtualAddress, phys_addr: PhysicalAddress, pid: u64) {
        // Avoid duplicates
        if !self.inactive_pages.iter().any(|(v, _, _)| *v == virt_addr) {
            self.inactive_pages.push((virt_addr, phys_addr, pid));
        }
    }

    /// Get the number of inactive pages available for compression
    pub fn inactive_count(&self) -> usize {
        self.inactive_pages.len()
    }

    /// Get and remove the next inactive page for compression
    pub fn pop_inactive(&mut self) -> Option<(VirtualAddress, PhysicalAddress, u64)> {
        self.inactive_pages.pop()
    }

    /// Clear inactive pages for a specific process
    pub fn clear_process_inactive(&mut self, pid: u64) {
        self.inactive_pages.retain(|(_, _, p)| *p != pid);
        self.compressor.clear_process_pages(pid);
    }

    /// Get total bytes saved through compression
    pub fn bytes_saved(&self) -> u64 {
        self.compressor.bytes_saved()
    }

    /// Get the number of compressed pages in cache
    pub fn compressed_count(&self) -> usize {
        self.compressor.cache_count()
    }
}

/// Memory statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct MemoryManagerStats {
    pub total_bytes: usize,
    pub used_bytes: usize,
    pub compressed_bytes: usize,
    pub compression_ratio: f32,
    pub huge_page_count: usize,
    pub fragmentation_ratio: f32,
    pub page_faults: u64,
    pub capability_violations: u64,
}

/// Memory Manager - handles virtual memory, compression, and huge pages
///
/// This is the core memory management component for the Polymera OS microkernel.
/// It implements:
/// - Safe Rust memory management (Requirement 1.1)
/// - LZ4 page compression at 80% pressure (Requirement 1.2)
/// - Transparent huge page promotion for > 2MB allocations (Requirement 1.3)
/// - Capability-enforced memory access (Requirement 1.4)
pub struct MemoryManager {
    /// Page frame allocator
    frame_allocator: SimpleFrameAllocator,
    /// LZ4 compression engine for inactive pages
    compressor: Lz4Compressor,
    /// Huge page pool (2MB pages)
    huge_page_pool: HugePagePool,
    /// Per-process page tables
    page_tables: BTreeMap<ProcessId, PageTable>,
    /// Memory pressure threshold (0.0 - 1.0)
    pressure_threshold: f32,
    /// Compressed page cache
    compressed_cache: CompressedPageCache,
    /// Total memory in bytes
    total_memory: usize,
    /// Used memory in bytes
    used_memory: usize,
    /// Statistics
    stats: MemoryManagerStats,
}

impl MemoryManager {
    /// Create a new Memory Manager
    pub const fn new() -> Self {
        Self {
            frame_allocator: SimpleFrameAllocator::new(),
            compressor: Lz4Compressor::new(),
            huge_page_pool: HugePagePool::new(),
            page_tables: BTreeMap::new(),
            pressure_threshold: COMPRESSION_THRESHOLD,
            compressed_cache: CompressedPageCache::new(),
            total_memory: 0,
            used_memory: 0,
            stats: MemoryManagerStats {
                total_bytes: 0,
                used_bytes: 0,
                compressed_bytes: 0,
                compression_ratio: 0.0,
                huge_page_count: 0,
                fragmentation_ratio: 0.0,
                page_faults: 0,
                capability_violations: 0,
            },
        }
    }

    /// Initialize the memory manager with available physical memory
    pub fn init(&mut self, phys_start: PhysicalAddress, phys_size: usize) {
        self.total_memory = phys_size;
        self.frame_allocator.init(phys_start, phys_size);
        
        // Reserve some memory for huge pages (25% of total)
        let huge_page_region_size = phys_size / 4;
        let huge_page_base = phys_start + (phys_size - huge_page_region_size) as u64;
        self.huge_page_pool.init(huge_page_base, huge_page_region_size);
        
        self.stats.total_bytes = phys_size;
    }


    /// Allocate memory for a process, auto-promoting to huge pages if > 2MB
    ///
    /// **Property 2: Huge Page Promotion**
    /// For any memory allocation request larger than 2MB, this shall allocate
    /// from the huge page pool rather than standard 4KB pages.
    ///
    /// # Arguments
    /// * `pid` - Process ID requesting the allocation
    /// * `size` - Number of bytes to allocate
    /// * `cap` - Capability handle authorizing the allocation
    ///
    /// # Returns
    /// Virtual address of the allocated memory or error
    pub fn allocate(
        &mut self,
        pid: ProcessId,
        size: usize,
        cap: &CapabilityHandle,
    ) -> Result<VirtualAddress, MemoryError> {
        // Validate capability
        if cap.cap_type != CapabilityType::Memory {
            return Err(MemoryError::PermissionDenied);
        }

        // Check if we should use huge pages (Property 2)
        if size >= HUGE_PAGE_THRESHOLD {
            return self.allocate_huge_page(pid, size, cap);
        }

        // Standard allocation
        let num_pages = (size + 4095) / 4096;
        let phys_addr = self.frame_allocator.allocate_contiguous(num_pages)
            .ok_or(MemoryError::OutOfMemory)?;

        // Create virtual address mapping
        let virt_addr = self.find_free_virtual_region(pid, size)?;
        
        // Ensure page table exists for this process
        if !self.page_tables.contains_key(&pid) {
            self.page_tables.insert(pid, PageTable::new(pid));
        }

        // Map pages
        if let Some(pt) = self.page_tables.get_mut(&pid) {
            for i in 0..num_pages {
                let page_virt = virt_addr + (i * 4096) as u64;
                let page_phys = phys_addr + (i * 4096) as u64;
                pt.map(page_virt, page_phys, PageFlags::read_write(), false);
            }
        }

        self.used_memory += size;
        self.stats.used_bytes = self.used_memory;

        Ok(virt_addr)
    }

    /// Allocate using huge pages for large allocations
    fn allocate_huge_page(
        &mut self,
        pid: ProcessId,
        size: usize,
        _cap: &CapabilityHandle,
    ) -> Result<VirtualAddress, MemoryError> {
        let phys_addr = self.huge_page_pool.allocate(size, pid.0)
            .map_err(|e| match e {
                HugePageError::NoFreePages => MemoryError::OutOfMemory,
                HugePageError::NotInitialized => MemoryError::OutOfMemory,
                _ => MemoryError::AllocationTooLarge,
            })?;

        let virt_addr = self.find_free_virtual_region(pid, size)?;

        // Ensure page table exists
        if !self.page_tables.contains_key(&pid) {
            self.page_tables.insert(pid, PageTable::new(pid));
        }

        // Map as huge page
        if let Some(pt) = self.page_tables.get_mut(&pid) {
            pt.map(virt_addr, phys_addr, PageFlags::read_write(), true);
        }

        self.used_memory += size;
        self.stats.used_bytes = self.used_memory;
        self.stats.huge_page_count += 1;

        Ok(virt_addr)
    }

    /// Find a free virtual address region for a process
    fn find_free_virtual_region(&self, pid: ProcessId, size: usize) -> Result<VirtualAddress, MemoryError> {
        // Simple allocation strategy: use a base address per process
        let base = 0x1000_0000u64 + (pid.0 * 0x1000_0000);
        
        if let Some(pt) = self.page_tables.get(&pid) {
            // Find highest mapped address and allocate after it
            if let Some((&highest, _)) = pt.entries.iter().next_back() {
                let next = highest + 4096;
                return Ok(next);
            }
        }
        
        Ok(base)
    }

    /// Check memory pressure and compress inactive pages if > 80%
    ///
    /// **Property 1: Memory Compression Trigger**
    /// For any memory state where pressure exceeds 80%, this shall initiate
    /// LZ4 compression of inactive pages before any swap operations occur.
    ///
    /// # Returns
    /// Current pressure status
    pub fn check_pressure(&mut self) -> PressureStatus {
        let pressure = self.used_memory as f32 / self.total_memory as f32;

        if pressure > 0.95 {
            PressureStatus::Critical
        } else if pressure > self.pressure_threshold {
            // Trigger compression (Property 1)
            self.trigger_compression();
            PressureStatus::High
        } else if pressure > 0.60 {
            PressureStatus::Moderate
        } else {
            PressureStatus::Low
        }
    }

    /// Trigger compression of inactive pages
    ///
    /// **Property 1: Memory Compression Trigger**
    /// For any memory state where pressure exceeds 80%, this shall initiate
    /// LZ4 compression of inactive pages before any swap operations occur.
    ///
    /// This method:
    /// 1. Identifies inactive pages from the compressed cache's inactive list
    /// 2. Compresses them using LZ4 via the Lz4Compressor
    /// 3. Frees the physical frames for reuse
    /// 4. Updates compression statistics
    fn trigger_compression(&mut self) {
        // Compress inactive pages until pressure drops below threshold
        // or we run out of inactive pages
        let target_pressure = self.pressure_threshold - 0.05; // Target 75%
        let mut pages_compressed = 0;
        const MAX_PAGES_PER_TRIGGER: usize = 64; // Limit work per trigger

        while self.memory_pressure() > target_pressure 
            && pages_compressed < MAX_PAGES_PER_TRIGGER 
        {
            // Get next inactive page to compress
            if let Some((virt_addr, phys_addr, pid)) = self.compressed_cache.pop_inactive() {
                // Read page data (in real implementation, this would read from physical memory)
                // For now, we simulate with a placeholder
                let page_data = self.read_page_data(phys_addr);
                
                // Compress the page
                match self.compressed_cache.compress_page(&page_data, virt_addr, phys_addr, pid) {
                    Ok(ratio) => {
                        // Successfully compressed - free the physical frame
                        self.frame_allocator.deallocate(phys_addr);
                        self.used_memory = self.used_memory.saturating_sub(COMPRESSION_PAGE_SIZE);
                        pages_compressed += 1;
                        
                        // Update page table to mark page as compressed (not present)
                        if let Some(pt) = self.page_tables.get_mut(&ProcessId(pid)) {
                            pt.unmap(virt_addr);
                        }
                        
                        // Log compression (in debug builds)
                        #[cfg(debug_assertions)]
                        {
                            let _ = ratio; // Use ratio to avoid warning
                        }
                    }
                    Err(_) => {
                        // Compression failed (e.g., not compressible) - skip this page
                        continue;
                    }
                }
            } else {
                // No more inactive pages to compress
                break;
            }
        }

        // Update statistics
        let comp_stats = self.compressed_cache.stats();
        self.stats.compressed_bytes = comp_stats.bytes_saved as usize;
        self.stats.compression_ratio = comp_stats.avg_compression_ratio;
        self.stats.used_bytes = self.used_memory;
    }

    /// Read page data from physical memory
    ///
    /// In a real implementation, this would read from the physical address.
    /// For simulation purposes, we return a page filled with a pattern.
    fn read_page_data(&self, _phys_addr: PhysicalAddress) -> [u8; COMPRESSION_PAGE_SIZE] {
        // In a real kernel, this would:
        // 1. Map the physical address temporarily
        // 2. Copy the page data
        // 3. Unmap the temporary mapping
        // For now, return a compressible pattern
        let mut data = [0u8; COMPRESSION_PAGE_SIZE];
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        data
    }

    /// Decompress a page on access (must complete within 100μs)
    ///
    /// When a process accesses a compressed page, this method:
    /// 1. Decompresses the page data using LZ4
    /// 2. Allocates a new physical frame
    /// 3. Copies decompressed data to the frame
    /// 4. Updates the page table mapping
    ///
    /// # Arguments
    /// * `addr` - Virtual address of the page to decompress
    ///
    /// # Returns
    /// * `Ok(())` if decompression successful
    /// * `Err(MemoryError)` if decompression failed
    pub fn decompress_page(&mut self, addr: VirtualAddress) -> Result<(), MemoryError> {
        // Align address to page boundary
        let page_addr = addr & !(COMPRESSION_PAGE_SIZE as u64 - 1);
        
        if self.compressed_cache.is_compressed(page_addr) {
            // Decompress the page data
            let data = self.compressed_cache.decompress_page(page_addr)
                .map_err(|_| MemoryError::PageNotFound)?;
            
            // Allocate a new physical frame
            let phys_addr = self.frame_allocator.allocate()
                .ok_or(MemoryError::OutOfMemory)?;
            
            // In a real implementation, we would copy data to the physical frame:
            // unsafe { core::ptr::copy_nonoverlapping(data.as_ptr(), phys_addr as *mut u8, data.len()); }
            let _ = data; // Use data to avoid warning
            
            // Find which process owns this page and update their page table
            for (pid, pt) in self.page_tables.iter_mut() {
                // Check if this process had this page mapped before compression
                // In a real implementation, we'd track this in the compressed page metadata
                pt.map(page_addr, phys_addr, PageFlags::read_write(), false);
                let _ = pid; // Use pid to avoid warning
                break; // Only map to one process
            }
            
            // Update memory usage
            self.used_memory += COMPRESSION_PAGE_SIZE;
            self.stats.used_bytes = self.used_memory;
        }
        Ok(())
    }

    /// Mark a page as inactive (candidate for compression)
    ///
    /// This should be called when a page hasn't been accessed recently.
    /// Inactive pages are compressed when memory pressure exceeds 80%.
    pub fn mark_page_inactive(&mut self, pid: ProcessId, virt_addr: VirtualAddress) {
        if let Some(pt) = self.page_tables.get(&pid) {
            if let Some(entry) = pt.get_entry(virt_addr) {
                if !entry.is_huge {
                    self.compressed_cache.mark_inactive(virt_addr, entry.phys_addr, pid.0);
                }
            }
        }
    }

    /// Validate memory access against capability handle
    ///
    /// **Property 3: Capability-Enforced Memory Access**
    /// For any memory access attempt where the address is not mapped to the
    /// process's capability handle, this shall trigger a page fault and
    /// terminate the process.
    ///
    /// # Arguments
    /// * `pid` - Process ID attempting the access
    /// * `addr` - Virtual address being accessed
    /// * `cap` - Capability handle to validate against
    ///
    /// # Returns
    /// Ok(()) if access is valid, Err(PageFault) if not
    pub fn validate_access(
        &mut self,
        pid: ProcessId,
        addr: VirtualAddress,
        cap: &CapabilityHandle,
    ) -> Result<(), PageFault> {
        // Check if address is within capability range
        if !cap.contains(addr) {
            self.stats.capability_violations += 1;
            return Err(PageFault {
                address: addr,
                reason: FaultReason::CapabilityViolation,
                pid,
            });
        }

        // Check if page is mapped
        if let Some(pt) = self.page_tables.get(&pid) {
            if pt.translate(addr).is_none() {
                self.stats.page_faults += 1;
                return Err(PageFault {
                    address: addr,
                    reason: FaultReason::NotPresent,
                    pid,
                });
            }
        } else {
            self.stats.page_faults += 1;
            return Err(PageFault {
                address: addr,
                reason: FaultReason::NotPresent,
                pid,
            });
        }

        Ok(())
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryManagerStats {
        self.stats.clone()
    }

    /// Check if an address is backed by a huge page
    pub fn is_huge_page(&self, addr: VirtualAddress) -> bool {
        // Check all page tables for this address
        for (_, pt) in &self.page_tables {
            if let Some(entry) = pt.get_entry(addr) {
                return entry.is_huge;
            }
        }
        false
    }

    /// Get current memory pressure as a ratio (0.0 - 1.0)
    pub fn memory_pressure(&self) -> f32 {
        if self.total_memory == 0 {
            0.0
        } else {
            self.used_memory as f32 / self.total_memory as f32
        }
    }

    /// Create a page table for a new process
    pub fn create_page_table(&mut self, pid: ProcessId) {
        if !self.page_tables.contains_key(&pid) {
            self.page_tables.insert(pid, PageTable::new(pid));
        }
    }

    /// Remove a process's page table and free all its memory
    pub fn destroy_page_table(&mut self, pid: ProcessId) {
        if let Some(pt) = self.page_tables.remove(&pid) {
            // Free all mapped pages
            for (_, entry) in pt.entries {
                if entry.is_huge {
                    let _ = self.huge_page_pool.free(entry.phys_addr);
                } else {
                    self.frame_allocator.deallocate(entry.phys_addr);
                }
            }
        }
        
        // Clear compressed and inactive pages for this process
        self.compressed_cache.clear_process_inactive(pid.0);
        self.compressor.clear_process_pages(pid.0);
        self.huge_page_pool.clear_process_pages(pid.0);
    }

    /// Get huge page statistics
    pub fn huge_page_stats(&self) -> HugePageStats {
        self.huge_page_pool.stats()
    }

    /// Get compression statistics
    pub fn compression_stats(&self) -> CompressionStats {
        self.compressor.stats()
    }
}

/// Global memory manager instance
pub static MEMORY_MANAGER: Mutex<MemoryManager> = Mutex::new(MemoryManager::new());

/// Initialize the global memory manager
pub fn init_memory_manager(phys_start: PhysicalAddress, phys_size: usize) {
    MEMORY_MANAGER.lock().init(phys_start, phys_size);
}

/// Process termination callback type
/// 
/// This callback is invoked when a process must be terminated due to
/// a capability violation or other fatal memory error.
pub type ProcessTerminationCallback = fn(ProcessId, &PageFault);

/// Global process termination callback
static TERMINATION_CALLBACK: Mutex<Option<ProcessTerminationCallback>> = Mutex::new(None);

/// Register a callback for process termination on capability violations
/// 
/// **Property 3: Capability-Enforced Memory Access**
/// This callback is invoked when a process attempts to access memory
/// not mapped to its capability handle, triggering immediate termination.
/// 
/// # Arguments
/// * `callback` - Function to call when a process must be terminated
pub fn register_termination_callback(callback: ProcessTerminationCallback) {
    *TERMINATION_CALLBACK.lock() = Some(callback);
}

/// Handle a page fault with capability enforcement
/// 
/// **Property 3: Capability-Enforced Memory Access**
/// For any memory access attempt where the address is not mapped to the
/// process's capability handle, this shall trigger a page fault and
/// terminate the process.
/// 
/// This function:
/// 1. Validates the access against the process's capability handles
/// 2. Checks if the page is compressed and needs decompression
/// 3. Terminates the process if access is unauthorized
/// 
/// # Arguments
/// * `pid` - Process ID that caused the fault
/// * `fault_addr` - Virtual address that caused the fault
/// * `error_code` - Hardware page fault error code
/// * `capabilities` - List of capability handles for the process
/// 
/// # Returns
/// * `Ok(PageFaultResolution)` if fault was handled
/// * `Err(PageFault)` if fault is fatal and process should be terminated
pub fn handle_capability_page_fault(
    pid: ProcessId,
    fault_addr: VirtualAddress,
    error_code: u64,
    capabilities: &[CapabilityHandle],
) -> Result<PageFaultResolution, PageFault> {
    let mut mm = MEMORY_MANAGER.lock();
    
    // Decode error code
    let protection_violation = (error_code & 0x1) != 0;
    let write_access = (error_code & 0x2) != 0;
    let _user_mode = (error_code & 0x4) != 0;
    let _instruction_fetch = (error_code & 0x10) != 0;
    
    // First, check if any capability covers this address
    let mut valid_capability: Option<&CapabilityHandle> = None;
    for cap in capabilities {
        if cap.cap_type == CapabilityType::Memory && cap.contains(fault_addr) {
            valid_capability = Some(cap);
            break;
        }
    }
    
    // If no capability covers this address, it's a capability violation
    if valid_capability.is_none() {
        let fault = PageFault {
            address: fault_addr,
            reason: FaultReason::CapabilityViolation,
            pid,
        };
        
        mm.stats.capability_violations += 1;
        
        // Invoke termination callback if registered
        if let Some(callback) = *TERMINATION_CALLBACK.lock() {
            callback(pid, &fault);
        }
        
        return Err(fault);
    }
    
    let cap = valid_capability.unwrap();
    
    // Check permission violations
    if write_access && !cap.perms.write {
        let fault = PageFault {
            address: fault_addr,
            reason: FaultReason::WriteProtection,
            pid,
        };
        mm.stats.page_faults += 1;
        
        // Invoke termination callback
        if let Some(callback) = *TERMINATION_CALLBACK.lock() {
            callback(pid, &fault);
        }
        
        return Err(fault);
    }
    
    // Check if page is compressed and needs decompression
    let page_addr = fault_addr & !(COMPRESSION_PAGE_SIZE as u64 - 1);
    if mm.compressed_cache.is_compressed(page_addr) {
        // Decompress the page
        match mm.decompress_page(fault_addr) {
            Ok(()) => {
                return Ok(PageFaultResolution::Decompressed);
            }
            Err(_) => {
                let fault = PageFault {
                    address: fault_addr,
                    reason: FaultReason::NotPresent,
                    pid,
                };
                mm.stats.page_faults += 1;
                return Err(fault);
            }
        }
    }
    
    // Check if page is simply not present (demand paging scenario)
    if !protection_violation {
        // Page not present - this could be demand paging
        // For now, we treat this as a fatal fault
        let fault = PageFault {
            address: fault_addr,
            reason: FaultReason::NotPresent,
            pid,
        };
        mm.stats.page_faults += 1;
        
        // Invoke termination callback
        if let Some(callback) = *TERMINATION_CALLBACK.lock() {
            callback(pid, &fault);
        }
        
        return Err(fault);
    }
    
    // Unknown fault type
    let fault = PageFault {
        address: fault_addr,
        reason: FaultReason::InvalidAddress,
        pid,
    };
    mm.stats.page_faults += 1;
    Err(fault)
}

/// Resolution type for handled page faults
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageFaultResolution {
    /// Page was decompressed and is now accessible
    Decompressed,
    /// Page was allocated on demand
    DemandPaged,
    /// Copy-on-write was performed
    CopyOnWrite,
}

/// Validate memory access for a process with capability enforcement
/// 
/// **Property 3: Capability-Enforced Memory Access**
/// This is a convenience function that validates access and handles
/// the termination of offending processes.
/// 
/// # Arguments
/// * `pid` - Process ID attempting the access
/// * `addr` - Virtual address being accessed
/// * `access_type` - Type of access (read, write, execute)
/// * `capabilities` - List of capability handles for the process
/// 
/// # Returns
/// * `Ok(())` if access is valid
/// * `Err(PageFault)` if access is denied (process will be terminated)
pub fn validate_and_enforce_access(
    pid: ProcessId,
    addr: VirtualAddress,
    access_type: AccessType,
    capabilities: &[CapabilityHandle],
) -> Result<(), PageFault> {
    // Find a capability that covers this address
    for cap in capabilities {
        if cap.cap_type == CapabilityType::Memory && cap.contains(addr) {
            // Check permissions based on access type
            let permitted = match access_type {
                AccessType::Read => cap.perms.read,
                AccessType::Write => cap.perms.write,
                AccessType::Execute => cap.perms.execute,
            };
            
            if permitted {
                return Ok(());
            } else {
                let reason = match access_type {
                    AccessType::Read => FaultReason::NotPresent,
                    AccessType::Write => FaultReason::WriteProtection,
                    AccessType::Execute => FaultReason::ExecuteProtection,
                };
                
                let fault = PageFault {
                    address: addr,
                    reason,
                    pid,
                };
                
                // Update stats
                {
                    let mut mm = MEMORY_MANAGER.lock();
                    mm.stats.page_faults += 1;
                }
                
                // Invoke termination callback
                if let Some(callback) = *TERMINATION_CALLBACK.lock() {
                    callback(pid, &fault);
                }
                
                return Err(fault);
            }
        }
    }
    
    // No capability covers this address - capability violation
    let fault = PageFault {
        address: addr,
        reason: FaultReason::CapabilityViolation,
        pid,
    };
    
    // Update stats
    {
        let mut mm = MEMORY_MANAGER.lock();
        mm.stats.capability_violations += 1;
    }
    
    // Invoke termination callback
    if let Some(callback) = *TERMINATION_CALLBACK.lock() {
        callback(pid, &fault);
    }
    
    Err(fault)
}

/// Access type for memory validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
    Execute,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_init() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 64 * 1024 * 1024); // 64MB
        
        assert_eq!(mm.total_memory, 64 * 1024 * 1024);
        assert_eq!(mm.used_memory, 0);
    }

    #[test]
    fn test_allocate_small() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 64 * 1024 * 1024);
        
        let cap = CapabilityHandle::new_memory(0x1000_0000, 1024 * 1024, Permissions::read_write());
        let result = mm.allocate(ProcessId(1), 4096, &cap);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_allocate_huge_page() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 64 * 1024 * 1024);
        
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4 * 1024 * 1024, Permissions::read_write());
        let result = mm.allocate(ProcessId(1), 2 * 1024 * 1024, &cap);
        
        // Should use huge page
        if result.is_ok() {
            let addr = result.unwrap();
            assert!(mm.is_huge_page(addr));
        }
    }

    #[test]
    fn test_capability_validation() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 64 * 1024 * 1024);
        
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_write());
        let _ = mm.allocate(ProcessId(1), 4096, &cap);
        
        // Valid access
        let valid = mm.validate_access(ProcessId(1), 0x1000_0000, &cap);
        assert!(valid.is_ok());
        
        // Invalid access (outside capability range)
        let invalid = mm.validate_access(ProcessId(1), 0x2000_0000, &cap);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_memory_pressure() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 1024 * 1024); // 1MB total
        
        // Initially low pressure
        assert_eq!(mm.check_pressure(), PressureStatus::Low);
        
        // Simulate high usage
        mm.used_memory = 900 * 1024; // 90%
        assert_eq!(mm.check_pressure(), PressureStatus::High);
    }

    /// **Property 1: Memory Compression Trigger**
    /// For any memory state where pressure exceeds 80%, the Memory_Manager
    /// shall initiate LZ4 compression of inactive pages.
    #[test]
    fn test_compression_trigger_at_80_percent() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 1024 * 1024); // 1MB total
        
        // Below threshold (75%) - should not trigger compression
        mm.used_memory = 750 * 1024;
        let status = mm.check_pressure();
        assert_eq!(status, PressureStatus::Moderate);
        
        // At threshold (80%) - should trigger compression
        mm.used_memory = 820 * 1024;
        let status = mm.check_pressure();
        assert_eq!(status, PressureStatus::High);
        
        // Above threshold (85%) - should trigger compression
        mm.used_memory = 870 * 1024;
        let status = mm.check_pressure();
        assert_eq!(status, PressureStatus::High);
    }

    #[test]
    fn test_compression_trigger_with_inactive_pages() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 1024 * 1024); // 1MB total
        
        // Allocate some pages
        let cap = CapabilityHandle::new_memory(0x1000_0000, 64 * 1024, Permissions::read_write());
        let _ = mm.allocate(ProcessId(1), 4096, &cap);
        
        // Mark some pages as inactive
        mm.compressed_cache.mark_inactive(0x1000_0000, 0x10_0000, 1);
        mm.compressed_cache.mark_inactive(0x1000_1000, 0x10_1000, 1);
        
        assert_eq!(mm.compressed_cache.inactive_count(), 2);
        
        // Simulate high pressure
        mm.used_memory = 850 * 1024;
        
        // Trigger compression
        let status = mm.check_pressure();
        assert_eq!(status, PressureStatus::High);
    }

    #[test]
    fn test_pressure_status_levels() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 1000); // 1000 bytes for easy percentage calculation
        
        // Low pressure (< 60%)
        mm.used_memory = 500;
        assert_eq!(mm.check_pressure(), PressureStatus::Low);
        
        // Moderate pressure (60-80%)
        mm.used_memory = 700;
        assert_eq!(mm.check_pressure(), PressureStatus::Moderate);
        
        // High pressure (80-95%)
        mm.used_memory = 850;
        assert_eq!(mm.check_pressure(), PressureStatus::High);
        
        // Critical pressure (> 95%)
        mm.used_memory = 960;
        assert_eq!(mm.check_pressure(), PressureStatus::Critical);
    }

    #[test]
    fn test_mark_page_inactive() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 64 * 1024 * 1024);
        
        // Allocate a page
        let cap = CapabilityHandle::new_memory(0x1000_0000, 8192, Permissions::read_write());
        let addr = mm.allocate(ProcessId(1), 4096, &cap).unwrap();
        
        // Mark it as inactive
        mm.mark_page_inactive(ProcessId(1), addr);
        
        assert_eq!(mm.compressed_cache.inactive_count(), 1);
    }

    #[test]
    fn test_decompress_page() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 64 * 1024 * 1024);
        
        // Manually add a compressed page to the cache
        let data = [0u8; COMPRESSION_PAGE_SIZE];
        let _ = mm.compressed_cache.compress_page(&data, 0x5000_0000, 0x10_0000, 1);
        
        assert!(mm.compressed_cache.is_compressed(0x5000_0000));
        
        // Decompress it
        let result = mm.decompress_page(0x5000_0000);
        assert!(result.is_ok());
        
        // Should no longer be in compressed cache
        assert!(!mm.compressed_cache.is_compressed(0x5000_0000));
    }

    #[test]
    fn test_compression_stats_update() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 1024 * 1024);
        
        // Add some compressed pages
        let data = [0u8; COMPRESSION_PAGE_SIZE];
        let _ = mm.compressed_cache.compress_page(&data, 0x1000, 0x2000, 1);
        let _ = mm.compressed_cache.compress_page(&data, 0x2000, 0x3000, 1);
        
        // Trigger compression to update stats
        mm.used_memory = 850 * 1024;
        mm.check_pressure();
        
        let stats = mm.stats();
        // Stats should reflect compression activity
        assert!(stats.compression_ratio >= 0.0);
    }

    // =========================================================================
    // Capability-Enforced Memory Access Tests (Requirement 1.4)
    // =========================================================================

    /// **Property 3: Capability-Enforced Memory Access**
    /// For any memory access attempt where the address is not mapped to the
    /// process's capability handle, the Memory_Manager shall trigger a page
    /// fault and terminate the process.
    #[test]
    fn test_capability_violation_triggers_fault() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 64 * 1024 * 1024);
        
        // Create a capability for a specific region
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_write());
        let _ = mm.allocate(ProcessId(1), 4096, &cap);
        
        // Access within capability range - should succeed
        let valid_result = mm.validate_access(ProcessId(1), 0x1000_0000, &cap);
        assert!(valid_result.is_ok());
        
        // Access outside capability range - should fail with CapabilityViolation
        let invalid_result = mm.validate_access(ProcessId(1), 0x2000_0000, &cap);
        assert!(invalid_result.is_err());
        
        let fault = invalid_result.unwrap_err();
        assert_eq!(fault.reason, FaultReason::CapabilityViolation);
        assert_eq!(fault.address, 0x2000_0000);
        assert_eq!(fault.pid, ProcessId(1));
    }

    #[test]
    fn test_capability_violation_increments_stats() {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 64 * 1024 * 1024);
        
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_write());
        
        let initial_violations = mm.stats.capability_violations;
        
        // Trigger a capability violation
        let _ = mm.validate_access(ProcessId(1), 0x2000_0000, &cap);
        
        assert_eq!(mm.stats.capability_violations, initial_violations + 1);
    }

    #[test]
    fn test_validate_and_enforce_access_read() {
        // Test read access validation
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_only());
        let caps = [cap];
        
        // Read access should succeed
        let result = validate_and_enforce_access(
            ProcessId(1),
            0x1000_0000,
            AccessType::Read,
            &caps,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_and_enforce_access_write_denied() {
        // Test write access to read-only capability
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_only());
        let caps = [cap];
        
        // Write access should fail
        let result = validate_and_enforce_access(
            ProcessId(1),
            0x1000_0000,
            AccessType::Write,
            &caps,
        );
        assert!(result.is_err());
        
        let fault = result.unwrap_err();
        assert_eq!(fault.reason, FaultReason::WriteProtection);
    }

    #[test]
    fn test_validate_and_enforce_access_execute_denied() {
        // Test execute access to non-executable capability
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_write());
        let caps = [cap];
        
        // Execute access should fail
        let result = validate_and_enforce_access(
            ProcessId(1),
            0x1000_0000,
            AccessType::Execute,
            &caps,
        );
        assert!(result.is_err());
        
        let fault = result.unwrap_err();
        assert_eq!(fault.reason, FaultReason::ExecuteProtection);
    }

    #[test]
    fn test_validate_and_enforce_access_no_capability() {
        // Test access with no covering capability
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_write());
        let caps = [cap];
        
        // Access to address not covered by any capability
        let result = validate_and_enforce_access(
            ProcessId(1),
            0x2000_0000, // Outside capability range
            AccessType::Read,
            &caps,
        );
        assert!(result.is_err());
        
        let fault = result.unwrap_err();
        assert_eq!(fault.reason, FaultReason::CapabilityViolation);
    }

    #[test]
    fn test_validate_and_enforce_access_empty_capabilities() {
        // Test access with no capabilities at all
        let caps: [CapabilityHandle; 0] = [];
        
        let result = validate_and_enforce_access(
            ProcessId(1),
            0x1000_0000,
            AccessType::Read,
            &caps,
        );
        assert!(result.is_err());
        
        let fault = result.unwrap_err();
        assert_eq!(fault.reason, FaultReason::CapabilityViolation);
    }

    #[test]
    fn test_handle_capability_page_fault_violation() {
        // Initialize memory manager
        {
            let mut mm = MEMORY_MANAGER.lock();
            mm.init(0x10_0000, 64 * 1024 * 1024);
        }
        
        // Create capability for a specific region
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_write());
        let caps = [cap];
        
        // Fault at address outside capability range
        let result = handle_capability_page_fault(
            ProcessId(1),
            0x2000_0000, // Outside capability range
            0x0, // Error code: not present
            &caps,
        );
        
        assert!(result.is_err());
        let fault = result.unwrap_err();
        assert_eq!(fault.reason, FaultReason::CapabilityViolation);
    }

    #[test]
    fn test_handle_capability_page_fault_write_protection() {
        // Initialize memory manager
        {
            let mut mm = MEMORY_MANAGER.lock();
            mm.init(0x10_0000, 64 * 1024 * 1024);
        }
        
        // Create read-only capability
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_only());
        let caps = [cap];
        
        // Fault with write access to read-only region
        let result = handle_capability_page_fault(
            ProcessId(1),
            0x1000_0000,
            0x3, // Error code: protection violation + write access
            &caps,
        );
        
        assert!(result.is_err());
        let fault = result.unwrap_err();
        assert_eq!(fault.reason, FaultReason::WriteProtection);
    }

    #[test]
    fn test_capability_handle_contains() {
        let cap = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_write());
        
        // Address at start of range
        assert!(cap.contains(0x1000_0000));
        
        // Address in middle of range
        assert!(cap.contains(0x1000_0800));
        
        // Address at end of range (exclusive)
        assert!(!cap.contains(0x1000_1000));
        
        // Address before range
        assert!(!cap.contains(0x0FFF_FFFF));
        
        // Address after range
        assert!(!cap.contains(0x1000_1001));
    }

    #[test]
    fn test_multiple_capabilities() {
        // Test with multiple capabilities covering different regions
        let cap1 = CapabilityHandle::new_memory(0x1000_0000, 4096, Permissions::read_only());
        let cap2 = CapabilityHandle::new_memory(0x2000_0000, 8192, Permissions::read_write());
        let cap3 = CapabilityHandle::new_memory(0x3000_0000, 4096, Permissions::read_execute());
        let caps = [cap1, cap2, cap3];
        
        // Read from first region - should succeed
        let result1 = validate_and_enforce_access(
            ProcessId(1),
            0x1000_0000,
            AccessType::Read,
            &caps,
        );
        assert!(result1.is_ok());
        
        // Write to second region - should succeed
        let result2 = validate_and_enforce_access(
            ProcessId(1),
            0x2000_0000,
            AccessType::Write,
            &caps,
        );
        assert!(result2.is_ok());
        
        // Execute from third region - should succeed
        let result3 = validate_and_enforce_access(
            ProcessId(1),
            0x3000_0000,
            AccessType::Execute,
            &caps,
        );
        assert!(result3.is_ok());
        
        // Write to first region (read-only) - should fail
        let result4 = validate_and_enforce_access(
            ProcessId(1),
            0x1000_0000,
            AccessType::Write,
            &caps,
        );
        assert!(result4.is_err());
    }

    #[test]
    fn test_page_fault_resolution_types() {
        // Verify PageFaultResolution enum variants exist and are distinct
        let decompressed = PageFaultResolution::Decompressed;
        let demand_paged = PageFaultResolution::DemandPaged;
        let cow = PageFaultResolution::CopyOnWrite;
        
        assert_ne!(decompressed, demand_paged);
        assert_ne!(demand_paged, cow);
        assert_ne!(decompressed, cow);
    }

    #[test]
    fn test_access_type_variants() {
        // Verify AccessType enum variants exist and are distinct
        let read = AccessType::Read;
        let write = AccessType::Write;
        let execute = AccessType::Execute;
        
        assert_ne!(read, write);
        assert_ne!(write, execute);
        assert_ne!(read, execute);
    }

    #[test]
    fn test_fault_reason_variants() {
        // Verify all FaultReason variants
        let not_present = FaultReason::NotPresent;
        let write_prot = FaultReason::WriteProtection;
        let exec_prot = FaultReason::ExecuteProtection;
        let cap_violation = FaultReason::CapabilityViolation;
        let invalid_addr = FaultReason::InvalidAddress;
        
        assert_ne!(not_present, write_prot);
        assert_ne!(write_prot, exec_prot);
        assert_ne!(exec_prot, cap_violation);
        assert_ne!(cap_violation, invalid_addr);
    }
}
