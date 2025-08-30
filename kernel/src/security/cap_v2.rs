/// CapTokens v2 - PQC-Signed Capability Token System
/// 
/// This module implements quantum-resistant capability tokens using Dilithium signatures
/// and Kyber encapsulation for end-to-end payload authentication.

use super::{SecurityError, SecurityResult, update_security_stats, get_current_time_ms};
use crate::{kprintln, klog};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use alloc::string::String;
use core::time::Duration;
use spin::Mutex;
use serde::{Serialize, Deserialize};

/// Capability token header structure
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapTokenHeader {
    /// Issuer DID (Decentralized Identifier)
    pub issuer_did: String,
    
    /// Subject process ID (token holder)
    pub subject_pid: u64,
    
    /// Destination process ID (target)
    pub dst_pid: u64,
    
    /// Scope flags (permissions)
    pub scope_flags: u32,
    
    /// Token validity start time (milliseconds since epoch)
    pub not_before: u64,
    
    /// Token validity end time (milliseconds since epoch)
    pub not_after: u64,
    
    /// Unique nonce for replay protection
    pub nonce: u128,
    
    /// Hash of the token purpose/context
    pub purpose_hash: [u8; 32],
}

impl CapTokenHeader {
    /// Create a new capability token header
    pub fn new(
        issuer_did: String,
        subject_pid: u64,
        dst_pid: u64,
        scope_flags: u32,
        not_before: u64,
        not_after: u64,
        nonce: u128,
        purpose_hash: [u8; 32],
    ) -> Self {
        Self {
            issuer_did,
            subject_pid,
            dst_pid,
            scope_flags,
            not_before,
            not_after,
            nonce,
            purpose_hash,
        }
    }
    
    /// Check if the token is currently valid
    pub fn is_valid(&self, now_ms: u64) -> bool {
        now_ms >= self.not_before && now_ms <= self.not_after
    }
    
    /// Check if the token is expired
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms > self.not_after
    }
    
    /// Check if the token is not yet valid
    pub fn is_early(&self, now_ms: u64) -> bool {
        now_ms < self.not_before
    }
    
    /// Get remaining validity time
    pub fn time_until_expiry(&self, now_ms: u64) -> Option<u64> {
        if self.is_expired(now_ms) {
            None
        } else {
            Some(self.not_after - now_ms)
        }
    }
    
    /// Check if token grants access to destination
    pub fn grants_access_to(&self, dst: u64) -> bool {
        self.dst_pid == dst
    }
    
    /// Check if token has specific scope permission
    pub fn has_scope(&self, scope: u32) -> bool {
        (self.scope_flags & scope) == scope
    }
    
    /// Serialize header for signing
    pub fn to_bytes(&self) -> Vec<u8> {
        // Use bincode for deterministic serialization
        bincode::serialize(self).unwrap_or_default()
    }
    
    /// Get header size in bytes
    pub fn size(&self) -> usize {
        self.to_bytes().len()
    }
}

impl core::fmt::Display for CapTokenHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CapToken[issuer:{}, subject:{}, dst:{}, scope:0x{:x}, valid:{}-{}, nonce:0x{:x}]",
               self.issuer_did, self.subject_pid, self.dst_pid, self.scope_flags,
               self.not_before, self.not_after, self.nonce)
    }
}

/// Capability token signature structure
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapTokenSignature {
    /// Dilithium2 signature over the header
    pub dilithium_signature: Vec<u8>,
    
    /// Kyber encapsulated session key (optional, for E2E payload auth)
    pub kyber_ciphertext: Option<Vec<u8>>,
    
    /// Signature algorithm identifier
    pub algorithm: SignatureAlgorithm,
}

impl CapTokenSignature {
    /// Create a new signature
    pub fn new(
        dilithium_signature: Vec<u8>,
        kyber_ciphertext: Option<Vec<u8>>,
        algorithm: SignatureAlgorithm,
    ) -> Self {
        Self {
            dilithium_signature,
            kyber_ciphertext,
            algorithm,
        }
    }
    
    /// Get signature size in bytes
    pub fn size(&self) -> usize {
        let base_size = self.dilithium_signature.len();
        let kyber_size = self.kyber_ciphertext.as_ref().map_or(0, |k| k.len());
        base_size + kyber_size + 1 // +1 for algorithm identifier
    }
}

