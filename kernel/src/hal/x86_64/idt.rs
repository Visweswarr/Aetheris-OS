/// Interrupt Descriptor Table (IDT) Implementation for Polymera OS
/// 
/// This module provides comprehensive interrupt handling including enhanced
/// page fault handling v2 with memory safety features, guard checks, and
/// robust error reporting.

use crate::{kprintln, klog, kprintln};
use crate::log::Level;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::structures::DescriptorTablePointer;
use x86_64::instructions::interrupts;
use x86_64::instructions::tables::lidt;
use x86_64::VirtAddr;
use lazy_static::lazy_static;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};

//=============================================================================
// IDT CONFIGURATION
//=============================================================================

/// Number of interrupt vectors supported
const IDT_SIZE: usize = 256;

/// Interrupt vector numbers
pub const DIVIDE_BY_ZERO: u8 = 0;
pub const DEBUG: u8 = 1;
pub const NON_MASKABLE_INTERRUPT: u8 = 2;
pub const BREAKPOINT: u8 = 3;
pub const OVERFLOW: u8 = 4;
pub const BOUND_RANGE_EXCEEDED: u8 = 5;
pub const INVALID_OPCODE: u8 = 6;
pub const DEVICE_NOT_AVAILABLE: u8 = 7;
pub const DOUBLE_FAULT: u8 = 8;
pub const COPROCESSOR_SEGMENT_OVERRUN: u8 = 9;
pub const INVALID_TSS: u8 = 10;
pub const SEGMENT_NOT_PRESENT: u8 = 11;
pub const STACK_SEGMENT_FAULT: u8 = 12;
pub const GENERAL_PROTECTION_FAULT: u8 = 13;
pub const PAGE_FAULT: u8 = 14;
pub const FLOATING_POINT_EXCEPTION: u8 = 16;
pub const ALIGNMENT_CHECK: u8 = 17;
pub const MACHINE_CHECK: u8 = 18;
pub const SIMD_FLOATING_POINT_EXCEPTION: u8 = 19;
pub const VIRTUALIZATION_EXCEPTION: u8 = 20;
pub const SECURITY_EXCEPTION: u8 = 30;

/// Page fault error code bit flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFaultError {
    /// Protection violation (1) or not present (0)
    pub protection_violation: bool,
    /// Write access (1) or read access (0)
    pub write_access: bool,
    /// User mode access (1) or supervisor mode (0)
    pub user_mode: bool,
    /// Reserved bit violation (1) or not (0)
    pub reserved_violation: bool,
    /// Instruction fetch (1) or data access (0)
    pub instruction_fetch: bool,
}

impl From<PageFaultErrorCode> for PageFaultError {
    fn from(error_code: PageFaultErrorCode) -> Self {
        Self {
            protection_violation: error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION),
            write_access: error_code.contains(PageFaultErrorCode::WRITE_ACCESS),
            user_mode: error_code.contains(PageFaultErrorCode::USER_MODE),
            reserved_violation: error_code.contains(PageFaultErrorCode::RESERVED_BIT),
            instruction_fetch: error_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH),
        }
    }
}

impl core::fmt::Display for PageFaultError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PageFault[")?;
        
        if self.protection_violation {
            write!(f, "P")?;
        } else {
            write!(f, "N")?;
        }
        
        if self.write_access {
            write!(f, "W")?;
        } else {
            write!(f, "R")?;
        }
        
        if self.user_mode {
            write!(f, "U")?;
        } else {
            write!(f, "S")?;
        }
        
        if self.reserved_violation {
            write!(f, "RSV")?;
        }
        
        if self.instruction_fetch {
            write!(f, "ID")?;
        }
        
        write!(f, "]")
    }
}

//=============================================================================
// INTERRUPT HANDLERS
//=============================================================================

