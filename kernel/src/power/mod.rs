//! Power Controller Module
//!
//! Manages CPU power states, frequency scaling (DVFS), and thermal monitoring.
//! 
//! Requirement: 8.2 - Atomic voltage/frequency change
//! Requirement: 8.4 - Thermal management

use spin::Mutex;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::kprintln;

/// CPU Power State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerState {
    pub frequency_khz: u32,
    pub voltage_mv: u32,
}

impl Default for PowerState {
    fn default() -> Self {
        Self {
            frequency_khz: 3_000_000, // 3.0 GHz
            voltage_mv: 1100,         // 1.1 V
        }
    }
}

/// Thermal reading
#[derive(Debug, Clone, Copy)]
pub struct ThermalReading {
    pub temperature_c: i32,
    pub timestamp: u64,
}

/// Global Power Manager
pub struct PowerManager {
    /// Per-CPU power states (simulating 256 CPUs max)
    cpu_states: [PowerState; 256],
    /// Per-CPU thermal readings
    thermal_readings: [ThermalReading; 256],
}

impl PowerManager {
    pub const fn new() -> Self {
        Self {
            cpu_states: [PowerState { frequency_khz: 3_000_000, voltage_mv: 1100 }; 256],
            thermal_readings: [ThermalReading { temperature_c: 40, timestamp: 0 }; 256],
        }
    }
    
    /// Set CPU frequency and voltage atomically
    /// 
    /// In a real implementation, this would write to MSRs or call ACPI tables.
    pub fn set_cpu_state(&mut self, cpu_id: usize, freq_khz: u32, voltage_mv: u32) -> Result<(), &'static str> {
        if cpu_id >= 256 {
            return Err("Invalid CPU ID");
        }
        
        // Atomic simulation: In real OS, use hardware lock or atomic sequence
        // Here we are inside a Mutex (from GLOBAL_POWER_MANAGER), so it is atomic for software observers.
        
        self.cpu_states[cpu_id] = PowerState {
            frequency_khz: freq_khz,
            voltage_mv,
        };
        
        // Simulate hardware effect
        kprintln!("[POWER] CPU {} set to {} KHz @ {} mV", cpu_id, freq_khz, voltage_mv);
        
        Ok(())
    }
    
    /// Get current CPU state
    pub fn get_cpu_state(&self, cpu_id: usize) -> Option<PowerState> {
        if cpu_id >= 256 { return None; }
        Some(self.cpu_states[cpu_id])
    }
    
    /// Update thermal reading (called by driver/interrupt)
    pub fn update_thermal(&mut self, cpu_id: usize, temp_c: i32) {
        if cpu_id < 256 {
            self.thermal_readings[cpu_id] = ThermalReading {
                temperature_c: temp_c,
                timestamp: crate::log::get_current_time_ms(),
            };
            
            // Thermal logic
            if temp_c > 90 {
                // Throttle immediately
                let _ = self.set_cpu_state(cpu_id, 800_000, 900);
                kprintln!("[POWER] CRITICAL THERMAL: CPU {} throttled due to {}C", cpu_id, temp_c);
            }
        }
    }
    
    /// Get current temperature
    pub fn get_temperature(&self, cpu_id: usize) -> i32 {
        if cpu_id < 256 {
             self.thermal_readings[cpu_id].temperature_c
        } else {
            0
        }
    }
}

static GLOBAL_POWER_MANAGER: Mutex<PowerManager> = Mutex::new(PowerManager::new());

/// Public API: Set CPU Performance State
pub fn set_performance_state(cpu_id: usize, freq_khz: u32, voltage_mv: u32) -> Result<(), &'static str> {
    let mut pm = GLOBAL_POWER_MANAGER.lock();
    pm.set_cpu_state(cpu_id, freq_khz, voltage_mv)
}

/// Public API: Report temperature
pub fn report_temperature(cpu_id: usize, temp_c: i32) {
    let mut pm = GLOBAL_POWER_MANAGER.lock();
    pm.update_thermal(cpu_id, temp_c);
}

/// Public API: Get temperature
pub fn get_temperature(cpu_id: usize) -> i32 {
    let pm = GLOBAL_POWER_MANAGER.lock();
    pm.get_temperature(cpu_id)
}
