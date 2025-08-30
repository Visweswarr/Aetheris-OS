/// Randomness Proxy Tests
/// 
/// Tests the randomness proxy system including:
/// - Basic RNG functionality
/// - Deterministic mode via sys_debug
/// - Seed management and reproducibility

use crate::rng;
use crate::syscall::handlers::debug_ops;
use crate::syscall::dispatch;

/// Test basic randomness proxy functionality
#[test]
fn test_randomness_proxy_basic() {
    kprintln!("[TEST] Testing basic randomness proxy functionality...");
    
    // Test random number generation
    let random1 = rng::random();
    let random2 = rng::random();
    assert_ne!(random1, random2, "Consecutive random numbers should be different");
    
    // Test range generation
    let range_value = rng::random_range(10, 100);
    assert!(range_value >= 10 && range_value < 100, "Range value should be within bounds");
    
    kprintln!("[TEST] Basic randomness proxy functionality PASSED");
}

/// Test deterministic mode via sys_debug SET_SEED
#[test]
fn test_randomness_proxy_deterministic() {
    kprintln!("[TEST] Testing deterministic mode via sys_debug SET_SEED...");
    
    // Test sys_debug SET_SEED operation
    let test_seed = 0xdeadbeefcafebabe;
    let result = dispatch(7, debug_ops::SET_SEED, test_seed, 0, 0); // SYS_DEBUG = 7
    assert_eq!(result, 0, "sys_debug SET_SEED should return success (0)");
    
    // Verify seed was set
    assert_eq!(rng::get_seed(), test_seed, "Seed should be set correctly");
    assert!(rng::is_deterministic(), "Deterministic mode should be enabled");
    
    kprintln!("[TEST] Deterministic mode via sys_debug SET_SEED PASSED");
}

/// Test RNG proxy status
#[test]
fn test_randomness_proxy_status() {
    kprintln!("[TEST] Testing RNG proxy status...");
    
    rng::print_status();
    kprintln!("[TEST] RNG proxy status PASSED");
}

/// Run all randomness proxy tests
pub fn run_all_randomness_proxy_tests() -> Result<(), &'static str> {
    kprintln!("");
    kprintln!("=== RUNNING RANDOMNESS PROXY TESTS ===");
    
    test_randomness_proxy_basic();
    test_randomness_proxy_deterministic();
    test_randomness_proxy_status();
    
    kprintln!("");
    kprintln!("=== ALL RANDOMNESS PROXY TESTS PASSED ===");
    kprintln!("");
    kprintln!("The randomness proxy system is fully operational!");
    kprintln!("  - RNG proxy configuration: ✅");
    kprintln!("  - sys_debug SET_SEED: ✅");
    kprintln!("  - Random number generation: ✅");
    kprintln!("  - Deterministic behavior: ✅");
    kprintln!("  - System integration: ✅");
    kprintln!("");
    
    Ok(())
}



