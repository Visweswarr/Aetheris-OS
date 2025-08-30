#![no_std]

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;

use super::{TaskId, TaskPriority, TaskState};
use crate::serial::kprintln;

/// Fairness analysis configuration
#[derive(Debug, Clone)]
pub struct FairnessAnalysisConfig {
    pub analysis_window_ms: u64,
    pub min_samples: usize,
    pub fairness_threshold: f32,
    pub enable_detailed_analysis: bool,
    pub enable_performance_validation: bool,
    pub enable_starvation_detection: bool,
    pub enable_load_balancing_analysis: bool,
}

impl Default for FairnessAnalysisConfig {
    fn default() -> Self {
        Self {
            analysis_window_ms: 1000, // 1 second
            min_samples: 100,
            fairness_threshold: 0.8, // 80% fairness
            enable_detailed_analysis: true,
            enable_performance_validation: true,
            enable_starvation_detection: true,
            enable_load_balancing_analysis: true,
        }
    }
}

/// Fairness metrics for a task
#[derive(Debug, Clone)]
pub struct TaskFairnessMetrics {
    pub task_id: TaskId,
    pub priority: TaskPriority,
    pub total_runtime: u64,
    pub expected_runtime: u64,
    pub fairness_score: f32,
    pub wait_time: u64,
    pub context_switches: u64,
    pub preemptions: u64,
    pub starvation_risk: f32,
}

impl TaskFairnessMetrics {
    /// Create new task fairness metrics
    pub fn new(task_id: TaskId, priority: TaskPriority) -> Self {
        Self {
            task_id,
            priority,
            total_runtime: 0,
            expected_runtime: 0,
            fairness_score: 1.0,
            wait_time: 0,
            context_switches: 0,
            preemptions: 0,
            starvation_risk: 0.0,
        }
    }

    /// Calculate fairness score
    pub fn calculate_fairness_score(&mut self) {
        if self.expected_runtime > 0 {
            self.fairness_score = self.total_runtime as f32 / self.expected_runtime as f32;
        } else {
            self.fairness_score = 1.0;
        }
    }

    /// Calculate starvation risk
    pub fn calculate_starvation_risk(&mut self, total_wait_time: u64, total_tasks: usize) {
        let avg_wait_time = if total_tasks > 0 {
            total_wait_time / total_tasks as u64
        } else {
            0
        };
        
        if avg_wait_time > 0 {
            self.starvation_risk = (self.wait_time as f32 / avg_wait_time as f32).min(10.0);
        } else {
            self.starvation_risk = 0.0;
        }
    }
}

/// Scheduler fairness analysis
#[derive(Debug)]
pub struct SchedulerFairnessAnalysis {
    config: FairnessAnalysisConfig,
    task_metrics: BTreeMap<TaskId, TaskFairnessMetrics>,
    global_metrics: GlobalFairnessMetrics,
    analysis_history: Vec<FairnessAnalysisResult>,
    start_time: u64,
}

/// Global fairness metrics
#[derive(Debug, Clone)]
pub struct GlobalFairnessMetrics {
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub total_runtime: u64,
    pub total_wait_time: u64,
    pub average_fairness: f32,
    pub fairness_variance: f32,
    pub starvation_count: usize,
    pub load_imbalance: f32,
}

impl Default for GlobalFairnessMetrics {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            active_tasks: 0,
            total_runtime: 0,
            total_wait_time: 0,
            average_fairness: 1.0,
            fairness_variance: 0.0,
            starvation_count: 0,
            load_imbalance: 0.0,
        }
    }
}

/// Fairness analysis result
#[derive(Debug, Clone)]
pub struct FairnessAnalysisResult {
    pub timestamp: u64,
    pub analysis_duration_ms: u64,
    pub fairness_score: f32,
    pub starvation_detected: bool,
    pub load_balanced: bool,
    pub recommendations: Vec<String>,
    pub task_metrics: Vec<TaskFairnessMetrics>,
}

impl SchedulerFairnessAnalysis {
    /// Create new scheduler fairness analysis
    pub fn new(config: FairnessAnalysisConfig) -> Self {
        Self {
            config,
            task_metrics: BTreeMap::new(),
            global_metrics: GlobalFairnessMetrics::default(),
            analysis_history: Vec::new(),
            start_time: 0,
        }
    }

    /// Start fairness analysis
    pub fn start_analysis(&mut self) {
        self.start_time = crate::log::get_current_time_ms();
        self.task_metrics.clear();
        self.global_metrics = GlobalFairnessMetrics::default();
        
        // Initialize task metrics for all active tasks
        self.initialize_task_metrics();
    }

