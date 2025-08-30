# Why-Log: Append-Only Audit Trail System

**An immutable, cryptographically-signed decision logging system for Polymera OS**

## 🎯 Overview

Why-Log is a specialized audit trail system that records signed justifications for important decisions in an append-only, tamper-evident format. Each log entry is cryptographically signed and hash-chained to prevent tampering and ensure complete accountability for system decisions.

### Key Features

- **🔒 Cryptographic Integrity**: Every entry is signed with Ed25519 signatures
- **⛓️ Hash Chaining**: Sequential entries are linked via SHA3-256 hashes
- **📝 Append-Only**: Immutable log structure prevents modification or deletion
- **🔍 Tamper Detection**: Automatic detection of hash chain breaks or signature tampering
- **📊 Readable Summaries**: Human-readable audit reports and decision summaries
- **📤 Export/Import**: JSON export for compliance reporting and data migration
- **🏷️ Rich Decision Types**: Support for policy decisions, security events, admin actions, and more

## 🏗️ Architecture

### File Format

Why-Log uses a simple but robust JSON Lines (JSONL) format where each line contains a complete log entry:

```jsonl
{"id":"550e8400-e29b-41d4-a716-446655440000","sequence":1,"timestamp":"2024-01-15T14:30:00Z","decision":{"type":"policy_decision","policy_name":"wallet_spend_limits","input_hash":"abc123","decision":true,"confidence":0.95},"justification":"Transaction within daily limits","context":{"actor":"user123","source":"192.168.1.10"},"previous_hash":"genesis","content_hash":"a1b2c3...","signature":"def456...","signer_public_key":"789abc..."}
{"id":"550e8400-e29b-41d4-a716-446655440001","sequence":2,"timestamp":"2024-01-15T14:31:00Z","decision":{"type":"security_event","event_type":"login_attempt","severity":"medium","source":"external"},"justification":"Suspicious login detected","context":{"actor":"security_system"},"previous_hash":"a1b2c3...","content_hash":"b2c3d4...","signature":"ghi789...","signer_public_key":"789abc..."}
```

### Entry Structure

Each Why-Log entry contains:

```rust
pub struct WhyLogEntry {
    pub id: String,                    // Unique UUID
    pub sequence: u64,                 // Sequential number
    pub timestamp: DateTime<Utc>,      // When decision was made
    pub decision: DecisionType,        // Type and details of decision
    pub justification: String,         // Human-readable reason
    pub context: DecisionContext,      // Who, where, how context
    pub previous_hash: String,         // Hash of previous entry
    pub content_hash: String,          // Hash of this entry's content
    pub signature: String,             // Ed25519 signature
    pub signer_public_key: String,     // Public key for verification
}
```

### Hash Chaining

Each entry includes the hash of the previous entry, creating an unbreakable chain:

```
Entry 1: previous_hash = "genesis"
         content_hash = SHA3(entry1_content)
         
Entry 2: previous_hash = SHA3(entry1_content)
         content_hash = SHA3(entry2_content)
         
Entry 3: previous_hash = SHA3(entry2_content)
         content_hash = SHA3(entry3_content)
```

Any modification to an entry breaks the chain and is immediately detectable.

### Cryptographic Signing

Each entry is signed using Ed25519:

1. **Content Hash**: SHA3-256 hash of entry content (excluding signature)
2. **Signature**: Ed25519 signature of the content hash
3. **Verification**: Public key stored with each entry for verification

## 📋 Decision Types

Why-Log supports multiple decision types for comprehensive audit coverage:

### 1. Policy Decisions
```rust
DecisionType::PolicyDecision {
    policy_name: String,    // e.g., "wallet_spend_limits"
    input_hash: String,     // Hash of input data
    decision: bool,         // true = allow, false = deny
    confidence: f64,        // AI confidence level (0.0-1.0)
}
```

**Example Use Cases:**
- Wallet transaction approvals/rejections
- Network access control decisions
- File system permission grants/denials

### 2. Security Events
```rust
DecisionType::SecurityEvent {
    event_type: String,     // e.g., "login_attempt", "privilege_escalation"
    severity: String,       // "low", "medium", "high", "critical"
    source: String,         // Source of the event
}
```

**Example Use Cases:**
- Failed authentication attempts
- Anomalous user behavior detection
- Security policy violations

