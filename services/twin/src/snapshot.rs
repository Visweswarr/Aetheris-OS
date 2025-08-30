use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use thiserror::Error;

use crate::sign::{Signature, Signer, Verifier};

/// Error types for snapshot operations
#[derive(Error, Debug)]
pub enum SnapshotError {
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Signature verification failed: {0}")]
    SignatureError(#[from] crate::sign::SignatureError),
    
    #[error("Schema validation failed: {0}")]
    SchemaValidationError(String),
    
    #[error("Snapshot version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },
    
    #[error("Snapshot timestamp is in the future: {timestamp}")]
    FutureTimestamp { timestamp: u64 },
    
    #[error("Snapshot has expired: {timestamp}, current: {current}")]
    ExpiredSnapshot { timestamp: u64, current: u64 },
    
    #[error("Invalid health score: {score} (must be 0.0-100.0)")]
    InvalidHealthScore { score: f64 },
    
    #[error("Invalid service ID: {id}")]
    InvalidServiceId { id: String },
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid dependency structure")]
    InvalidDependencyStructure,
    
    #[error("Snapshot size exceeds limit: {size} bytes")]
    SizeLimitExceeded { size: usize },
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub id: String,
    pub version: String,
    pub timestamp: u64,
    pub expires_at: u64,
    pub service_id: String,
    pub service_version: String,
    pub instance_id: String,
    pub environment: String,
    pub region: String,
    pub snapshot_type: SnapshotType,
    pub compression: CompressionType,
    pub encryption: Option<EncryptionType>,
    pub checksum: String,
    pub size_bytes: usize,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
}

/// Snapshot types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SnapshotType {
    Full,
    Incremental,
    HealthOnly,
    Configuration,
    State,
    Metrics,
}

/// Compression types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
    Zstd,
}

/// Encryption types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionType {
    None,
    AES256,
    ChaCha20Poly1305,
}

/// Service health state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthState {
    pub service_id: String,
    pub health_status: HealthStatus,
    pub health_score: f64,
    pub last_check: u64,
    pub check_duration_ms: u64,
    pub error_message: Option<String>,
    pub health_details: Vec<HealthCheckDetail>,
    pub metadata: ServiceMetadata,
    pub health_history: Vec<HealthHistoryEntry>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
    Maintenance,
    Unknown,
}

/// Health check detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckDetail {
    pub check_name: String,
    pub status: HealthStatus,
    pub description: String,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub data: HashMap<String, Value>,
    pub severity: Severity,
    pub timestamp: u64,
}

/// Severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    pub name: String,
    pub version: String,
    pub instance_id: String,
    pub environment: String,
    pub region: String,
    pub tags: Vec<String>,
    pub owner: String,
    pub description: String,
    pub contact: String,
    pub documentation_url: String,
}

/// Health history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthHistoryEntry {
    pub timestamp: u64,
    pub status: HealthStatus,
    pub health_score: f64,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

/// Service dependency state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependencyState {
    pub service_id: String,
    pub dependencies: Vec<Dependency>,
    pub health_summary: DependencyHealthSummary,
    pub dot_graph: String,
    pub max_depth: u32,
    pub total_dependencies: u32,
    pub critical_path: Vec<String>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub service_id: String,
    pub name: String,
    pub dependency_type: String,
    pub status: HealthStatus,
    pub health_score: f64,
    pub endpoint: String,
    pub timeout_seconds: u32,
    pub weight: f64,
    pub last_check: u64,
    pub metadata: ServiceMetadata,
}

/// Dependency health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealthSummary {
    pub total_count: u32,
    pub healthy_count: u32,
    pub degraded_count: u32,
    pub unhealthy_count: u32,
    pub critical_count: u32,
    pub overall_score: f64,
    pub critical_dependencies: Vec<String>,
    pub failed_dependencies: Vec<String>,
}

/// Service configuration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfigurationState {
    pub service_id: String,
    pub config_version: String,
    pub config_hash: String,
    pub config_data: Value,
    pub environment_variables: HashMap<String, String>,
    pub metadata: HashMap<String, Value>,
    pub last_updated: u64,
    pub source: String,
}

/// Service metrics state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetricsState {
    pub service_id: String,
    pub timestamp: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_io_bps: f64,
    pub request_rate_rps: f64,
    pub error_rate_eps: f64,
    pub response_time_ms: f64,
    pub active_connections: u32,
    pub queue_depth: u32,
    pub custom_metrics: HashMap<String, f64>,
}

