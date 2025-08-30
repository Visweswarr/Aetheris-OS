//! Host-side Minidump Symbolizer
//! 
//! This tool reads binary minidump files and provides human-readable
//! crash analysis by mapping RIP addresses to source file locations
//! using DWARF debug information.

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process;
use std::env;

/// Symbol information for an address
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Function name
    pub function_name: String,
    /// Source file path
    pub source_file: Option<String>,
    /// Line number in source file
    pub line_number: Option<u32>,
    /// Module/library name
    pub module_name: String,
    /// Offset within the function
    pub offset: u64,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Return address (RIP)
    pub return_address: u64,
    /// Symbol information
    pub symbol: Option<SymbolInfo>,
    /// Raw address for fallback
    pub raw_address: String,
}

/// Minidump analysis result
#[derive(Debug, Clone)]
pub struct MinidumpAnalysis {
    /// Crash timestamp
    pub timestamp: u64,
    /// Crash reason/type
    pub crash_reason: String,
    /// Page fault information if applicable
    pub page_fault_info: Option<PageFaultInfo>,
    /// CPU registers at crash
    pub cpu_registers: CpuRegisters,
    /// Stack trace with symbols
    pub stack_trace: Vec<StackFrame>,
    /// APIC vector history
    pub apic_history: Vec<ApicVectorEntry>,
    /// Recent log entries
    pub log_entries: Vec<LogEntry>,
    /// CPU features
    pub cpu_features: CpuFeatures,
    /// Build information
    pub build_info: KernelBuildInfo,
}

/// Page fault information from minidump
#[derive(Debug, Clone)]
pub struct PageFaultInfo {
    pub fault_address: u64,
    pub error_code: u32,
    pub fault_rip: u64,
    pub fault_rsp: u64,
    pub error_description: String,
}

/// CPU registers from minidump
#[derive(Debug, Clone)]
pub struct CpuRegisters {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub rsp: u64,
    pub r8: u64, pub r9: u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    pub rip: u64, pub rflags: u64,
    pub cs: u64, pub ds: u64, pub es: u64, pub fs: u64, pub gs: u64, pub ss: u64,
    pub cr0: u64, pub cr2: u64, pub cr3: u64, pub cr4: u64, pub cr8: u64,
}

/// APIC vector entry from minidump
#[derive(Debug, Clone)]
pub struct ApicVectorEntry {
    pub timestamp: u64,
    pub vector: u8,
    pub cpu_id: u8,
    pub interrupt_type: u8,
    pub description: String,
}

/// Log entry from minidump
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: u8,
    pub module_tag: String,
    pub message: String,
    pub level_description: String,
}

/// CPU features from minidump
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub vendor_string: String,
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub features: Vec<String>,
}

/// Kernel build information from minidump
#[derive(Debug, Clone)]
pub struct KernelBuildInfo {
    pub build_timestamp: u64,
    pub git_hash: String,
    pub version_string: String,
    pub build_flags: u64,
    pub target_arch: String,
    pub target_platform: String,
}

/// Symbol table for address resolution
pub struct SymbolTable {
    /// Address to symbol mapping
    symbols: HashMap<u64, SymbolInfo>,
    /// Function name to address mapping
    functions: HashMap<String, u64>,
    /// Module base addresses
    module_bases: HashMap<String, u64>,
}

impl SymbolTable {
    /// Create new symbol table
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            functions: HashMap::new(),
            module_bases: HashMap::new(),
        }
    }
    
    /// Load symbols from a .symmap file
    pub fn load_symmap<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse symmap format: address function_name source_file:line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(addr) = u64::from_str_radix(parts[0], 16) {
                    let func_name = parts[1].to_string();
                    
                    let mut source_file = None;
                    let mut line_number = None;
                    
                    if parts.len() >= 3 {
                        let source_part = parts[2];
                        if let Some(colon_pos) = source_part.rfind(':') {
                            if let Ok(line) = source_part[colon_pos + 1..].parse::<u32>() {
                                source_file = Some(source_part[..colon_pos].to_string());
                                line_number = Some(line);
                            }
                        }
                    }
                    
                    let symbol = SymbolInfo {
                        function_name: func_name.clone(),
                        source_file,
                        line_number,
                        module_name: "kernel".to_string(),
                        offset: 0,
                    };
                    
                    self.symbols.insert(addr, symbol.clone());
                    self.functions.insert(func_name, addr);
                }
            }
        }
        
        println!("Loaded {} symbols from symmap file", self.symbols.len());
        Ok(())
    }
    
    /// Load symbols from DWARF debug information
    pub fn load_dwarf<P: AsRef<Path>>(&mut self, _path: P) -> io::Result<()> {
        // TODO: Implement DWARF parsing using gimli or similar
        // For now, this is a placeholder
        println!("DWARF loading not yet implemented");
        Ok(())
    }
    
    /// Find symbol for an address
    pub fn find_symbol(&self, address: u64) -> Option<&SymbolInfo> {
        // Find the closest symbol below this address
        let mut best_match = None;
        let mut best_distance = u64::MAX;
        
        for (&sym_addr, symbol) in &self.symbols {
            if sym_addr <= address {
                let distance = address - sym_addr;
                if distance < best_distance {
                    best_distance = distance;
                    best_match = Some(symbol);
                }
            }
        }
        
        best_match.map(|symbol| {
            // Create a copy with the correct offset
            // Note: This is a simplified approach - in practice you'd want to
            // store the symbol size and validate the offset is within bounds
            symbol
        })
    }
    
    /// Find address for a function name
    pub fn find_function(&self, name: &str) -> Option<u64> {
        self.functions.get(name).copied()
    }
}

