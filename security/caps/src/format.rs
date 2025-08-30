use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use thiserror::Error;

use crate::sign::{Signature, SignatureAlgorithm};

/// Error types for capability token operations
#[derive(Error, Debug)]
pub enum CapabilityTokenError {
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Token expired: {expires_at}, current: {current}")]
    TokenExpired { expires_at: u64, current: u64 },
    
    #[error("Token not yet valid: {not_before}, current: {current}")]
    TokenNotYetValid { not_before: u64, current: u64 },
    
    #[error("Invalid token format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required claim: {claim}")]
    MissingClaim { claim: String },
    
    #[error("Invalid claim value: {claim} = {value}")]
    InvalidClaimValue { claim: String, value: String },
    
    #[error("Insufficient privileges: required {required}, granted {granted}")]
    InsufficientPrivileges { required: String, granted: String },
    
    #[error("Purpose mismatch: expected {expected}, got {actual}")]
    PurposeMismatch { expected: String, actual: String },
    
    #[error("Resource access denied: {resource}")]
    ResourceAccessDenied { resource: String },
    
    #[error("Scope violation: {scope}")]
    ScopeViolation { scope: String },
    
    #[error("Token size exceeds limit: {size} bytes")]
    SizeLimitExceeded { size: usize },
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Token validation failed: {0}")]
    ValidationError(String),
}

/// Capability token header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityTokenHeader {
    /// Token type (always "capability")
    pub typ: String,
    
    /// Algorithm used for signing
    pub alg: SignatureAlgorithm,
    
    /// Token version
    pub ver: String,
    
    /// Key ID for verification
    pub kid: String,
    
    /// Token ID (UUID)
    pub jti: String,
    
    /// Issuer
    pub iss: String,
    
    /// Subject (token holder)
    pub sub: String,
    
    /// Audience (intended recipient)
    pub aud: String,
    
    /// Issued at timestamp
    pub iat: u64,
    
    /// Not before timestamp
    pub nbf: u64,
    
    /// Expiration timestamp
    pub exp: u64,
    
    /// Additional header fields
    pub additional: HashMap<String, Value>,
}

/// Capability claim types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CapabilityClaimType {
    /// Resource access capability
    Resource,
    
    /// Action capability
    Action,
    
    /// Scope capability
    Scope,
    
    /// Time capability
    Time,
    
    /// Location capability
    Location,
    
    /// Device capability
    Device,
    
    /// Network capability
    Network,
    
    /// Data capability
    Data,
    
    /// Custom capability
    Custom(String),
}

/// Resource capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapability {
    /// Resource type
    pub resource_type: String,
    
    /// Resource identifier
    pub resource_id: String,
    
    /// Resource path/URI
    pub resource_path: String,
    
    /// Access permissions
    pub permissions: Vec<String>,
    
    /// Resource metadata
    pub metadata: HashMap<String, Value>,
}

/// Action capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCapability {
    /// Action name
    pub action_name: String,
    
    /// Action parameters
    pub parameters: HashMap<String, Value>,
    
    /// Action constraints
    pub constraints: HashMap<String, Value>,
    
    /// Action metadata
    pub metadata: HashMap<String, Value>,
}

/// Scope capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeCapability {
    /// Scope name
    pub scope_name: String,
    
    /// Scope level
    pub scope_level: u32,
    
    /// Scope hierarchy
    pub scope_hierarchy: Vec<String>,
    
    /// Scope constraints
    pub scope_constraints: HashMap<String, Value>,
}

/// Time capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeCapability {
    /// Valid from timestamp
    pub valid_from: u64,
    
    /// Valid until timestamp
    pub valid_until: u64,
    
    /// Time constraints
    pub time_constraints: HashMap<String, Value>,
    
    /// Recurring patterns
    pub recurring_patterns: Vec<String>,
}

/// Location capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCapability {
    /// Geographic coordinates
    pub coordinates: Option<(f64, f64)>,
    
    /// Geographic region
    pub region: Option<String>,
    
    /// Network location
    pub network_location: Option<String>,
    
    /// Location constraints
    pub location_constraints: HashMap<String, Value>,
}