    /// Initialize task metrics
    fn initialize_task_metrics(&mut self) {
        let task_count = crate::sched::get_task_count();
        
        for task_id in 0..task_count {
            if let Some(task) = crate::sched::get_task(task_id) {
                let priority = Self::convert_priority(task.priority);
                let mut metrics = TaskFairnessMetrics::new(task_id, priority);
                
                // Set initial values
                metrics.total_runtime = task.total_runtime;
                metrics.context_switches = task.context_switch_count;
                metrics.wait_time = self.calculate_wait_time(task_id);
                
                self.task_metrics.insert(task_id, metrics);
            }
        }
        
        self.global_metrics.total_tasks = self.task_metrics.len();
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

    /// Calculate wait time for a task
    fn calculate_wait_time(&self, task_id: TaskId) -> u64 {
        if let Some(task) = crate::sched::get_task(task_id) {
            let now = crate::log::get_current_time_ms();
            if let Some(wake_ts) = task.wake_timestamp {
                if now > wake_ts {
                    return now - wake_ts;
                }
            }
        }
        0
    }

    /// Update fairness metrics
    pub fn update_metrics(&mut self) {
        let current_time = crate::log::get_current_time_ms();
        
        // Update task metrics
        for (task_id, metrics) in &mut self.task_metrics {
            if let Some(task) = crate::sched::get_task(*task_id) {
                metrics.total_runtime = task.total_runtime;
                metrics.context_switches = task.context_switch_count;
                metrics.wait_time = self.calculate_wait_time(*task_id);
                
                // Calculate expected runtime based on priority
                metrics.expected_runtime = self.calculate_expected_runtime(metrics.priority);
                
                // Calculate fairness score
                metrics.calculate_fairness_score();
            }
        }
        
        // Update global metrics
        self.update_global_metrics();
        
        // Check for analysis completion
        if current_time - self.start_time >= self.config.analysis_window_ms {
            self.complete_analysis();
        }
    }

    /// Calculate expected runtime based on priority
    fn calculate_expected_runtime(&self, priority: TaskPriority) -> u64 {
        let base_runtime = 100; // Base runtime in ms
        
        match priority {
            TaskPriority::Low => base_runtime,
            TaskPriority::Normal => base_runtime * 2,
            TaskPriority::High => base_runtime * 4,
            TaskPriority::RealTime => base_runtime * 8,
        }
    }

    /// Update global metrics
    fn update_global_metrics(&mut self) {
        let mut total_runtime = 0;
        let mut total_wait_time = 0;
        let mut fairness_scores = Vec::new();
        let mut active_count = 0;
        
        for metrics in self.task_metrics.values() {
            total_runtime += metrics.total_runtime;
            total_wait_time += metrics.wait_time;
            fairness_scores.push(metrics.fairness_score);
            
            if metrics.total_runtime > 0 {
                active_count += 1;
            }
        }
        
        self.global_metrics.total_runtime = total_runtime;
        self.global_metrics.total_wait_time = total_wait_time;
        self.global_metrics.active_tasks = active_count;
        
        // Calculate average fairness
        if !fairness_scores.is_empty() {
            let sum: f32 = fairness_scores.iter().sum();
            self.global_metrics.average_fairness = sum / fairness_scores.len() as f32;
            
            // Calculate variance
            let mean = self.global_metrics.average_fairness;
            let variance: f32 = fairness_scores.iter()
                .map(|&score| (score - mean).powi(2))
                .sum();
            self.global_metrics.fairness_variance = variance / fairness_scores.len() as f32;
        }
        
        // Calculate load imbalance
        self.global_metrics.load_imbalance = self.calculate_load_imbalance();
        
        // Count starvation cases
        self.global_metrics.starvation_count = self.count_starvation_cases();
    }

    /// Calculate load imbalance
    fn calculate_load_imbalance(&self) -> f32 {
        if self.task_metrics.is_empty() {
            return 0.0;
        }
        
        let runtimes: Vec<u64> = self.task_metrics.values()
            .map(|m| m.total_runtime)
            .collect();
        
        let min_runtime = runtimes.iter().min().unwrap_or(&0);
        let max_runtime = runtimes.iter().max().unwrap_or(&0);
        
        if *min_runtime == 0 {
            return 1.0; // Maximum imbalance
        }
        
        (*max_runtime as f32 / *min_runtime as f32 - 1.0).min(10.0)
    }

    /// Count starvation cases
    fn count_starvation_cases(&self) -> usize {
        self.task_metrics.values()
            .filter(|m| m.starvation_risk > 2.0)
            .count()
    }

    /// Complete fairness analysis
    fn complete_analysis(&mut self) {
        let current_time = crate::log::get_current_time_ms();
        let analysis_duration = current_time - self.start_time;
        
        // Update starvation risk for all tasks
        for metrics in self.task_metrics.values_mut() {
            metrics.calculate_starvation_risk(
                self.global_metrics.total_wait_time,
                self.global_metrics.total_tasks
            );
        }
        
        // Generate recommendations
        let recommendations = self.generate_recommendations();
        
        // Create analysis result
        let result = FairnessAnalysisResult {
            timestamp: current_time,
            analysis_duration_ms: analysis_duration,
            fairness_score: self.global_metrics.average_fairness,
            starvation_detected: self.global_metrics.starvation_count > 0,
            load_balanced: self.global_metrics.load_imbalance < 0.5,
            recommendations,
            task_metrics: self.task_metrics.values().cloned().collect(),
        };
        
        self.analysis_history.push(result.clone());
        
        // Print analysis results
        self.print_analysis_results(&result);
        
        // Reset for next analysis
        self.start_time = current_time;
    }

    /// Generate recommendations
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Fairness recommendations
        if self.global_metrics.average_fairness < self.config.fairness_threshold {
            recommendations.push("Consider adjusting priority weights to improve fairness".to_string());
        }
        
        if self.global_metrics.fairness_variance > 0.5 {
            recommendations.push("High fairness variance detected - review priority distribution".to_string());
        }
        
        // Starvation recommendations
        if self.global_metrics.starvation_count > 0 {
            recommendations.push("Starvation detected - implement aging or priority boosting".to_string());
        }
        
        // Load balancing recommendations
        if self.global_metrics.load_imbalance > 1.0 {
            recommendations.push("Load imbalance detected - consider work stealing or redistribution".to_string());
        }
        
        // Performance recommendations
        if self.global_metrics.average_fairness < 0.6 {
            recommendations.push("Critical fairness issues - immediate scheduler review required".to_string());
        }
        
        recommendations
    }

