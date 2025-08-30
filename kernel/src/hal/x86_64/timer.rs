use crate::{kprintln, klog};
use lazy_static::lazy_static;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::instructions::port::Port;

use core::sync::atomic::{AtomicU8, Ordering};
use alloc::string::String;

use crate::hal::HalError;
use crate::time::{Duration, Instant};
use crate::sync::Mutex;

use super::apic::{ApicTimer, ApicTimerConfig, ApicTimerMode};
use super::hpet::{HpetTimer, HpetTimerConfig, HpetTimerMode};

/// Timer frequency in Hz (1000 Hz = 1ms per tick)
const TIMER_FREQUENCY: u32 = 1000;

/// PIT (Programmable Interval Timer) constants
const PIT_FREQUENCY: u32 = 1193182; // Base PIT frequency
const PIT_CHANNEL_0: u16 = 0x40;    // Channel 0 data port
const PIT_COMMAND: u16 = 0x43;      // Command register
const PIT_DIVISOR: u16 = (PIT_FREQUENCY / TIMER_FREQUENCY) as u16;

/// PIC (Programmable Interrupt Controller) constants
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// Timer interrupt vector (after PIC remapping)
const TIMER_IRQ: u8 = 32; // IRQ 0 mapped to interrupt 32

/// Global tick counter
static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Trace logging frequency (every 100 ticks = every 100ms)
const TRACE_INTERVAL: u64 = 100;

lazy_static! {
    /// Port access for PIT and PIC
    static ref PIT_CHANNEL_0_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(PIT_CHANNEL_0));
    static ref PIT_COMMAND_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(PIT_COMMAND));
    static ref PIC1_COMMAND_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(PIC1_COMMAND));
    static ref PIC1_DATA_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(PIC1_DATA));
    static ref PIC2_COMMAND_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(PIC2_COMMAND));
    static ref PIC2_DATA_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(PIC2_DATA));
}

/// Timer subsystem state
#[derive(Debug, Clone, Copy)]
pub enum TimerType {
    /// Legacy PIT (8253/8254)
    PIT,
    /// Advanced Programmable Interrupt Controller
    APIC,
    /// No timer available
    None,
}

static TIMER_TYPE: AtomicU64 = AtomicU64::new(TimerType::None as u64);

/// Timer selection result
#[derive(Debug, Clone, PartialEq)]
pub enum TimerKind {
    APIC,
    HPET,
}

/// Timer selection state
pub struct TimerSelector {
    selected_timer: AtomicU8, // 0 = None, 1 = APIC, 2 = HPET
    apic_timer: Mutex<Option<ApicTimer>>,
    hpet_timer: Mutex<Option<HpetTimer>>,
    calibration_data: Mutex<CalibrationData>,
}

/// Calibration data for selected timer
#[derive(Debug, Clone)]
struct CalibrationData {
    timer_kind: TimerKind,
    vector: u8,
    tsc_per_ms: Option<u64>,
    ns_per_tick: Option<u64>,
    calibration_time: Duration,
}

impl TimerSelector {
    /// Create a new timer selector
    pub fn new() -> Self {
        Self {
            selected_timer: AtomicU8::new(0),
            apic_timer: Mutex::new(None),
            hpet_timer: Mutex::new(None),
            calibration_data: Mutex::new(CalibrationData {
                timer_kind: TimerKind::APIC,
                vector: 0,
                tsc_per_ms: None,
                ns_per_tick: None,
                calibration_time: Duration::from_millis(0),
            }),
        }
    }

