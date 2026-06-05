use core::mem::size_of;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use crate::kprintln;

/// Size of the double fault stack
pub const DOUBLE_FAULT_STACK_SIZE: usize = 4096; // 4KB stack

/// Double fault stack - must be aligned to 16 bytes
#[repr(align(16))]
#[repr(C)]
pub struct DoubleFaultStack {
    pub data: [u8; DOUBLE_FAULT_STACK_SIZE],
}

impl DoubleFaultStack {
    /// Create a new double fault stack
    pub const fn new() -> Self {
        Self {
            data: [0; DOUBLE_FAULT_STACK_SIZE],
        }
    }

    /// Get the top of the stack (stack grows downward)
    pub fn top(&self) -> VirtAddr {
        let stack_top = self as *const _ as usize + DOUBLE_FAULT_STACK_SIZE;
        VirtAddr::new(stack_top as u64)
    }
}

/// Global double fault stack instance
pub static DOUBLE_FAULT_STACK: DoubleFaultStack = DoubleFaultStack::new();

/// Task State Segment for the kernel
/// 
/// This TSS contains the Interrupt Stack Table (IST) which provides
/// dedicated stacks for exception handlers, particularly the double fault handler.
pub static mut TSS: TaskStateSegment = TaskStateSegment::new();

/// Initialize the Task State Segment
/// 
/// Sets up the IST entries, particularly IST[1] for double fault handling.
/// This ensures that double faults have a dedicated stack and cannot cause
/// a triple fault due to stack corruption.
pub fn init() {
    kprintln!("[HAL] TSS init - setting up Interrupt Stack Table");
    
    unsafe {
        // Set up the double fault stack (IST[1])
        // IST[0] is reserved, IST[1] is for double fault
        TSS.interrupt_stack_table[1] = DOUBLE_FAULT_STACK.top();
        
        kprintln!("[HAL] TSS: Double fault stack at 0x{:016x}", DOUBLE_FAULT_STACK.top().as_u64());
        
        // Set up other IST entries if needed in the future
        // IST[2] could be for NMI, IST[3] for debug, etc.
        
        // Verify TSS is properly aligned
        let tss_ptr = &TSS as *const _ as usize;
        assert!(tss_ptr % 16 == 0, "TSS must be 16-byte aligned");
        
        kprintln!("[HAL] TSS initialized successfully");
    }
}

/// Get the TSS selector for loading into the CPU
pub fn get_tss_selector() -> u16 {
    // This will be set when we add the TSS descriptor to the GDT
    // For now, return a placeholder
    0x28 // Common TSS selector value
}

/// Test the TSS setup
#[cfg(test)]
pub fn test_tss_setup() {
    kprintln!("[TEST] Testing TSS setup...");
    
    // Verify double fault stack is properly aligned
    let stack_ptr = &DOUBLE_FAULT_STACK as *const _ as usize;
    assert!(stack_ptr % 16 == 0, "Double fault stack must be 16-byte aligned");
    
    // Verify stack size
    assert_eq!(DOUBLE_FAULT_STACK.data.len(), DOUBLE_FAULT_STACK_SIZE);
    
    // Verify stack top calculation
    let expected_top = stack_ptr + DOUBLE_FAULT_STACK_SIZE;
    assert_eq!(DOUBLE_FAULT_STACK.top().as_usize(), expected_top);
    
    kprintln!("[TEST] TSS setup test PASSED");
}



