//! Stable cryptography traits for Polymera OS.
//!
//! These traits define the public PQC boundary used by Web3, WIT hosts, wallet,
//! chain anchoring, and kernel facades. Concrete implementations may use
//! liboqs-compatible wrappers, pure Rust crates, or verified extracted C, but
//! callers should depend on this module instead of backend-specific names.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

/// Error returned by stable crypto trait implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoTraitError {
    /// The requested algorithm or parameter set is not supported by this backend.
    UnsupportedAlgorithm(String),
    /// Input bytes are malformed, truncated, or otherwise invalid.
    InvalidInput(String),
    /// Verification failed.
    VerificationFailed,
    /// Entropy source failed.
    EntropyUnavailable(String),
    /// Backend-specific failure.
    Backend(String),
}

impl core::fmt::Display for CryptoTraitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedAlgorithm(msg) => write!(f, "unsupported algorithm: {msg}"),
            Self::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Self::VerificationFailed => write!(f, "verification failed"),
            Self::EntropyUnavailable(msg) => write!(f, "entropy unavailable: {msg}"),
            Self::Backend(msg) => write!(f, "backend error: {msg}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CryptoTraitError {}

/// Result alias for stable crypto trait operations.
pub type CryptoTraitResult<T> = Result<T, CryptoTraitError>;

/// Public/secret keypair represented as opaque bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeypairBytes {
    /// Public key bytes.
    pub public_key: Vec<u8>,
    /// Secret key bytes. Implementations that store this type long-term must
    /// zeroize this field on drop or wrap it in a zeroizing container.
    pub secret_key: Vec<u8>,
}

/// KEM output represented as ciphertext plus shared secret bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KemEncapsulation {
    /// Ciphertext sent to the decapsulating party.
    pub ciphertext: Vec<u8>,
    /// Shared secret derived by encapsulation.
    pub shared_secret: Vec<u8>,
}

/// Stable KEM trait for Kyber/ML-KEM-style algorithms.
pub trait Kem {
    /// Generate a fresh KEM keypair.
    fn generate_keypair(&self) -> CryptoTraitResult<KeypairBytes>;

    /// Encapsulate to a public key.
    fn encapsulate(&self, public_key: &[u8]) -> CryptoTraitResult<KemEncapsulation>;

    /// Decapsulate a ciphertext using a secret key.
    fn decapsulate(&self, secret_key: &[u8], ciphertext: &[u8]) -> CryptoTraitResult<Vec<u8>>;
}

/// Stable signature trait for Dilithium/ML-DSA-style algorithms.
pub trait SignatureScheme {
    /// Generate a fresh signing keypair.
    fn generate_keypair(&self) -> CryptoTraitResult<KeypairBytes>;

    /// Sign a message using secret key bytes.
    fn sign(&self, secret_key: &[u8], message: &[u8]) -> CryptoTraitResult<Vec<u8>>;

    /// Verify a signature against public key bytes.
    fn verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> CryptoTraitResult<()>;
}

/// Hybrid signature envelope used by Web3 anchors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridSignatureEnvelope {
    /// Classical signature algorithm, usually `secp256k1`.
    pub classical_algorithm: String,
    /// Classical signature bytes verified on-chain where possible.
    pub classical_signature: Vec<u8>,
    /// PQC signature algorithm, usually `dilithium3`.
    pub pqc_algorithm: String,
    /// Hash of the PQC public key used by off-chain verifiers.
    pub pqc_public_key_hash: [u8; 32],
    /// Hash of the PQC signature bytes stored on-chain.
    pub pqc_signature_hash: [u8; 32],
    /// Digest of the signed payload.
    pub payload_hash: [u8; 32],
}

/// Stable hybrid-signature policy trait.
pub trait HybridSignature {
    /// Produce a hybrid signature envelope for a payload hash.
    fn sign_hybrid(&self, payload_hash: &[u8; 32]) -> CryptoTraitResult<HybridSignatureEnvelope>;

    /// Verify the complete hybrid envelope.
    fn verify_hybrid(&self, envelope: &HybridSignatureEnvelope) -> CryptoTraitResult<()>;
}

/// Stable entropy trait for platform RNG and QRNG adapters.
pub trait SecureEntropy {
    /// Fill `dest` with cryptographically suitable entropy.
    fn fill_random(&self, dest: &mut [u8]) -> CryptoTraitResult<()>;
}
