# Skill Runtime v0 — WASI Skills, Manifests, and Capability Bridge

## Overview

Skill Runtime v0 provides a minimal, deterministic runtime for loading and executing WASI skills (WebAssembly modules) in preview-only mode with strict quotas and capability validation. This system enables Jarvis-like autonomy by allowing skills to propose actions through a secure, sandboxed execution environment.

## Architecture

### Core Components

1. **Skill Manifest System** - CBOR-encoded skill declarations with capability requests and quotas
2. **Capability Broker** - Maps manifest requests to preview-scoped CapTokens v2
3. **WASI Host Environment** - Restricted hostcalls for preview-only execution
4. **WASM Execution Engine** - Instruction metering and memory limits
5. **Skill Registry** - Lifecycle management and statistics

### System Design

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Space    │    │   Skill Runtime │    │   Kernel Core   │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Skill WASM │ │◄──►│ │ WASI Host  │ │    │ │ Capability │ │
│ │   Module   │ │    │ │ Environment │ │    │ │   Store    │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │  Manifest  │ │◄──►│ │   Broker    │ │◄──►│ │   Policy    │ │
│ │   (CBOR)   │ │    │ │             │ │    │ │   Engine    │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Skill Manifest Schema

### SkillManifestV1

```rust
pub struct SkillManifestV1 {
    pub name: String,                    // Skill name (max 64 chars)
    pub version: u32,                    // Version number
    pub hostcalls: Vec<HostcallId>,      // Requested hostcalls
    pub requested_caps: Vec<CapRef>,     // Required capabilities
    pub mem_limit_bytes: u32,           // Memory limit (max 32 MiB)
    pub time_slice_ms: u32,             // Time slice (max 1000ms)
    pub deterministic: bool,             // Must be true
    pub entry_point: String,            // WASM entry point
}
```

### Hostcall Types

| ID | Name | Description | Parameters |
|----|------|-------------|------------|
| 1 | `wm_query_readonly` | Query World Model (read-only) | input_ptr, input_len, output_ptr, output_len |
| 2 | `emit_plan_action` | Emit plan action | action_ptr, action_len |
| 3 | `emit_evidence` | Emit evidence | evidence_ptr, evidence_len |
| 4 | `log_debug` | Rate-limited debug logging | log_ptr, log_len |
| 5 | `rng_deterministic` | Deterministic RNG | output_ptr, output_len |

### Limits and Constraints

- **Manifest Size**: ≤ 32 KiB
- **WASM Size**: ≤ 32 MiB
- **Memory Limit**: ≤ 32 MiB per skill
- **Time Slice**: ≤ 1000ms per invocation
- **Hostcalls**: ≤ 16 per skill
- **Capabilities**: ≤ 32 per skill
- **Input Size**: ≤ 64 KiB per invocation

## Capability Broker

### Policy Validation

The broker validates skill manifests against security policies:

1. **Resource Limits** - Memory and time constraints
2. **Determinism** - Non-deterministic skills prohibited
3. **Hostcall Safety** - Only allowed hostcalls permitted
4. **Capability Scope** - Reasonable capability requests

### Capability Mapping

```rust
pub struct BrokerSession {
    pub skill_id: u64,
    pub caps: Vec<CapTokenV2>,
    pub preview_only: bool,
}
```

- **Preview Scope**: All capabilities marked with `PREVIEW_SCOPE_FLAG`
- **Skill Scope**: All capabilities marked with `SKILL_SCOPE_FLAG`
- **Zeroization**: Capabilities automatically zeroized on session drop

## WASI Host Environment

### Restricted Execution

- **No File System Access** - Skills cannot read/write files
- **No Network Access** - Skills cannot make network calls
- **No Real Clock** - Only virtual clock timestamps
- **No External Entropy** - Deterministic RNG only

### Hostcall Implementation

```rust
pub struct HostcallContext {
    pub skill_id: u64,
    pub plan_actions: Vec<ActionV1>,
    pub evidence: Vec<EvidenceV1>,
    pub world_model_queries: Vec<WorldModelQuery>,
    pub log_count: AtomicU64,
    pub last_log_reset: AtomicU64,
}
```

### Rate Limiting

