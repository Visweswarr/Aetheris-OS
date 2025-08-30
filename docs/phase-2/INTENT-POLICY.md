# Intent Policy System

## Overview

The Intent Policy system provides fine-grained control over intent submission and execution through OPA→WASM integration. This system allows administrators to define complex policies that can allow, deny, or mutate intents based on various criteria including process capabilities, intent content, and system state.

## Architecture

### Policy Engine

The policy engine consists of three main components:

1. **Policy Loader**: Loads OPA→WASM policy modules
2. **Policy Evaluator**: Executes policies against intent submissions
3. **Decision Engine**: Applies policy decisions and enforces requirements

### Policy Decisions

The policy system supports three types of decisions:

- **Allow**: Permit the intent to proceed unchanged
- **Deny**: Block the intent with a reason
- **Mutate Scopes**: Add required capabilities before allowing

## Policy Input

When evaluating an intent, the policy engine receives the following information:

### Intent Metadata
```json
{
  "intent_id": "u128",
  "lane": "u8",
  "scope_flags": "u64",
  "ttl_ms": "u32",
  "text_len": "u32",
  "plan_len": "u32",
  "preview_len": "u16",
  "whylog_len": "u16"
}
```

### Process Context
```json
{
  "owner_pid": "ProcessId",
  "capabilities": ["CAP_FS_READ", "CAP_IPC_SEND"],
  "process_group": "string",
  "user_id": "u32"
}
```

### System State
```json
{
  "timestamp": "u64",
  "system_load": "f64",
  "queue_utilization": {
    "rt": "f64",
    "high": "f64",
    "best": "f64"
  },
  "rate_limits": {
    "per_process": "u32",
    "per_lane": "u32",
    "global": "u32"
  }
}
```

## Policy Examples

### Basic Allow/Deny Policy

```rego
package intent.policy

default allow = false

allow {
    input.intent.ttl_ms <= 300000  # Max 5 minutes
    input.intent.text_len <= 8192  # Max 8KB text
    input.intent.lane != 0         # No RT lane access
}

allow {
    input.intent.lane == 0         # RT lane
    input.process.capabilities[_] == "CAP_SYSTEM_ADMIN"
}
```

### Scope Mutation Policy

```rego
package intent.policy

default allow = false
default mutate_scopes = []

allow {
    input.intent.text_len > 8192
    count(mutate_scopes) > 0
}

mutate_scopes = ["CAP_FS_READ"] {
    input.intent.text_len > 8192
    not input.process.capabilities[_] == "CAP_FS_READ"
}

mutate_scopes = ["CAP_IPC_SEND"] {
    input.intent.plan_len > 0
    not input.process.capabilities[_] == "CAP_IPC_SEND"
}
```

### Rate Limiting Policy

```rego
package intent.policy

default allow = false

allow {
    input.intent.lane == 0         # RT lane
    input.system.rate_limits.per_process < 10
}

allow {
    input.intent.lane == 1         # HIGH lane
    input.system.rate_limits.per_process < 50
}

allow {
    input.intent.lane == 2         # BEST lane
    input.system.rate_limits.per_process < 100
}
```

### Content-Based Policy

```rego
package intent.policy

default allow = false

allow {
    input.intent.text_len > 0
    not contains(input.intent.text, "malicious_pattern")
    input.intent.ttl_ms <= 600000
}

deny_reason = "Content contains prohibited patterns" {
    contains(input.intent.text, "malicious_pattern")
}
```

## Development Mode

When running in development mode, the policy system uses simplified rules:

### Development Restrictions
- **Restricted Scopes**: Certain scope flags are blocked
- **TTL Limits**: Maximum TTL of 5 minutes
- **Content Limits**: Text size limited to 1KB
- **Lane Restrictions**: RT lane requires elevated capabilities

### Development Overrides
```rust
// Enable development mode
policy.set_dev_mode(true);

// Development mode automatically applies restrictions
// and provides helpful error messages
```

## Policy Compilation

### OPA to WASM

Policies are compiled from Rego to WebAssembly for efficient execution:

```bash
# Compile policy
opa build -t wasm -e intent/policy/allow policy.rego

# Load into kernel
policy.load_policy(wasm_blob);
```

### Policy Validation

Before loading, policies are validated for:

- **Syntax correctness**: Valid Rego syntax
- **Decision coverage**: All paths return a decision
- **Performance bounds**: Execution time limits
- **Memory safety**: No unsafe operations

## Integration Points

### Kernel Integration

The policy system integrates with the kernel through:

```rust
// Policy evaluation during intent submission
let policy_result = self.policy.evaluate_intent(&intent, pid)?;

match policy_result {
    PolicyDecision::Allow => { /* proceed */ },
    PolicyDecision::Deny(reason) => { /* block */ },
    PolicyDecision::MutateScopes(additional) => { /* add caps */ },
}
```

### Audit Integration

All policy decisions are logged for audit:

```rust
// Log policy decisions
match policy_result {
    PolicyDecision::Allow => {
        self.audit.log_accept(pid, &intent);
    },
    PolicyDecision::Deny(reason) => {
        self.audit.log_deny(pid, &intent, &reason);
    },
    PolicyDecision::MutateScopes(additional) => {
        self.audit.log_scope_mutation(pid, &intent, additional);
    },
}
```

## Performance Characteristics

### Policy Evaluation Time
- **Simple policies**: <10μs
- **Complex policies**: <100μs
- **WASM overhead**: <50μs

### Memory Usage
- **Policy storage**: <1MB per policy
- **Runtime memory**: <100KB per evaluation
- **Cache overhead**: <10MB total

## Security Considerations

### Policy Isolation
- **Process separation**: Policies cannot access kernel memory
- **Capability limits**: Policies cannot grant arbitrary capabilities
- **Resource bounds**: Execution time and memory limits enforced

### Policy Validation
- **Input sanitization**: All inputs validated before policy execution
- **Output validation**: Policy decisions verified before application
- **Audit logging**: All decisions logged for review

## Future Enhancements

### Advanced Features
- **Machine learning policies**: Adaptive policy rules
- **Distributed policies**: Multi-node policy coordination
- **Policy versioning**: Rolling policy updates
- **Policy templates**: Reusable policy components

### Performance Improvements
- **Policy caching**: Cache frequently used decisions
- **Parallel evaluation**: Evaluate multiple intents simultaneously
- **JIT compilation**: Optimize WASM execution
- **Policy optimization**: Reduce evaluation complexity

## Troubleshooting

### Common Issues

1. **Policy Load Failures**
   - Verify WASM compilation
   - Check policy syntax
   - Validate policy size limits

2. **Evaluation Errors**
   - Review input validation
   - Check policy logic
   - Verify capability mapping

3. **Performance Issues**
   - Profile policy execution
   - Optimize policy rules
   - Consider policy caching

### Debug Tools

```bash
# Policy validation
opa check policy.rego

# Policy testing
opa test policy.rego

# Policy profiling
opa eval --profile --data policy.rego data.json
```

## Best Practices

### Policy Design
- **Keep policies simple**: Complex policies are harder to debug
- **Use clear naming**: Descriptive policy and rule names
- **Document assumptions**: Clearly document policy requirements
- **Test thoroughly**: Comprehensive testing of all policy paths

### Policy Management
- **Version control**: Track policy changes in git
- **Rollback capability**: Maintain previous policy versions
- **Monitoring**: Track policy performance and decisions
- **Regular review**: Periodically review and update policies