/// Signature algorithm types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// Dilithium2 signature
    Dilithium2,
    /// Dilithium3 signature
    Dilithium3,
    /// Dilithium5 signature
    Dilithium5,
}

impl SignatureAlgorithm {
    /// Get the security level in bits
    pub fn security_level(&self) -> u32 {
        match self {
            SignatureAlgorithm::Dilithium2 => 128,
            SignatureAlgorithm::Dilithium3 => 192,
            SignatureAlgorithm::Dilithium5 => 256,
        }
    }
    
    /// Get the algorithm name
    pub fn name(&self) -> &'static str {
        match self {
            SignatureAlgorithm::Dilithium2 => "Dilithium2",
            SignatureAlgorithm::Dilithium3 => "Dilithium3",
            SignatureAlgorithm::Dilithium5 => "Dilithium5",
        }
    }
}

/// Complete capability token v2
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapTokenV2 {
    /// Token header
    pub header: CapTokenHeader,
    
    /// Token signature
    pub signature: CapTokenSignature,
    
    /// Token metadata
    pub metadata: CapTokenMetadata,
}

impl CapTokenV2 {
    /// Create a new capability token
    pub fn new(
        header: CapTokenHeader,
        signature: CapTokenSignature,
        metadata: CapTokenMetadata,
    ) -> Self {
        Self {
            header,
            signature,
            metadata,
        }
    }
    
    /// Validate the token structure
    pub fn is_well_formed(&self) -> bool {
        !self.header.issuer_did.is_empty() &&
        self.header.subject_pid > 0 &&
        self.header.dst_pid > 0 &&
        self.header.not_before < self.header.not_after &&
        !self.signature.dilithium_signature.is_empty()
    }
    
    /// Check if token is currently valid
    pub fn is_valid(&self, now_ms: u64) -> bool {
        self.is_well_formed() && self.header.is_valid(now_ms)
    }
    
    /// Check if token is expired
    pub fn is_expired(&self, now_ms: u64) -> bool {
        self.header.is_expired(now_ms)
    }
    
    /// Check if token is not yet valid
    pub fn is_early(&self, now_ms: u64) -> bool {
        self.header.is_early(now_ms)
    }
    
    /// Get token size in bytes
    pub fn size(&self) -> usize {
        self.header.size() + self.signature.size() + self.metadata.size()
    }
    
    /// Get token ID (hash of header)
    pub fn id(&self) -> u128 {
        use core::hash::{Hash, Hasher};
        use core::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.header.hash(&mut hasher);
        hasher.finish() as u128
    }
}

impl core::fmt::Display for CapTokenV2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CapTokenV2[{}]", self.header)
    }
}

/// Token metadata
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapTokenMetadata {
    /// Token creation timestamp
    pub created_at: u64,
    
    /// Token issuer signature
    pub issuer_signature: Vec<u8>,
    
    /// Additional context information
    pub context: BTreeMap<String, String>,
}

impl CapTokenMetadata {
    /// Create new metadata
    pub fn new(created_at: u64, issuer_signature: Vec<u8>) -> Self {
        Self {
            created_at,
            issuer_signature,
            context: BTreeMap::new(),
        }
    }
    
    /// Add context information
    pub fn add_context(&mut self, key: String, value: String) {
        self.context.insert(key, value);
    }
    
    /// Get context value
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }
    
    /// Get metadata size in bytes
    pub fn size(&self) -> usize {
        let base_size = 8 + self.issuer_signature.len(); // timestamp + signature
        let context_size: usize = self.context.iter()
            .map(|(k, v)| k.len() + v.len() + 2) // key + value + separators
            .sum();
        base_size + context_size
    }
}

/// Capability scope flags v2
pub mod scope_v2 {
    /// Permission to send messages
    pub const SEND: u32 = 0x1;
    
    /// Permission to receive messages
    pub const RECV: u32 = 0x2;
    
    /// Permission to create channels
    pub const CREATE_CHANNEL: u32 = 0x4;
    
    /// Permission to destroy channels
    pub const DESTROY_CHANNEL: u32 = 0x8;
    
