/// Kernel Tracing System for Polymera OS
/// 
/// Provides comprehensive tracing capabilities for process switching, IPC operations,
/// and system performance monitoring. Includes atomic counters for statistics
/// collection and analysis.

use core::sync::atomic::{AtomicU64, Ordering};
use crate::kprintln;
use spin::Mutex;

/// IPC Latency Histogram for performance tracking
/// Uses a tiny histogram with fixed buckets for efficient memory usage
#[derive(Debug, Clone)]
pub struct IpcLatencyHistogram {
    /// Fixed histogram buckets in microseconds
    buckets: [u32; 16],
    /// Total samples recorded
    total_samples: u64,
    /// Sum of all latencies for mean calculation
    latency_sum: u64,
    /// Minimum latency observed
    min_latency: u32,
    /// Maximum latency observed
    max_latency: u32,
}

impl IpcLatencyHistogram {
    /// Create a new latency histogram
    pub const fn new() -> Self {
        Self {
            buckets: [0; 16],
            total_samples: 0,
            latency_sum: 0,
            min_latency: u32::MAX,
            max_latency: 0,
        }
    }
    
    /// Record a latency measurement in microseconds
    pub fn record(&mut self, latency_us: u32) {
        self.total_samples += 1;
        self.latency_sum += latency_us as u64;
        
        if latency_us < self.min_latency {
            self.min_latency = latency_us;
        }
        if latency_us > self.max_latency {
            self.max_latency = latency_us;
        }
        
        // Map latency to histogram bucket
        let bucket_idx = self.latency_to_bucket(latency_us);
        if bucket_idx < self.buckets.len() {
            self.buckets[bucket_idx] += 1;
        }
    }
    
    /// Convert latency to histogram bucket index
    fn latency_to_bucket(&self, latency_us: u32) -> usize {
        // Bucket boundaries: 0, 10, 25, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 25000, 50000, 100000, 250000, 500000 μs
        match latency_us {
            0..=9 => 0,
            10..=24 => 1,
            25..=49 => 2,
            50..=99 => 3,
            100..=199 => 4,
            200..=499 => 5,
            500..=999 => 6,
            1000..=1999 => 7,
            2000..=4999 => 8,
            5000..=9999 => 9,
            10000..=24999 => 10,
            25000..=49999 => 11,
            50000..=99999 => 12,
            100000..=249999 => 13,
            250000..=499999 => 14,
            _ => 15,
        }
    }
    
    /// Get bucket boundaries in microseconds
    pub fn bucket_boundaries() -> [u32; 16] {
        [0, 10, 25, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 25000, 50000, 100000, 250000, 500000]
    }
    
    /// Calculate p50 (median) latency in microseconds
    pub fn p50(&self) -> Option<u32> {
        if self.total_samples == 0 {
            return None;
        }
        
        let target_count = self.total_samples / 2;
        self.percentile_at_count(target_count)
    }
    
    /// Calculate p95 latency in microseconds
    pub fn p95(&self) -> Option<u32> {
        if self.total_samples == 0 {
            return None;
        }
        
        let target_count = (self.total_samples * 95) / 100;
        self.percentile_at_count(target_count)
    }
    
    /// Calculate p99 latency in microseconds
    pub fn p99(&self) -> Option<u32> {
        if self.total_samples == 0 {
            return None;
        }
        
        let target_count = (self.total_samples * 99) / 100;
        self.percentile_at_count(target_count)
    }
    
    /// Calculate percentile at specific count
    fn percentile_at_count(&self, target_count: u64) -> Option<u32> {
        let mut cumulative_count = 0;
        let boundaries = Self::bucket_boundaries();
        
        for (i, &count) in self.buckets.iter().enumerate() {
            cumulative_count += count as u64;
            if cumulative_count >= target_count {
                return Some(boundaries[i]);
            }
        }
        
        // If we reach here, return the last boundary
        Some(boundaries[boundaries.len() - 1])
    }
    
    /// Get mean latency in microseconds
    pub fn mean(&self) -> Option<u32> {
        if self.total_samples == 0 {
            None
        } else {
            Some((self.latency_sum / self.total_samples) as u32)
        }
    }
    
    /// Get minimum latency in microseconds
    pub fn min(&self) -> Option<u32> {
        if self.min_latency == u32::MAX {
            None
        } else {
            Some(self.min_latency)
        }
    }
    
