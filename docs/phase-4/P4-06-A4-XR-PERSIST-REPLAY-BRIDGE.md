# P4-06-A4: XR Surface – Multi-User Persistence, Recording/Replay & Cross-Metaverse Bridge

## Overview

This document describes the implementation of **P4-06-A4**, the final step in the XR surface development. This phase adds durable multi-user persistence, session recording/replay, and a cross-metaverse bridge (Godot↔WebXR/IPFS) with audit anchoring and DAO-governed publish flows.

## Key Features

### 1. Durable Multi-User Persistence
- **Per-room durable state**: Scene + avatars + policies committed as NGFS snapshots
- **Snapshot pipelines**: Automated persistence with integrity verification
- **Room state management**: Persistent room configurations and participant data
- **Policy persistence**: DAO policies stored with room state

### 2. Session Recording/Replay
- **Time-ordered input capture**: All user inputs and events recorded with timestamps
- **Deterministic replay**: Headless re-execution to regenerate exact snapshot bytes
- **Export/import logs**: Recording data can be exported and imported
- **Replay modes**: Headless (for testing) and visualization modes

### 3. Cross-Metaverse Bridge
- **Scene export**: glTF 2.0 + KHR_mesh_quantization format
- **IPFS CAR packaging**: DAG-based asset packaging for decentralized storage
- **WebXR adapter**: Bridge between Godot and WebXR environments
- **Asset management**: Texture, mesh, and animation asset handling

### 4. Governance & Security
- **DAO approval**: Publish/export operations require DAO approval
- **CapToken scope**: Operations gated by capability tokens
- **On-chain anchoring**: Optional Ethereum Geth reference anchoring
- **Audit records**: All operations logged for compliance

## Technical Implementation

### Rust Core Services

#### Persistence Manager (`services/xr/persist.rs`)
```rust
pub trait PersistenceManager {
    async fn persist_room(&self, request: &PersistRoomRequest) -> SceneResult<PersistRoomResponse>;
}

pub struct PersistRoomRequest {
    pub room_id: String,
    pub reason: String,
    pub session_id: String,
}

pub struct PersistRoomResponse {
    pub snapshot_id: String,
    pub timestamp: u64,
    pub checksum: String,
}
```

#### Recording Manager (`services/xr/record.rs`)
```rust
pub trait RecordingManager {
    async fn start_recording(&self, request: &StartRecordingRequest) -> SceneResult<StartRecordingResponse>;
    async fn stop_recording(&self, request: &StopRecordingRequest) -> SceneResult<StopRecordingResponse>;
}

pub struct StartRecordingRequest {
    pub room_id: String,
    pub session_id: String,
}

pub struct StartRecordingResponse {
    pub recording_id: String,
    pub start_time: u64,
}
```

#### Replay Manager (`services/xr/replay.rs`)
```rust
pub trait ReplayManager {
    async fn replay_recording(&self, request: &ReplayRecordingRequest) -> SceneResult<ReplayRecordingResponse>;
}

pub struct ReplayRecordingRequest {
    pub recording_id: String,
    pub mode: ReplayMode,
    pub session_id: String,
}

pub enum ReplayMode {
    Headless,
    Visualization,
}
```

#### Bridge Manager (`services/xr/bridge.rs`)
```rust
pub trait BridgeManager {
    async fn export_scene(&self, request: &ExportSceneRequest) -> SceneResult<ExportSceneResponse>;
    async fn import_scene(&self, request: &ImportSceneRequest) -> SceneResult<ImportSceneResponse>;
    async fn publish_scene(&self, request: &PublishSceneRequest) -> SceneResult<PublishSceneResponse>;
}

pub struct ExportSceneRequest {
    pub room_id: String,
    pub formats: Vec<ExportFormat>,
    pub session_id: String,
}

pub enum ExportFormat {
    Gltf,
    Car,
    Webxr,
}
```

### C++ Godot Integration

#### Persistence Module (`ui/xr/godot_modules/aetheris/src/aetheris_persist.cpp`)
- **AetherisPersist**: Room persistence operations
- **AetherisRecord**: Recording management
- **AetherisReplay**: Replay functionality
- **AetherisBridge**: Cross-metaverse bridge operations

#### Recording UI (`ui/xr/godot_app/scripts/RecordingUI.gd`)
```gdscript
extends Control

var aetheris_record: AetherisRecord

func _ready():
    aetheris_record = AetherisRecord.new()
    aetheris_record.connect("recording_started", self, "_on_recording_started")
    aetheris_record.connect("recording_stopped", self, "_on_recording_stopped")

func start_recording():
    var result = aetheris_record.start_recording("room_001")
    if result.success:
        status_label.text = "Recording started: " + result.recording_id
```

