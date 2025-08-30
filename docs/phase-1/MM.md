# Memory Management (MM) - Phase 1

## Overview

The Memory Management system provides comprehensive memory allocation, paging, and virtual memory management capabilities including 4-level paging for x86_64, physical buddy allocation, and efficient kernel object allocation via slab allocators.

## Architecture Components

```
Memory Management System
├── Paging System (4-level x86_64)
│   ├── Page Tables (PML4, PDP, PD, PT)
│   ├── Kernel Section Mapping
│   └── Guard Pages
├── Physical Memory Management
│   ├── Buddy Allocator (4KiB pages, MAX_ORDER=10)
│   └── Frame Tracking
├── Virtual Memory API
│   ├── map_page()
│   ├── unmap_page()
│   ├── translate_va()
│   └── flush_tlb()
└── Slab Allocator
    ├── Fixed-size Pools (32, 64, 128, 256, 512 bytes)
    ├── kmalloc() / kfree()
    └── Poison Byte Protection
```

## Paging System

### 4-Level Paging Structure

```rust
pub struct PagingManager {
    level_4: &'static mut PageTable,
    frame_allocator: BuddyAllocator,
}

impl PagingManager {
    pub fn map_page(
        &mut self,
        vaddr: x86_64::VirtAddr,
        paddr: x86_64::PhysAddr,
        flags: PageTableFlags,
    ) -> Result<(), &'static str> {
        let page = Page::containing_address(vaddr);
        let frame = PhysFrame::containing_address(paddr);
        
        // Walk page tables and set entry
        let level_1 = self.get_level_1_table(page)?;
        level_1[page.p1_index()].set(frame, flags);
        
        // Flush TLB
        x86_64::instructions::tlb::flush(vaddr);
        
        Ok(())
    }
}
```

### Page Table Hierarchy

```
Virtual Address Translation:
- Bits 39-47: PML4 index
- Bits 30-38: PDP index  
- Bits 21-29: PD index
- Bits 12-20: PT index
- Bits 0-11:  Page offset
```

## Physical Memory Management

### Buddy Allocator

```rust
pub struct BuddyAllocator {
    free_lists: [AtomicU64; MAX_ORDER + 1],
    total_frames: usize,
    used_frames: AtomicU64,
}

const MAX_ORDER: usize = 10;  // Up to 4MB blocks
const FRAME_SIZE: usize = 4096;  // 4KB pages

impl BuddyAllocator {
    pub fn allocate_order(&mut self, order: usize) -> Option<PhysFrame<Size4KiB>> {
        if order > MAX_ORDER {
            return None;
        }
        
        // Try to allocate from free list
        if self.free_lists[order].load(Ordering::Relaxed) > 0 {
            return self.get_free_block(order);
        }
        
        // Split larger block if available
        for higher_order in (order + 1)..=MAX_ORDER {
            if self.free_lists[higher_order].load(Ordering::Relaxed) > 0 {
                return self.split_block(higher_order, order);
            }
        }
        
        None
    }
}
```

## Slab Allocator

### Fixed-Size Allocation Pools

```rust
pub struct KSlab {
    pool_size: usize,
    object_size: usize,
    free_list: Mutex<Vec<*mut u8>>,
    total_allocated: AtomicUsize,
    total_freed: AtomicUsize,
}

pub struct SlabAllocator {
    slabs: [KSlab; 5],  // 32, 64, 128, 256, 512 bytes
}

impl SlabAllocator {
    pub fn kmalloc(&self, size: usize) -> Option<*mut u8> {
        let slab_index = self.find_slab_index(size)?;
        let slab = &self.slabs[slab_index];
        slab.allocate()
    }
    
    pub fn kfree(&self, ptr: *mut u8, size: usize) {
        if let Some(slab_index) = self.find_slab_index(size) {
            let slab = &self.slabs[slab_index];
            slab.free(ptr);
        }
    }
}
```

## Virtual Memory API

### Core Functions

```rust
impl PagingManager {
    /// Map a virtual page to a physical frame
    pub fn map_page(
        &mut self,
        vaddr: x86_64::VirtAddr,
        paddr: x86_64::PhysAddr,
        flags: PageTableFlags,
    ) -> Result<(), &'static str>;
    
    /// Unmap a virtual page
    pub fn unmap_page(&mut self, vaddr: x86_64::VirtAddr) -> Result<(), &'static str>;
    
    /// Translate virtual address to physical address
    pub fn translate_va(&self, vaddr: x86_64::VirtAddr) -> Option<x86_64::PhysAddr>;
    
    /// Flush TLB for all entries
    pub fn flush_tlb(&self);
}
```

## Troubleshooting

### Common Issues

1. **Page Faults During Boot**: Check page table setup and identity mapping
2. **Memory Allocation Failures**: Verify physical memory availability and fragmentation
3. **Slab Allocator Corruption**: Check for double-free operations and poison bytes

### Debug Commands

```rust
// Enable memory debugging
const MM_DEBUG: bool = cfg!(debug_assertions);

fn mm_debug_print(msg: &str) {
    if MM_DEBUG {
        kprintln!("[MM_DEBUG] {}", msg);
    }
}

// Memory leak detection
impl PagingManager {
    pub fn detect_memory_leaks(&self) -> Vec<MemoryLeak>;
}
```

## References

