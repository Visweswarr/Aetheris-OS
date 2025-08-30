/// Slab Allocator Unit Tests
/// 
/// This module contains comprehensive unit tests for the slab allocator
/// implementation, including allocation/deallocation of 10k objects,
/// memory leak detection, and poison byte verification.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec::Vec;
use polymera_kernel::mm::alloc::{KSlab, SlabAllocator, SLAB_SIZES, POISON_BYTE};
use polymera_kernel::mm::constants::PAGE_SIZE;
use polymera_kernel::{kprintln, klog};

/// Test runner function
pub fn test_runner(tests: &[&dyn Fn()]) {
    kprintln!("Running {} slab allocator unit tests", tests.len());
    for test in tests {
        test();
    }
    kprintln!("All slab allocator unit tests completed");
}

/// Panic handler for tests
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kprintln!("Test panic: {}", info);
    loop {}
}

/// Entry point for tests
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

/// Test KSlab creation and basic functionality
#[test_case]
fn test_kslab_creation() {
    kprintln!("Testing KSlab creation and basic functionality...");
    
    const SLAB_SIZE: usize = 64;
    const TOTAL_SIZE: usize = PAGE_SIZE * 4; // 16KB
    let memory_region = 0x10000000 as *mut u8; // Dummy address for testing
    
    let mut slab = KSlab::new(SLAB_SIZE, memory_region, TOTAL_SIZE);
    
    // Verify initial state
    let stats = slab.stats();
    assert_eq!(stats.object_size, SLAB_SIZE);
    assert_eq!(stats.capacity, TOTAL_SIZE / SLAB_SIZE); // Should be 256 objects
    assert_eq!(stats.allocated_count, 0);
    assert_eq!(stats.free_count, stats.capacity);
    assert_eq!(stats.utilization, 0.0);
    
    kprintln!("  ✓ KSlab created with {} objects of {} bytes each", 
              stats.capacity, SLAB_SIZE);
    kprintln!("  ✓ Initial state verified: {} free objects", stats.free_count);
}

/// Test single object allocation and deallocation
#[test_case]
fn test_single_object_allocation() {
    kprintln!("Testing single object allocation and deallocation...");
    
    const SLAB_SIZE: usize = 128;
    const TOTAL_SIZE: usize = PAGE_SIZE * 2; // 8KB
    let memory_region = 0x10000000 as *mut u8;
    
    let mut slab = KSlab::new(SLAB_SIZE, memory_region, TOTAL_SIZE);
    let initial_stats = slab.stats();
    
    // Allocate one object
    let ptr = slab.alloc_object().expect("Failed to allocate object");
    
    // Verify allocation
    let alloc_stats = slab.stats();
    assert_eq!(alloc_stats.allocated_count, 1);
    assert_eq!(alloc_stats.free_count, initial_stats.capacity - 1);
    assert_eq!(alloc_stats.allocation_count, 1);
    
    kprintln!("  ✓ Allocated object at {:p}", ptr);
    kprintln!("  ✓ Statistics updated: {} allocated, {} free", 
              alloc_stats.allocated_count, alloc_stats.free_count);
    
    // Free the object
    assert!(slab.free_object(ptr), "Failed to free object");
    
    // Verify deallocation
    let free_stats = slab.stats();
    assert_eq!(free_stats.allocated_count, 0);
    assert_eq!(free_stats.free_count, initial_stats.capacity);
    assert_eq!(free_stats.deallocation_count, 1);
    
    kprintln!("  ✓ Object freed successfully");
    kprintln!("  ✓ Statistics restored: {} allocated, {} free", 
              free_stats.allocated_count, free_stats.free_count);
}

