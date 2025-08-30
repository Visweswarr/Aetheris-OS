//! Post-Quantum Cryptography (PQC) Foundation
//! 
//! This module provides safe Rust wrappers for post-quantum cryptographic algorithms,
//! integrating with liboqs for CRYSTALS-Kyber (KEM) and Dilithium (SIG) implementations.
//! 
//! ## Features
//! 
//! - **Memory Safety**: All secret material is automatically zeroized on drop
//! - **No Heap Leaks**: Comprehensive memory leak detection and prevention
//! - **Deterministic Builds**: Pinned liboqs commits for reproducible builds
//! - **Performance Optimized**: Meets Phase 2 performance targets
//! - **NIST Compliant**: Uses NIST-approved parameter sets
//! 
//! ## Algorithms
//! 
//! ### CRYSTALS-Kyber (Key Encapsulation Mechanism)
//! - **Kyber512**: 128-bit security level
//! - **Kyber768**: 192-bit security level  
//! - **Kyber1024**: 256-bit security level
//! 
//! ### CRYSTALS-Dilithium (Digital Signatures)
//! - **Dilithium2**: 128-bit security level
//! - **Dilithium3**: 192-bit security level
//! - **Dilithium5**: 256-bit security level
//! 
//! ## Usage Examples
//! 
//! ```rust
//! use crypto::pqc::{Kyber, Dilithium, KyberParameterSet, DilithiumParameterSet};
//! 
//! // Kyber KEM operations
//! let params = KyberParameterSet::Kyber768;
//! let (pk, sk) = Kyber::keygen(params)?;
//! let (ct, ss1) = Kyber::encapsulate(&pk)?;
//! let ss2 = Kyber::decapsulate(&ct, &sk)?;
//! assert_eq!(ss1.as_bytes(), ss2.as_bytes());
//! 
//! // Dilithium signature operations
//! let params = DilithiumParameterSet::Dilithium2;
//! let (pk, sk) = Dilithium::keygen(params)?;
//! let signature = Dilithium::sign(b"Hello, PQC!", &sk)?;
//! let is_valid = Dilithium::verify(b"Hello, PQC!", &signature, &pk)?;
//! assert!(is_valid);
//! ```
//! 
//! ## Performance Targets
//! 
//! | Operation | Kyber768 | Dilithium2 |
//! |-----------|----------|------------|
//! | **Key Generation** | < 1.2ms p50 | < 1.0ms p50 |
//! | **Encapsulation** | < 2.5ms p50 | < 1.0ms p50 |
//! | **Decapsulation** | < 2.5ms p50 | < 0.8ms p50 |
//! | **Signing** | N/A | < 1.0ms p50 |
//! | **Verification** | N/A | < 0.8ms p50 |
//! 
//! ## Security Features
//! 
//! - **Automatic Zeroization**: All secret keys are zeroized on drop
//! - **Memory Isolation**: Keys are stored in separate memory regions
//! - **Bounds Checking**: All array accesses are bounds-checked
//! - **Type Safety**: Rust type system prevents misuse
//! - **Constant Time**: Critical operations are constant-time
//! - **Side-Channel Resistance**: Protected against timing attacks

pub mod mem;
pub mod kyber;
pub mod dilithium;

// Re-export main types for convenience
pub use kyber::{
    Kyber,
    KyberParameterSet,
    KyberPublicKey,
    KyberSecretKey,
    KyberCiphertext,
    KyberSharedSecret,
};

pub use dilithium::{
    Dilithium,
    DilithiumParameterSet,
    DilithiumPublicKey,
    DilithiumSecretKey,
    DilithiumSignature,
};

pub use mem::{
    MemoryConfig,
    SecureMemory,
    LeakDetector,
    get_memory_config,
    set_memory_config,
    set_leak_detection,
    leak_detection_enabled,
    get_leak_detector,
    secure_alloc,
    secure_dealloc,
    secure_realloc,
};

/// PQC error types
#[derive(Debug, thiserror::Error)]
pub enum PqcError {
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Cryptographic operation failed: {0}")]
    CryptoError(String),
    
    #[error("Memory allocation failed: {0}")]
    MemoryError(String),
    
    #[error("Parameter set mismatch: expected {expected}, got {actual}")]
    ParameterMismatch { expected: String, actual: String },
    
    #[error("Key validation failed: {0}")]
    KeyValidationError(String),
    
    #[error("Signature validation failed: {0}")]
    SignatureValidationError(String),
}

/// Result type for PQC operations
pub type PqcResult<T> = Result<T, PqcError>;