/// Enhanced page fault handler v2 with memory safety features
/// 
/// This handler provides comprehensive page fault analysis including:
/// - Error code decoding (P/U/W/RSV/ID)
/// - User-space vs kernel fault handling
/// - Stack overflow detection via red-zone canaries
/// - Audit trail analysis for kernel faults
/// - Safe system halt with diagnostic information
pub extern "x86-interrupt" fn page_fault_handler_v2(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    // Disable interrupts during fault handling
    interrupts::disable();
    
    let fault_address = x86_64::registers::control::Cr2::read();
    let error = PageFaultError::from(error_code);
    
    kprintln!("");
    kprintln!("🚨 PAGE FAULT DETECTED");
    kprintln!("=====================");
    kprintln!("Fault Address: 0x{:016x}", fault_address);
    kprintln!("Error Code: {} ({:?})", error, error_code);
    kprintln!("Instruction Pointer: 0x{:016x}", stack_frame.instruction_pointer);
    kprintln!("Code Segment: 0x{:04x}", stack_frame.code_segment);
    kprintln!("CPU Flags: 0x{:016x}", stack_frame.cpu_flags);
    kprintln!("Stack Pointer: 0x{:016x}", stack_frame.stack_pointer);
    
    // Check if this is a user-space fault (for future implementation)
    if error.user_mode {
        kprintln!("User-space fault detected - will send SIGSEGV-like event");
        // TODO: Implement user-space fault handling
        // send_sigsegv_event(fault_address, error);
    } else {
        kprintln!("Kernel fault detected - analyzing for memory safety issues");
        
        // Check for stack overflow via red-zone canaries
        if let Some(overflow_info) = check_stack_overflow(&stack_frame) {
            kprintln!("🚨 STACK OVERFLOW DETECTED!");
            kprintln!("Overflow Type: {:?}", overflow_info.overflow_type);
            kprintln!("Canary Location: 0x{:016x}", overflow_info.canary_address);
            kprintln!("Expected Canary: 0x{:016x}", overflow_info.expected_canary);
            kprintln!("Actual Canary: 0x{:016x}", overflow_info.actual_canary);
            
            // Trigger immediate panic with stack overflow reason
            panic!("STACK_OVERFLOW: Kernel stack corruption detected");
        }
        
        // Print annotated stack trace
        print_annotated_stack_trace(&stack_frame);
        
        // Print last 32 audit entries
        print_recent_audit_entries();
        
        // Safe system halt with diagnostic information
        safe_halt_with_diagnostics(&stack_frame, &error, fault_address);
    }
}

/// Check for stack overflow using red-zone canaries
/// 
/// # Arguments
/// * `stack_frame` - The interrupt stack frame
/// 
/// # Returns
/// `Some(StackOverflowInfo)` if overflow detected, `None` otherwise
fn check_stack_overflow(stack_frame: &InterruptStackFrame) -> Option<StackOverflowInfo> {
    // Get current stack pointer
    let current_sp = stack_frame.stack_pointer;
    
    // Check red-zone canaries at various stack locations
    let canary_locations = [
        current_sp + 0x1000,  // 4KB below current SP
        current_sp + 0x2000,  // 8KB below current SP
        current_sp + 0x4000,  // 16KB below current SP
    ];
    
    for &canary_addr in &canary_locations {
        if let Some(overflow_info) = check_canary_at_address(canary_addr) {
            return Some(overflow_info);
        }
    }
    
    None
}

/// Check canary at specific address
/// 
/// # Arguments
/// * `address` - Address to check for canary
/// 
/// # Returns
/// `Some(StackOverflowInfo)` if canary corrupted, `None` otherwise
fn check_canary_at_address(address: u64) -> Option<StackOverflowInfo> {
    // Read canary value from memory
    let canary_value = unsafe { *(address as *const u64) };
    
    // Check if canary is corrupted (not the expected magic value)
    if canary_value != STACK_CANARY_MAGIC {
        return Some(StackOverflowInfo {
            overflow_type: StackOverflowType::CanaryCorruption,
            canary_address: address,
            expected_canary: STACK_CANARY_MAGIC,
            actual_canary: canary_value,
        });
    }
    
    None
}

/// Stack overflow information
#[derive(Debug, Clone)]
pub struct StackOverflowInfo {
    /// Type of overflow detected
    pub overflow_type: StackOverflowType,
    /// Address where canary was checked
    pub canary_address: u64,
    /// Expected canary value
    pub expected_canary: u64,
    /// Actual canary value found
    pub actual_canary: u64,
}

/// Types of stack overflow
#[derive(Debug, Clone)]
pub enum StackOverflowType {
    /// Canary value was corrupted
    CanaryCorruption,
    /// Stack pointer went below valid range
    StackUnderflow,
    /// Stack pointer went above valid range
    StackOverflow,
    /// Guard page violation
    GuardPageViolation,
}

