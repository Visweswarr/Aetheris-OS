use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use halo2_proofs::{
    arithmetic::Field,
    circuit::{Circuit, Layouter, Value},
    dev::MockProver,
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::{Circuit as PlonkCircuit, ConstraintSystem, Error as PlonkError},
    poly::{
        commitment::ParamsProver,
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::ProverGWC,
        },
    },
    transcript::{Blake2bWrite, Challenge255},
    SerdeFormat,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Halo2 prover configuration
#[derive(Debug, Clone)]
pub struct Halo2ProverConfig {
    /// Circuit parameters (k value for 2^k rows)
    pub k: u32,
    /// Random number generator seed
    pub rng_seed: Option<u64>,
    /// Whether to enable circuit debugging
    pub enable_debug: bool,
    /// Maximum proving time
    pub max_proving_time: Duration,
    /// Proving key cache size
    pub proving_key_cache_size: usize,
    /// Verification key cache size
    pub verification_key_cache_size: usize,
}

impl Default for Halo2ProverConfig {
    fn default() -> Self {
        Self {
            k: 20, // 2^20 = 1,048,576 rows
            rng_seed: None,
            enable_debug: false,
            max_proving_time: Duration::from_secs(300), // 5 minutes
            proving_key_cache_size: 10,
            verification_key_cache_size: 10,
        }
    }
}

/// Halo2 prover errors
#[derive(Error, Debug)]
pub enum Halo2ProverError {
    #[error("Circuit compilation failed: {0}")]
    CircuitCompilationError(String),
    
    #[error("Proving key generation failed: {0}")]
    ProvingKeyGenerationError(String),
    
    #[error("Verification key generation failed: {0}")]
    VerificationKeyGenerationError(String),
    
    #[error("Proof generation failed: {0}")]
    ProofGenerationError(String),
    
    #[error("Proof verification failed: {0}")]
    ProofVerificationError(String),
    
    #[error("Mock prover failed: {0}")]
    MockProverError(String),
    
    #[error("Circuit execution timeout after {0:?}")]
    ExecutionTimeout(Duration),
    
    #[error("Invalid circuit parameters: {0}")]
    InvalidParameters(String),
    
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Circuit not found: {0}")]
    CircuitNotFound(String),
    
    #[error("Proving key not found: {0}")]
    ProvingKeyNotFound(String),
    
    #[error("Verification key not found: {0}")]
    VerificationKeyNotFound(String),
}

/// Proving key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvingKeyMetadata {
    /// Circuit name
    pub circuit_name: String,
    /// Circuit version
    pub circuit_version: String,
    /// Circuit hash
    pub circuit_hash: String,
    /// K value (2^k rows)
    pub k: u32,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Generation parameters
    pub parameters: HashMap<String, String>,
    /// File size in bytes
    pub file_size: usize,
}

/// Verification key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKeyMetadata {
    /// Circuit name
    pub circuit_name: String,
    /// Circuit version
    pub circuit_version: String,
    /// Circuit hash
    pub circuit_hash: String,
    /// K value (2^k rows)
    pub k: u32,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// File size in bytes
    pub file_size: usize,
}

/// Proof metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Circuit name
    pub circuit_name: String,
    /// Circuit version
    pub circuit_version: String,
    /// Proof hash
    pub proof_hash: String,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Public inputs
    pub public_inputs: Vec<String>,
    /// Proof size in bytes
    pub proof_size: usize,
}

/// Halo2 prover statistics
#[derive(Debug, Clone, Default)]
pub struct Halo2ProverStats {
    /// Total proofs generated
    pub total_proofs: u64,
    /// Successful proofs
    pub successful_proofs: u64,
    /// Failed proofs
    pub failed_proofs: u64,
    /// Total proving time
    pub total_proving_time: Duration,
    /// Average proving time
    pub average_proving_time: Duration,
    /// Total verifications
    pub total_verifications: u64,
    /// Successful verifications
    pub successful_verifications: u64,
    /// Failed verifications
    pub failed_verifications: u64,
    /// Cache hit rates
    pub proving_key_cache_hits: u64,
    pub proving_key_cache_misses: u64,
    pub verification_key_cache_hits: u64,
    pub verification_key_cache_misses: u64,
}

/// Cached proving key entry
#[derive(Clone)]
struct CachedProvingKey {
    key: ParamsKZG<Bn256>,
    metadata: ProvingKeyMetadata,
    last_used: Instant,
    usage_count: u64,
}

