//! Session Key Types for Polymera OS
//! 
//! This module provides purpose-bound, time-scoped, and amount-limited session keys
//! with comprehensive validation and constraint checking.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// Errors that can occur during session key operations
#[derive(Debug, Error)]
pub enum SessionKeyError {
    #[error("Invalid session scope: {0}")]
    InvalidScope(String),
    
    #[error("Time constraint violation: {0}")]
    TimeConstraintViolation(String),
    
    #[error("Amount constraint violation: {0}")]
    AmountConstraintViolation(String),
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Quota exceeded")]
    QuotaExceeded,
}

/// Session key types and purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    Auth,
    ApiRead,
    ApiWrite,
    DataAccess,
    Network,
    Device,
    Custom,
}

impl SessionType {
    /// Get allowed operations for this session type
    pub fn allowed_operations(&self) -> Vec<String> {
        match self {
            SessionType::Auth => vec!["authenticate".to_string(), "verify".to_string()],
            SessionType::ApiRead => vec!["read".to_string(), "query".to_string(), "list".to_string()],
            SessionType::ApiWrite => vec!["create".to_string(), "update".to_string(), "delete".to_string()],
            SessionType::DataAccess => vec!["read".to_string(), "write".to_string(), "delete".to_string()],
            SessionType::Network => vec!["connect".to_string(), "transfer".to_string(), "monitor".to_string()],
            SessionType::Device => vec!["access".to_string(), "control".to_string(), "configure".to_string()],
            SessionType::Custom => vec!["custom".to_string()],
        }
    }
    
    /// Check if operation is allowed for this session type
    pub fn allows_operation(&self, operation: &str) -> bool {
        self.allowed_operations().contains(&operation.to_string())
    }
}

/// Geographic constraints for session scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicConstraints {
    pub allowed_countries: Vec<String>,
    pub allowed_regions: Vec<String>,
    pub allowed_cities: Vec<String>,
    pub radius_constraint: Option<GeographicRadius>,
    pub allowed_timezones: Vec<String>,
}

impl GeographicConstraints {
    /// Check if location is within constraints
    pub fn is_location_allowed(&self, country: &str, region: &str, city: &str, timezone: &str) -> bool {
        let country_allowed = self.allowed_countries.is_empty() || self.allowed_countries.contains(&country.to_string());
        let region_allowed = self.allowed_regions.is_empty() || self.allowed_regions.contains(&region.to_string());
        let city_allowed = self.allowed_cities.is_empty() || self.allowed_cities.contains(&city.to_string());
        let timezone_allowed = self.allowed_timezones.is_empty() || self.allowed_timezones.contains(&timezone.to_string());
        
        country_allowed && region_allowed && city_allowed && timezone_allowed
    }
}

/// Geographic radius constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicRadius {
    pub latitude: f64,
    pub longitude: f64,
    pub radius_km: f64,
}

/// Network constraints for session scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConstraints {
    pub allowed_ip_ranges: Vec<String>,
    pub allowed_network_types: Vec<NetworkType>,
    pub allowed_domains: Vec<String>,
    pub vpn_requirement: Option<VpnRequirement>,
}

impl NetworkConstraints {
    /// Check if network access is allowed
    pub fn is_network_allowed(&self, ip: &str, network_type: NetworkType, domain: &str, is_vpn: bool) -> bool {
        let ip_allowed = self.allowed_ip_ranges.is_empty() || self.is_ip_in_ranges(ip);
        let network_allowed = self.allowed_network_types.is_empty() || self.allowed_network_types.contains(&network_type);
        let domain_allowed = self.allowed_domains.is_empty() || self.allowed_domains.contains(&domain.to_string());
        
        let vpn_ok = if let Some(vpn_req) = &self.vpn_requirement {
            is_vpn && vpn_req.is_satisfied()
        } else {
            true
        };
        
        ip_allowed && network_allowed && domain_allowed && vpn_ok
    }
    
    /// Check if IP is within allowed ranges
    fn is_ip_in_ranges(&self, ip: &str) -> bool {
        self.allowed_ip_ranges.iter().any(|range| {
            if range.contains('/') {
                self.is_ip_in_cidr(ip, range)
            } else {
                ip == range
            }
        })
    }
    
