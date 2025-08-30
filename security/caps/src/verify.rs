use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use crate::format::{CapabilityToken, CapabilityTokenError, CapabilityTokenFormatter};
use crate::sign::{Signature, SignatureAlgorithm, Verifier};

/// Error types for capability token verification operations
#[derive(Error, Debug)]
pub enum CapabilityTokenVerifyError {
    #[error("Token verification failed: {0}")]
    VerificationError(String),
    
    #[error("Token validation failed: {0}")]
    ValidationError(#[from] CapabilityTokenError),
    
    #[error("Token expired: {expires_at}, current: {current}")]
    TokenExpired { expires_at: u64, current: u64 },
    
    #[error("Token not yet valid: {not_before}, current: {current}")]
    TokenNotYetValid { not_before: u64, current: u64 },
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    
    #[error("Algorithm mismatch: expected {expected}, got {actual}")]
    AlgorithmMismatch { expected: String, actual: String },
    
    #[error("Key not found: {key_id}")]
    KeyNotFound { key_id: String },
    
    #[error("Insufficient privileges: required {required}, granted {granted}")]
    InsufficientPrivileges { required: String, granted: String },
    
    #[error("Purpose mismatch: expected {expected}, got {actual}")]
    PurposeMismatch { expected: String, actual: String },
    
    #[error("Resource access denied: {resource}")]
    ResourceAccessDenied { resource: String },
    
    #[error("Scope violation: {scope}")]
    ScopeViolation { scope: String },
    
    #[error("Token format invalid: {0}")]
    InvalidFormat(String),
    
    #[error("Token size exceeds limit: {size} bytes")]
    SizeLimitExceeded { size: usize },
    
    #[error("Token tampering detected: {0}")]
    TokenTamperingDetected(String),
}

/// Capability token verification configuration
#[derive(Debug, Clone)]
pub struct CapabilityTokenVerifyConfig {
    /// Maximum token size in bytes
    pub max_token_size: usize,
    
    /// Clock skew tolerance in seconds
    pub clock_skew_tolerance: u64,
    
    /// Enable strict validation
    pub strict_validation: bool,
    
    /// Required signature algorithms
    pub required_algorithms: Vec<SignatureAlgorithm>,
    
    /// Trusted issuers
    pub trusted_issuers: Vec<String>,
    
    /// Trusted audiences
    pub trusted_audiences: Vec<String>,
    
    /// Minimum token level
    pub minimum_token_level: u32,
    
    /// Maximum token level
    pub maximum_token_level: u32,
    
    /// Validation rules
    pub validation_rules: HashMap<String, Value>,
}

impl Default for CapabilityTokenVerifyConfig {
    fn default() -> Self {
        Self {
            max_token_size: 64 * 1024, // 64KB
            clock_skew_tolerance: 300, // 5 minutes
            strict_validation: true,
            required_algorithms: vec![
                SignatureAlgorithm::Dilithium2,
                SignatureAlgorithm::Dilithium3,
                SignatureAlgorithm::Dilithium5,
            ],
            trusted_issuers: Vec::new(),
            trusted_audiences: Vec::new(),
            minimum_token_level: 0,
            maximum_token_level: u32::MAX,
            validation_rules: HashMap::new(),
        }
    }
}

/// Capability token validator
pub struct TokenValidator {
    config: CapabilityTokenVerifyConfig,
    verifiers: HashMap<String, Box<dyn Verifier>>,
}

impl TokenValidator {
    /// Create a new token validator
    pub fn new(config: CapabilityTokenVerifyConfig) -> Self {
        Self {
            config,
            verifiers: HashMap::new(),
        }
    }
    
    /// Add a verifier
    pub fn add_verifier(&mut self, key_id: String, verifier: Box<dyn Verifier>) {
        self.verifiers.insert(key_id, verifier);
    }
    
    /// Validate a capability token
    pub fn validate_token(&self, token: &CapabilityToken) -> Result<(), CapabilityTokenVerifyError> {
        // Basic format validation
        self.validate_token_format(token)?;
        
        // Timestamp validation
        self.validate_token_timestamps(token)?;
        
        // Signature validation
        self.validate_token_signature(token)?;
        
        // Claims validation
        self.validate_token_claims(token)?;
        
        // Security validation
        self.validate_token_security(token)?;
        
        Ok(())
    }
    