    /// Get maximum latency in microseconds
    pub fn max(&self) -> Option<u32> {
        if self.max_latency == 0 {
            None
        } else {
            Some(self.max_latency)
        }
    }
    
    /// Get total sample count
    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }
    
    /// Get histogram buckets
    pub fn buckets(&self) -> [u32; 16] {
        self.buckets
    }
    
    /// Reset histogram
    pub fn reset(&mut self) {
        self.buckets = [0; 16];
        self.total_samples = 0;
        self.latency_sum = 0;
        self.min_latency = u32::MAX;
        self.max_latency = 0;
    }
}

/// Wake-to-Run Latency Histogram for performance tracking
/// Tracks the time from when a task is woken until it's actually running on CPU
#[derive(Debug, Clone)]
pub struct WakeToRunHistogram {
    /// Fixed histogram buckets in microseconds
    buckets: [u32; 16],
    /// Total samples recorded
    total_samples: u64,
    /// Sum of all latencies for mean calculation
    latency_sum: u64,
    /// Minimum latency observed
    min_latency: u32,
    /// Maximum latency observed
    max_latency: u32,
}

impl WakeToRunHistogram {
    /// Create a new wake-to-run latency histogram
    pub const fn new() -> Self {
        Self {
            buckets: [0; 16],
            total_samples: 0,
            latency_sum: 0,
            min_latency: u32::MAX,
            max_latency: 0,
        }
    }
    
    /// Record a wake-to-run latency measurement in microseconds
    pub fn record(&mut self, latency_us: u32) {
        self.total_samples += 1;
        self.latency_sum += latency_us as u64;
        
        if latency_us < self.min_latency {
            self.min_latency = latency_us;
        }
        if latency_us > self.max_latency {
            self.max_latency = latency_us;
        }
        
        // Map latency to histogram bucket
        let bucket_idx = self.latency_to_bucket(latency_us);
        if bucket_idx < self.buckets.len() {
            self.buckets[bucket_idx] += 1;
        }
    }
    
    /// Convert latency to histogram bucket index
    fn latency_to_bucket(&self, latency_us: u32) -> usize {
        // Bucket boundaries: 0, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 25000, 50000, 100000, 250000, 500000 μs
        // Optimized for wake-to-run latencies (typically 1μs to 5ms)
        match latency_us {
            0..=9 => 0,
            10..=24 => 1,
            25..=49 => 2,
            50..=99 => 3,
            100..=249 => 4,
            250..=499 => 5,
            500..=999 => 6,
            1000..=2499 => 7,
            2500..=4999 => 8,
            5000..=9999 => 9,
            10000..=24999 => 10,
            25000..=49999 => 11,
            50000..=99999 => 12,
            100000..=249999 => 13,
            250000..=499999 => 14,
            _ => 15,
        }
    }
    
    /// Get bucket boundaries in microseconds
    pub fn bucket_boundaries() -> [u32; 16] {
        [0, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 25000, 50000, 100000, 250000, 500000]
    }
    
    /// Calculate p50 (median) latency in microseconds
    pub fn p50(&self) -> Option<u32> {
        if self.total_samples == 0 {
            return None;
        }
        
        let target_count = self.total_samples / 2;
        self.percentile_at_count(target_count)
    }
    
    /// Calculate p95 latency in microseconds
    pub fn p95(&self) -> Option<u32> {
        if self.total_samples == 0 {
            return None;
        }
        
        let target_count = (self.total_samples * 95) / 100;
        self.percentile_at_count(target_count)
    }
    
    /// Calculate p99 latency in microseconds
    pub fn p99(&self) -> Option<u32> {
        if self.total_samples == 0 {
            return None;
        }
        
        let target_count = (self.total_samples * 99) / 100;
        self.percentile_at_count(target_count)
    }
    
    /// Calculate percentile at specific count
    fn percentile_at_count(&self, target_count: u64) -> Option<u32> {
        let mut cumulative_count = 0;
        let boundaries = Self::bucket_boundaries();
        
        for (i, &count) in self.buckets.iter().enumerate() {
            cumulative_count += count as u64;
            if cumulative_count >= target_count {
                return Some(boundaries[i]);
            }
        }
        
        // If we reach here, return the last boundary
        Some(boundaries[boundaries.len() - 1])
    }
    
