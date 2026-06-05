//! Capability store stubs

use alloc::vec::Vec;
use alloc::string::String;
use super::cap_v2::{CapTokenV2, CapValidationResult, CapValidationFailure};

/// Capability store
pub struct CapStore {
    // Stub implementation
}

impl CapStore {
    pub fn new() -> Self {
        Self {}
    }
}

/// Validate a capability token
pub fn validate_capability_token(_token: &CapTokenV2) -> CapValidationResult {
    // Stub: always valid in development
    CapValidationResult::Valid
}
