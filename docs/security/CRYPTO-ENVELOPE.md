# NGFS Envelope Encryption Security

## Overview

NGFS v1 implements envelope encryption for CAS chunks using XChaCha20-Poly1305 AEAD with a two-layer key hierarchy:

1. **Data Encryption Key (DEK)**: Encrypts individual chunks
2. **Key Encryption Key (KEK)**: Encrypts DEKs, derived from KeyVault

## Security Guarantees

### Constant-Time Operations

All cryptographic operations in `libpolycrypto` are implemented with constant-time guarantees:

- **No timing-dependent branches on secrets**: All conditional operations use constant-time techniques
- **Fixed execution paths**: Encryption/decryption follow identical code paths regardless of input
- **Cache-timing resistance**: Memory access patterns are independent of secret data

### Input Validation

The library enforces strict input validation:

```c
// Buffer overlap detection prevents side-channel attacks
bool aeth_buffers_overlap(const void* ptr1, size_t len1, 
                         const void* ptr2, size_t len2);

// Length validation prevents buffer overflows
bool aeth_validate_buffer(const void* ptr, size_t len, size_t max_len);
```

### Memory Security

- **Secure zeroing**: `aeth_memzero()` ensures sensitive data is cleared even if compiler optimizes away memset
- **No dynamic allocation**: All buffers are pre-allocated to prevent heap-based attacks
- **Stack protection**: Compiler flags enable stack canaries and frame pointer validation

## Nonce Policy

### Virtual Clock Generation

Nonces are generated deterministically using a virtual clock:

```
nonce = [virtual_clock_counter (8 bytes)] + [mount_salt (16 bytes)]
```

- **Virtual clock**: Monotonic counter per mount, incremented after each encryption
- **Mount salt**: 16-byte random value unique to each filesystem mount
- **Deterministic**: Same inputs always produce same nonce sequence

### Nonce Uniqueness

- **Per-mount isolation**: Different mounts use different salt values
- **Monotonic counter**: Virtual clock ensures nonce uniqueness within mount
- **No reuse**: Each encryption operation uses a unique nonce

## Associated Data (AD) Design

### Deterministic Construction

AD includes all fields that affect encryption determinism:

```rust
pub struct AssociatedData {
    pub schema_hash: [u8; 32],    // NGFS schema hash
    pub key_id: String,           // Key identifier
    pub algorithm: EncryptionAlg, // Algorithm identifier
    pub chunk_len: u64,           // Chunk length
    pub metadata: BTreeMap<String, String>, // Sorted metadata
}
```

### Schema Hash Integration

- **Deterministic ordering**: All fields are sorted for consistent CBOR encoding
- **Schema versioning**: Schema hash prevents accidental schema drift
- **Tamper detection**: AD modification invalidates authentication

## Key Management

### Key Hierarchy

```
KeyVault (Root)
    ↓
KEK (Key Encryption Key) - X25519/Kyber KEM
    ↓
DEK (Data Encryption Key) - Random per chunk
    ↓
Chunk Data
```

### Key Derivation

- **X25519 ECDH**: Default KEK derivation (post-quantum upgradeable)
- **Kyber KEM**: Available with `--features pqc` for post-quantum security
- **Key rotation**: KEKs can be rotated without re-encrypting all data

### Key Lifecycle

1. **Generation**: DEKs generated randomly per chunk
2. **Encryption**: DEKs encrypted with KEK using XChaCha20-Poly1305
3. **Storage**: Encrypted DEKs stored with chunk metadata
4. **Decryption**: KEK decrypts DEK, DEK decrypts chunk data
5. **Zeroization**: All keys zeroized after use

## Attack Vectors and Mitigations

### Side-Channel Attacks

| Attack Vector | Mitigation |
|---------------|------------|
| Timing attacks | Constant-time implementation |
| Cache attacks | Fixed memory access patterns |
| Power analysis | No secret-dependent branches |

### Cryptographic Attacks

| Attack Vector | Mitigation |
|---------------|------------|
| Nonce reuse | Virtual clock + mount salt |
| Key compromise | Key rotation + forward secrecy |
| Algorithm attacks | AEAD with authentication |

### Implementation Attacks

| Attack Vector | Mitigation |
|---------------|------------|
| Buffer overflow | Strict length validation |
| Memory corruption | No dynamic allocation |
| Compiler optimization | Volatile function pointers |

## Security Properties

### Confidentiality

- **Chunk-level encryption**: Each chunk encrypted with unique DEK
- **Key isolation**: KEK compromise doesn't expose chunk data
- **Algorithm strength**: XChaCha20-Poly1305 provides 256-bit security

### Integrity

- **Authentication**: Poly1305 MAC prevents tampering
- **Associated data**: AD modification detected and rejected
- **Schema validation**: CBOR schema prevents malformed data

### Availability

- **Crash recovery**: Index rebuilding from segment data
- **Corruption detection**: Checksum validation identifies bad chunks
- **Graceful degradation**: Corrupt chunks logged and skipped

## Compliance and Standards

### Cryptographic Standards

- **XChaCha20-Poly1305**: RFC 8439, widely deployed
- **Blake3**: Fast, secure hash function
- **X25519**: RFC 7748, post-quantum upgradeable

### Security Requirements

- **FIPS 140-2**: Cryptographic module validation (planned)
- **Common Criteria**: Security evaluation (planned)
- **SOC 2**: Security controls audit (planned)

## Audit and Monitoring

### Security Events

All cryptographic operations generate audit events:

- `NGFS_ENCRYPT_OK`: Successful encryption
- `NGFS_DECRYPT_OK`: Successful decryption
- `NGFS_DENY`: Failed operation (with reason)

### Monitoring

- **Key usage**: Track KEK/DEK usage patterns
- **Performance**: Monitor encryption/decryption latency
- **Errors**: Alert on authentication failures

## Future Enhancements

### Post-Quantum Cryptography

- **Kyber KEM**: Already implemented, ready for activation
- **Dilithium signatures**: For snapshot authentication
- **Hybrid schemes**: Combine classical and post-quantum algorithms

### Advanced Features

- **Format-preserving encryption**: For structured data
- **Searchable encryption**: For metadata queries
- **Homomorphic encryption**: For computation on encrypted data

## References

- [RFC 8439: ChaCha20 and Poly1305](https://tools.ietf.org/html/rfc8439)
- [RFC 7748: Elliptic Curves for Security](https://tools.ietf.org/html/rfc7748)
- [NIST Post-Quantum Cryptography](https://csrc.nist.gov/projects/post-quantum-cryptography)
- [OWASP Cryptographic Storage](https://owasp.org/www-project-cheat-sheets/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
