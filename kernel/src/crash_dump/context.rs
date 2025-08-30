#![no_std]

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;

/// Task context information
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub task_id: u64,
    pub process_id: u64,
    pub priority: TaskPriority,
    pub state: TaskState,
    pub cpu_id: Option<u32>,
    pub wake_timestamp: Option<u64>,
    pub context_switch_count: u64,
    pub total_runtime: u64,
    pub last_scheduled: u64,
    pub stack_info: StackInfo,
    pub memory_usage: MemoryUsage,
    pub capabilities: Vec<CapabilityInfo>,
    pub is_valid: bool,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    RealTime = 3,
}

impl fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Normal => write!(f, "Normal"),
            TaskPriority::High => write!(f, "High"),
            TaskPriority::RealTime => write!(f, "RealTime"),
        }
    }
}

/// Task states
#[derive(Debug, Clone)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Sleeping,
    Terminated,
    Zombie,
    Unknown,
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskState::Ready => write!(f, "Ready"),
            TaskState::Running => write!(f, "Running"),
            TaskState::Blocked => write!(f, "Blocked"),
            TaskState::Sleeping => write!(f, "Sleeping"),
            TaskState::Terminated => write!(f, "Terminated"),
            TaskState::Zombie => write!(f, "Zombie"),
            TaskState::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Stack information
#[derive(Debug, Clone)]
pub struct StackInfo {
    pub stack_pointer: u64,
    pub base_pointer: u64,
    pub stack_size: usize,
    pub stack_used: usize,
    pub stack_free: usize,
    pub overflow_guard: bool,
}

impl StackInfo {
    /// Create new stack info
    pub fn new() -> Self {
        Self {
            stack_pointer: 0,
            base_pointer: 0,
            stack_size: 0,
            stack_used: 0,
            stack_free: 0,
            overflow_guard: false,
        }
    }

    /// Calculate stack usage
    pub fn calculate_usage(&mut self) {
        if self.stack_pointer > 0 && self.base_pointer > 0 {
            if self.stack_pointer <= self.base_pointer {
                self.stack_used = (self.base_pointer - self.stack_pointer) as usize;
            } else {
                self.stack_used = 0;
            }
            
            if self.stack_used <= self.stack_size {
                self.stack_free = self.stack_size - self.stack_used;
            } else {
                self.stack_free = 0;
                self.overflow_guard = true;
            }
        }
    }
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub total_allocated: usize,
    pub heap_usage: usize,
    pub stack_usage: usize,
    pub mmap_usage: usize,
    pub shared_memory: usize,
    pub peak_usage: usize,
}

impl MemoryUsage {
    /// Create new memory usage
    pub fn new() -> Self {
        Self {
            total_allocated: 0,
            heap_usage: 0,
            stack_usage: 0,
            mmap_usage: 0,
            shared_memory: 0,
            peak_usage: 0,
        }
    }
}

/// Capability information
#[derive(Debug, Clone)]
pub struct CapabilityInfo {
    pub token_id: u128,
    pub target: u64,
    pub permissions: u32,
    pub is_valid: bool,
    pub is_revoked: bool,
}

impl CapabilityInfo {
    /// Create new capability info
    pub fn new(token_id: u128, target: u64, permissions: u32) -> Self {
        Self {
            token_id,
            target,
            permissions,
            is_valid: true,
            is_revoked: false,
        }
    }
}

impl TaskContext {
    /// Create new task context
    pub fn new(task_id: u64) -> Self {
        Self {
            task_id,
            process_id: 0,
            priority: TaskPriority::Normal,
            state: TaskState::Unknown,
            cpu_id: None,
            wake_timestamp: None,
            context_switch_count: 0,
            total_runtime: 0,
            last_scheduled: 0,
            stack_info: StackInfo::new(),
            memory_usage: MemoryUsage::new(),
            capabilities: Vec::new(),
            is_valid: false,
        }
    }

    /// Capture task context
    pub fn capture(task_id: u64) -> Self {
        let mut context = Self::new(task_id);
        
        // Try to get task information from scheduler
        if let Some(task) = crate::sched::get_task(task_id) {
            context.process_id = task.process_id;
            context.priority = Self::convert_priority(task.priority);
            context.state = Self::convert_state(task.state);
            context.cpu_id = Some(task.cpu_id);
            context.wake_timestamp = task.wake_timestamp;
            context.context_switch_count = task.context_switch_count;
            context.total_runtime = task.total_runtime;
            context.last_scheduled = task.last_scheduled;
            
            // Capture stack information
            context.stack_info = Self::capture_stack_info(task_id);
            
            // Capture memory usage
            context.memory_usage = Self::capture_memory_usage(task_id);
            
            // Capture capabilities
            context.capabilities = Self::capture_capabilities(task_id);
            
            context.is_valid = true;
        }
        
        context
    }

