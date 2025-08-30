# Policy Guardrail v0

## Overview

Policy Guardrail v0 provides a deterministic, in-kernel policy evaluation system that ensures all agent and skill actions are checked before any side effects occur. The system integrates with Intent Kernel, World Model, Skills, and Event Fabric to provide comprehensive security and compliance controls.

## Architecture

### Core Components

1. **Policy Schema** - Stable, versioned schemas for policy inputs, outputs, and decisions
2. **WASM Policy Engine** - OPA→WASM interpreter for deterministic policy evaluation
3. **Simulation Engine** - Produces PlanDiff from PlanPreview without side effects
4. **Integration Layer** - Connects policy evaluation with kernel subsystems

### System Flow

```
Intent/Preview → Simulation Engine → PlanDiff → Policy Engine → Decision + Redactions
     ↓
Event Fabric ("policy.simulate") + Why-Log Integration
```

## Policy Schemas

### PolicyInputV1

```rust
pub struct PolicyInputV1 {
    pub intent: IntentV1,           // Intent being evaluated
    pub preview: PlanPreviewV1,     // Plan preview from intent
    pub wm_snapshot: u64,           // World Model snapshot reference
    pub caps: Vec<CapRef>,          // Available capabilities
    pub features: u64,              // Kernel feature bits
}
```

### PolicyDecisionV1

```rust
pub struct PolicyDecisionV1 {
    pub allow: bool,                    // Whether action is allowed
    pub reasons: Vec<String>,           // Human-readable reasons
    pub redactions: Vec<PathSpec>,      // Paths to redact from plan
}
```

### PlanDiffV1

```rust
pub struct PlanDiffV1 {
    pub adds: Vec<ActionV1>,        // Actions to add
    pub removes: Vec<u32>,          // Actions to remove (by index)
    pub edits: Vec<EditSpec>,       // Actions to edit
}
```

## WASM Policy Engine

### Features

- **Deterministic Execution** - No network calls, wall-clock dependencies, or external I/O
- **Instruction Metering** - Tracks WASM instruction execution for quota enforcement
- **Memory Limits** - Configurable memory bounds (default: 32 KiB)
- **Hash Verification** - Bundle integrity checking via Blake3 hashes

### Policy Bundle Loading

```rust
// Dev/test mode only
pub fn load_bundle(&mut self, wasm_data: &[u8], expected_hash: [u8; 32]) -> Result<(), &'static str>
```

### Policy Evaluation

```rust
pub fn eval(&self, input: &PolicyInputV1) -> PolicyEvalResult
```

## Simulation Engine

### Plan Simulation

The simulation engine processes a PlanPreview and produces a PlanDiff that represents what would change if the plan were executed:

1. **Action Analysis** - Examines each action's kind and parameters
2. **Change Classification** - Categorizes actions as add/remove/edit
3. **Size Enforcement** - Ensures diff fits within memory limits
4. **Redaction Application** - Applies policy-specified redactions

### Redaction System

Redactions use JSONPath-like specifications to remove sensitive information:

- `actions[0].params.secret` - Redacts secret parameter from first action
- `actions[1].kind` - Redacts action kind from second action
- `actions[*].params.password` - Redacts all password parameters

## Integration Points

### Intent Kernel Integration

Policy simulation is automatically triggered during intent preview and submission:

```rust
// In Intent Kernel preview path
let policy_result = PolicyKernel::simulate_intent(&intent, &preview, wm_snapshot, &caps, features);

match policy_result {
    Ok(sim_result) => {
        // Policy allowed - proceed with intent
        // Why-Log includes policy decision and reasons
    }
    Err(_) => {
        // Policy denied - intent remains preview-only
        // Why-Log includes denial reasons
    }
}
```

### Event Fabric Integration

Policy decisions are automatically published to Event Fabric:

```json
{
    "topic": "policy.simulate",
    "payload": {
        "decision": "allow|deny",
        "reasons_count": 2,
        "plan_diff_size": 1024,
        "why_digest": "abc123..."
    }
}
```

### Skills Integration

Skill preview results are automatically routed through policy simulation before returning to callers.

## Security Model

### Capability Requirements

- `CAP_POLICY_LOAD` - Load policy bundles (dev/test only)
- `CAP_POLICY_EVAL` - Evaluate policy inputs
- `CAP_POLICY_SIMULATE` - Run policy simulation

### Access Control

- **Bundle Loading** - Restricted to development/test environments
- **Policy Evaluation** - Requires appropriate capabilities
- **Simulation** - Available to authorized tasks with proper caps

### Audit Trail

All policy operations generate comprehensive audit events:

- `POLICY_BUNDLE_LOAD_OK` - Bundle loaded successfully
- `POLICY_BUNDLE_HASH_MISMATCH` - Bundle integrity check failed
- `POLICY_EVAL_ALLOW` - Policy evaluation allowed
- `POLICY_EVAL_DENY` - Policy evaluation denied
- `POLICY_SIM_ALLOW` - Policy simulation allowed
- `POLICY_SIM_DENY` - Policy simulation denied

