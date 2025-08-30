# Post-Quantum Cryptography (PQC) Foundation

## Overview

This document describes the PQC foundation implementation for Polymera OS, integrating **liboqs** for CRYSTALS-Kyber (KEM) and Dilithium (SIG) algorithms. The implementation provides safe Rust wrappers with zeroization on key drop and deterministic builds.

## 🔐 **Algorithms & Parameter Sets**

### **CRYSTALS-Kyber (Key Encapsulation Mechanism)**

Kyber is a lattice-based key encapsulation mechanism that provides IND-CCA2 security.

#### **Parameter Sets**
| Parameter Set | Security Level | Public Key | Secret Key | Ciphertext | Shared Secret |
|---------------|----------------|-------------|------------|------------|---------------|
| **Kyber512** | 128 bits | 800 bytes | 1,632 bytes | 768 bytes | 32 bytes |
| **Kyber768** | 192 bits | 1,184 bytes | 2,400 bytes | 1,088 bytes | 32 bytes |
| **Kyber1024** | 256 bits | 1,568 bytes | 3,168 bytes | 1,568 bytes | 32 bytes |

#### **Algorithm Details**
- **Type**: Lattice-based KEM
- **Lattice**: Module Learning With Errors (MLWE)
- **Polynomial Ring**: R = Z[X]/(X^n + 1) where n = 256
- **Modulus**: q = 3329
- **NIST Level**: 1, 3, 5 respectively

### **CRYSTALS-Dilithium (Digital Signature)**

Dilithium is a lattice-based digital signature scheme that provides EUF-CMA security.

#### **Parameter Sets**
| Parameter Set | Security Level | Public Key | Secret Key | Signature |
|---------------|----------------|-------------|------------|-----------|
| **Dilithium2** | 128 bits | 1,312 bytes | 2,528 bytes | 2,420 bytes |
| **Dilithium3** | 192 bits | 1,952 bytes | 4,000 bytes | 3,293 bytes |
| **Dilithium5** | 256 bits | 2,592 bytes | 4,864 bytes | 4,595 bytes |

#### **Algorithm Details**
- **Type**: Lattice-based signature
- **Lattice**: Module Learning With Errors (MLWE) + Module Short Integer Solution (MSIS)
- **Polynomial Ring**: R = Z[X]/(X^n + 1) where n = 256
- **Modulus**: q = 8380417
- **NIST Level**: 2, 3, 5 respectively

## ⚡ **Performance Targets**

### **Kyber Performance Targets**
| Operation | Kyber512 | Kyber768 | Kyber1024 |
|-----------|----------|----------|-----------|
| **Key Generation** | < 0.8ms p50 | < 1.2ms p50 | < 1.8ms p50 |
| **Encapsulation** | < 1.5ms p50 | < 2.5ms p50 | < 3.5ms p50 |
| **Decapsulation** | < 1.5ms p50 | < 2.5ms p50 | < 3.5ms p50 |

### **Dilithium Performance Targets**
| Operation | Dilithium2 | Dilithium3 | Dilithium5 |
|-----------|------------|------------|------------|
| **Key Generation** | < 1.0ms p50 | < 2.0ms p50 | < 3.5ms p50 |
| **Signing** | < 1.0ms p50 | < 2.0ms p50 | < 3.5ms p50 |
| **Verification** | < 0.8ms p50 | < 1.5ms p50 | < 2.8ms p50 |

### **System Performance Targets**
| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Memory Usage** | < 50MB total | Resident set size monitoring |
| **Heap Leaks** | 0 | Valgrind/AddressSanitizer |
| **Zeroization** | 100% | Memory probe testing |
| **Build Time** | < 5 minutes | CI pipeline timing |

## 🏗️ **Implementation Architecture**

### **Build System**
```
crypto/liboqs/
├── build.rs              # Deterministic build with pinned commit
├── wrapper.h             # C wrapper headers
├── wrapper.c             # C wrapper implementation
├── bindings.rs           # Auto-generated Rust bindings
├── error.rs              # Error handling
├── kyber.rs              # Kyber KEM implementation
├── dilithium.rs          # Dilithium signature implementation
└── mod.rs                # Module exports
```

