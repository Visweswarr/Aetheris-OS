# Fail-Open Guard: Policy Regression Prevention

## Overview

The Fail-Open Guard is a critical safety mechanism that prevents accidentally permissive policies from being deployed to production. It automatically detects when policy rules become too permissive and blocks deployments until the issue is resolved.

## Problem Statement

### Fail-Open Scenarios

A "fail-open" scenario occurs when a policy system becomes too permissive, allowing actions that should be denied. This can happen due to:

1. **Accidental Rule Changes** - Developers modifying policy rules without understanding security implications
2. **Rule Conflicts** - Multiple rules that interact in unexpected ways
3. **Missing Constraints** - Incomplete policy coverage for new attack vectors
4. **Logic Errors** - Bugs in policy rule implementation

### Security Impact

Fail-open scenarios can lead to:

- **Unauthorized Access** - Users gaining access to restricted resources
- **Privilege Escalation** - Tasks executing with excessive permissions
- **Data Breaches** - Sensitive information being exposed
- **System Compromise** - Malicious actions being allowed

## Detection Strategy

### Test Fixtures

The system uses deterministic test fixtures that represent known good/bad scenarios:

```rust
// Safe operation - should be allowed
allowed_input: IntentV1 {
    description: "safe operation".to_string(),
    intent_type: 1,
    priority: 1,
    // ... other fields
}

// Dangerous operation - should be denied
denied_input: IntentV1 {
    description: "dangerous operation".to_string(),
    intent_type: 2,
    priority: 1,
    // ... other fields
}
```

### Expected Allow Rates

Each fixture has an expected allow rate that represents the correct policy behavior:

- **0.0** - Should deny everything (strict security)
- **0.5** - Should allow safe, deny dangerous (balanced)
- **1.0** - Should allow everything (permissive - usually wrong)

### Detection Logic

```rust
let actual_allow_rate = if allowed && denied {
    0.5 // One allowed, one denied - correct behavior
} else if allowed {
    1.0 // Both allowed - potential fail-open!
} else if denied {
    0.0 // Both denied - overly restrictive
} else {
    0.5 // Neither clearly allowed/denied
};

let fail_open_detected = actual_allow_rate > expected_allow_rate;
```

## CI Integration

### Automated Detection

The fail-open detector runs automatically in CI:

```yaml
# .github/workflows/phase-2-gates.yml
- name: Run Policy Tests
  run: |
    bazel test //tests/policy:all
    
- name: Parse Fail-Open Results
  run: |
    # Extract JSON metrics from test output
    grep '{"test":"policy_sim"' bazel-testlogs/**/*.log | tail -1
    
- name: Check Fail-Open
  run: |
    # Fail CI if fail-open detected
    if [[ "$FAIL_OPEN_DETECTED" == "1" ]]; then
      echo "❌ FAIL-OPEN DETECTED: Policy is too permissive"
      exit 1
    fi
```

### Metrics Output

Tests output JSON metrics for CI analysis:

```json
{
    "test": "policy_sim",
    "allow_rate": 0.50,
    "fail_open_detected": 0
}
```

### CI Gates

The CI pipeline includes multiple fail-open checks:

1. **Policy Tests** - Run all policy test fixtures
2. **Fail-Open Detection** - Parse and validate allow rates
3. **Regression Prevention** - Block deployment if fail-open detected
4. **Artifact Upload** - Upload test results for review

## Test Fixtures

### Basic Safety Fixture

**Purpose**: Verify basic allow/deny logic works correctly

**Expected Behavior**: 
- Safe operations (intent_type=1, low cost) → ALLOW
- Dangerous operations (intent_type=2, high cost) → DENY

**Expected Allow Rate**: 0.5

**Fail-Open Detection**: If both safe and dangerous operations are allowed

### Capability Check Fixture

**Purpose**: Verify capability-based access control

**Expected Behavior**:
- Operations with required caps → ALLOW
- Operations without required caps → DENY

**Expected Allow Rate**: 0.5

**Fail-Open Detection**: If operations are allowed without proper capabilities

### Risk Assessment Fixture

**Purpose**: Verify risk-based decision making

**Expected Behavior**:
- Low-risk operations → ALLOW
- High-risk operations → DENY

**Expected Allow Rate**: 0.5

**Fail-Open Detection**: If high-risk operations are allowed

## Adding New Fixtures

### Fixture Creation

```rust
// Add to FailOpenDetector::new()
fixtures.push(PolicyFixture::new("new_security_check", 0.25));

// Or add dynamically
detector.add_fixture("custom_test", 0.0);
```

### Fixture Design Principles

1. **Deterministic** - Same input always produces same output
2. **Representative** - Covers real security scenarios
3. **Balanced** - Mix of allowed and denied cases
4. **Clear Intent** - Obvious why each case should be allowed/denied

### Fixture Validation

