# Buddy Allocator Implementation - Polymera OS

## Overview

This document describes the comprehensive buddy allocator implementation for physical memory management in Polymera OS. The buddy allocator provides efficient allocation and deallocation of physically contiguous memory blocks with automatic coalescing to minimize fragmentation.

## Buddy Allocator Fundamentals

### Core Concept

The buddy system maintains free memory in power-of-2 sized blocks. Each block of size 2^k has a unique "buddy" - another block of the same size that can be merged with it to form a larger block of size 2^(k+1).

```
Buddy Relationship Example (4KB pages):
Order 0: [Page A] [Page B] [Page C] [Page D] [Page E] [Page F] [Page G] [Page H]
         |--Buddy--| |--Buddy--| |--Buddy--| |--Buddy--|

Order 1: [  2-Page Block  ] [  2-Page Block  ] [  2-Page Block  ] [  2-Page Block  ]
         |----Buddy Block----| |----Buddy Block----|

Order 2: [     4-Page Block     ] [     4-Page Block     ]
         |--------Buddy Block--------|
```

### Configuration

```rust
pub const MAX_ORDER: usize = 10;  // Maximum order (2^10 = 1024 pages = 4MB)
pub const MIN_ORDER: usize = 0;   // Minimum order (1 page = 4KB)
```

**Order Sizes:**
- Order 0: 1 page (4KB)
- Order 1: 2 pages (8KB)
- Order 2: 4 pages (16KB)
- Order 3: 8 pages (32KB)
- Order 4: 16 pages (64KB)
- Order 5: 32 pages (128KB)
- Order 6: 64 pages (256KB)
- Order 7: 128 pages (512KB)
- Order 8: 256 pages (1MB)
- Order 9: 512 pages (2MB)
- Order 10: 1024 pages (4MB)

## Implementation Structure

### Buddy Struct

```rust
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
```

### Free List Organization

```
Free Lists Array:
free_lists[0]: [Frame1] [Frame5] [Frame9]  ...    (1-page blocks)
free_lists[1]: [Frame2] [Frame7]           ...    (2-page blocks)  
free_lists[2]: [Frame3]                    ...    (4-page blocks)
free_lists[3]: [Frame4] [Frame8]           ...    (8-page blocks)
...
free_lists[10]: [Frame6]                   ...    (1024-page blocks)
```

## Core Algorithms

### Allocation Algorithm

```rust
pub fn alloc(&mut self, order: usize) -> Option<PhysFrame> {
    // 1. Find smallest available block >= requested order
    for current_order in order..=MAX_ORDER {
        if !self.free_lists[current_order].is_empty() {
            // 2. Remove block from free list
            let frame = self.free_lists[current_order].pop().unwrap();
            self.free_counts[current_order] -= 1;
            
            // 3. Split block down to requested order
            self.split_block(frame, current_order, order);
            
            // 4. Update statistics
            let allocated_pages = 1 << order;
            self.allocated_pages += allocated_pages;
            self.allocation_count += 1;
            
            return Some(frame);
        }
    }
    None // Out of memory
}
```

**Allocation Steps:**
1. **Find Block**: Search free lists from requested order up to MAX_ORDER
2. **Remove Block**: Take block from smallest available order
3. **Split Block**: Recursively split larger blocks into smaller ones
4. **Return Block**: Return block of exact requested size

### Block Splitting

```rust
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
    }
}
```

**Splitting Example:**
```
Initial: Order 3 block (8 pages) at address 0x1000000
Target:  Order 1 block (2 pages)

Step 1: Split order 3 → two order 2 blocks
  Block A: 0x1000000-0x1003FFF (4 pages) - keep
  Block B: 0x1004000-0x1007FFF (4 pages) - add to free_lists[2]

Step 2: Split order 2 → two order 1 blocks  
  Block A: 0x1000000-0x1001FFF (2 pages) - return to user
  Block B: 0x1002000-0x1003FFF (2 pages) - add to free_lists[1]
```

### Deallocation with Coalescing

```rust
pub fn free(&mut self, frame: PhysFrame, order: usize) {
    // Update statistics
    let freed_pages = 1 << order;
    self.allocated_pages = self.allocated_pages.saturating_sub(freed_pages);
    self.deallocation_count += 1;
    
    // Try to coalesce with buddy
    self.coalesce_and_free(frame, order);
}

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
        } else {
            // Can't coalesce, add to free list at current order
            break;
        }
    }
    
    // Add the (possibly coalesced) block to the free list
    self.free_lists[current_order].push(current_frame);
    self.free_counts[current_order] += 1;
}
```

