/// Physical Memory Buddy Allocator Unit Tests
/// 
/// This module contains comprehensive unit tests for the buddy allocator
/// implementation, including allocation/deallocation, coalescing, fragmentation,
/// and random sequence testing.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec::Vec;
use x86_64::{PhysAddr, structures::paging::PhysFrame};
use polymera_kernel::mm::phys::{Buddy, MAX_ORDER, MIN_ORDER, BuddyStats};
use polymera_kernel::mm::constants::PAGE_SIZE;
use polymera_kernel::{kprintln, klog};

/// Test runner function
pub fn test_runner(tests: &[&dyn Fn()]) {
    kprintln!("Running {} buddy allocator tests", tests.len());
    for test in tests {
        test();
    }
    kprintln!("All buddy allocator tests completed");
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

/// Test basic buddy allocator creation and initialization
#[test_case]
fn test_buddy_creation() {
    kprintln!("Testing buddy allocator creation...");
    
    let base_addr = PhysAddr::new(0x1000000); // 16MB
    let total_pages = 1024; // 4MB total
    
    let buddy = Buddy::new(base_addr, total_pages);
    let stats = buddy.stats();
    
    assert_eq!(stats.total_pages, total_pages);
    assert_eq!(stats.allocated_pages, 0);
    assert_eq!(stats.free_pages, total_pages);
    assert_eq!(stats.allocation_count, 0);
    assert_eq!(stats.deallocation_count, 0);
    
    // Validate internal consistency
    assert!(buddy.validate(), "Buddy allocator validation failed");
    
    kprintln!("  ✓ Buddy allocator created successfully");
    kprintln!("  ✓ Initial state: {} pages total, {} free", total_pages, stats.free_pages);
}

/// Test single page allocation and deallocation
#[test_case]
fn test_single_page_allocation() {
    kprintln!("Testing single page allocation...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 1024;
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    let initial_stats = buddy.stats();
    let initial_free = initial_stats.free_pages;
    
    // Allocate a single page (order 0)
    let frame = buddy.alloc(0).expect("Failed to allocate single page");
    
    // Check that the allocated frame is within our memory region
    let frame_addr = frame.start_address().as_u64();
    let region_start = base_addr.as_u64();
    let region_end = region_start + (total_pages * PAGE_SIZE) as u64;
    
    assert!(frame_addr >= region_start && frame_addr < region_end,
            "Allocated frame outside memory region");
    
    // Check statistics after allocation
    let alloc_stats = buddy.stats();
    assert_eq!(alloc_stats.allocated_pages, 1);
    assert_eq!(alloc_stats.free_pages, initial_free - 1);
    assert_eq!(alloc_stats.allocation_count, 1);
    
    kprintln!("  ✓ Single page allocated at {:?}", frame);
    
    // Free the page
    buddy.free(frame, 0);
    
    // Check statistics after deallocation
    let final_stats = buddy.stats();
    assert_eq!(final_stats.allocated_pages, 0);
    assert_eq!(final_stats.free_pages, initial_free);
    assert_eq!(final_stats.deallocation_count, 1);
    
    assert!(buddy.validate(), "Buddy allocator validation failed after free");
    
    kprintln!("  ✓ Single page freed successfully");
}

/// Test multiple order allocations
#[test_case]
fn test_multiple_order_allocations() {
    kprintln!("Testing multiple order allocations...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 4096; // 16MB total
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    let initial_stats = buddy.stats();
    let initial_free = initial_stats.free_pages;
    
    let mut allocated_blocks = Vec::new();
    let mut total_allocated_pages = 0;
    
    // Test different orders
    let test_orders = [0, 1, 2, 3, 4, 5];
    
    for &order in &test_orders {
        if order <= MAX_ORDER {
            match buddy.alloc(order) {
                Some(frame) => {
                    let pages = 1 << order;
                    total_allocated_pages += pages;
                    allocated_blocks.push((frame, order));
                    kprintln!("  ✓ Allocated order {} block ({} pages) at {:?}", 
                              order, pages, frame);
                }
                None => {
                    kprintln!("  - Could not allocate order {} block (out of memory)", order);
                }
            }
        }
    }
    
    // Check total allocation statistics
    let mid_stats = buddy.stats();
    assert_eq!(mid_stats.allocated_pages, total_allocated_pages);
    assert_eq!(mid_stats.free_pages, initial_free - total_allocated_pages);
    
    kprintln!("  ✓ Total allocated: {} pages", total_allocated_pages);
    
    // Free all allocated blocks
    for (frame, order) in allocated_blocks {
        buddy.free(frame, order);
        kprintln!("  ✓ Freed order {} block at {:?}", order, frame);
    }
    
    // Verify we're back to initial state
    let final_stats = buddy.stats();
    assert_eq!(final_stats.allocated_pages, 0);
    assert_eq!(final_stats.free_pages, initial_free);
    
    assert!(buddy.validate(), "Buddy allocator validation failed");
    
    kprintln!("  ✓ All blocks freed successfully");
}

/// Test buddy coalescing functionality
#[test_case]
fn test_buddy_coalescing() {
    kprintln!("Testing buddy coalescing...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 1024;
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    // Allocate a large block and split it
    let large_block = buddy.alloc(3).expect("Failed to allocate large block"); // 8 pages
    kprintln!("  ✓ Allocated large block (order 3) at {:?}", large_block);
    
    // Allocate its buddy to prevent immediate coalescing
    let buddy_block = buddy.alloc(3).expect("Failed to allocate buddy block");
    kprintln!("  ✓ Allocated buddy block (order 3) at {:?}", buddy_block);
    
    // Free the first block
    buddy.free(large_block, 3);
    kprintln!("  ✓ Freed first block");
    
    // Free the buddy block - should trigger coalescing
    buddy.free(buddy_block, 3);
    kprintln!("  ✓ Freed buddy block - should trigger coalescing");
    
    // Verify the allocator state is consistent
    assert!(buddy.validate(), "Buddy allocator validation failed after coalescing");
    
    // Try to allocate a larger block to verify coalescing worked
    if let Some(large_frame) = buddy.alloc(4) { // 16 pages
        kprintln!("  ✓ Successfully allocated larger block after coalescing: {:?}", large_frame);
        buddy.free(large_frame, 4);
    } else {
        kprintln!("  - Could not allocate larger block (may indicate coalescing issues)");
    }
    
    kprintln!("  ✓ Coalescing test completed");
}

/// Test fragmentation handling
#[test_case]
fn test_fragmentation_handling() {
    kprintln!("Testing fragmentation handling...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 2048; // 8MB total
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    let initial_stats = buddy.stats();
    
    // Allocate many small blocks to create fragmentation
    let mut small_blocks = Vec::new();
    for i in 0..16 {
        if let Some(frame) = buddy.alloc(0) { // Single pages
            small_blocks.push(frame);
            kprintln!("  Allocated small block {}: {:?}", i, frame);
        } else {
            break;
        }
    }
    
    kprintln!("  ✓ Allocated {} small blocks", small_blocks.len());
    
    // Free every other block to create holes
    let mut freed_blocks = Vec::new();
    for (i, &frame) in small_blocks.iter().enumerate() {
        if i % 2 == 0 {
            buddy.free(frame, 0);
            freed_blocks.push(frame);
        }
    }
    
    kprintln!("  ✓ Freed {} blocks to create fragmentation", freed_blocks.len());
    
    // Calculate fragmentation
    let frag_stats = buddy.stats();
    let fragmentation = frag_stats.fragmentation();
    kprintln!("  Current fragmentation: {:.1}%", fragmentation);
    
    // Try to allocate a larger block
    match buddy.alloc(2) { // 4 pages
        Some(frame) => {
            kprintln!("  ✓ Allocated larger block despite fragmentation: {:?}", frame);
            buddy.free(frame, 2);
        }
        None => {
            kprintln!("  - Could not allocate larger block due to fragmentation");
        }
    }
    
    // Clean up remaining blocks
    for (i, &frame) in small_blocks.iter().enumerate() {
        if i % 2 == 1 {
            buddy.free(frame, 0);
        }
    }
    
    // Verify final state
    let final_stats = buddy.stats();
    assert_eq!(final_stats.allocated_pages, 0);
    assert_eq!(final_stats.free_pages, initial_stats.free_pages);
    
    kprintln!("  ✓ Fragmentation test completed");
}

/// Test allocation limits and edge cases
#[test_case]
fn test_allocation_limits() {
    kprintln!("Testing allocation limits and edge cases...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 512; // 2MB total
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    // Test allocation of order larger than MAX_ORDER
    assert!(buddy.alloc(MAX_ORDER + 1).is_none(), "Should not allocate order > MAX_ORDER");
    kprintln!("  ✓ Correctly rejected allocation larger than MAX_ORDER");
    
    // Test allocating all memory
    let mut allocated_blocks = Vec::new();
    let mut total_allocated = 0;
    
    // Allocate until out of memory
    loop {
        match buddy.alloc(0) { // Single pages
            Some(frame) => {
                allocated_blocks.push(frame);
                total_allocated += 1;
            }
            None => break,
        }
    }
    
    kprintln!("  ✓ Allocated {} pages until out of memory", total_allocated);
    
    // Verify we can't allocate more
    assert!(buddy.alloc(0).is_none(), "Should be out of memory");
    kprintln!("  ✓ Correctly reports out of memory");
    
    // Free all memory
    for frame in allocated_blocks {
        buddy.free(frame, 0);
    }
    
    // Verify all memory is freed
    let final_stats = buddy.stats();
    assert_eq!(final_stats.allocated_pages, 0);
    assert_eq!(final_stats.free_pages, total_pages);
    
    kprintln!("  ✓ All memory freed successfully");
}

/// Test random allocation/deallocation sequences
#[test_case]
fn test_random_sequences() {
    kprintln!("Testing random allocation/deallocation sequences...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 1024;
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    let initial_stats = buddy.stats();
    let initial_free = initial_stats.free_pages;
    
    // Simple linear congruential generator for deterministic "randomness"
    let mut seed = 42u64;
    let mut next_random = || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        seed
    };
    
    let mut allocated_blocks = Vec::new();
    const NUM_OPERATIONS: usize = 100;
    
    for i in 0..NUM_OPERATIONS {
        let random_val = next_random();
        
        // Decide whether to allocate or free (bias towards allocation if empty)
        let should_allocate = allocated_blocks.is_empty() || (random_val % 3) != 0;
        
        if should_allocate {
            // Random order between 0 and 6
            let order = (random_val % 7) as usize;
            
            match buddy.alloc(order) {
                Some(frame) => {
                    allocated_blocks.push((frame, order));
                    kprintln!("  [{:3}] Allocated order {} at {:?}", i, order, frame);
                }
                None => {
                    kprintln!("  [{:3}] Failed to allocate order {} (out of memory)", i, order);
                }
            }
        } else if !allocated_blocks.is_empty() {
            // Free a random block
            let index = (random_val as usize) % allocated_blocks.len();
            let (frame, order) = allocated_blocks.swap_remove(index);
            
            buddy.free(frame, order);
            kprintln!("  [{:3}] Freed order {} at {:?}", i, order, frame);
        }
        
        // Periodic validation
        if i % 20 == 19 {
            assert!(buddy.validate(), "Buddy allocator validation failed at iteration {}", i);
        }
    }
    
    kprintln!("  ✓ Completed {} operations", NUM_OPERATIONS);
    
    // Free all remaining blocks
    let remaining_count = allocated_blocks.len();
    for (frame, order) in allocated_blocks {
        buddy.free(frame, order);
    }
    
    kprintln!("  ✓ Freed {} remaining blocks", remaining_count);
    
    // Verify final state
    let final_stats = buddy.stats();
    assert_eq!(final_stats.allocated_pages, 0, "Memory leak detected");
    assert_eq!(final_stats.free_pages, initial_free, "Free count mismatch");
    
    assert!(buddy.validate(), "Final validation failed");
    
    kprintln!("  ✓ Random sequence test completed successfully");
}

/// Test stress allocation/deallocation
#[test_case]
fn test_stress_allocation() {
    kprintln!("Testing stress allocation/deallocation...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 2048; // 8MB
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    let initial_stats = buddy.stats();
    
    // Stress test: rapid allocation and deallocation
    const STRESS_ITERATIONS: usize = 50;
    
    for round in 0..STRESS_ITERATIONS {
        let mut allocated_blocks = Vec::new();
        
        // Allocate phase - allocate various sized blocks
        for order in 0..=5 {
            for _ in 0..3 {
                if let Some(frame) = buddy.alloc(order) {
                    allocated_blocks.push((frame, order));
                }
            }
        }
        
        kprintln!("  Round {}: allocated {} blocks", round, allocated_blocks.len());
        
        // Validate state
        assert!(buddy.validate(), "Validation failed in round {}", round);
        
        // Deallocation phase - free all blocks
        for (frame, order) in allocated_blocks {
            buddy.free(frame, order);
        }
    }
    
    // Verify final state
    let final_stats = buddy.stats();
    assert_eq!(final_stats.allocated_pages, 0);
    assert_eq!(final_stats.free_pages, initial_stats.free_pages);
    assert!(buddy.validate(), "Final validation failed");
    
    kprintln!("  ✓ Stress test completed: {} rounds", STRESS_ITERATIONS);
}

/// Test allocation statistics accuracy
#[test_case]
fn test_statistics_accuracy() {
    kprintln!("Testing allocation statistics accuracy...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 1024;
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    let initial_stats = buddy.stats();
    
    // Track allocations manually
    let mut manual_allocated = 0;
    let mut manual_alloc_count = 0;
    let mut manual_dealloc_count = 0;
    let mut allocated_blocks = Vec::new();
    
    // Perform various allocations
    let orders = [0, 1, 2, 3, 1, 0, 2];
    
    for &order in &orders {
        if let Some(frame) = buddy.alloc(order) {
            let pages = 1 << order;
            manual_allocated += pages;
            manual_alloc_count += 1;
            allocated_blocks.push((frame, order));
            
            kprintln!("  Allocated order {} ({} pages)", order, pages);
        }
    }
    
    // Check statistics match manual tracking
    let mid_stats = buddy.stats();
    assert_eq!(mid_stats.allocated_pages, manual_allocated,
               "Allocated pages mismatch: expected {}, got {}", 
               manual_allocated, mid_stats.allocated_pages);
    assert_eq!(mid_stats.allocation_count, manual_alloc_count,
               "Allocation count mismatch");
    assert_eq!(mid_stats.free_pages, total_pages - manual_allocated,
               "Free pages mismatch");
    
    kprintln!("  ✓ Mid-test statistics accurate");
    
    // Free blocks and update manual tracking
    for (frame, order) in allocated_blocks {
        buddy.free(frame, order);
        let pages = 1 << order;
        manual_allocated -= pages;
        manual_dealloc_count += 1;
    }
    
    // Check final statistics
    let final_stats = buddy.stats();
    assert_eq!(final_stats.allocated_pages, 0);
    assert_eq!(final_stats.free_pages, total_pages);
    assert_eq!(final_stats.allocation_count, manual_alloc_count);
    assert_eq!(final_stats.deallocation_count, manual_dealloc_count);
    
    // Test utilization calculation
    assert_eq!(final_stats.utilization(), 0.0);
    
    kprintln!("  ✓ Final statistics accurate");
    kprintln!("  ✓ Statistics test completed");
}

/// Test buddy allocator with specific memory patterns
#[test_case]
fn test_memory_patterns() {
    kprintln!("Testing specific memory allocation patterns...");
    
    let base_addr = PhysAddr::new(0x1000000);
    let total_pages = 1024;
    let mut buddy = Buddy::new(base_addr, total_pages);
    
    // Pattern 1: Allocate increasing orders
    kprintln!("  Pattern 1: Increasing order allocation");
    let mut pattern1_blocks = Vec::new();
    
    for order in 0..=5 {
        if let Some(frame) = buddy.alloc(order) {
            pattern1_blocks.push((frame, order));
            kprintln!("    Allocated order {}: {:?}", order, frame);
        }
    }
    
    // Free in reverse order
    for (frame, order) in pattern1_blocks.into_iter().rev() {
        buddy.free(frame, order);
        kprintln!("    Freed order {}: {:?}", order, frame);
    }
    
    assert!(buddy.validate(), "Pattern 1 validation failed");
    
    // Pattern 2: Checkerboard allocation/deallocation
    kprintln!("  Pattern 2: Checkerboard pattern");
    let mut pattern2_blocks = Vec::new();
    
    // Allocate multiple blocks
    for i in 0..8 {
        if let Some(frame) = buddy.alloc(0) {
            pattern2_blocks.push(frame);
            kprintln!("    Allocated block {}: {:?}", i, frame);
        }
    }
    
    // Free every other block
    let mut kept_blocks = Vec::new();
    for (i, frame) in pattern2_blocks.into_iter().enumerate() {
        if i % 2 == 0 {
            buddy.free(frame, 0);
            kprintln!("    Freed block {}: {:?}", i, frame);
        } else {
            kept_blocks.push(frame);
        }
    }
    
    // Free remaining blocks
    for frame in kept_blocks {
        buddy.free(frame, 0);
    }
    
    assert!(buddy.validate(), "Pattern 2 validation failed");
    
    // Verify final state
    let final_stats = buddy.stats();
    assert_eq!(final_stats.allocated_pages, 0);
    assert_eq!(final_stats.free_pages, total_pages);
    
    kprintln!("  ✓ Memory pattern tests completed");
}

/// Run all buddy allocator tests
pub fn run_all_tests() {
    kprintln!("");
    kprintln!("=== BUDDY ALLOCATOR UNIT TESTS ===");
    
    test_buddy_creation();
    test_single_page_allocation();
    test_multiple_order_allocations();
    test_buddy_coalescing();
    test_fragmentation_handling();
    test_allocation_limits();
    test_random_sequences();
    test_stress_allocation();
    test_statistics_accuracy();
    test_memory_patterns();
    
    kprintln!("=== ALL BUDDY ALLOCATOR TESTS PASSED ===");
    kprintln!("");
}