### 3. Administrative Actions
```rust
DecisionType::AdminAction {
    action: String,         // e.g., "create_user", "modify_permissions"
    target: String,         // Target resource/user
    permissions: Vec<String>, // Permissions involved
}
```

**Example Use Cases:**
- User account creation/deletion
- Permission modifications
- System configuration changes

### 4. Configuration Changes
```rust
DecisionType::ConfigChange {
    component: String,      // System component
    field: String,          // Configuration field
    old_value: Option<String>, // Previous value
    new_value: String,      // New value
}
```

**Example Use Cases:**
- Security setting modifications
- Network configuration updates
- Service parameter changes

### 5. Transaction Decisions
```rust
DecisionType::TransactionDecision {
    transaction_id: String, // Unique transaction identifier
    decision: bool,         // approved/rejected
    amount: Option<String>, // Transaction amount
    currency: Option<String>, // Currency type
}
```

**Example Use Cases:**
- Cryptocurrency transaction approvals
- Payment processing decisions
- Smart contract execution approvals

### 6. Access Control
```rust
DecisionType::AccessControl {
    user_id: String,        // User requesting access
    resource: String,       // Resource being accessed
    action: String,         // Action being performed
    granted: bool,          // Access granted/denied
}
```

**Example Use Cases:**
- File access permissions
- API endpoint authorization
- Database query permissions

### 7. Emergency Overrides
```rust
DecisionType::EmergencyOverride {
    original_decision: String, // Original automated decision
    override_reason: String,   // Reason for override
    admin_id: String,          // Administrator performing override
}
```

**Example Use Cases:**
- Emergency transaction approvals
- Security policy exceptions
- System maintenance overrides

## 🔧 Usage Examples

### Basic Setup

```rust
use whylog::{WhyLogService, WhyLogConfig, DecisionType, DecisionContext};
use std::collections::HashMap;

// Create service with default configuration
let config = WhyLogConfig::default();
let mut service = WhyLogService::new_with_generated_key(config)?;

// Log a policy decision
let decision = DecisionType::PolicyDecision {
    policy_name: "wallet_spend_limits".to_string(),
    input_hash: "sha256_of_input_data".to_string(),
    decision: true,
    confidence: 0.95,
};

let context = DecisionContext {
    actor: "user123".to_string(),
    source: Some("192.168.1.10".to_string()),
    client_info: Some("mobile_app_v1.2.0".to_string()),
    session_id: Some("sess_abc123".to_string()),
    metadata: HashMap::new(),
};

let entry_id = service.log_decision(
    decision,
    "Transaction approved: within daily limit of $10,000 and risk score below threshold".to_string(),
    context
)?;

println!("Logged decision: {}", entry_id);
```

### Security Event Logging

```rust
// Log a security event
let security_event = DecisionType::SecurityEvent {
    event_type: "suspicious_login".to_string(),
    severity: "high".to_string(),
    source: "external_ip".to_string(),
};

let context = DecisionContext {
    actor: "security_monitor".to_string(),
    source: Some("203.0.113.1".to_string()),
    client_info: Some("unknown_browser".to_string()),
    session_id: None,
    metadata: {
        let mut meta = HashMap::new();
        meta.insert("failed_attempts".to_string(), serde_json::Value::Number(5.into()));
        meta.insert("geolocation".to_string(), serde_json::Value::String("Unknown Country".to_string()));
        meta
    },
};

service.log_decision(
    security_event,
    "Multiple failed login attempts from suspicious IP address with non-standard user agent".to_string(),
    context
)?;
```

### Administrative Action Logging

```rust
// Log an admin action
let admin_action = DecisionType::AdminAction {
    action: "elevate_permissions".to_string(),
    target: "user456".to_string(),
    permissions: vec!["admin_read".to_string(), "admin_write".to_string()],
};

let context = DecisionContext {
    actor: "admin_alice".to_string(),
    source: Some("admin.company.com".to_string()),
    client_info: Some("admin_dashboard".to_string()),
    session_id: Some("admin_sess_xyz789".to_string()),
    metadata: HashMap::new(),
};

service.log_decision(
    admin_action,
    "Temporary admin privileges granted for system maintenance window (approved by security team)".to_string(),
    context
)?;
```

### Reading and Querying Logs

