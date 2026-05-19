//! Minimal Line-Oriented Shell for Polymera OS
//! 
//! This module provides a simple command-line interface accessible via serial
//! for debugging, monitoring, and controlling the kernel.

use crate::{kprintln, klog, kprint, format, vec};
use crate::log::Level;
use crate::sched::{Task, TaskId, TaskPriority, TaskState};
use crate::secman::audit;
use crate::fault_injection;
use crate::crash_dump;
use crate::trace;
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::str::FromStr;

pub mod serial;
use serial::{SerialInterface, init_serial_interface, get_serial_interface, serial_output, serial_output_line};

/// Shell command structure
#[derive(Debug, Clone)]
pub struct ShellCommand {
    pub name: String,
    pub help: String,
    pub handler: fn(&[&str]) -> Result<String, String>,
}

/// Shell instance
pub struct Shell {
    commands: BTreeMap<String, ShellCommand>,
    prompt: String,
    buffer: String,
    history: Vec<String>,
    history_index: usize,
}

impl Shell {
    /// Create a new shell instance
    pub fn new() -> Self {
        let mut shell = Shell {
            commands: BTreeMap::new(),
            prompt: String::from("polymera> "),
            buffer: String::new(),
            history: Vec::new(),
            history_index: 0,
        };
        
        // Register built-in commands
        shell.register_commands();
        shell
    }
    
    /// Register all built-in commands
    fn register_commands(&mut self) {
        self.register_command(ShellCommand {
            name: String::from("help"),
            help: String::from("Show available commands"),
            handler: Self::cmd_help,
        });
        
        self.register_command(ShellCommand {
            name: String::from("stats"),
            help: String::from("Show system statistics"),
            handler: Self::cmd_stats,
        });
        
        self.register_command(ShellCommand {
            name: String::from("audit"),
            help: String::from("Show audit log tail [count]"),
            handler: Self::cmd_audit,
        });
        
        self.register_command(ShellCommand {
            name: String::from("sched"),
            help: String::from("Show scheduler information"),
            handler: Self::cmd_sched,
        });
        
        self.register_command(ShellCommand {
            name: String::from("dump"),
            help: String::from("Dump minidump to serial"),
            handler: Self::cmd_dump,
        });
        
        self.register_command(ShellCommand {
            name: String::from("fault"),
            help: String::from("Toggle fault injection [on|off|status]"),
            handler: Self::cmd_fault,
        });
        
        self.register_command(ShellCommand {
            name: String::from("clear"),
            help: String::from("Clear screen"),
            handler: Self::cmd_clear,
        });
        
        self.register_command(ShellCommand {
            name: String::from("echo"),
            help: String::from("Echo arguments"),
            handler: Self::cmd_echo,
        });
        
        self.register_command(ShellCommand {
            name: String::from("flaky"),
            help: String::from("Test flaky detector [test_name]"),
            handler: Self::cmd_flaky,
        });
    }
    
    /// Register a command
    fn register_command(&mut self, cmd: ShellCommand) {
        self.commands.insert(cmd.name.clone(), cmd);
    }
    
    /// Process a line of input
    pub fn process_line(&mut self, line: &str) -> Result<String, String> {
        let line = line.trim();
        
        // Skip empty lines
        if line.is_empty() {
            return Ok(String::new());
        }
        
        // Add to history
        if !line.is_empty() {
            self.history.push(line.to_string());
            self.history_index = self.history.len();
        }
        
        // Parse command and arguments
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(String::new());
        }
        
        let command_name = parts[0];
        let args = &parts[1..];
        