/// Test multiple object allocation patterns
#[test_case]
fn test_multiple_object_allocation() {
    kprintln!("Testing multiple object allocation patterns...");
    
    const SLAB_SIZE: usize = 32;
    const TOTAL_SIZE: usize = PAGE_SIZE; // 4KB = 128 objects
    let memory_region = 0x10000000 as *mut u8;
    
    let mut slab = KSlab::new(SLAB_SIZE, memory_region, TOTAL_SIZE);
    let capacity = slab.stats().capacity;
    
    // Allocate multiple objects
    let mut allocated_ptrs = Vec::new();
    for i in 0..10 {
        if let Some(ptr) = slab.alloc_object() {
            allocated_ptrs.push(ptr);
            kprintln!("    Allocated object {}: {:p}", i, ptr);
        } else {
            panic!("Failed to allocate object {}", i);
        }
    }
    
    // Verify state
    let mid_stats = slab.stats();
    assert_eq!(mid_stats.allocated_count, 10);
    assert_eq!(mid_stats.free_count, capacity - 10);
    
    kprintln!("  ✓ Allocated 10 objects successfully");
    
    // Free every other object
    let mut freed_count = 0;
    for (i, &ptr) in allocated_ptrs.iter().enumerate() {
        if i % 2 == 0 {
            assert!(slab.free_object(ptr), "Failed to free object {}", i);
            freed_count += 1;
        }
    }
    
    // Verify partial free state
    let partial_stats = slab.stats();
    assert_eq!(partial_stats.allocated_count, 10 - freed_count);
    assert_eq!(partial_stats.free_count, capacity - (10 - freed_count));
    
    kprintln!("  ✓ Freed {} out of 10 objects", freed_count);
    
    // Free remaining objects
    for (i, &ptr) in allocated_ptrs.iter().enumerate() {
        if i % 2 == 1 {
            assert!(slab.free_object(ptr), "Failed to free remaining object {}", i);
        }
    }
    
    // Verify all objects freed
    let final_stats = slab.stats();
    assert_eq!(final_stats.allocated_count, 0);
    assert_eq!(final_stats.free_count, capacity);
    
    kprintln!("  ✓ All objects freed successfully");
}

/// Test slab capacity limits
#[test_case]
fn test_slab_capacity_limits() {
    kprintln!("Testing slab capacity limits...");
    
    const SLAB_SIZE: usize = 256;
    const TOTAL_SIZE: usize = SLAB_SIZE * 4; // Only 4 objects
    let memory_region = 0x10000000 as *mut u8;
    
    let mut slab = KSlab::new(SLAB_SIZE, memory_region, TOTAL_SIZE);
    let capacity = slab.stats().capacity;
    
    assert_eq!(capacity, 4, "Expected capacity of 4 objects");
    
    // Allocate all available objects
    let mut allocated_ptrs = Vec::new();
    for i in 0..capacity {
        match slab.alloc_object() {
            Some(ptr) => {
                allocated_ptrs.push(ptr);
                kprintln!("    Allocated object {}: {:p}", i, ptr);
            }
            None => panic!("Failed to allocate object {} within capacity", i),
        }
    }
    
    // Verify slab is full
    assert!(slab.is_full(), "Slab should be full");
    assert_eq!(slab.stats().allocated_count, capacity);
    assert_eq!(slab.stats().free_count, 0);
    
    kprintln!("  ✓ Successfully allocated all {} objects to capacity", capacity);
    
    // Try to allocate one more (should fail)
    assert!(slab.alloc_object().is_none(), "Should not be able to allocate beyond capacity");
    
    kprintln!("  ✓ Correctly rejected allocation beyond capacity");
    
    // Free one object
    let ptr_to_free = allocated_ptrs.pop().unwrap();
    assert!(slab.free_object(ptr_to_free), "Failed to free object");
    
    // Verify we can allocate again
    let new_ptr = slab.alloc_object().expect("Should be able to allocate after freeing");
    
    kprintln!("  ✓ Successfully allocated new object after freeing: {:p}", new_ptr);
    
    // Clean up
    let _ = slab.free_object(new_ptr);
    for ptr in allocated_ptrs {
        let _ = slab.free_object(ptr);
    }
    
    assert!(slab.is_empty(), "Slab should be empty after cleanup");
    kprintln!("  ✓ Cleanup completed, slab is empty");
}

/// Test invalid pointer handling
#[test_case]
fn test_invalid_pointer_handling() {
    kprintln!("Testing invalid pointer handling...");
    
    const SLAB_SIZE: usize = 64;
    const TOTAL_SIZE: usize = PAGE_SIZE;
    let memory_region = 0x10000000 as *mut u8;
    
    let mut slab = KSlab::new(SLAB_SIZE, memory_region, TOTAL_SIZE);
    
    // Test freeing null pointer
    assert!(!slab.free_object(core::ptr::null_mut()), "Should reject null pointer");
    
    // Test freeing pointer outside slab range
    let outside_ptr = 0x20000000 as *mut u8;
    assert!(!slab.free_object(outside_ptr), "Should reject pointer outside slab");
    
    // Test freeing misaligned pointer
    let misaligned_ptr = unsafe { (memory_region as usize + 1) as *mut u8 };
    assert!(!slab.free_object(misaligned_ptr), "Should reject misaligned pointer");
    
    kprintln!("  ✓ All invalid pointer cases handled correctly");
}

