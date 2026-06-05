//! Polyglot Runtime Manager
//!
//! Central coordinator for all language runtimes, managing sandbox lifecycle,
//! configuration, and metrics collection.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::klog;
use super::backend::{LanguageBackend, LanguageType, AppManifest, RuntimeError, Value};
use super::bridge::PolyglotBridge;
use super::registry::BackendRegistry;
use super::sandbox::{ExecutionSandbox, SandboxId, SandboxState, ResourceLimits, ResourceUsage};

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Default resource limits for new sandboxes
    pub default_limits: ResourceLimits,
    /// Maximum number of concurrent sandboxes
    pub max_sandboxes: usize,
    /// Enable debug mode
    pub debug_enabled: bool,
    /// Enable metrics collection
    pub metrics_enabled: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            default_limits: ResourceLimits::default(),
            max_sandboxes: 256,
            debug_enabled: false,
            metrics_enabled: true,
        }
    }
}

/// Execution result from running an application
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Return value from execution
    pub value: Value,
    /// Final resource usage
    pub resource_usage: ResourceUsage,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Runtime statistics
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    /// Total sandboxes created
    pub sandboxes_created: u64,
    /// Currently active sandboxes
    pub active_sandboxes: usize,
    /// Total executions performed
    pub total_executions: u64,
    /// Total CPU time consumed (ms)
    pub total_cpu_time_ms: u64,
    /// Total memory allocated (bytes)
    pub total_memory_bytes: usize,
    /// Per-backend statistics
    pub backend_stats: Vec<BackendStats>,
}

/// Per-backend statistics
#[derive(Debug, Clone)]
pub struct BackendStats {
    /// Language type
    pub language: LanguageType,
    /// Number of active sandboxes
    pub active_sandboxes: usize,
    /// Total executions
    pub executions: u64,
    /// Total CPU time (ms)
    pub cpu_time_ms: u64,
    /// Total memory (bytes)
    pub memory_bytes: usize,
    /// Syscall count
    pub syscall_count: u64,
}

/// Runtime metrics for monitoring
#[derive(Debug, Clone, Default)]
pub struct RuntimeMetrics {
    /// Memory usage per backend
    pub memory_by_backend: BTreeMap<u8, usize>,
    /// CPU time per backend
    pub cpu_time_by_backend: BTreeMap<u8, u64>,
    /// Syscall count per backend
    pub syscalls_by_backend: BTreeMap<u8, u64>,
    /// Warning thresholds
    pub thresholds: MetricThresholds,
    /// Emitted warnings
    pub warnings: Vec<ThresholdWarning>,
    /// Last metrics update timestamp (ms)
    pub last_update_ms: u64,
}

/// Threshold warning event
#[derive(Debug, Clone)]
pub struct ThresholdWarning {
    /// Timestamp when warning was emitted (ms)
    pub timestamp_ms: u64,
    /// Type of resource that exceeded threshold
    pub resource_type: WarningResourceType,
    /// Language backend (if applicable)
    pub language: Option<LanguageType>,
    /// Sandbox ID (if applicable)
    pub sandbox_id: Option<SandboxId>,
    /// Current value
    pub current_value: u64,
    /// Threshold value
    pub threshold_value: u64,
    /// Warning message
    pub message: String,
}

/// Resource types for threshold warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningResourceType {
    /// Memory usage
    Memory,
    /// CPU time
    CpuTime,
    /// Syscall count
    SyscallCount,
}

/// Result of a metrics query
#[derive(Debug, Clone)]
pub struct MetricsQueryResult {
    /// Timestamp of the metrics collection (ms)
    pub timestamp_ms: u64,
    /// Per-backend metrics
    pub backend_metrics: Vec<BackendMetrics>,
    /// Total memory usage across all backends
    pub total_memory_bytes: usize,
    /// Total CPU time across all backends
    pub total_cpu_time_ms: u64,
    /// Total syscall count across all backends
    pub total_syscall_count: u64,
}

