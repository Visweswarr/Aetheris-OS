//! Decentralized Identifier (DID) stubs

use alloc::string::String;
use alloc::vec::Vec;

/// DID document
#[derive(Debug, Clone)]
pub struct DidDocument {
    pub id: String,
    pub controller: Option<String>,
    pub verification_methods: Vec<VerificationMethod>,
}

impl DidDocument {
    pub fn new(id: String) -> Self {
        Self {
            id,
            controller: None,
            verification_methods: Vec::new(),
        }
    }
}

/// Verification method
#[derive(Debug, Clone)]
pub struct VerificationMethod {
    pub id: String,
    pub method_type: String,
    pub controller: String,
    pub public_key: Vec<u8>,
}

/// DID resolver
pub struct DidResolver;

impl DidResolver {
    pub fn new() -> Self {
        Self
    }
    
    pub fn resolve(&self, _did: &str) -> Option<DidDocument> {
        // Stub: returns None
        None
    }
}
