# 4-Level Paging Implementation for x86_64 - Polymera OS

## Overview

This document describes the comprehensive 4-level paging implementation for Polymera OS on x86_64 architecture, including kernel memory mapping, guard pages, and hardware integration using the `x86_64` crate.

## Architecture Overview

### 4-Level Paging Structure

```
Virtual Address (64-bit):
┌─────────────┬──────────┬──────────┬──────────┬──────────┬─────────────┐
│   Sign Ext  │   PML4   │   PDPT   │    PD    │    PT    │   Offset    │
│  (16 bits)  │ (9 bits) │ (9 bits) │ (9 bits) │ (9 bits) │  (12 bits)  │
└─────────────┴──────────┴──────────┴──────────┴──────────┴─────────────┘
      63-48        47-39      38-30      29-21      20-12      11-0

Page Table Hierarchy:
PML4 (Page Map Level 4) → PDPT (Page Directory Pointer Table) 
  → PD (Page Directory) → PT (Page Table) → Physical Page
```

### Memory Layout

```
Virtual Address Space (x86_64):
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF: User Space (128TB)
0x0000_8000_0000_0000 - 0xFFFF_7FFF_FFFF_FFFF: Hole (non-canonical)
0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF: Kernel Space (128TB)

Kernel Virtual Memory Layout:
0xFFFF_8000_0000_0000 - 0xFFFF_8000_0020_0000: .text    (2MB, R-X)
0xFFFF_8000_0020_0000 - 0xFFFF_8000_0030_0000: .rodata  (1MB, R--)
0xFFFF_8000_0030_0000 - 0xFFFF_8000_0040_0000: .data    (1MB, RW-)
0xFFFF_8000_0040_0000 - 0xFFFF_8000_0050_0000: .bss     (1MB, RW-)
0xFFFF_8000_1000_0000 - 0xFFFF_8000_1001_0000: Kernel Stack (4KB, RW-)
```

## Implementation Details

### Page Flags System

```rust
pub struct PageFlags {
    pub present: bool,           // Page is present in memory
    pub writable: bool,          // Page is writable
    pub user_accessible: bool,   // Page accessible from user mode
    pub executable: bool,        // Page is executable (NX bit)
    pub accessed: bool,          // Page has been accessed
    pub dirty: bool,             // Page has been written to
    pub write_through: bool,     // Write-through caching
    pub cache_disabled: bool,    // Cache disabled
}
```

#### Pre-defined Flag Sets

```rust
// Kernel code section (.text)
PageFlags::kernel_code()     // Present, Read-only, Executable, Kernel-only

// Kernel read-only data (.rodata)  
PageFlags::kernel_rodata()   // Present, Read-only, Non-executable, Kernel-only

// Kernel read-write data (.data, .bss)
PageFlags::kernel_data()     // Present, Read-write, Non-executable, Kernel-only

// Kernel stack
PageFlags::kernel_stack()    // Present, Read-write, Non-executable, Kernel-only

// User code
PageFlags::user_code()       // Present, Read-only, Executable, User-accessible

// User data
PageFlags::user_data()       // Present, Read-write, Non-executable, User-accessible

// Guard pages
PageFlags::guard_page()      // Not present (triggers page fault)
```

### Kernel Section Mapping

```rust
static KERNEL_SECTIONS: &[KernelSection] = &[
    KernelSection::new(
        VirtAddr::new(KERNEL_VIRT_START),                // .text start
        VirtAddr::new(KERNEL_VIRT_START + 0x200000),     // .text end (2MB)
        PhysAddr::new(0x100000),                         // Physical at 1MB
        ".text",
        PageFlags::kernel_code(),                        // R-X permissions
    ),
    KernelSection::new(
        VirtAddr::new(KERNEL_VIRT_START + 0x200000),     // .rodata start
        VirtAddr::new(KERNEL_VIRT_START + 0x300000),     // .rodata end (1MB)
        PhysAddr::new(0x300000),                         // Physical at 3MB
        ".rodata", 
        PageFlags::kernel_rodata(),                      // R-- permissions
    ),
    // ... .data and .bss sections
];
```

### Frame Allocator