/// Metrics for a single backend
#[derive(Debug, Clone)]
pub struct BackendMetrics {
    /// Language type
    pub language: LanguageType,
    /// Memory usage in bytes
    pub memory_bytes: usize,
    /// CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// Syscall count
    pub syscall_count: u64,
}

/// Metric thresholds for warnings
#[derive(Debug, Clone)]
pub struct MetricThresholds {
    /// Memory warning threshold (bytes)
    pub memory_warning_bytes: usize,
    /// CPU time warning threshold (ms)
    pub cpu_time_warning_ms: u64,
}

impl Default for MetricThresholds {
    fn default() -> Self {
        Self {
            memory_warning_bytes: 512 * 1024 * 1024, // 512 MB
            cpu_time_warning_ms: 60_000,              // 60 seconds
        }
    }
}

/// Backend health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendHealth {
    /// Backend is healthy and operational
    Healthy,
    /// Backend has crashed and is isolated
    Crashed,
    /// Backend is recovering (reload in progress)
    Recovering,
}

/// Information about a backend crash
#[derive(Debug, Clone)]
pub struct BackendCrashInfo {
    /// Language type of crashed backend
    pub language: LanguageType,
    /// Reason for the crash
    pub reason: String,
    /// Timestamp of the crash (ms)
    pub timestamp_ms: u64,
    /// Number of sandboxes affected
    pub affected_sandboxes: usize,
    /// IDs of terminated sandboxes
    pub terminated_sandbox_ids: Vec<SandboxId>,
}

/// Polyglot Runtime Manager - coordinates language backends
pub struct PolyglotRuntimeManager {
    /// Backend registry
    registry: BackendRegistry,
    /// Active execution sandboxes
    sandboxes: BTreeMap<u64, ExecutionSandbox>,
    /// Runtime configuration
    config: RuntimeConfig,
    /// Runtime bridge for syscalls and IPC
    bridge: PolyglotBridge,
    /// Runtime metrics
    metrics: RuntimeMetrics,
    /// Statistics counters
    stats: RuntimeStats,
    /// Revoked capability tokens
    revoked_tokens: Vec<u128>,
    /// Backend health status
    backend_health: BTreeMap<u8, BackendHealth>,
    /// Crashed backends (isolated)
    crashed_backends: BTreeSet<u8>,
    /// Crash history for diagnostics
    crash_history: Vec<BackendCrashInfo>,
}

impl PolyglotRuntimeManager {
    /// Create a new runtime manager
    pub fn new() -> Self {
        Self {
            registry: BackendRegistry::new(),
            sandboxes: BTreeMap::new(),
            config: RuntimeConfig {
                default_limits: ResourceLimits {
                    max_memory_bytes: 64 * 1024 * 1024,
                    max_cpu_time_ms: 30_000,
                    max_file_descriptors: 64,
                    max_network_connections: 16,
                },
                max_sandboxes: 256,
                debug_enabled: false,
                metrics_enabled: true,
            },
            bridge: PolyglotBridge::new(),
            metrics: RuntimeMetrics {
                memory_by_backend: BTreeMap::new(),
                cpu_time_by_backend: BTreeMap::new(),
                syscalls_by_backend: BTreeMap::new(),
                thresholds: MetricThresholds {
                    memory_warning_bytes: 512 * 1024 * 1024,
                    cpu_time_warning_ms: 60_000,
                },
                warnings: Vec::new(),
                last_update_ms: 0,
            },
            stats: RuntimeStats {
                sandboxes_created: 0,
                active_sandboxes: 0,
                total_executions: 0,
                total_cpu_time_ms: 0,
                total_memory_bytes: 0,
                backend_stats: Vec::new(),
            },
            revoked_tokens: Vec::new(),
            backend_health: BTreeMap::new(),
            crashed_backends: BTreeSet::new(),
            crash_history: Vec::new(),
        }
    }

