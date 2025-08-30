# Jarvis Memory — Intent Kernel + World Model Integration

## Overview

Jarvis Memory combines the Intent Kernel v0 and World Model v0 to provide a comprehensive agent memory system. The Intent Kernel handles task planning and execution tracking, while the World Model maintains persistent knowledge about entities, relationships, and system state.

## Architecture Integration

### Intent Kernel as Memory Consumer

The Intent Kernel uses World Model snapshots to:

1. **Context Retrieval**: Access current system state during intent preview
2. **Capability Validation**: Check entity permissions and capabilities
3. **State Tracking**: Monitor changes in system entities and resources
4. **Audit Trail**: Link why-log entries to specific world model snapshots

### World Model as Memory Store

The World Model provides:

1. **Entity Registry**: Centralized entity management with typed relationships
2. **State Snapshots**: Point-in-time views of system knowledge
3. **Query Interface**: Pattern-based fact retrieval for agents
4. **Provenance Tracking**: Source attribution for all knowledge

## Data Flow

### Intent Submission Flow

```
Intent Submit → World Model Query → Context Retrieval → Plan Generation → Why-Log Entry
     ↓              ↓                    ↓                ↓              ↓
  SYS_INTENT_   get_entity_caps()    Entity State    Action Plan    WM Snapshot ID
   SUBMIT       list_devices()       Capabilities    Constraints    Timestamp
```

### Memory Update Flow

```
Fact Update → World Model Put → Snapshot Creation → Intent Context Update → Why-Log Chain
     ↓            ↓                ↓                ↓                    ↓
  External      SYS_WM_PUT      New Epoch        Latest State        Hash Link
  Event        Fact Storage     Snapshot ID      Available to       Previous Entry
```

## Integration Points

### 1. Intent Preview with World Context

```rust
// Intent Kernel preview() method
pub fn preview(&self, intent: &IntentV1) -> PlanPreviewV1 {
    // Get current world model snapshot
    let world_snapshot = self.world_model.snapshot_create();
    
    // Query relevant context
    let entity_caps = self.world_model.get_entity_caps(&intent.requester);
    let available_devices = self.world_model.list_devices();
    let last_seen = self.world_model.last_seen(&intent.target);
    
    // Generate plan with context
    let plan = self.planner.plan(intent, &entity_caps, &available_devices);
    
    // Record why-log with world model context
    self.why_log.append(WhyLogEntry {
        reason: "Intent preview with world context",
        evidence: vec![
            EvidenceV1::new("world_snapshot", &world_snapshot.0.to_string()),
            EvidenceV1::new("entity_caps", &entity_caps.len().to_string()),
            EvidenceV1::new("available_devices", &available_devices.len().to_string()),
        ],
        prev_hash: self.why_log.tail_hash(),
    });
    
    plan
}
```

### 2. Why-Log with World Model References

```rust
// Why-Log entry structure
pub struct WhyLogEntry {
    pub timestamp: u64,
    pub reason: String,
    pub evidence: Vec<EvidenceV1>,
    pub world_snapshot_id: Option<u64>,  // Links to WM snapshot
    pub prev_hash: [u8; 32],
    pub signature: Option<[u8; 64]>,
}
```

### 3. Capability Validation

```rust
// Intent capability check
pub fn validate_intent_caps(&self, intent: &IntentV1) -> Result<(), CapError> {
    let entity_caps = self.world_model.get_entity_caps(&intent.requester);
    
    for required_cap in &intent.required_caps {
        if !entity_caps.iter().any(|cap| {
            if let ValueAtom::String(cap_name) = &cap.object {
                cap_name == required_cap
            } else {
                false
            }
        }) {
            return Err(CapError::MissingCapability(required_cap.clone()));
        }
    }
    
    Ok(())
}
```

## Memory Patterns

### 1. Entity Lifecycle Tracking

```rust
// Track entity state changes
pub fn track_entity_lifecycle(&self, entity_id: EntityId, event: &str) {
    let fact = FactV1 {
        subject: entity_id,
        predicate: PredId(8), // STATUS
        object: ValueAtom::String(event.to_string()),
        ts_vclock: self.virtual_clock.now(),
        provenance: "intent_kernel".to_string(),
    };
    
    self.world_model.put_facts(&[fact]).unwrap();
}
```

