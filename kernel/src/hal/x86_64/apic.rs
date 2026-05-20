/// Local APIC Timer Implementation for Polymera OS
/// 
/// This module provides Local APIC timer functionality to replace the legacy PIT
/// with higher precision timing, reduced jitter, and better preemption granularity.

use core::arch::x86_64::*;
use crate::hal::x86_64::msr::{rdmsr as __rdmsr, wrmsr as __wrmsr};
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use core::ptr;
use alloc::vec::Vec;
use alloc::string::String;

use crate::hal::HalError;
use crate::time::{Duration, Instant};
use crate::sync::Mutex;

/// APIC timer configuration
#[derive(Debug, Clone)]
pub struct ApicTimerConfig {
    pub vector: u8,
    pub divide_config: u8,
    pub initial_count: u32,
    pub mode: ApicTimerMode,
}

/// APIC timer modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApicTimerMode {
    OneShot,
    Periodic,
}

/// APIC timer state
pub struct ApicTimer {
    config: ApicTimerConfig,
    calibrated: bool,
    tsc_per_ms: AtomicU64,
    tick_count: AtomicU64,
    overrun_count: AtomicU32,
    jitter_histogram: Mutex<[u32; 64]>, // Fixed-size histogram for jitter tracking
}

impl ApicTimer {
    /// Create a new APIC timer
    pub fn new(config: ApicTimerConfig) -> Self {
        Self {
            config,
            calibrated: false,
            tsc_per_ms: AtomicU64::new(0),
            tick_count: AtomicU64::new(0),
            overrun_count: AtomicU32::new(0),
            jitter_histogram: Mutex::new([0; 64]),
        }
    }

    /// Initialize the APIC timer
    pub fn init(&mut self) -> Result<(), HalError> {
        // Check if LAPIC is present
        if !Self::is_lapic_present() {
            return Err(HalError::DeviceNotFound("LAPIC not present"));
        }

        // Enable LAPIC
        Self::enable_lapic()?;

        // Configure timer vector
        Self::set_timer_vector(self.config.vector)?;

        // Configure timer divide
        Self::set_timer_divide(self.config.divide_config)?;

        // Set initial count for periodic mode
        if self.config.mode == ApicTimerMode::Periodic {
            Self::set_timer_initial_count(self.config.initial_count)?;
        }

        // Enable timer
        Self::enable_timer()?;

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
            let tsc1 = Self::read_tsc();
            
            // Wait for next timer tick
            let tick_start = self.tick_count.load(Ordering::Relaxed);
            while self.tick_count.load(Ordering::Relaxed) == tick_start {
                core::hint::spin_loop();
            }
            
            let tsc2 = Self::read_tsc();
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

        // Convert to TSC per millisecond
        // Assuming 1000Hz timer (1ms per tick)
        let tsc_per_ms = median_tsc_per_tick * 1000;

        self.tsc_per_ms.store(tsc_per_ms, Ordering::Relaxed);
        self.calibrated = true;

        Ok(())
    }

    /// Get calibrated TSC per millisecond
    pub fn get_tsc_per_ms(&self) -> u64 {
        self.tsc_per_ms.load(Ordering::Relaxed)
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
        
        let mut histogram = self.jitter_histogram.lock();
        histogram[bin] = histogram[bin].saturating_add(1);
    }

    /// Get jitter statistics
    pub fn get_jitter_stats(&self) -> JitterStats {
        let histogram = self.jitter_histogram.lock();
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
        Self::send_eoi();
    }

    /// Set timer mode
    pub fn set_mode(&mut self, mode: ApicTimerMode) -> Result<(), HalError> {
        self.config.mode = mode;
        
        match mode {
            ApicTimerMode::OneShot => {
                Self::set_timer_mode_oneshot()?;
            }
            ApicTimerMode::Periodic => {
                Self::set_timer_mode_periodic()?;
                Self::set_timer_initial_count(self.config.initial_count)?;
            }
        }

        Ok(())
    }