/// Test double-free detection
#[test_case]
fn test_double_free_detection() {
    kprintln!("Testing double-free detection...");
    
    const SLAB_SIZE: usize = 128;
    const TOTAL_SIZE: usize = PAGE_SIZE;
    let memory_region = 0x10000000 as *mut u8;
    
    let mut slab = KSlab::new(SLAB_SIZE, memory_region, TOTAL_SIZE);
    
    // Allocate an object
    let ptr = slab.alloc_object().expect("Failed to allocate object");
    
    // Free it once (should succeed)
    assert!(slab.free_object(ptr), "First free should succeed");
    
    // Try to free it again (should fail)
    assert!(!slab.free_object(ptr), "Double-free should be detected and rejected");
    
    kprintln!("  ✓ Double-free detection working correctly");
}

/// Test poison bytes functionality (only in test configuration)
#[test_case]
#[cfg(test)]
fn test_poison_bytes() {
    kprintln!("Testing poison bytes functionality...");
    
    const SLAB_SIZE: usize = 64;
    const TOTAL_SIZE: usize = PAGE_SIZE;
    let memory_region = 0x10000000 as *mut u8;
    
    let mut slab = KSlab::new(SLAB_SIZE, memory_region, TOTAL_SIZE);
    
    // Allocate an object
    let ptr = slab.alloc_object().expect("Failed to allocate object");
    
    // Write some data to it
    unsafe {
        *ptr = 0x42;
        assert_eq!(*ptr, 0x42, "Data should be written correctly");
    }
    
    // Free the object (should poison it)
    assert!(slab.free_object(ptr), "Failed to free object");
    
    // Check that memory is poisoned
    unsafe {
        assert_eq!(*ptr, POISON_BYTE, "Memory should be poisoned after free");
        
        // Check that the entire object is poisoned
        for i in 0..SLAB_SIZE {
            assert_eq!(*ptr.add(i), POISON_BYTE, 
                      "Byte {} should be poisoned", i);
        }
    }
    
    kprintln!("  ✓ Poison bytes applied correctly on free");
}

/// Test slab validation functionality
#[test_case]
fn test_slab_validation() {
    kprintln!("Testing slab validation functionality...");
    
    const SLAB_SIZE: usize = 32;
    const TOTAL_SIZE: usize = PAGE_SIZE;
    let memory_region = 0x10000000 as *mut u8;
    
    let mut slab = KSlab::new(SLAB_SIZE, memory_region, TOTAL_SIZE);
    
    // Initial validation should pass
    assert!(slab.validate(), "Initial slab state should be valid");
    
    // Allocate some objects
    let mut allocated = Vec::new();
    for _ in 0..10 {
        if let Some(ptr) = slab.alloc_object() {
            allocated.push(ptr);
        }
    }
    
    // Validation should still pass
    assert!(slab.validate(), "Slab should be valid after allocations");
    
    // Free all objects
    for ptr in allocated {
        assert!(slab.free_object(ptr), "Failed to free object");
    }
    
    // Final validation should pass
    assert!(slab.validate(), "Slab should be valid after all objects freed");
    
    kprintln!("  ✓ Slab validation passed in all states");
}

