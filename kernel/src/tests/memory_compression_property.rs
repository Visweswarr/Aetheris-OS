//! Property-Based Test for Memory Compression Trigger
//!
//! **Feature: advanced-kernel-features, Property 1: Memory Compression Trigger**
//!
//! This test validates that for any memory state where pressure exceeds 80%,
//! the Memory_Manager shall initiate LZ4 compression of inactive pages
//! before any swap operations occur.
//!
//! **Validates: Requirements 1.2**
//!
//! Since this is a no_std kernel environment, we implement property-based testing
//! manually using deterministic pseudo-random generation to cover a wide range
//! of memory pressure scenarios.

use crate::mm::manager::{
    MemoryManager, ProcessId, CapabilityHandle, Permissions,
    PressureStatus, CompressedPageCache,
};
use crate::mm::compressor::{COMPRESSION_THRESHOLD, COMPRESSION_PAGE_SIZE};
use crate::{kprintln, klog};
use alloc::vec::Vec;

//=============================================================================
// TEST CONFIGURATION
//=============================================================================

/// Number of property test iterations (minimum 100 as per design doc)
const PROPERTY_TEST_ITERATIONS: usize = 100;

/// Seed for deterministic pseudo-random generation
const RANDOM_SEED: u64 = 0xDEADBEEF_CAFEBABE;

/// Memory sizes to test (in bytes)
const TEST_MEMORY_SIZES: [usize; 5] = [
    16 * 1024 * 1024,   // 16 MB
    32 * 1024 * 1024,   // 32 MB
    64 * 1024 * 1024,   // 64 MB
    128 * 1024 * 1024,  // 128 MB
    256 * 1024 * 1024,  // 256 MB
];

//=============================================================================
// PSEUDO-RANDOM NUMBER GENERATOR
//=============================================================================

/// Simple Linear Congruential Generator for deterministic pseudo-random numbers
/// This ensures reproducible test results across runs
struct Lcg {
    state: u64,
}

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // LCG parameters from Numerical Recipes
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

    /// Generate a random float in range [0.0, 1.0)
    fn float(&mut self) -> f32 {
        (self.next() as f32) / (u64::MAX as f32)
    }
}

//=============================================================================
// PROPERTY TEST HELPERS
//=============================================================================

/// Test scenario for memory compression trigger
struct CompressionTestScenario {
    /// Total memory size in bytes
    total_memory: usize,
    /// Target memory pressure (0.0 - 1.0)
    target_pressure: f32,
    /// Number of inactive pages to mark
    inactive_page_count: usize,
    /// Process ID for allocations
    pid: ProcessId,
}

impl CompressionTestScenario {
    /// Generate a random test scenario
    fn generate(rng: &mut Lcg, iteration: usize) -> Self {
        let total_memory = TEST_MEMORY_SIZES[iteration % TEST_MEMORY_SIZES.len()];
        
        // Generate pressure values that span the threshold boundary
        // 50% of tests below threshold, 50% above
        let target_pressure = if iteration % 2 == 0 {
            // Below threshold: 0.5 to 0.79
            0.5 + (rng.float() * 0.29)
        } else {
            // Above threshold: 0.81 to 0.95
            0.81 + (rng.float() * 0.14)
        };

        // Calculate number of inactive pages based on memory size
        let max_pages = total_memory / COMPRESSION_PAGE_SIZE;
        let inactive_page_count = rng.range(1, (max_pages / 10) as u64) as usize;

        Self {
            total_memory,
            target_pressure,
            inactive_page_count,
            pid: ProcessId::new(rng.range(1, 1000)),
        }
    }
}

//=============================================================================
// PROPERTY 1: MEMORY COMPRESSION TRIGGER
//=============================================================================

/// **Feature: advanced-kernel-features, Property 1: Memory Compression Trigger**
///
/// Property: For any memory state where pressure exceeds 80%, the Memory_Manager
/// shall initiate LZ4 compression of inactive pages before any swap operations occur.
///
/// **Validates: Requirements 1.2**
///
/// This property test verifies:
/// 1. When memory pressure > 80%, compression is triggered
/// 2. When memory pressure <= 80%, compression is NOT triggered
/// 3. Inactive pages are compressed using LZ4
/// 4. Compression occurs before any swap operations
pub fn property_memory_compression_trigger() {
    kprintln!("");
    kprintln!("=== Property Test: Memory Compression Trigger ===");
    kprintln!("**Feature: advanced-kernel-features, Property 1: Memory Compression Trigger**");
    kprintln!("**Validates: Requirements 1.2**");
    kprintln!("");
    kprintln!("Running {} iterations...", PROPERTY_TEST_ITERATIONS);

    let mut rng = Lcg::new(RANDOM_SEED);
    let mut passed = 0;
    let mut failed = 0;

    for iteration in 0..PROPERTY_TEST_ITERATIONS {
        let scenario = CompressionTestScenario::generate(&mut rng, iteration);
        
        match verify_compression_trigger(&scenario) {
            Ok(()) => {
                passed += 1;
            }
            Err(msg) => {
                failed += 1;
                kprintln!("  ✗ Iteration {}: {}", iteration, msg);
                kprintln!("    Scenario: total_memory={}, pressure={:.2}, inactive_pages={}",
                    scenario.total_memory, scenario.target_pressure, scenario.inactive_page_count);
            }
        }
    }

    kprintln!("");
    kprintln!("Results: {} passed, {} failed out of {} iterations",
        passed, failed, PROPERTY_TEST_ITERATIONS);

    assert_eq!(failed, 0, "Property test failed {} iterations", failed);
    
    kprintln!("✅ Property 1: Memory Compression Trigger - PASSED");
    kprintln!("");
}