    /// Get mean latency in microseconds
    pub fn mean(&self) -> Option<u32> {
        if self.total_samples == 0 {
            None
        } else {
            Some((self.latency_sum / self.total_samples) as u32)
        }
    }
    
    /// Get minimum latency in microseconds
    pub fn min(&self) -> Option<u32> {
        if self.min_latency == u32::MAX {
            None
        } else {
            Some(self.min_latency)
        }
    }
    
    /// Get maximum latency in microseconds
    pub fn max(&self) -> Option<u32> {
        if self.max_latency == 0 {
            None
        } else {
            Some(self.max_latency)
        }
    }
    
    /// Get total sample count
    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }
    
    /// Get histogram buckets
    pub fn buckets(&self) -> [u32; 16] {
        self.buckets
    }
    
    /// Reset histogram
    pub fn reset(&mut self) {
        self.buckets = [0; 16];
        self.total_samples = 0;
        self.latency_sum = 0;
        self.min_latency = u32::MAX;
        self.max_latency = 0;
    }
}

/// System statistics structure returned by sys_stats()
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SystemStats {
    /// Total timer ticks since boot
    pub ticks: u64,
    
    /// Total context switches performed
    pub ctx_switches: u64,
    
    /// Total messages sent via IPC
    pub msgs_sent: u64,
    
    /// Total messages received via IPC
    pub msgs_recvd: u64,
    
    /// System uptime in milliseconds
    pub uptime_ms: u64,
    
    /// Current number of active tasks
    pub active_tasks: u32,
    
    /// Current number of blocked tasks
    pub blocked_tasks: u32,
    
    /// Total page faults handled
    pub page_faults: u64,
    
    /// Total system calls made
    pub syscalls: u64,
    
    /// Total security validation failures
    pub security_failures: u64,
    
    /// IPC latency p50 in microseconds
    pub ipc_latency_p50_us: u32,
    
    /// IPC latency p95 in microseconds
    pub ipc_latency_p95_us: u32,
    
    /// IPC latency p99 in microseconds
    pub ipc_latency_p99_us: u32,
    
    /// IPC latency mean in microseconds
    pub ipc_latency_mean_us: u32,
    
    /// IPC latency min in microseconds
    pub ipc_latency_min_us: u32,
    
    /// IPC latency max in microseconds
    pub ipc_latency_max_us: u32,
    
    /// Total IPC latency samples
    pub ipc_latency_samples: u64,
    
    /// Wake-to-run latency p50 in microseconds
    pub wake_to_run_p50_us: u32,
    
    /// Wake-to-run latency p95 in microseconds
    pub wake_to_run_p95_us: u32,
    
    /// Wake-to-run latency p99 in microseconds
    pub wake_to_run_p99_us: u32,
    
    /// Wake-to-run latency mean in microseconds
    pub wake_to_run_mean_us: u32,
    
    /// Wake-to-run latency min in microseconds
    pub wake_to_run_min_us: u32,
    
    /// Wake-to-run latency max in microseconds
    pub wake_to_run_max_us: u32,
    
    /// Total wake-to-run latency samples
    pub wake_to_run_samples: u64,
    
    /// IPC authentication successful count
    pub ipc_auth_ok: u64,
    
    /// IPC authentication failed count
    pub ipc_auth_fail: u64,
    
    /// IPC capability-only authentication count
    pub ipc_cap_only_auth: u64,
    
    /// IPC full authentication count
    pub ipc_full_auth: u64,
    
    /// IPC MAC validation count
    pub ipc_mac_validations: u64,
    
    /// Average IPC authentication overhead in microseconds
    pub ipc_auth_avg_overhead_us: u32,
    
    /// Jitter budget active status
    pub jitter_budget_active: u32,
    
    /// Jitter budget activations count
    pub jitter_budget_activations: u64,
    
    /// Current jitter budget ticks
    pub jitter_budget_current_ticks: u64,
    
    /// Total jitter budget ticks
    pub jitter_budget_total_ticks: u64,
    
    /// Current RT task boost factor (multiplied by 100 for fixed-point)
    pub rt_task_boost_factor: u32,
    
    /// Consecutive high jitter count
    pub consecutive_high_jitter: u32,
    
    /// Jitter threshold in microseconds
    pub jitter_threshold_us: u32,
}

