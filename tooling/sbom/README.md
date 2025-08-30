# 🔒 Polymera OS SBOM & Signing Tools

Comprehensive supply chain security tools for Polymera OS, providing Software Bill of Materials (SBOM) generation, Sigstore keyless signing, and policy enforcement.

## 🎯 Overview

The SBOM & Signing Tools provide:

- **📋 SBOM Generation**: Comprehensive Software Bill of Materials for all components
- **🔐 Sigstore Signing**: Keyless cryptographic signing using Sigstore
- **🔒 Policy Enforcement**: Automated compliance checking and violation detection
- **📊 Reporting**: Detailed analysis and compliance reports
- **🔗 CI/CD Integration**: Seamless integration with GitHub Actions and other CI systems

## 🏗️ Architecture

```
tooling/sbom/
├── sbom_generator.py      # SBOM generation for all components
├── sigstore_signer.py     # Sigstore keyless signing tool
├── policy_enforcer.py     # Policy compliance enforcement
├── test_sbom_signing.py   # Comprehensive test suite
├── requirements.txt       # Python dependencies
└── README.md             # This documentation
```

## 🚀 Quick Start

### Prerequisites

- **Python**: 3.7+ with pip
- **Rust**: `cargo` and `cargo-sbom` (for Rust SBOM generation)
- **Sigstore**: `cosign` tool for signing and verification
- **System Tools**: `jq`, `git`, `tar`, `gzip`

### Installation

1. **Install Python dependencies**:
   ```bash
   cd tooling/sbom
   pip install -r requirements.txt
   ```

2. **Install Rust SBOM tools**:
   ```bash
   cargo install cargo-sbom
   ```

3. **Install Sigstore tools**:
   ```bash
   # Download cosign
   curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64"
   sudo mv cosign-linux-amd64 /usr/local/bin/cosign
   sudo chmod +x /usr/local/bin/cosign
   
   # Download rekor-cli
   curl -O -L "https://github.com/sigstore/rekor/releases/latest/download/rekor-cli-linux-amd64"
   sudo mv rekor-cli-linux-amd64 /usr/local/bin/rekor-cli
   sudo chmod +x /usr/local/bin/rekor-cli
   ```

4. **Verify installation**:
   ```bash
   python3 test_sbom_signing.py
   ```

## 📋 SBOM Generation

### Basic Usage

```bash
# Generate comprehensive SBOM for all components
python3 sbom_generator.py

# Generate SBOM in specific format
python3 sbom_generator.py --format cyclonedx-json

# Generate only component SBOMs
python3 sbom_generator.py --components-only

# Generate only Syft SBOM
python3 sbom_generator.py --syft-only

# Custom output directory
python3 sbom_generator.py --output-dir custom-sbom
```

### Component Coverage

The SBOM generator automatically detects and processes:

- **Rust Components**: Uses `cargo-sbom` for dependency analysis
- **TypeScript/Node.js**: Parses `package.json` and generates dependency lists
- **Python Components**: Analyzes `pyproject.toml` and `requirements.txt`
- **System Dependencies**: Uses Syft for system-level package detection

### Output Formats

- **SPDX 2.3**: Standard Software Package Data Exchange format
- **CycloneDX**: Lightweight SBOM format for security and compliance
- **JSON**: Machine-readable format for CI/CD integration
- **Markdown**: Human-readable summary reports

## 🔐 Sigstore Signing

### Basic Usage

```bash
# Sign a single artifact
python3 sigstore_signer.py artifact.bin

# Sign all artifacts in a directory
python3 sigstore_signer.py /path/to/artifacts/

# Sign with custom identity
python3 sigstore_signer.py --identity "https://my-org.com" artifact.bin

# Sign and verify
python3 sigstore_signer.py --verify artifact.bin

# Generate signing report
python3 sigstore_signer.py --report signing-report.md artifact.bin
```

### Signing Configuration

Environment variables for configuration:

```bash
export SIGSTORE_IDENTITY="https://github.com/polymera-os"
export SIGSTORE_ISSUER="https://token.actions.githubusercontent.com"
```

### Supported Artifacts

- **Binaries**: `.bin`, `.exe`, `.so`, `.dll`, `.dylib`
- **Archives**: `.tar.gz`, `.zip`, `.deb`, `.rpm`
- **Configuration**: `.json`, `.xml`, `.yaml`, `.yml`
- **Documentation**: `.md`, `.txt`, `.log`

### Signature Verification

```bash
# Verify individual signature
cosign verify-blob artifact.bin \
  --signature artifact.bin.sig \
  --certificate artifact.bin.cert

# Verify with identity constraints
cosign verify-blob artifact.bin \
  --signature artifact.bin.sig \
  --certificate artifact.bin.cert \
  --certificate-identity "https://github.com/polymera-os" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com"
```

