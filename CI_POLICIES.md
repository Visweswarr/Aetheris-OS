# Polymera OS CI Policies

## 🎯 Project Rules & CI Policies

This document defines the mandatory CI policies that enforce the project rules for Polymera OS development.

## 📝 Language & Technology Standards

### 1. Language Requirements
- **Kernel/Systems/Crypto/Net**: Must be written in **Rust**
- **Graphics Paths**: Must be written in **C++**
- **Web/Desktop UI**: Must be written in **TypeScript**
- **Agents/Tooling**: Must be written in **Python**

### 2. Technology Stack Requirements
- **Cryptography**: Default to **Kyber/Dilithium hybrids**
- **Zero-Knowledge**: Use **Noir/Halo2** for ZK flows
- **Policy Engine**: **OPA/Rego→WASM** for policy enforcement
- **Networking**: **libp2p/QUIC** for mesh networking
- **XR Support**: **OpenXR+Vulkan** for extended reality
- **Data Layer**: **Redpanda/Kafka**, **Postgres/Timescale**, **Redis**, **DuckDB**

### 3. Performance Mode Requirements
- **Regulated Workloads**: Must support **deterministic mode**
- **Performance Guarantees**: All operations must meet specified SLOs

## 🧪 Testing Requirements

### 1. Test Coverage Mandates
Every module **MUST** include:
- **Unit Tests**: Comprehensive component testing
- **Fuzz Tests**: Where applicable (crypto, parsing, network protocols)
- **Integration Tests**: Component interaction testing

### 2. Test Quality Standards
- **Coverage**: >90% code coverage required
- **Performance**: All tests must complete within specified time limits
- **Security**: All security tests must pass
- **Reliability**: Tests must be deterministic and repeatable

## ⚡ SLO Gates (Mandatory for Every Merge)

### 1. Identity Operations
- **Target**: <300ms p95
- **Measurement**: Identity verification, authentication, authorization
- **Gate**: **BLOCKING** - Merge blocked if SLO not met

### 2. XR Operations
- **Target**: <20ms MTP p95
- **Measurement**: Motion-to-photon latency for XR applications
- **Gate**: **BLOCKING** - Merge blocked if SLO not met

### 3. Anti-Abuse Confirmation
- **Target**: <3s p95
- **Measurement**: Anti-abuse system response time
- **Gate**: **BLOCKING** - Merge blocked if SLO not met

### 4. Mesh Synchronization
- **Target**: <60s p95
- **Measurement**: Mesh network synchronization time
- **Gate**: **BLOCKING** - Merge blocked if SLO not met

## 🔒 CI Enforcement Policies

### 1. Pre-Merge Requirements
- [ ] **Language Compliance**: Code written in correct language for component
- [ ] **Technology Stack**: Uses specified technology stack
- [ ] **Test Coverage**: All required test types present and passing
- [ ] **SLO Compliance**: All SLO gates pass
- [ ] **Security Scan**: No critical vulnerabilities
- [ ] **Performance Regression**: No performance degradation

### 2. Automated Checks
- **Language Detection**: Automated language verification per component
- **Dependency Scanning**: Technology stack compliance verification
- **Test Execution**: Automated test suite execution
- **Performance Benchmarking**: SLO compliance verification
- **Security Scanning**: Vulnerability and policy compliance checks

### 3. Manual Review Requirements
- **Architecture Review**: Technology choices align with project rules
- **Performance Review**: SLO compliance verification
- **Security Review**: Security implications assessment
- **Testing Review**: Test coverage and quality assessment

## 🚫 Merge Blocking Conditions

### 1. Language Violations
- **Rust Required**: Kernel, systems, crypto, networking code not in Rust
- **C++ Required**: Graphics code not in C++
- **TypeScript Required**: Web/desktop UI code not in TypeScript
- **Python Required**: Agent/tooling code not in Python

### 2. Technology Stack Violations
- **Cryptography**: Not using Kyber/Dilithium hybrids
- **ZK Framework**: Not using Noir/Halo2
- **Policy Engine**: Not using OPA/Rego→WASM
- **Networking**: Not using libp2p/QUIC
- **XR Support**: Not using OpenXR+Vulkan
- **Data Layer**: Not using specified data technologies

### 3. Testing Violations
- **Missing Tests**: Unit, fuzz, or integration tests missing
- **Test Failures**: Any test type failing
- **Coverage Insufficient**: <90% code coverage
- **Performance Tests**: Performance tests failing

### 4. SLO Violations
- **Identity SLO**: >300ms p95
- **XR SLO**: >20ms MTP p95
- **AA Confirm SLO**: >3s p95
- **Mesh Sync SLO**: >60s p95

## 🔧 CI Pipeline Configuration