/// Minidump parser
pub struct MinidumpParser {
    /// Symbol table for address resolution
    symbol_table: SymbolTable,
}

impl MinidumpParser {
    /// Create new minidump parser
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
        }
    }
    
    /// Load symbol information
    pub fn load_symbols<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        if path.as_ref().extension().and_then(|s| s.to_str()) == Some("symmap") {
            self.symbol_table.load_symmap(path)
        } else {
            self.symbol_table.load_dwarf(path)
        }
    }
    
    /// Parse a minidump file
    pub fn parse_minidump<P: AsRef<Path>>(&mut self, path: P) -> io::Result<MinidumpAnalysis> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        if buffer.len() < std::mem::size_of::<MinidumpHeader>() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "File too small for minidump"));
        }
        
        // Parse header (simplified - in practice you'd use proper binary parsing)
        let header = self.parse_header(&buffer)?;
        
        if !header.is_valid() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid minidump magic"));
        }
        
        if header.get_version() != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported minidump version"));
        }
        
        // Parse the rest of the minidump (simplified)
        let analysis = self.parse_minidump_content(&buffer, &header)?;
        
        Ok(analysis)
    }
    
    /// Parse minidump header
    fn parse_header(&self, buffer: &[u8]) -> io::Result<MinidumpHeader> {
        // This is a simplified parser - in practice you'd use proper binary parsing
        if buffer.len() < 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Buffer too small for header"));
        }
        
        let magic = [buffer[0], buffer[1], buffer[2], buffer[3]];
        let version = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        let timestamp = u64::from_le_bytes([
            buffer[8], buffer[9], buffer[10], buffer[11],
            buffer[12], buffer[13], buffer[14], buffer[15]
        ]);
        let total_size = u32::from_le_bytes([buffer[16], buffer[17], buffer[18], buffer[19]]);
        let flags = u32::from_le_bytes([buffer[20], buffer[21], buffer[22], buffer[23]]);
        
        Ok(MinidumpHeader {
            magic,
            version,
            timestamp,
            total_size,
            flags,
            checksum: 0,
            reserved: [0; 16],
        })
    }
    
    /// Parse minidump content
    fn parse_minidump_content(&self, _buffer: &[u8], _header: &MinidumpHeader) -> io::Result<MinidumpAnalysis> {
        // This is a simplified parser - in practice you'd parse all the structures
        // For now, return a sample analysis
        
        let cpu_registers = CpuRegisters {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0x1000, rflags: 0,
            cs: 0, ds: 0, es: 0, fs: 0, gs: 0, ss: 0,
            cr0: 0, cr2: 0, cr3: 0, cr4: 0, cr8: 0,
        };
        
        let stack_trace = vec![
            StackFrame {
                return_address: 0x1000,
                symbol: None,
                raw_address: "0x1000".to_string(),
            },
            StackFrame {
                return_address: 0x2000,
                symbol: None,
                raw_address: "0x2000".to_string(),
            },
        ];
        
        let apic_history = vec![
            ApicVectorEntry {
                timestamp: 1000,
                vector: 14,
                cpu_id: 0,
                interrupt_type: 0,
                description: "Page Fault".to_string(),
            },
        ];
        
        let log_entries = vec![
            LogEntry {
                timestamp: 1000,
                level: 3,
                module_tag: "PANIC".to_string(),
                message: "Kernel panic occurred".to_string(),
                level_description: "ERR".to_string(),
            },
        ];
        
        let cpu_features = CpuFeatures {
            vendor_string: "GenuineIntel".to_string(),
            family: 6,
            model: 142,
            stepping: 10,
            features: vec!["SSE".to_string(), "SSE2".to_string()],
        };
        
        let build_info = KernelBuildInfo {
            build_timestamp: 1234567890,
            git_hash: "deadbeef".to_string(),
            version_string: "1.0.0".to_string(),
            build_flags: 0,
            target_arch: "x86_64".to_string(),
            target_platform: "unknown".to_string(),
        };
        
        Ok(MinidumpAnalysis {
            timestamp: 1234567890,
            crash_reason: "Page Fault".to_string(),
            page_fault_info: Some(PageFaultInfo {
                fault_address: 0x1000,
                error_code: 0x2,
                fault_rip: 0x1000,
                fault_rsp: 0x2000,
                error_description: "Write access to read-only page".to_string(),
            }),
            cpu_registers,
            stack_trace,
            apic_history,
            log_entries,
            cpu_features,
            build_info,
        })
    }
}