    /// Check if IP is within CIDR range
    fn is_ip_in_cidr(&self, ip: &str, cidr: &str) -> bool {
        if let Some((network, _prefix)) = cidr.split_once('/') {
            ip.starts_with(network)
        } else {
            false
        }
    }
}

/// Network types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    Wifi,
    Ethernet,
    Mobile,
    Vpn,
    Corporate,
}

/// VPN requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnRequirement {
    pub vpn_provider: String,
    pub vpn_location: String,
    pub vpn_protocol: VpnProtocol,
}

impl VpnRequirement {
    /// Check if VPN requirement is satisfied
    pub fn is_satisfied(&self) -> bool {
        !self.vpn_provider.is_empty() && !self.vpn_location.is_empty()
    }
}

/// VPN protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VpnProtocol {
    OpenVpn,
    WireGuard,
    IkeV2,
}

/// Device constraints for session scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConstraints {
    pub allowed_device_types: Vec<DeviceType>,
    pub allowed_manufacturers: Vec<String>,
    pub required_security_features: Vec<DeviceSecurityFeature>,
}

impl DeviceConstraints {
    /// Check if device is allowed
    pub fn is_device_allowed(&self, device_type: DeviceType, manufacturer: &str, security_features: &[DeviceSecurityFeature]) -> bool {
        let type_allowed = self.allowed_device_types.is_empty() || self.allowed_device_types.contains(&device_type);
        let manufacturer_allowed = self.allowed_manufacturers.is_empty() || self.allowed_manufacturers.contains(&manufacturer.to_string());
        
        let security_ok = self.required_security_features.iter().all(|required| {
            security_features.contains(required)
        });
        
        type_allowed && manufacturer_allowed && security_ok
    }
}

/// Device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Server,
}

/// Device security features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceSecurityFeature {
    Biometric,
    Tee,
    SecureBoot,
}

/// Data constraints for session scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConstraints {
    pub allowed_data_types: Vec<DataType>,
    pub allowed_classifications: Vec<DataClassification>,
    pub allowed_data_sources: Vec<String>,
}

impl DataConstraints {
    /// Check if data access is allowed
    pub fn is_data_access_allowed(&self, data_type: DataType, classification: DataClassification, source: &str) -> bool {
        let type_allowed = self.allowed_data_types.is_empty() || self.allowed_data_types.contains(&data_type);
        let classification_allowed = self.allowed_classifications.is_empty() || self.allowed_classifications.contains(&classification);
        let source_allowed = self.allowed_data_sources.is_empty() || self.allowed_data_sources.contains(&source.to_string());
        
        type_allowed && classification_allowed && source_allowed
    }
}

/// Data types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Personal,
    Financial,
    Health,
    Technical,
}

/// Data classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

/// Session scope and access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionScope {
    pub resources: Vec<String>,
    pub actions: Vec<String>,
    pub geographic: GeographicConstraints,
    pub network: NetworkConstraints,
    pub device: DeviceConstraints,
    pub data: DataConstraints,
    pub custom_attributes: HashMap<String, String>,
}

impl SessionScope {
    /// Check if operation is allowed within scope
    pub fn is_operation_allowed(
        &self,
        action: &str,
        resource: &str,
        context: &ValidationContext,
    ) -> Result<bool, SessionKeyError> {
        // Check if action is allowed
        if !self.actions.is_empty() && !self.actions.contains(&action.to_string()) {
            return Err(SessionKeyError::InvalidScope(format!("Action '{}' not allowed", action)));
        }
        
        // Check if resource is allowed
        if !self.resources.is_empty() && !self.resources.contains(&resource.to_string()) {
            return Err(SessionKeyError::InvalidScope(format!("Resource '{}' not allowed", resource)));
        }
        
        // Check geographic constraints
        if !self.geographic.is_location_allowed(
            &context.current_location.country_code,
            &context.current_location.region,
            &context.current_location.city,
            &context.current_location.timezone,
        ) {
            return Err(SessionKeyError::InvalidScope("Geographic location not allowed".to_string()));
        }
        
        // Check network constraints
        if !self.network.is_network_allowed(
            &context.current_network.ip_address,
            context.current_network.network_type,
            &context.current_network.domain,
            context.current_network.is_vpn,
        ) {
            return Err(SessionKeyError::InvalidScope("Network access not allowed".to_string()));
        }
        
        // Check device constraints
        if !self.device.is_device_allowed(
            context.current_device.device_type,
            &context.current_device.manufacturer,
            &context.current_device.security_features,
        ) {
            return Err(SessionKeyError::InvalidScope("Device not allowed".to_string()));
        }
        
        Ok(true)
    }
}

