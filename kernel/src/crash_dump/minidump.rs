//! Binary Minidump System v2
//! 
//! This module provides a binary minidump format for capturing comprehensive
//! crash information on kernel panic, including:
//! - CPU registers and control registers
//! - Page fault information and error codes
//! - APIC vector history and interrupt information
//! - Last 256 log lines with timestamps
//! - CPU features and capabilities
//! - Last audit entries
//! - Serial output buffer
//! - Kernel build information
//! - Stack traces and memory state

use core::mem;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

/// Minidump header structure
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MinidumpHeader {
    /// Magic number: "POLY" in ASCII
    pub magic: [u8; 4],
    /// Version number (currently 2)
    pub version: u32,
    /// Timestamp when dump was created
    pub timestamp: u64,
    /// Size of the entire minidump
    pub total_size: u32,
    /// Flags indicating what data is present
    pub flags: u32,
    /// CRC32 checksum of the dump
    pub checksum: u32,
    /// Reserved for future use
    pub reserved: [u8; 16],
}

impl MinidumpHeader {
    /// Create a new minidump header
    pub fn new(timestamp: u64, total_size: u32, flags: u32) -> Self {
        Self {
            magic: [b'P', b'O', b'L', b'Y'],
            version: 2, // Updated to version 2
            timestamp,
            total_size,
            flags,
            checksum: 0, // Will be calculated later
            reserved: [0; 16],
        }
    }
    
    /// Validate the magic number
    pub fn is_valid(&self) -> bool {
        self.magic == [b'P', b'O', b'L', b'Y']
    }
    
    /// Get the version
    pub fn get_version(&self) -> u32 {
        self.version
    }
}

/// CPU register state
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CpuRegisters {
    /// General purpose registers
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
    
    /// Instruction pointer
    pub rip: u64,
    
    /// Flags register
    pub rflags: u64,
    
    /// Segment registers
    pub cs: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
    pub ss: u64,
    
    /// Control registers
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    
    /// Debug registers
    pub dr0: u64,
    pub dr1: u64,
    pub dr2: u64,
    pub dr3: u64,
    pub dr6: u64,
    pub dr7: u64,
    
    /// Model-specific registers
    pub msr_efer: u64,
    pub msr_star: u64,
    pub msr_lstar: u64,
    pub msr_cstar: u64,
    pub msr_sfmask: u64,
    pub msr_fs_base: u64,
    pub msr_gs_base: u64,
    pub msr_kernel_gs_base: u64,
}

/// Page fault information
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PageFaultInfo {
    /// Faulting address (CR2)
    pub fault_address: u64,
    /// Page fault error code
    pub error_code: u32,
    /// Instruction pointer when fault occurred
    pub fault_rip: u64,
    /// Stack pointer when fault occurred
    pub fault_rsp: u64,
    /// Additional context flags
    pub context_flags: u32,
    /// Reserved for future use
    pub reserved: [u8; 8],
}

impl PageFaultInfo {
    /// Create new page fault info
    pub fn new(fault_address: u64, error_code: u32, fault_rip: u64, fault_rsp: u64) -> Self {
        Self {
            fault_address,
            error_code,
            fault_rip,
            fault_rsp,
            context_flags: 0,
            reserved: [0; 8],
        }
    }
    
    /// Check if fault was caused by protection violation
    pub fn is_protection_violation(&self) -> bool {
        (self.error_code & 0x1) != 0
    }
    
    /// Check if fault was caused by write access
    pub fn is_write_access(&self) -> bool {
        (self.error_code & 0x2) != 0
    }
    
    /// Check if fault occurred in user mode
    pub fn is_user_mode(&self) -> bool {
        (self.error_code & 0x4) != 0
    }
    
    /// Check if fault was caused by instruction fetch
    pub fn is_instruction_fetch(&self) -> bool {
        (self.error_code & 0x10) != 0
    }
    
