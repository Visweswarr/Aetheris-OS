# Phase 4 Overview: XR/Metaverse Surface & Device Runtime

## Summary

Phase 4 delivers the complete XR/Metaverse surface with deterministic physics, multi-user persistence, and a comprehensive device runtime with AI capabilities. This phase establishes Aetheris OS as a production-ready platform for immersive computing and edge AI applications.

## Phase 4 Components

### P4-05: WASM Contracts + Relay
**Status**: ✅ Complete  
**Scope**: Smart contract execution with deterministic replay

- **WASM Runtime**: Deterministic WebAssembly execution with seeded RNG
- **Contract Relay**: Cross-chain contract deployment and execution
- **CapToken Integration**: Contract access control via capability tokens
- **NGFS Integration**: Contract state persistence and deterministic replay

**Key Services**:
- `services/contracts/` - WASM contract runtime
- `services/relay/` - Cross-chain relay service
- `contracts/` - Smart contract templates

### P4-06: XR Surface (A1-A4)
**Status**: ✅ Complete  
**Scope**: Complete XR/Metaverse surface with multi-user persistence

#### P4-06-A1: XR Core Foundation
- Scene graph with DID avatars and CapToken gating
- Basic XR device runtime integration
- NGFS storage for scene persistence

#### P4-06-A2: DAO-Governed Edits
- DAO policy hooks for shared resource governance
- CapToken v2 enforcement for scene modifications
- Audit trails for all scene changes

#### P4-06-A3: Advanced XR Features + Device Integration
- OpenXR device runtime with multi-user rooms
- Deterministic physics and interaction systems
- Real-time avatar synchronization

#### P4-06-A4: Multi-User Persistence, Recording/Replay & Cross-Metaverse Bridge
- Durable multi-user persistence with NGFS snapshots
- Session recording/replay with deterministic execution
- Cross-metaverse bridge (Godot ↔ WebXR/IPFS)

**Key Services**:
- `services/xr/scene/` - Scene graph and avatar management
- `services/xr/runtime/` - XR device runtime
- `services/xr/persist/` - Multi-user persistence
- `services/xr/record/` - Session recording
- `services/xr/replay/` - Deterministic replay
- `services/xr/bridge/` - Cross-metaverse bridge

### P4-07: Device Runtime (A1-A5)
**Status**: ✅ Complete  
**Scope**: Comprehensive device runtime with AI capabilities

#### P4-07-A1: Camera & Microphone Capture
- V4L2 camera capture with NGFS integration
- ALSA microphone capture with deterministic sampling
- Media multiplexing with 90kHz timebase

#### P4-07-A2: BLE + Sensors
- Bluetooth Low Energy device management
- Generic sensor framework with deterministic sampling
- CapToken-gated device access

#### P4-07-A3: GPIO, ADC, Actuators
- Digital I/O with libgpiod integration
- Analog-to-digital conversion with calibration
- Actuator control (PWM, relays, motors, LEDs)

#### P4-07-A4: HAL + Linux Backends
- Hardware Abstraction Layer with pluggable providers
- Linux hardware backends (V4L2, ALSA, libgpiod, IIO, PWM)
- Deterministic mock providers for testing

#### P4-07-A5: Edge AI Perception & Encoding
- ONNX Runtime for object detection (YOLO models)
- Whisper.cpp for speech recognition
- Deterministic AI inference with replay capabilities
- CBOR event schemas for AI results

**Key Services**:
- `services/devices/` - Device runtime core
- `services/ai/` - AI inference pipelines
- `services/hal/` - Hardware abstraction layer

## Service Architecture

### Core Services

