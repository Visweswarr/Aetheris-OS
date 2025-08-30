//! Quarantine System for Polymera OS Network
//!
//! Provides peer quarantine management with local denylist, signed entries,
//! automatic expiry, and comprehensive security enforcement.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn, error};
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use sha2::{Sha256, Digest};
use hex;
use uuid::Uuid;

/// Errors that can occur during quarantine operations
#[derive(Debug, Error)]
pub enum QuarantineError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Peer already quarantined: {0}")]
    PeerAlreadyQuarantined(String),

    #[error("Peer not found in quarantine: {0}")]
    PeerNotFound(String),

    #[error("Invalid quarantine entry: {0}")]
    InvalidEntry(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("File format error: {0}")]
    FileFormat(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Expired entry: {0}")]
    ExpiredEntry(String),
}

/// Result type for quarantine operations
pub type QuarantineResult<T> = Result<T, QuarantineError>;

/// Reasons for quarantining a peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineReason {
    /// Malicious behavior detected
    MaliciousBehavior,
    /// Protocol violations
    ProtocolViolation,
    /// Spam or flooding
    Spam,
    /// Invalid or corrupted data
    CorruptedData,
    /// Authentication failures
    AuthenticationFailure,
    /// Rate limiting violations
    RateLimitViolation,
    /// Consensus violations
    ConsensusViolation,
    /// Known bad actor
    KnownBadActor,
    /// Manual administrative action
    Manual,
    /// Custom reason
    Custom(String),
}

/// Severity levels for quarantine entries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineSeverity {
    Low,        // Temporary issue, short quarantine
    Medium,     // Moderate violation, standard quarantine
    High,       // Serious violation, extended quarantine
    Critical,   // Severe violation, long-term quarantine
    Permanent,  // Permanent ban
}

/// Quarantine entry representing a banned peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineEntry {
    /// Unique entry ID
    pub id: String,
    /// Peer identifier (multiaddr, peer ID, etc.)
    pub peer_id: String,
    /// Reason for quarantine
    pub reason: QuarantineReason,
    /// Severity level
    pub severity: QuarantineSeverity,
    /// When the quarantine was imposed
    pub quarantined_at: SystemTime,
    /// When the quarantine expires (None for permanent)
    pub expires_at: Option<SystemTime>,
    /// Who imposed the quarantine
    pub imposed_by: String,
    /// Additional evidence or description
    pub evidence: Option<String>,
    /// Metadata for the quarantine
    pub metadata: HashMap<String, String>,
    /// Digital signature of the entry
    pub signature: String,
    /// Public key used for signing
    pub signer_public_key: String,
}

/// Quarantine list containing all quarantined peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineList {
    /// Format version
    pub version: String,
    /// When the list was last updated
    pub last_updated: SystemTime,
    /// Authority that maintains this list
    pub authority: String,
    /// Public key of the authority
    pub authority_public_key: String,
    /// Quarantine entries
    pub entries: Vec<QuarantineEntry>,
    /// List signature
    pub signature: String,
}

/// Configuration for quarantine system
#[derive(Debug, Clone)]
pub struct QuarantineConfig {
    /// Path to quarantine data file
    pub data_file: PathBuf,
    /// Path to backup directory
    pub backup_dir: PathBuf,
    /// Signing keypair for local authority
    pub signing_keypair: Keypair,
    /// Trusted authority public keys
    pub trusted_authorities: HashSet<String>,
    /// Default quarantine duration by severity
    pub default_durations: HashMap<QuarantineSeverity, Duration>,
    /// Maximum entries to keep in memory
    pub max_entries: usize,
    /// Cleanup interval
    pub cleanup_interval: Duration,
    /// Backup retention count
    pub backup_retention: u32,
    /// Enable signature verification
    pub verify_signatures: bool,
}

/// Statistics about quarantine system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineStats {
    /// Total entries in quarantine
    pub total_entries: usize,
    /// Active (non-expired) entries
    pub active_entries: usize,
    /// Expired entries pending cleanup
    pub expired_entries: usize,
    /// Entries by reason
    pub entries_by_reason: HashMap<String, usize>,
    /// Entries by severity
    pub entries_by_severity: HashMap<String, usize>,
    /// Last cleanup time
    pub last_cleanup: Option<SystemTime>,
    /// File size
    pub file_size: u64,
}

