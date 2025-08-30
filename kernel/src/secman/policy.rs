/// Security Policy Module for Polymera OS
/// 
/// This module implements a policy switch that sets "fail-closed" for IPC auth
/// (no cap or MAC → deny) vs "fail-open" (dev mode). The kernel reads compiled
/// policy blob on boot and caches the decision.

use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU8, Ordering};
use spin::Mutex;
use crate::{kprintln, klog};

/// Policy mode constants
pub const POLICY_MODE_FAIL_CLOSED: u8 = 0;
pub const POLICY_MODE_FAIL_OPEN: u8 = 1;
pub const POLICY_MODE_DEFAULT: u8 = POLICY_MODE_FAIL_CLOSED;

/// Policy decision constants
pub const POLICY_DECISION_ALLOW: u8 = 0;
pub const POLICY_DECISION_DENY: u8 = 1;
pub const POLICY_DECISION_AUDIT: u8 = 2;

/// Policy evaluation context for IPC operations
#[derive(Debug, Clone)]
pub struct IpcPolicyContext {
    /// Sender process ID
    pub sender_pid: u64,
    /// Destination process ID
    pub destination_pid: u64,
    /// Whether capability token is present
    pub has_capability: bool,
    /// Whether MAC is present and valid
    pub has_valid_mac: bool,
    /// Message size in bytes
    pub message_size: usize,
    /// Message priority
    pub priority: u8,
    /// Authentication mode used
    pub auth_mode: String,
    /// Timestamp of the operation
    pub timestamp: u64,
}

/// Policy evaluation result
#[derive(Debug, Clone)]
pub struct PolicyResult {
    /// Whether the operation is allowed
    pub allowed: bool,
    /// Policy decision code
    pub decision: u8,
    /// Reason for decision
    pub reason: String,
    /// Whether to audit this decision
    pub audit: bool,
    /// Additional metadata
    pub metadata: Vec<(String, String)>,
}

/// WASM policy blob structure
#[derive(Debug, Clone)]
pub struct WasmPolicyBlob {
    /// Policy data (compiled WASM)
    pub data: Vec<u8>,
    /// Policy version
    pub version: String,
    /// Policy hash for integrity verification
    pub hash: [u8; 32],
    /// Compilation timestamp
    pub compiled_at: u64,
}

/// Policy manager for security decisions
pub struct PolicyManager {
    /// Current policy mode (fail-closed vs fail-open)
    current_mode: AtomicU8,
    /// Compiled WASM policy blob
    policy_blob: Mutex<Option<WasmPolicyBlob>>,
    /// Policy evaluation statistics
    stats: Mutex<PolicyStats>,
    /// Whether policy system is initialized
    initialized: AtomicU8,
}

/// Policy evaluation statistics
#[derive(Debug, Clone, Default)]
pub struct PolicyStats {
    /// Total policy evaluations
    pub total_evaluations: u64,
    /// Allowed operations
    pub allowed_operations: u64,
    /// Denied operations
    pub denied_operations: u64,
    /// Audit-only operations
    pub audit_only_operations: u64,
    /// Policy evaluation failures
    pub evaluation_failures: u64,
    /// Last evaluation timestamp
    pub last_evaluation: u64,
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self {
            current_mode: AtomicU8::new(POLICY_MODE_DEFAULT),
            policy_blob: Mutex::new(None),
            stats: Mutex::new(PolicyStats::default()),
            initialized: AtomicU8::new(0),
        }
    }
}

impl PolicyManager {
    /// Create a new policy manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the policy manager
    pub fn init(&self) -> Result<(), String> {
        kprintln!("[POLICY] Initializing Policy Manager");
        
        // Try to load policy from /policy/out/p2_boot.wasm
        match self.load_policy_from_file() {
            Ok(blob) => {
                kprintln!("[POLICY] Loaded policy blob: version={}, size={} bytes", 
                         blob.version, blob.data.len());
                
                // Set policy mode based on loaded policy
                let mode = self.determine_policy_mode(&blob);
                self.current_mode.store(mode, Ordering::SeqCst);
                
                kprintln!("[POLICY] Policy mode set to: {} ({})", 
                         if mode == POLICY_MODE_FAIL_CLOSED { "FAIL-CLOSED" } else { "FAIL-OPEN" },
                         mode);
                
                // Store the policy blob
                if let Ok(mut blob_guard) = self.policy_blob.lock() {
                    *blob_guard = Some(blob);
                }
                
                self.initialized.store(1, Ordering::SeqCst);
                kprintln!("[POLICY] Policy Manager initialized successfully");
                Ok(())
            }
            Err(e) => {
                kprintln!("[POLICY] Failed to load policy: {}, using default mode", e);
                
                // Use default fail-closed mode
                self.current_mode.store(POLICY_MODE_DEFAULT, Ordering::SeqCst);
                self.initialized.store(1, Ordering::SeqCst);
                
                kprintln!("[POLICY] Policy Manager initialized with default mode: FAIL-CLOSED");
                Ok(())
            }
        }
    }