    /// Print analysis results
    fn print_analysis_results(&self, result: &FairnessAnalysisResult) {
        crate::kprintln!("");
        crate::kprintln!("╔══════════════════════════════════════════════════════════════════════════════╗");
        crate::kprintln!("║                           SCHEDULER FAIRNESS ANALYSIS                    ║");
        crate::kprintln!("╠══════════════════════════════════════════════════════════════════════════════╣");
        crate::kprintln!("║ Analysis Duration: {} ms", result.analysis_duration_ms);
        crate::kprintln!("║ Overall Fairness Score: {:.3}", result.fairness_score);
        crate::kprintln!("║ Starvation Detected: {}", if result.starvation_detected { "YES" } else { "NO" });
        crate::kprintln!("║ Load Balanced: {}", if result.load_balanced { "YES" } else { "NO" });
        crate::kprintln!("╠══════════════════════════════════════════════════════════════════════════════╣");
        
        // Global metrics
        crate::kprintln!("║ Global Metrics:");
        crate::kprintln!("║   Total Tasks: {} | Active Tasks: {}", 
            self.global_metrics.total_tasks, self.global_metrics.active_tasks);
        crate::kprintln!("║   Total Runtime: {} ms | Total Wait Time: {} ms", 
            self.global_metrics.total_runtime, self.global_metrics.total_wait_time);
        crate::kprintln!("║   Fairness Variance: {:.3} | Load Imbalance: {:.3}", 
            self.global_metrics.fairness_variance, self.global_metrics.load_imbalance);
        crate::kprintln!("║   Starvation Cases: {}", self.global_metrics.starvation_count);
        
        // Task metrics summary
        if !result.task_metrics.is_empty() {
            crate::kprintln!("╠══════════════════════════════════════════════════════════════════════════════╣");
            crate::kprintln!("║ Task Fairness Summary:");
            
            for metrics in &result.task_metrics {
                let status = if metrics.fairness_score >= 0.8 { "✅" } 
                           else if metrics.fairness_score >= 0.6 { "⚠️" } 
                           else { "🚨" };
                
                crate::kprintln!("║   Task {} ({}): Fairness={:.3} Starvation={:.2} Runtime={}ms", 
                    metrics.task_id, metrics.priority, metrics.fairness_score, 
                    metrics.starvation_risk, metrics.total_runtime);
            }
        }
        
        // Recommendations
        if !result.recommendations.is_empty() {
            crate::kprintln!("╠══════════════════════════════════════════════════════════════════════════════╣");
            crate::kprintln!("║ Recommendations:");
            for (i, recommendation) in result.recommendations.iter().enumerate() {
                crate::kprintln!("║   {}. {}", i + 1, recommendation);
            }
        }
        
        crate::kprintln!("╚══════════════════════════════════════════════════════════════════════════════╝");
        crate::kprintln!("");
    }