    /// Convert internal priority to display priority
    fn convert_priority(priority: u8) -> TaskPriority {
        match priority {
            0 => TaskPriority::Low,
            1 => TaskPriority::Normal,
            2 => TaskPriority::High,
            3 => TaskPriority::RealTime,
            _ => TaskPriority::Normal,
        }
    }

    /// Convert internal state to display state
    fn convert_state(state: u8) -> TaskState {
        match state {
            0 => TaskState::Ready,
            1 => TaskState::Running,
            2 => TaskState::Blocked,
            3 => TaskState::Sleeping,
            4 => TaskState::Terminated,
            5 => TaskState::Zombie,
            _ => TaskState::Unknown,
        }
    }

    /// Capture stack information
    fn capture_stack_info(task_id: u64) -> StackInfo {
        let mut stack_info = StackInfo::new();
        
        // TODO: Implement proper stack capture
        // For now, use placeholder values
        stack_info.stack_pointer = 0xffff800000200000;
        stack_info.base_pointer = 0xffff800000200000;
        stack_info.stack_size = 0x1000;
        stack_info.calculate_usage();
        
        stack_info
    }

    /// Capture memory usage
    fn capture_memory_usage(task_id: u64) -> MemoryUsage {
        let mut memory_usage = MemoryUsage::new();
        
        // TODO: Implement proper memory usage capture
        // For now, use placeholder values
        memory_usage.total_allocated = 0x1000;
        memory_usage.heap_usage = 0x800;
        memory_usage.stack_usage = 0x200;
        memory_usage.mmap_usage = 0;
        memory_usage.shared_memory = 0;
        memory_usage.peak_usage = 0x1000;
        
        memory_usage
    }

    /// Capture capabilities
    fn capture_capabilities(task_id: u64) -> Vec<CapabilityInfo> {
        let mut capabilities = Vec::new();
        
        // TODO: Implement proper capability capture
        // For now, return empty list
        
        capabilities
    }

    /// Analyze task context for issues
    pub fn analyze(&self) -> TaskAnalysis {
        let mut analysis = TaskAnalysis::new();
        
        // Check for suspicious patterns
        if self.stack_info.overflow_guard {
            analysis.suspicious_patterns.push("Stack overflow detected".to_string());
            analysis.stack_overflow_suspected = true;
        }
        
        if self.stack_info.stack_used > self.stack_info.stack_size * 9 / 10 {
            analysis.suspicious_patterns.push("Stack usage >90%".to_string());
        }
        
        if self.memory_usage.total_allocated > 0x100000 { // 1MB
            analysis.suspicious_patterns.push("Excessive memory usage".to_string());
        }
        
        if self.context_switch_count > 10000 {
            analysis.suspicious_patterns.push("High context switch count".to_string());
        }
        
        if let Some(wake_ts) = self.wake_timestamp {
            let now = crate::log::get_current_time_ms();
            if now - wake_ts > 1000 { // 1 second
                analysis.suspicious_patterns.push("Task blocked for >1s".to_string());
            }
        }
        
        // Check for revoked capabilities
        let revoked_count = self.capabilities.iter().filter(|c| c.is_revoked).count();
        if revoked_count > 0 {
            analysis.suspicious_patterns.push(format!("{} revoked capabilities", revoked_count));
        }
        
        analysis
    }

    /// Print task context
    pub fn print(&self) {
        let marker = if self.is_valid { " " } else { "⚠️" };
        
        crate::kprintln!("║   Task Context {}:", marker);
        crate::kprintln!("║     Task ID: {} (Process: {})", self.task_id, self.process_id);
        crate::kprintln!("║     Priority: {} | State: {}", self.priority, self.state);
        crate::kprintln!("║     CPU: {} | Context Switches: {}", 
            self.cpu_id.map(|id| id.to_string()).unwrap_or_else(|| "None".to_string()),
            self.context_switch_count);
        
        if let Some(wake_ts) = self.wake_timestamp {
            crate::kprintln!("║     Wake Timestamp: {} ms", wake_ts);
        }
        
        crate::kprintln!("║     Total Runtime: {} ms | Last Scheduled: {} ms", 
            self.total_runtime, self.last_scheduled);
        
        // Stack information
        crate::kprintln!("║     Stack: SP=0x{:016x} BP=0x{:016x} Size={} Used={} Free={}", 
            self.stack_info.stack_pointer,
            self.stack_info.base_pointer,
            self.stack_info.stack_size,
            self.stack_info.stack_used,
            self.stack_info.stack_free
        );
        
        if self.stack_info.overflow_guard {
            crate::kprintln!("║       🚨 STACK OVERFLOW DETECTED");
        }
        
        // Memory usage
        crate::kprintln!("║     Memory: Total={} Heap={} Stack={} MMap={} Shared={} Peak={}", 
            self.memory_usage.total_allocated,
            self.memory_usage.heap_usage,
            self.memory_usage.stack_usage,
            self.memory_usage.mmap_usage,
            self.memory_usage.shared_memory,
            self.memory_usage.peak_usage
        );
        
        // Capabilities
        if !self.capabilities.is_empty() {
            crate::kprintln!("║     Capabilities: {} ({} valid, {} revoked)", 
                self.capabilities.len(),
                self.capabilities.iter().filter(|c| c.is_valid && !c.is_revoked).count(),
                self.capabilities.iter().filter(|c| c.is_revoked).count()
            );
        }
        
        // Analysis
        let analysis = self.analyze();
        if !analysis.suspicious_patterns.is_empty() {
            crate::kprintln!("║     Analysis:");
            for pattern in &analysis.suspicious_patterns {
                crate::kprintln!("║       ⚠️  {}", pattern);
            }
        }
        
        if analysis.stack_overflow_suspected {
            crate::kprintln!("║       🚨 STACK OVERFLOW SUSPECTED");
        }
    }

