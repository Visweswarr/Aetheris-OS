//! Determinism Module
//! 
//! This module provides deterministic behavior for Polymera OS, including:
//! - Virtualized time sources for IPC latency metrics
//! - Deterministic random number generation
//! - Replay seed support for testing
//! - Environment flag and compile-time feature support

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// Determinism configuration
pub struct DeterminismConfig {
    /// Whether determinism mode is enabled
    pub enabled: bool,
    /// Replay seed for deterministic behavior
    pub replay_seed: u64,
    /// Virtual time counter (in milliseconds)
    pub virtual_time_ms: AtomicU64,
    /// Virtual time counter (in microseconds)
    pub virtual_time_us: AtomicU64,
    /// Virtual tick counter
    pub virtual_ticks: AtomicU64,
}

impl DeterminismConfig {
    /// Create a new determinism configuration
    pub const fn new() -> Self {
        Self {
            enabled: false,
            replay_seed: 0,
            virtual_time_ms: AtomicU64::new(0),
            virtual_time_us: AtomicU64::new(0),
            virtual_ticks: AtomicU64::new(0),
        }
    }
    
    /// Enable determinism mode with a specific replay seed
    pub fn enable(&mut self, seed: u64) {
        self.enabled = true;
        self.replay_seed = seed;
        self.virtual_time_ms.store(0, Ordering::Relaxed);
        self.virtual_time_us.store(0, Ordering::Relaxed);
        self.virtual_ticks.store(0, Ordering::Relaxed);
        
        crate::kprintln!("[DETERMINISM] Enabled with replay seed: 0x{:016x}", seed);
    }
    
    /// Disable determinism mode
    pub fn disable(&mut self) {
        self.enabled = false;
        crate::kprintln!("[DETERMINISM] Disabled");
    }
    
    /// Check if determinism mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get the current replay seed
    pub fn get_replay_seed(&self) -> u64 {
        self.replay_seed
    }
    
    /// Advance virtual time by a specified amount
    pub fn advance_time(&self, ms: u64) {
        if self.enabled {
            let current_ms = self.virtual_time_ms.load(Ordering::Relaxed);
            self.virtual_time_ms.store(current_ms + ms, Ordering::Relaxed);
            
            let current_us = self.virtual_time_us.load(Ordering::Relaxed);
            self.virtual_time_us.store(current_us + (ms * 1000), Ordering::Relaxed);
        }
    }
    
    /// Advance virtual ticks by a specified amount
    pub fn advance_ticks(&self, ticks: u64) {
        if self.enabled {
            let current = self.virtual_ticks.load(Ordering::Relaxed);
            self.virtual_ticks.store(current + ticks, Ordering::Relaxed);
        }
    }
    
    /// Get current virtual time in milliseconds
    pub fn get_virtual_time_ms(&self) -> u64 {
        if self.enabled {
            self.virtual_time_ms.load(Ordering::Relaxed)
        } else {
            0 // Return 0 when not in determinism mode
        }
    }
    
    /// Get current virtual time in microseconds
pub fn get_virtual_time_us(&self) -> u64 {
    if self.enabled {
        self.virtual_time_us.load(Ordering::Relaxed)
    } else {
        0 // Return 0 when not in determinism mode
    }
}

/// Get current virtual tick count
pub fn get_virtual_ticks(&self) -> u64 {
    if self.enabled {
        self.virtual_ticks.load(Ordering::Relaxed)
    } else {
        0 // Return 0 when not in determinism mode
    }
}

/// Set virtual time to a specific value
pub fn set_virtual_time(&self, ms: u64) {
    if self.enabled {
        self.virtual_time_ms.store(ms, Ordering::Relaxed);
        self.virtual_time_us.store(ms * 1000, Ordering::Relaxed);
    }
}

/// Set virtual tick count to a specific value
pub fn set_virtual_ticks(&self, ticks: u64) {
    if self.enabled {
        self.virtual_ticks.store(ticks, Ordering::Relaxed);
    }
}
}

/// Global determinism configuration
static mut DETERMINISM_CONFIG: DeterminismConfig = DeterminismConfig::new();