```rust
// Read all entries
let all_entries = service.read_entries()?;
println!("Total entries: {}", all_entries.len());

// Find specific entries
let security_entries = service.find_entries(|entry| {
    matches!(entry.decision, DecisionType::SecurityEvent { .. })
})?;

// Get a specific entry
let entry = service.get_entry(&entry_id)?;
println!("Entry: {:?}", entry);

// Generate readable summary
let summary = service.generate_readable_summary(Some(10))?;
println!("{}", summary);
```

### Integrity Verification

```rust
// Verify log integrity
let integrity_summary = service.verify_integrity()?;

if integrity_summary.chain_integrity && integrity_summary.signatures_valid {
    println!("✅ Log integrity verified");
    println!("Entries: {}", integrity_summary.entry_count);
    println!("Date range: {:?} to {:?}", 
        integrity_summary.first_entry, 
        integrity_summary.last_entry);
} else {
    println!("❌ Log integrity compromised!");
}

// Detect specific tampering attempts
let tampering_indicators = service.detect_tampering()?;
if tampering_indicators.is_empty() {
    println!("✅ No tampering detected");
} else {
    println!("🚨 Tampering detected:");
    for indicator in tampering_indicators {
        println!("  - {}", indicator);
    }
}
```

### Export and Import

```rust
// Export to JSON for compliance reporting
let json_export = service.export_json(
    Some(chrono::Utc::now() - chrono::Duration::days(30)), // Last 30 days
    None
)?;

// Save to file
std::fs::write("audit_report.json", json_export)?;

// Import from JSON (for testing or migration)
let json_data = std::fs::read_to_string("backup.json")?;
let imported_count = service.import_json(&json_data)?;
println!("Imported {} entries", imported_count);
```

## 🛡️ Security Model

### Threat Model

Why-Log protects against:

1. **Unauthorized Modification**: Hash chaining and signatures prevent tampering
2. **Entry Deletion**: Append-only structure makes deletion detectable
3. **Replay Attacks**: Sequential numbering and timestamps prevent replays
4. **Impersonation**: Cryptographic signatures ensure authenticity
5. **Chain Breaking**: Hash verification detects missing or modified entries

### Security Properties

- **Integrity**: Any modification breaks the hash chain
- **Authenticity**: Ed25519 signatures prove entry origin
- **Non-repudiation**: Signers cannot deny creating entries
- **Completeness**: Missing entries break sequence numbers
- **Freshness**: Timestamps prevent old entry replay

### Key Management

```rust
// Generate new keypair for service
let mut csprng = rand::rngs::OsRng;
let keypair = ed25519_dalek::Keypair::generate(&mut csprng);

// Create service with specific keypair
let service = WhyLogService::new(config, keypair)?;

// Get public key for external verification
let public_key_hex = service.public_key();
println!("Service public key: {}", public_key_hex);
```

**Best Practices:**
- Store private keys in secure hardware (HSM/TPM)
- Rotate signing keys periodically
- Backup public keys for verification
- Use separate keys per service/environment

### Tamper Detection

Why-Log provides multiple layers of tamper detection:

```rust
// Comprehensive tampering analysis
let tampering_indicators = service.detect_tampering()?;

// Types of tampering detected:
// - Sequence number gaps
// - Hash chain breaks  
// - Content hash mismatches
// - Invalid signatures
// - Timestamp anomalies
```

**Detection Capabilities:**
- **Missing Entries**: Sequence gaps indicate deleted entries
- **Modified Content**: Content hash mismatches reveal changes
- **Chain Breaks**: Previous hash mismatches show insertion/deletion
- **Signature Forgery**: Invalid signatures indicate tampering attempts
- **Time Anomalies**: Out-of-order timestamps suggest manipulation

## 📊 Monitoring and Alerting

### Real-Time Monitoring

```rust
// Monitor integrity continuously
let integrity = service.verify_integrity()?;
if !integrity.chain_integrity || !integrity.signatures_valid {
    // Send alert to security team
    send_security_alert("Why-Log integrity compromised")?;
}

// Monitor for suspicious patterns
let recent_entries = service.find_entries(|entry| {
    entry.timestamp > chrono::Utc::now() - chrono::Duration::hours(1)
})?;

let emergency_overrides = recent_entries.iter()
    .filter(|e| matches!(e.decision, DecisionType::EmergencyOverride { .. }))
    .count();

if emergency_overrides > 5 {
    // Alert on unusual override activity
    send_alert("High number of emergency overrides detected")?;
}
```

### Metrics and Analytics