### 1. Language Compliance Stage
```yaml
- name: Language Compliance Check
  run: |
    # Verify Rust usage in kernel/systems/crypto/net
    # Verify C++ usage in graphics
    # Verify TypeScript usage in web/desktop UI
    # Verify Python usage in agents/tooling
```

### 2. Technology Stack Verification
```yaml
- name: Technology Stack Verification
  run: |
    # Verify Kyber/Dilithium usage
    # Verify Noir/Halo2 usage
    # Verify OPA/Rego→WASM usage
    # Verify libp2p/QUIC usage
    # Verify OpenXR+Vulkan usage
    # Verify data layer technologies
```

### 3. Test Execution Stage
```yaml
- name: Test Execution
  run: |
    # Run unit tests
    # Run fuzz tests (where applicable)
    # Run integration tests
    # Verify test coverage >90%
```

### 4. SLO Compliance Stage
```yaml
- name: SLO Compliance Check
  run: |
    # Test identity operations <300ms p95
    # Test XR operations <20ms MTP p95
    # Test AA confirm <3s p95
    # Test mesh sync <60s p95
```

### 5. Performance Regression Check
```yaml
- name: Performance Regression Check
  run: |
    # Compare against baseline performance
    # Block if any SLO degraded
    # Generate performance report
```

## 📊 Compliance Reporting

### 1. Daily Compliance Report
- **Language Compliance**: Percentage of code following language rules
- **Technology Stack**: Compliance with specified technologies
- **Test Coverage**: Current test coverage status
- **SLO Status**: Current SLO compliance status

### 2. Merge Request Compliance
- **Pre-Merge Checklist**: All requirements met
- **SLO Verification**: All SLO gates passing
- **Test Results**: All test types passing
- **Security Status**: No blocking vulnerabilities

### 3. Performance Dashboard
- **Real-time SLO Monitoring**: Live SLO compliance status
- **Performance Trends**: Historical performance data
- **Regression Alerts**: Performance degradation notifications
- **SLO Gate Status**: Current gate status for all SLOs

## 🚨 Escalation Procedures

### 1. SLO Violation Escalation
1. **Immediate Block**: Merge automatically blocked
2. **Team Notification**: Development team notified
3. **Performance Review**: Performance team investigation
4. **Root Cause Analysis**: Identify and fix performance issues
5. **Re-testing**: Verify SLO compliance restored
6. **Merge Approval**: Manual approval required after SLO compliance

### 2. Language/Technology Violation Escalation
1. **Code Review Block**: Merge blocked in code review
2. **Architecture Review**: Architecture team assessment
3. **Technology Alignment**: Verify technology choices
4. **Refactoring Plan**: Plan to align with project rules
5. **Implementation**: Implement required changes
6. **Verification**: Verify compliance restored

### 3. Testing Violation Escalation
1. **Test Failure Block**: Merge blocked due to test failures
2. **Test Analysis**: Identify failing test root cause
3. **Test Fix**: Fix failing tests
4. **Coverage Verification**: Ensure >90% coverage
5. **Re-execution**: Re-run all test suites
6. **Approval**: Manual approval after test compliance

## 📋 Compliance Checklist Template

### For Every Merge Request
```markdown
## CI Compliance Checklist

### Language Compliance
- [ ] Kernel/Systems/Crypto/Net code written in Rust
- [ ] Graphics code written in C++
- [ ] Web/Desktop UI code written in TypeScript
- [ ] Agent/Tooling code written in Python

### Technology Stack Compliance
- [ ] Uses Kyber/Dilithium hybrids for cryptography
- [ ] Uses Noir/Halo2 for ZK flows
- [ ] Uses OPA/Rego→WASM for policy
- [ ] Uses libp2p/QUIC for mesh networking
- [ ] Uses OpenXR+Vulkan for XR support
- [ ] Uses specified data layer technologies

### Testing Compliance
- [ ] Unit tests present and passing
- [ ] Fuzz tests present and passing (where applicable)
- [ ] Integration tests present and passing
- [ ] Test coverage >90%

### SLO Compliance
- [ ] Identity operations <300ms p95
- [ ] XR operations <20ms MTP p95
- [ ] AA confirm <3s p95
- [ ] Mesh sync <60s p95

### Security Compliance
- [ ] Security scan passed
- [ ] No critical vulnerabilities
- [ ] Policy compliance verified

### Performance Compliance
- [ ] No performance regression
- [ ] All SLOs maintained or improved
- [ ] Performance tests passing
```

## 🎯 Enforcement Summary

These CI policies ensure that:

1. **Every merge** satisfies all SLO gates
2. **All code** follows language and technology requirements
3. **All modules** include comprehensive testing
4. **Performance** is maintained or improved
5. **Security** is never compromised
6. **Quality** is consistently high

**Violation of any policy results in automatic merge blocking until compliance is restored.**

---

*These policies are enforced automatically by the CI pipeline and must be satisfied before any code can be merged into the main branch.*