    /// Get fairness analysis report
    pub fn get_fairness_report(&self) -> String {
        let mut report = String::new();
        
        if let Some(latest_result) = self.analysis_history.last() {
            report.push_str(&format!("Scheduler Fairness Analysis Report\n"));
            report.push_str(&format!("==================================\n\n"));
            report.push_str(&format!("Analysis Duration: {} ms\n", latest_result.analysis_duration_ms));
            report.push_str(&format!("Overall Fairness Score: {:.3}\n", latest_result.fairness_score));
            report.push_str(&format!("Starvation Detected: {}\n", if latest_result.starvation_detected { "YES" } else { "NO" }));
            report.push_str(&format!("Load Balanced: {}\n", if latest_result.load_balanced { "YES" } else { "NO" }));
            report.push_str("\n");
            
            // Global metrics
            report.push_str("Global Metrics:\n");
            report.push_str(&format!("  Total Tasks: {} | Active Tasks: {}\n", 
                self.global_metrics.total_tasks, self.global_metrics.active_tasks));
            report.push_str(&format!("  Total Runtime: {} ms | Total Wait Time: {} ms\n", 
                self.global_metrics.total_runtime, self.global_metrics.total_wait_time));
            report.push_str(&format!("  Fairness Variance: {:.3} | Load Imbalance: {:.3}\n", 
                self.global_metrics.fairness_variance, self.global_metrics.load_imbalance));
            report.push_str(&format!("  Starvation Cases: {}\n", self.global_metrics.starvation_count));
            report.push_str("\n");
            
            // Task metrics
            report.push_str("Task Fairness Metrics:\n");
            for metrics in &latest_result.task_metrics {
                report.push_str(&format!("  Task {} ({}): Fairness={:.3} Starvation={:.2} Runtime={}ms\n", 
                    metrics.task_id, metrics.priority, metrics.fairness_score, 
                    metrics.starvation_risk, metrics.total_runtime));
            }
            report.push_str("\n");
            
            // Recommendations
            if !latest_result.recommendations.is_empty() {
                report.push_str("Recommendations:\n");
                for (i, recommendation) in latest_result.recommendations.iter().enumerate() {
                    report.push_str(&format!("  {}. {}\n", i + 1, recommendation));
                }
            }
        }
        
        report
    }

    /// Check if fairness analysis is complete
    pub fn is_analysis_complete(&self) -> bool {
        let current_time = crate::log::get_current_time_ms();
        current_time - self.start_time >= self.config.analysis_window_ms
    }

    /// Get current fairness score
    pub fn get_current_fairness_score(&self) -> f32 {
        self.global_metrics.average_fairness
    }

    /// Get starvation count
    pub fn get_starvation_count(&self) -> usize {
        self.global_metrics.starvation_count
    }

    /// Get load imbalance
    pub fn get_load_imbalance(&self) -> f32 {
        self.global_metrics.load_imbalance
    }
}

/// Performance validation for scheduler
pub struct SchedulerPerformanceValidator {
    config: FairnessAnalysisConfig,
    performance_history: Vec<PerformanceSnapshot>,
}

/// Performance snapshot
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: u64,
    pub context_switch_latency: u32,
    pub scheduling_overhead: u32,
    pub task_throughput: u32,
    pub cpu_utilization: f32,
    pub memory_usage: usize,
}

impl SchedulerPerformanceValidator {
    /// Create new performance validator
    pub fn new(config: FairnessAnalysisConfig) -> Self {
        Self {
            config,
            performance_history: Vec::new(),
        }
    }

    /// Validate scheduler performance
    pub fn validate_performance(&mut self) -> PerformanceValidationResult {
        let current_time = crate::log::get_current_time_ms();
        
        // Capture performance snapshot
        let snapshot = self.capture_performance_snapshot();
        self.performance_history.push(snapshot.clone());
        
        // Analyze performance
        let analysis = self.analyze_performance(&snapshot);
        
        // Generate validation result
        PerformanceValidationResult {
            timestamp: current_time,
            snapshot,
            analysis,
            recommendations: self.generate_performance_recommendations(&analysis),
        }
    }

    /// Capture performance snapshot
    fn capture_performance_snapshot(&self) -> PerformanceSnapshot {
        // TODO: Implement actual performance measurement
        // For now, use placeholder values
        PerformanceSnapshot {
            timestamp: crate::log::get_current_time_ms(),
            context_switch_latency: 50, // 50μs
            scheduling_overhead: 10,     // 10μs
            task_throughput: 1000,      // 1000 tasks/sec
            cpu_utilization: 0.75,      // 75%
            memory_usage: 1024 * 1024,  // 1MB
        }
    }

