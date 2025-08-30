# Virtual Memory API Implementation - Polymera OS

## Overview

This document describes the comprehensive virtual memory API implementation for Polymera OS. The VM API provides high-level virtual memory management functions with RWX flag enforcement, double-mapping detection, and comprehensive error handling.

## API Functions

### Core Functions

The VM API provides three primary functions as requested:

```rust
/// Map a virtual page to a physical page
pub fn map_page(vaddr: u64, paddr: u64, flags: VmFlags) -> MemoryResult<()>

/// Unmap a virtual page
pub fn unmap_page(vaddr: u64) -> MemoryResult<()>

/// Translate virtual address to physical address
pub fn translate_va(vaddr: u64) -> Option<u64>
```

## VmFlags System

### RWX Permission Flags

The `VmFlags` struct provides comprehensive read, write, and execute permission control:

```rust
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
```

### Predefined Flag Combinations

```rust
impl VmFlags {
    /// Create read-only flags
    pub const fn read_only() -> Self
    
    /// Create read-write flags
    pub const fn read_write() -> Self
    
    /// Create read-execute flags
    pub const fn read_execute() -> Self
    
    /// Create read-write-execute flags
    pub const fn read_write_execute() -> Self
    
    /// Create user read-only flags
    pub const fn user_read_only() -> Self
    
    /// Create user read-write flags
    pub const fn user_read_write() -> Self
    
    /// Create user read-execute flags
    pub const fn user_read_execute() -> Self
}
```

### Flag Display

Flags are displayed in a standard format:
```
PRWXUAD format:
P = Present
R = Read
W = Write  
X = Execute
U = User
A = Accessed
D = Dirty

Example: "PRWX---" = Present, Read, Write, Execute, not User, not Accessed, not Dirty
```

## Function Implementations

### map_page(vaddr, paddr, flags)

```rust
pub fn map_page(vaddr: u64, paddr: u64, flags: VmFlags) -> MemoryResult<()> {
    let mut vm_manager = VM_MANAGER.lock();
    vm_manager.map_page(vaddr, paddr, flags)
}
```

**Features:**
- **Page Alignment**: Automatically aligns addresses to page boundaries
- **Double-Mapping Detection**: Returns `MemoryError::AlreadyMapped` if page already mapped
- **Address Validation**: Validates virtual and physical addresses
- **Hardware Integration**: Calls low-level paging functions
- **Metadata Tracking**: Maintains mapping information for validation

**Algorithm:**
1. Align virtual and physical addresses to page boundaries
2. Check if virtual page is already mapped (double-mapping detection)
3. Validate addresses (null page handling for kernel vs user)
4. Create mapping metadata entry
5. Call hardware paging function with converted flags
6. Store mapping metadata on success
7. Update statistics

**Error Handling:**
- `MemoryError::AlreadyMapped`: Page already mapped
- `MemoryError::InvalidAddress`: Invalid virtual/physical address
- Propagates paging system errors

### unmap_page(vaddr)

```rust
pub fn unmap_page(vaddr: u64) -> MemoryResult<()> {
    let mut vm_manager = VM_MANAGER.lock();
    vm_manager.unmap_page(vaddr)
}
```

**Features:**
- **Double-Free Detection**: Returns `MemoryError::NotMapped` if page not mapped
- **Page Alignment**: Automatically aligns address to page boundary
- **Hardware Integration**: Calls low-level paging functions
- **Metadata Cleanup**: Removes mapping information
- **Statistics Tracking**: Updates unmap counters

**Algorithm:**
1. Align virtual address to page boundary
2. Check if virtual page is currently mapped (double-free detection)
3. Call hardware paging unmap function
4. Remove mapping metadata on success
5. Update statistics

**Error Handling:**
- `MemoryError::NotMapped`: Page not currently mapped
- Propagates paging system errors

### translate_va(vaddr) -> Option<paddr>

```rust
pub fn translate_va(vaddr: u64) -> Option<u64> {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.translate_va(vaddr)
}
```

