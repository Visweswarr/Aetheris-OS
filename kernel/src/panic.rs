/// Enhanced Panic Handler for Polymera OS
/// 
/// Provides comprehensive debugging information including register dumps,
/// control register states, and recent audit trail for post-mortem analysis.

use core::panic::PanicInfo;
use crate::kprintln;
use crate::asm;

/// General purpose register state captured at panic
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GeneralPurposeRegisters {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl GeneralPurposeRegisters {
    /// Create a new GPR structure with all registers set to zero
    pub const fn zero() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
        }
    }
}

/// Control register state captured at panic
#[derive(Debug, Clone, Copy)]
pub struct ControlRegisters {
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cs: u16,
    pub ss: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub rflags: u64,
    pub rip: u64,
}

impl ControlRegisters {
    /// Create a new control register structure with all registers set to zero
    pub const fn zero() -> Self {
        Self {
            cr0: 0, cr2: 0, cr3: 0, cr4: 0,
            cs: 0, ss: 0, ds: 0, es: 0, fs: 0, gs: 0,
            rflags: 0, rip: 0,
        }
    }
}

/// Complete CPU state at time of panic
#[derive(Debug, Clone, Copy)]
pub struct CpuState {
    pub gpr: GeneralPurposeRegisters,
    pub ctrl: ControlRegisters,
    pub timestamp_ms: u64,
    pub panic_count: u32,
}

impl CpuState {
    /// Create a new CPU state structure with all registers set to zero
    pub const fn zero() -> Self {
        Self {
            gpr: GeneralPurposeRegisters::zero(),
            ctrl: ControlRegisters::zero(),
            timestamp_ms: 0,
            panic_count: 0,
        }
    }
}

/// Global panic counter to detect recursive panics
static mut PANIC_COUNT: u32 = 0;

/// Last captured CPU state (for debugging double panics)
static mut LAST_CPU_STATE: CpuState = CpuState::zero();

/// Safe wrapper for reading general purpose registers
/// 
/// Uses inline assembly to capture the current state of all GPRs.
/// This function is marked as unsafe because it uses inline assembly,
/// but the assembly itself is safe as it only reads registers.
unsafe fn read_general_purpose_registers() -> GeneralPurposeRegisters {
    let mut gpr = GeneralPurposeRegisters::zero();
    
    asm!("mov {}, rax", out(reg) gpr.rax);
    asm!("mov {}, rbx", out(reg) gpr.rbx);
    asm!("mov {}, rcx", out(reg) gpr.rcx);
    asm!("mov {}, rdx", out(reg) gpr.rdx);
    asm!("mov {}, rsi", out(reg) gpr.rsi);
    asm!("mov {}, rdi", out(reg) gpr.rdi);
    asm!("mov {}, rbp", out(reg) gpr.rbp);
    asm!("mov {}, rsp", out(reg) gpr.rsp);
    asm!("mov {}, r8", out(reg) gpr.r8);
    asm!("mov {}, r9", out(reg) gpr.r9);
    asm!("mov {}, r10", out(reg) gpr.r10);
    asm!("mov {}, r11", out(reg) gpr.r11);
    asm!("mov {}, r12", out(reg) gpr.r12);
    asm!("mov {}, r13", out(reg) gpr.r13);
    asm!("mov {}, r14", out(reg) gpr.r14);
    asm!("mov {}, r15", out(reg) gpr.r15);
    
    gpr
}

/// Safe wrapper for reading control registers
/// 
/// Uses inline assembly to capture control registers and segment selectors.
/// This function is marked as unsafe because it uses inline assembly.
unsafe fn read_control_registers() -> ControlRegisters {
    let mut ctrl = ControlRegisters::zero();
    
    asm!("mov {}, cr0", out(reg) ctrl.cr0);
    asm!("mov {}, cr2", out(reg) ctrl.cr2);
    asm!("mov {}, cr3", out(reg) ctrl.cr3);
    asm!("mov {}, cr4", out(reg) ctrl.cr4);
    
    let cs: u64; asm!("mov {}, cs", out(reg) cs); ctrl.cs = cs as u16;
    let ss: u64; asm!("mov {}, ss", out(reg) ss); ctrl.ss = ss as u16;
    let ds: u64; asm!("mov {}, ds", out(reg) ds); ctrl.ds = ds as u16;
    let es: u64; asm!("mov {}, es", out(reg) es); ctrl.es = es as u16;
    let fs: u64; asm!("mov {}, fs", out(reg) fs); ctrl.fs = fs as u16;
    let gs: u64; asm!("mov {}, gs", out(reg) gs); ctrl.gs = gs as u16;
    
    asm!("pushfq; pop {}", out(reg) ctrl.rflags);
    asm!("lea {}, [rip]", out(reg) ctrl.rip);
    
    ctrl
}

