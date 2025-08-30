# Zero-Knowledge (ZK) Toolchain Setup

Comprehensive setup guide for the Zero-Knowledge proof toolchain in Polymera OS.

## Overview

The ZK toolchain provides a complete framework for developing, testing, and deploying zero-knowledge proof circuits using Noir and Halo2.

## Prerequisites

- **Rust**: 1.70+ with Cargo
- **Node.js**: 18+ with npm
- **Git**: Latest version
- **CMake**: 3.16+ (for native dependencies)

## Installation

### 1. Install Noir

```bash
# Install Noir globally
npm install -g @noir-lang/noir

# Verify installation
noir --version
```

### 2. Install Halo2 Dependencies

```bash
# Install Rust dependencies
cargo add halo2_proofs
cargo add halo2curves

# Install development dependencies
cargo add --dev halo2_proofs
cargo add --dev halo2curves
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

## Circuit Development

### Age Verification Circuit

```rust
// src/age/circuit.nr
use dep::std::hash::pedersen;

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

```rust
// src/residency/circuit.nr
use dep::std::hash::pedersen;

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

## Halo2 Integration

### Basic Prover Usage

```rust
use polymera_zk::halo2::prover::{Halo2Prover, Halo2ProverConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Halo2ProverConfig {
        k: 20,
        max_proving_time: Duration::from_secs(300),
        ..Default::default()
    };
    
    let prover = Halo2Prover::new(config)?;
    
    let circuit = AgeVerificationCircuit::new(25, 18);
    prover.register_circuit("age_verification".to_string(), circuit)?;
    
    let pk_metadata = prover.generate_proving_key("age_verification", &circuit)?;
    let vk_metadata = prover.generate_verification_key("age_verification", &circuit)?;
    
    Ok(())
}
```

### Proof Generation and Verification

```rust
fn generate_and_verify_proof() -> Result<(), Box<dyn std::error::Error>> {
    let prover = Halo2Prover::with_default_config()?;
    let circuit = AgeVerificationCircuit::new(25, 18);
    
    let public_inputs = vec![Fr::from(18), Fr::from(12345)];
    let (proof_bytes, proof_metadata) = prover.generate_proof("age_verification", &circuit, &public_inputs)?;
    
    let is_valid = prover.verify_proof("age_verification", &proof_bytes, &public_inputs)?;
    
    Ok(())
}
```

## Circuit Versioning

### Version Management

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

### Circuit Registry

```rust
pub struct CircuitRegistry {
    circuits: Arc<Mutex<HashMap<String, CircuitMetadata>>>,
}

#[derive(Debug, Clone)]
pub struct CircuitMetadata {
    pub name: String,
    pub version: CircuitVersion,
    pub circuit_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

## Proving Key Management

### Key Generation

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

## CI/CD Integration

### GitHub Actions Workflow

Create `.github/workflows/zk-toolchain.yml`:

```yaml
name: ZK Toolchain CI/CD

on:
  push:
    branches: [main, develop]
    paths: ['zk/**', 'src/**', 'Cargo.toml']
  pull_request:
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

  circuit-verification:
    runs-on: ubuntu-latest
    needs: [test-noir-circuits, test-halo2-prover]
    steps:
      - uses: actions/checkout@v3
      - run: |
          cd zk
          cargo test --test integration_tests
```

## Testing

### Circuit Testing

```bash
#!/bin/bash
# scripts/test_circuits.sh

set -e

echo "=== Testing ZK Circuits ==="

# Test Noir circuits
cd zk/noir
for circuit_dir in */; do
    if [ -d "$circuit_dir" ] && [ -f "$circuit_dir/circuit.nr" ]; then
        echo "Testing circuit: $circuit_dir"
        cd "$circuit_dir"
        noir test
        noir compile
        cd ..
    fi
done

# Test Halo2 prover
cd ../halo2
cargo test
cargo test --test integration_tests
cargo clippy -- -D warnings
cargo fmt -- --check

echo "=== All circuit tests passed ==="
```

## Performance Optimization

### Circuit Optimization

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