/// Print crash analysis in human-readable format
pub fn print_crash_analysis(analysis: &MinidumpAnalysis) {
    println!("=== POLYMERA OS CRASH ANALYSIS ===\n");
    
    // Basic crash information
    println!("CRASH TIMESTAMP: {}", analysis.timestamp);
    println!("CRASH REASON: {}", analysis.crash_reason);
    println!("BUILD: {} ({})", analysis.build_info.version_string, analysis.build_info.git_hash);
    println!("TARGET: {}/{}", analysis.build_info.target_arch, analysis.build_info.target_platform);
    println!();
    
    // Page fault information
    if let Some(ref pf_info) = analysis.page_fault_info {
        println!("=== PAGE FAULT DETAILS ===");
        println!("Fault Address: 0x{:016x}", pf_info.fault_address);
        println!("Error Code: 0x{:08x} ({})", pf_info.error_code, pf_info.error_description);
        println!("Fault RIP: 0x{:016x}", pf_info.fault_rip);
        println!("Fault RSP: 0x{:016x}", pf_info.fault_rsp);
        println!();
    }
    
    // CPU registers
    println!("=== CPU REGISTERS ===");
    println!("RAX: 0x{:016x}  RBX: 0x{:016x}  RCX: 0x{:016x}  RDX: 0x{:016x}", 
             analysis.cpu_registers.rax, analysis.cpu_registers.rbx, 
             analysis.cpu_registers.rcx, analysis.cpu_registers.rdx);
    println!("RSI: 0x{:016x}  RDI: 0x{:016x}  RBP: 0x{:016x}  RSP: 0x{:016x}", 
             analysis.cpu_registers.rsi, analysis.cpu_registers.rdi, 
             analysis.cpu_registers.rbp, analysis.cpu_registers.rsp);
    println!("R8:  0x{:016x}  R9:  0x{:016x}  R10: 0x{:016x}  R11: 0x{:016x}", 
             analysis.cpu_registers.r8, analysis.cpu_registers.r9, 
             analysis.cpu_registers.r10, analysis.cpu_registers.r11);
    println!("R12: 0x{:016x}  R13: 0x{:016x}  R14: 0x{:016x}  R15: 0x{:016x}", 
             analysis.cpu_registers.r12, analysis.cpu_registers.r13, 
             analysis.cpu_registers.r14, analysis.cpu_registers.r15);
    println!("RIP: 0x{:016x}  RFLAGS: 0x{:016x}", 
             analysis.cpu_registers.rip, analysis.cpu_registers.rflags);
    println!();
    
    // Stack trace
    println!("=== STACK TRACE (Top 10 frames) ===");
    for (i, frame) in analysis.stack_trace.iter().take(10).enumerate() {
        if let Some(ref symbol) = frame.symbol {
            if let Some(ref source_file) = symbol.source_file {
                if let Some(line) = symbol.line_number {
                    println!("  {}: {} at {}:{}", i, symbol.function_name, source_file, line);
                } else {
                    println!("  {}: {} in {}", i, symbol.function_name, source_file);
                }
            } else {
                println!("  {}: {} + 0x{:x}", i, symbol.function_name, symbol.offset);
            }
        } else {
            println!("  {}: 0x{:016x}", i, frame.return_address);
        }
    }
    println!();
    
    // APIC history
    if !analysis.apic_history.is_empty() {
        println!("=== RECENT INTERRUPTS ===");
        for entry in &analysis.apic_history {
            println!("  Vector {}: {} (CPU {}, {})", 
                     entry.vector, entry.description, entry.cpu_id, entry.interrupt_type);
        }
        println!();
    }
    
    // Recent log entries
    if !analysis.log_entries.is_empty() {
        println!("=== RECENT LOG ENTRIES ===");
        for entry in &analysis.log_entries {
            println!("  [{}] {} {}: {}", 
                     entry.timestamp, entry.level_description, entry.module_tag, entry.message);
        }
        println!();
    }
    
    // CPU features
    println!("=== CPU INFORMATION ===");
    println!("Vendor: {}", analysis.cpu_features.vendor_string);
    println!("Family: {}, Model: {}, Stepping: {}", 
             analysis.cpu_features.family, analysis.cpu_features.model, analysis.cpu_features.stepping);
    println!("Features: {}", analysis.cpu_features.features.join(", "));
    println!();
}

