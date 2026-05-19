//! ⚠️  INSECURE TOY CRYPTOGRAPHY — DO NOT USE FOR SECURITY ⚠️
//!
//! This module provides **placeholder** post-quantum cryptographic types
//! for the in-kernel type system. It does NOT implement real PQC.
//!
//! **Why this exists**: The kernel's `no_std` environment cannot link against
//! the real `crypto/` crate (which uses liboqs FFI and requires `std`).
//! These types exist solely so that kernel IPC messages and capability tokens
//! can reference PQC key material and signatures in their type signatures.
//!
//! **Where the real PQC lives**: `crypto/` crate at the repo root, which
//! wraps liboqs and provides actual Dilithium/Kyber/SPHINCS+ via C FFI.
//!
//! **Security invariant**: All actual cryptographic operations MUST be
//! delegated to the `crypto/` crate via IPC. The kernel never performs
//! signing or verification itself — it only stores and routes opaque
//! byte blobs that the `keyvault` service interprets.
//!
//! If you are reading this and thinking "I'll just call `verify()` from
//! kernel code" — DON'T. It always returns `false` precisely so that
//! accidental use fails closed rather than open.

pub mod mem;
pub mod kyber;

use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// Compile-time guard: prevent accidental use in release builds
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(all(not(debug_assertions), not(feature = "insecure-toy-crypto")))]
compile_error!(
    "In-kernel PQC types are insecure placeholders. \
     Real PQC is in the standalone `crypto/` crate. \
     If you genuinely need these stubs in a release build, \
     enable `--features insecure-toy-crypto`."
);

// ─────────────────────────────────────────────────────────────────────────────
// Dilithium Parameter Sets (type-level only — no real crypto)
// ─────────────────────────────────────────────────────────────────────────────

/// Dilithium parameter set identifiers.
///
/// These are used for message typing only. No actual lattice arithmetic
/// is performed in this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DilithiumParameterSet {
    Dilithium2,
    Dilithium3,
    Dilithium5,
}

// ─────────────────────────────────────────────────────────────────────────────
// Insecure Placeholder Types
// ─────────────────────────────────────────────────────────────────────────────

/// ⚠️  INSECURE — Opaque container for a Dilithium public key blob.
///
/// This struct holds serialized key bytes for routing via IPC.
/// It does NOT perform any cryptographic operations.
/// Call the `keyvault` service over IPC for real verification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InsecureDilithiumPublicKey {
    /// Raw public key bytes (opaque — interpreted by `keyvault` service).
    pub data: Vec<u8>,
    /// Parameter set tag.
    pub parameter_set: DilithiumParameterSet,
}

impl InsecureDilithiumPublicKey {
    pub fn new(data: Vec<u8>, parameter_set: DilithiumParameterSet) -> Self {
        Self { data, parameter_set }
    }

    /// ⚠️  ALWAYS RETURNS FALSE.
    ///
    /// The kernel cannot verify Dilithium signatures. This method exists
    /// only to satisfy trait bounds and type signatures. It intentionally
    /// fails closed so that any accidental call path rejects the signature
    /// rather than accepting a forgery.
    ///
    /// For real verification, send the (message, signature, public_key) triple
    /// to the `keyvault` service via IPC syscall `SYS_CRYPTO_VERIFY`.
    #[inline(always)]
    pub fn verify(&self, _message: &[u8], _signature: &InsecureDilithiumSignature) -> bool {
        // SECURITY: Always reject. Real verification is via keyvault IPC.
        false
    }
}

// Backward-compatible aliases for code that still uses the old names.
// These are deprecated and will be removed in v0.6.0.
#[deprecated(note = "Renamed to InsecureDilithiumPublicKey — this type provides no real crypto")]
pub type DilithiumPublicKey = InsecureDilithiumPublicKey;

/// ⚠️  INSECURE — Opaque container for a Dilithium signature blob.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InsecureDilithiumSignature {
    /// Raw signature bytes (opaque).
    pub data: Vec<u8>,
}

impl InsecureDilithiumSignature {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

#[deprecated(note = "Renamed to InsecureDilithiumSignature — this type provides no real crypto")]
pub type DilithiumSignature = InsecureDilithiumSignature;