/// Complete service state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStateSnapshot {
    pub metadata: SnapshotMetadata,
    pub health_state: ServiceHealthState,
    pub dependency_state: ServiceDependencyState,
    pub configuration_state: ServiceConfigurationState,
    pub metrics_state: ServiceMetricsState,
    pub additional_state: HashMap<String, Value>,
    pub signature: Option<Signature>,
    pub schema_version: String,
}

/// Snapshot validation result
#[derive(Debug, Clone)]
pub struct SnapshotValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub health_score_consistent: bool,
    pub expected_health_score: f64,
    pub actual_health_score: f64,
}

/// Snapshot manager
pub struct SnapshotManager {
    schema_version: String,
    max_snapshot_size: usize,
    snapshot_expiration_seconds: u64,
    compression_enabled: bool,
    encryption_enabled: bool,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            schema_version: "1.0.0".to_string(),
            max_snapshot_size: 10 * 1024 * 1024, // 10MB
            snapshot_expiration_seconds: 24 * 60 * 60, // 24 hours
            compression_enabled: true,
            encryption_enabled: false,
        }
    }
    
    pub fn create_snapshot(
        &self,
        service_id: &str,
        service_version: &str,
        instance_id: &str,
        environment: &str,
        region: &str,
        health_state: ServiceHealthState,
        dependency_state: ServiceDependencyState,
        configuration_state: ServiceConfigurationState,
        metrics_state: ServiceMetricsState,
        additional_state: HashMap<String, Value>,
        snapshot_type: SnapshotType,
        description: Option<String>,
        author: Option<String>,
        parent_id: Option<String>,
    ) -> Result<ServiceStateSnapshot, SnapshotError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let expires_at = if self.snapshot_expiration_seconds > 0 {
            now + self.snapshot_expiration_seconds
        } else {
            0
        };
        
        let metadata = SnapshotMetadata {
            id: Uuid::new_v4().to_string(),
            version: self.schema_version.clone(),
            timestamp: now,
            expires_at,
            service_id: service_id.to_string(),
            service_version: service_version.to_string(),
            instance_id: instance_id.to_string(),
            environment: environment.to_string(),
            region: region.to_string(),
            snapshot_type,
            compression: if self.compression_enabled { CompressionType::Gzip } else { CompressionType::None },
            encryption: if self.encryption_enabled { Some(EncryptionType::AES256) } else { None },
            checksum: String::new(),
            size_bytes: 0,
            tags: vec!["twin".to_string(), "snapshot".to_string()],
            description,
            author,
            parent_id,
            children: Vec::new(),
        };
        
        let snapshot = ServiceStateSnapshot {
            metadata,
            health_state,
            dependency_state,
            configuration_state,
            metrics_state,
            additional_state,
            signature: None,
            schema_version: self.schema_version.clone(),
        };
        
        self.validate_snapshot(&snapshot)?;
        Ok(snapshot)
    }
    
    pub fn validate_snapshot(&self, snapshot: &ServiceStateSnapshot) -> Result<SnapshotValidationResult, SnapshotError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if snapshot.schema_version != self.schema_version {
            return Err(SnapshotError::VersionMismatch {
                expected: self.schema_version.clone(),
                actual: snapshot.schema_version.clone(),
            });
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if snapshot.metadata.timestamp > now {
            return Err(SnapshotError::FutureTimestamp {
                timestamp: snapshot.metadata.timestamp,
            });
        }
        
        if snapshot.metadata.expires_at > 0 && snapshot.metadata.expires_at < now {
            return Err(SnapshotError::ExpiredSnapshot {
                timestamp: snapshot.metadata.expires_at,
                current: now,
            });
        }
        
        if !(0.0..=100.0).contains(&snapshot.health_state.health_score) {
            errors.push(format!("Invalid health score: {}", snapshot.health_state.health_score));
        }
        
        if snapshot.metadata.service_id != snapshot.health_state.service_id {
            errors.push("Service ID mismatch between metadata and health state".to_string());
        }
        
        let snapshot_json = serde_json::to_string(snapshot)?;
        let size = snapshot_json.len();
        if size > self.max_snapshot_size {
            return Err(SnapshotError::SizeLimitExceeded { size });
        }
        
        let expected_health_score = self.calculate_expected_health_score(&snapshot.dependency_state);
        let health_score_consistent = (expected_health_score - snapshot.health_state.health_score).abs() < 1.0;
        
        if !health_score_consistent {
            warnings.push(format!(
                "Health score inconsistency: expected {:.2}, got {:.2}",
                expected_health_score, snapshot.health_state.health_score
            ));
        }
        
        Ok(SnapshotValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            health_score_consistent,
            expected_health_score,
            actual_health_score: snapshot.health_state.health_score,
        })
    }
    
    fn calculate_expected_health_score(&self, dependency_state: &ServiceDependencyState) -> f64 {
        if dependency_state.dependencies.is_empty() {
            return 100.0;
        }
        
        let total_weight: f64 = dependency_state.dependencies.iter().map(|d| d.weight).sum();
        if total_weight == 0.0 {
            return 100.0;
        }
        
        let weighted_score: f64 = dependency_state
            .dependencies
            .iter()
            .map(|d| d.health_score * d.weight)
            .sum();
        
        weighted_score / total_weight
    }
    
    pub fn restore_from_snapshot(
        &self,
        snapshot: &ServiceStateSnapshot,
        signer: &dyn Verifier,
    ) -> Result<ServiceStateSnapshot, SnapshotError> {
        if let Some(signature) = &snapshot.signature {
            signer.verify_signature(snapshot, signature)?;
        }
        
        let validation = self.validate_snapshot(snapshot)?;
        if !validation.is_valid {
            return Err(SnapshotError::SchemaValidationError(
                format!("Snapshot validation failed: {:?}", validation.errors)
            ));
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let restored_metadata = SnapshotMetadata {
            id: Uuid::new_v4().to_string(),
            version: self.schema_version.clone(),
            timestamp: now,
            expires_at: if self.snapshot_expiration_seconds > 0 {
                now + self.snapshot_expiration_seconds
            } else {
                0
            },
            service_id: snapshot.metadata.service_id.clone(),
            service_version: snapshot.metadata.service_version.clone(),
            instance_id: snapshot.metadata.instance_id.clone(),
            environment: snapshot.metadata.environment.clone(),
            region: snapshot.metadata.region.clone(),
            snapshot_type: SnapshotType::Full,
            compression: snapshot.metadata.compression.clone(),
            encryption: snapshot.metadata.encryption.clone(),
            checksum: String::new(),
            size_bytes: 0,
            tags: vec!["twin".to_string(), "restored".to_string()],
            description: Some("Restored from snapshot".to_string()),
            author: snapshot.metadata.author.clone(),
            parent_id: Some(snapshot.metadata.id.clone()),
            children: Vec::new(),
        };
        
        let restored_snapshot = ServiceStateSnapshot {
            metadata: restored_metadata,
            health_state: snapshot.health_state.clone(),
            dependency_state: snapshot.dependency_state.clone(),
            configuration_state: snapshot.configuration_state.clone(),
            metrics_state: snapshot.metrics_state.clone(),
            additional_state: snapshot.additional_state.clone(),
            signature: None,
            schema_version: self.schema_version.clone(),
        };
        
        Ok(restored_snapshot)
    }
    
    pub fn get_json_schema(&self) -> Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Service State Snapshot Schema",
            "description": "Schema for service state snapshots in Polymera OS",
            "type": "object",
            "required": ["metadata", "health_state", "dependency_state", "configuration_state", "metrics_state", "schema_version"],
            "properties": {
                "metadata": {
                    "type": "object",
                    "required": ["id", "version", "timestamp", "service_id", "service_version", "instance_id", "environment", "region", "snapshot_type"],
                    "properties": {
                        "id": { "type": "string", "format": "uuid" },
                        "version": { "type": "string" },
                        "timestamp": { "type": "integer", "minimum": 0 },
                        "service_id": { "type": "string", "minLength": 1 },
                        "health_score": { "type": "number", "minimum": 0.0, "maximum": 100.0 }
                    }
                }
            }
        })
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sign::MockSigner;
    
    #[test]
    fn test_create_snapshot() {
        let manager = SnapshotManager::new();
        
        let health_state = ServiceHealthState {
            service_id: "test-service".to_string(),
            health_status: HealthStatus::Healthy,
            health_score: 95.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            check_duration_ms: 50,
            error_message: None,
            health_details: Vec::new(),
            metadata: ServiceMetadata {
                name: "Test Service".to_string(),
                version: "1.0.0".to_string(),
                instance_id: "test-instance".to_string(),
                environment: "development".to_string(),
                region: "local".to_string(),
                tags: Vec::new(),
                owner: "Test Team".to_string(),
                description: "Test service for testing".to_string(),
                contact: "test@example.com".to_string(),
                documentation_url: "https://example.com/docs".to_string(),
            },
            health_history: Vec::new(),
        };
        
        let dependency_state = ServiceDependencyState {
            service_id: "test-service".to_string(),
            dependencies: Vec::new(),
            health_summary: DependencyHealthSummary {
                total_count: 0,
                healthy_count: 0,
                degraded_count: 0,
                unhealthy_count: 0,
                critical_count: 0,
                overall_score: 100.0,
                critical_dependencies: Vec::new(),
                failed_dependencies: Vec::new(),
            },
            dot_graph: "digraph G {}".to_string(),
            max_depth: 0,
            total_dependencies: 0,
            critical_path: Vec::new(),
        };
        
        let configuration_state = ServiceConfigurationState {
            service_id: "test-service".to_string(),
            config_version: "1.0.0".to_string(),
            config_hash: "abc123".to_string(),
            config_data: json!({}),
            environment_variables: HashMap::new(),
            metadata: HashMap::new(),
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            source: "file".to_string(),
        };
        
        let metrics_state = ServiceMetricsState {
            service_id: "test-service".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            cpu_usage_percent: 25.0,
            memory_usage_percent: 45.0,
            disk_usage_percent: 30.0,
            network_io_bps: 1024.0,
            request_rate_rps: 150.0,
            error_rate_eps: 0.5,
            response_time_ms: 45.0,
            active_connections: 25,
            queue_depth: 5,
            custom_metrics: HashMap::new(),
        };
        
        let snapshot = manager.create_snapshot(
            "test-service",
            "1.0.0",
            "test-instance",
            "development",
            "local",
            health_state,
            dependency_state,
            configuration_state,
            metrics_state,
            HashMap::new(),
            SnapshotType::Full,
            Some("Test snapshot".to_string()),
            Some("test-user".to_string()),
            None,
        ).unwrap();
        
        assert_eq!(snapshot.metadata.service_id, "test-service");
        assert_eq!(snapshot.health_state.health_score, 95.0);
        assert_eq!(snapshot.schema_version, "1.0.0");
    }
    
    #[test]
    fn test_validate_snapshot() {
        let manager = SnapshotManager::new();
        
        // Create a valid snapshot
        let health_state = ServiceHealthState {
            service_id: "test-service".to_string(),
            health_status: HealthStatus::Healthy,
            health_score: 95.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            check_duration_ms: 50,
            error_message: None,
            health_details: Vec::new(),
            metadata: ServiceMetadata {
                name: "Test Service".to_string(),
                version: "1.0.0".to_string(),
                instance_id: "test-instance".to_string(),
                environment: "development".to_string(),
                region: "local".to_string(),
                tags: Vec::new(),
                owner: "Test Team".to_string(),
                description: "Test service for testing".to_string(),
                contact: "test@example.com".to_string(),
                documentation_url: "https://example.com/docs".to_string(),
            },
            health_history: Vec::new(),
        };
        
        let dependency_state = ServiceDependencyState {
            service_id: "test-service".to_string(),
            dependencies: Vec::new(),
            health_summary: DependencyHealthSummary {
                total_count: 0,
                healthy_count: 0,
                degraded_count: 0,
                unhealthy_count: 0,
                critical_count: 0,
                overall_score: 100.0,
                critical_dependencies: Vec::new(),
                failed_dependencies: Vec::new(),
            },
            dot_graph: "digraph G {}".to_string(),
            max_depth: 0,
            total_dependencies: 0,
            critical_path: Vec::new(),
        };
        
        let configuration_state = ServiceConfigurationState {
            service_id: "test-service".to_string(),
            config_version: "1.0.0".to_string(),
            config_hash: "abc123".to_string(),
            config_data: json!({}),
            environment_variables: HashMap::new(),
            metadata: HashMap::new(),
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            source: "file".to_string(),
        };
        
        let metrics_state = ServiceMetricsState {
            service_id: "test-service".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            cpu_usage_percent: 25.0,
            memory_usage_percent: 45.0,
            disk_usage_percent: 30.0,
            network_io_bps: 1024.0,
            request_rate_rps: 150.0,
            error_rate_eps: 0.5,
            response_time_ms: 45.0,
            active_connections: 25,
            queue_depth: 5,
            custom_metrics: HashMap::new(),
        };
        
        let snapshot = ServiceStateSnapshot {
            metadata: SnapshotMetadata {
                id: Uuid::new_v4().to_string(),
                version: "1.0.0".to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                expires_at: 0,
                service_id: "test-service".to_string(),
                service_version: "1.0.0".to_string(),
                instance_id: "test-instance".to_string(),
                environment: "development".to_string(),
                region: "local".to_string(),
                snapshot_type: SnapshotType::Full,
                compression: CompressionType::None,
                encryption: None,
                checksum: String::new(),
                size_bytes: 0,
                tags: Vec::new(),
                description: None,
                author: None,
                parent_id: None,
                children: Vec::new(),
            },
            health_state,
            dependency_state,
            configuration_state,
            metrics_state,
            additional_state: HashMap::new(),
            signature: None,
            schema_version: "1.0.0".to_string(),
        };
        
        let validation = manager.validate_snapshot(&snapshot).unwrap();
        assert!(validation.is_valid);
        assert!(validation.health_score_consistent);
    }
    
    #[test]
    fn test_restore_from_snapshot() {
        let manager = SnapshotManager::new();
        let mock_signer = MockSigner::new();
        
        let health_state = ServiceHealthState {
            service_id: "test-service".to_string(),
            health_status: HealthStatus::Healthy,
            health_score: 95.0,
            last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            check_duration_ms: 50,
            error_message: None,
            health_details: Vec::new(),
            metadata: ServiceMetadata {
                name: "Test Service".to_string(),
                version: "1.0.0".to_string(),
                instance_id: "test-instance".to_string(),
                environment: "development".to_string(),
                region: "local".to_string(),
                tags: Vec::new(),
                owner: "Test Team".to_string(),
                description: "Test service for testing".to_string(),
                contact: "test@example.com".to_string(),
                documentation_url: "https://example.com/docs".to_string(),
            },
            health_history: Vec::new(),
        };
        
        let dependency_state = ServiceDependencyState {
            service_id: "test-service".to_string(),
            dependencies: Vec::new(),
            health_summary: DependencyHealthSummary {
                total_count: 0,
                healthy_count: 0,
                degraded_count: 0,
                unhealthy_count: 0,
                critical_count: 0,
                overall_score: 100.0,
                critical_dependencies: Vec::new(),
                failed_dependencies: Vec::new(),
            },
            dot_graph: "digraph G {}".to_string(),
            max_depth: 0,
            total_dependencies: 0,
            critical_path: Vec::new(),
        };
        
        let configuration_state = ServiceConfigurationState {
            service_id: "test-service".to_string(),
            config_version: "1.0.0".to_string(),
            config_hash: "abc123".to_string(),
            config_data: json!({}),
            environment_variables: HashMap::new(),
            metadata: HashMap::new(),
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            source: "file".to_string(),
        };
        
        let metrics_state = ServiceMetricsState {
            service_id: "test-service".to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            cpu_usage_percent: 25.0,
            memory_usage_percent: 45.0,
            disk_usage_percent: 30.0,
            network_io_bps: 1024.0,
            request_rate_rps: 150.0,
            error_rate_eps: 0.5,
            response_time_ms: 45.0,
            active_connections: 25,
            queue_depth: 5,
            custom_metrics: HashMap::new(),
        };
        
        let snapshot = ServiceStateSnapshot {
            metadata: SnapshotMetadata {
                id: Uuid::new_v4().to_string(),
                version: "1.0.0".to_string(),
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                expires_at: 0,
                service_id: "test-service".to_string(),
                service_version: "1.0.0".to_string(),
                instance_id: "test-instance".to_string(),
                environment: "development".to_string(),
                region: "local".to_string(),
                snapshot_type: SnapshotType::Full,
                compression: CompressionType::None,
                encryption: None,
                checksum: String::new(),
                size_bytes: 0,
                tags: Vec::new(),
                description: None,
                author: None,
                parent_id: None,
                children: Vec::new(),
            },
            health_state,
            dependency_state,
            configuration_state,
            metrics_state,
            additional_state: HashMap::new(),
            signature: None,
            schema_version: "1.0.0".to_string(),
        };
        
        let restored = manager.restore_from_snapshot(&snapshot, &mock_signer).unwrap();
        
        assert_eq!(restored.health_state.health_score, 95.0);
        assert_eq!(restored.metadata.snapshot_type, SnapshotType::Full);
        assert!(restored.metadata.tags.contains(&"restored".to_string()));
    }
}
