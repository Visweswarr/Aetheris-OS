//! Plan Store - Deterministic Plan IDs + Persistence + Retrieval
//!
//! This module provides:
//! - Deterministic Plan IDs using blake3(canonical_json(plan))
//! - Thread-safe in-memory index with DashMap
//! - Append-only NDJSON persistence with atomic writes
//! - Crash-safe log replay on startup
//! - Idempotent operations for creates and approvals

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{File, OpenOptions, create_dir_all};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use blake3::Hasher;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::{info, warn, debug};

use crate::planner::{Plan, PlanStep};

/// Deterministic Plan ID derived from blake3(canonical_json(plan))
pub type PlanId = String;

/// Plan metadata for tracking status and revisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanMetadata {
    pub plan_id: PlanId,
    pub created_at: DateTime<Utc>,
    pub user_id: String,
    pub status: PlanStatus,
    pub revision: u32,
}

/// Plan status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    Pending,
    Approved,
}

/// Complete plan record stored in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRecord {
    pub metadata: PlanMetadata,
    pub plan: Plan,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
}

/// Events persisted to NDJSON log
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum PlanEvent {
    PlanCreated {
        plan_id: PlanId,
        plan: Plan,
        created_at: DateTime<Utc>,
        user_id: String,
        revision: u32,
    },
    PlanApproved {
        plan_id: PlanId,
        approved_by: String,
        approved_at: DateTime<Utc>,
        revision: u32,
    },
    PlanRevised {
        plan_id: PlanId,
        parent_plan_id: PlanId,
        plan: Plan,
        revised_by: String,
        revised_at: DateTime<Utc>,
        revision: u32,
    },
}

/// Plan store errors
#[derive(Debug, Error)]
pub enum PlanStoreError {
    #[error("Plan not found: {plan_id}")]
    PlanNotFound { plan_id: PlanId },
    
    #[error("Plan already approved: {plan_id} revision {revision}")]
    PlanAlreadyApproved { plan_id: PlanId, revision: u32 },
    
    #[error("Invalid revision: {revision} for plan {plan_id}")]
    InvalidRevision { plan_id: PlanId, revision: u32 },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Internal error: {message}")]
    Internal { message: String },
}

/// List query parameters
#[derive(Debug, Clone)]
pub struct ListQuery {
    pub status: Option<PlanStatus>,
    pub limit: Option<usize>,
    pub cursor: Option<String>,
}

/// List response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub plans: Vec<PlanRecord>,
    pub next_cursor: Option<String>,
    pub total_count: usize,
}

/// Thread-safe plan store with persistence
pub struct PlanStore {
    /// In-memory index: PlanId -> PlanRecord
    index: Arc<DashMap<PlanId, PlanRecord>>,
    
    /// File write synchronization
    write_mutex: Arc<Mutex<()>>,
    
    /// Path to NDJSON persistence file
    log_path: PathBuf,
}

impl PlanStore {
    /// Create new plan store with specified log path
    pub async fn new<P: AsRef<Path>>(log_path: P) -> Result<Self, PlanStoreError> {
        let log_path = log_path.as_ref().to_path_buf();
        
        // Ensure data directory exists
        if let Some(parent) = log_path.parent() {
            create_dir_all(parent).await?;
        }
        
        let store = Self {
            index: Arc::new(DashMap::new()),
            write_mutex: Arc::new(Mutex::new(())),
            log_path,
        };
        
        // Replay log to rebuild index
        store.replay_log().await?;
        
        info!("Plan store initialized with {} plans", store.index.len());
        Ok(store)
    }
    
    /// Create new plan store with default data directory
    pub async fn default() -> Result<Self, PlanStoreError> {
        Self::new("data/ai_plans.ndjson").await
    }
    
    /// Generate deterministic Plan ID from canonical JSON
    pub fn generate_plan_id(plan: &Plan) -> Result<PlanId, PlanStoreError> {
        let canonical_json = canonicalize_plan(plan)?;
        let canonical_str = serde_json::to_string(&canonical_json)?;
        
        let mut hasher = Hasher::new();
        hasher.update(canonical_str.as_bytes());
        let hash = hasher.finalize();
        
        Ok(hash.to_hex().to_string())
    }
    