/// Device capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapability {
    /// Device type
    pub device_type: String,
    
    /// Device identifier
    pub device_id: String,
    
    /// Device capabilities
    pub device_capabilities: Vec<String>,
    
    /// Device constraints
    pub device_constraints: HashMap<String, Value>,
}

/// Network capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCapability {
    /// Network type
    pub network_type: String,
    
    /// Network identifier
    pub network_id: String,
    
    /// Network constraints
    pub network_constraints: HashMap<String, Value>,
    
    /// Network security level
    pub security_level: u32,
}

/// Data capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCapability {
    /// Data type
    pub data_type: String,
    
    /// Data classification
    pub data_classification: String,
    
    /// Data access level
    pub access_level: u32,
    
    /// Data constraints
    pub data_constraints: HashMap<String, Value>,
}

/// Custom capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomCapability {
    /// Custom claim name
    pub claim_name: String,
    
    /// Custom claim value
    pub claim_value: Value,
    
    /// Custom claim metadata
    pub metadata: HashMap<String, Value>,
}

/// Capability claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityClaim {
    /// Claim type
    pub claim_type: CapabilityClaimType,
    
    /// Claim data
    pub claim_data: CapabilityClaimData,
    
    /// Claim constraints
    pub constraints: HashMap<String, Value>,
    
    /// Claim metadata
    pub metadata: HashMap<String, Value>,
}

/// Capability claim data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityClaimData {
    Resource(ResourceCapability),
    Action(ActionCapability),
    Scope(ScopeCapability),
    Time(TimeCapability),
    Location(LocationCapability),
    Device(DeviceCapability),
    Network(NetworkCapability),
    Data(DataCapability),
    Custom(CustomCapability),
}

/// Capability token payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityTokenPayload {
    /// Token purpose
    pub purpose: String,
    
    /// Capability claims
    pub claims: Vec<CapabilityClaim>,
    
    /// Token scope
    pub scope: String,
    
    /// Token level
    pub level: u32,
    
    /// Token hierarchy
    pub hierarchy: Vec<String>,
    
    /// Token constraints
    pub constraints: HashMap<String, Value>,
    
    /// Token metadata
    pub metadata: HashMap<String, Value>,
    
    /// Additional payload fields
    pub additional: HashMap<String, Value>,
}

/// Complete capability token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    /// Token header
    pub header: CapabilityTokenHeader,
    
    /// Token payload
    pub payload: CapabilityTokenPayload,
    
    /// Token signature
    pub signature: Option<Signature>,
    
    /// Token format version
    pub format_version: String,
}

/// Token validation result
#[derive(Debug, Clone)]
pub struct TokenValidationResult {
    /// Whether the token is valid
    pub is_valid: bool,
    
    /// Validation errors
    pub errors: Vec<String>,
    
    /// Validation warnings
    pub warnings: Vec<String>,
    
    /// Token purpose validation
    pub purpose_valid: bool,
    
    /// Token scope validation
    pub scope_valid: bool,
    
    /// Token time validation
    pub time_valid: bool,
    
    /// Token privilege validation
    pub privilege_valid: bool,
    
    /// Token signature validation
    pub signature_valid: bool,
}

/// Capability token builder
pub struct CapabilityTokenBuilder {
    /// Token header
    header: CapabilityTokenHeader,
    
    /// Token payload
    payload: CapabilityTokenPayload,
    
    /// Token constraints
    constraints: HashMap<String, Value>,
}

