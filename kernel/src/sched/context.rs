/// Context Switching for Polymera OS Scheduler
/// 
/// This module implements CPU context management and switching for the kernel scheduler.
/// It provides the core functionality for saving and restoring task execution state.

use core::fmt;

/// CPU execution context for x86_64
/// 
/// This structure contains the minimal set of registers that need to be preserved
/// across context switches. The layout must match the assembly context_switch function.
/// 
/// Register preservation strategy:
/// - Caller-saved registers (rax, rcx, rdx, rsi, rdi, r8-r11): Not preserved (caller's responsibility)
/// - Callee-saved registers (rbx, rbp, r12-r15): Preserved by context switch
/// - Special registers (rip, rsp): Preserved for continuation point and stack
/// - Other registers (flags, segment registers): Handled by interrupt mechanism
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    /// Instruction pointer - where to resume execution
    pub rip: u64,
    
    /// Stack pointer - current stack position
    pub rsp: u64,
    
    /// Callee-saved general purpose registers
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl CpuContext {
    /// Create a new empty CPU context
    /// 
    /// All registers are initialized to zero. This is used for the initial
    /// context creation and as a default state.
    /// 
    /// # Returns
    /// A new CpuContext with all registers set to 0
    pub const fn new() -> Self {
        Self {
            rip: 0,
            rsp: 0,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }
    
    /// Create a CPU context for a new task
    /// 
    /// Sets up the initial context for a task that will start execution
    /// at the specified entry point with the given stack.
    /// 
    /// # Arguments
    /// * `entry_point` - Function address where the task will start execution
    /// * `stack_top` - Top address of the task's stack
    /// 
    /// # Returns
    /// A new CpuContext configured for task startup
    pub fn new_task(entry_point: u64, stack_top: u64) -> Self {
        Self {
            rip: entry_point,
            rsp: stack_top,
            rbx: 0,
            rbp: 0, // Frame pointer starts at 0 for new tasks
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }
    
    /// Create a CPU context from a stack pointer
    /// 
    /// This is used when resuming a task that has been context-switched out.
    /// The stack is assumed to contain the saved context.
    /// 
    /// # Arguments
    /// * `stack_top` - Top of the stack containing saved context
    /// 
    /// # Returns
    /// A CpuContext that will resume from the saved state
    pub fn from_stack(stack_top: u64) -> Self {
        // In a full implementation, we would read the context from the stack
        // For Phase 1, we'll use a simplified approach
        Self {
            rip: 0, // Will be loaded from stack during context switch
            rsp: stack_top,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }
    
    /// Check if this context is valid
    /// 
    /// Performs basic validation to ensure the context contains reasonable values.
    /// 
    /// # Returns
    /// `true` if the context appears valid, `false` otherwise
    pub fn is_valid(&self) -> bool {
        // Basic sanity checks
        
        // Stack pointer should be non-zero and aligned
        if self.rsp == 0 || (self.rsp & 0x7) != 0 {
            return false;
        }
        
        // Instruction pointer should be non-zero for active tasks
        if self.rip == 0 {
            return false;
        }
        
        // Stack pointer should be in a reasonable range (above 1MB)
        if self.rsp < 0x100000 {
            return false;
        }
        
        true
    }
    
    /// Set the instruction pointer
    /// 
    /// # Arguments
    /// * `rip` - New instruction pointer value
    pub fn set_rip(&mut self, rip: u64) {
        self.rip = rip;
    }
    
    /// Set the stack pointer
    /// 
    /// # Arguments
    /// * `rsp` - New stack pointer value
    pub fn set_rsp(&mut self, rsp: u64) {
        self.rsp = rsp;
    }
    
    /// Get the instruction pointer
    /// 
    /// # Returns
    /// Current instruction pointer value
    pub fn get_rip(&self) -> u64 {
        self.rip
    }
    
    /// Get the stack pointer
    /// 
    /// # Returns
    /// Current stack pointer value
    pub fn get_rsp(&self) -> u64 {
        self.rsp
    }
}

impl Default for CpuContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CpuContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CpuContext {{ rip: 0x{:016x}, rsp: 0x{:016x}, rbp: 0x{:016x} }}",
            self.rip, self.rsp, self.rbp
        )
    }
}