/// Time-based constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConstraints {
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub max_duration: Option<Duration>,
    pub allowed_windows: Vec<TimeWindow>,
    pub required_timezone: Option<String>,
}

impl TimeConstraints {
    /// Check if current time is within constraints
    pub fn is_time_valid(&self, current_time: DateTime<Utc>) -> Result<bool, SessionKeyError> {
        // Check if session has started
        if current_time < self.start_time {
            return Err(SessionKeyError::TimeConstraintViolation("Session has not started yet".to_string()));
        }
        
        // Check if session has ended
        if let Some(end_time) = self.end_time {
            if current_time > end_time {
                return Err(SessionKeyError::SessionExpired);
            }
        }
        
        Ok(true)
    }
    
    /// Get remaining time
    pub fn get_remaining_time(&self, current_time: DateTime<Utc>) -> Option<Duration> {
        self.end_time.map(|end| end - current_time)
    }
}

/// Time windows for allowed access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    pub day_of_week: u8, // 0 = Sunday, 6 = Saturday
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
    pub timezone: String,
}

/// Amount-based constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountConstraints {
    pub max_operations: Option<u64>,
    pub max_data_volume: Option<u64>,
    pub max_financial_amount: Option<u64>,
    pub max_sessions: Option<u32>,
    pub rate_limit: Option<RateLimit>,
    pub usage_quotas: Vec<UsageQuota>,
}

impl AmountConstraints {
    /// Check if operation is within constraints
    pub fn is_operation_allowed(&self, current_usage: &UsageStats) -> Result<bool, SessionKeyError> {
        // Check operation count
        if let Some(max_ops) = self.max_operations {
            if current_usage.operations_count >= max_ops {
                return Err(SessionKeyError::QuotaExceeded);
            }
        }
        
        // Check data volume
        if let Some(max_volume) = self.max_data_volume {
            if current_usage.data_volume_used >= max_volume {
                return Err(SessionKeyError::QuotaExceeded);
            }
        }
        
        // Check financial amount
        if let Some(max_amount) = self.max_financial_amount {
            if current_usage.financial_amount_used >= max_amount {
                return Err(SessionKeyError::QuotaExceeded);
            }
        }
        
        Ok(true)
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_window: u32,
    pub window_duration: Duration,
    pub burst_allowance: u32,
    pub algorithm: RateLimitAlgorithm,
}

/// Rate limiting algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    TokenBucket,
    LeakyBucket,
    FixedWindow,
}

/// Usage quota configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageQuota {
    pub quota_type: QuotaType,
    pub limit: u64,
    pub period: Duration,
}

/// Quota types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaType {
    Operations,
    DataVolume,
    FinancialAmount,
    Sessions,
}

/// Security information for session keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    pub algorithm: String,
    pub key_strength: u32,
    pub pq_security_level: PostQuantumSecurityLevel,
    pub required_factors: Vec<AuthenticationFactor>,
    pub security_policies: Vec<String>,
}

/// Post-quantum security levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostQuantumSecurityLevel {
    None,
    Level1,
    Level3,
    Level5,
}

/// Authentication factors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthenticationFactor {
    Password,
    Biometric,
    HardwareToken,
    SoftwareToken,
}

/// Session metadata and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub name: String,
    pub tags: Vec<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub version: String,
    pub custom_metadata: HashMap<String, String>,
}

/// Session state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub status: SessionStatus,
    pub current_usage: UsageStats,
    pub last_activity: DateTime<Utc>,
    pub errors: Vec<SessionError>,
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Paused,
    Suspended,
    Expired,
    Revoked,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub operations_count: u64,
    pub data_volume_used: u64,
    pub financial_amount_used: u64,
    pub session_duration: Duration,
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            operations_count: 0,
            data_volume_used: 0,
            financial_amount_used: 0,
            session_duration: Duration::zero(),
        }
    }
}

/// Session errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionError {
    pub error_code: String,
    pub error_message: String,
    pub error_field: String,
    pub severity: ValidationErrorSeverity,
}