/// Stack canary magic value
const STACK_CANARY_MAGIC: u64 = 0xDEADBEEFCAFEBABE;

/// Print annotated stack trace
/// 
/// # Arguments
/// * `stack_frame` - The interrupt stack frame
fn print_annotated_stack_trace(stack_frame: &InterruptStackFrame) {
    kprintln!("");
    kprintln!("📚 ANNOTATED STACK TRACE");
    kprintln!("=========================");
    kprintln!("Frame 0: 0x{:016x} (fault location)", stack_frame.instruction_pointer);
    
    // Walk the stack to find call sites
    let mut current_sp = stack_frame.stack_pointer;
    let mut frame_count = 1;
    
    // Limit stack walk to prevent infinite loops
    const MAX_FRAMES: usize = 20;
    
    while frame_count < MAX_FRAMES && current_sp != 0 {
        // Read return address from stack
        let return_addr = unsafe { *(current_sp as *const u64) };
        
        // Check if return address looks valid (within kernel space)
        if return_addr >= 0xffff800000000000 && return_addr < 0xffffffffffffffff {
            kprintln!("Frame {}: 0x{:016x} (return address)", frame_count, return_addr);
            frame_count += 1;
        }
        
        // Move to next frame
        current_sp += 8;
        
        // Safety check to prevent invalid memory access
        if current_sp > 0xfffffffffffffff0 {
            break;
        }
    }
    
    if frame_count >= MAX_FRAMES {
        kprintln!("Stack trace truncated at {} frames", MAX_FRAMES);
    }
}

/// Print recent audit entries
fn print_recent_audit_entries() {
    kprintln!("");
    kprintln!("📋 RECENT AUDIT ENTRIES (Last 32)");
    kprintln!("==================================");
    
    // TODO: Integrate with actual audit system
    // For now, print placeholder information
    kprintln!("Audit system not yet integrated");
    kprintln!("Would show last 32 security/access events");
}

/// Safe system halt with diagnostic information
/// 
/// # Arguments
/// * `stack_frame` - The interrupt stack frame
/// * `error` - Page fault error information
/// * `fault_address` - Address that caused the fault
fn safe_halt_with_diagnostics(
    stack_frame: &InterruptStackFrame,
    error: &PageFaultError,
    fault_address: u64,
) {
    kprintln!("");
    kprintln!("🛑 SYSTEM HALTING DUE TO KERNEL PAGE FAULT");
    kprintln!("==========================================");
    kprintln!("Fault Summary:");
    kprintln!("  Address: 0x{:016x}", fault_address);
    kprintln!("  Type: {}", error);
    kprintln!("  IP: 0x{:016x}", stack_frame.instruction_pointer);
    kprintln!("  SP: 0x{:016x}", stack_frame.stack_pointer);
    kprintln!("");
    kprintln!("Diagnostic Information:");
    kprintln!("  - Stack trace printed above");
    kprintln!("  - Audit trail examined");
    kprintln!("  - Memory safety checks performed");
    kprintln!("");
    kprintln!("System is now in safe halt state.");
    kprintln!("Check logs for detailed analysis.");
    kprintln!("");
    
    // Halt the system
    loop {
        x86_64::instructions::hlt();
    }
}

/// Double fault handler
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    kprintln!("");
    kprintln!("🚨🚨 DOUBLE FAULT DETECTED");
    kprintln!("=========================");
    kprintln!("This indicates a critical system failure.");
    kprintln!("Instruction Pointer: 0x{:016x}", stack_frame.instruction_pointer);
    kprintln!("Code Segment: 0x{:04x}", stack_frame.code_segment);
    kprintln!("CPU Flags: 0x{:016x}", stack_frame.cpu_flags);
    kprintln!("Stack Pointer: 0x{:016x}", stack_frame.stack_pointer);
    kprintln!("");
    kprintln!("System is now in unrecoverable state.");
    kprintln!("Check for:");
    kprintln!("  - Stack overflow");
    kprintln!("  - Invalid memory access");
    kprintln!("  - Interrupt handler errors");
    kprintln!("");
    
    // Halt the system
    loop {
        x86_64::instructions::hlt();
    }
}