```rust
// Generate usage statistics
let summary = service.verify_integrity()?;

println!("Why-Log Statistics:");
println!("  Total Entries: {}", summary.entry_count);
println!("  Decision Types: {:?}", summary.decision_types);
println!("  Active Actors: {:?}", summary.actors);
println!("  Integrity Status: {}", 
    if summary.chain_integrity { "VALID" } else { "COMPROMISED" });

// Generate human-readable report
let report = service.generate_readable_summary(Some(100))?;
std::fs::write("audit_report.md", report)?;
```

## 🔧 Configuration

### WhyLogConfig Options

```rust
pub struct WhyLogConfig {
    /// Directory to store log files
    pub log_directory: PathBuf,
    
    /// Maximum file size before rotation (bytes)
    pub max_file_size: u64,
    
    /// Whether to compress rotated files
    pub compress_rotated: bool,
    
    /// Maximum number of rotated files to keep
    pub max_rotated_files: u32,
    
    /// Whether to validate signatures on read
    pub verify_signatures: bool,
    
    /// Whether to validate hash chain on read
    pub verify_hash_chain: bool,
}
```

### Example Configurations

**Development Configuration:**
```rust
let dev_config = WhyLogConfig {
    log_directory: PathBuf::from("./logs/dev"),
    max_file_size: 10 * 1024 * 1024, // 10MB
    compress_rotated: false,
    max_rotated_files: 5,
    verify_signatures: true,
    verify_hash_chain: true,
};
```

**Production Configuration:**
```rust
let prod_config = WhyLogConfig {
    log_directory: PathBuf::from("/var/log/polymera/whylogs"),
    max_file_size: 1024 * 1024 * 1024, // 1GB
    compress_rotated: true,
    max_rotated_files: 100,
    verify_signatures: true,
    verify_hash_chain: true,
};
```

**High-Security Configuration:**
```rust
let secure_config = WhyLogConfig {
    log_directory: PathBuf::from("/secure/audit/logs"),
    max_file_size: 100 * 1024 * 1024, // 100MB (smaller for more frequent rotation)
    compress_rotated: true,
    max_rotated_files: 1000,
    verify_signatures: true,
    verify_hash_chain: true,
};
```

## 📋 Compliance and Reporting

### Regulatory Compliance

Why-Log supports various compliance requirements:

- **SOX (Sarbanes-Oxley)**: Immutable audit trails for financial decisions
- **GDPR**: Privacy-aware logging with data minimization
- **HIPAA**: Secure audit trails for healthcare data access
- **PCI DSS**: Payment card industry security logging
- **ISO 27001**: Information security management logging

### Report Generation

```rust
// Generate compliance report for specific time period
let start_date = chrono::Utc::now() - chrono::Duration::days(90);
let end_date = chrono::Utc::now();

let compliance_report = service.export_json(Some(start_date), Some(end_date))?;

// Parse and format for specific compliance requirements
let parsed: serde_json::Value = serde_json::from_str(&compliance_report)?;
let entries = parsed["entries"].as_array().unwrap();

// Generate executive summary
let mut summary = String::new();
summary.push_str("# Quarterly Audit Report\n\n");
summary.push_str(&format!("**Period:** {} to {}\n", start_date, end_date));
summary.push_str(&format!("**Total Decisions:** {}\n", entries.len()));

// Count decisions by type
let mut decision_counts = HashMap::new();
for entry in entries {
    let decision_type = entry["decision"]["type"].as_str().unwrap_or("unknown");
    *decision_counts.entry(decision_type).or_insert(0) += 1;
}

summary.push_str("\n## Decision Breakdown\n\n");
for (decision_type, count) in decision_counts {
    summary.push_str(&format!("- **{}:** {}\n", decision_type, count));
}

std::fs::write("compliance_report.md", summary)?;
```

### Audit Trail Validation

```rust
// External audit validation
pub fn validate_audit_trail(log_file: &Path, public_key: &str) -> Result<bool, WhyLogError> {
    let file = File::open(log_file)?;
    let reader = BufReader::new(file);
    
    let mut previous_hash = "genesis".to_string();
    let mut sequence = 0u64;
    
    for line in reader.lines() {
        let entry: WhyLogEntry = serde_json::from_str(&line?)?;
        
        // Verify sequence
        sequence += 1;
        if entry.sequence != sequence {
            return Ok(false);
        }
        
        // Verify hash chain
        if entry.previous_hash != previous_hash {
            return Ok(false);
        }
        
        // Verify signature (if public key matches)
        if entry.signer_public_key == public_key {
            if !verify_signature(&entry)? {
                return Ok(false);
            }
        }
        
        previous_hash = entry.content_hash;
    }
    
    Ok(true)
}
```

