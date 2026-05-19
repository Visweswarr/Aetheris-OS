//! ⚠️  INSECURE TOY CRYPTOGRAPHY — DO NOT USE FOR SECURITY ⚠️
//!
//! Opaque Kyber KEM type stubs for kernel IPC message typing.
//! Real Kyber lives in the standalone `crypto/` crate.

use alloc::vec::Vec;
use super::super::{CryptoResult, CryptoError};

/// Kyber parameter set identifiers (type-level only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KyberParameterSet {
    Kyber512,
    Kyber768,
    Kyber1024,
}

/// ⚠️  INSECURE — Opaque container for Kyber public key bytes.
#[derive(Debug, Clone)]
pub struct InsecureKyberPublicKey {
    pub data: Vec<u8>,
    pub parameter_set: KyberParameterSet,
}

impl InsecureKyberPublicKey {
    pub fn new(data: Vec<u8>, parameter_set: KyberParameterSet) -> Self {
        Self { data, parameter_set }
    }
}

// Backward-compatible alias.
#[deprecated(note = "Renamed to InsecureKyberPublicKey — this type provides no real crypto")]
pub type KyberPublicKey = InsecureKyberPublicKey;

/// ⚠️  INSECURE — Opaque container for Kyber secret key bytes.
#[derive(Debug, Clone)]
pub struct InsecureKyberSecretKey {
    pub data: Vec<u8>,
    pub parameter_set: KyberParameterSet,
}

impl InsecureKyberSecretKey {
    pub fn new(data: Vec<u8>, parameter_set: KyberParameterSet) -> Self {
        Self { data, parameter_set }
    }
}

#[deprecated(note = "Renamed to InsecureKyberSecretKey — this type provides no real crypto")]
pub type KyberSecretKey = InsecureKyberSecretKey;

/// ⚠️  INSECURE — Stub KEM that produces ZEROS.
///
/// All operations return zeroed byte vectors. These are structurally valid
/// but cryptographically meaningless. Real KEM is in the `crypto/` crate.
pub struct InsecureKyberKem;

impl InsecureKyberKem {
    /// Returns a zeroed keypair. NOT REAL KEY GENERATION.
    pub fn generate_keypair(parameter_set: KyberParameterSet) -> CryptoResult<(InsecureKyberPublicKey, InsecureKyberSecretKey)> {
        let pk = InsecureKyberPublicKey::new(alloc::vec![0u8; 32], parameter_set);
        let sk = InsecureKyberSecretKey::new(alloc::vec![0u8; 32], parameter_set);
        Ok((pk, sk))
    }

    /// Returns zeroed (ciphertext, shared_secret). NOT REAL ENCAPSULATION.
    pub fn encapsulate(_public_key: &InsecureKyberPublicKey) -> CryptoResult<(Vec<u8>, Vec<u8>)> {
        Ok((alloc::vec![0u8; 32], alloc::vec![0u8; 32]))
    }

    /// Returns zeroed shared_secret. NOT REAL DECAPSULATION.
    pub fn decapsulate(_secret_key: &InsecureKyberSecretKey, _ciphertext: &[u8]) -> CryptoResult<Vec<u8>> {
        Ok(alloc::vec![0u8; 32])
    }
}

// Backward-compatible alias.
#[deprecated(note = "Renamed to InsecureKyberKem — this type provides no real crypto")]
pub type KyberKem = InsecureKyberKem;