- [Phase 1 SPEC](SPEC.md) - Overall Phase 1 specifications
- [Hardware Abstraction Layer](HAL.md) - HAL implementation details
- [Boot System](BOOT.md) - Boot sequence and initialization
- [x86_64 Paging Reference](https://wiki.osdev.org/Paging)

## Overview

The Memory Management system provides comprehensive memory allocation, paging, and virtual memory management capabilities including 4-level paging for x86_64, physical buddy allocation, and efficient kernel object allocation via slab allocators.

## Architecture Components

```
Memory Management System
├── Paging System (4-level x86_64)
│   ├── Page Tables (PML4, PDP, PD, PT)
│   ├── Kernel Section Mapping
│   └── Guard Pages
├── Physical Memory Management
│   ├── Buddy Allocator (4KiB pages, MAX_ORDER=10)
│   └── Frame Tracking
├── Virtual Memory API
│   ├── map_page()
│   ├── unmap_page()
│   ├── translate_va()
│   └── flush_tlb()
└── Slab Allocator
    ├── Fixed-size Pools (32, 64, 128, 256, 512 bytes)
    ├── kmalloc() / kfree()
    └── Poison Byte Protection
```

## Paging System

### 4-Level Paging Structure

```rust
pub struct PagingManager {
    level_4: &'static mut PageTable,
    frame_allocator: BuddyAllocator,
}

impl PagingManager {
    pub fn map_page(
        &mut self,
        vaddr: x86_64::VirtAddr,
        paddr: x86_64::PhysAddr,
        flags: PageTableFlags,
    ) -> Result<(), &'static str> {
        let page = Page::containing_address(vaddr);
        let frame = PhysFrame::containing_address(paddr);
        
        // Walk page tables and set entry
        let level_1 = self.get_level_1_table(page)?;
        level_1[page.p1_index()].set(frame, flags);
        
        // Flush TLB
        x86_64::instructions::tlb::flush(vaddr);
        
        Ok(())
    }
}
```

### Page Table Hierarchy

```
Virtual Address Translation:
- Bits 39-47: PML4 index
- Bits 30-38: PDP index  
- Bits 21-29: PD index
- Bits 12-20: PT index
- Bits 0-11:  Page offset
```

## Physical Memory Management

### Buddy Allocator

```rust
pub struct BuddyAllocator {
    free_lists: [AtomicU64; MAX_ORDER + 1],
    total_frames: usize,
    used_frames: AtomicU64,
}

const MAX_ORDER: usize = 10;  // Up to 4MB blocks
const FRAME_SIZE: usize = 4096;  // 4KB pages

impl BuddyAllocator {
    pub fn allocate_order(&mut self, order: usize) -> Option<PhysFrame<Size4KiB>> {
        if order > MAX_ORDER {
            return None;
        }
        
        // Try to allocate from free list
        if self.free_lists[order].load(Ordering::Relaxed) > 0 {
            return self.get_free_block(order);
        }
        
        // Split larger block if available
        for higher_order in (order + 1)..=MAX_ORDER {
            if self.free_lists[higher_order].load(Ordering::Relaxed) > 0 {
                return self.split_block(higher_order, order);
            }
        }
        
        None
    }
}
```

## Slab Allocator

### Fixed-Size Allocation Pools

```rust
pub struct KSlab {
    pool_size: usize,
    object_size: usize,
    free_list: Mutex<Vec<*mut u8>>,
    total_allocated: AtomicUsize,
    total_freed: AtomicUsize,
}

pub struct SlabAllocator {
    slabs: [KSlab; 5],  // 32, 64, 128, 256, 512 bytes
}

impl SlabAllocator {
    pub fn kmalloc(&self, size: usize) -> Option<*mut u8> {
        let slab_index = self.find_slab_index(size)?;
        let slab = &self.slabs[slab_index];
        slab.allocate()
    }
    
    pub fn kfree(&self, ptr: *mut u8, size: usize) {
        if let Some(slab_index) = self.find_slab_index(size) {
            let slab = &self.slabs[slab_index];
            slab.free(ptr);
        }
    }
}
```

## Virtual Memory API

### Core Functions

```rust
impl PagingManager {
    /// Map a virtual page to a physical frame
    pub fn map_page(
        &mut self,
        vaddr: x86_64::VirtAddr,
        paddr: x86_64::PhysAddr,
        flags: PageTableFlags,
    ) -> Result<(), &'static str>;
    
    /// Unmap a virtual page
    pub fn unmap_page(&mut self, vaddr: x86_64::VirtAddr) -> Result<(), &'static str>;
    
    /// Translate virtual address to physical address
    pub fn translate_va(&self, vaddr: x86_64::VirtAddr) -> Option<x86_64::PhysAddr>;
    
    /// Flush TLB for all entries
    pub fn flush_tlb(&self);
}
```

## Troubleshooting

### Common Issues

1. **Page Faults During Boot**: Check page table setup and identity mapping
2. **Memory Allocation Failures**: Verify physical memory availability and fragmentation
3. **Slab Allocator Corruption**: Check for double-free operations and poison bytes

### Debug Commands

```rust
// Enable memory debugging
const MM_DEBUG: bool = cfg!(debug_assertions);

fn mm_debug_print(msg: &str) {
    if MM_DEBUG {
        kprintln!("[MM_DEBUG] {}", msg);
    }
}

// Memory leak detection
impl PagingManager {
    pub fn detect_memory_leaks(&self) -> Vec<MemoryLeak>;
}
```

## References

- [Phase 1 SPEC](SPEC.md) - Overall Phase 1 specifications
- [Hardware Abstraction Layer](HAL.md) - HAL implementation details
- [Boot System](BOOT.md) - Boot sequence and initialization
- [x86_64 Paging Reference](https://wiki.osdev.org/Paging)




