//! Minimal Memory Store (Local, Private)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::ipc::{ToolCallRequest, ToolCallResponse};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub access_count: u64,
    pub last_accessed: u64,
    pub user_id: String,
    pub session_id: Option<String>,
    pub redacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub base_path: PathBuf,
    pub max_entries: usize,
    pub default_ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
    pub enable_redaction: bool,
    pub redaction_policies: Vec<RedactionPolicy>,
    pub enable_ngfs: bool,
    pub ngfs_snapshot_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionPolicy {
    pub name: String,
    pub pattern: String,
    pub replacement: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
    pub total_access_count: u64,
    pub storage_size_bytes: u64,
    pub last_cleanup: u64,
    pub last_ngfs_snapshot: Option<u64>,
}

#[derive(Clone)]
pub struct MemoryStore {
    config: MemoryConfig,
    entries: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    tag_index: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    redaction_patterns: Arc<RwLock<Vec<(regex::Regex, String)>>>,
    stats: Arc<RwLock<MemoryStats>>,
}

impl MemoryStore {
    pub fn new(config: MemoryConfig) -> Result<Self> {
        if config.max_entries == 0 {
            return Err(crate::error::AiCoreError::ConfigError(
                "max_entries must be greater than 0".to_string(),
            ));
        }
        if config.default_ttl_seconds == 0 {
            return Err(crate::error::AiCoreError::ConfigError(
                "default_ttl_seconds must be greater than 0".to_string(),
            ));
        }
        for policy in &config.redaction_policies {
            regex::Regex::new(&policy.pattern).map_err(|error| {
                crate::error::AiCoreError::ConfigError(format!(
                    "invalid redaction regex '{}': {}",
                    policy.name, error
                ))
            })?;
        }
        let redaction_patterns: Vec<(regex::Regex, String)> = config
            .redaction_policies
            .iter()
            .map(|p| {
                Ok((
                    regex::Regex::new(&p.pattern).map_err(|error| {
                        crate::error::AiCoreError::ConfigError(format!(
                            "invalid redaction regex '{}': {}",
                            p.name, error
                        ))
                    })?,
                    p.replacement.clone(),
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
            redaction_patterns: Arc::new(RwLock::new(redaction_patterns)),
            stats: Arc::new(RwLock::new(MemoryStats {
                total_entries: 0,
                active_entries: 0,
                expired_entries: 0,
                total_access_count: 0,
                storage_size_bytes: 0,
                last_cleanup: 0,
                last_ngfs_snapshot: None,
            })),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        // Initializing memory store
        fs::create_dir_all(&self.config.base_path).await.ok();
        self.load_from_disk().await?;
        Ok(())
    }

    pub async fn put(
        &self,
        key: String,
        value: String,
        tags: Vec<String>,
        ttl_seconds: Option<u64>,
        user_id: String,
        session_id: Option<String>,
    ) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = ttl_seconds.map(|ttl| now + ttl);
        let (redacted_value, was_redacted) = if self.config.enable_redaction {
            self.apply_redaction(&value).await?
        } else {
            (value, false)
        };

        let entry = MemoryEntry {
            key: key.clone(),
            value: redacted_value,
            tags: tags.clone(),
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
            user_id,
            session_id,
            redacted: was_redacted,
        };

        self.entries.write().await.insert(key.clone(), entry);
        self.update_tag_index(&key, &tags).await?;
        self.update_stats().await?;
        self.save_to_disk().await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<MemoryEntry>> {
        let mut entries = self.entries.write().await;
        if let Some(entry) = entries.get_mut(key) {
            if let Some(expires_at) = entry.expires_at {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now > expires_at {
                    entries.remove(key);
                    drop(entries);
                    self.update_tag_index_remove(key).await?;
                    self.update_stats().await?;
                    self.save_to_disk().await?;
                    return Ok(None);
                }
            }
            entry.access_count += 1;
            entry.last_accessed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(Some(entry.clone()))
        } else {
            Ok(None)
        }
    }

    pub async fn query(&self, tags: &[String]) -> Result<Vec<MemoryEntry>> {
        let entries = self.entries.read().await;
        let tag_index = self.tag_index.read().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut matching_keys = HashSet::new();
        for tag in tags {
            if let Some(keys) = tag_index.get(tag) {
                if matching_keys.is_empty() {
                    matching_keys = keys.clone();
                } else {
                    matching_keys = matching_keys.intersection(keys).cloned().collect();
                }
            } else {
                return Ok(Vec::new());
            }
        }

        let mut results: Vec<_> = matching_keys
            .iter()
            .filter_map(|key| entries.get(key))
            .filter(|e| e.expires_at.is_none_or(|exp| now <= exp))
            .cloned()
            .collect();
        results.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        Ok(results)
    }

    pub async fn delete(&self, key: &str) -> Result<bool> {
        let removed = self.entries.write().await.remove(key).is_some();
        if removed {
            self.update_tag_index_remove(key).await?;
            self.update_stats().await?;
            self.save_to_disk().await?;
        }
        Ok(removed)
    }

    pub async fn clear(&self) -> Result<()> {
        self.entries.write().await.clear();
        self.tag_index.write().await.clear();
        self.update_stats().await?;
        self.save_to_disk().await?;
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<MemoryStats> {
        Ok(self.stats.read().await.clone())
    }

    async fn apply_redaction(&self, value: &str) -> Result<(String, bool)> {
        let patterns = self.redaction_patterns.read().await;
        let mut result = value.to_string();
        let mut was_redacted = false;
        for (pattern, replacement) in patterns.iter() {
            if pattern.is_match(&result) {
                result = pattern.replace_all(&result, replacement.as_str()).to_string();
                was_redacted = true;
            }
        }
        Ok((result, was_redacted))
    }

    async fn update_tag_index(&self, key: &str, tags: &[String]) -> Result<()> {
        let mut index = self.tag_index.write().await;
        for tag in tags {
            index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(key.to_string());
        }
        Ok(())
    }

    async fn save_to_disk(&self) -> Result<()> {
        fs::create_dir_all(&self.config.base_path).await.ok();
        let data_file = self.config.base_path.join("memory_data.cbor");
        let entries = self.entries.read().await;
        let data = serde_cbor::to_vec(&*entries)
            .map_err(|error| crate::error::AiCoreError::InternalError(error.to_string()))?;
        fs::write(data_file, data)
            .await
            .map_err(|error| crate::error::AiCoreError::IoError(error.to_string()))?;
        Ok(())
    }

    async fn update_tag_index_remove(&self, key: &str) -> Result<()> {
        let mut index = self.tag_index.write().await;
        for keys in index.values_mut() {
            keys.remove(key);
        }
        index.retain(|_, keys| !keys.is_empty());
        Ok(())
    }

    async fn update_stats(&self) -> Result<()> {
        let entries = self.entries.read().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut stats = self.stats.write().await;
        stats.total_entries = entries.len();
        stats.active_entries = entries
            .values()
            .filter(|e| e.expires_at.is_none_or(|exp| now <= exp))
            .count();
        stats.expired_entries = stats.total_entries - stats.active_entries;
        stats.total_access_count = entries.values().map(|e| e.access_count).sum();
        stats.storage_size_bytes = entries
            .values()
            .map(|e| (e.key.len() + e.value.len()) as u64)
            .sum();
        Ok(())
    }

    async fn load_from_disk(&self) -> Result<()> {
        let data_file = self.config.base_path.join("memory_data.cbor");
        if data_file.exists() {
            if let Ok(data) = fs::read(&data_file).await {
                if let Ok(loaded) = serde_cbor::from_slice::<HashMap<String, MemoryEntry>>(&data) {
                    let mut entries = self.entries.write().await;
                    let mut index = self.tag_index.write().await;
                    for (key, entry) in loaded.iter() {
                        for tag in &entry.tags {
                            index
                                .entry(tag.clone())
                                .or_insert_with(HashSet::new)
                                .insert(key.clone());
                        }
                    }
                    let _count = loaded.len();
                    entries.extend(loaded);
                    // Loaded memory entries from disk
                }
            }
        }
        Ok(())
    }

    pub async fn handle_put_tool(&self, request: &ToolCallRequest) -> Result<ToolCallResponse> {
        let args: HashMap<String, serde_json::Value> =
            serde_json::from_str(&request.arguments).unwrap_or_default();
        let key = args
            .get("key")
            .and_then(|v| v.as_str())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| crate::error::AiCoreError::InvalidInput("mem.put requires key".to_string()))?
            .to_string();
        let value = args
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::AiCoreError::InvalidInput("mem.put requires value".to_string()))?
            .to_string();
        let tags: Vec<String> = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let ttl = args.get("ttl_seconds").and_then(|v| v.as_u64());
        let user_id = request
            .user_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        self.put(
            key.clone(),
            value,
            tags,
            ttl,
            user_id,
            request.session_id.clone(),
        )
        .await?;

        Ok(ToolCallResponse {
            result: format!("Stored: {}", key),
            success: true,
            error_message: String::new(),
            metrics: None,
            cbor_payload: None,
            call_id: request.call_id.clone(),
            tool_version: "1.0.0".to_string(),
            metadata: None,
            warnings: Vec::new(),
            exit_code: 0,
        })
    }

    pub async fn handle_get_tool(&self, request: &ToolCallRequest) -> Result<ToolCallResponse> {
        let args: HashMap<String, serde_json::Value> =
            serde_json::from_str(&request.arguments).unwrap_or_default();
        let key = args
            .get("key")
            .and_then(|v| v.as_str())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| crate::error::AiCoreError::InvalidInput("mem.get requires key".to_string()))?;
        let entry = self.get(key).await?;
        let result = entry
            .map(|e| serde_json::to_string(&e).unwrap_or_default())
            .unwrap_or_else(|| "null".to_string());

        Ok(ToolCallResponse {
            result,
            success: true,
            error_message: String::new(),
            metrics: None,
            cbor_payload: None,
            call_id: request.call_id.clone(),
            tool_version: "1.0.0".to_string(),
            metadata: None,
            warnings: Vec::new(),
            exit_code: 0,
        })
    }

    pub async fn handle_query_tool(&self, request: &ToolCallRequest) -> Result<ToolCallResponse> {
        let args: HashMap<String, serde_json::Value> =
            serde_json::from_str(&request.arguments).unwrap_or_default();
        let tags: Vec<String> = args
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .ok_or_else(|| crate::error::AiCoreError::InvalidInput("mem.query requires tags".to_string()))?;
        let entries = self.query(&tags).await?;
        let result = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());

        Ok(ToolCallResponse {
            result,
            success: true,
            error_message: String::new(),
            metrics: None,
            cbor_payload: None,
            call_id: request.call_id.clone(),
            tool_version: "1.0.0".to_string(),
            metadata: None,
            warnings: Vec::new(),
            exit_code: 0,
        })
    }

    pub async fn handle_delete_tool(&self, request: &ToolCallRequest) -> Result<ToolCallResponse> {
        let args: HashMap<String, serde_json::Value> =
            serde_json::from_str(&request.arguments).unwrap_or_default();
        let key = args.get("key").and_then(|v| v.as_str()).unwrap_or("");
        let deleted = self.delete(key).await?;

        Ok(ToolCallResponse {
            result: format!("Deleted: {}", deleted),
            success: true,
            error_message: String::new(),
            metrics: None,
            cbor_payload: None,
            call_id: request.call_id.clone(),
            tool_version: "1.0.0".to_string(),
            metadata: None,
            warnings: Vec::new(),
            exit_code: 0,
        })
    }

    pub async fn handle_stats_tool(&self, request: &ToolCallRequest) -> Result<ToolCallResponse> {
        let stats = self.get_stats().await?;
        let result = serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string());

        Ok(ToolCallResponse {
            result,
            success: true,
            error_message: String::new(),
            metrics: None,
            cbor_payload: None,
            call_id: request.call_id.clone(),
            tool_version: "1.0.0".to_string(),
            metadata: None,
            warnings: Vec::new(),
            exit_code: 0,
        })
    }
}

