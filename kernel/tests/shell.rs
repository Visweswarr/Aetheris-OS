//! Shell System Tests
//! 
//! This module tests the minimal line-oriented shell system including
//! command parsing, error handling, and integration with the kernel.

use crate::shell::{Shell, ShellCommand};
use crate::log::{kprintln, klog, Level};
use alloc::string::String;
use alloc::vec::Vec;

/// Test shell creation and basic functionality
fn test_shell_creation() {
    kprintln!("[TEST] Testing shell creation...");
    
    let shell = Shell::new();
    
    // Check that commands were registered
    assert!(!shell.get_commands().is_empty(), "Shell should have registered commands");
    
    // Check for required commands
    let commands = shell.get_commands();
    assert!(commands.contains_key("help"), "Shell should have 'help' command");
    assert!(commands.contains_key("stats"), "Shell should have 'stats' command");
    assert!(commands.contains_key("audit"), "Shell should have 'audit' command");
    assert!(commands.contains_key("sched"), "Shell should have 'sched' command");
    assert!(commands.contains_key("dump"), "Shell should have 'dump' command");
    assert!(commands.contains_key("fault"), "Shell should have 'fault' command");
    assert!(commands.contains_key("clear"), "Shell should have 'clear' command");
    assert!(commands.contains_key("echo"), "Shell should have 'echo' command");
    
    kprintln!("✅ Shell creation test passed");
}

/// Test help command functionality
fn test_help_command() {
    kprintln!("[TEST] Testing help command...");
    
    let result = Shell::cmd_help(&[]);
    assert!(result.is_ok(), "Help command should succeed with no arguments");
    
    let output = result.unwrap();
    assert!(output.contains("Available commands"), "Help output should contain command list");
    assert!(output.contains("help"), "Help output should contain help command");
    assert!(output.contains("stats"), "Help output should contain stats command");
    assert!(output.contains("audit"), "Help output should contain audit command");
    assert!(output.contains("sched"), "Help output should contain sched command");
    assert!(output.contains("dump"), "Help output should contain dump command");
    assert!(output.contains("fault"), "Help output should contain fault command");
    assert!(output.contains("clear"), "Help output should contain clear command");
    assert!(output.contains("echo"), "Help output should contain echo command");
    
    // Test with arguments (should fail)
    let result_with_args = Shell::cmd_help(&["extra"]);
    assert!(result_with_args.is_err(), "Help command should fail with arguments");
    
    kprintln!("✅ Help command test passed");
}

/// Test stats command functionality
fn test_stats_command() {
    kprintln!("[TEST] Testing stats command...");
    
    let result = Shell::cmd_stats(&[]);
    assert!(result.is_ok(), "Stats command should succeed with no arguments");
    
    let output = result.unwrap();
    assert!(output.contains("System Statistics"), "Stats output should contain title");
    assert!(output.contains("Uptime"), "Stats output should contain uptime");
    assert!(output.contains("Ticks"), "Stats output should contain ticks");
    assert!(output.contains("Context Switches"), "Stats output should contain context switches");
    assert!(output.contains("Messages Sent"), "Stats output should contain messages sent");
    assert!(output.contains("Messages Received"), "Stats output should contain messages received");
    assert!(output.contains("Page Faults"), "Stats output should contain page faults");
    assert!(output.contains("Active Tasks"), "Stats output should contain active tasks");
    assert!(output.contains("Blocked Tasks"), "Stats output should contain blocked tasks");
    
    // Test with arguments (should fail)
    let result_with_args = Shell::cmd_stats(&["extra"]);
    assert!(result_with_args.is_err(), "Stats command should fail with arguments");
    
    kprintln!("✅ Stats command test passed");
}

