/// High Precision Event Timer (HPET) Implementation
/// 
/// This module provides HPET support as a fallback when Local APIC timer is not available.
/// HPET offers high precision timing with low jitter for robust timer functionality.

use core::arch::x86_64::*;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use core::ptr;
use alloc::vec::Vec;

use crate::hal::HalError;
use crate::time::{Duration, Instant};
use crate::sync::Mutex;

/// HPET timer configuration
#[derive(Debug, Clone)]
pub struct HpetTimerConfig {
    pub vector: u8,
    pub frequency_hz: u64,
    pub mode: HpetTimerMode,
}

/// HPET timer modes
#[derive(Debug, Clone, PartialEq)]
pub enum HpetTimerMode {
    OneShot,
    Periodic,
}

/// HPET timer state
pub struct HpetTimer {
    config: HpetTimerConfig,
    calibrated: bool,
    ns_per_tick: AtomicU64,
    tick_count: AtomicU64,
    overrun_count: AtomicU32,
    jitter_histogram: Mutex<[u32; 64]>, // Fixed-size histogram for jitter tracking
    base_address: *mut u64,
}

impl HpetTimer {
    /// Create a new HPET timer
    pub fn new(config: HpetTimerConfig) -> Self {
        Self {
            config,
            calibrated: false,
            ns_per_tick: AtomicU64::new(0),
            tick_count: AtomicU64::new(0),
            overrun_count: AtomicU32::new(0),
            jitter_histogram: Mutex::new([0; 64]),
            base_address: ptr::null_mut(),
        }
    }

    /// Initialize the HPET timer
    pub fn init(&mut self) -> Result<(), HalError> {
        // Check if HPET is present
        if !self::is_hpet_present() {
            return Err(HalError::DeviceNotFound("HPET not present"));
        }

        // Map HPET MMIO
        let base_address = self::map_hpet_mmio()?;
        self.base_address = base_address;

        // Verify HPET capabilities
        self::verify_hpet_capabilities(base_address)?;

        // Configure timer
        self::configure_timer(base_address, &self.config)?;

        // Calibrate against TSC
        self.calibrate()?;

        Ok(())
    }

    /// Calibrate timer against TSC
    pub fn calibrate(&mut self) -> Result<(), HalError> {
        const CALIBRATION_WINDOW_MS: u64 = 10;
        const MIN_SAMPLES: usize = 100;

        let mut samples = Vec::new();
        let start_time = Instant::now();
        let target_duration = Duration::from_millis(CALIBRATION_WINDOW_MS);

        // Collect TSC samples over calibration window
        while start_time.elapsed() < target_duration && samples.len() < MIN_SAMPLES {
            let tsc1 = self::read_tsc();
            
            // Wait for next timer tick
            let tick_start = self.tick_count.load(Ordering::Relaxed);
            while self.tick_count.load(Ordering::Relaxed) == tick_start {
                core::hint::spin_loop();
            }
            
            let tsc2 = self::read_tsc();
            let tsc_delta = tsc2.wrapping_sub(tsc1);
            
            if tsc_delta > 0 {
                samples.push(tsc_delta);
            }
        }

        if samples.len() < MIN_SAMPLES {
            return Err(HalError::CalibrationFailed("Insufficient samples"));
        }

        // Calculate median TSC delta per tick
        samples.sort_unstable();
        let median_tsc_per_tick = samples[samples.len() / 2];

        // Convert to nanoseconds per tick
        let tsc_freq = self::get_tsc_frequency();
        let ns_per_tick = (median_tsc_per_tick * 1_000_000_000) / tsc_freq;

        self.ns_per_tick.store(ns_per_tick, Ordering::Relaxed);
        self.calibrated = true;

        Ok(())
    }

    /// Get calibrated nanoseconds per tick
    pub fn get_ns_per_tick(&self) -> u64 {
        self.ns_per_tick.load(Ordering::Relaxed)
    }

