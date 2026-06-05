//! Network policy engine for POSIX networking
//! 
//! This module provides policy-based access control for network operations
//! using OPA (Open Policy Agent) and WASM-based policy evaluation.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Module, Store, Instance, Func, Memory, Config};
use regorus::{Engine as RegoEngine, Policy, Query};
use tokio::sync::RwLock;
use std::path::Path;

/// Policy decision result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// Whether the operation is allowed
    pub allowed: bool,
    /// Reason for the decision
    pub reason: String,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
    /// Policy that made the decision
    pub policy_id: String,
    /// Decision timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Policy engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEngine {
    /// Policy rules
    pub rules: HashMap<String, PolicyRule>,
    /// Default policy
    pub default_policy: PolicyDecision,
    /// Enable audit logging
    pub enable_audit: bool,
    /// Policy cache size
    pub cache_size: usize,
}

/// Policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule priority (higher = more important)
    pub priority: u32,
    /// Rule conditions
    pub conditions: Vec<PolicyCondition>,
    /// Rule actions
    pub actions: PolicyActions,
    /// Rule enabled
    pub enabled: bool,
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    /// Process capability condition
    ProcessCapability { capability: String },
    /// Network namespace condition
    NetworkNamespace { namespace: String },
    /// Address condition
    Address { address: String, cidr: bool },
    /// Port condition
    Port { port: u16, range: Option<(u16, u16)> },
    /// Protocol condition
    Protocol { protocol: String },
    /// Time condition
    Time { start: String, end: String },
    /// Resource condition
    Resource { resource: String, limit: u64 },
    /// Custom condition
    Custom { expression: String },
}

/// Policy actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActions {
    /// Allow action
    pub allow: bool,
    /// Deny action
    pub deny: bool,
    /// Log action
    pub log: bool,
    /// Audit action
    pub audit: bool,
    /// Rate limit action
    pub rate_limit: Option<RateLimit>,
    /// Custom actions
    pub custom: HashMap<String, serde_json::Value>,
}

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests per second
    pub requests_per_second: u32,
    /// Burst size
    pub burst_size: u32,
    /// Window size in seconds
    pub window_size: u64,
}

/// Policy context for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    /// Process capability
    pub process_cap: String,
    /// Network namespace
    pub namespace: String,
    /// Operation type
    pub operation: String,
    /// Target address
    pub target_address: Option<SocketAddr>,
    /// Source address
    pub source_address: Option<SocketAddr>,
    /// Protocol
    pub protocol: Option<String>,
    /// Port
    pub port: Option<u16>,
    /// Additional context
    pub additional: HashMap<String, serde_json::Value>,
}

/// Policy engine implementation
pub struct PolicyEngine {
    /// Policy rules
    rules: Arc<RwLock<HashMap<String, PolicyRule>>>,
    /// Default policy
    default_policy: PolicyDecision,
    /// Rego engine for policy evaluation
    rego_engine: Arc<RwLock<RegoEngine>>,
    /// WASM engine for compiled policies
    wasm_engine: Engine,
    /// Compiled WASM modules
    wasm_modules: Arc<RwLock<HashMap<String, Module>>>,
    /// Policy cache
    policy_cache: Arc<RwLock<HashMap<String, CachedDecision>>>,
    /// Audit log
    audit_log: Arc<RwLock<Vec<PolicyAuditEntry>>>,
    /// Enable audit logging
    enable_audit: bool,
    /// Policy compilation statistics
    stats: Arc<RwLock<PolicyStats>>,
}

/// Cached policy decision with TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDecision {
    /// The decision
    pub decision: PolicyDecision,
    /// Cache timestamp
    pub cached_at: Instant,
    /// Time to live
    pub ttl: Duration,
}

impl CachedDecision {
    /// Check if cache entry is still valid
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}