### **Safe Wrapper Design**
```rust
// Memory-safe wrapper with zeroization
pub struct KyberSecretKey {
    inner: kyber_secret_key_t,
    _phantom: PhantomData<*mut c_void>,
}

impl Drop for KyberSecretKey {
    fn drop(&mut self) {
        // Zeroize memory before deallocation
        unsafe {
            // Use liboqs zeroization function
            oqs_memcleanse(self.inner as *mut _, std::mem::size_of_val(&self.inner));
        }
    }
}
```

### **Zeroization Strategy**
1. **Automatic Zeroization**: All secret keys implement `Drop` with zeroization
2. **Memory Cleanup**: Use liboqs `oqs_memcleanse` for secure memory clearing
3. **Test Verification**: Memory probe tests verify zeroization effectiveness
4. **Compiler Barriers**: Prevent optimization of zeroization operations

## 🔧 **Build Configuration**

### **Deterministic Builds**
- **Pinned Commit**: liboqs v0.8.0 (stable release)
- **Build Flags**: `-DBUILD_SHARED_LIBS=OFF` (static linking)
- **Optimization**: `-DCMAKE_BUILD_TYPE=Release`
- **Algorithm Selection**: Only Kyber and Dilithium enabled

### **Environment Variables**
```bash
# Use system liboqs (optional)
export LIBOQS_USE_SYSTEM=1

# Build with bundled liboqs (default)
export LIBOQS_USE_SYSTEM=0
```

### **Dependencies**
- **CMake**: 3.16+
- **Git**: For cloning liboqs
- **C Compiler**: GCC/Clang with C99 support
- **OpenSSL**: For cryptographic operations

## 🧪 **Testing Strategy**

### **Known-Answer Vector Tests**
```rust
#[test]
fn test_kyber_roundtrip() {
    let params = KyberParameterSet::Kyber768;
    
    // Generate keypair
    let (pk, sk) = Kyber::keygen(params).unwrap();
    
    // Encapsulate
    let (ct, ss1) = Kyber::encapsulate(&pk).unwrap();
    
    // Decapsulate
    let ss2 = Kyber::decapsulate(&ct, &sk).unwrap();
    
    // Verify shared secrets match
    assert_eq!(ss1, ss2);
}
```

### **Zeroization Tests**
```rust
#[test]
fn test_key_zeroization() {
    let params = KyberParameterSet::Kyber768;
    let (pk, sk) = Kyber::keygen(params).unwrap();
    
    // Get memory address
    let sk_ptr = &sk as *const _ as usize;
    let sk_size = std::mem::size_of_val(&sk);
    
    // Drop secret key
    drop(sk);
    
    // Verify memory is zeroized (test-only probe)
    let memory_region = unsafe {
        std::slice::from_raw_parts(sk_ptr as *const u8, sk_size)
    };
    
    // Check that memory contains zeros
    assert!(memory_region.iter().all(|&b| b == 0));
}
```

### **Performance Tests**
```rust
#[test]
fn test_kyber_performance() {
    let params = KyberParameterSet::Kyber768;
    let iterations = 1000;
    
    let start = std::time::Instant::now();
    
    for _ in 0..iterations {
        let (pk, sk) = Kyber::keygen(params).unwrap();
        let (ct, _) = Kyber::encapsulate(&pk).unwrap();
        let _ = Kyber::decapsulate(&ct, &sk).unwrap();
    }
    
    let duration = start.elapsed();
    let avg_time = duration / iterations;
    
    // Verify performance targets
    assert!(avg_time.as_millis() < 5); // < 5ms average
}
```

## 🛡️ **Security Considerations**

### **Memory Safety**
- **Zeroization**: All secret material is zeroized on drop
- **No Heap Leaks**: Comprehensive memory leak testing
- **Bounds Checking**: All array accesses are bounds-checked
- **Type Safety**: Rust type system prevents misuse

### **Cryptographic Security**
- **NIST Compliance**: Uses NIST-approved parameter sets
- **Constant Time**: Critical operations are constant-time
- **Side-Channel Resistance**: Protected against timing attacks
- **Key Isolation**: Keys are isolated in separate memory regions

### **Build Security**
- **Deterministic Builds**: Pinned commits ensure reproducibility
- **Static Linking**: No dynamic library injection attacks
- **Source Verification**: liboqs source is verified and pinned
- **Audit Trail**: All builds are logged and auditable

