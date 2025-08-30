# Dev Secure Boot System

## Overview

The **Dev Secure Boot** system provides a development-mode chain of trust for Polymera OS kernel images using post-quantum cryptography (Dilithium2). This system ensures kernel integrity and authenticity during early boot while maintaining flexibility for development workflows.

## Architecture

### Components

1. **Kernel Attestation Module** (`kernel/src/boot/attest.rs`)
   - Verifies kernel image signatures at early boot
   - Provides development bypass functionality
   - Exposes attestation information via syscalls

2. **Host-Side Signing Tool** (`tooling/attest/sign.rs`)
   - Signs kernel images with development keys
   - Embeds build ID and signing certificate
   - Supports both development and production builds

3. **Host-Side Verification Tool** (`tooling/attest/verify.rs`)
   - Verifies signed kernel images
   - Validates signatures and certificates
   - Provides detailed verification reports

### Data Flow

```
[Build System] → [Kernel Image] → [Signing Tool] → [Signed Image] → [QEMU/Device] → [Boot Verification]
```

## Key Features

### Development Mode Chain of Trust
- **Kernel Signature Verification**: Dilithium2 signatures over kernel images
- **Build ID Embedding**: Git hash or build identifier for traceability
- **Certificate Management**: Development signing certificates
- **Dev Bypass**: Optional bypass for development testing

### Security Properties
- **Post-Quantum Security**: Uses NIST PQC standard Dilithium2
- **Integrity Protection**: Prevents kernel tampering
- **Authenticity Verification**: Ensures kernel source
- **Replay Protection**: Build ID prevents old image reuse

## Usage

### Building and Signing

#### 1. Build Kernel Image
```bash
# Build the kernel
cargo build --release --package polymera-kernel

# The kernel image will be available at target/release/polymera-kernel.bin
```

#### 2. Sign Kernel Image
```bash
# Navigate to attestation tools
cd tooling/attest

# Sign with development key
cargo run --bin kernel-signer \
  -i ../../target/release/polymera-kernel.bin \
  -o ../../target/release/signed-kernel.bin \
  -k dev-key.pem \
  -t development

# Sign with production key
cargo run --bin kernel-signer \
  -i ../../target/release/polymera-kernel.bin \
  -o ../../target/release/signed-kernel.bin \
  -k prod-key.pem \
  -t production \
  -b v1.0.0
```

#### 3. Verify Signed Image
```bash
# Basic verification
cargo run --bin kernel-verifier ../../target/release/signed-kernel.bin

# Verbose verification with expected values
cargo run --bin kernel-verifier \
  -v \
  -b abc123 \
  -t development \
  -c trusted-dev-cert.pem \
  ../../target/release/signed-kernel.bin
```

### Boot Verification

#### Development Mode
```bash
# QEMU with development bypass enabled
qemu-system-x86_64 \
  -kernel target/release/signed-kernel.bin \
  -append "DEV_BYPASS=1" \
  -serial stdio

# The kernel will boot with signature verification bypassed
```

#### Production Mode
```bash
# QEMU with strict verification
qemu-system-x86_64 \
  -kernel target/release/signed-kernel.bin \
  -serial stdio

# The kernel will verify signatures and refuse to boot if invalid
```

### Runtime Attestation

#### Print Attestation Information
```bash
# From within the running kernel
sys_debug(PRINT_ATTEST, 0, 0, 0)

# Output will show:
# === Kernel Attestation Information ===
# Magic: POLY
# Version: 1
# Flags: 0x01
# Build Type: Development
# Build ID: abc123def456
# Certificate Fingerprint: 1a2b3c4d...
# Verification Count: 1
# Successful Verifications: 1
# ⚠️  DEVELOPMENT BYPASS ENABLED
```

## Key Management

### Development Keys

#### Generating Development Keys
```bash
# Generate Dilithium2 keypair
openssl genpkey -algorithm dilithium2 -out dev-key.pem

# Extract public key
openssl pkey -in dev-key.pem -pubout -out dev-cert.pem
```