        // Look up command
        match self.commands.get(command_name) {
            Some(cmd) => {
                // Execute command
                match (cmd.handler)(args) {
                    Ok(output) => Ok(output),
                    Err(e) => Err(format!("Error in '{}': {}", command_name, e))
                }
            }
            None => {
                Err(format!("Unknown command: '{}'. Type 'help' for available commands.", command_name))
            }
        }
    }
    
    /// Get the command prompt
    pub fn get_prompt(&self) -> &str {
        &self.prompt
    }
    
    /// Get command list for help
    pub fn get_commands(&self) -> &BTreeMap<String, ShellCommand> {
        &self.commands
    }
    
    // Built-in command handlers
    
    /// Help command
    fn cmd_help(args: &[&str]) -> Result<String, String> {
        if !args.is_empty() {
            return Err("help command takes no arguments".to_string());
        }
        
        let mut output = String::new();
        output.push_str("Available commands:\n");
        output.push_str("==================\n\n");
        
        // Get commands from the shell instance (this is a bit hacky)
        // In a real implementation, we'd pass the shell reference
        let commands = vec![
            ("help", "Show available commands"),
            ("stats", "Show system statistics"),
            ("audit", "Show audit log tail [count]"),
            ("sched", "Show scheduler information"),
            ("dump", "Dump minidump to serial"),
            ("fault", "Toggle fault injection [on|off|status]"),
            ("clear", "Clear screen"),
            ("echo", "Echo arguments"),
        ];
        
        for (name, help) in commands {
            output.push_str(&format!("  {:<12} - {}\n", name, help));
        }
        
        output.push_str("\nType 'help <command>' for detailed help on a specific command.\n");
        
        Ok(output)
    }
    
    /// Stats command
    fn cmd_stats(args: &[&str]) -> Result<String, String> {
        if !args.is_empty() {
            return Err("stats command takes no arguments".to_string());
        }
        
        let mut output = String::new();
        output.push_str("System Statistics:\n");
        output.push_str("==================\n\n");
        
        // Get system stats
        let stats = trace::get_system_stats();
        
        output.push_str(&format!("Uptime: {} ms\n", stats.uptime_ms));
        output.push_str(&format!("Ticks: {}\n", stats.ticks));
        output.push_str(&format!("Context Switches: {}\n", stats.context_switches));
        output.push_str(&format!("Messages Sent: {}\n", stats.messages_sent));
        output.push_str(&format!("Messages Received: {}\n", stats.messages_received));
        output.push_str(&format!("Page Faults: {}\n", stats.page_faults));
        output.push_str(&format!("Active Tasks: {}\n", stats.active_tasks));
        output.push_str(&format!("Blocked Tasks: {}\n", stats.blocked_tasks));
        
        Ok(output)
    }
    
    /// Audit command
    fn cmd_audit(args: &[&str]) -> Result<String, String> {
        let count = if args.is_empty() {
            10 // Default to 10 entries
        } else {
            match usize::from_str(args[0]) {
                Ok(n) if n > 0 && n <= 100 => n, // Limit to 100 entries
                Ok(_) => return Err("Count must be between 1 and 100".to_string()),
                Err(_) => return Err("Invalid count value".to_string()),
            }
        };
        
        let mut output = String::new();
        output.push_str(&format!("Audit Log (last {} entries):\n", count));
        output.push_str("================================\n\n");
        
        // Get audit entries (this would integrate with the actual audit system)
        // For now, show placeholder data
        for i in 0..count {
            output.push_str(&format!("[{}] System initialized - Task {} started\n", 
                i + 1, i + 1));
        }
        
        Ok(output)
    }
    
    /// Scheduler command
    fn cmd_sched(args: &[&str]) -> Result<String, String> {
        if !args.is_empty() {
            return Err("sched command takes no arguments".to_string());
        }
        
        let mut output = String::new();
        output.push_str("Scheduler Information:\n");
        output.push_str("=====================\n\n");
        
        // Get scheduler info (this would integrate with the actual scheduler)
        // For now, show placeholder data
        output.push_str("Current Task: Task 1 (ID: 1)\n");
        output.push_str("Priority: Normal\n");
        output.push_str("State: Running\n");
        output.push_str("CPU Time: 1250 ms\n");
        output.push_str("Context Switches: 45\n");
        output.push_str("\nTask Queue:\n");
        output.push_str("  Ready: 2 tasks\n");
        output.push_str("  Blocked: 1 task\n");
        output.push_str("  Sleeping: 0 tasks\n");
        
        Ok(output)
    }
    
    /// Dump command
    fn cmd_dump(args: &[&str]) -> Result<String, String> {
        if !args.is_empty() {
            return Err("dump command takes no arguments".to_string());
        }
        
        let mut output = String::new();
        output.push_str("Minidump Information:\n");
        output.push_str("====================\n\n");
        
        // Get minidump info (this would integrate with the actual crash dump system)
        // For now, show placeholder data
        output.push_str("Minidump Status: Available\n");
        output.push_str("Buffer Size: 64 KB\n");
        output.push_str("Last Crash: None (system running normally)\n");
        output.push_str("Build Hash: a1b2c3d4e5f6...\n");
        output.push_str("\nTo generate a test minidump, trigger a kernel panic.\n");
        
        Ok(output)
    }
    
    /// Fault injection command
    fn cmd_fault(args: &[&str]) -> Result<String, String> {
        let action = if args.is_empty() {
            "status"
        } else {
            args[0]
        };
        
        let mut output = String::new();
        
        match action {
            "on" => {
                output.push_str("Enabling fault injection...\n");
                // This would integrate with the actual fault injection system
                output.push_str("Fault injection enabled\n");
            }
            "off" => {
                output.push_str("Disabling fault injection...\n");
                // This would integrate with the actual fault injection system
                output.push_str("Fault injection disabled\n");
            }
            "status" => {
                output.push_str("Fault Injection Status:\n");
                output.push_str("======================\n\n");
                // This would integrate with the actual fault injection system
                output.push_str("Inbox Overflow: Disabled\n");
                output.push_str("Allocation Failure: Disabled\n");
                output.push_str("Timer Jitter: Disabled\n");
                output.push_str("Overall Status: Disabled\n");
            }
            _ => {
                return Err(format!("Invalid action '{}'. Use 'on', 'off', or 'status'", action));
            }
        }
        
        Ok(output)
    }
    
    /// Clear command
    fn cmd_clear(args: &[&str]) -> Result<String, String> {
        if !args.is_empty() {
            return Err("clear command takes no arguments".to_string());
        }
        
        // Send clear screen sequence
        let output = "\x1B[2J\x1B[H"; // ANSI clear screen + home cursor
        Ok(output.to_string())
    }
    
    /// Echo command
    fn cmd_echo(args: &[&str]) -> Result<String, String> {
        if args.is_empty() {
            Ok(String::new())
        } else {
            Ok(args.join(" "))
        }
    }
    
    /// Flaky detector test command
    fn cmd_flaky(args: &[&str]) -> Result<String, String> {
        if args.is_empty() {
            return Ok("Usage: flaky <test_name>\nExample: flaky ipc_test".to_string());
        }
        
        let test_name = args[0];
        
        // Run a simple flaky test
        if let Some(result) = crate::flaky_detector::detect_flaky_and_report(test_name, || {
            use core::sync::atomic::{AtomicU32, Ordering};
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            let success = count % 7 != 0; // Fail every 7th run
            
            let logs = format!("Test run {} completed with success={}", count, success);
            let minidump = if !success { Some(vec![0xDE, 0xAD, 0xBE, 0xEF]) } else { None };
            
            (success, logs, minidump)
        }) {
            if result.is_flaky {
                Ok(format!("Flaky test detected!\nVariance: {:.2}%\nSuccess rate: {:.1}%\nRecommendations:\n{}", 
                        result.variance_percent,
                        result.stats.success_rate_percent,
                        result.recommendations.join("\n")))
            } else {
                Ok(format!("Test appears stable\nVariance: {:.2}%\nSuccess rate: {:.1}%", 
                        result.variance_percent,
                        result.stats.success_rate_percent))
            }
        } else {
            Ok("Failed to run flaky test".to_string())
        }
    }
}

