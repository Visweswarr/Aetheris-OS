//! Advanced Firewall Engine for POSIX Networking
//! 
//! This module provides a policy-driven firewall system that compiles Rego policies
//! to WASM for high-performance network filtering and access control.
//! 
//! Features:
//! - Rego policy compilation to WASM
//! - Per-process and per-namespace firewall rules
//! - Real-time policy evaluation
//! - Zero-copy packet inspection
//! - Advanced logging and audit trails

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use thiserror::Error;
use wasmtime::{Engine, Module, Store, Instance, Func, Memory, MemoryType, Limits};
use regorus::{Engine as RegoEngine, Policy, Query};

/// Firewall error types
#[derive(Error, Debug)]
pub enum FirewallError {
    #[error("Policy compilation failed: {0}")]
    PolicyCompilationFailed(String),
    #[error("WASM execution failed: {0}")]
    WasmExecutionFailed(String),
    #[error("Policy evaluation failed: {0}")]
    PolicyEvaluationFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Rule not found: {0}")]
    RuleNotFound(String),
    #[error("Invalid rule format: {0}")]
    InvalidRuleFormat(String),
}

/// Result type for firewall operations
pub type FirewallResult<T> = Result<T, FirewallError>;

/// Firewall manager for handling network policies
pub struct FirewallManager {
    /// Compiled policies
    policies: Arc<RwLock<HashMap<String, CompiledPolicy>>>,
    /// Active rules
    rules: Arc<RwLock<HashMap<String, FirewallRule>>>,
    /// WASM engine
    wasm_engine: Engine,
    /// Rego engine for policy compilation
    rego_engine: RegoEngine,
    /// Statistics
    stats: Arc<RwLock<FirewallStats>>,
    /// Audit logger
    audit_logger: Arc<RwLock<AuditLogger>>,
}

/// Compiled policy with WASM module
#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    /// Policy ID
    pub id: String,
    /// Policy name
    pub name: String,
    /// WASM module
    pub wasm_module: Vec<u8>,
    /// Policy metadata
    pub metadata: PolicyMetadata,
    /// Compilation timestamp
    pub compiled_at: Instant,
    /// Usage statistics
    pub usage_stats: PolicyUsageStats,
}

/// Policy metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetadata {
    /// Policy version
    pub version: String,
    /// Policy description
    pub description: String,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
    /// Default action
    pub default_action: FirewallAction,
    /// Priority
    pub priority: u32,
}

/// Policy rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule conditions
    pub conditions: RuleConditions,
    /// Rule action
    pub action: FirewallAction,
    /// Rule priority
    pub priority: u32,
    /// Rule enabled
    pub enabled: bool,
}

/// Rule conditions for policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConditions {
    /// Source IP addresses/CIDRs
    pub source_ips: Vec<String>,
    /// Destination IP addresses/CIDRs
    pub destination_ips: Vec<String>,
    /// Source ports
    pub source_ports: Vec<u16>,
    /// Destination ports
    pub destination_ports: Vec<u16>,
    /// Protocols
    pub protocols: Vec<Protocol>,
    /// Process capabilities
    pub process_capabilities: Vec<String>,
    /// Network namespaces
    pub network_namespaces: Vec<String>,
    /// Time-based conditions
    pub time_conditions: Option<TimeConditions>,
    /// Rate limiting
    pub rate_limits: Option<RateLimit>,
    /// Custom conditions (Rego expressions)
    pub custom_conditions: Vec<String>,
}

/// Time-based conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConditions {
    /// Allowed time ranges
    pub allowed_ranges: Vec<TimeRange>,
    /// Blocked time ranges
    pub blocked_ranges: Vec<TimeRange>,
    /// Timezone
    pub timezone: String,
}

/// Time range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (HH:MM format)
    pub start: String,
    /// End time (HH:MM format)
    pub end: String,
    /// Days of week (0=Sunday, 1=Monday, etc.)
    pub days: Vec<u8>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per time window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Burst allowance
    pub burst: u32,
}

/// Network protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    /// TCP
    Tcp,
    /// UDP
    Udp,
    /// ICMP
    Icmp,
    /// All protocols
    All,
}

