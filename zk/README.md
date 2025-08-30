# Zero-Knowledge (ZK) Toolchain

A comprehensive toolchain for developing, testing, and deploying zero-knowledge proof circuits in Polymera OS.

## Overview

The ZK toolchain provides a complete framework for zero-knowledge proof development using:

- **Noir**: Domain-specific language for writing ZK circuits
- **Halo2**: Advanced proving system with PLONK protocol
- **Circuit Versioning**: Semantic versioning for circuit evolution
- **Proving Key Management**: Secure key generation and storage

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Noir DSL     │    │   Halo2 Prover  │    │   Circuit      │
│                 │    │                 │    │   Registry     │
│  - age.nr      │───▶│  - prover.rs    │───▶│  - Versioning  │
│  - residency.nr│    │  - Key Mgmt     │    │  - Metadata    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Directory Structure

```
zk/
├── noir/                    # Noir circuit definitions
│   ├── age/                # Age verification circuits
│   │   └── circuit.nr      # Age verification circuit
│   └── residency/          # Residency verification circuits
│       └── circuit.nr      # Residency verification circuit
├── halo2/                  # Halo2 prover implementation
│   └── prover.rs          # Main prover implementation
├── test_zk_toolchain.sh   # Test script for toolchain
└── README.md              # This file
```

## Quick Start

### 1. Install Dependencies

```bash
# Install Noir
npm install -g @noir-lang/noir

# Install Rust dependencies
cargo add halo2_proofs
cargo add halo2curves
```

### 2. Test the Toolchain

```bash
# Make test script executable
chmod +x test_zk_toolchain.sh

# Run tests
./test_zk_toolchain.sh
```

### 3. Compile Circuits

```bash
# Compile age verification circuit
cd noir/age
noir compile

# Compile residency verification circuit
cd ../residency
noir compile
```

## Circuit Examples

### Age Verification Circuit

The age verification circuit proves that a person is above a certain age without revealing their exact age:

```rust
#[circuit]
contract AgeVerification {
    pub age_threshold: Field;
    pub age_commitment: Field;
    
    age: Field;
    age_secret: Field;
    
    pub fn verify_age(
        self,
        age_threshold: Field,
        age_commitment: Field,
        age: Field,
        age_secret: Field,
    ) -> bool {
        let is_old_enough = age >= age_threshold;
        is_old_enough = true;
        
        let expected_commitment = pedersen::hash([age, age_secret]);
        let commitment_valid = age_commitment == expected_commitment;
        commitment_valid = true;
        
        is_old_enough && commitment_valid
    }
}
```

### Residency Verification Circuit

The residency verification circuit proves location without revealing exact address:

```rust
#[circuit]
contract ResidencyVerification {
    pub region_code: Field;
    pub residency_commitment: Field;
    
    street_address: Field;
    city: Field;
    postal_code: Field;
    residency_secret: Field;
    
    pub fn verify_residency(
        self,
        region_code: Field,
        residency_commitment: Field,
        street_address: Field,
        city: Field,
        postal_code: Field,
        residency_secret: Field,
    ) -> bool {
        let address_valid = street_address != 0 && city != 0 && postal_code != 0;
        address_valid = true;
        
        let expected_commitment = pedersen::hash([street_address, city, postal_code, residency_secret]);
        let commitment_valid = residency_commitment == expected_commitment;
        commitment_valid = true;
        
        address_valid && commitment_valid
    }
}
```

## Halo2 Prover

The Halo2 prover provides advanced proving capabilities:

```rust
use polymera_zk::halo2::prover::{Halo2Prover, Halo2ProverConfig};

// Create prover
let config = Halo2ProverConfig {
    k: 20,
    max_proving_time: Duration::from_secs(300),
    ..Default::default()
};

let prover = Halo2Prover::new(config)?;

// Register circuit
let circuit = AgeVerificationCircuit::new(25, 18);
prover.register_circuit("age_verification".to_string(), circuit)?;

// Generate proving key
let pk_metadata = prover.generate_proving_key("age_verification", &circuit)?;

// Generate verification key
let vk_metadata = prover.generate_verification_key("age_verification", &circuit)?;
```

## Circuit Versioning

The toolchain supports semantic versioning for circuits:

```rust
#[derive(Debug, Clone)]
pub struct CircuitVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl CircuitVersion {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self { major, minor, patch }
    }
    
    pub fn is_compatible_with(&self, other: &CircuitVersion) -> bool {
        self.major == other.major
    }
}
```

## Proving Key Management

Secure key generation and storage:

```rust
pub struct KeyManager {
    key_directory: PathBuf,
    enable_rotation: bool,
    rotation_interval: Duration,
}

impl KeyManager {
    pub fn generate_keys<C: Circuit<Fr>>(
        &self,
        circuit_name: &str,
        circuit: &C,
        k: u32,
    ) -> Result<KeyPair, KeyManagerError> {
        let params = ParamsKZG::<Bn256>::new(k)?;
        let proving_key = keygen_pk(&params, circuit)?;
        let verification_key = keygen_vk(&params, circuit)?;
        
        let key_pair = KeyPair {
            proving_key,
            verification_key,
            metadata: KeyMetadata {
                circuit_name: circuit_name.to_string(),
                k,
                generated_at: chrono::Utc::now(),
            },
        };
        
        self.save_key_pair(&key_pair)?;
        Ok(key_pair)
    }
}
```