/// Quarantine system implementation
#[derive(Debug)]
pub struct QuarantineSystem {
    /// Configuration
    config: QuarantineConfig,
    /// In-memory quarantine list
    quarantine_list: Arc<RwLock<QuarantineList>>,
    /// Peer lookup cache (peer_id -> entry_id)
    peer_cache: Arc<RwLock<HashMap<String, String>>>,
    /// Authority info
    authority_name: String,
    /// Last file modification time
    last_file_mtime: Arc<RwLock<Option<SystemTime>>>,
}

impl QuarantineSystem {
    /// Create a new quarantine system
    pub fn new(config: QuarantineConfig, authority_name: String) -> QuarantineResult<Self> {
        let authority_public_key = hex::encode(config.signing_keypair.public.to_bytes());
        
        let initial_list = QuarantineList {
            version: "1.0".to_string(),
            last_updated: SystemTime::now(),
            authority: authority_name.clone(),
            authority_public_key,
            entries: Vec::new(),
            signature: String::new(),
        };

        let system = Self {
            config,
            quarantine_list: Arc::new(RwLock::new(initial_list)),
            peer_cache: Arc::new(RwLock::new(HashMap::new())),
            authority_name,
            last_file_mtime: Arc::new(RwLock::new(None)),
        };

        // Load existing quarantine data if available
        if system.config.data_file.exists() {
            system.load_from_file()?;
        } else {
            // Create initial empty file
            system.save_to_file()?;
        }

        Ok(system)
    }

