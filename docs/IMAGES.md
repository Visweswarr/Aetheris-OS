# Polymera OS System Image Layout

This document describes the system image layout, A/B slot system, atomic rollback mechanisms, and signed manifest schema for Polymera OS.

## 🏗️ Overview

Polymera OS uses a robust system image layout with the following key features:

- **A/B Slot System**: Dual boot partitions for seamless updates
- **Atomic Rollback**: Safe rollback to previous versions
- **Signed Manifests**: Cryptographic verification of image integrity
- **Rollback Protection**: Prevention of downgrade attacks
- **Partition Management**: Flexible partition layout and management

## 🔀 A/B Slot System

### Slot Architecture

The A/B slot system provides two identical sets of partitions:

```
┌─────────────────────────────────────────────────────────────┐
│                    Device Storage                          │
├─────────────────────────────────────────────────────────────┤
│  Slot A                    │  Slot B                      │
│  ┌─────────┬─────────────┐ │  ┌─────────┬─────────────┐   │
│  │ Boot    │ System      │ │  │ Boot    │ System      │   │
│  │ Recovery│ Data        │ │  │ Recovery│ Data        │   │
│  │ Vendor  │ User        │ │  │ Vendor  │ User        │   │
│  └─────────┴─────────────┘ │  └─────────┴─────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Slot States

Each slot can be in one of several states:

- **Active**: Currently booted slot
- **Successful**: Slot that has booted successfully
- **Failed**: Slot that has failed to boot
- **Unbootable**: Slot marked as unusable

### Slot Priority

Slots are prioritized based on:
1. **Success Status**: Successful slots have higher priority
2. **Slot Priority**: Slot A has higher priority than Slot B
3. **Boot Attempts**: Slots with fewer failed attempts preferred

## 🔄 Atomic Rollback

### Rollback Process

1. **Update Installation**: New image installed to inactive slot
2. **Verification**: Image integrity and signature verified
3. **Slot Activation**: Inactive slot marked as active
4. **Boot Attempt**: System attempts to boot from new slot
5. **Success/Failure Handling**: Slot marked as successful or failed
6. **Rollback Decision**: Automatic rollback if new slot fails

### Rollback Protection

Rollback protection prevents downgrade attacks:

```rust
pub struct RollbackProtection {
    pub min_version: String,           // Minimum allowed version
    pub max_version: Option<String>,   // Maximum allowed version
    pub anti_rollback_version: u64,    // Anti-rollback counter
    pub enabled: bool,                 // Protection enabled
}
```

### Anti-Rollback Version

The anti-rollback version is a monotonically increasing counter that prevents downgrades:

- **Increment**: Increased with each successful update
- **Verification**: Checked during boot and update processes
- **Storage**: Stored in secure, tamper-resistant location

## 🔐 Signed Manifest Schema

### Manifest Structure

The PolyImage manifest is a JSON document that describes the complete system image:

```json
{
  "$schema": "https://polymera-os.org/schemas/manifest-1.0.0.json",
  "version": "1.0.0",
  "image_id": "polymera-os-v1.0.0",
  "description": "Polymera OS v1.0.0 System Image",
  "image_type": "system",
  "architecture": "x86_64",
  "platform": "generic",
  "build_timestamp": 1640995200,
  "build_host": "build-server",
  "build_user": "build-user",
  "source_revision": "abc123def456",
  "source_branch": "main",
  "slots": [...],
  "partitions": [...],
  "rollback_protection": {...},
  "signature": {...}
}
```

### Slot Information

```json
{
  "slots": [
    {
      "slot": "A",
      "priority": 2,
      "successful": true,
      "active": true,
      "last_updated": 1640995200,
      "boot_attempts": 0,
      "max_boot_attempts": 3
    },
    {
      "slot": "B",
      "priority": 1,
      "successful": false,
      "active": false,
      "last_updated": 1640995200,
      "boot_attempts": 0,
      "max_boot_attempts": 3
    }
  ]
}
```

### Partition Information

```json
{
  "partitions": [
    {
      "name": "system",
      "image_type": "system",
      "path": "/system.img",
      "size": 1073741824,
      "hash_algorithm": "sha256",
      "hash": "abc123def456...",
      "read_only": true,
      "mount_point": "/system",
      "flags": ["verified", "compressed"]
    }
  ]
}
```

### Rollback Protection

```json
{
  "rollback_protection": {
    "min_version": "1.0.0",
    "max_version": null,
    "anti_rollback_version": 5,
    "enabled": true
  }
}
```

### Digital Signature

```json
{
  "signature": {
    "algorithm": "ed25519",
    "key_id": "polymera-os-key-1",
    "value": "base64-encoded-signature",
    "certificate_chain": null,
    "timestamp": 1640995200
  }
}
```

## 🛠️ Image Creation

### Using mkimage Tool

The `mkimage` tool creates complete system images:

```bash
# Basic image creation
./mkimage \
  --output ./output \
  --image-id "polymera-os-v1.0.0" \
  --description "Polymera OS v1.0.0" \
  --source ./source \
  --architecture x86_64 \
  --platform generic