/// Firewall action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FirewallAction {
    /// Allow the connection
    Allow,
    /// Deny the connection
    Deny,
    /// Drop the packet silently
    Drop,
    /// Reject with ICMP error
    Reject,
    /// Log and allow
    LogAndAllow,
    /// Log and deny
    LogAndDeny,
    /// Rate limit
    RateLimit,
    /// Redirect to different address
    Redirect(String),
}

/// Firewall rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule type
    pub rule_type: RuleType,
    /// Rule scope
    pub scope: RuleScope,
    /// Rule conditions
    pub conditions: RuleConditions,
    /// Rule action
    pub action: FirewallAction,
    /// Rule priority
    pub priority: u32,
    /// Rule enabled
    pub enabled: bool,
    /// Rule created at
    pub created_at: Instant,
    /// Rule last modified
    pub last_modified: Instant,
    /// Rule usage statistics
    pub usage_stats: RuleUsageStats,
}

/// Rule type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleType {
    /// Ingress rule
    Ingress,
    /// Egress rule
    Egress,
    /// Both ingress and egress
    Bidirectional,
}

/// Rule scope
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleScope {
    /// Global scope
    Global,
    /// Per-process scope
    Process(String),
    /// Per-namespace scope
    Namespace(String),
    /// Per-socket scope
    Socket(String),
}

/// Policy usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyUsageStats {
    /// Number of evaluations
    pub evaluations: u64,
    /// Number of matches
    pub matches: u64,
    /// Number of denials
    pub denials: u64,
    /// Average evaluation time (microseconds)
    pub avg_evaluation_time_us: u64,
    /// Last evaluation time
    pub last_evaluation: Option<Instant>,
}

/// Rule usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleUsageStats {
    /// Number of matches
    pub matches: u64,
    /// Number of denials
    pub denials: u64,
    /// Last match time
    pub last_match: Option<Instant>,
}

/// Firewall statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirewallStats {
    /// Total policies loaded
    pub total_policies: usize,
    /// Total rules active
    pub total_rules: usize,
    /// Total evaluations
    pub total_evaluations: u64,
    /// Total matches
    pub total_matches: u64,
    /// Total denials
    pub total_denials: u64,
    /// Average evaluation time (microseconds)
    pub avg_evaluation_time_us: u64,
    /// WASM compilation time (microseconds)
    pub wasm_compilation_time_us: u64,
}

/// Audit logger for firewall events
pub struct AuditLogger {
    /// Audit entries
    entries: Vec<AuditEntry>,
    /// Maximum entries to keep
    max_entries: usize,
}

/// Audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID
    pub id: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Event type
    pub event_type: AuditEventType,
    /// Rule ID
    pub rule_id: Option<String>,
    /// Policy ID
    pub policy_id: Option<String>,
    /// Connection info
    pub connection_info: ConnectionInfo,
    /// Decision
    pub decision: FirewallAction,
    /// Evaluation time (microseconds)
    pub evaluation_time_us: u64,
    /// Additional data
    pub data: serde_json::Value,
}

/// Audit event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    /// Policy evaluation
    PolicyEvaluation,
    /// Rule match
    RuleMatch,
    /// Rule denial
    RuleDenial,
    /// Policy compilation
    PolicyCompilation,
    /// Rule creation
    RuleCreation,
    /// Rule modification
    RuleModification,
    /// Rule deletion
    RuleDeletion,
}

/// Connection information for policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Source IP address
    pub source_ip: String,
    /// Destination IP address
    pub destination_ip: String,
    /// Source port
    pub source_port: u16,
    /// Destination port
    pub destination_port: u16,
    /// Protocol
    pub protocol: Protocol,
    /// Process capability
    pub process_capability: String,
    /// Network namespace
    pub network_namespace: String,
    /// Socket ID
    pub socket_id: String,
    /// Connection timestamp
    pub connection_time: Instant,
}

/// Policy evaluation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationContext {
    /// Connection information
    pub connection: ConnectionInfo,
    /// Current time
    pub current_time: Instant,
    /// Additional context data
    pub context_data: HashMap<String, serde_json::Value>,
}