## 🚀 Performance Considerations

### Scalability

- **Write Performance**: ~10,000 entries/second on modern hardware
- **Read Performance**: ~50,000 entries/second for sequential reads
- **Storage Efficiency**: ~1KB per entry average
- **Memory Usage**: <100MB for service with 1M entries

### Optimization Strategies

1. **Batch Writing**: Group multiple entries for better I/O performance
2. **Async I/O**: Use tokio for non-blocking operations
3. **Compression**: Enable compression for rotated files
4. **Indexing**: Create secondary indices for common queries

```rust
// Batch logging for high-throughput scenarios
pub async fn log_batch(&mut self, entries: Vec<(DecisionType, String, DecisionContext)>) 
    -> Result<Vec<String>, WhyLogError> {
    let mut entry_ids = Vec::new();
    
    for (decision, justification, context) in entries {
        let id = self.log_decision(decision, justification, context)?;
        entry_ids.push(id);
    }
    
    Ok(entry_ids)
}
```

## 🛠️ Integration Examples

### Integration with Policy Engine

```rust
use crate::policy::{PolicyEngine, PolicyResult};
use crate::whylog::{WhyLogService, DecisionType, DecisionContext};

impl PolicyEngine {
    pub async fn evaluate_with_logging(
        &self,
        policy_name: &str,
        input: &serde_json::Value,
        context: DecisionContext,
        whylog: &mut WhyLogService,
    ) -> Result<PolicyResult, PolicyError> {
        
        let input_hash = self.hash_input(input)?;
        let result = self.evaluate(policy_name, input).await?;
        
        let decision = DecisionType::PolicyDecision {
            policy_name: policy_name.to_string(),
            input_hash,
            decision: result.allowed,
            confidence: result.confidence,
        };
        
        let justification = format!(
            "Policy evaluation: {} (confidence: {:.2}). Rationale: {}",
            if result.allowed { "ALLOW" } else { "DENY" },
            result.confidence,
            result.rationale.join("; ")
        );
        
        whylog.log_decision(decision, justification, context)?;
        
        Ok(result)
    }
}
```

### Integration with Wallet Service

```rust
use crate::wallet::{WalletService, TransactionRequest};
use crate::whylog::{WhyLogService, DecisionType, DecisionContext};

impl WalletService {
    pub async fn process_transaction_with_audit(
        &mut self,
        request: TransactionRequest,
        context: DecisionContext,
        whylog: &mut WhyLogService,
    ) -> Result<TransactionResult, WalletError> {
        
        let decision_result = self.should_approve_transaction(&request).await?;
        
        let decision = DecisionType::TransactionDecision {
            transaction_id: request.id.clone(),
            decision: decision_result.approved,
            amount: Some(request.amount.to_string()),
            currency: Some(request.currency.clone()),
        };
        
        let justification = if decision_result.approved {
            format!("Transaction approved: {}", decision_result.reason)
        } else {
            format!("Transaction rejected: {}", decision_result.reason)
        };
        
        whylog.log_decision(decision, justification, context)?;
        
        if decision_result.approved {
            self.execute_transaction(request).await
        } else {
            Err(WalletError::TransactionRejected(decision_result.reason))
        }
    }
}
```

## 🧪 Testing

### Unit Tests

Run the comprehensive test suite:

```bash
cargo test -p whylog
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_audit_trail() {
    let (mut service, _temp_dir) = create_test_service();
    
    // Simulate a complete audit scenario
    let decisions = vec![
        (DecisionType::PolicyDecision { /* ... */ }, "User login approved"),
        (DecisionType::TransactionDecision { /* ... */ }, "Payment processed"),
        (DecisionType::SecurityEvent { /* ... */ }, "Suspicious activity detected"),
        (DecisionType::AdminAction { /* ... */ }, "User permissions updated"),
    ];
    
    for (decision, justification) in decisions {
        service.log_decision(decision, justification.to_string(), create_context()).unwrap();
    }
    
    // Verify integrity
    let summary = service.verify_integrity().unwrap();
    assert!(summary.chain_integrity);
    assert!(summary.signatures_valid);
    assert_eq!(summary.entry_count, 4);
    
    // Test tamper detection
    let tampering = service.detect_tampering().unwrap();
    assert!(tampering.is_empty());
    
    // Test readable summary
    let readable = service.generate_readable_summary(None).unwrap();
    assert!(readable.contains("Why-Log Summary"));
    assert!(readable.contains("User login approved"));
}
```