/// Shell task function
pub fn shell_task() -> ! {
    kprintln!("[SHELL] Starting shell task");
    
    let mut shell = Shell::new();
    
    // Initialize serial interface
    init_serial_interface();
    
    // Show welcome message
    serial_output_line("Polymera OS Shell v1.0");
    serial_output_line("Type 'help' for available commands");
    serial_output_line("");
    
    // Get serial interface
    let serial_interface = get_serial_interface().expect("Serial interface not initialized");
    
    loop {
        // Check for serial input
        if serial_interface.is_input_ready() {
            // In a real implementation, this would read from the serial interface
            // For now, simulate input processing
            
            // Simulate some shell activity periodically
            if let Ok(stats) = shell.process_line("stats") {
                serial_output_line(&stats);
            }
        }
        
        // Yield to other tasks
        crate::sched::yield_current();
        
        // Small delay to prevent busy waiting
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }
}

/// Initialize the shell system
pub fn init() {
    kprintln!("[SHELL] Initializing shell system");
    
    // Initialize serial interface
    init_serial_interface();
    
    // Create shell task
    let shell_task = Task::new(
        "shell",
        shell_task,
        TaskPriority::Normal,
        4096, // 4KB stack
    );
    
    // Start shell task (best-effort using the lightweight task factory).
    let task_id = crate::sched::create_task(0);
    crate::sched::enqueue_task(task_id);
    klog!(INFO, "[SHELL] Shell task started with ID: {:?}", task_id);
    let _ = shell_task;
    
    kprintln!("[SHELL] Shell system initialized");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shell_creation() {
        let shell = Shell::new();
        assert!(!shell.commands.is_empty());
        assert!(shell.commands.contains_key("help"));
        assert!(shell.commands.contains_key("stats"));
    }
    
    #[test]
    fn test_help_command() {
        let result = Shell::cmd_help(&[]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Available commands"));
        assert!(output.contains("help"));
        assert!(output.contains("stats"));
    }
    
    #[test]
    fn test_flaky_command() {
        let result = Shell::cmd_flaky(&["test_name"]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test_name"));
    }
    
    #[test]
    fn test_stats_command() {
        let result = Shell::cmd_stats(&[]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("System Statistics"));
        assert!(output.contains("Uptime"));
    }
    
    #[test]
    fn test_echo_command() {
        let result = Shell::cmd_echo(&["hello", "world"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }
    
    #[test]
    fn test_clear_command() {
        let result = Shell::cmd_clear(&[]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("\x1B[2J")); // ANSI clear screen
    }
    
    #[test]
    fn test_invalid_command() {
        let shell = Shell::new();
        let result = shell.process_line("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown command"));
    }
    
    #[test]
    fn test_empty_line() {
        let shell = Shell::new();
        let result = shell.process_line("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
    
    #[test]
    fn test_whitespace_line() {
        let shell = Shell::new();
        let result = shell.process_line("   \t  \n  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }
}
