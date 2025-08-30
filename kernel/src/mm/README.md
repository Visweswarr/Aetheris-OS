# Memory Management Module (MM) - Polymera OS

## Overview

The Memory Management (MM) module provides comprehensive memory management functionality for Polymera OS, including physical memory allocation, virtual memory management, paging, and various memory allocators.

## Module Structure

```
kernel/src/mm/
├── mod.rs           - Main MM module with init_mm() and utilities
├── paging.rs        - Paging system and page table management  
├── phys.rs          - Physical memory management
├── virt.rs          - Virtual memory management
├── alloc.rs         - Memory allocators (bump, block, etc.)
└── README.md        - This documentation
```

## Initialization Sequence

The MM module is initialized early in the boot sequence through `init_mm()`:

```rust
pub fn init_mm() {
    // 1. Initialize physical memory management
    phys::init();
    
    // 2. Initialize paging system
    paging::init();
    
    // 3. Initialize virtual memory management
    virt::init();
    
    // 4. Initialize memory allocators
    alloc::init();
}
```

### Boot Integration

```rust
// In boot.rs
pub fn init() {
    // HAL initialization first
    X64Hal::init_cpu();
    X64Hal::init_timer();
    X64Hal::enable_interrupts();
    
    // MM initialization early in sequence
    crate::mm::init_mm();  // ← Added here
    
    // Then scheduler, syscalls, demo tasks...
}
```

## Submodule Details

### 1. Physical Memory Management (`phys.rs`)

**Purpose**: Manages physical memory allocation and tracking.

**Key Features**:
- Memory region tracking with type classification
- Physical page allocation/deallocation
- Memory statistics and utilization tracking
- Support for different memory types (Available, Reserved, Kernel, DMA, etc.)

**API**:
```rust
// Allocate physical memory
pub fn allocate_physical(size: u64, align: u64) -> MemoryResult<u64>
pub fn deallocate_physical(addr: u64, size: u64) -> MemoryResult<()>

// Page-based allocation
pub fn allocate_pages(page_count: u64) -> MemoryResult<u64>
pub fn deallocate_pages(addr: u64, page_count: u64) -> MemoryResult<()>

// Statistics and information
pub fn get_total_memory() -> u64
pub fn get_available_memory() -> u64
pub fn print_memory_regions()
```

**Sample Memory Layout** (Phase 1):
```
0x0000_0000 - 0x0009_F000: Available (640KB)
0x0009_F000 - 0x000A_0000: Reserved (4KB) 
0x0010_0000 - 0x4000_0000: Available (1GB)
```

### 2. Paging System (`paging.rs`)

**Purpose**: Handles virtual-to-physical address translation and page table management.

**Key Features**:
- Page table entry management with flags
- Virtual address mapping/unmapping
- TLB management and flushing
- Page fault handling framework
- Page statistics tracking

**API**:
```rust
// Page mapping
pub fn map_page(virtual_addr: u64, physical_addr: u64, flags: PageFlags) -> MemoryResult<()>
pub fn unmap_page(virtual_addr: u64) -> MemoryResult<()>

// Address translation
pub fn translate_address(virtual_addr: u64) -> MemoryResult<u64>

// TLB management
pub fn flush_tlb(virtual_addr: Option<u64>)

// Page fault handling
pub fn handle_page_fault(fault_addr: u64, error_code: u64) -> MemoryResult<()>
```

**Page Flags**:
```rust
pub struct PageFlags {
    pub present: bool,        // Page is present in memory
    pub writable: bool,       // Page is writable
    pub user_accessible: bool, // Page accessible from user mode
    pub executable: bool,     // Page is executable
    pub accessed: bool,       // Page has been accessed
    pub dirty: bool,         // Page has been written to
}
```

### 3. Virtual Memory Management (`virt.rs`)

**Purpose**: Manages virtual address spaces and Virtual Memory Areas (VMAs).

**Key Features**:
- Virtual Memory Area (VMA) tracking
- Virtual address space allocation
- Protection and permission management
- User/kernel address space separation
- VMA type classification

**API**:
```rust
// Virtual memory allocation
pub fn allocate_virtual(size: u64, vma_type: VmaType, protection: VmaProtection) -> MemoryResult<u64>
pub fn deallocate_virtual(addr: u64, size: u64) -> MemoryResult<()>

// VMA management
pub fn find_vma(addr: u64) -> Option<VirtualMemoryArea>
pub fn is_address_accessible(addr: u64, for_write: bool) -> bool

// Virtual-to-physical mapping
pub fn map_virtual_to_physical(virtual_addr: u64, physical_addr: u64, size: u64, protection: VmaProtection) -> MemoryResult<()>
```