impl SystemStats {
    /// Create a new SystemStats with all counters at zero
    pub const fn zero() -> Self {
        Self {
            ticks: 0,
            ctx_switches: 0,
            msgs_sent: 0,
            msgs_recvd: 0,
            uptime_ms: 0,
            active_tasks: 0,
            blocked_tasks: 0,
            page_faults: 0,
            syscalls: 0,
            security_failures: 0,
            ipc_latency_p50_us: 0,
            ipc_latency_p95_us: 0,
            ipc_latency_p99_us: 0,
            ipc_latency_mean_us: 0,
            ipc_latency_min_us: 0,
            ipc_latency_max_us: 0,
            ipc_latency_samples: 0,
            wake_to_run_p50_us: 0,
            wake_to_run_p95_us: 0,
            wake_to_run_p99_us: 0,
            wake_to_run_mean_us: 0,
            wake_to_run_min_us: 0,
            wake_to_run_max_us: 0,
            wake_to_run_samples: 0,
            ipc_auth_ok: 0,
            ipc_auth_fail: 0,
            ipc_cap_only_auth: 0,
            ipc_full_auth: 0,
            ipc_mac_validations: 0,
            ipc_auth_avg_overhead_us: 0,
            jitter_budget_active: 0,
            jitter_budget_activations: 0,
            jitter_budget_current_ticks: 0,
            jitter_budget_total_ticks: 0,
            rt_task_boost_factor: 100, // 1.0x boost (no boost)
            consecutive_high_jitter: 0,
            jitter_threshold_us: 250,
        }
    }
    
    /// Calculate messages per second based on uptime
    pub fn msgs_per_second(&self) -> f32 {
        if self.uptime_ms > 0 {
            ((self.msgs_sent + self.msgs_recvd) as f32 * 1000.0) / (self.uptime_ms as f32)
        } else {
            0.0
        }
    }
    
    /// Calculate context switches per second
    pub fn ctx_switches_per_second(&self) -> f32 {
        if self.uptime_ms > 0 {
            (self.ctx_switches as f32 * 1000.0) / (self.uptime_ms as f32)
        } else {
            0.0
        }
    }
    
    /// Calculate average ticks per second (should be close to timer frequency)
    pub fn ticks_per_second(&self) -> f32 {
        if self.uptime_ms > 0 {
            (self.ticks as f32 * 1000.0) / (self.uptime_ms as f32)
        } else {
            0.0
        }
    }
}

/// Global atomic counters for system statistics
static TICKS_COUNTER: AtomicU64 = AtomicU64::new(0);
static CTX_SWITCHES_COUNTER: AtomicU64 = AtomicU64::new(0);
static MSGS_SENT_COUNTER: AtomicU64 = AtomicU64::new(0);
static MSGS_RECVD_COUNTER: AtomicU64 = AtomicU64::new(0);
static PAGE_FAULTS_COUNTER: AtomicU64 = AtomicU64::new(0);
static SYSCALLS_COUNTER: AtomicU64 = AtomicU64::new(0);
static SECURITY_FAILURES_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Global IPC latency histogram
static IPC_LATENCY_HISTOGRAM: Mutex<IpcLatencyHistogram> = Mutex::new(IpcLatencyHistogram::new());

/// Global wake-to-run latency histogram
static WAKE_TO_RUN_HISTOGRAM: Mutex<WakeToRunHistogram> = Mutex::new(WakeToRunHistogram::new());

/// Initialize the tracing subsystem
pub fn init_trace() {
    kprintln!("[TRACE] Initializing kernel tracing subsystem");
    
    // Clear all counters
    TICKS_COUNTER.store(0, Ordering::Relaxed);
    CTX_SWITCHES_COUNTER.store(0, Ordering::Relaxed);
    MSGS_SENT_COUNTER.store(0, Ordering::Relaxed);
    MSGS_RECVD_COUNTER.store(0, Ordering::Relaxed);
    PAGE_FAULTS_COUNTER.store(0, Ordering::Relaxed);
    SYSCALLS_COUNTER.store(0, Ordering::Relaxed);
    SECURITY_FAILURES_COUNTER.store(0, Ordering::Relaxed);
    
    // Clear histograms
    IPC_LATENCY_HISTOGRAM.lock().reset();
    WAKE_TO_RUN_HISTOGRAM.lock().reset();
    
    kprintln!("[TRACE] Kernel tracing subsystem initialized");
}

