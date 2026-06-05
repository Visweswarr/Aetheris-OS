//! Script to generate AI Core Service capability policy file
//!
//! This script generates a CBOR-encoded policy file that defines
//! all capabilities, resources, and access rules for the AI Core Service.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_cbor;

/// Capability policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    /// Policy version
    pub version: String,
    /// Policy metadata
    pub metadata: PolicyMetadata,
    /// Default policy (deny-by-default)
    pub default_policy: DefaultPolicy,
    /// Capability definitions
    pub capabilities: HashMap<String, CapabilityDefinition>,
    /// Resource definitions
    pub resources: HashMap<String, ResourceDefinition>,
    /// Access rules
    pub access_rules: Vec<AccessRule>,
    /// Issuer configurations
    pub issuers: HashMap<String, IssuerPolicy>,
}

/// Policy metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetadata {
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: String,
    /// Created timestamp
    pub created_at: u64,
    /// Last updated timestamp
    pub updated_at: u64,
    /// Policy author
    pub author: String,
    /// Policy version
    pub version: String,
}

/// Default policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultPolicy {
    /// Default action (deny-by-default)
    pub default_action: String,
    /// Default reason
    pub default_reason: String,
    /// Whether to log denied requests
    pub log_denied: bool,
    /// Whether to audit all requests
    pub audit_all: bool,
}

/// Capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    /// Capability name
    pub name: String,
    /// Capability description
    pub description: String,
    /// Capability category
    pub category: String,
    /// Required resources
    pub required_resources: Vec<String>,
    /// Allowed actions
    pub allowed_actions: Vec<String>,
    /// Capability conditions
    pub conditions: Vec<CapabilityCondition>,
    /// Capability metadata
    pub metadata: HashMap<String, String>,
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    /// Resource name
    pub name: String,
    /// Resource description
    pub description: String,
    /// Resource type
    pub resource_type: String,
    /// Resource pattern
    pub pattern: String,
    /// Resource attributes
    pub attributes: HashMap<String, String>,
    /// Resource metadata
    pub metadata: HashMap<String, String>,
}

/// Access rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRule {
    /// Rule ID
    pub rule_id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule priority (higher = more important)
    pub priority: u32,
    /// Rule effect (allow/deny)
    pub effect: String,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Required resources
    pub required_resources: Vec<String>,
    /// Required actions
    pub required_actions: Vec<String>,
    /// Rule conditions
    pub conditions: Vec<RuleCondition>,
    /// Rule metadata
    pub metadata: HashMap<String, String>,
}

/// Capability condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityCondition {
    /// Condition field
    pub field: String,
    /// Condition operator
    pub operator: String,
    /// Condition value
    pub value: String,
    /// Condition description
    pub description: String,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    /// Condition field
    pub field: String,
    /// Condition operator
    pub operator: String,
    /// Condition value
    pub value: String,
    /// Condition description
    pub description: String,
}

/// Issuer policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerPolicy {
    /// Issuer name
    pub name: String,
    /// Issuer description
    pub description: String,
    /// Whether issuer is trusted
    pub trusted: bool,
    /// Maximum token lifetime (seconds)
    pub max_token_lifetime: u64,
    /// Allowed capabilities
    pub allowed_capabilities: Vec<String>,
    /// Issuer metadata
    pub metadata: HashMap<String, String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let policy = create_ai_core_policy();
    
    // Serialize to CBOR
    let cbor_data = serde_cbor::to_vec(&policy)?;
    
    // Write to file
    std::fs::write("configs/caps/ai_core.policy.cbor", cbor_data)?;
    
    println!("Generated AI Core Service capability policy file: configs/caps/ai_core.policy.cbor");
    println!("Policy contains {} capabilities, {} resources, {} access rules", 
             policy.capabilities.len(), 
             policy.resources.len(), 
             policy.access_rules.len());
    
    Ok(())
}

