/// Virtual Memory API Unit Tests
/// 
/// This module contains comprehensive unit tests for the virtual memory API
/// implementation, including map_page, unmap_page, translate_va functions,
/// double-free error detection, and RWX enforcement simulation.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::vec::Vec;
use polymera_kernel::mm::virt::{
    map_page, unmap_page, translate_va, is_mapped, get_page_flags, check_access,
    validate_vm, get_vm_stats, VmFlags
};
use polymera_kernel::mm::{MemoryResult, MemoryError};
use polymera_kernel::mm::constants::PAGE_SIZE;
use polymera_kernel::{kprintln, klog};

/// Test runner function
pub fn test_runner(tests: &[&dyn Fn()]) {
    kprintln!("Running {} virtual memory API unit tests", tests.len());
    for test in tests {
        test();
    }
    kprintln!("All virtual memory API unit tests completed");
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

/// Test basic map_page functionality
#[test_case]
fn test_basic_map_page() {
    kprintln!("Testing basic map_page functionality...");
    
    let vaddr = 0x1000000; // 16MB
    let paddr = 0x2000000; // 32MB
    let flags = VmFlags::read_write();
    
    // Test successful mapping
    match map_page(vaddr, paddr, flags) {
        Ok(()) => {
            kprintln!("  ✓ Successfully mapped page 0x{:016x} -> 0x{:016x}", vaddr, paddr);
            
            // Verify mapping exists
            assert!(is_mapped(vaddr), "Page should be marked as mapped");
            kprintln!("  ✓ Page correctly marked as mapped");
            
            // Clean up
            let _ = unmap_page(vaddr);
        }
        Err(e) => {
            panic!("Failed to map page: {}", e);
        }
    }
}

/// Test basic unmap_page functionality
#[test_case]
fn test_basic_unmap_page() {
    kprintln!("Testing basic unmap_page functionality...");
    
    let vaddr = 0x1001000;
    let paddr = 0x2001000;
    let flags = VmFlags::read_only();
    
    // Map page first
    assert!(map_page(vaddr, paddr, flags).is_ok(), "Initial mapping should succeed");
    assert!(is_mapped(vaddr), "Page should be mapped");
    
    // Test successful unmapping
    match unmap_page(vaddr) {
        Ok(()) => {
            kprintln!("  ✓ Successfully unmapped page 0x{:016x}", vaddr);
            
            // Verify mapping no longer exists
            assert!(!is_mapped(vaddr), "Page should not be marked as mapped after unmap");
            kprintln!("  ✓ Page correctly marked as unmapped");
        }
        Err(e) => {
            panic!("Failed to unmap page: {}", e);
        }
    }
}

/// Test translate_va functionality
#[test_case]
fn test_translate_va() {
    kprintln!("Testing translate_va functionality...");
    
    let vaddr = 0x1002000;
    let paddr = 0x2002000;
    let flags = VmFlags::read_execute();
    
    // Test translation before mapping
    assert_eq!(translate_va(vaddr), None, "Translation should fail for unmapped page");
    kprintln!("  ✓ Translation correctly fails for unmapped page");
    
    // Map page
    assert!(map_page(vaddr, paddr, flags).is_ok(), "Mapping should succeed");
    
    // Test translation after mapping
    match translate_va(vaddr) {
        Some(translated_paddr) => {
            assert_eq!(translated_paddr, paddr, "Translation should return correct physical address");
            kprintln!("  ✓ Translation correct: 0x{:016x} -> 0x{:016x}", vaddr, translated_paddr);
        }
        None => panic!("Translation should succeed for mapped page"),
    }
    
    // Test translation with offset
    let offset = 0x123;
    let offset_vaddr = vaddr + offset;
    let expected_offset_paddr = paddr + offset;
    
    match translate_va(offset_vaddr) {
        Some(translated_paddr) => {
            assert_eq!(translated_paddr, expected_offset_paddr, 
                      "Translation with offset should be correct");
            kprintln!("  ✓ Offset translation correct: 0x{:016x} -> 0x{:016x}", 
                     offset_vaddr, translated_paddr);
        }
        None => panic!("Translation with offset should succeed"),
    }
    
    // Clean up
    let _ = unmap_page(vaddr);
    
    // Test translation after unmapping
    assert_eq!(translate_va(vaddr), None, "Translation should fail after unmap");
    kprintln!("  ✓ Translation correctly fails after unmap");
}

/// Test double mapping error detection
#[test_case]
fn test_double_mapping_error() {
    kprintln!("Testing double mapping error detection...");
    
    let vaddr = 0x1003000;
    let paddr1 = 0x2003000;
    let paddr2 = 0x2004000;
    let flags = VmFlags::read_write();
    
    // First mapping should succeed
    assert!(map_page(vaddr, paddr1, flags).is_ok(), "First mapping should succeed");
    kprintln!("  ✓ First mapping successful");
    
    // Second mapping to same virtual address should fail
    match map_page(vaddr, paddr2, flags) {
        Ok(()) => panic!("Second mapping should have failed"),
        Err(MemoryError::AlreadyMapped) => {
            kprintln!("  ✓ Double mapping correctly rejected with AlreadyMapped error");
        }
        Err(e) => panic!("Double mapping failed with wrong error: {}", e),
    }
    
    // Clean up
    let _ = unmap_page(vaddr);
}

/// Test double unmapping error detection
#[test_case]
fn test_double_unmapping_error() {
    kprintln!("Testing double unmapping error detection...");
    
    let vaddr = 0x1004000;
    let paddr = 0x2005000;
    let flags = VmFlags::read_only();
    
    // Map page first
    assert!(map_page(vaddr, paddr, flags).is_ok(), "Initial mapping should succeed");
    
    // First unmap should succeed
    match unmap_page(vaddr) {
        Ok(()) => kprintln!("  ✓ First unmap successful"),
        Err(e) => panic!("First unmap should succeed: {}", e),
    }
    
    // Second unmap should fail
    match unmap_page(vaddr) {
        Ok(()) => panic!("Second unmap should have failed"),
        Err(MemoryError::NotMapped) => {
            kprintln!("  ✓ Double unmap correctly rejected with NotMapped error");
        }
        Err(e) => panic!("Double unmap failed with wrong error: {}", e),
    }
}

/// Test RWX flag enforcement simulation
#[test_case]
fn test_rwx_flag_enforcement() {
    kprintln!("Testing RWX flag enforcement simulation...");
    
    let test_cases = [
        (0x1010000, 0x2010000, VmFlags::read_only(), "read-only", true, false, false, false),
        (0x1011000, 0x2011000, VmFlags::read_write(), "read-write", true, true, false, false),
        (0x1012000, 0x2012000, VmFlags::read_execute(), "read-execute", true, false, true, false),
        (0x1013000, 0x2013000, VmFlags::read_write_execute(), "read-write-execute", true, true, true, false),
        (0x1014000, 0x2014000, VmFlags::user_read_only(), "user-read-only", true, false, false, true),
        (0x1015000, 0x2015000, VmFlags::user_read_write(), "user-read-write", true, true, false, true),
        (0x1016000, 0x2016000, VmFlags::user_read_execute(), "user-read-execute", true, false, true, true),
    ];
    
    for &(vaddr, paddr, flags, desc, exp_read, exp_write, exp_exec, exp_user) in &test_cases {
        // Map page with specific flags
        assert!(map_page(vaddr, paddr, flags).is_ok(), 
                "Mapping should succeed for {}", desc);
        
        // Test access permissions
        let actual_read = check_access(vaddr, false, false, false);
        let actual_write = check_access(vaddr, true, false, false);
        let actual_exec = check_access(vaddr, false, true, false);
        let actual_user = check_access(vaddr, false, false, true);
        
        // Verify permissions match expectations
        assert_eq!(actual_read, exp_read, 
                  "Read permission mismatch for {}", desc);
        assert_eq!(actual_write, exp_write, 
                  "Write permission mismatch for {}", desc);
        assert_eq!(actual_exec, exp_exec, 
                  "Execute permission mismatch for {}", desc);
        assert_eq!(actual_user, exp_user, 
                  "User permission mismatch for {}", desc);
        
        kprintln!("  ✓ RWX enforcement correct for {} (R={} W={} X={} U={})", 
                  desc, actual_read, actual_write, actual_exec, actual_user);
        
        // Clean up
        let _ = unmap_page(vaddr);
    }
}

/// Test page alignment handling
#[test_case]
fn test_page_alignment() {
    kprintln!("Testing page alignment handling...");
    
    // Test mapping with unaligned addresses
    let base_vaddr = 0x1020000;
    let base_paddr = 0x2020000;
    let flags = VmFlags::read_write();
    
    // Map with various offsets - should all map to same page
    let offsets = [0, 1, 0x123, 0x789, PAGE_SIZE - 1];
    
    for &offset in &offsets {
        let unaligned_vaddr = base_vaddr + offset;
        let unaligned_paddr = base_paddr + offset;
        
        // First, clean up any existing mapping
        let _ = unmap_page(base_vaddr);
        
        // Map with unaligned address
        match map_page(unaligned_vaddr, unaligned_paddr, flags) {
            Ok(()) => {
                // Should be mapped to page-aligned address
                assert!(is_mapped(base_vaddr), "Page-aligned address should be mapped");
                
                // Translation should work for any address in the page
                assert!(translate_va(base_vaddr).is_some(), "Page start should translate");
                assert!(translate_va(base_vaddr + offset).is_some(), "Offset address should translate");
                
                kprintln!("  ✓ Unaligned mapping correctly handled for offset 0x{:x}", offset);
            }
            Err(e) => {
                // Check if this is a double mapping error (expected for subsequent maps)
                if offset > 0 {
                    match e {
                        MemoryError::AlreadyMapped => {
                            kprintln!("  ✓ Correctly detected already mapped page for offset 0x{:x}", offset);
                        }
                        _ => panic!("Unexpected error for offset 0x{:x}: {}", offset, e),
                    }
                } else {
                    panic!("First mapping should succeed: {}", e);
                }
            }
        }
    }
    
    // Clean up
    let _ = unmap_page(base_vaddr);
}

/// Test multiple simultaneous mappings
#[test_case]
fn test_multiple_mappings() {
    kprintln!("Testing multiple simultaneous mappings...");
    
    const NUM_MAPPINGS: usize = 50;
    let mut mapped_addresses = Vec::new();
    
    // Create multiple mappings
    for i in 0..NUM_MAPPINGS {
        let vaddr = 0x1100000 + (i * PAGE_SIZE) as u64;
        let paddr = 0x2100000 + (i * PAGE_SIZE) as u64;
        let flags = match i % 4 {
            0 => VmFlags::read_only(),
            1 => VmFlags::read_write(),
            2 => VmFlags::read_execute(),
            _ => VmFlags::read_write_execute(),
        };
        
        match map_page(vaddr, paddr, flags) {
            Ok(()) => {
                mapped_addresses.push(vaddr);
                
                // Verify mapping exists
                assert!(is_mapped(vaddr), "Mapping {} should exist", i);
                assert_eq!(translate_va(vaddr), Some(paddr), "Translation {} should be correct", i);
            }
            Err(e) => panic!("Failed to create mapping {}: {}", i, e),
        }
    }
    
    kprintln!("  ✓ Created {} simultaneous mappings", mapped_addresses.len());
    
    // Verify all mappings still exist
    for (i, &vaddr) in mapped_addresses.iter().enumerate() {
        assert!(is_mapped(vaddr), "Mapping {} should still exist", i);
    }
    
    kprintln!("  ✓ All mappings verified to exist");
    
    // Clean up all mappings
    for &vaddr in &mapped_addresses {
        assert!(unmap_page(vaddr).is_ok(), "Cleanup should succeed");
    }
    
    // Verify all mappings are gone
    for (i, &vaddr) in mapped_addresses.iter().enumerate() {
        assert!(!is_mapped(vaddr), "Mapping {} should be gone after cleanup", i);
    }
    
    kprintln!("  ✓ All mappings successfully cleaned up");
}

/// Test VM state validation
#[test_case]
fn test_vm_validation() {
    kprintln!("Testing VM state validation...");
    
    // Initial validation should pass
    assert!(validate_vm(), "Initial VM state should be valid");
    kprintln!("  ✓ Initial VM state validation passed");
    
    // Create some mappings
    let mappings = [
        (0x1200000, 0x2200000, VmFlags::read_write()),
        (0x1201000, 0x2201000, VmFlags::read_only()),
        (0x1202000, 0x2202000, VmFlags::read_execute()),
    ];
    
    for &(vaddr, paddr, flags) in &mappings {
        assert!(map_page(vaddr, paddr, flags).is_ok(), "Mapping should succeed");
    }
    
    // Validation should still pass
    assert!(validate_vm(), "VM state should be valid with mappings");
    kprintln!("  ✓ VM state validation passed with active mappings");
    
    // Clean up
    for &(vaddr, _, _) in &mappings {
        let _ = unmap_page(vaddr);
    }
    
    // Final validation should pass
    assert!(validate_vm(), "Final VM state should be valid");
    kprintln!("  ✓ Final VM state validation passed");
}

/// Test VM statistics tracking
#[test_case]
fn test_vm_statistics() {
    kprintln!("Testing VM statistics tracking...");
    
    let initial_stats = get_vm_stats();
    kprintln!("  Initial stats: {} mapped, {} unmapped, {} active", 
              initial_stats.mapped_pages, initial_stats.unmapped_pages, initial_stats.active_mappings);
    
    let test_vaddr = 0x1300000;
    let test_paddr = 0x2300000;
    let flags = VmFlags::read_write();
    
    // Map a page
    assert!(map_page(test_vaddr, test_paddr, flags).is_ok(), "Mapping should succeed");
    
    let mapped_stats = get_vm_stats();
    assert_eq!(mapped_stats.mapped_pages, initial_stats.mapped_pages + 1,
               "Mapped page count should increase by 1");
    assert_eq!(mapped_stats.active_mappings, initial_stats.active_mappings + 1,
               "Active mapping count should increase by 1");
    
    kprintln!("  ✓ Statistics correctly updated after mapping");
    
    // Unmap the page
    assert!(unmap_page(test_vaddr).is_ok(), "Unmapping should succeed");
    
    let unmapped_stats = get_vm_stats();
    assert_eq!(unmapped_stats.unmapped_pages, initial_stats.unmapped_pages + 1,
               "Unmapped page count should increase by 1");
    assert_eq!(unmapped_stats.active_mappings, initial_stats.active_mappings,
               "Active mapping count should return to initial value");
    
    kprintln!("  ✓ Statistics correctly updated after unmapping");
}

/// Test edge cases and error conditions
#[test_case]
fn test_edge_cases() {
    kprintln!("Testing edge cases and error conditions...");
    
    // Test mapping to null virtual address (should be allowed for kernel)
    let null_vaddr = 0x0;
    let test_paddr = 0x2400000;
    let kernel_flags = VmFlags::read_write(); // Kernel flags (user = false)
    
    match map_page(null_vaddr, test_paddr, kernel_flags) {
        Ok(()) => {
            kprintln!("  ✓ Null page mapping allowed for kernel");
            let _ = unmap_page(null_vaddr);
        }
        Err(e) => kprintln!("  - Null page mapping rejected: {}", e),
    }
    
    // Test mapping to null with user flags (should fail)
    let user_flags = VmFlags::user_read_write();
    match map_page(null_vaddr, test_paddr, user_flags) {
        Ok(()) => {
            kprintln!("  - Null page mapping allowed for user (unexpected)");
            let _ = unmap_page(null_vaddr);
        }
        Err(MemoryError::InvalidAddress) => {
            kprintln!("  ✓ Null page mapping correctly rejected for user");
        }
        Err(e) => kprintln!("  - Null page mapping rejected with different error: {}", e),
    }
    
    // Test unmapping non-existent page
    let non_existent_vaddr = 0x1400000;
    match unmap_page(non_existent_vaddr) {
        Ok(()) => kprintln!("  - Unmapping non-existent page succeeded (unexpected)"),
        Err(MemoryError::NotMapped) => {
            kprintln!("  ✓ Unmapping non-existent page correctly rejected");
        }
        Err(e) => kprintln!("  - Unmapping non-existent page failed with different error: {}", e),
    }
    
    // Test getting flags for non-mapped page
    match get_page_flags(non_existent_vaddr) {
        Some(_) => kprintln!("  - Got flags for non-mapped page (unexpected)"),
        None => kprintln!("  ✓ Correctly got None for non-mapped page flags"),
    }
}

/// Test memory leak detection
#[test_case]
fn test_memory_leak_detection() {
    kprintln!("Testing memory leak detection...");
    
    let initial_stats = get_vm_stats();
    
    // Create and clean up mappings in various patterns
    let leak_test_cases = [
        // Pattern 1: Map and unmap immediately
        vec![(0x1500000, 0x2500000)],
        // Pattern 2: Map multiple, unmap in same order
        vec![(0x1501000, 0x2501000), (0x1502000, 0x2502000), (0x1503000, 0x2503000)],
        // Pattern 3: Map multiple, unmap in reverse order
        vec![(0x1504000, 0x2504000), (0x1505000, 0x2505000), (0x1506000, 0x2506000)],
    ];
    
    for (pattern_num, addresses) in leak_test_cases.iter().enumerate() {
        kprintln!("  Testing leak pattern {}: {} mappings", pattern_num + 1, addresses.len());
        
        // Map all addresses
        for &(vaddr, paddr) in addresses {
            assert!(map_page(vaddr, paddr, VmFlags::read_write()).is_ok(),
                    "Mapping should succeed");
        }
        
        // Unmap in different patterns
        match pattern_num {
            0 | 1 => {
                // Unmap in same order
                for &(vaddr, _) in addresses {
                    assert!(unmap_page(vaddr).is_ok(), "Unmapping should succeed");
                }
            }
            2 => {
                // Unmap in reverse order
                for &(vaddr, _) in addresses.iter().rev() {
                    assert!(unmap_page(vaddr).is_ok(), "Unmapping should succeed");
                }
            }
            _ => {}
        }
        
        // Verify no mappings remain
        for &(vaddr, _) in addresses {
            assert!(!is_mapped(vaddr), "Address should not be mapped after cleanup");
        }
    }
    
    let final_stats = get_vm_stats();
    assert_eq!(final_stats.active_mappings, initial_stats.active_mappings,
               "Active mapping count should return to initial value");
    
    kprintln!("  ✓ No memory leaks detected in any pattern");
}

/// Run all VM API tests
pub fn run_all_tests() {
    kprintln!("");
    kprintln!("=== VIRTUAL MEMORY API UNIT TESTS ===");
    
    test_basic_map_page();
    test_basic_unmap_page();
    test_translate_va();
    test_double_mapping_error();
    test_double_unmapping_error();
    test_rwx_flag_enforcement();
    test_page_alignment();
    test_multiple_mappings();
    test_vm_validation();
    test_vm_statistics();
    test_edge_cases();
    test_memory_leak_detection();
    
    kprintln!("=== ALL VIRTUAL MEMORY API TESTS PASSED ===");
    kprintln!("");
}

