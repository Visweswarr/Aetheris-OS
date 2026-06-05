//! CapToken v2 (COSE_Sign1 + CBOR claims) — security core (library skeleton)
//! This module defines types and a verifier for online verification.
//! Note: Not yet wired; kernel currently uses its own policy engine; echo server uses a local verifier.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub iat: u64,
    pub nbf: u64,
    pub exp: u64,
    pub jti: Vec<u8>, // 16 bytes
    pub scopes: Vec<String>,
    #[serde(default)]
    pub ctx: Option<serde_cbor::Value>,
}

#[derive(Debug)]
pub enum CapError {
    Invalid(String),
    Expired,
    NotYetValid,
    Audience,
    Replay,
}

pub struct JwkKey { pub kid: Option<String>, pub x: Vec<u8> }

pub struct Jwks { pub keys: Vec<JwkKey> }

pub struct Verifier {
    pub jwks: Jwks,
    pub audience: String,
    pub skew: i64,
}

impl Verifier {
    pub fn verify(&self, _bytes: &[u8]) -> Result<CapClaims, CapError> {
        // Placeholder skeleton: actual signature/time checks implemented in kernel/tools.
        Err(CapError::Invalid("not wired".into()))
    }
}