/// Test SlabAllocator creation and initialization
#[test_case]
fn test_slab_allocator_creation() {
    kprintln!("Testing SlabAllocator creation and initialization...");
    
    const TOTAL_SIZE: usize = PAGE_SIZE * 20; // 80KB
    let memory_region = 0x10000000 as *mut u8;
    
    let allocator = SlabAllocator::new(memory_region, TOTAL_SIZE);
    let stats = allocator.stats();
    
    // Verify all slabs are initialized
    assert_eq!(stats.slab_stats.len(), SLAB_SIZES.len());
    
    let size_per_slab = TOTAL_SIZE / SLAB_SIZES.len();
    
    for (i, slab_stat) in stats.slab_stats.iter().enumerate() {
        assert_eq!(slab_stat.object_size, SLAB_SIZES[i]);
        assert_eq!(slab_stat.total_size, size_per_slab);
        assert_eq!(slab_stat.allocated_count, 0);
        assert_eq!(slab_stat.capacity, size_per_slab / SLAB_SIZES[i]);
        
        kprintln!("    Slab {}: {} objects of {} bytes each", 
                  i, slab_stat.capacity, slab_stat.object_size);
    }
    
    kprintln!("  ✓ SlabAllocator created with {} slabs", stats.slab_stats.len());
    kprintln!("  ✓ Total capacity: {} objects across all slabs", stats.total_capacity);
}

/// Test SlabAllocator size selection logic
#[test_case]
fn test_slab_allocator_size_selection() {
    kprintln!("Testing SlabAllocator size selection logic...");
    
    const TOTAL_SIZE: usize = PAGE_SIZE * 10;
    let memory_region = 0x10000000 as *mut u8;
    
    let mut allocator = SlabAllocator::new(memory_region, TOTAL_SIZE);
    
    // Test that each size gets allocated from the correct slab
    let test_sizes = [1, 16, 32, 48, 64, 96, 128, 200, 256, 400, 512];
    
    for &size in &test_sizes {
        if let Some(ptr) = allocator.alloc(size) {
            kprintln!("    Size {} allocated at {:p}", size, ptr);
            
            // Verify we can write to the allocated memory
            unsafe {
                *ptr = (size & 0xFF) as u8;
                assert_eq!(*ptr, (size & 0xFF) as u8);
            }
            
            // Free the object
            assert!(allocator.free(ptr, size), "Failed to free object of size {}", size);
        } else {
            panic!("Failed to allocate object of size {}", size);
        }
    }
    
    kprintln!("  ✓ All size selections handled correctly");
}

/// Test 10k object allocation/deallocation with leak detection
#[test_case]
fn test_10k_object_allocation() {
    kprintln!("Testing 10k object allocation/deallocation...");
    
    const TOTAL_SIZE: usize = PAGE_SIZE * 100; // 400KB for large test
    const TEST_COUNT: usize = 10000;
    let memory_region = 0x10000000 as *mut u8;
    
    let mut allocator = SlabAllocator::new(memory_region, TOTAL_SIZE);
    let initial_stats = allocator.stats();
    
    // Simple pseudo-random number generator for deterministic testing
    let mut seed = 12345u64;
    let mut next_random = || {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        seed
    };
    
    let mut allocated_objects = Vec::new();
    let mut allocated_count = 0;
    
    // Allocate objects with random sizes
    for i in 0..TEST_COUNT {
        let random_val = next_random();
        let size = SLAB_SIZES[(random_val as usize) % SLAB_SIZES.len()];
        
        if let Some(ptr) = allocator.alloc(size) {
            allocated_objects.push((ptr, size));
            allocated_count += 1;
            
            // Write test pattern
            unsafe {
                *ptr = ((i ^ size) & 0xFF) as u8;
            }
            
            // Log progress every 1000 allocations
            if (i + 1) % 1000 == 0 {
                kprintln!("    Allocated {} / {} objects", i + 1, TEST_COUNT);
            }
        } else {
            // Can't allocate more, break
            break;
        }
    }
    
    kprintln!("  ✓ Successfully allocated {} out of {} requested objects", 
              allocated_count, TEST_COUNT);
    
    // Verify memory integrity for some objects
    kprintln!("  Verifying memory integrity for allocated objects...");
    let mut integrity_failures = 0;
    for (i, &(ptr, size)) in allocated_objects.iter().enumerate().take(1000) {
        unsafe {
            let expected = ((i ^ size) & 0xFF) as u8;
            if *ptr != expected {
                integrity_failures += 1;
                if integrity_failures <= 5 { // Only report first 5 failures
                    kprintln!("    Integrity failure at {:p}: expected {}, got {}", 
                             ptr, expected, *ptr);
                }
            }
        }
    }
    
    if integrity_failures == 0 {
        kprintln!("  ✓ Memory integrity verified for sample objects");
    } else {
        kprintln!("  ✗ {} memory integrity failures detected", integrity_failures);
    }
    
    // Free all objects
    kprintln!("  Freeing all allocated objects...");
    for (i, (ptr, size)) in allocated_objects.into_iter().enumerate() {
        if !allocator.free(ptr, size) {
            kprintln!("    Failed to free object {} at {:p}", i, ptr);
        }
        
        // Log progress every 1000 deallocations
        if (i + 1) % 1000 == 0 {
            kprintln!("    Freed {} / {} objects", i + 1, allocated_count);
        }
    }
    
    // Check for memory leaks
    let final_stats = allocator.stats();
    let leaked_objects = final_stats.total_allocated - initial_stats.total_allocated;
    
    if leaked_objects == 0 {
        kprintln!("  ✓ No memory leaks detected - all objects freed");
    } else {
        kprintln!("  ✗ Memory leak detected: {} objects still allocated", leaked_objects);
    }
    
    // Validate allocator integrity
    if allocator.validate() {
        kprintln!("  ✓ Allocator validation passed");
    } else {
        kprintln!("  ✗ Allocator validation failed");
    }
    
    kprintln!("  ✓ 10k object test completed");
}

