//! Capability Tokens - Purpose-bound, Time-scoped, Least-privilege Security System
//! 
//! This module provides a comprehensive system for creating, signing, and verifying
//! capability tokens with post-quantum cryptographic signatures.

pub mod format;
pub mod sign;
pub mod verify;

pub use format::{
    CapabilityToken, CapabilityTokenHeader, CapabilityTokenPayload, CapabilityTokenError,
    CapabilityClaim, CapabilityClaimType, CapabilityClaimData, CapabilityTokenBuilder,
    CapabilityTokenFormatter, TokenValidationResult,
    ResourceCapability, ActionCapability, ScopeCapability, TimeCapability,
    LocationCapability, DeviceCapability, NetworkCapability, DataCapability, CustomCapability,
};

pub use sign::{
    Signature, SignatureAlgorithm, KeyPair, Signer, Verifier, DilithiumSigner,
    DilithiumVerifier, MockSigner, SignatureError,
};

pub use verify::{
    TokenValidator, CapabilityTokenVerifyConfig, CapabilityTokenVerifyError,
    MockTokenValidator,
};

/// Capability token service configuration
#[derive(Debug, Clone)]
pub struct CapabilityTokenServiceConfig {
    /// Service name
    pub service_name: String,
    
    /// Service version
    pub service_version: String,
    
    /// Default signature algorithm
    pub default_algorithm: SignatureAlgorithm,
    
    /// Maximum token size in bytes
    pub max_token_size: usize,
    
    /// Default token expiration in seconds
    pub default_expiration: u64,
    
    /// Clock skew tolerance in seconds
    pub clock_skew_tolerance: u64,
    
    /// Enable strict validation
    pub strict_validation: bool,
    
    /// Required claims for all tokens
    pub required_claims: Vec<String>,
    
    /// Trusted issuers
    pub trusted_issuers: Vec<String>,
    
    /// Trusted audiences
    pub trusted_audiences: Vec<String>,
    
    /// Minimum token level
    pub minimum_token_level: u32,
    
    /// Maximum token level
    pub maximum_token_level: u32,
}

impl Default for CapabilityTokenServiceConfig {
    fn default() -> Self {
        Self {
            service_name: "capability-token-service".to_string(),
            service_version: "0.1.0".to_string(),
            default_algorithm: SignatureAlgorithm::Dilithium3,
            max_token_size: 64 * 1024, // 64KB
            default_expiration: 3600, // 1 hour
            clock_skew_tolerance: 300, // 5 minutes
            strict_validation: true,
            required_claims: vec!["purpose".to_string(), "scope".to_string()],
            trusted_issuers: Vec::new(),
            trusted_audiences: Vec::new(),
            minimum_token_level: 0,
            maximum_token_level: u32::MAX,
        }
    }
}

/// Capability token service for managing tokens
pub struct CapabilityTokenService {
    config: CapabilityTokenServiceConfig,
    signer: Box<dyn Signer>,
    validator: TokenValidator,
}

impl CapabilityTokenService {
    /// Create a new capability token service
    pub fn new(
        config: CapabilityTokenServiceConfig,
        signer: Box<dyn Signer>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let verify_config = verify::CapabilityTokenVerifyConfig {
            max_token_size: config.max_token_size,
            clock_skew_tolerance: config.clock_skew_tolerance,
            strict_validation: config.strict_validation,
            required_algorithms: vec![config.default_algorithm.clone()],
            trusted_issuers: config.trusted_issuers.clone(),
            trusted_audiences: config.trusted_audiences.clone(),
            minimum_token_level: config.minimum_token_level,
            maximum_token_level: config.maximum_token_level,
            validation_rules: HashMap::new(),
        };
        
        let validator = TokenValidator::new(verify_config);
        
        Ok(Self {
            config,
            signer,
            validator,
        })
    }
    
    /// Create and sign a capability token
    pub fn create_token(
        &self,
        issuer: &str,
        subject: &str,
        audience: &str,
        purpose: &str,
        scope: &str,
        level: u32,
    ) -> Result<CapabilityToken, Box<dyn std::error::Error>> {
        let token = CapabilityTokenBuilder::new(
            issuer,
            subject,
            audience,
            purpose,
            scope,
            level,
        )
        .with_expiration(self.config.default_expiration)
        .with_algorithm(self.config.default_algorithm.clone())
        .build();
        
        // Sign the token
        let token_json = CapabilityTokenFormatter::to_json_compact(&token)?;
        let signature = self.signer.sign(token_json.as_bytes())?;
        
        let mut signed_token = token;
        signed_token.signature = Some(signature);
        
        // Validate the signed token
        self.validator.validate_token(&signed_token)?;
        
        Ok(signed_token)
    }
    
    /// Verify a capability token
    pub fn verify_token(&self, token: &CapabilityToken) -> Result<(), Box<dyn std::error::Error>> {
        self.validator.validate_token(token)?;
        Ok(())
    }
    
