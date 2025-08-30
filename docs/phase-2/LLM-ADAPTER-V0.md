# LLM Adapter v0

## Overview

The LLM Adapter v0 provides a kernel-level facade for Large Language Model interactions, enabling deterministic, policy-gated access to LLM capabilities while maintaining security and resource constraints. This system serves as the foundation for AI-assisted operations in Polymera OS.

## Architecture

### Core Components

1. **Session Manager**: Manages individual LLM sessions with quota enforcement and redaction
2. **Backend Registry**: Pluggable backend system for different LLM providers
3. **Chunk Ring Buffer**: Bounded streaming buffer for completion chunks
4. **Quota Enforcement**: Token, byte, and time slice limits per session
5. **Policy Integration**: All requests pass through policy evaluation gates

### Backend Types

- **NullBackend**: Deterministic echo backend for testing and CI (default in release)
- **DevLocalBackend**: Development backend for user-space LLM integration (feature-gated)

## Schemas

### PromptV1

```rust
pub struct PromptV1 {
    pub messages: Vec<MessageV1>,
    pub tools: Option<Vec<ToolDefV1>>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}
```

### MessageV1

```rust
pub struct MessageV1 {
    pub role: u8,  // ROLE_SYSTEM, ROLE_USER, ROLE_ASSISTANT, ROLE_TOOL
    pub content: String,
}
```

### CompletionChunkV1

```rust
pub struct CompletionChunkV1 {
    pub seq: u32,
    pub token: Option<String>,
    pub finish: Option<FinishReason>,
    pub tool_calls: Option<Vec<ToolCallV1>>,
}
```

### AdapterConfigV1

```rust
pub struct AdapterConfigV1 {
    pub backend: u8,
    pub quotas: QuotaLimitsV1,
    pub redactions: Vec<RedactionRuleV1>,
}
```

## Capabilities

### Required Capabilities

- `CAP_LLM_USE`: Basic LLM operations (session open/close, send/receive)
- `CAP_LLM_DEV`: Development backend access (requires policy allow)

### Capability Scopes

- **Session Management**: Open, close, and query session information
- **Prompt Processing**: Send prompts with redaction and quota enforcement
- **Chunk Reception**: Receive streaming completion chunks
- **Backend Selection**: Choose between available backends

## Syscalls

### SYS_LLM_SESSION_OPEN

Opens a new LLM session with specified configuration.

**Input**: `LlmSessionOpenRequest`
- `config_ptr`: Pointer to `AdapterConfigV1`
- `config_len`: Size of configuration

**Output**: Session handle on success, negative errno on failure

**Audit**: `LlmSessionOpen` on success, `LlmSessionDeny` on failure

### SYS_LLM_SEND

Sends a prompt to the LLM backend.

**Input**: `LlmSendRequest`
- `session_handle`: Active session identifier
- `prompt_ptr`: Pointer to `PromptV1`
- `prompt_len`: Size of prompt

**Output**: 0 on success, negative errno on failure

**Audit**: `LlmSendOk` on success, `LlmSendDeny` on failure

### SYS_LLM_RECV

Receives completion chunks from the LLM backend.

**Input**: `LlmRecvRequest`
- `session_handle`: Active session identifier
- `max_chunks`: Maximum number of chunks to receive

**Output**: `LlmRecvResponse` with chunks and metadata

**Audit**: `LlmRecvOk` on success, `LlmRecvDeny` on failure

### SYS_LLM_CLOSE

Closes an LLM session and releases resources.

**Input**: `LlmCloseRequest`
- `session_handle`: Session identifier to close

**Output**: 0 on success, negative errno on failure

**Audit**: `LlmSessionClose`

## Quota Enforcement

### Token Per Minute (TPM)

- **Default**: 1000 tokens/minute
- **Enforcement**: Per-session counter with time-based reset
- **Violation**: Returns `ETIME` and audits quota breach

### Bytes Per Minute (BPM)

- **Default**: 10,000 bytes/minute
- **Enforcement**: Per-session byte counter with time-based reset
- **Violation**: Returns `ETIME` and audits quota breach

### Time Slice (TS)

- **Default**: 2 seconds per request
- **Enforcement**: Per-request time limit
- **Violation**: Returns `ETIME` and audits quota breach

## Redaction System

### Redaction Rules

```rust
pub struct RedactionRuleV1 {
    pub pattern: String,  // Pattern to match (e.g., "email", "phone", "key")
    pub replacement: String,  // Replacement text
}
```

### Supported Patterns

- **Email**: Replaces `@` with `[EMAIL]`
- **Phone**: Replaces phone number patterns
- **API Keys**: Replaces key-like strings
- **Custom**: User-defined patterns

### Redaction Process

1. **Pre-flight**: Applied before sending to backend
2. **Deterministic**: Same input always produces same redacted output
3. **Idempotent**: Multiple applications don't change result

## Event Fabric Integration

### Published Events

- **`llm.token`**: Token chunks (HI priority lane)
- **`llm.toolcall`**: Tool call events (MED priority lane)
- **`llm.finish`**: Completion events (MED priority lane)
- **`llm.session.open`**: Session creation (LO priority lane)
- **`llm.session.close`**: Session termination (LO priority lane)

### Event Structure

```rust
pub struct LlmEvent {
    pub topic: String,
    pub session_id: u64,
    pub timestamp: u64,
    pub data: Vec<u8>,
}
```

## Backend Implementation

### NullBackend

**Purpose**: Deterministic testing and CI environment

**Behavior**:
- Echoes input text word-by-word
- Generates predictable completion chunks
- No external network calls
- Deterministic across runs

**Use Cases**:
- Unit testing
- CI/CD pipelines
- Development without external dependencies