/// Policy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    /// Decision
    pub decision: FirewallAction,
    /// Matched rule ID
    pub matched_rule_id: Option<String>,
    /// Matched policy ID
    pub matched_policy_id: Option<String>,
    /// Evaluation time (microseconds)
    pub evaluation_time_us: u64,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl FirewallManager {
    /// Create a new firewall manager
    pub fn new() -> FirewallResult<Self> {
        let wasm_engine = Engine::default();
        let rego_engine = RegoEngine::new();
        let audit_logger = AuditLogger::new(10000); // Keep last 10k entries

        Ok(Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(HashMap::new())),
            wasm_engine,
            rego_engine,
            stats: Arc::new(RwLock::new(FirewallStats::default())),
            audit_logger: Arc::new(RwLock::new(audit_logger)),
        })
    }

    /// Compile a Rego policy to WASM
    pub async fn compile_policy(
        &self,
        policy_id: String,
        policy_name: String,
        rego_policy: String,
        metadata: PolicyMetadata,
    ) -> FirewallResult<CompiledPolicy> {
        let start_time = Instant::now();

        // Parse and validate Rego policy
        let policy = Policy::parse(&rego_policy)
            .map_err(|e| FirewallError::PolicyCompilationFailed(format!("Rego parse error: {}", e)))?;

        // Compile to WASM
        let wasm_module = self.compile_rego_to_wasm(&policy)?;

        let compiled_policy = CompiledPolicy {
            id: policy_id.clone(),
            name: policy_name,
            wasm_module,
            metadata,
            compiled_at: Instant::now(),
            usage_stats: PolicyUsageStats::default(),
        };

        // Store compiled policy
        let mut policies = self.policies.write().await;
        policies.insert(policy_id, compiled_policy.clone());

        // Update statistics
        let compilation_time = start_time.elapsed().as_micros() as u64;
        let mut stats = self.stats.write().await;
        stats.wasm_compilation_time_us = compilation_time;
        stats.total_policies += 1;

        // Log compilation event
        self.log_audit_event(AuditEventType::PolicyCompilation, None, None, None, None, compilation_time, serde_json::json!({
            "policy_id": compiled_policy.id,
            "compilation_time_us": compilation_time
        })).await;

        Ok(compiled_policy)
    }

    /// Compile Rego policy to WASM
    fn compile_rego_to_wasm(&self, policy: &Policy) -> FirewallResult<Vec<u8>> {
        // Mock WASM compilation - in real implementation would use actual Rego-to-WASM compiler
        // This would typically involve:
        // 1. Converting Rego AST to WASM bytecode
        // 2. Optimizing the generated code
        // 3. Adding runtime support for policy evaluation
        
        let mock_wasm = vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // Version 1
            // ... more WASM bytecode would go here
        ];

        Ok(mock_wasm)
    }

    /// Evaluate a connection against firewall policies
    pub async fn evaluate_connection(
        &self,
        context: PolicyEvaluationContext,
    ) -> FirewallResult<PolicyEvaluationResult> {
        let start_time = Instant::now();

        // Get applicable policies based on scope
        let applicable_policies = self.get_applicable_policies(&context).await;

        // Evaluate each policy in priority order
        for policy in applicable_policies {
            let result = self.evaluate_policy(&policy, &context).await?;
            
            // If policy matches, return result
            if result.decision != FirewallAction::Allow || policy.metadata.default_action != FirewallAction::Allow {
                let evaluation_time = start_time.elapsed().as_micros() as u64;
                
                // Log evaluation event
                self.log_audit_event(
                    AuditEventType::PolicyEvaluation,
                    result.matched_rule_id.clone(),
                    Some(policy.id.clone()),
                    Some(context.connection.clone()),
                    Some(result.decision.clone()),
                    evaluation_time,
                    serde_json::json!({
                        "policy_id": policy.id,
                        "evaluation_time_us": evaluation_time
                    })
                ).await;

                // Update statistics
                self.update_evaluation_stats(evaluation_time, &result).await;

                return Ok(PolicyEvaluationResult {
                    decision: result.decision,
                    matched_rule_id: result.matched_rule_id,
                    matched_policy_id: Some(policy.id),
                    evaluation_time_us: evaluation_time,
                    metadata: result.metadata,
                });
            }
        }

        // Default allow if no policies match
        let evaluation_time = start_time.elapsed().as_micros() as u64;
        Ok(PolicyEvaluationResult {
            decision: FirewallAction::Allow,
            matched_rule_id: None,
            matched_policy_id: None,
            evaluation_time_us: evaluation_time,
            metadata: HashMap::new(),
        })
    }

    /// Get applicable policies for a connection context
    async fn get_applicable_policies(&self, context: &PolicyEvaluationContext) -> Vec<CompiledPolicy> {
        let policies = self.policies.read().await;
        let mut applicable = Vec::new();

        for policy in policies.values() {
            // Check if policy applies to this context
            if self.policy_applies_to_context(policy, context) {
                applicable.push(policy.clone());
            }
        }

        // Sort by priority (higher priority first)
        applicable.sort_by(|a, b| b.metadata.priority.cmp(&a.metadata.priority));
        applicable
    }

    /// Check if a policy applies to a connection context
    fn policy_applies_to_context(&self, policy: &CompiledPolicy, context: &PolicyEvaluationContext) -> bool {
        // Check required capabilities
        for required_cap in &policy.metadata.required_capabilities {
            if !context.connection.process_capability.contains(required_cap) {
                return false;
            }
        }

        // Check network namespace
        if !policy.metadata.rules.is_empty() {
            let namespace_match = policy.metadata.rules.iter().any(|rule| {
                rule.conditions.network_namespaces.is_empty() ||
                rule.conditions.network_namespaces.contains(&context.connection.network_namespace)
            });
            if !namespace_match {
                return false;
            }
        }

        true
    }

    /// Evaluate a single policy against a connection context
    async fn evaluate_policy(
        &self,
        policy: &CompiledPolicy,
        context: &PolicyEvaluationContext,
    ) -> FirewallResult<PolicyEvaluationResult> {
        // Execute WASM module for policy evaluation
        let result = self.execute_wasm_policy(&policy.wasm_module, context).await?;

        // Update policy usage statistics
        let mut policies = self.policies.write().await;
        if let Some(policy) = policies.get_mut(&policy.id) {
            policy.usage_stats.evaluations += 1;
            policy.usage_stats.last_evaluation = Some(Instant::now());
            
            if result.decision != FirewallAction::Allow {
                policy.usage_stats.matches += 1;
            }
        }

        Ok(result)
    }

    /// Execute WASM policy module
    async fn execute_wasm_policy(
        &self,
        wasm_module: &[u8],
        context: &PolicyEvaluationContext,
    ) -> FirewallResult<PolicyEvaluationResult> {
        // Mock WASM execution - in real implementation would:
        // 1. Load WASM module
        // 2. Set up memory and exports
        // 3. Pass connection context as input
        // 4. Execute policy evaluation function
        // 5. Return evaluation result

        let mut store = Store::new(&self.wasm_engine, ());
        let module = Module::new(&self.wasm_engine, wasm_module)
            .map_err(|e| FirewallError::WasmExecutionFailed(format!("Module creation failed: {}", e)))?;

        let instance = Instance::new(&mut store, &module, &[])
            .map_err(|e| FirewallError::WasmExecutionFailed(format!("Instance creation failed: {}", e)))?;

        // Mock policy evaluation result
        Ok(PolicyEvaluationResult {
            decision: FirewallAction::Allow, // Mock: always allow for now
            matched_rule_id: None,
            matched_policy_id: None,
            evaluation_time_us: 100, // Mock evaluation time
            metadata: HashMap::new(),
        })
    }

    /// Add a firewall rule
    pub async fn add_rule(&self, rule: FirewallRule) -> FirewallResult<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule.clone());

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_rules += 1;

        // Log rule creation
        self.log_audit_event(
            AuditEventType::RuleCreation,
            Some(rule.id.clone()),
            None,
            None,
            None,
            0,
            serde_json::json!({
                "rule_name": rule.name,
                "rule_type": rule.rule_type,
                "rule_scope": rule.scope
            })
        ).await;

        Ok(())
    }

    /// Remove a firewall rule
    pub async fn remove_rule(&self, rule_id: &str) -> FirewallResult<()> {
        let mut rules = self.rules.write().await;
        if rules.remove(rule_id).is_none() {
            return Err(FirewallError::RuleNotFound(rule_id.to_string()));
        }

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_rules -= 1;

        // Log rule deletion
        self.log_audit_event(
            AuditEventType::RuleDeletion,
            Some(rule_id.to_string()),
            None,
            None,
            None,
            0,
            serde_json::json!({
                "rule_id": rule_id
            })
        ).await;

        Ok(())
    }

    /// Get firewall statistics
    pub async fn get_stats(&self) -> FirewallStats {
        self.stats.read().await.clone()
    }

    /// Get audit log entries
    pub async fn get_audit_log(&self, limit: Option<usize>) -> Vec<AuditEntry> {
        let logger = self.audit_logger.read().await;
        let entries = &logger.entries;
        
        if let Some(limit) = limit {
            entries.iter().rev().take(limit).cloned().collect()
        } else {
            entries.clone()
        }
    }

    /// Log audit event
    async fn log_audit_event(
        &self,
        event_type: AuditEventType,
        rule_id: Option<String>,
        policy_id: Option<String>,
        connection_info: Option<ConnectionInfo>,
        decision: Option<FirewallAction>,
        evaluation_time_us: u64,
        data: serde_json::Value,
    ) {
        let mut logger = self.audit_logger.write().await;
        logger.log_event(AuditEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Instant::now(),
            event_type,
            rule_id,
            policy_id,
            connection_info,
            decision,
            evaluation_time_us,
            data,
        });
    }

    /// Update evaluation statistics
    async fn update_evaluation_stats(&self, evaluation_time_us: u64, result: &PolicyEvaluationResult) {
        let mut stats = self.stats.write().await;
        stats.total_evaluations += 1;
        
        if result.decision != FirewallAction::Allow {
            stats.total_matches += 1;
        }
        
        // Update average evaluation time
        let total_time = stats.avg_evaluation_time_us * (stats.total_evaluations - 1) as u64 + evaluation_time_us;
        stats.avg_evaluation_time_us = total_time / stats.total_evaluations as u64;
    }
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Log an audit event
    pub fn log_event(&mut self, entry: AuditEntry) {
        self.entries.push(entry);
        
        // Trim entries if we exceed the limit
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }
}