### TypeScript SDK

#### XR Bridge (`tooling/ts/xr_bridge.ts`)
```typescript
export class XRBridge {
    // Persistence operations
    async persistRoom(roomId: string, reason: string): Promise<string>
    async loadPersistentRoomState(snapshotId: string): Promise<RoomState>
    async getSnapshotMetadata(snapshotId: string): Promise<SnapshotMetadata>
    
    // Recording operations
    async startRecording(roomId: string): Promise<string>
    async stopRecording(recordingId: string): Promise<RecordingSummary>
    async exportRecording(recordingId: string): Promise<ArrayBuffer>
    
    // Replay operations
    async replayRecording(recordingId: string, mode: ReplayMode): Promise<string>
    async stopReplay(replayId: string): Promise<void>
    async verifyReplayDeterminism(recordingId: string): Promise<boolean>
    
    // Bridge operations
    async exportScene(roomId: string, formats: ExportFormat[]): Promise<string>
    async importScene(bundle: ArrayBuffer, policyMode: ImportPolicyMode): Promise<ImportResult>
    async publishScene(roomId: string, anchor: boolean): Promise<PublishResult>
}
```

### Go CLI Tool

#### New Commands (`go/tooling/xrctl/main.go`)
```bash
# Persistence commands
xrctl persist room <room_id> --reason "backup"
xrctl persist load <snapshot_id>
xrctl persist list

# Recording commands
xrctl record start <room_id>
xrctl record stop <recording_id>
xrctl record export <recording_id> --format json

# Replay commands
xrctl replay start <recording_id> --mode headless
xrctl replay stop <replay_id>
xrctl replay verify <recording_id>

# Bridge commands
xrctl bridge export <room_id> --format gltf,car
xrctl bridge import <bundle_file> --policy-mode strict
xrctl bridge publish <room_id> --anchor
```

### Python Validator

#### New Validation Tests (`tooling/python/xr_validator.py`)
```python
class XRValidator:
    async def validate_persistence_integrity(self) -> ValidationResult:
        """Validate persistence integrity and snapshot consistency"""
        
    async def validate_recording_determinism(self) -> ValidationResult:
        """Validate recording determinism and replay consistency"""
        
    async def validate_bridge_export_import(self) -> ValidationResult:
        """Validate bridge export/import functionality"""
```

## Data Formats

### NGFS Snapshot Format
```json
{
  "version": "1.0",
  "room_id": "room_001",
  "timestamp": 1640995200000,
  "checksum": "blake3_hash",
  "scene": {
    "nodes": [...],
    "physics": {...},
    "avatars": [...]
  },
  "policies": [...],
  "metadata": {...}
}
```

### Recording Log Format
```json
{
  "recording_id": "rec_001",
  "room_id": "room_001",
  "start_time": 1640995200000,
  "end_time": 1640995260000,
  "events": [
    {
      "timestamp": 1640995201000,
      "type": "input",
      "session_id": "sess_001",
      "data": {...}
    }
  ],
  "final_snapshot": {...}
}
```

### glTF Export Format
```json
{
  "asset": {"version": "2.0"},
  "scenes": [{"nodes": [0]}],
  "nodes": [
    {
      "name": "node_001",
      "translation": [1, 2, 3],
      "rotation": [0, 0, 0, 1],
      "scale": [1, 1, 1]
    }
  ],
  "meshes": [...],
  "materials": [...],
  "textures": [...],
  "extensions": {
    "KHR_mesh_quantization": {...}
  }
}
```

### IPFS CAR Format
```json
{
  "version": 1,
  "roots": ["QmRootHash"],
  "blocks": [
    {
      "cid": "QmBlockHash",
      "data": "base64_encoded_data"
    }
  ]
}
```

## Security & Governance

### Capability Tokens
- **persist:room**: Required for room persistence operations
- **record:start**: Required to start recording sessions
- **record:stop**: Required to stop recording sessions
- **replay:execute**: Required for replay operations
- **bridge:export**: Required for scene export operations
- **bridge:import**: Required for scene import operations
- **bridge:publish**: Required for scene publishing operations

### DAO Policies
```rego
package xr.policies

# Persistence policy
allow_persist {
    input.operation == "persist_room"
    input.session.capabilities[_] == "persist:room"
    input.room.public == true
}

# Recording policy
allow_record {
    input.operation == "start_recording"
    input.session.capabilities[_] == "record:start"
    input.room.participants[input.session.did]
}

# Bridge policy
allow_export {
    input.operation == "export_scene"
    input.session.capabilities[_] == "bridge:export"
    input.room.export_allowed == true
}
```

