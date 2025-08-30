# Phase 2 Specification: PQC + Syscall/ABI Hardening

## Overview

Phase 2 focuses on **Post-Quantum Cryptography (PQC) integration** and **system call/ABI hardening** to establish a quantum-resistant security foundation while maintaining performance and stability guarantees established in Phase 1.5.

## 🎯 **Core Objectives**

### 1. **Post-Quantum Cryptography Integration**
- **Kyber KEM** (Key Encapsulation Mechanism) for secure key exchange
- **Dilithium** digital signatures for authentication and integrity
- Integration with **SecMan** (Security Manager) and **PolyBus** (IPC system)

### 2. **System Call & ABI Hardening**
- **Stable syscall interface** with versioning and compatibility guarantees
- **User/task boundary enforcement** with capability-based security
- **APIC timer integration** for precise timing and scheduling
- **Page-fault hardening** with enhanced security validation

## 🔐 **PQC Architecture**

### **Kyber KEM Integration**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │    │   PolyBus IPC   │    │   SecMan Core   │
│                 │◄──►│                 │◄──►│                 │
│  - Key Request  │    │  - Auth Header  │    │  - Kyber KEM    │
│  - Session Mgmt │    │  - Cap Tokens   │    │  - Key Store    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Key Components:**
- **Kyber-512** for initial key establishment
- **Kyber-768** for high-security applications
- **Hybrid approach** combining classical + PQC for transition period

### **Dilithium Signature Integration**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Capability    │    │   Signature     │    │   Verification  │
│   Token        │───►│   Generation    │───►│   Engine        │
│                 │    │   (Dilithium)   │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Signature Schemes:**
- **Dilithium2** for general-purpose signatures
- **Dilithium3** for high-security requirements
- **Dilithium5** for maximum security (when performance allows)

## 🏗️ **System Architecture**

### **Capability Token Format**
```
┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│   Version   │   Type      │   Rights    │   Expiry    │   Signature │
│   (8 bits)  │   (8 bits)  │  (16 bits)  │  (32 bits)  │ (256 bits)  │
└─────────────┴─────────────┴─────────────┴─────────────┴─────────────┘
```

**Token Types:**
- **READ** (0x01): Read-only access
- **WRITE** (0x02): Write access
- **EXEC** (0x04): Execution rights
- **ADMIN** (0x08): Administrative privileges
- **CUSTOM** (0x10-0xFF): Application-specific rights

### **User/Kernel Boundary**
```
┌─────────────────────────────────────────────────────────────────┐
│                        User Space                               │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   App A     │  │   App B     │  │   App C     │            │
│  │             │  │             │  │             │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│                    System Call Interface                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   Syscall   │  │   Cap      │  │   ABI      │            │
│  │   Table     │  │   Check    │  │   Version  │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│                        Kernel Space                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   SecMan    │  │   PolyBus   │  │   APIC     │            │
│  │             │  │             │  │   Timer    │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
└─────────────────────────────────────────────────────────────────┘
```

## ⏱️ **Performance Requirements & KPIs**

### **PQC Performance Targets**
| Operation | Target (p50) | Target (p95) | Target (p99) |
|-----------|--------------|--------------|--------------|
| **Kyber-512 KEM** | < 1.5ms | < 2.5ms | < 4.0ms |
| **Kyber-768 KEM** | < 2.5ms | < 4.0ms | < 6.0ms |
| **Dilithium2 Sign** | < 1.0ms | < 1.8ms | < 3.0ms |
| **Dilithium2 Verify** | < 0.8ms | < 1.5ms | < 2.5ms |
| **Dilithium3 Sign** | < 2.0ms | < 3.5ms | < 5.5ms |
| **Dilithium3 Verify** | < 1.5ms | < 2.8ms | < 4.5ms |

### **System Performance Targets**
| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **IPC Auth Overhead** | < 15% | Baseline vs PQC-enabled IPC |
| **Syscall Conformance** | 100% | ABI compatibility tests |
| **Cap Token Validation** | < 50μs | Capability check latency |
| **Page Fault Handling** | < 100μs | Enhanced security validation |
| **APIC Timer Precision** | ±1μs | Hardware timer accuracy |

