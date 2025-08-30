//! Fault Injection System
//! 
//! This module provides controlled fault injection capabilities for testing
//! system resilience and error handling. It can force various fault conditions
//! including inbox overflow, allocation failures, and timer jitter.

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use core::sync::atomic::AtomicU64;

/// Fault injection configuration
#[derive(Debug, Clone)]
pub struct FaultInjectionConfig {
    /// Enable inbox overflow injection
    pub inbox_overflow_enabled: bool,
    /// Force inbox overflow every N pushes
    pub inbox_overflow_interval: u32,
    
    /// Enable allocation failure injection
    pub alloc_failure_enabled: bool,
    /// Force allocation failure every Nth kmalloc
    pub alloc_failure_interval: u32,
    
    /// Enable timer jitter injection
    pub timer_jitter_enabled: bool,
    /// Timer jitter percentage (+/- N%)
    pub timer_jitter_percentage: u32,
    
    /// Enable all fault injection
    pub global_enabled: bool,
}

impl Default for FaultInjectionConfig {
    fn default() -> Self {
        Self {
            inbox_overflow_enabled: false,
            inbox_overflow_interval: 100,
            alloc_failure_enabled: false,
            alloc_failure_interval: 100,
            timer_jitter_enabled: false,
            timer_jitter_percentage: 10,
            global_enabled: false,
        }
    }
}

/// Fault injection state
pub struct FaultInjectionState {
    /// Inbox push counter
    inbox_push_count: AtomicU32,
    /// Allocation counter
    alloc_count: AtomicU32,
    /// Timer tick counter
    timer_tick_count: AtomicU64,
    /// Base timer value
    base_timer_value: AtomicU64,
}

impl FaultInjectionState {
    /// Create new fault injection state
    pub const fn new() -> Self {
        Self {
            inbox_push_count: AtomicU32::new(0),
            alloc_count: AtomicU32::new(0),
            timer_tick_count: AtomicU64::new(0),
            base_timer_value: AtomicU64::new(0),
        }
    }
    
    /// Increment inbox push counter and check if overflow should be forced
    pub fn should_force_inbox_overflow(&self, config: &FaultInjectionConfig) -> bool {
        if !config.global_enabled || !config.inbox_overflow_enabled {
            return false;
        }
        
        let count = self.inbox_push_count.fetch_add(1, Ordering::Relaxed);
        count % config.inbox_overflow_interval == 0
    }
    
    /// Increment allocation counter and check if failure should be forced
    pub fn should_force_alloc_failure(&self, config: &FaultInjectionConfig) -> bool {
        if !config.global_enabled || !config.alloc_failure_enabled {
            return false;
        }
        
        let count = self.alloc_count.fetch_add(1, Ordering::Relaxed);
        count % config.alloc_failure_interval == 0
    }
    
    /// Get timer jitter value
    pub fn get_timer_jitter(&self, config: &FaultInjectionConfig) -> i64 {
        if !config.global_enabled || !config.timer_jitter_enabled {
            return 0;
        }
        
        let tick = self.timer_tick_count.fetch_add(1, Ordering::Relaxed);
        
        // Use tick count to generate pseudo-random jitter
        let jitter_base = (tick % 100) as i64;
        let jitter_range = (config.timer_jitter_percentage as i64 * 2) + 1;
        let jitter = (jitter_base % jitter_range) - (config.timer_jitter_percentage as i64);
        
        jitter
    }
    
    /// Set base timer value
    pub fn set_base_timer(&self, value: u64) {
        self.base_timer_value.store(value, Ordering::Relaxed);
    }
    
    /// Get base timer value
    pub fn get_base_timer(&self) -> u64 {
        self.base_timer_value.load(Ordering::Relaxed)
    }
    
    /// Reset all counters
    pub fn reset(&self) {
        self.inbox_push_count.store(0, Ordering::Relaxed);
        self.alloc_count.store(0, Ordering::Relaxed);
        self.timer_tick_count.store(0, Ordering::Relaxed);
    }
}

/// Global fault injection configuration
static mut FAULT_CONFIG: FaultInjectionConfig = FaultInjectionConfig::default();

/// Global fault injection state
static mut FAULT_STATE: FaultInjectionState = FaultInjectionState::new();

/// Get current fault injection configuration
pub fn get_fault_config() -> &'static FaultInjectionConfig {
    unsafe { &FAULT_CONFIG }
}

