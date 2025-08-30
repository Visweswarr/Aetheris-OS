# NGFS Smart Contract Sandbox - Security Guide

## Overview

The NGFS Smart Contract Sandbox provides a deterministic, sandboxed, and privacy-preserving Web3 layer for smart contract execution within Polymera OS. This document outlines the security architecture, threat model, and best practices for the contract sandbox system.

## Security Architecture

### Sandbox Isolation

The contract sandbox implements multiple layers of isolation to prevent malicious contracts from compromising the host system:

```
┌─────────────────────────────────────────────────────────────┐
│                    Host OS                                  │
├─────────────────────────────────────────────────────────────┤
│  NGFS Contract Sandbox                                      │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  WASM Runtime (WASI)                                    │ │
│  │  - Memory sandboxing                                    │ │
│  │  - Instruction counting                                 │ │
│  │  - System call filtering                                │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Gas Metering Engine                                    │ │
│  │  - Instruction limits                                   │ │
│  │  - Memory bounds                                        │ │
│  │  - Storage operation limits                             │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  Access Control Layer                                   │ │
│  │  - CapToken verification                                │ │
│  │  - NGFS read-only access                                │ │
│  │  - Vault write permissions                              │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### WASM Runtime Security

The sandbox uses WebAssembly with WASI (WebAssembly System Interface) to provide:

- **Memory Isolation**: Each contract runs in its own memory space
- **Instruction Counting**: Deterministic gas metering
- **System Call Filtering**: Only allowed operations permitted
- **No Network Access**: Complete network isolation
- **No File System Access**: Controlled storage access only

### Gas Metering

Deterministic resource tracking prevents resource exhaustion attacks:

```rust
pub struct GasConfig {
    pub max_instructions: u64,      // 1M instructions default
    pub max_memory_mb: u64,         // 64MB default
    pub max_storage_ops: u64,       // 1000 operations default
    pub instruction_cost: u64,      // Cost per instruction
    pub memory_cost_per_mb: u64,    // Cost per MB allocated
    pub storage_op_cost: u64,       // Cost per storage operation
}
```

## Threat Model

### Attack Vectors

#### 1. Resource Exhaustion
- **Threat**: Malicious contracts consuming excessive CPU, memory, or storage
- **Mitigation**: Strict gas limits, memory bounds, and operation counting
- **Detection**: Gas meter monitoring and automatic termination

#### 2. Information Leakage
- **Threat**: Contracts accessing unauthorized data or leaking secrets
- **Mitigation**: NGFS read-only access, CapToken-based vault permissions
- **Detection**: Access logging and audit trails

#### 3. Determinism Violation
- **Threat**: Non-deterministic execution breaking consensus
- **Mitigation**: Virtual monotonic clock, no random sources, no network
- **Detection**: Multiple execution validation and variance analysis

#### 4. ZK Proof Attacks
- **Threat**: Malicious or invalid zero-knowledge proofs
- **Mitigation**: Cryptographic verification, circuit validation
- **Detection**: Proof verification and algorithm validation

#### 5. WASM Exploitation
- **Threat**: Malicious WASM bytecode exploiting runtime vulnerabilities
- **Mitigation**: WASM validation, sandbox isolation, no unsafe operations
- **Detection**: Bytecode analysis and runtime monitoring

### Security Boundaries

#### Contract Isolation
- Each contract runs in isolated memory space
- No shared state between contracts
- Separate gas meters per execution

#### Storage Access Control
- NGFS: Read-only access to content-addressed storage
- Vault: Write access only with valid CapTokens
- No direct file system access

#### Network Isolation
- Complete network access prohibition
- No external API calls
- No inter-process communication

## Security Features

### CapToken Integration

All contract operations require valid CapTokens with appropriate permissions:

```rust
pub struct CapTokenV2 {
    pub issuer: Did,
    pub subject: Did,
    pub capabilities: Vec<Capability>,
    pub expires_at: Option<DateTime<Utc>>,
    pub signature: Vec<u8>,
}

impl ContractSandbox {
    pub async fn run_contract(&self, wasm: &[u8], input: &[u8], 
                            gas_limit: u64, zk_mode: bool,
                            cap_token: &CapTokenV2) -> Result<ResultV1> {
        // Verify CapToken permissions
        self.verify_contract_permissions(cap_token, "execute")?;
        
        // Execute contract with verified permissions
        self.execute_wasm(wasm, input, gas_limit, zk_mode).await
    }
}
```

### Deterministic Execution

The sandbox ensures completely deterministic execution:

```rust
impl ContractSandbox {
    fn create_wasi_context(&self) -> WasiCtx {
        WasiCtxBuilder::new()
            .env("NGFS_DETERMINISTIC", "1")
            .env("NGFS_NO_NETWORK", "1")
            .env("NGFS_READONLY_STORAGE", "1")
            .build()
            .expect("Failed to create WASI context")
    }
    
    fn setup_virtual_clock(&self) -> VirtualClock {
        VirtualClock::new()
            .with_monotonic_time()
            .without_wall_clock()
            .without_random()
    }
}
```

### ZK Proof Security

Zero-knowledge proofs provide cryptographic guarantees:

```rust
pub struct ZKProof {
    pub algorithm: ZKAlgorithm,
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<Vec<u8>>,
    pub circuit_hash: Vec<u8>,
    pub prover_version: String,
    pub proof_size: u64,
    pub verification_key: Vec<u8>,
}