    /// Register a new language backend
    pub fn register_backend(&mut self, backend: Arc<dyn LanguageBackend>) -> Result<(), RuntimeError> {
        self.registry.register(backend).map_err(|e| {
            match e {
                super::registry::RegistryError::ValidationFailed { language, missing_methods } => {
                    RuntimeError::BackendValidationFailed { language, missing_methods }
                }
                super::registry::RegistryError::AlreadyRegistered(lang) => {
                    RuntimeError::SandboxCreationFailed(crate::kformat!("Backend already registered: {}", lang))
                }
                _ => RuntimeError::SandboxCreationFailed(crate::kformat!("{}", e))
            }
        })
    }

    /// Create a new execution sandbox for an application
    pub fn create_sandbox(&mut self, manifest: &AppManifest) -> Result<SandboxId, RuntimeError> {
        // Check if we have a backend for this language
        if !self.registry.is_registered(manifest.language) {
            return Err(RuntimeError::BackendNotFound(manifest.language));
        }

        // Check if the backend is healthy (not crashed)
        if !self.is_backend_healthy(manifest.language) {
            return Err(RuntimeError::BackendCrashed {
                language: manifest.language,
                reason: "Backend is crashed and isolated".into(),
            });
        }

        // Check sandbox limit
        if self.sandboxes.len() >= self.config.max_sandboxes {
            return Err(RuntimeError::SandboxCreationFailed(
                "Maximum sandbox limit reached".into()
            ));
        }

        // Determine resource limits
        let limits = ResourceLimits {
            max_memory_bytes: manifest.resources.max_memory_bytes
                .max(self.config.default_limits.max_memory_bytes),
            ..self.config.default_limits.clone()
        };

        // Create the sandbox
        let sandbox = ExecutionSandbox::new(manifest.language, limits);
        let sandbox_id = sandbox.id;

        self.sandboxes.insert(sandbox_id.0, sandbox);
        self.stats.sandboxes_created += 1;
        self.stats.active_sandboxes = self.sandboxes.len();

        klog!(INFO, "[POLYGLOT] Created sandbox {:?} for {} application '{}'",
              sandbox_id, manifest.language, manifest.name);

        Ok(sandbox_id)
    }

    /// Execute an application in its sandbox
    pub fn execute(
        &mut self,
        sandbox_id: SandboxId,
        entry_point: &str,
        args: &[Value],
    ) -> Result<ExecutionResult, RuntimeError> {
        let start_time = crate::security::get_current_time_ms();

        // Get the sandbox
        let sandbox = self.sandboxes.get_mut(&sandbox_id.0)
            .ok_or_else(|| RuntimeError::SandboxCreationFailed("Sandbox not found".into()))?;

        // Start the sandbox if not already running
        if sandbox.state == SandboxState::Created {
            sandbox.start()?;
        }

        // Get the backend
        let backend = self.registry.get(sandbox.language)
            .ok_or_else(|| RuntimeError::BackendNotFound(sandbox.language))?;

        // Execute (placeholder - actual execution would use the loaded module)
        let result_value = if let Some(ref module) = sandbox.module {
            backend.execute(module, entry_point, args)?
        } else {
            Value::Null
        };

        let end_time = crate::security::get_current_time_ms();
        let execution_time = end_time - start_time;

        // Update statistics
        sandbox.record_cpu_time(execution_time);
        let resource_usage = sandbox.usage.clone();
        
        self.stats.total_executions += 1;
        self.stats.total_cpu_time_ms += execution_time;

        // Check thresholds and emit warnings
        self.check_thresholds(sandbox_id);

        Ok(ExecutionResult {
            value: result_value,
            resource_usage,
            execution_time_ms: execution_time,
        })
    }

    /// Terminate a sandbox and cleanup resources
    pub fn terminate_sandbox(&mut self, sandbox_id: SandboxId) -> Result<ResourceUsage, RuntimeError> {
        let sandbox = self.sandboxes.remove(&sandbox_id.0)
            .ok_or_else(|| RuntimeError::SandboxCreationFailed("Sandbox not found".into()))?;

        // Record final statistics before cleanup
        let final_usage = sandbox.usage.clone();
        
        self.stats.active_sandboxes = self.sandboxes.len();
        self.stats.total_memory_bytes = self.stats.total_memory_bytes
            .saturating_sub(final_usage.memory_bytes);

        klog!(INFO, "[POLYGLOT] Terminated sandbox {:?}, final usage: {:?}", sandbox_id, final_usage);

        Ok(final_usage)
    }