    /// Check if a peer is quarantined
    pub fn is_quarantined(&self, peer_id: &str) -> QuarantineResult<bool> {
        let cache = self.peer_cache.read().unwrap();
        
        if let Some(entry_id) = cache.get(peer_id) {
            let list = self.quarantine_list.read().unwrap();
            if let Some(entry) = list.entries.iter().find(|e| e.id == *entry_id) {
                // Check if entry has expired
                if let Some(expires_at) = entry.expires_at {
                    if SystemTime::now() >= expires_at {
                        // Entry has expired, but we don't remove it here to avoid write locks
                        return Ok(false);
                    }
                }
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Get quarantine entry for a peer
    pub fn get_quarantine_entry(&self, peer_id: &str) -> QuarantineResult<Option<QuarantineEntry>> {
        let cache = self.peer_cache.read().unwrap();
        
        if let Some(entry_id) = cache.get(peer_id) {
            let list = self.quarantine_list.read().unwrap();
            if let Some(entry) = list.entries.iter().find(|e| e.id == *entry_id) {
                // Check if entry has expired
                if let Some(expires_at) = entry.expires_at {
                    if SystemTime::now() >= expires_at {
                        return Ok(None);
                    }
                }
                return Ok(Some(entry.clone()));
            }
        }
        
        Ok(None)
    }

    /// Add a peer to quarantine
    pub fn quarantine_peer(
        &self,
        peer_id: String,
        reason: QuarantineReason,
        severity: QuarantineSeverity,
        evidence: Option<String>,
        metadata: Option<HashMap<String, String>>,
    ) -> QuarantineResult<String> {
        // Check if peer is already quarantined
        if self.is_quarantined(&peer_id)? {
            return Err(QuarantineError::PeerAlreadyQuarantined(peer_id));
        }

        let now = SystemTime::now();
        let entry_id = Uuid::new_v4().to_string();
        
        // Calculate expiry time based on severity
        let expires_at = if severity == QuarantineSeverity::Permanent {
            None
        } else {
            self.config.default_durations.get(&severity)
                .map(|duration| now + *duration)
        };

        let mut entry = QuarantineEntry {
            id: entry_id.clone(),
            peer_id: peer_id.clone(),
            reason,
            severity,
            quarantined_at: now,
            expires_at,
            imposed_by: self.authority_name.clone(),
            evidence,
            metadata: metadata.unwrap_or_default(),
            signature: String::new(),
            signer_public_key: hex::encode(self.config.signing_keypair.public.to_bytes()),
        };

        // Sign the entry
        entry.signature = self.sign_entry(&entry)?;

        // Add to quarantine list
        {
            let mut list = self.quarantine_list.write().unwrap();
            list.entries.push(entry);
            list.last_updated = now;
            
            // Update cache
            let mut cache = self.peer_cache.write().unwrap();
            cache.insert(peer_id.clone(), entry_id.clone());
        }

        // Save to file
        self.save_to_file()?;

        info!(
            peer_id = peer_id,
            entry_id = entry_id,
            reason = ?entry.reason,
            severity = ?entry.severity,
            "Peer quarantined"
        );

        Ok(entry_id)
    }

    /// Remove a peer from quarantine
    pub fn unquarantine_peer(&self, peer_id: &str) -> QuarantineResult<bool> {
        let mut removed = false;
        
        {
            let mut list = self.quarantine_list.write().unwrap();
            let initial_len = list.entries.len();
            
            list.entries.retain(|entry| entry.peer_id != peer_id);
            
            if list.entries.len() < initial_len {
                removed = true;
                list.last_updated = SystemTime::now();
                
                // Update cache
                let mut cache = self.peer_cache.write().unwrap();
                cache.remove(peer_id);
            }
        }

        if removed {
            self.save_to_file()?;
            info!(peer_id = peer_id, "Peer removed from quarantine");
        }

        Ok(removed)
    }

    /// Extend quarantine for a peer
    pub fn extend_quarantine(
        &self,
        peer_id: &str,
        additional_duration: Duration,
        new_reason: Option<QuarantineReason>,
    ) -> QuarantineResult<()> {
        let mut updated = false;
        
        {
            let mut list = self.quarantine_list.write().unwrap();
            
            if let Some(entry) = list.entries.iter_mut().find(|e| e.peer_id == peer_id) {
                // Extend expiry time
                if let Some(expires_at) = entry.expires_at {
                    entry.expires_at = Some(expires_at + additional_duration);
                } else {
                    // Was permanent, now set expiry
                    entry.expires_at = Some(SystemTime::now() + additional_duration);
                }
                
                // Update reason if provided
                if let Some(reason) = new_reason {
                    entry.reason = reason;
                }
                
                // Re-sign the entry
                entry.signature = self.sign_entry(entry)?;
                
                list.last_updated = SystemTime::now();
                updated = true;
            }
        }

        if updated {
            self.save_to_file()?;
            info!(
                peer_id = peer_id,
                additional_duration = ?additional_duration,
                "Quarantine extended"
            );
            Ok(())
        } else {
            Err(QuarantineError::PeerNotFound(peer_id.to_string()))
        }
    }

    /// Get all quarantined peers
    pub fn list_quarantined_peers(&self, include_expired: bool) -> QuarantineResult<Vec<QuarantineEntry>> {
        let list = self.quarantine_list.read().unwrap();
        let now = SystemTime::now();
        
        let entries = if include_expired {
            list.entries.clone()
        } else {
            list.entries.iter()
                .filter(|entry| {
                    entry.expires_at.map_or(true, |expires_at| now < expires_at)
                })
                .cloned()
                .collect()
        };
        
        Ok(entries)
    }

    /// Cleanup expired entries
    pub fn cleanup_expired(&self) -> QuarantineResult<u32> {
        let now = SystemTime::now();
        let mut removed_count = 0;
        
        {
            let mut list = self.quarantine_list.write().unwrap();
            let initial_len = list.entries.len();
            
            // Collect expired entry peer IDs for cache cleanup
            let expired_peers: Vec<String> = list.entries.iter()
                .filter(|entry| {
                    entry.expires_at.map_or(false, |expires_at| now >= expires_at)
                })
                .map(|entry| entry.peer_id.clone())
                .collect();
            
            // Remove expired entries
            list.entries.retain(|entry| {
                entry.expires_at.map_or(true, |expires_at| now < expires_at)
            });
            
            removed_count = (initial_len - list.entries.len()) as u32;
            
            if removed_count > 0 {
                list.last_updated = now;
                
                // Update cache
                let mut cache = self.peer_cache.write().unwrap();
                for peer_id in expired_peers {
                    cache.remove(&peer_id);
                }
            }
        }

        if removed_count > 0 {
            self.save_to_file()?;
            info!(removed_count = removed_count, "Expired quarantine entries cleaned up");
        }

        Ok(removed_count)
    }

    /// Import quarantine entries from another authority
    pub fn import_entries(
        &self,
        entries: Vec<QuarantineEntry>,
        verify_signatures: bool,
    ) -> QuarantineResult<u32> {
        let mut imported_count = 0;
        let now = SystemTime::now();
        
        {
            let mut list = self.quarantine_list.write().unwrap();
            let mut cache = self.peer_cache.write().unwrap();
            
            for entry in entries {
                // Verify signature if required
                if verify_signatures && !self.verify_entry_signature(&entry)? {
                    warn!(entry_id = entry.id, "Skipping entry with invalid signature");
                    continue;
                }
                
                // Check if entry has expired
                if let Some(expires_at) = entry.expires_at {
                    if now >= expires_at {
                        debug!(entry_id = entry.id, "Skipping expired entry");
                        continue;
                    }
                }
                
                // Check if peer is already quarantined
                if cache.contains_key(&entry.peer_id) {
                    debug!(peer_id = entry.peer_id, "Peer already quarantined, skipping");
                    continue;
                }
                
                // Add entry
                cache.insert(entry.peer_id.clone(), entry.id.clone());
                list.entries.push(entry);
                imported_count += 1;
            }
            
            if imported_count > 0 {
                list.last_updated = now;
            }
        }

        if imported_count > 0 {
            self.save_to_file()?;
            info!(imported_count = imported_count, "Quarantine entries imported");
        }

        Ok(imported_count)
    }

    /// Export quarantine entries
    pub fn export_entries(&self, include_expired: bool) -> QuarantineResult<Vec<QuarantineEntry>> {
        self.list_quarantined_peers(include_expired)
    }

    /// Load quarantine data from file
    pub fn load_from_file(&self) -> QuarantineResult<()> {
        let file = File::open(&self.config.data_file)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        
        // Read entire file
        use std::io::Read;
        reader.read_to_string(&mut content)?;
        
        if content.trim().is_empty() {
            // Empty file, use default list
            return Ok(());
        }
        
        let loaded_list: QuarantineList = serde_json::from_str(&content)?;
        
        // Verify list signature if required
        if self.config.verify_signatures && !self.verify_list_signature(&loaded_list)? {
            return Err(QuarantineError::InvalidSignature(
                "Quarantine list signature verification failed".to_string()
            ));
        }
        
        // Rebuild cache
        let mut peer_cache = HashMap::new();
        for entry in &loaded_list.entries {
            peer_cache.insert(entry.peer_id.clone(), entry.id.clone());
        }
        
        // Update in-memory structures
        {
            let mut list = self.quarantine_list.write().unwrap();
            *list = loaded_list;
        }
        
        {
            let mut cache = self.peer_cache.write().unwrap();
            *cache = peer_cache;
        }
        
        // Update file modification time
        {
            let metadata = std::fs::metadata(&self.config.data_file)?;
            let mut mtime = self.last_file_mtime.write().unwrap();
            *mtime = metadata.modified().ok();
        }
        
        info!("Quarantine data loaded from file");
        Ok(())
    }

    /// Save quarantine data to file
    pub fn save_to_file(&self) -> QuarantineResult<()> {
        // Create backup directory if it doesn't exist
        if !self.config.backup_dir.exists() {
            std::fs::create_dir_all(&self.config.backup_dir)?;
        }
        
        // Create backup of existing file
        if self.config.data_file.exists() {
            let backup_name = format!(
                "quarantine_backup_{}.json",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
            let backup_path = self.config.backup_dir.join(backup_name);
            std::fs::copy(&self.config.data_file, backup_path)?;
        }
        
        let mut list = self.quarantine_list.write().unwrap();
        
        // Sign the list
        list.signature = self.sign_list(&list)?;
        
        // Write to temporary file first
        let temp_file = self.config.data_file.with_extension("tmp");
        {
            let file = File::create(&temp_file)?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, &*list)?;
            writer.flush()?;
        }
        
        // Atomic move
        std::fs::rename(&temp_file, &self.config.data_file)?;
        
        // Cleanup old backups
        self.cleanup_old_backups()?;
        
        debug!("Quarantine data saved to file");
        Ok(())
    }

    /// Get quarantine statistics
    pub fn get_statistics(&self) -> QuarantineResult<QuarantineStats> {
        let list = self.quarantine_list.read().unwrap();
        let now = SystemTime::now();
        
        let mut active_entries = 0;
        let mut expired_entries = 0;
        let mut entries_by_reason = HashMap::new();
        let mut entries_by_severity = HashMap::new();
        
        for entry in &list.entries {
            // Check if active
            let is_active = entry.expires_at.map_or(true, |expires_at| now < expires_at);
            
            if is_active {
                active_entries += 1;
            } else {
                expired_entries += 1;
            }
            
            // Count by reason
            let reason_key = match &entry.reason {
                QuarantineReason::Custom(custom) => format!("custom_{}", custom),
                other => format!("{:?}", other).to_lowercase(),
            };
            *entries_by_reason.entry(reason_key).or_insert(0) += 1;
            
            // Count by severity
            let severity_key = format!("{:?}", entry.severity).to_lowercase();
            *entries_by_severity.entry(severity_key).or_insert(0) += 1;
        }
        
        let file_size = if self.config.data_file.exists() {
            std::fs::metadata(&self.config.data_file)?.len()
        } else {
            0
        };
        
        Ok(QuarantineStats {
            total_entries: list.entries.len(),
            active_entries,
            expired_entries,
            entries_by_reason,
            entries_by_severity,
            last_cleanup: None, // Would track this separately in production
            file_size,
        })
    }

    /// Check if file has been modified externally
    pub fn check_file_changes(&self) -> QuarantineResult<bool> {
        if !self.config.data_file.exists() {
            return Ok(false);
        }
        
        let metadata = std::fs::metadata(&self.config.data_file)?;
        let current_mtime = metadata.modified().ok();
        
        let last_mtime = self.last_file_mtime.read().unwrap();
        
        Ok(current_mtime != *last_mtime)
    }

    /// Reload from file if it has changed
    pub fn reload_if_changed(&self) -> QuarantineResult<bool> {
        if self.check_file_changes()? {
            self.load_from_file()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(self: Arc<Self>) {
        let mut interval_timer = interval(self.config.cleanup_interval);
        
        loop {
            interval_timer.tick().await;
            
            if let Err(e) = self.cleanup_expired() {
                error!(error = %e, "Failed to cleanup expired quarantine entries");
            }
            
            if let Err(e) = self.reload_if_changed() {
                error!(error = %e, "Failed to reload quarantine file");
            }
        }
    }

    // Private helper methods
    
    fn sign_entry(&self, entry: &QuarantineEntry) -> QuarantineResult<String> {
        let content = self.entry_signing_content(entry)?;
        let signature = self.config.signing_keypair.sign(content.as_bytes());
        Ok(hex::encode(signature.to_bytes()))
    }
    
    fn verify_entry_signature(&self, entry: &QuarantineEntry) -> QuarantineResult<bool> {
        let content = self.entry_signing_content(entry)?;
        
        // Decode public key
        let public_key_bytes = hex::decode(&entry.signer_public_key)
            .map_err(|e| QuarantineError::Crypto(format!("Invalid public key hex: {}", e)))?;
        
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| QuarantineError::Crypto(format!("Invalid public key: {}", e)))?;
        
        // Decode signature
        let signature_bytes = hex::decode(&entry.signature)
            .map_err(|e| QuarantineError::Crypto(format!("Invalid signature hex: {}", e)))?;
        
        let signature = Signature::from_bytes(&signature_bytes)
            .map_err(|e| QuarantineError::Crypto(format!("Invalid signature: {}", e)))?;
        
        // Verify signature
        Ok(public_key.verify(content.as_bytes(), &signature).is_ok())
    }
    
    fn entry_signing_content(&self, entry: &QuarantineEntry) -> QuarantineResult<String> {
        let content = serde_json::json!({
            "id": entry.id,
            "peer_id": entry.peer_id,
            "reason": entry.reason,
            "severity": entry.severity,
            "quarantined_at": entry.quarantined_at.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            "expires_at": entry.expires_at.map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs()),
            "imposed_by": entry.imposed_by,
            "evidence": entry.evidence,
            "metadata": entry.metadata,
            "signer_public_key": entry.signer_public_key
        });
        
        Ok(content.to_string())
    }
    
    fn sign_list(&self, list: &QuarantineList) -> QuarantineResult<String> {
        let content = self.list_signing_content(list)?;
        let signature = self.config.signing_keypair.sign(content.as_bytes());
        Ok(hex::encode(signature.to_bytes()))
    }
    
    fn verify_list_signature(&self, list: &QuarantineList) -> QuarantineResult<bool> {
        // Check if we trust this authority
        if !self.config.trusted_authorities.contains(&list.authority_public_key) {
            return Ok(false);
        }
        
        let content = self.list_signing_content(list)?;
        
        // Decode public key
        let public_key_bytes = hex::decode(&list.authority_public_key)
            .map_err(|e| QuarantineError::Crypto(format!("Invalid authority public key hex: {}", e)))?;
        
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| QuarantineError::Crypto(format!("Invalid authority public key: {}", e)))?;
        
        // Decode signature
        let signature_bytes = hex::decode(&list.signature)
            .map_err(|e| QuarantineError::Crypto(format!("Invalid list signature hex: {}", e)))?;
        
        let signature = Signature::from_bytes(&signature_bytes)
            .map_err(|e| QuarantineError::Crypto(format!("Invalid list signature: {}", e)))?;
        
        // Verify signature
        Ok(public_key.verify(content.as_bytes(), &signature).is_ok())
    }
    
    fn list_signing_content(&self, list: &QuarantineList) -> QuarantineResult<String> {
        // Create a copy without signature for signing
        let mut list_for_signing = list.clone();
        list_for_signing.signature = String::new();
        
        let content = serde_json::to_string(&list_for_signing)?;
        Ok(content)
    }
    
    fn cleanup_old_backups(&self) -> QuarantineResult<()> {
        if !self.config.backup_dir.exists() {
            return Ok(());
        }
        
        let mut backup_files: Vec<_> = std::fs::read_dir(&self.config.backup_dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && 
                   path.file_name()?.to_str()?.starts_with("quarantine_backup_") &&
                   path.extension()? == "json" {
                    Some((path, entry.metadata().ok()?.modified().ok()?))
                } else {
                    None
                }
            })
            .collect();
        
        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Remove old backups beyond retention limit
        for (path, _) in backup_files.into_iter().skip(self.config.backup_retention as usize) {
            if let Err(e) = std::fs::remove_file(&path) {
                warn!(path = ?path, error = %e, "Failed to remove old backup file");
            }
        }
        
        Ok(())
    }
}

impl Default for QuarantineConfig {
    fn default() -> Self {
        let mut default_durations = HashMap::new();
        default_durations.insert(QuarantineSeverity::Low, Duration::from_secs(300)); // 5 minutes
        default_durations.insert(QuarantineSeverity::Medium, Duration::from_secs(3600)); // 1 hour
        default_durations.insert(QuarantineSeverity::High, Duration::from_secs(86400)); // 1 day
        default_durations.insert(QuarantineSeverity::Critical, Duration::from_secs(604800)); // 1 week
        
        let mut csprng = rand::rngs::OsRng;
        let keypair = Keypair::generate(&mut csprng);
        
        Self {
            data_file: PathBuf::from("data/quarantine.json"),
            backup_dir: PathBuf::from("data/quarantine_backups"),
            signing_keypair: keypair,
            trusted_authorities: HashSet::new(),
            default_durations,
            max_entries: 10000,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            backup_retention: 10,
            verify_signatures: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::timeout;

    fn create_test_config() -> (QuarantineConfig, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("quarantine.json");
        let backup_dir = temp_dir.path().join("backups");
        
        let mut config = QuarantineConfig::default();
        config.data_file = data_file;
        config.backup_dir = backup_dir;
        config.cleanup_interval = Duration::from_millis(100); // Fast cleanup for testing
        
        (config, temp_dir)
    }

    #[tokio::test]
    async fn test_quarantine_peer() {
        let (config, _temp_dir) = create_test_config();
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        let peer_id = "peer123";
        
        // Initially not quarantined
        assert!(!system.is_quarantined(peer_id).unwrap());
        
        // Quarantine the peer
        let entry_id = system.quarantine_peer(
            peer_id.to_string(),
            QuarantineReason::MaliciousBehavior,
            QuarantineSeverity::High,
            Some("Test evidence".to_string()),
            None,
        ).unwrap();
        
        // Should now be quarantined
        assert!(system.is_quarantined(peer_id).unwrap());
        
        // Get the entry
        let entry = system.get_quarantine_entry(peer_id).unwrap().unwrap();
        assert_eq!(entry.id, entry_id);
        assert_eq!(entry.peer_id, peer_id);
        assert_eq!(entry.reason, QuarantineReason::MaliciousBehavior);
        assert_eq!(entry.severity, QuarantineSeverity::High);
        assert_eq!(entry.evidence, Some("Test evidence".to_string()));
    }

    #[tokio::test]
    async fn test_quarantine_expiry() {
        let (mut config, _temp_dir) = create_test_config();
        // Set very short duration for testing
        config.default_durations.insert(QuarantineSeverity::Low, Duration::from_millis(100));
        
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        let peer_id = "peer123";
        
        // Quarantine with low severity (short duration)
        system.quarantine_peer(
            peer_id.to_string(),
            QuarantineReason::Spam,
            QuarantineSeverity::Low,
            None,
            None,
        ).unwrap();
        
        // Should be quarantined initially
        assert!(system.is_quarantined(peer_id).unwrap());
        
        // Wait for expiry
        sleep(Duration::from_millis(150)).await;
        
        // Should no longer be quarantined
        assert!(!system.is_quarantined(peer_id).unwrap());
        
        // Entry should not be returned
        assert!(system.get_quarantine_entry(peer_id).unwrap().is_none());
    }

    #[tokio::test]
    async fn test_permanent_quarantine() {
        let (config, _temp_dir) = create_test_config();
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        let peer_id = "bad_peer";
        
        // Quarantine permanently
        system.quarantine_peer(
            peer_id.to_string(),
            QuarantineReason::KnownBadActor,
            QuarantineSeverity::Permanent,
            Some("Known malicious actor".to_string()),
            None,
        ).unwrap();
        
        // Should be quarantined
        assert!(system.is_quarantined(peer_id).unwrap());
        
        // Entry should have no expiry
        let entry = system.get_quarantine_entry(peer_id).unwrap().unwrap();
        assert!(entry.expires_at.is_none());
        
        // Should still be quarantined after some time
        sleep(Duration::from_millis(200)).await;
        assert!(system.is_quarantined(peer_id).unwrap());
    }

    #[tokio::test]
    async fn test_unquarantine_peer() {
        let (config, _temp_dir) = create_test_config();
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        let peer_id = "peer123";
        
        // Quarantine the peer
        system.quarantine_peer(
            peer_id.to_string(),
            QuarantineReason::ProtocolViolation,
            QuarantineSeverity::Medium,
            None,
            None,
        ).unwrap();
        
        assert!(system.is_quarantined(peer_id).unwrap());
        
        // Unquarantine the peer
        let removed = system.unquarantine_peer(peer_id).unwrap();
        assert!(removed);
        
        // Should no longer be quarantined
        assert!(!system.is_quarantined(peer_id).unwrap());
        
        // Trying to unquarantine again should return false
        let removed_again = system.unquarantine_peer(peer_id).unwrap();
        assert!(!removed_again);
    }

    #[tokio::test]
    async fn test_extend_quarantine() {
        let (config, _temp_dir) = create_test_config();
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        let peer_id = "peer123";
        
        // Quarantine the peer
        system.quarantine_peer(
            peer_id.to_string(),
            QuarantineReason::Spam,
            QuarantineSeverity::Low,
            None,
            None,
        ).unwrap();
        
        let original_entry = system.get_quarantine_entry(peer_id).unwrap().unwrap();
        let original_expiry = original_entry.expires_at.unwrap();
        
        // Extend quarantine
        system.extend_quarantine(
            peer_id,
            Duration::from_secs(3600),
            Some(QuarantineReason::MaliciousBehavior),
        ).unwrap();
        
        let extended_entry = system.get_quarantine_entry(peer_id).unwrap().unwrap();
        let new_expiry = extended_entry.expires_at.unwrap();
        
        // Expiry should be later
        assert!(new_expiry > original_expiry);
        
        // Reason should be updated
        assert_eq!(extended_entry.reason, QuarantineReason::MaliciousBehavior);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let (mut config, _temp_dir) = create_test_config();
        config.default_durations.insert(QuarantineSeverity::Low, Duration::from_millis(50));
        
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        // Add some peers with short expiry
        for i in 0..3 {
            system.quarantine_peer(
                format!("peer{}", i),
                QuarantineReason::Spam,
                QuarantineSeverity::Low,
                None,
                None,
            ).unwrap();
        }
        
        // Add one with longer expiry
        system.quarantine_peer(
            "long_peer".to_string(),
            QuarantineReason::MaliciousBehavior,
            QuarantineSeverity::High,
            None,
            None,
        ).unwrap();
        
        // All should be quarantined initially
        let stats = system.get_statistics().unwrap();
        assert_eq!(stats.total_entries, 4);
        assert_eq!(stats.active_entries, 4);
        
        // Wait for short ones to expire
        sleep(Duration::from_millis(100)).await;
        
        // Cleanup
        let removed = system.cleanup_expired().unwrap();
        assert_eq!(removed, 3);
        
        // Only long_peer should remain
        let stats = system.get_statistics().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.active_entries, 1);
        
        assert!(!system.is_quarantined("peer0").unwrap());
        assert!(!system.is_quarantined("peer1").unwrap());
        assert!(!system.is_quarantined("peer2").unwrap());
        assert!(system.is_quarantined("long_peer").unwrap());
    }

    #[tokio::test]
    async fn test_file_persistence() {
        let (config, _temp_dir) = create_test_config();
        
        // Create first system and add peers
        {
            let system1 = QuarantineSystem::new(config.clone(), "test_authority".to_string()).unwrap();
            
            system1.quarantine_peer(
                "peer1".to_string(),
                QuarantineReason::MaliciousBehavior,
                QuarantineSeverity::High,
                Some("Evidence 1".to_string()),
                None,
            ).unwrap();
            
            system1.quarantine_peer(
                "peer2".to_string(),
                QuarantineReason::Spam,
                QuarantineSeverity::Medium,
                None,
                None,
            ).unwrap();
        }
        
        // Create second system and verify data is loaded
        {
            let system2 = QuarantineSystem::new(config.clone(), "test_authority".to_string()).unwrap();
            
            assert!(system2.is_quarantined("peer1").unwrap());
            assert!(system2.is_quarantined("peer2").unwrap());
            
            let entry1 = system2.get_quarantine_entry("peer1").unwrap().unwrap();
            assert_eq!(entry1.reason, QuarantineReason::MaliciousBehavior);
            assert_eq!(entry1.evidence, Some("Evidence 1".to_string()));
            
            let entry2 = system2.get_quarantine_entry("peer2").unwrap().unwrap();
            assert_eq!(entry2.reason, QuarantineReason::Spam);
        }
    }

    #[tokio::test]
    async fn test_statistics() {
        let (config, _temp_dir) = create_test_config();
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        // Add various peers
        system.quarantine_peer(
            "peer1".to_string(),
            QuarantineReason::MaliciousBehavior,
            QuarantineSeverity::High,
            None,
            None,
        ).unwrap();
        
        system.quarantine_peer(
            "peer2".to_string(),
            QuarantineReason::Spam,
            QuarantineSeverity::Low,
            None,
            None,
        ).unwrap();
        
        system.quarantine_peer(
            "peer3".to_string(),
            QuarantineReason::MaliciousBehavior,
            QuarantineSeverity::Critical,
            None,
            None,
        ).unwrap();
        
        let stats = system.get_statistics().unwrap();
        
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.active_entries, 3);
        assert_eq!(stats.expired_entries, 0);
        
        // Check reason counts
        assert_eq!(stats.entries_by_reason.get("maliciousbehavior"), Some(&2));
        assert_eq!(stats.entries_by_reason.get("spam"), Some(&1));
        
        // Check severity counts
        assert_eq!(stats.entries_by_severity.get("high"), Some(&1));
        assert_eq!(stats.entries_by_severity.get("low"), Some(&1));
        assert_eq!(stats.entries_by_severity.get("critical"), Some(&1));
    }