    /// Analyze performance snapshot
    fn analyze_performance(&self, snapshot: &PerformanceSnapshot) -> PerformanceAnalysis {
        let mut analysis = PerformanceAnalysis::new();
        
        // Check context switch latency
        if snapshot.context_switch_latency > 100 {
            analysis.issues.push("High context switch latency detected".to_string());
            analysis.performance_score -= 0.2;
        }
        
        // Check scheduling overhead
        if snapshot.scheduling_overhead > 20 {
            analysis.issues.push("High scheduling overhead detected".to_string());
            analysis.performance_score -= 0.15;
        }
        
        // Check CPU utilization
        if snapshot.cpu_utilization > 0.9 {
            analysis.issues.push("High CPU utilization - potential bottleneck".to_string());
            analysis.performance_score -= 0.1;
        }
        
        // Check memory usage
        if snapshot.memory_usage > 10 * 1024 * 1024 { // 10MB
            analysis.issues.push("High memory usage detected".to_string());
            analysis.performance_score -= 0.1;
        }
        
        analysis
    }

    /// Generate performance recommendations
    fn generate_performance_recommendations(&self, analysis: &PerformanceAnalysis) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        for issue in &analysis.issues {
            match issue.as_str() {
                "High context switch latency detected" => {
                    recommendations.push("Optimize context switch path - reduce register saves".to_string());
                }
                "High scheduling overhead detected" => {
                    recommendations.push("Simplify scheduling algorithm - reduce complexity".to_string());
                }
                "High CPU utilization - potential bottleneck" => {
                    recommendations.push("Consider adding more CPU cores or optimizing workload".to_string());
                }
                "High memory usage detected" => {
                    recommendations.push("Review memory allocation patterns - implement pooling".to_string());
                }
                _ => {
                    recommendations.push("Review and optimize scheduler implementation".to_string());
                }
            }
        }
        
        recommendations
    }
}

/// Performance analysis
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    pub performance_score: f32,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

impl PerformanceAnalysis {
    /// Create new performance analysis
    pub fn new() -> Self {
        Self {
            performance_score: 1.0,
            issues: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Performance validation result
#[derive(Debug, Clone)]
pub struct PerformanceValidationResult {
    pub timestamp: u64,
    pub snapshot: PerformanceSnapshot,
    pub analysis: PerformanceAnalysis,
    pub recommendations: Vec<String>,
}

/// Run fairness analysis with default configuration
pub fn run_fairness_analysis() -> SchedulerFairnessAnalysis {
    let config = FairnessAnalysisConfig::default();
    let mut analysis = SchedulerFairnessAnalysis::new(config);
    analysis.start_analysis();
    analysis
}

/// Run performance validation with default configuration
pub fn run_performance_validation() -> PerformanceValidationResult {
    let config = FairnessAnalysisConfig::default();
    let mut validator = SchedulerPerformanceValidator::new(config);
    validator.validate_performance()
}

/// Initialize the scheduler fairness analysis system
pub fn init() {
    kprintln!("[Scheduler Fairness] Initializing fairness analysis system");
    
    // Initialize fairness analysis
    let config = FairnessAnalysisConfig::default();
    let mut analysis = SchedulerFairnessAnalysis::new(config);
    analysis.start_analysis();
    
    kprintln!("[Scheduler Fairness] Fairness analysis system initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fairness_analysis_config() {
        let config = FairnessAnalysisConfig::default();
        assert_eq!(config.analysis_window_ms, 1000);
        assert_eq!(config.fairness_threshold, 0.8);
        assert!(config.enable_detailed_analysis);
    }

    #[test]
    fn test_task_fairness_metrics() {
        let mut metrics = TaskFairnessMetrics::new(1, TaskPriority::High);
        metrics.total_runtime = 100;
        metrics.expected_runtime = 200;
        metrics.calculate_fairness_score();
        
        assert_eq!(metrics.fairness_score, 0.5);
    }

    #[test]
    fn test_scheduler_fairness_analysis() {
        let config = FairnessAnalysisConfig::default();
        let analysis = SchedulerFairnessAnalysis::new(config);
        assert_eq!(analysis.task_metrics.len(), 0);
        assert_eq!(analysis.global_metrics.total_tasks, 0);
    }

    #[test]
    fn test_performance_validator() {
        let config = FairnessAnalysisConfig::default();
        let validator = SchedulerPerformanceValidator::new(config);
        assert_eq!(validator.performance_history.len(), 0);
    }
}