/// External assembly function for context switching
/// 
/// This function is implemented in assembly (switch.S) and performs the actual
/// low-level context switch between tasks.
/// 
/// # Arguments
/// * `old` - Mutable pointer to current task's context (will be updated with current state)
/// * `new` - Pointer to new task's context (will be loaded into CPU)
/// 
/// # Safety
/// This function is unsafe because it directly manipulates CPU registers and stack.
/// The caller must ensure that:
/// - Both context pointers are valid
/// - The new context points to a valid execution state
/// - The stack pointed to by new.rsp is valid and properly aligned
extern "C" {
    pub fn context_switch(old: *mut CpuContext, new: *const CpuContext);
}

/// Safe wrapper for context switching
/// 
/// Provides a safe interface for context switching with additional validation
/// and error handling.
/// 
/// # Arguments
/// * `old_context` - Mutable reference to current task's context
/// * `new_context` - Reference to new task's context
/// 
/// # Returns
/// `Ok(())` if context switch was successful, `Err(ContextSwitchError)` otherwise
pub fn safe_context_switch(
    old_context: &mut CpuContext,
    new_context: &CpuContext,
) -> Result<(), ContextSwitchError> {
    // Validate the new context before switching
    if !new_context.is_valid() {
        return Err(ContextSwitchError::InvalidContext);
    }
    
    // Perform the context switch
    unsafe {
        context_switch(old_context as *mut CpuContext, new_context as *const CpuContext);
    }
    
    Ok(())
}

/// Context switch error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextSwitchError {
    /// The target context contains invalid data
    InvalidContext,
    
    /// The stack pointer is invalid or misaligned
    InvalidStack,
    
    /// The instruction pointer is invalid
    InvalidInstructionPointer,
}

impl fmt::Display for ContextSwitchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextSwitchError::InvalidContext => write!(f, "Invalid CPU context"),
            ContextSwitchError::InvalidStack => write!(f, "Invalid stack pointer"),
            ContextSwitchError::InvalidInstructionPointer => write!(f, "Invalid instruction pointer"),
        }
    }
}

/// Context switching statistics for debugging and monitoring
#[derive(Debug, Clone, Copy)]
pub struct ContextSwitchStats {
    /// Total number of context switches performed
    pub total_switches: u64,
    
    /// Number of failed context switch attempts
    pub failed_switches: u64,
    
    /// Average time per context switch (in CPU cycles, if available)
    pub avg_switch_cycles: u64,
}

impl ContextSwitchStats {
    /// Create new context switch statistics
    pub const fn new() -> Self {
        Self {
            total_switches: 0,
            failed_switches: 0,
            avg_switch_cycles: 0,
        }
    }
    
    /// Record a successful context switch
    pub fn record_switch(&mut self, cycles: u64) {
        self.total_switches += 1;
        
        // Update running average (simplified)
        if self.total_switches == 1 {
            self.avg_switch_cycles = cycles;
        } else {
            self.avg_switch_cycles = (self.avg_switch_cycles + cycles) / 2;
        }
    }
    
    /// Record a failed context switch
    pub fn record_failure(&mut self) {
        self.failed_switches += 1;
    }
    
    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f32 {
        let total_attempts = self.total_switches + self.failed_switches;
        if total_attempts == 0 {
            100.0
        } else {
            (self.total_switches as f32 / total_attempts as f32) * 100.0
        }
    }
}

impl fmt::Display for ContextSwitchStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Context Switch Stats: {} successful, {} failed ({:.1}% success rate), avg {} cycles",
            self.total_switches,
            self.failed_switches,
            self.success_rate(),
            self.avg_switch_cycles
        )
    }
}

/// Architecture-specific context operations
/// 
/// This trait provides an interface for architecture-specific context
/// operations that may vary between different CPU architectures.
pub trait ArchContext {
    /// Save floating point state
    fn save_fpu_state(&mut self, fpu_buffer: &mut [u8]);
    
    /// Restore floating point state
    fn restore_fpu_state(&self, fpu_buffer: &[u8]);
    
    /// Get the size of FPU state buffer needed
    fn fpu_state_size() -> usize;
}