## 🔧 **Technical Specifications**

### **Crypto FFI Interface**
```rust
// Kyber KEM operations
pub trait KyberKEM {
    fn keygen() -> Result<(PublicKey, SecretKey), CryptoError>;
    fn encapsulate(pk: &PublicKey) -> Result<(Ciphertext, SharedSecret), CryptoError>;
    fn decapsulate(ct: &Ciphertext, sk: &SecretKey) -> Result<SharedSecret, CryptoError>;
}

// Dilithium signature operations
pub trait DilithiumSig {
    fn keygen() -> Result<(VerificationKey, SigningKey), CryptoError>;
    fn sign(sk: &SigningKey, msg: &[u8]) -> Result<Signature, CryptoError>;
    fn verify(vk: &VerificationKey, msg: &[u8], sig: &Signature) -> Result<bool, CryptoError>;
}
```

### **Capability Store**
```rust
pub struct CapabilityStore {
    tokens: HashMap<CapTokenId, Capability>,
    rights_cache: LruCache<CapTokenId, Rights>,
    expiry_monitor: ExpiryMonitor,
}

impl CapabilityStore {
    pub fn validate_token(&self, token: &CapToken) -> Result<Rights, CapError>;
    pub fn check_rights(&self, token: &CapToken, required: Rights) -> Result<(), CapError>;
    pub fn revoke_token(&mut self, token_id: CapTokenId) -> Result<(), CapError>;
}
```

### **Enhanced Page Fault Handler**
```rust
pub struct PageFaultHandlerV2 {
    security_validator: SecurityValidator,
    capability_checker: CapabilityChecker,
    audit_logger: AuditLogger,
}

impl PageFaultHandlerV2 {
    pub fn handle_fault(&self, fault: PageFault) -> Result<(), FaultError> {
        // 1. Validate fault source and permissions
        // 2. Check capability tokens
        // 3. Apply security policies
        // 4. Log security events
        // 5. Handle fault or reject
    }
}
```

## 🧪 **Testing & Validation**

### **PQC Test Suite**
- **Vector tests** using NIST PQC test vectors
- **Performance benchmarks** across different key sizes
- **Interoperability tests** with reference implementations
- **Side-channel resistance** validation

### **ABI Compatibility Tests**
- **Syscall table validation** (100% conformance)
- **Backward compatibility** with Phase 1.5
- **Forward compatibility** for future extensions
- **Performance regression** detection

### **Security Validation**
- **Capability token** integrity and expiry
- **User/kernel boundary** enforcement
- **Page fault** security validation
- **APIC timer** precision and security

## 📊 **Success Metrics**

### **Phase 2 Completion Criteria**
1. ✅ **PQC Integration Complete**
   - Kyber KEM operational in SecMan
   - Dilithium signatures in PolyBus
   - Performance targets met

2. ✅ **Syscall/ABI Hardened**
   - Stable interface established
   - 100% conformance achieved
   - Backward compatibility maintained

3. ✅ **Security Boundaries Enforced**
   - User/kernel separation
   - Capability-based access control
   - Enhanced page fault handling

4. ✅ **Performance Targets Met**
   - PQC operations within latency bounds
   - IPC overhead < 15%
   - APIC timer precision ±1μs

## 🔄 **Migration & Compatibility**

### **Phase 1.5 → Phase 2 Transition**
- **Hybrid crypto** support during transition
- **Gradual migration** to PQC-only operations
- **Backward compatibility** maintained
- **Performance monitoring** throughout transition

### **Future Extensibility**
- **Modular crypto** implementations
- **Pluggable signature** schemes
- **Extensible capability** system
- **Versioned ABI** support

---

**Document Version**: 1.0  
**Last Updated**: Phase 2 Kickoff  
**Next Review**: Phase 2 Design Review