/// Capture complete CPU state at panic time
unsafe fn capture_cpu_state() -> CpuState {
    let gpr = read_general_purpose_registers();
    let ctrl = read_control_registers();
    let timestamp_ms = crate::log::get_current_time_ms();
    
    CpuState {
        gpr,
        ctrl,
        timestamp_ms,
        panic_count: PANIC_COUNT,
    }
}

/// Print general purpose register dump
fn print_gpr_dump(gpr: &GeneralPurposeRegisters) {
    kprintln!("=== GENERAL PURPOSE REGISTERS ===");
    kprintln!("RAX: 0x{:016x}  RBX: 0x{:016x}  RCX: 0x{:016x}  RDX: 0x{:016x}", 
              gpr.rax, gpr.rbx, gpr.rcx, gpr.rdx);
    kprintln!("RSI: 0x{:016x}  RDI: 0x{:016x}  RBP: 0x{:016x}  RSP: 0x{:016x}", 
              gpr.rsi, gpr.rdi, gpr.rbp, gpr.rsp);
    kprintln!("R8:  0x{:016x}  R9:  0x{:016x}  R10: 0x{:016x}  R11: 0x{:016x}", 
              gpr.r8, gpr.r9, gpr.r10, gpr.r11);
    kprintln!("R12: 0x{:016x}  R13: 0x{:016x}  R14: 0x{:016x}  R15: 0x{:016x}", 
              gpr.r12, gpr.r13, gpr.r14, gpr.r15);
}

/// Print control register dump
fn print_control_dump(ctrl: &ControlRegisters) {
    kprintln!("=== CONTROL REGISTERS ===");
    kprintln!("CR0: 0x{:016x}  CR2: 0x{:016x}  CR3: 0x{:016x}  CR4: 0x{:016x}", 
              ctrl.cr0, ctrl.cr2, ctrl.cr3, ctrl.cr4);
    kprintln!("CS:  0x{:04x}         SS:  0x{:04x}         DS:  0x{:04x}         ES:  0x{:04x}", 
              ctrl.cs, ctrl.ss, ctrl.ds, ctrl.es);
    kprintln!("FS:  0x{:04x}         GS:  0x{:04x}         RFLAGS: 0x{:016x}", 
              ctrl.fs, ctrl.gs, ctrl.rflags);
    kprintln!("RIP: 0x{:016x}", ctrl.rip);
    
    // Decode RFLAGS bits for better readability
    kprintln!("RFLAGS breakdown:");
    if ctrl.rflags & (1 << 0) != 0 { kprintln!("  CF (Carry Flag)"); }
    if ctrl.rflags & (1 << 2) != 0 { kprintln!("  PF (Parity Flag)"); }
    if ctrl.rflags & (1 << 4) != 0 { kprintln!("  AF (Auxiliary Carry Flag)"); }
    if ctrl.rflags & (1 << 6) != 0 { kprintln!("  ZF (Zero Flag)"); }
    if ctrl.rflags & (1 << 7) != 0 { kprintln!("  SF (Sign Flag)"); }
    if ctrl.rflags & (1 << 8) != 0 { kprintln!("  TF (Trap Flag)"); }
    if ctrl.rflags & (1 << 9) != 0 { kprintln!("  IF (Interrupt Flag)"); }
    if ctrl.rflags & (1 << 10) != 0 { kprintln!("  DF (Direction Flag)"); }
    if ctrl.rflags & (1 << 11) != 0 { kprintln!("  OF (Overflow Flag)"); }
    let iopl = (ctrl.rflags >> 12) & 0x3;
    kprintln!("  IOPL: {} (I/O Privilege Level)", iopl);
    if ctrl.rflags & (1 << 14) != 0 { kprintln!("  NT (Nested Task Flag)"); }
    if ctrl.rflags & (1 << 16) != 0 { kprintln!("  RF (Resume Flag)"); }
    if ctrl.rflags & (1 << 17) != 0 { kprintln!("  VM (Virtual 8086 Mode)"); }
    if ctrl.rflags & (1 << 18) != 0 { kprintln!("  AC (Alignment Check)"); }
    if ctrl.rflags & (1 << 19) != 0 { kprintln!("  VIF (Virtual Interrupt Flag)"); }
    if ctrl.rflags & (1 << 20) != 0 { kprintln!("  VIP (Virtual Interrupt Pending)"); }
    if ctrl.rflags & (1 << 21) != 0 { kprintln!("  ID (CPUID Detection Flag)"); }
}