/// Validation error severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Main session key structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    pub id: String,
    pub keystore_key_id: String,
    pub session_type: SessionType,
    pub scope: SessionScope,
    pub time_constraints: TimeConstraints,
    pub amount_constraints: AmountConstraints,
    pub security_info: SecurityInfo,
    pub metadata: SessionMetadata,
    pub state: SessionState,
}

impl SessionKey {
    /// Create a new session key
    pub fn new(
        keystore_key_id: String,
        session_type: SessionType,
        scope: SessionScope,
        time_constraints: TimeConstraints,
        amount_constraints: AmountConstraints,
        security_info: SecurityInfo,
        name: String,
        created_by: String,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4().to_string(),
            keystore_key_id,
            session_type,
            scope,
            time_constraints,
            amount_constraints,
            security_info,
            metadata: SessionMetadata {
                name,
                tags: Vec::new(),
                created_by,
                created_at: now,
                modified_at: now,
                version: "1.0.0".to_string(),
                custom_metadata: HashMap::new(),
            },
            state: SessionState {
                status: SessionStatus::Active,
                current_usage: UsageStats::default(),
                last_activity: now,
                errors: Vec::new(),
            },
        }
    }
    
    /// Check if operation is allowed
    pub fn is_operation_allowed(
        &self,
        action: &str,
        resource: &str,
        context: &ValidationContext,
    ) -> Result<bool, SessionKeyError> {
        // First validate the session
        self.validate(context)?;
        
        // Then check if operation is within scope
        self.scope.is_operation_allowed(action, resource, context)
    }
    
    /// Validate session key
    pub fn validate(&self, context: &ValidationContext) -> Result<(), SessionKeyError> {
        // Validate time constraints
        self.time_constraints.is_time_valid(context.current_time)?;
        
        // Validate amount constraints
        self.amount_constraints.is_operation_allowed(&self.state.current_usage)?;
        
        Ok(())
    }
}

/// Validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub current_time: DateTime<Utc>,
    pub current_location: GeographicLocation,
    pub current_network: NetworkInfo,
    pub current_device: DeviceInfo,
}

/// Geographic location
#[derive(Debug, Clone)]
pub struct GeographicLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,
    pub region: String,
    pub city: String,
    pub timezone: String,
}

/// Network information
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub ip_address: String,
    pub network_type: NetworkType,
    pub is_vpn: bool,
    pub domain: String,
}

/// Device information
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_type: DeviceType,
    pub manufacturer: String,
    pub model: String,
    pub operating_system: String,
    pub security_features: Vec<DeviceSecurityFeature>,
}

/// Session key validator
pub struct SessionKeyValidator;