    /// Generate task context report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        let marker = if self.is_valid { " " } else { "⚠️" };
        
        report.push_str(&format!("Task Context {}:\n", marker));
        report.push_str(&format!("  Task ID: {} (Process: {})\n", self.task_id, self.process_id));
        report.push_str(&format!("  Priority: {} | State: {}\n", self.priority, self.state));
        report.push_str(&format!("  CPU: {} | Context Switches: {}\n", 
            self.cpu_id.map(|id| id.to_string()).unwrap_or_else(|| "None".to_string()),
            self.context_switch_count));
        
        if let Some(wake_ts) = self.wake_timestamp {
            report.push_str(&format!("  Wake Timestamp: {} ms\n", wake_ts));
        }
        
        report.push_str(&format!("  Total Runtime: {} ms | Last Scheduled: {} ms\n", 
            self.total_runtime, self.last_scheduled));
        
        // Stack information
        report.push_str(&format!("  Stack: SP=0x{:016x} BP=0x{:016x} Size={} Used={} Free={}\n", 
            self.stack_info.stack_pointer,
            self.stack_info.base_pointer,
            self.stack_info.stack_size,
            self.stack_info.stack_used,
            self.stack_info.stack_free
        ));
        
        if self.stack_info.overflow_guard {
            report.push_str("    🚨 STACK OVERFLOW DETECTED\n");
        }
        
        // Memory usage
        report.push_str(&format!("  Memory: Total={} Heap={} Stack={} MMap={} Shared={} Peak={}\n", 
            self.memory_usage.total_allocated,
            self.memory_usage.heap_usage,
            self.memory_usage.stack_usage,
            self.memory_usage.mmap_usage,
            self.memory_usage.shared_memory,
            self.memory_usage.peak_usage
        ));
        
        // Capabilities
        if !self.capabilities.is_empty() {
            report.push_str(&format!("  Capabilities: {} ({} valid, {} revoked)\n", 
                self.capabilities.len(),
                self.capabilities.iter().filter(|c| c.is_valid && !c.is_revoked).count(),
                self.capabilities.iter().filter(|c| c.is_revoked).count()
            ));
        }
        
        // Analysis
        let analysis = self.analyze();
        if !analysis.suspicious_patterns.is_empty() {
            report.push_str("  Analysis:\n");
            for pattern in &analysis.suspicious_patterns {
                report.push_str(&format!("    ⚠️  {}\n", pattern));
            }
        }
        
        if analysis.stack_overflow_suspected {
            report.push_str("    🚨 STACK OVERFLOW SUSPECTED\n");
        }
        
        report
    }
}

/// Task analysis results
#[derive(Debug, Clone)]
pub struct TaskAnalysis {
    pub suspicious_patterns: Vec<String>,
    pub stack_overflow_suspected: bool,
    pub memory_issues: bool,
    pub scheduling_issues: bool,
    pub capability_issues: bool,
}

impl TaskAnalysis {
    /// Create new task analysis
    pub fn new() -> Self {
        Self {
            suspicious_patterns: Vec::new(),
            stack_overflow_suspected: false,
            memory_issues: false,
            scheduling_issues: false,
            capability_issues: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_context_creation() {
        let context = TaskContext::new(123);
        assert_eq!(context.task_id, 123);
        assert!(!context.is_valid);
    }

    #[test]
    fn test_task_priority_display() {
        let priority = TaskPriority::High;
        assert_eq!(priority.to_string(), "High");
    }

    #[test]
    fn test_task_state_display() {
        let state = TaskState::Running;
        assert_eq!(state.to_string(), "Running");
    }

    #[test]
    fn test_stack_info_calculation() {
        let mut stack_info = StackInfo::new();
        stack_info.stack_pointer = 0x1000;
        stack_info.base_pointer = 0x2000;
        stack_info.stack_size = 0x1000;
        stack_info.calculate_usage();
        
        assert_eq!(stack_info.stack_used, 0x1000);
        assert_eq!(stack_info.stack_free, 0);
    }

    #[test]
    fn test_memory_usage_creation() {
        let memory_usage = MemoryUsage::new();
        assert_eq!(memory_usage.total_allocated, 0);
        assert_eq!(memory_usage.peak_usage, 0);
    }
}