```rust
#[test]
fn test_new_fixture() {
    let fixture = PolicyFixture::new("new_test", 0.25);
    
    // Verify fixture creates valid inputs
    assert!(!fixture.allowed_input.intent.description.is_empty());
    assert!(!fixture.denied_input.intent.description.is_empty());
    
    // Verify expected rate is reasonable
    assert!(fixture.expected_allow_rate >= 0.0);
    assert!(fixture.expected_allow_rate <= 1.0);
}
```

## Fail-Open Scenarios

### Common Causes

1. **Missing Deny Rules**
   ```rego
   # BAD: No explicit deny rule
   allow {
       input.intent.intent_type == "admin"
   }
   
   # GOOD: Explicit deny rule
   allow {
       input.intent.intent_type == "admin"
       input.caps[_] == "admin"
   }
   
   deny {
       input.intent.intent_type == "admin"
       not input.caps[_] == "admin"
   }
   ```

2. **Overly Broad Allow Rules**
   ```rego
   # BAD: Too broad
   allow {
       input.intent.intent_type == "read"
   }
   
   # GOOD: More specific
   allow {
       input.intent.intent_type == "read"
       input.intent.priority <= 5
       input.preview.cost <= 100
   }
   ```

3. **Missing Constraints**
   ```rego
   # BAD: No resource constraints
   allow {
       input.intent.intent_type == "write"
   }
   
   # GOOD: With resource constraints
   allow {
       input.intent.intent_type == "write"
       input.preview.cost <= 1000
       input.intent.priority <= 3
       input.caps[_] == "write"
   }
   ```

### Detection Examples

```rust
// Example 1: Missing deny rule
let fixture = PolicyFixture::new("admin_access", 0.0);
// Expected: 0.0 (deny all admin operations)
// Actual: 1.0 (allow all admin operations)
// Result: FAIL-OPEN DETECTED

// Example 2: Overly permissive
let fixture = PolicyFixture::new("file_access", 0.5);
// Expected: 0.5 (allow safe, deny dangerous)
// Actual: 1.0 (allow everything)
// Result: FAIL-OPEN DETECTED

// Example 3: Correct behavior
let fixture = PolicyFixture::new("basic_safety", 0.5);
// Expected: 0.5 (allow safe, deny dangerous)
// Actual: 0.5 (allow safe, deny dangerous)
// Result: PASS
```

## Remediation

### Immediate Actions

When fail-open is detected:

1. **Block Deployment** - CI automatically fails
2. **Investigate Root Cause** - Review recent policy changes
3. **Fix Policy Rules** - Add missing deny rules or constraints
4. **Re-run Tests** - Verify fail-open is resolved

### Policy Review Process

1. **Change Analysis** - What changed in the policy?
2. **Impact Assessment** - What security controls were weakened?
3. **Rule Validation** - Are all deny conditions present?
4. **Test Coverage** - Do fixtures cover the affected scenarios?

### Common Fixes

1. **Add Explicit Deny Rules**
   ```rego
   deny {
       input.intent.intent_type == "dangerous"
   }
   ```

2. **Strengthen Allow Conditions**
   ```rego
   allow {
       input.intent.intent_type == "safe"
       input.caps[_] == "basic"
       input.preview.cost <= 100
   }
   ```

3. **Add Resource Constraints**
   ```rego
   allow {
       input.intent.intent_type == "read"
       input.preview.cost <= 500
       input.intent.priority <= 5
   }
   ```

## Monitoring and Alerting

### Metrics Collection

Track fail-open detection over time:

- **Fail-Open Rate** - Percentage of tests detecting fail-open
- **Detection Latency** - Time from change to detection
- **False Positives** - Incorrect fail-open detections

### Alerting

- **CI Failures** - Immediate notification of fail-open
- **Policy Changes** - Review required for security-impacting changes
- **Trend Analysis** - Increasing fail-open rates indicate systemic issues

## Best Practices

### Policy Development

1. **Fail-Safe Default** - Default to deny, explicitly allow
2. **Explicit Deny Rules** - Always include deny conditions
3. **Resource Constraints** - Limit scope of allowed operations
4. **Capability Checks** - Verify proper authorization

### Testing

1. **Comprehensive Fixtures** - Cover all security scenarios
2. **Edge Cases** - Test boundary conditions
3. **Regression Testing** - Ensure fixes don't break existing rules
4. **Automated Validation** - CI gates prevent deployment of broken policies

### Review Process

1. **Security Review** - All policy changes require security review
2. **Change Documentation** - Document why changes are needed
3. **Impact Assessment** - Evaluate security implications
4. **Rollback Plan** - Plan for reverting problematic changes

## Conclusion

The Fail-Open Guard is a critical safety mechanism that prevents security regressions in Polymera OS. By automatically detecting when policies become too permissive, it ensures that the system maintains its security posture and prevents unauthorized actions.

The combination of deterministic test fixtures, automated CI detection, and comprehensive monitoring creates a robust defense against policy regressions, allowing developers to iterate quickly while maintaining security.
