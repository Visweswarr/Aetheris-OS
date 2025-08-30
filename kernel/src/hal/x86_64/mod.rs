pub mod gdt;
pub mod idt;
pub mod timer;
pub mod apic;
pub mod hpet;
pub mod tss;

#[cfg(debug_assertions)]
pub mod idt_test;

#[cfg(debug_assertions)]
pub mod timer_test;

pub struct X64Hal;

impl crate::hal::Hal for X64Hal {
    fn init_cpu() -> Result<(), &'static str> { 
        let _selectors = gdt::init(); 
        tss::init();
        idt::init(); 
        Ok(())
    }
    fn init_timer() -> Result<(), &'static str> { 
        // Try APIC timer first (better performance)
        if apic::is_apic_timer_available() {
            match apic::init_apic_timer() {
                Ok(()) => {
                    // Set scheduler tick source to APIC
                    crate::sched::tick::set_tick_source(crate::sched::tick::TickSource::APIC);
                    return Ok(());
                }
                Err(e) => {
                    crate::kprintln!("[HAL] APIC timer initialization failed: {}, trying HPET fallback", e);
                }
            }
        }
        
        // Try HPET as fallback (high precision, low jitter)
        if hpet::is_hpet_available() {
            match hpet::init_hpet() {
                Ok(()) => {
                    // Set scheduler tick source to HPET
                    crate::sched::tick::set_tick_source(crate::sched::tick::TickSource::HPET);
                    return Ok(());
                }
                Err(e) => {
                    crate::kprintln!("[HAL] HPET timer initialization failed: {}, falling back to PIT", e);
                }
            }
        }
        
        // Fall back to PIT timer (legacy, always available)
        timer::init(); 
        crate::sched::tick::set_tick_source(crate::sched::tick::TickSource::PIT);
        Ok(())
    }
    fn enable_interrupts() -> Result<(), &'static str> { 
        unsafe { x86_64::instructions::interrupts::enable(); } 
        Ok(())
    }
}

impl X64Hal {
    /// Test the IDT page fault handler (for debugging)
    #[allow(dead_code)]
    pub fn test_page_fault() {
        idt::test_page_fault();
    }
    
    /// Test the IDT double fault handler (for debugging)
    #[allow(dead_code)]
    pub fn test_double_fault() {
        idt::test_double_fault();
    }
    
    /// Trigger a breakpoint exception (for debugging)
    #[allow(dead_code)]
    pub fn test_breakpoint() {
        crate::kprintln!("Triggering breakpoint...");
        x86_64::instructions::interrupts::int3();
    }
    
    /// Test timer functionality
    #[allow(dead_code)]
    pub fn test_timer() {
        timer::test_timer();
    }
    
    /// Print timer statistics
    #[allow(dead_code)]
    pub fn print_timer_stats() {
        timer::print_timer_stats();
    }
    
    /// Get current system uptime in milliseconds
    pub fn get_uptime_ms() -> u64 {
        timer::get_uptime_ms()
    }
    
    /// Get current system uptime in seconds
    pub fn get_uptime_seconds() -> u64 {
        timer::get_uptime_seconds()
    }
    
    /// Get current tick count
    pub fn get_tick_count() -> u64 {
        timer::get_tick_count()
    }
    
    /// Test APIC timer functionality
    #[allow(dead_code)]
    pub fn test_apic_timer() {
        if apic::is_apic_timer_available() {
            apic::print_apic_timer_stats();
        } else {
            crate::kprintln!("[HAL] APIC timer not available");
        }
    }
    
    /// Test HPET timer functionality
    #[allow(dead_code)]
    pub fn test_hpet_timer() {
        if hpet::is_hpet_available() {
            hpet::print_hpet_stats();
        } else {
            crate::kprintln!("[HAL] HPET timer not available");
        }
    }
    
    /// Test timer fallback functionality
    #[allow(dead_code)]
    pub fn test_timer_fallback() {
        crate::kprintln!("[HAL] Testing timer fallback sequence...");
        
        // Test APIC
        if apic::is_apic_timer_available() {
            crate::kprintln!("[HAL] APIC timer: AVAILABLE");
            if apic::init_apic_timer().is_ok() {
                crate::kprintln!("[HAL] APIC timer: INITIALIZED");
            } else {
                crate::kprintln!("[HAL] APIC timer: INITIALIZATION FAILED");
            }
        } else {
            crate::kprintln!("[HAL] APIC timer: NOT AVAILABLE");
        }
        
        // Test HPET
        if hpet::is_hpet_available() {
            crate::kprintln!("[HAL] HPET timer: AVAILABLE");
            if hpet::init_hpet().is_ok() {
                crate::kprintln!("[HAL] HPET timer: INITIALIZED");
            } else {
                crate::kprintln!("[HAL] HPET timer: INITIALIZATION FAILED");
            }
        } else {
            crate::kprintln!("[HAL] HPET timer: NOT AVAILABLE");
        }
        
        // Test PIT
        crate::kprintln!("[HAL] PIT timer: ALWAYS AVAILABLE");
        timer::init();
        crate::kprintln!("[HAL] PIT timer: INITIALIZED");
    }
            crate::kprintln!("[HAL] APIC timer not available");
        }
    }
    
    /// Print APIC timer statistics
    #[allow(dead_code)]
    pub fn print_apic_timer_stats() {
        apic::print_apic_timer_stats();
    }
    
    /// Get APIC timer metrics
    #[allow(dead_code)]
    pub fn get_apic_timer_metrics() -> Option<apic::TimerMetrics> {
        apic::get_apic_timer_metrics().cloned()
    }
    
    /// Run APIC preemption demo
    #[allow(dead_code)]
    pub fn run_apic_preemption_demo() {
        crate::sched::tick::apic_preemption_demo();
    }
    
    /// Print tick jitter statistics
    #[allow(dead_code)]
    pub fn print_tick_jitter_stats() {
        crate::sched::tick::print_jitter_stats();
    }
}