# With rollback protection
./mkimage \
  --output ./output \
  --image-id "polymera-os-v1.0.0" \
  --description "Polymera OS v1.0.0" \
  --source ./source \
  --rollback \
  --min-version "1.0.0"

# With digital signature
./mkimage \
  --output ./output \
  --image-id "polymera-os-v1.0.0" \
  --description "Polymera OS v1.0.0" \
  --source ./source \
  --sign \
  --private-key ./private.key \
  --certificate ./certificate.pem
```

### Source Directory Structure

```
source/
├── system/          # System partition files
├── boot.img         # Bootloader image
├── recovery.img     # Recovery image
├── vendor.img       # Vendor partition
└── data/            # Data partition files
```

### Output Structure

```
output/
├── manifest.json        # Image manifest
├── build_summary.txt    # Build summary
├── system.img           # System partition
├── boot.img            # Bootloader
├── recovery.img        # Recovery
├── vendor.img          # Vendor partition
└── data.tar.gz         # Data partition archive
```

## 🔍 Image Verification

### Manifest Validation

```rust
let manifest = PolyImageManifest::from_file("manifest.json")?;
let validation = manifest.validate();

if validation.valid {
    println!("✅ Manifest is valid");
} else {
    println!("❌ Manifest validation failed:");
    for error in &validation.errors {
        println!("   - {}", error);
    }
}
```

### Signature Verification

```rust
// Verify digital signature
let signature_valid = verify_signature(&manifest)?;

// Verify hash integrity
let hash_valid = verify_partition_hashes(&manifest)?;

// Check rollback protection
let rollback_valid = check_rollback_protection(&manifest)?;
```

### Partition Verification

```rust
for partition in &manifest.partitions {
    let file_path = format!("{}/{}", output_dir, partition.name);
    let calculated_hash = calculate_file_hash(&file_path)?;
    
    if calculated_hash == partition.hash {
        println!("✅ Partition {} verified", partition.name);
    } else {
        println!("❌ Partition {} hash mismatch", partition.name);
    }
}
```

## 🚀 Update Process

### 1. Download Update

```bash
# Download new image
wget https://updates.polymera-os.org/v1.1.0/image.zip
unzip image.zip
```

### 2. Verify Update

```bash
# Verify manifest signature
./verify_image --manifest manifest.json --public-key public.pem

# Verify partition integrity
./verify_image --manifest manifest.json --verify-partitions
```

### 3. Install Update

```bash
# Install to inactive slot
./install_update --manifest manifest.json --slot B

# Mark slot as active
./activate_slot --slot B
```

### 4. Boot and Verify

```bash
# Reboot to new slot
reboot

# Check slot status
./slot_status
```

### 5. Success/Failure Handling

```bash
# Mark slot as successful (if boot succeeds)
./mark_slot_successful --slot B

# Mark slot as failed (if boot fails)
./mark_slot_failed --slot B

# Rollback to previous slot (automatic)
./rollback --reason "boot_failure"
```

## 🔒 Security Features

### Cryptographic Verification

- **Hash Algorithms**: SHA-256, SHA-512, Blake3
- **Signature Algorithms**: Ed25519, ECDSA P-256, RSA-2048/4096
- **Certificate Chains**: X.509 certificate validation
- **Key Management**: Secure key storage and rotation

### Rollback Protection

- **Version Checking**: Minimum version enforcement
- **Anti-Rollback Counter**: Monotonic version tracking
- **Secure Storage**: Tamper-resistant version storage
- **Boot Verification**: Version checks during boot

### Integrity Protection

- **Partition Hashing**: Individual partition verification
- **Manifest Signing**: Complete manifest protection
- **Chain of Trust**: Boot chain verification
- **Secure Boot**: UEFI secure boot integration

## 📊 Monitoring and Debugging

### Slot Status Monitoring

```bash
# View slot status
./slot_status --detailed

# Monitor boot attempts
./monitor_boots --slot A

# Check rollback protection
./check_rollback --verbose
```

### Debug Information

```bash
# Generate debug report
./debug_report --output debug.log

# Verify image components
./verify_image --verbose --all

# Check partition layout
./partition_info --manifest manifest.json
```

### Log Analysis

```bash
# Analyze boot logs
./analyze_boots --log boot.log

# Check update history
./update_history --since "2024-01-01"