**VMA Types**:
- `KernelCode`, `KernelData`, `KernelHeap`, `KernelStack`
- `UserCode`, `UserData`, `UserHeap`, `UserStack`
- `Shared`, `Device`

**Protection Flags**:
```rust
pub struct VmaProtection {
    pub read: bool,     // Memory is readable
    pub write: bool,    // Memory is writable  
    pub execute: bool,  // Memory is executable
    pub user: bool,     // Memory is user accessible
}
```

### 4. Memory Allocators (`alloc.rs`)

**Purpose**: Provides various memory allocation strategies for different use cases.

**Key Features**:
- Bump allocator for early boot allocation
- Block allocator for fixed-size allocations
- Allocation statistics and monitoring
- Global allocator interface

**API**:
```rust
// Kernel memory allocation
pub fn kmalloc(size: usize, align: usize) -> MemoryResult<*mut u8>
pub fn kfree(ptr: *mut u8, size: usize, align: usize) -> MemoryResult<()>
pub fn kcalloc(size: usize, align: usize) -> MemoryResult<*mut u8>
pub fn krealloc(ptr: *mut u8, old_size: usize, new_size: usize, align: usize) -> MemoryResult<*mut u8>

// Statistics
pub fn get_allocator_stats() -> (AllocatorStats, AllocatorStats)
pub fn print_allocator_stats()
```

**Allocator Types**:

1. **Bump Allocator**:
   - Simple forward-only allocation
   - No deallocation support
   - Ideal for early boot phase
   - 16MB allocation space at 0x2000000

2. **Block Allocator**:
   - Fixed-size block allocation (64 bytes)
   - Efficient for small allocations
   - Bitmap-based free list
   - 16MB allocation space at 0x3000000

## Memory Layout

### Address Space Organization

```
Virtual Address Space (x86_64):
0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF: User Space (128TB)
0x0000_8000_0000_0000 - 0xFFFF_7FFF_FFFF_FFFF: Hole (non-canonical)
0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF: Kernel Space (128TB)
```

### Kernel Virtual Memory Layout

```
0xFFFF_8000_0000_0000 - 0xFFFF_8000_1000_0000: Kernel Code (16MB)
0xFFFF_8000_1000_0000 - 0xFFFF_8000_2000_0000: Kernel Heap (16MB)
0xFFFF_8000_2000_0000 - 0xFFFF_8000_3000_0000: Bump Allocator (16MB)
0xFFFF_8000_3000_0000 - 0xFFFF_8000_4000_0000: Block Allocator (16MB)
```

### Physical Memory Layout (Sample)

```
0x0000_0000 - 0x0009_F000: Low Memory (640KB) - Available
0x0009_F000 - 0x000A_0000: Reserved (4KB) - BIOS/VGA
0x0010_0000 - 0x1000_0000: Low Extended (15MB) - Available  
0x1000_0000 - 0x2000_0000: Kernel (16MB) - Reserved
0x2000_0000 - 0x3000_0000: Bump Allocator (16MB) - Allocated
0x3000_0000 - 0x4000_0000: Block Allocator (16MB) - Allocated
0x4000_0000 - 0x4000_0000: High Memory - Available
```

## Constants and Utilities

### Memory Constants

```rust
pub mod constants {
    pub const PAGE_SIZE: usize = 4096;                    // 4KB
    pub const LARGE_PAGE_SIZE: usize = 2 * 1024 * 1024;   // 2MB
    pub const HUGE_PAGE_SIZE: usize = 1024 * 1024 * 1024; // 1GB
    pub const KERNEL_VIRT_START: u64 = 0xFFFF_8000_0000_0000;
    pub const USER_VIRT_END: u64 = 0x0000_7FFF_FFFF_FFFF;
    pub const DEFAULT_HEAP_SIZE: usize = 64 * 1024 * 1024; // 64MB
    pub const MAX_ALLOCATION_SIZE: usize = 16 * 1024 * 1024; // 16MB
}
```

### Utility Functions

```rust
pub mod utils {
    // Page alignment utilities
    pub fn round_up_to_page(addr: u64) -> u64
    pub fn round_down_to_page(addr: u64) -> u64
    pub fn is_page_aligned(addr: u64) -> bool
    
    // Page calculations
    pub fn bytes_to_pages(bytes: u64) -> u64
    pub fn pages_to_bytes(pages: u64) -> u64
    
    // Address space checks
    pub fn is_kernel_address(addr: u64) -> bool
    pub fn is_user_address(addr: u64) -> bool
}
```

## Statistics and Monitoring

### Memory Statistics

The MM module provides comprehensive statistics for monitoring memory usage:

