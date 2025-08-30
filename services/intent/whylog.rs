//! Why-Log Service
//!
//! An append-only audit trail system that records signed justifications for important decisions.
//! Each log entry is cryptographically signed and hash-chained to prevent tampering and ensure
//! complete audit trails for compliance and accountability.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use chrono::{DateTime, Utc};
use sha3::{Digest, Sha3_256};
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use hex;
use uuid::Uuid;

/// Errors that can occur during Why-Log operations
#[derive(Debug, Error)]
pub enum WhyLogError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Hash chain broken at entry {0}")]
    BrokenHashChain(u64),

    #[error("Log corruption detected: {0}")]
    LogCorruption(String),

    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("Invalid log format: {0}")]
    InvalidFormat(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),
}

/// Types of decisions that can be logged
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DecisionType {
    /// Policy decision (allow/deny)
    PolicyDecision {
        policy_name: String,
        input_hash: String,
        decision: bool,
        confidence: f64,
    },
    /// Security event
    SecurityEvent {
        event_type: String,
        severity: String,
        source: String,
    },
    /// Administrative action
    AdminAction {
        action: String,
        target: String,
        permissions: Vec<String>,
    },
    /// System configuration change
    ConfigChange {
        component: String,
        field: String,
        old_value: Option<String>,
        new_value: String,
    },
    /// Transaction approval/rejection
    TransactionDecision {
        transaction_id: String,
        decision: bool,
        amount: Option<String>,
        currency: Option<String>,
    },
    /// Access control decision
    AccessControl {
        user_id: String,
        resource: String,
        action: String,
        granted: bool,
    },
    /// Emergency override
    EmergencyOverride {
        original_decision: String,
        override_reason: String,
        admin_id: String,
    },
}

/// Context information for a decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    /// User or system making the decision
    pub actor: String,
    /// IP address or source location
    pub source: Option<String>,
    /// User agent or client information
    pub client_info: Option<String>,
    /// Session or request ID
    pub session_id: Option<String>,
    /// Additional contextual metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A single Why-Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhyLogEntry {
    /// Unique entry ID
    pub id: String,
    /// Sequential entry number
    pub sequence: u64,
    /// Timestamp when decision was made
    pub timestamp: DateTime<Utc>,
    /// Type and details of the decision
    pub decision: DecisionType,
    /// Human-readable justification
    pub justification: String,
    /// Context information
    pub context: DecisionContext,
    /// Hash of the previous entry (for chaining)
    pub previous_hash: String,
    /// Hash of this entry's content
    pub content_hash: String,
    /// Cryptographic signature of the entry
    pub signature: String,
    /// Public key used for signing
    pub signer_public_key: String,
}

/// Summary statistics for a Why-Log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhyLogSummary {
    /// Total number of entries
    pub entry_count: u64,
    /// First entry timestamp
    pub first_entry: Option<DateTime<Utc>>,
    /// Last entry timestamp
    pub last_entry: Option<DateTime<Utc>>,
    /// Entries by decision type
    pub decision_types: HashMap<String, u64>,
    /// Entries by actor
    pub actors: HashMap<String, u64>,
    /// Hash chain integrity status
    pub chain_integrity: bool,
    /// Signature verification status
    pub signatures_valid: bool,
}

/// Configuration for Why-Log service
#[derive(Debug, Clone)]
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

impl Default for WhyLogConfig {
    fn default() -> Self {
        Self {
            log_directory: PathBuf::from("data/whylogs"),
            max_file_size: 100 * 1024 * 1024, // 100MB
            compress_rotated: true,
            max_rotated_files: 10,
            verify_signatures: true,
            verify_hash_chain: true,
        }
    }
}

/// Why-Log service for recording and verifying decision audit trails
pub struct WhyLogService {
    config: WhyLogConfig,
    keypair: Keypair,
    current_file: Option<File>,
    current_sequence: u64,
    last_hash: String,
}

impl WhyLogService {
    /// Create a new Why-Log service
    pub fn new(config: WhyLogConfig, keypair: Keypair) -> Result<Self, WhyLogError> {
        // Ensure log directory exists
        std::fs::create_dir_all(&config.log_directory)?;

        let mut service = Self {
            config,
            keypair,
            current_file: None,
            current_sequence: 0,
            last_hash: "genesis".to_string(),
        };

        // Initialize from existing logs
        service.initialize_from_existing_logs()?;

        Ok(service)
    }