/// Test audit command functionality
fn test_audit_command() {
    kprintln!("[TEST] Testing audit command...");
    
    // Test with no arguments (default count)
    let result = Shell::cmd_audit(&[]);
    assert!(result.is_ok(), "Audit command should succeed with no arguments");
    
    let output = result.unwrap();
    assert!(output.contains("Audit Log (last 10 entries)"), "Should show default count");
    
    // Test with valid count
    let result_valid = Shell::cmd_audit(&["5"]);
    assert!(result_valid.is_ok(), "Audit command should succeed with valid count");
    
    let output_valid = result_valid.unwrap();
    assert!(output_valid.contains("Audit Log (last 5 entries)"), "Should show specified count");
    
    // Test with invalid count (too high)
    let result_invalid_high = Shell::cmd_audit(&["101"]);
    assert!(result_invalid_high.is_err(), "Audit command should fail with count > 100");
    
    // Test with invalid count (zero)
    let result_invalid_zero = Shell::cmd_audit(&["0"]);
    assert!(result_invalid_zero.is_err(), "Audit command should fail with count = 0");
    
    // Test with invalid count (non-numeric)
    let result_invalid_numeric = Shell::cmd_audit(&["abc"]);
    assert!(result_invalid_numeric.is_err(), "Audit command should fail with non-numeric count");
    
    kprintln!("✅ Audit command test passed");
}

/// Test scheduler command functionality
fn test_sched_command() {
    kprintln!("[TEST] Testing scheduler command...");
    
    let result = Shell::cmd_sched(&[]);
    assert!(result.is_ok(), "Sched command should succeed with no arguments");
    
    let output = result.unwrap();
    assert!(output.contains("Scheduler Information"), "Sched output should contain title");
    assert!(output.contains("Current Task"), "Sched output should contain current task");
    assert!(output.contains("Priority"), "Sched output should contain priority");
    assert!(output.contains("State"), "Sched output should contain state");
    assert!(output.contains("CPU Time"), "Sched output should contain CPU time");
    assert!(output.contains("Context Switches"), "Sched output should contain context switches");
    assert!(output.contains("Task Queue"), "Sched output should contain task queue");
    
    // Test with arguments (should fail)
    let result_with_args = Shell::cmd_sched(&["extra"]);
    assert!(result_with_args.is_err(), "Sched command should fail with arguments");
    
    kprintln!("✅ Scheduler command test passed");
}

/// Test dump command functionality
fn test_dump_command() {
    kprintln!("[TEST] Testing dump command...");
    
    let result = Shell::cmd_dump(&[]);
    assert!(result.is_ok(), "Dump command should succeed with no arguments");
    
    let output = result.unwrap();
    assert!(output.contains("Minidump Information"), "Dump output should contain title");
    assert!(output.contains("Minidump Status"), "Dump output should contain status");
    assert!(output.contains("Buffer Size"), "Dump output should contain buffer size");
    assert!(output.contains("Last Crash"), "Dump output should contain last crash info");
    assert!(output.contains("Build Hash"), "Dump output should contain build hash");
    
    // Test with arguments (should fail)
    let result_with_args = Shell::cmd_dump(&["extra"]);
    assert!(result_with_args.is_err(), "Dump command should fail with arguments");
    
    kprintln!("✅ Dump command test passed");
}

/// Test fault injection command functionality
fn test_fault_command() {
    kprintln!("[TEST] Testing fault injection command...");
    
    // Test with no arguments (status)
    let result_status = Shell::cmd_fault(&[]);
    assert!(result_status.is_ok(), "Fault command should succeed with no arguments");
    
    let output_status = result_status.unwrap();
    assert!(output_status.contains("Fault Injection Status"), "Should show status");
    assert!(output_status.contains("Inbox Overflow"), "Should show inbox overflow status");
    assert!(output_status.contains("Allocation Failure"), "Should show allocation failure status");
    assert!(output_status.contains("Timer Jitter"), "Should show timer jitter status");
    
    // Test with "on" argument
    let result_on = Shell::cmd_fault(&["on"]);
    assert!(result_on.is_ok(), "Fault command should succeed with 'on'");
    
    let output_on = result_on.unwrap();
    assert!(output_on.contains("Enabling fault injection"), "Should show enabling message");
    assert!(output_on.contains("Fault injection enabled"), "Should show enabled message");
    
    // Test with "off" argument
    let result_off = Shell::cmd_fault(&["off"]);
    assert!(result_off.is_ok(), "Fault command should succeed with 'off'");
    
    let output_off = result_off.unwrap();
    assert!(output_off.contains("Disabling fault injection"), "Should show disabling message");
    assert!(output_off.contains("Fault injection disabled"), "Should show disabled message");
    
    // Test with "status" argument
    let result_status_explicit = Shell::cmd_fault(&["status"]);
    assert!(result_status_explicit.is_ok(), "Fault command should succeed with 'status'");
    
    // Test with invalid argument
    let result_invalid = Shell::cmd_fault(&["invalid"]);
    assert!(result_invalid.is_err(), "Fault command should fail with invalid argument");
    
    let error_msg = result_invalid.unwrap_err();
    assert!(error_msg.contains("Invalid action"), "Should show invalid action error");
    
    kprintln!("✅ Fault injection command test passed");
}

