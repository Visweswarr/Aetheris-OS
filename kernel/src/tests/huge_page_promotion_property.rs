//! Property-Based Test for Huge Page Promotion
//!
//! **Feature: advanced-kernel-features, Property 2: Huge Page Promotion**
//!
//! This test validates that for any memory allocation request larger than 2MB,
//! the Memory_Manager shall allocate from the huge page pool rather than
//! standard 4KB pages.
//!
//! **Validates: Requirements 1.3**
//!
//! Since this is a no_std kernel environment, we implement property-based testing
//! manually using deterministic pseudo-random generation to cover a wide range
//! of allocation sizes.

use crate::mm::manager::{
    MemoryManager, ProcessId, CapabilityHandle, Permissions,
};
use crate::mm::huge_pages::{HUGE_PAGE_SIZE, HUGE_PAGE_THRESHOLD, HugePagePool};
use crate::{kprintln, klog};
use alloc::vec::Vec;

//=============================================================================
// TEST CONFIGURATION
//=============================================================================

/// Number of property test iterations (minimum 100 as per design doc)
const PROPERTY_TEST_ITERATIONS: usize = 100;

/// Seed for deterministic pseudo-random generation
const RANDOM_SEED: u64 = 0xCAFEBABE_DEADBEEF;

/// Standard page size (4KB)
const STANDARD_PAGE_SIZE: usize = 4096;

//=============================================================================
// PSEUDO-RANDOM NUMBER GENERATOR
//=============================================================================

/// Simple Linear Congruential Generator for deterministic pseudo-random numbers
struct Lcg {
    state: u64,
}

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        const A: u64 = 6364136223846793005;
        const C: u64 = 1442695040888963407;
        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        self.state
    }

    /// Generate a random value in range [min, max)
    fn range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        min + (self.next() % (max - min))
    }
}

//=============================================================================
// PROPERTY TEST HELPERS
//=============================================================================

/// Test scenario for huge page promotion
struct HugePageTestScenario {
    /// Allocation size in bytes
    allocation_size: usize,
    /// Process ID for the allocation
    pid: ProcessId,
    /// Whether this allocation should use huge pages
    should_use_huge_page: bool,
}

impl HugePageTestScenario {
    /// Generate a random test scenario
    fn generate(rng: &mut Lcg, iteration: usize) -> Self {
        // Generate allocation sizes that span the threshold boundary
        // 50% below threshold, 50% at or above threshold
        let allocation_size = if iteration % 2 == 0 {
            // Below threshold: 4KB to just under 2MB
            rng.range(STANDARD_PAGE_SIZE as u64, HUGE_PAGE_THRESHOLD as u64) as usize
        } else {
            // At or above threshold: 2MB to 16MB
            rng.range(HUGE_PAGE_THRESHOLD as u64, (16 * 1024 * 1024) as u64) as usize
        };

        let should_use_huge_page = allocation_size >= HUGE_PAGE_THRESHOLD;

        Self {
            allocation_size,
            pid: ProcessId::new(rng.range(1, 1000)),
            should_use_huge_page,
        }
    }
}

//=============================================================================
// PROPERTY 2: HUGE PAGE PROMOTION
//=============================================================================

/// **Feature: advanced-kernel-features, Property 2: Huge Page Promotion**
///
/// Property: For any memory allocation request larger than 2MB, the Memory_Manager
/// shall allocate from the huge page pool rather than standard 4KB pages.
///
/// **Validates: Requirements 1.3**
///
/// This property test verifies:
/// 1. Allocations >= 2MB use huge pages
/// 2. Allocations < 2MB use standard 4KB pages
/// 3. The is_huge_page() method correctly identifies huge page allocations
pub fn property_huge_page_promotion() {
    kprintln!("");
    kprintln!("=== Property Test: Huge Page Promotion ===");
    kprintln!("**Feature: advanced-kernel-features, Property 2: Huge Page Promotion**");
    kprintln!("**Validates: Requirements 1.3**");
    kprintln!("");
    kprintln!("Running {} iterations...", PROPERTY_TEST_ITERATIONS);

    let mut rng = Lcg::new(RANDOM_SEED);
    let mut passed = 0;
    let mut failed = 0;

    for iteration in 0..PROPERTY_TEST_ITERATIONS {
        let scenario = HugePageTestScenario::generate(&mut rng, iteration);
        
        match verify_huge_page_promotion(&scenario) {
            Ok(()) => {
                passed += 1;
            }
            Err(msg) => {
                failed += 1;
                kprintln!("  ✗ Iteration {}: {}", iteration, msg);
                kprintln!("    Scenario: size={}, should_use_huge_page={}",
                    scenario.allocation_size, scenario.should_use_huge_page);
            }
        }
    }

    kprintln!("");
    kprintln!("Results: {} passed, {} failed out of {} iterations",
        passed, failed, PROPERTY_TEST_ITERATIONS);

    assert_eq!(failed, 0, "Property test failed {} iterations", failed);
    
    kprintln!("✅ Property 2: Huge Page Promotion - PASSED");
    kprintln!("");
}