```rust
pub struct SimpleFrameAllocator {
    next_frame: PhysFrame,       // Next frame to allocate
    end_frame: PhysFrame,        // End of allocation region
    allocated_frames: AtomicU64, // Statistics counter
}

impl FrameAllocator<Size4KiB> for SimpleFrameAllocator {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame> {
        // Allocate next available frame
        // Update statistics
        // Return unused frame for mapping
    }
}
```

### Guard Pages Implementation

```rust
pub struct StackInfo {
    pub stack_bottom: VirtAddr,  // Stack bottom address
    pub stack_top: VirtAddr,     // Stack top address  
    pub name: &'static str,      // Stack name for debugging
}

impl StackInfo {
    pub fn guard_page_below(&self) -> VirtAddr {
        VirtAddr::new(self.stack_bottom.as_u64() - PAGE_SIZE as u64)
    }
    
    pub fn guard_page_above(&self) -> VirtAddr {
        self.stack_top
    }
}
```

#### Guard Page Protection

Guard pages are mapped with `PageFlags::guard_page()` which sets:
- `present: false` - Page not present in memory
- All other flags disabled

Accessing a guard page triggers a page fault with error code indicating:
- Protection violation (page not present)
- Exact fault address for debugging
- Stack overflow/underflow detection

### API Functions

#### Core Mapping Functions

```rust
/// Map virtual address to physical address
pub fn map_page(
    virtual_addr: VirtAddr, 
    physical_addr: PhysAddr, 
    flags: PageFlags
) -> MemoryResult<()>

/// Unmap virtual address
pub fn unmap_page(virtual_addr: VirtAddr) -> MemoryResult<()>

/// Translate virtual to physical address
pub fn translate_address(virtual_addr: VirtAddr) -> MemoryResult<PhysAddr>

/// Flush TLB for specific page or entire TLB
pub fn flush_tlb(virtual_addr: Option<VirtAddr>)
```

#### Page Fault Handling

```rust
/// Handle page fault with detailed error analysis
pub fn handle_page_fault(fault_addr: VirtAddr, error_code: u64) -> MemoryResult<()>
```

**Error Code Decoding:**
- Bit 0: Protection violation (1) vs page not present (0)
- Bit 1: Write access (1) vs read access (0)  
- Bit 2: User mode (1) vs kernel mode (0)
- Bit 3: Reserved bit violation (1)
- Bit 4: Instruction fetch (1) vs data access (0)

#### Statistics and Monitoring

```rust
pub struct PagingStats {
    pub total_pages: u64,           // Total pages in system
    pub allocated_pages: u64,       // Currently allocated pages
    pub free_pages: u64,            // Available pages
    pub page_table_pages: u64,      // Pages used for page tables
    pub page_faults: u64,           // Number of page faults
    pub tlb_flushes: u64,           // Number of TLB flushes
    pub kernel_pages_mapped: u64,   // Kernel pages mapped
    pub guard_pages_created: u64,   // Guard pages created
}
```

## Hardware Integration

### x86_64 Crate Integration

```rust
use x86_64::structures::paging::{
    PageTable, PageTableFlags, PhysFrame, Page, Size4KiB, 
    Mapper, FrameAllocator, UnusedPhysFrame, OffsetPageTable
};
use x86_64::registers::control::Cr3;
use x86_64::{PhysAddr, VirtAddr};
```

### CR3 Register Management

```rust
// Get current page table from CR3
let (level_4_table_frame, _) = Cr3::read();

// Create offset page table for safe access
let phys_offset = VirtAddr::new(0xFFFF_8000_0000_0000);
let level_4_table_ptr = (phys_offset + level_4_table_frame.start_address().as_u64()).as_mut_ptr();
let level_4_table = unsafe { &mut *(level_4_table_ptr as *mut PageTable) };
let mapper = unsafe { OffsetPageTable::new(level_4_table, phys_offset) };
```

### TLB Management

```rust
// Flush specific page
x86_64::instructions::tlb::flush(Page::containing_address(addr));

// Flush entire TLB
x86_64::instructions::tlb::flush_all();
```

### Safe Wrappers