**Features:**
- **Offset Preservation**: Correctly handles page offsets
- **Metadata Lookup**: Uses cached mapping information for performance
- **Hardware Fallback**: Falls back to hardware translation if metadata missing
- **Page Alignment Handling**: Works with any address within a page

**Algorithm:**
1. Split address into page base and offset
2. Look up mapping in metadata cache
3. If found, return physical page + offset
4. If not found, try hardware translation as fallback
5. Return None if no translation available

## Error Detection and Handling

### Double-Mapping Detection

```rust
// Check if page is already mapped
if self.mappings.contains_key(&page_vaddr) {
    self.double_map_attempts += 1;
    klog!(ERROR, "[VM] Attempt to map already mapped page: 0x{:016x}", page_vaddr);
    return Err(MemoryError::AlreadyMapped);
}
```

**Detection Method:**
- Maintains BTreeMap of all active mappings
- O(log n) lookup to check for existing mappings
- Tracks statistics for double-mapping attempts

### Double-Free Detection

```rust
// Check if page is mapped
if !self.mappings.contains_key(&page_vaddr) {
    self.double_unmap_attempts += 1;
    klog!(ERROR, "[VM] Attempt to unmap non-mapped page: 0x{:016x}", page_vaddr);
    return Err(MemoryError::NotMapped);
}
```

**Detection Method:**
- Checks mapping existence before unmapping
- Tracks statistics for double-unmap attempts
- Provides clear error messages

## RWX Enforcement Simulation

### Access Permission Checking

```rust
pub fn check_access(vaddr: u64, write: bool, execute: bool, user: bool) -> bool {
    let vm_manager = VM_MANAGER.lock();
    vm_manager.check_access(vaddr, write, execute, user)
}
```

**Implementation:**
```rust
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
```

**Enforcement Rules:**
1. **Present Check**: Page must be present in memory
2. **Read Check**: All accesses require read permission
3. **Write Check**: Write operations require write permission
4. **Execute Check**: Code execution requires execute permission
5. **User Check**: User-mode access requires user permission

### Permission Matrix

| Access Type | Required Flags | Description |
|-------------|---------------|-------------|
| Read | `present && read` | Basic read access |
| Write | `present && read && write` | Write operations |
| Execute | `present && read && execute` | Code execution |
| User Read | `present && read && user` | User-mode read |
| User Write | `present && read && write && user` | User-mode write |
| User Execute | `present && read && execute && user` | User-mode execute |

## Virtual Memory Manager

### Core Data Structure

```rust
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
```

### Page Mapping Metadata

```rust
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
```

**Features:**
- **Complete Mapping Info**: Stores all mapping details
- **Timestamp Tracking**: Records when mapping was created
- **Flag Storage**: Maintains permission information
- **Copy Semantics**: Efficient copying for queries

## Statistics and Monitoring

### VM Statistics

```rust
#[derive(Debug, Clone, Copy)]
pub struct VmStats {
    pub mapped_pages: u64,
    pub unmapped_pages: u64,
    pub active_mappings: u64,
    pub failed_mappings: u64,
    pub double_map_attempts: u64,
    pub double_unmap_attempts: u64,
}
```

### Statistics API

```rust
/// Get virtual memory statistics
pub fn get_vm_stats() -> VmStats

/// Validate all virtual memory mappings
pub fn validate_vm() -> bool

/// Print all virtual memory mappings
pub fn print_vm_mappings()
```

### Example Statistics Output

```
Final VM API statistics:
  Mapped pages: 157
  Unmapped pages: 155
  Active mappings: 2
  Failed mappings: 0
  Double map attempts: 5
  Double unmap attempts: 3
```

## Validation and Integrity

### Mapping Validation

```rust
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
```

**Validation Checks:**
1. **Address Alignment**: All addresses must be page-aligned
2. **Consistency**: Virtual addresses must match mapping entries
3. **Data Integrity**: Mapping data must be internally consistent

## Comprehensive Testing

### Test Coverage

The VM API includes extensive unit tests covering:

1. **Basic Functionality**
   - `test_basic_map_page()`: Basic mapping operations
   - `test_basic_unmap_page()`: Basic unmapping operations
   - `test_translate_va()`: Address translation with offsets

