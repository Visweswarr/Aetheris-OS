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
}
