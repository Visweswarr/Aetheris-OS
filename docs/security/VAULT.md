# NGFS Personal Data Vault - Security Guide

## Overview

The NGFS Personal Data Vault (PDV) is a secure, self-sovereign keychain subsystem that provides encrypted storage for user secrets, keys, and personal files. This document outlines the security architecture, threat model, and best practices for the vault system.

## Security Architecture

### Encryption Layers

The vault implements a multi-layered encryption approach:

```
┌─────────────────────────────────────────────────────────────┐
│                    Vault Entry                              │
├─────────────────────────────────────────────────────────────┤
│  Metadata (CBOR, unencrypted)                              │
│  - Entry ID, kind, timestamps, tags                        │
│  - Subject DID, capabilities                               │
├─────────────────────────────────────────────────────────────┤
│  EncEnvelopeV1 (C AEAD)                                    │
│  - XChaCha20-Poly1305 encryption                           │
│  - 256-bit key, 24-byte nonce, 16-byte tag                │
│  - Additional authenticated data (AAD)                     │
├─────────────────────────────────────────────────────────────┤
│  Plaintext Content                                          │
│  - User secrets, keys, documents                           │
│  - Verifiable credentials                                   │
└─────────────────────────────────────────────────────────────┘
```

### Key Management

#### Key Hierarchy

```
┌─────────────────┐
│   Master Key    │ ← Root of trust (KeyVault)
│   (PQC Ready)   │
└─────────────────┘
         │
         ▼
┌─────────────────┐
│   KEK (Key      │ ← Key Encryption Key
│   Encryption)   │   (XChaCha20-Poly1305)
└─────────────────┘
         │
         ▼
┌─────────────────┐
│   DEK (Data     │ ← Data Encryption Key
│   Encryption)   │   (per-entry, unique)
└─────────────────┘
```

#### Key Derivation

- **KEK**: Derived from Master Key using HKDF-SHA256
- **DEK**: Randomly generated per entry using secure RNG
- **Nonce**: Randomly generated per encryption operation
- **AAD**: Includes entry metadata for binding

### Access Control Model

#### CapToken v2 Enforcement

The vault enforces fine-grained access control through CapTokens:

```rust
pub struct Capability {
    pub operation: String,        // read, write, delete, list, search, admin
    pub resource: String,         // vault:*, vault:entry-id, credential:issue
    pub conditions: Option<CapConditions>, // Time, IP, rate limits
    pub expires_at: Option<DateTime<Utc>>, // Expiration
}
```

**Resource Patterns:**
- `vault:*`: Access to all vault entries
- `vault:entry-id`: Access to specific entry
- `vault:entry-prefix*`: Access to entries with matching prefix
- `credential:issue`: Ability to issue verifiable credentials

**Operation Types:**
- `read`: Decrypt and view entry content
- `write`: Create or modify entries
- `delete`: Mark entries as revoked (logical deletion)
- `list`: View entry metadata without decryption
- `search`: Query entries with filters
- `admin`: Administrative operations

#### DID Binding

Each vault entry is bound to a specific Decentralized Identifier (DID):

- **Subject DID**: The DID that owns the entry
- **Verification Method**: Cryptographic method for access verification
- **DID Document**: Contains public keys and verification methods

### Threat Model

#### Attack Vectors

1. **Cryptographic Attacks**
   - Brute force against encryption keys
   - Side-channel attacks on encryption operations
   - Quantum attacks on classical cryptography

2. **Access Control Bypass**
   - CapToken forgery or tampering
   - Privilege escalation
   - Replay attacks

3. **Data Exfiltration**
   - Memory dumps
   - Logging of sensitive data
   - Network interception

4. **Availability Attacks**
   - Denial of service
   - Resource exhaustion
   - Key deletion

#### Mitigation Strategies

1. **Cryptographic Security**
   - Post-quantum cryptography ready
   - Constant-time operations
   - Secure random number generation
   - Key rotation support

2. **Access Control**
   - Cryptographic signature verification
   - Time-based expiration
   - Rate limiting
   - Audit logging

3. **Data Protection**
   - Zeroization of secrets
   - No plaintext logging
   - Memory protection
   - Secure deletion

4. **Availability**
   - Redundant storage
   - Backup and recovery
   - Resource limits
   - Graceful degradation

## Security Features

### Encryption

#### EncEnvelopeV1

The canonical encryption envelope provides:

- **AEAD Security**: Authenticated encryption with associated data
- **Nonce Uniqueness**: Each encryption uses a unique nonce
- **AAD Binding**: Metadata bound to encrypted content
- **Tag Verification**: Integrity protection against tampering

#### Key Rotation

The vault supports secure key rotation:

1. **KEK Rotation**: Master key can be rotated without re-encrypting data
2. **DEK Rotation**: Individual entries can be re-encrypted with new keys
3. **Forward Secrecy**: Compromised keys don't affect previously encrypted data

### Access Control

#### CapToken Verification

Each operation verifies the CapToken:

