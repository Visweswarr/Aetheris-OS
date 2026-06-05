use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ZK proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProof {
    /// ZK algorithm used
    pub algorithm: ZKAlgorithm,
    /// Serialized proof data
    pub proof_data: Vec<u8>,
    /// Public inputs for verification
    pub public_inputs: Vec<Vec<u8>>,
    /// Hash of the circuit
    pub circuit_hash: Vec<u8>,
    /// Version of the prover
    pub prover_version: String,
    /// Size of proof in bytes
    pub proof_size: u64,
    /// Verification key for the circuit
    pub verification_key: Vec<u8>,
}

/// ZK algorithms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZKAlgorithm {
    Halo2,
    Noir,
    Plonk,
    Custom,
}

impl ZKAlgorithm {
    /// Get the string representation of the algorithm
    pub fn as_str(&self) -> &'static str {
        match self {
            ZKAlgorithm::Halo2 => "halo2",
            ZKAlgorithm::Noir => "noir",
            ZKAlgorithm::Plonk => "plonk",
            ZKAlgorithm::Custom => "custom",
        }
    }
    
    /// Create from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "halo2" => Some(ZKAlgorithm::Halo2),
            "noir" => Some(ZKAlgorithm::Noir),
            "plonk" => Some(ZKAlgorithm::Plonk),
            "custom" => Some(ZKAlgorithm::Custom),
            _ => None,
        }
    }
}

/// ZK proof generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofGenerationRequest {
    /// Input data for proof generation
    pub input: Vec<u8>,
    /// ZK algorithm to use
    pub algorithm: ZKAlgorithm,
    /// Circuit identifier
    pub circuit_id: String,
    /// Additional parameters
    pub parameters: HashMap<String, serde_cbor::Value>,
}

/// ZK proof verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerificationRequest {
    /// Proof data to verify
    pub proof: ZKProof,
    /// Public inputs for verification
    pub public_inputs: Vec<Vec<u8>>,
    /// Circuit identifier
    pub circuit_id: String,
}

/// ZK proof generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofGenerationResult {
    /// Whether generation succeeded
    pub success: bool,
    /// Generated proof
    pub proof: Option<ZKProof>,
    /// Error message if failed
    pub error: Option<String>,
    /// Generation time in milliseconds
    pub generation_time_ms: u64,
    /// Memory used during generation
    pub memory_used: u64,
}

/// ZK proof verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerificationResult {
    /// Whether verification succeeded
    pub success: bool,
    /// Verification time in milliseconds
    pub verification_time_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// ZKVM service for proof generation and verification
pub struct ZKVMService {
    /// Available circuits
    circuits: HashMap<String, CircuitInfo>,
    /// Active provers
    provers: HashMap<ZKAlgorithm, Box<dyn Prover>>,
    /// Active verifiers
    verifiers: HashMap<ZKAlgorithm, Box<dyn Verifier>>,
}

/// Circuit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitInfo {
    /// Circuit identifier
    pub id: String,
    /// Circuit name
    pub name: String,
    /// Circuit description
    pub description: Option<String>,
    /// ZK algorithm used
    pub algorithm: ZKAlgorithm,
    /// Circuit hash
    pub circuit_hash: Vec<u8>,
    /// Verification key
    pub verification_key: Vec<u8>,
    /// Circuit size (gates)
    pub circuit_size: u64,
    /// Maximum proof size
    pub max_proof_size: u64,
    /// Estimated proving time
    pub estimated_proving_time_ms: u64,
    /// Estimated verification time
    pub estimated_verification_time_ms: u64,
}

/// Prover trait for ZK proof generation
pub trait Prover: Send + Sync {
    /// Generate a proof
    fn prove(&self, request: &ProofGenerationRequest) -> Result<ZKProof, Box<dyn std::error::Error>>;
    
    /// Get algorithm supported by this prover
    fn algorithm(&self) -> ZKAlgorithm;
    
    /// Get prover version
    fn version(&self) -> String;
}

