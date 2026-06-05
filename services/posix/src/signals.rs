use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    SIGINT,
    SIGTERM,
    SIGCHLD,
    SIGKILL,
    SIGSTOP,
    SIGCONT,
    SIGUSR1,
    SIGUSR2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvent {
    pub signal: Signal,
    pub target_cap: String,
    pub sender_cap: Option<String>,
    pub timestamp: u64,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalHandler {
    pub signal: Signal,
    pub handler_type: HandlerType,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandlerType {
    Default,
    Ignore,
    Custom(String), // Function name or capability
}

pub struct SignalDispatcher {
    signal_queue: Arc<Mutex<VecDeque<SignalEvent>>>,
    handlers: Arc<Mutex<HashMap<String, Vec<SignalHandler>>>>, // cap_proc -> handlers
    pending_signals: Arc<Mutex<HashMap<String, Vec<SignalEvent>>>>, // cap_proc -> pending
    virtual_clock: Arc<Mutex<u64>>,
    audit_log: Arc<Mutex<Vec<String>>>,
}

impl SignalDispatcher {
    pub fn new() -> Self {
        Self {
            signal_queue: Arc::new(Mutex::new(VecDeque::new())),
            handlers: Arc::new(Mutex::new(HashMap::new())),
            pending_signals: Arc::new(Mutex::new(HashMap::new())),
            virtual_clock: Arc::new(Mutex::new(0)),
            audit_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn send_signal(&self, event: SignalEvent) -> Result<(), String> {
        let start = Instant::now();
        
        // Validate signal sending capabilities
        if let Some(sender_cap) = &event.sender_cap {
            if !self.validate_signal_capability(sender_cap, &event.signal) {
                self.log_audit(&format!("SIGNAL_DENIED: {} -> {} (insufficient capability)", 
                    sender_cap, event.target_cap));
                return Err("Insufficient signal capability".to_string());
            }
        }

        // Check if target process exists and is valid
        if !self.validate_target_process(&event.target_cap) {
            self.log_audit(&format!("SIGNAL_DENIED: target process {} not found", event.target_cap));
            return Err("Target process not found".to_string());
        }

        // Handle special signals that cannot be blocked
        if matches!(event.signal, Signal::SIGKILL | Signal::SIGSTOP) {
            self.deliver_immediate_signal(event.clone())?;
        } else {
            // Add to signal queue for normal delivery
            self.signal_queue.lock().unwrap().push_back(event.clone());
            
            // Add to pending signals for target process
            self.pending_signals.lock().unwrap()
                .entry(event.target_cap.clone())
                .or_insert_with(Vec::new)
                .push(event.clone());
        }

        let duration = start.elapsed();
        self.log_audit(&format!("SIGNAL_SENT: {:?} -> {} from {:?} ({:?})", 
            event.signal, event.target_cap, event.sender_cap, duration));

        Ok(())
    }

    pub fn send_signal_to_pid(&self, pid: u32, signal: Signal, sender_cap: Option<String>) -> Result<(), String> {
        // Find process capability by PID
        let target_cap = self.find_process_cap_by_pid(pid)?;
        
        let event = SignalEvent {
            signal,
            target_cap,
            sender_cap,
            timestamp: self.get_virtual_time(),
            data: None,
        };

        self.send_signal(event)
    }

    pub fn send_signal_to_group(&self, group_id: u32, signal: Signal, sender_cap: Option<String>) -> Result<usize, String> {
        let mut delivered_count = 0;
        
        // In a real implementation, would iterate through process group
        // For now, simulate sending to multiple processes
        let group_processes = self.get_process_group(group_id);
        
        for process_cap in group_processes {
            let event = SignalEvent {
                signal: signal.clone(),
                target_cap: process_cap,
                sender_cap: sender_cap.clone(),
                timestamp: self.get_virtual_time(),
                data: None,
            };
            
            if self.send_signal(event).is_ok() {
                delivered_count += 1;
            }
        }

        self.log_audit(&format!("SIGNAL_GROUP_SENT: {:?} to group {} ({} processes)", 
            signal, group_id, delivered_count));
        
        Ok(delivered_count)
    }

    pub fn block_signals(&self, cap_proc: &str, signal_mask: Vec<Signal>) -> Result<(), String> {
        // In real implementation, would set signal mask for process
        self.log_audit(&format!("SIGNAL_MASK_SET: {} blocked {:?}", cap_proc, signal_mask));
        Ok(())
    }

    pub fn unblock_signals(&self, cap_proc: &str, signal_mask: Vec<Signal>) -> Result<(), String> {
        // In real implementation, would unset signal mask for process
        self.log_audit(&format!("SIGNAL_MASK_UNSET: {} unblocked {:?}", cap_proc, signal_mask));
        Ok(())
    }

    pub fn get_signal_mask(&self, cap_proc: &str) -> Result<Vec<Signal>, String> {
        // In real implementation, would return current signal mask
        Ok(vec![]) // Empty mask for now
    }

    pub fn register_handler(&self, cap_proc: &str, handler: SignalHandler) -> Result<(), String> {
        // Validate handler registration capabilities
        if !handler.capabilities.contains(&"signal:handle".to_string()) {
            return Err("Insufficient handler capability".to_string());
        }

        self.handlers.lock().unwrap()
            .entry(cap_proc.to_string())
            .or_insert_with(Vec::new)
            .push(handler.clone());

        self.log_audit(&format!("HANDLER_REGISTERED: {:?} for {}", handler.signal, cap_proc));
        Ok(())
    }

    pub fn deliver_signals(&self, cap_proc: &str) -> Result<Vec<SignalEvent>, String> {
        let mut pending = self.pending_signals.lock().unwrap();
        let signals = pending.remove(cap_proc).unwrap_or_default();
        
        if !signals.is_empty() {
            self.log_audit(&format!("SIGNALS_DELIVERED: {} signals to {}", signals.len(), cap_proc));
        }

        Ok(signals)
    }

    pub fn handle_ctrl_c(&self, target_cap: &str) -> Result<(), String> {
        let event = SignalEvent {
            signal: Signal::SIGINT,
            target_cap: target_cap.to_string(),
            sender_cap: Some("kernel".to_string()),
            timestamp: self.get_virtual_time(),
            data: Some("Ctrl+C pressed".to_string()),
        };

        self.send_signal(event)
    }

    pub fn handle_child_exit(&self, parent_cap: &str, child_cap: &str, exit_code: i32) -> Result<(), String> {
        let event = SignalEvent {
            signal: Signal::SIGCHLD,
            target_cap: parent_cap.to_string(),
            sender_cap: Some(child_cap.to_string()),
            timestamp: self.get_virtual_time(),
            data: Some(exit_code.to_string()),
        };

        self.send_signal(event)
    }

    pub fn setup_timer_signal(&self, cap_proc: &str, duration_ms: u64) -> Result<(), String> {
        // In real implementation, would set up actual timer
        let event = SignalEvent {
            signal: Signal::SIGUSR1,
            target_cap: cap_proc.to_string(),
            sender_cap: Some("timer".to_string()),
            timestamp: self.get_virtual_time() + duration_ms,
            data: Some(format!("timer:{}", duration_ms)),
        };

        self.send_signal(event)
    }

    pub fn get_pending_count(&self, cap_proc: &str) -> usize {
        self.pending_signals.lock().unwrap()
            .get(cap_proc)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    pub fn clear_pending(&self, cap_proc: &str) -> usize {
        let mut pending = self.pending_signals.lock().unwrap();
        let count = pending.get(cap_proc).map(|v| v.len()).unwrap_or(0);
        pending.remove(cap_proc);
        count
    }

    fn validate_signal_capability(&self, sender_cap: &str, signal: &Signal) -> bool {
        // Basic validation - in real implementation would check actual capabilities
        match signal {
            Signal::SIGKILL | Signal::SIGSTOP => {
                // Only privileged processes can send these
                sender_cap == "kernel" || sender_cap.contains("admin")
            },
            _ => true, // Most signals can be sent by any process
        }
    }

    fn validate_target_process(&self, target_cap: &str) -> bool {
        // In real implementation, would check if process exists and is valid
        // For now, just validate the capability format
        target_cap.starts_with("cap_proc_") && target_cap.len() > 10
    }

    fn find_process_cap_by_pid(&self, pid: u32) -> Result<String, String> {
        // In real implementation, would look up process capability by PID
        // For now, generate a mock capability
        Ok(format!("cap_proc_{:08x}", pid))
    }

    fn get_process_group(&self, group_id: u32) -> Vec<String> {
        // In real implementation, would return all processes in the group
        // For now, return mock process capabilities
        vec![
            format!("cap_proc_{:08x}", group_id * 1000 + 1),
            format!("cap_proc_{:08x}", group_id * 1000 + 2),
        ]
    }

    fn deliver_immediate_signal(&self, event: SignalEvent) -> Result<(), String> {
        // Handle signals that cannot be blocked (SIGKILL, SIGSTOP)
        self.log_audit(&format!("IMMEDIATE_SIGNAL: {:?} -> {} (unblockable)", 
            event.signal, event.target_cap));
        
        // In real implementation, would immediately affect the target process
        match event.signal {
            Signal::SIGKILL => {
                // Terminate process immediately
                self.log_audit(&format!("PROCESS_TERMINATED: {} by SIGKILL", event.target_cap));
            },
            Signal::SIGSTOP => {
                // Stop process immediately
                self.log_audit(&format!("PROCESS_STOPPED: {} by SIGSTOP", event.target_cap));
            },
            _ => {
                // Should not happen for unblockable signals
                return Err("Invalid unblockable signal".to_string());
            }
        }
        
        Ok(())
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

    pub fn get_queue_size(&self) -> usize {
        self.signal_queue.lock().unwrap().len()
    }

    pub fn process_signal_queue(&self) -> usize {
        let mut queue = self.signal_queue.lock().unwrap();
        let processed = queue.len();
        
        // In real implementation, would actually deliver signals to processes
        while let Some(event) = queue.pop_front() {
            self.log_audit(&format!("SIGNAL_PROCESSED: {:?} -> {}", event.signal, event.target_cap));
        }
        
        processed
    }
}

// Timer integration for deterministic signal delivery
pub struct DeterministicTimer {
    virtual_time: Arc<Mutex<u64>>,
    scheduled_signals: Arc<Mutex<Vec<(u64, SignalEvent)>>>,
}

impl DeterministicTimer {
    pub fn new() -> Self {
        Self {
            virtual_time: Arc::new(Mutex::new(0)),
            scheduled_signals: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn schedule_signal(&self, delay_ms: u64, event: SignalEvent) {
        let trigger_time = {
            let mut time = self.virtual_time.lock().unwrap();
            *time += delay_ms;
            *time
        };

        self.scheduled_signals.lock().unwrap().push((trigger_time, event));
    }

    pub fn tick(&self, dispatcher: &SignalDispatcher) -> usize {
        let current_time = *self.virtual_time.lock().unwrap();
        let mut scheduled = self.scheduled_signals.lock().unwrap();
        
        let mut delivered = 0;
        scheduled.retain(|(trigger_time, event)| {
            if *trigger_time <= current_time {
                let _ = dispatcher.send_signal(event.clone());
                delivered += 1;
                false
            } else {
                true
            }
        });

        delivered
    }

    pub fn advance_time(&self, ms: u64) {
        let mut time = self.virtual_time.lock().unwrap();
        *time += ms;
    }

    pub fn get_time(&self) -> u64 {
        *self.virtual_time.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_dispatcher_creation() {
        let dispatcher = SignalDispatcher::new();
        assert_eq!(dispatcher.get_queue_size(), 0);
    }

    #[test]
    fn test_send_signal() {
        let dispatcher = SignalDispatcher::new();
        let event = SignalEvent {
            signal: Signal::SIGINT,
            target_cap: "cap_proc_001".to_string(),
            sender_cap: Some("cap_proc_002".to_string()),
            timestamp: 12345,
            data: None,
        };

        let result = dispatcher.send_signal(event);
        assert!(result.is_ok());
        assert_eq!(dispatcher.get_queue_size(), 1);
        assert_eq!(dispatcher.get_pending_count("cap_proc_001"), 1);
    }

    #[test]
    fn test_register_handler() {
        let dispatcher = SignalDispatcher::new();
        let handler = SignalHandler {
            signal: Signal::SIGINT,
            handler_type: HandlerType::Custom("my_handler".to_string()),
            capabilities: vec!["signal:handle".to_string()],
        };

        let result = dispatcher.register_handler("cap_proc_001", handler);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ctrl_c_handling() {
        let dispatcher = SignalDispatcher::new();
        let result = dispatcher.handle_ctrl_c("cap_proc_001");
        assert!(result.is_ok());
        assert_eq!(dispatcher.get_pending_count("cap_proc_001"), 1);
    }

    #[test]
    fn test_deterministic_timer() {
        let timer = DeterministicTimer::new();
        let dispatcher = SignalDispatcher::new();
        
        let event = SignalEvent {
            signal: Signal::SIGUSR1,
            target_cap: "cap_proc_001".to_string(),
            sender_cap: Some("timer".to_string()),
            timestamp: 0,
            data: Some("timer_test".to_string()),
        };

        timer.schedule_signal(100, event);
        timer.advance_time(50);
        assert_eq!(timer.tick(&dispatcher), 0); // Not yet time
        
        timer.advance_time(60);
        assert_eq!(timer.tick(&dispatcher), 1); // Now delivered
    }

    #[test]
    fn test_send_signal_to_pid() {
        let dispatcher = SignalDispatcher::new();
        let result = dispatcher.send_signal_to_pid(123, Signal::SIGUSR1, Some("cap_proc_001".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_send_signal_to_group() {
        let dispatcher = SignalDispatcher::new();
        let result = dispatcher.send_signal_to_group(1, Signal::SIGTERM, Some("cap_proc_001".to_string()));
        assert!(result.is_ok());
        assert!(result.unwrap() > 0); // Should deliver to at least one process
    }

    #[test]
    fn test_signal_masking() {
        let dispatcher = SignalDispatcher::new();
        let signal_mask = vec![Signal::SIGINT, Signal::SIGTERM];
        
        let result = dispatcher.block_signals("cap_proc_001", signal_mask.clone());
        assert!(result.is_ok());
        
        let result = dispatcher.unblock_signals("cap_proc_001", signal_mask);
        assert!(result.is_ok());
        
        let mask = dispatcher.get_signal_mask("cap_proc_001").unwrap();
        assert!(mask.is_empty()); // Should be empty after unblocking
    }

    #[test]
    fn test_immediate_signal_delivery() {
        let dispatcher = SignalDispatcher::new();
        let event = SignalEvent {
            signal: Signal::SIGKILL,
            target_cap: "cap_proc_001".to_string(),
            sender_cap: Some("kernel".to_string()),
            timestamp: 12345,
            data: None,
        };

        let result = dispatcher.send_signal(event);
        assert!(result.is_ok());
        
        // SIGKILL should be delivered immediately, not queued
        assert_eq!(dispatcher.get_queue_size(), 0);
    }

    #[test]
    fn test_signal_validation() {
        let dispatcher = SignalDispatcher::new();
        
        // Test invalid target process
        let event = SignalEvent {
            signal: Signal::SIGUSR1,
            target_cap: "invalid_cap".to_string(),
            sender_cap: Some("cap_proc_001".to_string()),
            timestamp: 12345,
            data: None,
        };

        let result = dispatcher.send_signal(event);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Target process not found"));
    }
}