```rust
pub struct MemoryStats {
    pub total_physical: u64,      // Total physical memory
    pub available_physical: u64,  // Available physical memory
    pub used_physical: u64,       // Used physical memory
    pub total_virtual: u64,       // Total virtual memory
    pub used_virtual: u64,        // Used virtual memory
    pub allocated_pages: u64,     // Number of allocated pages
    pub free_pages: u64,          // Number of free pages
    pub page_size: u64,           // Page size
}
```

### Statistics Functions

```rust
// Global memory statistics
pub fn get_memory_stats() -> MemoryStats
pub fn print_memory_info()

// Subsystem-specific statistics
pub fn get_physical_stats() -> PhysicalMemoryStats
pub fn get_virtual_stats() -> VirtualMemoryStats  
pub fn get_paging_stats() -> PagingStats
pub fn get_allocator_stats() -> (AllocatorStats, AllocatorStats)
```

## Error Handling

### Memory Error Types

```rust
pub enum MemoryError {
    OutOfMemory,           // Out of memory
    InvalidAddress,        // Invalid address
    PageNotFound,          // Page not found
    PermissionDenied,      // Permission denied
    AlignmentError,        // Alignment error
    DoubleFree,           // Double free attempt
    AllocationTooLarge,   // Allocation too large
    Fragmentation,        // Memory fragmentation
}

pub type MemoryResult<T> = Result<T, MemoryError>;
```

## Testing and Validation

### Test Functions

Each submodule includes comprehensive test functions:

```rust
// Module-specific tests
pub fn test_physical_memory()  // phys.rs
pub fn test_paging()          // paging.rs  
pub fn test_virtual_memory()  // virt.rs
pub fn test_allocators()      // alloc.rs

// Global MM tests
pub fn run_mm_tests()         // mod.rs
```

### Expected Test Output

```
=== MEMORY MANAGEMENT TESTS ===
Testing MM utility functions:
  ✓ Utility functions working correctly
Testing memory statistics:
  ✓ Memory statistics functional

Testing physical memory management...
  ✓ Allocated 4KB at 0x0000000001000000
  ✓ Deallocated 4KB
  ✓ Allocated 10 pages at 0x0000000001001000
  ✓ Deallocated 10 pages

Testing paging functionality...
  ✓ Page mapping successful
  ✓ Address translation: 0x0000000000001000
  ✓ Page unmapping successful

Testing virtual memory management...
  ✓ Allocated 64KB of kernel heap at 0xFFFF800001000000
  ✓ Address is accessible for writing
  ✓ Found VMA: Kernel Heap rw--
  ✓ Deallocated virtual memory

Testing memory allocators...
  ✓ Allocated 1024 bytes at 0x2000000
  ✓ Memory write/read successful
  ✓ Deallocated memory
  ✓ Allocated 32-byte block at 0x3000000
  ✓ Freed 32-byte block
=== MM TESTS COMPLETE ===
```

## Integration with Other Subsystems

### Scheduler Integration

The MM module integrates with the scheduler for:
- Task memory space management
- Stack allocation for new tasks
- Memory protection between tasks

### HAL Integration

The MM module works with the HAL for:
- Architecture-specific paging mechanisms
- Memory mapping for device access
- Platform-specific memory layout

### Future Integration Points

- **PolyBus IPC**: Shared memory regions for inter-process communication
- **Security**: Memory encryption and protection features
- **Performance**: NUMA-aware allocation and page coloring
- **Virtualization**: Guest physical memory management

## Phase 1 Limitations

The current Phase 1 implementation includes simplified versions of:

1. **Memory Detection**: Hardcoded memory layout instead of bootloader/UEFI detection
2. **Page Table Management**: Basic mapping without full MMU setup
3. **Allocator Complexity**: Simple allocators instead of full buddy/slab systems
4. **Security Features**: Basic protection without advanced security mechanisms
5. **Performance Optimizations**: No NUMA awareness or advanced caching strategies

## Future Enhancements

### Phase 2 Planned Features

1. **Advanced Allocators**:
   - Buddy allocator for variable-size allocation
   - Slab allocator for object caching
   - SLUB allocator improvements

2. **Memory Management Unit (MMU)**:
   - Full page table setup and management
   - Hardware memory protection
   - SMEP/SMAP support

3. **Advanced Virtual Memory**:
   - Copy-on-write (COW) pages
   - Demand paging and swapping
   - Memory-mapped files

4. **Performance Features**:
   - NUMA-aware allocation
   - Page coloring for cache optimization
   - Transparent huge pages

5. **Security Enhancements**:
   - Memory encryption (Intel CET, ARM Pointer Authentication)
   - Control Flow Integrity (CFI)
   - Address Space Layout Randomization (ASLR)

This MM module provides a solid foundation for memory management in Polymera OS, with clear interfaces and comprehensive functionality that can be extended as the system grows in complexity.