/// PQC algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcAlgorithm {
    /// Kyber KEM
    Kyber(KyberParameterSet),
    /// Dilithium signature
    Dilithium(DilithiumParameterSet),
}

impl PqcAlgorithm {
    /// Get the security level in bits
    pub fn security_level(&self) -> u32 {
        match self {
            PqcAlgorithm::Kyber(params) => params.security_level(),
            PqcAlgorithm::Dilithium(params) => params.security_level(),
        }
    }

    /// Get the algorithm name
    pub fn algorithm_name(&self) -> &'static str {
        match self {
            PqcAlgorithm::Kyber(params) => params.algorithm_name(),
            PqcAlgorithm::Dilithium(params) => params.algorithm_name(),
        }
    }

    /// Check if this is a KEM algorithm
    pub fn is_kem(&self) -> bool {
        matches!(self, PqcAlgorithm::Kyber(_))
    }

    /// Check if this is a signature algorithm
    pub fn is_signature(&self) -> bool {
        matches!(self, PqcAlgorithm::Dilithium(_))
    }
}

/// Unified PQC interface for algorithm-agnostic operations
pub struct PqcInterface;

impl PqcInterface {
    /// Generate a new keypair for the specified algorithm
    pub fn generate_keypair(algorithm: PqcAlgorithm) -> PqcResult<PqcKeypair> {
        match algorithm {
            PqcAlgorithm::Kyber(params) => {
                let (pk, sk) = Kyber::keygen(params)
                    .map_err(|e| PqcError::CryptoError(e.to_string()))?;
                Ok(PqcKeypair::Kyber { public: pk, secret: sk })
            }
            PqcAlgorithm::Dilithium(params) => {
                let (pk, sk) = Dilithium::keygen(params)
                    .map_err(|e| PqcError::CryptoError(e.to_string()))?;
                Ok(PqcKeypair::Dilithium { public: pk, secret: sk })
            }
        }
    }

    /// Get the security level for an algorithm
    pub fn get_security_level(algorithm: PqcAlgorithm) -> u32 {
        algorithm.security_level()
    }

    /// Get the algorithm name
    pub fn get_algorithm_name(algorithm: PqcAlgorithm) -> &'static str {
        algorithm.algorithm_name()
    }

    /// Check if an algorithm meets minimum security requirements
    pub fn meets_security_requirement(algorithm: PqcAlgorithm, min_bits: u32) -> bool {
        algorithm.security_level() >= min_bits
    }
}

/// Unified keypair type
pub enum PqcKeypair {
    /// Kyber keypair
    Kyber {
        public: KyberPublicKey,
        secret: KyberSecretKey,
    },
    /// Dilithium keypair
    Dilithium {
        public: DilithiumPublicKey,
        secret: DilithiumSecretKey,
    },
}

impl PqcKeypair {
    /// Get the algorithm type
    pub fn algorithm(&self) -> PqcAlgorithm {
        match self {
            PqcKeypair::Kyber { public, .. } => PqcAlgorithm::Kyber(public.parameter_set()),
            PqcKeypair::Dilithium { public, .. } => PqcAlgorithm::Dilithium(public.parameter_set()),
        }
    }

    /// Get the security level
    pub fn security_level(&self) -> u32 {
        self.algorithm().security_level()
    }

    /// Get the algorithm name
    pub fn algorithm_name(&self) -> &'static str {
        self.algorithm().algorithm_name()
    }

    /// Check if this is a KEM keypair
    pub fn is_kem(&self) -> bool {
        self.algorithm().is_kem()
    }

    /// Check if this is a signature keypair
    pub fn is_signature(&self) -> bool {
        self.algorithm().is_signature()
    }
}

/// PQC performance metrics
#[derive(Debug, Clone)]
pub struct PqcMetrics {
    /// Algorithm used
    pub algorithm: PqcAlgorithm,
    /// Operation type
    pub operation: String,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// PQC performance monitor
pub struct PqcPerformanceMonitor {
    metrics: std::sync::Mutex<Vec<PqcMetrics>>,
}

impl PqcPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Record a performance metric
    pub fn record_metric(&self, metric: PqcMetrics) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(metric);
        }
    }

    /// Get all recorded metrics
    pub fn get_metrics(&self) -> Vec<PqcMetrics> {
        self.metrics.lock().unwrap().clone()
    }

    /// Get metrics for a specific algorithm
    pub fn get_metrics_for_algorithm(&self, algorithm: PqcAlgorithm) -> Vec<PqcMetrics> {
        self.metrics.lock().unwrap()
            .iter()
            .filter(|m| m.algorithm == algorithm)
            .cloned()
            .collect()
    }

    /// Get performance statistics for an algorithm
    pub fn get_performance_stats(&self, algorithm: PqcAlgorithm) -> Option<PqcPerformanceStats> {
        let metrics = self.get_metrics_for_algorithm(algorithm);
        if metrics.is_empty() {
            return None;
        }

        let durations: Vec<f64> = metrics.iter()
            .filter(|m| m.success)
            .map(|m| m.duration_ms)
            .collect();

        if durations.is_empty() {
            return None;
        }

        let count = durations.len();
        let total = durations.iter().sum::<f64>();
        let mean = total / count as f64;
        let min = durations.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = durations.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // Calculate percentiles
        let mut sorted = durations.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p50 = sorted[sorted.len() / 2];
        let p95 = sorted[(sorted.len() * 95) / 100];
        let p99 = sorted[(sorted.len() * 99) / 100];

        Some(PqcPerformanceStats {
            algorithm,
            count,
            mean,
            min,
            max,
            p50,
            p95,
            p99,
        })
    }

    /// Clear all metrics
    pub fn clear_metrics(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }
}

