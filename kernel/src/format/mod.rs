//! Kernel Formatting Module
//! 
//! This module provides enhanced formatting capabilities for the kernel:
//! - Hexdump functionality for binary data
//! - Extended formatting support for u128 and other types
//! - Format stability testing
//! - Enhanced Display trait implementations

use core::fmt::{self, Write, Display};
use alloc::string::ToString;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

/// Hexdump configuration
pub struct HexdumpConfig {
    /// Number of bytes per line
    pub bytes_per_line: usize,
    /// Show ASCII representation
    pub show_ascii: bool,
    /// Show offset
    pub show_offset: bool,
    /// Address base (for offset calculation)
    pub address_base: usize,
}

impl Default for HexdumpConfig {
    fn default() -> Self {
        Self {
            bytes_per_line: 16,
            show_ascii: true,
            show_offset: true,
            address_base: 0,
        }
    }
}

/// Hexdump formatter
pub struct HexdumpFormatter<'a> {
    data: &'a [u8],
    config: HexdumpConfig,
}

impl<'a> HexdumpFormatter<'a> {
    /// Create a new hexdump formatter
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            config: HexdumpConfig::default(),
        }
    }
    
    /// Format the hexdump to a string
    pub fn to_string(&self) -> alloc::string::String {
        let mut output = alloc::string::String::new();
        let _ = self.write_to(&mut output);
        output
    }
    
    /// Write the hexdump to a writer
    pub fn write_to<W: Write>(&self, writer: &mut W) -> fmt::Result {
        if self.data.is_empty() {
            return write!(writer, "<empty>");
        }
        
        let mut offset = 0;
        let mut line_buffer = alloc::vec::Vec::new();
        let mut ascii_buffer = alloc::vec::Vec::new();
        
        while offset < self.data.len() {
            let line_end = core::cmp::min(offset + self.config.bytes_per_line, self.data.len());
            let line_data = &self.data[offset..line_end];
            
            // Clear buffers for this line
            line_buffer.clear();
            ascii_buffer.clear();
            
            // Format hex bytes
            for &byte in line_data {
                line_buffer.push(format!("{:02x}", byte));
                
                // Add ASCII representation
                if self.config.show_ascii {
                    let ascii_char = if byte.is_ascii_graphic() || byte == b' ' {
                        byte as char
                    } else {
                        '.'
                    };
                    ascii_buffer.push(ascii_char);
                }
            }
            
            // Pad the line if needed
            while line_buffer.len() < self.config.bytes_per_line {
                line_buffer.push("  ".to_string());
                if self.config.show_ascii {
                    ascii_buffer.push(' ');
                }
            }
            
            // Write offset
            if self.config.show_offset {
                write!(writer, "{:08x}: ", offset + self.config.address_base)?;
            }
            
            // Write hex bytes
            write!(writer, "{}", line_buffer.join(" "))?;
            
            // Write ASCII representation
            if self.config.show_ascii {
                write!(writer, "  |{}|", ascii_buffer.iter().collect::<String>())?;
            }
            
            writeln!(writer)?;
            
            offset = line_end;
        }
        
        Ok(())
    }
}

impl<'a> Display for HexdumpFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_to(f)
    }
}

/// Print a hexdump of binary data
pub fn kprint_hex(data: &[u8]) {
    let formatter = HexdumpFormatter::new(data);
    crate::kprintln!("{}", formatter);
}

/// Extended formatting support for u128
pub trait ExtendedFormatting {
    /// Format as hexadecimal with consistent width
    fn fmt_hex(&self) -> alloc::string::String;
    
    /// Format as decimal with grouping
    fn fmt_decimal_grouped(&self) -> alloc::string::String;
}

impl ExtendedFormatting for u128 {
    fn fmt_hex(&self) -> alloc::string::String {
        format!("0x{:032x}", self)
    }
    
    fn fmt_decimal_grouped(&self) -> alloc::string::String {
        let mut s = self.to_string();
        let mut i = s.len();
        while i > 3 {
            i -= 3;
            s.insert(i, '_');
        }
        s
    }
}

/// Newtype wrapper for u128 to enable custom Display formatting
/// (Cannot implement Display for u128 directly due to orphan rules)
#[derive(Debug, Clone, Copy)]
pub struct U128Display(pub u128);

impl Display for U128Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use hex format for u128 by default
        write!(f, "0x{:032x}", self.0)
    }
}

impl From<u128> for U128Display {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

/// Capability token formatter
pub struct CapabilityFormatter {
    /// The capability token value
    pub value: u128,
    /// Whether to show full details
    pub verbose: bool,
}

impl CapabilityFormatter {
    /// Create a new capability formatter
    pub fn new(value: u128) -> Self {
        Self {
            value,
            verbose: false,
        }
    }
    
    /// Format the capability token
    pub fn format(&self) -> alloc::string::String {
        if self.verbose {
            format!(
                "Cap[0x{:032x}] (dec: {})",
                self.value,
                self.value.fmt_decimal_grouped()
            )
        } else {
            format!("0x{:032x}", self.value)
        }
    }
}

impl Display for CapabilityFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Format a capability token for display
pub fn format_capability(value: u128) -> CapabilityFormatter {
    CapabilityFormatter::new(value)
}

/// Format a capability token for display (verbose)
pub fn format_capability_verbose(value: u128) -> CapabilityFormatter {
    let mut f = CapabilityFormatter::new(value);
    f.verbose = true;
    f
}

/// Test format stability
pub fn test_format_stability() -> Result<(), &'static str> {
    crate::kprintln!("[FORMAT] Testing format stability...");
    
    // Test u128 formatting consistency
    let test_value: u128 = 0x1234567890abcdef1234567890abcdef;
    
    let hex1 = test_value.fmt_hex();
    let hex2 = test_value.fmt_hex();
    assert_eq!(hex1, hex2, "u128 hex formatting should be consistent");
    
    let dec1 = test_value.fmt_decimal_grouped();
    let dec2 = test_value.fmt_decimal_grouped();
    assert_eq!(dec1, dec2, "u128 decimal formatting should be consistent");
    
    // Test capability formatting consistency
    let cap1 = format_capability(test_value);
    let cap2 = format_capability(test_value);
    assert_eq!(cap1.format(), cap2.format(), "Capability formatting should be consistent");
    
    // Test hexdump formatting consistency
    let test_data = b"Hello, Polymera OS!";
    let dump1 = HexdumpFormatter::new(test_data).to_string();
    let dump2 = HexdumpFormatter::new(test_data).to_string();
    assert_eq!(dump1, dump2, "Hexdump formatting should be consistent");
    
    crate::kprintln!("[FORMAT] Format stability test PASSED");
    Ok(())
}

/// Print format examples
pub fn print_format_examples() {
    crate::kprintln!("");
    crate::kprintln!("=== FORMAT EXAMPLES ===");
    
    let test_u128: u128 = 0x1234567890abcdef1234567890abcdef;
    
    crate::kprintln!("u128 Examples:");
    crate::kprintln!("  Hex: {}", test_u128.fmt_hex());
    crate::kprintln!("  Decimal: {}", test_u128.fmt_decimal_grouped());
    
    crate::kprintln!("Capability Examples:");
    crate::kprintln!("  Simple: {}", format_capability(test_u128));
    crate::kprintln!("  Verbose: {}", format_capability_verbose(test_u128));
    
    crate::kprintln!("Hexdump Examples:");
    let test_data = b"Polymera OS Kernel - Secure by Design";
    kprint_hex(test_data);
    
    crate::kprintln!("==========================");
    crate::kprintln!("");
}