## Performance Characteristics

### Latency Targets

- **Policy Evaluation**: p95 ≤ 2ms
- **Simulation**: p95 ≤ 5ms
- **Bundle Loading**: ≤ 100ms (dev mode only)

### Resource Limits

- **Memory**: 32 KiB per policy input/output
- **Plan Diff**: 32 KiB maximum size
- **Instructions**: 1M instruction limit per evaluation

### Deterministic Behavior

- **Virtual Clock** - No wall-clock dependencies
- **Stable Hashing** - Blake3 for all integrity checks
- **Reproducible Results** - Same input always produces same output

## Configuration

### Constants

```rust
pub mod constants {
    pub const MAX_POLICY_IO_SIZE: usize = 32 * 1024;        // 32 KiB
    pub const MAX_PLAN_DIFF_SIZE: usize = 32 * 1024;        // 32 KiB
    pub const DEFAULT_POLICY_MEMORY_LIMIT: u32 = 32 * 1024; // 32 KiB
    pub const DEFAULT_POLICY_INSTRUCTION_LIMIT: u64 = 1_000_000;
}
```

### Environment Variables

- `POLICY_DEV_MODE` - Enable bundle loading (default: false)
- `POLICY_FAIL_OPEN` - Fail-open behavior (default: false)
- `POLICY_VERBOSE` - Enable detailed logging (default: false)

## Fail-Open Detection

### Detection Strategy

The system includes automated fail-open detection to prevent accidentally permissive policies:

1. **Test Fixtures** - Deterministic allow/deny test cases
2. **Expected Rates** - Known good allow rates for each fixture
3. **CI Gates** - Automated detection of policy regressions
4. **Metrics** - JSON output for CI analysis

### CI Integration

```json
{
    "test": "policy_sim",
    "allow_rate": 0.50,
    "fail_open_detected": 0
}
```

## Testing Strategy

### Test Categories

1. **Schema Stability** - CBOR roundtrip, hash consistency
2. **Engine Evaluation** - Bundle loading, policy evaluation
3. **Simulation** - Plan diff generation, redaction application
4. **Fail-Open Detection** - Policy regression prevention
5. **Integration** - Intent, Skills, Event Fabric integration

### Test Fixtures

- **Basic Safety** - Safe vs dangerous operations
- **Capability Check** - Capability validation
- **Risk Assessment** - Risk-based decision making

## Future Enhancements

### Phase 3 Features

- **Advanced Redactions** - Complex path specifications
- **Policy Composition** - Multiple policy bundles
- **Dynamic Updates** - Runtime policy modifications
- **Performance Optimization** - JIT compilation (optional)

### Long-term Vision

- **Machine Learning** - Adaptive policy learning
- **Policy Marketplace** - Community policy sharing
- **Compliance Frameworks** - Industry-standard policies
- **Cross-Platform** - Policy portability

## Troubleshooting

### Common Issues

1. **Policy Bundle Not Loaded**
   - Check if `POLICY_DEV_MODE` is enabled
   - Verify bundle hash matches expected value
   - Ensure bundle size is within memory limits

2. **Simulation Fails**
   - Check capability requirements
   - Verify input size limits
   - Review policy decision reasons

3. **Performance Issues**
   - Monitor instruction counts
   - Check memory usage
   - Review policy complexity

### Debug Tools

- **Policy Stats** - `PolicyKernel::get_stats()`
- **Event Fabric** - Subscribe to "policy.simulate" events
- **Audit Logs** - Review POLICY_* audit events
- **Why-Log** - Check policy decision reasoning

## Examples

### Basic Policy Evaluation

```rust
let input = PolicyInputV1 {
    intent: IntentV1 { /* ... */ },
    preview: PlanPreviewV1 { /* ... */ },
    wm_snapshot: 42,
    caps: vec![1, 2, 3],
    features: 0x1234,
};

let result = eval_policy(&input);
match result.decision {
    Some(decision) => {
        if decision.allow {
            println!("Policy allowed with reasons: {:?}", decision.reasons);
        } else {
            println!("Policy denied with reasons: {:?}", decision.reasons);
        }
    }
    None => println!("Policy evaluation failed: {:?}", result.error),
}
```

### Policy Simulation

```rust
let sim_result = PolicyKernel::simulate_intent(
    &intent,
    &preview,
    wm_snapshot,
    &caps,
    features
)?;

println!("Plan diff: {} adds, {} removes, {} edits",
    sim_result.plan_diff.adds.len(),
    sim_result.plan_diff.removes.len(),
    sim_result.plan_diff.edits.len()
);

println!("Why-log digest: {}", hex::encode(sim_result.why_digest));
```

## Conclusion

Policy Guardrail v0 provides a robust foundation for secure, deterministic policy evaluation in Polymera OS. The system ensures that all autonomous actions are properly vetted before execution, while maintaining performance and providing comprehensive audit trails.

The integration with Intent Kernel, World Model, Skills, and Event Fabric creates a cohesive security architecture that prevents unauthorized actions and maintains system integrity across all agent operations.
