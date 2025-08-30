# World Model v0 — Deterministic Graph Store with Time Snapshots

## Overview

World Model v0 provides a deterministic, in-kernel graph store for Jarvis memory core. It enables agents and policies to maintain shared state through typed entities, relations, and facts with time-travel snapshots and a minimal query API.

## Architecture

### Core Components

1. **Schema System** (`kernel/src/world/schema.rs`)
   - Typed nodes (Entity) and edges (Relation) with fixed headers
   - Value payloads: atoms (u64, i64, f64, bool, byte-string, short-string)
   - CBOR encoding with field tags for deterministic serialization

2. **Storage Engine** (`kernel/src/world/store.rs`)
   - Append-only segment log with compacted columnar index
   - Subject→predicate and predicate→subject indexing
   - Memory budget enforcement (8 MiB configurable)

3. **Query Engine** (`kernel/src/world/query.rs`)
   - Pattern queries (spo, sp?, ?p?, range on ts_vclock)
   - Limit/offset pagination and "exists" checks
   - Plan-friendly helpers: get_entity_caps(), list_devices(), last_seen()

4. **Snapshot System**
   - Epoch-based snapshots with stable IDs
   - Consistent read-view (RCU-style)
   - Delta compression and retention policy

## Data Model

### Fact Structure

```rust
pub struct FactV1 {
    pub subject: EntityId,      // Entity identifier (u128)
    pub predicate: PredId,      // Predicate identifier (u64)
    pub object: ValueAtom,      // Typed value
    pub ts_vclock: u64,        // Virtual clock timestamp
    pub provenance: String,     // Source/provenance info
}
```

### Value Types

```rust
pub enum ValueAtom {
    U64(u64),
    I64(i64),
    F64(f64),
    Bool(bool),
    Bytes(Vec<u8>),
    String(String),
}
```

### Entity Types

- **USER**: Human or system user entities
- **DEVICE**: Hardware devices and peripherals
- **SERVICE**: Software services and daemons
- **RESOURCE**: Files, memory, network resources
- **POLICY**: Security and governance policies
- **INTENT**: User intentions and goals
- **ACTION**: Executed actions and operations
- **RELATION**: Relationship metadata

### Predicate Types

- **HAS_CAPABILITY**: Entity capability mapping
- **LOCATED_AT**: Physical or logical location
- **OWNS**: Ownership relationships
- **ACCESSES**: Resource access patterns
- **DEPENDS_ON**: Dependency relationships
- **TRUSTS**: Trust relationships
- **LAST_SEEN**: Last activity timestamp
- **STATUS**: Current entity status
- **METADATA**: Additional entity metadata
- **INTENT_STATE**: Intent execution state

## API Reference

### Core Operations

#### SYS_WM_PUT
Insert or update facts in the world model.

**Input**: CBOR-encoded array of facts
**Output**: `{ok_count, snapshot_id}`
**Capability**: `CAP_WM_WRITE`

**Behavior**:
- Validates fact schema and size limits
- Enforces memory budget (8 MiB)
- Creates new epoch snapshot
- Returns count of successfully inserted facts

**Audit**: `WM_PUT_OK` or `WM_PUT_ENOSPC`

#### SYS_WM_QUERY
Query facts using pattern matching and time ranges.

**Input**: `{pattern, ts_range?, limit, offset}`
**Output**: `{rows[], next_offset?}`
**Capability**: `CAP_WM_READ`

**Behavior**:
- Pattern matching: subject, predicate, object (wildcards supported)
- Time range filtering on ts_vclock
- Deterministic ordering: (subject asc, predicate asc, ts_vclock desc)
- Enforces limit ≤ 1024, offset ≤ 1M

**Audit**: `WM_QUERY_OK` or `WM_QUERY_FAILED`

#### SYS_WM_SNAPSHOT
Create new snapshots or open existing ones.

**Input**: `{op: "create"|"open", id?}`
**Output**: `{id, created, bytes_estimate}`
**Capability**: `CAP_WM_READ`

**Behavior**:
- "create": yields new epoch snapshot ID
- "open": validates existing ID for consistent read-view
- Returns memory usage estimate

**Audit**: `WM_SNAPSHOT_NEW`, `WM_SNAPSHOT_OPEN`, or `WM_SNAPSHOT_NOT_FOUND`