pub fn default_redaction_policies() -> Vec<RedactionPolicy> {
    vec![
        RedactionPolicy {
            name: "password".to_string(),
            pattern: r"(?i)(password|passwd|pwd)\s*[:=]\s*\S+".to_string(),
            replacement: "[REDACTED]".to_string(),
            description: None,
        },
        RedactionPolicy {
            name: "api_key".to_string(),
            pattern: r"(?i)(api[_-]?key|apikey)\s*[:=]\s*\S+".to_string(),
            replacement: "[REDACTED]".to_string(),
            description: None,
        },
        RedactionPolicy {
            name: "token".to_string(),
            pattern: r"(?i)(token|bearer)\s*[:=]\s*\S+".to_string(),
            replacement: "[REDACTED]".to_string(),
            description: None,
        },
        RedactionPolicy {
            name: "secret".to_string(),
            pattern: r"(?i)(secret)\s*[:=]\s*\S+".to_string(),
            replacement: "[REDACTED]".to_string(),
            description: None,
        },
        RedactionPolicy {
            name: "email".to_string(),
            pattern: r"(?i)[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}".to_string(),
            replacement: "[EMAIL_REDACTED]".to_string(),
            description: None,
        },
        RedactionPolicy {
            name: "phone".to_string(),
            pattern: r"(?x)(?:\+?\d[\d\s().-]{7,}\d)".to_string(),
            replacement: "[PHONE_REDACTED]".to_string(),
            description: None,
        },
    ]
}

pub fn default_memory_config(base_path: PathBuf) -> MemoryConfig {
    MemoryConfig {
        base_path,
        max_entries: 10000,
        default_ttl_seconds: 86400,
        cleanup_interval_seconds: 3600,
        enable_redaction: true,
        redaction_policies: default_redaction_policies(),
        enable_ngfs: false,
        ngfs_snapshot_interval_seconds: 86400,
    }
}