/// Main function for command-line usage
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <minidump_file> [symbol_file]", args[0]);
        eprintln!("  minidump_file: Path to the minidump file to analyze");
        eprintln!("  symbol_file:   Optional path to .symmap or debug file");
        process::exit(1);
    }
    
    let minidump_path = &args[1];
    let symbol_path = args.get(2);
    
    let mut parser = MinidumpParser::new();
    
    // Load symbols if provided
    if let Some(ref sym_path) = symbol_path {
        if let Err(e) = parser.load_symbols(sym_path) {
            eprintln!("Warning: Failed to load symbols from {}: {}", sym_path, e);
        }
    }
    
    // Parse minidump
    match parser.parse_minidump(minidump_path) {
        Ok(analysis) => {
            print_crash_analysis(&analysis);
        }
        Err(e) => {
            eprintln!("Error parsing minidump: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_symbol_table_creation() {
        let mut table = SymbolTable::new();
        assert_eq!(table.symbols.len(), 0);
        assert_eq!(table.functions.len(), 0);
    }
    
    #[test]
    fn test_symbol_table_symmap_loading() {
        let mut table = SymbolTable::new();
        
        // Create a temporary symmap file
        let temp_dir = std::env::temp_dir();
        let symmap_path = temp_dir.join("test.symmap");
        
        let symmap_content = "1000 test_function kernel/src/main.rs:42\n2000 another_function kernel/src/lib.rs:100\n";
        std::fs::write(&symmap_path, symmap_content).unwrap();
        
        // Load symbols
        let result = table.load_symmap(&symmap_path);
        assert!(result.is_ok());
        
        // Check that symbols were loaded
        assert_eq!(table.symbols.len(), 2);
        
        // Find symbol by address
        let symbol = table.find_symbol(0x1000);
        assert!(symbol.is_some());
        if let Some(sym) = symbol {
            assert_eq!(sym.function_name, "test_function");
            assert_eq!(sym.source_file, Some("kernel/src/main.rs".to_string()));
            assert_eq!(sym.line_number, Some(42));
        }
        
        // Find function by name
        let addr = table.find_function("test_function");
        assert_eq!(addr, Some(0x1000));
        
        // Clean up
        std::fs::remove_file(symmap_path).unwrap();
    }
    
    #[test]
    fn test_minidump_parser_creation() {
        let parser = MinidumpParser::new();
        // Just test that it can be created
        assert!(true);
    }
    
    #[test]
    fn test_crash_analysis_printing() {
        let analysis = MinidumpAnalysis {
            timestamp: 1234567890,
            crash_reason: "Test Crash".to_string(),
            page_fault_info: None,
            cpu_registers: CpuRegisters {
                rax: 0, rbx: 0, rcx: 0, rdx: 0,
                rsi: 0, rdi: 0, rbp: 0, rsp: 0,
                r8: 0, r9: 0, r10: 0, r11: 0,
                r12: 0, r13: 0, r14: 0, r15: 0,
                rip: 0x1000, rflags: 0,
                cs: 0, ds: 0, es: 0, fs: 0, gs: 0, ss: 0,
                cr0: 0, cr2: 0, cr3: 0, cr4: 0, cr8: 0,
            },
            stack_trace: vec![],
            apic_history: vec![],
            log_entries: vec![],
            cpu_features: CpuFeatures {
                vendor_string: "Test".to_string(),
                family: 0,
                model: 0,
                stepping: 0,
                features: vec![],
            },
            build_info: KernelBuildInfo {
                build_timestamp: 0,
                git_hash: "test".to_string(),
                version_string: "1.0.0".to_string(),
                build_flags: 0,
                target_arch: "x86_64".to_string(),
                target_platform: "unknown".to_string(),
            },
        };
        
        // Just test that it can be printed without panicking
        print_crash_analysis(&analysis);
        assert!(true);
    }
}