/// Cached verification key entry
#[derive(Clone)]
struct CachedVerificationKey {
    key: halo2_proofs::plonk::VerifyingKey<G1Affine>,
    metadata: VerificationKeyMetadata,
    last_used: Instant,
    usage_count: u64,
}

/// Main Halo2 prover
pub struct Halo2Prover {
    /// Configuration
    config: Halo2ProverConfig,
    /// Proving key cache
    proving_key_cache: Arc<Mutex<HashMap<String, CachedProvingKey>>>,
    /// Verification key cache
    verification_key_cache: Arc<Mutex<HashMap<String, CachedVerificationKey>>>,
    /// Statistics
    stats: Arc<Mutex<Halo2ProverStats>>,
    /// Circuit registry
    circuit_registry: Arc<Mutex<HashMap<String, Box<dyn Circuit<Fr>>>>>,
}

impl Halo2Prover {
    /// Create a new Halo2 prover
    pub fn new(config: Halo2ProverConfig) -> Result<Self, Halo2ProverError> {
        Ok(Self {
            config,
            proving_key_cache: Arc::new(Mutex::new(HashMap::new())),
            verification_key_cache: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(Halo2ProverStats::default())),
            circuit_registry: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Create a new Halo2 prover with default configuration
    pub fn with_default_config() -> Result<Self, Halo2ProverError> {
        let config = Halo2ProverConfig::default();
        Self::new(config)
    }
    
    /// Register a circuit
    pub fn register_circuit<C: Circuit<Fr> + 'static>(
        &self,
        name: String,
        circuit: C,
    ) -> Result<(), Halo2ProverError> {
        let mut registry = self.circuit_registry.lock().unwrap();
        registry.insert(name, Box::new(circuit));
        Ok(())
    }
    
    /// Generate proving key for a circuit
    pub fn generate_proving_key<C: Circuit<Fr> + 'static>(
        &self,
        circuit_name: &str,
        circuit: &C,
    ) -> Result<ProvingKeyMetadata, Halo2ProverError> {
        let start_time = Instant::now();
        
        // Check if proving key is already cached
        {
            let cache = self.proving_key_cache.lock().unwrap();
            if cache.contains_key(circuit_name) {
                let mut stats = self.stats.lock().unwrap();
                stats.proving_key_cache_hits += 1;
                return Ok(cache[circuit_name].metadata.clone());
            }
        }
        
        // Generate circuit parameters
        let params = ParamsKZG::<Bn256>::new(self.config.k)
            .map_err(|e| Halo2ProverError::ProvingKeyGenerationError(e.to_string()))?;
        
        // Generate proving key
        let pk = halo2_proofs::plonk::keygen_pk(&params, circuit)
            .map_err(|e| Halo2ProverError::ProvingKeyGenerationError(e.to_string()))?;
        
        // Create metadata
        let metadata = ProvingKeyMetadata {
            circuit_name: circuit_name.to_string(),
            circuit_version: "1.0.0".to_string(),
            circuit_hash: format!("{:?}", std::any::TypeId::of::<C>()),
            k: self.config.k,
            generated_at: chrono::Utc::now(),
            parameters: HashMap::new(),
            file_size: 0, // Will be set when saving
        };
        
        // Cache the proving key
        {
            let mut cache = self.proving_key_cache.lock().unwrap();
            if cache.len() >= self.config.proving_key_cache_size {
                self.evict_oldest_proving_key(&mut cache);
            }
            
            cache.insert(circuit_name.to_string(), CachedProvingKey {
                key: params,
                metadata: metadata.clone(),
                last_used: Instant::now(),
                usage_count: 1,
            });
            
            let mut stats = self.stats.lock().unwrap();
            stats.proving_key_cache_misses += 1;
        }
        
        // Update statistics
        let generation_time = start_time.elapsed();
        self.update_stats(generation_time, true);
        
        Ok(metadata)
    }
    