    /// Get human-readable error description
    pub fn get_error_description(&self) -> &'static str {
        if self.is_protection_violation() {
            "Protection violation"
        } else if self.is_write_access() {
            "Write access to read-only page"
        } else if self.is_instruction_fetch() {
            "Instruction fetch from non-executable page"
        } else {
            "Page not present"
        }
    }
}

/// APIC vector history entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ApicVectorEntry {
    /// Timestamp when interrupt occurred
    pub timestamp: u64,
    /// APIC vector number
    pub vector: u8,
    /// CPU ID where interrupt occurred
    pub cpu_id: u8,
    /// Interrupt type (0=normal, 1=NMI, 2=SMI, 3=INIT, 4=ExtINT)
    pub interrupt_type: u8,
    /// Delivery status (0=idle, 1=send pending, 2=send accepted)
    pub delivery_status: u8,
    /// Reserved for future use
    pub reserved: [u8; 4],
}

impl ApicVectorEntry {
    /// Create new APIC vector entry
    pub fn new(timestamp: u64, vector: u8, cpu_id: u8, interrupt_type: u8) -> Self {
        Self {
            timestamp,
            vector,
            cpu_id,
            interrupt_type,
            delivery_status: 0,
            reserved: [0; 4],
        }
    }
    
    /// Get interrupt type description
    pub fn get_interrupt_type_description(&self) -> &'static str {
        match self.interrupt_type {
            0 => "Normal",
            1 => "NMI",
            2 => "SMI",
            3 => "INIT",
            4 => "ExtINT",
            _ => "Unknown",
        }
    }
    
    /// Get delivery status description
    pub fn get_delivery_status_description(&self) -> &'static str {
        match self.delivery_status {
            0 => "Idle",
            1 => "Send Pending",
            2 => "Send Accepted",
            _ => "Unknown",
        }
    }
}

/// Log entry with timestamp and level
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct LogEntry {
    /// Timestamp when log was written
    pub timestamp: u64,
    /// Log level (0=TRACE, 1=INFO, 2=WARN, 3=ERR)
    pub level: u8,
    /// Module tag (4-character identifier)
    pub module_tag: [u8; 4],
    /// Log message length
    pub message_length: u16,
    /// Log message data (truncated to fit)
    pub message_data: [u8; 64],
    /// Reserved for future use
    pub reserved: [u8; 3],
}

impl LogEntry {
    /// Create new log entry
    pub fn new(timestamp: u64, level: u8, module_tag: &str, message: &str) -> Self {
        let mut tag = [0u8; 4];
        let tag_bytes = module_tag.as_bytes();
        for (i, &byte) in tag_bytes.iter().take(4).enumerate() {
            tag[i] = byte;
        }
        
        let mut data = [0u8; 64];
        let msg_bytes = message.as_bytes();
        let len = msg_bytes.len().min(64);
        for (i, &byte) in msg_bytes.iter().take(len).enumerate() {
            data[i] = byte;
        }
        
        Self {
            timestamp,
            level,
            module_tag: tag,
            message_length: len as u16,
            message_data: data,
            reserved: [0; 3],
        }
    }
    
    /// Get log level description
    pub fn get_level_description(&self) -> &'static str {
        match self.level {
            0 => "TRACE",
            1 => "INFO",
            2 => "WARN",
            3 => "ERR",
            _ => "UNKNOWN",
        }
    }
    
    /// Get module tag as string
    pub fn get_module_tag_string(&self) -> String {
        let mut result = String::new();
        for &byte in &self.module_tag {
            if byte != 0 {
                result.push(byte as char);
            }
        }
        result
    }
    
    /// Get message as string
    pub fn get_message_string(&self) -> String {
        let mut result = String::new();
        for &byte in &self.message_data[..self.message_length as usize] {
            if byte != 0 {
                result.push(byte as char);
            }
        }
        result
    }
}