### DevLocalBackend

**Purpose**: Development integration with user-space LLM

**Features**:
- PolyBus communication with user process
- Policy-gated access
- Feature flag protection (`cfg(feature="dev-llm")`)
- No direct network access from kernel

**Configuration**:
- Requires `CAP_LLM_DEV` capability
- Policy evaluation before use
- User-space process handles actual LLM calls

## Security Model

### Policy Integration

All LLM operations pass through policy evaluation:

1. **Input Validation**: Schema and size checks
2. **Capability Verification**: Required capabilities present
3. **Policy Evaluation**: OPA→WASM policy engine
4. **Redaction Application**: PII and sensitive data removal
5. **Audit Logging**: Complete operation audit trail

### Resource Isolation

- **Per-session Memory**: Bounded ring buffer (8 MiB max)
- **Time Limits**: Strict per-request time slicing
- **No Network**: Kernel never makes external network calls
- **Deterministic**: No external entropy sources

## Performance Characteristics

### Latency Targets

- **Session Open**: < 100ms p95
- **Prompt Send**: < 50ms p95
- **Chunk Receive**: < 10ms p95
- **Session Close**: < 20ms p95

### Throughput Limits

- **Concurrent Sessions**: 100 maximum
- **Chunks per Request**: 64 maximum
- **Payload Size**: 32 KiB maximum per operation

## Configuration

### Kernel Configuration

```rust
pub struct LlmServiceConfig {
    pub max_sessions: u32,
    pub default_quotas: QuotaLimitsV1,
    pub ring_buffer_size: usize,
    pub enable_dev_backend: bool,
}
```

### Environment Variables

- `POLYMERA_LLM_MAX_SESSIONS`: Maximum concurrent sessions
- `POLYMERA_LLM_RING_SIZE`: Ring buffer size in bytes
- `POLYMERA_LLM_DEV_ENABLED`: Enable development backend

## Testing Strategy

### Unit Tests

- **Schema Stability**: CBOR round-trip and hash consistency
- **Redaction Rules**: Pattern matching and replacement
- **Quota Enforcement**: Time and resource limits
- **Backend Behavior**: Null backend determinism

### Integration Tests

- **Policy Integration**: Capability and policy enforcement
- **Event Fabric**: Event publishing and subscription
- **Session Lifecycle**: Complete session management
- **Error Handling**: Graceful failure modes

### Performance Tests

- **Latency Benchmarks**: P95 timing measurements
- **Throughput Tests**: Concurrent session handling
- **Memory Usage**: Ring buffer efficiency
- **Quota Accuracy**: Resource limit enforcement

## Future Enhancements

### Phase 3 Features

- **Model Weight Loading**: Secure model storage and loading
- **Advanced Tool Calls**: Side-effect capable tool execution
- **Streaming Responses**: Real-time chunk delivery
- **Model Switching**: Dynamic backend selection

### Phase 4 Features

- **Federated Learning**: Multi-model collaboration
- **Advanced Redaction**: ML-based content detection
- **Performance Optimization**: JIT compilation and caching
- **Distributed Inference**: Multi-node LLM execution

## Troubleshooting

### Common Issues

1. **Quota Exceeded**
   - Check session quota configuration
   - Verify time slice limits
   - Review usage patterns

2. **Redaction Failures**
   - Validate redaction rule patterns
   - Check pattern syntax
   - Verify replacement text

3. **Backend Unavailable**
   - Confirm backend feature flags
   - Check capability permissions
   - Verify policy configuration

### Debug Information

- **Audit Logs**: Complete operation audit trail
- **Service Statistics**: Usage counters and metrics
- **Event Fabric**: Real-time operation events
- **Session Info**: Detailed session state

## Examples

### Basic Session Usage

```rust
// Open session
let config = AdapterConfigV1 {
    backend: BACKEND_NULL,
    quotas: QuotaLimitsV1::default(),
    redactions: vec![],
};
let session = open_llm_session(config)?;

// Send prompt
let prompt = PromptV1 {
    messages: vec![MessageV1::user("Hello, world!")],
    tools: None,
    max_tokens: None,
    temperature: None,
};
send_llm_prompt(session, &mut prompt)?;

// Receive chunks
let chunks = receive_llm_chunks(session, 10)?;
for chunk in chunks {
    if let Some(token) = chunk.token {
        print!("{}", token);
    }
}

// Close session
close_llm_session(session)?;
```

### Redaction Configuration

```rust
let redactions = vec![
    RedactionRuleV1 {
        pattern: "email".to_string(),
        replacement: "[EMAIL]".to_string(),
    },
    RedactionRuleV1 {
        pattern: "phone".to_string(),
        replacement: "[PHONE]".to_string(),
    },
];

let config = AdapterConfigV1 {
    backend: BACKEND_NULL,
    quotas: QuotaLimitsV1::default(),
    redactions,
};
```

### Tool Integration

```rust
let tools = vec![
    ToolDefV1 {
        name: "get_weather".to_string(),
        description: "Get current weather".to_string(),
        parameters: r#"{"type":"object","properties":{"city":{"type":"string"}}}"#.to_string(),
    },
];

let prompt = PromptV1 {
    messages: vec![MessageV1::user("What's the weather like?")],
    tools: Some(tools),
    max_tokens: None,
    temperature: None,
};
```

## Conclusion

The LLM Adapter v0 provides a secure, deterministic foundation for AI-assisted operations in Polymera OS. Through careful design of quotas, redaction, and policy integration, it enables safe LLM usage while maintaining system security and performance characteristics.

The system is designed for extensibility, with clear interfaces for backend integration and future enhancements. The deterministic nature ensures consistent behavior across environments, making it suitable for both development and production use.