    /// Check if timer is calibrated
    pub fn is_calibrated(&self) -> bool {
        self.calibrated
    }

    /// Get current tick count
    pub fn get_tick_count(&self) -> u64 {
        self.tick_count.load(Ordering::Relaxed)
    }

    /// Get overrun count
    pub fn get_overrun_count(&self) -> u32 {
        self.overrun_count.load(Ordering::Relaxed)
    }

    /// Record jitter measurement
    pub fn record_jitter(&self, jitter_us: u32) {
        // Map jitter to histogram bin (0-63, each bin represents ~4us)
        let bin = (jitter_us / 4).min(63) as usize;
        
        if let Ok(mut histogram) = self.jitter_histogram.lock() {
            histogram[bin] = histogram[bin].saturating_add(1);
        }
    }

    /// Get jitter statistics
    pub fn get_jitter_stats(&self) -> JitterStats {
        if let Ok(histogram) = self.jitter_histogram.lock() {
            let mut total_samples = 0;
            let mut cumulative_jitter = 0u64;
            
            for (bin, &count) in histogram.iter().enumerate() {
                total_samples += count;
                let jitter_us = (bin as u32 * 4) + 2; // Center of bin
                cumulative_jitter += (jitter_us as u64) * (count as u64);
            }

            if total_samples > 0 {
                let mean_jitter = cumulative_jitter / total_samples;
                
                // Calculate p95 (simplified - find 95th percentile)
                let p95_index = (total_samples * 95) / 100;
                let mut current_count = 0;
                let mut p95_jitter = 0u32;
                
                for (bin, &count) in histogram.iter().enumerate() {
                    current_count += count;
                    if current_count >= p95_index {
                        p95_jitter = (bin as u32 * 4) + 2;
                        break;
                    }
                }

                JitterStats {
                    mean_us: mean_jitter as u32,
                    p95_us: p95_jitter,
                    total_samples,
                }
            } else {
                JitterStats {
                    mean_us: 0,
                    p95_us: 0,
                    total_samples: 0,
                }
            }
        } else {
            JitterStats {
                mean_us: 0,
                p95_us: 0,
                total_samples: 0,
            }
        }
    }

    /// Handle timer interrupt
    pub fn handle_interrupt(&self) {
        // Increment tick count
        self.tick_count.fetch_add(1, Ordering::Relaxed);

        // Check for overruns (if we're falling behind)
        let current_tick = self.tick_count.load(Ordering::Relaxed);
        let expected_tick = (Instant::now().as_millis() / 1000) as u64; // Assuming 1000Hz
        
        if current_tick < expected_tick.saturating_sub(2) {
            self.overrun_count.fetch_add(1, Ordering::Relaxed);
        }

        // Acknowledge interrupt
        self::acknowledge_interrupt(self.base_address);
    }

    /// Set timer mode
    pub fn set_mode(&mut self, mode: HpetTimerMode) -> Result<(), HalError> {
        self.config.mode = mode;
        
        match mode {
            HpetTimerMode::OneShot => {
                self::set_timer_mode_oneshot(self.base_address)?;
            }
            HpetTimerMode::Periodic => {
                self::set_timer_mode_periodic(self.base_address)?;
            }
        }

        Ok(())
    }

    /// Set timer frequency (in Hz)
    pub fn set_frequency(&mut self, freq_hz: u64) -> Result<(), HalError> {
        if freq_hz == 0 || freq_hz > 10000 {
            return Err(HalError::InvalidParameter("Invalid frequency"));
        }

        self.config.frequency_hz = freq_hz;
        
        // Calculate comparator value
        let main_counter_freq = self::get_main_counter_frequency(self.base_address);
        let comparator_value = main_counter_freq / freq_hz;
        
        self::set_timer_comparator(self.base_address, comparator_value)?;

        Ok(())
    }
}