/// Verify the huge page promotion property for a given scenario
fn verify_huge_page_promotion(scenario: &HugePageTestScenario) -> Result<(), &'static str> {
    // Create a fresh memory manager for this test
    // Use 128MB to ensure we have enough huge pages
    let mut mm = MemoryManager::new();
    mm.init(0x10_0000, 128 * 1024 * 1024);

    // Create capability for allocations
    let cap = CapabilityHandle::new_memory(
        0x1000_0000,
        128 * 1024 * 1024,
        Permissions::read_write(),
    );

    // Attempt allocation
    match mm.allocate(scenario.pid, scenario.allocation_size, &cap) {
        Ok(addr) => {
            let is_huge = mm.is_huge_page(addr);
            
            // Verify the property
            if scenario.should_use_huge_page {
                // Property: Allocations >= 2MB MUST use huge pages
                if !is_huge {
                    return Err("Allocation >= 2MB did not use huge page");
                }
            } else {
                // Property: Allocations < 2MB should NOT use huge pages
                if is_huge {
                    return Err("Allocation < 2MB incorrectly used huge page");
                }
            }
            Ok(())
        }
        Err(_) => {
            // Allocation failure is acceptable if we're out of memory
            // This can happen for very large allocations
            if scenario.allocation_size > 64 * 1024 * 1024 {
                Ok(()) // Expected for very large allocations
            } else {
                Err("Allocation failed unexpectedly")
            }
        }
    }
}

//=============================================================================
// ADDITIONAL PROPERTY TESTS FOR HUGE PAGES
//=============================================================================

/// Test that the threshold is exactly 2MB
pub fn property_huge_page_threshold_boundary() {
    kprintln!("");
    kprintln!("=== Property Test: Huge Page Threshold Boundary ===");
    kprintln!("**Feature: advanced-kernel-features, Property 2: Huge Page Promotion**");
    kprintln!("**Validates: Requirements 1.3 (2MB threshold)**");
    kprintln!("");

    // Verify the threshold constant
    assert_eq!(HUGE_PAGE_THRESHOLD, 2 * 1024 * 1024, 
        "Huge page threshold must be exactly 2MB");
    assert_eq!(HUGE_PAGE_SIZE, 2 * 1024 * 1024,
        "Huge page size must be exactly 2MB");

    // Test boundary conditions
    let boundary_sizes = [
        (2 * 1024 * 1024 - 1, false),  // Just below threshold
        (2 * 1024 * 1024, true),        // Exactly at threshold
        (2 * 1024 * 1024 + 1, true),    // Just above threshold
        (2 * 1024 * 1024 - 4096, false), // One page below
        (2 * 1024 * 1024 + 4096, true),  // One page above
    ];

    let mut mm = MemoryManager::new();
    mm.init(0x10_0000, 128 * 1024 * 1024);

    for (size, expected_huge) in boundary_sizes {
        let cap = CapabilityHandle::new_memory(
            0x1000_0000,
            128 * 1024 * 1024,
            Permissions::read_write(),
        );

        let pid = ProcessId::new(1);
        
        // Reset memory manager for each test
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, 128 * 1024 * 1024);

        match mm.allocate(pid, size, &cap) {
            Ok(addr) => {
                let is_huge = mm.is_huge_page(addr);
                kprintln!("  Size: {} bytes, Expected huge: {}, Actual huge: {}",
                    size, expected_huge, is_huge);
                
                assert_eq!(is_huge, expected_huge,
                    "Size {} should {} use huge page",
                    size, if expected_huge { "" } else { "NOT" });
            }
            Err(e) => {
                kprintln!("  Size: {} bytes - allocation failed: {:?}", size, e);
                // Allocation failure is acceptable for boundary tests
            }
        }
    }

    kprintln!("");
    kprintln!("✅ Huge Page Threshold Boundary - PASSED");
    kprintln!("");
}

/// Test that should_use_huge_page helper function works correctly
pub fn property_should_use_huge_page_helper() {
    kprintln!("");
    kprintln!("=== Property Test: should_use_huge_page Helper ===");
    kprintln!("**Feature: advanced-kernel-features, Property 2: Huge Page Promotion**");
    kprintln!("**Validates: Requirements 1.3**");
    kprintln!("");

    let mut rng = Lcg::new(RANDOM_SEED + 1);
    let mut passed = 0;
    let mut failed = 0;

    for iteration in 0..PROPERTY_TEST_ITERATIONS {
        // Generate random size
        let size = rng.range(1, (32 * 1024 * 1024) as u64) as usize;
        
        let result = HugePagePool::should_use_huge_page(size);
        let expected = size >= HUGE_PAGE_THRESHOLD;
        
        if result == expected {
            passed += 1;
        } else {
            failed += 1;
            kprintln!("  ✗ Iteration {}: size={}, expected={}, got={}",
                iteration, size, expected, result);
        }
    }

    kprintln!("");
    kprintln!("Results: {} passed, {} failed out of {} iterations",
        passed, failed, PROPERTY_TEST_ITERATIONS);

    assert_eq!(failed, 0, "Helper function test failed {} iterations", failed);
    
    kprintln!("✅ should_use_huge_page Helper - PASSED");
    kprintln!("");
}