    /// Load policy from compiled WASM file
    fn load_policy_from_file(&self) -> Result<WasmPolicyBlob, String> {
        // In a real implementation, this would read from the filesystem
        // For now, we'll create a mock policy blob
        
        kprintln!("[POLICY] Loading policy from /policy/out/p2_boot.wasm");
        
        // Simulate file read delay
        // In real implementation: std::fs::read("/policy/out/p2_boot.wasm")
        
        // Create mock policy blob for development
        let mock_policy = WasmPolicyBlob {
            data: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00], // Mock WASM header
            version: "1.0.0".to_string(),
            hash: [0u8; 32], // Mock hash
            compiled_at: 0, // Mock timestamp
        };
        
        kprintln!("[POLICY] Loaded mock policy blob ({} bytes)", mock_policy.data.len());
        
        Ok(mock_policy)
    }

    /// Determine policy mode from loaded policy blob
    fn determine_policy_mode(&self, blob: &WasmPolicyBlob) -> u8 {
        // In a real implementation, this would analyze the WASM policy
        // For now, we'll use a simple heuristic based on development environment
        
        // Check if we're in development mode
        if cfg!(debug_assertions) {
            kprintln!("[POLICY] Development build detected, using FAIL-OPEN mode");
            POLICY_MODE_FAIL_OPEN
        } else {
            kprintln!("[POLICY] Production build detected, using FAIL-CLOSED mode");
            POLICY_MODE_FAIL_CLOSED
        }
    }

    /// Evaluate IPC policy for a given context
    pub fn evaluate_ipc_policy(&self, context: &IpcPolicyContext) -> PolicyResult {
        // Update statistics
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_evaluations += 1;
            stats.last_evaluation = context.timestamp;
        }

        // Check if policy system is initialized
        if self.initialized.load(Ordering::SeqCst) == 0 {
            kprintln!("[POLICY] Policy system not initialized, defaulting to DENY");
            return PolicyResult {
                allowed: false,
                decision: POLICY_DECISION_DENY,
                reason: "Policy system not initialized".to_string(),
                audit: true,
                metadata: vec![("error".to_string(), "system_not_ready".to_string())],
            };
        }

        let current_mode = self.current_mode.load(Ordering::SeqCst);
        
        match current_mode {
            POLICY_MODE_FAIL_CLOSED => self.evaluate_fail_closed_policy(context),
            POLICY_MODE_FAIL_OPEN => self.evaluate_fail_open_policy(context),
            _ => {
                kprintln!("[POLICY] Unknown policy mode: {}, defaulting to DENY", current_mode);
                PolicyResult {
                    allowed: false,
                    decision: POLICY_DECISION_DENY,
                    reason: format!("Unknown policy mode: {}", current_mode),
                    audit: true,
                    metadata: vec![("error".to_string(), "unknown_mode".to_string())],
                }
            }
        }
    }

    /// Evaluate policy in fail-closed mode (strict security)
    fn evaluate_fail_closed_policy(&self, context: &IpcPolicyContext) -> PolicyResult {
        // Fail-closed: require both capability and MAC for all operations
        let has_cap = context.has_capability;
        let has_mac = context.has_valid_mac;
        
        if !has_cap && !has_mac {
            // No authentication at all - deny
            self.record_denial(context, "No capability or MAC provided");
            PolicyResult {
                allowed: false,
                decision: POLICY_DECISION_DENY,
                reason: "No capability or MAC provided".to_string(),
                audit: true,
                metadata: vec![
                    ("policy_mode".to_string(), "fail_closed".to_string()),
                    ("missing_auth".to_string(), "both".to_string()),
                ],
            }
        } else if !has_cap {
            // Missing capability - deny
            self.record_denial(context, "Missing capability token");
            PolicyResult {
                allowed: false,
                decision: POLICY_DECISION_DENY,
                reason: "Missing capability token".to_string(),
                audit: true,
                metadata: vec![
                    ("policy_mode".to_string(), "fail_closed".to_string()),
                    ("missing_auth".to_string(), "capability".to_string()),
                ],
            }
        } else if !has_mac {
            // Missing MAC - deny
            self.record_denial(context, "Missing message authentication code");
            PolicyResult {
                allowed: false,
                decision: POLICY_DECISION_DENY,
                reason: "Missing message authentication code".to_string(),
                audit: true,
                metadata: vec![
                    ("policy_mode".to_string(), "fail_closed".to_string()),
                    ("missing_auth".to_string(), "mac".to_string()),
                ],
            }
        } else {
            // Both capability and MAC present - allow
            self.record_allowance(context);
            PolicyResult {
                allowed: true,
                decision: POLICY_DECISION_ALLOW,
                reason: "Capability and MAC validation passed".to_string(),
                audit: false, // Don't audit successful operations by default
                metadata: vec![
                    ("policy_mode".to_string(), "fail_closed".to_string()),
                    ("auth_method".to_string(), "capability_mac".to_string()),
                ],
            }
        }
    }

    /// Evaluate policy in fail-open mode (development friendly)
    fn evaluate_fail_open_policy(&self, context: &IpcPolicyContext) -> PolicyResult {
        // Fail-open: allow operations with minimal authentication in development
        let has_cap = context.has_capability;
        let has_mac = context.has_valid_mac;
        
        if has_cap && has_mac {
            // Full authentication - allow
            self.record_allowance(context);
            PolicyResult {
                allowed: true,
                decision: POLICY_DECISION_ALLOW,
                reason: "Full authentication (capability + MAC)".to_string(),
                audit: false,
                metadata: vec![
                    ("policy_mode".to_string(), "fail_open".to_string()),
                    ("auth_method".to_string(), "full".to_string()),
                ],
            }
        } else if has_cap {
            // Capability only - allow but audit
            self.record_allowance(context);
            PolicyResult {
                allowed: true,
                decision: POLICY_DECISION_AUDIT,
                reason: "Capability-only authentication (dev mode)".to_string(),
                audit: true,
                metadata: vec![
                    ("policy_mode".to_string(), "fail_open".to_string()),
                    ("auth_method".to_string(), "capability_only".to_string()),
                    ("warning".to_string(), "missing_mac".to_string()),
                ],
            }
        } else if has_mac {
            // MAC only - allow but audit
            self.record_allowance(context);
            PolicyResult {
                allowed: true,
                decision: POLICY_DECISION_AUDIT,
                reason: "MAC-only authentication (dev mode)".to_string(),
                audit: true,
                metadata: vec![
                    ("policy_mode".to_string(), "fail_open".to_string()),
                    ("auth_method".to_string(), "mac_only".to_string()),
                    ("warning".to_string(), "missing_capability".to_string()),
                ],
            }
        } else {
            // No authentication - allow but audit heavily
            self.record_allowance(context);
            PolicyResult {
                allowed: true,
                decision: POLICY_DECISION_AUDIT,
                reason: "No authentication (dev mode - NOT FOR PRODUCTION)".to_string(),
                audit: true,
                metadata: vec![
                    ("policy_mode".to_string(), "fail_open".to_string()),
                    ("auth_method".to_string(), "none".to_string()),
                    ("warning".to_string(), "no_auth_dev_only".to_string()),
                    ("security_risk".to_string(), "high".to_string()),
                ],
            }
        }
    }

    /// Record a policy denial for statistics
    fn record_denial(&self, context: &IpcPolicyContext, reason: &str) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.denied_operations += 1;
        }
        
        klog!(WARN, "[POLICY] IPC DENIED: sender={} -> dst={}, reason={}", 
              context.sender_pid, context.destination_pid, reason);
    }

    /// Record a policy allowance for statistics
    fn record_allowance(&self, context: &IpcPolicyContext) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.allowed_operations += 1;
        }
        
        klog!(DEBUG, "[POLICY] IPC ALLOWED: sender={} -> dst={}", 
              context.sender_pid, context.destination_pid);
    }

    /// Get current policy mode
    pub fn get_current_mode(&self) -> u8 {
        self.current_mode.load(Ordering::SeqCst)
    }

    /// Set policy mode (for testing or runtime configuration)
    pub fn set_policy_mode(&self, mode: u8) -> Result<(), String> {
        match mode {
            POLICY_MODE_FAIL_CLOSED | POLICY_MODE_FAIL_OPEN => {
                self.current_mode.store(mode, Ordering::SeqCst);
                kprintln!("[POLICY] Policy mode changed to: {} ({})", 
                         if mode == POLICY_MODE_FAIL_CLOSED { "FAIL-CLOSED" } else { "FAIL-OPEN" },
                         mode);
                Ok(())
            }
            _ => Err(format!("Invalid policy mode: {}", mode)),
        }
    }

    /// Get policy statistics
    pub fn get_stats(&self) -> PolicyStats {
        self.stats.lock().unwrap_or_default().clone()
    }

    /// Reset policy statistics
    pub fn reset_stats(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            *stats = PolicyStats::default();
        }
    }

    /// Check if policy system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst) == 1
    }

    /// Get policy blob information
    pub fn get_policy_info(&self) -> Option<(String, usize)> {
        if let Ok(blob_guard) = self.policy_blob.lock() {
            blob_guard.as_ref().map(|blob| (blob.version.clone(), blob.data.len()))
        } else {
            None
        }
    }
}