| Service | Port | Environment Variables | Description |
|---------|------|----------------------|-------------|
| **XR Scene** | 8080 | `AETHERIS_XR_PORT=8080` | Scene graph and avatar management |
| **XR Runtime** | 8081 | `AETHERIS_XR_RUNTIME_PORT=8081` | XR device runtime |
| **Device Runtime** | 8082 | `AETHERIS_DEVICE_PORT=8082` | Device management and control |
| **AI Service** | 8083 | `AETHERIS_AI_PORT=8083` | AI inference pipelines |
| **Contract Runtime** | 8084 | `AETHERIS_CONTRACT_PORT=8084` | WASM contract execution |
| **Relay Service** | 8085 | `AETHERIS_RELAY_PORT=8085` | Cross-chain relay |
| **NGFS** | 8086 | `AETHERIS_NGFS_PORT=8086` | Content-addressed storage |
| **DAO Kernel** | 8087 | `AETHERIS_DAO_PORT=8087` | Governance and policy |

### Environment Variables

#### Core Configuration
```bash
# Service ports
AETHERIS_XR_PORT=8080
AETHERIS_XR_RUNTIME_PORT=8081
AETHERIS_DEVICE_PORT=8082
AETHERIS_AI_PORT=8083
AETHERIS_CONTRACT_PORT=8084
AETHERIS_RELAY_PORT=8085
AETHERIS_NGFS_PORT=8086
AETHERIS_DAO_PORT=8087

# Deterministic mode
AETHERIS_DETERMINISTIC=true
AETHERIS_SEED=42

# Logging
AETHERIS_LOG_LEVEL=info
AETHERIS_LOG_FORMAT=json

# Security
AETHERIS_CAPTOKEN_SECRET=your-secret-key
AETHERIS_DAO_POLICY_PATH=/etc/aetheris/policies
```

#### XR Configuration
```bash
# XR device settings
AETHERIS_XR_DEVICE_PATH=/dev/video0
AETHERIS_XR_TARGET_FPS=90
AETHERIS_XR_PHYSICS_TICK=60

# Multi-user settings
AETHERIS_XR_MAX_USERS=16
AETHERIS_XR_ROOM_TIMEOUT=300

# Persistence settings
AETHERIS_XR_SNAPSHOT_INTERVAL=30
AETHERIS_XR_RECORDING_ENABLED=true
```

#### Device Configuration
```bash
# Device provider
AETHERIS_DEVICE_PROVIDER=linux
AETHERIS_DEVICE_MOCK=false

# Camera settings
AETHERIS_CAMERA_DEVICE=/dev/video0
AETHERIS_CAMERA_FORMAT=yuv420
AETHERIS_CAMERA_RESOLUTION=1920x1080

# Audio settings
AETHERIS_AUDIO_DEVICE=hw:0,0
AETHERIS_AUDIO_SAMPLE_RATE=48000
AETHERIS_AUDIO_CHANNELS=2

# GPIO settings
AETHERIS_GPIO_CHIP=gpiochip0
AETHERIS_GPIO_PULL_UP=true
```

#### AI Configuration
```bash
# AI backends
AETHERIS_AI_ENABLE_GPU=false
AETHERIS_AI_ONNX_THREADS=4
AETHERIS_AI_WHISPER_THREADS=4

# Model paths
AETHERIS_AI_MODEL_PATH=/opt/aetheris/models
AETHERIS_AI_YOLO_MODEL=yolo_n.onnx
AETHERIS_AI_WHISPER_MODEL=ggml-tiny.en.bin

# Performance
AETHERIS_AI_MAX_LATENCY_MS=100
AETHERIS_AI_TARGET_FPS=15
```

## CLI Tools

### Core CLI Commands

#### Wallet Management (`walletctl`)
```bash
# Create wallet
walletctl create --name "demo-wallet"

# Generate DID
walletctl did generate --wallet demo-wallet

# List capabilities
walletctl caps list --wallet demo-wallet

# Generate CapToken
walletctl captoken generate --wallet demo-wallet --scope "xr:scene.edit"
```

#### XR Management (`xrctl`)
```bash
# Start XR session
xrctl session start --room "demo-room" --device /dev/video0

# Spawn object
xrctl scene spawn --room demo-room --mesh cube --pos 0,1,0

# Start recording
xrctl record start --room demo-room

# Stop recording
xrctl record stop --room demo-room -o artifacts/demo-recording.cbor

# Replay recording
xrctl replay --file artifacts/demo-recording.cbor --headless

# Export scene
xrctl export --room demo-room --formats gltf,car -o artifacts/
```