/// Print last N audit entries
fn print_audit_tail(count: usize) {
    kprintln!("=== LAST {} AUDIT ENTRIES ===", count);
    
    // Get audit entries from the security manager
    let entries = crate::secman::audit::get_recent_entries(count);
    
    if entries.is_empty() {
        kprintln!("No audit entries available");
        return;
    }
    
    kprintln!("Format: [timestamp] PID:OP_CODE ARG");
    kprintln!("----------------------------------------");
    
    for (i, entry) in entries.iter().enumerate() {
        let op_name = match entry.op {
            // Syscall operations
            1 => "SYS_YIELD",
            2 => "SYS_EXIT", 
            3 => "SYS_SEND",
            4 => "SYS_RECV",
            5 => "SYS_CHAN_CREATE",
            
            // Security operations
            100 => "SEC_CAP_GRANT",
            101 => "SEC_CAP_REVOKE",
            102 => "SEC_AUTH_FAIL",
            103 => "SEC_ACCESS_DENIED",
            104 => "IPC_DENY_CAP",
            
            // Process operations
            200 => "PROC_CREATE",
            201 => "PROC_DESTROY",
            202 => "PROC_SCHEDULE",
            
            // Memory operations
            300 => "MEM_ALLOC",
            301 => "MEM_FREE",
            302 => "MEM_MAP",
            303 => "MEM_UNMAP",
            
            // System operations
            400 => "SYS_BOOT",
            401 => "SYS_SHUTDOWN",
            402 => "SYS_PANIC",
            
            _ => "UNKNOWN",
        };
        
        kprintln!("[{:>6}] {}:{} 0x{:x} ({})", 
                  entry.ts, entry.pid, entry.op, entry.arg, op_name);
    }
    
    kprintln!("=== END AUDIT TRAIL ===");
}

/// Enhanced stack trace analysis
fn analyze_stack(rsp: u64, rbp: u64) {
    kprintln!("=== STACK ANALYSIS ===");
    kprintln!("Stack Pointer (RSP): 0x{:016x}", rsp);
    kprintln!("Base Pointer (RBP):  0x{:016x}", rbp);
    
    // Basic stack validation
    if rsp == 0 {
        kprintln!("WARNING: Stack pointer is NULL!");
        return;
    }
    
    if rbp == 0 {
        kprintln!("WARNING: Base pointer is NULL!");
        return;
    }
    
    if rsp > rbp {
        kprintln!("WARNING: Stack pointer is above base pointer (stack corruption?)");
    }
    
    let stack_size = rbp.saturating_sub(rsp);
    kprintln!("Current stack frame size: {} bytes", stack_size);
    
    if stack_size > 8192 {
        kprintln!("WARNING: Large stack frame detected ({}KB)", stack_size / 1024);
    }
    
    // TODO: Walk the stack frames when we have better memory management
    kprintln!("Stack frame walking not yet implemented");
}

/// Memory analysis at panic time
fn analyze_memory_state(cr2: u64, cr3: u64) {
    kprintln!("=== MEMORY ANALYSIS ===");
    kprintln!("Page Fault Address (CR2): 0x{:016x}", cr2);
    kprintln!("Page Directory Base (CR3): 0x{:016x}", cr3);
    
    if cr2 != 0 {
        kprintln!("Last page fault occurred at address: 0x{:016x}", cr2);
        
        // Analyze the faulting address
        if cr2 < 0x1000 {
            kprintln!("  -> NULL pointer dereference or very low address");
        } else if cr2 >= 0xFFFF800000000000 {
            kprintln!("  -> Kernel space address");
        } else if cr2 >= 0x400000 {
            kprintln!("  -> User space address");
        } else {
            kprintln!("  -> Low user space address");
        }
        
        // Check alignment
        if cr2 % 8 != 0 {
            kprintln!("  -> Unaligned access (not 8-byte aligned)");
        }
        if cr2 % 4096 == 0 {
            kprintln!("  -> Page boundary access");
        }
    } else {
        kprintln!("No recent page fault");
    }
}