#### SYS_WM_EXPORT
Export snapshot data in CBOR format.

**Input**: `{id, subject_filter?}`
**Output**: `{cbor_blob_ptr, len}`
**Capability**: `CAP_WM_EXPORT`

**Behavior**:
- Exports snapshot slice (subject filter or full snapshot)
- CBOR encoding with schema versioning
- Audited with byte size

**Audit**: `WM_EXPORT_OK` or `WM_EXPORT_FAILED`

### Helper Functions

#### get_entity_caps(entity_id)
Retrieve all capabilities for a specific entity.

**Returns**: Vector of facts with predicate type `HAS_CAPABILITY`

#### list_devices()
List all device entities in the world model.

**Returns**: Vector of entity IDs with predicate type `LOCATED_AT`

#### last_seen(entity_id)
Get the most recent activity timestamp for an entity.

**Returns**: Option<u64> timestamp from `LAST_SEEN` predicate

## Performance Targets

### Throughput
- **Fact Insertion**: ≥ 50k facts/min in QEMU smoke
- **Query Performance**: p95 ≤ 2ms for pattern queries
- **Snapshot Creation**: p95 ≤ 800µs

### Memory Management
- **Heap Budget**: 8 MiB configurable limit
- **Segment Size**: 64 KiB maximum per segment
- **Snapshot Count**: 100 maximum snapshots
- **Query Limits**: 1024 rows per query, 1M offset maximum

## Determinism Rules

### Virtual Clock
- All timestamps use kernel virtual clock counter
- No wall-clock time dependencies
- Deterministic across identical runs

### Serialization
- CBOR encoding with sorted fields
- Field tags for stable binary representation
- Schema hash computed at build time

### Query Results
- Deterministic ordering: (subject, predicate, ts_vclock)
- No random access patterns
- Consistent pagination behavior

## Security Model

### Capability Requirements
- **CAP_WM_WRITE**: Insert/update facts
- **CAP_WM_READ**: Query facts and create snapshots
- **CAP_WM_EXPORT**: Export snapshot data

### Policy Integration
- All operations audited with structured events
- Capability validation at syscall boundary
- No network access or external dependencies

### Data Integrity
- Append-only fact storage
- Hash-linked snapshot chain
- Tamper-evident audit trail

## Integration Points

### Intent Kernel
- Preview() can read WM helpers for context
- Why-Log notes include WM snapshot ID used
- Entity capability validation

### Policy System
- WM facts drive policy decisions
- Trust relationships and access patterns
- Compliance and governance metadata

### Audit System
- Structured audit events for all operations
- Performance metrics and statistics
- Security violation detection

## Testing Strategy

### Unit Tests
- **Schema Compatibility**: CBOR round-trip, hash stability
- **Storage Operations**: Put/query, deduplication, budget enforcement
- **Query Patterns**: Pattern matching, time ranges, pagination
- **Snapshot Consistency**: Isolation, export, chain verification

### Integration Tests
- **Intent Integration**: WM helpers in intent preview
- **Performance Gates**: Throughput and latency validation
- **Determinism**: Identical results across runs

### CI Gates
- **World Model Stage**: Comprehensive test suite
- **Performance Validation**: p95 thresholds enforced
- **Schema Stability**: Generated code verification

## Future Enhancements

### Phase 3 (Device/FS Integration)
- Persistent storage backend
- Transaction support
- Advanced indexing strategies

### Phase 4 (Agent Runtime)
- Real-time fact streaming
- Event-driven updates
- Distributed world model

### Phase 5 (Advanced AI)
- Semantic fact extraction
- Knowledge graph inference
- Learning and adaptation

## Implementation Status

✅ **Core Schema**: Entity, Predicate, Value types with CBOR encoding
✅ **Storage Engine**: Segmented storage with columnar indexing
✅ **Query System**: Pattern matching and time range queries
✅ **Snapshot System**: Epoch-based snapshots with read views
✅ **Syscall Interface**: Four core operations with capability checks
✅ **Audit Integration**: Comprehensive audit codes and events
✅ **Feature Flags**: WORLD_MODEL_V0 feature bit
✅ **Test Suite**: Unit and integration tests with CI gates
✅ **Documentation**: Complete API reference and design docs

The World Model v0 system is fully implemented and provides a solid foundation for Jarvis memory core functionality.
