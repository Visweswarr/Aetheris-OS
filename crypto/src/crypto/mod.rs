//! Main crypto module implementation

pub mod liboqs;

// Re-export main types and functions
pub use liboqs::{
    // Core types
    Algorithm, AlgorithmFamily, AlgorithmInfo,
    
    // Kyber KEM
    KyberKem, KyberParameterSet,
    KyberPublicKey, KyberSecretKey, KyberCiphertext, KyberSharedSecret,
    
    // Dilithium signatures
    Dilithium, DilithiumParameterSet,
    DilithiumPublicKey, DilithiumSecretKey, DilithiumSignature,
    
    // Error handling
    LibOqsError, LibOqsResult,
    
    // Utility functions
    supported_algorithms, algorithms_by_security_level, algorithms_by_family,
    is_algorithm_supported, get_algorithm_info,
};

// Version and metadata
pub const CRYPTO_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CRYPTO_DESCRIPTION: &str = "Polymera OS Cryptography Module";

/// Cryptographic algorithm categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoCategory {
    /// Key Encapsulation Mechanism
    Kem,
    /// Digital Signature
    Signature,
    /// Hash Function
    Hash,
    /// Symmetric Encryption
    Symmetric,
    /// Random Number Generation
    Random,
}

impl CryptoCategory {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CryptoCategory::Kem => "Key Encapsulation Mechanism",
            CryptoCategory::Signature => "Digital Signature",
            CryptoCategory::Hash => "Hash Function",
            CryptoCategory::Symmetric => "Symmetric Encryption",
            CryptoCategory::Random => "Random Number Generation",
        }
    }

    /// Get the abbreviation
    pub fn abbreviation(&self) -> &'static str {
        match self {
            CryptoCategory::Kem => "KEM",
            CryptoCategory::Signature => "SIG",
            CryptoCategory::Hash => "HASH",
            CryptoCategory::Symmetric => "SYM",
            CryptoCategory::Random => "RNG",
        }
    }
}

/// Security level categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    /// 128-bit security (Level 1)
    Level1 = 128,
    /// 192-bit security (Level 3)
    Level3 = 192,
    /// 256-bit security (Level 5)
    Level5 = 256,
}

impl SecurityLevel {
    /// Get the bit strength
    pub fn bits(&self) -> u32 {
        *self as u32
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            SecurityLevel::Level1 => "128-bit security (Level 1)",
            SecurityLevel::Level3 => "192-bit security (Level 3)",
            SecurityLevel::Level5 => "256-bit security (Level 5)",
        }
    }

    /// Check if this level meets minimum requirements
    pub fn meets_requirement(&self, minimum: SecurityLevel) -> bool {
        self >= &minimum
    }
}

/// Cryptographic context for managing algorithms and keys
pub struct CryptoContext {
    /// Available algorithms
    algorithms: Vec<Algorithm>,
    /// Default security level
    default_security_level: SecurityLevel,
    /// Enable post-quantum algorithms
    enable_pqc: bool,
}

impl CryptoContext {
    /// Create a new crypto context
    pub fn new() -> Self {
        Self {
            algorithms: supported_algorithms(),
            default_security_level: SecurityLevel::Level1,
            enable_pqc: true,
        }
    }

    /// Create a context with specific security level
    pub fn with_security_level(level: SecurityLevel) -> Self {
        Self {
            algorithms: algorithms_by_security_level(level.bits()),
            default_security_level: level,
            enable_pqc: true,
        }
    }

    /// Get available algorithms
    pub fn algorithms(&self) -> &[Algorithm] {
        &self.algorithms
    }

    /// Get algorithms by category
    pub fn algorithms_by_category(&self, category: CryptoCategory) -> Vec<Algorithm> {
        self.algorithms
            .iter()
            .filter(|alg| match category {
                CryptoCategory::Kem => alg.is_kem(),
                CryptoCategory::Signature => alg.is_signature(),
                _ => false, // Other categories not yet implemented
            })
            .cloned()
            .collect()
    }

    /// Get algorithms by security level
    pub fn algorithms_by_security_level(&self, level: SecurityLevel) -> Vec<Algorithm> {
        self.algorithms
            .iter()
            .filter(|alg| alg.security_level() == level.bits())
            .cloned()
            .collect()
    }

    /// Check if an algorithm is available
    pub fn has_algorithm(&self, algorithm: &Algorithm) -> bool {
        self.algorithms.contains(algorithm)
    }

    /// Get the default security level
    pub fn default_security_level(&self) -> SecurityLevel {
        self.default_security_level
    }

    /// Set the default security level
    pub fn set_default_security_level(&mut self, level: SecurityLevel) {
        self.default_security_level = level;
        self.algorithms = algorithms_by_security_level(level.bits());
    }

    /// Check if post-quantum algorithms are enabled
    pub fn pqc_enabled(&self) -> bool {
        self.enable_pqc
    }

    /// Enable or disable post-quantum algorithms
    pub fn set_pqc_enabled(&mut self, enabled: bool) {
        self.enable_pqc = enabled;
        if enabled {
            self.algorithms = supported_algorithms();
        } else {
            // Filter out PQC algorithms (currently all are PQC)
            self.algorithms.clear();
        }
    }
}