impl CapabilityTokenBuilder {
    /// Create a new capability token builder
    pub fn new(
        issuer: &str,
        subject: &str,
        audience: &str,
        purpose: &str,
        scope: &str,
        level: u32,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let header = CapabilityTokenHeader {
            typ: "capability".to_string(),
            alg: SignatureAlgorithm::Dilithium3,
            ver: "1.0.0".to_string(),
            kid: Uuid::new_v4().to_string(),
            jti: Uuid::new_v4().to_string(),
            iss: issuer.to_string(),
            sub: subject.to_string(),
            aud: audience.to_string(),
            iat: now,
            nbf: now,
            exp: now + 3600, // Default 1 hour expiration
            additional: HashMap::new(),
        };
        
        let payload = CapabilityTokenPayload {
            purpose: purpose.to_string(),
            claims: Vec::new(),
            scope: scope.to_string(),
            level,
            hierarchy: Vec::new(),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
            additional: HashMap::new(),
        };
        
        Self {
            header,
            payload,
            constraints: HashMap::new(),
        }
    }
    
    /// Set token algorithm
    pub fn with_algorithm(mut self, algorithm: SignatureAlgorithm) -> Self {
        self.header.alg = algorithm;
        self
    }
    
    /// Set token expiration
    pub fn with_expiration(mut self, expires_in_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.header.exp = now + expires_in_seconds;
        self
    }
    
    /// Set token not before
    pub fn with_not_before(mut self, not_before: u64) -> Self {
        self.header.nbf = not_before;
        self
    }
    
    /// Set token key ID
    pub fn with_key_id(mut self, key_id: &str) -> Self {
        self.header.kid = key_id.to_string();
        self
    }
    