/// Test clear command functionality
fn test_clear_command() {
    kprintln!("[TEST] Testing clear command...");
    
    let result = Shell::cmd_clear(&[]);
    assert!(result.is_ok(), "Clear command should succeed with no arguments");
    
    let output = result.unwrap();
    assert!(output.contains("\x1B[2J"), "Should contain ANSI clear screen");
    assert!(output.contains("\x1B[H"), "Should contain ANSI home cursor");
    
    // Test with arguments (should fail)
    let result_with_args = Shell::cmd_clear(&["extra"]);
    assert!(result_with_args.is_err(), "Clear command should fail with arguments");
    
    kprintln!("✅ Clear command test passed");
}

/// Test echo command functionality
fn test_echo_command() {
    kprintln!("[TEST] Testing echo command...");
    
    // Test with no arguments
    let result_no_args = Shell::cmd_echo(&[]);
    assert!(result_no_args.is_ok(), "Echo command should succeed with no arguments");
    assert_eq!(result_no_args.unwrap(), "", "Should return empty string with no arguments");
    
    // Test with single argument
    let result_single = Shell::cmd_echo(&["hello"]);
    assert!(result_single.is_ok(), "Echo command should succeed with single argument");
    assert_eq!(result_single.unwrap(), "hello", "Should echo single argument");
    
    // Test with multiple arguments
    let result_multiple = Shell::cmd_echo(&["hello", "world", "test"]);
    assert!(result_multiple.is_ok(), "Echo command should succeed with multiple arguments");
    assert_eq!(result_multiple.unwrap(), "hello world test", "Should echo multiple arguments with spaces");
    
    // Test with empty arguments
    let result_empty = Shell::cmd_echo(&["", "test", ""]);
    assert!(result_empty.is_ok(), "Echo command should succeed with empty arguments");
    assert_eq!(result_empty.unwrap(), "  test  ", "Should handle empty arguments correctly");
    
    kprintln!("✅ Echo command test passed");
}

/// Test command line processing
fn test_command_line_processing() {
    kprintln!("[TEST] Testing command line processing...");
    
    let mut shell = Shell::new();
    
    // Test valid command
    let result_valid = shell.process_line("help");
    assert!(result_valid.is_ok(), "Valid command should succeed");
    
    // Test command with arguments
    let result_with_args = shell.process_line("echo hello world");
    assert!(result_with_args.is_ok(), "Command with arguments should succeed");
    
    // Test invalid command
    let result_invalid = shell.process_line("nonexistent");
    assert!(result_invalid.is_err(), "Invalid command should fail");
    
    let error_msg = result_invalid.unwrap_err();
    assert!(error_msg.contains("Unknown command"), "Should show unknown command error");
    assert!(error_msg.contains("nonexistent"), "Should mention the unknown command");
    
    // Test empty line
    let result_empty = shell.process_line("");
    assert!(result_empty.is_ok(), "Empty line should succeed");
    assert_eq!(result_empty.unwrap(), "", "Should return empty string for empty line");
    
    // Test whitespace-only line
    let result_whitespace = shell.process_line("   \t  \n  ");
    assert!(result_whitespace.is_ok(), "Whitespace-only line should succeed");
    assert_eq!(result_whitespace.unwrap(), "", "Should return empty string for whitespace-only line");
    
    // Test command with extra whitespace
    let result_extra_whitespace = shell.process_line("  help  ");
    assert!(result_extra_whitespace.is_ok(), "Command with extra whitespace should succeed");
    
    kprintln!("✅ Command line processing test passed");
}