#### Key Rotation
```bash
# 1. Generate new development keypair
openssl genpkey -algorithm dilithium2 -out dev-key-new.pem

# 2. Update trusted certificate in kernel
# Edit kernel/src/boot/attest.rs to include new certificate

# 3. Rebuild and sign kernel with new key
cargo build --release --package polymera-kernel
cargo run --bin kernel-signer \
  -i target/release/polymera-kernel.bin \
  -o target/release/signed-kernel.bin \
  -k dev-key-new.pem \
  -t development

# 4. Test new signed image
cargo run --bin kernel-verifier \
  -c dev-cert-new.pem \
  target/release/signed-kernel.bin
```

### Production Keys

#### Generating Production Keys
```bash
# Generate production keypair (use secure hardware if available)
openssl genpkey -algorithm dilithium2 -out prod-key.pem

# Extract public key
openssl pkey -in prod-key.pem -pubout -out prod-cert.pem

# Store private key securely (hardware security module recommended)
```

#### Production Key Rotation
```bash
# 1. Generate new production keypair
openssl genpkey -algorithm dilithium2 -out prod-key-new.pem

# 2. Update kernel with new certificate
# Edit kernel/src/boot/attest.rs

# 3. Rebuild and sign kernel
cargo build --release --package polymera-kernel
cargo run --bin kernel-signer \
  -i target/release/polymera-kernel.bin \
  -o target/release/signed-kernel.bin \
  -k prod-key-new.pem \
  -t production \
  -b v1.1.0

# 4. Verify new image
cargo run --bin kernel-verifier \
  -t production \
  -c prod-cert-new.pem \
  target/release/signed-kernel.bin
```

## CI Integration

### Automated Signing

#### GitHub Actions Workflow
```yaml
name: "Kernel Signing"
on:
  push:
    branches: [main, develop]
    paths: ['kernel/**', 'Cargo.toml', 'Cargo.lock']

jobs:
  sign-kernel:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: "Setup Rust"
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: "Build Kernel"
        run: |
          cargo build --release --package polymera-kernel
      
      - name: "Sign Kernel"
        run: |
          cd tooling/attest
          cargo run --bin kernel-signer \
            -i ../../target/release/polymera-kernel.bin \
            -o ../../target/release/signed-kernel.bin \
            -k ${{ secrets.DEV_SIGNING_KEY }} \
            -t development
      
      - name: "Verify Signature"
        run: |
          cd tooling/attest
          cargo run --bin kernel-verifier \
            -v \
            -c ${{ secrets.DEV_CERTIFICATE }} \
            ../../target/release/signed-kernel.bin
      
      - name: "Upload Artifacts"
        uses: actions/upload-artifact@v3
        with:
          name: signed-kernel
          path: target/release/signed-kernel.bin
```

### Automated Testing

#### QEMU Boot Tests
```yaml
name: "Secure Boot Testing"
on:
  pull_request:
    paths: ['kernel/**', 'tooling/attest/**']

jobs:
  test-secure-boot:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: "Setup QEMU"
        run: |
          sudo apt-get update
          sudo apt-get install -y qemu-system-x86
      
      - name: "Build and Sign Kernel"
        run: |
          cargo build --release --package polymera-kernel
          cd tooling/attest
          cargo run --bin kernel-signer \
            -i ../../target/release/polymera-kernel.bin \
            -o ../../target/release/signed-kernel.bin \
            -k dev-key.pem \
            -t development
      
      - name: "Test Boot with Dev Bypass"
        run: |
          timeout 30s qemu-system-x86_64 \
            -kernel target/release/signed-kernel.bin \
            -append "DEV_BYPASS=1" \
            -serial stdio \
            -nographic \
            -no-reboot \
            -no-shutdown | grep -q "Development bypass enabled"
      
      - name: "Test Boot without Dev Bypass"
        run: |
          timeout 30s qemu-system-x86_64 \
            -kernel target/release/signed-kernel.bin \
            -serial stdio \
            -nographic \
            -no-reboot \
            -no-shutdown | grep -q "Kernel signature verification successful"
```

