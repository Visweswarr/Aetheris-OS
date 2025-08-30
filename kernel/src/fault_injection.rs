//! Fault Injection Module
//! 
//! This module provides fault injection capabilities for testing system
//! resilience under various failure conditions. It allows controlled
//! injection of faults to verify graceful degradation and proper
//! error handling.

use core::sync::atomic::{AtomicU32, Ordering};
use alloc::collections::BTreeMap;
use alloc::string::String;

/// Fault injection configuration
#[derive(Debug, Clone)]
pub struct FaultInjectionConfig {
    /// Whether fault injection is enabled
    pub enabled: bool,
    
    /// Fault type identifier
    pub fault_type: u32,
    
    /// Frequency of fault injection (every Nth operation)
    pub frequency: u32,
    
    /// Current operation counter
    pub operation_counter: u32,
    
    /// Total faults injected
    pub total_faults_injected: u32,
    
    /// Description of the fault
    pub description: String,
}

impl FaultInjectionConfig {
    /// Create a new fault injection configuration
    pub fn new(fault_type: u32, frequency: u32, description: &str) -> Self {
        Self {
            enabled: false,
            fault_type,
            frequency,
            operation_counter: 0,
            total_faults_injected: 0,
            description: description.to_string(),
        }
    }
    
    /// Check if a fault should be injected for the current operation
    pub fn should_inject_fault(&mut self) -> bool {
        if !self.enabled || self.frequency == 0 {
            return false;
        }
        
        self.operation_counter += 1;
        
        if self.operation_counter % self.frequency == 0 {
            self.total_faults_injected += 1;
            true
        } else {
            false
        }
    }
    
    /// Reset the operation counter
    pub fn reset_counter(&mut self) {
        self.operation_counter = 0;
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> FaultInjectionStats {
        FaultInjectionStats {
            fault_type: self.fault_type,
            enabled: self.enabled,
            frequency: self.frequency,
            operation_counter: self.operation_counter,
            total_faults_injected: self.total_faults_injected,
            description: self.description.clone(),
        }
    }
}

/// Fault injection statistics
#[derive(Debug, Clone)]
pub struct FaultInjectionStats {
    /// Fault type identifier
    pub fault_type: u32,
    
    /// Whether fault injection is enabled
    pub enabled: bool,
    
    /// Frequency of fault injection
    pub frequency: u32,
    
    /// Current operation counter
    pub operation_counter: u32,
    
    /// Total faults injected
    pub total_faults_injected: u32,
    
    /// Description of the fault
    pub description: String,
}

/// Global fault injection manager
pub struct FaultInjectionManager {
    /// Fault injection configurations by type
    configs: BTreeMap<u32, FaultInjectionConfig>,
    
    /// Global fault injection enabled flag
    global_enabled: AtomicU32,
}

impl FaultInjectionManager {
    /// Create a new fault injection manager
    pub fn new() -> Self {
        let mut manager = Self {
            configs: BTreeMap::new(),
            global_enabled: AtomicU32::new(1), // Enabled by default
        };
        
        // Initialize default fault types
        manager.register_fault_type(1, "Inbox overflow fault injection");
        
        manager
    }
    
    /// Register a new fault type
    pub fn register_fault_type(&mut self, fault_type: u32, description: &str) {
        let config = FaultInjectionConfig::new(fault_type, 0, description);
        self.configs.insert(fault_type, config);
    }
    
    /// Enable fault injection for a specific fault type
    pub fn enable_fault_injection(&mut self, fault_type: u32, frequency: u32) -> Result<(), &'static str> {
        if let Some(config) = self.configs.get_mut(&fault_type) {
            config.enabled = true;
            config.frequency = frequency;
            config.reset_counter();
            Ok(())
        } else {
            Err("Unknown fault type")
        }
    }
    
    /// Disable fault injection for a specific fault type
    pub fn disable_fault_injection(&mut self, fault_type: u32) -> Result<(), &'static str> {
        if let Some(config) = self.configs.get_mut(&fault_type) {
            config.enabled = false;
            config.frequency = 0;
            Ok(())
        } else {
            Err("Unknown fault type")
        }
    }
    
    /// Check if a fault should be injected for a specific fault type
    pub fn should_inject_fault(&mut self, fault_type: u32) -> bool {
        if let Some(config) = self.configs.get_mut(&fault_type) {
            config.should_inject_fault()
        } else {
            false
        }
    }
    
    /// Get fault injection statistics for a specific fault type
    pub fn get_fault_stats(&self, fault_type: u32) -> Option<FaultInjectionStats> {
        self.configs.get(&fault_type).map(|config| config.get_stats())
    }
    
    /// Get all fault injection statistics
    pub fn get_all_fault_stats(&self) -> alloc::vec::Vec<FaultInjectionStats> {
        self.configs.values().map(|config| config.get_stats()).collect()
    }
    
    /// Check if global fault injection is enabled
    pub fn is_globally_enabled(&self) -> bool {
        self.global_enabled.load(Ordering::Relaxed) != 0
    }
    
    /// Enable global fault injection
    pub fn enable_global(&self) {
        self.global_enabled.store(1, Ordering::Relaxed);
    }
    
    /// Disable global fault injection
    pub fn disable_global(&self) {
        self.global_enabled.store(0, Ordering::Relaxed);
    }
    
    /// Reset all fault injection counters
    pub fn reset_all_counters(&mut self) {
        for config in self.configs.values_mut() {
            config.reset_counter();
        }
    }
}