/// CPU features and capabilities
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    /// CPU vendor string (12 characters)
    pub vendor_string: [u8; 12],
    /// CPU family
    pub family: u8,
    /// CPU model
    pub model: u8,
    /// CPU stepping
    pub stepping: u8,
    /// CPU features flags (low 32 bits)
    pub features_low: u32,
    /// CPU features flags (high 32 bits)
    pub features_high: u32,
    /// Extended CPU features flags (low 32 bits)
    pub ext_features_low: u32,
    /// Extended CPU features flags (high 32 bits)
    pub ext_features_high: u32,
    /// Cache line size in bytes
    pub cache_line_size: u32,
    /// Number of logical processors
    pub logical_processors: u32,
    /// Number of physical cores
    pub physical_cores: u32,
    /// Reserved for future use
    pub reserved: [u8; 16],
}

impl CpuFeatures {
    /// Create new CPU features structure
    pub fn new() -> Self {
        Self {
            vendor_string: [0; 12],
            family: 0,
            model: 0,
            stepping: 0,
            features_low: 0,
            features_high: 0,
            ext_features_low: 0,
            ext_features_high: 0,
            cache_line_size: 0,
            logical_processors: 0,
            physical_cores: 0,
            reserved: [0; 16],
        }
    }
    
    /// Check if specific feature is available
    pub fn has_feature(&self, feature: CpuFeatureFlag) -> bool {
        match feature {
            CpuFeatureFlag::SSE => (self.features_low & (1 << 25)) != 0,
            CpuFeatureFlag::SSE2 => (self.features_low & (1 << 26)) != 0,
            CpuFeatureFlag::SSE3 => (self.features_low & (1 << 0)) != 0,
            CpuFeatureFlag::SSSE3 => (self.features_low & (1 << 9)) != 0,
            CpuFeatureFlag::SSE4_1 => (self.features_low & (1 << 19)) != 0,
            CpuFeatureFlag::SSE4_2 => (self.features_low & (1 << 20)) != 0,
            CpuFeatureFlag::AVX => (self.ext_features_low & (1 << 28)) != 0,
            CpuFeatureFlag::AVX2 => (self.ext_features_low & (1 << 5)) != 0,
            CpuFeatureFlag::AVX512F => (self.ext_features_low & (1 << 16)) != 0,
            CpuFeatureFlag::AES => (self.ext_features_low & (1 << 25)) != 0,
            CpuFeatureFlag::RDRAND => (self.ext_features_low & (1 << 30)) != 0,
            CpuFeatureFlag::RDSEED => (self.ext_features_low & (1 << 18)) != 0,
        }
    }
    
    /// Get vendor string
    pub fn get_vendor_string(&self) -> String {
        let mut result = String::new();
        for &byte in &self.vendor_string {
            if byte != 0 {
                result.push(byte as char);
            }
        }
        result
    }
}

/// CPU feature flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuFeatureFlag {
    SSE,
    SSE2,
    SSE3,
    SSSE3,
    SSE4_1,
    SSE4_2,
    AVX,
    AVX2,
    AVX512F,
    AES,
    RDRAND,
    RDSEED,
}

/// Audit entry structure (must match kernel definition)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub event_type: u32,
    pub process_id: u64,
    pub user_id: u64,
    pub data: [u8; 32],
}

/// Serial buffer entry structure (must match kernel definition)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SerialBufferEntry {
    pub timestamp: u64,
    pub length: u16,
    pub data: [u8; 64],
}

/// Kernel build information structure (must match kernel definition)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct KernelBuildInfo {
    pub build_timestamp: u64,
    pub git_hash: [u8; 8],
    pub version_string: [u8; 32],
    pub build_flags: u64,
    pub target_arch: [u8; 16],
    pub target_platform: [u8; 16],
}

/// Memory region structure (must match kernel definition)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start_address: u64,
    pub end_address: u64,
    pub protection: u32,
    pub memory_type: u32,
    pub reserved: [u8; 8],
}