## Security Considerations

### Development vs Production

#### Development Mode
- **Purpose**: Development and testing
- **Security Level**: Reduced (bypass enabled)
- **Key Management**: Local development keys
- **Use Case**: Development workflow, CI/CD testing

#### Production Mode
- **Purpose**: Production deployment
- **Security Level**: Full (no bypass)
- **Key Management**: Secure hardware, key rotation
- **Use Case**: Production systems, security-critical deployments

### Threat Model

#### Protected Against
- **Kernel Tampering**: Signature verification prevents modification
- **Image Substitution**: Build ID prevents old image reuse
- **Unauthorized Images**: Certificate validation ensures source
- **Runtime Attacks**: Early boot verification

#### Not Protected Against
- **Hardware Attacks**: Physical access to boot media
- **Firmware Attacks**: UEFI/BIOS compromise
- **Supply Chain**: Compromised build system
- **Key Compromise**: Private key exposure

### Security Best Practices

#### Key Management
- **Secure Storage**: Use hardware security modules (HSMs)
- **Key Rotation**: Regular key rotation schedule
- **Access Control**: Limit access to signing keys
- **Audit Logging**: Log all signing operations

#### Build Security
- **Build Isolation**: Isolate build environment
- **Dependency Verification**: Verify all dependencies
- **Artifact Signing**: Sign all build artifacts
- **Chain of Trust**: Establish complete trust chain

## Troubleshooting

### Common Issues

#### Signature Verification Fails
```bash
# Check key format
openssl pkey -in key.pem -text -noout

# Verify certificate matches
cargo run --bin kernel-verifier \
  -v \
  -c certificate.pem \
  signed-kernel.bin

# Check build ID consistency
git rev-parse HEAD
```

#### Boot Refuses Unsigned Image
```bash
# Enable development bypass
qemu-system-x86_64 \
  -kernel signed-kernel.bin \
  -append "DEV_BYPASS=1" \
  -serial stdio

# Check kernel logs for verification details
```

#### Build ID Mismatch
```bash
# Verify current git hash
git rev-parse --short HEAD

# Rebuild and sign with correct build ID
cargo build --release --package polymera-kernel
cargo run --bin kernel-signer \
  -i target/release/polymera-kernel.bin \
  -o target/release/signed-kernel.bin \
  -k dev-key.pem \
  -b $(git rev-parse --short HEAD)
```

### Debug Information

#### Enable Verbose Logging
```bash
# Kernel compilation with debug
RUST_LOG=debug cargo build --release --package polymera-kernel

# Verification with verbose output
cargo run --bin kernel-verifier \
  -v \
  signed-kernel.bin
```

#### Runtime Debugging
```bash
# From within kernel
sys_debug(PRINT_ATTEST, 0, 0, 0)

# Check verification statistics
# Look for verification count and success rate
```

## Future Enhancements

### Planned Features
- **Hardware Root of Trust**: TPM integration
- **Certificate Chain**: Multi-level certificate validation
- **Revocation Lists**: Key and certificate revocation
- **Secure Boot Integration**: UEFI secure boot compatibility

### Research Areas
- **Post-Quantum Migration**: Transition to new PQC standards
- **Performance Optimization**: Faster signature verification
- **Memory Protection**: Secure memory for key operations
- **Attestation Protocols**: Remote attestation capabilities

## References

### Standards
- **NIST PQC**: Post-Quantum Cryptography Standards
- **Dilithium2**: CRYSTALS-Dilithium Digital Signature Algorithm
- **Secure Boot**: UEFI Secure Boot Specification

### Tools
- **OpenSSL**: Cryptographic toolkit
- **QEMU**: System emulator for testing
- **Git**: Version control for build tracking

### Documentation
- **Kernel Development**: Rust kernel development guide
- **Cryptography**: Post-quantum cryptography primer
- **Security**: Secure boot implementation guide

---

**Status**: ✅ Implemented  
**Phase**: 2  
**Security Level**: Development Mode  
**Next Steps**: Production hardening, hardware integration