    /// Create or get existing plan (idempotent)
    pub async fn create_plan(
        &self,
        plan: Plan,
        user_id: String,
    ) -> Result<PlanRecord, PlanStoreError> {
        let plan_id = Self::generate_plan_id(&plan)?;
        
        // Check if plan already exists
        if let Some(existing) = self.index.get(&plan_id) {
            debug!("Plan {} already exists, returning existing record", plan_id);
            return Ok(existing.clone());
        }
        
        let now = Utc::now();
        let revision = 1u32;
        
        let metadata = PlanMetadata {
            plan_id: plan_id.clone(),
            created_at: now,
            user_id: user_id.clone(),
            status: PlanStatus::Pending,
            revision,
        };
        
        let record = PlanRecord {
            metadata,
            plan: plan.clone(),
            approved_by: None,
            approved_at: None,
        };
        
        // Persist event
        let event = PlanEvent::PlanCreated {
            plan_id: plan_id.clone(),
            plan,
            created_at: now,
            user_id,
            revision,
        };
        
        self.append_event(&event).await?;
        
        // Update index
        self.index.insert(plan_id.clone(), record.clone());
        
        info!("Created plan {} revision {}", plan_id, revision);
        Ok(record)
    }
    
    /// Get plan by ID
    pub fn get_plan(&self, plan_id: &str) -> Result<PlanRecord, PlanStoreError> {
        self.index
            .get(plan_id)
            .map(|entry| entry.clone())
            .ok_or_else(|| PlanStoreError::PlanNotFound {
                plan_id: plan_id.to_string(),
            })
    }
    
    /// Approve plan (idempotent)
    pub async fn approve_plan(
        &self,
        plan_id: &str,
        revision: Option<u32>,
        user_id: String,
        modifications: Option<Vec<PlanStep>>,
    ) -> Result<PlanRecord, PlanStoreError> {
        let mut record = self.get_plan(plan_id)?;
        
        // Check revision if specified
        if let Some(rev) = revision {
            if record.metadata.revision != rev {
                return Err(PlanStoreError::InvalidRevision {
                    plan_id: plan_id.to_string(),
                    revision: rev,
                });
            }
        }
        
        // Handle modifications (creates new revision)
        if let Some(new_steps) = modifications {
            let mut modified_plan = record.plan.clone();
            modified_plan.steps = new_steps;
            
            let new_plan_id = Self::generate_plan_id(&modified_plan)?;
            let new_revision = record.metadata.revision + 1;
            let now = Utc::now();
            
            let new_metadata = PlanMetadata {
                plan_id: new_plan_id.clone(),
                created_at: now,
                user_id: user_id.clone(),
                status: PlanStatus::Approved,
                revision: new_revision,
            };
            
            let new_record = PlanRecord {
                metadata: new_metadata,
                plan: modified_plan.clone(),
                approved_by: Some(user_id.clone()),
                approved_at: Some(now),
            };
            
            // Persist revision event
            let event = PlanEvent::PlanRevised {
                plan_id: new_plan_id.clone(),
                parent_plan_id: plan_id.to_string(),
                plan: modified_plan,
                revised_by: user_id,
                revised_at: now,
                revision: new_revision,
            };
            
            self.append_event(&event).await?;
            self.index.insert(new_plan_id.clone(), new_record.clone());
            
            info!("Created revision {} for plan {}", new_revision, new_plan_id);
            return Ok(new_record);
        }
        
        // Check if already approved (idempotent)
        if record.metadata.status == PlanStatus::Approved {
            debug!("Plan {} already approved, returning existing state", plan_id);
            return Ok(record);
        }
        
        // Approve current revision
        let now = Utc::now();
        record.metadata.status = PlanStatus::Approved;
        record.approved_by = Some(user_id.clone());
        record.approved_at = Some(now);
        
        // Persist approval event
        let event = PlanEvent::PlanApproved {
            plan_id: plan_id.to_string(),
            approved_by: user_id,
            approved_at: now,
            revision: record.metadata.revision,
        };
        
        self.append_event(&event).await?;
        
        // Update index
        self.index.insert(plan_id.to_string(), record.clone());
        
        info!("Approved plan {} revision {}", plan_id, record.metadata.revision);
        Ok(record)
    }
    
    /// List plans with optional filtering and pagination
    pub fn list_plans(&self, query: ListQuery) -> ListResponse {
        let mut plans: Vec<PlanRecord> = self.index
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        // Filter by status
        if let Some(status) = &query.status {
            plans.retain(|record| &record.metadata.status == status);
        }
        
        // Sort for stable pagination (lexicographic by plan_id, then revision)
        plans.sort_by(|a, b| {
            a.metadata.plan_id.cmp(&b.metadata.plan_id)
                .then_with(|| a.metadata.revision.cmp(&b.metadata.revision))
        });
        
        let total_count = plans.len();
        
        // Apply cursor-based pagination
        if let Some(cursor) = &query.cursor {
            if let Some(pos) = plans.iter().position(|p| &p.metadata.plan_id > cursor) {
                plans = plans.into_iter().skip(pos).collect();
            } else {
                plans.clear();
            }
        }
        
        // Apply limit
        let next_cursor = if let Some(limit) = query.limit {
            if plans.len() > limit {
                let cursor_plan = plans[limit].metadata.plan_id.clone();
                plans.truncate(limit);
                Some(cursor_plan)
            } else {
                None
            }
        } else {
            None
        };
        
        ListResponse {
            plans,
            next_cursor,
            total_count,
        }
    }
    
