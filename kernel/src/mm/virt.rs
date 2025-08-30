/// Virtual Memory Management for Polymera OS
/// 
/// This module handles virtual address space management, virtual memory allocation,
/// and virtual-to-physical mappings.

use crate::{kprintln, klog};
use super::{MemoryResult, MemoryError, constants::*};
use x86_64::{VirtAddr, PhysAddr};
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

/// Virtual memory area descriptor
#[derive(Debug, Clone, Copy)]
pub struct VirtualMemoryArea {
    /// Start virtual address
    pub start: u64,
    
    /// End virtual address (exclusive)
    pub end: u64,
    
    /// VMA type
    pub vma_type: VmaType,
    
    /// Protection flags
    pub protection: VmaProtection,
}

impl VirtualMemoryArea {
    /// Create a new VMA
    pub const fn new(start: u64, end: u64, vma_type: VmaType, protection: VmaProtection) -> Self {
        Self { start, end, vma_type, protection }
    }
    
    /// Get the size of this VMA
    pub const fn size(&self) -> u64 {
        self.end - self.start
    }
    
    /// Check if this VMA contains the given address
    pub const fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end
    }
    
    /// Check if this VMA overlaps with another VMA
    pub const fn overlaps(&self, other: &VirtualMemoryArea) -> bool {
        !(self.end <= other.start || other.end <= self.start)
    }
}

/// Types of virtual memory areas
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmaType {
    /// Kernel code
    KernelCode,
    
    /// Kernel data
    KernelData,
    
    /// Kernel heap
    KernelHeap,
    
    /// Kernel stack
    KernelStack,
    
    /// User code
    UserCode,
    
    /// User data
    UserData,
    
    /// User heap
    UserHeap,
    
    /// User stack
    UserStack,
    
    /// Shared memory
    Shared,
    
    /// Device memory
    Device,
}

impl core::fmt::Display for VmaType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            VmaType::KernelCode => write!(f, "Kernel Code"),
            VmaType::KernelData => write!(f, "Kernel Data"),
            VmaType::KernelHeap => write!(f, "Kernel Heap"),
            VmaType::KernelStack => write!(f, "Kernel Stack"),
            VmaType::UserCode => write!(f, "User Code"),
            VmaType::UserData => write!(f, "User Data"),
            VmaType::UserHeap => write!(f, "User Heap"),
            VmaType::UserStack => write!(f, "User Stack"),
            VmaType::Shared => write!(f, "Shared"),
            VmaType::Device => write!(f, "Device"),
        }
    }
}

/// Virtual memory area protection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmaProtection {
    /// Memory is readable
    pub read: bool,
    
    /// Memory is writable
    pub write: bool,
    
    /// Memory is executable
    pub execute: bool,
    
    /// Memory is user accessible
    pub user: bool,
}

impl VmaProtection {
    /// Create protection flags for read-only data
    pub const fn read_only() -> Self {
        Self { read: true, write: false, execute: false, user: false }
    }
    
    /// Create protection flags for read-write data
    pub const fn read_write() -> Self {
        Self { read: true, write: true, execute: false, user: false }
    }
    
    /// Create protection flags for executable code
    pub const fn read_execute() -> Self {
        Self { read: true, write: false, execute: true, user: false }
    }
    
    /// Create protection flags for user read-write data
    pub const fn user_read_write() -> Self {
        Self { read: true, write: true, execute: false, user: true }
    }
    
    /// Create protection flags for user executable code
    pub const fn user_read_execute() -> Self {
        Self { read: true, write: false, execute: true, user: true }
    }
}

impl core::fmt::Display for VmaProtection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}{}{}",
               if self.read { "r" } else { "-" },
               if self.write { "w" } else { "-" },
               if self.execute { "x" } else { "-" },
               if self.user { "u" } else { "-" })
    }
}

//=============================================================================
// VIRTUAL MEMORY API IMPLEMENTATION
//=============================================================================

/// Page flags for virtual memory mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmFlags {
    /// Page is readable
    pub read: bool,
    
    /// Page is writable  
    pub write: bool,
    
    /// Page is executable
    pub execute: bool,
    
    /// Page is user-accessible
    pub user: bool,
    
    /// Page is present in memory
    pub present: bool,
    
    /// Page has been accessed
    pub accessed: bool,
    
    /// Page has been modified
    pub dirty: bool,
}

impl VmFlags {
    /// Create read-only flags
    pub const fn read_only() -> Self {
        Self {
            read: true, write: false, execute: false, user: false,
            present: true, accessed: false, dirty: false,
        }
    }
    
    /// Create read-write flags
    pub const fn read_write() -> Self {
        Self {
            read: true, write: true, execute: false, user: false,
            present: true, accessed: false, dirty: false,
        }
    }
    
    /// Create read-execute flags
    pub const fn read_execute() -> Self {
        Self {
            read: true, write: false, execute: true, user: false,
            present: true, accessed: false, dirty: false,
        }
    }
    
    /// Create read-write-execute flags
    pub const fn read_write_execute() -> Self {
        Self {
            read: true, write: true, execute: true, user: false,
            present: true, accessed: false, dirty: false,
        }
    }
    
    /// Create user read-only flags
    pub const fn user_read_only() -> Self {
        Self {
            read: true, write: false, execute: false, user: true,
            present: true, accessed: false, dirty: false,
        }
    }
    