    /// Select and initialize the best available timer
    pub fn select_timer(&mut self) -> Result<TimerKind, HalError> {
        // Try APIC first (preferred)
        if let Ok(apic_timer) = self::try_init_apic() {
            let mut apic_guard = self.apic_timer.lock().map_err(|_| {
                HalError::LockError("Failed to acquire APIC timer lock")
            })?;
            
            *apic_guard = Some(apic_timer);
            
            // Store calibration data
            if let Ok(mut cal_data) = self.calibration_data.lock() {
                cal_data.timer_kind = TimerKind::APIC;
                cal_data.vector = 32; // APIC timer vector
                cal_data.tsc_per_ms = apic_guard.as_ref().map(|t| t.get_tsc_per_ms());
                cal_data.calibration_time = Duration::from_millis(10);
            }
            
            self.selected_timer.store(1, Ordering::Relaxed);
            
            // Print boot banner
            self.print_boot_banner(TimerKind::APIC)?;
            
            return Ok(TimerKind::APIC);
        }

        // Fallback to HPET
        if let Ok(hpet_timer) = self::try_init_hpet() {
            let mut hpet_guard = self.hpet_timer.lock().map_err(|_| {
                HalError::LockError("Failed to acquire HPET timer lock")
            })?;
            
            *hpet_guard = Some(hpet_timer);
            
            // Store calibration data
            if let Ok(mut cal_data) = self.calibration_data.lock() {
                cal_data.timer_kind = TimerKind::HPET;
                cal_data.vector = 33; // HPET timer vector
                cal_data.ns_per_tick = hpet_guard.as_ref().map(|t| t.get_ns_per_tick());
                cal_data.calibration_time = Duration::from_millis(10);
            }
            
            self.selected_timer.store(2, Ordering::Relaxed);
            
            // Print boot banner
            self.print_boot_banner(TimerKind::HPET)?;
            
            return Ok(TimerKind::HPET);
        }

        // No timer available
        Err(HalError::DeviceNotFound("No timer available (APIC or HPET)"))
    }

    /// Get the currently selected timer kind
    pub fn get_timer_kind(&self) -> Option<TimerKind> {
        match self.selected_timer.load(Ordering::Relaxed) {
            1 => Some(TimerKind::APIC),
            2 => Some(TimerKind::HPET),
            _ => None,
        }
    }