/// System state analysis
fn analyze_system_state(cpu_state: &CpuState) {
    kprintln!("=== SYSTEM STATE ANALYSIS ===");
    kprintln!("Panic timestamp: {}ms", cpu_state.timestamp_ms);
    kprintln!("Panic count: {}", cpu_state.panic_count);
    
    if cpu_state.panic_count > 1 {
        kprintln!("WARNING: Multiple panics detected! (Double/Triple fault risk)");
    }
    
    // Analyze interrupt state
    if cpu_state.ctrl.rflags & (1 << 9) != 0 {
        kprintln!("Interrupts: ENABLED");
    } else {
        kprintln!("Interrupts: DISABLED");
    }
    
    // Get current task information if available
    let current_task = crate::sched::get_current_task_id();
    if current_task != 0 {
        kprintln!("Current task ID: {}", current_task);
    } else {
        kprintln!("No current task (idle or early boot)");
    }
    
    // System uptime
    kprintln!("System uptime: {}ms", cpu_state.timestamp_ms);
    
    // Rate limiting statistics
    let (active_entries, suppressions) = crate::log::get_rate_limit_stats();
    kprintln!("Log rate limiter: {} active, {} suppressions", active_entries, suppressions);
}

/// Main panic handler with comprehensive debugging
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts immediately to prevent further issues
    unsafe {
        asm!("cli", options(preserves_flags, nostack));
    }
    
    // Increment panic counter and check for recursive panics
    unsafe {
        PANIC_COUNT += 1;
        
        if PANIC_COUNT > 3 {
            // Triple panic - just halt immediately
            kprintln!("TRIPLE PANIC DETECTED - IMMEDIATE HALT");
            loop {
                asm!("hlt", options(preserves_flags, nostack));
            }
        }
        
        if PANIC_COUNT > 1 {
            kprintln!("DOUBLE PANIC DETECTED - MINIMAL OUTPUT");
            kprintln!("Previous panic state was:");
            print_gpr_dump(&LAST_CPU_STATE.gpr);
            kprintln!("Current panic: {}", info);
            loop {
                asm!("hlt", options(preserves_flags, nostack));
            }
        }
    }
    
    // Log panic to audit trail
    crate::secman::audit::log(crate::secman::audit::AuditEntry::new(
        0, // System operation
        402, // SYS_PANIC
        unsafe { PANIC_COUNT as u64 }
    ));
    
    // Capture CPU state before we potentially corrupt anything
    let cpu_state = unsafe { capture_cpu_state() };
    unsafe {
        LAST_CPU_STATE = cpu_state;
    }
    
    // Write minidump to fixed buffer for post-mortem analysis
    if let Err(e) = write_minidump_to_buffer(&cpu_state, info) {
        kprintln!("[MINIDUMP] Failed to write minidump: {}", e);
    } else {
        kprintln!("[MINIDUMP] Successfully written to buffer");
    }
    
    // Print the panic banner
    kprintln!("");
    kprintln!("██████╗  █████╗ ███╗   ██╗██╗ ██████╗");
    kprintln!("██╔══██╗██╔══██╗████╗  ██║██║██╔════╝");
    kprintln!("██████╔╝███████║██╔██╗ ██║██║██║     ");
    kprintln!("██╔═══╝ ██╔══██║██║╚██╗██║██║██║     ");
    kprintln!("██║     ██║  ██║██║ ╚████║██║╚██████╗");
    kprintln!("╚═╝     ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝ ╚═════╝");
    kprintln!("");
    kprintln!("🚨 POLYMERA OS KERNEL PANIC 🚨");
    kprintln!("Panic #{} at timestamp {}ms", cpu_state.panic_count, cpu_state.timestamp_ms);
    kprintln!("");
    
    // Print panic information
    kprintln!("=== PANIC INFORMATION ===");
    kprintln!("{}", info);
    kprintln!("");
    
    // Print register dumps
    print_gpr_dump(&cpu_state.gpr);
    kprintln!("");
    print_control_dump(&cpu_state.ctrl);
    kprintln!("");
    
    // Analyze stack state
    analyze_stack(cpu_state.gpr.rsp, cpu_state.gpr.rbp);
    kprintln!("");
    
    // Analyze memory state
    analyze_memory_state(cpu_state.ctrl.cr2, cpu_state.ctrl.cr3);
    kprintln!("");
    
    // Analyze system state
    analyze_system_state(&cpu_state);
    kprintln!("");
    
    // Print recent audit trail
    print_audit_tail(32);
    kprintln!("");
    
    // Final halt message
    kprintln!("=== SYSTEM HALTED ===");
    kprintln!("The system has encountered a fatal error and must stop.");
    kprintln!("Please review the above information for debugging.");
    kprintln!("System will now halt indefinitely.");
    kprintln!("");
    
    // Halt the system
    loop {
        unsafe {
            asm!("hlt", options(preserves_flags, nostack));
        }
    }
}