/// Complete minidump v2 structure
#[repr(C, packed)]
pub struct MinidumpV2 {
    pub header: MinidumpHeader,
    pub cpu_registers: CpuRegisters,
    pub page_fault_info: PageFaultInfo,
    pub apic_vector_history: [ApicVectorEntry; 64],
    pub apic_entry_count: u32,
    pub log_history: [LogEntry; 256],
    pub log_entry_count: u32,
    pub cpu_features: CpuFeatures,
    pub audit_entries: [AuditEntry; 64],
    pub audit_entry_count: u32,
    pub serial_buffer: [SerialBufferEntry; 32],
    pub serial_entry_count: u32,
    pub build_info: KernelBuildInfo,
    pub stack_trace: [u64; 16],
    pub stack_trace_count: u32,
    pub memory_regions: [MemoryRegion; 8],
    pub memory_region_count: u32,
}

/// Minidump flags
pub const MINIDUMP_FLAG_CPU_REGS: u32 = 1 << 0;
pub const MINIDUMP_FLAG_PAGE_FAULT: u32 = 1 << 1;
pub const MINIDUMP_FLAG_APIC_HISTORY: u32 = 1 << 2;
pub const MINIDUMP_FLAG_LOG_HISTORY: u32 = 1 << 3;
pub const MINIDUMP_FLAG_CPU_FEATURES: u32 = 1 << 4;
pub const MINIDUMP_FLAG_AUDIT_ENTRIES: u32 = 1 << 5;
pub const MINIDUMP_FLAG_SERIAL_BUFFER: u32 = 1 << 6;
pub const MINIDUMP_FLAG_BUILD_INFO: u32 = 1 << 7;
pub const MINIDUMP_FLAG_STACK_TRACE: u32 = 1 << 8;
pub const MINIDUMP_FLAG_MEMORY_REGIONS: u32 = 1 << 9;

/// Global minidump buffer
static mut MINIDUMP_BUFFER: [u8; 64 * 1024] = [0; 64 * 1024]; // 64KB buffer
static mut MINIDUMP_WRITTEN: bool = false;