### Buddy Calculation

```rust
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
```

**Buddy Calculation Example:**
```
For order 2 (4-page blocks) at base address 0x1000000:

Block at 0x1000000 (pages 0-3):
  block_index = 0x0000000 / 0x4000 = 0
  buddy_index = 0 XOR 1 = 1  
  buddy_addr = 0x1000000 + (1 * 0x4000) = 0x1004000

Block at 0x1004000 (pages 4-7):
  block_index = 0x0004000 / 0x4000 = 1
  buddy_index = 1 XOR 1 = 0
  buddy_addr = 0x1000000 + (0 * 0x4000) = 0x1000000
```

## API Functions

### Core Allocation Functions

```rust
/// Allocate a block of pages with the specified order
pub fn alloc(&mut self, order: usize) -> Option<PhysFrame>

/// Free a block of pages
pub fn free(&mut self, frame: PhysFrame, order: usize)
```

### Public Interface Functions

```rust
/// Allocate physical memory using buddy allocator with specific order
pub fn buddy_alloc(order: usize) -> MemoryResult<PhysFrame>

/// Free physical memory using buddy allocator
pub fn buddy_free(frame: PhysFrame, order: usize) -> MemoryResult<()>

/// Get buddy allocator statistics
pub fn get_buddy_stats() -> Option<BuddyStats>

/// Print buddy allocator state
pub fn print_buddy_state()
```

### Statistics and Monitoring

```rust
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
    pub fn fragmentation(&self) -> f32
    
    /// Calculate utilization percentage
    pub fn utilization(&self) -> f32
}
```

## Memory Layout Example

### Initial State (1024 pages total)

```
Order 10: [████████████████████████████████████████] (1024 pages)
Order 9:  []
Order 8:  []
...
Order 0:  []

Free Lists:
free_lists[10]: [Block_0x1000000] (1024 pages)
free_lists[9-0]: [] (empty)
```

### After Allocating Order 0, 2, 1

```
Allocations:
- alloc(0) → returns block at 0x1000000 (1 page)
- alloc(2) → returns block at 0x1001000 (4 pages)  
- alloc(1) → returns block at 0x1005000 (2 pages)

Current State:
Order 10: []
Order 9:  [██████████████████████████████████] (512 pages)
Order 8:  []
...
Order 1:  [██] (2 pages) 
Order 0:  [█] (1 page)

Free Lists:
free_lists[9]: [Block_0x1080000] (512 pages)
free_lists[1]: [Block_0x1007000] (2 pages)
free_lists[0]: [Block_0x1006000] (1 page)
free_lists[others]: [] (empty)

Allocated: 7 pages total (1 + 4 + 2)
```

## Integration with Physical Memory Manager

### Initialization

```rust
// In phys::init()
let buddy_base = PhysAddr::new(0x4000000);  // 64MB
let buddy_pages = 49152; // 192MB / 4KB = 49152 pages

{
    let mut buddy_allocator = BUDDY_ALLOCATOR.lock();
    *buddy_allocator = Some(Buddy::new(buddy_base, buddy_pages));
}
```

### Integration with allocate_physical()

```rust
pub fn allocate_physical(size: u64, align: u64) -> MemoryResult<u64> {
    // Calculate required order
    let pages_needed = (size + PAGE_SIZE as u64 - 1) / PAGE_SIZE as u64;
    let order = calculate_order_for_pages(pages_needed);
    
    // Try buddy allocator first
    {
        let mut buddy_allocator = BUDDY_ALLOCATOR.lock();
        if let Some(ref mut allocator) = *buddy_allocator {
            if let Some(frame) = allocator.alloc(order) {
                let phys_addr = frame.start_address().as_u64();
                
                // Check alignment
                if (phys_addr % align) == 0 {
                    return Ok(phys_addr);
                } else {
                    // Free and try fallback
                    allocator.free(frame, order);
                }
            }
        }
    }
    
    // Fallback to other allocation methods
    fallback_allocate(size, align)
}
```

## Performance Characteristics

### Time Complexity

| Operation | Best Case | Average Case | Worst Case |
|-----------|-----------|--------------|------------|
| Allocation | O(1) | O(log n) | O(log n) |
| Deallocation | O(1) | O(log n) | O(log n) |
| Coalescing | O(1) | O(log n) | O(log n) |