/// Jitter statistics
#[derive(Debug, Clone)]
pub struct JitterStats {
    pub mean_us: u32,
    pub p95_us: u32,
    pub total_samples: u32,
}

// HPET register offsets
const HPET_CAPABILITIES: u64 = 0x000;
const HPET_CONFIG: u64 = 0x010;
const HPET_ISR: u64 = 0x020;
const HPET_MAIN_COUNTER: u64 = 0x0F0;
const HPET_TIMER0_CONFIG: u64 = 0x100;
const HPET_TIMER0_COMPARATOR: u64 = 0x108;
const HPET_TIMER0_FSB: u64 = 0x110;

// Low-level HPET operations
impl HpetTimer {
    /// Check if HPET is present
    fn is_hpet_present() -> bool {
        // Check ACPI tables for HPET
        // For now, assume HPET is available in QEMU
        true
    }

    /// Map HPET MMIO region
    fn map_hpet_mmio() -> Result<*mut u64, HalError> {
        // HPET is typically mapped at 0xFED00000
        const HPET_BASE_ADDRESS: u64 = 0xFED00000;
        const HPET_SIZE: usize = 0x1000; // 4KB

        // In a real implementation, this would use proper memory mapping
        // For now, we'll use the direct address (QEMU-specific)
        let base_address = HPET_BASE_ADDRESS as *mut u64;
        
        if base_address.is_null() {
            return Err(HalError::DeviceNotFound("Failed to map HPET MMIO"));
        }

        Ok(base_address)
    }

    /// Verify HPET capabilities
    fn verify_hpet_capabilities(base_address: *mut u64) -> Result<(), HalError> {
        let capabilities = unsafe { ptr::read_volatile(base_address.add(HPET_CAPABILITIES / 8)) };
        
        // Check if HPET is supported
        if (capabilities & 0x8000000000000000) == 0 {
            return Err(HalError::DeviceNotFound("HPET not supported"));
        }

        // Check counter width
        let counter_width = ((capabilities >> 32) & 0xFF) as u32;
        if counter_width < 32 {
            return Err(HalError::DeviceNotFound("HPET counter width too small"));
        }

        // Check number of timers
        let num_timers = ((capabilities >> 8) & 0x1F) as u32;
        if num_timers == 0 {
            return Err(HalError::DeviceNotFound("No HPET timers available"));
        }

        Ok(())
    }

    /// Configure HPET timer
    fn configure_timer(base_address: *mut u64, config: &HpetTimerConfig) -> Result<(), HalError> {
        // Enable HPET
        let mut hpet_config = unsafe { ptr::read_volatile(base_address.add(HPET_CONFIG / 8)) };
        hpet_config |= 1; // Enable HPET
        unsafe {
            ptr::write_volatile(base_address.add(HPET_CONFIG / 8), hpet_config);
        }

        // Configure Timer 0
        let timer_config = unsafe { ptr::read_volatile(base_address.add(HPET_TIMER0_CONFIG / 8)) };
        let mut new_config = timer_config;
        
        // Set mode
        match config.mode {
            HpetTimerMode::OneShot => {
                new_config &= !(1 << 3); // Clear periodic bit
            }
            HpetTimerMode::Periodic => {
                new_config |= 1 << 3; // Set periodic bit
            }
        }

        // Set interrupt routing
        new_config &= !(0xFF << 9); // Clear vector field
        new_config |= (config.vector as u64) << 9; // Set vector

        // Enable timer
        new_config |= 1 << 2; // Enable timer

        // Write configuration
        unsafe {
            ptr::write_volatile(base_address.add(HPET_TIMER0_CONFIG / 8), new_config);
        }

        // Set initial comparator value
        let main_counter_freq = Self::get_main_counter_frequency(base_address);
        let comparator_value = main_counter_freq / config.frequency_hz;
        Self::set_timer_comparator(base_address, comparator_value)?;

        Ok(())
    }

