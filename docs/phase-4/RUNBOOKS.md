# Phase 4 Runbooks: Operational Procedures

## Table of Contents

1. [Service Management](#service-management)
2. [Key Rotation & CapToken Management](#key-rotation--captoken-management)
3. [Deterministic Replay Procedures](#deterministic-replay-procedures)
4. [Known Failure Modes](#known-failure-modes)
5. [Quick Triage Checklists](#quick-triage-checklists)
6. [Emergency Procedures](#emergency-procedures)

## Service Management

### Service Restart Procedures

#### XR Scene Service
```bash
# Graceful restart
sudo systemctl restart aetheris-xr-scene

# Force restart (if graceful fails)
sudo systemctl stop aetheris-xr-scene
sudo pkill -f "aetheris-xr-scene"
sudo systemctl start aetheris-xr-scene

# Verify restart
curl -s http://localhost:8080/health | jq '.status'
xrctl session list
```

#### XR Runtime Service
```bash
# Graceful restart
sudo systemctl restart aetheris-xr-runtime

# Force restart
sudo systemctl stop aetheris-xr-runtime
sudo pkill -f "aetheris-xr-runtime"
sudo systemctl start aetheris-xr-runtime

# Verify restart
curl -s http://localhost:8081/health | jq '.status'
xrctl device list
```

#### Device Runtime Service
```bash
# Graceful restart
sudo systemctl restart aetheris-device

# Force restart
sudo systemctl stop aetheris-device
sudo pkill -f "aetheris-device"
sudo systemctl start aetheris-device

# Verify restart
curl -s http://localhost:8082/health | jq '.status'
devctl devices list
```

#### AI Service
```bash
# Graceful restart
sudo systemctl restart aetheris-ai

# Force restart
sudo systemctl stop aetheris-ai
sudo pkill -f "aetheris-ai"
sudo systemctl start aetheris-ai

# Verify restart
curl -s http://localhost:8083/health | jq '.status'
devctl ai stats
```

#### Contract Runtime Service
```bash
# Graceful restart
sudo systemctl restart aetheris-contract

# Force restart
sudo systemctl stop aetheris-contract
sudo pkill -f "aetheris-contract"
sudo systemctl start aetheris-contract

# Verify restart
curl -s http://localhost:8084/health | jq '.status'
contractctl list
```

#### Relay Service
```bash
# Graceful restart
sudo systemctl restart aetheris-relay

# Force restart
sudo systemctl stop aetheris-relay
sudo pkill -f "aetheris-relay"
sudo systemctl start aetheris-relay

# Verify restart
curl -s http://localhost:8085/health | jq '.status'
relayctl status
```

#### NGFS Service
```bash
# Graceful restart
sudo systemctl restart aetheris-ngfs

# Force restart
sudo systemctl stop aetheris-ngfs
sudo pkill -f "aetheris-ngfs"
sudo systemctl start aetheris-ngfs

# Verify restart
curl -s http://localhost:8086/health | jq '.status'
ngfsctl status
```

#### DAO Kernel Service
```bash
# Graceful restart
sudo systemctl restart aetheris-dao

# Force restart
sudo systemctl stop aetheris-dao
sudo pkill -f "aetheris-dao"
sudo systemctl start aetheris-dao

# Verify restart
curl -s http://localhost:8087/health | jq '.status'
daoctl status
```

### Bulk Service Operations

#### Restart All Services
```bash
# Graceful restart all services
sudo systemctl restart aetheris-xr-scene aetheris-xr-runtime aetheris-device aetheris-ai aetheris-contract aetheris-relay aetheris-ngfs aetheris-dao

# Verify all services
for port in 8080 8081 8082 8083 8084 8085 8086 8087; do
    echo "Checking port $port..."
    curl -s http://localhost:$port/health | jq '.status' || echo "Service on port $port is down"
done
```

#### Stop All Services
```bash
# Graceful stop
sudo systemctl stop aetheris-xr-scene aetheris-xr-runtime aetheris-device aetheris-ai aetheris-contract aetheris-relay aetheris-ngfs aetheris-dao

# Force stop (emergency)
sudo pkill -f "aetheris-"
```

#### Start All Services
```bash
# Start in dependency order
sudo systemctl start aetheris-ngfs
sleep 2
sudo systemctl start aetheris-dao
sleep 2
sudo systemctl start aetheris-contract
sleep 2
sudo systemctl start aetheris-relay
sleep 2
sudo systemctl start aetheris-device
sleep 2
sudo systemctl start aetheris-ai
sleep 2
sudo systemctl start aetheris-xr-runtime
sleep 2
sudo systemctl start aetheris-xr-scene
```

## Key Rotation & CapToken Management

### Wallet Key Rotation

#### Generate New Wallet Keys
```bash
# Create new wallet with new keys
walletctl create --name "new-wallet-$(date +%Y%m%d)"

# Generate new DID
walletctl did generate --wallet new-wallet-$(date +%Y%m%d)

# Export new keys for backup
walletctl export --wallet new-wallet-$(date +%Y%m%d) --output /secure/backup/new-wallet-$(date +%Y%m%d).json
```

#### Rotate Existing Wallet Keys
```bash
# Backup current wallet
walletctl export --wallet current-wallet --output /secure/backup/current-wallet-$(date +%Y%m%d).json

# Generate new keys for existing wallet
walletctl rotate --wallet current-wallet

# Verify new keys
walletctl did list --wallet current-wallet
```

### CapToken Rotation

#### Rotate CapTokens for All Services
```bash
# Generate new CapTokens for XR services
walletctl captoken generate --wallet current-wallet --scope "xr:scene.edit,xr:scene.read,xr:device.access" --expiry 30d

# Generate new CapTokens for device services
walletctl captoken generate --wallet current-wallet --scope "device:camera.read,device:mic.read,device:gpio.write" --expiry 30d

# Generate new CapTokens for AI services
walletctl captoken generate --wallet current-wallet --scope "ai:infer.vision,ai:infer.audio,ai:model.load" --expiry 30d

# Generate new CapTokens for contract services
walletctl captoken generate --wallet current-wallet --scope "contract:deploy,contract:execute,contract:read" --expiry 30d
```

#### Update Service CapTokens
```bash
# Update XR service CapTokens
sudo systemctl stop aetheris-xr-scene aetheris-xr-runtime
sudo cp /secure/new-captokens/xr-*.token /etc/aetheris/captokens/
sudo systemctl start aetheris-xr-runtime aetheris-xr-scene

# Update device service CapTokens
sudo systemctl stop aetheris-device
sudo cp /secure/new-captokens/device-*.token /etc/aetheris/captokens/
sudo systemctl start aetheris-device

# Update AI service CapTokens
sudo systemctl stop aetheris-ai
sudo cp /secure/new-captokens/ai-*.token /etc/aetheris/captokens/
sudo systemctl start aetheris-ai

# Update contract service CapTokens
sudo systemctl stop aetheris-contract
sudo cp /secure/new-captokens/contract-*.token /etc/aetheris/captokens/
sudo systemctl start aetheris-contract
```

### DAO Policy Updates

#### Update DAO Policies
```bash
# Backup current policies
sudo cp -r /etc/aetheris/policies /secure/backup/policies-$(date +%Y%m%d)

# Update policies
sudo cp /secure/new-policies/*.json /etc/aetheris/policies/

# Reload DAO service
sudo systemctl reload aetheris-dao

# Verify policy updates
daoctl policy list
```

## Deterministic Replay Procedures

### XR Scene Replay

#### Record XR Session
```bash
# Start recording
xrctl record start --room "demo-room" --output artifacts/demo-recording.cbor

# Perform actions to record
xrctl scene spawn --room demo-room --mesh cube --pos 0,1,0
xrctl scene spawn --room demo-room --mesh sphere --pos 1,1,1
xrctl scene move --room demo-room --object cube --pos 2,1,0

# Stop recording
xrctl record stop --room demo-room
```

#### Replay XR Session
```bash
# Basic replay
xrctl replay --file artifacts/demo-recording.cbor

# Headless replay (no visualization)
xrctl replay --file artifacts/demo-recording.cbor --headless

# Deterministic replay with verification
xrctl replay --file artifacts/demo-recording.cbor --headless --expect-byte-stable

# Replay with specific seed
xrctl replay --file artifacts/demo-recording.cbor --seed 42 --headless
```

#### Verify Deterministic Replay
```bash
# Run replay multiple times and compare outputs
for i in {1..3}; do
    xrctl replay --file artifacts/demo-recording.cbor --headless --output artifacts/replay-$i.cbor
done

# Compare outputs (should be identical)
sha256sum artifacts/replay-*.cbor
```

### AI Event Replay

#### Record AI Events
```bash
# Start AI recording
devctl ai record start --session vision_123 --output artifacts/ai-recording.cbor

# Perform AI operations
devctl ai vision process --session vision_123 --input frame1.jpg
devctl ai vision process --session vision_123 --input frame2.jpg
devctl ai audio process --session audio_123 --input audio1.wav

# Stop recording
devctl ai record stop --session vision_123
```

#### Replay AI Events
```bash
# Replay AI events
devctl ai replay events --snapshot ai-recording.cbor --topic detections

# Replay with deterministic verification
devctl ai replay events --snapshot ai-recording.cbor --topic detections --check-determinism

# Replay specific event types
devctl ai replay events --snapshot ai-recording.cbor --topic transcripts
devctl ai replay events --snapshot ai-recording.cbor --topic vad
```

#### Verify AI Deterministic Replay
```bash
# Run AI replay multiple times
for i in {1..3}; do
    devctl ai replay events --snapshot ai-recording.cbor --topic detections --output artifacts/ai-replay-$i.json
done

# Compare AI outputs
sha256sum artifacts/ai-replay-*.json
```

### Combined XR + AI Replay

#### Record Combined Session
```bash
# Start combined recording
xrctl record start --room "combined-room" --output artifacts/combined-recording.cbor
devctl ai record start --session combined_ai --output artifacts/combined-ai.cbor

# Perform combined actions
xrctl scene spawn --room combined-room --mesh cube --pos 0,1,0
devctl ai vision process --session combined_ai --input frame.jpg
xrctl scene move --room combined-room --object cube --pos 1,1,0
devctl ai audio process --session combined_ai --input audio.wav

# Stop recordings
xrctl record stop --room combined-room
devctl ai record stop --session combined_ai
```

#### Replay Combined Session
```bash
# Replay XR and AI in sync
xrctl replay --file artifacts/combined-recording.cbor --headless &
devctl ai replay events --snapshot artifacts/combined-ai.cbor --topic detections --check-determinism
```

## Known Failure Modes

### Service Startup Failures

#### Port Already in Use
**Symptoms**: Service fails to start with "address already in use" error
**Causes**: 
- Previous service instance not properly terminated
- Another service using the same port
- System resource exhaustion

**Resolution**:
```bash
# Find process using port
sudo netstat -tlnp | grep :8080
sudo lsof -i :8080

# Kill process
sudo kill -9 <PID>

# Restart service
sudo systemctl start aetheris-xr-scene
```

#### Missing Dependencies
**Symptoms**: Service fails to start with "module not found" or "library not found" errors
**Causes**:
- Missing system libraries
- Incomplete installation
- Version mismatches

**Resolution**:
```bash
# Check system dependencies
ldd /usr/local/bin/aetheris-xr-scene

# Install missing dependencies
sudo apt-get install -y libssl-dev libffi-dev

# Reinstall service
sudo systemctl stop aetheris-xr-scene
sudo make install
sudo systemctl start aetheris-xr-scene
```

#### Permission Denied
**Symptoms**: Service fails to start with "permission denied" errors
**Causes**:
- Incorrect file permissions
- Missing capabilities
- SELinux/AppArmor restrictions

**Resolution**:
```bash
# Check file permissions
ls -la /usr/local/bin/aetheris-xr-scene

# Fix permissions
sudo chmod +x /usr/local/bin/aetheris-xr-scene
sudo chown aetheris:aetheris /usr/local/bin/aetheris-xr-scene

# Check SELinux/AppArmor
sudo setsebool -P aetheris_enabled 1
sudo aa-complain aetheris-xr-scene
```

### Device Access Failures

#### Camera Device Not Found
**Symptoms**: Camera operations fail with "device not found" error
**Causes**:
- Camera device not connected
- Incorrect device path
- Permission issues

**Resolution**:
```bash
# List available video devices
ls -la /dev/video*

# Check device permissions
ls -la /dev/video0

# Fix permissions
sudo chmod 666 /dev/video0
sudo usermod -a -G video aetheris

# Test device access
v4l2-ctl --device=/dev/video0 --list-formats
```

#### Audio Device Not Found
**Symptoms**: Audio operations fail with "device not found" error
**Causes**:
- Audio device not connected
- Incorrect device path
- ALSA configuration issues

**Resolution**:
```bash
# List available audio devices
aplay -l
arecord -l

# Check ALSA configuration
cat /etc/asound.conf

# Test device access
arecord -D hw:0,0 -f cd -t wav -d 5 test.wav
```

#### GPIO Access Denied
**Symptoms**: GPIO operations fail with "permission denied" error
**Causes**:
- Missing GPIO permissions
- Incorrect chip configuration
- libgpiod not properly installed

**Resolution**:
```bash
# Check GPIO permissions
ls -la /dev/gpiochip*

# Fix permissions
sudo chmod 666 /dev/gpiochip0
sudo usermod -a -G gpio aetheris

# Test GPIO access
gpiodetect
gpioinfo gpiochip0
```

### AI Model Failures

#### Model Loading Failed
**Symptoms**: AI operations fail with "model loading failed" error
**Causes**:
- Model file not found
- Corrupted model file
- Insufficient memory

**Resolution**:
```bash
# Check model file
ls -la /opt/aetheris/models/yolo_n.onnx

# Verify model integrity
file /opt/aetheris/models/yolo_n.onnx

# Check available memory
free -h

# Reinstall model
sudo cp /secure/backup/models/yolo_n.onnx /opt/aetheris/models/
```

#### Inference Performance Issues
**Symptoms**: AI inference takes too long or fails
**Causes**:
- Insufficient CPU resources
- Memory pressure
- Model complexity

**Resolution**:
```bash
# Check system resources
top
htop

# Reduce model complexity
export AETHERIS_AI_YOLO_MODEL=yolo_n.onnx
export AETHERIS_AI_WHISPER_MODEL=ggml-tiny.en.bin

# Increase thread count
export AETHERIS_AI_ONNX_THREADS=8
export AETHERIS_AI_WHISPER_THREADS=8
```

### Deterministic Replay Failures

#### Hash Mismatch
**Symptoms**: Replay fails with "deterministic hash mismatch" error
**Causes**:
- Different seed values
- System state differences
- Clock synchronization issues

**Resolution**:
```bash
# Verify seed consistency
echo $AETHERIS_SEED

# Check system time
date
ntpq -p

# Replay with explicit seed
xrctl replay --file recording.cbor --seed 42 --headless
```

#### Byte Instability
**Symptoms**: Replay produces different byte outputs
**Causes**:
- Non-deterministic operations
- Random number generation
- System state dependencies

**Resolution**:
```bash
# Enable strict deterministic mode
export AETHERIS_DETERMINISTIC=true
export AETHERIS_SEED=42

# Check for non-deterministic operations
grep -r "rand()" /usr/local/bin/aetheris-*
grep -r "random()" /usr/local/bin/aetheris-*

# Replay with verification
xrctl replay --file recording.cbor --headless --expect-byte-stable
```

## Quick Triage Checklists

### Service Health Check
```bash
# Quick health check for all services
for port in 8080 8081 8082 8083 8084 8085 8086 8087; do
    echo -n "Port $port: "
    curl -s --max-time 5 http://localhost:$port/health | jq -r '.status' || echo "DOWN"
done
```

### Device Access Check
```bash
# Check device accessibility
echo "Camera devices:"
ls -la /dev/video* 2>/dev/null || echo "No camera devices"

echo "Audio devices:"
aplay -l 2>/dev/null || echo "No audio devices"

echo "GPIO chips:"
ls -la /dev/gpiochip* 2>/dev/null || echo "No GPIO devices"
```

### AI Model Check
```bash
# Check AI model availability
echo "Vision models:"
ls -la /opt/aetheris/models/*.onnx 2>/dev/null || echo "No vision models"

echo "Audio models:"
ls -la /opt/aetheris/models/*.bin 2>/dev/null || echo "No audio models"
```

### CapToken Check
```bash
# Check CapToken validity
echo "XR CapTokens:"
ls -la /etc/aetheris/captokens/xr-*.token 2>/dev/null || echo "No XR CapTokens"

echo "Device CapTokens:"
ls -la /etc/aetheris/captokens/device-*.token 2>/dev/null || echo "No device CapTokens"

echo "AI CapTokens:"
ls -la /etc/aetheris/captokens/ai-*.token 2>/dev/null || echo "No AI CapTokens"
```

### Log Analysis
```bash
# Check recent errors
echo "Recent errors in logs:"
tail -n 100 /var/log/aetheris/*.log | grep -i error | tail -n 10

echo "Recent warnings in logs:"
tail -n 100 /var/log/aetheris/*.log | grep -i warning | tail -n 10
```

## Emergency Procedures

### Complete System Reset
```bash
# Stop all services
sudo systemctl stop aetheris-xr-scene aetheris-xr-runtime aetheris-device aetheris-ai aetheris-contract aetheris-relay aetheris-ngfs aetheris-dao

# Clear all temporary data
sudo rm -rf /tmp/aetheris-*
sudo rm -rf /var/cache/aetheris/*

# Reset configuration to defaults
sudo cp /etc/aetheris/config.default /etc/aetheris/config

# Restart services in order
sudo systemctl start aetheris-ngfs
sleep 5
sudo systemctl start aetheris-dao
sleep 5
sudo systemctl start aetheris-contract
sleep 5
sudo systemctl start aetheris-relay
sleep 5
sudo systemctl start aetheris-device
sleep 5
sudo systemctl start aetheris-ai
sleep 5
sudo systemctl start aetheris-xr-runtime
sleep 5
sudo systemctl start aetheris-xr-scene
```

### Data Recovery
```bash
# Restore from backup
sudo systemctl stop aetheris-ngfs
sudo cp -r /secure/backup/ngfs-$(date -d "yesterday" +%Y%m%d) /var/lib/aetheris/ngfs
sudo systemctl start aetheris-ngfs

# Verify data integrity
ngfsctl verify
```

### Security Incident Response
```bash
# Revoke all CapTokens
sudo rm -f /etc/aetheris/captokens/*.token

# Generate new CapTokens
walletctl captoken generate --wallet emergency-wallet --scope "xr:scene.edit,device:camera.read,ai:infer.vision" --expiry 1d

# Update service configurations
sudo systemctl reload aetheris-xr-scene aetheris-xr-runtime aetheris-device aetheris-ai

# Check for unauthorized access
grep -i "unauthorized\|forbidden\|denied" /var/log/aetheris/*.log
```

### Performance Emergency
```bash
# Reduce resource usage
export AETHERIS_AI_ONNX_THREADS=2
export AETHERIS_AI_WHISPER_THREADS=2
export AETHERIS_XR_TARGET_FPS=30

# Restart services with reduced resources
sudo systemctl restart aetheris-ai aetheris-xr-scene aetheris-xr-runtime

# Monitor resource usage
htop
iotop
```

This runbook provides comprehensive operational procedures for managing Phase 4 services, handling failures, and maintaining system health. Regular practice of these procedures ensures reliable operation of the Aetheris OS platform.