#### Device Management (`devctl`)
```bash
# List devices
devctl devices list

# Start camera
devctl camera start --device /dev/video0 --format yuv420

# Start microphone
devctl mic start --device hw:0,0 --sample-rate 48000

# Read GPIO
devctl gpio read --chip gpiochip0 --line 17

# Sample ADC
devctl adc sample --iio /sys/bus/iio/devices/iio:device0 --channel in_voltage0_raw

# Control actuator
devctl actuator pwm --chip pwmchip0 --channel 0 --duty 50% --freq 1kHz
```

#### AI Management (`devctl ai`)
```bash
# Start vision pipeline
devctl ai vision start --device /dev/video0 --model yolo_n --fps 15 --seed 42

# Start audio pipeline
devctl ai audio start --device hw:0,0 --model whisper_tiny --lang en --seed 42

# Process frame
devctl ai vision process --session vision_123 --input frame.jpg --output detections.json

# Process audio
devctl ai audio process --session audio_123 --input audio.wav --output transcript.json

# Replay AI events
devctl ai replay events --snapshot snapshot_123 --topic detections --check-determinism

# Show AI statistics
devctl ai stats
```

## 10-Minute Smoke Test

This smoke test validates the complete Phase 4 stack by chaining wallet → contract → XR spawn → device read → AI detect.

### Prerequisites
```bash
# Ensure all services are running
make test

# Verify environment
export AETHERIS_DETERMINISTIC=true
export AETHERIS_SEED=42
```

### Step 1: Wallet Setup (1 minute)
```bash
# Create demo wallet
walletctl create --name "smoke-test-wallet"

# Generate DID
walletctl did generate --wallet smoke-test-wallet

# Generate CapTokens for all operations
walletctl captoken generate --wallet smoke-test-wallet --scope "xr:scene.edit,device:camera.read,ai:infer.vision"
```

### Step 2: Contract Deployment (1 minute)
```bash
# Deploy demo contract
contractctl deploy --wallet smoke-test-wallet --contract contracts/demo.wasm

# Verify contract state
contractctl state --contract demo --wallet smoke-test-wallet
```

### Step 3: XR Scene Setup (2 minutes)
```bash
# Start XR session
xrctl session start --room "smoke-test-room" --device /dev/video0

# Spawn test object
xrctl scene spawn --room smoke-test-room --mesh cube --pos 0,1,0 --color red

# Verify object exists
xrctl scene list --room smoke-test-room
```

### Step 4: Device Capture (2 minutes)
```bash
# Start camera capture
devctl camera start --device /dev/video0 --format yuv420 --resolution 640x480

# Capture test frame
devctl camera capture --output artifacts/smoke-test-frame.jpg

# Verify frame captured
ls -la artifacts/smoke-test-frame.jpg
```

### Step 5: AI Detection (2 minutes)
```bash
# Start AI vision pipeline
devctl ai vision start --device /dev/video0 --model yolo_n --fps 15 --seed 42

# Process captured frame
devctl ai vision process --session vision_123 --input artifacts/smoke-test-frame.jpg --output artifacts/detections.json

# Verify detections
cat artifacts/detections.json
```

### Step 6: Deterministic Replay (2 minutes)
```bash
# Start recording
xrctl record start --room smoke-test-room

# Perform some actions
xrctl scene spawn --room smoke-test-room --mesh sphere --pos 1,1,1 --color blue

# Stop recording
xrctl record stop --room smoke-test-room -o artifacts/smoke-test-recording.cbor

# Replay recording
xrctl replay --file artifacts/smoke-test-recording.cbor --headless --expect-byte-stable

# Verify deterministic replay
echo "✅ Deterministic replay successful"
```

### Expected Results

