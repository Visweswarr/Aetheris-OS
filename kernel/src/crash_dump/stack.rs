#![no_std]

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub address: u64,
    pub function_name: Option<String>,
    pub module_name: Option<String>,
    pub offset: u64,
    pub is_kernel: bool,
    pub is_valid: bool,
}

impl StackFrame {
    /// Create new stack frame
    pub fn new(address: u64) -> Self {
        Self {
            address,
            function_name: None,
            module_name: None,
            offset: 0,
            is_kernel: true, // Assume kernel for now
            is_valid: true,
        }
    }

    /// Analyze stack frame for validity
    pub fn analyze(&mut self) {
        // Check if address is in valid kernel range
        self.is_valid = self.address >= 0xffff800000000000 && self.address <= 0xffffffffffffffff;
        
        // Try to resolve function name (simplified for now)
        if self.is_valid {
            self.function_name = self.resolve_function_name();
            self.module_name = Some("kernel".to_string());
            self.offset = self.address & 0xfff; // Page offset
        }
    }

    /// Resolve function name from address
    fn resolve_function_name(&self) -> Option<String> {
        // TODO: Implement proper symbol resolution
        // For now, return a placeholder
        Some(format!("unknown_function_0x{:x}", self.address))
    }
}

impl fmt::Display for StackFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid {
            write!(f, "0x{:016x} in {} (+0x{:x})", 
                self.address,
                self.function_name.as_ref().unwrap_or(&"unknown".to_string()),
                self.offset
            )
        } else {
            write!(f, "0x{:016x} <invalid>", self.address)
        }
    }
}

/// Stack trace with analysis
#[derive(Debug)]
pub struct StackTrace {
    pub frames: Vec<StackFrame>,
    pub max_depth: usize,
    pub is_complete: bool,
    pub analysis: StackAnalysis,
}

/// Stack analysis results
#[derive(Debug, Clone)]
pub struct StackAnalysis {
    pub total_frames: usize,
    pub valid_frames: usize,
    pub kernel_frames: usize,
    pub user_frames: usize,
    pub suspicious_patterns: Vec<String>,
    pub stack_overflow_suspected: bool,
    pub corruption_suspected: bool,
}

impl StackAnalysis {
    /// Create new stack analysis
    pub fn new() -> Self {
        Self {
            total_frames: 0,
            valid_frames: 0,
            kernel_frames: 0,
            user_frames: 0,
            suspicious_patterns: Vec::new(),
            stack_overflow_suspected: false,
            corruption_suspected: false,
        }
    }

    /// Analyze stack trace for suspicious patterns
    pub fn analyze_patterns(&mut self, frames: &[StackFrame]) {
        self.total_frames = frames.len();
        self.valid_frames = frames.iter().filter(|f| f.is_valid).count();
        self.kernel_frames = frames.iter().filter(|f| f.is_kernel).count();
        self.user_frames = frames.iter().filter(|f| !f.is_kernel).count();

        // Check for suspicious patterns
        if frames.len() > 100 {
            self.suspicious_patterns.push("Excessive stack depth (>100 frames)".to_string());
        }

        // Check for repeated addresses (potential corruption)
        let mut addresses = Vec::new();
        for frame in frames {
            addresses.push(frame.address);
        }
        addresses.sort();
        addresses.dedup();
        
        if addresses.len() < frames.len() * 3 / 4 {
            self.suspicious_patterns.push("Repeated addresses detected (potential corruption)".to_string());
            self.corruption_suspected = true;
        }

        // Check for stack overflow patterns
        let mut consecutive_invalid = 0;
        for frame in frames {
            if !frame.is_valid {
                consecutive_invalid += 1;
                if consecutive_invalid > 5 {
                    self.stack_overflow_suspected = true;
                    self.suspicious_patterns.push("Stack overflow suspected".to_string());
                    break;
                }
            } else {
                consecutive_invalid = 0;
            }
        }

        // Check for suspicious address ranges
        for frame in frames {
            if frame.address == 0 {
                self.suspicious_patterns.push("Null pointer in stack trace".to_string());
            } else if frame.address < 0x1000 {
                self.suspicious_patterns.push("Address in low memory range".to_string());
            } else if frame.address > 0x7fffffffffff && frame.address < 0xffff800000000000 {
                self.suspicious_patterns.push("Address in invalid range".to_string());
            }
        }
    }
}

impl StackTrace {
    /// Capture stack trace up to specified depth
    pub fn capture(max_depth: usize) -> Self {
        let mut frames = Vec::new();
        let mut current_rbp: u64;
        
        unsafe {
            // Get current RBP
            asm!("mov {}, rbp", out(reg) current_rbp);
        }

        let mut depth = 0;
        while depth < max_depth && current_rbp != 0 {
            // Read return address from stack
            let return_address = unsafe { *(current_rbp as *const u64).add(1) };
            
            if return_address == 0 || return_address < 0x1000 {
                break;
            }

            let mut frame = StackFrame::new(return_address);
            frame.analyze();
            frames.push(frame);

            // Move to previous frame
            current_rbp = unsafe { *(current_rbp as *const u64) };
            depth += 1;
        }

        let mut analysis = StackAnalysis::new();
        analysis.analyze_patterns(&frames);

        Self {
            frames,
            max_depth,
            is_complete: depth < max_depth,
            analysis,
        }
    }

