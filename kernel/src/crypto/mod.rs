//! Cryptographic primitives module for the Polymera OS kernel.
//!
//! ## Architecture
//!
//! The kernel itself does NOT perform real cryptographic operations.
//! All signing, verification, key generation, and KEM are delegated
//! to the `keyvault` system service via IPC, which in turn uses the
//! standalone `crypto/` crate (real liboqs FFI).
//!
//! This module provides:
//! - **Opaque type stubs** for PQC key/signature blobs (for IPC message typing)
//! - **Error types** shared across the crypto boundary
//! - The `dilithium` submodule (re-exports from `pqc`)
//!
//! For the real PQC implementation, see `crypto/` at the repo root.

pub mod pqc;
pub mod dilithium;

use alloc::vec::Vec;
use alloc::string::String;

/// Cryptographic operation error.
#[derive(Debug, Clone)]
pub enum CryptoError {
    InvalidKey,
    InvalidSignature,
    VerificationFailed,
    EncryptionFailed,
    DecryptionFailed,
    KeyGenerationFailed,
    InvalidParameter,
}

impl core::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoError::InvalidKey => write!(f, "Invalid key"),
            CryptoError::InvalidSignature => write!(f, "Invalid signature"),
            CryptoError::VerificationFailed => write!(f, "Verification failed"),
            CryptoError::EncryptionFailed => write!(f, "Encryption failed"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed"),
            CryptoError::KeyGenerationFailed => write!(f, "Key generation failed"),
            CryptoError::InvalidParameter => write!(f, "Invalid parameter"),
        }
    }
}

pub type CryptoResult<T> = Result<T, CryptoError>;