## 🔒 Policy Enforcement

### Basic Usage

```bash
# Enforce policies on artifacts
python3 policy_enforcer.py /path/to/artifacts/

# Use custom policy configuration
python3 policy_enforcer.py --config policy.yaml /path/to/artifacts/

# Check SBOM completeness
python3 policy_enforcer.py --sbom-check /path/to/sbom/

# Generate compliance summary
python3 policy_enforcer.py --summary /path/to/artifacts/

# Don't fail on violations (for testing)
python3 policy_enforcer.py --no-fail /path/to/artifacts/
```

### Policy Configuration

Create a `policy.yaml` file:

```yaml
# Policy configuration for Polymera OS
require_signatures: true
require_sbom: true
blocked_extensions: [".tmp", ".log", ".cache"]
required_components:
  - kernel
  - security
  - services
  - tests
  - ui
  - tooling
signature_verification: true
sbom_verification: true
fail_on_violation: true
```

### Policy Checks

The enforcer performs the following checks:

1. **Signature Requirements**: All artifacts must be signed
2. **SBOM Coverage**: All components must have SBOM documentation
3. **Component Completeness**: Required components must be present
4. **Signature Validity**: Signatures must be cryptographically valid
5. **Policy Compliance**: Overall compliance with security policies

## 🔗 CI/CD Integration

### GitHub Actions Integration

The tools are designed to integrate seamlessly with GitHub Actions:

```yaml
- name: Generate SBOM
  run: |
    cd tooling/sbom
    python3 sbom_generator.py --output-dir sbom/

- name: Sign Artifacts
  run: |
    cd tooling/sbom
    python3 sigstore_signer.py --verify artifacts/

- name: Enforce Policy
  run: |
    cd tooling/sbom
    python3 policy_enforcer.py --summary artifacts/
```

### Jenkins Integration

```groovy
stage('Supply Chain Security') {
    steps {
        sh '''
            cd tooling/sbom
            
            # Generate SBOM
            python3 sbom_generator.py --output-dir sbom/
            
            # Sign artifacts
            python3 sigstore_signer.py --verify artifacts/
            
            # Enforce policy
            python3 policy_enforcer.py --summary artifacts/
        '''
    }
    post {
        always {
            archiveArtifacts artifacts: 'sbom/**/*, artifacts/**/*'
        }
    }
}
```

### GitLab CI Integration

```yaml
supply-chain-security:
  stage: security
  script:
    - cd tooling/sbom
    - python3 sbom_generator.py --output-dir sbom/
    - python3 sigstore_signer.py --verify artifacts/
    - python3 policy_enforcer.py --summary artifacts/
  artifacts:
    paths:
      - sbom/
      - artifacts/
    reports:
      security: policy-compliance-summary.json
```

## 📊 Reporting and Analysis

### SBOM Reports

The SBOM generator produces comprehensive reports:

- **Component Analysis**: Detailed breakdown of all components
- **Dependency Trees**: Visual representation of dependencies
- **License Information**: License compliance analysis
- **Vulnerability Data**: Security vulnerability information
- **Compliance Status**: Regulatory compliance indicators

### Signing Reports

The Sigstore signer generates detailed signing reports:

- **Signing Summary**: Count of signed vs. unsigned artifacts
- **Signature Details**: Hash values, timestamps, and certificates
- **Verification Results**: Signature validation status
- **Identity Information**: Signing identity and issuer details
- **Error Analysis**: Detailed error information for failed operations

### Policy Reports

The policy enforcer provides compliance reports:

- **Violation Summary**: Count and severity of policy violations
- **Compliance Status**: Overall compliance percentage
- **Recommendations**: Actionable improvement suggestions
- **Component Coverage**: Missing or incomplete components
- **Security Analysis**: Security posture assessment

## 🔧 Advanced Configuration

### Custom SBOM Templates

Create custom SBOM templates for specific use cases:

```python
from sbom_generator import SBOMGenerator

# Custom component definitions
custom_components = {
    "custom-component": {
        "path": "custom/path",
        "type": "rust",
        "description": "Custom component description"
    }
}

generator = SBOMGenerator()
generator.components.update(custom_components)
```

### Custom Signing Policies

Implement custom signing policies:

```python
from sigstore_signer import SigstoreSigner

# Custom signing configuration
signer = SigstoreSigner(
    identity="https://custom-identity.com",
    issuer="https://custom-issuer.com"
)

# Custom file patterns
custom_patterns = ["*.custom", "*.special"]
signer.sign_directory("/path/to/artifacts", file_patterns=custom_patterns)
```

### Custom Policy Rules

Extend policy enforcement with custom rules:

```python
from policy_enforcer import PolicyEnforcer

# Custom policy configuration
custom_policy = {
    "custom_rule": True,
    "custom_threshold": 0.95
}

enforcer = PolicyEnforcer(config_file="custom-policy.yaml")
enforcer.policy.update(custom_policy)
```