    /// Get runtime statistics
    pub fn get_stats(&self) -> RuntimeStats {
        let mut stats = self.stats.clone();
        
        // Collect per-backend stats
        stats.backend_stats = self.registry.list().iter().map(|(lang, _info)| {
            let active = self.sandboxes.values()
                .filter(|s| s.language == *lang)
                .count();
            let (cpu, mem, syscalls) = self.sandboxes.values()
                .filter(|s| s.language == *lang)
                .fold((0u64, 0usize, 0u64), |(c, m, s), sandbox| {
                    (c + sandbox.usage.cpu_time_ms,
                     m + sandbox.usage.memory_bytes,
                     s + sandbox.usage.syscall_count)
                });
            
            BackendStats {
                language: *lang,
                active_sandboxes: active,
                executions: 0, // Would need per-backend tracking
                cpu_time_ms: cpu,
                memory_bytes: mem,
                syscall_count: syscalls,
            }
        }).collect();

        stats
    }

    /// Apply configuration to the runtime
    pub fn apply_config(&mut self, config: RuntimeConfig) {
        // Configuration changes only affect new sandboxes
        // Existing sandboxes retain their original configuration
        self.config = config;
        klog!(INFO, "[POLYGLOT] Applied new runtime configuration");
    }

    /// Get current configuration
    pub fn get_config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Revoke a capability token across all sandboxes
    pub fn revoke_capability(&mut self, token_id: u128) {
        // Add to revoked list
        self.revoked_tokens.push(token_id);

        // Propagate to all sandboxes
        for sandbox in self.sandboxes.values_mut() {
            sandbox.revoke_capability(token_id);
        }

        // Propagate to the bridge
        self.bridge.revoke_token(token_id);

        klog!(WARN, "[POLYGLOT] Revoked capability token 0x{:x} across all sandboxes", token_id);
    }

    /// Check if a token has been revoked
    pub fn is_token_revoked(&self, token_id: u128) -> bool {
        self.revoked_tokens.contains(&token_id)
    }

    /// Get the runtime bridge
    pub fn bridge(&mut self) -> &mut PolyglotBridge {
        &mut self.bridge
    }

    /// Get a sandbox by ID
    pub fn get_sandbox(&self, sandbox_id: SandboxId) -> Option<&ExecutionSandbox> {
        self.sandboxes.get(&sandbox_id.0)
    }

    /// Get a mutable sandbox by ID
    pub fn get_sandbox_mut(&mut self, sandbox_id: SandboxId) -> Option<&mut ExecutionSandbox> {
        self.sandboxes.get_mut(&sandbox_id.0)
    }

    /// List all active sandbox IDs
    pub fn list_sandboxes(&self) -> Vec<SandboxId> {
        self.sandboxes.keys().map(|k| SandboxId(*k)).collect()
    }