/// Record IPC latency measurement (called when message is received)
pub fn record_ipc_latency(latency_us: u32) {
    let mut histogram = IPC_LATENCY_HISTOGRAM.lock();
    histogram.record(latency_us);
}

/// Get current IPC latency statistics
pub fn get_ipc_latency_stats() -> (u32, u32, u32, u32, u32, u32, u64) {
    let histogram = IPC_LATENCY_HISTOGRAM.lock();
    (
        histogram.p50().unwrap_or(0),
        histogram.p95().unwrap_or(0),
        histogram.p99().unwrap_or(0),
        histogram.mean().unwrap_or(0),
        histogram.min().unwrap_or(0),
        histogram.max().unwrap_or(0),
        histogram.total_samples(),
    )
}

/// Record wake-to-run latency measurement (called when task starts running)
pub fn record_wake_to_run_latency(latency_us: u32) {
    let mut histogram = WAKE_TO_RUN_HISTOGRAM.lock();
    histogram.record(latency_us);
}

/// Get current wake-to-run latency statistics
pub fn get_wake_to_run_stats() -> (u32, u32, u32, u32, u32, u32, u64) {
    let histogram = WAKE_TO_RUN_HISTOGRAM.lock();
    (
        histogram.p50().unwrap_or(0),
        histogram.p95().unwrap_or(0),
        histogram.p99().unwrap_or(0),
        histogram.mean().unwrap_or(0),
        histogram.min().unwrap_or(0),
        histogram.max().unwrap_or(0),
        histogram.total_samples(),
    )
}

/// Increment the tick counter (called by timer interrupt)
pub fn trace_tick() {
    if cfg!(feature = "deterministic") && crate::determinism::is_determinism_enabled() {
        crate::determinism::advance_global_ticks(1);
    } else {
        TICKS_COUNTER.fetch_add(1, Ordering::Relaxed);
    }
}