    /// Add resource capability
    pub fn with_resource_capability(
        mut self,
        resource_type: &str,
        resource_id: &str,
        resource_path: &str,
        permissions: Vec<String>,
    ) -> Self {
        let resource_cap = ResourceCapability {
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            resource_path: resource_path.to_string(),
            permissions,
            metadata: HashMap::new(),
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Resource,
            claim_data: CapabilityClaimData::Resource(resource_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add action capability
    pub fn with_action_capability(
        mut self,
        action_name: &str,
        parameters: HashMap<String, Value>,
    ) -> Self {
        let action_cap = ActionCapability {
            action_name: action_name.to_string(),
            parameters,
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Action,
            claim_data: CapabilityClaimData::Action(action_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add scope capability
    pub fn with_scope_capability(
        mut self,
        scope_name: &str,
        scope_level: u32,
        scope_hierarchy: Vec<String>,
    ) -> Self {
        let scope_cap = ScopeCapability {
            scope_name: scope_name.to_string(),
            scope_level,
            scope_hierarchy,
            scope_constraints: HashMap::new(),
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Scope,
            claim_data: CapabilityClaimData::Scope(scope_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add time capability
    pub fn with_time_capability(
        mut self,
        valid_from: u64,
        valid_until: u64,
    ) -> Self {
        let time_cap = TimeCapability {
            valid_from,
            valid_until,
            time_constraints: HashMap::new(),
            recurring_patterns: Vec::new(),
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Time,
            claim_data: CapabilityClaimData::Time(time_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add location capability
    pub fn with_location_capability(
        mut self,
        coordinates: Option<(f64, f64)>,
        region: Option<String>,
    ) -> Self {
        let location_cap = LocationCapability {
            coordinates,
            region,
            network_location: None,
            location_constraints: HashMap::new(),
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Location,
            claim_data: CapabilityClaimData::Location(location_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add device capability
    pub fn with_device_capability(
        mut self,
        device_type: &str,
        device_id: &str,
        device_capabilities: Vec<String>,
    ) -> Self {
        let device_cap = DeviceCapability {
            device_type: device_type.to_string(),
            device_id: device_id.to_string(),
            device_capabilities,
            device_constraints: HashMap::new(),
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Device,
            claim_data: CapabilityClaimData::Device(device_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add network capability
    pub fn with_network_capability(
        mut self,
        network_type: &str,
        network_id: &str,
        security_level: u32,
    ) -> Self {
        let network_cap = NetworkCapability {
            network_type: network_type.to_string(),
            network_id: network_id.to_string(),
            network_constraints: HashMap::new(),
            security_level,
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Network,
            claim_data: CapabilityClaimData::Network(network_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add data capability
    pub fn with_data_capability(
        mut self,
        data_type: &str,
        data_classification: &str,
        access_level: u32,
    ) -> Self {
        let data_cap = DataCapability {
            data_type: data_type.to_string(),
            data_classification: data_classification.to_string(),
            access_level,
            data_constraints: HashMap::new(),
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Data,
            claim_data: CapabilityClaimData::Data(data_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add custom capability
    pub fn with_custom_capability(
        mut self,
        claim_name: &str,
        claim_value: Value,
    ) -> Self {
        let custom_cap = CustomCapability {
            claim_name: claim_name.to_string(),
            claim_value,
            metadata: HashMap::new(),
        };
        
        let claim = CapabilityClaim {
            claim_type: CapabilityClaimType::Custom(claim_name.to_string()),
            claim_data: CapabilityClaimData::Custom(custom_cap),
            constraints: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        self.payload.claims.push(claim);
        self
    }
    
    /// Add constraint
    pub fn with_constraint(mut self, key: &str, value: Value) -> Self {
        self.payload.constraints.insert(key.to_string(), value);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.payload.metadata.insert(key.to_string(), value);
        self
    }
    
    /// Set hierarchy
    pub fn with_hierarchy(mut self, hierarchy: Vec<String>) -> Self {
        self.payload.hierarchy = hierarchy;
        self
    }
    
    /// Build the capability token
    pub fn build(self) -> CapabilityToken {
        CapabilityToken {
            header: self.header,
            payload: self.payload,
            signature: None, // Will be added after signing
            format_version: "1.0.0".to_string(),
        }
    }
}

/// Capability token formatter
pub struct CapabilityTokenFormatter;

impl CapabilityTokenFormatter {
    /// Format token as JSON
    pub fn to_json(token: &CapabilityToken) -> Result<String, CapabilityTokenError> {
        serde_json::to_string_pretty(token)
            .map_err(CapabilityTokenError::SerializationError)
    }
    
    /// Format token as compact JSON
    pub fn to_json_compact(token: &CapabilityToken) -> Result<String, CapabilityTokenError> {
        serde_json::to_string(token)
            .map_err(CapabilityTokenError::SerializationError)
    }
    
    /// Format token as base64
    pub fn to_base64(token: &CapabilityToken) -> Result<String, CapabilityTokenError> {
        let json = Self::to_json_compact(token)?;
        Ok(base64::encode(json.as_bytes()))
    }
    
    /// Parse token from JSON
    pub fn from_json(json_str: &str) -> Result<CapabilityToken, CapabilityTokenError> {
        serde_json::from_str(json_str)
            .map_err(CapabilityTokenError::SerializationError)
    }
    
    /// Parse token from base64
    pub fn from_base64(base64_str: &str) -> Result<CapabilityToken, CapabilityTokenError> {
        let json_bytes = base64::decode(base64_str)
            .map_err(|e| CapabilityTokenError::InvalidFormat(format!("Invalid base64: {}", e)))?;
        
        let json_str = String::from_utf8(json_bytes)
            .map_err(|e| CapabilityTokenError::InvalidFormat(format!("Invalid UTF-8: {}", e)))?;
        
        Self::from_json(&json_str)
    }
    
    /// Get token size in bytes
    pub fn get_token_size(token: &CapabilityToken) -> Result<usize, CapabilityTokenError> {
        let json = Self::to_json_compact(token)?;
        Ok(json.len())
    }
    
    /// Validate token size against limit
    pub fn validate_token_size(
        token: &CapabilityToken,
        max_size: usize,
    ) -> Result<(), CapabilityTokenError> {
        let size = Self::get_token_size(token)?;
        if size > max_size {
            return Err(CapabilityTokenError::SizeLimitExceeded { size });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capability_token_builder() {
        let token = CapabilityTokenBuilder::new(
            "test-issuer",
            "test-subject",
            "test-audience",
            "test-purpose",
            "test-scope",
            1,
        )
        .with_expiration(7200) // 2 hours
        .with_resource_capability(
            "file",
            "test-file-123",
            "/path/to/file",
            vec!["read".to_string(), "write".to_string()],
        )
        .with_action_capability(
            "process",
            HashMap::new(),
        )
        .with_scope_capability(
            "user",
            2,
            vec!["admin".to_string(), "user".to_string()],
        )
        .with_time_capability(
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 7200,
        )
        .with_location_capability(
            Some((40.7128, -74.0060)), // New York coordinates
            Some("US".to_string()),
        )
        .with_device_capability(
            "mobile",
            "device-123",
            vec!["camera".to_string(), "gps".to_string()],
        )
        .with_network_capability(
            "wifi",
            "network-123",
            2,
        )
        .with_data_capability(
            "personal",
            "confidential",
            1,
        )
        .with_custom_capability(
            "custom_claim",
            json!("custom_value"),
        )
        .with_constraint("max_requests", json!(100))
        .with_metadata("created_by", json!("test_user"))
        .with_hierarchy(vec!["admin".to_string(), "user".to_string()])
        .build();
        
        assert_eq!(token.header.typ, "capability");
        assert_eq!(token.header.iss, "test-issuer");
        assert_eq!(token.header.sub, "test-subject");
        assert_eq!(token.header.aud, "test-audience");
        assert_eq!(token.payload.purpose, "test-purpose");
        assert_eq!(token.payload.scope, "test-scope");
        assert_eq!(token.payload.level, 1);
        assert_eq!(token.payload.claims.len(), 8);
        assert_eq!(token.payload.hierarchy.len(), 2);
    }
    
    #[test]
    fn test_token_formatting() {
        let token = CapabilityTokenBuilder::new(
            "test-issuer",
            "test-subject",
            "test-audience",
            "test-purpose",
            "test-scope",
            1,
        )
        .with_resource_capability(
            "file",
            "test-file-123",
            "/path/to/file",
            vec!["read".to_string()],
        )
        .build();
        
        // Test JSON formatting
        let json = CapabilityTokenFormatter::to_json(&token).unwrap();
        assert!(json.contains("capability"));
        assert!(json.contains("test-purpose"));
        
        // Test compact JSON formatting
        let compact_json = CapabilityTokenFormatter::to_json_compact(&token).unwrap();
        assert!(compact_json.contains("capability"));
        assert!(compact_json.contains("test-purpose"));
        
        // Test base64 formatting
        let base64 = CapabilityTokenFormatter::to_base64(&token).unwrap();
        assert!(!base64.is_empty());
        
        // Test parsing from JSON
        let parsed_token = CapabilityTokenFormatter::from_json(&json).unwrap();
        assert_eq!(parsed_token.header.typ, "capability");
        assert_eq!(parsed_token.payload.purpose, "test-purpose");
        
        // Test parsing from base64
        let parsed_from_base64 = CapabilityTokenFormatter::from_base64(&base64).unwrap();
        assert_eq!(parsed_from_base64.header.typ, "capability");
        assert_eq!(parsed_from_base64.payload.purpose, "test-purpose");
    }
    
    #[test]
    fn test_token_size_validation() {
        let token = CapabilityTokenBuilder::new(
            "test-issuer",
            "test-subject",
            "test-audience",
            "test-purpose",
            "test-scope",
            1,
        )
        .with_resource_capability(
            "file",
            "test-file-123",
            "/path/to/file",
            vec!["read".to_string()],
        )
        .build();
        
        // Test size calculation
        let size = CapabilityTokenFormatter::get_token_size(&token).unwrap();
        assert!(size > 0);
        
        // Test size validation
        CapabilityTokenFormatter::validate_token_size(&token, size + 100).unwrap();
        
        // Test size limit exceeded
        let result = CapabilityTokenFormatter::validate_token_size(&token, size - 100);
        assert!(result.is_err());
        match result {
            Err(CapabilityTokenError::SizeLimitExceeded { size: actual_size }) => {
                assert_eq!(actual_size, size);
            }
            _ => panic!("Expected SizeLimitExceeded error"),
        }
    }
}