    /// Create user read-write flags
    pub const fn user_read_write() -> Self {
        Self {
            read: true, write: true, execute: false, user: true,
            present: true, accessed: false, dirty: false,
        }
    }
    
    /// Create user read-execute flags
    pub const fn user_read_execute() -> Self {
        Self {
            read: true, write: false, execute: true, user: true,
            present: true, accessed: false, dirty: false,
        }
    }
    
    /// Convert to paging PageFlags
    pub fn to_page_flags(self) -> super::paging::PageFlags {
        super::paging::PageFlags {
            present: self.present,
            writable: self.write,
            user_accessible: self.user,
            executable: self.execute,
            accessed: self.accessed,
            dirty: self.dirty,
        }
    }
}

impl core::fmt::Display for VmFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}{}{}{}{}{}{}",
               if self.present { "P" } else { "-" },
               if self.read { "R" } else { "-" },
               if self.write { "W" } else { "-" },
               if self.execute { "X" } else { "-" },
               if self.user { "U" } else { "-" },
               if self.accessed { "A" } else { "-" },
               if self.dirty { "D" } else { "-" })
    }
}

/// Page mapping entry
#[derive(Debug, Clone, Copy)]
pub struct PageMapping {
    /// Virtual address
    pub vaddr: u64,
    
    /// Physical address
    pub paddr: u64,
    
    /// Page flags
    pub flags: VmFlags,
    
    /// Mapping timestamp
    pub mapped_at: u64,
}

impl PageMapping {
    /// Create a new page mapping
    pub fn new(vaddr: u64, paddr: u64, flags: VmFlags) -> Self {
        Self {
            vaddr,
            paddr,
            flags,
            mapped_at: get_current_timestamp(),
        }
    }
}

/// Virtual memory manager state
#[derive(Debug)]
pub struct VirtualMemoryManager {
    /// Page mappings (virtual address -> mapping info)
    mappings: BTreeMap<u64, PageMapping>,
    
    /// Statistics
    mapped_pages: u64,
    unmapped_pages: u64,
    failed_mappings: u64,
    double_map_attempts: u64,
    double_unmap_attempts: u64,
}

impl VirtualMemoryManager {
    /// Create a new virtual memory manager
    pub fn new() -> Self {
        Self {
            mappings: BTreeMap::new(),
            mapped_pages: 0,
            unmapped_pages: 0,
            failed_mappings: 0,
            double_map_attempts: 0,
            double_unmap_attempts: 0,
        }
    }
    
    /// Map a virtual page to a physical page
    /// 
    /// # Arguments
    /// * `vaddr` - Virtual address to map
    /// * `paddr` - Physical address to map to
    /// * `flags` - Page flags
    /// 
    /// # Returns
    /// Result indicating success or error
    pub fn map_page(&mut self, vaddr: u64, paddr: u64, flags: VmFlags) -> MemoryResult<()> {
        let page_vaddr = vaddr & !(PAGE_SIZE as u64 - 1); // Align to page boundary
        let page_paddr = paddr & !(PAGE_SIZE as u64 - 1); // Align to page boundary
        
        // Check if page is already mapped
        if self.mappings.contains_key(&page_vaddr) {
            self.double_map_attempts += 1;
            klog!(ERROR, "[VM] Attempt to map already mapped page: 0x{:016x}", page_vaddr);
            return Err(MemoryError::AlreadyMapped);
        }
        
        // Validate addresses
        if page_vaddr == 0 && !flags.user {
            // Allow null page mapping for kernel if explicitly requested
        } else if page_vaddr == 0 {
            return Err(MemoryError::InvalidAddress);
        }
        
        // Create mapping entry
        let mapping = PageMapping::new(page_vaddr, page_paddr, flags);
        
        // Call low-level paging function
        match super::paging::map_page(
            VirtAddr::new(page_vaddr),
            PhysAddr::new(page_paddr),
            flags.to_page_flags()
        ) {
            Ok(()) => {
                // Store mapping metadata
                self.mappings.insert(page_vaddr, mapping);
                self.mapped_pages += 1;
                
                klog!(TRACE, "[VM] Mapped page: 0x{:016x} -> 0x{:016x} ({})",
                      page_vaddr, page_paddr, flags);
                
                Ok(())
            }
            Err(e) => {
                self.failed_mappings += 1;
                klog!(ERROR, "[VM] Failed to map page 0x{:016x}: {}", page_vaddr, e);
                Err(e)
            }
        }
    }
    
    /// Unmap a virtual page
    /// 
    /// # Arguments
    /// * `vaddr` - Virtual address to unmap
    /// 
    /// # Returns
    /// Result indicating success or error
    pub fn unmap_page(&mut self, vaddr: u64) -> MemoryResult<()> {
        let page_vaddr = vaddr & !(PAGE_SIZE as u64 - 1); // Align to page boundary
        
        // Check if page is mapped
        if !self.mappings.contains_key(&page_vaddr) {
            self.double_unmap_attempts += 1;
            klog!(ERROR, "[VM] Attempt to unmap non-mapped page: 0x{:016x}", page_vaddr);
            return Err(MemoryError::NotMapped);
        }
        
        // Call low-level paging function
        match super::paging::unmap_page(VirtAddr::new(page_vaddr)) {
            Ok(()) => {
                // Remove mapping metadata
                if let Some(mapping) = self.mappings.remove(&page_vaddr) {
                    self.unmapped_pages += 1;
                    
                    klog!(TRACE, "[VM] Unmapped page: 0x{:016x} -> 0x{:016x} ({})",
                          page_vaddr, mapping.paddr, mapping.flags);
                }
                
                Ok(())
            }
            Err(e) => {
                klog!(ERROR, "[VM] Failed to unmap page 0x{:016x}: {}", page_vaddr, e);
                Err(e)
            }
        }
    }
    