impl ZKProof {
    pub fn verify(&self, input: &[u8]) -> Result<bool> {
        match self.algorithm {
            ZKAlgorithm::Halo2 => self.verify_halo2(input),
            ZKAlgorithm::Noir => self.verify_noir(input),
            ZKAlgorithm::Plonk => self.verify_plonk(input),
            ZKAlgorithm::Custom => self.verify_custom(input),
        }
    }
}
```

### Audit Logging

Comprehensive logging of all contract operations:

```rust
#[derive(Serialize, Deserialize)]
pub struct ContractAuditLog {
    pub timestamp: DateTime<Utc>,
    pub contract_id: String,
    pub operation: String,
    pub gas_used: u64,
    pub memory_peak: u64,
    pub storage_accesses: Vec<StorageAccess>,
    pub events: Vec<ContractEvent>,
    pub zk_proof_generated: bool,
    pub execution_time: Duration,
    pub success: bool,
    pub error_details: Option<String>,
    pub cap_token_used: String,
    pub user_did: String,
}
```

## Best Practices

### Contract Development

#### 1. Gas Optimization
- Minimize instruction count
- Use efficient algorithms
- Avoid unnecessary memory allocation
- Batch storage operations

#### 2. Security Validation
- Validate all inputs
- Use bounded loops
- Implement proper error handling
- Avoid unsafe operations

#### 3. Deterministic Logic
- No random number generation
- No system clock access
- No external dependencies
- Pure functional operations

### Sandbox Configuration

#### 1. Gas Limits
```rust
// Conservative gas configuration for production
let production_config = GasConfig {
    max_instructions: 500_000,      // Reduced from 1M
    max_memory_mb: 32,              // Reduced from 64MB
    max_storage_ops: 500,           // Reduced from 1000
    instruction_cost: 2,            // Increased cost
    memory_cost_per_mb: 2000,       // Increased cost
    storage_op_cost: 20,            // Increased cost
};
```

#### 2. Capability Restrictions
```rust
// Minimal capabilities for untrusted contracts
let restricted_capabilities = vec![
    Capability::new("ngfs:read", vec!["public"]),
    Capability::new("vault:read", vec!["own"]),
];
```

#### 3. ZK Algorithm Selection
```rust
// Production ZK configuration
let zk_config = ZKConfig {
    allowed_algorithms: vec![ZKAlgorithm::Halo2, ZKAlgorithm::Plonk],
    max_proof_size: 1024 * 1024,   // 1MB limit
    require_verification: true,
    circuit_validation: true,
};
```

### Monitoring and Alerting

#### 1. Performance Monitoring
- Gas usage patterns
- Memory allocation trends
- Execution time statistics
- Storage access frequency

#### 2. Security Monitoring
- Failed CapToken verifications
- Gas limit violations
- Memory limit violations
- Unusual execution patterns

#### 3. Audit Trail Analysis
- Contract execution history
- User access patterns
- Resource consumption trends
- Security incident investigation

## Incident Response

### Security Breach Procedures

#### 1. Immediate Response
- Stop affected contract execution
- Isolate compromised sandbox instances
- Preserve audit logs and evidence
- Notify security team

#### 2. Investigation
- Analyze audit logs
- Review contract bytecode
- Examine execution traces
- Identify attack vectors

#### 3. Recovery
- Update security configurations
- Patch vulnerabilities
- Restore from secure backups
- Implement additional controls

#### 4. Post-Incident
- Document lessons learned
- Update security policies
- Enhance monitoring capabilities
- Conduct security training

### Vulnerability Disclosure

#### 1. Responsible Disclosure
- Coordinate with security researchers
- Provide timely patches
- Maintain transparency
- Credit responsible disclosure

#### 2. Security Advisories
- Clear vulnerability descriptions
- Impact assessment
- Mitigation steps
- Timeline for fixes

## Compliance and Standards

### Cryptographic Standards

- **Hash Functions**: Blake3-256 for content addressing
- **Encryption**: XChaCha20-Poly1305 for data encryption
- **Key Derivation**: HKDF for key generation
- **Digital Signatures**: Ed25519 for CapToken verification

### Security Standards

- **OWASP**: Web application security guidelines
- **NIST**: Cybersecurity framework compliance
- **ISO 27001**: Information security management
- **SOC 2**: Security and availability controls

### Privacy Standards

- **GDPR**: Data protection compliance
- **CCPA**: California privacy rights
- **Zero-Knowledge**: Privacy-preserving computation
- **Data Minimization**: Minimal data collection

## Future Security Enhancements

### Advanced Sandboxing

- **Hardware Isolation**: CPU virtualization features
- **Memory Encryption**: Encrypted memory regions
- **Side-Channel Protection**: Timing attack mitigation
- **Quantum Resistance**: Post-quantum cryptography

### Enhanced ZK Security

- **Circuit Verification**: Automated circuit validation
- **Proof Aggregation**: Efficient proof batching
- **Recursive Proofs**: Scalable verification
- **Custom Circuits**: Domain-specific optimization

### Threat Intelligence

- **ML-Based Detection**: Anomaly detection
- **Behavioral Analysis**: Pattern recognition
- **Threat Hunting**: Proactive security
- **Intelligence Sharing**: Community collaboration

## Conclusion

The NGFS Smart Contract Sandbox provides enterprise-grade security for Web3 contract execution within the OS layer. The multi-layered security architecture, comprehensive threat modeling, and extensive monitoring capabilities ensure that contracts can execute safely while maintaining the determinism and privacy guarantees required for production use.

By following the security best practices outlined in this guide and maintaining vigilance through continuous monitoring and incident response procedures, organizations can confidently deploy smart contracts in the NGFS environment while maintaining the highest security standards.

The integration of zero-knowledge proofs, capability-based access control, and deterministic execution creates a unique security model that combines the benefits of blockchain technology with the reliability and performance of traditional operating systems.
