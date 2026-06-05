use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadHandle {
    pub cap_thread: String,
    pub tid: u32,
    pub process_cap: String,
    pub state: ThreadState,
    pub priority: u8,
    pub capabilities: Vec<String>,
    pub created_at: u64,
    pub cpu_quota: ThreadQuota,
    pub stack_size: usize,
    pub detached: bool,
    pub cpu_time: u64,
    pub exit_status: i32,
    pub cpu_affinity: Option<u64>,
    pub scheduling_policy: SchedulingPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub thread_id: String,
    pub process_cap: String,
    pub state: ThreadState,
    pub priority: u8,
    pub cpu_quota: ThreadQuota,
    pub detached: bool,
    pub created_at: u64,
    pub cpu_time: u64,
    pub exit_status: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreadState {
    Ready,
    Running,
    Blocked,
    Sleeping,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadQuota {
    pub cpu_time_ms: u64,
    pub max_cpu_time_ms: u64,
    pub memory_bytes: usize,
    pub max_memory_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCreateRequest {
    pub process_cap: String,
    pub entry_point: String,
    pub stack_size: Option<usize>,
    pub priority: Option<u8>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingPolicy {
    pub policy_type: PolicyType,
    pub time_slice_ms: u64,
    pub priority_levels: u8,
    pub quota_enforcement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyType {
    RoundRobin,
    PriorityBased,
    Deterministic,
    RealTime,
}

pub struct ThreadManager {
    threads: Arc<Mutex<HashMap<String, ThreadHandle>>>,
    process_threads: Arc<Mutex<HashMap<String, Vec<String>>>>, // process_cap -> thread_caps
    tid_counter: Arc<Mutex<u32>>,
    cap_counter: Arc<Mutex<u64>>,
    scheduling_policy: Arc<Mutex<SchedulingPolicy>>,
    virtual_clock: Arc<Mutex<u64>>,
    audit_log: Arc<Mutex<Vec<String>>>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            threads: Arc::new(Mutex::new(HashMap::new())),
            process_threads: Arc::new(Mutex::new(HashMap::new())),
            tid_counter: Arc::new(Mutex::new(1)),
            cap_counter: Arc::new(Mutex::new(1)),
            scheduling_policy: Arc::new(Mutex::new(SchedulingPolicy {
                policy_type: PolicyType::Deterministic,
                time_slice_ms: 10,
                priority_levels: 8,
                quota_enforcement: true,
            })),
            virtual_clock: Arc::new(Mutex::new(0)),
            audit_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn create_thread(&self, request: ThreadCreateRequest) -> Result<String, String> {
        // Validate thread creation capabilities
        if !request.capabilities.contains(&"thread:create".to_string()) {
            return Err("Insufficient thread creation capability".to_string());
        }

        // Check process thread limits
        let process_thread_count = self.get_process_thread_count(&request.process_cap);
        if process_thread_count >= 64 { // Max 64 threads per process
            return Err("Process thread limit exceeded".to_string());
        }

        let cap_thread = self.generate_thread_cap();
        let tid = self.allocate_tid();
        let stack_size = request.stack_size.unwrap_or(8192); // 8KB default stack
        let priority = request.priority.unwrap_or(4); // Default priority

        let thread = ThreadHandle {
            cap_thread: cap_thread.clone(),
            tid,
            process_cap: request.process_cap.clone(),
            state: ThreadState::Ready,
            priority,
            capabilities: request.capabilities.clone(),
            created_at: self.get_virtual_time(),
            cpu_quota: ThreadQuota {
                cpu_time_ms: 0,
                max_cpu_time_ms: 1000, // 1 second default quota
                memory_bytes: stack_size,
                max_memory_bytes: stack_size * 2,
            },
            stack_size,
            detached: false,
            cpu_time: 0,
            exit_status: 0,
            cpu_affinity: None,
            scheduling_policy: SchedulingPolicy {
                policy_type: PolicyType::RoundRobin,
                time_slice_ms: 10,
                priority_levels: 8,
                quota_enforcement: true,
            },
        };

        // Store thread
        self.threads.lock().unwrap().insert(cap_thread.clone(), thread);
        
        // Add to process thread list
        self.process_threads.lock().unwrap()
            .entry(request.process_cap.clone())
            .or_insert_with(Vec::new)
            .push(cap_thread.clone());

        self.log_audit(&format!("THREAD_CREATED: {} in process {} (tid: {})", 
            cap_thread, request.process_cap, tid));

        Ok(cap_thread)
    }

    pub fn start_thread(&self, cap_thread: &str) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Invalid thread capability".to_string())?;

        match thread.state {
            ThreadState::Ready => {
                thread.state = ThreadState::Running;
                self.log_audit(&format!("THREAD_STARTED: {}", cap_thread));
                Ok(())
            },
            _ => Err(format!("Thread not in ready state: {:?}", thread.state)),
        }
    }

    pub fn suspend_thread(&self, cap_thread: &str) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Invalid thread capability".to_string())?;

        match thread.state {
            ThreadState::Running => {
                thread.state = ThreadState::Blocked;
                self.log_audit(&format!("THREAD_SUSPENDED: {}", cap_thread));
                Ok(())
            },
            _ => Err(format!("Thread not running: {:?}", thread.state)),
        }
    }

    pub fn resume_thread(&self, cap_thread: &str) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Invalid thread capability".to_string())?;

        match thread.state {
            ThreadState::Blocked | ThreadState::Sleeping => {
                thread.state = ThreadState::Running;
                self.log_audit(&format!("THREAD_RESUMED: {}", cap_thread));
                Ok(())
            },
            _ => Err(format!("Thread not suspended: {:?}", thread.state)),
        }
    }

    pub fn terminate_thread(&self, cap_thread: &str) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Invalid thread capability".to_string())?;

        thread.state = ThreadState::Terminated;
        
        // Remove from process thread list
        let process_cap = thread.process_cap.clone();
        drop(threads);
        
        let mut process_threads = self.process_threads.lock().unwrap();
        if let Some(thread_list) = process_threads.get_mut(&process_cap) {
            thread_list.retain(|cap| cap != cap_thread);
        }

        self.log_audit(&format!("THREAD_TERMINATED: {}", cap_thread));
        Ok(())
    }

    pub fn join_thread(&self, cap_thread: &str, requester_cap: &str) -> Result<i32, String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Thread not found".to_string())?;

        if !thread.capabilities.contains(&"thread:join".to_string()) {
            return Err("Insufficient thread join capability".to_string());
        }

        // In real implementation, would block until thread terminates
        // For now, return the thread's exit status
        match thread.state {
            ThreadState::Terminated => {
                self.log_audit(&format!("THREAD_JOINED: {} by {} (exit: {})", 
                    cap_thread, requester_cap, thread.exit_status));
                Ok(thread.exit_status)
            },
            _ => {
                // Thread still running, would block in real implementation
                self.log_audit(&format!("THREAD_JOIN_BLOCKED: {} by {}", cap_thread, requester_cap));
                Ok(0) // Mock return for running thread
            }
        }
    }

    pub fn detach_thread(&self, cap_thread: &str, requester_cap: &str) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Thread not found".to_string())?;

        if !thread.capabilities.contains(&"thread:detach".to_string()) {
            return Err("Insufficient thread detach capability".to_string());
        }

        thread.detached = true;
        self.log_audit(&format!("THREAD_DETACHED: {} by {}", cap_thread, requester_cap));
        Ok(())
    }

    pub fn get_thread_info(&self, cap_thread: &str) -> Result<ThreadInfo, String> {
        let threads = self.threads.lock().unwrap();
        let thread = threads.get(cap_thread)
            .ok_or_else(|| "Thread not found".to_string())?;

        Ok(ThreadInfo {
            thread_id: cap_thread.to_string(),
            process_cap: thread.process_cap.clone(),
            state: thread.state.clone(),
            priority: thread.priority,
            cpu_quota: thread.cpu_quota.clone(),
            detached: thread.detached,
            created_at: thread.created_at,
            cpu_time: thread.cpu_time,
            exit_status: thread.exit_status,
        })
    }

    pub fn list_threads(&self, process_cap: &str, requester_cap: &str) -> Result<Vec<ThreadInfo>, String> {
        if !self.check_thread_list_capability(requester_cap) {
            return Err("Insufficient thread list capability".to_string());
        }

        let threads = self.threads.lock().unwrap();
        let mut thread_infos = Vec::new();

        for (thread_id, thread) in threads.iter() {
            if thread.process_cap == process_cap {
                thread_infos.push(ThreadInfo {
                    thread_id: thread_id.clone(),
                    process_cap: thread.process_cap.clone(),
                    state: thread.state.clone(),
                    priority: thread.priority,
                    cpu_quota: thread.cpu_quota.clone(),
                    detached: thread.detached,
                    created_at: thread.created_at,
                    cpu_time: thread.cpu_time,
                    exit_status: thread.exit_status,
                });
            }
        }

        Ok(thread_infos)
    }

    pub fn set_thread_affinity(&self, cap_thread: &str, cpu_mask: u64, requester_cap: &str) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Thread not found".to_string())?;

        if !thread.capabilities.contains(&"thread:affinity".to_string()) {
            return Err("Insufficient thread affinity capability".to_string());
        }

        thread.cpu_affinity = Some(cpu_mask);
        self.log_audit(&format!("THREAD_AFFINITY_SET: {} by {} (mask: 0x{:x})", 
            cap_thread, requester_cap, cpu_mask));
        Ok(())
    }

    pub fn get_thread_affinity(&self, cap_thread: &str) -> Result<u64, String> {
        let threads = self.threads.lock().unwrap();
        let thread = threads.get(cap_thread)
            .ok_or_else(|| "Thread not found".to_string())?;

        Ok(thread.cpu_affinity.unwrap_or(0xFFFFFFFFFFFFFFFF)) // Default to all CPUs
    }

    pub fn set_thread_scheduling_policy(&self, cap_thread: &str, policy: SchedulingPolicy, requester_cap: &str) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Thread not found".to_string())?;

        if !thread.capabilities.contains(&"thread:sched".to_string()) {
            return Err("Insufficient thread scheduling capability".to_string());
        }

        thread.scheduling_policy = policy;
        self.log_audit(&format!("THREAD_SCHED_POLICY_SET: {} by {} ({:?})", 
            cap_thread, requester_cap, thread.scheduling_policy));
        Ok(())
    }

    pub fn get_thread_scheduling_policy(&self, cap_thread: &str) -> Result<SchedulingPolicy, String> {
        let threads = self.threads.lock().unwrap();
        let thread = threads.get(cap_thread)
            .ok_or_else(|| "Thread not found".to_string())?;

        Ok(thread.scheduling_policy.clone())
    }

    pub fn get_thread_statistics(&self) -> HashMap<String, usize> {
        let threads = self.threads.lock().unwrap();
        let mut stats = HashMap::new();
        
        let mut running_count = 0;
        let mut sleeping_count = 0;
        let mut terminated_count = 0;
        let mut detached_count = 0;
        
        for thread in threads.values() {
            match thread.state {
                ThreadState::Running => running_count += 1,
                ThreadState::Sleeping => sleeping_count += 1,
                ThreadState::Terminated => terminated_count += 1,
                _ => {}
            }
            
            if thread.detached {
                detached_count += 1;
            }
        }
        
        stats.insert("total_threads".to_string(), threads.len());
        stats.insert("running_threads".to_string(), running_count);
        stats.insert("sleeping_threads".to_string(), sleeping_count);
        stats.insert("terminated_threads".to_string(), terminated_count);
        stats.insert("detached_threads".to_string(), detached_count);
        
        stats
    }

    pub fn set_thread_priority(&self, cap_thread: &str, priority: u8) -> Result<(), String> {
        if priority > 7 {
            return Err("Invalid priority level".to_string());
        }

        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Invalid thread capability".to_string())?;

        thread.priority = priority;
        self.log_audit(&format!("THREAD_PRIORITY_SET: {} -> {}", cap_thread, priority));
        Ok(())
    }

    pub fn update_thread_quota(&self, cap_thread: &str, cpu_time_ms: u64) -> Result<(), String> {
        let mut threads = self.threads.lock().unwrap();
        let thread = threads.get_mut(cap_thread)
            .ok_or_else(|| "Invalid thread capability".to_string())?;

        thread.cpu_quota.cpu_time_ms += cpu_time_ms;
        
        // Check quota enforcement
        let policy = self.scheduling_policy.lock().unwrap();
        if policy.quota_enforcement && thread.cpu_quota.cpu_time_ms > thread.cpu_quota.max_cpu_time_ms {
            thread.state = ThreadState::Blocked;
            self.log_audit(&format!("THREAD_QUOTA_EXCEEDED: {} ({}ms > {}ms)", 
                cap_thread, thread.cpu_quota.cpu_time_ms, thread.cpu_quota.max_cpu_time_ms));
            return Err("Thread CPU quota exceeded".to_string());
        }

        Ok(())
    }

    pub fn get_thread(&self, cap_thread: &str) -> Result<ThreadHandle, String> {
        self.threads.lock().unwrap()
            .get(cap_thread)
            .cloned()
            .ok_or_else(|| "Invalid thread capability".to_string())
    }

    pub fn list_process_threads(&self, process_cap: &str) -> Vec<String> {
        self.process_threads.lock().unwrap()
            .get(process_cap)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_process_thread_count(&self, process_cap: &str) -> usize {
        self.process_threads.lock().unwrap()
            .get(process_cap)
            .map(|threads| threads.len())
            .unwrap_or(0)
    }

    pub fn cleanup_process_threads(&self, process_cap: &str) -> usize {
        let thread_caps = self.list_process_threads(process_cap);
        let count = thread_caps.len();
        
        // Terminate all threads for the process
        for cap_thread in thread_caps {
            let _ = self.terminate_thread(&cap_thread);
        }
        
        // Remove process from thread mapping
        self.process_threads.lock().unwrap().remove(process_cap);
        
        self.log_audit(&format!("PROCESS_THREADS_CLEANUP: {} ({} threads)", process_cap, count));
        count
    }

    fn generate_thread_cap(&self) -> String {
        let mut counter = self.cap_counter.lock().unwrap();
        let cap = format!("cap_thread_{:016x}", *counter);
        *counter += 1;
        cap
    }

    fn allocate_tid(&self) -> u32 {
        let mut counter = self.tid_counter.lock().unwrap();
        let tid = *counter;
        *counter += 1;
        tid
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

    fn check_thread_list_capability(&self, requester_cap: &str) -> bool {
        // In real implementation, would check actual capabilities
        requester_cap.contains("thread:list") || requester_cap.contains("all")
    }

    pub fn get_audit_log(&self) -> Vec<String> {
        self.audit_log.lock().unwrap().clone()
    }

    pub fn get_thread_count(&self) -> usize {
        self.threads.lock().unwrap().len()
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        let threads = self.threads.lock().unwrap();
        let mut stats = HashMap::new();
        
        stats.insert("total_threads".to_string(), threads.len());
        
        let mut state_counts = HashMap::new();
        for thread in threads.values() {
            let state_name = format!("{:?}", thread.state);
            *state_counts.entry(state_name).or_insert(0) += 1;
        }
        
        for (state, count) in state_counts {
            stats.insert(format!("threads_{}", state.to_lowercase()), count);
        }
        
        stats
    }
}

// Deterministic scheduler for testing
pub struct DeterministicScheduler {
    thread_manager: Arc<ThreadManager>,
    current_thread: Arc<Mutex<Option<String>>>,
    time_slice_counter: Arc<Mutex<u64>>,
    virtual_clock: Arc<Mutex<u64>>,
}

impl DeterministicScheduler {
    pub fn new(thread_manager: Arc<ThreadManager>) -> Self {
        Self {
            thread_manager,
            current_thread: Arc::new(Mutex::new(None)),
            time_slice_counter: Arc::new(Mutex::new(0)),
            virtual_clock: Arc::new(Mutex::new(0)),
        }
    }

    pub fn schedule(&self) -> Option<String> {
        let threads = self.thread_manager.threads.lock().unwrap();
        
        // Simple round-robin scheduling for deterministic behavior
        let ready_threads: Vec<_> = threads.iter()
            .filter(|(_, thread)| matches!(thread.state, ThreadState::Ready | ThreadState::Running))
            .collect();

        if ready_threads.is_empty() {
            return None;
        }

        let mut counter = self.time_slice_counter.lock().unwrap();
        let index = (*counter as usize) % ready_threads.len();
        *counter += 1;

        let selected_thread = ready_threads[index].0.clone();
        *self.current_thread.lock().unwrap() = Some(selected_thread.clone());
        
        Some(selected_thread)
    }

    pub fn tick(&self) -> Option<String> {
        self.advance_time(1);
        
        // Update current thread CPU quota
        if let Some(current_cap) = self.current_thread.lock().unwrap().as_ref() {
            let _ = self.thread_manager.update_thread_quota(current_cap, 1);
        }
        
        // Schedule next thread
        self.schedule()
    }

    pub fn advance_time(&self, ms: u64) {
        let mut clock = self.virtual_clock.lock().unwrap();
        *clock += ms;
    }

    pub fn get_time(&self) -> u64 {
        *self.virtual_clock.lock().unwrap()
    }

    pub fn get_current_thread(&self) -> Option<String> {
        self.current_thread.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_manager_creation() {
        let tm = ThreadManager::new();
        assert_eq!(tm.get_thread_count(), 0);
    }

    #[test]
    fn test_create_thread() {
        let tm = ThreadManager::new();
        let request = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "main".to_string(),
            stack_size: Some(16384),
            priority: Some(5),
            capabilities: vec!["thread:create".to_string()],
        };

        let cap_thread = tm.create_thread(request).unwrap();
        assert_eq!(tm.get_thread_count(), 1);
        
        let thread = tm.get_thread(&cap_thread).unwrap();
        assert_eq!(thread.priority, 5);
        assert_eq!(thread.stack_size, 16384);
    }

    #[test]
    fn test_thread_lifecycle() {
        let tm = ThreadManager::new();
        let request = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "worker".to_string(),
            stack_size: None,
            priority: None,
            capabilities: vec!["thread:create".to_string()],
        };

        let cap_thread = tm.create_thread(request).unwrap();
        
        // Start thread
        assert!(tm.start_thread(&cap_thread).is_ok());
        let thread = tm.get_thread(&cap_thread).unwrap();
        assert!(matches!(thread.state, ThreadState::Running));
        
        // Suspend thread
        assert!(tm.suspend_thread(&cap_thread).is_ok());
        let thread = tm.get_thread(&cap_thread).unwrap();
        assert!(matches!(thread.state, ThreadState::Blocked));
        
        // Resume thread
        assert!(tm.resume_thread(&cap_thread).is_ok());
        let thread = tm.get_thread(&cap_thread).unwrap();
        assert!(matches!(thread.state, ThreadState::Running));
        
        // Terminate thread
        assert!(tm.terminate_thread(&cap_thread).is_ok());
        let thread = tm.get_thread(&cap_thread).unwrap();
        assert!(matches!(thread.state, ThreadState::Terminated));
    }

    #[test]
    fn test_deterministic_scheduler() {
        let tm = Arc::new(ThreadManager::new());
        let scheduler = DeterministicScheduler::new(tm.clone());
        
        // Create some threads
        for i in 0..3 {
            let request = ThreadCreateRequest {
                process_cap: format!("cap_proc_{:03}", i),
                entry_point: "worker".to_string(),
                stack_size: None,
                priority: None,
                capabilities: vec!["thread:create".to_string()],
            };
            let cap_thread = tm.create_thread(request).unwrap();
            tm.start_thread(&cap_thread).unwrap();
        }
        
        // Test scheduling
        let scheduled1 = scheduler.schedule();
        assert!(scheduled1.is_some());
        
        let scheduled2 = scheduler.schedule();
        assert!(scheduled2.is_some());
        assert_ne!(scheduled1, scheduled2); // Should be different due to round-robin
    }

    #[test]
    fn test_thread_join() {
        let tm = ThreadManager::new();
        let request = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "worker".to_string(),
            stack_size: None,
            priority: None,
            capabilities: vec!["thread:create".to_string(), "thread:join".to_string()],
        };

        let cap_thread = tm.create_thread(request).unwrap();
        
        // Test joining a running thread (should return 0 in mock)
        let exit_status = tm.join_thread(&cap_thread, "cap_proc_001").unwrap();
        assert_eq!(exit_status, 0);
        
        // Terminate thread and test joining terminated thread
        tm.terminate_thread(&cap_thread).unwrap();
        let exit_status = tm.join_thread(&cap_thread, "cap_proc_001").unwrap();
        assert_eq!(exit_status, 0); // Default exit status
    }

    #[test]
    fn test_thread_detach() {
        let tm = ThreadManager::new();
        let request = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "worker".to_string(),
            stack_size: None,
            priority: None,
            capabilities: vec!["thread:create".to_string(), "thread:detach".to_string()],
        };

        let cap_thread = tm.create_thread(request).unwrap();
        
        let result = tm.detach_thread(&cap_thread, "cap_proc_001");
        assert!(result.is_ok());
        
        let thread = tm.get_thread(&cap_thread).unwrap();
        assert!(thread.detached);
    }

    #[test]
    fn test_thread_info() {
        let tm = ThreadManager::new();
        let request = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "worker".to_string(),
            stack_size: Some(16384),
            priority: Some(6),
            capabilities: vec!["thread:create".to_string()],
        };

        let cap_thread = tm.create_thread(request).unwrap();
        
        let info = tm.get_thread_info(&cap_thread).unwrap();
        assert_eq!(info.thread_id, cap_thread);
        assert_eq!(info.process_cap, "cap_proc_001");
        assert_eq!(info.priority, 6);
        assert!(!info.detached);
    }

    #[test]
    fn test_list_threads() {
        let tm = ThreadManager::new();
        let capabilities = vec!["thread:create".to_string(), "thread:list".to_string()];
        
        // Create multiple threads
        for i in 0..3 {
            let request = ThreadCreateRequest {
                process_cap: "cap_proc_001".to_string(),
                entry_point: format!("worker_{}", i),
                stack_size: None,
                priority: None,
                capabilities: capabilities.clone(),
            };
            tm.create_thread(request).unwrap();
        }
        
        let threads = tm.list_threads("cap_proc_001", "cap_proc_001").unwrap();
        assert_eq!(threads.len(), 3);
    }

    #[test]
    fn test_thread_affinity() {
        let tm = ThreadManager::new();
        let request = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "worker".to_string(),
            stack_size: None,
            priority: None,
            capabilities: vec!["thread:create".to_string(), "thread:affinity".to_string()],
        };

        let cap_thread = tm.create_thread(request).unwrap();
        
        let cpu_mask = 0x0F; // CPUs 0-3
        let result = tm.set_thread_affinity(&cap_thread, cpu_mask, "cap_proc_001");
        assert!(result.is_ok());
        
        let affinity = tm.get_thread_affinity(&cap_thread).unwrap();
        assert_eq!(affinity, cpu_mask);
    }

    #[test]
    fn test_thread_scheduling_policy() {
        let tm = ThreadManager::new();
        let request = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "worker".to_string(),
            stack_size: None,
            priority: None,
            capabilities: vec!["thread:create".to_string(), "thread:sched".to_string()],
        };

        let cap_thread = tm.create_thread(request).unwrap();
        
        let policy = SchedulingPolicy {
            policy_type: PolicyType::FIFO,
            time_slice_ms: 5,
            priority_levels: 16,
            quota_enforcement: false,
        };
        
        let result = tm.set_thread_scheduling_policy(&cap_thread, policy.clone(), "cap_proc_001");
        assert!(result.is_ok());
        
        let retrieved_policy = tm.get_thread_scheduling_policy(&cap_thread).unwrap();
        assert_eq!(retrieved_policy.policy_type, PolicyType::FIFO);
        assert_eq!(retrieved_policy.time_slice_ms, 5);
    }

    #[test]
    fn test_thread_statistics() {
        let tm = ThreadManager::new();
        let capabilities = vec!["thread:create".to_string()];
        
        // Create threads in different states
        let request1 = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "worker1".to_string(),
            stack_size: None,
            priority: None,
            capabilities: capabilities.clone(),
        };
        let cap_thread1 = tm.create_thread(request1).unwrap();
        tm.start_thread(&cap_thread1).unwrap();
        
        let request2 = ThreadCreateRequest {
            process_cap: "cap_proc_001".to_string(),
            entry_point: "worker2".to_string(),
            stack_size: None,
            priority: None,
            capabilities: capabilities.clone(),
        };
        let cap_thread2 = tm.create_thread(request2).unwrap();
        tm.terminate_thread(&cap_thread2).unwrap();
        
        let stats = tm.get_thread_statistics();
        assert_eq!(stats["total_threads"], 2);
        assert_eq!(stats["running_threads"], 1);
        assert_eq!(stats["terminated_threads"], 1);
    }
}