    /// Translate virtual address to physical address
    /// 
    /// # Arguments
    /// * `vaddr` - Virtual address to translate
    /// 
    /// # Returns
    /// Physical address if mapped, None otherwise
    pub fn translate_va(&self, vaddr: u64) -> Option<u64> {
        let page_vaddr = vaddr & !(PAGE_SIZE as u64 - 1); // Align to page boundary
        let offset = vaddr & (PAGE_SIZE as u64 - 1);       // Get page offset
        
        if let Some(mapping) = self.mappings.get(&page_vaddr) {
            Some(mapping.paddr + offset)
        } else {
            // Try hardware translation as fallback
            match super::paging::translate_address(VirtAddr::new(vaddr)) {
                Ok(phys_addr) => Some(phys_addr.as_u64()),
                Err(_) => None,
            }
        }
    }
    
    /// Get mapping information for a virtual address
    pub fn get_mapping(&self, vaddr: u64) -> Option<PageMapping> {
        let page_vaddr = vaddr & !(PAGE_SIZE as u64 - 1);
        self.mappings.get(&page_vaddr).copied()
    }
    
    /// Check if a virtual address is mapped
    pub fn is_mapped(&self, vaddr: u64) -> bool {
        let page_vaddr = vaddr & !(PAGE_SIZE as u64 - 1);
        self.mappings.contains_key(&page_vaddr)
    }
    
    /// Get the flags for a mapped page
    pub fn get_page_flags(&self, vaddr: u64) -> Option<VmFlags> {
        let page_vaddr = vaddr & !(PAGE_SIZE as u64 - 1);
        self.mappings.get(&page_vaddr).map(|m| m.flags)
    }
    
    /// Check if access to a virtual address is allowed
    pub fn check_access(&self, vaddr: u64, write: bool, execute: bool, user: bool) -> bool {
        if let Some(flags) = self.get_page_flags(vaddr) {
            // Check if page is present
            if !flags.present {
                return false;
            }
            
            // Check read access (always required)
            if !flags.read {
                return false;
            }
            
            // Check write access if requested
            if write && !flags.write {
                return false;
            }
            
            // Check execute access if requested
            if execute && !flags.execute {
                return false;
            }
            
            // Check user access if requested
            if user && !flags.user {
                return false;
            }
            
            true
        } else {
            false
        }
    }
    
    /// Update page flags
    pub fn update_flags(&mut self, vaddr: u64, new_flags: VmFlags) -> MemoryResult<()> {
        let page_vaddr = vaddr & !(PAGE_SIZE as u64 - 1);
        
        if let Some(mapping) = self.mappings.get_mut(&page_vaddr) {
            // Update hardware page table
            match super::paging::map_page(
                VirtAddr::new(page_vaddr),
                PhysAddr::new(mapping.paddr),
                new_flags.to_page_flags()
            ) {
                Ok(()) => {
                    mapping.flags = new_flags;
                    klog!(TRACE, "[VM] Updated flags for page 0x{:016x}: {}", page_vaddr, new_flags);
                    Ok(())
                }
                Err(e) => {
                    klog!(ERROR, "[VM] Failed to update flags for page 0x{:016x}: {}", page_vaddr, e);
                    Err(e)
                }
            }
        } else {
            Err(MemoryError::NotMapped)
        }
    }
    
    /// Get statistics
    pub fn stats(&self) -> VmStats {
        VmStats {
            mapped_pages: self.mapped_pages,
            unmapped_pages: self.unmapped_pages,
            active_mappings: self.mappings.len() as u64,
            failed_mappings: self.failed_mappings,
            double_map_attempts: self.double_map_attempts,
            double_unmap_attempts: self.double_unmap_attempts,
        }
    }
    
    /// Validate all mappings
    pub fn validate(&self) -> bool {
        for (&vaddr, mapping) in &self.mappings {
            // Check alignment
            if vaddr % PAGE_SIZE as u64 != 0 {
                klog!(ERROR, "[VM] Validation failed: misaligned virtual address 0x{:016x}", vaddr);
                return false;
            }
            
            if mapping.paddr % PAGE_SIZE as u64 != 0 {
                klog!(ERROR, "[VM] Validation failed: misaligned physical address 0x{:016x}", mapping.paddr);
                return false;
            }
            
            // Check that virtual address matches mapping
            if vaddr != mapping.vaddr {
                klog!(ERROR, "[VM] Validation failed: address mismatch 0x{:016x} != 0x{:016x}", 
                      vaddr, mapping.vaddr);
                return false;
            }
        }
        
        true
    }
    
    /// Print all mappings
    pub fn print_mappings(&self) {
        kprintln!("");
        kprintln!("=== VIRTUAL MEMORY MAPPINGS ===");
        kprintln!("Active mappings: {}", self.mappings.len());
        
        for (&vaddr, mapping) in &self.mappings {
            kprintln!("0x{:016x} -> 0x{:016x} {} (mapped at {})",
                      vaddr, mapping.paddr, mapping.flags, mapping.mapped_at);
        }
        
        kprintln!("=== END MAPPINGS ===");
        kprintln!("");
    }
}