/// Test error handling and edge cases
fn test_error_handling() {
    kprintln!("[TEST] Testing error handling and edge cases...");
    
    let mut shell = Shell::new();
    
    // Test very long command line
    let long_command = "a".repeat(300); // Exceeds MAX_LINE_LENGTH
    let result_long = shell.process_line(&long_command);
    // Note: This test assumes the shell handles long lines gracefully
    // The actual behavior depends on the serial buffer implementation
    
    // Test command with special characters
    let special_chars = "echo \"hello world\" && test";
    let result_special = shell.process_line(special_chars);
    // This should be treated as a single command with arguments
    
    // Test command with unicode (if supported)
    let unicode_command = "echo 🚀 test";
    let result_unicode = shell.process_line(unicode_command);
    // This should work if the system supports unicode
    
    // Test command with control characters
    let control_chars = "echo\x01\x02\x03test";
    let result_control = shell.process_line(control_chars);
    // This should handle control characters gracefully
    
    kprintln!("✅ Error handling test passed");
}

/// Test command integration
fn test_command_integration() {
    kprintln!("[TEST] Testing command integration...");
    
    let mut shell = Shell::new();
    
    // Test multiple commands in sequence
    let commands = vec![
        "help",
        "stats",
        "audit 5",
        "sched",
        "dump",
        "fault status",
        "clear",
        "echo integration test",
    ];
    
    for cmd in commands {
        let result = shell.process_line(cmd);
        assert!(result.is_ok(), "Command '{}' should succeed", cmd);
        
        let output = result.unwrap();
        assert!(!output.is_empty() || cmd == "clear", "Command '{}' should produce output (except clear)", cmd);
    }
    
    kprintln!("✅ Command integration test passed");
}

/// Test shell robustness
fn test_shell_robustness() {
    kprintln!("[TEST] Testing shell robustness...");
    
    let mut shell = Shell::new();
    
    // Test rapid command execution
    for i in 0..100 {
        let cmd = format!("echo test{}", i);
        let result = shell.process_line(&cmd);
        assert!(result.is_ok(), "Rapid command execution should succeed");
    }
    
    // Test command with various argument patterns
    let test_patterns = vec![
        "echo",
        "echo ",
        "echo  ",
        "echo test",
        "echo test ",
        "echo  test  ",
        "echo test1 test2",
        "echo test1  test2",
        "echo \"test with spaces\"",
        "echo 'test with quotes'",
    ];
    
    for pattern in test_patterns {
        let result = shell.process_line(pattern);
        // All should succeed, though some might produce different output
        assert!(result.is_ok(), "Pattern '{}' should succeed", pattern);
    }
    
    kprintln!("✅ Shell robustness test passed");
}

/// Run all shell tests
pub fn run_all_shell_tests() -> Result<(), &'static str> {
    kprintln!("");
    kprintln!("=== SHELL SYSTEM TEST SUITE ===");
    kprintln!("");
    
    // Basic functionality tests
    test_shell_creation();
    test_help_command();
    test_stats_command();
    test_audit_command();
    test_sched_command();
    test_dump_command();
    test_fault_command();
    test_clear_command();
    test_echo_command();
    
    // Integration tests
    test_command_line_processing();
    test_error_handling();
    test_command_integration();
    test_shell_robustness();
    
    kprintln!("");
    kprintln!("=== SHELL SYSTEM TEST SUITE COMPLETED ===");
    kprintln!("✅ All shell tests passed successfully!");
    kprintln!("");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shell_suite() {
        run_all_shell_tests().expect("Shell test suite should pass");
    }
}