    /// Check thresholds and emit warnings
    fn check_thresholds(&mut self, sandbox_id: SandboxId) {
        let now = crate::security::get_current_time_ms();
        
        let (language, memory_bytes, cpu_time_ms) = if let Some(sandbox) = self.sandboxes.get(&sandbox_id.0) {
            (sandbox.language, sandbox.usage.memory_bytes, sandbox.usage.cpu_time_ms)
        } else {
            return;
        };

        // Check memory threshold
        if memory_bytes > self.metrics.thresholds.memory_warning_bytes {
            let warning = ThresholdWarning {
                timestamp_ms: now,
                resource_type: WarningResourceType::Memory,
                language: Some(language),
                sandbox_id: Some(sandbox_id),
                current_value: memory_bytes as u64,
                threshold_value: self.metrics.thresholds.memory_warning_bytes as u64,
                message: crate::kformat!(
                    "Sandbox {:?} memory usage {} exceeds threshold {}",
                    sandbox_id, memory_bytes, self.metrics.thresholds.memory_warning_bytes
                ),
            };
            self.metrics.warnings.push(warning);
            klog!(WARN, "[POLYGLOT] Sandbox {:?} memory usage {} exceeds threshold {}",
                  sandbox_id, memory_bytes, self.metrics.thresholds.memory_warning_bytes);
        }
        
        // Check CPU time threshold
        if cpu_time_ms > self.metrics.thresholds.cpu_time_warning_ms {
            let warning = ThresholdWarning {
                timestamp_ms: now,
                resource_type: WarningResourceType::CpuTime,
                language: Some(language),
                sandbox_id: Some(sandbox_id),
                current_value: cpu_time_ms,
                threshold_value: self.metrics.thresholds.cpu_time_warning_ms,
                message: crate::kformat!(
                    "Sandbox {:?} CPU time {} ms exceeds threshold {} ms",
                    sandbox_id, cpu_time_ms, self.metrics.thresholds.cpu_time_warning_ms
                ),
            };
            self.metrics.warnings.push(warning);
            klog!(WARN, "[POLYGLOT] Sandbox {:?} CPU time {} ms exceeds threshold {} ms",
                  sandbox_id, cpu_time_ms, self.metrics.thresholds.cpu_time_warning_ms);
        }
    }

    /// Set metric thresholds
    pub fn set_thresholds(&mut self, thresholds: MetricThresholds) {
        self.metrics.thresholds = thresholds;
    }

    /// Get metrics for monitoring
    pub fn get_metrics(&self) -> &RuntimeMetrics {
        &self.metrics
    }

    // ========================================================================
    // Metrics and Monitoring (Task 12)
    // ========================================================================

    /// Collect and update metrics from all sandboxes
    pub fn collect_metrics(&mut self) {
        let now = crate::security::get_current_time_ms();
        
        // Clear previous per-backend metrics
        self.metrics.memory_by_backend.clear();
        self.metrics.cpu_time_by_backend.clear();
        self.metrics.syscalls_by_backend.clear();
        
        // Aggregate metrics from all sandboxes
        for sandbox in self.sandboxes.values() {
            let lang_id = sandbox.language as u8;
            
            // Memory
            *self.metrics.memory_by_backend.entry(lang_id).or_insert(0) += sandbox.usage.memory_bytes;
            
            // CPU time
            *self.metrics.cpu_time_by_backend.entry(lang_id).or_insert(0) += sandbox.usage.cpu_time_ms;
            
            // Syscalls
            *self.metrics.syscalls_by_backend.entry(lang_id).or_insert(0) += sandbox.usage.syscall_count;
        }
        
        self.metrics.last_update_ms = now;
    }

    /// Query runtime statistics with per-backend breakdown
    /// Returns memory_bytes, cpu_time_ms, and syscall_count for each active backend
    pub fn query_metrics(&mut self) -> MetricsQueryResult {
        // Ensure metrics are up to date
        self.collect_metrics();
        
        let mut backend_metrics = Vec::new();
        
        // Get metrics for each registered backend
        for (lang, _info) in self.registry.list() {
            let lang_id = lang as u8;
            
            let memory_bytes = self.metrics.memory_by_backend.get(&lang_id).copied().unwrap_or(0);
            let cpu_time_ms = self.metrics.cpu_time_by_backend.get(&lang_id).copied().unwrap_or(0);
            let syscall_count = self.metrics.syscalls_by_backend.get(&lang_id).copied().unwrap_or(0);
            
            backend_metrics.push(BackendMetrics {
                language: lang,
                memory_bytes,
                cpu_time_ms,
                syscall_count,
            });
        }
        
        MetricsQueryResult {
            timestamp_ms: self.metrics.last_update_ms,
            backend_metrics,
            total_memory_bytes: self.metrics.memory_by_backend.values().sum(),
            total_cpu_time_ms: self.metrics.cpu_time_by_backend.values().sum(),
            total_syscall_count: self.metrics.syscalls_by_backend.values().sum(),
        }
    }

    /// Get all emitted threshold warnings
    pub fn get_warnings(&self) -> &[ThresholdWarning] {
        &self.metrics.warnings
    }