    /// Print stack trace
    pub fn print(&self) {
        crate::kprintln!("║   Stack Trace ({} frames, max depth: {}):", self.frames.len(), self.max_depth);
        
        if self.frames.is_empty() {
            crate::kprintln!("║     <empty or invalid>");
            return;
        }

        for (i, frame) in self.frames.iter().enumerate() {
            let marker = if frame.is_valid { " " } else { "⚠️" };
            crate::kprintln!("║     #{:2} {} {}", i, marker, frame);
        }

        // Print analysis
        if !self.analysis.suspicious_patterns.is_empty() {
            crate::kprintln!("║   Analysis:");
            for pattern in &self.analysis.suspicious_patterns {
                crate::kprintln!("║     ⚠️  {}", pattern);
            }
        }

        crate::kprintln!("║   Summary: {} valid frames, {} kernel frames, {} user frames", 
            self.analysis.valid_frames, self.analysis.kernel_frames, self.analysis.user_frames);
        
        if self.analysis.stack_overflow_suspected {
            crate::kprintln!("║     🚨 STACK OVERFLOW SUSPECTED");
        }
        
        if self.analysis.corruption_suspected {
            crate::kprintln!("║     🚨 STACK CORRUPTION SUSPECTED");
        }
    }

    /// Generate stack trace report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("Stack Trace ({} frames, max depth: {}):\n", self.frames.len(), self.max_depth));
        
        if self.frames.is_empty() {
            report.push_str("  <empty or invalid>\n");
            return report;
        }

        for (i, frame) in self.frames.iter().enumerate() {
            let marker = if frame.is_valid { " " } else { "⚠️" };
            report.push_str(&format!("  #{:2} {} {}\n", i, marker, frame));
        }

        // Add analysis
        if !self.analysis.suspicious_patterns.is_empty() {
            report.push_str("Analysis:\n");
            for pattern in &self.analysis.suspicious_patterns {
                report.push_str(&format!("  ⚠️  {}\n", pattern));
            }
        }

        report.push_str(&format!("Summary: {} valid frames, {} kernel frames, {} user frames\n", 
            self.analysis.valid_frames, self.analysis.kernel_frames, self.analysis.user_frames));
        
        if self.analysis.stack_overflow_suspected {
            report.push_str("  🚨 STACK OVERFLOW SUSPECTED\n");
        }
        
        if self.analysis.corruption_suspected {
            report.push_str("  🚨 STACK CORRUPTION SUSPECTED\n");
        }
        
        report
    }

    /// Get stack trace as vector of addresses
    pub fn get_addresses(&self) -> Vec<u64> {
        self.frames.iter().map(|f| f.address).collect()
    }

    /// Check if stack trace contains specific address
    pub fn contains_address(&self, address: u64) -> bool {
        self.frames.iter().any(|f| f.address == address)
    }

    /// Get frames in specific address range
    pub fn get_frames_in_range(&self, start: u64, end: u64) -> Vec<&StackFrame> {
        self.frames.iter()
            .filter(|f| f.address >= start && f.address <= end)
            .collect()
    }
}

/// Stack trace utilities
pub struct StackTraceUtils;

impl StackTraceUtils {
    /// Get current stack pointer
    pub fn get_current_sp() -> u64 {
        let mut sp: u64;
        unsafe {
            asm!("mov {}, rsp", out(reg) sp);
        }
        sp
    }

    /// Get current base pointer
    pub fn get_current_bp() -> u64 {
        let mut bp: u64;
        unsafe {
            asm!("mov {}, rbp", out(reg) bp);
        }
        bp
    }

    /// Get current instruction pointer
    pub fn get_current_ip() -> u64 {
        let mut ip: u64;
        unsafe {
            asm!("lea {}, [rip]", out(reg) ip);
        }
        ip
    }

    /// Check if address is in kernel space
    pub fn is_kernel_address(address: u64) -> bool {
        address >= 0xffff800000000000
    }

    /// Check if address is in user space
    pub fn is_user_address(address: u64) -> bool {
        address < 0x7fffffffffff
    }

    /// Check if address is valid
    pub fn is_valid_address(address: u64) -> bool {
        address != 0 && address >= 0x1000 && 
        (Self::is_kernel_address(address) || Self::is_user_address(address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_frame_creation() {
        let frame = StackFrame::new(0x1000);
        assert_eq!(frame.address, 0x1000);
        assert!(frame.is_valid);
    }

    #[test]
    fn test_stack_frame_analysis() {
        let mut frame = StackFrame::new(0xffff800000001000);
        frame.analyze();
        assert!(frame.is_valid);
        assert!(frame.is_kernel);
    }

    #[test]
    fn test_stack_analysis() {
        let mut analysis = StackAnalysis::new();
        let frames = vec![
            StackFrame::new(0xffff800000001000),
            StackFrame::new(0xffff800000002000),
        ];
        
        analysis.analyze_patterns(&frames);
        assert_eq!(analysis.total_frames, 2);
        assert_eq!(analysis.valid_frames, 2);
    }

    #[test]
    fn test_stack_trace_utils() {
        assert!(StackTraceUtils::is_kernel_address(0xffff800000000000));
        assert!(StackTraceUtils::is_user_address(0x1000));
        assert!(StackTraceUtils::is_valid_address(0x1000));
        assert!(!StackTraceUtils::is_valid_address(0));
    }
}
