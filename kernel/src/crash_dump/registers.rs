#![no_std]

use core::fmt;
use alloc::string::String;

/// x86_64 register state
#[repr(C)]
#[derive(Debug, Clone)]
pub struct RegisterState {
    // General purpose registers
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
    
    // Instruction pointer and flags
    pub rip: u64,
    pub rflags: u64,
    
    // Segment registers
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub fs: u16,
    pub gs: u16,
    pub ss: u16,
    
    // Control registers
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    
    // Debug registers
    pub dr0: u64,
    pub dr1: u64,
    pub dr2: u64,
    pub dr3: u64,
    pub dr6: u64,
    pub dr7: u64,
    
    // Model specific registers
    pub msr_efer: u64,
    pub msr_star: u64,
    pub msr_lstar: u64,
    pub msr_cstar: u64,
    pub msr_sfmask: u64,
}

impl RegisterState {
    /// Create empty register state
    pub fn new() -> Self {
        Self {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rflags: 0,
            cs: 0, ds: 0, es: 0, fs: 0, gs: 0, ss: 0,
            cr0: 0, cr2: 0, cr3: 0, cr4: 0, cr8: 0,
            dr0: 0, dr1: 0, dr2: 0, dr3: 0, dr6: 0, dr7: 0,
            msr_efer: 0, msr_star: 0, msr_lstar: 0, msr_cstar: 0, msr_sfmask: 0,
        }
    }

    /// Capture current register state
    pub fn capture() -> Self {
        let mut state = Self::new();
        
        // Capture general purpose registers
        unsafe {
            asm!(
                "mov {}, rax", out(reg) state.rax,
                "mov {}, rbx", out(reg) state.rbx,
                "mov {}, rcx", out(reg) state.rcx,
                "mov {}, rdx", out(reg) state.rdx,
                "mov {}, rsi", out(reg) state.rsi,
                "mov {}, rdi", out(reg) state.rdi,
                "mov {}, rbp", out(reg) state.rbp,
                "mov {}, rsp", out(reg) state.rsp,
                "mov {}, r8", out(reg) state.r8,
                "mov {}, r9", out(reg) state.r9,
                "mov {}, r10", out(reg) state.r10,
                "mov {}, r11", out(reg) state.r11,
                "mov {}, r12", out(reg) state.r12,
                "mov {}, r13", out(reg) state.r13,
                "mov {}, r14", out(reg) state.r14,
                "mov {}, r15", out(reg) state.r15,
                "lea {}, [rip]", out(reg) state.rip,
                "pushfq; pop {}", out(reg) state.rflags,
                "mov {}, cs", out(reg) state.cs,
                "mov {}, ds", out(reg) state.ds,
                "mov {}, es", out(reg) state.es,
                "mov {}, fs", out(reg) state.fs,
                "mov {}, gs", out(reg) state.gs,
                "mov {}, ss", out(reg) state.ss,
                "mov {}, cr0", out(reg) state.cr0,
                "mov {}, cr2", out(reg) state.cr2,
                "mov {}, cr3", out(reg) state.cr3,
                "mov {}, cr4", out(reg) state.cr4,
                "mov {}, cr8", out(reg) state.cr8,
            );
        }
        
        state
    }

    /// Get RFLAGS breakdown
    pub fn get_rflags_breakdown(&self) -> RFlagsBreakdown {
        RFlagsBreakdown {
            cf: (self.rflags & 0x1) != 0,
            pf: (self.rflags & 0x4) != 0,
            af: (self.rflags & 0x10) != 0,
            zf: (self.rflags & 0x40) != 0,
            sf: (self.rflags & 0x80) != 0,
            tf: (self.rflags & 0x100) != 0,
            if_: (self.rflags & 0x200) != 0,
            df: (self.rflags & 0x400) != 0,
            of: (self.rflags & 0x800) != 0,
            iopl: ((self.rflags >> 12) & 0x3) as u8,
            nt: (self.rflags & 0x4000) != 0,
            rf: (self.rflags & 0x10000) != 0,
            vm: (self.rflags & 0x20000) != 0,
            ac: (self.rflags & 0x40000) != 0,
            vif: (self.rflags & 0x80000) != 0,
            vip: (self.rflags & 0x100000) != 0,
            id: (self.rflags & 0x200000) != 0,
        }
    }