/// Public function to trigger a test panic (for testing purposes)
pub fn test_panic_handler() {
    panic!("Test panic triggered for debugging verification");
}

/// Get the current panic count (for debugging)
pub fn get_panic_count() -> u32 {
    unsafe { PANIC_COUNT }
}

/// Reset panic count (for testing)
pub unsafe fn reset_panic_count() {
    PANIC_COUNT = 0;
}

/// Write minidump to fixed buffer for post-mortem analysis
fn write_minidump_to_buffer(cpu_state: &CpuState, panic_info: &PanicInfo) -> Result<(), &'static str> {
    use crate::crash_dump::minidump::{CpuRegisters, write_minidump};
    
    // Convert panic CPU state to minidump CPU registers
    let mut cpu_regs = CpuRegisters::new();
    
    // Copy general purpose registers
    cpu_regs.rax = cpu_state.gpr.rax;
    cpu_regs.rbx = cpu_state.gpr.rbx;
    cpu_regs.rcx = cpu_state.gpr.rcx;
    cpu_regs.rdx = cpu_state.gpr.rdx;
    cpu_regs.rsi = cpu_state.gpr.rsi;
    cpu_regs.rdi = cpu_state.gpr.rdi;
    cpu_regs.rbp = cpu_state.gpr.rbp;
    cpu_regs.rsp = cpu_state.gpr.rsp;
    cpu_regs.r8 = cpu_state.gpr.r8;
    cpu_regs.r9 = cpu_state.gpr.r9;
    cpu_regs.r10 = cpu_state.gpr.r10;
    cpu_regs.r11 = cpu_state.gpr.r11;
    cpu_regs.r12 = cpu_state.gpr.r12;
    cpu_regs.r13 = cpu_state.gpr.r13;
    cpu_regs.r14 = cpu_state.gpr.r14;
    cpu_regs.r15 = cpu_state.gpr.r15;
    
    // Copy control registers
    cpu_regs.rip = cpu_state.ctrl.rip;
    cpu_regs.rflags = cpu_state.ctrl.rflags;
    cpu_regs.cs = cpu_state.ctrl.cs as u64;
    cpu_regs.ss = cpu_state.ctrl.ss as u64;
    cpu_regs.ds = cpu_state.ctrl.ds as u64;
    cpu_regs.es = cpu_state.ctrl.es as u64;
    cpu_regs.fs = cpu_state.ctrl.fs as u64;
    cpu_regs.gs = cpu_state.ctrl.gs as u64;
    cpu_regs.cr0 = cpu_state.ctrl.cr0;
    cpu_regs.cr2 = cpu_state.ctrl.cr2;
    cpu_regs.cr3 = cpu_state.ctrl.cr3;
    cpu_regs.cr4 = cpu_state.ctrl.cr4;
    
    // Create a simple stack trace (just the current frame for now)
    let stack_trace = [cpu_state.ctrl.rip, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    
    // Write the minidump
    write_minidump(
        &cpu_regs,
        panic_info,
        &stack_trace,
        None,
        &[],
        &[],
        &crate::crash_dump::CpuFeatures::new(),
    )
}