/// Get current fault injection state
pub fn get_fault_state() -> &'static FaultInjectionState {
    unsafe { &FAULT_STATE }
}

/// Update fault injection configuration
pub fn update_fault_config(new_config: FaultInjectionConfig) {
    unsafe {
        FAULT_CONFIG = new_config;
        
        // Reset state when configuration changes
        FAULT_STATE.reset();
        
        crate::kprintln!("[FAULT_INJECTION] Configuration updated:");
        crate::kprintln!("  Global enabled: {}", new_config.global_enabled);
        crate::kprintln!("  Inbox overflow: {} (every {} pushes)", 
                        new_config.inbox_overflow_enabled, new_config.inbox_overflow_interval);
        crate::kprintln!("  Alloc failure: {} (every {} allocations)", 
                        new_config.alloc_failure_enabled, new_config.alloc_failure_interval);
        crate::kprintln!("  Timer jitter: {} (+/- {}%)", 
                        new_config.timer_jitter_enabled, new_config.timer_jitter_percentage);
    }
}

/// Check if inbox overflow should be forced
pub fn should_force_inbox_overflow() -> bool {
    let config = get_fault_config();
    let state = get_fault_state();
    state.should_force_inbox_overflow(config)
}

/// Check if allocation failure should be forced
pub fn should_force_alloc_failure() -> bool {
    let config = get_fault_config();
    let state = get_fault_state();
    state.should_force_alloc_failure(config)
}

/// Get timer jitter value
pub fn get_timer_jitter() -> i64 {
    let config = get_fault_config();
    let state = get_fault_state();
    state.get_timer_jitter(config)
}

/// Set base timer value
pub fn set_base_timer(value: u64) {
    let state = get_fault_state();
    state.set_base_timer(value);
}

/// Get base timer value
pub fn get_base_timer() -> u64 {
    let state = get_fault_state();
    state.get_base_timer()
}

/// Reset fault injection state
pub fn reset_fault_state() {
    let state = get_fault_state();
    state.reset();
    crate::kprintln!("[FAULT_INJECTION] State reset");
}

/// Initialize fault injection system
pub fn init() {
    crate::kprintln!("[FAULT_INJECTION] Initializing fault injection system");
    
    // Set default configuration
    let config = FaultInjectionConfig::default();
    update_fault_config(config);
    
    crate::kprintln!("[FAULT_INJECTION] Fault injection system initialized");
}

/// Test fault injection system
pub fn test_fault_injection() -> Result<(), &'static str> {
    crate::kprintln!("[FAULT_INJECTION] Testing fault injection system...");
    
    // Test configuration update
    let test_config = FaultInjectionConfig {
        inbox_overflow_enabled: true,
        inbox_overflow_interval: 5,
        alloc_failure_enabled: true,
        alloc_failure_interval: 3,
        timer_jitter_enabled: true,
        timer_jitter_percentage: 5,
        global_enabled: true,
    };
    
    update_fault_config(test_config);
    
    // Test inbox overflow injection
    let mut overflow_count = 0;
    for i in 0..20 {
        if should_force_inbox_overflow() {
            overflow_count += 1;
            crate::kprintln!("[FAULT_INJECTION] Forced inbox overflow at push {}", i);
        }
    }
    
    if overflow_count != 4 { // Should be forced every 5th push
        return Err("Inbox overflow injection test failed");
    }
    
    // Test allocation failure injection
    let mut failure_count = 0;
    for i in 0..15 {
        if should_force_alloc_failure() {
            failure_count += 1;
            crate::kprintln!("[FAULT_INJECTION] Forced alloc failure at allocation {}", i);
        }
    }
    
    if failure_count != 5 { // Should be forced every 3rd allocation
        return Err("Allocation failure injection test failed");
    }
    
    // Test timer jitter
    let mut jitter_values = Vec::new();
    for _ in 0..100 {
        let jitter = get_timer_jitter();
        jitter_values.push(jitter);
    }
    
    let min_jitter = jitter_values.iter().min().unwrap();
    let max_jitter = jitter_values.iter().max().unwrap();
    
    if *min_jitter < -5 || *max_jitter > 5 {
        return Err("Timer jitter injection test failed");
    }
    
    // Reset to default configuration
    let default_config = FaultInjectionConfig::default();
    update_fault_config(default_config);
    
    crate::kprintln!("[FAULT_INJECTION] All tests passed successfully");
    Ok(())
}
