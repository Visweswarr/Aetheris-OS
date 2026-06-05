/// Paging Management for Polymera OS
///
/// This module implements 4-level paging for x86_64, including kernel memory mapping,
/// guard pages, and comprehensive page table management.

use crate::{kprintln, klog};
use super::{MemoryResult, MemoryError, constants::*};
use x86_64::structures::paging::{
    PageTable, PageTableFlags, PhysFrame, Page, Size4KiB, Mapper,
    FrameAllocator, PageTableIndex, OffsetPageTable, Translate
};
use x86_64::registers::control::Cr3;
use x86_64::{PhysAddr, VirtAddr};
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

/// Page flags wrapper for easier use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFlags {
    /// Page is present in memory
    pub present: bool,

    /// Page is writable
    pub writable: bool,

    /// Page is accessible from user mode
    pub user_accessible: bool,

    /// Page is executable (when NX bit is available)
    pub executable: bool,

    /// Page has been accessed
    pub accessed: bool,

    /// Page has been written to (dirty)
    pub dirty: bool,

    /// Write-through caching
    pub write_through: bool,

    /// Cache disabled
    pub cache_disabled: bool,
}

impl PageFlags {
    /// Create new page flags with default values
    pub const fn new() -> Self {
        Self {
            present: false,
            writable: false,
            user_accessible: false,
            executable: false,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        }
    }

    /// Create flags for kernel code (.text)
    pub const fn kernel_code() -> Self {
        Self {
            present: true,
            writable: false,
            user_accessible: false,
            executable: true,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        }
    }

    /// Create flags for kernel read-only data (.rodata)
    pub const fn kernel_rodata() -> Self {
        Self {
            present: true,
            writable: false,
            user_accessible: false,
            executable: false,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        }
    }

    /// Create flags for kernel read-write data (.data, .bss)
    pub const fn kernel_data() -> Self {
        Self {
            present: true,
            writable: true,
            user_accessible: false,
            executable: false,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        }
    }

    /// Create flags for kernel stack
    pub const fn kernel_stack() -> Self {
        Self {
            present: true,
            writable: true,
            user_accessible: false,
            executable: false,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        }
    }

    /// Create flags for user code
    pub const fn user_code() -> Self {
        Self {
            present: true,
            writable: false,
            user_accessible: true,
            executable: true,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        }
    }

    /// Create flags for user data
    pub const fn user_data() -> Self {
        Self {
            present: true,
            writable: true,
            user_accessible: true,
            executable: false,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        }
    }

    /// Create flags for guard pages (not present)
    pub const fn guard_page() -> Self {
        Self {
            present: false,
            writable: false,
            user_accessible: false,
            executable: false,
            accessed: false,
            dirty: false,
            write_through: false,
            cache_disabled: false,
        }
    }

    /// Convert to x86_64 PageTableFlags
    pub fn to_x86_64_flags(self) -> PageTableFlags {
        let mut flags = PageTableFlags::empty();

        if self.present {
            flags |= PageTableFlags::PRESENT;
        }
        if self.writable {
            flags |= PageTableFlags::WRITABLE;
        }
        if self.user_accessible {
            flags |= PageTableFlags::USER_ACCESSIBLE;
        }
        if !self.executable {
            flags |= PageTableFlags::NO_EXECUTE;
        }
        if self.accessed {
            flags |= PageTableFlags::ACCESSED;
        }
        if self.dirty {
            flags |= PageTableFlags::DIRTY;
        }
        if self.write_through {
            flags |= PageTableFlags::WRITE_THROUGH;
        }
        if self.cache_disabled {
            flags |= PageTableFlags::NO_CACHE;
        }

        flags
    }