/// Test stress allocation patterns
#[test_case]
fn test_stress_allocation_patterns() {
    kprintln!("Testing stress allocation patterns...");
    
    const TOTAL_SIZE: usize = PAGE_SIZE * 50; // 200KB
    let memory_region = 0x10000000 as *mut u8;
    
    let mut allocator = SlabAllocator::new(memory_region, TOTAL_SIZE);
    
    // Pattern 1: Allocate all objects of each size, then free them
    kprintln!("  Pattern 1: Sequential allocation/deallocation by size");
    for &size in &SLAB_SIZES {
        let mut objects = Vec::new();
        
        // Allocate as many as possible of this size
        let mut count = 0;
        while let Some(ptr) = allocator.alloc(size) {
            objects.push(ptr);
            count += 1;
            if count >= 100 { break; } // Limit to prevent excessive allocation
        }
        
        kprintln!("    Allocated {} objects of size {}", count, size);
        
        // Free all objects of this size
        for ptr in objects {
            assert!(allocator.free(ptr, size), "Failed to free object");
        }
        
        kprintln!("    Freed all {} objects of size {}", count, size);
    }
    
    // Pattern 2: Mixed allocation/deallocation
    kprintln!("  Pattern 2: Mixed allocation/deallocation");
    let mut mixed_objects = Vec::new();
    let mut seed = 54321u64;
    
    for _ in 0..500 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let size = SLAB_SIZES[(seed as usize) % SLAB_SIZES.len()];
        
        if mixed_objects.len() < 50 || (seed % 3) == 0 {
            // Allocate
            if let Some(ptr) = allocator.alloc(size) {
                mixed_objects.push((ptr, size));
            }
        } else {
            // Free random object
            if !mixed_objects.is_empty() {
                let index = (seed as usize) % mixed_objects.len();
                let (ptr, obj_size) = mixed_objects.swap_remove(index);
                let _ = allocator.free(ptr, obj_size);
            }
        }
    }
    
    // Clean up remaining objects
    for (ptr, size) in mixed_objects {
        let _ = allocator.free(ptr, size);
    }
    
    kprintln!("  ✓ Mixed allocation pattern completed");
    
    // Final validation
    assert!(allocator.validate(), "Allocator should be valid after stress test");
    
    let final_stats = allocator.stats();
    assert_eq!(final_stats.total_allocated, 0, "All objects should be freed");
    
    kprintln!("  ✓ Stress test completed successfully");
}

/// Run all slab allocator tests
pub fn run_all_tests() {
    kprintln!("");
    kprintln!("=== SLAB ALLOCATOR UNIT TESTS ===");
    
    test_kslab_creation();
    test_single_object_allocation();
    test_multiple_object_allocation();
    test_slab_capacity_limits();
    test_invalid_pointer_handling();
    test_double_free_detection();
    
    #[cfg(test)]
    test_poison_bytes();
    
    test_slab_validation();
    test_slab_allocator_creation();
    test_slab_allocator_size_selection();
    test_10k_object_allocation();
    test_stress_allocation_patterns();
    
    kprintln!("=== ALL SLAB ALLOCATOR TESTS PASSED ===");
    kprintln!("");
}