# Verify rollback events
./rollback_events --slot A
```

## 🧪 Testing

### Test Scenarios

1. **Normal Update**: Install and boot new image
2. **Failed Update**: Simulate boot failure and rollback
3. **Rollback Protection**: Attempt downgrade (should fail)
4. **Signature Verification**: Test with invalid signatures
5. **Hash Verification**: Test with corrupted partitions

### Test Commands

```bash
# Run all tests
cargo test

# Test specific functionality
cargo test --test manifest_tests
cargo test --test rollback_tests
cargo test --test signature_tests

# Integration tests
./tests/integration_test.sh
```

### Test Images

```bash
# Create test image
./mkimage \
  --output ./test_output \
  --image-id "test-image" \
  --description "Test Image" \
  --source ./test_source \
  --rollback \
  --min-version "0.1.0"

# Verify test image
./verify_image --manifest ./test_output/manifest.json
```

## 📚 API Reference

### Core Types

```rust
pub struct PolyImageManifest {
    pub version: String,
    pub image_id: String,
    pub slots: Vec<SlotInfo>,
    pub partitions: Vec<PartitionImage>,
    pub rollback_protection: RollbackProtection,
    pub signature: Signature,
}

pub struct SlotInfo {
    pub slot: String,
    pub priority: u8,
    pub successful: bool,
    pub active: bool,
    pub boot_attempts: u32,
}

pub struct PartitionImage {
    pub name: String,
    pub image_type: ImageType,
    pub hash: String,
    pub read_only: bool,
}
```

### Key Methods

```rust
impl PolyImageManifest {
    pub fn validate(&self) -> ManifestValidation;
    pub fn is_rollback_allowed(&self, version: &str) -> bool;
    pub fn mark_slot_successful(&mut self, slot: &str) -> bool;
    pub fn mark_slot_failed(&mut self, slot: &str) -> bool;
    pub fn get_next_boot_slot(&self) -> Option<&SlotInfo>;
}
```

## 🔧 Configuration

### Environment Variables

```bash
# PolyImage configuration
export POLYIMAGE_OUTPUT_DIR="/var/lib/polyimage"
export POLYIMAGE_KEY_DIR="/etc/polyimage/keys"
export POLYIMAGE_LOG_LEVEL="info"
export POLYIMAGE_ENABLE_ROLLBACK="true"
export POLYIMAGE_MIN_VERSION="1.0.0"
```

### Configuration Files

```toml
# /etc/polyimage/config.toml
[general]
output_dir = "/var/lib/polyimage"
log_level = "info"
enable_rollback = true

[rollback]
min_version = "1.0.0"
max_boot_attempts = 3

[signing]
algorithm = "ed25519"
key_file = "/etc/polyimage/keys/private.key"
certificate_file = "/etc/polyimage/keys/certificate.pem"

[slots]
count = 2
slot_a_priority = 2
slot_b_priority = 1
```

## 🚨 Troubleshooting

### Common Issues

#### 1. Slot Boot Failure
```bash
# Check slot status
./slot_status --slot A

# View boot logs
./view_logs --slot A --type boot

# Reset slot
./reset_slot --slot A
```

#### 2. Rollback Protection Error
```bash
# Check current version
./version_info

# Verify rollback settings
./check_rollback --verbose

# Update rollback configuration
./update_rollback --min-version "1.0.0"
```

#### 3. Signature Verification Failure
```bash
# Check public key
./key_info --public-key public.pem

# Verify certificate
./verify_certificate --cert cert.pem

# Regenerate keys
./generate_keys --algorithm ed25519
```

### Debug Commands

```bash
# Generate debug report
./debug_report --output debug.log

# Check image integrity
./verify_image --verbose --all

# Analyze partition layout
./partition_analysis --manifest manifest.json
```

## 📖 Additional Resources

### Documentation
- [PolyImage API Reference](../services/polyimage/)
- [Update Process Guide](./UPDATES.md)
- [Security Best Practices](./SECURITY.md)
- [Troubleshooting Guide](./TROUBLESHOOTING.md)

### Tools
- [mkimage](../services/polyimage/mkimage.rs): Image creation tool
- [verify_image](../tools/verify_image.rs): Image verification tool
- [slot_manager](../tools/slot_manager.rs): Slot management tool
- [rollback_manager](../tools/rollback_manager.rs): Rollback management tool

### Examples
- [Basic Image Creation](../examples/basic_image/)
- [Rollback Protection](../examples/rollback_protection/)
- [Signature Verification](../examples/signature_verification/)
- [A/B Slot Management](../examples/slot_management/)

---

**Happy Imaging! 🎉**

For questions or issues, please refer to the troubleshooting section or create an issue in the repository.