1. **Signature Verification**: Cryptographic verification of issuer
2. **Expiration Check**: Token must not be expired
3. **Capability Match**: Token must grant required operation
4. **Resource Authorization**: Token must cover target resource
5. **Condition Validation**: Time, IP, and rate limits enforced

#### DID Verification

DID-based access control:

1. **Document Resolution**: Resolve DID to document
2. **Method Verification**: Verify verification method exists
3. **Key Validation**: Validate public keys
4. **Revocation Check**: Check for revoked DIDs

### Audit and Logging

#### Event Emission

The vault emits security events:

```rust
Event "vault.op" {
    operation: String,      // Operation type
    entry_id: String,       // Target entry
    subject_did: String,    // User DID
    success: bool,          // Operation result
    timestamp: DateTime,    // When operation occurred
    metadata: Map,          // Additional context
}
```

#### Audit Records

Comprehensive audit trail:

- **VAULT_OP_OK**: Successful operations
- **VAULT_OP_DENY**: Denied operations with reason
- **VAULT_OP_ERROR**: Failed operations with error details
- **VAULT_OP_ADMIN**: Administrative operations

## Best Practices

### Development

#### Secure Coding

1. **Memory Management**
   - Use zeroizing containers for secrets
   - Clear sensitive data after use
   - Avoid logging sensitive information

2. **Error Handling**
   - Don't leak information in error messages
   - Use consistent error codes
   - Log security-relevant errors

3. **Input Validation**
   - Validate all inputs
   - Sanitize file paths and IDs
   - Check capability requirements

#### Testing

1. **Security Tests**
   - Test CapToken enforcement
   - Verify encryption/decryption
   - Check access control boundaries

2. **Penetration Testing**
   - Attempt privilege escalation
   - Test input validation
   - Verify audit logging

### Deployment

#### Key Management

1. **Master Key Security**
   - Store in hardware security module (HSM)
   - Use strong key derivation
   - Implement key rotation

2. **Access Control**
   - Limit CapToken issuance
   - Use short expiration times
   - Monitor for abuse

#### Monitoring

1. **Security Events**
   - Monitor for failed operations
   - Track unusual access patterns
   - Alert on security violations

2. **Performance Metrics**
   - Monitor encryption/decryption times
   - Track memory usage
   - Watch for resource exhaustion

### Operational Security

#### Access Management

1. **CapToken Lifecycle**
   - Issue minimal required capabilities
   - Set appropriate expiration times
   - Revoke unused tokens

2. **User Management**
   - Verify DID ownership
   - Monitor user activity
   - Implement least privilege

#### Incident Response

1. **Detection**
   - Monitor audit logs
   - Watch for anomalies
   - Use automated alerts

2. **Response**
   - Isolate affected systems
   - Revoke compromised tokens
   - Investigate root cause

3. **Recovery**
   - Restore from backups
   - Rotate affected keys
   - Update security measures

## Compliance and Standards

### Cryptographic Standards

- **NIST SP 800-38D**: GCM mode requirements
- **RFC 8439**: ChaCha20-Poly1305 specification
- **RFC 5869**: HKDF key derivation
- **RFC 6979**: Deterministic signatures

### Security Frameworks

- **OWASP**: Web application security
- **NIST Cybersecurity Framework**: Risk management
- **ISO 27001**: Information security management
- **SOC 2**: Security controls and processes

### Privacy Regulations

- **GDPR**: Data protection and privacy
- **CCPA**: California privacy rights
- **HIPAA**: Healthcare data protection
- **SOX**: Financial data security

## Future Enhancements

### Post-Quantum Cryptography

1. **Lattice-Based Encryption**
   - Kyber for key encapsulation
   - Dilithium for digital signatures
   - SPHINCS+ for hash-based signatures

2. **Hybrid Schemes**
   - Classical + quantum-resistant
   - Gradual migration path
   - Backward compatibility

### Advanced Access Control

1. **Attribute-Based Encryption**
   - Fine-grained access control
   - Policy-based encryption
   - Dynamic permissions

2. **Zero-Knowledge Proofs**
   - Privacy-preserving authentication
   - Selective disclosure
   - Verifiable credentials

### Threat Intelligence

1. **Machine Learning**
   - Anomaly detection
   - Threat pattern recognition
   - Automated response

2. **Blockchain Integration**
   - Immutable audit logs
   - Decentralized identity
   - Smart contract policies

## Conclusion

The NGFS Personal Data Vault provides enterprise-grade security for sensitive data storage. The multi-layered encryption, comprehensive access control, and extensive audit logging ensure that user secrets remain protected while maintaining usability and compliance.

The vault's architecture is designed to evolve with emerging threats and cryptographic advances, particularly in the area of post-quantum cryptography. The polyglot toolchain enables comprehensive testing and validation across multiple programming languages and platforms.

By following the security best practices outlined in this document, organizations can deploy the vault with confidence, knowing that their sensitive data is protected by state-of-the-art security measures and comprehensive monitoring capabilities.