/// Verifier trait for ZK proof verification
pub trait Verifier: Send + Sync {
    /// Verify a proof
    fn verify(&self, request: &ProofVerificationRequest) -> Result<bool, Box<dyn std::error::Error>>;
    
    /// Get algorithm supported by this verifier
    fn algorithm(&self) -> ZKAlgorithm;
    
    /// Get verifier version
    fn version(&self) -> String;
}

impl ZKVMService {
    /// Create a new ZKVM service
    pub fn new() -> Self {
        Self {
            circuits: HashMap::new(),
            provers: HashMap::new(),
            verifiers: HashMap::new(),
        }
    }
    
    /// Register a circuit
    pub fn register_circuit(&mut self, circuit: CircuitInfo) {
        self.circuits.insert(circuit.id.clone(), circuit);
    }
    
    /// Register a prover
    pub fn register_prover(&mut self, prover: Box<dyn Prover>) {
        let algorithm = prover.algorithm();
        self.provers.insert(algorithm, prover);
    }
    
    /// Register a verifier
    pub fn register_verifier(&mut self, verifier: Box<dyn Verifier>) {
        let algorithm = verifier.algorithm();
        self.verifiers.insert(algorithm, verifier);
    }
    
    /// Generate a ZK proof
    pub async fn generate_proof(
        &self,
        request: ProofGenerationRequest,
    ) -> Result<ProofGenerationResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Check if circuit exists
        let circuit = self.circuits.get(&request.circuit_id)
            .ok_or("Circuit not found")?;
        
        // Check if algorithm matches
        if circuit.algorithm != request.algorithm {
            return Err("Algorithm mismatch between request and circuit".into());
        }
        
        // Get prover for algorithm
        let prover = self.provers.get(&request.algorithm)
            .ok_or("No prover available for algorithm")?;
        
        // Generate proof
        let result = prover.prove(&request);
        
        let generation_time = start_time.elapsed();
        let generation_time_ms = generation_time.as_millis() as u64;
        
        match result {
            Ok(proof) => {
                // Estimate memory usage (simplified)
                let memory_used = proof.proof_size + proof.public_inputs.iter().map(|v| v.len()).sum::<usize>() as u64;
                
                Ok(ProofGenerationResult {
                    success: true,
                    proof: Some(proof),
                    error: None,
                    generation_time_ms,
                    memory_used,
                })
            }
            Err(e) => Ok(ProofGenerationResult {
                success: false,
                proof: None,
                error: Some(e.to_string()),
                generation_time_ms,
                memory_used: 0,
            }),
        }
    }
    
    /// Verify a ZK proof
    pub async fn verify_proof(
        &self,
        request: ProofVerificationRequest,
    ) -> Result<ProofVerificationResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        // Check if circuit exists
        let circuit = self.circuits.get(&request.circuit_id)
            .ok_or("Circuit not found")?;
        
        // Check if algorithm matches
        if circuit.algorithm != request.proof.algorithm {
            return Err("Algorithm mismatch between proof and circuit".into());
        }
        
        // Get verifier for algorithm
        let verifier = self.verifiers.get(&request.proof.algorithm)
            .ok_or("No verifier available for algorithm")?;
        
        // Verify proof
        let result = verifier.verify(&request);
        
        let verification_time = start_time.elapsed();
        let verification_time_ms = verification_time.as_millis() as u64;
        
        match result {
            Ok(valid) => Ok(ProofVerificationResult {
                success: valid,
                verification_time_ms,
                error: None,
            }),
            Err(e) => Ok(ProofVerificationResult {
                success: false,
                verification_time_ms,
                error: Some(e.to_string()),
            }),
        }
    }
    
    /// List available circuits
    pub fn list_circuits(&self) -> Vec<&CircuitInfo> {
        self.circuits.values().collect()
    }
    
    /// Get circuit by ID
    pub fn get_circuit(&self, id: &str) -> Option<&CircuitInfo> {
        self.circuits.get(id)
    }
    
    /// List available algorithms
    pub fn list_algorithms(&self) -> Vec<ZKAlgorithm> {
        let mut algorithms = std::collections::HashSet::new();
        
        for prover in self.provers.keys() {
            algorithms.insert(prover.clone());
        }
        
        for verifier in self.verifiers.keys() {
            algorithms.insert(verifier.clone());
        }
        
        algorithms.into_iter().collect()
    }
    
    /// Get service statistics
    pub fn get_stats(&self) -> ZKVMStats {
        ZKVMStats {
            total_circuits: self.circuits.len() as u64,
            total_provers: self.provers.len() as u64,
            total_verifiers: self.verifiers.len() as u64,
            supported_algorithms: self.list_algorithms().len() as u64,
        }
    }
}