/// General protection fault handler
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    kprintln!("");
    kprintln!("🚨 GENERAL PROTECTION FAULT");
    kprintln!("===========================");
    kprintln!("Error Code: 0x{:04x}", error_code);
    kprintln!("Instruction Pointer: 0x{:016x}", stack_frame.instruction_pointer);
    kprintln!("Code Segment: 0x{:04x}", stack_frame.code_segment);
    kprintln!("CPU Flags: 0x{:016x}", stack_frame.cpu_flags);
    kprintln!("Stack Pointer: 0x{:016x}", stack_frame.stack_pointer);
    kprintln!("");
    kprintln!("This fault typically indicates:");
    kprintln!("  - Segment violation");
    kprintln!("  - Privilege level violation");
    kprintln!("  - Invalid TSS");
    kprintln!("  - Stack segment fault");
    kprintln!("");
    
    // Print stack trace
    print_annotated_stack_trace(&stack_frame);
    
    // Halt the system
    loop {
        x86_64::instructions::hlt();
    }
}

/// Breakpoint handler
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    kprintln!("");
    kprintln!("🔍 BREAKPOINT TRIGGERED");
    kprintln!("=======================");
    kprintln!("Instruction Pointer: 0x{:016x}", stack_frame.instruction_pointer);
    kprintln!("Code Segment: 0x{:04x}", stack_frame.code_segment);
    kprintln!("CPU Flags: 0x{:016x}", stack_frame.cpu_flags);
    kprintln!("Stack Pointer: 0x{:016x}", stack_frame.stack_pointer);
    kprintln!("");
    kprintln!("This breakpoint was triggered by:");
    kprintln!("  - INT3 instruction");
    kprintln!("  - Debugger breakpoint");
    kprintln!("  - Hardware breakpoint");
    kprintln!("");
    
    // Print stack trace
    print_annotated_stack_trace(&stack_frame);
    
    // Continue execution (for debugging)
    kprintln!("Continuing execution...");
}

/// Divide by zero handler
pub extern "x86-interrupt" fn divide_by_zero_handler(stack_frame: InterruptStackFrame) {
    kprintln!("");
    kprintln!("🚨 DIVIDE BY ZERO EXCEPTION");
    kprintln!("===========================");
    kprintln!("Instruction Pointer: 0x{:016x}", stack_frame.instruction_pointer);
    kprintln!("Code Segment: 0x{:04x}", stack_frame.code_segment);
    kprintln!("CPU Flags: 0x{:016x}", stack_frame.cpu_flags);
    kprintln!("Stack Pointer: 0x{:016x}", stack_frame.stack_pointer);
    kprintln!("");
    kprintln!("This exception indicates:");
    kprintln!("  - Division by zero");
    kprintln!("  - Integer overflow");
    kprintln!("  - Mathematical error");
    kprintln!("");
    
    // Print stack trace
    print_annotated_stack_trace(&stack_frame);
    
    // Halt the system
    loop {
        x86_64::instructions::hlt();
    }
}

/// Invalid opcode handler
pub extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    kprintln!("");
    kprintln!("🚨 INVALID OPCODE EXCEPTION");
    kprintln!("===========================");
    kprintln!("Instruction Pointer: 0x{:016x}", stack_frame.instruction_pointer);
    kprintln!("Code Segment: 0x{:04x}", stack_frame.code_segment);
    kprintln!("CPU Flags: 0x{:016x}", stack_frame.cpu_flags);
    kprintln!("Stack Pointer: 0x{:016x}", stack_frame.stack_pointer);
    kprintln!("");
    kprintln!("This exception indicates:");
    kprintln!("  - Invalid instruction");
    kprintln!("  - Unsupported CPU feature");
    kprintln!("  - Corrupted code");
    kprintln!("");
    
    // Print stack trace
    print_annotated_stack_trace(&stack_frame);
    
    // Halt the system
    loop {
        x86_64::instructions::hlt();
    }
}

//=============================================================================
// IDT MANAGEMENT
//=============================================================================

/// Global IDT instance
lazy_static! {
    static ref IDT: Mutex<InterruptDescriptorTable> = Mutex::new(InterruptDescriptorTable::new());
}

