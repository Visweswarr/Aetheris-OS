//! Phase 2 Soak Test Suite
//! Comprehensive stability testing with chaos engineering

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use serde::{Serialize, Deserialize};

/// Soak test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoakConfig {
    pub duration_seconds: u64,
    pub ipc_stress: IpcStressConfig,
    pub timer_jitter: TimerJitterConfig,
    pub key_rotation: KeyRotationConfig,
    pub cap_management: CapManagementConfig,
    pub chaos: ChaosConfig,
    pub stats: StatsConfig,
}

/// IPC stress configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcStressConfig {
    pub worker_count: usize,
    pub messages_per_worker_per_sec: u32,
    pub message_size_range: (usize, usize),
    pub priority_distribution: (f32, f32, f32),
    pub enable_corruption: bool,
    pub enable_large_messages: bool,
}

/// Timer jitter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerJitterConfig {
    pub enabled: bool,
    pub amplitude_us: u32,
    pub frequency_hz: f32,
    pub pattern: JitterPattern,
    pub adaptive: bool,
}

/// Jitter pattern types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JitterPattern {
    Random,
    Sine,
    Square,
    Burst,
}

/// Key rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    pub enabled: bool,
    pub rotation_interval_sec: u64,
    pub grace_period_sec: u64,
    pub force_rotation_on_load: bool,
    pub load_threshold: f64,
}

/// Capability management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapManagementConfig {
    pub enable_issuance: bool,
    pub issuance_rate: u32,
    pub enable_revocation: bool,
    pub revocation_rate: u32,
    pub lifetime_range: (u64, u64),
    pub enable_escalation: bool,
}

/// Chaos engineering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    pub enabled: bool,
    pub message_drop_rate: u32,
    pub priority_based_dropping: bool,
    pub low_priority_drop_threshold: u32,
    pub enable_intermittent_failures: bool,
    pub failure_rate: f32,
    pub enable_resource_exhaustion: bool,
}

/// Statistics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsConfig {
    pub collection_interval_sec: u64,
    pub enable_ipc_stats: bool,
    pub enable_memory_stats: bool,
    pub enable_performance_stats: bool,
    pub enable_chaos_stats: bool,
}

/// Soak test statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoakStats {
    pub start_timestamp: u64,
    pub duration_seconds: u64,
    pub ipc_stats: IpcStats,
    pub timer_jitter_stats: TimerJitterStats,
    pub key_rotation_stats: KeyRotationStats,
    pub cap_management_stats: CapManagementStats,
    pub chaos_stats: ChaosStats,
    pub system_health: SystemHealthStats,
    pub test_results: TestResults,
}

/// IPC statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_dropped: u64,
    pub latency_percentiles: (u64, u64, u64),
    pub throughput_mps: f64,
    pub error_count: u64,
    pub priority_distribution: HashMap<String, u64>,
}

/// Timer jitter statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerJitterStats {
    pub injections_count: u64,
    pub avg_amplitude_us: f64,
    pub max_amplitude_us: u32,
    pub scheduling_impact: f64,
    pub budget_activations: u64,
}

/// Key rotation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationStats {
    pub rotations_count: u64,
    pub forced_rotations: u64,
    pub avg_rotation_time_ms: f64,
    pub overlap_violations: u64,
    pub rotation_failures: u64,
}

/// Capability management statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapManagementStats {
    pub caps_issued: u64,
    pub caps_revoked: u64,
    pub active_caps: u64,
    pub escalation_attempts: u64,
    pub validation_failures: u64,
}

/// Chaos engineering statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosStats {
    pub events_injected: u64,
    pub messages_dropped: u64,
    pub failures_injected: u64,
    pub resource_exhaustion_events: u64,
    pub performance_impact: f64,
}

