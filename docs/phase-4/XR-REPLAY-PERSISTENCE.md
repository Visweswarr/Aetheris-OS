# XR Replay & Persistence Guide

## Overview

This document provides comprehensive guidance for XR scene recording, replay, persistence, and cross-metaverse bridge operations in Aetheris OS Phase 4. It covers deterministic replay procedures, bridge export/import workflows, and troubleshooting common synchronization issues.

## Table of Contents

1. [Recording & Replay](#recording--replay)
2. [Bridge Export/Import](#bridge-exportimport)
3. [Common Desync Causes & Fixes](#common-desync-causes--fixes)
4. [Troubleshooting](#troubleshooting)
5. [Best Practices](#best-practices)

## Recording & Replay

### Basic Recording Workflow

#### Start Recording
```bash
# Start recording a room session
xrctl record start --room "demo-room" --output artifacts/demo-recording.cbor

# Start recording with specific options
xrctl record start --room "demo-room" \
  --output artifacts/demo-recording.cbor \
  --include-physics \
  --include-audio \
  --compression-level 6
```

#### Perform Actions to Record
```bash
# Spawn objects
xrctl scene spawn --room demo-room --mesh cube --pos 0,1,0 --color red
xrctl scene spawn --room demo-room --mesh sphere --pos 1,1,1 --color blue

# Move objects
xrctl scene move --room demo-room --object cube --pos 2,1,0
xrctl scene rotate --room demo-room --object sphere --rotation 0,45,0

# Interact with objects
xrctl scene interact --room demo-room --object cube --action grab
xrctl scene interact --room demo-room --object sphere --action release
```

#### Stop Recording
```bash
# Stop recording and save
xrctl record stop --room demo-room

# Stop recording with summary
xrctl record stop --room demo-room --summary artifacts/recording-summary.json
```

### Deterministic Replay

#### Basic Replay
```bash
# Replay recording with visualization
xrctl replay --file artifacts/demo-recording.cbor

# Replay in headless mode
xrctl replay --file artifacts/demo-recording.cbor --headless

# Replay with specific seed
xrctl replay --file artifacts/demo-recording.cbor --seed 42 --headless
```

#### Deterministic Replay Verification
```bash
# Replay multiple times and compare outputs
for i in {1..3}; do
    xrctl replay --file artifacts/demo-recording.cbor --headless --output artifacts/replay-$i.cbor
done

# Compare outputs (should be identical)
sha256sum artifacts/replay-*.cbor
```

#### Advanced Replay Options
```bash
# Replay with specific time range
xrctl replay --file artifacts/demo-recording.cbor --start-time 10.5 --end-time 30.0

# Replay with speed multiplier
xrctl replay --file artifacts/demo-recording.cbor --speed 2.0

# Replay with specific physics settings
xrctl replay --file artifacts/demo-recording.cbor --physics-tick 60 --target-fps 90
```

### Persistence Operations

#### Save Scene Snapshot
```bash
# Save current room state
xrctl persist snapshot save --room demo-room --output artifacts/room-snapshot.cbor

# Save with metadata
xrctl persist snapshot save --room demo-room \
  --output artifacts/room-snapshot.cbor \
  --metadata "description=Demo room state" \
  --metadata "version=1.0" \
  --metadata "author=admin"
```

#### Load Scene Snapshot
```bash
# Load snapshot into new room
xrctl persist snapshot load --input artifacts/room-snapshot.cbor --room demo-room-restored

# Load snapshot with validation
xrctl persist snapshot load --input artifacts/room-snapshot.cbor \
  --room demo-room-restored \
  --validate \
  --strict
```

#### Snapshot Management
```bash
# List available snapshots
xrctl persist snapshot list

# Get snapshot info
xrctl persist snapshot info --input artifacts/room-snapshot.cbor

# Compare snapshots
xrctl persist snapshot compare \
  --input1 artifacts/snapshot1.cbor \
  --input2 artifacts/snapshot2.cbor
```

## Bridge Export/Import

### Export Checklist

#### Pre-Export Validation
- [ ] **Room State**: Verify room is in stable state
- [ ] **Object Count**: Confirm all objects are present
- [ ] **Physics State**: Ensure physics simulation is stable
- [ ] **Permissions**: Verify export permissions are granted
- [ ] **Storage Space**: Check available disk space
- [ ] **Network**: Ensure stable network connection

#### Export Scene to glTF
```bash
# Export scene to glTF format
xrctl bridge export --room demo-room \
  --format gltf \
  --output artifacts/demo-room.gltf \
  --include-materials \
  --include-animations \
  --include-physics

# Export with specific options
xrctl bridge export --room demo-room \
  --format gltf \
  --output artifacts/demo-room.gltf \
  --quality high \
  --texture-size 2048 \
  --compression draco
```

#### Export to IPFS CAR
```bash
# Export scene to IPFS CAR format
xrctl bridge export --room demo-room \
  --format car \
  --output artifacts/demo-room.car \
  --include-assets \
  --include-metadata

# Export with IPFS pinning
xrctl bridge export --room demo-room \
  --format car \
  --output artifacts/demo-room.car \
  --pin-to-ipfs \
  --ipfs-gateway https://ipfs.io
```

#### Export Manifest
```bash
# Generate export manifest
xrctl bridge export --room demo-room \
  --format manifest \
  --output artifacts/demo-room-manifest.json \
  --include-hashes \
  --include-dependencies
```

### Import Checklist

#### Pre-Import Validation
- [ ] **File Integrity**: Verify export files are not corrupted
- [ ] **Format Support**: Confirm format is supported
- [ ] **Permissions**: Verify import permissions are granted
- [ ] **Storage Space**: Check available disk space
- [ ] **Network**: Ensure stable network connection
- [ ] **Dependencies**: Verify all dependencies are available

#### Import from glTF
```bash
# Import scene from glTF
xrctl bridge import --input artifacts/demo-room.gltf \
  --room demo-room-imported \
  --format gltf \
  --validate \
  --strict

# Import with specific options
xrctl bridge import --input artifacts/demo-room.gltf \
  --room demo-room-imported \
  --format gltf \
  --scale 1.0 \
  --position 0,0,0 \
  --rotation 0,0,0
```

#### Import from IPFS CAR
```bash
# Import scene from IPFS CAR
xrctl bridge import --input artifacts/demo-room.car \
  --room demo-room-imported \
  --format car \
  --validate \
  --strict

# Import with IPFS gateway
xrctl bridge import --input artifacts/demo-room.car \
  --room demo-room-imported \
  --format car \
  --ipfs-gateway https://ipfs.io \
  --timeout 300
```

#### Import Validation
```bash
# Validate imported scene
xrctl bridge validate --room demo-room-imported \
  --check-geometry \
  --check-materials \
  --check-physics \
  --check-permissions

# Compare with original
xrctl bridge compare \
  --room1 demo-room \
  --room2 demo-room-imported \
  --tolerance 0.001
```

### Cross-Metaverse Bridge

#### WebXR Adapter
```bash
# Export for WebXR
xrctl bridge export --room demo-room \
  --format webxr \
  --output artifacts/demo-room-webxr.json \
  --include-pose \
  --include-interactions \
  --include-audio

# Import from WebXR
xrctl bridge import --input artifacts/demo-room-webxr.json \
  --room demo-room-webxr \
  --format webxr \
  --validate
```

#### OpenSimulator Bridge
```bash
# Export for OpenSimulator
xrctl bridge export --room demo-room \
  --format opensim \
  --output artifacts/demo-room-opensim.xml \
  --include-primitives \
  --include-scripts \
  --include-textures

# Import from OpenSimulator
xrctl bridge import --input artifacts/demo-room-opensim.xml \
  --room demo-room-opensim \
  --format opensim \
  --validate
```

## Common Desync Causes & Fixes

### Clock Source Issues

#### Problem: Inconsistent Timestamps
**Symptoms:**
- Replay events occur at wrong times
- Physics simulation desyncs
- Audio/video synchronization issues

**Causes:**
- System clock drift
- Different time zones
- NTP synchronization issues
- Virtual machine clock skew

**Solutions:**
```bash
# Check system time synchronization
ntpq -p
timedatectl status

# Force NTP synchronization
sudo ntpdate -s time.nist.gov
sudo systemctl restart ntp

# Set deterministic time source
export AETHERIS_TIME_SOURCE=monotonic
export AETHERIS_TIME_OFFSET=0
```

#### Problem: Physics Tick Desync
**Symptoms:**
- Objects move differently in replay
- Physics simulation inconsistencies
- Collision detection failures

**Causes:**
- Inconsistent physics tick rates
- Different physics engines
- Floating-point precision issues

**Solutions:**
```bash
# Set consistent physics settings
export AETHERIS_XR_PHYSICS_TICK=60
export AETHERIS_XR_PHYSICS_SUBSTEPS=4
export AETHERIS_XR_PHYSICS_GRAVITY=9.81

# Use deterministic physics
export AETHERIS_XR_PHYSICS_DETERMINISTIC=true
export AETHERIS_XR_PHYSICS_SEED=42
```

### RNG Seed Issues

#### Problem: Non-Deterministic Random Numbers
**Symptoms:**
- Different random values in replay
- Inconsistent procedural generation
- Unpredictable behavior

**Causes:**
- Unseeded random number generators
- Different RNG implementations
- Seed not properly initialized

**Solutions:**
```bash
# Set deterministic RNG seed
export AETHERIS_SEED=42
export AETHERIS_RNG_ALGORITHM=pcg32
export AETHERIS_RNG_DETERMINISTIC=true

# Verify seed consistency
xrctl debug rng-status
xrctl debug rng-test --iterations 1000
```

#### Problem: Seed Leakage
**Symptoms:**
- Predictable random sequences
- Security vulnerabilities
- Reproducible attacks

**Causes:**
- Weak seed generation
- Seed reuse
- Predictable seed sources

**Solutions:**
```bash
# Use cryptographically secure seeds
export AETHERIS_SEED_SOURCE=urandom
export AETHERIS_SEED_ENTROPY=256
export AETHERIS_SEED_ROTATION=1000

# Rotate seeds regularly
xrctl security rotate-seed --entropy 256
```

### Policy Drift Issues

#### Problem: CapToken Expiration
**Symptoms:**
- Access denied errors
- Permission failures
- Security violations

**Causes:**
- Expired CapTokens
- Changed permissions
- Policy updates

**Solutions:**
```bash
# Check CapToken status
xrctl security captoken-status --room demo-room

# Refresh CapTokens
xrctl security captoken-refresh --room demo-room

# Update policies
xrctl security policy-update --room demo-room --policy new-policy.json
```

#### Problem: DAO Policy Changes
**Symptoms:**
- Governance failures
- Policy enforcement errors
- Access control issues

**Causes:**
- DAO policy updates
- Voting result changes
- Policy conflicts

**Solutions:**
```bash
# Check DAO policy status
xrctl dao policy-status --room demo-room

# Sync DAO policies
xrctl dao policy-sync --room demo-room

# Resolve policy conflicts
xrctl dao policy-resolve --room demo-room --conflict-id conflict-123
```

### Network Synchronization Issues

#### Problem: Network Latency
**Symptoms:**
- Delayed updates
- Stuttering animations
- Desynchronized states

**Causes:**
- High network latency
- Packet loss
- Bandwidth limitations

**Solutions:**
```bash
# Check network status
xrctl network status --room demo-room

# Optimize network settings
export AETHERIS_NETWORK_BUFFER_SIZE=8192
export AETHERIS_NETWORK_TIMEOUT=5000
export AETHERIS_NETWORK_RETRY_COUNT=3

# Use local network optimization
xrctl network optimize --room demo-room --local
```

#### Problem: Packet Ordering
**Symptoms:**
- Out-of-order updates
- State inconsistencies
- Race conditions

**Causes:**
- Network packet reordering
- Multiple network paths
- Load balancing issues

**Solutions:**
```bash
# Enable packet ordering
export AETHERIS_NETWORK_ORDERING=true
export AETHERIS_NETWORK_SEQUENCE_NUMBERS=true

# Use reliable transport
xrctl network transport --room demo-room --reliable
```

## Troubleshooting

### Recording Issues

#### Problem: Recording Fails to Start
**Symptoms:**
- "Recording failed to start" error
- Permission denied errors
- Disk space issues

**Solutions:**
```bash
# Check disk space
df -h
du -sh artifacts/

# Check permissions
ls -la artifacts/
chmod 755 artifacts/

# Check room status
xrctl room status --room demo-room
```

#### Problem: Recording Corruption
**Symptoms:**
- Corrupted recording files
- Replay failures
- Checksum mismatches

**Solutions:**
```bash
# Validate recording integrity
xrctl record validate --file artifacts/demo-recording.cbor

# Repair corrupted recording
xrctl record repair --file artifacts/demo-recording.cbor --output artifacts/demo-recording-repaired.cbor

# Check file system
fsck /dev/sda1
```

### Replay Issues

#### Problem: Replay Desync
**Symptoms:**
- Objects in wrong positions
- Physics simulation differences
- Timing inconsistencies

**Solutions:**
```bash
# Check replay environment
xrctl replay debug --file artifacts/demo-recording.cbor

# Verify deterministic settings
echo $AETHERIS_DETERMINISTIC
echo $AETHERIS_SEED

# Run replay with debug output
xrctl replay --file artifacts/demo-recording.cbor --debug --verbose
```

#### Problem: Replay Performance Issues
**Symptoms:**
- Slow replay execution
- High CPU usage
- Memory leaks

**Solutions:**
```bash
# Optimize replay performance
xrctl replay --file artifacts/demo-recording.cbor --optimize --headless

# Monitor resource usage
top
htop
iostat

# Adjust replay settings
export AETHERIS_REPLAY_THREADS=4
export AETHERIS_REPLAY_MEMORY_LIMIT=1024
```

### Persistence Issues

#### Problem: Snapshot Corruption
**Symptoms:**
- Corrupted snapshot files
- Load failures
- Data integrity issues

**Solutions:**
```bash
# Validate snapshot integrity
xrctl persist snapshot validate --input artifacts/room-snapshot.cbor

# Repair corrupted snapshot
xrctl persist snapshot repair --input artifacts/room-snapshot.cbor --output artifacts/room-snapshot-repaired.cbor

# Check storage integrity
xrctl storage check --room demo-room
```

#### Problem: Snapshot Version Mismatch
**Symptoms:**
- Version compatibility errors
- Feature not supported errors
- Migration failures

**Solutions:**
```bash
# Check snapshot version
xrctl persist snapshot info --input artifacts/room-snapshot.cbor

# Migrate snapshot to current version
xrctl persist snapshot migrate --input artifacts/room-snapshot.cbor --output artifacts/room-snapshot-migrated.cbor

# Check compatibility
xrctl persist snapshot compatibility --input artifacts/room-snapshot.cbor
```

## Best Practices

### Recording Best Practices

1. **Stable Environment**
   - Ensure stable network connection
   - Use consistent hardware configuration
   - Maintain stable system resources

2. **Deterministic Settings**
   - Set consistent RNG seeds
   - Use deterministic physics settings
   - Maintain consistent time sources

3. **Quality Control**
   - Validate recordings before use
   - Test replay functionality
   - Monitor recording performance

### Replay Best Practices

1. **Environment Consistency**
   - Use identical system configuration
   - Maintain consistent environment variables
   - Ensure stable network conditions

2. **Performance Optimization**
   - Use headless mode for testing
   - Optimize resource usage
   - Monitor system performance

3. **Validation**
   - Compare replay outputs
   - Verify deterministic behavior
   - Test edge cases

### Persistence Best Practices

1. **Data Integrity**
   - Regular backup procedures
   - Checksum validation
   - Version control

2. **Performance**
   - Optimize snapshot size
   - Use compression
   - Monitor storage usage

3. **Security**
   - Encrypt sensitive data
   - Use secure storage
   - Implement access controls

### Bridge Best Practices

1. **Format Compatibility**
   - Use standard formats
   - Validate imports/exports
   - Test cross-platform compatibility

2. **Data Preservation**
   - Maintain metadata
   - Preserve relationships
   - Handle dependencies

3. **Performance**
   - Optimize file sizes
   - Use efficient compression
   - Stream large datasets

This comprehensive guide ensures reliable XR scene recording, replay, persistence, and cross-metaverse bridge operations in Aetheris OS Phase 4.