impl SessionKeyValidator {
    /// Validate session key scope
    pub fn validate_scope(
        session_key: &SessionKey,
        action: &str,
        resource: &str,
        context: &ValidationContext,
    ) -> Result<bool, SessionKeyError> {
        session_key.is_operation_allowed(action, resource, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_session_type_operations() {
        let session_type = SessionType::ApiRead;
        assert!(session_type.allows_operation("read"));
        assert!(session_type.allows_operation("query"));
        assert!(!session_type.allows_operation("write"));
    }
    
    #[test]
    fn test_geographic_constraints() {
        let constraints = GeographicConstraints {
            allowed_countries: vec!["US".to_string()],
            allowed_regions: vec!["CA".to_string()],
            allowed_cities: vec![],
            radius_constraint: None,
            allowed_timezones: vec!["America/Los_Angeles".to_string()],
        };
        
        assert!(constraints.is_location_allowed("US", "CA", "San Francisco", "America/Los_Angeles"));
        assert!(!constraints.is_location_allowed("CA", "ON", "Toronto", "America/Toronto"));
    }
    
    #[test]
    fn test_invalid_scope_rejection() {
        // Create a session key with geographic constraints
        let scope = SessionScope {
            resources: vec!["test-resource".to_string()],
            actions: vec!["read".to_string()],
            geographic: GeographicConstraints {
                allowed_countries: vec!["US".to_string()],
                allowed_regions: vec![],
                allowed_cities: vec![],
                radius_constraint: None,
                allowed_timezones: vec![],
            },
            network: NetworkConstraints {
                allowed_ip_ranges: vec![],
                allowed_network_types: vec![],
                allowed_domains: vec![],
                vpn_requirement: None,
            },
            device: DeviceConstraints {
                allowed_device_types: vec![],
                allowed_manufacturers: vec![],
                required_security_features: vec![],
            },
            data: DataConstraints {
                allowed_data_types: vec![],
                allowed_classifications: vec![],
                allowed_data_sources: vec![],
            },
            custom_attributes: HashMap::new(),
        };
        
        let time_constraints = TimeConstraints {
            start_time: Utc::now() - Duration::hours(1),
            end_time: Some(Utc::now() + Duration::hours(1)),
            max_duration: None,
            allowed_windows: vec![],
            required_timezone: None,
        };
        
        let amount_constraints = AmountConstraints {
            max_operations: Some(100),
            max_data_volume: Some(1024),
            max_financial_amount: Some(1000),
            max_sessions: Some(1),
            rate_limit: None,
            usage_quotas: vec![],
        };
        
        let security_info = SecurityInfo {
            algorithm: "Ed25519".to_string(),
            key_strength: 256,
            pq_security_level: PostQuantumSecurityLevel::None,
            required_factors: vec![AuthenticationFactor::Password],
            security_policies: vec!["basic".to_string()],
        };
        
        let session_key = SessionKey::new(
            "test-keystore-key".to_string(),
            SessionType::ApiRead,
            scope,
            time_constraints,
            amount_constraints,
            security_info,
            "Test Session Key".to_string(),
            "test-user".to_string(),
        );
        
        // Create a context with invalid location (not US)
        let context = ValidationContext {
            current_time: Utc::now(),
            current_location: GeographicLocation {
                latitude: 51.5074,
                longitude: -0.1278,
                country_code: "GB".to_string(), // Not US
                region: "England".to_string(),
                city: "London".to_string(),
                timezone: "Europe/London".to_string(),
            },
            current_network: NetworkInfo {
                ip_address: "192.168.1.1".to_string(),
                network_type: NetworkType::Wifi,
                is_vpn: false,
                domain: "example.com".to_string(),
            },
            current_device: DeviceInfo {
                device_type: DeviceType::Desktop,
                manufacturer: "Generic".to_string(),
                model: "Desktop".to_string(),
                operating_system: "Linux".to_string(),
                security_features: vec![],
            },
        };
        
        // Should reject operation due to geographic constraint
        let result = session_key.is_operation_allowed("read", "test-resource", &context);
        assert!(result.is_err());
        
        if let Err(SessionKeyError::InvalidScope(msg)) = result {
            assert!(msg.contains("Geographic location not allowed"));
        } else {
            panic!("Expected InvalidScope error");
        }
    }
    
    #[test]
    fn test_invalid_action_rejection() {
        let scope = SessionScope {
            resources: vec!["test-resource".to_string()],
            actions: vec!["read".to_string()], // Only read is allowed
            geographic: GeographicConstraints {
                allowed_countries: vec![],
                allowed_regions: vec![],
                allowed_cities: vec![],
                radius_constraint: None,
                allowed_timezones: vec![],
            },
            network: NetworkConstraints {
                allowed_ip_ranges: vec![],
                allowed_network_types: vec![],
                allowed_domains: vec![],
                vpn_requirement: None,
            },
            device: DeviceConstraints {
                allowed_device_types: vec![],
                allowed_manufacturers: vec![],
                required_security_features: vec![],
            },
            data: DataConstraints {
                allowed_data_types: vec![],
                allowed_classifications: vec![],
                allowed_data_sources: vec![],
            },
            custom_attributes: HashMap::new(),
        };
        
        let time_constraints = TimeConstraints {
            start_time: Utc::now() - Duration::hours(1),
            end_time: Some(Utc::now() + Duration::hours(1)),
            max_duration: None,
            allowed_windows: vec![],
            required_timezone: None,
        };
        
        let amount_constraints = AmountConstraints {
            max_operations: Some(100),
            max_data_volume: Some(1024),
            max_financial_amount: Some(1000),
            max_sessions: Some(1),
            rate_limit: None,
            usage_quotas: vec![],
        };
        
        let security_info = SecurityInfo {
            algorithm: "Ed25519".to_string(),
            key_strength: 256,
            pq_security_level: PostQuantumSecurityLevel::None,
            required_factors: vec![AuthenticationFactor::Password],
            security_policies: vec!["basic".to_string()],
        };
        
        let session_key = SessionKey::new(
            "test-keystore-key".to_string(),
            SessionType::ApiRead,
            scope,
            time_constraints,
            amount_constraints,
            security_info,
            "Test Session Key".to_string(),
            "test-user".to_string(),
        );
        
        let context = ValidationContext {
            current_time: Utc::now(),
            current_location: GeographicLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                country_code: "US".to_string(),
                region: "CA".to_string(),
                city: "San Francisco".to_string(),
                timezone: "America/Los_Angeles".to_string(),
            },
            current_network: NetworkInfo {
                ip_address: "192.168.1.1".to_string(),
                network_type: NetworkType::Wifi,
                is_vpn: false,
                domain: "example.com".to_string(),
            },
            current_device: DeviceInfo {
                device_type: DeviceType::Desktop,
                manufacturer: "Generic".to_string(),
                model: "Desktop".to_string(),
                operating_system: "Linux".to_string(),
                security_features: vec![],
            },
        };
        
        // Should reject write operation (not in allowed actions)
        let result = session_key.is_operation_allowed("write", "test-resource", &context);
        assert!(result.is_err());
        
        if let Err(SessionKeyError::InvalidScope(msg)) = result {
            assert!(msg.contains("Action 'write' not allowed"));
        } else {
            panic!("Expected InvalidScope error for action");
        }
    }
    
    #[test]
    fn test_invalid_resource_rejection() {
        let scope = SessionScope {
            resources: vec!["allowed-resource".to_string()], // Only this resource is allowed
            actions: vec!["read".to_string()],
            geographic: GeographicConstraints {
                allowed_countries: vec![],
                allowed_regions: vec![],
                allowed_cities: vec![],
                radius_constraint: None,
                allowed_timezones: vec![],
            },
            network: NetworkConstraints {
                allowed_ip_ranges: vec![],
                allowed_network_types: vec![],
                allowed_domains: vec![],
                vpn_requirement: None,
            },
            device: DeviceConstraints {
                allowed_device_types: vec![],
                allowed_manufacturers: vec![],
                required_security_features: vec![],
            },
            data: DataConstraints {
                allowed_data_types: vec![],
                allowed_classifications: vec![],
                allowed_data_sources: vec![],
            },
            custom_attributes: HashMap::new(),
        };
        
        let time_constraints = TimeConstraints {
            start_time: Utc::now() - Duration::hours(1),
            end_time: Some(Utc::now() + Duration::hours(1)),
            max_duration: None,
            allowed_windows: vec![],
            required_timezone: None,
        };
        
        let amount_constraints = AmountConstraints {
            max_operations: Some(100),
            max_data_volume: Some(1024),
            max_financial_amount: Some(1000),
            max_sessions: Some(1),
            rate_limit: None,
            usage_quotas: vec![],
        };
        
        let security_info = SecurityInfo {
            algorithm: "Ed25519".to_string(),
            key_strength: 256,
            pq_security_level: PostQuantumSecurityLevel::None,
            required_factors: vec![AuthenticationFactor::Password],
            security_policies: vec!["basic".to_string()],
        };
        
        let session_key = SessionKey::new(
            "test-keystore-key".to_string(),
            SessionType::ApiRead,
            scope,
            time_constraints,
            amount_constraints,
            security_info,
            "Test Session Key".to_string(),
            "test-user".to_string(),
        );
        
        let context = ValidationContext {
            current_time: Utc::now(),
            current_location: GeographicLocation {
                latitude: 37.7749,
                longitude: -122.4194,
                country_code: "US".to_string(),
                region: "CA".to_string(),
                city: "San Francisco".to_string(),
                timezone: "America/Los_Angeles".to_string(),
            },
            current_network: NetworkInfo {
                ip_address: "192.168.1.1".to_string(),
                network_type: NetworkType::Wifi,
                is_vpn: false,
                domain: "example.com".to_string(),
            },
            current_device: DeviceInfo {
                device_type: DeviceType::Desktop,
                manufacturer: "Generic".to_string(),
                model: "Desktop".to_string(),
                operating_system: "Linux".to_string(),
                security_features: vec![],
            },
        };
        
        // Should reject access to disallowed resource
        let result = session_key.is_operation_allowed("read", "forbidden-resource", &context);
        assert!(result.is_err());
        
        if let Err(SessionKeyError::InvalidScope(msg)) = result {
            assert!(msg.contains("Resource 'forbidden-resource' not allowed"));
        } else {
            panic!("Expected InvalidScope error for resource");
        }
    }
}