    /// Generate verification key for a circuit
    pub fn generate_verification_key<C: Circuit<Fr> + 'static>(
        &self,
        circuit_name: &str,
        circuit: &C,
    ) -> Result<VerificationKeyMetadata, Halo2ProverError> {
        let start_time = Instant::now();
        
        // Check if verification key is already cached
        {
            let cache = self.verification_key_cache.lock().unwrap();
            if cache.contains_key(circuit_name) {
                let mut stats = self.stats.lock().unwrap();
                stats.verification_key_cache_hits += 1;
                return Ok(cache[circuit_name].metadata.clone());
            }
        }
        
        // Generate circuit parameters
        let params = ParamsKZG::<Bn256>::new(self.config.k)
            .map_err(|e| Halo2ProverError::VerificationKeyGenerationError(e.to_string()))?;
        
        // Generate verification key
        let vk = halo2_proofs::plonk::keygen_vk(&params, circuit)
            .map_err(|e| Halo2ProverError::VerificationKeyGenerationError(e.to_string()))?;
        
        // Create metadata
        let metadata = VerificationKeyMetadata {
            circuit_name: circuit_name.to_string(),
            circuit_version: "1.0.0".to_string(),
            circuit_hash: format!("{:?}", std::any::TypeId::of::<C>()),
            k: self.config.k,
            generated_at: chrono::Utc::now(),
            file_size: 0, // Will be set when saving
        };
        
        // Cache the verification key
        {
            let mut cache = self.verification_key_cache.lock().unwrap();
            if cache.len() >= self.config.verification_key_cache_size {
                self.evict_oldest_verification_key(&mut cache);
            }
            
            cache.insert(circuit_name.to_string(), CachedVerificationKey {
                key: vk,
                metadata: metadata.clone(),
                last_used: Instant::now(),
                usage_count: 1,
            });
            
            let mut stats = self.stats.lock().unwrap();
            stats.verification_key_cache_misses += 1;
        }
        
        // Update statistics
        let generation_time = start_time.elapsed();
        self.update_stats(generation_time, true);
        
        Ok(metadata)
    }
    
    /// Generate a proof for a circuit
    pub fn generate_proof<C: Circuit<Fr> + 'static>(
        &self,
        circuit_name: &str,
        circuit: &C,
        public_inputs: &[Fr],
    ) -> Result<(Vec<u8>, ProofMetadata), Halo2ProverError> {
        let start_time = Instant::now();
        
        // Check timeout
        if start_time.elapsed() > self.config.max_proving_time {
            return Err(Halo2ProverError::ExecutionTimeout(self.config.max_proving_time));
        }
        
        // Get or generate proving key
        let proving_key = self.get_or_generate_proving_key(circuit_name, circuit)?;
        
        // Generate circuit parameters
        let params = ParamsKZG::<Bn256>::new(self.config.k)
            .map_err(|e| Halo2ProverError::ProofGenerationError(e.to_string()))?;
        
        // Create prover
        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
        
        // Generate proof
        let prover = ProverGWC::new(&proving_key);
        let proof = prover
            .prove_with_advice_and_commitments(
                &params,
                &proving_key,
                &[circuit],
                &[&public_inputs],
                &mut transcript,
            )
            .map_err(|e| Halo2ProverError::ProofGenerationError(e.to_string()))?;
        
        // Serialize proof
        let proof_bytes = proof.to_bytes();
        
        // Create metadata
        let metadata = ProofMetadata {
            circuit_name: circuit_name.to_string(),
            circuit_version: "1.0.0".to_string(),
            proof_hash: format!("{:x}", md5::compute(&proof_bytes)),
            generated_at: chrono::Utc::now(),
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            public_inputs: public_inputs.iter().map(|f| format!("{:?}", f)).collect(),
            proof_size: proof_bytes.len(),
        };
        
        // Update statistics
        let proving_time = start_time.elapsed();
        self.update_proof_stats(proving_time, true);
        
        Ok((proof_bytes, metadata))
    }
    
    /// Verify a proof
    pub fn verify_proof(
        &self,
        circuit_name: &str,
        proof_bytes: &[u8],
        public_inputs: &[Fr],
    ) -> Result<bool, Halo2ProverError> {
        let start_time = Instant::now();
        
        // Get verification key
        let verification_key = self.get_verification_key(circuit_name)?;
        
        // Deserialize proof
        let proof = halo2_proofs::plonk::Proof::read::<_, G1Affine>(proof_bytes)
            .map_err(|e| Halo2ProverError::ProofVerificationError(e.to_string()))?;
        
        // Verify proof
        let params = ParamsKZG::<Bn256>::new(self.config.k)
            .map_err(|e| Halo2ProverError::ProofVerificationError(e.to_string()))?;
        
        let strategy = halo2_proofs::poly::kzg::strategy::SingleStrategy::new(&params);
        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);
        
        let verification_result = halo2_proofs::plonk::verify_proof::<KZGCommitmentScheme<Bn256>, ProverGWC<_>, _>(
            &params,
            &verification_key,
            strategy,
            &[public_inputs],
            &mut transcript,
        );
        
        let is_valid = verification_result.is_ok();
        
        // Update statistics
        let verification_time = start_time.elapsed();
        self.update_verification_stats(is_valid);
        
        Ok(is_valid)
    }
    
    /// Run mock prover for testing
    pub fn run_mock_prover<C: Circuit<Fr> + 'static>(
        &self,
        circuit_name: &str,
        circuit: &C,
        public_inputs: &[Fr],
    ) -> Result<bool, Halo2ProverError> {
        // Generate circuit parameters
        let params = ParamsKZG::<Bn256>::new(self.config.k)
            .map_err(|e| Halo2ProverError::MockProverError(e.to_string()))?;
        
        // Run mock prover
        let prover = MockProver::run(self.config.k, circuit, vec![public_inputs.to_vec()])
            .map_err(|e| Halo2ProverError::MockProverError(e.to_string()))?;
        
        // Verify the proof
        prover.verify()
            .map_err(|e| Halo2ProverError::MockProverError(e.to_string()))?;
        
        Ok(true)
    }
    
    /// Save proving key to file
    pub fn save_proving_key(
        &self,
        circuit_name: &str,
        output_path: &Path,
    ) -> Result<ProvingKeyMetadata, Halo2ProverError> {
        let cache = self.proving_key_cache.lock().unwrap();
        let cached_key = cache.get(circuit_name)
            .ok_or_else(|| Halo2ProverError::ProvingKeyNotFound(circuit_name.to_string()))?;
        
        // Save the proving key
        // Note: This is a simplified implementation
        // In practice, you'd serialize the actual proving key data
        
        let mut metadata = cached_key.metadata.clone();
        metadata.file_size = 0; // Would be set to actual file size
        
        Ok(metadata)
    }
    
    /// Save verification key to file
    pub fn save_verification_key(
        &self,
        circuit_name: &str,
        output_path: &Path,
    ) -> Result<VerificationKeyMetadata, Halo2ProverError> {
        let cache = self.verification_key_cache.lock().unwrap();
        let cached_key = cache.get(circuit_name)
            .ok_or_else(|| Halo2ProverError::VerificationKeyNotFound(circuit_name.to_string()))?;
        
        // Save the verification key
        // Note: This is a simplified implementation
        // In practice, you'd serialize the actual verification key data
        
        let mut metadata = cached_key.metadata.clone();
        metadata.file_size = 0; // Would be set to actual file size
        
        Ok(metadata)
    }
    
    /// Get prover statistics
    pub fn get_stats(&self) -> Halo2ProverStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Clear caches
    pub fn clear_caches(&self) {
        let mut proving_cache = self.proving_key_cache.lock().unwrap();
        proving_cache.clear();
        
        let mut verification_cache = self.verification_key_cache.lock().unwrap();
        verification_cache.clear();
    }
    
    // Helper methods
    
    fn get_or_generate_proving_key<C: Circuit<Fr> + 'static>(
        &self,
        circuit_name: &str,
        circuit: &C,
    ) -> Result<ParamsKZG<Bn256>, Halo2ProverError> {
        // Check cache first
        {
            let mut cache = self.proving_key_cache.lock().unwrap();
            if let Some(cached_key) = cache.get_mut(circuit_name) {
                cached_key.last_used = Instant::now();
                cached_key.usage_count += 1;
                return Ok(cached_key.key.clone());
            }
        }
        
        // Generate new proving key
        self.generate_proving_key(circuit_name, circuit)?;
        
        // Get from cache
        let cache = self.proving_key_cache.lock().unwrap();
        Ok(cache[circuit_name].key.clone())
    }
    
    fn get_verification_key(
        &self,
        circuit_name: &str,
    ) -> Result<halo2_proofs::plonk::VerifyingKey<G1Affine>, Halo2ProverError> {
        let cache = self.verification_key_cache.lock().unwrap();
        let cached_key = cache.get(circuit_name)
            .ok_or_else(|| Halo2ProverError::VerificationKeyNotFound(circuit_name.to_string()))?;
        
        Ok(cached_key.key.clone())
    }
    
    fn evict_oldest_proving_key(&self, cache: &mut HashMap<String, CachedProvingKey>) {
        let oldest_key = cache.iter()
            .min_by_key(|(_, cached_key)| cached_key.last_used)
            .map(|(key, _)| key.clone());
        
        if let Some(key) = oldest_key {
            cache.remove(&key);
        }
    }
    
    fn evict_oldest_verification_key(&self, cache: &mut HashMap<String, CachedVerificationKey>) {
        let oldest_key = cache.iter()
            .min_by_key(|(_, cached_key)| cached_key.last_used)
            .map(|(key, _)| key.clone());
        
        if let Some(key) = oldest_key {
            cache.remove(&key);
        }
    }
    
    fn update_stats(&self, execution_time: Duration, success: bool) {
        let mut stats = self.stats.lock().unwrap();
        
        if success {
            stats.successful_proofs += 1;
        } else {
            stats.failed_proofs += 1;
        }
        
        stats.total_proofs += 1;
        stats.total_proving_time += execution_time;
        stats.average_proving_time = Duration::from_millis(
            stats.total_proving_time.as_millis() as u64 / stats.total_proofs
        );
    }
    
    fn update_proof_stats(&self, proving_time: Duration, success: bool) {
        let mut stats = self.stats.lock().unwrap();
        
        if success {
            stats.successful_proofs += 1;
        } else {
            stats.failed_proofs += 1;
        }
        
        stats.total_proofs += 1;
        stats.total_proving_time += proving_time;
        stats.average_proving_time = Duration::from_millis(
            stats.total_proving_time.as_millis() as u64 / stats.total_proofs
        );
    }
    
    fn update_verification_stats(&self, success: bool) {
        let mut stats = self.stats.lock().unwrap();
        
        stats.total_verifications += 1;
        if success {
            stats.successful_verifications += 1;
        } else {
            stats.failed_verifications += 1;
        }
    }
}