    /// Administrative permissions
    pub const ADMIN: u32 = 0x10;
    
    /// Permission to delegate capabilities
    pub const DELEGATE: u32 = 0x20;
    
    /// Permission to revoke capabilities
    pub const REVOKE: u32 = 0x40;
    
    /// Permission to audit capabilities
    pub const AUDIT: u32 = 0x80;
    
    /// All permissions
    pub const ALL: u32 = SEND | RECV | CREATE_CHANNEL | DESTROY_CHANNEL | 
                         ADMIN | DELEGATE | REVOKE | AUDIT;
    
    /// Default permissions for normal processes
    pub const DEFAULT: u32 = SEND | RECV;
    
    /// Read-only permissions
    pub const READ_ONLY: u32 = RECV;
    
    /// Write-only permissions
    pub const WRITE_ONLY: u32 = SEND;
}

/// Capability token validation result
#[derive(Debug, Clone)]
pub struct CapValidationResult {
    /// Whether the token is valid
    pub is_valid: bool,
    
    /// Reason for validation failure (if any)
    pub failure_reason: Option<CapValidationFailure>,
    
    /// Validation timestamp
    pub validated_at: u64,
    
    /// Validation duration in microseconds
    pub validation_duration_us: u64,
}

impl CapValidationResult {
    /// Create successful validation result
    pub fn success(validation_duration_us: u64) -> Self {
        Self {
            is_valid: true,
            failure_reason: None,
            validated_at: get_current_time_ms(),
            validation_duration_us,
        }
    }
    
    /// Create failed validation result
    pub fn failure(reason: CapValidationFailure, validation_duration_us: u64) -> Self {
        Self {
            is_valid: false,
            failure_reason: Some(reason),
            validated_at: get_current_time_ms(),
            validation_duration_us,
        }
    }
}

/// Capability validation failure reasons
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapValidationFailure {
    /// Token is malformed
    Malformed,
    
    /// Token is expired
    Expired,
    
    /// Token is not yet valid
    Early,
    
    /// Token signature is invalid
    InvalidSignature,
    
    /// Token issuer is unknown
    UnknownIssuer,
    
    /// Token destination mismatch
    DestinationMismatch,
    
    /// Token scope insufficient
    InsufficientScope,
    
    /// Token nonce replay detected
    ReplayDetected,
    
    /// Token is revoked
    Revoked,
    
    /// Internal validation error
    InternalError,
}

impl CapValidationFailure {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CapValidationFailure::Malformed => "Token is malformed",
            CapValidationFailure::Expired => "Token is expired",
            CapValidationFailure::Early => "Token is not yet valid",
            CapValidationFailure::InvalidSignature => "Token signature is invalid",
            CapValidationFailure::UnknownIssuer => "Token issuer is unknown",
            CapValidationFailure::DestinationMismatch => "Token destination mismatch",
            CapValidationFailure::InsufficientScope => "Token scope insufficient",
            CapValidationFailure::ReplayDetected => "Token nonce replay detected",
            CapValidationFailure::Revoked => "Token is revoked",
            CapValidationFailure::InternalError => "Internal validation error",
        }
    }
    
    /// Get audit reason code
    pub fn audit_code(&self) -> u32 {
        match self {
            CapValidationFailure::Malformed => 0x1001,
            CapValidationFailure::Expired => 0x1002,
            CapValidationFailure::Early => 0x1003,
            CapValidationFailure::InvalidSignature => 0x1004,
            CapValidationFailure::UnknownIssuer => 0x1005,
            CapValidationFailure::DestinationMismatch => 0x1006,
            CapValidationFailure::InsufficientScope => 0x1007,
            CapValidationFailure::ReplayDetected => 0x1008,
            CapValidationFailure::Revoked => 0x1009,
            CapValidationFailure::InternalError => 0x100A,
        }
    }
}

/// Capability token builder
pub struct CapTokenBuilder {
    header: Option<CapTokenHeader>,
    signature: Option<CapTokenSignature>,
    metadata: Option<CapTokenMetadata>,
}