- **Debug Logs**: 100 logs per second per skill
- **Hostcalls**: Bounded by instruction and time quotas
- **Memory Access**: Bounded by skill memory limits

## WASM Execution Engine

### Instruction Metering

- **Max Instructions**: 1,000,000 per invocation
- **Instruction Counter**: Atomic increment per opcode
- **Quota Enforcement**: Hard limits with immediate termination

### Memory Management

- **Initial Memory**: 64 KiB
- **Memory Growth**: Bounded by skill manifest limit
- **Memory Zeroization**: Automatic cleanup on termination

### Supported Opcodes

| Opcode | Name | Description |
|--------|------|-------------|
| 0x00 | `unreachable` | Unreachable instruction |
| 0x01 | `nop` | No operation |
| 0x02 | `block` | Block structure |
| 0x03 | `loop` | Loop structure |
| 0x04 | `if` | Conditional execution |
| 0x05 | `else` | Else branch |
| 0x0B | `end` | End block/loop/if |
| 0x20 | `local.get` | Get local variable |
| 0x21 | `local.set` | Set local variable |
| 0x41 | `i32.const` | 32-bit integer constant |
| 0x6A | `i32.add` | 32-bit integer addition |
| 0xFC | `call_host` | Hostcall invocation |

## Skill Registry

### Lifecycle Management

```rust
pub struct SkillRegistry {
    pub next_skill_id: AtomicU64,
    pub loaded_skills: Mutex<Vec<LoadedSkill>>,
    pub broker: CapabilityBroker,
    pub counters: SkillCounters,
}
```

### Statistics and Monitoring

```rust
pub struct SkillStats {
    pub skills_loaded: u64,
    pub skills_unloaded: u64,
    pub skills_invoked: u64,
    pub skills_failed: u64,
    pub total_instructions: u64,
    pub total_memory_bytes: u64,
    pub total_execution_time_us: u64,
}
```

## Syscall Interface

### SYS_SKILL_LOAD

Load a skill with manifest and WASM module.

**Parameters:**
- `manifest_ptr`: Pointer to CBOR manifest
- `manifest_len`: Length of manifest
- `wasm_ptr`: Pointer to WASM bytes
- `wasm_len`: Length of WASM module

**Returns:**
- `skill_handle`: Unique identifier for loaded skill

**Capabilities Required:**
- `CAP_SKILL_LOAD`

### SYS_SKILL_INVOKE_PREVIEW

Execute a skill in preview mode.

**Parameters:**
- `handle`: Skill handle from load
- `input_ptr`: Pointer to input data
- `input_len`: Length of input data

**Returns:**
- `0` on success

**Capabilities Required:**
- `CAP_SKILL_INVOKE`

### SYS_SKILL_UNLOAD

Unload a skill and release resources.

**Parameters:**
- `handle`: Skill handle to unload

**Returns:**
- `0` on success

**Capabilities Required:**
- `CAP_SKILL_LOAD`

### SYS_SKILL_STATUS

Get information about a loaded skill.

**Parameters:**
- `handle`: Skill handle
- `info_ptr`: Pointer to output buffer
- `info_len`: Length of output buffer

**Returns:**
- Number of bytes written

**Capabilities Required:**
- `CAP_SKILL_QUERY`

### SYS_SKILL_LIST

List all loaded skills.

**Parameters:**
- `list_ptr`: Pointer to output buffer
- `list_len`: Length of output buffer

**Returns:**
- Number of bytes written

**Capabilities Required:**
- `CAP_SKILL_QUERY`

### SYS_SKILL_STATS

Get skill runtime statistics.

**Parameters:**
- `stats_ptr`: Pointer to output buffer
- `stats_len`: Length of output buffer

**Returns:**
- Number of bytes written

**Capabilities Required:**
- `CAP_SKILL_QUERY`

## Security Model

### Capability Enforcement

- **Load/Unload**: Requires `CAP_SKILL_LOAD`
- **Invoke**: Requires `CAP_SKILL_INVOKE`
- **Query**: Requires `CAP_SKILL_QUERY`
- **Preview Scope**: All skills run in preview-only mode

### Policy Gates