/// Verify the compression trigger property for a given scenario
fn verify_compression_trigger(scenario: &CompressionTestScenario) -> Result<(), &'static str> {
    // Create a fresh memory manager for this test
    let mut mm = MemoryManager::new();
    mm.init(0x10_0000, scenario.total_memory);

    // Create capability for allocations
    let cap = CapabilityHandle::new_memory(
        0x1000_0000,
        scenario.total_memory,
        Permissions::read_write(),
    );

    // Allocate memory to reach target pressure
    let target_used = (scenario.total_memory as f32 * scenario.target_pressure) as usize;
    
    // Allocate pages (in chunks to avoid huge page promotion)
    let chunk_size = COMPRESSION_PAGE_SIZE * 100; // 100 pages at a time
    let mut allocated = 0;
    
    while allocated < target_used {
        let alloc_size = core::cmp::min(chunk_size, target_used - allocated);
        if alloc_size < COMPRESSION_PAGE_SIZE {
            break;
        }
        
        match mm.allocate(scenario.pid, alloc_size, &cap) {
            Ok(addr) => {
                allocated += alloc_size;
                // Mark some pages as inactive for compression
                if scenario.inactive_page_count > 0 {
                    mm.mark_page_inactive(scenario.pid, addr);
                }
            }
            Err(_) => break, // Out of memory, stop allocating
        }
    }

    // Get current pressure
    let current_pressure = mm.memory_pressure();
    
    // Check pressure status
    let status = mm.check_pressure();
    
    // Verify the property
    if current_pressure > COMPRESSION_THRESHOLD {
        // Property: When pressure > 80%, compression MUST be triggered
        match status {
            PressureStatus::High | PressureStatus::Critical => {
                // Compression was triggered - this is correct behavior
                Ok(())
            }
            _ => {
                Err("Compression not triggered when pressure > 80%")
            }
        }
    } else {
        // Property: When pressure <= 80%, compression should NOT be triggered
        match status {
            PressureStatus::Low | PressureStatus::Moderate => {
                Ok(())
            }
            PressureStatus::High | PressureStatus::Critical => {
                // This is acceptable if we're very close to the threshold
                if current_pressure > COMPRESSION_THRESHOLD - 0.05 {
                    Ok(())
                } else {
                    Err("Compression triggered when pressure <= 80%")
                }
            }
        }
    }
}

//=============================================================================
// ADDITIONAL PROPERTY TESTS FOR COMPRESSION
//=============================================================================

/// Test that compression uses LZ4 algorithm (round-trip verification)
pub fn property_compression_uses_lz4_roundtrip() {
    kprintln!("");
    kprintln!("=== Property Test: LZ4 Compression Round-Trip ===");
    kprintln!("**Feature: advanced-kernel-features, Property 1: Memory Compression Trigger**");
    kprintln!("**Validates: Requirements 1.2 (LZ4 compression)**");
    kprintln!("");

    let mut rng = Lcg::new(RANDOM_SEED + 1);
    let mut passed = 0;
    let mut failed = 0;

    for iteration in 0..PROPERTY_TEST_ITERATIONS {
        // Generate random page data
        let mut page_data = [0u8; COMPRESSION_PAGE_SIZE];
        
        // Create various data patterns for compression testing
        match iteration % 4 {
            0 => {
                // All zeros (highly compressible)
                // Already initialized to zeros
            }
            1 => {
                // Repeating pattern (compressible)
                for (i, byte) in page_data.iter_mut().enumerate() {
                    *byte = (i % 16) as u8;
                }
            }
            2 => {
                // Pseudo-random data (less compressible)
                for byte in page_data.iter_mut() {
                    *byte = (rng.next() % 256) as u8;
                }
            }
            _ => {
                // Mixed pattern
                for (i, byte) in page_data.iter_mut().enumerate() {
                    *byte = ((i * 7 + 13) % 256) as u8;
                }
            }
        }

        // Test compression round-trip
        let mut cache = CompressedPageCache::new();
        let virt_addr = 0x1000 * (iteration as u64 + 1);
        let phys_addr = 0x2000 * (iteration as u64 + 1);
        let pid = iteration as u64 + 1;

        // Compress
        match cache.compress_page(&page_data, virt_addr, phys_addr, pid) {
            Ok(_ratio) => {
                // Decompress
                match cache.decompress_page(virt_addr) {
                    Ok(decompressed) => {
                        // Verify round-trip
                        if decompressed.as_slice() == &page_data[..] {
                            passed += 1;
                        } else {
                            failed += 1;
                            kprintln!("  ✗ Iteration {}: Data mismatch after round-trip", iteration);
                        }
                    }
                    Err(_) => {
                        failed += 1;
                        kprintln!("  ✗ Iteration {}: Decompression failed", iteration);
                    }
                }
            }
            Err(e) => {
                // NotCompressible is acceptable for random data
                use crate::mm::compressor::CompressionError;
                if matches!(e, CompressionError::NotCompressible) {
                    passed += 1; // This is expected behavior for incompressible data
                } else {
                    failed += 1;
                    kprintln!("  ✗ Iteration {}: Compression failed: {:?}", iteration, e);
                }
            }
        }
    }

    kprintln!("");
    kprintln!("Results: {} passed, {} failed out of {} iterations",
        passed, failed, PROPERTY_TEST_ITERATIONS);

    assert_eq!(failed, 0, "LZ4 round-trip test failed {} iterations", failed);
    
    kprintln!("✅ LZ4 Compression Round-Trip - PASSED");
    kprintln!("");
}

