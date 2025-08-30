//! Determinism Tests Module
//! 
//! This module provides comprehensive testing for deterministic behavior:
//! - Harness-based testing for IPC operations
//! - End-to-end determinism validation
//! - Byte-identical log verification

pub mod harness;
pub mod e2e;

pub use harness::*;
pub use e2e::*;

/// Run all determinism tests
pub fn run_all_determinism_tests() -> Result<(), &'static str> {
    kprintln!("[DETERMINISM_TESTS] Running all determinism tests...");
    
    // Run harness tests
    kprintln!("[DETERMINISM_TESTS] Running harness tests...");
    let harness_result = run_harness_tests();
    
    // Run e2e tests
    kprintln!("[DETERMINISM_TESTS] Running e2e tests...");
    let e2e_result = run_e2e_tests();
    
    // Check results
    if harness_result.is_ok() && e2e_result.is_ok() {
        kprintln!("[DETERMINISM_TESTS] ✅ All determinism tests passed");
        Ok(())
    } else {
        kprintln!("[DETERMINISM_TESTS] ❌ Some determinism tests failed");
        Err("Determinism tests failed")
    }
}

/// Run harness-based determinism tests
fn run_harness_tests() -> Result<(), &'static str> {
    let config = DeterminismTestConfig::default();
    let mut harness = DeterminismTestHarness::new(config);
    harness.run_test_sequence()
}

/// Run end-to-end determinism tests
fn run_e2e_tests() -> Result<(), &'static str> {
    let success = run_deterministic_e2e_test();
    if success {
        Ok(())
    } else {
        Err("E2E determinism test failed")
    }
}