    #[tokio::test]
    async fn test_signature_verification() {
        let (config, _temp_dir) = create_test_config();
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        // Add a peer
        system.quarantine_peer(
            "peer1".to_string(),
            QuarantineReason::MaliciousBehavior,
            QuarantineSeverity::High,
            None,
            None,
        ).unwrap();
        
        let entry = system.get_quarantine_entry("peer1").unwrap().unwrap();
        
        // Verify the signature
        assert!(system.verify_entry_signature(&entry).unwrap());
        
        // Corrupt the signature
        let mut corrupted_entry = entry.clone();
        corrupted_entry.signature = "invalid_signature".to_string();
        
        assert!(!system.verify_entry_signature(&corrupted_entry).unwrap());
    }

    #[tokio::test]
    async fn test_already_quarantined_error() {
        let (config, _temp_dir) = create_test_config();
        let system = QuarantineSystem::new(config, "test_authority".to_string()).unwrap();
        
        let peer_id = "peer123";
        
        // Quarantine the peer
        system.quarantine_peer(
            peer_id.to_string(),
            QuarantineReason::Spam,
            QuarantineSeverity::Low,
            None,
            None,
        ).unwrap();
        
        // Try to quarantine again
        let result = system.quarantine_peer(
            peer_id.to_string(),
            QuarantineReason::MaliciousBehavior,
            QuarantineSeverity::High,
            None,
            None,
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            QuarantineError::PeerAlreadyQuarantined(id) => {
                assert_eq!(id, peer_id);
            }
            _ => panic!("Expected PeerAlreadyQuarantined error"),
        }
    }

    #[tokio::test]
    async fn test_background_cleanup() {
        let (mut config, _temp_dir) = create_test_config();
        config.default_durations.insert(QuarantineSeverity::Low, Duration::from_millis(50));
        config.cleanup_interval = Duration::from_millis(100);
        
        let system = Arc::new(QuarantineSystem::new(config, "test_authority".to_string()).unwrap());
        
        // Add a peer with short expiry
        system.quarantine_peer(
            "peer1".to_string(),
            QuarantineReason::Spam,
            QuarantineSeverity::Low,
            None,
            None,
        ).unwrap();
        
        assert!(system.is_quarantined("peer1").unwrap());
        
        // Start background cleanup
        let system_clone = Arc::clone(&system);
        let cleanup_task = tokio::spawn(async move {
            system_clone.start_cleanup_task().await;
        });
        
        // Wait for cleanup to occur
        sleep(Duration::from_millis(200)).await;
        
        // Peer should be automatically removed
        assert!(!system.is_quarantined("peer1").unwrap());
        
        // Cancel the background task
        cleanup_task.abort();
    }
}