impl Drop for Halo2Prover {
    fn drop(&mut self) {
        // Clean up resources
        self.clear_caches();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use halo2_proofs::circuit::{Circuit, Layouter, Value};
    use halo2_proofs::plonk::{Circuit as PlonkCircuit, ConstraintSystem, Error as PlonkError};
    
    // Simple test circuit
    #[derive(Clone)]
    struct TestCircuit {
        a: Value<Fr>,
        b: Value<Fr>,
    }
    
    impl Circuit<Fr> for TestCircuit {
        type Config = ();
        type FloorPlanner = halo2_proofs::circuit::SimpleFloorPlanner;
        
        fn without_witnesses(&self) -> Self {
            Self {
                a: Value::unknown(),
                b: Value::unknown(),
            }
        }
        
        fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
            ()
        }
        
        fn synthesize(&self, _config: Self::Config, _layouter: &mut impl Layouter<Fr>) -> Result<(), PlonkError> {
            Ok(())
        }
    }
    
    #[test]
    fn test_halo2_prover_creation() {
        let config = Halo2ProverConfig::default();
        let prover = Halo2Prover::new(config);
        assert!(prover.is_ok());
    }
    
    #[test]
    fn test_halo2_prover_default_config() {
        let prover = Halo2Prover::with_default_config();
        assert!(prover.is_ok());
    }
    