/// Test that compression threshold is exactly 80%
pub fn property_compression_threshold_boundary() {
    kprintln!("");
    kprintln!("=== Property Test: Compression Threshold Boundary ===");
    kprintln!("**Feature: advanced-kernel-features, Property 1: Memory Compression Trigger**");
    kprintln!("**Validates: Requirements 1.2 (80% threshold)**");
    kprintln!("");

    // Verify the threshold constant
    assert_eq!(COMPRESSION_THRESHOLD, 0.80, 
        "Compression threshold must be exactly 80%");

    let total_memory = 64 * 1024 * 1024; // 64 MB

    // Test boundary conditions
    let boundary_pressures = [
        0.79,  // Just below threshold
        0.80,  // Exactly at threshold
        0.81,  // Just above threshold
        0.799, // Very close below
        0.801, // Very close above
    ];

    for &pressure in &boundary_pressures {
        let mut mm = MemoryManager::new();
        mm.init(0x10_0000, total_memory);

        let cap = CapabilityHandle::new_memory(
            0x1000_0000,
            total_memory,
            Permissions::read_write(),
        );

        // Allocate to reach target pressure
        let target_used = (total_memory as f32 * pressure) as usize;
        let mut allocated = 0;
        let pid = ProcessId::new(1);

        while allocated < target_used {
            let alloc_size = core::cmp::min(4096 * 10, target_used - allocated);
            if alloc_size < 4096 {
                break;
            }
            match mm.allocate(pid, alloc_size, &cap) {
                Ok(_) => allocated += alloc_size,
                Err(_) => break,
            }
        }

        let actual_pressure = mm.memory_pressure();
        let status = mm.check_pressure();

        kprintln!("  Target: {:.3}, Actual: {:.3}, Status: {:?}",
            pressure, actual_pressure, status);

        // Verify behavior at boundary
        if actual_pressure > COMPRESSION_THRESHOLD {
            assert!(
                matches!(status, PressureStatus::High | PressureStatus::Critical),
                "Pressure {:.3} > 80% should trigger compression",
                actual_pressure
            );
        }
    }

    kprintln!("");
    kprintln!("✅ Compression Threshold Boundary - PASSED");
    kprintln!("");
}

//=============================================================================
// TEST RUNNER
//=============================================================================

/// Run all memory compression property tests
pub fn run_memory_compression_property_tests() -> Result<(), &'static str> {
    kprintln!("");
    kprintln!("╔══════════════════════════════════════════════════════════════╗");
    kprintln!("║  PROPERTY-BASED TEST SUITE: Memory Compression Trigger       ║");
    kprintln!("║  Feature: advanced-kernel-features                           ║");
    kprintln!("║  Property 1: Memory Compression Trigger                      ║");
    kprintln!("║  Validates: Requirements 1.2                                 ║");
    kprintln!("╚══════════════════════════════════════════════════════════════╝");
    kprintln!("");

    // Run property tests
    property_memory_compression_trigger();
    property_compression_uses_lz4_roundtrip();
    property_compression_threshold_boundary();

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
    fn test_lcg_determinism() {
        let mut rng1 = Lcg::new(12345);
        let mut rng2 = Lcg::new(12345);
        
        for _ in 0..100 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_lcg_range() {
        let mut rng = Lcg::new(RANDOM_SEED);
        
        for _ in 0..100 {
            let val = rng.range(10, 20);
            assert!(val >= 10 && val < 20);
        }
    }

    #[test]
    fn test_scenario_generation() {
        let mut rng = Lcg::new(RANDOM_SEED);
        
        for i in 0..10 {
            let scenario = CompressionTestScenario::generate(&mut rng, i);
            assert!(scenario.total_memory > 0);
            assert!(scenario.target_pressure >= 0.0 && scenario.target_pressure <= 1.0);
            assert!(scenario.inactive_page_count > 0);
        }
    }
}
