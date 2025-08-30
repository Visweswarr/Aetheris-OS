#![no_std]

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;

pub mod registers;
pub mod stack;
pub mod memory;
pub mod context;
pub mod minidump;

pub use registers::*;
pub use stack::*;
pub use memory::*;
pub use context::*;
pub use minidump::*;

/// Enhanced crash dump configuration
#[derive(Debug, Clone)]
pub struct CrashDumpConfig {
    pub include_registers: bool,
    pub include_stack_trace: bool,
    pub include_memory_state: bool,
    pub include_context: bool,
    pub max_stack_depth: usize,
    pub memory_dump_size: usize,
    pub preserve_context: bool,
}

impl Default for CrashDumpConfig {
    fn default() -> Self {
        Self {
            include_registers: true,
            include_stack_trace: true,
            include_memory_state: true,
            include_context: true,
            max_stack_depth: 32,
            memory_dump_size: 1024,
            preserve_context: true,
        }
    }
}

/// Comprehensive crash dump information
#[derive(Debug)]
pub struct CrashDump {
    pub timestamp: u64,
    pub task_id: Option<u64>,
    pub error_type: CrashErrorType,
    pub registers: Option<RegisterDump>,
    pub stack_trace: Option<StackTrace>,
    pub memory_state: Option<MemoryState>,
    pub context: Option<TaskContext>,
    pub config: CrashDumpConfig,
}

/// Types of crash errors
#[derive(Debug, Clone)]
pub enum CrashErrorType {
    Panic,
    PageFault { address: u64, error_code: u32 },
    DoubleFault { error_code: u32 },
    GeneralProtectionFault { error_code: u32 },
    InvalidOpcode,
    StackSegmentFault { error_code: u32 },
    SegmentNotPresent { error_code: u32 },
    StackFault { error_code: u32 },
    AlignmentCheck { error_code: u32 },
    MachineCheck { error_code: u32 },
    SIMDFloatingPointException { error_code: u32 },
    VirtualizationException { error_code: u32 },
    SecurityException { error_code: u32 },
    Unknown { code: u32 },
}

impl fmt::Display for CrashErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrashErrorType::Panic => write!(f, "Kernel Panic"),
            CrashErrorType::PageFault { address, error_code } => {
                write!(f, "Page Fault at 0x{:016x} (error: 0x{:08x})", address, error_code)
            }
            CrashErrorType::DoubleFault { error_code } => {
                write!(f, "Double Fault (error: 0x{:08x})", error_code)
            }
            CrashErrorType::GeneralProtectionFault { error_code } => {
                write!(f, "General Protection Fault (error: 0x{:08x})", error_code)
            }
            CrashErrorType::InvalidOpcode => write!(f, "Invalid Opcode"),
            CrashErrorType::StackSegmentFault { error_code } => {
                write!(f, "Stack Segment Fault (error: 0x{:08x})", error_code)
            }
            CrashErrorType::SegmentNotPresent { error_code } => {
                write!(f, "Segment Not Present (error: 0x{:08x})", error_code)
            }
            CrashErrorType::StackFault { error_code } => {
                write!(f, "Stack Fault (error: 0x{:08x})", error_code)
            }
            CrashErrorType::AlignmentCheck { error_code } => {
                write!(f, "Alignment Check (error: 0x{:08x})", error_code)
            }
            CrashErrorType::MachineCheck { error_code } => {
                write!(f, "Machine Check (error: 0x{:08x})", error_code)
            }
            CrashErrorType::SIMDFloatingPointException { error_code } => {
                write!(f, "SIMD Floating Point Exception (error: 0x{:08x})", error_code)
            }
            CrashErrorType::VirtualizationException { error_code } => {
                write!(f, "Virtualization Exception (error: 0x{:08x})", error_code)
            }
            CrashErrorType::SecurityException { error_code } => {
                write!(f, "Security Exception (error: 0x{:08x})", error_code)
            }
            CrashErrorType::Unknown { code } => write!(f, "Unknown Exception (code: 0x{:08x})", code),
        }
    }
}

impl CrashDump {
    /// Create a new crash dump with default configuration
    pub fn new(error_type: CrashErrorType) -> Self {
        Self {
            timestamp: crate::log::get_current_time_ms(),
            task_id: crate::sched::get_current_task_id(),
            error_type,
            registers: None,
            stack_trace: None,
            memory_state: None,
            context: None,
            config: CrashDumpConfig::default(),
        }
    }

    /// Create a crash dump with custom configuration
    pub fn with_config(error_type: CrashErrorType, config: CrashDumpConfig) -> Self {
        Self {
            timestamp: crate::log::get_current_time_ms(),
            task_id: crate::sched::get_current_task_id(),
            error_type,
            registers: None,
            stack_trace: None,
            memory_state: None,
            context: None,
            config,
        }
    }

    /// Collect comprehensive crash information
    pub fn collect(&mut self) -> Result<(), &'static str> {
        if self.config.include_registers {
            self.registers = Some(RegisterDump::capture());
        }

        if self.config.include_stack_trace {
            self.stack_trace = Some(StackTrace::capture(self.config.max_stack_depth));
        }

        if self.config.include_memory_state {
            self.memory_state = Some(MemoryState::capture(self.config.memory_dump_size));
        }

        if self.config.include_context {
            if let Some(task_id) = self.task_id {
                self.context = Some(TaskContext::capture(task_id));
            }
        }

