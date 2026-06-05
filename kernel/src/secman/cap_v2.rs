//! Capability Token V2 stubs

use alloc::vec::Vec;
use alloc::string::String;

/// Capability token V2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapTokenV2 {
    pub header: CapTokenHeader,
    pub signature: CapTokenSignature,
    pub metadata: CapTokenMetadata,
}

impl CapTokenV2 {
    pub fn new(header: CapTokenHeader, signature: CapTokenSignature, metadata: CapTokenMetadata) -> Self {
        Self { header, signature, metadata }
    }
}

/// Capability token header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapTokenHeader {
    pub version: u8,
    pub cap_type: u16,
    pub permissions: u32,
    pub issuer_id: u64,
    pub subject_id: u64,
    pub expiry: u64,
}

impl CapTokenHeader {
    pub fn new(cap_type: u16, permissions: u32) -> Self {
        Self {
            version: 2,
            cap_type,
            permissions,
            issuer_id: 0,
            subject_id: 0,
            expiry: 0,
        }
    }
}

/// Capability token signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapTokenSignature {
    pub algorithm: SignatureAlgorithm,
    pub data: Vec<u8>,
}

impl CapTokenSignature {
    pub fn new(algorithm: SignatureAlgorithm, data: Vec<u8>) -> Self {
        Self { algorithm, data }
    }
}

/// Signature algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    Ed25519,
    Dilithium2,
    Dilithium3,
    Dilithium5,
}

/// Capability token metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapTokenMetadata {
    pub created_at: u64,
    pub tags: Vec<String>,
}

impl CapTokenMetadata {
    pub fn new(created_at: u64, tags: Vec<String>) -> Self {
        Self { created_at, tags }
    }
}

/// Capability validation result
#[derive(Debug, Clone)]
pub enum CapValidationResult {
    Valid,
    Invalid(CapValidationFailure),
}

/// Capability validation failure reasons
#[derive(Debug, Clone)]
pub enum CapValidationFailure {
    Expired,
    InvalidSignature,
    InsufficientPermissions,
    RevokedCapability,
    InvalidIssuer,
    InvalidSubject,
    MalformedToken,
}