/// Increment the context switch counter
pub fn trace_context_switch() {
    CTX_SWITCHES_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Increment the message sent counter
pub fn trace_message_sent() {
    MSGS_SENT_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Increment the message received counter
pub fn trace_message_received() {
    MSGS_RECVD_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Increment the page fault counter
pub fn trace_page_fault() {
    PAGE_FAULTS_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Increment the syscall counter
pub fn trace_syscall() {
    SYSCALLS_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Increment the security failure counter
pub fn trace_security_failure() {
    SECURITY_FAILURES_COUNTER.fetch_add(1, Ordering::Relaxed);
}

/// Get current system statistics
pub fn get_system_stats() -> SystemStats {
    let uptime_ms = if cfg!(feature = "deterministic") && crate::determinism::is_determinism_enabled() {
        crate::determinism::get_global_time_ms()
    } else {
        crate::log::get_current_time_ms()
    };
    
    // Count active and blocked tasks (placeholder - integrate with actual scheduler)
    let active_tasks = 1; // TODO: Get from scheduler
    let blocked_tasks = 0; // TODO: Get from scheduler
    
    // Get IPC latency statistics
    let (p50, p95, p99, mean, min, max, samples) = get_ipc_latency_stats();
    
    // Get wake-to-run latency statistics
    let (wake_p50, wake_p95, wake_p99, wake_mean, wake_min, wake_max, wake_samples) = get_wake_to_run_stats();
    
    SystemStats {
        ticks: if cfg!(feature = "deterministic") && crate::determinism::is_determinism_enabled() {
            crate::determinism::get_global_ticks()
        } else {
            TICKS_COUNTER.load(Ordering::Relaxed)
        },
        ctx_switches: CTX_SWITCHES_COUNTER.load(Ordering::Relaxed),
        msgs_sent: MSGS_SENT_COUNTER.load(Ordering::Relaxed),
        msgs_recvd: MSGS_RECVD_COUNTER.load(Ordering::Relaxed),
        uptime_ms,
        active_tasks,
        blocked_tasks,
        page_faults: PAGE_FAULTS_COUNTER.load(Ordering::Relaxed),
        syscalls: SYSCALLS_COUNTER.load(Ordering::Relaxed),
        security_failures: SECURITY_FAILURES_COUNTER.load(Ordering::Relaxed),
        ipc_latency_p50_us: p50,
        ipc_latency_p95_us: p95,
        ipc_latency_p99_us: p99,
        ipc_latency_mean_us: mean,
        ipc_latency_min_us: min,
        ipc_latency_max_us: max,
        ipc_latency_samples: samples,
        wake_to_run_p50_us: wake_p50,
        wake_to_run_p95_us: wake_p95,
        wake_to_run_p99_us: wake_p99,
        wake_to_run_mean_us: wake_mean,
        wake_to_run_min_us: wake_min,
        wake_to_run_max_us: wake_max,
        wake_to_run_samples: wake_samples,
        ipc_auth_ok: 0, // Will be populated from IPC auth manager
        ipc_auth_fail: 0, // Will be populated from IPC auth manager
        ipc_cap_only_auth: 0, // Will be populated from IPC auth manager
        ipc_full_auth: 0, // Will be populated from IPC auth manager
        ipc_mac_validations: 0, // Will be populated from IPC auth manager
        ipc_auth_avg_overhead_us: 0, // Will be populated from IPC auth manager
        jitter_budget_active: if crate::sched::tick::is_jitter_budget_active() { 1 } else { 0 },
        jitter_budget_activations: crate::sched::tick::get_jitter_budget().get_total_overruns(),
        jitter_budget_current_ticks: crate::sched::tick::get_jitter_budget().get_boost_remaining_ms() as u64,
        jitter_budget_total_ticks: crate::sched::tick::get_jitter_budget().get_total_overruns(),
        rt_task_boost_factor: (crate::sched::tick::get_rt_task_boost_factor() * 100.0) as u32,
        consecutive_high_jitter: crate::sched::tick::get_jitter_budget().get_consecutive_overruns(),
        jitter_threshold_us: crate::sched::tick::get_jitter_budget().get_max_jitter_us(),
    }
}

/// Print comprehensive system statistics
pub fn print_system_stats() {
    let stats = get_system_stats();
    
    kprintln!("");
    kprintln!("=== POLYMERA OS SYSTEM STATISTICS ===");
    kprintln!("System Uptime: {}ms ({:.1}s)", stats.uptime_ms, stats.uptime_ms as f32 / 1000.0);
    kprintln!("");
    
    kprintln!("Timer & Scheduling:");
    kprintln!("  Timer ticks: {} ({:.1} Hz)", stats.ticks, stats.ticks_per_second());
    kprintln!("  Context switches: {} ({:.2}/s)", stats.ctx_switches, stats.ctx_switches_per_second());
    kprintln!("  Active tasks: {}", stats.active_tasks);
    kprintln!("  Blocked tasks: {}", stats.blocked_tasks);
    kprintln!("");
    
    kprintln!("Inter-Process Communication:");
    kprintln!("  Messages sent: {}", stats.msgs_sent);
    kprintln!("  Messages received: {}", stats.msgs_recvd);
    kprintln!("  Total messages: {}", stats.msgs_sent + stats.msgs_recvd);
    kprintln!("  Message rate: {:.2} msgs/s", stats.msgs_per_second());
    kprintln!("");
    
    kprintln!("IPC Latency Statistics ({} samples):", stats.ipc_latency_samples);
    kprintln!("  P50 (median): {}μs", stats.ipc_latency_p50_us);
    kprintln!("  P95: {}μs", stats.ipc_latency_p95_us);
    kprintln!("  P99: {}μs", stats.ipc_latency_p99_us);
    kprintln!("  Mean: {}μs", stats.ipc_latency_mean_us);
    kprintln!("  Min: {}μs", stats.ipc_latency_min_us);
    kprintln!("  Max: {}μs", stats.ipc_latency_max_us);
    kprintln!("");
    
    kprintln!("Wake-to-Run Latency Statistics ({} samples):", stats.wake_to_run_samples);
    kprintln!("  P50 (median): {}μs", stats.wake_to_run_p50_us);
    kprintln!("  P95: {}μs", stats.wake_to_run_p95_us);
    kprintln!("  P99: {}μs", stats.wake_to_run_p99_us);
    kprintln!("  Mean: {}μs", stats.wake_to_run_mean_us);
    kprintln!("  Min: {}μs", stats.wake_to_run_min_us);
    kprintln!("  Max: {}μs", stats.wake_to_run_max_us);
    kprintln!("");
    
    kprintln!("System Activity:");
    kprintln!("  System calls: {}", stats.syscalls);
    kprintln!("  Page faults: {}", stats.page_faults);
    kprintln!("  Security failures: {}", stats.security_failures);
    kprintln!("");
    
    kprintln!("IPC Authentication Statistics:");
    kprintln!("  Authentication successful: {}", stats.ipc_auth_ok);
    kprintln!("  Authentication failed: {}", stats.ipc_auth_fail);
    kprintln!("  Capability-only auth: {}", stats.ipc_cap_only_auth);
    kprintln!("  Full authentication: {}", stats.ipc_full_auth);
    kprintln!("  MAC validations: {}", stats.ipc_mac_validations);
    kprintln!("  Average auth overhead: {}μs", stats.ipc_auth_avg_overhead_us);
    kprintln!("");
    
    kprintln!("Jitter Budget Management:");
    kprintln!("  Budget Active: {}", if stats.jitter_budget_active > 0 { "Yes" } else { "No" });
    kprintln!("  Budget Activations: {}", stats.jitter_budget_activations);
    kprintln!("  Current Budget Ticks: {}/{}", stats.jitter_budget_current_ticks, 100); // Default duration
    kprintln!("  Total Budget Ticks: {}", stats.jitter_budget_total_ticks);
    kprintln!("  RT Task Boost Factor: {:.2}x", stats.rt_task_boost_factor as f32 / 100.0);
    kprintln!("  Consecutive High Jitter: {}/{}", stats.consecutive_high_jitter, 3); // Default threshold
    kprintln!("  Jitter Threshold: {}μs", stats.jitter_threshold_us);
    kprintln!("");
    
    // Calculate efficiency metrics
    if stats.ticks > 0 {
        let ctx_switches_per_tick = stats.ctx_switches as f32 / stats.ticks as f32;
        let msgs_per_tick = (stats.msgs_sent + stats.msgs_recvd) as f32 / stats.ticks as f32;
        
        kprintln!("Efficiency Metrics:");
        kprintln!("  Context switches per tick: {:.4}", ctx_switches_per_tick);
        kprintln!("  Messages per tick: {:.4}", msgs_per_tick);
        
        if stats.ctx_switches > 0 {
            let avg_task_runtime = stats.uptime_ms as f32 / stats.ctx_switches as f32;
            kprintln!("  Average task runtime: {:.2}ms", avg_task_runtime);
        }
    }
    
    kprintln!("=== END SYSTEM STATISTICS ===");
    kprintln!("");
}

/// Print IPC latency histogram
pub fn print_ipc_latency_histogram() {
    let histogram = IPC_LATENCY_HISTOGRAM.lock();
    let boundaries = IpcLatencyHistogram::bucket_boundaries();
    let buckets = histogram.buckets();
    
    kprintln!("");
    kprintln!("=== IPC LATENCY HISTOGRAM ({} samples) ===", histogram.total_samples());
    kprintln!("Bucket (μs)    Count    %");
    kprintln!("------------------------");
    
    let total = histogram.total_samples() as f32;
    for (i, &count) in buckets.iter().enumerate() {
        if count > 0 {
            let percentage = (count as f32 / total) * 100.0;
            kprintln!("{:>8}    {:>5}  {:>5.1}%", boundaries[i], count, percentage);
        }
    }
    
    kprintln!("");
    kprintln!("Percentiles:");
    if let Some(p50) = histogram.p50() {
        kprintln!("  P50 (median): {}μs", p50);
    }
    if let Some(p95) = histogram.p95() {
        kprintln!("  P95: {}μs", p95);
    }
    if let Some(p99) = histogram.p99() {
        kprintln!("  P99: {}μs", p99);
    }
    if let Some(mean) = histogram.mean() {
        kprintln!("  Mean: {}μs", mean);
    }
    if let Some(min) = histogram.min() {
        kprintln!("  Min: {}μs", min);
    }
    if let Some(max) = histogram.max() {
        kprintln!("  Max: {}μs", max);
    }
    
    kprintln!("=== END IPC LATENCY HISTOGRAM ===");
    kprintln!("");
}

/// Print wake-to-run latency histogram
pub fn print_wake_to_run_histogram() {
    let histogram = WAKE_TO_RUN_HISTOGRAM.lock();
    let boundaries = WakeToRunHistogram::bucket_boundaries();
    let buckets = histogram.buckets();
    
    kprintln!("");
    kprintln!("=== WAKE-TO-RUN LATENCY HISTOGRAM ({} samples) ===", histogram.total_samples());
    kprintln!("Bucket (μs)    Count    %");
    kprintln!("------------------------");
    
    let total = histogram.total_samples() as f32;
    for (i, &count) in buckets.iter().enumerate() {
        if count > 0 {
            let percentage = (count as f32 / total) * 100.0;
            kprintln!("{:>8}    {:>5}  {:>5.1}%", boundaries[i], count, percentage);
        }
    }
    
    kprintln!("");
    kprintln!("Percentiles:");
    if let Some(p50) = histogram.p50() {
        kprintln!("  P50 (median): {}μs", p50);
    }
    if let Some(p95) = histogram.p95() {
        kprintln!("  P95: {}μs", p95);
    }
    if let Some(p99) = histogram.p99() {
        kprintln!("  P99: {}μs", p99);
    }
    if let Some(mean) = histogram.mean() {
        kprintln!("  Mean: {}μs", mean);
    }
    if let Some(min) = histogram.min() {
        kprintln!("  Min: {}μs", min);
    }
    if let Some(max) = histogram.max() {
        kprintln!("  Max: {}μs", max);
    }
    
    kprintln!("=== END WAKE-TO-RUN LATENCY HISTOGRAM ===");
    kprintln!("");
}

/// Test the tracing system
pub fn test_trace() {
    kprintln!("");
    kprintln!("=== TRACING SYSTEM TEST ===");
    
    // Test IPC latency histogram
    kprintln!("Testing IPC latency histogram...");
    record_ipc_latency(50);
    record_ipc_latency(100);
    record_ipc_latency(150);
    record_ipc_latency(200);
    record_ipc_latency(250);
    
    // Test wake-to-run latency histogram
    kprintln!("Testing wake-to-run latency histogram...");
    record_wake_to_run_latency(10);
    record_wake_to_run_latency(25);
    record_wake_to_run_latency(50);
    record_wake_to_run_latency(100);
    record_wake_to_run_latency(250);
    
    // Test counter increments
    kprintln!("Testing counter increments...");
    for _ in 0..5 {
        trace_tick();
        trace_syscall();
    }
    trace_page_fault();
    trace_security_failure();
    trace_message_sent();
    trace_message_received();
    trace_context_switch();
    
    // Print current statistics
    print_system_stats();
    print_ipc_latency_histogram();
    print_wake_to_run_histogram();
    
    kprintln!("=== TRACING SYSTEM TEST COMPLETE ===");
    kprintln!("");
}

/// IPC operation kind for fabric-level tracing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcOperation {
    Send,
    Receive,
    CreateChannel,
    DestroyChannel,
    Poll,
}

/// Lightweight IPC trace event (sender, receiver, payload size, priority).
#[inline]
pub fn trace_ipc<S, D, P>(_sender: S, _dst: D, _payload_size: u32, _priority: P) {
    trace_message_sent();
}

/// Generic IPC operation trace (used by extended tracing paths).
#[inline]
pub fn trace_ipc_operation<A, B, C, D>(
    _op: IpcOperation,
    _a: A,
    _b: B,
    _c: C,
    _d: D,
) {
    trace_message_received();
}

/// Record a process switch event (stub – feeds the context-switch counter).
#[inline]
pub fn trace_process_switch<A, B>(_from: A, _to: B) {
    trace_context_switch();
}

/// Dump the most recent process-switch records (stub).
pub fn print_recent_process_switches() {
    kprintln!("[trace] recent process switches: (none recorded)");
}

/// Dump the most recent IPC traces (stub).
pub fn print_recent_ipc_traces() {
    kprintln!("[trace] recent ipc traces: (none recorded)");
}

/// Print per-task statistics (stub).
pub fn print_task_stats() {
    kprintln!("[trace] task stats: (none recorded)");
}