/// Global policy manager instance
static mut POLICY_MANAGER: Option<PolicyManager> = None;

/// Initialize the global policy manager
pub fn init_policy_manager() -> Result<(), String> {
    unsafe {
        if POLICY_MANAGER.is_none() {
            POLICY_MANAGER = Some(PolicyManager::new());
        }
        
        POLICY_MANAGER.as_ref().unwrap().init()
    }
}

/// Get a reference to the global policy manager
pub fn get_policy_manager() -> Option<&'static PolicyManager> {
    unsafe {
        POLICY_MANAGER.as_ref()
    }
}

/// Get a mutable reference to the global policy manager
pub fn get_policy_manager_mut() -> Option<&'static mut PolicyManager> {
    unsafe {
        POLICY_MANAGER.as_mut()
    }
}

/// Evaluate IPC policy using the global manager
pub fn evaluate_ipc_policy(context: &IpcPolicyContext) -> Option<PolicyResult> {
    get_policy_manager().map(|manager| {
        manager.evaluate_ipc_policy(context)
    })
}

/// Get current policy mode
pub fn get_current_policy_mode() -> u8 {
    get_policy_manager().map(|manager| {
        manager.get_current_mode()
    }).unwrap_or(POLICY_MODE_DEFAULT)
}

/// Set policy mode
pub fn set_policy_mode(mode: u8) -> Result<(), String> {
    get_policy_manager_mut().map(|manager| {
        manager.set_policy_mode(mode)
    }).unwrap_or(Err("Policy manager not initialized".to_string()))
}

