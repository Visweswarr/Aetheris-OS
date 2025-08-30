# Event Fabric v0 - Deterministic, Low-Latency Event System

## Overview

Event Fabric v0 provides a deterministic, low-latency event system for exchanging signals between kernel subsystems, skills, and user tasks. It enables real-time communication with priority-aware delivery and strict capability checks, forming the foundation for Jarvis-style autonomous systems.

## Architecture

### Core Components

- **Topics**: Namespaced event channels with capability requirements
- **Subscriptions**: Per-task event filtering with priority lanes
- **Event Fabric**: Central routing and delivery system
- **Priority Lanes**: HI (lossless), MED (drop-oldest), LO (drop-oldest)
- **Bounded Queues**: Per-lane inboxes with configurable limits

### Design Principles

- **Deterministic**: Virtual clock timestamps, no external dependencies
- **Low-Latency**: Optimized for real-time event delivery
- **Capability-Aware**: Strict access control via capability tokens
- **Priority-Based**: Three-tier priority system with different backpressure policies
- **Bounded**: Memory and event count limits prevent unbounded growth

## Topic System

### Topic Descriptors

Topics are statically defined at build time with the following structure:

```rust
pub struct TopicDesc {
    pub name: &'static str,           // e.g., "intent.created"
    pub id: u32,                      // Stable numeric identifier
    pub caps_required_pub: u64,       // Capabilities for publishing
    pub caps_required_sub: u64,       // Capabilities for subscribing
    pub description: &'static str,    // Human-readable description
}
```

### Topic Categories

#### Intent Topics
- `intent.created` - New intent submitted
- `intent.previewed` - Intent preview completed
- `intent.completed` - Intent execution completed
- `intent.cancelled` - Intent was cancelled

#### World Model Topics
- `wm.put` - Facts added to world model
- `wm.snapshot.new` - New snapshot created
- `wm.query` - Query executed

#### Skill Topics
- `skill.loaded` - Skill loaded into runtime
- `skill.invoked` - Skill invoked for preview
- `skill.preview` - Preview result available
- `skill.unloaded` - Skill unloaded

#### System Topics
- `sys.timer` - Timer tick events
- `sys.audit` - Audit notifications

### Pattern Matching

Topics support both exact and prefix matching:

- **Exact**: `intent.created` matches only that specific topic
- **Prefix**: `intent.*` matches all intent-related topics

Prefix subscriptions require elevated capabilities (e.g., `CAP_INTENT_SUBMIT` or `CAP_WM_WRITE`).

## Priority Lanes

### Lane Characteristics

| Lane | Priority | Capacity | Policy | Use Case |
|------|----------|----------|---------|----------|
| HI   | 0        | 256      | Lossless| Critical events, intents |
| MED  | 1        | 512      | Drop-oldest | World model updates |
| LO   | 2        | 1024     | Drop-oldest | System events, logging |

### Backpressure Handling

- **HI Lane**: Lossless until capacity reached, then blocks
- **MED/LO Lanes**: Drop oldest events when full, maintain throughput
- **Audit**: All drops are logged with `EV_DROP_MED`/`EV_DROP_LO` codes

## Event Structure

### Event Header

```rust
pub struct EventHeader {
    pub topic_id: u32,      // Topic identifier
    pub ts_vclock: u64,     // Virtual clock timestamp
    pub prio: u8,           // Priority (0=HI, 1=MED, 2=LO)
    pub size: u16,          // Payload size in bytes
    pub flags: u8,          // Event flags
    pub id: u64,            // Unique event identifier
}
```

### Event Constraints

- **Maximum Payload**: 4 KiB
- **Event ID**: Globally unique, monotonically increasing
- **Timestamp**: Virtual clock counter (deterministic)
- **Flags**: Reserved for future use

## Subscription Management

### Subscription Lifecycle

1. **Create**: Task subscribes to topic pattern with priority lane
2. **Active**: Events matching pattern delivered to task's inbox
3. **Remove**: Task unsubscribes, inbox cleaned up

### Subscription Patterns

```rust
pub enum Pattern {
    Exact(String),    // e.g., "intent.created"
    Prefix(String),   // e.g., "intent.*"
}
```

### Capability Requirements

- **Basic Subscription**: No special capabilities required
- **Prefix Subscription**: Requires elevated capabilities
- **Topic-Specific**: Some topics require specific caps (e.g., `CAP_WM_READ`)

## Syscall Interface

### Event Subscription

```c
int sys_event_subscribe(const char* pattern, uint8_t lane, uint32_t task_id);
```

- **pattern**: Topic pattern (exact or prefix)
- **lane**: Priority lane (0=HI, 1=MED, 2=LO)
- **task_id**: Target task identifier
- **Returns**: Subscription handle or error code

### Event Publishing

```c
int sys_event_publish(const char* topic, uint8_t priority, 
                     const void* payload, uint16_t payload_len);
```

- **topic**: Topic name to publish to
- **priority**: Event priority (0=HI, 1=MED, 2=LO)
- **payload**: Event data
- **payload_len**: Data length (≤ 4 KiB)
- **Returns**: Number of subscribers or error code

### Event Polling

```c
int sys_event_poll(uint16_t max_events, uint32_t timeout_ms, uint32_t task_id);
```