2. **Error Detection**
   - `test_double_mapping_error()`: Double-mapping detection
   - `test_double_unmapping_error()`: Double-unmapping detection
   - `test_edge_cases()`: Various edge cases and error conditions

3. **RWX Enforcement**
   - `test_rwx_flag_enforcement()`: Permission flag validation
   - All flag combinations tested (R--, RW-, R-X, RWX, U-R-, U-RW, U-RX)

4. **System Properties**
   - `test_page_alignment()`: Page alignment handling
   - `test_multiple_mappings()`: Simultaneous mappings
   - `test_vm_validation()`: System integrity validation
   - `test_vm_statistics()`: Statistics tracking accuracy
   - `test_memory_leak_detection()`: Memory leak prevention

### Example Test Scenarios

**Basic Map/Unmap Test:**
```rust
let vaddr = 0x400000; // 4MB
let paddr = 0x1000000; // 16MB
let flags = VmFlags::read_write();

// Test mapping
assert!(map_page(vaddr, paddr, flags).is_ok());
assert!(is_mapped(vaddr));
assert_eq!(translate_va(vaddr), Some(paddr));

// Test unmapping
assert!(unmap_page(vaddr).is_ok());
assert!(!is_mapped(vaddr));
assert_eq!(translate_va(vaddr), None);
```

**Double-Mapping Error Test:**
```rust
let vaddr = 0x500000;
let paddr1 = 0x2000000;
let paddr2 = 0x3000000;

// First mapping succeeds
assert!(map_page(vaddr, paddr1, VmFlags::read_only()).is_ok());

// Second mapping fails
assert_eq!(map_page(vaddr, paddr2, VmFlags::read_write()),
           Err(MemoryError::AlreadyMapped));
```

**RWX Enforcement Test:**
```rust
let test_cases = [
    (VmFlags::read_only(), true, false, false, false),
    (VmFlags::read_write(), true, true, false, false),
    (VmFlags::read_execute(), true, false, true, false),
    (VmFlags::user_read_write(), true, true, false, true),
];

for (flags, exp_read, exp_write, exp_exec, exp_user) in test_cases {
    assert!(map_page(vaddr, paddr, flags).is_ok());
    
    assert_eq!(check_access(vaddr, false, false, false), exp_read);
    assert_eq!(check_access(vaddr, true, false, false), exp_write);
    assert_eq!(check_access(vaddr, false, true, false), exp_exec);
    assert_eq!(check_access(vaddr, false, false, true), exp_user);
    
    unmap_page(vaddr).unwrap();
}
```

## Integration with Existing Systems

### Paging System Integration

The VM API integrates seamlessly with the existing paging system:

```rust
// Convert VmFlags to PageFlags for hardware layer
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

// Call hardware paging functions
match super::paging::map_page(
    VirtAddr::new(page_vaddr),
    PhysAddr::new(page_paddr),
    flags.to_page_flags()
) {
    Ok(()) => { /* success */ }
    Err(e) => { /* handle error */ }
}
```

### Memory Management Module Exports

```rust
// Re-exported VM API functions
pub use virt::{
    map_page, unmap_page, translate_va, is_mapped, get_page_flags, check_access,
    update_flags, get_vm_stats, validate_vm, print_vm_mappings, test_vm_api,
    VmFlags, VmStats, PageMapping, VirtualMemoryManager
};
```

## Performance Characteristics

### Time Complexity

| Operation | Best Case | Average Case | Worst Case |
|-----------|-----------|--------------|------------|
| map_page | O(log n) | O(log n) | O(log n) |
| unmap_page | O(log n) | O(log n) | O(log n) |
| translate_va | O(log n) | O(log n) | O(log n) |
| is_mapped | O(log n) | O(log n) | O(log n) |
| check_access | O(log n) | O(log n) | O(log n) |

**Note:** n = number of active mappings. BTreeMap provides O(log n) operations.

### Space Complexity