    /// Clear all warnings
    pub fn clear_warnings(&mut self) {
        self.metrics.warnings.clear();
    }

    /// Manually emit a threshold warning (for testing or external triggers)
    pub fn emit_warning(&mut self, warning: ThresholdWarning) {
        self.metrics.warnings.push(warning);
    }

    // ========================================================================
    // Fault Isolation & Recovery (Task 9)
    // ========================================================================

    /// Report a backend crash and isolate it from other backends
    /// 
    /// This method:
    /// 1. Marks the backend as crashed
    /// 2. Terminates all sandboxes using that backend
    /// 3. Records crash information for diagnostics
    /// 4. Ensures other backends remain operational
    pub fn report_backend_crash(&mut self, language: LanguageType, reason: String) -> BackendCrashInfo {
        let lang_id = language as u8;
        let timestamp = crate::security::get_current_time_ms();

        // Mark backend as crashed
        self.backend_health.insert(lang_id, BackendHealth::Crashed);
        self.crashed_backends.insert(lang_id);

        // Find and terminate all sandboxes using this backend
        let affected_sandbox_ids: Vec<SandboxId> = self.sandboxes
            .iter()
            .filter(|(_, sandbox)| sandbox.language == language)
            .map(|(id, _)| SandboxId(*id))
            .collect();

        let affected_count = affected_sandbox_ids.len();

        // Terminate affected sandboxes
        for sandbox_id in &affected_sandbox_ids {
            let _ = self.sandboxes.remove(&sandbox_id.0);
        }

        self.stats.active_sandboxes = self.sandboxes.len();

        // Record crash info
        let crash_info = BackendCrashInfo {
            language,
            reason: reason.clone(),
            timestamp_ms: timestamp,
            affected_sandboxes: affected_count,
            terminated_sandbox_ids: affected_sandbox_ids,
        };

        self.crash_history.push(crash_info.clone());

        klog!(ERROR, "[POLYGLOT] Backend {} crashed: {}. Terminated {} sandboxes.",
              language, reason, affected_count);

        crash_info
    }

    /// Attempt to recover a crashed backend
    /// 
    /// This method:
    /// 1. Marks the backend as recovering
    /// 2. Attempts to reload the backend
    /// 3. If successful, marks it as healthy
    /// 4. Returns success/failure status
    pub fn recover_backend(&mut self, language: LanguageType) -> Result<(), RuntimeError> {
        let lang_id = language as u8;

        // Check if backend is actually crashed
        if !self.crashed_backends.contains(&lang_id) {
            return Ok(()); // Nothing to recover
        }

        // Mark as recovering
        self.backend_health.insert(lang_id, BackendHealth::Recovering);

        klog!(INFO, "[POLYGLOT] Attempting to recover backend {}", language);

        // Check if backend is still registered and can be validated
        if let Some(backend) = self.registry.get(language) {
            match backend.validate() {
                Ok(()) => {
                    // Backend recovered successfully
                    self.backend_health.insert(lang_id, BackendHealth::Healthy);
                    self.crashed_backends.remove(&lang_id);
                    
                    klog!(INFO, "[POLYGLOT] Backend {} recovered successfully", language);
                    Ok(())
                }
                Err(e) => {
                    // Recovery failed, keep it crashed
                    self.backend_health.insert(lang_id, BackendHealth::Crashed);
                    
                    klog!(ERROR, "[POLYGLOT] Backend {} recovery failed: {:?}", language, e);
                    Err(RuntimeError::BackendCrashed {
                        language,
                        reason: crate::kformat!("Recovery failed: {:?}", e),
                    })
                }
            }
        } else {
            // Backend not registered, cannot recover
            self.backend_health.insert(lang_id, BackendHealth::Crashed);
            
            Err(RuntimeError::BackendNotFound(language))
        }
    }

    /// Check if a backend is healthy (not crashed)
    pub fn is_backend_healthy(&self, language: LanguageType) -> bool {
        let lang_id = language as u8;
        !self.crashed_backends.contains(&lang_id)
    }