impl CapTokenBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            header: None,
            signature: None,
            metadata: None,
        }
    }
    
    /// Set the token header
    pub fn with_header(mut self, header: CapTokenHeader) -> Self {
        self.header = Some(header);
        self
    }
    
    /// Set the token signature
    pub fn with_signature(mut self, signature: CapTokenSignature) -> Self {
        self.signature = Some(signature);
        self
    }
    
    /// Set the token metadata
    pub fn with_metadata(mut self, metadata: CapTokenMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Build the capability token
    pub fn build(self) -> Result<CapTokenV2, CapTokenBuildError> {
        let header = self.header.ok_or(CapTokenBuildError::MissingHeader)?;
        let signature = self.signature.ok_or(CapTokenBuildError::MissingSignature)?;
        let metadata = self.metadata.ok_or(CapTokenBuildError::MissingMetadata)?;
        
        let token = CapTokenV2::new(header, signature, metadata);
        
        if !token.is_well_formed() {
            return Err(CapTokenBuildError::MalformedToken);
        }
        
        Ok(token)
    }
}

/// Capability token build errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapTokenBuildError {
    /// Missing header
    MissingHeader,
    /// Missing signature
    MissingSignature,
    /// Missing metadata
    MissingMetadata,
    /// Token is malformed
    MalformedToken,
}

impl core::fmt::Display for CapTokenBuildError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CapTokenBuildError::MissingHeader => write!(f, "Missing token header"),
            CapTokenBuildError::MissingSignature => write!(f, "Missing token signature"),
            CapTokenBuildError::MissingMetadata => write!(f, "Missing token metadata"),
            CapTokenBuildError::MalformedToken => write!(f, "Token is malformed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cap_token_header_creation() {
        let header = CapTokenHeader::new(
            "did:example:issuer".to_string(),
            1001,
            2001,
            scope_v2::SEND | scope_v2::RECV,
            1000,
            2000,
            0x1234567890abcdef,
            [0u8; 32],
        );
        
        assert_eq!(header.issuer_did, "did:example:issuer");
        assert_eq!(header.subject_pid, 1001);
        assert_eq!(header.dst_pid, 2001);
        assert_eq!(header.scope_flags, scope_v2::SEND | scope_v2::RECV);
        assert_eq!(header.not_before, 1000);
        assert_eq!(header.not_after, 2000);
        assert_eq!(header.nonce, 0x1234567890abcdef);
    }
    
    #[test]
    fn test_cap_token_header_validation() {
        let header = CapTokenHeader::new(
            "did:example:issuer".to_string(),
            1001,
            2001,
            scope_v2::SEND,
            1000,
            2000,
            0x1234567890abcdef,
            [0u8; 32],
        );
        
        // Test valid time range
        assert!(header.is_valid(1500));
        assert!(!header.is_expired(1500));
        assert!(!header.is_early(1500));
        
        // Test expired
        assert!(!header.is_valid(2500));
        assert!(header.is_expired(2500));
        
        // Test early
        assert!(!header.is_valid(500));
        assert!(header.is_early(500));
        
        // Test destination access
        assert!(header.grants_access_to(2001));
        assert!(!header.grants_access_to(3001));
        
        // Test scope permissions
        assert!(header.has_scope(scope_v2::SEND));
        assert!(!header.has_scope(scope_v2::RECV));
    }
    
    #[test]
    fn test_cap_token_builder() {
        let header = CapTokenHeader::new(
            "did:example:issuer".to_string(),
            1001,
            2001,
            scope_v2::SEND,
            1000,
            2000,
            0x1234567890abcdef,
            [0u8; 32],
        );
        
        let signature = CapTokenSignature::new(
            vec![1, 2, 3, 4],
            None,
            SignatureAlgorithm::Dilithium2,
        );
        
        let metadata = CapTokenMetadata::new(1500, vec![5, 6, 7, 8]);
        
        let token = CapTokenBuilder::new()
            .with_header(header)
            .with_signature(signature)
            .with_metadata(metadata)
            .build()
            .unwrap();
        
        assert!(token.is_well_formed());
        assert_eq!(token.header.subject_pid, 1001);
        assert_eq!(token.header.dst_pid, 2001);
    }
    
    #[test]
    fn test_signature_algorithm() {
        let algo = SignatureAlgorithm::Dilithium2;
        assert_eq!(algo.security_level(), 128);
        assert_eq!(algo.name(), "Dilithium2");
    }
}
