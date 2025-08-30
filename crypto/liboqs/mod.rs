//! Post-Quantum Cryptography implementation for Polymera OS
//! 
//! This module provides safe Rust wrappers around the liboqs C library,
//! implementing NIST PQC standardization candidates:
//! 
//! - **Kyber**: Key Encapsulation Mechanism (KEM)
//! - **Dilithium**: Digital Signature Algorithm
//! 
//! # Features
//! 
//! - Memory-safe Rust interfaces with automatic cleanup
//! - Zeroization of sensitive data
//! - Comprehensive error handling
//! - Test vectors for validation
//! - Feature flags for modular compilation
//! 
//! # Example Usage
//! 
//! ```rust
//! use polymera_crypto::kyber::{KyberKem, KyberParameterSet};
//! use polymera_crypto::dilithium::{Dilithium, DilithiumParameterSet};
//! 
//! // Generate Kyber keypair
//! let params = KyberParameterSet::Kyber512;
//! let (public_key, secret_key) = KyberKem::generate_keypair(params)?;
//! 
//! // Encapsulate shared secret
//! let (ciphertext, shared_secret) = KyberKem::encapsulate(&public_key)?;
//! 
//! // Decapsulate shared secret
//! let decapsulated = KyberKem::decapsulate(&secret_key, &ciphertext)?;
//! 
//! // Generate Dilithium keypair
//! let sig_params = DilithiumParameterSet::Dilithium2;
//! let (sig_pub, sig_sec) = Dilithium::generate_keypair(sig_params)?;
//! 
//! // Sign and verify
//! let message = b"Hello, Polymera OS!";
//! let signature = Dilithium::sign(&sig_sec, message)?;
//! let is_valid = Dilithium::verify(&sig_pub, message, &signature)?;
//! ```

pub mod error;
pub mod bindings;
pub mod kyber;
pub mod dilithium;

// Re-export main types and functions
pub use error::{LibOqsError, LibOqsResult};
pub use kyber::{KyberKem, KyberParameterSet, KyberPublicKey, KyberSecretKey, KyberCiphertext, KyberSharedSecret};
pub use dilithium::{Dilithium, DilithiumParameterSet, DilithiumPublicKey, DilithiumSecretKey, DilithiumSignature};

/// Version information
pub const LIBOQS_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const LIBOQS_DESCRIPTION: &str = "Post-Quantum Cryptography for Polymera OS";

/// Supported algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    /// Kyber KEM
    Kyber(KyberParameterSet),
    /// Dilithium signature
    Dilithium(DilithiumParameterSet),
}

impl Algorithm {
    /// Get the algorithm name
    pub fn name(&self) -> &'static str {
        match self {
            Algorithm::Kyber(params) => params.algorithm_name(),
            Algorithm::Dilithium(params) => params.algorithm_name(),
        }
    }

    /// Get the security level in bits
    pub fn security_level(&self) -> u32 {
        match self {
            Algorithm::Kyber(params) => params.security_level(),
            Algorithm::Dilithium(params) => params.security_level(),
        }
    }

    /// Check if this is a KEM algorithm
    pub fn is_kem(&self) -> bool {
        matches!(self, Algorithm::Kyber(_))
    }

    /// Check if this is a signature algorithm
    pub fn is_signature(&self) -> bool {
        matches!(self, Algorithm::Dilithium(_))
    }
}

/// Algorithm family
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmFamily {
    /// Key Encapsulation Mechanism
    Kem,
    /// Digital Signature
    Signature,
}

impl AlgorithmFamily {
    /// Get all algorithms in this family
    pub fn algorithms(&self) -> Vec<Algorithm> {
        match self {
            AlgorithmFamily::Kem => vec![
                Algorithm::Kyber(KyberParameterSet::Kyber512),
                Algorithm::Kyber(KyberParameterSet::Kyber768),
                Algorithm::Kyber(KyberParameterSet::Kyber1024),
            ],
            AlgorithmFamily::Signature => vec![
                Algorithm::Dilithium(DilithiumParameterSet::Dilithium2),
                Algorithm::Dilithium(DilithiumParameterSet::Dilithium3),
                Algorithm::Dilithium(DilithiumParameterSet::Dilithium5),
            ],
        }
    }
}

/// Get all supported algorithms
pub fn supported_algorithms() -> Vec<Algorithm> {
    let mut algorithms = Vec::new();
    
    // Add Kyber variants
    algorithms.extend([
        Algorithm::Kyber(KyberParameterSet::Kyber512),
        Algorithm::Kyber(KyberParameterSet::Kyber768),
        Algorithm::Kyber(KyberParameterSet::Kyber1024),
    ]);
    
            // Add Dilithium variants
        algorithms.extend([
            Algorithm::Dilithium(DilithiumParameterSet::Dilithium2),
            Algorithm::Dilithium(DilithiumParameterSet::Dilithium3),
            Algorithm::Dilithium(DilithiumParameterSet::Dilithium5),
        ]);
    
    algorithms
}

/// Get algorithms by security level
pub fn algorithms_by_security_level(level: u32) -> Vec<Algorithm> {
    supported_algorithms()
        .into_iter()
        .filter(|alg| alg.security_level() == level)
        .collect()
}

/// Get algorithms by family
pub fn algorithms_by_family(family: AlgorithmFamily) -> Vec<Algorithm> {
    family.algorithms()
}

/// Check if an algorithm is supported
pub fn is_algorithm_supported(algorithm: &Algorithm) -> bool {
    supported_algorithms().contains(algorithm)
}