/// ZKVM service statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKVMStats {
    /// Total number of circuits
    pub total_circuits: u64,
    /// Total number of provers
    pub total_provers: u64,
    /// Total number of verifiers
    pub total_verifiers: u64,
    /// Number of supported algorithms
    pub supported_algorithms: u64,
}

/// Mock prover for testing
pub struct MockProver {
    algorithm: ZKAlgorithm,
    version: String,
}

impl MockProver {
    /// Create a new mock prover
    pub fn new(algorithm: ZKAlgorithm) -> Self {
        Self {
            algorithm,
            version: "1.0.0".to_string(),
        }
    }
}

impl Prover for MockProver {
    fn prove(&self, request: &ProofGenerationRequest) -> Result<ZKProof, Box<dyn std::error::Error>> {
        // Generate a mock proof
        let proof = ZKProof {
            algorithm: self.algorithm.clone(),
            proof_data: format!("mock_proof_{}", request.circuit_id).into_bytes(),
            public_inputs: vec![request.input.clone()],
            circuit_hash: b"mock_circuit_hash".to_vec(),
            prover_version: self.version.clone(),
            proof_size: 1024,
            verification_key: b"mock_verification_key".to_vec(),
        };
        
        Ok(proof)
    }
    
    fn algorithm(&self) -> ZKAlgorithm {
        self.algorithm.clone()
    }
    
    fn version(&self) -> String {
        self.version.clone()
    }
}

/// Mock verifier for testing
pub struct MockVerifier {
    algorithm: ZKAlgorithm,
    version: String,
}

impl MockVerifier {
    /// Create a new mock verifier
    pub fn new(algorithm: ZKAlgorithm) -> Self {
        Self {
            algorithm,
            version: "1.0.0".to_string(),
        }
    }
}

impl Verifier for MockVerifier {
    fn verify(&self, _request: &ProofVerificationRequest) -> Result<bool, Box<dyn std::error::Error>> {
        // Mock verification always succeeds
        Ok(true)
    }
    
    fn algorithm(&self) -> ZKAlgorithm {
        self.algorithm.clone()
    }
    