    /// Validate token format
    fn validate_token_format(&self, token: &CapabilityToken) -> Result<(), CapabilityTokenVerifyError> {
        // Check token type
        if token.header.typ != "capability" {
            return Err(CapabilityTokenVerifyError::InvalidFormat(
                format!("Invalid token type: {}", token.header.typ)
            ));
        }
        
        // Check required fields
        if token.header.iss.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing issuer".to_string()));
        }
        
        if token.header.sub.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing subject".to_string()));
        }
        
        if token.header.aud.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing audience".to_string()));
        }
        
        if token.payload.purpose.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing purpose".to_string()));
        }
        
        if token.payload.scope.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing scope".to_string()));
        }
        
        // Check token size
        let token_size = CapabilityTokenFormatter::get_token_size(token)
            .map_err(|e| CapabilityTokenVerifyError::InvalidFormat(e.to_string()))?;
        
        if token_size > self.config.max_token_size {
            return Err(CapabilityTokenVerifyError::SizeLimitExceeded { size: token_size });
        }
        
        Ok(())
    }
    
    /// Validate token timestamps
    fn validate_token_timestamps(&self, token: &CapabilityToken) -> Result<(), CapabilityTokenVerifyError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check not before time
        if token.header.nbf > 0 {
            let tolerance = if token.header.nbf > now {
                token.header.nbf - now
            } else {
                now - token.header.nbf
            };
            
            if tolerance > self.config.clock_skew_tolerance {
                if token.header.nbf > now {
                    return Err(CapabilityTokenVerifyError::TokenNotYetValid {
                        not_before: token.header.nbf,
                        current: now,
                    });
                }
            }
        }
        
        // Check expiration time
        if token.header.exp > 0 {
            let tolerance = if token.header.exp > now {
                token.header.exp - now
            } else {
                now - token.header.exp
            };
            
            if tolerance > self.config.clock_skew_tolerance {
                if token.header.exp <= now {
                    return Err(CapabilityTokenVerifyError::TokenExpired {
                        expires_at: token.header.exp,
                        current: now,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate token signature
    fn validate_token_signature(&self, token: &CapabilityToken) -> Result<(), CapabilityTokenVerifyError> {
        // Check if signature exists
        let signature = token.signature.as_ref()
            .ok_or_else(|| CapabilityTokenVerifyError::InvalidSignature("No signature found".to_string()))?;
        
        // Check algorithm
        if !self.config.required_algorithms.contains(&signature.algorithm) {
            return Err(CapabilityTokenVerifyError::AlgorithmMismatch {
                expected: format!("{:?}", self.config.required_algorithms[0]),
                actual: format!("{:?}", signature.algorithm),
            });
        }
        
        // Check key ID
        if signature.key_id.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidSignature("Empty key ID".to_string()));
        }
        
        // Find verifier
        let verifier = self.verifiers.get(&signature.key_id)
            .ok_or_else(|| CapabilityTokenVerifyError::KeyNotFound { key_id: signature.key_id.clone() })?;
        
        // Verify signature
        let token_json = CapabilityTokenFormatter::to_json_compact(token)
            .map_err(|e| CapabilityTokenVerifyError::InvalidSignature(e.to_string()))?;
        
        let is_valid = verifier.verify_signature(token_json.as_bytes(), signature)
            .map_err(|e| CapabilityTokenVerifyError::SignatureVerificationFailed(e.to_string()))?;
        
        if !is_valid {
            return Err(CapabilityTokenVerifyError::InvalidSignature("Signature verification failed".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate token claims
    fn validate_token_claims(&self, token: &CapabilityToken) -> Result<(), CapabilityTokenVerifyError> {
        // Check token level
        if token.payload.level < self.config.minimum_token_level {
            return Err(CapabilityTokenVerifyError::InsufficientPrivileges {
                required: format!("Level {}", self.config.minimum_token_level),
                granted: format!("Level {}", token.payload.level),
            });
        }
        
        if token.payload.level > self.config.maximum_token_level {
            return Err(CapabilityTokenVerifyError::InsufficientPrivileges {
                required: format!("Level <= {}", self.config.maximum_token_level),
                granted: format!("Level {}", token.payload.level),
            });
        }
        
        // Check trusted issuers if configured
        if !self.config.trusted_issuers.is_empty() {
            if !self.config.trusted_issuers.contains(&token.header.iss) {
                return Err(CapabilityTokenVerifyError::InvalidFormat(
                    format!("Untrusted issuer: {}", token.header.iss)
                ));
            }
        }
        
        // Check trusted audiences if configured
        if !self.config.trusted_audiences.is_empty() {
            if !self.config.trusted_audiences.contains(&token.header.aud) {
                return Err(CapabilityTokenVerifyError::InvalidFormat(
                    format!("Untrusted audience: {}", token.header.aud)
                ));
            }
        }
        
        // Validate individual claims
        for claim in &token.payload.claims {
            self.validate_claim(claim)?;
        }
        
        Ok(())
    }
    
    /// Validate individual claim
    fn validate_claim(&self, claim: &crate::format::CapabilityClaim) -> Result<(), CapabilityTokenVerifyError> {
        match &claim.claim_data {
            crate::format::CapabilityClaimData::Resource(resource) => {
                self.validate_resource_claim(resource)?;
            }
            crate::format::CapabilityClaimData::Action(action) => {
                self.validate_action_claim(action)?;
            }
            crate::format::CapabilityClaimData::Scope(scope) => {
                self.validate_scope_claim(scope)?;
            }
            crate::format::CapabilityClaimData::Time(time) => {
                self.validate_time_claim(time)?;
            }
            crate::format::CapabilityClaimData::Location(location) => {
                self.validate_location_claim(location)?;
            }
            crate::format::CapabilityClaimData::Device(device) => {
                self.validate_device_claim(device)?;
            }
            crate::format::CapabilityClaimData::Network(network) => {
                self.validate_network_claim(network)?;
            }
            crate::format::CapabilityClaimData::Data(data) => {
                self.validate_data_claim(data)?;
            }
            crate::format::CapabilityClaimData::Custom(custom) => {
                self.validate_custom_claim(custom)?;
            }
        }
        
        Ok(())
    }
    
    /// Validate resource claim
    fn validate_resource_claim(&self, resource: &crate::format::ResourceCapability) -> Result<(), CapabilityTokenVerifyError> {
        if resource.resource_type.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty resource type".to_string()));
        }
        
        if resource.resource_id.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty resource ID".to_string()));
        }
        
        if resource.resource_path.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty resource path".to_string()));
        }
        
        if resource.permissions.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("No permissions specified".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate action claim
    fn validate_action_claim(&self, action: &crate::format::ActionCapability) -> Result<(), CapabilityTokenVerifyError> {
        if action.action_name.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty action name".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate scope claim
    fn validate_scope_claim(&self, scope: &crate::format::ScopeCapability) -> Result<(), CapabilityTokenVerifyError> {
        if scope.scope_name.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty scope name".to_string()));
        }
        
        if scope.scope_hierarchy.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty scope hierarchy".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate time claim
    fn validate_time_claim(&self, time: &crate::format::TimeCapability) -> Result<(), CapabilityTokenVerifyError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if time.valid_from > 0 && time.valid_from > now + self.config.clock_skew_tolerance {
            return Err(CapabilityTokenVerifyError::TokenNotYetValid {
                not_before: time.valid_from,
                current: now,
            });
        }
        
        if time.valid_until > 0 && time.valid_until <= now - self.config.clock_skew_tolerance {
            return Err(CapabilityTokenVerifyError::TokenExpired {
                expires_at: time.valid_until,
                current: now,
            });
        }
        
        Ok(())
    }
    
    /// Validate location claim
    fn validate_location_claim(&self, _location: &crate::format::LocationCapability) -> Result<(), CapabilityTokenVerifyError> {
        // Location validation can be extended based on requirements
        Ok(())
    }
    
    /// Validate device claim
    fn validate_device_claim(&self, device: &crate::format::DeviceCapability) -> Result<(), CapabilityTokenVerifyError> {
        if device.device_type.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty device type".to_string()));
        }
        
        if device.device_id.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty device ID".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate network claim
    fn validate_network_claim(&self, network: &crate::format::NetworkCapability) -> Result<(), CapabilityTokenVerifyError> {
        if network.network_type.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty network type".to_string()));
        }
        
        if network.network_id.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty network ID".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate data claim
    fn validate_data_claim(&self, data: &crate::format::DataCapability) -> Result<(), CapabilityTokenVerifyError> {
        if data.data_type.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty data type".to_string()));
        }
        
        if data.data_classification.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty data classification".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate custom claim
    fn validate_custom_claim(&self, custom: &crate::format::CustomCapability) -> Result<(), CapabilityTokenVerifyError> {
        if custom.claim_name.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Empty custom claim name".to_string()));
        }
        
        Ok(())
    }
    
    /// Validate token security
    fn validate_token_security(&self, token: &CapabilityToken) -> Result<(), CapabilityTokenVerifyError> {
        // Check for potential tampering
        if self.config.strict_validation {
            // Verify token ID uniqueness
            if token.header.jti.is_empty() {
                return Err(CapabilityTokenVerifyError::TokenTamperingDetected("Empty token ID".to_string()));
            }
            
            // Verify version consistency
            if token.header.ver != token.format_version {
                return Err(CapabilityTokenVerifyError::TokenTamperingDetected("Version mismatch".to_string()));
            }
            
            // Check for suspicious patterns
            if token.header.iss == token.header.sub {
                return Err(CapabilityTokenVerifyError::TokenTamperingDetected("Issuer equals subject".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Verify token for specific purpose
    pub fn verify_token_for_purpose(
        &self,
        token: &CapabilityToken,
        expected_purpose: &str,
    ) -> Result<(), CapabilityTokenVerifyError> {
        // First validate the token
        self.validate_token(token)?;
        
        // Check purpose match
        if token.payload.purpose != expected_purpose {
            return Err(CapabilityTokenVerifyError::PurposeMismatch {
                expected: expected_purpose.to_string(),
                actual: token.payload.purpose.clone(),
            });
        }
        
        Ok(())
    }
    
    /// Verify token for specific scope
    pub fn verify_token_for_scope(
        &self,
        token: &CapabilityToken,
        expected_scope: &str,
    ) -> Result<(), CapabilityTokenVerifyError> {
        // First validate the token
        self.validate_token(token)?;
        
        // Check scope match
        if token.payload.scope != expected_scope {
            return Err(CapabilityTokenVerifyError::ScopeViolation {
                scope: format!("Expected: {}, Got: {}", expected_scope, token.payload.scope),
            });
        }
        
        Ok(())
    }
    
    /// Verify token for resource access
    pub fn verify_token_for_resource(
        &self,
        token: &CapabilityToken,
        resource_type: &str,
        resource_id: &str,
        required_permission: &str,
    ) -> Result<(), CapabilityTokenVerifyError> {
        // First validate the token
        self.validate_token(token)?;
        
        // Find resource claim
        let resource_claim = token.payload.claims.iter().find(|claim| {
            matches!(claim.claim_data, crate::format::CapabilityClaimData::Resource(_))
        });
        
        let resource_claim = resource_claim.ok_or_else(|| {
            CapabilityTokenVerifyError::ResourceAccessDenied {
                resource: format!("{}:{}", resource_type, resource_id),
            }
        })?;
        
        if let crate::format::CapabilityClaimData::Resource(resource) = &resource_claim.claim_data {
            // Check resource type and ID
            if resource.resource_type != resource_type || resource.resource_id != resource_id {
                return Err(CapabilityTokenVerifyError::ResourceAccessDenied {
                    resource: format!("{}:{}", resource_type, resource_id),
                });
            }
            
            // Check permission
            if !resource.permissions.contains(&required_permission.to_string()) {
                return Err(CapabilityTokenVerifyError::InsufficientPrivileges {
                    required: required_permission.to_string(),
                    granted: format!("{:?}", resource.permissions),
                });
            }
        }
        
        Ok(())
    }
    
    /// Get validator configuration
    pub fn get_config(&self) -> &CapabilityTokenVerifyConfig {
        &self.config
    }
    
    /// Update validator configuration
    pub fn update_config(&mut self, config: CapabilityTokenVerifyConfig) {
        self.config = config;
    }
}

/// Mock token validator for testing
pub struct MockTokenValidator {
    config: CapabilityTokenVerifyConfig,
}

impl MockTokenValidator {
    /// Create a new mock token validator
    pub fn new() -> Self {
        Self {
            config: CapabilityTokenVerifyConfig::default(),
        }
    }
    
    /// Validate a capability token (mock implementation)
    pub fn validate_token(&self, token: &CapabilityToken) -> Result<(), CapabilityTokenVerifyError> {
        // Basic format validation
        if token.header.typ != "capability" {
            return Err(CapabilityTokenVerifyError::InvalidFormat(
                format!("Invalid token type: {}", token.header.typ)
            ));
        }
        
        if token.header.iss.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing issuer".to_string()));
        }
        
        if token.header.sub.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing subject".to_string()));
        }
        
        if token.header.aud.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing audience".to_string()));
        }
        
        if token.payload.purpose.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing purpose".to_string()));
        }
        
        if token.payload.scope.is_empty() {
            return Err(CapabilityTokenVerifyError::InvalidFormat("Missing scope".to_string()));
        }
        
        // Mock timestamp validation (always pass)
        // Mock signature validation (always pass)
        // Mock claims validation (always pass)
        // Mock security validation (always pass)
        
        Ok(())
    }
    
    /// Get mock validator configuration
    pub fn get_config(&self) -> &CapabilityTokenVerifyConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::CapabilityTokenBuilder;
    
    #[test]
    fn test_token_validator_config() {
        let config = CapabilityTokenVerifyConfig::default();
        
        assert_eq!(config.max_token_size, 64 * 1024);
        assert_eq!(config.clock_skew_tolerance, 300);
        assert!(config.strict_validation);
        assert_eq!(config.required_algorithms.len(), 3);
        assert!(config.required_algorithms.contains(&SignatureAlgorithm::Dilithium3));
        assert_eq!(config.minimum_token_level, 0);
        assert_eq!(config.maximum_token_level, u32::MAX);
    }
    
    #[test]
    fn test_mock_token_validator() {
        let validator = MockTokenValidator::new();
        
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
        
        let result = validator.validate_token(&token);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_token_validator_format_validation() {
        let config = CapabilityTokenVerifyConfig::default();
        let mut validator = TokenValidator::new(config);
        
        // Test valid token
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
        
        // This will fail signature validation since no verifier is added
        // but format validation should pass
        let result = validator.validate_token(&token);
        assert!(result.is_err());
        match result {
            Err(CapabilityTokenVerifyError::KeyNotFound { .. }) => {
                // Expected error for missing verifier
            }
            _ => panic!("Expected KeyNotFound error"),
        }
    }
    
    #[test]
    fn test_token_validator_timestamp_validation() {
        let config = CapabilityTokenVerifyConfig::default();
        let mut validator = TokenValidator::new(config);
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Test expired token
        let token = CapabilityTokenBuilder::new(
            "test-issuer",
            "test-subject",
            "test-audience",
            "test-purpose",
            "test-scope",
            1,
        )
        .with_expiration(1) // 1 second expiration
        .with_resource_capability(
            "file",
            "test-file-123",
            "/path/to/file",
            vec!["read".to_string()],
        )
        .build();
        
        // Wait for token to expire
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        let result = validator.validate_token(&token);
        assert!(result.is_err());
        match result {
            Err(CapabilityTokenVerifyError::TokenExpired { .. }) => {
                // Expected error for expired token
            }
            _ => panic!("Expected TokenExpired error"),
        }
    }
}