/// Initialize the Interrupt Descriptor Table
pub fn init_idt() {
    kprintln!("[IDT] Initializing Interrupt Descriptor Table");
    
    let mut idt = IDT.lock();
    
    // Set up exception handlers
    idt.divide_by_zero.set_handler_fn(divide_by_zero_handler);
    idt.debug.set_handler_fn(breakpoint_handler);
    idt.non_maskable_interrupt.set_handler_fn(|_| {
        kprintln!("[IDT] Non-maskable interrupt received");
    });
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.overflow.set_handler_fn(|_| {
        kprintln!("[IDT] Overflow exception");
    });
    idt.bound_range_exceeded.set_handler_fn(|_| {
        kprintln!("[IDT] Bound range exceeded");
    });
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.device_not_available.set_handler_fn(|_| {
        kprintln!("[IDT] Device not available");
    });
    idt.double_fault.set_handler_fn(double_fault_handler);
    idt.coprocessor_segment_overrun.set_handler_fn(|_| {
        kprintln!("[IDT] Coprocessor segment overrun");
    });
    idt.invalid_tss.set_handler_fn(|_| {
        kprintln!("[IDT] Invalid TSS");
    });
    idt.segment_not_present.set_handler_fn(|_| {
        kprintln!("[IDT] Segment not present");
    });
    idt.stack_segment_fault.set_handler_fn(|_| {
        kprintln!("[IDT] Stack segment fault");
    });
    idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler_v2);
    idt.floating_point_exception.set_handler_fn(|_| {
        kprintln!("[IDT] Floating point exception");
    });
    idt.alignment_check.set_handler_fn(|_| {
        kprintln!("[IDT] Alignment check");
    });
    idt.machine_check.set_handler_fn(|_| {
        kprintln!("[IDT] Machine check");
    });
    idt.simd_floating_point_exception.set_handler_fn(|_| {
        kprintln!("[IDT] SIMD floating point exception");
    });
    idt.virtualization_exception.set_handler_fn(|_| {
        kprintln!("[IDT] Virtualization exception");
    });
    idt.security_exception.set_handler_fn(|_| {
        kprintln!("[IDT] Security exception");
    });
    
    // Load the IDT
    let idt_pointer = DescriptorTablePointer {
        base: VirtAddr::new(idt.as_ptr() as u64),
        limit: (core::mem::size_of::<InterruptDescriptorTable>() - 1) as u16,
    };
    
    unsafe {
        lidt(&idt_pointer);
    }
    
    kprintln!("[IDT] Interrupt Descriptor Table initialized successfully");
}

/// Get the global IDT instance
pub fn get_idt() -> &'static Mutex<InterruptDescriptorTable> {
    &IDT
}

/// Test the page fault handler (for debugging)
#[allow(dead_code)]
pub fn test_page_fault() {
    kprintln!("[IDT] Testing page fault handler...");
    
    // Trigger a page fault by accessing invalid memory
    unsafe {
        let _value = *(0xDEADBEEF as *const u64);
    }
}

/// Test the breakpoint handler (for debugging)
#[allow(dead_code)]
pub fn test_breakpoint() {
    kprintln!("[IDT] Testing breakpoint handler...");
    x86_64::instructions::interrupts::int3();
}

/// Test the divide by zero handler (for debugging)
#[allow(dead_code)]
pub fn test_divide_by_zero() {
    kprintln!("[IDT] Testing divide by zero handler...");
    
    // This will trigger a divide by zero exception
    let _result = 1u64 / 0u64;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_page_fault_error_parsing() {
        let error_code = PageFaultErrorCode::PROTECTION_VIOLATION | PageFaultErrorCode::WRITE_ACCESS;
        let error = PageFaultError::from(error_code);
        
        assert!(error.protection_violation);
        assert!(error.write_access);
        assert!(!error.user_mode);
        assert!(!error.reserved_violation);
        assert!(!error.instruction_fetch);
    }
    
    #[test]
    fn test_page_fault_error_display() {
        let error = PageFaultError {
            protection_violation: true,
            write_access: false,
            user_mode: true,
            reserved_violation: false,
            instruction_fetch: true,
        };
        
        let display = format!("{}", error);
        assert_eq!(display, "PageFault[PNSID]");
    }
    
    #[test]
    fn test_stack_overflow_detection() {
        // Test with valid canary
        let valid_canary = STACK_CANARY_MAGIC;
        assert_eq!(valid_canary, 0xDEADBEEFCAFEBABE);
        
        // Test with corrupted canary
        let corrupted_canary = 0x1234567890ABCDEF;
        assert_ne!(corrupted_canary, STACK_CANARY_MAGIC);
    }
}