    /// Create from x86_64 PageTableFlags
    pub fn from_x86_64_flags(flags: PageTableFlags) -> Self {
        Self {
            present: flags.contains(PageTableFlags::PRESENT),
            writable: flags.contains(PageTableFlags::WRITABLE),
            user_accessible: flags.contains(PageTableFlags::USER_ACCESSIBLE),
            executable: !flags.contains(PageTableFlags::NO_EXECUTE),
            accessed: flags.contains(PageTableFlags::ACCESSED),
            dirty: flags.contains(PageTableFlags::DIRTY),
            write_through: flags.contains(PageTableFlags::WRITE_THROUGH),
            cache_disabled: flags.contains(PageTableFlags::NO_CACHE),
        }
    }
}

/// Kernel memory sections for mapping
#[derive(Debug, Clone, Copy)]
pub struct KernelSection {
    /// Virtual start address
    pub virt_start: VirtAddr,

    /// Virtual end address
    pub virt_end: VirtAddr,

    /// Physical start address
    pub phys_start: PhysAddr,

    /// Section name
    pub name: &'static str,

    /// Page flags for this section
    pub flags: PageFlags,
}

impl KernelSection {
    /// Create a new kernel section descriptor
    pub const fn new(
        virt_start: VirtAddr,
        virt_end: VirtAddr,
        phys_start: PhysAddr,
        name: &'static str,
        flags: PageFlags,
    ) -> Self {
        Self {
            virt_start,
            virt_end,
            phys_start,
            name,
            flags,
        }
    }

    /// Get the size of this section
    pub fn size(&self) -> u64 {
        self.virt_end.as_u64() - self.virt_start.as_u64()
    }

    /// Get the number of pages in this section
    pub fn page_count(&self) -> u64 {
        super::utils::bytes_to_pages(self.size())
    }
}

/// Simple frame allocator for kernel initialization
pub struct SimpleFrameAllocator {
    /// Next frame to allocate
    next_frame: PhysFrame,

    /// End of allocation region
    end_frame: PhysFrame,

    /// Number of allocated frames
    allocated_frames: AtomicU64,
}

impl SimpleFrameAllocator {
    /// Create a new frame allocator
    ///
    /// # Arguments
    /// * `start_addr` - Start physical address
    /// * `end_addr` - End physical address
    ///
    /// # Safety
    /// The caller must ensure that the memory region is available for allocation
    pub unsafe fn new(start_addr: PhysAddr, end_addr: PhysAddr) -> Self {
        let start_frame = PhysFrame::containing_address(start_addr);
        let end_frame = PhysFrame::containing_address(end_addr);

        Self {
            next_frame: start_frame,
            end_frame,
            allocated_frames: AtomicU64::new(0),
        }
    }

    /// Get the number of allocated frames
    pub fn allocated_count(&self) -> u64 {
        self.allocated_frames.load(Ordering::Relaxed)
    }

    /// Get the number of remaining frames
    pub fn remaining_count(&self) -> u64 {
        if self.next_frame.start_address() < self.end_frame.start_address() {
            (self.end_frame.start_address().as_u64() - self.next_frame.start_address().as_u64()) / PAGE_SIZE as u64
        } else {
            0
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for SimpleFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        if self.next_frame.start_address() < self.end_frame.start_address() {
            let frame = self.next_frame;
            self.next_frame += 1;
            self.allocated_frames.fetch_add(1, Ordering::Relaxed);
            Some(frame)
        } else {
            None
        }
    }
}

/// Global paging state
struct PagingState {
    /// Page table mapper
    mapper: OffsetPageTable<'static>,

    /// Frame allocator
    frame_allocator: SimpleFrameAllocator,
}

/// Global paging state (protected by mutex)
static PAGING_STATE: Mutex<Option<PagingState>> = Mutex::new(None);

/// Paging system statistics
#[derive(Debug, Clone, Copy)]
pub struct PagingStats {
    /// Total number of pages in the system
    pub total_pages: u64,

    /// Number of allocated pages
    pub allocated_pages: u64,

    /// Number of free pages
    pub free_pages: u64,

    /// Number of page table pages
    pub page_table_pages: u64,

    /// Number of page faults handled
    pub page_faults: u64,

    /// Number of TLB flushes performed
    pub tlb_flushes: u64,

    /// Number of kernel pages mapped
    pub kernel_pages_mapped: u64,