## 🧪 Testing and Validation

### Running Tests

```bash
# Run comprehensive test suite
python3 test_sbom_signing.py

# Test individual components
python3 -c "from sbom_generator import SBOMGenerator; print('SBOM Generator OK')"
python3 -c "from sigstore_signer import SigstoreSigner; print('Sigstore Signer OK')"
python3 -c "from policy_enforcer import PolicyEnforcer; print('Policy Enforcer OK')"
```

### Test Coverage

The test suite covers:

- **Unit Tests**: Individual component functionality
- **Integration Tests**: End-to-end workflow testing
- **CLI Tests**: Command-line interface validation
- **Error Handling**: Exception and error condition testing
- **Policy Validation**: Policy enforcement verification

### Validation Scripts

```bash
# Validate SBOM format
jq empty sbom/*.json

# Validate signatures
for sig in artifacts/*.sig; do
    base_file="${sig%.sig}"
    cosign verify-blob "$base_file" --signature "$sig" --certificate "${base_file}.cert"
done

# Validate policy compliance
python3 policy_enforcer.py --summary artifacts/
```

## 🔍 Troubleshooting

### Common Issues

#### SBOM Generation Failures

```bash
# Check Rust toolchain
rustup show
cargo --version

# Install missing tools
cargo install cargo-sbom

# Check component paths
ls -la kernel/ security/ services/ tests/ ui/ tooling/
```

#### Signing Failures

```bash
# Check cosign installation
cosign version

# Verify identity configuration
echo $SIGSTORE_IDENTITY
echo $SIGSTORE_ISSUER

# Check network connectivity
curl -I https://rekor.sigstore.dev
```

#### Policy Enforcement Failures

```bash
# Check policy configuration
cat policy.yaml

# Verify artifact signatures
ls -la artifacts/*.sig artifacts/*.cert

# Check component completeness
ls -la | grep -E "(kernel|security|services|tests|ui|tooling)"
```

### Debug Mode

Enable debug output for troubleshooting:

```bash
# Set debug environment variables
export RUST_LOG=debug
export PYTHONPATH="${PYTHONPATH}:$(pwd)/tooling/sbom"

# Run with verbose output
python3 -v sbom_generator.py --components-only
python3 -v sigstore_signer.py --verify artifacts/
python3 -v policy_enforcer.py --summary artifacts/
```

### Log Analysis

Review logs for detailed error information:

```bash
# Check system logs
journalctl -u polymera-sbom

# Check application logs
tail -f sbom-generation.log
tail -f signing.log
tail -f policy-enforcement.log
```

## 📚 API Reference

### SBOM Generator API

```python
class SBOMGenerator:
    def __init__(self, output_dir: str = "sbom", format: str = "spdx-json")
    def generate_comprehensive_sbom(self) -> Dict[str, Any]
    def generate_syft_sbom(self) -> bool
    def merge_sboms(self) -> Dict[str, Any]
    def generate_summary_report(self) -> str
```

### Sigstore Signer API

```python
class SigstoreSigner:
    def __init__(self, identity: str = None, issuer: str = None)
    def sign_artifact(self, artifact_path: Path, output_dir: Path = None) -> Dict[str, Any]
    def sign_directory(self, directory_path: Path, output_dir: Path = None) -> List[Dict[str, Any]]
    def verify_signature(self, artifact_path: Path, signature_file: Path, certificate_file: Path) -> bool
    def generate_signing_report(self, results: List[Dict[str, Any]], output_file: Path = None) -> str
```

### Policy Enforcer API

```python
class PolicyEnforcer:
    def __init__(self, config_file: str = None)
    def enforce_policy(self, paths: List[Path]) -> bool
    def check_sbom_completeness(self, sbom_dir: Path) -> Dict[str, Any]
    def generate_compliance_summary(self) -> str
```

## 🤝 Contributing

### Adding New Components

1. **Update component definitions** in `sbom_generator.py`
2. **Add component-specific logic** for SBOM generation
3. **Update test suite** with new component tests
4. **Document component requirements** in this README

### Extending Signing Capabilities

1. **Add new signature formats** to `sigstore_signer.py`
2. **Implement custom verification logic** for new formats
3. **Add format detection** and automatic handling
4. **Update documentation** with new capabilities

### Enhancing Policy Enforcement

1. **Add new policy rules** to `policy_enforcer.py`
2. **Implement custom compliance checks** for new rules
3. **Add policy validation** and error handling
4. **Update test coverage** for new policies

## 📄 License

This tooling is part of Polymera OS and follows the same licensing terms.

---

**🔒 Secure Supply Chain, Secure Software!** 🔒

For questions, issues, or contributions, please refer to the main Polymera OS documentation or submit an issue in the repository.