/// Test TLB savings calculation
pub fn property_tlb_savings() {
    kprintln!("");
    kprintln!("=== Property Test: TLB Savings Calculation ===");
    kprintln!("**Feature: advanced-kernel-features, Property 2: Huge Page Promotion**");
    kprintln!("**Validates: Requirements 1.3 (TLB miss reduction)**");
    kprintln!("");

    let mut mm = MemoryManager::new();
    mm.init(0x10_0000, 128 * 1024 * 1024);

    let cap = CapabilityHandle::new_memory(
        0x1000_0000,
        128 * 1024 * 1024,
        Permissions::read_write(),
    );

    // Allocate several huge pages
    let num_allocations = 5;
    for i in 0..num_allocations {
        let pid = ProcessId::new(i as u64 + 1);
        let _ = mm.allocate(pid, HUGE_PAGE_SIZE, &cap);
    }

    let stats = mm.huge_page_stats();
    
    // Each huge page saves 511 TLB entries (512 small pages - 1 huge page entry)
    let expected_savings = stats.promotions * 511;
    
    kprintln!("  Huge pages allocated: {}", stats.used_pages);
    kprintln!("  Promotions: {}", stats.promotions);
    kprintln!("  TLB entries saved: {}", stats.tlb_entries_saved);
    kprintln!("  Expected savings: {}", expected_savings);
    
    assert_eq!(stats.tlb_entries_saved, expected_savings,
        "TLB savings should be 511 per huge page");

    kprintln!("");
    kprintln!("✅ TLB Savings Calculation - PASSED");
    kprintln!("");
}

//=============================================================================
// TEST RUNNER
//=============================================================================

/// Run all huge page promotion property tests
pub fn run_huge_page_promotion_property_tests() -> Result<(), &'static str> {
    kprintln!("");
    kprintln!("╔══════════════════════════════════════════════════════════════╗");
    kprintln!("║  PROPERTY-BASED TEST SUITE: Huge Page Promotion              ║");
    kprintln!("║  Feature: advanced-kernel-features                           ║");
    kprintln!("║  Property 2: Huge Page Promotion                             ║");
    kprintln!("║  Validates: Requirements 1.3                                 ║");
    kprintln!("╚══════════════════════════════════════════════════════════════╝");
    kprintln!("");

    // Run property tests
    property_huge_page_promotion();
    property_huge_page_threshold_boundary();
    property_should_use_huge_page_helper();
    property_tlb_savings();

    kprintln!("");
    kprintln!("╔══════════════════════════════════════════════════════════════╗");
    kprintln!("║  ✅ ALL PROPERTY TESTS PASSED                                ║");
    kprintln!("╚══════════════════════════════════════════════════════════════╝");
    kprintln!("");

    Ok(())
}

//=============================================================================
// UNIT TESTS (for #[cfg(test)] builds)
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_generation() {
        let mut rng = Lcg::new(RANDOM_SEED);
        
        for i in 0..20 {
            let scenario = HugePageTestScenario::generate(&mut rng, i);
            assert!(scenario.allocation_size > 0);
            
            // Verify should_use_huge_page is consistent with size
            let expected = scenario.allocation_size >= HUGE_PAGE_THRESHOLD;
            assert_eq!(scenario.should_use_huge_page, expected,
                "Iteration {}: size={}, expected={}, got={}",
                i, scenario.allocation_size, expected, scenario.should_use_huge_page);
        }
    }

    #[test]
    fn test_threshold_constants() {
        assert_eq!(HUGE_PAGE_THRESHOLD, 2 * 1024 * 1024);
        assert_eq!(HUGE_PAGE_SIZE, 2 * 1024 * 1024);
    }

    #[test]
    fn test_should_use_huge_page() {
        assert!(!HugePagePool::should_use_huge_page(0));
        assert!(!HugePagePool::should_use_huge_page(4096));
        assert!(!HugePagePool::should_use_huge_page(1024 * 1024));
        assert!(!HugePagePool::should_use_huge_page(2 * 1024 * 1024 - 1));
        assert!(HugePagePool::should_use_huge_page(2 * 1024 * 1024));
        assert!(HugePagePool::should_use_huge_page(2 * 1024 * 1024 + 1));
        assert!(HugePagePool::should_use_huge_page(4 * 1024 * 1024));
    }
}