- **Per Mapping**: 40 bytes (64-bit addresses + flags + timestamp)
- **BTreeMap Overhead**: ~24 bytes per node (typical implementation)
- **Total Per Mapping**: ~64 bytes
- **Manager Overhead**: ~80 bytes (statistics + metadata)

For 1000 active mappings: ~64KB memory usage

### Performance Optimizations

1. **BTreeMap for Mappings**: Efficient O(log n) operations
2. **Metadata Caching**: Avoids hardware TLB queries when possible
3. **Page Alignment**: Automatic alignment reduces validation overhead
4. **Lock Granularity**: Single lock per VM manager (suitable for kernel)

## Example Usage

### Basic Page Mapping

```rust
use polymera_kernel::mm::{map_page, unmap_page, translate_va, VmFlags};

// Map a page with read-write permissions
let vaddr = 0x400000;
let paddr = 0x1000000;
let flags = VmFlags::read_write();

match map_page(vaddr, paddr, flags) {
    Ok(()) => {
        println!("Page mapped successfully");
        
        // Translate the address
        if let Some(translated) = translate_va(vaddr + 0x123) {
            println!("Translation: 0x{:x} -> 0x{:x}", vaddr + 0x123, translated);
        }
        
        // Unmap when done
        unmap_page(vaddr).unwrap();
    }
    Err(e) => println!("Mapping failed: {}", e),
}
```

### Permission Checking

```rust
use polymera_kernel::mm::{map_page, check_access, VmFlags};

// Map page with specific permissions
let vaddr = 0x500000;
let paddr = 0x2000000;
let flags = VmFlags::user_read_execute();

map_page(vaddr, paddr, flags).unwrap();

// Check various access permissions
assert!(check_access(vaddr, false, false, false));  // Read: OK
assert!(!check_access(vaddr, true, false, false));  // Write: Not allowed
assert!(check_access(vaddr, false, true, false));   // Execute: OK
assert!(check_access(vaddr, false, false, true));   // User: OK
```

### Bulk Operations

```rust
use polymera_kernel::mm::{map_page, unmap_page, VmFlags};

// Map multiple pages
let base_vaddr = 0x600000;
let base_paddr = 0x3000000;
let num_pages = 10;

for i in 0..num_pages {
    let vaddr = base_vaddr + i * 4096;
    let paddr = base_paddr + i * 4096;
    let flags = VmFlags::read_write();
    
    map_page(vaddr, paddr, flags).unwrap();
}

// Clean up all pages
for i in 0..num_pages {
    let vaddr = base_vaddr + i * 4096;
    unmap_page(vaddr).unwrap();
}
```

## Error Handling Best Practices

### Error Classification

```rust
match map_page(vaddr, paddr, flags) {
    Ok(()) => {
        // Success - mapping created
    }
    Err(MemoryError::AlreadyMapped) => {
        // Application error - check logic
        log::error!("Attempted to map already mapped page");
    }
    Err(MemoryError::InvalidAddress) => {
        // Application error - validate inputs
        log::error!("Invalid address provided");
    }
    Err(MemoryError::OutOfMemory) => {
        // System error - handle gracefully
        log::warn!("System out of memory");
    }
    Err(e) => {
        // Other system errors
        log::error!("Unexpected mapping error: {}", e);
    }
}
```

### Cleanup Patterns

```rust
// RAII-style cleanup
struct MappedPage {
    vaddr: u64,
}

impl MappedPage {
    fn new(vaddr: u64, paddr: u64, flags: VmFlags) -> Result<Self, MemoryError> {
        map_page(vaddr, paddr, flags)?;
        Ok(MappedPage { vaddr })
    }
}

impl Drop for MappedPage {
    fn drop(&mut self) {
        let _ = unmap_page(self.vaddr);
    }
}
```

This comprehensive VM API implementation provides robust virtual memory management with excellent error detection, comprehensive testing, and seamless integration with the existing memory management system. The implementation fully satisfies the requirements for `map_page(vaddr, paddr, flags)` with RWX flags, `unmap_page(vaddr)`, `translate_va(vaddr) -> Option<paddr>`, and comprehensive testing including double-free error detection and RWX enforcement simulation.