/// Global virtual time counters for deterministic builds
static VIRTUAL_TIME_MS: AtomicU64 = AtomicU64::new(0);
static VIRTUAL_TIME_US: AtomicU64 = AtomicU64::new(0);
static VIRTUAL_TICKS: AtomicU64 = AtomicU64::new(0);

/// Deterministic random number generator
pub struct DeterministicRng {
    seed: u64,
    state: u64,
}

impl DeterministicRng {
    /// Create a new deterministic RNG with a seed
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            state: seed,
        }
    }
    
    /// Generate the next random number
    pub fn next(&mut self) -> u64 {
        // Simple linear congruential generator for determinism
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }
    
    /// Generate a random number in a range
    pub fn next_range(&mut self, min: u64, max: u64) -> u64 {
        let range = max - min;
        min + (self.next() % range)
    }
    
    /// Reset the RNG to its initial state
    pub fn reset(&mut self) {
        self.state = self.seed;
    }
    
    /// Get the current seed
    pub fn get_seed(&self) -> u64 {
        self.seed
    }
}

/// Get the global determinism configuration
pub fn get_config() -> &'static DeterminismConfig {
    unsafe { &DETERMINISM_CONFIG }
}

/// Get a mutable reference to the global determinism configuration
pub fn get_config_mut() -> &'static mut DeterminismConfig {
    unsafe { &mut DETERMINISM_CONFIG }
}

/// Enable determinism mode with a replay seed
pub fn enable_determinism(seed: u64) {
    let config = get_config_mut();
    config.enable(seed);
}

/// Disable determinism mode
pub fn disable_determinism() {
    let config = get_config_mut();
    config.disable();
}

/// Check if determinism mode is enabled
pub fn is_determinism_enabled() -> bool {
    get_config().is_enabled()
}

/// Get the current replay seed
pub fn get_replay_seed() -> u64 {
    get_config().get_replay_seed()
}

/// Advance virtual time by a specified amount
pub fn advance_virtual_time(ms: u64) {
    get_config().advance_time(ms);
}

/// Advance virtual ticks by a specified amount
pub fn advance_virtual_ticks(ticks: u64) {
    get_config().advance_ticks(ticks);
}

/// Get current virtual time in milliseconds
pub fn get_virtual_time_ms() -> u64 {
    get_config().get_virtual_time_ms()
}

/// Get current virtual time in microseconds
pub fn get_virtual_time_us() -> u64 {
    get_config().get_virtual_time_us()
}

/// Get current virtual tick count
pub fn get_virtual_ticks() -> u64 {
    get_config().get_virtual_ticks()
}

/// Set virtual time to a specific value
pub fn set_virtual_time(ms: u64) {
    get_config().set_virtual_time(ms);
}

/// Set virtual tick count to a specific value
pub fn set_virtual_tick_count(ticks: u64) {
    get_config().set_virtual_ticks(ticks);
}

/// Get global virtual time (deterministic mode) or real time (normal mode)
pub fn get_global_time_ms() -> u64 {
    if is_determinism_enabled() {
        VIRTUAL_TIME_MS.load(Ordering::Relaxed)
    } else {
        get_config().get_virtual_time_ms()
    }
}

/// Get global virtual time in microseconds (deterministic mode) or real time (normal mode)
pub fn get_global_time_us() -> u64 {
    if is_determinism_enabled() {
        VIRTUAL_TIME_US.load(Ordering::Relaxed)
    } else {
        get_config().get_virtual_time_us()
    }
}

/// Get global virtual tick count (deterministic mode) or real tick count (normal mode)
pub fn get_global_ticks() -> u64 {
    if is_determinism_enabled() {
        VIRTUAL_TICKS.load(Ordering::Relaxed)
    } else {
        get_config().get_virtual_ticks()
    }
}

/// Advance global virtual time (deterministic mode only)
pub fn advance_global_time(ms: u64) {
    if is_determinism_enabled() {
        let current_ms = VIRTUAL_TIME_MS.load(Ordering::Relaxed);
        VIRTUAL_TIME_MS.store(current_ms + ms, Ordering::Relaxed);
        
        let current_us = VIRTUAL_TIME_US.load(Ordering::Relaxed);
        VIRTUAL_TIME_US.store(current_us + (ms * 1000), Ordering::Relaxed);
    }
}