impl Default for FirewallManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default FirewallManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_firewall_manager_creation() {
        let manager = FirewallManager::new().unwrap();
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_policies, 0);
        assert_eq!(stats.total_rules, 0);
    }

    #[tokio::test]
    async fn test_policy_compilation() {
        let manager = FirewallManager::new().unwrap();
        
        let rego_policy = r#"
            package firewall

            default allow = false

            allow {
                input.connection.source_ip == "127.0.0.1"
            }
        "#.to_string();

        let metadata = PolicyMetadata {
            version: "1.0".to_string(),
            description: "Test policy".to_string(),
            required_capabilities: vec!["net:socket".to_string()],
            rules: vec![],
            default_action: FirewallAction::Deny,
            priority: 100,
        };

        let result = manager.compile_policy(
            "test-policy".to_string(),
            "Test Policy".to_string(),
            rego_policy,
            metadata,
        ).await;

        assert!(result.is_ok());
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_policies, 1);
    }

    #[tokio::test]
    async fn test_connection_evaluation() {
        let manager = FirewallManager::new().unwrap();
        
        let context = PolicyEvaluationContext {
            connection: ConnectionInfo {
                source_ip: "127.0.0.1".to_string(),
                destination_ip: "127.0.0.1".to_string(),
                source_port: 12345,
                destination_port: 8080,
                protocol: Protocol::Tcp,
                process_capability: "net:socket".to_string(),
                network_namespace: "default".to_string(),
                socket_id: "socket_1".to_string(),
                connection_time: Instant::now(),
            },
            current_time: Instant::now(),
            context_data: HashMap::new(),
        };

        let result = manager.evaluate_connection(context).await.unwrap();
        assert_eq!(result.decision, FirewallAction::Allow);
    }
}