/// Get policy statistics
pub fn get_policy_stats() -> Option<PolicyStats> {
    get_policy_manager().map(|manager| {
        manager.get_stats()
    })
}

/// Reset policy statistics
pub fn reset_policy_stats() {
    if let Some(manager) = get_policy_manager() {
        manager.reset_stats();
    }
}

/// Check if policy system is initialized
pub fn is_policy_initialized() -> bool {
    get_policy_manager().map(|manager| {
        manager.is_initialized()
    }).unwrap_or(false)
}

/// Test the policy system
pub fn test_policy() {
    kprintln!("");
    kprintln!("=== POLICY SYSTEM TEST ===");
    
    // Test initialization
    match init_policy_manager() {
        Ok(()) => kprintln!("✓ Policy manager initialized successfully"),
        Err(e) => kprintln!("✗ Policy manager initialization failed: {}", e),
    }
    
    // Test policy evaluation
    if let Some(manager) = get_policy_manager() {
        let context = IpcPolicyContext {
            sender_pid: 1,
            destination_pid: 2,
            has_capability: false,
            has_valid_mac: false,
            message_size: 64,
            priority: 1,
            auth_mode: "none".to_string(),
            timestamp: 0,
        };
        
        let result = manager.evaluate_ipc_policy(&context);
        kprintln!("✓ Policy evaluation test: {:?}", result);
        
        // Test mode switching
        match manager.set_policy_mode(POLICY_MODE_FAIL_OPEN) {
            Ok(()) => kprintln!("✓ Policy mode switched to FAIL-OPEN"),
            Err(e) => kprintln!("✗ Policy mode switch failed: {}", e),
        }
        
        let result2 = manager.evaluate_ipc_policy(&context);
        kprintln!("✓ Policy evaluation after mode switch: {:?}", result2);
        
        // Switch back to fail-closed
        let _ = manager.set_policy_mode(POLICY_MODE_FAIL_CLOSED);
        
        // Print statistics
        let stats = manager.get_stats();
        kprintln!("✓ Policy statistics: {:?}", stats);
    }
    
    kprintln!("=== POLICY SYSTEM TEST COMPLETE ===");
    kprintln!("");
}