/// Virtual memory statistics
#[derive(Debug, Clone, Copy)]
pub struct VmStats {
    pub mapped_pages: u64,
    pub unmapped_pages: u64,
    pub active_mappings: u64,
    pub failed_mappings: u64,
    pub double_map_attempts: u64,
    pub double_unmap_attempts: u64,
}

/// Global virtual memory manager
static VM_MANAGER: Mutex<VirtualMemoryManager> = Mutex::new(VirtualMemoryManager::new());

/// Global timestamp counter
static TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Get current timestamp
fn get_current_timestamp() -> u64 {
    TIMESTAMP_COUNTER.fetch_add(1, Ordering::Relaxed)
}

//=============================================================================
// PUBLIC VM API FUNCTIONS
//=============================================================================

/// Map a virtual page to a physical page
/// 
/// # Arguments
/// * `vaddr` - Virtual address to map
/// * `paddr` - Physical address to map to  
/// * `flags` - Page flags (RWX permissions)
/// 
/// # Returns
/// Result indicating success or error
pub fn map_page(vaddr: u64, paddr: u64, flags: VmFlags) -> MemoryResult<()> {
    let mut vm_manager = VM_MANAGER.lock();
    vm_manager.map_page(vaddr, paddr, flags)
}

/// Unmap a virtual page
/// 
/// # Arguments
/// * `vaddr` - Virtual address to unmap
/// 
/// # Returns
/// Result indicating success or error
pub fn unmap_page(vaddr: u64) -> MemoryResult<()> {
    let mut vm_manager = VM_MANAGER.lock();
    vm_manager.unmap_page(vaddr)
}

/// Translate virtual address to physical address
/// 
/// # Arguments
/// * `vaddr` - Virtual address to translate
/// 
/// # Returns
/// Physical address if mapped, None otherwise
pub fn translate_va(vaddr: u64) -> Option<u64> {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.translate_va(vaddr)
}

/// Check if a virtual address is mapped
pub fn is_mapped(vaddr: u64) -> bool {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.is_mapped(vaddr)
}

/// Get page flags for a virtual address
pub fn get_page_flags(vaddr: u64) -> Option<VmFlags> {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.get_page_flags(vaddr)
}

/// Check access permissions for a virtual address
pub fn check_access(vaddr: u64, write: bool, execute: bool, user: bool) -> bool {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.check_access(vaddr, write, execute, user)
}

/// Update page flags
pub fn update_flags(vaddr: u64, new_flags: VmFlags) -> MemoryResult<()> {
    let mut vm_manager = VM_MANAGER.lock();
    vm_manager.update_flags(vaddr, new_flags)
}

/// Get virtual memory statistics
pub fn get_vm_stats() -> VmStats {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.stats()
}

/// Validate all virtual memory mappings
pub fn validate_vm() -> bool {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.validate()
}

/// Print all virtual memory mappings
pub fn print_vm_mappings() {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.print_mappings();
}

//=============================================================================
// END VIRTUAL MEMORY API IMPLEMENTATION
//=============================================================================

/// Virtual memory statistics
#[derive(Debug, Clone, Copy)]
pub struct VirtualMemoryStats {
    /// Total virtual address space
    pub total_virtual_space: u64,
    
    /// Used virtual space
    pub used_virtual_space: u64,
    
    /// Kernel virtual space usage
    pub kernel_virtual_space: u64,
    
    /// User virtual space usage
    pub user_virtual_space: u64,
    
    /// Number of VMAs
    pub vma_count: u64,
    
    /// Number of virtual allocations
    pub virtual_allocations: u64,
    
    /// Number of virtual deallocations
    pub virtual_deallocations: u64,
}

impl VirtualMemoryStats {
    /// Create new virtual memory statistics
    pub const fn new() -> Self {
        Self {
            total_virtual_space: 0,
            used_virtual_space: 0,
            kernel_virtual_space: 0,
            user_virtual_space: 0,
            vma_count: 0,
            virtual_allocations: 0,
            virtual_deallocations: 0,
        }
    }
    
    /// Calculate virtual memory utilization
    pub fn utilization(&self) -> f32 {
        if self.total_virtual_space == 0 {
            0.0
        } else {
            (self.used_virtual_space as f32 / self.total_virtual_space as f32) * 100.0
        }
    }
}

/// Global virtual memory statistics
static mut VIRT_STATS: VirtualMemoryStats = VirtualMemoryStats::new();

/// Virtual memory areas table (simplified for Phase 1)
const MAX_VMAS: usize = 64;
static mut VMAS: [Option<VirtualMemoryArea>; MAX_VMAS] = [None; MAX_VMAS];
static mut VMA_COUNT: usize = 0;