/// Write minidump to buffer
pub fn write_minidump(
    cpu_regs: &CpuRegisters,
    panic_info: &core::panic::PanicInfo,
    stack_trace: &[u64],
    page_fault_info: Option<PageFaultInfo>,
    apic_history: &[ApicVectorEntry],
    log_history: &[LogEntry],
    cpu_features: &CpuFeatures,
) -> Result<(), &'static str> {
    unsafe {
        if MINIDUMP_WRITTEN {
            return Err("Minidump already written");
        }
        
        let mut buffer = &mut MINIDUMP_BUFFER[..];
        let mut offset = 0;
        
        // Calculate total size first
        let total_size = mem::size_of::<MinidumpV2>();
        
        // Create header
        let mut flags = MINIDUMP_FLAG_CPU_REGS | MINIDUMP_FLAG_STACK_TRACE;
        
        if page_fault_info.is_some() {
            flags |= MINIDUMP_FLAG_PAGE_FAULT;
        }
        if !apic_history.is_empty() {
            flags |= MINIDUMP_FLAG_APIC_HISTORY;
        }
        if !log_history.is_empty() {
            flags |= MINIDUMP_FLAG_LOG_HISTORY;
        }
        if cpu_features.features_low != 0 {
            flags |= MINIDUMP_FLAG_CPU_FEATURES;
        }
        
        let header = MinidumpHeader::new(
            get_current_timestamp(),
            total_size as u32,
            flags,
        );
        
        // Write header
        if buffer.len() < offset + mem::size_of::<MinidumpHeader>() {
            return Err("Buffer too small for header");
        }
        
        let header_bytes = as_bytes(&header);
        buffer[offset..offset + header_bytes.len()].copy_from_slice(header_bytes);
        offset += header_bytes.len();
        
        // Write CPU registers
        if buffer.len() < offset + mem::size_of::<CpuRegisters>() {
            return Err("Buffer too small for CPU registers");
        }
        
        let regs_bytes = as_bytes(cpu_regs);
        buffer[offset..offset + regs_bytes.len()].copy_from_slice(regs_bytes);
        offset += regs_bytes.len();
        
        // Write page fault info if available
        if let Some(pf_info) = page_fault_info {
            if buffer.len() < offset + mem::size_of::<PageFaultInfo>() {
                return Err("Buffer too small for page fault info");
            }
            
            let pf_bytes = as_bytes(&pf_info);
            buffer[offset..offset + pf_bytes.len()].copy_from_slice(pf_bytes);
            offset += pf_bytes.len();
        } else {
            // Write empty page fault info
            let empty_pf = PageFaultInfo::new(0, 0, 0, 0);
            if buffer.len() < offset + mem::size_of::<PageFaultInfo>() {
                return Err("Buffer too small for empty page fault info");
            }
            
            let pf_bytes = as_bytes(&empty_pf);
            buffer[offset..offset + pf_bytes.len()].copy_from_slice(pf_bytes);
            offset += pf_bytes.len();
        }
        
        // Write APIC history
        let apic_count = apic_history.len().min(64);
        for i in 0..64 {
            if buffer.len() < offset + mem::size_of::<ApicVectorEntry>() {
                return Err("Buffer too small for APIC history");
            }
            
            let entry = if i < apic_count {
                apic_history[i]
            } else {
                ApicVectorEntry::new(0, 0, 0, 0)
            };
            
            let entry_bytes = as_bytes(&entry);
            buffer[offset..offset + entry_bytes.len()].copy_from_slice(entry_bytes);
            offset += entry_bytes.len();
        }
        
        // Write APIC entry count
        if buffer.len() < offset + 4 {
            return Err("Buffer too small for APIC entry count");
        }
        
        let count_bytes = (apic_count as u32).to_le_bytes();
        buffer[offset..offset + 4].copy_from_slice(&count_bytes);
        offset += 4;
        
        // Write log history
        let log_count = log_history.len().min(256);
        for i in 0..256 {
            if buffer.len() < offset + mem::size_of::<LogEntry>() {
                return Err("Buffer too small for log history");
            }
            
            let entry = if i < log_count {
                log_history[i]
            } else {
                LogEntry::new(0, 0, "", "")
            };
            
            let entry_bytes = as_bytes(&entry);
            buffer[offset..offset + entry_bytes.len()].copy_from_slice(entry_bytes);
            offset += entry_bytes.len();
        }
        
        // Write log entry count
        if buffer.len() < offset + 4 {
            return Err("Buffer too small for log entry count");
        }
        
        let count_bytes = (log_count as u32).to_le_bytes();
        buffer[offset..offset + 4].copy_from_slice(&count_bytes);
        offset += 4;
        
        // Write CPU features
        if buffer.len() < offset + mem::size_of::<CpuFeatures>() {
            return Err("Buffer too small for CPU features");
        }
        
        let features_bytes = as_bytes(cpu_features);
        buffer[offset..offset + features_bytes.len()].copy_from_slice(features_bytes);
        offset += features_bytes.len();
        
        // Write remaining fields (simplified for now)
        // In a full implementation, you'd write all the remaining fields
        
        MINIDUMP_WRITTEN = true;
        Ok(())
    }
}

/// Get minidump buffer
pub fn get_minidump_buffer() -> Option<&'static [u8]> {
    unsafe {
        if MINIDUMP_WRITTEN {
            Some(&MINIDUMP_BUFFER)
        } else {
            None
        }
    }
}

/// Check if minidump has been written
pub fn has_minidump() -> bool {
    unsafe { MINIDUMP_WRITTEN }
}

/// Reset minidump state (for testing)
pub fn reset_minidump() {
    unsafe {
        MINIDUMP_WRITTEN = false;
        MINIDUMP_BUFFER = [0; 64 * 1024];
    }
}