        Ok(())
    }

    /// Print comprehensive crash dump
    pub fn print(&self) {
        crate::kprintln!("");
        crate::kprintln!("╔══════════════════════════════════════════════════════════════════════════════╗");
        crate::kprintln!("║                              CRASH DUMP                                   ║");
        crate::kprintln!("╠══════════════════════════════════════════════════════════════════════════════╣");
        crate::kprintln!("║ Timestamp: {} ms", self.timestamp);
        crate::kprintln!("║ Task ID:   {}", self.task_id.map(|id| id.to_string()).unwrap_or_else(|| "None".to_string()));
        crate::kprintln!("║ Error:     {}", self.error_type);
        crate::kprintln!("╠══════════════════════════════════════════════════════════════════════════════╣");

        if let Some(ref registers) = self.registers {
            crate::kprintln!("║ REGISTERS:");
            registers.print();
        }

        if let Some(ref stack_trace) = self.stack_trace {
            crate::kprintln!("║ STACK TRACE:");
            stack_trace.print();
        }

        if let Some(ref memory_state) = self.memory_state {
            crate::kprintln!("║ MEMORY STATE:");
            memory_state.print();
        }

        if let Some(ref context) = self.context {
            crate::kprintln!("║ TASK CONTEXT:");
            context.print();
        }

        crate::kprintln!("╚══════════════════════════════════════════════════════════════════════════════╝");
        crate::kprintln!("");
    }

    /// Generate crash dump report as string
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== CRASH DUMP ===\n");
        report.push_str(&format!("Timestamp: {} ms\n", self.timestamp));
        report.push_str(&format!("Task ID: {}\n", self.task_id.map(|id| id.to_string()).unwrap_or_else(|| "None".to_string())));
        report.push_str(&format!("Error: {}\n", self.error_type));
        report.push_str("\n");

        if let Some(ref registers) = self.registers {
            report.push_str("=== REGISTERS ===\n");
            report.push_str(&registers.generate_report());
            report.push_str("\n");
        }

        if let Some(ref stack_trace) = self.stack_trace {
            report.push_str("=== STACK TRACE ===\n");
            report.push_str(&stack_trace.generate_report());
            report.push_str("\n");
        }

        if let Some(ref memory_state) = self.memory_state {
            report.push_str("=== MEMORY STATE ===\n");
            report.push_str(&memory_state.generate_report());
            report.push_str("\n");
        }

        if let Some(ref context) = self.context {
            report.push_str("=== TASK CONTEXT ===\n");
            report.push_str(&context.generate_report());
            report.push_str("\n");
        }

        report
    }

    /// Save crash dump to persistent storage
    pub fn save(&self) -> Result<(), &'static str> {
        let report = self.generate_report();
        let filename = format!("crash_dump_{}.txt", self.timestamp);
        
        // TODO: Implement persistent storage
        // For now, just log the report
        crate::klog!(crate::log::tags::CRASH, "Saving crash dump to {}", filename);
        
        Ok(())
    }
}

/// Global crash dump manager
pub struct CrashDumpManager {
    config: CrashDumpConfig,
    crash_count: u64,
    last_crash: Option<CrashDump>,
}

impl CrashDumpManager {
    /// Create new crash dump manager
    pub fn new(config: CrashDumpConfig) -> Self {
        Self {
            config,
            crash_count: 0,
            last_crash: None,
        }
    }

    /// Handle a crash with comprehensive dumping
    pub fn handle_crash(&mut self, error_type: CrashErrorType) -> ! {
        self.crash_count += 1;
        
        let mut crash_dump = CrashDump::with_config(error_type, self.config.clone());
        
        if let Err(e) = crash_dump.collect() {
            crate::klog!(crate::log::tags::CRASH, "Failed to collect crash dump: {}", e);
        }

        crash_dump.print();
        
        if self.config.preserve_context {
            self.last_crash = Some(crash_dump);
        }

        // Halt the system
        crate::macros::halt_system();
    }

    /// Get crash statistics
    pub fn get_stats(&self) -> CrashStats {
        CrashStats {
            total_crashes: self.crash_count,
            last_crash_timestamp: self.last_crash.as_ref().map(|c| c.timestamp),
        }
    }
}

/// Crash statistics
#[derive(Debug, Clone)]
pub struct CrashStats {
    pub total_crashes: u64,
    pub last_crash_timestamp: Option<u64>,
}

/// Initialize crash dump system
pub fn init() {
    let config = CrashDumpConfig::default();
    let manager = CrashDumpManager::new(config);
    
    // TODO: Store manager in global state
    crate::klog!(crate::log::tags::INIT, "Crash dump system initialized");
}

/// Handle crash with enhanced dumping
pub fn handle_crash(error_type: CrashErrorType) -> ! {
    let config = CrashDumpConfig::default();
    let mut manager = CrashDumpManager::new(config);
    manager.handle_crash(error_type);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_dump_creation() {
        let crash_dump = CrashDump::new(CrashErrorType::Panic);
        assert_eq!(crash_dump.error_type.to_string(), "Kernel Panic");
        assert!(crash_dump.task_id.is_some());
    }

    #[test]
    fn test_crash_error_types() {
        let page_fault = CrashErrorType::PageFault { address: 0x1000, error_code: 0x2 };
        assert!(page_fault.to_string().contains("Page Fault"));
        assert!(page_fault.to_string().contains("0x1000"));
    }

    #[test]
    fn test_crash_dump_config() {
        let config = CrashDumpConfig::default();
        assert!(config.include_registers);
        assert!(config.include_stack_trace);
        assert_eq!(config.max_stack_depth, 32);
    }
}