### Audit Records
```json
{
  "timestamp": 1640995200000,
  "operation": "persist_room",
  "session_id": "sess_001",
  "room_id": "room_001",
  "result": "success",
  "snapshot_id": "snap_001",
  "checksum": "blake3_hash",
  "policy_decision": "allow"
}
```

## Performance Requirements

### Latency Targets
- **Persistence operations**: p95 ≤ 200ms
- **Recording start/stop**: p95 ≤ 100ms
- **Replay execution**: p95 ≤ 500ms
- **Export operations**: p95 ≤ 2s
- **Import operations**: p95 ≤ 3s

### Throughput Targets
- **Concurrent recordings**: ≥ 10 per room
- **Export bundle size**: ≤ 100MB
- **Replay events**: ≥ 1000 events/second
- **Snapshot operations**: ≥ 100 per minute

### Determinism Requirements
- **Replay consistency**: 100% byte-for-byte identical snapshots
- **Cross-platform**: Identical results across Rust, C++, TypeScript, Go, Python
- **Version stability**: Backward compatibility for 3 major versions

## Testing & Validation

### Unit Tests
- Persistence manager mock implementations
- Recording manager event capture
- Replay manager deterministic execution
- Bridge manager export/import cycles

### Integration Tests
- End-to-end persistence workflows
- Multi-user recording scenarios
- Cross-metaverse bridge operations
- DAO policy enforcement

### Performance Tests
- Latency benchmarking
- Throughput stress testing
- Memory usage profiling
- Determinism verification

### CI/CD Pipeline
```yaml
name: P4-06-A4 XR Persistence, Recording & Bridge
on: [push, pull_request]
jobs:
  rust-persistence-service:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Test persistence service
        run: cargo test --package xr-persistence
      - name: Test recording service
        run: cargo test --package xr-recording
      - name: Test replay service
        run: cargo test --package xr-replay
      - name: Test bridge service
        run: cargo test --package xr-bridge
  
  godot-persistence-module:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Godot module
        run: cd ui/xr/godot_modules/aetheris && scons
      - name: Test persistence module
        run: cd ui/xr/godot_app && godot --headless --script test_persistence.gd
  
  typescript-bridge-sdk:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install dependencies
        run: cd tooling/ts && npm install
      - name: Test bridge SDK
        run: cd tooling/ts && npm test
  
  go-cli-tool:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build CLI tool
        run: cd go/tooling/xrctl && go build
      - name: Test CLI commands
        run: cd go/tooling/xrctl && go test
  
  python-validator:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Python dependencies
        run: pip install -r tooling/python/requirements.txt
      - name: Run validation tests
        run: cd tooling/python && python xr_validator.py --test persistence,recording,bridge
```

## Deployment & Operations

### Service Configuration
```toml
[xr.persistence]
enabled = true
snapshot_interval = "5m"
retention_days = 30
compression = "zstd"

[xr.recording]
enabled = true
max_duration = "1h"
max_events = 10000
export_formats = ["json", "cbor"]

[xr.replay]
enabled = true
max_concurrent = 5
headless_mode = true
determinism_check = true

[xr.bridge]
enabled = true
export_formats = ["gltf", "car", "webxr"]
ipfs_gateway = "https://ipfs.io/ipfs/"
ethereum_rpc = "https://mainnet.infura.io/v3/..."
```

### Monitoring & Metrics
- **Persistence metrics**: Snapshot count, size, duration
- **Recording metrics**: Active recordings, event rate, storage usage
- **Replay metrics**: Replay count, success rate, determinism score
- **Bridge metrics**: Export/import count, bundle size, success rate

### Alerting
- **Persistence failures**: Snapshot creation failures
- **Recording issues**: Storage quota exceeded
- **Replay failures**: Determinism violations
- **Bridge errors**: Export/import failures

## Future Enhancements

### Phase 5 Considerations
- **Distributed persistence**: Multi-node snapshot replication
- **Advanced replay**: Time-travel debugging, event filtering
- **Enhanced bridge**: Real-time sync between metaverses
- **AI integration**: Automated scene generation, content moderation

### Scalability Improvements
- **Sharded persistence**: Room-based sharding
- **Streaming replay**: Real-time event streaming
- **CDN integration**: Global asset distribution
- **Blockchain anchoring**: Multi-chain support

## Conclusion

P4-06-A4 completes the XR surface development with comprehensive persistence, recording/replay, and cross-metaverse bridge capabilities. The implementation provides:

- **Durable state management** with NGFS snapshots
- **Deterministic replay** for testing and debugging
- **Cross-metaverse interoperability** via glTF and IPFS
- **DAO-governed operations** with audit trails
- **Polyglot API parity** across all supported languages

This foundation enables advanced XR applications with persistent worlds, collaborative experiences, and seamless metaverse interoperability while maintaining security, performance, and determinism requirements.
