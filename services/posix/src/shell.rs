use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::broker::SyscallBroker;
use crate::vfs::CapabilityAwareVFS;
use crate::shims::PolyglotShims;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub args: Vec<String>,
    pub capabilities: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub duration: Duration,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellState {
    pub current_directory: String,
    pub environment: HashMap<String, String>,
    pub history: Vec<String>,
    pub capabilities: Vec<String>,
}

pub struct AeshShell {
    broker: Arc<SyscallBroker>,
    vfs: Arc<CapabilityAwareVFS>,
    shims: Arc<PolyglotShims>,
    state: Arc<Mutex<ShellState>>,
    builtin_commands: HashMap<String, fn(&AeshShell, &[String]) -> CommandResult>,
}

impl AeshShell {
    pub fn new() -> Self {
        let mut shell = Self {
            broker: Arc::new(SyscallBroker::new()),
            vfs: Arc::new(CapabilityAwareVFS::new()),
            shims: Arc::new(PolyglotShims::new()),
            state: Arc::new(Mutex::new(ShellState {
                current_directory: "/".to_string(),
                environment: HashMap::new(),
                history: Vec::new(),
                capabilities: vec!["posix:basic".to_string(), "filesystem:read".to_string(), "filesystem:write".to_string()],
            })),
            builtin_commands: HashMap::new(),
        };
        
        shell.initialize_builtins();
        shell.initialize_environment();
        shell
    }

    fn initialize_builtins(&mut self) {
        self.builtin_commands.insert("ls".to_string(), AeshShell::cmd_ls);
        self.builtin_commands.insert("cat".to_string(), AeshShell::cmd_cat);
        self.builtin_commands.insert("echo".to_string(), AeshShell::cmd_echo);
        self.builtin_commands.insert("stat".to_string(), AeshShell::cmd_stat);
        self.builtin_commands.insert("cd".to_string(), AeshShell::cmd_cd);
        self.builtin_commands.insert("pwd".to_string(), AeshShell::cmd_pwd);
        self.builtin_commands.insert("mkdir".to_string(), AeshShell::cmd_mkdir);
        self.builtin_commands.insert("rm".to_string(), AeshShell::cmd_rm);
        self.builtin_commands.insert("touch".to_string(), AeshShell::cmd_touch);
        self.builtin_commands.insert("ngfsctl".to_string(), AeshShell::cmd_ngfsctl);
        self.builtin_commands.insert("help".to_string(), AeshShell::cmd_help);
        self.builtin_commands.insert("clear".to_string(), AeshShell::cmd_clear);
    }

    fn initialize_environment(&self) {
        let mut state = self.state.lock().unwrap();
        state.environment.insert("PATH".to_string(), "/bin:/usr/bin:/usr/local/bin".to_string());
        state.environment.insert("HOME".to_string(), "/home/user".to_string());
        state.environment.insert("USER".to_string(), "user".to_string());
        state.environment.insert("SHELL".to_string(), "aesh".to_string());
        state.environment.insert("TERM".to_string(), "xterm-256color".to_string());
    }

    pub fn execute_command(&self, input: &str) -> CommandResult {
        let start = Instant::now();
        let timestamp = self.get_virtual_time();
        
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return CommandResult {
                success: true,
                output: "".to_string(),
                error: None,
                duration: start.elapsed(),
                timestamp,
            };
        }
        
        let command_name = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
        
        let command = Command {
            name: command_name.to_string(),
            args: args.clone(),
            capabilities: self.get_current_capabilities(),
            timestamp,
        };
        
        let result = if let Some(builtin) = self.builtin_commands.get(command_name) {
            builtin(self, &args)
        } else {
            self.execute_external_command(command_name, &args)
        };
        
        let duration = start.elapsed();
        self.add_to_history(input);
        
