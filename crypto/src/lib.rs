//! Polymera OS Cryptography Module
//! 
//! This crate provides post-quantum cryptographic primitives for Polymera OS,
//! implementing NIST PQC standardization candidates with memory-safe Rust interfaces.
//! 
//! # Features
//! 
//! - **Kyber KEM**: Key Encapsulation Mechanism for secure key exchange
//! - **Dilithium**: Digital Signature Algorithm for authentication
//! - **Memory Safety**: Zeroization and automatic cleanup of sensitive data
//! - **Performance**: Optimized implementations with benchmarking support
//! - **Standards**: NIST PQC standardization candidates
//! 
//! # Quick Start
//! 
//! ```rust
//! use polymera_crypto::kyber::{KyberKem, KyberParameterSet};
//! use polymera_crypto::dilithium::{Dilithium, DilithiumParameterSet};
//! 
//! // Generate Kyber keypair for key exchange
//! let kem_params = KyberParameterSet::Kyber512;
//! let (kem_pub, kem_sec) = KyberKem::generate_keypair(kem_params)?;
//! 
//! // Generate Dilithium keypair for digital signatures
//! let sig_params = DilithiumParameterSet::Dilithium2;
//! let (sig_pub, sig_sec) = Dilithium::generate_keypair(sig_params)?;
//! 
//! // Use for secure communication
//! let (ciphertext, shared_secret) = KyberKem::encapsulate(&kem_pub)?;
//! let signature = Dilithium::sign(&sig_sec, b"Secure message")?;
//! 
//! // Verify and decrypt
//! let is_valid = Dilithium::verify(&sig_pub, b"Secure message", &signature)?;
//! let decrypted = KyberKem::decapsulate(&kem_sec, &ciphertext)?;
//! 
//! assert_eq!(shared_secret.as_slice(), decrypted.as_slice());
//! assert!(is_valid);
//! ```
//! 
//! # Algorithm Support
//! 
//! ## Kyber KEM
//! 
//! | Parameter Set | Security Level | Public Key | Secret Key | Ciphertext | Shared Secret |
//! |---------------|----------------|------------|------------|------------|---------------|
//! | Kyber512      | 128 bits       | 800 bytes  | 1632 bytes | 768 bytes  | 32 bytes      |
//! | Kyber768      | 192 bits       | 1184 bytes | 2400 bytes | 1088 bytes | 32 bytes      |
//! | Kyber1024     | 256 bits       | 1568 bytes | 3168 bytes | 1568 bytes | 32 bytes      |
//! 
//! ## Dilithium Digital Signatures
//! 
//! | Parameter Set | Security Level | Public Key | Secret Key | Signature |
//! |---------------|----------------|------------|------------|-----------|
//! | Dilithium2    | 128 bits       | 1312 bytes | 2528 bytes | 2420 bytes|
//! | Dilithium3    | 192 bits       | 1952 bytes | 4000 bytes | 3293 bytes|
//! | Dilithium5    | 256 bits       | 2592 bytes | 4864 bytes | 4595 bytes|
//! 
//! # Security Features
//! 
//! - **Zeroization**: All sensitive data is automatically zeroized on drop
//! - **Bounds Checking**: Runtime validation prevents buffer overflows
//! - **Ownership**: Clear ownership semantics prevent use-after-free
//! - **RAII**: Automatic resource cleanup prevents memory leaks
//! - **Constant-Time**: Sensitive operations use constant-time algorithms
//! 
//! # Performance
//! 
//! Performance measurements on Intel i7-10700K @ 3.80GHz:
//! 
//! - **Kyber512**: Key generation ~50μs, encapsulation ~40μs, decapsulation ~45μs
//! - **Kyber768**: Key generation ~75μs, encapsulation ~60μs, decapsulation ~65μs
//! - **Kyber1024**: Key generation ~100μs, encapsulation ~80μs, decapsulation ~85μs
//! - **Dilithium2**: Key generation ~100μs, signing ~80μs, verification ~60μs
//! - **Dilithium3**: Key generation ~150μs, signing ~120μs, verification ~90μs
//! - **Dilithium5**: Key generation ~200μs, signing ~160μs, verification ~120μs
//! 
//! # Feature Flags
//! 
//! ```toml
//! [dependencies]
//! polymera-crypto = { version = "0.1.0", features = ["kyber", "dilithium"] }
//! ```
//! 
//! - `default`: Basic functionality
//! - `kyber`: Enable Kyber KEM algorithms
//! - `dilithium`: Enable Dilithium signature algorithms
//! - `full`: Enable all features and optimizations
//! - `serde`: Enable serialization support
//! - `async`: Enable asynchronous operations
//! 
//! # Error Handling
//! 
//! The crate uses a comprehensive error handling system:
//! 
//! ```rust
//! use polymera_crypto::LibOqsError;
//! 
//! match KyberKem::generate_keypair(KyberParameterSet::Kyber512) {
//!     Ok((public_key, secret_key)) => {
//!         // Use keys...
//!     }
//!     Err(LibOqsError::Memory) => {
//!         eprintln!("Failed to allocate memory for keys");
//!     }
//!     Err(LibOqsError::Crypto(msg)) => {
//!         eprintln!("Cryptographic error: {}", msg);
//!     }
//!     Err(e) => {
//!         eprintln!("Unexpected error: {:?}", e);
//!     }
//! }
//! ```
//! 
//! # Testing
//! 
//! Run the test suite:
//! 
//! ```bash
//! cargo test
//! 
//! # Run specific tests
//! cargo test test_kyber_operations
//! cargo test test_dilithium_operations
//! 
//! # Run with features
//! cargo test --features "full"
//! ```
//! 
//! # Benchmarking
//! 
//! Run performance benchmarks:
//! 
//! ```bash
//! cargo bench
//! 
//! # Run specific benchmarks
//! cargo bench kyber_operations
//! cargo bench dilithium_operations
//! ```
//! 
//! # Building
//! 
//! ## Prerequisites
//! 
//! - Rust 1.70+ with Cargo
//! - CMake 3.16+
//! - Git
//! - C compiler (GCC/Clang)
//! - OpenSSL development libraries
//! 
//! ## Build Commands
//! 
//! ```bash
//! # Build with bundled liboqs (default)
//! cargo build --release
//! 
//! # Build with system liboqs
//! LIBOQS_USE_SYSTEM=1 cargo build --release
//! 
//! # Build with specific features
//! cargo build --release --features "kyber,dilithium,full"
//! ```
//! 
//! # License
//! 
//! This project is licensed under either of
//! 
//! * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
//! * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
//! 
//! at your option.
//! 
//! # Contributing
//! 
//! Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.
//! 
//! # Security
//! 
//! This crate is designed with security in mind, but cryptographic implementations
//! should always be reviewed by security experts. Please report any security issues
//! to [security@polymera-os.org](mailto:security@polymera-os.org).

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

// Re-export the main crypto module
pub use crate::crypto::*;
pub use crate::traits::*;

// Internal module structure
mod crypto;
pub mod traits;

// Re-export version and metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = "Post-Quantum Cryptography for Polymera OS";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(!DESCRIPTION.is_empty());
    }

    #[test]
    fn test_crypto_module() {
        // Test that the crypto module can be imported
        use crypto::*;
        
        // Test basic functionality
        let algorithms = supported_algorithms();
        assert!(!algorithms.is_empty());
        
        let kyber_count = algorithms.iter().filter(|alg| alg.is_kem()).count();
        let sig_count = algorithms.iter().filter(|alg| alg.is_signature()).count();
        
        assert_eq!(kyber_count, 3); // Kyber512, Kyber768, Kyber1024
        assert_eq!(sig_count, 3);   // Dilithium2, Dilithium3, Dilithium5
    }
}