    /// Append event to NDJSON log with atomic write
    async fn append_event(&self, event: &PlanEvent) -> Result<(), PlanStoreError> {
        let _lock = self.write_mutex.lock().await;
        
        // Serialize to canonical JSON
        let canonical_event = canonicalize_event(event)?;
        let line = serde_json::to_string(&canonical_event)?;
        
        // Atomic append with fsync
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await?;
        
        file.write_all(line.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.sync_all().await?;
        
        debug!("Appended event to log: {}", line);
        Ok(())
    }
    
    /// Replay NDJSON log to rebuild index
    async fn replay_log(&self) -> Result<(), PlanStoreError> {
        if !self.log_path.exists() {
            info!("No existing log file, starting fresh");
            return Ok(());
        }
        
        let file = File::open(&self.log_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut event_count = 0;
        
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<PlanEvent>(&line) {
                Ok(event) => {
                    self.apply_event(event);
                    event_count += 1;
                }
                Err(e) => {
                    warn!("Failed to parse log line: {} - {}", line, e);
                }
            }
        }
        
        info!("Replayed {} events from log", event_count);
        Ok(())
    }
    
    /// Apply event to in-memory index
    fn apply_event(&self, event: PlanEvent) {
        match event {
            PlanEvent::PlanCreated {
                plan_id,
                plan,
                created_at,
                user_id,
                revision,
            } => {
                let metadata = PlanMetadata {
                    plan_id: plan_id.clone(),
                    created_at,
                    user_id,
                    status: PlanStatus::Pending,
                    revision,
                };
                
                let record = PlanRecord {
                    metadata,
                    plan,
                    approved_by: None,
                    approved_at: None,
                };
                
                self.index.insert(plan_id, record);
            }
            
            PlanEvent::PlanApproved {
                plan_id,
                approved_by,
                approved_at,
                revision: _,
            } => {
                if let Some(mut record) = self.index.get_mut(&plan_id) {
                    record.metadata.status = PlanStatus::Approved;
                    record.approved_by = Some(approved_by);
                    record.approved_at = Some(approved_at);
                }
            }
            
            PlanEvent::PlanRevised {
                plan_id,
                parent_plan_id: _,
                plan,
                revised_by,
                revised_at,
                revision,
            } => {
                let metadata = PlanMetadata {
                    plan_id: plan_id.clone(),
                    created_at: revised_at,
                    user_id: revised_by.clone(),
                    status: PlanStatus::Approved,
                    revision,
                };
                
                let record = PlanRecord {
                    metadata,
                    plan,
                    approved_by: Some(revised_by),
                    approved_at: Some(revised_at),
                };
                
                self.index.insert(plan_id, record);
            }
        }
    }
}

/// Convert plan to canonical JSON representation
fn canonicalize_plan(plan: &Plan) -> Result<Value, PlanStoreError> {
    let json = serde_json::to_value(plan)?;
    Ok(canonicalize_value(json))
}

/// Convert event to canonical JSON representation
fn canonicalize_event(event: &PlanEvent) -> Result<Value, PlanStoreError> {
    let json = serde_json::to_value(event)?;
    Ok(canonicalize_value(json))
}

/// Recursively canonicalize JSON value (convert Map to BTreeMap)
fn canonicalize_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let btree: BTreeMap<String, Value> = map
                .into_iter()
                .map(|(k, v)| (k, canonicalize_value(v)))
                .collect();
            Value::Object(btree.into_iter().collect())
        }
        Value::Array(arr) => {
            Value::Array(arr.into_iter().map(canonicalize_value).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::planner::{PlanStep, PlanStatus as PlannerPlanStatus};
    
    fn create_test_plan(id_suffix: &str) -> Plan {
        let mut plan = Plan::new(
            &format!("Test Plan {}", id_suffix),
            &format!("A test plan {}", id_suffix),
            "Test goal",
            "test-user",
        );
        plan.id = format!("test-plan-{}", id_suffix);
        plan.status = PlannerPlanStatus::Ready;
        let mut step = PlanStep::new("Test Step", "A test step", "action");
        step.id = format!("step-1-{}", id_suffix);
        step.estimated_duration_us = Some(10_000_000);
        plan.add_step(step);
        plan.created_at = 1_700_000_000 * 90_000;
        plan.updated_at = plan.created_at;
        plan.tags = vec!["test".to_string()];
        plan
    }
    
    #[tokio::test]
    async fn test_canonicalization_deterministic() {
        let plan1 = create_test_plan("canon");
        let plan2 = create_test_plan("canon");
        
        let id1 = PlanStore::generate_plan_id(&plan1).unwrap();
        let id2 = PlanStore::generate_plan_id(&plan2).unwrap();
        
        assert_eq!(id1, id2, "Same plans should have same IDs");
    }
    
    #[tokio::test]
    async fn test_create_and_fetch_plan() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.ndjson");
        let store = PlanStore::new(&log_path).await.unwrap();
        
        let plan = create_test_plan("fetch");
        let record = store.create_plan(plan.clone(), "test-user".to_string()).await.unwrap();
        
        let fetched = store.get_plan(&record.metadata.plan_id).unwrap();
        assert_eq!(fetched.metadata.plan_id, record.metadata.plan_id);
        assert_eq!(fetched.plan.name, plan.name);
    }
    
    #[tokio::test]
    async fn test_idempotent_create() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.ndjson");
        let store = PlanStore::new(&log_path).await.unwrap();
        
        let plan = create_test_plan("idempotent");
        let record1 = store.create_plan(plan.clone(), "test-user".to_string()).await.unwrap();
        let record2 = store.create_plan(plan, "test-user".to_string()).await.unwrap();
        
        assert_eq!(record1.metadata.plan_id, record2.metadata.plan_id);
        assert_eq!(record1.metadata.created_at, record2.metadata.created_at);
    }
    
    #[tokio::test]
    async fn test_approve_plan() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.ndjson");
        let store = PlanStore::new(&log_path).await.unwrap();
        
        let plan = create_test_plan("approve");
        let record = store.create_plan(plan, "test-user".to_string()).await.unwrap();
        
        let approved = store.approve_plan(
            &record.metadata.plan_id,
            None,
            "approver".to_string(),
            None,
        ).await.unwrap();
        
        assert_eq!(approved.metadata.status, PlanStatus::Approved);
        assert_eq!(approved.approved_by, Some("approver".to_string()));
    }
    
    #[tokio::test]
    async fn test_idempotent_approve() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.ndjson");
        let store = PlanStore::new(&log_path).await.unwrap();
        
        let plan = create_test_plan("idempotent-approve");
        let record = store.create_plan(plan, "test-user".to_string()).await.unwrap();
        
        let approved1 = store.approve_plan(
            &record.metadata.plan_id,
            None,
            "approver".to_string(),
            None,
        ).await.unwrap();
        
        let approved2 = store.approve_plan(
            &record.metadata.plan_id,
            None,
            "approver".to_string(),
            None,
        ).await.unwrap();
        
        assert_eq!(approved1.approved_at, approved2.approved_at);
    }
    
    #[tokio::test]
    async fn test_replay_log() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.ndjson");
        
        // Create store and add plan
        {
            let store = PlanStore::new(&log_path).await.unwrap();
            let plan = create_test_plan("replay");
            let record = store.create_plan(plan, "test-user".to_string()).await.unwrap();
            store.approve_plan(&record.metadata.plan_id, None, "approver".to_string(), None).await.unwrap();
        }
        
        // Create new store (simulates restart)
        let store2 = PlanStore::new(&log_path).await.unwrap();
        assert_eq!(store2.index.len(), 1);
        
        let plan_id = store2.index.iter().next().unwrap().key().clone();
        let record = store2.get_plan(&plan_id).unwrap();
        assert_eq!(record.metadata.status, PlanStatus::Approved);
    }
    
    #[tokio::test]
    async fn test_list_pagination() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.ndjson");
        let store = PlanStore::new(&log_path).await.unwrap();
        
        // Create multiple plans
        for i in 0..5 {
            let plan = create_test_plan(&format!("page-{:02}", i));
            store.create_plan(plan, "test-user".to_string()).await.unwrap();
        }
        
        // Test pagination
        let query = ListQuery {
            status: None,
            limit: Some(2),
            cursor: None,
        };
        
        let response = store.list_plans(query);
        assert_eq!(response.plans.len(), 2);
        assert!(response.next_cursor.is_some());
        assert_eq!(response.total_count, 5);
    }
}