    /// Create a new Why-Log service with a generated keypair
    pub fn new_with_generated_key(config: WhyLogConfig) -> Result<Self, WhyLogError> {
        let mut csprng = rand::rngs::OsRng;
        let keypair = Keypair::generate(&mut csprng);
        Self::new(config, keypair)
    }

    /// Log a decision with justification
    pub fn log_decision(
        &mut self,
        decision: DecisionType,
        justification: String,
        context: DecisionContext,
    ) -> Result<String, WhyLogError> {
        let entry_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // Create entry
        let mut entry = WhyLogEntry {
            id: entry_id.clone(),
            sequence: self.current_sequence + 1,
            timestamp,
            decision,
            justification,
            context,
            previous_hash: self.last_hash.clone(),
            content_hash: String::new(), // Will be calculated
            signature: String::new(),    // Will be calculated
            signer_public_key: hex::encode(self.keypair.public.to_bytes()),
        };

        // Calculate content hash
        entry.content_hash = self.calculate_entry_hash(&entry)?;

        // Sign the entry
        entry.signature = self.sign_entry(&entry)?;

        // Write entry to file
        self.write_entry(&entry)?;

        // Update state
        self.current_sequence = entry.sequence;
        self.last_hash = entry.content_hash.clone();

        Ok(entry_id)
    }

    /// Read all entries from the log
    pub fn read_entries(&self) -> Result<Vec<WhyLogEntry>, WhyLogError> {
        let mut entries = Vec::new();
        
        for log_file in self.get_log_files()? {
            let file = File::open(&log_file)?;
            let reader = BufReader::new(file);
            
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                
                let entry: WhyLogEntry = serde_json::from_str(&line)?;
                entries.push(entry);
            }
        }

        // Sort by sequence number
        entries.sort_by(|a, b| a.sequence.cmp(&b.sequence));