/// Initialize virtual memory management
pub fn init() {
    kprintln!("[VIRT] Initializing virtual memory manager");
    
    // For Phase 1, we'll set up basic virtual memory areas
    // In a real implementation, this would:
    // 1. Set up kernel virtual address space
    // 2. Initialize virtual memory allocator
    // 3. Set up user address space templates
    // 4. Initialize memory mapping functions
    
    unsafe {
        // Set up initial kernel VMAs
        add_vma(VirtualMemoryArea::new(
            KERNEL_VIRT_START,
            KERNEL_VIRT_START + 0x1000000, // 16MB kernel space
            VmaType::KernelCode,
            VmaProtection::read_execute()
        ));
        
        add_vma(VirtualMemoryArea::new(
            KERNEL_VIRT_START + 0x1000000,
            KERNEL_VIRT_START + 0x2000000, // 16MB kernel heap
            VmaType::KernelHeap,
            VmaProtection::read_write()
        ));
        
        // Calculate virtual address space
        VIRT_STATS.total_virtual_space = 0x10000000000000; // 64-bit address space (theoretical)
        VIRT_STATS.kernel_virtual_space = 0x2000000; // 32MB initial kernel space
        VIRT_STATS.used_virtual_space = VIRT_STATS.kernel_virtual_space;
        VIRT_STATS.vma_count = VMA_COUNT as u64;
    }
    
    klog!(INFO, "[VIRT] Virtual memory manager initialized");
    klog!(INFO, "[VIRT] Kernel virtual space: {} MB", get_kernel_virtual_usage() / (1024 * 1024));
    
    // Test the VM API
    test_vm_api();
}

/// Add a VMA to the VMA table
unsafe fn add_vma(vma: VirtualMemoryArea) {
    if VMA_COUNT < MAX_VMAS {
        VMAS[VMA_COUNT] = Some(vma);
        VMA_COUNT += 1;
        klog!(TRACE, "[VIRT] Added VMA: 0x{:016x}-0x{:016x} ({}) {}", 
              vma.start, vma.end, vma.vma_type, vma.protection);
    }
}

/// Allocate virtual memory
/// 
/// # Arguments
/// * `size` - Size in bytes to allocate
/// * `vma_type` - Type of virtual memory area
/// * `protection` - Protection flags
/// 
/// # Returns
/// Virtual address of allocated memory or error
pub fn allocate_virtual(size: u64, vma_type: VmaType, protection: VmaProtection) -> MemoryResult<u64> {
    if size == 0 {
        return Err(MemoryError::InvalidAddress);
    }
    
    // Round up to page boundary
    let aligned_size = super::utils::round_up_to_page(size);
    
    // For Phase 1, we'll use a simple allocation strategy
    // In a real implementation, this would:
    // 1. Find a suitable virtual address range
    // 2. Check for conflicts with existing VMAs
    // 3. Create new VMA
    // 4. Optionally map physical pages
    
    let virt_addr = match vma_type {
        VmaType::KernelHeap | VmaType::KernelData | VmaType::KernelStack => {
            // Allocate in kernel space
            KERNEL_VIRT_START + get_kernel_virtual_usage()
        }
        _ => {
            // Allocate in user space
            0x400000 + get_user_virtual_usage()
        }
    };
    
    // Create VMA for this allocation
    let vma = VirtualMemoryArea::new(virt_addr, virt_addr + aligned_size, vma_type, protection);
    
    unsafe {
        add_vma(vma);
        VIRT_STATS.virtual_allocations += 1;
        VIRT_STATS.used_virtual_space += aligned_size;
        
        match vma_type {
            VmaType::KernelCode | VmaType::KernelData | VmaType::KernelHeap | VmaType::KernelStack => {
                VIRT_STATS.kernel_virtual_space += aligned_size;
            }
            _ => {
                VIRT_STATS.user_virtual_space += aligned_size;
            }
        }
    }
    
    klog!(TRACE, "[VIRT] Allocated {} bytes of {} at 0x{:016x}", 
          aligned_size, vma_type, virt_addr);
    
    Ok(virt_addr)
}

/// Deallocate virtual memory
/// 
/// # Arguments
/// * `addr` - Virtual address to deallocate
/// * `size` - Size of the allocation
/// 
/// # Returns
/// Result indicating success or error
pub fn deallocate_virtual(addr: u64, size: u64) -> MemoryResult<()> {
    if addr == 0 || size == 0 {
        return Err(MemoryError::InvalidAddress);
    }
    
    let aligned_size = super::utils::round_up_to_page(size);
    
    // Find and remove the VMA
    unsafe {
        for i in 0..VMA_COUNT {
            if let Some(vma) = VMAS[i] {
                if vma.start == addr && vma.size() >= aligned_size {
                    // Remove this VMA
                    VMAS[i] = None;
                    
                    // Compact the array
                    for j in i..VMA_COUNT - 1 {
                        VMAS[j] = VMAS[j + 1];
                    }
                    VMA_COUNT -= 1;
                    VMAS[VMA_COUNT] = None;
                    
                    // Update statistics
                    VIRT_STATS.virtual_deallocations += 1;
                    VIRT_STATS.used_virtual_space -= aligned_size;
                    
                    match vma.vma_type {
                        VmaType::KernelCode | VmaType::KernelData | VmaType::KernelHeap | VmaType::KernelStack => {
                            VIRT_STATS.kernel_virtual_space -= aligned_size;
                        }
                        _ => {
                            VIRT_STATS.user_virtual_space -= aligned_size;
                        }
                    }
                    
                    klog!(TRACE, "[VIRT] Deallocated {} bytes at 0x{:016x}", aligned_size, addr);
                    return Ok(());
                }
            }
        }
    }
    
    Err(MemoryError::InvalidAddress)
}

/// Find VMA containing the given address
/// 
/// # Arguments
/// * `addr` - Virtual address to search for
/// 
/// # Returns
/// VMA containing the address or None
pub fn find_vma(addr: u64) -> Option<VirtualMemoryArea> {
    unsafe {
        for i in 0..VMA_COUNT {
            if let Some(vma) = VMAS[i] {
                if vma.contains(addr) {
                    return Some(vma);
                }
            }
        }
    }
    None
}