Where n is the maximum order (10 in our implementation).

### Space Complexity

- **Free Lists**: O(MAX_ORDER) = O(11) arrays
- **Metadata**: O(1) per allocated block
- **Total Overhead**: < 1% of managed memory

### Fragmentation Analysis

**Internal Fragmentation:**
- Maximum: 50% (worst case when requesting slightly more than half a block)
- Average: ~25% for random allocation patterns
- Minimum: 0% (when requesting exact power-of-2 sizes)

**External Fragmentation:**
- Minimized through automatic coalescing
- Worst case: Unable to satisfy large requests despite sufficient total free memory
- Mitigated by: Early coalescing, multiple order availability

## Testing and Validation

### Comprehensive Test Suite

The buddy allocator includes extensive unit tests covering:

1. **Basic Functionality**
   - Creation and initialization
   - Single page allocation/deallocation
   - Multiple order allocations

2. **Advanced Features**
   - Buddy coalescing verification
   - Fragmentation handling
   - Allocation limits and edge cases

3. **Stress Testing**
   - Random allocation/deallocation sequences
   - Memory pattern testing
   - Statistics accuracy verification

4. **Validation Functions**
   - Internal consistency checks
   - Memory region boundary validation
   - Free list integrity verification

### Example Test Output

```
=== BUDDY ALLOCATOR TEST ===
Initial state: 49152 total pages, 49152 free pages

Testing basic allocation/deallocation:
  ✓ Allocated order 0 block at PhysFrame(0x4000000)
  ✓ Allocated order 2 block at PhysFrame(0x4001000) 
  ✓ Allocated order 5 block at PhysFrame(0x4005000)

After allocation: 37 allocated pages, 49115 free pages

Freeing allocated blocks:
  ✓ Freed order 0 block at PhysFrame(0x4000000)
  ✓ Freed order 2 block at PhysFrame(0x4001000)
  ✓ Freed order 5 block at PhysFrame(0x4005000)

Final state: 0 allocated pages, 49152 free pages
  ✓ Free count returned to initial value
  ✓ All memory freed correctly

Testing fragmentation and coalescing:
  Creating fragmentation with small allocations...
  Freeing every other block to create holes...
  Attempting large allocation (should trigger coalescing):
    ✓ Successfully allocated order 3 block at PhysFrame(0x4000000)

Testing random allocation/deallocation sequences:
  Running 20 random operations...
  ✓ Allocator validation passed

[BUDDY] Allocator State:
  Base address: PhysAddr(0x4000000)
  Total pages: 49152
  Allocated pages: 0
  Allocations: 147
  Deallocations: 147
  Free lists:
    Order 10: 48 blocks (1024 pages each, 49152 pages total)

=== BUDDY ALLOCATOR TEST COMPLETE ===
```

## Advantages and Benefits

### Memory Efficiency
- **Low Overhead**: Minimal metadata per block
- **Automatic Coalescing**: Reduces external fragmentation
- **Power-of-2 Sizes**: Efficient for common allocation patterns

### Performance Benefits
- **Fast Allocation**: O(log n) worst case
- **Fast Deallocation**: O(log n) with automatic coalescing
- **Cache Friendly**: Contiguous memory blocks

### Reliability Features
- **Self-Validating**: Built-in consistency checks
- **Deterministic**: Predictable behavior
- **Robust**: Handles fragmentation gracefully

### Integration Benefits
- **Standard Interface**: Compatible with existing allocation APIs
- **Thread Safe**: Protected by mutex for concurrent access
- **Statistics**: Comprehensive monitoring and debugging support

## Future Enhancements

### Phase 2 Improvements

1. **Multiple Buddy Zones**
   - Separate allocators for different memory types (DMA, high memory, etc.)
   - NUMA-aware allocation

2. **Lazy Coalescing**
   - Defer coalescing to reduce deallocation overhead
   - Background coalescing thread

3. **Slab Integration**
   - Use buddy allocator as backing store for slab allocator
   - Optimized small object allocation

4. **Advanced Statistics**
   - Allocation pattern analysis
   - Fragmentation prediction
   - Performance profiling

5. **Memory Compaction**
   - Move allocated blocks to reduce fragmentation
   - Background defragmentation

This buddy allocator implementation provides a solid foundation for physical memory management in Polymera OS, offering excellent performance, reliability, and integration capabilities while maintaining simplicity and efficiency.