All hardware interactions are wrapped in safe Rust interfaces:

```rust
/// Safe wrapper for page mapping
pub fn map_page(virtual_addr: VirtAddr, physical_addr: PhysAddr, flags: PageFlags) -> MemoryResult<()> {
    // Validate alignment
    if !virtual_addr.is_aligned(PAGE_SIZE as u64) || !physical_addr.is_aligned(PAGE_SIZE as u64) {
        return Err(MemoryError::AlignmentError);
    }
    
    // Convert to x86_64 types
    let page = Page::containing_address(virtual_addr);
    let frame = PhysFrame::containing_address(physical_addr);
    let page_table_flags = flags.to_x86_64_flags();
    
    // Perform mapping with automatic TLB flush
    let map_result = unsafe {
        state.mapper.map_to(page, frame, page_table_flags, &mut state.frame_allocator)
    };
    
    // Handle result and update statistics
    match map_result {
        Ok(mapping) => {
            mapping.flush();  // Automatic TLB flush
            // Update statistics
            Ok(())
        }
        Err(err) => Err(MemoryError::PageNotFound)
    }
}
```

## Initialization Sequence

### 1. Paging System Initialization

```rust
pub fn init() {
    // 1. Get current page table from CR3
    let (level_4_table_frame, _) = Cr3::read();
    
    // 2. Create offset page table for safe access
    let mapper = unsafe { OffsetPageTable::new(level_4_table, phys_offset) };
    
    // 3. Initialize frame allocator
    let frame_allocator = SimpleFrameAllocator::new(start_addr, end_addr);
    
    // 4. Set up global paging state
    *PAGING_STATE.lock() = Some(PagingState { mapper, frame_allocator });
    
    // 5. Map kernel sections
    map_kernel_sections()?;
    
    // 6. Create kernel stacks with guard pages
    create_kernel_stacks_with_guards()?;
}
```

### 2. Kernel Section Mapping

```rust
fn map_kernel_sections() -> MemoryResult<()> {
    for section in KERNEL_SECTIONS {
        let page_count = section.page_count();
        let mut current_virt = section.virt_start;
        let mut current_phys = section.phys_start;
        
        for _ in 0..page_count {
            map_page(current_virt, current_phys, section.flags)?;
            current_virt += PAGE_SIZE as u64;
            current_phys += PAGE_SIZE as u64;
        }
    }
}
```

### 3. Guard Page Creation

```rust
fn create_kernel_stacks_with_guards() -> MemoryResult<()> {
    let kernel_stacks = [
        StackInfo::new(stack_bottom, stack_top, "main_kernel_stack"),
        StackInfo::new(stack_bottom, stack_top, "interrupt_stack"),
        StackInfo::new(stack_bottom, stack_top, "exception_stack"),
    ];
    
    for stack in &kernel_stacks {
        // Map actual stack pages
        map_stack_pages(stack)?;
        
        // Create guard pages (not present)
        let guard_below = stack.guard_page_below();
        let guard_above = stack.guard_page_above();
        
        map_guard_page(guard_below, "below")?;
        map_guard_page(guard_above, "above")?;
    }
}
```

## Testing and Validation

### Comprehensive Test Suite

```rust
pub fn test_paging() {
    // 1. Test address translation for kernel sections
    for section in KERNEL_SECTIONS {
        let test_addr = section.virt_start;
        match translate_address(test_addr) {
            Ok(phys_addr) => println!("✓ {} translation successful", section.name),
            Err(e) => println!("✗ {} translation failed: {}", section.name, e),
        }
    }
    
    // 2. Test page mapping and unmapping
    let test_virt = VirtAddr::new(0xFFFF_8000_8000_0000);
    let test_phys = PhysAddr::new(0x8000000);
    
    map_page(test_virt, test_phys, PageFlags::kernel_data())?;
    // Test translation
    unmap_page(test_virt)?;
    // Verify unmapped
    
    // 3. Test TLB operations
    flush_tlb(Some(test_virt));  // Single page flush
    flush_tlb(None);             // Global flush
    
    // 4. Print comprehensive statistics
    print_paging_stats();
    print_kernel_mappings();
}
```