/// Check if virtual address is valid and accessible
/// 
/// # Arguments
/// * `addr` - Virtual address to check
/// * `for_write` - Whether access is for writing
/// 
/// # Returns
/// True if address is valid and accessible
pub fn is_address_accessible(addr: u64, for_write: bool) -> bool {
    if let Some(vma) = find_vma(addr) {
        if for_write {
            vma.protection.write
        } else {
            vma.protection.read
        }
    } else {
        false
    }
}

/// Map virtual address to physical address
/// 
/// # Arguments
/// * `virtual_addr` - Virtual address to map
/// * `physical_addr` - Physical address to map to
/// * `size` - Size of mapping
/// * `protection` - Protection flags
/// 
/// # Returns
/// Result indicating success or error
pub fn map_virtual_to_physical(virtual_addr: u64, physical_addr: u64, size: u64, protection: VmaProtection) -> MemoryResult<()> {
    // This would integrate with the paging system
    // For Phase 1, we'll defer to the paging module
    
    let page_flags = super::paging::PageFlags {
        present: true,
        writable: protection.write,
        user_accessible: protection.user,
        executable: protection.execute,
        accessed: false,
        dirty: false,
    };
    
    let page_count = super::utils::bytes_to_pages(size);
    
    for i in 0..page_count {
        let virt_page = virtual_addr + i * PAGE_SIZE as u64;
        let phys_page = physical_addr + i * PAGE_SIZE as u64;
        
        super::paging::map_page(virt_page, phys_page, page_flags)?;
    }
    
    klog!(TRACE, "[VIRT] Mapped {} pages: 0x{:016x} -> 0x{:016x}", 
          page_count, virtual_addr, physical_addr);
    
    Ok(())
}

/// Get total virtual address space size
pub fn get_total_virtual_space() -> u64 {
    unsafe { VIRT_STATS.total_virtual_space }
}

/// Get used virtual address space
pub fn get_used_virtual_space() -> u64 {
    unsafe { VIRT_STATS.used_virtual_space }
}

/// Get kernel virtual space usage
pub fn get_kernel_virtual_usage() -> u64 {
    unsafe { VIRT_STATS.kernel_virtual_space }
}

/// Get user virtual space usage
pub fn get_user_virtual_usage() -> u64 {
    unsafe { VIRT_STATS.user_virtual_space }
}

/// Get virtual memory statistics
pub fn get_virtual_stats() -> VirtualMemoryStats {
    unsafe {
        VIRT_STATS.vma_count = VMA_COUNT as u64;
        VIRT_STATS
    }
}

/// Print virtual memory areas
pub fn print_virtual_areas() {
    kprintln!("");
    kprintln!("=== VIRTUAL MEMORY AREAS ===");
    
    unsafe {
        for i in 0..VMA_COUNT {
            if let Some(vma) = VMAS[i] {
                kprintln!("VMA {}: 0x{:016x}-0x{:016x} ({} KB) {} {}", 
                          i,
                          vma.start,
                          vma.end,
                          vma.size() / 1024,
                          vma.vma_type,
                          vma.protection);
            }
        }
    }
    
    kprintln!("=== END VIRTUAL MEMORY AREAS ===");
    kprintln!("");
}

/// Print virtual memory statistics
pub fn print_virtual_stats() {
    let stats = get_virtual_stats();
    
    kprintln!("");
    kprintln!("=== VIRTUAL MEMORY STATISTICS ===");
    kprintln!("Total virtual space: {} TB", stats.total_virtual_space / (1024 * 1024 * 1024 * 1024));
    kprintln!("Used virtual space: {} KB ({} MB)", stats.used_virtual_space / 1024, stats.used_virtual_space / (1024 * 1024));
    kprintln!("Kernel virtual space: {} KB ({} MB)", stats.kernel_virtual_space / 1024, stats.kernel_virtual_space / (1024 * 1024));
    kprintln!("User virtual space: {} KB ({} MB)", stats.user_virtual_space / 1024, stats.user_virtual_space / (1024 * 1024));
    kprintln!("VMAs: {}", stats.vma_count);
    kprintln!("Allocations: {}", stats.virtual_allocations);
    kprintln!("Deallocations: {}", stats.virtual_deallocations);
    kprintln!("Utilization: {:.6}%", stats.utilization());
    kprintln!("=== END VIRTUAL MEMORY STATISTICS ===");
    kprintln!("");
}

/// Test virtual memory management
pub fn test_virtual_memory() {
    kprintln!("Testing virtual memory management...");
    
    // Test virtual allocation
    match allocate_virtual(65536, VmaType::KernelHeap, VmaProtection::read_write()) {
        Ok(addr) => {
            kprintln!("  ✓ Allocated 64KB of kernel heap at 0x{:016x}", addr);
            
            // Test address validation
            if is_address_accessible(addr, true) {
                kprintln!("  ✓ Address is accessible for writing");
            } else {
                kprintln!("  ✗ Address is not accessible for writing");
            }
            
            // Test VMA lookup
            if let Some(vma) = find_vma(addr) {
                kprintln!("  ✓ Found VMA: {} {}", vma.vma_type, vma.protection);
            } else {
                kprintln!("  ✗ VMA not found");
            }
            
            // Test deallocation
            match deallocate_virtual(addr, 65536) {
                Ok(()) => kprintln!("  ✓ Deallocated virtual memory"),
                Err(e) => kprintln!("  ✗ Virtual deallocation failed: {}", e),
            }
        }
        Err(e) => kprintln!("  ✗ Virtual allocation failed: {}", e),
    }
    
    // Print VMAs and statistics
    print_virtual_areas();
    print_virtual_stats();
    
    kprintln!("Virtual memory test completed");
}