/// Global fault injection manager instance
static mut FAULT_INJECTION_MANAGER: Option<FaultInjectionManager> = None;

/// Initialize the fault injection system
pub fn init() {
    unsafe {
        FAULT_INJECTION_MANAGER = Some(FaultInjectionManager::new());
    }
    
    crate::log::klog!(crate::log::Level::INFO, [crate::log::tags::FAULT_INJECTION], 
        "Fault injection system initialized");
}

/// Get a mutable reference to the global fault injection manager
fn get_manager_mut() -> &'static mut FaultInjectionManager {
    unsafe {
        FAULT_INJECTION_MANAGER.as_mut().expect("Fault injection manager not initialized")
    }
}

/// Get a reference to the global fault injection manager
fn get_manager() -> &'static FaultInjectionManager {
    unsafe {
        FAULT_INJECTION_MANAGER.as_ref().expect("Fault injection manager not initialized")
    }
}

/// Enable inbox overflow fault injection
pub fn enable_inbox_overflow_fault(frequency: u32) {
    let manager = get_manager_mut();
    if let Err(e) = manager.enable_fault_injection(1, frequency) {
        crate::log::klog!(crate::log::Level::ERROR, [crate::log::tags::FAULT_INJECTION], 
            "Failed to enable inbox overflow fault injection: {}", e);
    } else {
        crate::log::klog!(crate::log::Level::INFO, [crate::log::tags::FAULT_INJECTION], 
            "Inbox overflow fault injection enabled every {} pushes", frequency);
    }
}

/// Disable inbox overflow fault injection
pub fn disable_inbox_overflow_fault() {
    let manager = get_manager_mut();
    if let Err(e) = manager.disable_fault_injection(1) {
        crate::log::klog!(crate::log::Level::ERROR, [crate::log::tags::FAULT_INJECTION], 
            "Failed to disable inbox overflow fault injection: {}", e);
    } else {
        crate::log::klog!(crate::log::Level::INFO, [crate::log::tags::FAULT_INJECTION], 
            "Inbox overflow fault injection disabled");
    }
}

/// Check if inbox overflow fault should be injected
pub fn should_inject_inbox_overflow_fault() -> bool {
    if !get_manager().is_globally_enabled() {
        return false;
    }
    
    let manager = get_manager_mut();
    manager.should_inject_fault(1)
}

/// Get fault injection statistics
pub fn get_fault_injection_stats() -> alloc::vec::Vec<FaultInjectionStats> {
    get_manager().get_all_fault_stats()
}

/// Get inbox overflow fault statistics
pub fn get_inbox_overflow_fault_stats() -> Option<FaultInjectionStats> {
    get_manager().get_fault_stats(1)
}

/// Reset all fault injection counters
pub fn reset_fault_injection_counters() {
    let manager = get_manager_mut();
    manager.reset_all_counters();
    
    crate::log::klog!(crate::log::Level::INFO, [crate::log::tags::FAULT_INJECTION], 
        "All fault injection counters reset");
}

/// Test function for fault injection functionality
pub fn test_fault_injection() -> Result<(), &'static str> {
    crate::log::kprintln!("Testing fault injection functionality...");
    
    // Test enabling fault injection
    enable_inbox_overflow_fault(5);
    
    // Test that fault injection is enabled
    let stats = get_inbox_overflow_fault_stats().ok_or("Failed to get fault stats")?;
    if !stats.enabled || stats.frequency != 5 {
        return Err("Fault injection not properly enabled");
    }
    
    // Test fault injection logic
    let mut fault_count = 0;
    for i in 0..20 {
        if should_inject_inbox_overflow_fault() {
            fault_count += 1;
        }
    }
    
    // Should inject fault every 5th operation (at 5, 10, 15, 20)
    if fault_count != 4 {
        return Err("Fault injection frequency not working correctly");
    }
    
    // Test disabling fault injection
    disable_inbox_overflow_fault();
    let stats = get_inbox_overflow_fault_stats().ok_or("Failed to get fault stats")?;
    if stats.enabled {
        return Err("Fault injection not properly disabled");
    }
    
    crate::log::kprintln!("✅ Fault injection tests passed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fault_injection_config() {
        let mut config = FaultInjectionConfig::new(1, 3, "Test fault");
        
        // Test initial state
        assert!(!config.enabled);
        assert_eq!(config.frequency, 3);
        assert_eq!(config.operation_counter, 0);
        assert_eq!(config.total_faults_injected, 0);
        
        // Enable and test fault injection
        config.enabled = true;
        
        // Test fault injection every 3rd operation
        let mut faults = 0;
        for _ in 0..10 {
            if config.should_inject_fault() {
                faults += 1;
            }
        }
        
        assert_eq!(faults, 3); // Should inject at operations 3, 6, 9
        assert_eq!(config.operation_counter, 10);
        assert_eq!(config.total_faults_injected, 3);
    }
    
    #[test]
    fn test_fault_injection_manager() {
        let mut manager = FaultInjectionManager::new();
        
        // Test fault type registration
        assert!(manager.configs.contains_key(&1));
        
        // Test enabling fault injection
        assert!(manager.enable_fault_injection(1, 2).is_ok());
        
        // Test fault injection logic
        let mut faults = 0;
        for _ in 0..10 {
            if manager.should_inject_fault(1) {
                faults += 1;
            }
        }
        
        assert_eq!(faults, 5); // Should inject every 2nd operation
        
        // Test disabling fault injection
        assert!(manager.disable_fault_injection(1).is_ok());
        assert!(!manager.should_inject_fault(1));
    }
}