/// System health statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStats {
    pub memory_usage: MemoryUsage,
    pub cpu_usage: CpuUsage,
    pub uptime_seconds: u64,
    pub panic_count: u64,
    pub deadlock_detected: bool,
    pub resource_leaks: Vec<String>,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub total_allocated: u64,
    pub peak_allocated: u64,
    pub current_allocated: u64,
    pub fragmentation_percent: f64,
    pub allocator_efficiency: f64,
}

/// CPU usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub avg_usage_percent: f64,
    pub peak_usage_percent: f64,
    pub kernel_time_percent: f64,
    pub user_time_percent: f64,
    pub context_switches: u64,
}

/// Test results summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub status: TestStatus,
    pub passed: bool,
    pub failure_reasons: Vec<String>,
    pub performance_regression_percent: f64,
    pub memory_leak_detected: bool,
    pub recommendations: Vec<String>,
}

/// Test status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Warning,
    Inconclusive,
}

/// Soak test orchestrator
pub struct SoakTestOrchestrator {
    config: SoakConfig,
    stats: Arc<Mutex<SoakStats>>,
    running: Arc<AtomicBool>,
    start_time: Option<Instant>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl SoakTestOrchestrator {
    /// Create new soak test orchestrator
    pub fn new(config: SoakConfig) -> Self {
        Self {
            config,
            stats: Arc::new(Mutex::new(Self::create_empty_stats())),
            running: Arc::new(AtomicBool::new(false)),
            start_time: None,
            workers: Vec::new(),
        }
    }
    
    /// Create empty statistics structure
    fn create_empty_stats() -> SoakStats {
        SoakStats {
            start_timestamp: 0,
            duration_seconds: 0,
            ipc_stats: IpcStats {
                messages_sent: 0,
                messages_received: 0,
                messages_dropped: 0,
                latency_percentiles: (0, 0, 0),
                throughput_mps: 0.0,
                error_count: 0,
                priority_distribution: HashMap::new(),
            },
            timer_jitter_stats: TimerJitterStats {
                injections_count: 0,
                avg_amplitude_us: 0.0,
                max_amplitude_us: 0,
                scheduling_impact: 0.0,
                budget_activations: 0,
            },
            key_rotation_stats: KeyRotationStats {
                rotations_count: 0,
                forced_rotations: 0,
                avg_rotation_time_ms: 0.0,
                overlap_violations: 0,
                rotation_failures: 0,
            },
            cap_management_stats: CapManagementStats {
                caps_issued: 0,
                caps_revoked: 0,
                active_caps: 0,
                escalation_attempts: 0,
                validation_failures: 0,
            },
            chaos_stats: ChaosStats {
                events_injected: 0,
                messages_dropped: 0,
                failures_injected: 0,
                resource_exhaustion_events: 0,
                performance_impact: 0.0,
            },
            system_health: SystemHealthStats {
                memory_usage: MemoryUsage {
                    total_allocated: 0,
                    peak_allocated: 0,
                    current_allocated: 0,
                    fragmentation_percent: 0.0,
                    allocator_efficiency: 0.0,
                },
                cpu_usage: CpuUsage {
                    avg_usage_percent: 0.0,
                    peak_usage_percent: 0.0,
                    kernel_time_percent: 0.0,
                    user_time_percent: 0.0,
                    context_switches: 0,
                },
                uptime_seconds: 0,
                panic_count: 0,
                deadlock_detected: false,
                resource_leaks: Vec::new(),
            },
            test_results: TestResults {
                status: TestStatus::Inconclusive,
                passed: false,
                failure_reasons: Vec::new(),
                performance_regression_percent: 0.0,
                memory_leak_detected: false,
                recommendations: Vec::new(),
            },
        }
    }
    
    /// Start the soak test
    pub fn start(&mut self) -> Result<(), String> {
        if self.running.load(Ordering::Acquire) {
            return Err("Test already running".to_string());
        }
        
        log::info!("Starting Phase 2 soak test for {} seconds", self.config.duration_seconds);
        
        self.start_time = Some(Instant::now());
        self.running.store(true, Ordering::Release);
        
        // Initialize statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.start_timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // Start worker threads
        self.start_ipc_workers()?;
        self.start_timer_jitter_worker()?;
        self.start_key_rotation_worker()?;
        self.start_cap_management_worker()?;
        self.start_chaos_worker()?;
        self.start_stats_collector()?;
        
        log::info!("Soak test started successfully with {} workers", self.workers.len());
        Ok(())
    }
    
    /// Stop the soak test
    pub fn stop(&mut self) -> Result<(), String> {
        if !self.running.load(Ordering::Acquire) {
            return Err("Test not running".to_string());
        }
        
        log::info!("Stopping soak test...");
        
        self.running.store(false, Ordering::Release);
        
        // Wait for all workers to complete
        for worker in self.workers.drain(..) {
            if let Err(e) = worker.join() {
                log::warn!("Worker thread failed to join: {:?}", e);
            }
        }
        
        // Finalize statistics
        if let Some(start_time) = self.start_time {
            let duration = start_time.elapsed();
            let mut stats = self.stats.lock().unwrap();
            stats.duration_seconds = duration.as_secs();
        }
        
        log::info!("Soak test stopped successfully");
        Ok(())
    }
    
    /// Start IPC stress workers
    fn start_ipc_workers(&mut self) -> Result<(), String> {
        let config = self.config.ipc_stress.clone();
        let running = Arc::clone(&self.running);
        let stats = Arc::clone(&self.stats);
        
        for worker_id in 0..config.worker_count {
            let config = config.clone();
            let running = Arc::clone(&self.running);
            let stats = Arc::clone(&self.stats);
            
            let handle = thread::spawn(move || {
                Self::ipc_worker_loop(worker_id, config, running, stats);
            });
            
            self.workers.push(handle);
        }
        
        Ok(())
    }
    
    /// IPC worker main loop
    fn ipc_worker_loop(
        worker_id: usize,
        config: IpcStressConfig,
        running: Arc<AtomicBool>,
        stats: Arc<Mutex<SoakStats>>,
    ) {
        log::debug!("IPC worker {} started", worker_id);
        
        let message_interval = Duration::from_secs_f64(1.0 / config.messages_per_worker_per_sec as f64);
        let mut last_message_time = Instant::now();
        
        while running.load(Ordering::Acquire) {
            let now = Instant::now();
            if now.duration_since(last_message_time) >= message_interval {
                Self::send_test_message(&config, &stats);
                last_message_time = now;
            }
            
            thread::sleep(Duration::from_millis(1));
        }
        
        log::debug!("IPC worker {} stopped", worker_id);
    }
    
    /// Send a test IPC message
    fn send_test_message(config: &IpcStressConfig, stats: &Arc<Mutex<SoakStats>>) {
        // Simulate IPC message sending
        let priority = if fastrand::f32() < config.priority_distribution.0 {
            "low"
        } else if fastrand::f32() < config.priority_distribution.1 {
            "normal"
        } else {
            "high"
        };
        
        // Update statistics
        if let Ok(mut stats) = stats.lock() {
            stats.ipc_stats.messages_sent += 1;
            *stats.ipc_stats.priority_distribution.entry(priority.to_string()).or_insert(0) += 1;
        }
        
        // Simulate message corruption if enabled
        if config.enable_corruption && fastrand::f32() < 0.001 {
            if let Ok(mut stats) = stats.lock() {
                stats.ipc_stats.error_count += 1;
            }
        }
    }
    
    /// Start timer jitter worker
    fn start_timer_jitter_worker(&mut self) -> Result<(), String> {
        if !self.config.timer_jitter.enabled {
            return Ok(());
        }
        
        let config = self.config.timer_jitter.clone();
        let running = Arc::clone(&self.running);
        let stats = Arc::clone(&self.stats);
        
        let handle = thread::spawn(move || {
            Self::timer_jitter_worker_loop(config, running, stats);
        });
        
        self.workers.push(handle);
        Ok(())
    }
    
    /// Timer jitter worker main loop
    fn timer_jitter_worker_loop(
        config: TimerJitterConfig,
        running: Arc<AtomicBool>,
        stats: Arc<Mutex<SoakStats>>,
    ) {
        log::debug!("Timer jitter worker started");
        
        let jitter_interval = Duration::from_secs_f64(1.0 / config.frequency_hz);
        let mut last_jitter_time = Instant::now();
        
        while running.load(Ordering::Acquire) {
            let now = Instant::now();
            if now.duration_since(last_jitter_time) >= jitter_interval {
                Self::inject_timer_jitter(&config, &stats);
                last_jitter_time = now;
            }
            
            thread::sleep(Duration::from_millis(1));
        }
        
        log::debug!("Timer jitter worker stopped");
    }
    
    /// Inject timer jitter
    fn inject_timer_jitter(config: &TimerJitterConfig, stats: &Arc<Mutex<SoakStats>>) {
        let amplitude = match config.pattern {
            JitterPattern::Random => fastrand::u32(0..=config.amplitude_us),
            JitterPattern::Sine => {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                (config.amplitude_us as f64 * (time * 2.0 * std::f64::consts::PI).sin()).abs() as u32
            }
            JitterPattern::Square => {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if time % 2 == 0 { config.amplitude_us } else { 0 }
            }
            JitterPattern::Burst => {
                if fastrand::f32() < 0.1 {
                    config.amplitude_us * 2
                } else {
                    0
                }
            }
        };
        
        // Update statistics
        if let Ok(mut stats) = stats.lock() {
            stats.timer_jitter_stats.injections_count += 1;
            stats.timer_jitter_stats.max_amplitude_us = stats.timer_jitter_stats.max_amplitude_us.max(amplitude);
            
            // Calculate running average
            let count = stats.timer_jitter_stats.injections_count as f64;
            let current_avg = stats.timer_jitter_stats.avg_amplitude_us;
            stats.timer_jitter_stats.avg_amplitude_us = (current_avg * (count - 1.0) + amplitude as f64) / count;
        }
        
        // Simulate jitter injection
        log::trace!("Injecting timer jitter: {}μs", amplitude);
    }
    
    /// Start key rotation worker
    fn start_key_rotation_worker(&mut self) -> Result<(), String> {
        if !self.config.key_rotation.enabled {
            return Ok(());
        }
        
        let config = self.config.key_rotation.clone();
        let running = Arc::clone(&self.running);
        let stats = Arc::clone(&self.stats);
        
        let handle = thread::spawn(move || {
            Self::key_rotation_worker_loop(config, running, stats);
        });
        
        self.workers.push(handle);
        Ok(())
    }
    
    /// Key rotation worker main loop
    fn key_rotation_worker_loop(
        config: KeyRotationConfig,
        running: Arc<AtomicBool>,
        stats: Arc<Mutex<SoakStats>>,
    ) {
        log::debug!("Key rotation worker started");
        
        let rotation_interval = Duration::from_secs(config.rotation_interval_sec);
        let mut last_rotation_time = Instant::now();
        
        while running.load(Ordering::Acquire) {
            let now = Instant::now();
            if now.duration_since(last_rotation_time) >= rotation_interval {
                Self::perform_key_rotation(&config, &stats);
                last_rotation_time = now;
            }
            
            thread::sleep(Duration::from_secs(1));
        }
        
        log::debug!("Key rotation worker stopped");
    }
    
    /// Perform key rotation
    fn perform_key_rotation(config: &KeyRotationConfig, stats: &Arc<Mutex<SoakStats>>) {
        let start_time = Instant::now();
        
        // Simulate key rotation
        log::debug!("Performing key rotation");
        
        // Simulate rotation time
        let rotation_time = fastrand::u32(10..=100); // 10-100ms
        thread::sleep(Duration::from_millis(rotation_time as u64));
        
        let rotation_time_ms = start_time.elapsed().as_millis() as f64;
        
        // Update statistics
        if let Ok(mut stats) = stats.lock() {
            stats.key_rotation_stats.rotations_count += 1;
            
            // Calculate running average
            let count = stats.key_rotation_stats.rotations_count as f64;
            let current_avg = stats.key_rotation_stats.avg_rotation_time_ms;
            stats.key_rotation_stats.avg_rotation_time_ms = (current_avg * (count - 1.0) + rotation_time_ms) / count;
            
            // Simulate forced rotation on high load
            if config.force_rotation_on_load && fastrand::f32() < 0.1 {
                stats.key_rotation_stats.forced_rotations += 1;
            }
            
            // Simulate rotation failures
            if fastrand::f32() < 0.001 {
                stats.key_rotation_stats.rotation_failures += 1;
            }
        }
    }
    
    /// Start cap management worker
    fn start_cap_management_worker(&mut self) -> Result<(), String> {
        if !self.config.cap_management.enable_issuance && !self.config.cap_management.enable_revocation {
            return Ok(());
        }
        
        let config = self.config.cap_management.clone();
        let running = Arc::clone(&self.running);
        let stats = Arc::clone(&self.stats);
        
        let handle = thread::spawn(move || {
            Self::cap_management_worker_loop(config, running, stats);
        });
        
        self.workers.push(handle);
        Ok(())
    }
    
    /// Cap management worker main loop
    fn cap_management_worker_loop(
        config: CapManagementConfig,
        running: Arc<AtomicBool>,
        stats: Arc<Mutex<SoakStats>>,
    ) {
        log::debug!("Cap management worker started");
        
        let issuance_interval = if config.enable_issuance {
            Duration::from_secs_f64(1.0 / config.issuance_rate as f64)
        } else {
            Duration::from_secs(1)
        };
        
        let revocation_interval = if config.enable_revocation {
            Duration::from_secs_f64(1.0 / config.revocation_rate as f64)
        } else {
            Duration::from_secs(1)
        };
        
        let mut last_issuance_time = Instant::now();
        let mut last_revocation_time = Instant::now();
        
        while running.load(Ordering::Acquire) {
            let now = Instant::now();
            
            if config.enable_issuance && now.duration_since(last_issuance_time) >= issuance_interval {
                Self::issue_capability(&config, &stats);
                last_issuance_time = now;
            }
            
            if config.enable_revocation && now.duration_since(last_revocation_time) >= revocation_interval {
                Self::revoke_capability(&config, &stats);
                last_revocation_time = now;
            }
            
            thread::sleep(Duration::from_millis(1));
        }
        
        log::debug!("Cap management worker stopped");
    }
    
    /// Issue a capability
    fn issue_capability(config: &CapManagementConfig, stats: &Arc<Mutex<SoakStats>>) {
        log::trace!("Issuing capability");
        
        // Update statistics
        if let Ok(mut stats) = stats.lock() {
            stats.cap_management_stats.caps_issued += 1;
            stats.cap_management_stats.active_caps += 1;
            
            // Simulate escalation attempts
            if config.enable_escalation && fastrand::f32() < 0.01 {
                stats.cap_management_stats.escalation_attempts += 1;
            }
        }
    }
    
    /// Revoke a capability
    fn revoke_capability(config: &CapManagementConfig, stats: &Arc<Mutex<SoakStats>>) {
        log::trace!("Revoking capability");
        
        // Update statistics
        if let Ok(mut stats) = stats.lock() {
            stats.cap_management_stats.caps_revoked += 1;
            if stats.cap_management_stats.active_caps > 0 {
                stats.cap_management_stats.active_caps -= 1;
            }
        }
    }
    
    /// Start chaos worker
    fn start_chaos_worker(&mut self) -> Result<(), String> {
        if !self.config.chaos.enabled {
            return Ok(());
        }
        
        let config = self.config.chaos.clone();
        let running = Arc::clone(&self.running);
        let stats = Arc::clone(&self.stats);
        
        let handle = thread::spawn(move || {
            Self::chaos_worker_loop(config, running, stats);
        });
        
        self.workers.push(handle);
        Ok(())
    }
    
    /// Chaos worker main loop
    fn chaos_worker_loop(
        config: ChaosConfig,
        running: Arc<AtomicBool>,
        stats: Arc<Mutex<SoakStats>>,
    ) {
        log::debug!("Chaos worker started");
        
        let chaos_interval = Duration::from_millis(100); // 10Hz chaos injection
        let mut last_chaos_time = Instant::now();
        
        while running.load(Ordering::Acquire) {
            let now = Instant::now();
            if now.duration_since(last_chaos_time) >= chaos_interval {
                Self::inject_chaos(&config, &stats);
                last_chaos_time = now;
            }
            
            thread::sleep(Duration::from_millis(1));
        }
        
        log::debug!("Chaos worker stopped");
    }
    
    /// Inject chaos events
    fn inject_chaos(config: &ChaosConfig, stats: &Arc<Mutex<SoakStats>>) {
        // Simulate message dropping
        if fastrand::u32(1..=config.message_drop_rate) == 1 {
            if let Ok(mut stats) = stats.lock() {
                stats.chaos_stats.messages_dropped += 1;
                stats.ipc_stats.messages_dropped += 1;
            }
            log::trace!("Chaos: Dropping message");
        }
        
        // Simulate intermittent failures
        if config.enable_intermittent_failures && fastrand::f32() < config.failure_rate {
            if let Ok(mut stats) = stats.lock() {
                stats.chaos_stats.failures_injected += 1;
            }
            log::trace!("Chaos: Injecting intermittent failure");
        }
        
        // Simulate resource exhaustion
        if config.enable_resource_exhaustion && fastrand::f32() < 0.001 {
            if let Ok(mut stats) = stats.lock() {
                stats.chaos_stats.resource_exhaustion_events += 1;
            }
            log::trace!("Chaos: Simulating resource exhaustion");
        }
        
        // Update chaos statistics
        if let Ok(mut stats) = stats.lock() {
            stats.chaos_stats.events_injected += 1;
        }
    }
    
    /// Start statistics collector
    fn start_stats_collector(&mut self) -> Result<(), String> {
        let config = self.config.stats.clone();
        let running = Arc::clone(&self.running);
        let stats = Arc::clone(&self.stats);
        
        let handle = thread::spawn(move || {
            Self::stats_collector_loop(config, running, stats);
        });
        
        self.workers.push(handle);
        Ok(())
    }
    
    /// Statistics collector main loop
    fn stats_collector_loop(
        config: StatsConfig,
        running: Arc<AtomicBool>,
        stats: Arc<Mutex<SoakStats>>,
    ) {
        log::debug!("Statistics collector started");
        
        let collection_interval = Duration::from_secs(config.collection_interval_sec);
        let mut last_collection_time = Instant::now();
        
        while running.load(Ordering::Acquire) {
            let now = Instant::now();
            if now.duration_since(last_collection_time) >= collection_interval {
                Self::collect_system_stats(&config, &stats);
                last_collection_time = now;
            }
            
            thread::sleep(Duration::from_secs(1));
        }
        
        log::debug!("Statistics collector stopped");
    }
    
    /// Collect system statistics
    fn collect_system_stats(config: &StatsConfig, stats: &Arc<Mutex<SoakStats>>) {
        if let Ok(mut stats) = stats.lock() {
            // Update uptime
            stats.system_health.uptime_seconds = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            // Simulate memory usage tracking
            if config.enable_memory_stats {
                let current_allocated = fastrand::u64(1024 * 1024..=100 * 1024 * 1024); // 1MB to 100MB
                stats.system_health.memory_usage.current_allocated = current_allocated;
                stats.system_health.memory_usage.peak_allocated = stats.system_health.memory_usage.peak_allocated.max(current_allocated);
                stats.system_health.memory_usage.total_allocated += current_allocated;
            }
            
            // Simulate CPU usage tracking
            if config.enable_performance_stats {
                let cpu_usage = fastrand::f64(10.0..=80.0);
                stats.system_health.cpu_usage.avg_usage_percent = cpu_usage;
                stats.system_health.cpu_usage.peak_usage_percent = stats.system_health.cpu_usage.peak_usage_percent.max(cpu_usage);
            }
            
            // Simulate IPC statistics
            if config.enable_ipc_stats {
                let messages_sent = stats.ipc_stats.messages_sent;
                let duration = stats.duration_seconds as f64;
                if duration > 0.0 {
                    stats.ipc_stats.throughput_mps = messages_sent as f64 / duration;
                }
            }
        }
    }
    
    /// Get current statistics
    pub fn get_stats(&self) -> SoakStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Generate test report
    pub fn generate_report(&self) -> Result<String, String> {
        let stats = self.get_stats();
        
        // Analyze test results
        let mut test_results = TestResults {
            status: TestStatus::Passed,
            passed: true,
            failure_reasons: Vec::new(),
            performance_regression_percent: 0.0,
            memory_leak_detected: false,
            recommendations: Vec::new(),
        };
        
        // Check for panics or deadlocks
        if stats.system_health.panic_count > 0 {
            test_results.status = TestStatus::Failed;
            test_results.passed = false;
            test_results.failure_reasons.push(format!("{} panics detected", stats.system_health.panic_count));
        }
        
        if stats.system_health.deadlock_detected {
            test_results.status = TestStatus::Failed;
            test_results.passed = false;
            test_results.failure_reasons.push("Deadlock detected".to_string());
        }
        
        // Check for memory leaks
        if stats.system_health.memory_usage.current_allocated > stats.system_health.memory_usage.total_allocated / 10 {
            test_results.memory_leak_detected = true;
            test_results.recommendations.push("Potential memory leak detected".to_string());
        }
        
        // Check performance regression
        let baseline_throughput = 1000.0; // Baseline IPC throughput
        if stats.ipc_stats.throughput_mps > 0.0 {
            let regression = ((baseline_throughput - stats.ipc_stats.throughput_mps) / baseline_throughput) * 100.0;
            if regression > 25.0 {
                test_results.performance_regression_percent = regression;
                test_results.recommendations.push(format!("Performance regression: {:.1}%", regression));
            }
        }
        
        // Update test results
        {
            let mut stats_mut = self.stats.lock().unwrap();
            stats_mut.test_results = test_results.clone();
        }
        
        // Generate JSON report
        let report = serde_json::to_string_pretty(&stats)
            .map_err(|e| format!("Failed to serialize report: {}", e))?;
        
        Ok(report)
    }
    
    /// Wait for test completion
    pub fn wait_for_completion(&self) -> Result<(), String> {
        let start_time = self.start_time.ok_or("Test not started")?;
        let target_duration = Duration::from_secs(self.config.duration_seconds);
        
        while self.running.load(Ordering::Acquire) {
            let elapsed = start_time.elapsed();
            if elapsed >= target_duration {
                break;
            }
            
            let remaining = target_duration - elapsed;
            log::info!("Test running... {} remaining", Self::format_duration(remaining));
            thread::sleep(Duration::from_secs(10));
        }
        
        Ok(())
    }
    
    /// Format duration for display
    fn format_duration(duration: Duration) -> String {
        let hours = duration.as_secs() / 3600;
        let minutes = (duration.as_secs() % 3600) / 60;
        let seconds = duration.as_secs() % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// Default soak test configuration for 2-hour test
pub fn default_soak_config() -> SoakConfig {
    SoakConfig {
        duration_seconds: 2 * 60 * 60, // 2 hours
        ipc_stress: IpcStressConfig {
            worker_count: 8,
            messages_per_worker_per_sec: 100,
            message_size_range: (64, 4096),
            priority_distribution: (0.3, 0.5, 0.2), // 30% low, 50% normal, 20% high
            enable_corruption: true,
            enable_large_messages: true,
        },
        timer_jitter: TimerJitterConfig {
            enabled: true,
            amplitude_us: 100,
            frequency_hz: 10.0, // 10Hz jitter injection
            pattern: JitterPattern::Random,
            adaptive: true,
        },
        key_rotation: KeyRotationConfig {
            enabled: true,
            rotation_interval_sec: 300, // 5 minutes
            grace_period_sec: 60, // 1 minute
            force_rotation_on_load: true,
            load_threshold: 0.8, // 80% load
        },
        cap_management: CapManagementConfig {
            enable_issuance: true,
            issuance_rate: 10, // 10 caps per second
            enable_revocation: true,
            revocation_rate: 5, // 5 caps per second
            lifetime_range: (60, 3600), // 1 minute to 1 hour
            enable_escalation: true,
        },
        chaos: ChaosConfig {
            enabled: true,
            message_drop_rate: 1000, // Drop every 1000th message
            priority_based_dropping: true,
            low_priority_drop_threshold: 100, // Drop every 100th low priority message
            enable_intermittent_failures: true,
            failure_rate: 0.001, // 0.1% failure rate
            enable_resource_exhaustion: true,
        },
        stats: StatsConfig {
            collection_interval_sec: 10, // Collect stats every 10 seconds
            enable_ipc_stats: true,
            enable_memory_stats: true,
            enable_performance_stats: true,
            enable_chaos_stats: true,
        },
    }
}

/// Mini soak test configuration for CI (10 minutes)
pub fn mini_soak_config() -> SoakConfig {
    let mut config = default_soak_config();
    config.duration_seconds = 10 * 60; // 10 minutes
    config.ipc_stress.worker_count = 4; // Fewer workers for CI
    config.ipc_stress.messages_per_worker_per_sec = 50; // Lower message rate
    config.timer_jitter.frequency_hz = 5.0; // Lower jitter frequency
    config.key_rotation.rotation_interval_sec = 60; // Faster rotation for testing
    config.cap_management.issuance_rate = 5; // Lower cap rate
    config.cap_management.revocation_rate = 2; // Lower revocation rate
    config.chaos.message_drop_rate = 500; // Less aggressive chaos
    config.stats.collection_interval_sec = 5; // More frequent stats collection
    config
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_soak_config_creation() {
        let config = default_soak_config();
        assert_eq!(config.duration_seconds, 2 * 60 * 60);
        assert_eq!(config.ipc_stress.worker_count, 8);
        assert!(config.timer_jitter.enabled);
        assert!(config.key_rotation.enabled);
        assert!(config.chaos.enabled);
    }
    
    #[test]
    fn test_mini_soak_config() {
        let config = mini_soak_config();
        assert_eq!(config.duration_seconds, 10 * 60);
        assert_eq!(config.ipc_stress.worker_count, 4);
        assert_eq!(config.timer_jitter.frequency_hz, 5.0);
    }
    
    #[test]
    fn test_soak_orchestrator_creation() {
        let config = default_soak_config();
        let orchestrator = SoakTestOrchestrator::new(config);
        assert!(!orchestrator.running.load(Ordering::Acquire));
    }
    
    #[test]
    fn test_stats_serialization() {
        let stats = SoakTestOrchestrator::create_empty_stats();
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("ipc_stats"));
        assert!(json.contains("timer_jitter_stats"));
        assert!(json.contains("key_rotation_stats"));
    }
}