fn create_ai_core_policy() -> CapabilityPolicy {
    let mut capabilities = HashMap::new();
    let mut resources = HashMap::new();
    let mut access_rules = Vec::new();
    let mut issuers = HashMap::new();
    
    // Define capabilities
    capabilities.insert("ai:chat".to_string(), CapabilityDefinition {
        name: "ai:chat".to_string(),
        description: "Chat with AI assistant".to_string(),
        category: "ai".to_string(),
        required_resources: vec!["ai_core".to_string()],
        allowed_actions: vec!["generate_response".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    capabilities.insert("ai:tools".to_string(), CapabilityDefinition {
        name: "ai:tools".to_string(),
        description: "Execute AI tools".to_string(),
        category: "ai".to_string(),
        required_resources: vec!["tools".to_string()],
        allowed_actions: vec!["execute".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    capabilities.insert("ai:intents".to_string(), CapabilityDefinition {
        name: "ai:intents".to_string(),
        description: "Execute system intents".to_string(),
        category: "ai".to_string(),
        required_resources: vec!["system".to_string()],
        allowed_actions: vec!["execute".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    capabilities.insert("ai:memory".to_string(), CapabilityDefinition {
        name: "ai:memory".to_string(),
        description: "Access AI memory store".to_string(),
        category: "ai".to_string(),
        required_resources: vec!["memory".to_string()],
        allowed_actions: vec!["read".to_string(), "write".to_string(), "delete".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    capabilities.insert("ai:admin".to_string(), CapabilityDefinition {
        name: "ai:admin".to_string(),
        description: "Administrative access to AI Core Service".to_string(),
        category: "admin".to_string(),
        required_resources: vec!["ai_core".to_string()],
        allowed_actions: vec!["*".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    // Define resources
    resources.insert("ai_core".to_string(), ResourceDefinition {
        name: "ai_core".to_string(),
        description: "AI Core Service".to_string(),
        resource_type: "service".to_string(),
        pattern: "ai_core".to_string(),
        attributes: HashMap::new(),
        metadata: HashMap::new(),
    });
    
    resources.insert("tools".to_string(), ResourceDefinition {
        name: "tools".to_string(),
        description: "AI tools".to_string(),
        resource_type: "tool".to_string(),
        pattern: "tools:*".to_string(),
        attributes: HashMap::new(),
        metadata: HashMap::new(),
    });
    
    resources.insert("system".to_string(), ResourceDefinition {
        name: "system".to_string(),
        description: "System resources".to_string(),
        resource_type: "system".to_string(),
        pattern: "system:*".to_string(),
        attributes: HashMap::new(),
        metadata: HashMap::new(),
    });
    
    resources.insert("memory".to_string(), ResourceDefinition {
        name: "memory".to_string(),
        description: "AI memory store".to_string(),
        resource_type: "storage".to_string(),
        pattern: "memory:*".to_string(),
        attributes: HashMap::new(),
        metadata: HashMap::new(),
    });
    
    // Define access rules
    access_rules.push(AccessRule {
        rule_id: "rule_001".to_string(),
        name: "Allow AI Chat".to_string(),
        description: "Allow users to chat with AI assistant".to_string(),
        priority: 100,
        effect: "allow".to_string(),
        required_capabilities: vec!["ai:chat".to_string()],
        required_resources: vec!["ai_core".to_string()],
        required_actions: vec!["generate_response".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    access_rules.push(AccessRule {
        rule_id: "rule_002".to_string(),
        name: "Allow Tool Execution".to_string(),
        description: "Allow users to execute AI tools".to_string(),
        priority: 100,
        effect: "allow".to_string(),
        required_capabilities: vec!["ai:tools".to_string()],
        required_resources: vec!["tools".to_string()],
        required_actions: vec!["execute".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    access_rules.push(AccessRule {
        rule_id: "rule_003".to_string(),
        name: "Allow System Intents".to_string(),
        description: "Allow users to execute system intents".to_string(),
        priority: 100,
        effect: "allow".to_string(),
        required_capabilities: vec!["ai:intents".to_string()],
        required_resources: vec!["system".to_string()],
        required_actions: vec!["execute".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    access_rules.push(AccessRule {
        rule_id: "rule_004".to_string(),
        name: "Allow Memory Access".to_string(),
        description: "Allow users to access AI memory store".to_string(),
        priority: 100,
        effect: "allow".to_string(),
        required_capabilities: vec!["ai:memory".to_string()],
        required_resources: vec!["memory".to_string()],
        required_actions: vec!["read".to_string(), "write".to_string(), "delete".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    access_rules.push(AccessRule {
        rule_id: "rule_005".to_string(),
        name: "Allow Admin Access".to_string(),
        description: "Allow administrators full access".to_string(),
        priority: 200,
        effect: "allow".to_string(),
        required_capabilities: vec!["ai:admin".to_string()],
        required_resources: vec!["ai_core".to_string()],
        required_actions: vec!["*".to_string()],
        conditions: vec![],
        metadata: HashMap::new(),
    });
    
    // Define issuers
    issuers.insert("aetheris-system".to_string(), IssuerPolicy {
        name: "Aetheris System".to_string(),
        description: "System issuer for AI Core Service".to_string(),
        trusted: true,
        max_token_lifetime: 3600, // 1 hour
        allowed_capabilities: vec![
            "ai:chat".to_string(),
            "ai:tools".to_string(),
            "ai:intents".to_string(),
            "ai:memory".to_string(),
            "ai:admin".to_string(),
        ],
        metadata: HashMap::new(),
    });
    
    CapabilityPolicy {
        version: "1.0.0".to_string(),
        metadata: PolicyMetadata {
            name: "AI Core Service Capability Policy".to_string(),
            description: "Capability policy for AI Core Service with deny-by-default security model".to_string(),
            created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            updated_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            author: "Aetheris OS Team".to_string(),
            version: "1.0.0".to_string(),
        },
        default_policy: DefaultPolicy {
            default_action: "deny".to_string(),
            default_reason: "Access denied by default policy".to_string(),
            log_denied: true,
            audit_all: true,
        },
        capabilities,
        resources,
        access_rules,
        issuers,
    }
}