        Ok(entries)
    }

    /// Find entries by criteria
    pub fn find_entries<F>(&self, predicate: F) -> Result<Vec<WhyLogEntry>, WhyLogError>
    where
        F: Fn(&WhyLogEntry) -> bool,
    {
        let entries = self.read_entries()?;
        Ok(entries.into_iter().filter(predicate).collect())
    }

    /// Get entry by ID
    pub fn get_entry(&self, entry_id: &str) -> Result<WhyLogEntry, WhyLogError> {
        let entries = self.find_entries(|entry| entry.id == entry_id)?;
        entries.into_iter().next()
            .ok_or_else(|| WhyLogError::EntryNotFound(entry_id.to_string()))
    }

    /// Verify the integrity of the entire log
    pub fn verify_integrity(&self) -> Result<WhyLogSummary, WhyLogError> {
        let entries = self.read_entries()?;
        
        if entries.is_empty() {
            return Ok(WhyLogSummary {
                entry_count: 0,
                first_entry: None,
                last_entry: None,
                decision_types: HashMap::new(),
                actors: HashMap::new(),
                chain_integrity: true,
                signatures_valid: true,
            });
        }

        let mut summary = WhyLogSummary {
            entry_count: entries.len() as u64,
            first_entry: Some(entries.first().unwrap().timestamp),
            last_entry: Some(entries.last().unwrap().timestamp),
            decision_types: HashMap::new(),
            actors: HashMap::new(),
            chain_integrity: true,
            signatures_valid: true,
        };

        // Verify hash chain
        let mut expected_previous_hash = "genesis".to_string();
        for entry in &entries {
            if entry.previous_hash != expected_previous_hash {
                summary.chain_integrity = false;
                break;
            }
            
            // Verify content hash
            let calculated_hash = self.calculate_entry_hash(entry)?;
            if calculated_hash != entry.content_hash {
                summary.chain_integrity = false;
                break;
            }
            
            expected_previous_hash = entry.content_hash.clone();
        }

        // Verify signatures
        if self.config.verify_signatures {
            for entry in &entries {
                if !self.verify_entry_signature(entry)? {
                    summary.signatures_valid = false;
                    break;
                }
            }
        }

        // Generate statistics
        for entry in &entries {
            // Count decision types
            let decision_type = match &entry.decision {
                DecisionType::PolicyDecision { .. } => "policy_decision",
                DecisionType::SecurityEvent { .. } => "security_event",
                DecisionType::AdminAction { .. } => "admin_action",
                DecisionType::ConfigChange { .. } => "config_change",
                DecisionType::TransactionDecision { .. } => "transaction_decision",
                DecisionType::AccessControl { .. } => "access_control",
                DecisionType::EmergencyOverride { .. } => "emergency_override",
            };
            *summary.decision_types.entry(decision_type.to_string()).or_insert(0) += 1;

            // Count actors
            *summary.actors.entry(entry.context.actor.clone()).or_insert(0) += 1;
        }

        Ok(summary)
    }

    /// Generate a human-readable summary of recent decisions
    pub fn generate_readable_summary(&self, limit: Option<usize>) -> Result<String, WhyLogError> {
        let entries = self.read_entries()?;
        let recent_entries = if let Some(limit) = limit {
            entries.into_iter().rev().take(limit).collect::<Vec<_>>()
        } else {
            entries
        };

        let mut summary = String::new();
        summary.push_str("# Why-Log Summary\n\n");

        if recent_entries.is_empty() {
            summary.push_str("No entries found.\n");
            return Ok(summary);
        }

        summary.push_str(&format!("## Recent Decisions ({} entries)\n\n", recent_entries.len()));

        for entry in recent_entries.iter().rev() {
            summary.push_str(&format!(
                "### Entry {} - {}\n",
                entry.sequence,
                entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
            ));

            summary.push_str(&format!("**Actor:** {}\n", entry.context.actor));
            summary.push_str(&format!("**Decision:** {}\n", self.format_decision(&entry.decision)));
            summary.push_str(&format!("**Justification:** {}\n", entry.justification));

            if let Some(source) = &entry.context.source {
                summary.push_str(&format!("**Source:** {}\n", source));
            }

            if let Some(session_id) = &entry.context.session_id {
                summary.push_str(&format!("**Session:** {}\n", session_id));
            }

            summary.push_str("\n---\n\n");
        }

        // Add integrity status
        let integrity = self.verify_integrity()?;
        summary.push_str("## Integrity Status\n\n");
        summary.push_str(&format!("- **Chain Integrity:** {}\n", 
            if integrity.chain_integrity { "✅ Valid" } else { "❌ Broken" }));
        summary.push_str(&format!("- **Signatures Valid:** {}\n", 
            if integrity.signatures_valid { "✅ Valid" } else { "❌ Invalid" }));
        summary.push_str(&format!("- **Total Entries:** {}\n", integrity.entry_count));

        Ok(summary)
    }

    /// Export entries to JSON format
    pub fn export_json(&self, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Result<String, WhyLogError> {
        let mut entries = self.read_entries()?;

        // Filter by date range if specified
        if let Some(start) = start_date {
            entries.retain(|e| e.timestamp >= start);
        }
        if let Some(end) = end_date {
            entries.retain(|e| e.timestamp <= end);
        }

        let export_data = serde_json::json!({
            "export_timestamp": Utc::now(),
            "entry_count": entries.len(),
            "entries": entries
        });

        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    /// Import entries from JSON format (for testing/migration)
    pub fn import_json(&mut self, json_data: &str) -> Result<usize, WhyLogError> {
        let import_data: serde_json::Value = serde_json::from_str(json_data)?;
        let entries: Vec<WhyLogEntry> = serde_json::from_value(
            import_data.get("entries").ok_or_else(|| 
                WhyLogError::InvalidFormat("Missing 'entries' field".to_string()))?
                .clone()
        )?;

        let mut imported_count = 0;
        for entry in entries {
            // Verify entry integrity before importing
            if self.verify_entry_signature(&entry)? {
                self.write_entry(&entry)?;
                imported_count += 1;
            }
        }

        // Reinitialize state
        self.initialize_from_existing_logs()?;

        Ok(imported_count)
    }

    /// Detect tamper attempts by analyzing log inconsistencies
    pub fn detect_tampering(&self) -> Result<Vec<String>, WhyLogError> {
        let mut tampering_indicators = Vec::new();
        let entries = self.read_entries()?;

        if entries.is_empty() {
            return Ok(tampering_indicators);
        }

        // Check for sequence gaps
        for (i, entry) in entries.iter().enumerate() {
            let expected_sequence = i as u64 + 1;
            if entry.sequence != expected_sequence {
                tampering_indicators.push(format!(
                    "Sequence gap: expected {}, found {} at entry {}",
                    expected_sequence, entry.sequence, entry.id
                ));
            }
        }

        // Check hash chain continuity
        let mut expected_previous_hash = "genesis".to_string();
        for entry in &entries {
            if entry.previous_hash != expected_previous_hash {
                tampering_indicators.push(format!(
                    "Hash chain break: entry {} has previous_hash {}, expected {}",
                    entry.id, entry.previous_hash, expected_previous_hash
                ));
            }

            // Verify content hash
            let calculated_hash = self.calculate_entry_hash(entry)?;
            if calculated_hash != entry.content_hash {
                tampering_indicators.push(format!(
                    "Content hash mismatch: entry {} has invalid content hash",
                    entry.id
                ));
            }

            expected_previous_hash = entry.content_hash.clone();
        }

        // Check for signature tampering
        for entry in &entries {
            if !self.verify_entry_signature(entry)? {
                tampering_indicators.push(format!(
                    "Invalid signature: entry {} has corrupted or invalid signature",
                    entry.id
                ));
            }
        }

        // Check for timestamp anomalies
        for window in entries.windows(2) {
            if window[1].timestamp < window[0].timestamp {
                tampering_indicators.push(format!(
                    "Timestamp anomaly: entry {} has earlier timestamp than previous entry {}",
                    window[1].id, window[0].id
                ));
            }
        }

        Ok(tampering_indicators)
    }

    /// Private helper methods
    fn initialize_from_existing_logs(&mut self) -> Result<(), WhyLogError> {
        let entries = self.read_entries()?;
        
        if let Some(last_entry) = entries.last() {
            self.current_sequence = last_entry.sequence;
            self.last_hash = last_entry.content_hash.clone();
        } else {
            self.current_sequence = 0;
            self.last_hash = "genesis".to_string();
        }

        Ok(())
    }

    fn get_current_log_file(&mut self) -> Result<&mut File, WhyLogError> {
        if self.current_file.is_none() || self.should_rotate_file()? {
            self.rotate_log_file()?;
        }

        Ok(self.current_file.as_mut().unwrap())
    }

    fn should_rotate_file(&self) -> Result<bool, WhyLogError> {
        if let Some(ref file) = self.current_file {
            let metadata = file.metadata()?;
            Ok(metadata.len() >= self.config.max_file_size)
        } else {
            Ok(true)
        }
    }

    fn rotate_log_file(&mut self) -> Result<(), WhyLogError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("whylog_{}.jsonl", timestamp);
        let filepath = self.config.log_directory.join(filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filepath)?;

        self.current_file = Some(file);
        Ok(())
    }

    fn write_entry(&mut self, entry: &WhyLogEntry) -> Result<(), WhyLogError> {
        let file = self.get_current_log_file()?;
        let mut writer = BufWriter::new(file);
        
        let json_line = serde_json::to_string(entry)?;
        writeln!(writer, "{}", json_line)?;
        writer.flush()?;

        Ok(())
    }

    fn calculate_entry_hash(&self, entry: &WhyLogEntry) -> Result<String, WhyLogError> {
        let content = serde_json::json!({
            "id": entry.id,
            "sequence": entry.sequence,
            "timestamp": entry.timestamp,
            "decision": entry.decision,
            "justification": entry.justification,
            "context": entry.context,
            "previous_hash": entry.previous_hash,
            "signer_public_key": entry.signer_public_key
        });

        let content_bytes = serde_json::to_vec(&content)?;
        let mut hasher = Sha3_256::new();
        hasher.update(&content_bytes);
        let hash = hasher.finalize();

        Ok(hex::encode(hash))
    }

    fn sign_entry(&self, entry: &WhyLogEntry) -> Result<String, WhyLogError> {
        let content_bytes = entry.content_hash.as_bytes();
        let signature = self.keypair.sign(content_bytes);
        Ok(hex::encode(signature.to_bytes()))
    }

    fn verify_entry_signature(&self, entry: &WhyLogEntry) -> Result<bool, WhyLogError> {
        // Decode public key
        let public_key_bytes = hex::decode(&entry.signer_public_key)
            .map_err(|e| WhyLogError::Crypto(format!("Invalid public key hex: {}", e)))?;
        
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| WhyLogError::Crypto(format!("Invalid public key: {}", e)))?;

        // Decode signature
        let signature_bytes = hex::decode(&entry.signature)
            .map_err(|e| WhyLogError::Crypto(format!("Invalid signature hex: {}", e)))?;
        
        let signature = Signature::from_bytes(&signature_bytes)
            .map_err(|e| WhyLogError::Crypto(format!("Invalid signature: {}", e)))?;

        // Verify signature
        let content_bytes = entry.content_hash.as_bytes();
        Ok(public_key.verify(content_bytes, &signature).is_ok())
    }

    fn get_log_files(&self) -> Result<Vec<PathBuf>, WhyLogError> {
        let mut files = Vec::new();
        
        for entry in std::fs::read_dir(&self.config.log_directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("whylog_") && filename.ends_with(".jsonl") {
                        files.push(path);
                    }
                }
            }
        }

        // Sort by filename (which includes timestamp)
        files.sort();
        Ok(files)
    }

    fn format_decision(&self, decision: &DecisionType) -> String {
        match decision {
            DecisionType::PolicyDecision { policy_name, decision, confidence, .. } => {
                format!("Policy '{}' {} (confidence: {:.2})", 
                    policy_name, 
                    if *decision { "ALLOW" } else { "DENY" }, 
                    confidence)
            },
            DecisionType::SecurityEvent { event_type, severity, .. } => {
                format!("Security Event: {} ({})", event_type, severity)
            },
            DecisionType::AdminAction { action, target, .. } => {
                format!("Admin Action: {} on {}", action, target)
            },
            DecisionType::ConfigChange { component, field, new_value, .. } => {
                format!("Config Change: {}.{} = {}", component, field, new_value)
            },
            DecisionType::TransactionDecision { transaction_id, decision, amount, currency } => {
                let amount_str = if let (Some(amt), Some(cur)) = (amount, currency) {
                    format!(" ({}{})", amt, cur)
                } else { String::new() };
                format!("Transaction {} {}{}", 
                    transaction_id, 
                    if *decision { "APPROVED" } else { "REJECTED" },
                    amount_str)
            },
            DecisionType::AccessControl { user_id, resource, action, granted } => {
                format!("Access Control: {} {} on {} - {}", 
                    user_id, action, resource,
                    if *granted { "GRANTED" } else { "DENIED" })
            },
            DecisionType::EmergencyOverride { original_decision, admin_id, .. } => {
                format!("Emergency Override by {}: {}", admin_id, original_decision)
            },
        }
    }

    /// Get the public key for this service (for external verification)
    pub fn public_key(&self) -> String {
        hex::encode(self.keypair.public.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_service() -> (WhyLogService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = WhyLogConfig {
            log_directory: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let service = WhyLogService::new_with_generated_key(config).unwrap();
        (service, temp_dir)
    }

    #[test]
    fn test_log_and_read_decision() {
        let (mut service, _temp_dir) = create_test_service();

        let decision = DecisionType::PolicyDecision {
            policy_name: "wallet_spend_limits".to_string(),
            input_hash: "abc123".to_string(),
            decision: true,
            confidence: 0.95,
        };

        let context = DecisionContext {
            actor: "user123".to_string(),
            source: Some("192.168.1.10".to_string()),
            client_info: Some("mobile_app".to_string()),
            session_id: Some("sess_456".to_string()),
            metadata: HashMap::new(),
        };

        let entry_id = service.log_decision(
            decision,
            "Transaction within daily limits and low risk score".to_string(),
            context
        ).unwrap();

        let entries = service.read_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, entry_id);
        assert_eq!(entries[0].sequence, 1);
        assert_eq!(entries[0].justification, "Transaction within daily limits and low risk score");
    }

    #[test]
    fn test_hash_chain_integrity() {
        let (mut service, _temp_dir) = create_test_service();

        // Log multiple decisions
        for i in 0..3 {
            let decision = DecisionType::TransactionDecision {
                transaction_id: format!("tx_{}", i),
                decision: i % 2 == 0,
                amount: Some("100.0".to_string()),
                currency: Some("USD".to_string()),
            };

            let context = DecisionContext {
                actor: format!("user{}", i),
                source: None,
                client_info: None,
                session_id: None,
                metadata: HashMap::new(),
            };

            service.log_decision(
                decision,
                format!("Decision for transaction {}", i),
                context
            ).unwrap();
        }

        // Verify integrity
        let summary = service.verify_integrity().unwrap();
        assert!(summary.chain_integrity);
        assert!(summary.signatures_valid);
        assert_eq!(summary.entry_count, 3);
    }

    #[test]
    fn test_tamper_detection() {
        let (mut service, _temp_dir) = create_test_service();

        // Log a decision
        let decision = DecisionType::SecurityEvent {
            event_type: "login_attempt".to_string(),
            severity: "medium".to_string(),
            source: "external".to_string(),
        };

        let context = DecisionContext {
            actor: "security_system".to_string(),
            source: Some("10.0.0.1".to_string()),
            client_info: None,
            session_id: None,
            metadata: HashMap::new(),
        };

        service.log_decision(
            decision,
            "Suspicious login attempt detected".to_string(),
            context
        ).unwrap();

        // Initially no tampering
        let tampering = service.detect_tampering().unwrap();
        assert!(tampering.is_empty());

        // Note: In a real scenario, we would need to externally modify files to test tampering detection
        // For now, we verify the detection method works with valid data
    }

    #[test]
    fn test_readable_summary() {
        let (mut service, _temp_dir) = create_test_service();

        let decision = DecisionType::AdminAction {
            action: "create_user".to_string(),
            target: "user789".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        let context = DecisionContext {
            actor: "admin123".to_string(),
            source: Some("admin.example.com".to_string()),
            client_info: Some("admin_panel".to_string()),
            session_id: Some("admin_sess_001".to_string()),
            metadata: HashMap::new(),
        };

        service.log_decision(
            decision,
            "Creating new user account as requested by department manager".to_string(),
            context
        ).unwrap();

        let summary = service.generate_readable_summary(None).unwrap();
        assert!(summary.contains("Why-Log Summary"));
        assert!(summary.contains("admin123"));
        assert!(summary.contains("Creating new user account"));
        assert!(summary.contains("Chain Integrity"));
    }

    #[test]
    fn test_find_entries() {
        let (mut service, _temp_dir) = create_test_service();

        // Log decisions with different actors
        for i in 0..5 {
            let decision = DecisionType::AccessControl {
                user_id: format!("user{}", i),
                resource: "database".to_string(),
                action: "read".to_string(),
                granted: i % 2 == 0,
            };

            let context = DecisionContext {
                actor: if i < 3 { "system".to_string() } else { "admin".to_string() },
                source: None,
                client_info: None,
                session_id: None,
                metadata: HashMap::new(),
            };

            service.log_decision(
                decision,
                format!("Access control decision for user{}", i),
                context
            ).unwrap();
        }

        // Find entries by system actor
        let system_entries = service.find_entries(|e| e.context.actor == "system").unwrap();
        assert_eq!(system_entries.len(), 3);

        // Find entries where access was granted
        let granted_entries = service.find_entries(|e| {
            if let DecisionType::AccessControl { granted, .. } = &e.decision {
                *granted
            } else {
                false
            }
        }).unwrap();
        assert_eq!(granted_entries.len(), 3); // 0, 2, 4
    }

    #[test]
    fn test_export_import_json() {
        let (mut service, _temp_dir) = create_test_service();

        // Log some decisions
        for i in 0..3 {
            let decision = DecisionType::ConfigChange {
                component: "security".to_string(),
                field: format!("setting_{}", i),
                old_value: Some(format!("old_{}", i)),
                new_value: format!("new_{}", i),
            };

            let context = DecisionContext {
                actor: "config_manager".to_string(),
                source: None,
                client_info: None,
                session_id: None,
                metadata: HashMap::new(),
            };

            service.log_decision(
                decision,
                format!("Configuration update {}", i),
                context
            ).unwrap();
        }

        // Export to JSON
        let json_export = service.export_json(None, None).unwrap();
        assert!(json_export.contains("entry_count"));
        assert!(json_export.contains("config_manager"));

        // Create new service and import
        let (mut new_service, _temp_dir2) = create_test_service();
        let imported_count = new_service.import_json(&json_export).unwrap();
        assert_eq!(imported_count, 3);

        // Verify imported entries
        let entries = new_service.read_entries().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].context.actor, "config_manager");
    }
}