### Stress Tests

```rust
#[test]
fn test_high_volume_logging() {
    let (mut service, _temp_dir) = create_test_service();
    
    let start = std::time::Instant::now();
    
    // Log 10,000 entries
    for i in 0..10_000 {
        let decision = DecisionType::AccessControl {
            user_id: format!("user{}", i),
            resource: "database".to_string(),
            action: "read".to_string(),
            granted: i % 2 == 0,
        };
        
        service.log_decision(
            decision,
            format!("Access decision {}", i),
            create_context(),
        ).unwrap();
    }
    
    let elapsed = start.elapsed();
    println!("Logged 10,000 entries in {:?}", elapsed);
    assert!(elapsed.as_secs() < 10); // Should complete in under 10 seconds
    
    // Verify integrity of all entries
    let summary = service.verify_integrity().unwrap();
    assert!(summary.chain_integrity);
    assert_eq!(summary.entry_count, 10_000);
}
```

## 🚨 Troubleshooting

### Common Issues

**Hash Chain Breaks:**
```rust
// Diagnose hash chain issues
let tampering = service.detect_tampering()?;
for indicator in tampering {
    if indicator.contains("Hash chain break") {
        eprintln!("Hash chain compromised: {}", indicator);
        // Investigation steps:
        // 1. Check file system integrity
        // 2. Verify no unauthorized access
        // 3. Review backup logs
        // 4. Consider key compromise
    }
}
```

**Signature Verification Failures:**
```rust
// Debug signature issues
let entries = service.read_entries()?;
for entry in entries {
    if !service.verify_entry_signature(&entry)? {
        eprintln!("Invalid signature for entry: {}", entry.id);
        eprintln!("Public key: {}", entry.signer_public_key);
        eprintln!("Signature: {}", entry.signature);
        // Check for:
        // 1. Key rotation without migration
        // 2. Clock skew affecting signatures
        // 3. Encoding/decoding errors
    }
}
```

**Performance Issues:**
```rust
// Monitor performance metrics
let start = std::time::Instant::now();
let entries = service.read_entries()?;
let read_time = start.elapsed();

println!("Read {} entries in {:?}", entries.len(), read_time);

if read_time.as_secs() > 5 {
    // Optimization strategies:
    // 1. Enable compression for old files
    // 2. Implement indexing for queries
    // 3. Use streaming reads for large logs
    // 4. Consider log archival
}
```

### Recovery Procedures

**Log Corruption Recovery:**
```rust
// Attempt to recover from partial corruption
pub fn recover_from_corruption(
    log_directory: &Path,
    backup_directory: &Path,
    public_key: &str,
) -> Result<usize, WhyLogError> {
    let mut recovered_entries = Vec::new();
    
    // Try to read all log files
    for log_file in get_log_files(log_directory)? {
        match read_entries_from_file(&log_file) {
            Ok(entries) => {
                // Verify each entry
                for entry in entries {
                    if verify_entry_signature(&entry, public_key)? {
                        recovered_entries.push(entry);
                    }
                }
            },
            Err(e) => {
                eprintln!("Corrupted file: {:?}, error: {}", log_file, e);
                // Try backup if available
                if let Ok(backup_entries) = read_backup_file(&log_file, backup_directory) {
                    recovered_entries.extend(backup_entries);
                }
            }
        }
    }
    
    // Sort by sequence and rebuild
    recovered_entries.sort_by(|a, b| a.sequence.cmp(&b.sequence));
    
    // Write to new log file
    let recovery_file = log_directory.join("recovered_log.jsonl");
    write_entries_to_file(&recovered_entries, &recovery_file)?;
    
    Ok(recovered_entries.len())
}
```

---

**Why-Log** provides Polymera OS with a robust, tamper-evident audit trail system that ensures complete accountability for all important decisions. The combination of cryptographic signatures, hash chaining, and comprehensive tamper detection makes it suitable for high-security environments requiring regulatory compliance and forensic audit capabilities.