- **Resource Limits**: Hard caps on memory and time
- **Hostcall Validation**: Only permitted hostcalls allowed
- **Determinism**: Non-deterministic execution prohibited
- **Capability Scope**: Reasonable capability requests only

### Audit Trail

All skill operations generate audit events:

- **SKILL_LOAD_OK/DENY**: Skill loading success/failure
- **SKILL_INVOKE_OK/QUOTA**: Skill execution success/quota exceeded
- **SKILL_UNLOAD_OK/DENY**: Skill unloading success/failure
- **SKILL_STATUS_OK/NOT_FOUND**: Status retrieval success/failure

## Performance Targets

### Throughput

- **Skill Loading**: ≤ 100ms per skill
- **Skill Invocation**: ≤ 250ms per invocation
- **Memory Usage**: ≤ 32 MiB per skill
- **Instruction Count**: ≤ 1M instructions per invocation

### Resource Efficiency

- **Memory Overhead**: ≤ 10% per skill
- **CPU Overhead**: ≤ 5% per invocation
- **Context Switch**: ≤ 1ms per hostcall
- **Registry Lookup**: ≤ 100μs per operation

## Determinism Rules

### Execution Guarantees

1. **Identical Input**: Same input produces identical output
2. **No External State**: Skills cannot access external resources
3. **Virtual Time**: Only kernel virtual clock timestamps
4. **Deterministic RNG**: Seeded by skill ID and invocation count

### Quota Enforcement

1. **Instruction Limits**: Hard cap on instruction count
2. **Memory Limits**: Bounded memory allocation
3. **Time Limits**: Maximum execution time per invocation
4. **Hostcall Limits**: Rate limiting and bounds checking

## Integration Points

### Intent Kernel

- **Preview Integration**: Skills can emit plan actions and evidence
- **World Model Access**: Read-only access to entity/relation data
- **Capability Validation**: Skills inherit caller capabilities

### World Model

- **Query Interface**: Skills can query facts and entities
- **Snapshot Access**: Consistent read views during execution
- **Evidence Collection**: Skills can emit structured evidence

### Policy System

- **Manifest Validation**: Policy-driven skill approval
- **Capability Brokering**: Dynamic capability assignment
- **Resource Limits**: Policy-enforced quotas and constraints

## Testing Strategy

### Test Categories

1. **Manifest Schema**: CBOR round-trip and validation
2. **Load and Invoke**: Skill lifecycle and execution
3. **Quota Enforcement**: Memory, time, and instruction limits
4. **Capability Broker**: Policy validation and capability mapping
5. **Determinism**: Identical input/output verification

### Test Coverage

- **Unit Tests**: Individual component testing
- **Integration Tests**: End-to-end skill execution
- **Performance Tests**: Quota and timing validation
- **Security Tests**: Capability and policy enforcement

## Future Enhancements

### Phase 3 Features

1. **Dynamic Linking**: Support for shared skill libraries
2. **Skill Composition**: Multi-skill execution pipelines
3. **Advanced Hostcalls**: Device and filesystem access
4. **Skill Marketplace**: Distributed skill distribution

### Advanced Capabilities

1. **Skill Signing**: Cryptographic skill verification
2. **Skill Updates**: Hot-swappable skill versions
3. **Skill Dependencies**: Inter-skill communication
4. **Skill Monitoring**: Real-time execution metrics

## Implementation Status

### Completed Components

- ✅ Skill Manifest Schema (CBOR + validation)
- ✅ Capability Broker (Policy + capability mapping)
- ✅ WASI Host Environment (Restricted hostcalls)
- ✅ WASM Execution Engine (Metering + limits)
- ✅ Skill Registry (Lifecycle + statistics)
- ✅ Syscall Interface (Load/Invoke/Unload/Query)
- ✅ Security Model (Capabilities + policy)
- ✅ Test Suite (Comprehensive coverage)

### Current Status

**Skill Runtime v0 is fully implemented and provides:**

1. **Deterministic Execution** with strict quotas and limits
2. **Preview-Only Mode** for safe skill evaluation
3. **Capability Enforcement** with policy validation
4. **Comprehensive Auditing** of all operations
5. **Performance Monitoring** with detailed metrics
6. **Security Isolation** through sandboxed execution

The system is ready for production use as the foundation for Jarvis skills core functionality.