impl Default for CryptoContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Cryptographic utilities and helper functions
pub mod utils {
    use super::*;

    /// Get algorithm recommendations for a given security level
    pub fn get_recommendations(security_level: SecurityLevel) -> Vec<Algorithm> {
        algorithms_by_security_level(security_level.bits())
    }

    /// Check if algorithms provide forward secrecy
    pub fn provides_forward_secrecy(_algorithm: &Algorithm) -> bool {
        // All current algorithms provide forward secrecy
        // Kyber: Each encapsulation generates a new shared secret
        // Dilithium: Each signature is independent
        true
    }

    /// Get performance characteristics for an algorithm
    pub fn get_performance_info(algorithm: &Algorithm) -> PerformanceInfo {
        match algorithm {
            Algorithm::Kyber(params) => PerformanceInfo {
                key_generation: match params {
                    KyberParameterSet::Kyber512 => "~50 μs",
                    KyberParameterSet::Kyber768 => "~75 μs",
                    KyberParameterSet::Kyber1024 => "~100 μs",
                },
                operation: match params {
                    KyberParameterSet::Kyber512 => "~40-45 μs",
                    KyberParameterSet::Kyber768 => "~60-65 μs",
                    KyberParameterSet::Kyber1024 => "~80-85 μs",
                },
                memory_usage: "Low",
                cpu_intensive: false,
            },
            Algorithm::Dilithium(params) => PerformanceInfo {
                key_generation: match params {
                    DilithiumParameterSet::Dilithium2 => "~100 μs",
                    DilithiumParameterSet::Dilithium3 => "~150 μs",
                    DilithiumParameterSet::Dilithium5 => "~200 μs",
                },
                operation: match params {
                    DilithiumParameterSet::Dilithium2 => "~80-60 μs",
                    DilithiumParameterSet::Dilithium3 => "~120-90 μs",
                    DilithiumParameterSet::Dilithium5 => "~160-120 μs",
                },
                memory_usage: "Medium",
                cpu_intensive: true,
            },
        }
    }
}

/// Performance information for cryptographic algorithms
#[derive(Debug, Clone)]
pub struct PerformanceInfo {
    pub key_generation: &'static str,
    pub operation: &'static str,
    pub memory_usage: &'static str,
    pub cpu_intensive: bool,
}

impl PerformanceInfo {
    /// Get a formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Key Gen: {}, Operation: {}, Memory: {}, CPU: {}",
            self.key_generation,
            self.operation,
            self.memory_usage,
            if self.cpu_intensive { "High" } else { "Low" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_category() {
        let kem = CryptoCategory::Kem;
        assert_eq!(kem.description(), "Key Encapsulation Mechanism");
        assert_eq!(kem.abbreviation(), "KEM");

        let sig = CryptoCategory::Signature;
        assert_eq!(sig.description(), "Digital Signature");
        assert_eq!(sig.abbreviation(), "SIG");
    }

    #[test]
    fn test_security_level() {
        let level1 = SecurityLevel::Level1;
        assert_eq!(level1.bits(), 128);
        assert_eq!(level1.description(), "128-bit security (Level 1)");
        assert!(level1.meets_requirement(SecurityLevel::Level1));
        assert!(!level1.meets_requirement(SecurityLevel::Level3));

        let level5 = SecurityLevel::Level5;
        assert_eq!(level5.bits(), 256);
        assert!(level5.meets_requirement(SecurityLevel::Level1));
        assert!(level5.meets_requirement(SecurityLevel::Level3));
    }

    #[test]
    fn test_crypto_context() {
        let mut context = CryptoContext::new();
        assert!(!context.algorithms().is_empty());
        assert_eq!(context.default_security_level(), SecurityLevel::Level1);
        assert!(context.pqc_enabled());

        context.set_default_security_level(SecurityLevel::Level3);
        assert_eq!(context.default_security_level(), SecurityLevel::Level3);
        assert_eq!(context.algorithms().len(), 2); // Kyber768 + Dilithium3

        let kem_algorithms = context.algorithms_by_category(CryptoCategory::Kem);
        assert_eq!(kem_algorithms.len(), 1); // Only Kyber768 at Level 3
        assert!(kem_algorithms.iter().all(|alg| alg.is_kem()));
    }

    #[test]
    fn test_performance_info() {
        let kyber = Algorithm::Kyber(KyberParameterSet::Kyber512);
        let info = utils::get_performance_info(&kyber);
        assert!(info.key_generation.contains("μs"));
        assert!(!info.cpu_intensive);

        let dilithium = Algorithm::Dilithium(DilithiumParameterSet::Dilithium2);
        let info = utils::get_performance_info(&dilithium);
        assert!(info.key_generation.contains("μs"));
        assert!(info.cpu_intensive);
    }

    #[test]
    fn test_forward_secrecy() {
        let kyber = Algorithm::Kyber(KyberParameterSet::Kyber512);
        assert!(utils::provides_forward_secrecy(&kyber));

        let dilithium = Algorithm::Dilithium(DilithiumParameterSet::Dilithium2);
        assert!(utils::provides_forward_secrecy(&dilithium));
    }
}