### Expected Test Output

```
=== PAGING FUNCTIONALITY TEST ===
Testing address translation:
  ✓ .text section: 0xFFFF800000000000 -> 0x0000000000100000
  ✓ .rodata section: 0xFFFF800000200000 -> 0x0000000000300000  
  ✓ .data section: 0xFFFF800000300000 -> 0x0000000000400000
  ✓ .bss section: 0xFFFF800000400000 -> 0x0000000000500000

Testing page mapping:
  ✓ Test page mapped successfully
  ✓ Test page translation correct
  ✓ Test page unmapped successfully
  ✓ Test page correctly unmapped

Testing TLB operations:
  ✓ Single page TLB flush completed
  ✓ Global TLB flush completed

=== PAGING STATISTICS ===
Page Management:
  Total pages: 262144
  Allocated pages: 1024
  Free pages: 261120
  Utilization: 0.4%

Kernel Mapping:
  Kernel pages mapped: 1280
  Guard pages created: 6
  Page table pages: 4

Frame Allocator:
  Allocated frames: 128
  Remaining frames: 28544

System Events:
  Page faults: 0
  TLB flushes: 12

=== KERNEL MEMORY MAPPINGS ===
.text section:
  Virtual:  0xFFFF800000000000 - 0xFFFF800000200000 (2048 KB)
  Physical: 0x0000000000100000 - 0x0000000000300000
  Flags: PRESENT READ_ONLY EXECUTABLE KERNEL
  Pages: 512

.rodata section:
  Virtual:  0xFFFF800000200000 - 0xFFFF800000300000 (1024 KB)
  Physical: 0x0000000000300000 - 0x0000000000400000
  Flags: PRESENT READ_ONLY NO_EXECUTE KERNEL
  Pages: 256
=== END PAGING TEST ===
```

## Performance Considerations

### TLB Optimization

- **Selective Flushing**: Only flush specific pages when possible
- **Batch Operations**: Group multiple mappings before TLB flush
- **Large Pages**: Future support for 2MB/1GB pages for kernel mappings

### Memory Efficiency

- **Page Table Sharing**: Share page tables between similar mappings
- **Lazy Allocation**: Allocate page tables on demand
- **Compaction**: Merge adjacent mappings where possible

### Security Features

- **NX Bit Enforcement**: Prevent execution of data pages
- **User/Kernel Separation**: Strict permission boundaries  
- **Guard Page Detection**: Immediate stack overflow detection
- **Write Protection**: Enforce read-only sections

## Integration Points

### Memory Manager Integration

```rust
// MM module exports paging functions
pub use paging::{map_page, unmap_page, flush_tlb, translate_address, PageFlags};
pub use x86_64::{VirtAddr, PhysAddr};
```

### Scheduler Integration

```rust
// Scheduler can create new address spaces
let new_page_table = create_user_page_table()?;
let task = Task::new_with_page_table(new_page_table);
```

### Exception Handling Integration

```rust
// IDT integration for page fault handling
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let fault_addr = x86_64::registers::control::Cr2::read();
    crate::mm::paging::handle_page_fault(fault_addr, error_code.bits()).unwrap_or_else(|_| {
        panic!("Unhandled page fault at {:?}", fault_addr);
    });
}
```

## Future Enhancements

### Phase 2 Features

1. **Advanced Page Table Management**:
   - Copy-on-write (COW) pages
   - Demand paging with swap support
   - Memory-mapped files

2. **Large Page Support**:
   - 2MB pages for kernel sections
   - 1GB pages for large data structures
   - Transparent huge pages

3. **NUMA Support**:
   - NUMA-aware page allocation
   - Page migration between nodes
   - Local memory optimization

4. **Security Enhancements**:
   - SMEP/SMAP support
   - Control Flow Integrity (CFI)
   - Address Space Layout Randomization (ASLR)

5. **Performance Optimizations**:
   - TLB shootdown optimization
   - Page table caching
   - Prefaulting for sequential access

This implementation provides a robust foundation for memory management in Polymera OS, with comprehensive hardware integration, safety guarantees, and extensive monitoring capabilities.

