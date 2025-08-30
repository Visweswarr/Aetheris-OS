# NGFS v1 Release Process Guide

This document outlines the complete process for releasing NGFS v1 (Next-Generation Filesystem) as part of Polymera OS Phase 3.

## 🎯 Overview

NGFS v1 represents the complete implementation of Polymera OS's content-addressed, DID-bound encrypted filesystem. This release includes all core features from content addressing through on-chain audit anchoring, providing a comprehensive foundation for secure, verifiable, and scalable storage.

## 📋 Release Checklist

### Pre-Release Validation

- [ ] All NGFS v1 features implemented (P3-01-A01 through P3-01-A14)
- [ ] All tests passing across all languages (Rust, Go, Python, TypeScript, C, Solidity)
- [ ] Performance gates validated (diff ≤1s for 10k entries, vault read ≤100µs median)
- [ ] Documentation complete and up-to-date
- [ ] CI/CD pipeline configured and tested
- [ ] Release artifacts prepared

### Release Components

1. **Core NGFS**
   - Content-Addressed Storage (CAS)
   - Manifest Management
   - Snapshot System
   - Integrity Sentinel

2. **Advanced Features**
   - Personal Data Vault
   - Snapshot Diff & History
   - FUSE Mount
   - Smart Contract Sandbox
   - On-Chain Audit Anchoring

3. **Performance & Validation**
   - Performance gates
   - Test coverage validation
   - Schema consistency checks

## 🚀 Release Methods

### Method 1: Automated CI/CD (Recommended)

The primary release method uses the GitHub Actions workflow:

```bash
# Trigger the release workflow
# Navigate to: .github/workflows/phase-3-ngfs.yml
# Click "Run workflow" → "workflow_dispatch"
```

This will:
- Run all NGFS tests
- Validate performance gates
- Generate release artifacts
- Create Git tag `v0.3.0-ngfs`
- Generate serial banner

### Method 2: Manual Release Script

Use the provided release scripts:

#### Linux/macOS
```bash
# Full release with performance tests
./scripts/ngfs-release.sh

# Release without performance tests
./scripts/ngfs-release.sh v0.3.0-ngfs true

# Custom version
./scripts/ngfs-release.sh v0.3.0-ngfs-custom
```

#### Windows
```cmd
REM Full release with performance tests
scripts\ngfs-release.bat

REM Release without performance tests
scripts\ngfs-release.bat v0.3.0-ngfs true

REM Custom version
scripts\ngfs-release.bat v0.3.0-ngfs-custom
```

## 🔧 Prerequisites

Before running the release process, ensure you have:

- **Bazel**: Build system for polyglot projects
- **Rust**: For core NGFS services
- **Go**: For CLI tools and utilities
- **Python**: For verification and testing tools
- **Node.js**: For UI components and TypeScript tests
- **Git**: For version control and tagging

### Installation Commands

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install bazel rustc cargo golang-go python3 nodejs npm git

# macOS
brew install bazel rust go python node git

# Windows
# Install via official installers for each tool
```

## 📊 Performance Gates

NGFS v1 includes strict performance requirements:

### Diff Processing
- **Requirement**: ≤1 second for 10,000 entries
- **Measurement**: End-to-end diff generation time
- **Validation**: Automated in CI pipeline

### Vault Read Latency
- **Requirement**: ≤100 microseconds median
- **Measurement**: 1,000 sequential read operations
- **Validation**: Automated in CI pipeline

### Baseline Storage
Performance results are stored as baselines for future regression testing:

```json
{
  "timestamp": "2024-12-XX",
  "git_commit": "abc123...",
  "git_tag": "v0.3.0-ngfs",
  "performance_gates": {
    "diff_10k_entries_max_ms": 1000,
    "vault_read_median_max_us": 100,
    "actual_results": {
      "diff_10k_entries_ms": 850,
      "vault_read_median_us": 75
    }
  }
}
```

## 🧪 Testing Strategy

### Test Coverage Requirements

- **Rust**: Core services, data structures, algorithms
- **Go**: CLI tools, integration tests
- **Python**: Verification tools, performance tests
- **TypeScript**: UI components, user interactions
- **Solidity**: Smart contract functionality
- **C**: ZKVM integration, low-level operations

### Test Execution

```bash
# Run all tests
bazel test //tests/... //tooling/python/tests/... //ui/.../tests/...

# Run specific test suites
bazel test //tests/ngfs:...
bazel test //tests/contracts:...
bazel test //tests/anchors:...

# Run Python tests
cd tooling/python
python -m pytest tests/ -v

# Run TypeScript tests
npm test
```

## 📁 Release Artifacts

Upon successful release, the following artifacts are generated:

1. **Release Summary** (`ngfs-release-summary.json`)
   - Component status
   - Performance baseline
   - Test coverage statistics

2. **Serial Banner** (`ngfs-serial-banner.txt`)
   - Standardized release confirmation
   - Required format: `[NGFS OK] v0.3.0-ngfs anchored; vault/contract/diff integrity PASS`

3. **Test Results** (`test-results/`)
   - Rust test logs
   - Python test logs
   - TypeScript test logs

4. **Performance Data** (`perf-results/`)
   - Performance test logs
   - Performance baseline JSON
   - Gate validation results

5. **Git Tag** (`v0.3.0-ngfs`)
   - Annotated tag with release message
   - Pushed to remote repository

## 🔍 Validation Steps

### Critical File Check
The release process validates the existence of all critical files:

- Core NGFS services
- CLI tools
- Python verification tools
- UI components
- Smart contracts
- Schema definitions

### Schema Consistency
Validates that all CDDL schemas are consistent and properly generated.

### Test Coverage
Ensures comprehensive test coverage across all components and languages.

## 🚨 Troubleshooting

### Common Issues

1. **Performance Gate Failures**
   - Check system resources
   - Verify test data size
   - Review recent code changes

2. **Build Failures**
   - Verify all dependencies installed
   - Check Bazel configuration
   - Review build logs

3. **Test Failures**
   - Run tests individually
   - Check test data fixtures
   - Verify environment setup

### Recovery Steps

1. **Fix Issues**: Address any failing tests or performance issues
2. **Re-run Validation**: Execute the release process again
3. **Verify Changes**: Ensure fixes don't introduce new issues
4. **Document Issues**: Record any problems for future reference

## 📈 Post-Release Activities

### Immediate Actions
1. **Verify Tag**: Confirm Git tag was created and pushed
2. **Check Artifacts**: Review all generated release artifacts
3. **Update Documentation**: Ensure release notes are accessible
4. **Notify Team**: Communicate successful release

### Follow-up Tasks
1. **Deploy to Production**: Begin production deployment process
2. **Monitor Performance**: Track real-world performance metrics
3. **Collect Feedback**: Gather user and developer feedback
4. **Plan Next Phase**: Begin P3-02 (Device Runtime) planning

## 🔗 Related Documentation

- [NGFS v1 Technical Documentation](NGFS-V1.md)
- [NGFS v1 Release Notes](RELEASE-NOTES-NGFS.md)
- [Phase 3 Overview](../README.md)
- [CI/CD Configuration](../../.github/workflows/phase-3-ngfs.yml)

## 📞 Support

For issues during the release process:

1. **Check Logs**: Review CI/CD logs and test outputs
2. **Review Documentation**: Consult this guide and related docs
3. **Team Consultation**: Engage with the development team
4. **Issue Tracking**: Create GitHub issues for persistent problems

---

**Release Status**: Ready for Production  
**Next Phase**: P3-02 (Device Runtime)  
**Maintenance**: Ongoing support and updates