### 2. Intent State Persistence

```rust
// Store intent execution state
pub fn update_intent_state(&self, intent_id: u128, state: &str) {
    let fact = FactV1 {
        subject: EntityId(intent_id),
        predicate: PredId(10), // INTENT_STATE
        object: ValueAtom::String(state.to_string()),
        ts_vclock: self.virtual_clock.now(),
        provenance: "intent_kernel".to_string(),
    };
    
    self.world_model.put_facts(&[fact]).unwrap();
}
```

### 3. Relationship Discovery

```rust
// Discover entity relationships
pub fn discover_relationships(&self, entity_id: EntityId) -> Vec<FactV1> {
    let pattern = Pattern::new().with_subject(entity_id);
    let range = Range::new();
    
    match self.world_model.query(&pattern, &range, 100, 0) {
        Ok(rows) => rows.facts,
        Err(_) => Vec::new(),
    }
}
```

## Performance Considerations

### Memory Budget Management

- **Intent Kernel**: ~2 MiB for why-logs and intent storage
- **World Model**: 8 MiB for facts and snapshots
- **Shared Budget**: Coordinated memory allocation

### Snapshot Strategy

- **Intent Preview**: Use latest snapshot for current context
- **Why-Log Storage**: Reference specific snapshot IDs
- **Memory Cleanup**: Compact old snapshots periodically

### Query Optimization

- **Pattern Caching**: Cache common query patterns
- **Index Usage**: Leverage subject→predicate and predicate→subject indexes
- **Batch Operations**: Group related fact updates

## Security and Audit

### Capability Enforcement

- **Intent Submission**: Requires `CAP_INTENT_SUBMIT`
- **World Model Access**: Requires `CAP_WM_READ`/`CAP_WM_WRITE`
- **Cross-Capability**: Intent operations may require multiple capabilities

### Audit Trail

- **Intent Events**: All intent operations logged with why-log references
- **World Model Events**: Fact updates and queries audited
- **Cross-Reference**: Why-log entries link to world model snapshots

### Data Integrity

- **Hash Chaining**: Why-log entries form tamper-evident chain
- **Snapshot Consistency**: World model snapshots provide consistent views
- **Provenance Tracking**: All facts include source attribution

## Testing Strategy

### Integration Tests

1. **Intent with World Context**
   - Submit intent requiring specific capabilities
   - Verify world model context retrieval
   - Check why-log snapshot references

2. **Memory Consistency**
   - Update world model facts
   - Verify intent preview sees changes
   - Validate snapshot isolation

3. **Performance Gates**
   - Intent preview with world model queries
   - Memory budget enforcement
   - Snapshot creation and access

### Determinism Validation

- **Identical Runs**: Same inputs produce same outputs
- **Snapshot Stability**: Consistent snapshot IDs across runs
- **Hash Consistency**: Why-log chain hashes stable

## Future Enhancements

### Phase 3 (Device Integration)
- **Device State**: Real-time device status in world model
- **Resource Tracking**: File system and network resource monitoring
- **Policy Integration**: World model facts drive policy decisions

### Phase 4 (Agent Runtime)
- **Memory Sharing**: Multiple agents share world model
- **Event Streaming**: Real-time fact updates
- **Distributed Memory**: Cross-node world model synchronization

### Phase 5 (Advanced AI)
- **Semantic Memory**: Natural language fact extraction
- **Learning Patterns**: Adaptive memory organization
- **Predictive Context**: Anticipate agent needs

## Implementation Status

✅ **Intent Kernel Integration**: World model context in intent preview
✅ **Why-Log References**: Snapshot IDs in why-log entries
✅ **Capability Validation**: Entity capability checking
✅ **Memory Patterns**: Entity lifecycle and intent state tracking
✅ **Performance Optimization**: Coordinated memory management
✅ **Security Model**: Capability enforcement and audit trails
✅ **Testing Framework**: Integration tests with CI gates
✅ **Documentation**: Complete integration guide

The Jarvis Memory system successfully integrates Intent Kernel v0 and World Model v0 to provide a comprehensive agent memory foundation.