        CommandResult {
            success: result.success,
            output: result.output,
            error: result.error,
            duration,
            timestamp,
        }
    }

    fn execute_external_command(&self, command: &str, args: &[String]) -> CommandResult {
        CommandResult {
            success: false,
            output: "".to_string(),
            error: Some(format!("Command not found: {}", command)),
            duration: Duration::from_millis(0),
            timestamp: self.get_virtual_time(),
        }
    }

    fn cmd_ls(&self, args: &[String]) -> CommandResult {
        let path = if args.is_empty() { "." } else { &args[0] };
        let full_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("{}/{}", self.get_current_directory(), path)
        };
        
        match self.vfs.list_directory(&full_path, &self.get_current_capabilities()) {
            Ok(entries) => {
                let output: Vec<String> = entries.iter()
                    .map(|entry| {
                        let type_char = match entry.entry_type {
                            crate::vfs::EntryType::Directory => 'd',
                            crate::vfs::EntryType::File => '-',
                            crate::vfs::EntryType::Symlink => 'l',
                            crate::vfs::EntryType::Special => 's',
                        };
                        format!("{}{:o} {} {} {}", 
                            type_char, entry.mode, entry.size, entry.name, 
                            if entry.entry_type == crate::vfs::EntryType::Directory { "/" } else { "" })
                    })
                    .collect();
                
                CommandResult {
                    success: true,
                    output: output.join("\n"),
                    error: None,
                    duration: Duration::from_millis(0),
                    timestamp: self.get_virtual_time(),
                }
            },
            Err(e) => CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(e),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn cmd_cat(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                output: "".to_string(),
                error: Some("cat: missing file operand".to_string()),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            };
        }
        
        let path = &args[0];
        let full_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("{}/{}", self.get_current_directory(), path)
        };
        
        match self.vfs.read_file(&full_path, 0, 1024, &self.get_current_capabilities()) {
            Ok(data) => {
                let output = String::from_utf8_lossy(&data);
                CommandResult {
                    success: true,
                    output: output.to_string(),
                    error: None,
                    duration: Duration::from_millis(0),
                    timestamp: self.get_virtual_time(),
                }
            },
            Err(e) => CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(e),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn cmd_echo(&self, args: &[String]) -> CommandResult {
        let output = args.join(" ");
        CommandResult {
            success: true,
            output,
            error: None,
            duration: Duration::from_millis(0),
            timestamp: self.get_virtual_time(),
        }
    }

    fn cmd_stat(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                output: "".to_string(),
                error: Some("stat: missing file operand".to_string()),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            };
        }
        
        let path = &args[0];
        let full_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("{}/{}", self.get_current_directory(), path)
        };
        
        match self.vfs.stat_file(&full_path, &self.get_current_capabilities()) {
            Ok(file_info) => {
                let output = format!(
                    "File: {}\nSize: {}\nMode: {:o}\nUID: {}\nGID: {}\nCreated: {}\nModified: {}",
                    file_info.path, file_info.size, file_info.mode, 
                    file_info.uid, file_info.gid, file_info.created_at, file_info.modified_at
                );
                
                CommandResult {
                    success: true,
                    output,
                    error: None,
                    duration: Duration::from_millis(0),
                    timestamp: self.get_virtual_time(),
                }
            },
            Err(e) => CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(e),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn cmd_cd(&self, args: &[String]) -> CommandResult {
        let new_dir = if args.is_empty() {
            "/home/user".to_string()
        } else {
            let path = &args[0];
            if path.starts_with('/') {
                path.to_string()
            } else if path == ".." {
                let current = self.get_current_directory();
                let parent = std::path::Path::new(&current).parent().unwrap_or(std::path::Path::new("/"));
                parent.to_str().unwrap().to_string()
            } else if path == "." {
                self.get_current_directory()
            } else {
                format!("{}/{}", self.get_current_directory(), path)
            }
        };
        
        // Check if directory exists
        match self.vfs.list_directory(&new_dir, &self.get_current_capabilities()) {
            Ok(_) => {
                let mut state = self.state.lock().unwrap();
                state.current_directory = new_dir;
                
                CommandResult {
                    success: true,
                    output: "".to_string(),
                    error: None,
                    duration: Duration::from_millis(0),
                    timestamp: self.get_virtual_time(),
                }
            },
            Err(e) => CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(e),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn cmd_pwd(&self, _args: &[String]) -> CommandResult {
        CommandResult {
            success: true,
            output: self.get_current_directory(),
            error: None,
            duration: Duration::from_millis(0),
            timestamp: self.get_virtual_time(),
        }
    }

    fn cmd_mkdir(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                output: "".to_string(),
                error: Some("mkdir: missing directory operand".to_string()),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            };
        }
        
        let path = &args[0];
        let full_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("{}/{}", self.get_current_directory(), path)
        };
        
        match self.vfs.create_directory(&full_path, 0o755, &self.get_current_capabilities()) {
            Ok(_) => CommandResult {
                success: true,
                output: format!("Created directory: {}", full_path),
                error: None,
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
            Err(e) => CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(e),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn cmd_rm(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                output: "".to_string(),
                error: Some("rm: missing file operand".to_string()),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            };
        }
        
        let path = &args[0];
        let full_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("{}/{}", self.get_current_directory(), path)
        };
        
        match self.vfs.remove_file(&full_path, &self.get_current_capabilities()) {
            Ok(_) => CommandResult {
                success: true,
                output: format!("Removed: {}", full_path),
                error: None,
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
            Err(e) => CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(e),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn cmd_touch(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                output: "".to_string(),
                error: Some("touch: missing file operand".to_string()),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            };
        }
        
        let path = &args[0];
        let full_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("{}/{}", self.get_current_directory(), path)
        };
        
        // Create empty file by writing 0 bytes
        match self.vfs.write_file(&full_path, 0, b"", &self.get_current_capabilities()) {
            Ok(_) => CommandResult {
                success: true,
                output: format!("Created: {}", full_path),
                error: None,
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
            Err(e) => CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(e),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn cmd_ngfsctl(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: true,
                output: "NGFS Control Tool\nUsage: ngfsctl <command> [args]\nCommands: status, snapshot, vault, anchor".to_string(),
                error: None,
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            };
        }
        
        let command = &args[0];
        match command.as_str() {
            "status" => CommandResult {
                success: true,
                output: format!(
                    "NGFS Status:\n- VFS Mounts: {}\n- Files: {}\n- Directories: {}\n- Shims: {}\n- Broker Calls: {}",
                    self.vfs.get_mount_count(),
                    self.vfs.get_file_count(),
                    self.vfs.get_directory_count(),
                    self.shims.get_shim_count(),
                    self.broker.get_file_descriptor_count()
                ),
                error: None,
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
            "snapshot" => CommandResult {
                success: true,
                output: "Snapshot operations available".to_string(),
                error: None,
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
            "vault" => CommandResult {
                success: true,
                output: "Personal Data Vault operations available".to_string(),
                error: None,
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
            "anchor" => CommandResult {
                success: true,
                output: "Blockchain anchoring operations available".to_string(),
                error: None,
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
            _ => CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(format!("Unknown ngfsctl command: {}", command)),
                duration: Duration::from_millis(0),
                timestamp: self.get_virtual_time(),
            },
        }
    }

    fn cmd_help(&self, _args: &[String]) -> CommandResult {
        let help_text = r#"Available commands:
  ls [path]           - List directory contents
  cat <file>          - Display file contents
  echo <text>         - Print text
  stat <file>         - Display file status
  cd [directory]      - Change directory
  pwd                 - Print working directory
  mkdir <directory>   - Create directory
  rm <file>           - Remove file
  touch <file>        - Create empty file
  ngfsctl <command>   - NGFS control operations
  help                - Show this help
  clear               - Clear screen
  exit                - Exit shell"#;
        
        CommandResult {
            success: true,
            output: help_text.to_string(),
            error: None,
            duration: Duration::from_millis(0),
            timestamp: self.get_virtual_time(),
        }
    }

    fn cmd_clear(&self, _args: &[String]) -> CommandResult {
        CommandResult {
            success: true,
            output: "\x1B[2J\x1B[1;1H".to_string(),
            error: None,
            duration: Duration::from_millis(0),
            timestamp: self.get_virtual_time(),
        }
    }

    fn get_current_directory(&self) -> String {
        self.state.lock().unwrap().current_directory.clone()
    }

    fn get_current_capabilities(&self) -> Vec<String> {
        self.state.lock().unwrap().capabilities.clone()
    }

    fn add_to_history(&self, command: &str) {
        let mut state = self.state.lock().unwrap();
        state.history.push(command.to_string());
        if state.history.len() > 1000 {
            state.history.remove(0);
        }
    }

    fn get_virtual_time(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn get_prompt(&self) -> String {
        let current_dir = self.get_current_directory();
        format!("aesh:{}> ", current_dir)
    }

    pub fn get_state(&self) -> ShellState {
        self.state.lock().unwrap().clone()
    }

    pub fn get_broker(&self) -> Arc<SyscallBroker> {
        self.broker.clone()
    }

    pub fn get_vfs(&self) -> Arc<CapabilityAwareVFS> {
        self.vfs.clone()
    }

    pub fn get_shims(&self) -> Arc<PolyglotShims> {
        self.shims.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_creation() {
        let shell = AeshShell::new();
        let state = shell.get_state();
        assert_eq!(state.current_directory, "/");
        assert!(!state.capabilities.is_empty());
    }

    #[test]
    fn test_echo_command() {
        let shell = AeshShell::new();
        let result = shell.execute_command("echo Hello World");
        assert!(result.success);
        assert_eq!(result.output, "Hello World");
    }

    #[test]
    fn test_pwd_command() {
        let shell = AeshShell::new();
        let result = shell.execute_command("pwd");
        assert!(result.success);
        assert_eq!(result.output, "/");
    }

    #[test]
    fn test_help_command() {
        let shell = AeshShell::new();
        let result = shell.execute_command("help");
        assert!(result.success);
        assert!(result.output.contains("Available commands"));
    }

    #[test]
    fn test_ngfsctl_status() {
        let shell = AeshShell::new();
        let result = shell.execute_command("ngfsctl status");
        assert!(result.success);
        assert!(result.output.contains("NGFS Status"));
    }

    #[test]
    fn test_invalid_command() {
        let shell = AeshShell::new();
        let result = shell.execute_command("nonexistent_command");
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}