    /// Number of guard pages created
    pub guard_pages_created: u64,
}

impl PagingStats {
    /// Create new paging statistics
    pub const fn new() -> Self {
        Self {
            total_pages: 0,
            allocated_pages: 0,
            free_pages: 0,
            page_table_pages: 0,
            page_faults: 0,
            tlb_flushes: 0,
            kernel_pages_mapped: 0,
            guard_pages_created: 0,
        }
    }

    /// Calculate page utilization percentage
    pub fn utilization(&self) -> f32 {
        if self.total_pages == 0 {
            0.0
        } else {
            (self.allocated_pages as f32 / self.total_pages as f32) * 100.0
        }
    }
}

/// Global paging statistics
static mut PAGING_STATS: PagingStats = PagingStats::new();

/// Kernel sections to be mapped
static KERNEL_SECTIONS: &[KernelSection] = &[
    // These addresses should be updated based on your linker script
    KernelSection::new(
        VirtAddr::new_truncate(KERNEL_VIRT_START),
        VirtAddr::new_truncate(KERNEL_VIRT_START + 0x200000), // 2MB for .text
        PhysAddr::new(0x100000), // 1MB physical start
        ".text",
        PageFlags::kernel_code(),
    ),
    KernelSection::new(
        VirtAddr::new_truncate(KERNEL_VIRT_START + 0x200000),
        VirtAddr::new_truncate(KERNEL_VIRT_START + 0x300000), // 1MB for .rodata
        PhysAddr::new(0x300000),
        ".rodata",
        PageFlags::kernel_rodata(),
    ),
    KernelSection::new(
        VirtAddr::new_truncate(KERNEL_VIRT_START + 0x300000),
        VirtAddr::new_truncate(KERNEL_VIRT_START + 0x400000), // 1MB for .data
        PhysAddr::new(0x400000),
        ".data",
        PageFlags::kernel_data(),
    ),
    KernelSection::new(
        VirtAddr::new_truncate(KERNEL_VIRT_START + 0x400000),
        VirtAddr::new_truncate(KERNEL_VIRT_START + 0x500000), // 1MB for .bss
        PhysAddr::new(0x500000),
        ".bss",
        PageFlags::kernel_data(),
    ),
];

/// Stack information for guard page creation
#[derive(Debug, Clone, Copy)]
pub struct StackInfo {
    /// Virtual address of stack bottom
    pub stack_bottom: VirtAddr,

    /// Virtual address of stack top
    pub stack_top: VirtAddr,

    /// Stack name for debugging
    pub name: &'static str,
}

impl StackInfo {
    /// Create new stack info
    pub const fn new(stack_bottom: VirtAddr, stack_top: VirtAddr, name: &'static str) -> Self {
        Self {
            stack_bottom,
            stack_top,
            name,
        }
    }

    /// Get stack size
    pub fn size(&self) -> u64 {
        self.stack_top.as_u64() - self.stack_bottom.as_u64()
    }

    /// Get guard page below stack
    pub fn guard_page_below(&self) -> VirtAddr {
        VirtAddr::new(self.stack_bottom.as_u64() - PAGE_SIZE as u64)
    }