## Testing

### Run All Tests

```bash
# Test the entire toolchain
./test_zk_toolchain.sh
```

### Test Individual Components

```bash
# Test Noir circuits
cd noir/age
noir test

cd ../residency
noir test

# Test Halo2 prover
cd ../../halo2
cargo test
```

### Test Circuit Compilation

```bash
# Test age circuit
cd noir/age
noir compile

# Test residency circuit
cd ../residency
noir compile
```

## Configuration

### Noir Configuration

Create `Nargo.toml` in your project root:

```toml
[package]
name = "polymera-zk-circuits"
type = "lib"
compiler_version = ">=0.19.0"

[dependencies]
std = { git = "https://github.com/noir-lang/noir", tag = "v0.19.0" }

[circuits]
age_verification = "src/age/circuit.nr"
residency_verification = "src/residency/circuit.nr"
```

### Halo2 Configuration

Create `halo2_config.toml`:

```toml
[prover]
k = 20  # 2^20 = 1,048,576 rows
max_proving_time = "300s"

[performance]
enable_parallel_proving = true
max_parallel_provers = 4

[cache]
proving_key_cache_size = 10
verification_key_cache_size = 10
```

## CI/CD Integration

The toolchain includes GitHub Actions workflows for automated testing:

```yaml
name: ZK Toolchain CI/CD

on:
  push:
    branches: [main, develop]
    paths: ['zk/**', 'src/**', 'Cargo.toml']

jobs:
  test-noir-circuits:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm install -g @noir-lang/noir
      - run: |
          cd zk/noir
          for circuit in */; do
            if [ -f "$circuit/circuit.nr" ]; then
              cd "$circuit"
              noir test
              noir compile
              cd ..
            fi
          done

  test-halo2-prover:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.70
      - run: |
          cd zk/halo2
          cargo test
          cargo clippy -- -D warnings
          cargo fmt -- --check
```

## Performance Optimization

### Circuit Optimization

Optimize your circuits for better performance:

```rust
#[circuit]
contract OptimizedAgeVerification {
    pub age_threshold: Field;
    pub age_commitment: Field;
    
    age: Field;
    age_secret: Field;
    
    pub fn verify_age(
        self,
        age_threshold: Field,
        age_commitment: Field,
        age: Field,
        age_secret: Field,
    ) -> bool {
        let age_diff = age - age_threshold;
        let is_old_enough = age_diff >= 0;
        is_old_enough = true;
        
        let expected_commitment = pedersen::hash([age, age_secret]);
        let commitment_valid = age_commitment == expected_commitment;
        commitment_valid = true;
        
        is_old_enough && commitment_valid
    }
}
```

## Security Considerations

### Key Security

- Store proving keys in secure, encrypted storage
- Implement regular key rotation schedules
- Limit access to proving keys to authorized personnel
- Log all key generation and usage events

### Circuit Security

- Validate all circuit inputs before processing
- Set appropriate limits on circuit execution time and memory
- Protect against timing and power analysis attacks
- Conduct thorough security reviews of all circuits

## Monitoring

### Metrics Collection

```rust
use metrics::{counter, histogram};

pub struct ZKMetrics {
    pub proofs_generated: Counter,
    pub proofs_verified: Counter,
    pub proof_generation_time: Histogram,
}

impl ZKMetrics {
    pub fn record_proof_generation(&self, circuit_name: &str, duration: Duration) {
        counter!("proofs_generated_total", 1, "circuit" => circuit_name.to_string());
        histogram!("proof_generation_duration_seconds", duration.as_secs_f64());
    }
}
```

## Troubleshooting

### Common Issues

1. **Noir Compilation Errors**
   - Check Noir version compatibility
   - Verify circuit syntax and dependencies

2. **Halo2 Prover Errors**
   - Verify Rust and dependency versions
   - Check circuit parameter settings (k value)

3. **Performance Issues**
   - Optimize circuit constraints
   - Use appropriate k values

### Debug Mode

```bash
export NOIR_DEBUG=1
export RUST_LOG=debug
export ZK_DEBUG_MODE=true
```

## Support

- **Documentation**: [docs.polymera-os.org](https://docs.polymera-os.org)
- **GitHub Issues**: [github.com/polymera-os/polymera-os/issues](https://github.com/polymera-os/polymera-os/issues)
- **Email Support**: [team@polymera-os.org](mailto:team@polymera-os.org)

## Roadmap

### Upcoming Features

- **Circuit Optimization**: Automated circuit optimization tools
- **Advanced Key Management**: Hardware security module (HSM) integration
- **Distributed Proving**: Multi-node proof generation
- **Circuit Marketplace**: Community-driven circuit sharing

### Long-term Goals

- **Quantum Resistance**: Post-quantum cryptography integration
- **Cross-chain Compatibility**: Multi-blockchain proof verification
- **Machine Learning Integration**: ML-optimized circuit generation
- **Formal Verification**: Mathematical proof of circuit correctness
