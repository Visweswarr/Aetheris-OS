use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessHandle {
    pub cap_proc: String,
    pub pid: u32,
    pub parent_cap: Option<String>,
    pub capabilities: Vec<String>,
    pub created_at: u64,
    pub state: ProcessState,
    pub ngfs_snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessState {
    Running,
    Sleeping,
    Zombie,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRequest {
    pub executable: String,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub capabilities: Vec<String>,
    pub ngfs_snapshot: Option<String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResponse {
    pub success: bool,
    pub cap_proc: Option<String>,
    pub pid: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitOptions {
    pub non_blocking: bool,
    pub wait_for_any: bool,
    pub wait_for_stopped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitResult {
    pub pid: u32,
    pub exit_status: i32,
    pub exit_signal: Option<String>,
    pub was_stopped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub state: ProcessState,
    pub capabilities: Vec<String>,
    pub created_at: u64,
    pub cpu_time: u64,
    pub memory_usage: usize,
    pub ngfs_snapshot: Option<String>,
}

pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<String, ProcessHandle>>>,
    pid_counter: Arc<Mutex<u32>>,
    cap_counter: Arc<Mutex<u64>>,
    audit_log: Arc<Mutex<Vec<String>>>,
    virtual_clock: Arc<Mutex<u64>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            pid_counter: Arc::new(Mutex::new(1000)),
            cap_counter: Arc::new(Mutex::new(1)),
            audit_log: Arc::new(Mutex::new(Vec::new())),
            virtual_clock: Arc::new(Mutex::new(0)),
        }
    }

    pub fn spawn(&self, request: SpawnRequest) -> SpawnResponse {
        let start = Instant::now();
        
        // Validate capabilities
        if !self.validate_spawn_capabilities(&request.capabilities) {
            self.log_audit(&format!("SPAWN_DENIED: insufficient capabilities for {}", request.executable));
            return SpawnResponse {
                success: false,
                cap_proc: None,
                pid: None,
                error: Some("Insufficient capabilities".to_string()),
            };
        }

        // Generate capability handle and PID
        let cap_proc = self.generate_cap_handle();
        let pid = self.allocate_pid();
        
        // Create process handle
        let process = ProcessHandle {
            cap_proc: cap_proc.clone(),
            pid,
            parent_cap: None, // TODO: Get from current context
            capabilities: request.capabilities.clone(),
            created_at: self.get_virtual_time(),
            state: ProcessState::Running,
            ngfs_snapshot: request.ngfs_snapshot.clone(),
        };

        // Store process
        self.processes.lock().unwrap().insert(cap_proc.clone(), process);
        
        self.log_audit(&format!("SPAWN_SUCCESS: {} -> cap_proc:{} pid:{}", 
            request.executable, cap_proc, pid));

        SpawnResponse {
            success: true,
            cap_proc: Some(cap_proc),
            pid: Some(pid),
            error: None,
        }
    }

    pub fn fork(&self, parent_cap: &str) -> SpawnResponse {
        let parent = match self.get_process(parent_cap) {
            Ok(p) => p,
            Err(e) => return SpawnResponse {
                success: false,
                cap_proc: None,
                pid: None,
                error: Some(e),
            },
        };

        // Create child process with inherited capabilities
        let child_cap = self.generate_cap_handle();
        let child_pid = self.allocate_pid();
        
        let child = ProcessHandle {
            cap_proc: child_cap.clone(),
            pid: child_pid,
            parent_cap: Some(parent_cap.to_string()),
            capabilities: parent.capabilities.clone(),
            created_at: self.get_virtual_time(),
            state: ProcessState::Running,
            ngfs_snapshot: parent.ngfs_snapshot.clone(),
        };

        self.processes.lock().unwrap().insert(child_cap.clone(), child);
        
        self.log_audit(&format!("FORK_SUCCESS: parent:{} -> child:{} pid:{}", 
            parent_cap, child_cap, child_pid));

        SpawnResponse {
            success: true,
            cap_proc: Some(child_cap),
            pid: Some(child_pid),
            error: None,
        }
    }

    pub fn exec(&self, cap_proc: &str, executable: &str, args: Vec<String>) -> Result<(), String> {
        let mut processes = self.processes.lock().unwrap();
        let process = processes.get_mut(cap_proc)
            .ok_or_else(|| "Invalid process capability".to_string())?;

        // Validate exec capabilities
        if !process.capabilities.contains(&"process:exec".to_string()) {
            return Err("Insufficient exec capability".to_string());
        }

        // Validate executable exists and is executable
        if !self.validate_executable(executable) {
            return Err(format!("Executable not found or not executable: {}", executable));
        }

        // Update process state and metadata (in real implementation, would replace process image)
        process.state = ProcessState::Running;
        
        self.log_audit(&format!("EXEC_SUCCESS: cap_proc:{} -> {} args:{:?}", cap_proc, executable, args));
        Ok(())
    }

    pub fn waitpid(&self, cap_proc: &str, pid: u32, options: WaitOptions) -> Result<WaitResult, String> {
        let processes = self.processes.lock().unwrap();
        
        // Find the target process
        let target_process = processes.values()
            .find(|p| p.pid == pid)
            .ok_or_else(|| "Process not found".to_string())?;

        // Validate wait capabilities
        if !self.validate_wait_capability(cap_proc, target_process) {
            return Err("Insufficient wait capability".to_string());
        }

        // Check if process has terminated
        match target_process.state {
            ProcessState::Terminated | ProcessState::Zombie => {
                let result = WaitResult {
                    pid: target_process.pid,
                    exit_status: 0, // In real implementation, would get actual exit status
                    exit_signal: None,
                    was_stopped: false,
                };
                
                self.log_audit(&format!("WAIT_SUCCESS: cap_proc:{} -> pid:{} status:{:?}", 
                    cap_proc, pid, target_process.state));
                Ok(result)
            },
            _ => {
                if options.non_blocking {
                    Err("Process not ready for wait".to_string())
                } else {
                    // In real implementation, would block until process terminates
                    Ok(WaitResult {
                        pid: target_process.pid,
                        exit_status: 0,
                        exit_signal: None,
                        was_stopped: false,
                    })
                }
            }
        }
    }

    pub fn get_process_info(&self, cap_proc: &str) -> Result<ProcessInfo, String> {
        let process = self.get_process(cap_proc)?;
        
        Ok(ProcessInfo {
            pid: process.pid,
            parent_pid: process.parent_cap.as_ref()
                .and_then(|parent_cap| self.get_process(parent_cap).ok())
                .map(|p| p.pid),
            state: process.state.clone(),
            capabilities: process.capabilities.clone(),
            created_at: process.created_at,
            cpu_time: 0, // In real implementation, would track actual CPU time
            memory_usage: 0, // In real implementation, would track actual memory usage
            ngfs_snapshot: process.ngfs_snapshot.clone(),
        })
    }

    pub fn list_processes_detailed(&self, capabilities: &[String]) -> Result<Vec<ProcessInfo>, String> {
        if !capabilities.contains(&"process:list".to_string()) {
            return Err("Insufficient list capability".to_string());
        }

        let processes = self.processes.lock().unwrap();
        let mut process_infos = Vec::new();
        
        for process in processes.values() {
            let info = ProcessInfo {
                pid: process.pid,
                parent_pid: process.parent_cap.as_ref()
                    .and_then(|parent_cap| processes.get(parent_cap))
                    .map(|p| p.pid),
                state: process.state.clone(),
                capabilities: process.capabilities.clone(),
                created_at: process.created_at,
                cpu_time: 0,
                memory_usage: 0,
                ngfs_snapshot: process.ngfs_snapshot.clone(),
            };
            process_infos.push(info);
        }
        
        Ok(process_infos)
    }

    fn validate_executable(&self, executable: &str) -> bool {
        // In real implementation, would check if file exists and has execute permissions
        // For now, just validate the path format
        executable.starts_with('/') && !executable.is_empty()
    }

    fn validate_wait_capability(&self, waiter_cap: &str, target_process: &ProcessHandle) -> bool {
        // Parent can wait for child, or process with appropriate capabilities
        if let Some(parent_cap) = &target_process.parent_cap {
            if parent_cap == waiter_cap {
                return true;
            }
        }
        
        // Check if waiter has process management capabilities
        if let Ok(waiter_process) = self.get_process(waiter_cap) {
            waiter_process.capabilities.contains(&"process:wait".to_string()) ||
            waiter_process.capabilities.contains(&"all".to_string())
        } else {
            false
        }
    }

    pub fn kill(&self, cap_proc: &str, signal: &str) -> Result<(), String> {
        let mut processes = self.processes.lock().unwrap();
        let process = processes.get_mut(cap_proc)
            .ok_or_else(|| "Invalid process capability".to_string())?;

        match signal {
            "SIGTERM" | "SIGKILL" => {
                process.state = ProcessState::Terminated;
                self.log_audit(&format!("KILL_SUCCESS: cap_proc:{} signal:{}", cap_proc, signal));
                Ok(())
            },
            "SIGSTOP" => {
                process.state = ProcessState::Stopped;
                self.log_audit(&format!("STOP_SUCCESS: cap_proc:{} signal:{}", cap_proc, signal));
                Ok(())
            },
            _ => Err(format!("Unsupported signal: {}", signal)),
        }
    }

    pub fn wait(&self, cap_proc: &str) -> Result<ProcessState, String> {
        let process = self.get_process(cap_proc)?;
        
        // In real implementation, would block until state change
        match process.state {
            ProcessState::Terminated | ProcessState::Zombie => {
                self.log_audit(&format!("WAIT_SUCCESS: cap_proc:{} state:{:?}", cap_proc, process.state));
                Ok(process.state)
            },
            _ => Ok(process.state),
        }
    }

    pub fn list_processes(&self, capabilities: &[String]) -> Result<Vec<ProcessHandle>, String> {
        if !capabilities.contains(&"process:list".to_string()) {
            return Err("Insufficient list capability".to_string());
        }

        let processes = self.processes.lock().unwrap();
        Ok(processes.values().cloned().collect())
    }

    pub fn get_process(&self, cap_proc: &str) -> Result<ProcessHandle, String> {
        self.processes.lock().unwrap()
            .get(cap_proc)
            .cloned()
            .ok_or_else(|| "Invalid process capability".to_string())
    }

    fn validate_spawn_capabilities(&self, capabilities: &[String]) -> bool {
        // Basic capability validation - in real implementation would be more sophisticated
        capabilities.contains(&"process:spawn".to_string()) || 
        capabilities.contains(&"all".to_string())
    }

    fn generate_cap_handle(&self) -> String {
        let mut counter = self.cap_counter.lock().unwrap();
        let cap = format!("cap_proc_{:016x}", *counter);
        *counter += 1;
        cap
    }

    fn allocate_pid(&self) -> u32 {
        let mut counter = self.pid_counter.lock().unwrap();
        let pid = *counter;
        *counter += 1;
        pid
    }

    fn get_virtual_time(&self) -> u64 {
        let mut clock = self.virtual_clock.lock().unwrap();
        *clock += 1;
        *clock
    }

    fn log_audit(&self, message: &str) {
        let mut audit_log = self.audit_log.lock().unwrap();
        audit_log.push(format!("[{}] {}", self.get_virtual_time(), message));
        if audit_log.len() > 1000 {
            audit_log.remove(0);
        }
    }

    pub fn get_audit_log(&self) -> Vec<String> {
        self.audit_log.lock().unwrap().clone()
    }

    pub fn get_process_count(&self) -> usize {
        self.processes.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_manager_creation() {
        let pm = ProcessManager::new();
        assert_eq!(pm.get_process_count(), 0);
    }

    #[test]
    fn test_spawn_process() {
        let pm = ProcessManager::new();
        let request = SpawnRequest {
            executable: "/bin/echo".to_string(),
            args: vec!["hello".to_string()],
            env: vec![],
            capabilities: vec!["process:spawn".to_string()],
            ngfs_snapshot: None,
            working_dir: None,
        };
        
        let response = pm.spawn(request);
        assert!(response.success);
        assert!(response.cap_proc.is_some());
        assert_eq!(pm.get_process_count(), 1);
    }

    #[test]
    fn test_fork_process() {
        let pm = ProcessManager::new();
        
        // First spawn a parent
        let spawn_req = SpawnRequest {
            executable: "/bin/bash".to_string(),
            args: vec![],
            env: vec![],
            capabilities: vec!["process:spawn".to_string(), "process:fork".to_string()],
            ngfs_snapshot: None,
            working_dir: None,
        };
        let parent_resp = pm.spawn(spawn_req);
        let parent_cap = parent_resp.cap_proc.unwrap();
        
        // Then fork
        let fork_resp = pm.fork(&parent_cap);
        assert!(fork_resp.success);
        assert_eq!(pm.get_process_count(), 2);
    }

    #[test]
    fn test_kill_process() {
        let pm = ProcessManager::new();
        
        let spawn_req = SpawnRequest {
            executable: "/bin/sleep".to_string(),
            args: vec!["10".to_string()],
            env: vec![],
            capabilities: vec!["process:spawn".to_string()],
            ngfs_snapshot: None,
            working_dir: None,
        };
        let resp = pm.spawn(spawn_req);
        let cap_proc = resp.cap_proc.unwrap();
        
        let result = pm.kill(&cap_proc, "SIGTERM");
        assert!(result.is_ok());
        
        let process = pm.get_process(&cap_proc).unwrap();
        assert!(matches!(process.state, ProcessState::Terminated));
    }

    #[test]
    fn test_waitpid() {
        let pm = ProcessManager::new();
        
        let spawn_req = SpawnRequest {
            executable: "/bin/test".to_string(),
            args: vec![],
            env: vec![],
            capabilities: vec!["process:spawn".to_string()],
            ngfs_snapshot: None,
            working_dir: None,
        };
        let resp = pm.spawn(spawn_req);
        let cap_proc = resp.cap_proc.unwrap();
        let pid = resp.pid.unwrap();
        
        // Terminate the process first
        pm.kill(&cap_proc, "SIGTERM").unwrap();
        
        let wait_options = WaitOptions {
            non_blocking: true,
            wait_for_any: false,
            wait_for_stopped: false,
        };
        
        let result = pm.waitpid(&cap_proc, pid, wait_options);
        assert!(result.is_ok());
        let wait_result = result.unwrap();
        assert_eq!(wait_result.pid, pid);
    }

    #[test]
    fn test_process_info() {
        let pm = ProcessManager::new();
        
        let spawn_req = SpawnRequest {
            executable: "/bin/echo".to_string(),
            args: vec!["hello".to_string()],
            env: vec![],
            capabilities: vec!["process:spawn".to_string()],
            ngfs_snapshot: Some("snapshot_001".to_string()),
            working_dir: None,
        };
        let resp = pm.spawn(spawn_req);
        let cap_proc = resp.cap_proc.unwrap();
        
        let info = pm.get_process_info(&cap_proc).unwrap();
        assert_eq!(info.pid, resp.pid.unwrap());
        assert_eq!(info.ngfs_snapshot, Some("snapshot_001".to_string()));
        assert!(matches!(info.state, ProcessState::Running));
    }

    #[test]
    fn test_list_processes_detailed() {
        let pm = ProcessManager::new();
        
        // Spawn a few processes
        for i in 0..3 {
            let spawn_req = SpawnRequest {
                executable: format!("/bin/test{}", i),
                args: vec![],
                env: vec![],
                capabilities: vec!["process:spawn".to_string()],
                ngfs_snapshot: None,
                working_dir: None,
            };
            pm.spawn(spawn_req);
        }
        
        let capabilities = vec!["process:list".to_string()];
        let processes = pm.list_processes_detailed(&capabilities).unwrap();
        assert_eq!(processes.len(), 3);
        
        // Check that all processes have valid info
        for process_info in processes {
            assert!(process_info.pid > 0);
            assert!(matches!(process_info.state, ProcessState::Running));
        }
    }
}