    /// Set timer frequency (in Hz)
    pub fn set_frequency(&mut self, freq_hz: u32) -> Result<(), HalError> {
        if freq_hz == 0 || freq_hz > 10000 {
            return Err(HalError::InvalidParameter("Invalid frequency"));
        }

        // Calculate initial count based on frequency
        let apic_freq = Self::get_apic_frequency()?;
        let initial_count = apic_freq / freq_hz;
        
        self.config.initial_count = initial_count;
        
        if self.config.mode == ApicTimerMode::Periodic {
            Self::set_timer_initial_count(initial_count)?;
        }

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

// Low-level APIC operations
impl ApicTimer {
    /// Check if LAPIC is present
    fn is_lapic_present() -> bool {
        // Check CPUID for APIC support
        let cpuid_result = unsafe { __cpuid(1) };
        (cpuid_result.edx & (1 << 9)) != 0 // APIC bit in feature flags
    }

    /// Enable LAPIC
    fn enable_lapic() -> Result<(), HalError> {
        // Read APIC base from MSR
        let apic_base = unsafe { __rdmsr(0x1B) };
        let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;

        if apic_base_addr.is_null() {
            return Err(HalError::DeviceNotFound("Invalid APIC base address"));
        }

        // Enable APIC (bit 11 in APIC_BASE MSR)
        unsafe {
            __wrmsr(0x1B, apic_base | (1 << 11));
        }

        // Enable APIC in SVR (Spurious Vector Register)
        let svr_offset = 0xF0;
        let svr_value = unsafe { ptr::read_volatile(apic_base_addr.add(svr_offset / 4)) };
        let svr_enabled = svr_value | (1 << 8); // APIC Enable bit
        unsafe {
            ptr::write_volatile(apic_base_addr.add(svr_offset / 4), svr_enabled);
        }

        Ok(())
    }

    /// Set timer vector
    fn set_timer_vector(vector: u8) -> Result<(), HalError> {
        let apic_base = unsafe { __rdmsr(0x1B) };
        let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;
        
        let lvt_timer_offset = 0x320;
        let lvt_value = (vector as u32) | 0x20000; // Timer mode: periodic
        
        unsafe {
            ptr::write_volatile(apic_base_addr.add(lvt_timer_offset / 4), lvt_value);
        }

        Ok(())
    }

    /// Set timer divide configuration
    fn set_timer_divide(divide: u8) -> Result<(), HalError> {
        let apic_base = unsafe { __rdmsr(0x1B) };
        let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;
        
        let timer_divide_offset = 0x3E0;
        let divide_value = (divide as u32) << 20;
        
        unsafe {
            ptr::write_volatile(apic_base_addr.add(timer_divide_offset / 4), divide_value);
        }

        Ok(())
    }

    /// Set timer initial count
    fn set_timer_initial_count(count: u32) -> Result<(), HalError> {
        let apic_base = unsafe { __rdmsr(0x1B) };
        let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;
        
        let timer_initial_count_offset = 0x380;
        
        unsafe {
            ptr::write_volatile(apic_base_addr.add(timer_initial_count_offset / 4), count);
        }

        Ok(())
    }

    /// Enable timer
    fn enable_timer() -> Result<(), HalError> {
        let apic_base = unsafe { __rdmsr(0x1B) };
        let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;
        
        let lvt_timer_offset = 0x320;
        let lvt_value = unsafe { ptr::read_volatile(apic_base_addr.add(lvt_timer_offset / 4)) };
        let lvt_enabled = lvt_value & !0x10000; // Clear masked bit
        
        unsafe {
            ptr::write_volatile(apic_base_addr.add(lvt_timer_offset / 4), lvt_enabled);
        }

        Ok(())
    }

    /// Set timer to one-shot mode
    fn set_timer_mode_oneshot() -> Result<(), HalError> {
        let apic_base = unsafe { __rdmsr(0x1B) };
        let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;
        
        let lvt_timer_offset = 0x320;
        let lvt_value = unsafe { ptr::read_volatile(apic_base_addr.add(lvt_timer_offset / 4)) };
        let lvt_oneshot = (lvt_value & !0x60000) | 0x00000; // One-shot mode
        
        unsafe {
            ptr::write_volatile(apic_base_addr.add(lvt_timer_offset / 4), lvt_oneshot);
        }

        Ok(())
    }

    /// Set timer to periodic mode
    fn set_timer_mode_periodic() -> Result<(), HalError> {
        let apic_base = unsafe { __rdmsr(0x1B) };
        let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;
        
        let lvt_timer_offset = 0x320;
        let lvt_value = unsafe { ptr::read_volatile(apic_base_addr.add(lvt_timer_offset / 4)) };
        let lvt_periodic = (lvt_value & !0x60000) | 0x20000; // Periodic mode
        
        unsafe {
            ptr::write_volatile(apic_base_addr.add(lvt_timer_offset / 4), lvt_periodic);
        }

        Ok(())
    }

    /// Get APIC frequency
    fn get_apic_frequency() -> Result<u32, HalError> {
        // Default APIC frequency (can be calibrated more precisely)
        Ok(100_000_000) // 100 MHz
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

    /// Send EOI (End of Interrupt)
    fn send_eoi() {
        let apic_base = unsafe { __rdmsr(0x1B) };
        let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;
        
        let eoi_offset = 0xB0;
        
        unsafe {
            ptr::write_volatile(apic_base_addr.add(eoi_offset / 4), 0);
        }
    }
}

/// Default APIC timer configuration
impl Default for ApicTimerConfig {
    fn default() -> Self {
        Self {
            vector: 32, // IRQ 0
            divide_config: 3, // Divide by 16
            initial_count: 62500, // 1000Hz timer (assuming 100MHz APIC)
            mode: ApicTimerMode::Periodic,
        }
    }
}

/// Timer metrics exposed to the HAL.
#[derive(Debug, Clone)]
pub struct TimerMetrics {
    pub tick_count: u64,
    pub overrun_count: u32,
    pub tsc_per_ms: u64,
}

/// Detect whether an APIC timer is usable on this CPU.
pub fn is_apic_timer_available() -> bool {
    let cpuid_result = unsafe { __cpuid(1) };
    (cpuid_result.edx & (1 << 9)) != 0
}

/// Initialize the APIC timer subsystem (stub – uses default configuration).
pub fn init_apic_timer() -> Result<(), &'static str> {
    if !is_apic_timer_available() {
        return Err("APIC timer not available");
    }
    Ok(())
}

/// Print APIC timer statistics (stub).
pub fn print_apic_timer_stats() {
    crate::kprintln!("[APIC] timer stats: stub");
}

/// Get APIC timer metrics if available.
pub fn get_apic_timer_metrics() -> Option<TimerMetrics> {
    if is_apic_timer_available() {
        Some(TimerMetrics { tick_count: 0, overrun_count: 0, tsc_per_ms: 0 })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apic_timer_creation() {
        let config = ApicTimerConfig::default();
        let timer = ApicTimer::new(config);
        
        assert_eq!(timer.get_tick_count(), 0);
        assert_eq!(timer.get_overrun_count(), 0);
        assert!(!timer.is_calibrated());
    }

    #[test]
    fn test_jitter_recording() {
        let config = ApicTimerConfig::default();
        let timer = ApicTimer::new(config);
        
        // Record some jitter values
        timer.record_jitter(10);  // 10us
        timer.record_jitter(50);  // 50us
        timer.record_jitter(100); // 100us
        
        let stats = timer.get_jitter_stats();
        assert_eq!(stats.total_samples, 3);
        assert!(stats.p95_us >= 100); // p95 should be at least 100us
    }

    #[test]
    fn test_timer_config_validation() {
        let mut config = ApicTimerConfig::default();
        let mut timer = ApicTimer::new(config.clone());
        
        // Test frequency setting
        assert!(timer.set_frequency(1000).is_ok());
        assert!(timer.set_frequency(0).is_err());
        assert!(timer.set_frequency(20000).is_err());
        
        // Test mode switching
        assert!(timer.set_mode(ApicTimerMode::OneShot).is_ok());
        assert!(timer.set_mode(ApicTimerMode::Periodic).is_ok());
    }
}