    /// Verify token for specific purpose
    pub fn verify_token_for_purpose(
        &self,
        token: &CapabilityToken,
        expected_purpose: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.validator.verify_token_for_purpose(token, expected_purpose)?;
        Ok(())
    }
    
    /// Verify token for specific scope
    pub fn verify_token_for_scope(
        &self,
        token: &CapabilityToken,
        expected_scope: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.validator.verify_token_for_scope(token, expected_scope)?;
        Ok(())
    }
    
    /// Verify token for resource access
    pub fn verify_token_for_resource(
        &self,
        token: &CapabilityToken,
        resource_type: &str,
        resource_id: &str,
        required_permission: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.validator.verify_token_for_resource(token, resource_type, resource_id, required_permission)?;
        Ok(())
    }
    
    /// Get service configuration
    pub fn get_config(&self) -> &CapabilityTokenServiceConfig {
        &self.config
    }
    
    /// Get signer algorithm
    pub fn get_algorithm(&self) -> SignatureAlgorithm {
        self.config.default_algorithm.clone()
    }
    
    /// Get signer key ID
    pub fn get_key_id(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.signer.get_key_id()?)
    }
    
    /// Get signer public key
    pub fn get_public_key(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(self.signer.get_public_key()?)
    }
}

impl Default for CapabilityTokenService {
    fn default() -> Self {
        let config = CapabilityTokenServiceConfig::default();
        let signer = Box::new(MockSigner::new());
        Self::new(config, signer).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_capability_token_service_config() {
        let config = CapabilityTokenServiceConfig::default();
        
        assert_eq!(config.service_name, "capability-token-service");
        assert_eq!(config.service_version, "0.1.0");
        assert_eq!(config.default_algorithm, SignatureAlgorithm::Dilithium3);
        assert_eq!(config.max_token_size, 64 * 1024);
        assert_eq!(config.default_expiration, 3600);
        assert_eq!(config.clock_skew_tolerance, 300);
        assert!(config.strict_validation);
        assert_eq!(config.required_claims.len(), 2);
        assert!(config.required_claims.contains(&"purpose".to_string()));
        assert!(config.required_claims.contains(&"scope".to_string()));
    }
    
    #[test]
    fn test_capability_token_service_creation() {
        let config = CapabilityTokenServiceConfig::default();
        let signer = Box::new(MockSigner::new());
        let service = CapabilityTokenService::new(config, signer).unwrap();
        
        assert_eq!(service.get_algorithm(), SignatureAlgorithm::Dilithium3);
        assert!(service.get_key_id().is_ok());
        assert!(service.get_public_key().is_ok());
    }
    
    #[test]
    fn test_capability_token_creation() {
        let service = CapabilityTokenService::default();
        
        let token = service.create_token(
            "test-issuer",
            "test-subject",
            "test-audience",
            "test-purpose",
            "test-scope",
            1,
        ).unwrap();
        
        assert_eq!(token.header.typ, "capability");
        assert_eq!(token.header.iss, "test-issuer");
        assert_eq!(token.header.sub, "test-subject");
        assert_eq!(token.header.aud, "test-audience");
        assert_eq!(token.payload.purpose, "test-purpose");
        assert_eq!(token.payload.scope, "test-scope");
        assert_eq!(token.payload.level, 1);
        assert!(token.signature.is_some());
    }
    
    #[test]
    fn test_capability_token_verification() {
        let service = CapabilityTokenService::default();
        
        let token = service.create_token(
            "test-issuer",
            "test-subject",
            "test-audience",
            "test-purpose",
            "test-scope",
            1,
        ).unwrap();
        
        // Verify the token
        let result = service.verify_token(&token);
        assert!(result.is_ok());
        
        // Verify for specific purpose
        let result = service.verify_token_for_purpose(&token, "test-purpose");
        assert!(result.is_ok());
        
        // Verify for specific scope
        let result = service.verify_token_for_scope(&token, "test-scope");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_capability_token_with_claims() {
        let service = CapabilityTokenService::default();
        
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
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 7200,
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
            serde_json::json!("custom_value"),
        )
        .with_constraint("max_requests", serde_json::json!(100))
        .with_metadata("created_by", serde_json::json!("test_user"))
        .with_hierarchy(vec!["admin".to_string(), "user".to_string()])
        .build();
        
        // Sign the token
        let token_json = CapabilityTokenFormatter::to_json_compact(&token).unwrap();
        let signature = service.signer.sign(token_json.as_bytes()).unwrap();
        
        let mut signed_token = token;
        signed_token.signature = Some(signature);
        
        // Verify the token
        let result = service.verify_token(&signed_token);
        assert!(result.is_ok());
        
        // Verify resource access
        let result = service.verify_token_for_resource(
            &signed_token,
            "file",
            "test-file-123",
            "read",
        );
        assert!(result.is_ok());
        
        // Verify resource access with wrong permission
        let result = service.verify_token_for_resource(
            &signed_token,
            "file",
            "test-file-123",
            "delete",
        );
        assert!(result.is_err());
    }
}