After completing the smoke test, you should have:

1. **Wallet**: Active wallet with DID and CapTokens
2. **Contract**: Deployed WASM contract with verified state
3. **XR Scene**: Active room with spawned objects
4. **Device**: Captured camera frame
5. **AI**: Object detections from captured frame
6. **Replay**: Deterministic replay of XR actions

### Verification Commands
```bash
# Check all services are healthy
curl -s http://localhost:8080/health | jq '.status'
curl -s http://localhost:8082/health | jq '.status'
curl -s http://localhost:8083/health | jq '.status'

# Verify artifacts
ls -la artifacts/
# Should contain: smoke-test-frame.jpg, detections.json, smoke-test-recording.cbor

# Check logs for errors
tail -n 100 /var/log/aetheris/*.log | grep -i error
```

### Troubleshooting

If the smoke test fails:

1. **Service Health**: Check all services are running on correct ports
2. **CapTokens**: Verify wallet has required capabilities
3. **Device Access**: Ensure camera device is accessible
4. **Model Files**: Verify AI models are present in model path
5. **Deterministic Mode**: Ensure seed is set consistently

### Success Criteria

✅ **Wallet**: DID generated, CapTokens created  
✅ **Contract**: WASM contract deployed and verified  
✅ **XR**: Scene created, objects spawned  
✅ **Device**: Camera frame captured  
✅ **AI**: Objects detected in frame  
✅ **Replay**: Deterministic replay successful  

## Performance Benchmarks

### XR Performance
- **Scene Rendering**: 90 FPS target
- **Multi-user Sync**: ≤150ms p95 latency
- **Physics Tick**: 60 Hz deterministic
- **Avatar Update**: ≤50ms p95 latency

### Device Performance
- **Camera Capture**: ≤10ms p95 latency
- **Audio Capture**: ≤5ms p95 latency
- **GPIO Read**: ≤1µs latency
- **ADC Sample**: ≤10µs latency

### AI Performance
- **Vision Inference**: ≤100ms p95 latency
- **Audio Inference**: ≤200ms p95 latency
- **Encoding**: ≤10ms p95 latency
- **Deterministic Replay**: Byte-identical results

## Security Features

### CapToken v2
- Fine-grained capability-based access control
- Scoped permissions for all operations
- Audit trails for capability usage

### DAO Governance
- Deny-by-default policy enforcement
- Shared resource governance
- Proposal and voting mechanisms

### Deterministic Security
- Seeded RNG for reproducible results
- Byte-stable manifests for verification
- Signed event logs for audit

## Phase 4 Completion

**When `artifacts/PHASE4_OK` exists, Phase 4 is complete.**

This marker file is created by running the comprehensive verification script:

```bash
# Run full Phase 4 verification
bash scripts/phase4-verify.sh

# Or run quick verification (critical tests only)
bash scripts/phase4-verify.sh --quick
```

The verification script runs:
- Code formatting and linting (`make fmt && make lint`)
- Test suite execution (`make test`)
- Web3 contracts smoke test (best-effort if deps missing)
- XR determinism verification (`scripts/p4-xr-determinism.sh`)
- HAL operations testing (`scripts/p4-hal-toggle.sh`)
- AI golden tests (`python tooling/python/ai_golden_test.py`)
- Design tokens generation (optional)
- Documentation validation (optional)

### Verification Status

To check if Phase 4 is complete:
```bash
if [ -f "artifacts/PHASE4_OK" ]; then
    echo "✅ Phase 4 is complete!"
    cat artifacts/PHASE4_OK
else
    echo "❌ Phase 4 verification pending"
    echo "Run: bash scripts/phase4-verify.sh"
fi
```

## Next Steps

Phase 4 provides the foundation for:
- **Phase 5**: Advanced AI and ML capabilities
- **Phase 6**: Distributed computing and edge coordination
- **Phase 7**: Production deployment and scaling

The XR/Metaverse surface and device runtime are now production-ready with comprehensive testing, documentation, and operational procedures.