    /// Get the APIC timer (if selected)
    pub fn get_apic_timer(&self) -> Option<&ApicTimer> {
        if self.selected_timer.load(Ordering::Relaxed) == 1 {
            if let Ok(guard) = self.apic_timer.lock() {
                guard.as_ref()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the HPET timer (if selected)
    pub fn get_hpet_timer(&self) -> Option<&HpetTimer> {
        if self.selected_timer.load(Ordering::Relaxed) == 2 {
            if let Ok(guard) = self.hpet_timer.lock() {
                guard.as_ref()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the active timer (either APIC or HPET)
    pub fn get_active_timer(&self) -> Option<ActiveTimer> {
        match self.get_timer_kind() {
            Some(TimerKind::APIC) => {
                if let Some(apic) = self.get_apic_timer() {
                    Some(ActiveTimer::APIC(apic))
                } else {
                    None
                }
            }
            Some(TimerKind::HPET) => {
                if let Some(hpet) = self.get_hpet_timer() {
                    Some(ActiveTimer::HPET(hpet))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Print boot banner with timer selection and calibration info
    fn print_boot_banner(&self, timer_kind: TimerKind) -> Result<(), HalError> {
        if let Ok(cal_data) = self.calibration_data.lock() {
            match timer_kind {
                TimerKind::APIC => {
                    if let Some(tsc_per_ms) = cal_data.tsc_per_ms {
                        crate::kprintln!(
                            "[TIMER] selected=APIC cal_tsc_per_ms={} vector={}",
                            tsc_per_ms,
                            cal_data.vector
                        );
                    } else {
                        crate::kprintln!(
                            "[TIMER] selected=APIC cal_tsc_per_ms=unknown vector={}",
                            cal_data.vector
                        );
                    }
                }
                TimerKind::HPET => {
                    if let Some(ns_per_tick) = cal_data.ns_per_tick {
                        crate::kprintln!(
                            "[TIMER] selected=HPET cal_ns_per_tick={} vector={}",
                            ns_per_tick,
                            cal_data.vector
                        );
                    } else {
                        crate::kprintln!(
                            "[TIMER] selected=HPET cal_ns_per_tick=unknown vector={}",
                            cal_data.vector
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Get calibration data for metrics
    pub fn get_calibration_data(&self) -> Option<CalibrationData> {
        if let Ok(cal_data) = self.calibration_data.lock() {
            Some(cal_data.clone())
        } else {
            None
        }
    }

    /// Check if timer is available and working
    pub fn is_timer_available(&self) -> bool {
        self.selected_timer.load(Ordering::Relaxed) != 0
    }

    /// Get timer statistics for the active timer
    pub fn get_timer_stats(&self) -> Option<TimerStats> {
        match self.get_active_timer() {
            Some(ActiveTimer::APIC(apic)) => {
                let jitter_stats = apic.get_jitter_stats();
                Some(TimerStats {
                    timer_kind: TimerKind::APIC,
                    tick_count: apic.get_tick_count(),
                    overrun_count: apic.get_overrun_count(),
                    jitter_mean_us: jitter_stats.mean_us,
                    jitter_p95_us: jitter_stats.p95_us,
                    jitter_samples: jitter_stats.total_samples,
                })
            }
            Some(ActiveTimer::HPET(hpet)) => {
                let jitter_stats = hpet.get_jitter_stats();
                Some(TimerStats {
                    timer_kind: TimerKind::HPET,
                    tick_count: hpet.get_tick_count(),
                    overrun_count: hpet.get_overrun_count(),
                    jitter_mean_us: jitter_stats.mean_us,
                    jitter_p95_us: jitter_stats.p95_us,
                    jitter_samples: jitter_stats.total_samples,
                })
            }
            None => None,
        }
    }

    /// Handle timer interrupt (delegates to active timer)
    pub fn handle_timer_interrupt(&self) {
        if let Some(active_timer) = self.get_active_timer() {
            match active_timer {
                ActiveTimer::APIC(apic) => {
                    apic.handle_interrupt();
                }
                ActiveTimer::HPET(hpet) => {
                    hpet.handle_interrupt();
                }
            }
        }
    }

    /// Record jitter measurement for the active timer
    pub fn record_jitter(&self, jitter_us: u32) {
        if let Some(active_timer) = self.get_active_timer() {
            match active_timer {
                ActiveTimer::APIC(apic) => {
                    apic.record_jitter(jitter_us);
                }
                ActiveTimer::HPET(hpet) => {
                    hpet.record_jitter(jitter_us);
                }
            }
        }
    }

    /// Get jitter statistics for the active timer
    pub fn get_jitter_stats(&self) -> Option<JitterStats> {
        if let Some(active_timer) = self.get_active_timer() {
            match active_timer {
                ActiveTimer::APIC(apic) => {
                    Some(apic.get_jitter_stats())
                }
                ActiveTimer::HPET(hpet) => {
                    Some(hpet.get_jitter_stats())
                }
            }
        } else {
            None
        }
    }
}

/// Active timer reference
pub enum ActiveTimer<'a> {
    APIC(&'a ApicTimer),
    HPET(&'a HpetTimer),
}

/// Timer statistics
#[derive(Debug, Clone)]
pub struct TimerStats {
    pub timer_kind: TimerKind,
    pub tick_count: u64,
    pub overrun_count: u32,
    pub jitter_mean_us: u32,
    pub jitter_p95_us: u32,
    pub jitter_samples: u32,
}

/// Jitter statistics
#[derive(Debug, Clone)]
pub struct JitterStats {
    pub mean_us: u32,
    pub p95_us: u32,
    pub total_samples: u32,
}

// Timer initialization functions
impl TimerSelector {
    /// Try to initialize APIC timer
    fn try_init_apic() -> Result<ApicTimer, HalError> {
        let config = ApicTimerConfig::default();
        let mut timer = ApicTimer::new(config);
        
        // Try to initialize
        timer.init()?;
        
        // Verify it's working
        if !timer.is_calibrated() {
            return Err(HalError::CalibrationFailed("APIC timer calibration failed"));
        }
        
        Ok(timer)
    }

    /// Try to initialize HPET timer
    fn try_init_hpet() -> Result<HpetTimer, HalError> {
        let config = HpetTimerConfig::default();
        let mut timer = HpetTimer::new(config);
        
        // Try to initialize
        timer.init()?;
        
        // Verify it's working
        if !timer.is_calibrated() {
            return Err(HalError::CalibrationFailed("HPET timer calibration failed"));
        }
        
        Ok(timer)
    }
}

/// Global timer selector instance
static mut TIMER_SELECTOR: Option<TimerSelector> = None;

/// Initialize the timer selector
pub fn init_timer_selector() -> Result<TimerKind, HalError> {
    unsafe {
        if TIMER_SELECTOR.is_none() {
            TIMER_SELECTOR = Some(TimerSelector::new());
        }
        
        if let Some(selector) = &mut TIMER_SELECTOR {
            selector.select_timer()
        } else {
            Err(HalError::InitializationFailed("Failed to create timer selector"))
        }
    }
}

/// Get the timer selector instance
pub fn get_timer_selector() -> Option<&'static TimerSelector> {
    unsafe {
        TIMER_SELECTOR.as_ref()
    }
}

/// Get the currently selected timer kind
pub fn get_timer_kind() -> Option<TimerKind> {
    get_timer_selector().and_then(|selector| selector.get_timer_kind())
}

/// Check if APIC timer is selected
pub fn is_apic_selected() -> bool {
    get_timer_kind() == Some(TimerKind::APIC)
}

/// Check if HPET timer is selected
pub fn is_hpet_selected() -> bool {
    get_timer_kind() == Some(TimerKind::HPET)
}

/// Handle timer interrupt (global handler)
pub fn handle_timer_interrupt() {
    if let Some(selector) = get_timer_selector() {
        selector.handle_timer_interrupt();
    }
}

/// Record jitter measurement (global function)
pub fn record_jitter(jitter_us: u32) {
    if let Some(selector) = get_timer_selector() {
        selector.record_jitter(jitter_us);
    }
}

/// Get jitter statistics (global function)
pub fn get_jitter_stats() -> Option<JitterStats> {
    get_timer_selector().and_then(|selector| selector.get_jitter_stats())
}

/// Get timer statistics (global function)
pub fn get_timer_stats() -> Option<TimerStats> {
    get_timer_selector().and_then(|selector| selector.get_timer_stats())
}

/// Print timer status (for debugging)
pub fn print_timer_status() {
    if let Some(selector) = get_timer_selector() {
        if let Some(stats) = selector.get_timer_stats() {
            crate::kprintln!("=== Timer Status ===");
            crate::kprintln!("Selected: {:?}", stats.timer_kind);
            crate::kprintln!("Tick Count: {}", stats.tick_count);
            crate::kprintln!("Overrun Count: {}", stats.overrun_count);
            crate::kprintln!("Jitter - Mean: {}µs, P95: {}µs, Samples: {}", 
                           stats.jitter_mean_us, stats.jitter_p95_us, stats.jitter_samples);
            crate::kprintln!("==================");
        } else {
            crate::kprintln!("No timer statistics available");
        }
    } else {
        crate::kprintln!("Timer selector not initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_selector_creation() {
        let selector = TimerSelector::new();
        assert_eq!(selector.get_timer_kind(), None);
        assert!(!selector.is_timer_available());
    }

    #[test]
    fn test_timer_kind_enum() {
        assert_eq!(TimerKind::APIC, TimerKind::APIC);
        assert_eq!(TimerKind::HPET, TimerKind::HPET);
        assert_ne!(TimerKind::APIC, TimerKind::HPET);
    }

    #[test]
    fn test_timer_stats_creation() {
        let stats = TimerStats {
            timer_kind: TimerKind::APIC,
            tick_count: 1000,
            overrun_count: 5,
            jitter_mean_us: 50,
            jitter_p95_us: 100,
            jitter_samples: 1000,
        };
        
        assert_eq!(stats.timer_kind, TimerKind::APIC);
        assert_eq!(stats.tick_count, 1000);
        assert_eq!(stats.overrun_count, 5);
        assert_eq!(stats.jitter_mean_us, 50);
        assert_eq!(stats.jitter_p95_us, 100);
        assert_eq!(stats.jitter_samples, 1000);
    }

    #[test]
    fn test_jitter_stats_creation() {
        let stats = JitterStats {
            mean_us: 25,
            p95_us: 75,
            total_samples: 500,
        };
        
        assert_eq!(stats.mean_us, 25);
        assert_eq!(stats.p95_us, 75);
        assert_eq!(stats.total_samples, 500);
    }
}

/// Initialize the timer subsystem
pub fn init() {
    kprintln!("[HAL] Timer init - detecting available timers");
    
    // For Phase 1, we'll use PIT as it's universally available
    // Future phases will add APIC detection and support
    if init_pit() {
        TIMER_TYPE.store(TimerType::PIT as u64, Ordering::Relaxed);
        kprintln!("[HAL] Timer initialized: PIT at {}Hz", TIMER_FREQUENCY);
    } else {
        kprintln!("[HAL] Timer initialization failed!");
        TIMER_TYPE.store(TimerType::None as u64, Ordering::Relaxed);
    }
}

/// Initialize the Programmable Interval Timer (PIT)
fn init_pit() -> bool {
    kprintln!("[HAL] Initializing PIT (8253/8254)");
    
    // First, remap the PIC to avoid conflicts with exceptions
    remap_pic();
    
    // Configure PIT Channel 0 for periodic interrupts
    unsafe {
        // Command: Channel 0, Low/High byte, Mode 2 (rate generator), Binary
        let command = 0b00110100; // 0x34
        PIT_COMMAND_PORT.lock().write(command);
        
        // Set the divisor for desired frequency
        let divisor_low = (PIT_DIVISOR & 0xFF) as u8;
        let divisor_high = ((PIT_DIVISOR >> 8) & 0xFF) as u8;
        
        PIT_CHANNEL_0_PORT.lock().write(divisor_low);
        PIT_CHANNEL_0_PORT.lock().write(divisor_high);
    }
    
    kprintln!("[HAL] PIT configured: divisor={}, frequency={}Hz", PIT_DIVISOR, TIMER_FREQUENCY);
    
    // Timer interrupt handler is installed directly in IDT (idt.rs)
    
    // Enable timer interrupt (IRQ 0)
    enable_timer_irq();
    
    kprintln!("[HAL] PIT initialization complete");
    true
}

/// Remap the Programmable Interrupt Controller (PIC)
/// 
/// The PIC needs to be remapped because the default vectors (0-15) conflict
/// with CPU exceptions. We remap IRQ 0-7 to interrupts 32-39 and IRQ 8-15 to 40-47.
fn remap_pic() {
    kprintln!("[HAL] Remapping PIC: IRQ 0-7 -> INT 32-39, IRQ 8-15 -> INT 40-47");
    
    unsafe {
        // Save current masks
        let pic1_mask = PIC1_DATA_PORT.lock().read();
        let pic2_mask = PIC2_DATA_PORT.lock().read();
        
        // Start initialization sequence for both PICs
        PIC1_COMMAND_PORT.lock().write(0x11); // ICW1: Initialize + ICW4 needed
        PIC2_COMMAND_PORT.lock().write(0x11);
        
        // ICW2: Set vector offsets
        PIC1_DATA_PORT.lock().write(32);  // PIC1 starts at interrupt 32
        PIC2_DATA_PORT.lock().write(40);  // PIC2 starts at interrupt 40
        
        // ICW3: Tell PIC1 there's a slave PIC at IRQ2
        PIC1_DATA_PORT.lock().write(0x04); // Binary: 00000100
        PIC2_DATA_PORT.lock().write(0x02); // Slave ID: 2
        
        // ICW4: Set mode (8086 mode, not buffered, normal EOI)
        PIC1_DATA_PORT.lock().write(0x01);
        PIC2_DATA_PORT.lock().write(0x01);
        
        // Restore saved masks
        PIC1_DATA_PORT.lock().write(pic1_mask);
        PIC2_DATA_PORT.lock().write(pic2_mask);
    }
    
    kprintln!("[HAL] PIC remapping complete");
}

/// Timer interrupt handler is installed directly in the IDT (see idt.rs)
/// 
/// The handler is registered at vector 32 (IRQ 0 after PIC remapping) and
/// calls timer_interrupt_handler() on each timer tick.

/// Enable timer interrupt (IRQ 0) in the PIC
fn enable_timer_irq() {
    unsafe {
        // Read current mask
        let mut mask = PIC1_DATA_PORT.lock().read();
        
        // Clear bit 0 to enable IRQ 0 (timer)
        mask &= !0x01;
        
        // Write back the mask
        PIC1_DATA_PORT.lock().write(mask);
    }
    
    kprintln!("[HAL] Timer IRQ enabled (IRQ 0)");
}

/// Timer interrupt service routine (ISR)
/// 
/// This function is called every timer tick (1000 times per second)
pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Increment the global tick counter
    let tick = TICK_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    
    // TRACE logging every 100 ticks (every 100ms)
    if tick % TRACE_INTERVAL == 0 {
        klog!(TRACE, "[TIMER] Tick #{} ({}ms elapsed)", tick, tick);
    }
    
    // Call the scheduler tick hook (stub for now)
    on_tick(tick);
    
    // Send End of Interrupt (EOI) to PIC
    send_eoi();
}

/// Scheduler tick callback with full integration
/// 
/// This function is called on every timer tick and integrates with the scheduler
/// to perform task switching and time accounting.
fn on_tick(tick_count: u64) {
    // Apply fault injection timer jitter if enabled
    let jitter = crate::fault_injection::get_timer_jitter();
    if jitter != 0 {
        // Add jitter to the tick count for fault injection testing
        let adjusted_tick = tick_count.wrapping_add_signed(jitter);
        crate::kprintln!("[FAULT_INJECTION] Timer jitter: {}ms (tick {} -> {})", 
                        jitter, tick_count, adjusted_tick);
        
        // Use adjusted tick for fault injection testing
        crate::fault_injection::set_base_timer(adjusted_tick);
    }
    
    // Update logging timestamp for rate limiting (convert ticks to milliseconds)
    crate::log::update_timestamp_ms(tick_count * (1000 / TIMER_FREQUENCY_HZ));
    
    // Trace the tick for statistics
    crate::trace::trace_tick();
    
    // Call scheduler tick handler for preemptive scheduling
    crate::sched::tick::on_timer_tick(tick_count);
    
    // Track basic statistics
    if tick_count % 1000 == 0 {
        let seconds = tick_count / 1000;
        klog!(INFO, "[TIMER] System uptime: {}s ({}k ticks)", seconds, tick_count / 1000);
        
        // Print scheduler statistics periodically
        if tick_count % 5000 == 0 { // Every 5 seconds
            let stats = crate::sched::get_scheduler_stats();
            klog!(INFO, "[TIMER] Scheduler: {} tasks ({} ready, {} running)", 
                  stats.total_tasks, stats.ready_tasks, stats.running_tasks);
        }
    }
}

/// Send End of Interrupt (EOI) to the PIC
/// 
/// This tells the PIC that we've finished handling the interrupt
/// and it can send the next interrupt if one is pending.
fn send_eoi() {
    unsafe {
        // Send EOI to PIC1 (required for all IRQs)
        PIC1_COMMAND_PORT.lock().write(0x20);
        
        // For IRQs 8-15, we would also need to send EOI to PIC2:
        // PIC2_COMMAND_PORT.lock().write(0x20);
        // But IRQ 0 (timer) only requires PIC1 EOI
    }
}

/// Get the current tick count
pub fn get_tick_count() -> u64 {
    TICK_COUNTER.load(Ordering::Relaxed)
}

/// Get the current uptime in milliseconds
pub fn get_uptime_ms() -> u64 {
    get_tick_count() // Since we tick at 1000Hz, ticks = milliseconds
}

/// Get the current uptime in seconds
pub fn get_uptime_seconds() -> u64 {
    get_tick_count() / 1000
}

/// Get the timer type currently in use
pub fn get_timer_type() -> TimerType {
    match TIMER_TYPE.load(Ordering::Relaxed) {
        0 => TimerType::None,
        1 => TimerType::PIT,
        2 => TimerType::APIC,
        _ => TimerType::None,
    }
}

/// Timer statistics for debugging
pub fn print_timer_stats() {
    let tick_count = get_tick_count();
    let uptime_ms = get_uptime_ms();
    let uptime_sec = get_uptime_seconds();
    let timer_type = get_timer_type();
    
    kprintln!("");
    kprintln!("=== TIMER STATISTICS ===");
    kprintln!("Timer Type: {:?}", timer_type);
    kprintln!("Frequency: {}Hz", TIMER_FREQUENCY);
    kprintln!("Tick Count: {}", tick_count);
    kprintln!("Uptime: {}ms ({}s)", uptime_ms, uptime_sec);
    kprintln!("PIT Divisor: {}", PIT_DIVISOR);
    kprintln!("=== END TIMER STATISTICS ===");
    kprintln!("");
}

/// Test function to verify timer functionality
#[allow(dead_code)]
pub fn test_timer() {
    kprintln!("Testing timer functionality...");
    
    let start_tick = get_tick_count();
    kprintln!("Start tick: {}", start_tick);
    
    // Wait for some ticks (this is a simple delay, not recommended for production)
    let target_tick = start_tick + 10;
    while get_tick_count() < target_tick {
        x86_64::instructions::hlt(); // Wait for interrupts
    }
    
    let end_tick = get_tick_count();
    kprintln!("End tick: {} (waited {} ticks)", end_tick, end_tick - start_tick);
    
    print_timer_stats();
}

/// Future APIC timer initialization (stub)
#[allow(dead_code)]
fn init_apic() -> bool {
    kprintln!("[HAL] APIC timer initialization not yet implemented");
    
    // TODO: Implement APIC timer for better performance and accuracy
    // APIC provides:
    // - Higher resolution timing
    // - Per-CPU timers (important for SMP)
    // - Better integration with modern CPUs
    // - Reduced interrupt latency
    
    false
}

/// Initialize timer subsystem based on CPU capabilities
#[allow(dead_code)]
pub fn init_best_available() {
    kprintln!("[HAL] Detecting best available timer...");
    
    // Try APIC first (better performance)
    if init_apic() {
        TIMER_TYPE.store(TimerType::APIC as u64, Ordering::Relaxed);
        kprintln!("[HAL] Using APIC timer");
        return;
    }
    
    // Fall back to PIT (universally available)
    if init_pit() {
        TIMER_TYPE.store(TimerType::PIT as u64, Ordering::Relaxed);
        kprintln!("[HAL] Using PIT timer");
        return;
    }
    
    // No timer available
    TIMER_TYPE.store(TimerType::None as u64, Ordering::Relaxed);
    kprintln!("[HAL] No timer available!");
}