impl Default for PqcPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// PQC performance statistics
#[derive(Debug, Clone)]
pub struct PqcPerformanceStats {
    /// Algorithm
    pub algorithm: PqcAlgorithm,
    /// Number of operations
    pub count: usize,
    /// Mean duration (ms)
    pub mean: f64,
    /// Minimum duration (ms)
    pub min: f64,
    /// Maximum duration (ms)
    pub max: f64,
    /// 50th percentile (ms)
    pub p50: f64,
    /// 95th percentile (ms)
    pub p95: f64,
    /// 99th percentile (ms)
    pub p99: f64,
}

/// Global performance monitor instance
static PERFORMANCE_MONITOR: once_cell::sync::Lazy<PqcPerformanceMonitor> = 
    once_cell::sync::Lazy::new(PqcPerformanceMonitor::new);

/// Get the global performance monitor
pub fn get_performance_monitor() -> &'static PqcPerformanceMonitor {
    &PERFORMANCE_MONITOR
}

/// Record a performance metric
pub fn record_performance_metric(metric: PqcMetrics) {
    PERFORMANCE_MONITOR.record_metric(metric);
}

/// Get performance statistics for an algorithm
pub fn get_performance_stats(algorithm: PqcAlgorithm) -> Option<PqcPerformanceStats> {
    PERFORMANCE_MONITOR.get_performance_stats(algorithm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pqc_algorithm_enum() {
        let kyber = PqcAlgorithm::Kyber(KyberParameterSet::Kyber768);
        let dilithium = PqcAlgorithm::Dilithium(DilithiumParameterSet::Dilithium2);

        assert_eq!(kyber.security_level(), 192);
        assert_eq!(dilithium.security_level(), 128);
        assert_eq!(kyber.algorithm_name(), "Kyber768");
        assert_eq!(dilithium.algorithm_name(), "Dilithium2");
        assert!(kyber.is_kem());
        assert!(dilithium.is_signature());
    }

    #[test]
    fn test_pqc_interface() {
        let algorithm = PqcAlgorithm::Kyber(KyberParameterSet::Kyber768);
        let keypair = PqcInterface::generate_keypair(algorithm).unwrap();

        assert!(keypair.is_kem());
        assert_eq!(keypair.security_level(), 192);
        assert_eq!(keypair.algorithm_name(), "Kyber768");
    }

    #[test]
    fn test_performance_monitor() {
        let monitor = PqcPerformanceMonitor::new();
        let algorithm = PqcAlgorithm::Kyber(KyberParameterSet::Kyber768);

        let metric = PqcMetrics {
            algorithm,
            operation: "keygen".to_string(),
            duration_ms: 1.5,
            success: true,
            error: None,
        };

        monitor.record_metric(metric);
        let stats = monitor.get_performance_stats(algorithm).unwrap();

        assert_eq!(stats.count, 1);
        assert_eq!(stats.mean, 1.5);
        assert_eq!(stats.p50, 1.5);
    }

    #[test]
    fn test_security_requirements() {
        let kyber_768 = PqcAlgorithm::Kyber(KyberParameterSet::Kyber768);
        let dilithium_2 = PqcAlgorithm::Dilithium(DilithiumParameterSet::Dilithium2);

        assert!(PqcInterface::meets_security_requirement(kyber_768, 128));
        assert!(PqcInterface::meets_security_requirement(kyber_768, 192));
        assert!(!PqcInterface::meets_security_requirement(kyber_768, 256));

        assert!(PqcInterface::meets_security_requirement(dilithium_2, 128));
        assert!(!PqcInterface::meets_security_requirement(dilithium_2, 192));
    }
}