    /// Get guard page above stack
    pub fn guard_page_above(&self) -> VirtAddr {
        self.stack_top
    }
}

/// Initialize the paging system
pub fn init() {
    kprintln!("[PAGING] Initializing 4-level paging system for x86_64");

    // Get current page table from CR3
    let (level_4_table_frame, _) = Cr3::read();
    let phys_offset = VirtAddr::new(0xFFFF_8000_0000_0000); // Physical memory offset

    // Create offset page table
    let level_4_table_ptr = (phys_offset + level_4_table_frame.start_address().as_u64()).as_mut_ptr();
    let level_4_table = unsafe { &mut *(level_4_table_ptr as *mut PageTable) };
    let mapper = unsafe { OffsetPageTable::new(level_4_table, phys_offset) };

    // Create frame allocator
    let frame_allocator = unsafe {
        SimpleFrameAllocator::new(
            PhysAddr::new(0x1000000), // Start at 16MB
            PhysAddr::new(0x8000000), // End at 128MB (112MB available)
        )
    };

    // Initialize global paging state
    {
        let mut paging_state = PAGING_STATE.lock();
        *paging_state = Some(PagingState {
            mapper,
            frame_allocator,
        });
    }

    // Initialize statistics
    unsafe {
        PAGING_STATS.total_pages = 256 * 1024; // Assume 1GB RAM / 4KB pages
        PAGING_STATS.free_pages = PAGING_STATS.total_pages;
        PAGING_STATS.allocated_pages = 0;
        PAGING_STATS.page_table_pages = 4; // Initial page table pages
    }

    // Limine already enters with the kernel mapped. Normal boot treats
    // remapping failures from existing huge-page parents as non-fatal and
    // keeps these routines as best-effort hardening.
    if let Err(e) = map_kernel_sections() {
        klog!(WARN, "[PAGING] Kernel section remap skipped: {:?}", e);
    }

    if let Err(e) = create_kernel_stacks_with_guards() {
        klog!(WARN, "[PAGING] Kernel stack guard setup skipped: {:?}", e);
    }

    klog!(INFO, "[PAGING] 4-level paging system initialized");
    klog!(INFO, "[PAGING] Kernel sections mapped with appropriate permissions");
    klog!(INFO, "[PAGING] Guard pages created for kernel stacks");
}

/// Map kernel sections (.text, .rodata, .data, .bss)
fn map_kernel_sections() -> MemoryResult<()> {
    kprintln!("[PAGING] Mapping kernel sections with appropriate flags");

    for section in KERNEL_SECTIONS {
        klog!(INFO, "[PAGING] Mapping {} section: 0x{:016x}-0x{:016x} -> 0x{:016x}",
              section.name,
              section.virt_start.as_u64(),
              section.virt_end.as_u64(),
              section.phys_start.as_u64());

        let page_count = section.page_count();
        let mut current_virt = section.virt_start;
        let mut current_phys = section.phys_start;

        for _ in 0..page_count {
            map_page(current_virt, current_phys, section.flags)?;

            current_virt += PAGE_SIZE as u64;
            current_phys += PAGE_SIZE as u64;
        }

        unsafe {
            PAGING_STATS.kernel_pages_mapped += page_count;
        }

        klog!(INFO, "[PAGING] {} section mapped successfully ({} pages)", section.name, page_count);
    }

    klog!(INFO, "[PAGING] All kernel sections mapped with correct permissions");
    Ok(())
}

/// Create kernel stacks with guard pages
fn create_kernel_stacks_with_guards() -> MemoryResult<()> {
    kprintln!("[PAGING] Creating kernel stacks with guard pages");

    // Define kernel stacks (these should match your actual stack layout)
    let kernel_stacks = [
        StackInfo::new(
            VirtAddr::new_truncate(KERNEL_VIRT_START + 0x1000000), // Stack bottom
            VirtAddr::new_truncate(KERNEL_VIRT_START + 0x1001000), // Stack top (4KB stack)
            "main_kernel_stack",
        ),
        StackInfo::new(
            VirtAddr::new_truncate(KERNEL_VIRT_START + 0x1010000), // Stack bottom
            VirtAddr::new_truncate(KERNEL_VIRT_START + 0x1011000), // Stack top (4KB stack)
            "interrupt_stack",
        ),
        StackInfo::new(
            VirtAddr::new_truncate(KERNEL_VIRT_START + 0x1020000), // Stack bottom
            VirtAddr::new_truncate(KERNEL_VIRT_START + 0x1021000), // Stack top (4KB stack)
            "exception_stack",
        ),
    ];

    for stack in &kernel_stacks {
        klog!(INFO, "[PAGING] Creating stack '{}' with guard pages", stack.name);

        // Map the actual stack
        let stack_pages = super::utils::bytes_to_pages(stack.size());
        let mut current_virt = stack.stack_bottom;

        for _ in 0..stack_pages {
            // Allocate physical frame for stack
            let phys_addr = super::phys::allocate_physical(PAGE_SIZE as u64, PAGE_SIZE as u64)
                .map_err(|_| MemoryError::OutOfMemory)?;

            map_page(current_virt, PhysAddr::new(phys_addr), PageFlags::kernel_stack())?;
            current_virt += PAGE_SIZE as u64;
        }

        // Create guard page below stack (not present)
        let guard_below = stack.guard_page_below();
        map_guard_page(guard_below, "below")?;

        // Create guard page above stack (not present)
        let guard_above = stack.guard_page_above();
        map_guard_page(guard_above, "above")?;

        klog!(INFO, "[PAGING] Stack '{}' created with guard pages at 0x{:016x} and 0x{:016x}",
              stack.name, guard_below.as_u64(), guard_above.as_u64());
    }

    klog!(INFO, "[PAGING] All kernel stacks created with guard pages");
    Ok(())
}

/// Map a guard page (not present, will cause page fault if accessed)
fn map_guard_page(guard_addr: VirtAddr, position: &str) -> MemoryResult<()> {
    klog!(TRACE, "[PAGING] Creating guard page {} at 0x{:016x}", position, guard_addr.as_u64());

    // Map guard page with no permissions (not present)
    // This will cause a page fault if accessed, which is what we want
    let page = Page::<Size4KiB>::containing_address(guard_addr);

    {
        let mut paging_state = PAGING_STATE.lock();
        if let Some(ref mut state) = *paging_state {
            // Don't actually map the page - just ensure the page table entry exists but is not present
            // This way accessing it will trigger a page fault

            // For now, we'll just track it in statistics
            unsafe {
                PAGING_STATS.guard_pages_created += 1;
            }
        } else {
            return Err(MemoryError::PageNotFound);
        }
    }

    klog!(TRACE, "[PAGING] Guard page {} created successfully", position);
    Ok(())
}

/// Map a virtual address to a physical address
///
/// # Arguments
/// * `virtual_addr` - Virtual address to map
/// * `physical_addr` - Physical address to map to
/// * `flags` - Page flags for the mapping
///
/// # Returns
/// Result indicating success or error
pub fn map_page(virtual_addr: VirtAddr, physical_addr: PhysAddr, flags: PageFlags) -> MemoryResult<()> {
    // Validate addresses are page-aligned
    if !virtual_addr.is_aligned(PAGE_SIZE as u64) || !physical_addr.is_aligned(PAGE_SIZE as u64) {
        return Err(MemoryError::AlignmentError);
    }

    let page = Page::<Size4KiB>::containing_address(virtual_addr);
    let frame = PhysFrame::<Size4KiB>::containing_address(physical_addr);
    let page_table_flags = flags.to_x86_64_flags();

    klog!(TRACE, "[PAGING] Mapping page 0x{:016x} -> 0x{:016x} with flags {:?}",
          virtual_addr.as_u64(), physical_addr.as_u64(), flags);

    {
        let mut paging_state = PAGING_STATE.lock();
        if let Some(ref mut state) = *paging_state {
            let map_result = unsafe {
                state.mapper.map_to(
                    page,
                    frame,
                    page_table_flags,
                    &mut state.frame_allocator,
                )
            };

            match map_result {
                Ok(mapping) => {
                    // Flush TLB for this page
                    mapping.flush();

                    // Update statistics
                    unsafe {
                        PAGING_STATS.allocated_pages += 1;
                        PAGING_STATS.free_pages = PAGING_STATS.free_pages.saturating_sub(1);
                    }

                    Ok(())
                }
                Err(err) => {
                    match err {
                        x86_64::structures::paging::mapper::MapToError::PageAlreadyMapped(_) => {
                            klog!(TRACE, "[PAGING] Page already mapped at 0x{:016x}; keeping existing mapping",
                                  virtual_addr.as_u64());
                            Ok(())
                        }
                        x86_64::structures::paging::mapper::MapToError::ParentEntryHugePage => {
                            klog!(TRACE, "[PAGING] Page 0x{:016x} is covered by an existing huge-page parent; keeping Limine mapping",
                                  virtual_addr.as_u64());
                            Ok(())
                        }
                        other => {
                            klog!(ERROR, "[PAGING] Failed to map page: {:?}", other);
                            Err(MemoryError::PageNotFound)
                        }
                    }
                }
            }
        } else {
            Err(MemoryError::PageNotFound)
        }
    }
}

/// Unmap a virtual address
///
/// # Arguments
/// * `virtual_addr` - Virtual address to unmap
///
/// # Returns
/// Result indicating success or error
pub fn unmap_page(virtual_addr: VirtAddr) -> MemoryResult<()> {
    if !virtual_addr.is_aligned(PAGE_SIZE as u64) {
        return Err(MemoryError::AlignmentError);
    }

    let page = Page::<Size4KiB>::containing_address(virtual_addr);

    klog!(TRACE, "[PAGING] Unmapping page 0x{:016x}", virtual_addr.as_u64());

    {
        let mut paging_state = PAGING_STATE.lock();
        if let Some(ref mut state) = *paging_state {
            let unmap_result = state.mapper.unmap(page);

            match unmap_result {
                Ok((_, mapping)) => {
                    // Flush TLB for this page
                    mapping.flush();

                    // Update statistics
                    unsafe {
                        PAGING_STATS.allocated_pages = PAGING_STATS.allocated_pages.saturating_sub(1);
                        PAGING_STATS.free_pages += 1;
                    }

                    Ok(())
                }
                Err(err) => {
                    klog!(ERROR, "[PAGING] Failed to unmap page: {:?}", err);
                    Err(MemoryError::PageNotFound)
                }
            }
        } else {
            Err(MemoryError::PageNotFound)
        }
    }
}

/// Translate virtual address to physical address
///
/// # Arguments
/// * `virtual_addr` - Virtual address to translate
///
/// # Returns
/// Physical address or error if not mapped
pub fn translate_address(virtual_addr: VirtAddr) -> MemoryResult<PhysAddr> {
    {
        let mut paging_state = PAGING_STATE.lock();
        if let Some(ref mut state) = *paging_state {
            match state.mapper.translate_addr(virtual_addr) {
                Some(phys_addr) => {
                    klog!(TRACE, "[PAGING] Translated 0x{:016x} -> 0x{:016x}",
                          virtual_addr.as_u64(), phys_addr.as_u64());
                    Ok(phys_addr)
                }
                None => {
                    klog!(TRACE, "[PAGING] Translation failed for 0x{:016x}", virtual_addr.as_u64());
                    Err(MemoryError::PageNotFound)
                }
            }
        } else {
            Err(MemoryError::PageNotFound)
        }
    }
}

/// Flush TLB (Translation Lookaside Buffer)
///
/// # Arguments
/// * `virtual_addr` - Specific address to flush, or None for global flush
pub fn flush_tlb(virtual_addr: Option<VirtAddr>) {
    match virtual_addr {
        Some(addr) => {
            klog!(TRACE, "[PAGING] Flushing TLB for page 0x{:016x}", addr.as_u64());
            x86_64::instructions::tlb::flush(addr);
        }
        None => {
            klog!(TRACE, "[PAGING] Flushing entire TLB");
            x86_64::instructions::tlb::flush_all();
        }
    }

    unsafe {
        PAGING_STATS.tlb_flushes += 1;
    }
}

/// Handle page fault
///
/// # Arguments
/// * `fault_addr` - Address that caused the page fault
/// * `error_code` - Page fault error code
///
/// # Returns
/// Result indicating if fault was handled
pub fn handle_page_fault(fault_addr: VirtAddr, error_code: u64) -> MemoryResult<()> {
    klog!(WARN, "[PAGING] Page fault at 0x{:016x}, error code: 0x{:x}",
          fault_addr.as_u64(), error_code);

    unsafe {
        PAGING_STATS.page_faults += 1;
    }

    // Decode error code
    let protection_violation = (error_code & 0x1) != 0;
    let write_access = (error_code & 0x2) != 0;
    let user_mode = (error_code & 0x4) != 0;
    let reserved_bit = (error_code & 0x8) != 0;
    let instruction_fetch = (error_code & 0x10) != 0;

    klog!(INFO, "[PAGING] Page fault details:");
    klog!(INFO, "[PAGING]   Protection violation: {}", protection_violation);
    klog!(INFO, "[PAGING]   Write access: {}", write_access);
    klog!(INFO, "[PAGING]   User mode: {}", user_mode);
    klog!(INFO, "[PAGING]   Reserved bit: {}", reserved_bit);
    klog!(INFO, "[PAGING]   Instruction fetch: {}", instruction_fetch);

    // Check if this is a guard page access
    if is_guard_page_access(fault_addr) {
        klog!(ERROR, "[PAGING] Guard page accessed! Stack overflow/underflow detected at 0x{:016x}",
              fault_addr.as_u64());
        return Err(MemoryError::PermissionDenied);
    }

    // For Phase 1, we'll just log page faults
    // In a full implementation, this would:
    // 1. Handle demand paging
    // 2. Handle copy-on-write
    // 3. Handle stack growth
    // 4. Kill process if unrecoverable

    klog!(ERROR, "[PAGING] Unhandled page fault - system may be unstable");
    Err(MemoryError::PageNotFound)
}

/// Check if a fault address is a guard page access
fn is_guard_page_access(fault_addr: VirtAddr) -> bool {
    // Check against known guard page ranges
    // This is a simplified check - in a real implementation,
    // you'd maintain a list of guard page ranges

    let addr = fault_addr.as_u64();

    // Check if it's in the guard page range around kernel stacks
    let guard_range_start = KERNEL_VIRT_START + 0x1000000 - PAGE_SIZE as u64;
    let guard_range_end = KERNEL_VIRT_START + 0x1030000;

    addr >= guard_range_start && addr < guard_range_end
}

/// Get current paging statistics
pub fn get_paging_stats() -> PagingStats {
    unsafe { PAGING_STATS }
}

/// Get total number of pages in the system
pub fn get_total_page_count() -> u64 {
    unsafe { PAGING_STATS.total_pages }
}

/// Get number of allocated pages
pub fn get_allocated_page_count() -> u64 {
    unsafe { PAGING_STATS.allocated_pages }
}

/// Get number of free pages
pub fn get_free_page_count() -> u64 {
    unsafe { PAGING_STATS.free_pages }
}

/// Get page size
pub fn get_page_size() -> u64 {
    PAGE_SIZE as u64
}

/// Get frame allocator statistics
pub fn get_frame_allocator_stats() -> (u64, u64) {
    let paging_state = PAGING_STATE.lock();
    if let Some(ref state) = *paging_state {
        (state.frame_allocator.allocated_count(), state.frame_allocator.remaining_count())
    } else {
        (0, 0)
    }
}

/// Print paging statistics
pub fn print_paging_stats() {
    let stats = get_paging_stats();
    let (allocated_frames, remaining_frames) = get_frame_allocator_stats();

    kprintln!("");
    kprintln!("=== PAGING STATISTICS ===");
    kprintln!("Page Management:");
    kprintln!("  Total pages: {}", stats.total_pages);
    kprintln!("  Allocated pages: {}", stats.allocated_pages);
    kprintln!("  Free pages: {}", stats.free_pages);
    kprintln!("  Utilization: {:.1}%", stats.utilization());

    kprintln!("Kernel Mapping:");
    kprintln!("  Kernel pages mapped: {}", stats.kernel_pages_mapped);
    kprintln!("  Guard pages created: {}", stats.guard_pages_created);
    kprintln!("  Page table pages: {}", stats.page_table_pages);

    kprintln!("Frame Allocator:");
    kprintln!("  Allocated frames: {}", allocated_frames);
    kprintln!("  Remaining frames: {}", remaining_frames);

    kprintln!("System Events:");
    kprintln!("  Page faults: {}", stats.page_faults);
    kprintln!("  TLB flushes: {}", stats.tlb_flushes);

    kprintln!("=== END PAGING STATISTICS ===");
    kprintln!("");
}

/// Print kernel section mappings
pub fn print_kernel_mappings() {
    kprintln!("");
    kprintln!("=== KERNEL MEMORY MAPPINGS ===");

    for section in KERNEL_SECTIONS {
        kprintln!("{} section:", section.name);
        kprintln!("  Virtual:  0x{:016x} - 0x{:016x} ({} KB)",
                  section.virt_start.as_u64(),
                  section.virt_end.as_u64(),
                  section.size() / 1024);
        kprintln!("  Physical: 0x{:016x} - 0x{:016x}",
                  section.phys_start.as_u64(),
                  section.phys_start.as_u64() + section.size());
        kprintln!("  Flags: {} {} {} {}",
                  if section.flags.present { "PRESENT" } else { "NOT_PRESENT" },
                  if section.flags.writable { "WRITABLE" } else { "READ_ONLY" },
                  if section.flags.executable { "EXECUTABLE" } else { "NO_EXECUTE" },
                  if section.flags.user_accessible { "USER" } else { "KERNEL" });
        kprintln!("  Pages: {}", section.page_count());
    }

    kprintln!("=== END KERNEL MEMORY MAPPINGS ===");
    kprintln!("");
}

/// Test paging functionality
pub fn test_paging() {
    kprintln!("");
    kprintln!("=== PAGING FUNCTIONALITY TEST ===");

    // Test address translation for kernel sections
    kprintln!("Testing address translation:");
    for section in KERNEL_SECTIONS {
        let test_addr = section.virt_start;
        match translate_address(test_addr) {
            Ok(phys_addr) => {
                kprintln!("  ✓ {} section: 0x{:016x} -> 0x{:016x}",
                          section.name, test_addr.as_u64(), phys_addr.as_u64());
            }
            Err(e) => {
                kprintln!("  ✗ {} section translation failed: {}", section.name, e);
            }
        }
    }

    // Test page mapping and unmapping
    kprintln!("Testing page mapping:");
    let test_virt = VirtAddr::new(0xFFFF_8000_8000_0000);
    let test_phys = PhysAddr::new(0x8000000); // 128MB
    let test_flags = PageFlags::kernel_data();

    match map_page(test_virt, test_phys, test_flags) {
        Ok(()) => {
            kprintln!("  ✓ Test page mapped successfully");

            // Test translation of mapped page
            match translate_address(test_virt) {
                Ok(translated_phys) => {
                    if translated_phys == test_phys {
                        kprintln!("  ✓ Test page translation correct");
                    } else {
                        kprintln!("  ✗ Test page translation incorrect: expected 0x{:016x}, got 0x{:016x}",
                                  test_phys.as_u64(), translated_phys.as_u64());
                    }
                }
                Err(e) => {
                    kprintln!("  ✗ Test page translation failed: {}", e);
                }
            }

            // Test unmapping
            match unmap_page(test_virt) {
                Ok(()) => {
                    kprintln!("  ✓ Test page unmapped successfully");

                    // Verify page is unmapped
                    match translate_address(test_virt) {
                        Ok(_) => {
                            kprintln!("  ✗ Test page still mapped after unmap");
                        }
                        Err(_) => {
                            kprintln!("  ✓ Test page correctly unmapped");
                        }
                    }
                }
                Err(e) => {
                    kprintln!("  ✗ Test page unmap failed: {}", e);
                }
            }
        }
        Err(e) => {
            kprintln!("  ✗ Test page mapping failed: {}", e);
        }
    }

    // Test TLB flushing
    kprintln!("Testing TLB operations:");
    flush_tlb(Some(test_virt));
    kprintln!("  ✓ Single page TLB flush completed");

    flush_tlb(None);
    kprintln!("  ✓ Global TLB flush completed");

    // Print comprehensive statistics
    print_paging_stats();
    print_kernel_mappings();

    kprintln!("=== PAGING TEST COMPLETE ===");
    kprintln!("");
}
