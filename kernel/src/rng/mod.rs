//! Randomness Proxy Module
//! 
//! This module provides a centralized randomness interface that can be:
//! - Seeded in Determinism mode for reproducible testing
//! - Controlled via sys_debug operations
//! - Used throughout the kernel for consistent random number generation

use core::sync::atomic::{AtomicU64, Ordering};
use crate::determinism;

/// Randomness proxy configuration
pub struct RngProxyConfig {
    /// Whether the proxy is in deterministic mode
    pub deterministic: bool,
    /// Current seed value
    pub seed: AtomicU64,
    /// Current state for deterministic generation
    pub state: AtomicU64,
}

impl RngProxyConfig {
    /// Create a new RNG proxy configuration
    pub const fn new() -> Self {
        Self {
            deterministic: false,
            seed: AtomicU64::new(0),
            state: AtomicU64::new(0),
        }
    }
    
    /// Set the seed and enable deterministic mode
    pub fn set_seed(&self, new_seed: u64) {
        self.seed.store(new_seed, Ordering::Relaxed);
        self.state.store(new_seed, Ordering::Relaxed);
        
        crate::kprintln!("[RNG] Seed set to 0x{:016x}, deterministic mode enabled", new_seed);
    }
    
    /// Get the current seed
    pub fn get_seed(&self) -> u64 {
        self.seed.load(Ordering::Relaxed)
    }
    
    /// Check if deterministic mode is enabled
    pub fn is_deterministic(&self) -> bool {
        self.deterministic
    }
    
    /// Enable deterministic mode
    pub fn enable_deterministic(&self) {
        self.deterministic = true;
        crate::kprintln!("[RNG] Deterministic mode enabled");
    }
    
    /// Disable deterministic mode
    pub fn disable_deterministic(&self) {
        self.deterministic = false;
        crate::kprintln!("[RNG] Deterministic mode disabled");
    }
    
    /// Generate next deterministic random number
    pub fn next_deterministic(&self) -> u64 {
        let current_state = self.state.load(Ordering::Relaxed);
        
        // Linear congruential generator for determinism
        let new_state = current_state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state.store(new_state, Ordering::Relaxed);
        
        new_state
    }
    
    /// Reset to initial seed state
    pub fn reset(&self) {
        let seed = self.seed.load(Ordering::Relaxed);
        self.state.store(seed, Ordering::Relaxed);
        crate::kprintln!("[RNG] Reset to seed 0x{:016x}", seed);
    }
}

/// Global RNG proxy configuration
static mut RNG_PROXY_CONFIG: RngProxyConfig = RngProxyConfig::new();

/// Get the global RNG proxy configuration
pub fn get_config() -> &'static RngProxyConfig {
    unsafe { &RNG_PROXY_CONFIG }
}

/// Get a mutable reference to the global RNG proxy configuration
pub fn get_config_mut() -> &'static mut RngProxyConfig {
    unsafe { &mut RNG_PROXY_CONFIG }
}

/// Set the RNG seed (called from sys_debug)
pub fn set_seed(seed: u64) -> bool {
    let config = get_config();
    
    // Set the seed
    config.set_seed(seed);
    
    // Enable deterministic mode
    config.enable_deterministic();
    
    // Also enable determinism mode if not already enabled
    if !determinism::is_determinism_enabled() {
        determinism::enable_determinism(seed);
    }
    
    crate::kprintln!("[RNG] Seed set to 0x{:016x} via sys_debug", seed);
    true
}

/// Get the current RNG seed
pub fn get_seed() -> u64 {
    get_config().get_seed()
}

/// Check if deterministic mode is enabled
pub fn is_deterministic() -> bool {
    get_config().is_deterministic()
}

/// Generate a random number
/// 
/// In deterministic mode, uses the seeded LCG.
/// In non-deterministic mode, uses system entropy (simulated).
pub fn random() -> u64 {
    #[cfg(feature = "deterministic")]
    {
        if crate::determinism::is_determinism_enabled() {
            return crate::determinism::get_deterministic_random();
        }
    }
    
    let config = get_config();
    
    if config.is_deterministic() {
        // Use deterministic generation
        config.next_deterministic()
    } else {
        // Use system entropy (simulated for now)
        // In a real implementation, this would use hardware RNG or system entropy
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        
        // Simple hash-based generation
        let mut hash = count;
        hash ^= hash >> 13;
        hash = hash.wrapping_mul(0x5bd1e995);
        hash ^= hash >> 15;
        hash
    }
}

