//! Twin Snapshot Schema - Service State Snapshot System
//! 
//! This module provides a comprehensive system for creating, signing, and restoring
//! service state snapshots with restart protocol support.

pub mod snapshot;
pub mod sign;

pub use snapshot::{
    ServiceStateSnapshot, SnapshotMetadata, SnapshotType, CompressionType, EncryptionType,
    ServiceHealthState, HealthStatus, HealthCheckDetail, Severity, ServiceMetadata,
    HealthHistoryEntry, ServiceDependencyState, Dependency, DependencyHealthSummary,
    ServiceConfigurationState, ServiceMetricsState, SnapshotValidationResult, SnapshotManager,
};

pub use sign::{
    Signature, SignatureAlgorithm, KeyPair, Signer, Verifier, DilithiumSigner,
    DilithiumVerifier, MockSigner, SignatureManager, SignatureError,
};

/// Twin service configuration
#[derive(Debug, Clone)]
pub struct TwinServiceConfig {
    /// Service name
    pub service_name: String,
    
    /// Service version
    pub service_version: String,
    
    /// Snapshot storage directory
    pub snapshot_dir: String,
    
    /// Maximum snapshot size in bytes
    pub max_snapshot_size: usize,
    
    /// Snapshot retention period in seconds
    pub snapshot_retention_seconds: u64,
    
    /// Compression enabled
    pub compression_enabled: bool,
    
    /// Encryption enabled
    pub encryption_enabled: bool,
    
    /// Signature algorithm
    pub signature_algorithm: SignatureAlgorithm,
    
    /// Auto-snapshot interval in seconds
    pub auto_snapshot_interval: u64,
    
    /// Health score threshold for snapshots
    pub health_score_threshold: f64,
}

impl Default for TwinServiceConfig {
    fn default() -> Self {
        Self {
            service_name: "twin-service".to_string(),
            service_version: "0.1.0".to_string(),
            snapshot_dir: "./snapshots".to_string(),
            max_snapshot_size: 10 * 1024 * 1024, // 10MB
            snapshot_retention_seconds: 7 * 24 * 60 * 60, // 7 days
            compression_enabled: true,
            encryption_enabled: false,
            signature_algorithm: SignatureAlgorithm::Dilithium3,
            auto_snapshot_interval: 300, // 5 minutes
            health_score_threshold: 70.0,
        }
    }
}

/// Twin service for managing snapshots
pub struct TwinService {
    config: TwinServiceConfig,
    snapshot_manager: SnapshotManager,
    signature_manager: SignatureManager,
}

impl TwinService {
    /// Create a new twin service
    pub fn new(config: TwinServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let snapshot_manager = SnapshotManager::new();
        let signature_manager = SignatureManager::new();
        
        Ok(Self {
            config,
            snapshot_manager,
            signature_manager,
        })
    }
    
    /// Create a snapshot of service state
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
        additional_state: HashMap<String, serde_json::Value>,
        snapshot_type: SnapshotType,
        description: Option<String>,
        author: Option<String>,
        parent_id: Option<String>,
    ) -> Result<ServiceStateSnapshot, Box<dyn std::error::Error>> {
        let snapshot = self.snapshot_manager.create_snapshot(
            service_id,
            service_version,
            instance_id,
            environment,
            region,
            health_state,
            dependency_state,
            configuration_state,
            metrics_state,
            additional_state,
            snapshot_type,
            description,
            author,
            parent_id,
        )?;
        
        Ok(snapshot)
    }
    
    /// Sign a snapshot
    pub fn sign_snapshot(
        &self,
        snapshot: &ServiceStateSnapshot,
        key_id: &str,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let snapshot_data = serde_json::to_vec(snapshot)?;
        let signature = self.signature_manager.sign_data(key_id, &snapshot_data)?;
        
        Ok(signature)
    }
    
    /// Restore service state from snapshot
    pub fn restore_from_snapshot(
        &self,
        snapshot: &ServiceStateSnapshot,
        key_id: &str,
    ) -> Result<ServiceStateSnapshot, Box<dyn std::error::Error>> {
        let restored = self.snapshot_manager.restore_from_snapshot(snapshot, &self.signature_manager)?;
        
        Ok(restored)
    }
    
    /// Validate snapshot
    pub fn validate_snapshot(
        &self,
        snapshot: &ServiceStateSnapshot,
    ) -> Result<SnapshotValidationResult, Box<dyn std::error::Error>> {
        let validation = self.snapshot_manager.validate_snapshot(snapshot)?;
        
        Ok(validation)
    }
    
    /// Get JSON schema
    pub fn get_json_schema(&self) -> serde_json::Value {
        self.snapshot_manager.get_json_schema()
    }
    
    /// Add signer
    pub fn add_signer(&mut self, key_id: String, signer: Box<dyn Signer>) {
        self.signature_manager.add_signer(key_id, signer);
    }
    
    /// Add verifier
    pub fn add_verifier(&mut self, key_id: String, verifier: Box<dyn Verifier>) {
        self.signature_manager.add_verifier(key_id, verifier);
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &TwinServiceConfig {
        &self.config
    }
}

impl Default for TwinService {
    fn default() -> Self {
        let config = TwinServiceConfig::default();
        Self::new(config).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;
    
    #[test]
    fn test_twin_service_creation() {
        let config = TwinServiceConfig::default();
        let service = TwinService::new(config).unwrap();
        
        assert_eq!(service.get_config().service_name, "twin-service");
        assert_eq!(service.get_config().signature_algorithm, SignatureAlgorithm::Dilithium3);
    }
    
    #[test]
    fn test_snapshot_creation() {
        let mut service = TwinService::default();
        
        // Add a mock signer
        let mock_signer = MockSigner::new();
        service.add_signer("test-key".to_string(), Box::new(mock_signer));
        
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
            config_data: serde_json::json!({}),
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
        
        let snapshot = service.create_snapshot(
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
    fn test_snapshot_validation() {
        let service = TwinService::default();
        
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
            config_data: serde_json::json!({}),
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
        
        let validation = service.validate_snapshot(&snapshot).unwrap();
        assert!(validation.is_valid);
        assert!(validation.health_score_consistent);
    }
}