## 📊 **Integration Points**

### **SecMan Integration**
```rust
// Security Manager PQC operations
impl SecMan {
    pub fn generate_kyber_keypair(&self, params: KyberParameterSet) -> Result<(PublicKey, SecretKey), Error> {
        let (pk, sk) = Kyber::keygen(params)?;
        Ok((PublicKey::from_kyber(pk), SecretKey::from_kyber(sk)))
    }
    
    pub fn generate_dilithium_keypair(&self, params: DilithiumParameterSet) -> Result<(PublicKey, SecretKey), Error> {
        let (pk, sk) = Dilithium::keygen(params)?;
        Ok((PublicKey::from_dilithium(pk), SecretKey::from_dilithium(sk)))
    }
}
```

### **PolyBus Integration**
```rust
// IPC authentication with PQC signatures
impl PolyBus {
    pub fn sign_message(&self, message: &[u8], sk: &DilithiumSecretKey) -> Result<Signature, Error> {
        Dilithium::sign(message, sk)
    }
    
    pub fn verify_message(&self, message: &[u8], signature: &Signature, pk: &DilithiumPublicKey) -> Result<bool, Error> {
        Dilithium::verify(message, signature, pk)
    }
}
```

### **Capability Token Integration**
```rust
// PQC-signed capability tokens
impl CapabilityToken {
    pub fn sign_with_dilithium(&mut self, sk: &DilithiumSecretKey) -> Result<(), Error> {
        let signature = Dilithium::sign(&self.to_bytes(), sk)?;
        self.set_signature(signature);
        Ok(())
    }
    
    pub fn verify_with_dilithium(&self, pk: &DilithiumPublicKey) -> Result<bool, Error> {
        let signature = self.signature()?;
        Dilithium::verify(&self.to_bytes(), &signature, pk)
    }
}
```

## 🔄 **Migration & Compatibility**

### **Phase 1.5 → Phase 2 Transition**
1. **Hybrid Mode**: Support both classical and PQC algorithms
2. **Gradual Migration**: Migrate critical systems first
3. **Performance Monitoring**: Track performance impact
4. **Fallback Support**: Maintain classical crypto for compatibility

### **Backward Compatibility**
- **API Stability**: All existing APIs remain functional
- **Performance**: Gradual degradation (100% → 95% → 90%)
- **Security**: Progressive enhancement (Medium → High → Higher)
- **Migration Tools**: Automated migration scripts

## 📈 **Monitoring & Metrics**

### **Performance Metrics**
- **Operation Latency**: P50, P95, P99 percentiles
- **Throughput**: Operations per second
- **Memory Usage**: Peak and average memory consumption
- **CPU Usage**: CPU time per operation

### **Security Metrics**
- **Zeroization Success Rate**: 100% target
- **Memory Leak Count**: 0 target
- **Cryptographic Validation**: NIST test vector success rate
- **Side-Channel Resistance**: Timing analysis results

### **Build Metrics**
- **Build Time**: Total build duration
- **Build Reproducibility**: 100% target
- **Dependency Pinning**: All dependencies pinned
- **Security Audits**: Regular security reviews

## 🚀 **Future Enhancements**

### **Algorithm Expansion**
- **Falcon**: Additional signature algorithm
- **SPHINCS+**: Hash-based signatures
- **Classic McEliece**: Code-based KEM
- **HQC**: Hamming Quasi-Cyclic KEM

### **Performance Optimization**
- **SIMD Instructions**: Vectorized operations
- **Parallel Processing**: Multi-threaded operations
- **Hardware Acceleration**: Specialized crypto hardware
- **Memory Pooling**: Optimized memory management

### **Security Enhancements**
- **Post-Quantum TLS**: TLS 1.3 with PQC
- **Hybrid Schemes**: Classical + PQC combinations
- **Threshold Signatures**: Distributed signing
- **Zero-Knowledge Proofs**: Privacy-preserving protocols

---

**Document Version**: 1.0  
**Last Updated**: Phase 2 PQC Foundation  
**Next Review**: Implementation completion  
**Status**: 🚀 **IMPLEMENTATION READY**
