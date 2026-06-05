use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallRequest {
    pub syscall: String,
    pub args: Vec<String>,
    pub pid: u32,
    pub timestamp: u64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallResponse {
    pub success: bool,
    pub result: String,
    pub error_code: Option<i32>,
    pub timestamp: u64,
    pub audit_trail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDescriptor {
    pub fd: i32,
    pub path: String,
    pub mode: String,
    pub capabilities: Vec<String>,
    pub created_at: u64,
}

pub struct SyscallBroker {
    file_descriptors: Arc<Mutex<HashMap<i32, FileDescriptor>>>,
    next_fd: Arc<Mutex<i32>>,
    audit_log: Arc<Mutex<Vec<String>>>,
    virtual_clock: Arc<Mutex<u64>>,
    performance_baselines: Arc<Mutex<HashMap<String, Duration>>>,
}

impl SyscallBroker {
    pub fn new() -> Self {
        Self {
            file_descriptors: Arc::new(Mutex::new(HashMap::new())),
            next_fd: Arc::new(Mutex::new(3)),
            audit_log: Arc::new(Mutex::new(Vec::new())),
            virtual_clock: Arc::new(Mutex::new(0)),
            performance_baselines: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn handle_syscall(&self, request: SyscallRequest) -> SyscallResponse {
        let start = Instant::now();
        let mut audit_trail = Vec::new();
        
        audit_trail.push(format!("[{}] PID {}: {} {:?}", 
            request.timestamp, request.pid, request.syscall, request.args));
        
        let result = match request.syscall.as_str() {
            "open" => self.handle_open(&request, &mut audit_trail),
            "read" => self.handle_read(&request, &mut audit_trail),
            "write" => self.handle_write(&request, &mut audit_trail),
            "stat" => self.handle_stat(&request, &mut audit_trail),
            "mmap" => self.handle_mmap(&request, &mut audit_trail),
            "signal" => self.handle_signal(&request, &mut audit_trail),
            "close" => self.handle_close(&request, &mut audit_trail),
            // Advanced POSIX syscalls
            "fork" => self.handle_fork(&request, &mut audit_trail),
            "exec" => self.handle_exec(&request, &mut audit_trail),
            "waitpid" => self.handle_waitpid(&request, &mut audit_trail),
            "kill" => self.handle_kill(&request, &mut audit_trail),
            "getpid" => self.handle_getpid(&request, &mut audit_trail),
            "getppid" => self.handle_getppid(&request, &mut audit_trail),
            "pipe" => self.handle_pipe(&request, &mut audit_trail),
            "msgget" => self.handle_msgget(&request, &mut audit_trail),
            "msgsnd" => self.handle_msgsnd(&request, &mut audit_trail),
            "msgrcv" => self.handle_msgrcv(&request, &mut audit_trail),
            "shmget" => self.handle_shmget(&request, &mut audit_trail),
            "shmat" => self.handle_shmat(&request, &mut audit_trail),
            "pthread_create" => self.handle_pthread_create(&request, &mut audit_trail),
            "pthread_join" => self.handle_pthread_join(&request, &mut audit_trail),
            _ => Err(format!("Unsupported syscall: {}", request.syscall)),
        };
        
        let duration = start.elapsed();
        self.record_performance(&request.syscall, duration);
        
        let response = match result {
            Ok(result) => SyscallResponse {
                success: true,
                result,
                error_code: None,
                timestamp: request.timestamp,
                audit_trail: audit_trail.join(" | "),
            },
            Err(error) => SyscallResponse {
                success: false,
                result: error.clone(),
                error_code: Some(-1),
                timestamp: request.timestamp,
                audit_trail: audit_trail.join(" | "),
            },
        };
        
        self.log_audit(&response.audit_trail);
        response
    }

    fn handle_open(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 2 {
            return Err("open: insufficient arguments".to_string());
        }
        
        let path = &request.args[0];
        let mode = &request.args[1];
        
        if !self.check_capability(&request.capabilities, "filesystem:read") {
            audit_trail.push("DENIED: insufficient filesystem capability".to_string());
            return Err("Permission denied".to_string());
        }
        
        let fd = self.allocate_fd();
        let file_desc = FileDescriptor {
            fd,
            path: path.clone(),
            mode: mode.clone(),
            capabilities: request.capabilities.clone(),
            created_at: request.timestamp,
        };
        
        self.file_descriptors.lock().unwrap().insert(fd, file_desc);
        audit_trail.push(format!("ALLOWED: opened {} as fd {}", path, fd));
        
        Ok(fd.to_string())
    }

    fn handle_read(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 2 {
            return Err("read: insufficient arguments".to_string());
        }
        
        let fd: i32 = request.args[0].parse().map_err(|_| "Invalid fd")?;
        let size: usize = request.args[1].parse().map_err(|_| "Invalid size")?;
        
        let file_desc = self.get_file_descriptor(fd)?;
        
        if !self.check_capability(&request.capabilities, "filesystem:read") {
            audit_trail.push("DENIED: insufficient read capability".to_string());
            return Err("Permission denied".to_string());
        }
        
        audit_trail.push(format!("ALLOWED: read {} bytes from fd {}", size, fd));
        
        let mock_data = "A".repeat(size.min(1024));
        Ok(mock_data)
    }

    fn handle_write(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 2 {
            return Err("write: insufficient arguments".to_string());
        }
        
        let fd: i32 = request.args[0].parse().map_err(|_| "Invalid fd")?;
        let data = &request.args[1];
        
        let file_desc = self.get_file_descriptor(fd)?;
        
        if !self.check_capability(&request.capabilities, "filesystem:write") {
            audit_trail.push("DENIED: insufficient write capability".to_string());
            return Err("Permission denied".to_string());
        }
        
        audit_trail.push(format!("ALLOWED: wrote {} bytes to fd {}", data.len(), fd));
        
        Ok(data.len().to_string())
    }

    fn handle_stat(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 1 {
            return Err("stat: insufficient arguments".to_string());
        }
        
        let path = &request.args[0];
        
        if !self.check_capability(&request.capabilities, "filesystem:read") {
            audit_trail.push("DENIED: insufficient stat capability".to_string());
            return Err("Permission denied".to_string());
        }
        
        audit_trail.push(format!("ALLOWED: stat {}", path));
        
        let mock_stat = format!("{{'path': '{}', 'size': 1024, 'mode': '0644', 'uid': 1000, 'gid': 1000}}", path);
        Ok(mock_stat)
    }

    fn handle_mmap(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 3 {
            return Err("mmap: insufficient arguments".to_string());
        }
        
        let addr = &request.args[0];
        let length: usize = request.args[1].parse().map_err(|_| "Invalid length")?;
        let prot = &request.args[2];
        
        if !self.check_capability(&request.capabilities, "memory:mmap") {
            audit_trail.push("DENIED: insufficient mmap capability".to_string());
            return Err("Permission denied".to_string());
        }
        
        if length > 1024 * 1024 {
            audit_trail.push("DENIED: mmap size too large".to_string());
            return Err("Invalid argument".to_string());
        }
        
        audit_trail.push(format!("ALLOWED: mmap {} bytes at {}", length, addr));
        
        Ok(format!("0x{:x}", 0x10000000))
    }

    fn handle_signal(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 2 {
            return Err("signal: insufficient arguments".to_string());
        }
        
        let pid: u32 = request.args[0].parse().map_err(|_| "Invalid pid")?;
        let signal = &request.args[1];
        
        if !self.check_capability(&request.capabilities, "process:signal") {
            audit_trail.push("DENIED: insufficient signal capability".to_string());
            return Err("Permission denied".to_string());
        }
        
        audit_trail.push(format!("ALLOWED: signal {} to pid {}", signal, pid));
        
        Ok("0".to_string())
    }

    fn handle_close(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 1 {
            return Err("close: insufficient arguments".to_string());
        }
        
        let fd: i32 = request.args[0].parse().map_err(|_| "Invalid fd")?;
        
        if !self.check_capability(&request.capabilities, "filesystem:read") {
            audit_trail.push("DENIED: insufficient close capability".to_string());
            return Err("Permission denied".to_string());
        }
        
        self.file_descriptors.lock().unwrap().remove(&fd);
        audit_trail.push(format!("ALLOWED: closed fd {}", fd));
        
        Ok("0".to_string())
    }

    fn allocate_fd(&self) -> i32 {
        let mut next_fd = self.next_fd.lock().unwrap();
        let fd = *next_fd;
        *next_fd += 1;
        fd
    }

    fn get_file_descriptor(&self, fd: i32) -> Result<FileDescriptor, String> {
        self.file_descriptors.lock().unwrap()
            .get(&fd)
            .cloned()
            .ok_or_else(|| format!("Invalid file descriptor: {}", fd))
    }

    fn check_capability(&self, capabilities: &[String], required: &str) -> bool {
        capabilities.iter().any(|cap| cap == required || cap == "all")
    }

    fn record_performance(&self, syscall: &str, duration: Duration) {
        let mut baselines = self.performance_baselines.lock().unwrap();
        if let Some(baseline) = baselines.get(syscall) {
            if duration > *baseline * 2 {
                self.log_audit(&format!("PERF_WARNING: {} took {:?} (baseline: {:?})", 
                    syscall, duration, baseline));
            }
        } else {
            baselines.insert(syscall.to_string(), duration);
        }
    }

    fn log_audit(&self, message: &str) {
        let mut audit_log = self.audit_log.lock().unwrap();
        audit_log.push(format!("[{}] {}", self.get_virtual_time(), message));
        if audit_log.len() > 1000 {
            audit_log.remove(0);
        }
    }

    fn get_virtual_time(&self) -> u64 {
        let mut clock = self.virtual_clock.lock().unwrap();
        *clock += 1;
        *clock
    }

    pub fn get_audit_log(&self) -> Vec<String> {
        self.audit_log.lock().unwrap().clone()
    }

    pub fn get_performance_baselines(&self) -> HashMap<String, Duration> {
        self.performance_baselines.lock().unwrap().clone()
    }

    pub fn get_file_descriptor_count(&self) -> usize {
        self.file_descriptors.lock().unwrap().len()
    }

    // Advanced POSIX syscall handlers
    fn handle_fork(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if !self.check_capability(&request.capabilities, "process:fork") {
            audit_trail.push("DENIED: insufficient fork capability".to_string());
            return Err("Permission denied".to_string());
        }

        // In real implementation, would call process manager fork
        audit_trail.push("ALLOWED: fork() syscall".to_string());
        Ok("1001".to_string()) // Mock child PID
    }

    fn handle_exec(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 1 {
            return Err("exec: insufficient arguments".to_string());
        }

        let executable = &request.args[0];
        
        if !self.check_capability(&request.capabilities, "process:exec") {
            audit_trail.push("DENIED: insufficient exec capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push(format!("ALLOWED: exec({})", executable));
        Ok("0".to_string())
    }

    fn handle_waitpid(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 2 {
            return Err("waitpid: insufficient arguments".to_string());
        }

        let pid: u32 = request.args[0].parse().map_err(|_| "Invalid pid")?;
        let options: i32 = request.args[1].parse().map_err(|_| "Invalid options")?;

        if !self.check_capability(&request.capabilities, "process:wait") {
            audit_trail.push("DENIED: insufficient wait capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push(format!("ALLOWED: waitpid({}, {})", pid, options));
        Ok(format!("{}:0", pid)) // Mock result: pid:exit_status
    }

    fn handle_kill(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 2 {
            return Err("kill: insufficient arguments".to_string());
        }

        let pid: u32 = request.args[0].parse().map_err(|_| "Invalid pid")?;
        let signal: i32 = request.args[1].parse().map_err(|_| "Invalid signal")?;

        if !self.check_capability(&request.capabilities, "process:signal") {
            audit_trail.push("DENIED: insufficient kill capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push(format!("ALLOWED: kill({}, {})", pid, signal));
        Ok("0".to_string())
    }

    fn handle_getpid(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if !self.check_capability(&request.capabilities, "process:info") {
            audit_trail.push("DENIED: insufficient getpid capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push("ALLOWED: getpid()".to_string());
        Ok(request.pid.to_string())
    }

    fn handle_getppid(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if !self.check_capability(&request.capabilities, "process:info") {
            audit_trail.push("DENIED: insufficient getppid capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push("ALLOWED: getppid()".to_string());
        Ok("1".to_string()) // Mock parent PID
    }

    fn handle_pipe(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if !self.check_capability(&request.capabilities, "ipc:pipe") {
            audit_trail.push("DENIED: insufficient pipe capability".to_string());
            return Err("Permission denied".to_string());
        }

        let read_fd = self.allocate_fd();
        let write_fd = self.allocate_fd();

        // Create pipe file descriptors
        let read_desc = FileDescriptor {
            fd: read_fd,
            path: "pipe_read".to_string(),
            mode: "r".to_string(),
            capabilities: request.capabilities.clone(),
            created_at: request.timestamp,
        };

        let write_desc = FileDescriptor {
            fd: write_fd,
            path: "pipe_write".to_string(),
            mode: "w".to_string(),
            capabilities: request.capabilities.clone(),
            created_at: request.timestamp,
        };

        self.file_descriptors.lock().unwrap().insert(read_fd, read_desc);
        self.file_descriptors.lock().unwrap().insert(write_fd, write_desc);

        audit_trail.push(format!("ALLOWED: pipe() -> [{}, {}]", read_fd, write_fd));
        Ok(format!("{},{}", read_fd, write_fd))
    }

    fn handle_msgget(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 1 {
            return Err("msgget: insufficient arguments".to_string());
        }

        let key: i32 = request.args[0].parse().map_err(|_| "Invalid key")?;

        if !self.check_capability(&request.capabilities, "ipc:msgget") {
            audit_trail.push("DENIED: insufficient msgget capability".to_string());
            return Err("Permission denied".to_string());
        }

        let msgid = key.abs() as u32; // Simple key to ID mapping
        audit_trail.push(format!("ALLOWED: msgget({}) -> {}", key, msgid));
        Ok(msgid.to_string())
    }

    fn handle_msgsnd(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 3 {
            return Err("msgsnd: insufficient arguments".to_string());
        }

        let msgid: u32 = request.args[0].parse().map_err(|_| "Invalid msgid")?;
        let data = &request.args[1];
        let flags: i32 = request.args[2].parse().map_err(|_| "Invalid flags")?;

        if !self.check_capability(&request.capabilities, "ipc:msgsnd") {
            audit_trail.push("DENIED: insufficient msgsnd capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push(format!("ALLOWED: msgsnd({}, {} bytes, {})", msgid, data.len(), flags));
        Ok("0".to_string())
    }

    fn handle_msgrcv(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 3 {
            return Err("msgrcv: insufficient arguments".to_string());
        }

        let msgid: u32 = request.args[0].parse().map_err(|_| "Invalid msgid")?;
        let size: usize = request.args[1].parse().map_err(|_| "Invalid size")?;
        let flags: i32 = request.args[2].parse().map_err(|_| "Invalid flags")?;

        if !self.check_capability(&request.capabilities, "ipc:msgrcv") {
            audit_trail.push("DENIED: insufficient msgrcv capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push(format!("ALLOWED: msgrcv({}, {} bytes, {})", msgid, size, flags));
        let mock_data = "A".repeat(size.min(1024));
        Ok(mock_data)
    }

    fn handle_shmget(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 2 {
            return Err("shmget: insufficient arguments".to_string());
        }

        let key: i32 = request.args[0].parse().map_err(|_| "Invalid key")?;
        let size: usize = request.args[1].parse().map_err(|_| "Invalid size")?;

        if !self.check_capability(&request.capabilities, "memory:shmget") {
            audit_trail.push("DENIED: insufficient shmget capability".to_string());
            return Err("Permission denied".to_string());
        }

        if size > 1024 * 1024 * 16 { // 16MB limit
            audit_trail.push("DENIED: shared memory size too large".to_string());
            return Err("Invalid argument".to_string());
        }

        let shmid = key.abs() as u32; // Simple key to ID mapping
        audit_trail.push(format!("ALLOWED: shmget({}, {} bytes) -> {}", key, size, shmid));
        Ok(shmid.to_string())
    }

    fn handle_shmat(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 1 {
            return Err("shmat: insufficient arguments".to_string());
        }

        let shmid: u32 = request.args[0].parse().map_err(|_| "Invalid shmid")?;

        if !self.check_capability(&request.capabilities, "memory:shmat") {
            audit_trail.push("DENIED: insufficient shmat capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push(format!("ALLOWED: shmat({})", shmid));
        Ok(format!("0x{:x}", 0x20000000)) // Mock shared memory address
    }

    fn handle_pthread_create(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 2 {
            return Err("pthread_create: insufficient arguments".to_string());
        }

        let thread_func = &request.args[0];
        let thread_arg = &request.args[1];

        if !self.check_capability(&request.capabilities, "thread:create") {
            audit_trail.push("DENIED: insufficient pthread_create capability".to_string());
            return Err("Permission denied".to_string());
        }

        let tid = 2000 + request.pid; // Mock thread ID
        audit_trail.push(format!("ALLOWED: pthread_create({}, {}) -> {}", thread_func, thread_arg, tid));
        Ok(tid.to_string())
    }

    fn handle_pthread_join(&self, request: &SyscallRequest, audit_trail: &mut Vec<String>) -> Result<String, String> {
        if request.args.len() < 1 {
            return Err("pthread_join: insufficient arguments".to_string());
        }

        let tid: u32 = request.args[0].parse().map_err(|_| "Invalid tid")?;

        if !self.check_capability(&request.capabilities, "thread:join") {
            audit_trail.push("DENIED: insufficient pthread_join capability".to_string());
            return Err("Permission denied".to_string());
        }

        audit_trail.push(format!("ALLOWED: pthread_join({})", tid));
        Ok("0".to_string()) // Mock return value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_broker_creation() {
        let broker = SyscallBroker::new();
        assert_eq!(broker.get_file_descriptor_count(), 0);
    }

    #[test]
    fn test_open_syscall() {
        let broker = SyscallBroker::new();
        let request = SyscallRequest {
            syscall: "open".to_string(),
            args: vec!["/tmp/test.txt".to_string(), "r".to_string()],
            pid: 1000,
            timestamp: 1234567890,
            capabilities: vec!["filesystem:read".to_string()],
        };
        
        let response = broker.handle_syscall(request);
        assert!(response.success);
        assert_eq!(broker.get_file_descriptor_count(), 1);
    }

    #[test]
    fn test_open_syscall_denied() {
        let broker = SyscallBroker::new();
        let request = SyscallRequest {
            syscall: "open".to_string(),
            args: vec!["/tmp/test.txt".to_string(), "r".to_string()],
            pid: 1000,
            timestamp: 1234567890,
            capabilities: vec!["network:connect".to_string()],
        };
        
        let response = broker.handle_syscall(request);
        assert!(!response.success);
        assert_eq!(broker.get_file_descriptor_count(), 0);
    }

    #[test]
    fn test_read_syscall() {
        let broker = SyscallBroker::new();
        
        let open_request = SyscallRequest {
            syscall: "open".to_string(),
            args: vec!["/tmp/test.txt".to_string(), "r".to_string()],
            pid: 1000,
            timestamp: 1234567890,
            capabilities: vec!["filesystem:read".to_string()],
        };
        let open_response = broker.handle_syscall(open_request);
        let fd = open_response.result;
        
        let read_request = SyscallRequest {
            syscall: "read".to_string(),
            args: vec![fd, "100".to_string()],
            pid: 1000,
            timestamp: 1234567890,
            capabilities: vec!["filesystem:read".to_string()],
        };
        
        let read_response = broker.handle_syscall(read_request);
        assert!(read_response.success);
        assert_eq!(read_response.result.len(), 100);
    }

    #[test]
    fn test_fork_syscall() {
        let broker = SyscallBroker::new();
        let request = SyscallRequest {
            syscall: "fork".to_string(),
            args: vec![],
            pid: 1000,
            timestamp: 1234567890,
            capabilities: vec!["process:fork".to_string()],
        };
        
        let response = broker.handle_syscall(request);
        assert!(response.success);
        assert_eq!(response.result, "1001");
    }

    #[test]
    fn test_pipe_syscall() {
        let broker = SyscallBroker::new();
        let request = SyscallRequest {
            syscall: "pipe".to_string(),
            args: vec![],
            pid: 1000,
            timestamp: 1234567890,
            capabilities: vec!["ipc:pipe".to_string()],
        };
        
        let response = broker.handle_syscall(request);
        assert!(response.success);
        assert!(response.result.contains(','));
        
        let fds: Vec<&str> = response.result.split(',').collect();
        assert_eq!(fds.len(), 2);
        assert_eq!(broker.get_file_descriptor_count(), 2);
    }

    #[test]
    fn test_pthread_create_syscall() {
        let broker = SyscallBroker::new();
        let request = SyscallRequest {
            syscall: "pthread_create".to_string(),
            args: vec!["worker_func".to_string(), "arg1".to_string()],
            pid: 1000,
            timestamp: 1234567890,
            capabilities: vec!["thread:create".to_string()],
        };
        
        let response = broker.handle_syscall(request);
        assert!(response.success);
        assert_eq!(response.result, "3000"); // 2000 + 1000
    }

    #[test]
    fn test_denied_syscall() {
        let broker = SyscallBroker::new();
        let request = SyscallRequest {
            syscall: "fork".to_string(),
            args: vec![],
            pid: 1000,
            timestamp: 1234567890,
            capabilities: vec!["filesystem:read".to_string()], // Wrong capability
        };
        
        let response = broker.handle_syscall(request);
        assert!(!response.success);
        assert!(response.result.contains("Permission denied"));
    }
}