/// Test virtual memory API functionality
pub fn test_vm_api() {
    kprintln!("");
    kprintln!("=== VIRTUAL MEMORY API TESTS ===");
    
    let initial_stats = get_vm_stats();
    kprintln!("Initial state: {} active mappings", initial_stats.active_mappings);
    
    // Test basic mapping functionality
    kprintln!("Testing basic map/unmap functionality:");
    
    let test_vaddr = 0x400000; // 4MB
    let test_paddr = 0x1000000; // 16MB
    let flags = VmFlags::read_write();
    
    // Test mapping a page
    match map_page(test_vaddr, test_paddr, flags) {
        Ok(()) => {
            kprintln!("  ✓ Mapped page: 0x{:016x} -> 0x{:016x} ({})", test_vaddr, test_paddr, flags);
            
            // Verify the mapping exists
            if is_mapped(test_vaddr) {
                kprintln!("  ✓ Page is correctly marked as mapped");
            } else {
                kprintln!("  ✗ Page is not marked as mapped");
            }
            
            // Test translation
            match translate_va(test_vaddr) {
                Some(paddr) => {
                    if paddr == test_paddr {
                        kprintln!("  ✓ Translation correct: 0x{:016x} -> 0x{:016x}", test_vaddr, paddr);
                    } else {
                        kprintln!("  ✗ Translation incorrect: expected 0x{:016x}, got 0x{:016x}", test_paddr, paddr);
                    }
                }
                None => kprintln!("  ✗ Translation failed"),
            }
            
            // Test translation with offset
            let offset_vaddr = test_vaddr + 0x123;
            match translate_va(offset_vaddr) {
                Some(paddr) => {
                    let expected_paddr = test_paddr + 0x123;
                    if paddr == expected_paddr {
                        kprintln!("  ✓ Offset translation correct: 0x{:016x} -> 0x{:016x}", offset_vaddr, paddr);
                    } else {
                        kprintln!("  ✗ Offset translation incorrect: expected 0x{:016x}, got 0x{:016x}", expected_paddr, paddr);
                    }
                }
                None => kprintln!("  ✗ Offset translation failed"),
            }
            
            // Test unmapping
            match unmap_page(test_vaddr) {
                Ok(()) => {
                    kprintln!("  ✓ Successfully unmapped page");
                    
                    // Verify the mapping no longer exists
                    if !is_mapped(test_vaddr) {
                        kprintln!("  ✓ Page is correctly marked as unmapped");
                    } else {
                        kprintln!("  ✗ Page is still marked as mapped after unmap");
                    }
                    
                    // Test translation after unmap
                    match translate_va(test_vaddr) {
                        Some(_) => kprintln!("  ✗ Translation still works after unmap"),
                        None => kprintln!("  ✓ Translation correctly fails after unmap"),
                    }
                }
                Err(e) => kprintln!("  ✗ Failed to unmap page: {}", e),
            }
        }
        Err(e) => kprintln!("  ✗ Failed to map page: {}", e),
    }
    
    // Test double mapping error detection
    kprintln!("Testing double mapping error detection:");
    let double_vaddr = 0x500000;
    let double_paddr1 = 0x2000000;
    let double_paddr2 = 0x3000000;
    
    // Map the page first time
    match map_page(double_vaddr, double_paddr1, VmFlags::read_only()) {
        Ok(()) => {
            kprintln!("  ✓ First mapping successful");
            
            // Try to map it again (should fail)
            match map_page(double_vaddr, double_paddr2, VmFlags::read_write()) {
                Ok(()) => kprintln!("  ✗ Double mapping was allowed (should have failed)"),
                Err(MemoryError::AlreadyMapped) => kprintln!("  ✓ Double mapping correctly rejected"),
                Err(e) => kprintln!("  ✗ Double mapping failed with wrong error: {}", e),
            }
            
            // Clean up
            let _ = unmap_page(double_vaddr);
        }
        Err(e) => kprintln!("  ✗ First mapping failed: {}", e),
    }
    
    // Test double unmapping error detection  
    kprintln!("Testing double unmapping error detection:");
    let unmap_vaddr = 0x600000;
    let unmap_paddr = 0x4000000;
    
    // Map and then unmap
    if map_page(unmap_vaddr, unmap_paddr, VmFlags::read_write()).is_ok() {
        match unmap_page(unmap_vaddr) {
            Ok(()) => {
                kprintln!("  ✓ First unmap successful");
                
                // Try to unmap again (should fail)
                match unmap_page(unmap_vaddr) {
                    Ok(()) => kprintln!("  ✗ Double unmap was allowed (should have failed)"),
                    Err(MemoryError::NotMapped) => kprintln!("  ✓ Double unmap correctly rejected"),
                    Err(e) => kprintln!("  ✗ Double unmap failed with wrong error: {}", e),
                }
            }
            Err(e) => kprintln!("  ✗ First unmap failed: {}", e),
        }
    }
    
    // Test RWX flag enforcement simulation
    kprintln!("Testing RWX flag enforcement simulation:");
    
    let rwx_tests = [
        (0x700000, 0x5000000, VmFlags::read_only(), "read-only"),
        (0x701000, 0x5001000, VmFlags::read_write(), "read-write"),
        (0x702000, 0x5002000, VmFlags::read_execute(), "read-execute"),
        (0x703000, 0x5003000, VmFlags::read_write_execute(), "read-write-execute"),
        (0x704000, 0x5004000, VmFlags::user_read_only(), "user-read-only"),
        (0x705000, 0x5005000, VmFlags::user_read_write(), "user-read-write"),
        (0x706000, 0x5006000, VmFlags::user_read_execute(), "user-read-execute"),
    ];
    
    for &(vaddr, paddr, flags, description) in &rwx_tests {
        match map_page(vaddr, paddr, flags) {
            Ok(()) => {
                kprintln!("  ✓ Mapped {} page at 0x{:016x}", description, vaddr);
                
                // Test access checks
                let read_ok = check_access(vaddr, false, false, false);
                let write_ok = check_access(vaddr, true, false, false);
                let execute_ok = check_access(vaddr, false, true, false);
                let user_ok = check_access(vaddr, false, false, true);
                
                kprintln!("    Access checks: R={} W={} X={} U={} (expected: R={} W={} X={} U={})",
                          read_ok, write_ok, execute_ok, user_ok,
                          flags.read, flags.write, flags.execute, flags.user);
                
                // Verify flags match expectations
                if read_ok == flags.read && write_ok == flags.write && 
                   execute_ok == flags.execute && user_ok == flags.user {
                    kprintln!("    ✓ RWX enforcement correct for {}", description);
                } else {
                    kprintln!("    ✗ RWX enforcement incorrect for {}", description);
                }
                
                // Clean up
                let _ = unmap_page(vaddr);
            }
            Err(e) => kprintln!("  ✗ Failed to map {} page: {}", description, e),
        }
    }
    
    // Test large-scale mapping and unmapping
    kprintln!("Testing large-scale mapping and unmapping:");
    const NUM_TEST_PAGES: usize = 100;
    let mut mapped_addresses = Vec::new();
    
    // Map many pages
    for i in 0..NUM_TEST_PAGES {
        let vaddr = 0x800000 + (i * PAGE_SIZE);
        let paddr = 0x6000000 + (i * PAGE_SIZE);
        let flags = match i % 4 {
            0 => VmFlags::read_only(),
            1 => VmFlags::read_write(),
            2 => VmFlags::read_execute(),
            _ => VmFlags::read_write_execute(),
        };
        
        match map_page(vaddr as u64, paddr as u64, flags) {
            Ok(()) => mapped_addresses.push(vaddr as u64),
            Err(e) => {
                kprintln!("  Failed to map page {}: {}", i, e);
                break;
            }
        }
        
        if (i + 1) % 25 == 0 {
            kprintln!("  Mapped {} / {} pages", i + 1, NUM_TEST_PAGES);
        }
    }
    
    kprintln!("  ✓ Successfully mapped {} pages", mapped_addresses.len());
    
    // Verify all mappings exist
    let mut valid_count = 0;
    for &vaddr in &mapped_addresses {
        if is_mapped(vaddr) && translate_va(vaddr).is_some() {
            valid_count += 1;
        }
    }
    
    if valid_count == mapped_addresses.len() {
        kprintln!("  ✓ All {} mappings are valid", valid_count);
    } else {
        kprintln!("  ✗ Only {} out of {} mappings are valid", valid_count, mapped_addresses.len());
    }
    
    // Unmap all pages
    let mut unmapped_count = 0;
    for &vaddr in &mapped_addresses {
        if unmap_page(vaddr).is_ok() {
            unmapped_count += 1;
        }
    }
    
    kprintln!("  ✓ Successfully unmapped {} pages", unmapped_count);
    
    // Verify all mappings are gone
    let mut remaining_count = 0;
    for &vaddr in &mapped_addresses {
        if is_mapped(vaddr) {
            remaining_count += 1;
        }
    }
    
    if remaining_count == 0 {
        kprintln!("  ✓ All mappings successfully removed");
    } else {
        kprintln!("  ✗ {} mappings still remain", remaining_count);
    }
    
    // Validate virtual memory state
    if validate_vm() {
        kprintln!("  ✓ Virtual memory validation passed");
    } else {
        kprintln!("  ✗ Virtual memory validation failed");
    }
    
    // Check for memory leaks
    let final_stats = get_vm_stats();
    if final_stats.active_mappings == initial_stats.active_mappings {
        kprintln!("  ✓ No memory leaks detected");
    } else {
        kprintln!("  ✗ Memory leak: {} mappings still active", 
                  final_stats.active_mappings - initial_stats.active_mappings);
    }
    
    // Print final statistics
    kprintln!("");
    kprintln!("Final VM API statistics:");
    kprintln!("  Mapped pages: {}", final_stats.mapped_pages);
    kprintln!("  Unmapped pages: {}", final_stats.unmapped_pages);
    kprintln!("  Active mappings: {}", final_stats.active_mappings);
    kprintln!("  Failed mappings: {}", final_stats.failed_mappings);
    kprintln!("  Double map attempts: {}", final_stats.double_map_attempts);
    kprintln!("  Double unmap attempts: {}", final_stats.double_unmap_attempts);
    
    kprintln!("=== VIRTUAL MEMORY API TESTS COMPLETE ===");
    kprintln!("");
}
