//! Dilithium signature scheme re-exports.
//!
//! ⚠️  These are insecure placeholder types — see `pqc.rs` for details.

#[allow(deprecated)]
pub use super::pqc::DilithiumPublicKey as DilithiumPubKey;
#[allow(deprecated)]
pub use super::pqc::DilithiumSignature;