/// Generate a random number in a range
pub fn random_range(min: u64, max: u64) -> u64 {
    #[cfg(feature = "deterministic")]
    {
        if crate::determinism::is_determinism_enabled() {
            return crate::determinism::get_deterministic_random_range(min, max);
        }
    }
    
    let range = max - min;
    min + (random() % range)
}

/// Generate random bytes
pub fn random_bytes(buffer: &mut [u8]) {
    for (i, byte) in buffer.iter_mut().enumerate() {
        if i % 8 == 0 {
            // Generate new random value every 8 bytes
            let random_value = random();
            *byte = (random_value & 0xFF) as u8;
        } else {
            // Use remaining bits from previous random value
            let random_value = random();
            *byte = ((random_value >> ((i % 8) * 8)) & 0xFF) as u8;
        }
    }
}

/// Generate a random UUID (simplified)
pub fn random_uuid() -> [u8; 16] {
    let mut uuid = [0u8; 16];
    random_bytes(&mut uuid);
    
    // Set version (4) and variant bits
    uuid[6] = (uuid[6] & 0x0F) | 0x40; // Version 4
    uuid[8] = (uuid[8] & 0x3F) | 0x80; // Variant 1
    
    uuid
}

/// Initialize the RNG proxy
pub fn init() {
    crate::kprintln!("[RNG] Initializing randomness proxy...");
    
    let config = get_config_mut();
    
    // Check if determinism mode is enabled
    if determinism::is_determinism_enabled() {
        let seed = determinism::get_replay_seed();
        config.set_seed(seed);
        config.enable_deterministic();
        crate::kprintln!("[RNG] Initialized with determinism seed: 0x{:016x}", seed);
    } else {
        crate::kprintln!("[RNG] Initialized in non-deterministic mode");
    }
}

/// Print RNG proxy status
pub fn print_status() {
    let config = get_config();
    
    crate::kprintln!("");
    crate::kprintln!("=== RNG PROXY STATUS ===");
    crate::kprintln!("Deterministic Mode: {}", config.is_deterministic());
    crate::kprintln!("Current Seed: 0x{:016x}", config.get_seed());
    crate::kprintln!("Current State: 0x{:016x}", config.state.load(Ordering::Relaxed));
    crate::kprintln!("Determinism Integration: {}", determinism::is_determinism_enabled());
    
    if determinism::is_determinism_enabled() {
        crate::kprintln!("Determinism Seed: 0x{:016x}", determinism::get_replay_seed());
    }
    
    crate::kprintln!("==========================");
    crate::kprintln!("");
}

/// Test the RNG proxy functionality
#[cfg(test)]
pub fn test_rng_proxy() {
    kprintln!("[TEST] Testing RNG proxy functionality...");
    
    // Test 1: Seed setting
    let test_seed = 0x1234567890abcdef;
    set_seed(test_seed);
    assert_eq!(get_seed(), test_seed, "Seed should be set correctly");
    assert!(is_deterministic(), "Deterministic mode should be enabled");
    
    // Test 2: Deterministic generation
    let rng1 = random();
    let rng2 = random();
    assert_ne!(rng1, rng2, "Consecutive random numbers should be different");
    
    // Test 3: Range generation
    let range_value = random_range(10, 100);
    assert!(range_value >= 10 && range_value < 100, "Range value should be within bounds");
    
    // Test 4: Byte generation
    let mut bytes = [0u8; 16];
    random_bytes(&mut bytes);
    assert!(bytes.iter().any(|&b| b != 0), "Random bytes should not be all zeros");
    
    // Test 5: UUID generation
    let uuid = random_uuid();
    assert_eq!(uuid[6] & 0xF0, 0x40, "UUID version should be 4");
    assert_eq!(uuid[8] & 0xC0, 0x80, "UUID variant should be 1");
    
    kprintln!("[TEST] RNG proxy functionality test PASSED");
}



