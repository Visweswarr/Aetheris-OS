# Release Signing Guide

This document describes the release signing process for Aetheris OS artifacts, ensuring authenticity and integrity of distributed components.

## Table of Contents

1. [Overview](#overview)
2. [Key Management](#key-management)
3. [Signing Tools](#signing-tools)
4. [Artifacts to Sign](#artifacts-to-sign)
5. [Signing Process](#signing-process)
6. [Verification Commands](#verification-commands)
7. [CI/CD Integration](#cicd-integration)
8. [Troubleshooting](#troubleshooting)

## Overview

Aetheris OS uses cryptographic signatures to ensure the authenticity and integrity of release artifacts. All critical components are signed using either `cosign` (for container images and large binaries) or `minisign` (for smaller files and schemas).

### Signing Strategy
- **Container Images**: Signed with `cosign` using keyless signing or long-term keys
- **CLI Binaries**: Signed with `minisign` for fast verification
- **CBOR Schemas**: Signed with `minisign` for schema integrity
- **SBOM Files**: Signed with `minisign` for supply chain verification

## Key Management

### Key Storage

#### Production Keys
- **Location**: Hardware Security Modules (HSM) or secure key vaults
- **Access**: Limited to release managers and CI/CD systems
- **Rotation**: Annual rotation with 30-day overlap period
- **Backup**: Encrypted backups in geographically distributed locations

#### Development Keys
- **Location**: Local development machines with encrypted storage
- **Access**: Individual developers for testing
- **Rotation**: Quarterly rotation
- **Backup**: Encrypted local backups

### Key Types

#### Cosign Keys
```bash
# Generate cosign key pair
cosign generate-key-pair

# Output files:
# cosign.key (private key - keep secure)
# cosign.pub (public key - distribute)
```

#### Minisign Keys
```bash
# Generate minisign key pair
minisign -G -s minisign.key -p minisign.pub

# Output files:
# minisign.key (private key - keep secure)
# minisign.pub (public key - distribute)
```

### Key Distribution

#### Public Keys
- **Repository**: Stored in `keys/` directory
- **Documentation**: Listed in this file
- **Verification**: Keys are signed by project maintainers

#### Private Keys
- **CI/CD**: Stored as encrypted secrets
- **Local Development**: Encrypted with GPG or similar
- **Access Control**: Multi-factor authentication required

## Signing Tools

### Cosign
**Purpose**: Container images and large binary artifacts
**Installation**: 
```bash
# Install cosign
curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64"
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
sudo chmod +x /usr/local/bin/cosign
```

**Usage**:
```bash
# Sign container image
cosign sign --key cosign.key ghcr.io/aetheris/aetheris-os:latest

# Sign binary file
cosign sign-blob --key cosign.key --output-file artifact.sig artifact.bin
```

### Minisign
**Purpose**: Small files, schemas, and CLI binaries
**Installation**:
```bash
# Ubuntu/Debian
sudo apt-get install minisign

# macOS
brew install minisign

# From source
git clone https://github.com/jedisct1/minisign.git
cd minisign && make && sudo make install
```

**Usage**:
```bash
# Sign file
minisign -S -s minisign.key -m artifact.bin

# Output: artifact.bin.minisig
```

## Artifacts to Sign

### CLI Binaries
**Location**: `artifacts/pkg/`
**Files**:
- `netctl` (Rust binary)
- `xrctl` (Rust binary)
- `devctl` (Rust binary)
- `netctl-go` (Go binary)
- `xrctl-go` (Go binary)
- `devctl-go` (Go binary)

**Signing Method**: `minisign`
**Rationale**: Fast verification, small signature files

### CBOR Schemas
**Location**: `schemas/`
**Files**:
- `ai.vision.events.cddl`
- `ai.audio.events.cddl`
- All `.cddl` schema files

**Signing Method**: `minisign`
**Rationale**: Schema integrity critical for deterministic behavior

### Container Images
**Location**: Container registries
**Images**:
- `ghcr.io/aetheris/aetheris-os:latest`
- `ghcr.io/aetheris/aetheris-os:stable`
- `ghcr.io/aetheris/aetheris-os:v1.0.0`

**Signing Method**: `cosign`
**Rationale**: Industry standard for container signing

### SBOM Files
**Location**: `artifacts/sbom/`
**Files**:
- `sbom.spdx.json`
- `sbom.cyclonedx.json`

**Signing Method**: `minisign`
**Rationale**: Supply chain verification

### Release Archives
**Location**: Release assets
**Files**:
- `aetheris-os-v1.0.0-linux-amd64.tar.gz`
- `aetheris-os-v1.0.0-windows-amd64.zip`
- `aetheris-os-v1.0.0-macos-amd64.tar.gz`

**Signing Method**: `minisign`
**Rationale**: Distribution integrity

## Signing Process

### Pre-Release Checklist
- [ ] All tests passing
- [ ] Security scan completed
- [ ] SBOM generated and verified
- [ ] Documentation updated
- [ ] Release notes prepared

### Automated Signing (CI/CD)
```yaml
# GitHub Actions example
- name: Sign Artifacts
  run: |
    # Sign CLI binaries
    for binary in artifacts/pkg/*; do
      minisign -S -s ${{ secrets.MINISIGN_KEY }} -m "$binary"
    done
    
    # Sign schemas
    for schema in schemas/*.cddl; do
      minisign -S -s ${{ secrets.MINISIGN_KEY }} -m "$schema"
    done
    
    # Sign SBOM files
    for sbom in artifacts/sbom/*.json; do
      minisign -S -s ${{ secrets.MINISIGN_KEY }} -m "$sbom"
    done
    
    # Sign container image
    cosign sign --key ${{ secrets.COSIGN_KEY }} \
      ghcr.io/aetheris/aetheris-os:${{ github.ref_name }}
```

### Manual Signing
```bash
# Set up environment
export MINISIGN_KEY_PATH="/path/to/minisign.key"
export COSIGN_KEY_PATH="/path/to/cosign.key"

# Sign all artifacts
make sign-all

# Or sign individually
make sign-binaries
make sign-schemas
make sign-sbom
make sign-containers
```

## Verification Commands

### CLI Binaries
```bash
# Verify individual binary
minisign -V -p keys/minisign.pub -m artifacts/pkg/netctl

# Verify all binaries
for binary in artifacts/pkg/*; do
  echo "Verifying $binary..."
  minisign -V -p keys/minisign.pub -m "$binary" || echo "❌ Verification failed"
done
```

### CBOR Schemas
```bash
# Verify individual schema
minisign -V -p keys/minisign.pub -m schemas/ai.vision.events.cddl

# Verify all schemas
for schema in schemas/*.cddl; do
  echo "Verifying $schema..."
  minisign -V -p keys/minisign.pub -m "$schema" || echo "❌ Verification failed"
done
```

### Container Images
```bash
# Verify container image
cosign verify --key keys/cosign.pub ghcr.io/aetheris/aetheris-os:latest

# Verify with keyless signing
cosign verify ghcr.io/aetheris/aetheris-os:latest
```

### SBOM Files
```bash
# Verify SBOM files
for sbom in artifacts/sbom/*.json; do
  echo "Verifying $sbom..."
  minisign -V -p keys/minisign.pub -m "$sbom" || echo "❌ Verification failed"
done
```

### Release Archives
```bash
# Verify release archive
minisign -V -p keys/minisign.pub -m aetheris-os-v1.0.0-linux-amd64.tar.gz

# Extract and verify contents
tar -xzf aetheris-os-v1.0.0-linux-amd64.tar.gz
for binary in aetheris-os-v1.0.0-linux-amd64/bin/*; do
  minisign -V -p keys/minisign.pub -m "$binary"
done
```

### Comprehensive Verification
```bash
# Verify all signed artifacts
make verify-all

# Or verify by category
make verify-binaries
make verify-schemas
make verify-containers
make verify-sbom
```

## CI/CD Integration

### GitHub Actions
```yaml
name: Release Signing

on:
  release:
    types: [published]

jobs:
  sign:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install signing tools
        run: |
          # Install cosign
          curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64"
          sudo mv cosign-linux-amd64 /usr/local/bin/cosign
          sudo chmod +x /usr/local/bin/cosign
          
          # Install minisign
          sudo apt-get update && sudo apt-get install -y minisign
      
      - name: Sign artifacts
        run: make sign-all
        env:
          MINISIGN_KEY: ${{ secrets.MINISIGN_KEY }}
          COSIGN_KEY: ${{ secrets.COSIGN_KEY }}
      
      - name: Upload signed artifacts
        uses: actions/upload-artifact@v4
        with:
          name: signed-artifacts
          path: |
            artifacts/pkg/*.minisig
            schemas/*.minisig
            artifacts/sbom/*.minisig
```

### Verification in CI
```yaml
name: Verify Signatures

on:
  pull_request:
    paths:
      - 'artifacts/**'
      - 'schemas/**'

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install verification tools
        run: |
          sudo apt-get update && sudo apt-get install -y minisign
          curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64"
          sudo mv cosign-linux-amd64 /usr/local/bin/cosign
          sudo chmod +x /usr/local/bin/cosign
      
      - name: Verify signatures
        run: make verify-all
```

## Troubleshooting

### Common Issues

#### Signature Verification Failed
**Symptoms**:
- `minisign: verification failed`
- `cosign: verification failed`

**Causes**:
- Corrupted file
- Wrong public key
- Modified file after signing

**Solutions**:
```bash
# Check file integrity
sha256sum artifact.bin

# Verify with correct public key
minisign -V -p keys/minisign.pub -m artifact.bin

# Re-download and verify
wget https://releases.aetheris.io/artifact.bin
minisign -V -p keys/minisign.pub -m artifact.bin
```

#### Missing Signature Files
**Symptoms**:
- `minisign: signature file not found`
- `cosign: no signatures found`

**Causes**:
- Signature file not uploaded
- Wrong file extension
- Missing signature generation

**Solutions**:
```bash
# Check for signature files
ls -la *.minisig
ls -la *.sig

# Generate missing signatures
minisign -S -s minisign.key -m artifact.bin
cosign sign-blob --key cosign.key --output-file artifact.sig artifact.bin
```

#### Key Mismatch
**Symptoms**:
- `minisign: signature verification failed`
- `cosign: invalid signature`

**Causes**:
- Wrong public key used
- Key rotation not updated
- Corrupted key file

**Solutions**:
```bash
# Verify key fingerprint
minisign -V -p keys/minisign.pub -m artifact.bin -f

# Update public key
wget https://keys.aetheris.io/minisign.pub -O keys/minisign.pub

# Check key validity
minisign -V -p keys/minisign.pub -m artifact.bin
```

### Debug Commands

#### Verbose Verification
```bash
# Minisign with verbose output
minisign -V -p keys/minisign.pub -m artifact.bin -v

# Cosign with debug output
cosign verify --key keys/cosign.pub --verbose ghcr.io/aetheris/aetheris-os:latest
```

#### Signature Information
```bash
# Show signature details
minisign -V -p keys/minisign.pub -m artifact.bin -f

# Show cosign signature details
cosign verify --key keys/cosign.pub --output json ghcr.io/aetheris/aetheris-os:latest
```

#### Key Information
```bash
# Show minisign key details
minisign -V -p keys/minisign.pub -m /dev/null -f

# Show cosign key details
cosign public-key --key keys/cosign.key
```

## Security Best Practices

### Key Management
1. **Use Hardware Security Modules (HSM)** for production keys
2. **Rotate keys regularly** (annually for production, quarterly for development)
3. **Limit key access** to authorized personnel only
4. **Monitor key usage** for unauthorized access
5. **Backup keys securely** with encryption

### Signing Process
1. **Sign in secure environments** with minimal network access
2. **Verify signatures immediately** after generation
3. **Use multiple signing methods** for critical artifacts
4. **Document signing procedures** and maintain audit logs
5. **Test verification processes** regularly

### Distribution
1. **Distribute public keys** through multiple channels
2. **Provide clear verification instructions** for users
3. **Monitor for signature verification failures**
4. **Respond quickly** to security incidents
5. **Maintain signature compatibility** across versions

This comprehensive signing guide ensures the security and integrity of Aetheris OS releases while providing clear procedures for both maintainers and users.