    /// Set timer to one-shot mode
    fn set_timer_mode_oneshot(base_address: *mut u64) -> Result<(), HalError> {
        let timer_config = unsafe { ptr::read_volatile(base_address.add(HPET_TIMER0_CONFIG / 8)) };
        let new_config = timer_config & !(1 << 3); // Clear periodic bit
        
        unsafe {
            ptr::write_volatile(base_address.add(HPET_TIMER0_CONFIG / 8), new_config);
        }

        Ok(())
    }

    /// Set timer to periodic mode
    fn set_timer_mode_periodic(base_address: *mut u64) -> Result<(), HalError> {
        let timer_config = unsafe { ptr::read_volatile(base_address.add(HPET_TIMER0_CONFIG / 8)) };
        let new_config = timer_config | (1 << 3); // Set periodic bit
        
        unsafe {
            ptr::write_volatile(base_address.add(HPET_TIMER0_CONFIG / 8), new_config);
        }

        Ok(())
    }

    /// Set timer comparator value
    fn set_timer_comparator(base_address: *mut u64, value: u64) -> Result<(), HalError> {
        unsafe {
            ptr::write_volatile(base_address.add(HPET_TIMER0_COMPARATOR / 8), value);
        }

        Ok(())
    }

    /// Get main counter frequency
    fn get_main_counter_frequency(base_address: *mut u64) -> u64 {
        let capabilities = unsafe { ptr::read_volatile(base_address.add(HPET_CAPABILITIES / 8)) };
        let period_fs = capabilities & 0xFFFFFFFF;
        
        // Convert femtoseconds to frequency
        1_000_000_000_000_000 / period_fs
    }

    /// Get TSC frequency
    fn get_tsc_frequency() -> u64 {
        // Default TSC frequency (can be calibrated more precisely)
        2_400_000_000 // 2.4 GHz
    }

    /// Read TSC with proper fencing
    fn read_tsc() -> u64 {
        unsafe {
            // Serialize instruction execution
            _mm_lfence();
            let tsc = __rdtsc();
            _mm_lfence();
            tsc
        }
    }

    /// Acknowledge interrupt
    fn acknowledge_interrupt(base_address: *mut u64) {
        // Read ISR to acknowledge
        let _isr = unsafe { ptr::read_volatile(base_address.add(HPET_ISR / 8)) };
    }
}

/// Default HPET timer configuration
impl Default for HpetTimerConfig {
    fn default() -> Self {
        Self {
            vector: 33, // IRQ 1
            frequency_hz: 1000, // 1000Hz
            mode: HpetTimerMode::Periodic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hpet_timer_creation() {
        let config = HpetTimerConfig::default();
        let timer = HpetTimer::new(config);
        
        assert_eq!(timer.get_tick_count(), 0);
        assert_eq!(timer.get_overrun_count(), 0);
        assert!(!timer.is_calibrated());
    }

    #[test]
    fn test_jitter_recording() {
        let config = HpetTimerConfig::default();
        let timer = HpetTimer::new(config);
        
        // Record some jitter values
        timer.record_jitter(15);  // 15us
        timer.record_jitter(55);  // 55us
        timer.record_jitter(105); // 105us
        
        let stats = timer.get_jitter_stats();
        assert_eq!(stats.total_samples, 3);
        assert!(stats.p95_us >= 105); // p95 should be at least 105us
    }

    #[test]
    fn test_timer_config_validation() {
        let mut config = HpetTimerConfig::default();
        let mut timer = HpetTimer::new(config.clone());
        
        // Test frequency setting
        assert!(timer.set_frequency(1000).is_ok());
        assert!(timer.set_frequency(0).is_err());
        assert!(timer.set_frequency(20000).is_err());
        
        // Test mode switching
        assert!(timer.set_mode(HpetTimerMode::OneShot).is_ok());
        assert!(timer.set_mode(HpetTimerMode::Periodic).is_ok());
    }
}