/// Policy audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAuditEntry {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Process capability
    pub process_cap: String,
    /// Operation
    pub operation: String,
    /// Target
    pub target: String,
    /// Decision
    pub decision: PolicyDecision,
    /// Context
    pub context: PolicyContext,
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Result<Self, String> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_multi_memory(true);
        
        let wasm_engine = Engine::new(&config)
            .map_err(|e| format!("Failed to create WASM engine: {}", e))?;

        Ok(Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            default_policy: PolicyDecision {
                allowed: false,
                reason: "Default deny policy".to_string(),
                context: HashMap::new(),
                policy_id: "default".to_string(),
                timestamp: chrono::Utc::now(),
            },
            rego_engine: Arc::new(RwLock::new(RegoEngine::new())),
            wasm_engine,
            wasm_modules: Arc::new(RwLock::new(HashMap::new())),
            policy_cache: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            enable_audit: true,
            stats: Arc::new(RwLock::new(PolicyStats::default())),
        })
    }

    /// Load and compile a Rego policy to WASM
    pub async fn load_rego_policy(&self, policy_id: String, rego_code: String) -> Result<(), String> {
        // Compile Rego to WASM using OPA
        let wasm_bytes = self.compile_rego_to_wasm(&rego_code)?;
        
        // Load WASM module
        let module = Module::new(&self.wasm_engine, &wasm_bytes)
            .map_err(|e| format!("Failed to load WASM module: {}", e))?;
        
        // Store compiled module
        let mut modules = self.wasm_modules.write().await;
        modules.insert(policy_id, module);
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.policies_compiled += 1;
        
        Ok(())
    }

    /// Compile Rego code to WASM
    fn compile_rego_to_wasm(&self, rego_code: &str) -> Result<Vec<u8>, String> {
        // Mock implementation - in real implementation would use OPA CLI or library
        // to compile Rego to WASM
        let mock_wasm = self.create_mock_wasm_module(rego_code)?;
        Ok(mock_wasm)
    }

    /// Create a mock WASM module for testing
    fn create_mock_wasm_module(&self, _rego_code: &str) -> Result<Vec<u8>, String> {
        // Mock WASM module that implements basic policy evaluation
        // In real implementation, this would be generated by OPA
        let mock_wasm = include_bytes!("../../../../policy/net/policy.wasm");
        Ok(mock_wasm.to_vec())
    }

    /// Evaluate policy using WASM module
    pub async fn evaluate_wasm_policy(
        &self,
        policy_id: &str,
        context: &PolicyContext,
    ) -> Result<PolicyDecision, String> {
        let modules = self.wasm_modules.read().await;
        let module = modules.get(policy_id)
            .ok_or_else(|| format!("Policy {} not found", policy_id))?;

        // Create WASM store and instance
        let mut store = Store::new(&self.wasm_engine, ());
        let instance = Instance::new(&mut store, module, &[])
            .map_err(|e| format!("Failed to create WASM instance: {}", e))?;

        // Get the policy evaluation function
        let eval_func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "evaluate_policy")
            .map_err(|e| format!("Failed to get evaluate_policy function: {}", e))?;

        // Serialize context to JSON
        let context_json = serde_json::to_string(context)
            .map_err(|e| format!("Failed to serialize context: {}", e))?;

        // Allocate memory for context
        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| "Memory not found".to_string())?;

        let context_ptr = self.allocate_string(&mut store, memory, &context_json)?;
        let context_len = context_json.len() as i32;

        // Call WASM function
        let result_ptr = eval_func.call(&mut store, (context_ptr, context_len))
            .map_err(|e| format!("WASM function call failed: {}", e))?;

        // Read result from memory
        let result_json = self.read_string(&mut store, memory, result_ptr as usize)?;
        let decision: PolicyDecision = serde_json::from_str(&result_json)
            .map_err(|e| format!("Failed to deserialize decision: {}", e))?;

        Ok(decision)
    }

    /// Allocate string in WASM memory
    fn allocate_string(
        &self,
        store: &mut Store<()>,
        memory: Memory,
        s: &str,
    ) -> Result<i32, String> {
        let bytes = s.as_bytes();
        let len = bytes.len();
        
        // Mock memory allocation - in real implementation would use proper allocator
        let ptr = 0x1000; // Mock pointer
        
        // Write string to memory
        memory.write(store, ptr, bytes)
            .map_err(|e| format!("Failed to write to memory: {}", e))?;
        
        Ok(ptr as i32)
    }

    /// Read string from WASM memory
    fn read_string(
        &self,
        store: &mut Store<()>,
        memory: Memory,
        ptr: usize,
    ) -> Result<String, String> {
        // Mock string reading - in real implementation would read length first
        let mut buffer = vec![0u8; 1024]; // Mock buffer size
        memory.read(store, ptr, &mut buffer)
            .map_err(|e| format!("Failed to read from memory: {}", e))?;
        
        // Find null terminator
        let null_pos = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());
        let result = String::from_utf8(buffer[..null_pos].to_vec())
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;
        
        Ok(result)
    }

    /// Load policy from file
    pub async fn load_policy_from_file(&self, policy_id: String, file_path: &Path) -> Result<(), String> {
        let rego_code = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read policy file: {}", e))?;
        
        self.load_rego_policy(policy_id, rego_code).await
    }

    /// Hot-reload policy
    pub async fn reload_policy(&self, policy_id: &str, rego_code: String) -> Result<(), String> {
        // Remove old policy
        let mut modules = self.wasm_modules.write().await;
        modules.remove(policy_id);
        drop(modules);

        // Load new policy
        self.load_rego_policy(policy_id.to_string(), rego_code).await
    }

    /// Get policy compilation statistics
    pub async fn get_compilation_stats(&self) -> PolicyCompilationStats {
        let stats = self.stats.read().await;
        PolicyCompilationStats {
            policies_compiled: stats.policies_compiled,
            compilation_errors: stats.compilation_errors,
            last_compilation: stats.last_compilation,
            average_compilation_time: stats.average_compilation_time,
        }
    }

    /// Add a policy rule
    pub fn add_rule(&mut self, rule: PolicyRule) -> Result<(), String> {
        if self.rules.contains_key(&rule.id) {
            return Err("Rule already exists".to_string());
        }

        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove a policy rule
    pub fn remove_rule(&mut self, rule_id: &str) -> Result<(), String> {
        if rule_id == "default" {
            return Err("Cannot remove default rule".to_string());
        }

        self.rules.remove(rule_id);
        Ok(())
    }

    /// Update a policy rule
    pub fn update_rule(&mut self, rule: PolicyRule) -> Result<(), String> {
        if !self.rules.contains_key(&rule.id) {
            return Err("Rule not found".to_string());
        }

        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Check if a bind operation is allowed
    pub async fn check_bind(
        &self,
        process_cap: &str,
        address: &SocketAddr,
    ) -> PolicyDecision {
        let context = PolicyContext {
            process_cap: process_cap.to_string(),
            namespace: "default".to_string(), // TODO: Get from namespace manager
            operation: "bind".to_string(),
            target_address: Some(*address),
            source_address: None,
            protocol: None,
            port: Some(address.port()),
            additional: HashMap::new(),
        };

        self.evaluate_policy(&context).await
    }

    /// Check if a connect operation is allowed
    pub async fn check_connect(
        &self,
        process_cap: &str,
        address: &SocketAddr,
    ) -> PolicyDecision {
        let context = PolicyContext {
            process_cap: process_cap.to_string(),
            namespace: "default".to_string(), // TODO: Get from namespace manager
            operation: "connect".to_string(),
            target_address: Some(*address),
            source_address: None,
            protocol: None,
            port: Some(address.port()),
            additional: HashMap::new(),
        };

        self.evaluate_policy(&context).await
    }

    /// Check if a listen operation is allowed
    pub async fn check_listen(
        &self,
        process_cap: &str,
        address: &SocketAddr,
    ) -> PolicyDecision {
        let context = PolicyContext {
            process_cap: process_cap.to_string(),
            namespace: "default".to_string(), // TODO: Get from namespace manager
            operation: "listen".to_string(),
            target_address: Some(*address),
            source_address: None,
            protocol: None,
            port: Some(address.port()),
            additional: HashMap::new(),
        };

        self.evaluate_policy(&context).await
    }

    /// Check if a send operation is allowed
    pub async fn check_send(
        &self,
        process_cap: &str,
        target_address: &SocketAddr,
        data_size: usize,
    ) -> PolicyDecision {
        let mut additional = HashMap::new();
        additional.insert("data_size".to_string(), serde_json::Value::Number(data_size.into()));

        let context = PolicyContext {
            process_cap: process_cap.to_string(),
            namespace: "default".to_string(), // TODO: Get from namespace manager
            operation: "send".to_string(),
            target_address: Some(*target_address),
            source_address: None,
            protocol: None,
            port: Some(target_address.port()),
            additional,
        };

        self.evaluate_policy(&context).await
    }

    /// Check if a receive operation is allowed
    pub async fn check_recv(
        &self,
        process_cap: &str,
        source_address: &SocketAddr,
        buffer_size: usize,
    ) -> PolicyDecision {
        let mut additional = HashMap::new();
        additional.insert("buffer_size".to_string(), serde_json::Value::Number(buffer_size.into()));

        let context = PolicyContext {
            process_cap: process_cap.to_string(),
            namespace: "default".to_string(), // TODO: Get from namespace manager
            operation: "recv".to_string(),
            target_address: None,
            source_address: Some(*source_address),
            protocol: None,
            port: Some(source_address.port()),
            additional,
        };

        self.evaluate_policy(&context).await
    }

    /// Check if DNS resolution is allowed
    pub async fn check_dns_resolve(
        &self,
        process_cap: &str,
        domain: &str,
    ) -> PolicyDecision {
        let mut additional = HashMap::new();
        additional.insert("domain".to_string(), serde_json::Value::String(domain.to_string()));

        let context = PolicyContext {
            process_cap: process_cap.to_string(),
            namespace: "default".to_string(), // TODO: Get from namespace manager
            operation: "dns_resolve".to_string(),
            target_address: None,
            source_address: None,
            protocol: None,
            port: None,
            additional,
        };

        self.evaluate_policy(&context).await
    }

    /// Evaluate policy for a given context
    async fn evaluate_policy(&self, context: &PolicyContext) -> PolicyDecision {
        // Check cache first
        let cache_key = self.generate_cache_key(context);
        if let Some(cached_decision) = self.policy_cache.get(&cache_key) {
            return cached_decision.clone();
        }

        // Evaluate rules in priority order
        let mut sorted_rules: Vec<_> = self.rules.values().collect();
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        for rule in sorted_rules {
            if !rule.enabled {
                continue;
            }

            if self.evaluate_rule(rule, context).await {
                let decision = PolicyDecision {
                    allowed: rule.actions.allow,
                    reason: format!("Rule: {}", rule.name),
                    context: context.additional.clone(),
                    policy_id: rule.id.clone(),
                    timestamp: chrono::Utc::now(),
                };

                // Cache the decision
                // Note: In real implementation, would need mutable reference
                // self.policy_cache.insert(cache_key, decision.clone());

                // Log audit entry
                if self.enable_audit {
                    self.log_audit(context, &decision);
                }

                return decision;
            }
        }

        // Return default policy if no rules match
        self.default_policy.clone()
    }

    /// Evaluate a specific rule against context
    async fn evaluate_rule(&self, rule: &PolicyRule, context: &PolicyContext) -> bool {
        for condition in &rule.conditions {
            if !self.evaluate_condition(condition, context).await {
                return false;
            }
        }
        true
    }

    /// Evaluate a specific condition against context
    async fn evaluate_condition(&self, condition: &PolicyCondition, context: &PolicyContext) -> bool {
        match condition {
            PolicyCondition::ProcessCapability { capability } => {
                context.process_cap.contains(capability)
            },
            PolicyCondition::NetworkNamespace { namespace } => {
                context.namespace == *namespace
            },
            PolicyCondition::Address { address, cidr } => {
                if *cidr {
                    // CIDR matching
                    if let Some(target_addr) = context.target_address {
                        self.matches_cidr(&target_addr.ip(), address)
                    } else {
                        false
                    }
                } else {
                    // Exact address matching
                    if let Some(target_addr) = context.target_address {
                        target_addr.ip().to_string() == *address
                    } else {
                        false
                    }
                }
            },
            PolicyCondition::Port { port, range } => {
                if let Some(target_port) = context.port {
                    if let Some((start, end)) = range {
                        target_port >= *start && target_port <= *end
                    } else {
                        target_port == *port
                    }
                } else {
                    false
                }
            },
            PolicyCondition::Protocol { protocol } => {
                if let Some(context_protocol) = &context.protocol {
                    context_protocol == protocol
                } else {
                    false
                }
            },
            PolicyCondition::Time { start, end } => {
                // Mock time evaluation
                let now = chrono::Utc::now().time();
                let start_time = start.parse::<chrono::NaiveTime>().unwrap_or_default();
                let end_time = end.parse::<chrono::NaiveTime>().unwrap_or_default();
                now >= start_time && now <= end_time
            },
            PolicyCondition::Resource { resource, limit } => {
                // Mock resource evaluation
                if let Some(value) = context.additional.get(resource) {
                    if let Some(num) = value.as_u64() {
                        num <= *limit
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            PolicyCondition::Custom { expression } => {
                // Mock custom expression evaluation
                // In real implementation, would use a proper expression evaluator
                expression.contains("allow")
            },
        }
    }

    /// Check if IP address matches CIDR
    fn matches_cidr(&self, ip: &std::net::IpAddr, cidr: &str) -> bool {
        // Mock CIDR matching - in real implementation would use proper CIDR library
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                if cidr.starts_with("127.0.0.0/8") {
                    ipv4.octets()[0] == 127
                } else if cidr.starts_with("10.0.0.0/8") {
                    ipv4.octets()[0] == 10
                } else if cidr.starts_with("192.168.0.0/16") {
                    ipv4.octets()[0] == 192 && ipv4.octets()[1] == 168
                } else {
                    false
                }
            },
            std::net::IpAddr::V6(_) => {
                // Mock IPv6 CIDR matching
                cidr.contains("::1")
            },
        }
    }

    /// Generate cache key for context
    fn generate_cache_key(&self, context: &PolicyContext) -> String {
        format!("{}:{}:{}:{}:{}",
            context.process_cap,
            context.namespace,
            context.operation,
            context.target_address.map(|a| a.to_string()).unwrap_or_default(),
            context.source_address.map(|a| a.to_string()).unwrap_or_default()
        )
    }

    /// Log audit entry
    fn log_audit(&self, context: &PolicyContext, decision: &PolicyDecision) {
        let audit_entry = PolicyAuditEntry {
            timestamp: chrono::Utc::now(),
            process_cap: context.process_cap.clone(),
            operation: context.operation.clone(),
            target: context.target_address.map(|a| a.to_string()).unwrap_or_default(),
            decision: decision.clone(),
            context: context.clone(),
        };

        // In real implementation, would store in audit log
        // self.audit_log.push(audit_entry);
    }

    /// Get policy statistics
    pub fn get_stats(&self) -> PolicyStats {
        PolicyStats {
            total_rules: self.rules.len(),
            enabled_rules: self.rules.values().filter(|r| r.enabled).count(),
            cache_size: self.policy_cache.len(),
            audit_entries: self.audit_log.len(),
        }
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> &Vec<PolicyAuditEntry> {
        &self.audit_log
    }

    /// Clear audit log
    pub fn clear_audit_log(&mut self) {
        self.audit_log.clear();
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default PolicyEngine")
    }
}

/// Policy statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyStats {
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub cache_size: usize,
    pub audit_entries: usize,
    pub policies_compiled: usize,
    pub compilation_errors: usize,
    pub last_compilation: Option<Instant>,
    pub average_compilation_time: Duration,
}

/// Policy compilation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCompilationStats {
    pub policies_compiled: usize,
    pub compilation_errors: usize,
    pub last_compilation: Option<Instant>,
    pub average_compilation_time: Duration,
}

/// Network policy (simplified)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkPolicy {
    /// Policy rules
    pub rules: Vec<String>,
    /// Default action
    pub default_action: String,
    /// Enable logging
    pub enable_logging: bool,
}