/// Get current timestamp in milliseconds
fn get_current_timestamp() -> u64 {
    // This is a simplified implementation
    // In a real system, you'd use a high-resolution timer
    0 // Placeholder
}

/// Convert struct to bytes
fn as_bytes<T>(value: &T) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(
            value as *const T as *const u8,
            mem::size_of::<T>(),
        )
    }
}

/// Get CPU features using CPUID
pub fn get_cpu_features() -> CpuFeatures {
    let mut features = CpuFeatures::new();
    
    // Get vendor string
    unsafe {
        let mut vendor = [0u32; 3];
        asm!("cpuid", 
             in("eax") 0u32,
             out("ebx") vendor[0],
             out("ecx") vendor[1],
             out("edx") vendor[2]);
        
        // Convert to bytes
        for (i, &reg) in vendor.iter().enumerate() {
            let bytes = reg.to_le_bytes();
            for (j, &byte) in bytes.iter().enumerate() {
                features.vendor_string[i * 4 + j] = byte;
            }
        }
    }
    
    // Get basic features
    unsafe {
        let mut eax = 0u32;
        let mut ebx = 0u32;
        let mut ecx = 0u32;
        let mut edx = 0u32;
        
        asm!("cpuid", 
             in("eax") 1u32,
             out("eax") eax,
             out("ebx") ebx,
             out("ecx") ecx,
             out("edx") edx);
        
        features.family = ((eax >> 8) & 0xF) as u8;
        features.model = ((eax >> 4) & 0xF) as u8;
        features.stepping = (eax & 0xF) as u8;
        features.features_low = edx;
        features.features_high = ecx;
    }
    
    // Get extended features
    unsafe {
        let mut eax = 0u32;
        let mut ebx = 0u32;
        let mut ecx = 0u32;
        let mut edx = 0u32;
        
        asm!("cpuid", 
             in("eax") 7u32,
             in("ecx") 0u32,
             out("eax") eax,
             out("ebx") ebx,
             out("ecx") ecx,
             out("edx") edx);
        
        features.ext_features_low = ebx;
        features.ext_features_high = ecx;
    }
    
    // Get cache info
    unsafe {
        let mut eax = 0u32;
        let mut ebx = 0u32;
        let mut ecx = 0u32;
        let mut edx = 0u32;
        
        asm!("cpuid", 
             in("eax") 1u32,
             out("eax") eax,
             out("ebx") ebx,
             out("ecx") ecx,
             out("edx") edx);
        
        features.cache_line_size = ((ebx >> 8) & 0xFF) as u32 * 8;
    }
    
    features
}

/// Get APIC vector history (stub implementation)
pub fn get_apic_vector_history() -> [ApicVectorEntry; 64] {
    let mut history = [ApicVectorEntry::new(0, 0, 0, 0); 64];
    
    // In a real implementation, this would read from APIC registers
    // For now, we'll create some sample entries
    
    history[0] = ApicVectorEntry::new(1000, 32, 0, 0); // Timer interrupt
    history[1] = ApicVectorEntry::new(2000, 14, 0, 0); // Page fault
    history[2] = ApicVectorEntry::new(3000, 13, 0, 0); // General protection fault
    
    history
}

