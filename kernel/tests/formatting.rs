/// Formatting System Tests
/// 
/// Tests the enhanced formatting capabilities including:
/// - Hexdump functionality
/// - u128 formatting support
/// - Capability token formatting
/// - Format stability verification

use crate::format;

/// Test basic formatting functionality
#[test]
fn test_basic_formatting() {
    kprintln!("[TEST] Testing basic formatting functionality...");
    
    // Test u128 formatting
    let test_u128: u128 = 0x1234567890abcdef1234567890abcdef;
    
    // Test hex formatting
    let hex_str = test_u128.fmt_hex();
    assert_eq!(hex_str, "0x1234567890abcdef1234567890abcdef", "u128 hex formatting should be correct");
    
    // Test decimal formatting
    let dec_str = test_u128.fmt_decimal_grouped();
    assert!(dec_str.contains("_"), "Decimal formatting should include grouping separators");
    
    kprintln!("[TEST] Basic formatting functionality PASSED");
}

/// Test hexdump functionality
#[test]
fn test_hexdump() {
    kprintln!("[TEST] Testing hexdump functionality...");
    
    // Test empty data
    let empty_data: &[u8] = &[];
    let empty_dump = format::HexdumpFormatter::new(empty_data).to_string();
    assert_eq!(empty_dump, "<empty>", "Empty data should show <empty>");
    
    // Test small data
    let small_data = b"Hello";
    let small_dump = format::HexdumpFormatter::new(small_data).to_string();
    assert!(small_dump.contains("48 65 6c 6c 6f"), "Small data hexdump should contain correct hex values");
    assert!(small_dump.contains("Hello"), "Small data hexdump should contain ASCII representation");
    
    // Test larger data
    let large_data = b"Polymera OS Kernel - Secure by Design";
    let large_dump = format::HexdumpFormatter::new(large_data).to_string();
    assert!(large_dump.contains("50 6f 6c 79 6d 65 72 61"), "Large data hexdump should contain correct hex values");
    
    kprintln!("[TEST] Hexdump functionality PASSED");
}

/// Test capability formatting
#[test]
fn test_capability_formatting() {
    kprintln!("[TEST] Testing capability formatting...");
    
    let test_cap: u128 = 0xdeadbeefcafebabe1234567890abcdef;
    
    // Test simple capability formatting
    let cap_formatter = format::format_capability(test_cap);
    let cap_str = cap_formatter.format();
    assert!(cap_str.contains("0xdeadbeefcafebabe1234567890abcdef"), "Capability should contain hex value");
    
    // Test verbose capability formatting
    let verbose_cap = format::CapabilityFormatter {
        value: test_cap,
        verbose: true,
    };
    let verbose_str = verbose_cap.format();
    assert!(verbose_str.contains("Cap["), "Verbose capability should contain Cap[ prefix");
    assert!(verbose_str.contains("dec:"), "Verbose capability should contain decimal representation");
    
    kprintln!("[TEST] Capability formatting PASSED");
}

/// Test format stability
#[test]
fn test_format_stability() {
    kprintln!("[TEST] Testing format stability...");
    
    let result = format::test_format_stability();
    assert!(result.is_ok(), "Format stability test should pass");
    
    kprintln!("[TEST] Format stability PASSED");
}

/// Test u128 Display trait implementation
#[test]
fn test_u128_display() {
    kprintln!("[TEST] Testing u128 Display trait implementation...");
    
    let test_u128: u128 = 0x1234567890abcdef1234567890abcdef;
    
    // Test that u128 can be formatted with "{}"
    let formatted = format!("{}", test_u128);
    assert!(formatted.starts_with("0x"), "u128 Display should start with 0x");
    assert!(formatted.len() == 34, "u128 Display should be 32 hex chars + 0x prefix");
    
    kprintln!("[TEST] u128 Display trait implementation PASSED");
}

/// Test hexdump with different configurations
#[test]
fn test_hexdump_configurations() {
    kprintln!("[TEST] Testing hexdump configurations...");
    
    let test_data = b"Polymera OS";
    
    // Test default configuration
    let default_dump = format::HexdumpFormatter::new(test_data).to_string();
    assert!(default_dump.contains(": "), "Default config should show offset");
    assert!(default_dump.contains("|"), "Default config should show ASCII");
    
    // Test custom configuration
    let custom_config = format::HexdumpConfig {
        bytes_per_line: 8,
        show_ascii: false,
        show_offset: true,
        address_base: 0x1000,
    };
    
    let custom_formatter = format::HexdumpFormatter {
        data: test_data,
        config: custom_config,
    };
    
    let custom_dump = custom_formatter.to_string();
    assert!(custom_dump.contains("1000:"), "Custom config should show custom address base");
    assert!(!custom_dump.contains("|"), "Custom config without ASCII should not show |");
    
    kprintln!("[TEST] Hexdump configurations PASSED");
}

/// Run all formatting tests
pub fn run_all_formatting_tests() -> Result<(), &'static str> {
    kprintln!("");
    kprintln!("=== RUNNING FORMATTING TESTS ===");
    
    test_basic_formatting();
    test_hexdump();
    test_capability_formatting();
    test_format_stability();
    test_u128_display();
    test_hexdump_configurations();
    
    kprintln!("");
    kprintln!("=== ALL FORMATTING TESTS PASSED ===");
    kprintln!("");
    kprintln!("The formatting system is fully operational!");
    kprintln!("  - Hexdump functionality: ✅");
    kprintln!("  - u128 formatting support: ✅");
    kprintln!("  - Capability token formatting: ✅");
    kprintln!("  - Format stability: ✅");
    kprintln!("  - Display trait implementations: ✅");
    kprintln!("");
    
    Ok(())
}



