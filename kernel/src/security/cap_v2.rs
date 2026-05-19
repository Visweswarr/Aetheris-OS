//! Capability Token V2 for Polymera OS
//!
//! This module provides the V2 capability token implementation with
//! post-quantum cryptographic signatures.

use alloc::vec::Vec;
use alloc::string::String;

/// Re-export of `crate::security::scope_v2` so callers can use `cap_v2::scope_v2`.
pub use super::scope_v2;

/// Capability Token V2 with PQC signatures
#[derive(Debug, Clone)]
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
    pub fn new(header: CapTokenHeader, signature: CapTokenSignature, metadata: CapTokenMetadata) -> Self {
        Self { header, signature, metadata }
    }
    
    /// Validate the token
    pub fn validate(&self) -> CapValidationResult {
        // Basic validation - check expiry
        let current_time = crate::security::get_current_time_ms();
        if self.header.expiry_ms < current_time {
            return CapValidationResult::Failure(CapValidationFailure::Expired);
        }
        CapValidationResult::Success
    }

    /// Best-effort zeroization for broker session teardown.
    pub fn zeroize(&mut self) {
        self.signature.bytes.fill(0);
        self.metadata.attributes.clear();
        self.header.id = 0;
        self.header.dst = 0;
        self.header.scope = 0;
        self.header.expiry_ms = 0;
    }
}

/// Capability token header
#[derive(Debug, Clone)]
pub struct CapTokenHeader {
    /// Token ID
    pub id: u128,
    /// Destination process ID
    pub dst: u64,
    /// Capability scope flags
    pub scope: u64,
    /// Expiry timestamp in milliseconds
    pub expiry_ms: u64,
    /// Version
    pub version: u8,
}

impl CapTokenHeader {
    /// Create a new header
    pub fn new(id: u128, dst: u64, scope: u64, expiry_ms: u64) -> Self {
        Self {
            id,
            dst,
            scope,
            expiry_ms,
            version: 2,
        }
    }

    /// Stable byte representation used for capability IDs and MAC material.
    pub fn to_bytes(&self) -> [u8; 41] {
        let mut out = [0u8; 41];
        out[0..16].copy_from_slice(&self.id.to_le_bytes());
        out[16..24].copy_from_slice(&self.dst.to_le_bytes());
        out[24..32].copy_from_slice(&self.scope.to_le_bytes());
        out[32..40].copy_from_slice(&self.expiry_ms.to_le_bytes());
        out[40] = self.version;
        out
    }
}

/// Capability token signature
#[derive(Debug, Clone)]
pub struct CapTokenSignature {
    /// Signature algorithm
    pub algorithm: SignatureAlgorithm,
    /// Signature bytes
    pub bytes: Vec<u8>,
}

impl CapTokenSignature {
    /// Create a new signature
    pub fn new(algorithm: SignatureAlgorithm, bytes: Vec<u8>) -> Self {
        Self { algorithm, bytes }
    }
}

/// Signature algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    /// Ed25519 signature
    Ed25519,
    /// Dilithium post-quantum signature
    Dilithium,
    /// None (for testing)
    None,
}

/// Capability token metadata
#[derive(Debug, Clone)]
pub struct CapTokenMetadata {
    /// Issuer ID
    pub issuer_id: u64,
    /// Additional attributes
    pub attributes: Vec<(String, String)>,
}

impl CapTokenMetadata {
    /// Create new metadata
    pub fn new(issuer_id: u64, attributes: Vec<(String, String)>) -> Self {
        Self { issuer_id, attributes }
    }
}

/// Capability validation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapValidationResult {
    /// Validation succeeded
    Success,
    /// Validation failed
    Failure(CapValidationFailure),
}

/// Capability validation failure reasons
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapValidationFailure {
    /// Token has expired
    Expired,
    /// Invalid signature
    InvalidSignature,
    /// Invalid scope
    InvalidScope,
    /// Token revoked
    Revoked,
    /// Unknown error
    Unknown,
}

impl core::fmt::Display for CapValidationFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CapValidationFailure::Expired => write!(f, "Token expired"),
            CapValidationFailure::InvalidSignature => write!(f, "Invalid signature"),
            CapValidationFailure::InvalidScope => write!(f, "Invalid scope"),
            CapValidationFailure::Revoked => write!(f, "Token revoked"),
            CapValidationFailure::Unknown => write!(f, "Unknown error"),
        }
    }
}