    /// Check if register value looks suspicious
    pub fn analyze_suspicious_values(&self) -> Vec<String> {
        let mut suspicious = Vec::new();
        
        // Check for null pointers
        if self.rax == 0 { suspicious.push("RAX is null pointer".to_string()); }
        if self.rbx == 0 { suspicious.push("RBX is null pointer".to_string()); }
        if self.rcx == 0 { suspicious.push("RCX is null pointer".to_string()); }
        if self.rdx == 0 { suspicious.push("RDX is null pointer".to_string()); }
        
        // Check for invalid RIP
        if self.rip < 0x1000 { suspicious.push("RIP in low memory".to_string()); }
        if self.rip > 0x7fffffffffff { suspicious.push("RIP in invalid range".to_string()); }
        
        // Check for invalid RSP
        if self.rsp < 0x1000 { suspicious.push("RSP in low memory".to_string()); }
        if self.rsp > 0x7fffffffffff { suspicious.push("RSP in invalid range".to_string()); }
        
        // Check for invalid segment registers
        if self.cs == 0 { suspicious.push("CS is null".to_string()); }
        if self.ss == 0 { suspicious.push("SS is null".to_string()); }
        
        suspicious
    }
}

/// RFLAGS register breakdown
#[derive(Debug, Clone)]
pub struct RFlagsBreakdown {
    pub cf: bool,   // Carry Flag
    pub pf: bool,   // Parity Flag
    pub af: bool,   // Auxiliary Carry Flag
    pub zf: bool,   // Zero Flag
    pub sf: bool,   // Sign Flag
    pub tf: bool,   // Trap Flag
    pub if_: bool,  // Interrupt Enable Flag
    pub df: bool,   // Direction Flag
    pub of: bool,   // Overflow Flag
    pub iopl: u8,   // I/O Privilege Level
    pub nt: bool,   // Nested Task
    pub rf: bool,   // Resume Flag
    pub vm: bool,   // Virtual 8086 Mode
    pub ac: bool,   // Alignment Check
    pub vif: bool,  // Virtual Interrupt Flag
    pub vip: bool,  // Virtual Interrupt Pending
    pub id: bool,   // ID Flag
}

impl fmt::Display for RFlagsBreakdown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CF={} PF={} AF={} ZF={} SF={} TF={} IF={} DF={} OF={} IOPL={} NT={} RF={} VM={} AC={} VIF={} VIP={} ID={}",
            if self.cf { "1" } else { "0" },
            if self.pf { "1" } else { "0" },
            if self.af { "1" } else { "0" },
            if self.zf { "1" } else { "0" },
            if self.sf { "1" } else { "0" },
            if self.tf { "1" } else { "0" },
            if self.if_ { "1" } else { "0" },
            if self.df { "1" } else { "0" },
            if self.of { "1" } else { "0" },
            self.iopl,
            if self.nt { "1" } else { "0" },
            if self.rf { "1" } else { "0" },
            if self.vm { "1" } else { "0" },
            if self.ac { "1" } else { "0" },
            if self.vif { "1" } else { "0" },
            if self.vip { "1" } else { "0" },
            if self.id { "1" } else { "0" }
        )
    }
}

/// Register dump with analysis
#[derive(Debug)]
pub struct RegisterDump {
    pub state: RegisterState,
    pub suspicious_values: Vec<String>,
}

impl RegisterDump {
    /// Capture current register state
    pub fn capture() -> Self {
        let state = RegisterState::capture();
        let suspicious_values = state.analyze_suspicious_values();
        
        Self {
            state,
            suspicious_values,
        }
    }