/// Get algorithm information
pub fn get_algorithm_info(algorithm: &Algorithm) -> AlgorithmInfo {
    match algorithm {
        Algorithm::Kyber(params) => AlgorithmInfo {
            name: params.algorithm_name(),
            family: AlgorithmFamily::Kem,
            security_level: params.security_level(),
            public_key_length: params.public_key_length(),
            secret_key_length: params.secret_key_length(),
            additional_info: match params {
                KyberParameterSet::Kyber512 => "Ciphertext: 768 bytes, Shared Secret: 32 bytes",
                KyberParameterSet::Kyber768 => "Ciphertext: 1088 bytes, Shared Secret: 32 bytes",
                KyberParameterSet::Kyber1024 => "Ciphertext: 1568 bytes, Shared Secret: 32 bytes",
            },
        },
        Algorithm::Dilithium(params) => AlgorithmInfo {
            name: params.algorithm_name(),
            family: AlgorithmFamily::Signature,
            security_level: params.security_level(),
            public_key_length: params.public_key_length(),
            secret_key_length: params.secret_key_length(),
            additional_info: match params {
                DilithiumParameterSet::Dilithium2 => "Signature: 2420 bytes",
                DilithiumParameterSet::Dilithium3 => "Signature: 3293 bytes",
                DilithiumParameterSet::Dilithium5 => "Signature: 4595 bytes",
            },
        },
    }
}

/// Algorithm information
#[derive(Debug, Clone)]
pub struct AlgorithmInfo {
    pub name: &'static str,
    pub family: AlgorithmFamily,
    pub security_level: u32,
    pub public_key_length: usize,
    pub secret_key_length: usize,
    pub additional_info: &'static str,
}

impl AlgorithmInfo {
    /// Get a formatted description
    pub fn description(&self) -> String {
        format!(
            "{} ({}-bit security) - Public Key: {} bytes, Secret Key: {} bytes, {}",
            self.name,
            self.security_level,
            self.public_key_length,
            self.secret_key_length,
            self.additional_info
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_enum() {
        let kyber = Algorithm::Kyber(KyberParameterSet::Kyber512);
        assert_eq!(kyber.name(), "Kyber512");
        assert_eq!(kyber.security_level(), 128);
        assert!(kyber.is_kem());
        assert!(!kyber.is_signature());

        let dilithium = Algorithm::Dilithium(DilithiumParameterSet::Dilithium2);
        assert_eq!(dilithium.name(), "Dilithium2");
        assert_eq!(dilithium.security_level(), 128);
        assert!(!dilithium.is_kem());
        assert!(dilithium.is_signature());
    }

    #[test]
    fn test_algorithm_family() {
        let kem_algorithms = AlgorithmFamily::Kem.algorithms();
        assert_eq!(kem_algorithms.len(), 3);
        assert!(kem_algorithms.iter().all(|alg| alg.is_kem()));

        let sig_algorithms = AlgorithmFamily::Signature.algorithms();
        assert_eq!(sig_algorithms.len(), 3);
        assert!(sig_algorithms.iter().all(|alg| alg.is_signature()));
    }

    #[test]
    fn test_supported_algorithms() {
        let algorithms = supported_algorithms();
        assert_eq!(algorithms.len(), 6); // 3 Kyber + 3 Dilithium
        
        let kyber_count = algorithms.iter().filter(|alg| alg.is_kem()).count();
        let sig_count = algorithms.iter().filter(|alg| alg.is_signature()).count();
        
        assert_eq!(kyber_count, 3);
        assert_eq!(sig_count, 3);
    }

    #[test]
    fn test_algorithms_by_security_level() {
        let level_128 = algorithms_by_security_level(128);
        assert_eq!(level_128.len(), 2); // Kyber512 + Dilithium2
        
        let level_192 = algorithms_by_security_level(192);
        assert_eq!(level_192.len(), 2); // Kyber768 + Dilithium3
        
        let level_256 = algorithms_by_security_level(256);
        assert_eq!(level_256.len(), 2); // Kyber1024 + Dilithium5
    }

    #[test]
    fn test_algorithm_info() {
        let kyber_info = get_algorithm_info(&Algorithm::Kyber(KyberParameterSet::Kyber512));
        assert_eq!(kyber_info.name, "Kyber512");
        assert_eq!(kyber_info.family, AlgorithmFamily::Kem);
        assert_eq!(kyber_info.security_level, 128);
        assert_eq!(kyber_info.public_key_length, 800);
        assert_eq!(kyber_info.secret_key_length, 1632);
        assert!(kyber_info.additional_info.contains("768 bytes"));

        let dilithium_info = get_algorithm_info(&Algorithm::Dilithium(DilithiumParameterSet::Dilithium2));
        assert_eq!(dilithium_info.name, "Dilithium2");
        assert_eq!(dilithium_info.family, AlgorithmFamily::Signature);
        assert_eq!(dilithium_info.security_level, 128);
        assert_eq!(dilithium_info.public_key_length, 1312);
        assert_eq!(dilithium_info.secret_key_length, 2528);
        assert!(dilithium_info.additional_info.contains("2420 bytes"));
    }

    #[test]
    fn test_algorithm_support() {
        let kyber = Algorithm::Kyber(KyberParameterSet::Kyber512);
        assert!(is_algorithm_supported(&kyber));

        let dilithium = Algorithm::Dilithium(DilithiumParameterSet::Dilithium3);
        assert!(is_algorithm_supported(&dilithium));
    }
}