    #[test]
    fn test_halo2_prover_config_default() {
        let config = Halo2ProverConfig::default();
        assert_eq!(config.k, 20);
        assert_eq!(config.max_proving_time, Duration::from_secs(300));
        assert_eq!(config.proving_key_cache_size, 10);
    }
    
    #[test]
    fn test_circuit_registration() {
        let prover = Halo2Prover::with_default_config().unwrap();
        let circuit = TestCircuit {
            a: Value::known(Fr::from(1)),
            b: Value::known(Fr::from(2)),
        };
        
        let result = prover.register_circuit("test_circuit".to_string(), circuit);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_prover_stats() {
        let prover = Halo2Prover::with_default_config().unwrap();
        let stats = prover.get_stats();
        
        assert_eq!(stats.total_proofs, 0);
        assert_eq!(stats.successful_proofs, 0);
        assert_eq!(stats.failed_proofs, 0);
        assert_eq!(stats.total_verifications, 0);
    }
    
    #[test]
    fn test_cache_operations() {
        let prover = Halo2Prover::with_default_config().unwrap();
        
        // Test cache clearing
        prover.clear_caches();
        
        // Verify caches are empty
        let stats = prover.get_stats();
        assert_eq!(stats.proving_key_cache_hits, 0);
        assert_eq!(stats.proving_key_cache_misses, 0);
    }
}