    /// Get the health status of a backend
    pub fn get_backend_health(&self, language: LanguageType) -> BackendHealth {
        let lang_id = language as u8;
        self.backend_health.get(&lang_id).copied().unwrap_or(BackendHealth::Healthy)
    }

    /// Get all crashed backends
    pub fn get_crashed_backends(&self) -> Vec<LanguageType> {
        self.crashed_backends
            .iter()
            .filter_map(|&id| match id {
                0 => Some(LanguageType::Wasm),
                1 => Some(LanguageType::Python),
                2 => Some(LanguageType::JavaScript),
                3 => Some(LanguageType::Rust),
                4 => Some(LanguageType::Go),
                5 => Some(LanguageType::Cpp),
                6 => Some(LanguageType::CSharp),
                _ => None,
            })
            .collect()
    }

    /// Get crash history
    pub fn get_crash_history(&self) -> &[BackendCrashInfo] {
        &self.crash_history
    }

    /// Clear crash history (for testing or after review)
    pub fn clear_crash_history(&mut self) {
        self.crash_history.clear();
    }

    /// Simulate a backend crash (for testing fault isolation)
    /// 
    /// This is useful for testing that fault isolation works correctly.
    #[cfg(test)]
    pub fn simulate_crash(&mut self, language: LanguageType, reason: &str) -> BackendCrashInfo {
        self.report_backend_crash(language, reason.into())
    }

    /// Check if any sandboxes are still running for a given language
    pub fn has_active_sandboxes_for(&self, language: LanguageType) -> bool {
        self.sandboxes.values().any(|s| s.language == language)
    }

    /// Get count of active sandboxes for a given language
    pub fn active_sandbox_count_for(&self, language: LanguageType) -> usize {
        self.sandboxes.values().filter(|s| s.language == language).count()
    }
}

impl Default for PolyglotRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Syscall Integration Methods (Task 13)
// ============================================================================

impl PolyglotRuntimeManager {
    /// Check if a sandbox exists
    pub fn has_sandbox(&self, sandbox_id: SandboxId) -> bool {
        self.sandboxes.contains_key(&sandbox_id.0)
    }

    /// Get the state of a sandbox
    pub fn get_sandbox_state(&self, sandbox_id: SandboxId) -> Option<SandboxState> {
        self.sandboxes.get(&sandbox_id.0).map(|s| s.state)
    }

    /// Create a sandbox with specific resource limits (for syscall handler)
    pub fn create_sandbox_with_limits(
        &mut self,
        language: LanguageType,
        limits: ResourceLimits,
    ) -> Result<SandboxId, RuntimeError> {
        // Check if we have a backend for this language
        if !self.registry.is_registered(language) {
            return Err(RuntimeError::BackendNotFound(language));
        }

        // Check if the backend is healthy (not crashed)
        if !self.is_backend_healthy(language) {
            return Err(RuntimeError::BackendCrashed {
                language: language,
                reason: "Backend is crashed and isolated".into(),
            });
        }

        // Check sandbox limit
        if self.sandboxes.len() >= self.config.max_sandboxes {
            return Err(RuntimeError::SandboxCreationFailed(
                "Maximum sandbox limit reached".into()
            ));
        }

        // Create the sandbox
        let sandbox = ExecutionSandbox::new(language, limits);
        let sandbox_id = sandbox.id;

        self.sandboxes.insert(sandbox_id.0, sandbox);
        self.stats.sandboxes_created += 1;
        self.stats.active_sandboxes = self.sandboxes.len();

        klog!(INFO, "[POLYGLOT] Created sandbox {:?} for {} with custom limits",
              sandbox_id, language);

        Ok(sandbox_id)
    }

    /// Get resource usage for a sandbox
    pub fn get_sandbox_usage(&self, sandbox_id: SandboxId) -> Option<ResourceUsage> {
        self.sandboxes.get(&sandbox_id.0).map(|s| s.usage.clone())
    }

    /// Get the number of active sandboxes
    pub fn active_sandbox_count(&self) -> usize {
        self.sandboxes.len()
    }
}