/// Advance global virtual ticks (deterministic mode only)
pub fn advance_global_ticks(ticks: u64) {
    if is_determinism_enabled() {
        let current = VIRTUAL_TICKS.load(Ordering::Relaxed);
        VIRTUAL_TICKS.store(current + ticks, Ordering::Relaxed);
    }
}

/// Get deterministic random number (deterministic mode) or real random number (normal mode)
pub fn get_deterministic_random() -> u64 {
    if is_determinism_enabled() {
        // Use a deterministic sequence based on the replay seed
        let seed = get_replay_seed();
        let mut rng = DeterministicRng::new(seed);
        
        // Advance based on current virtual time to ensure uniqueness
        let time_factor = get_global_time_ms() / 1000; // Every second
        for _ in 0..time_factor {
            rng.next();
        }
        
        rng.next()
    } else {
        // In normal mode, return a real random number
        // This would need to be implemented based on hardware RNG
        0 // Placeholder for real RNG
    }
}

/// Get deterministic random number in range (deterministic mode) or real random number (normal mode)
pub fn get_deterministic_random_range(min: u64, max: u64) -> u64 {
    if is_determinism_enabled() {
        let random_value = get_deterministic_random();
        let range = max - min;
        min + (random_value % range)
    } else {
        // In normal mode, return a real random number in range
        min // Placeholder for real RNG
    }
}

/// Create a deterministic RNG instance
pub fn create_deterministic_rng() -> DeterministicRng {
    DeterministicRng::new(get_replay_seed())
}

/// Print determinism system status
pub fn print_status() {
    kprintln!("=== DETERMINISM SYSTEM STATUS ===");
    kprintln!("Enabled: {}", is_determinism_enabled());
    kprintln!("Replay Seed: 0x{:016x}", get_replay_seed());
    kprintln!("Virtual Time: {}ms", get_global_time_ms());
    kprintln!("Virtual Ticks: {}", get_global_ticks());
    kprintln!("================================");
}

/// Get current virtual tick count
pub fn get_virtual_ticks() -> u64 {
    get_config().get_virtual_ticks()
}

/// Create a deterministic RNG instance
pub fn create_deterministic_rng() -> DeterministicRng {
    let seed = get_replay_seed();
    DeterministicRng::new(seed)
}

/// Initialize determinism system
pub fn init() {
    // Check for environment variable or compile-time feature
    #[cfg(feature = "determinism")]
    {
        enable_determinism(0x1234567890abcdef);
        crate::kprintln!("[DETERMINISM] Initialized with compile-time feature");
    }
    
    #[cfg(not(feature = "determinism"))]
    {
        // Check for environment variable (simulated)
        // In a real implementation, this would check actual environment variables
        if let Some(seed) = get_env_determinism_seed() {
            enable_determinism(seed);
            crate::kprintln!("[DETERMINISM] Initialized from environment variable");
        } else {
            crate::kprintln!("[DETERMINISM] Not enabled (use --features=determinism or set DETERMINISM_SEED)");
        }
    }
}

/// Get determinism seed from environment (simulated)
fn get_env_determinism_seed() -> Option<u64> {
    // In a real implementation, this would read from environment variables
    // For now, return None to simulate no environment variable set
    None
}

/// Print determinism status
pub fn print_status() {
    let config = get_config();
    
    crate::kprintln!("");
    crate::kprintln!("=== DETERMINISM STATUS ===");
    crate::kprintln!("Enabled: {}", config.is_enabled());
    
    if config.is_enabled() {
        crate::kprintln!("Replay Seed: 0x{:016x}", config.get_replay_seed());
        crate::kprintln!("Virtual Time: {}ms ({}μs)", 
            config.get_virtual_time_ms(), 
            config.get_virtual_time_us());
        crate::kprintln!("Virtual Ticks: {}", config.get_virtual_ticks());
    }
    
    crate::kprintln!("==========================");
    crate::kprintln!("");
}