- **max_events**: Maximum events to return (≤ 64)
- **timeout_ms**: Poll timeout (not implemented in v0)
- **task_id**: Source task identifier
- **Returns**: Number of events returned or error code

### Event Acknowledgment

```c
int sys_event_ack(uint64_t event_id, uint32_t task_id);
```

- **event_id**: Event identifier to acknowledge
- **task_id**: Task identifier
- **Returns**: 0 on success or error code

## Integration Points

### Intent Kernel Integration

The Intent Kernel automatically publishes events for key operations:

```rust
// Intent submitted
EventKernel::publish_intent_event("created", intent_id, metadata)?;

// Intent previewed
EventKernel::publish_intent_event("previewed", intent_id, metadata)?;

// Intent completed
EventKernel::publish_intent_event("completed", intent_id, metadata)?;
```

### World Model Integration

The World Model publishes events for data changes:

```rust
// Facts added
EventKernel::publish_wm_event("put", entity_id, count)?;

// Snapshot created
EventKernel::publish_wm_event("snapshot.new", entity_id, 1)?;
```

### Skill Runtime Integration

Skills can publish events during execution:

```rust
// Skill loaded
EventKernel::publish_skill_event("loaded", skill_id, metadata)?;

// Skill preview result
EventKernel::publish_skill_event("preview", skill_id, metadata)?;
```

## Performance Targets

### Latency Targets

- **p95 Latency**: ≤ 400µs (APIC), ≤ 550µs (HPET)
- **Average Latency**: ≤ 200µs
- **Throughput**: ≥ 100k events/sec (HI lane)

### Memory Budgets

- **Per-Task Inbox**: 256 KiB default
- **HI Lane**: 64 KiB
- **MED Lane**: 128 KiB
- **LO Lane**: 64 KiB

### Queue Capacities

- **HI Lane**: 256 events
- **MED Lane**: 512 events
- **LO Lane**: 1024 events

## Security Model

### Capability Checks

- **Publishing**: Requires `CAP_EVENT_PUB`
- **Subscribing**: Requires `CAP_EVENT_SUB`
- **Topic-Specific**: Additional caps may be required per topic

### Access Control

- **Prefix Subscriptions**: Elevated capabilities required
- **Topic Validation**: All topics must be pre-defined
- **Payload Limits**: Maximum 4 KiB per event

### Audit Trail

All Event Fabric operations generate audit events:

- `EV_SUB_OK` / `EV_SUB_DENY`
- `EV_PUB_OK` / `EV_PUB_DENY`
- `EV_POLL_OK` / `EV_POLL_DENY`
- `EV_DROP_MED` / `EV_DROP_LO`

## Configuration

### Build-Time Configuration

Topics and capabilities are defined statically at build time:

```rust
pub static TOPICS: &[TopicDesc] = &[
    TopicDesc {
        name: "intent.created",
        id: 1,
        caps_required_pub: CAP_INTENT_SUBMIT,
        caps_required_sub: CAP_INTENT_QUERY,
        description: "New intent submitted to the system",
    },
    // ... more topics
];
```

### Runtime Configuration

- **Inbox Budgets**: Configurable per task
- **Queue Capacities**: Adjustable per lane
- **Drop Policies**: Configurable backpressure handling

## Testing Strategy

### Test Categories

1. **Capability Tests**: Verify access control enforcement
2. **Routing Tests**: Test event delivery to subscribers
3. **Backpressure Tests**: Verify drop-oldest behavior
4. **Performance Tests**: Measure latency and throughput
5. **Integration Tests**: Test with Intent, WM, and Skills

### Performance Metrics

Tests output JSON metrics for CI analysis:

```json
{"test":"event_latency_benchmark","events":50000,"p95_us":350,"avg_us":180}
{"test":"lane_performance_comparison","hi_events_per_sec":120000,"med_events_per_sec":80000,"lo_events_per_sec":60000}
```

## Future Enhancements

### Planned Features

- **Timeout Support**: Implement poll timeouts
- **Event Filtering**: Advanced subscription filters
- **Event Persistence**: Optional event storage
- **Multi-Cast**: Efficient delivery to multiple subscribers
- **Event Replay**: Historical event access

### Performance Improvements

- **Lock-Free Queues**: Reduce contention
- **Batch Processing**: Optimize bulk operations
- **Memory Pooling**: Reduce allocation overhead
- **SIMD Operations**: Vectorized event processing

## Troubleshooting

### Common Issues

1. **High Latency**: Check queue depths and drop rates
2. **Memory Usage**: Monitor inbox budgets and event sizes
3. **Capability Errors**: Verify required capabilities are present
4. **Event Loss**: Check lane capacities and drop policies

### Debug Information

- **Fabric Stats**: Overall system statistics
- **Task Stats**: Per-task inbox information
- **Audit Logs**: Security and access control events
- **Performance Metrics**: Latency and throughput data

## Conclusion

Event Fabric v0 provides a robust foundation for real-time event communication in Polymera OS. Its deterministic design, priority-aware delivery, and strict capability controls enable secure, high-performance event processing for autonomous systems.

The system balances performance with safety, providing lossless delivery for critical events while maintaining throughput for lower-priority operations. Integration with Intent Kernel, World Model, and Skill Runtime creates a cohesive event-driven architecture for Jarvis-style autonomy.