    /// Print register dump
    pub fn print(&self) {
        let state = &self.state;
        let rflags = state.get_rflags_breakdown();
        
        crate::kprintln!("║   General Purpose Registers:");
        crate::kprintln!("║     RAX: 0x{:016x}  RBX: 0x{:016x}  RCX: 0x{:016x}  RDX: 0x{:016x}", 
            state.rax, state.rbx, state.rcx, state.rdx);
        crate::kprintln!("║     RSI: 0x{:016x}  RDI: 0x{:016x}  RBP: 0x{:016x}  RSP: 0x{:016x}", 
            state.rsi, state.rdi, state.rbp, state.rsp);
        crate::kprintln!("║     R8:  0x{:016x}  R9:  0x{:016x}  R10: 0x{:016x}  R11: 0x{:016x}", 
            state.r8, state.r9, state.r10, state.r11);
        crate::kprintln!("║     R12: 0x{:016x}  R13: 0x{:016x}  R14: 0x{:016x}  R15: 0x{:016x}", 
            state.r12, state.r13, state.r14, state.r15);
        
        crate::kprintln!("║   Instruction Pointer:");
        crate::kprintln!("║     RIP: 0x{:016x}", state.rip);
        
        crate::kprintln!("║   Flags Register:");
        crate::kprintln!("║     RFLAGS: 0x{:016x} ({})", state.rflags, rflags);
        
        crate::kprintln!("║   Segment Registers:");
        crate::kprintln!("║     CS: 0x{:04x}  DS: 0x{:04x}  ES: 0x{:04x}  FS: 0x{:04x}  GS: 0x{:04x}  SS: 0x{:04x}", 
            state.cs, state.ds, state.es, state.fs, state.gs, state.ss);
        
        crate::kprintln!("║   Control Registers:");
        crate::kprintln!("║     CR0: 0x{:016x}  CR2: 0x{:016x}  CR3: 0x{:016x}  CR4: 0x{:016x}  CR8: 0x{:016x}", 
            state.cr0, state.cr2, state.cr3, state.cr4, state.cr8);
        
        if !self.suspicious_values.is_empty() {
            crate::kprintln!("║   Suspicious Values:");
            for value in &self.suspicious_values {
                crate::kprintln!("║     ⚠️  {}", value);
            }
        }
    }

    /// Generate register dump report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        let state = &self.state;
        let rflags = state.get_rflags_breakdown();
        
        report.push_str(&format!("General Purpose Registers:\n"));
        report.push_str(&format!("  RAX: 0x{:016x}  RBX: 0x{:016x}  RCX: 0x{:016x}  RDX: 0x{:016x}\n", 
            state.rax, state.rbx, state.rcx, state.rdx));
        report.push_str(&format!("  RSI: 0x{:016x}  RDI: 0x{:016x}  RBP: 0x{:016x}  RSP: 0x{:016x}\n", 
            state.rsi, state.rdi, state.rbp, state.rsp));
        report.push_str(&format!("  R8:  0x{:016x}  R9:  0x{:016x}  R10: 0x{:016x}  R11: 0x{:016x}\n", 
            state.r8, state.r9, state.r10, state.r11));
        report.push_str(&format!("  R12: 0x{:016x}  R13: 0x{:016x}  R14: 0x{:016x}  R15: 0x{:016x}\n", 
            state.r12, state.r13, state.r14, state.r15));
        
        report.push_str(&format!("Instruction Pointer:\n"));
        report.push_str(&format!("  RIP: 0x{:016x}\n", state.rip));
        
        report.push_str(&format!("Flags Register:\n"));
        report.push_str(&format!("  RFLAGS: 0x{:016x} ({})\n", state.rflags, rflags));
        
        report.push_str(&format!("Segment Registers:\n"));
        report.push_str(&format!("  CS: 0x{:04x}  DS: 0x{:04x}  ES: 0x{:04x}  FS: 0x{:04x}  GS: 0x{:04x}  SS: 0x{:04x}\n", 
            state.cs, state.ds, state.es, state.fs, state.gs, state.ss));
        
        report.push_str(&format!("Control Registers:\n"));
        report.push_str(&format!("  CR0: 0x{:016x}  CR2: 0x{:016x}  CR3: 0x{:016x}  CR4: 0x{:016x}  CR8: 0x{:016x}\n", 
            state.cr0, state.cr2, state.cr3, state.cr4, state.cr8));
        
        if !self.suspicious_values.is_empty() {
            report.push_str("Suspicious Values:\n");
            for value in &self.suspicious_values {
                report.push_str(&format!("  ⚠️  {}\n", value));
            }
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_state_creation() {
        let state = RegisterState::new();
        assert_eq!(state.rax, 0);
        assert_eq!(state.rip, 0);
    }

    #[test]
    fn test_rflags_breakdown() {
        let mut state = RegisterState::new();
        state.rflags = 0x246; // Set some flags
        
        let breakdown = state.get_rflags_breakdown();
        assert_eq!(breakdown.cf, false);
        assert_eq!(breakdown.zf, true);
        assert_eq!(breakdown.sf, true);
    }

    #[test]
    fn test_suspicious_value_detection() {
        let mut state = RegisterState::new();
        state.rax = 0; // Null pointer
        state.rip = 0x500; // Low memory
        
        let suspicious = state.analyze_suspicious_values();
        assert!(suspicious.len() >= 2);
        assert!(suspicious.iter().any(|s| s.contains("RAX is null pointer")));
        assert!(suspicious.iter().any(|s| s.contains("RIP in low memory")));
    }
}