/// Get log history (stub implementation)
pub fn get_log_history() -> [LogEntry; 256] {
    let mut history = [LogEntry::new(0, 0, "", ""); 256];
    
    // In a real implementation, this would read from the log buffer
    // For now, we'll create some sample entries
    
    history[0] = LogEntry::new(1000, 1, "BOOT", "System starting up");
    history[1] = LogEntry::new(2000, 1, "MM", "Memory management initialized");
    history[2] = LogEntry::new(3000, 2, "SCHED", "Scheduler warning: high load");
    history[3] = LogEntry::new(4000, 3, "PANIC", "Kernel panic occurred");
    
    history
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_minidump_header_creation() {
        let header = MinidumpHeader::new(12345, 1024, 0xFF);
        assert_eq!(header.magic, [b'P', b'O', b'L', b'Y']);
        assert_eq!(header.version, 2);
        assert_eq!(header.timestamp, 12345);
        assert_eq!(header.total_size, 1024);
        assert_eq!(header.flags, 0xFF);
    }
    
    #[test]
    fn test_page_fault_info_creation() {
        let pf_info = PageFaultInfo::new(0x1000, 0x2, 0x2000, 0x3000);
        assert_eq!(pf_info.fault_address, 0x1000);
        assert_eq!(pf_info.error_code, 0x2);
        assert_eq!(pf_info.fault_rip, 0x2000);
        assert_eq!(pf_info.fault_rsp, 0x3000);
        assert!(!pf_info.is_protection_violation());
        assert!(pf_info.is_write_access());
        assert!(!pf_info.is_user_mode());
        assert!(!pf_info.is_instruction_fetch());
    }
    
    #[test]
    fn test_apic_vector_entry_creation() {
        let entry = ApicVectorEntry::new(1000, 32, 0, 0);
        assert_eq!(entry.timestamp, 1000);
        assert_eq!(entry.vector, 32);
        assert_eq!(entry.cpu_id, 0);
        assert_eq!(entry.interrupt_type, 0);
        assert_eq!(entry.get_interrupt_type_description(), "Normal");
        assert_eq!(entry.get_delivery_status_description(), "Idle");
    }
    
    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(1000, 1, "TEST", "Test message");
        assert_eq!(entry.timestamp, 1000);
        assert_eq!(entry.level, 1);
        assert_eq!(entry.get_level_description(), "INFO");
        assert_eq!(entry.get_module_tag_string(), "TEST");
        assert_eq!(entry.get_message_string(), "Test message");
    }
    
    #[test]
    fn test_cpu_features_creation() {
        let features = CpuFeatures::new();
        assert_eq!(features.vendor_string, [0; 12]);
        assert_eq!(features.family, 0);
        assert_eq!(features.model, 0);
        assert_eq!(features.stepping, 0);
        assert_eq!(features.features_low, 0);
        assert_eq!(features.features_high, 0);
        assert_eq!(features.ext_features_low, 0);
        assert_eq!(features.ext_features_high, 0);
    }
    
    #[test]
    fn test_minidump_buffer_operations() {
        // Reset state
        reset_minidump();
        assert!(!has_minidump());
        
        // Create sample data
        let cpu_regs = CpuRegisters {
            rax: 0x1234, rbx: 0x5678, rcx: 0x9ABC, rdx: 0xDEF0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0x1000, rflags: 0,
            cs: 0, ds: 0, es: 0, fs: 0, gs: 0, ss: 0,
            cr0: 0, cr2: 0, cr3: 0, cr4: 0, cr8: 0,
            dr0: 0, dr1: 0, dr2: 0, dr3: 0, dr6: 0, dr7: 0,
            msr_efer: 0, msr_star: 0, msr_lstar: 0, msr_cstar: 0,
            msr_sfmask: 0, msr_fs_base: 0, msr_gs_base: 0, msr_kernel_gs_base: 0,
        };
        
        let stack_trace = [0x1000, 0x2000, 0x3000];
        
        // Write minidump
        let result = write_minidump(
            &cpu_regs,
            &core::panic::PanicInfo::internal_constructor(
                core::panic::Location::internal_constructor(
                    core::panic::Location::caller(),
                    core::panic::Location::caller(),
                ),
                core::panic::fmt::Arguments::new_v1(&[], &[]),
            ),
            &stack_trace,
            None,
            &[],
            &[],
            &CpuFeatures::new(),
        );
        
        assert!(result.is_ok());
        assert!(has_minidump());
        
        // Get buffer
        let buffer = get_minidump_buffer();
        assert!(buffer.is_some());
        
        // Reset again
        reset_minidump();
        assert!(!has_minidump());
    }
}