    fn version(&self) -> String {
        self.version.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_zkvm_service_creation() {
        let service = ZKVMService::new();
        let stats = service.get_stats();
        
        assert_eq!(stats.total_circuits, 0);
        assert_eq!(stats.total_provers, 0);
        assert_eq!(stats.total_verifiers, 0);
    }

    #[tokio::test]
    async fn test_circuit_registration() {
        let mut service = ZKVMService::new();
        
        let circuit = CircuitInfo {
            id: "test-circuit".to_string(),
            name: "Test Circuit".to_string(),
            description: Some("A test circuit".to_string()),
            algorithm: ZKAlgorithm::Halo2,
            circuit_hash: b"test_hash".to_vec(),
            verification_key: b"test_key".to_vec(),
            circuit_size: 1000,
            max_proof_size: 2048,
            estimated_proving_time_ms: 100,
            estimated_verification_time_ms: 10,
        };
        
        service.register_circuit(circuit);
        
        let stats = service.get_stats();
        assert_eq!(stats.total_circuits, 1);
        
        let retrieved = service.get_circuit("test-circuit");
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_prover_registration() {
        let mut service = ZKVMService::new();
        
        let prover = MockProver::new(ZKAlgorithm::Halo2);
        service.register_prover(Box::new(prover));
        
        let stats = service.get_stats();
        assert_eq!(stats.total_provers, 1);
        
        let algorithms = service.list_algorithms();
        assert_eq!(algorithms.len(), 1);
        assert!(matches!(algorithms[0], ZKAlgorithm::Halo2));
    }

    #[tokio::test]
    async fn test_proof_generation() {
        let mut service = ZKVMService::new();
        
        // Register circuit
        let circuit = CircuitInfo {
            id: "test-circuit".to_string(),
            name: "Test Circuit".to_string(),
            description: None,
            algorithm: ZKAlgorithm::Halo2,
            circuit_hash: b"test_hash".to_vec(),
            verification_key: b"test_key".to_vec(),
            circuit_size: 1000,
            max_proof_size: 2048,
            estimated_proving_time_ms: 100,
            estimated_verification_time_ms: 10,
        };
        service.register_circuit(circuit);
        
        // Register prover
        let prover = MockProver::new(ZKAlgorithm::Halo2);
        service.register_prover(Box::new(prover));
        
        // Generate proof
        let request = ProofGenerationRequest {
            input: b"test_input".to_vec(),
            algorithm: ZKAlgorithm::Halo2,
            circuit_id: "test-circuit".to_string(),
            parameters: HashMap::new(),
        };
        
        let result = service.generate_proof(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.proof.is_some());
        assert!(result.generation_time_ms > 0);
    }

    #[tokio::test]
    async fn test_proof_verification() {
        let mut service = ZKVMService::new();
        
        // Register circuit
        let circuit = CircuitInfo {
            id: "test-circuit".to_string(),
            name: "Test Circuit".to_string(),
            description: None,
            algorithm: ZKAlgorithm::Halo2,
            circuit_hash: b"test_hash".to_vec(),
            verification_key: b"test_key".to_vec(),
            circuit_size: 1000,
            max_proof_size: 2048,
            estimated_proving_time_ms: 100,
            estimated_verification_time_ms: 10,
        };
        service.register_circuit(circuit);
        
        // Register verifier
        let verifier = MockVerifier::new(ZKAlgorithm::Halo2);
        service.register_verifier(Box::new(verifier));
        
        // Create proof for verification
        let proof = ZKProof {
            algorithm: ZKAlgorithm::Halo2,
            proof_data: b"test_proof".to_vec(),
            public_inputs: vec![b"test_input".to_vec()],
            circuit_hash: b"test_hash".to_vec(),
            prover_version: "1.0.0".to_string(),
            proof_size: 1024,
            verification_key: b"test_key".to_vec(),
        };
        
        // Verify proof
        let request = ProofVerificationRequest {
            proof,
            public_inputs: vec![b"test_input".to_vec()],
            circuit_id: "test-circuit".to_string(),
        };
        
        let result = service.verify_proof(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.verification_time_ms > 0);
    }

    #[test]
    fn test_zk_algorithm_conversions() {
        assert_eq!(ZKAlgorithm::Halo2.as_str(), "halo2");
        assert_eq!(ZKAlgorithm::Noir.as_str(), "noir");
        assert_eq!(ZKAlgorithm::Plonk.as_str(), "plonk");
        assert_eq!(ZKAlgorithm::Custom.as_str(), "custom");
        
        assert_eq!(ZKAlgorithm::from_str("halo2"), Some(ZKAlgorithm::Halo2));
        assert_eq!(ZKAlgorithm::from_str("noir"), Some(ZKAlgorithm::Noir));
        assert_eq!(ZKAlgorithm::from_str("plonk"), Some(ZKAlgorithm::Plonk));
        assert_eq!(ZKAlgorithm::from_str("custom"), Some(ZKAlgorithm::Custom));
        assert_eq!(ZKAlgorithm::from_str("unknown"), None);
    }
}