/// x86_64-specific context implementation
impl ArchContext for CpuContext {
    fn save_fpu_state(&mut self, _fpu_buffer: &mut [u8]) {
        // TODO: Implement FPU/SSE/AVX state saving
        // This would use FXSAVE or XSAVE instructions
    }
    
    fn restore_fpu_state(&self, _fpu_buffer: &[u8]) {
        // TODO: Implement FPU/SSE/AVX state restoration
        // This would use FXRSTOR or XRSTOR instructions
    }
    
    fn fpu_state_size() -> usize {
        // Standard FXSAVE area is 512 bytes
        // XSAVE can be larger depending on features
        512
    }
}

/// Helper function to create a context for kernel threads
/// 
/// Kernel threads run in kernel space and have simplified context requirements.
/// 
/// # Arguments
/// * `entry_point` - Function address where the kernel thread will start
/// * `stack_top` - Top address of the kernel thread's stack
/// * `arg` - Optional argument to pass to the kernel thread
/// 
/// # Returns
/// A CpuContext configured for a kernel thread
pub fn create_kernel_thread_context(
    entry_point: u64,
    stack_top: u64,
    arg: Option<u64>,
) -> CpuContext {
    let mut context = CpuContext::new_task(entry_point, stack_top);
    
    // Set up argument in RDI (first argument register in x86_64 ABI)
    if let Some(arg_value) = arg {
        context.rbx = arg_value; // Store in preserved register for now
    }
    
    context
}

/// Helper function to create a context for user processes
/// 
/// User processes run in user space and require additional setup for
/// privilege level transitions.
/// 
/// # Arguments
/// * `entry_point` - Function address where the user process will start
/// * `user_stack` - Top address of the user process's stack
/// * `kernel_stack` - Top address of the kernel stack for this process
/// 
/// # Returns
/// A CpuContext configured for a user process
#[allow(dead_code)]
pub fn create_user_process_context(
    entry_point: u64,
    user_stack: u64,
    kernel_stack: u64,
) -> CpuContext {
    // For Phase 1, we'll create a simplified user context
    // In a full implementation, this would set up privilege levels,
    // segment selectors, and user/kernel stack separation
    
    let mut context = CpuContext::new_task(entry_point, kernel_stack);
    
    // Store user stack pointer in a preserved register
    // The actual implementation would use proper stack switching
    context.r12 = user_stack;
    
    context
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_creation() {
        let ctx = CpuContext::new();
        assert_eq!(ctx.rip, 0);
        assert_eq!(ctx.rsp, 0);
        assert_eq!(ctx.rbx, 0);
    }
    
    #[test]
    fn test_new_task_context() {
        let entry = 0x401000;
        let stack = 0x700000;
        let ctx = CpuContext::new_task(entry, stack);
        
        assert_eq!(ctx.rip, entry);
        assert_eq!(ctx.rsp, stack);
        assert_eq!(ctx.rbp, 0);
    }
    
    #[test]
    fn test_context_validation() {
        let mut ctx = CpuContext::new();
        assert!(!ctx.is_valid()); // Empty context is invalid
        
        ctx.rip = 0x401000;
        ctx.rsp = 0x700000;
        assert!(ctx.is_valid()); // Properly initialized context is valid
        
        ctx.rsp = 0x1; // Misaligned stack
        assert!(!ctx.is_valid());
    }
    
    #[test]
    fn test_kernel_thread_context() {
        let entry = 0x401000;
        let stack = 0x700000;
        let arg = Some(0x12345678);
        
        let ctx = create_kernel_thread_context(entry, stack, arg);
        
        assert_eq!(ctx.rip, entry);
        assert_eq!(ctx.rsp, stack);
        assert_eq!(ctx.rbx, 0x12345678);
    }
    
    #[test]
    fn test_context_switch_stats() {
        let mut stats = ContextSwitchStats::new();
        
        assert_eq!(stats.total_switches, 0);
        assert_eq!(stats.success_rate(), 100.0);
        
        stats.record_switch(1000);
        stats.record_switch(1200);
        
        assert_eq!(stats.total_switches, 2);
        assert_eq!(stats.avg_switch_cycles, 1100);
        
        stats.record_failure();
        assert_eq!(stats.success_rate(), 2.0 / 3.0 * 100.0);
    }
}