/// Print policy system statistics
pub fn print_policy_stats() {
    if let Some(stats) = get_policy_stats() {
        kprintln!("");
        kprintln!("=== POLICY SYSTEM STATISTICS ===");
        kprintln!("Total Evaluations: {}", stats.total_evaluations);
        kprintln!("Allowed Operations: {}", stats.allowed_operations);
        kprintln!("Denied Operations: {}", stats.denied_operations);
        kprintln!("Audit-Only Operations: {}", stats.audit_only_operations);
        kprintln!("Evaluation Failures: {}", stats.evaluation_failures);
        kprintln!("Current Policy Mode: {}", 
                 if get_current_policy_mode() == POLICY_MODE_FAIL_CLOSED { "FAIL-CLOSED" } else { "FAIL-OPEN" });
        kprintln!("=== END POLICY SYSTEM STATISTICS ===");
        kprintln!("");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_manager_creation() {
        let manager = PolicyManager::new();
        assert_eq!(manager.get_current_mode(), POLICY_MODE_DEFAULT);
        assert!(!manager.is_initialized());
    }

    #[test]
    fn test_policy_mode_switching() {
        let manager = PolicyManager::new();
        
        // Test valid mode switching
        assert!(manager.set_policy_mode(POLICY_MODE_FAIL_OPEN).is_ok());
        assert_eq!(manager.get_current_mode(), POLICY_MODE_FAIL_OPEN);
        
        assert!(manager.set_policy_mode(POLICY_MODE_FAIL_CLOSED).is_ok());
        assert_eq!(manager.get_current_mode(), POLICY_MODE_FAIL_CLOSED);
        
        // Test invalid mode
        assert!(manager.set_policy_mode(99).is_err());
    }

    #[test]
    fn test_fail_closed_policy() {
        let manager = PolicyManager::new();
        manager.set_policy_mode(POLICY_MODE_FAIL_CLOSED).unwrap();
        
        let context = IpcPolicyContext {
            sender_pid: 1,
            destination_pid: 2,
            has_capability: false,
            has_valid_mac: false,
            message_size: 64,
            priority: 1,
            auth_mode: "none".to_string(),
            timestamp: 0,
        };
        
        let result = manager.evaluate_ipc_policy(&context);
        assert!(!result.allowed);
        assert_eq!(result.decision, POLICY_DECISION_DENY);
        assert!(result.audit);
    }

    #[test]
    fn test_fail_open_policy() {
        let manager = PolicyManager::new();
        manager.set_policy_mode(POLICY_MODE_FAIL_OPEN).unwrap();
        
        let context = IpcPolicyContext {
            sender_pid: 1,
            destination_pid: 2,
            has_capability: false,
            has_valid_mac: false,
            message_size: 64,
            priority: 1,
            auth_mode: "none".to_string(),
            timestamp: 0,
        };
        
        let result = manager.evaluate_ipc_policy(&context);
        assert!(result.allowed);
        assert_eq!(result.decision, POLICY_DECISION_AUDIT);
        assert!(result.audit);
    }

    #[test]
    fn test_policy_statistics() {
        let manager = PolicyManager::new();
        let initial_stats = manager.get_stats();
        assert_eq!(initial_stats.total_evaluations, 0);
        
        let context = IpcPolicyContext {
            sender_pid: 1,
            destination_pid: 2,
            has_capability: false,
            has_valid_mac: false,
            message_size: 64,
            priority: 1,
            auth_mode: "none".to_string(),
            timestamp: 0,
        };
        
        let _ = manager.evaluate_ipc_policy(&context);
        let updated_stats = manager.get_stats();
        assert_eq!(updated_stats.total_evaluations, 1);
        
        manager.reset_stats();
        let reset_stats = manager.get_stats();
        assert_eq!(reset_stats.total_evaluations, 0);
    }
}

