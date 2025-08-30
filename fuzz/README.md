# 🚀 Polymera OS Fuzz Suite

A comprehensive fuzzing infrastructure for Polymera OS that tests Rust, Python, and Go components for security vulnerabilities and robustness issues.

## 🎯 Overview

The Fuzz Suite provides automated security testing through fuzzing techniques across multiple programming languages:

- **🦀 Rust**: Uses `cargo-fuzz` with libFuzzer for capability token parsers and cryptographic operations
- **🐍 Python**: Uses `Atheris` for JSON and base64 parsing robustness
- **🐹 Go**: Uses `go-fuzz` for parser testing and edge case handling

## 🏗️ Architecture

```
fuzz/
├── rust/                    # Rust fuzzing targets
│   ├── Cargo.toml          # Rust dependencies and targets
│   ├── src/                # Fuzzer source code
│   │   ├── capability_token_format_fuzzer.rs
│   │   ├── capability_token_sign_fuzzer.rs
│   │   ├── capability_token_verify_fuzzer.rs
│   │   ├── json_parser_fuzzer.rs
│   │   ├── base64_parser_fuzzer.rs
│   │   └── uuid_parser_fuzzer.rs
│   └── fuzz.sh             # Rust fuzzing orchestration
├── python/                  # Python fuzzing targets
│   ├── requirements.txt     # Python dependencies
│   ├── json_parser_fuzzer.py
│   ├── base64_parser_fuzzer.py
│   └── fuzz.sh             # Python fuzzing orchestration
├── go/                      # Go fuzzing targets
│   ├── go.mod              # Go module definition
│   ├── json_parser_fuzzer.go
│   └── fuzz.sh             # Go fuzzing orchestration
├── .github/workflows/       # CI/CD integration
│   └── fuzz.yml            # GitHub Actions workflow
├── run_fuzz_suite.sh        # Main orchestration script
└── README.md               # This documentation
```

## 🚀 Quick Start

### Prerequisites

- **Rust**: `cargo`, `rustc` (stable channel)
- **Python**: `python3`, `pip`, `venv`
- **Go**: `go` 1.21+
- **System**: `timeout` command, `bash`

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd polymera-os/fuzz
   ```

2. **Make scripts executable**:
   ```bash
   chmod +x run_fuzz_suite.sh
   chmod +x rust/fuzz.sh
   chmod +x python/fuzz.sh
   chmod +x go/fuzz.sh
   ```

3. **Run the complete fuzz suite**:
   ```bash
   ./run_fuzz_suite.sh
   ```

### Individual Language Fuzzing

#### Rust Fuzzing
```bash
cd rust
./fuzz.sh
```

#### Python Fuzzing
```bash
cd python
./fuzz.sh
```

#### Go Fuzzing
```bash
cd go
./fuzz.sh
```

## ⚙️ Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `FUZZ_DURATION` | 300 | Duration per target in seconds |
| `MAX_CRASHES` | 20 | Maximum crashes to collect per target |
| `TOTAL_TIMEOUT` | 3600 | Total timeout for entire suite (1 hour) |

### Custom Configuration

```bash
# Run with custom duration and crash limits
FUZZ_DURATION=600 MAX_CRASHES=50 ./run_fuzz_suite.sh

# Run individual language with custom settings
cd rust
FUZZ_DURATION=120 MAX_CRASHES=10 ./fuzz.sh
```

## 🎯 Fuzz Targets

### Rust Targets

| Target | Purpose | Tests |
|--------|---------|-------|
| `capability_token_format_fuzzer` | Capability token parsing | JSON deserialization, base64 decoding, size validation |
| `capability_token_sign_fuzzer` | Cryptographic signing | Dilithium algorithms, key generation, signature creation |
| `capability_token_verify_fuzzer` | Token validation | Format validation, timestamp checking, claim verification |
| `json_parser_fuzzer` | JSON parsing | serde_json robustness, malformed input handling |
| `base64_parser_fuzzer` | Base64 encoding/decoding | Padding handling, invalid characters, edge cases |
| `uuid_parser_fuzzer` | UUID parsing | Format validation, invalid characters, edge cases |

### Python Targets

| Target | Purpose | Tests |
|--------|---------|-------|
| `json_parser_fuzzer` | JSON parsing | `json.loads()` robustness, malformed input handling |
| `base64_parser_fuzzer` | Base64 encoding/decoding | `base64` module robustness, edge cases |

### Go Targets

| Target | Purpose | Tests |
|--------|---------|-------|
| `json_parser_fuzzer` | JSON parsing | `encoding/json` robustness, malformed input handling |

## 📊 Understanding Results

### Output Structure

```
fuzz/
├── artifacts/               # Collected crash artifacts
│   ├── rust_20231201_143022/
│   ├── python_20231201_143022/
│   └── go_20231201_143022/
├── reports/                 # Generated reports
│   └── fuzz_report_20231201_143022.md
├── corpus/                  # Seed data for fuzzers
└── crashes/                 # Temporary crash storage
```

### Report Interpretation

#### ✅ No Crashes Found
- **Status**: Green - Codebase appears robust
- **Action**: Consider increasing fuzz duration for more thorough testing
- **Next**: Integrate into CI/CD pipeline for continuous testing

#### ⚠️ Crashes Found
- **Status**: Red - Security vulnerabilities detected
- **Action**: Immediate investigation required
- **Next**: 
  1. Review crash artifacts
  2. Identify root cause
  3. Fix vulnerabilities
  4. Re-run fuzzing to verify fixes

### Crash Artifacts

- **`.fuzz` files**: Raw input that caused crashes
- **`crash_report.txt`**: Detailed crash information
- **Environment details**: System info, versions, timestamps

## 🔧 CI/CD Integration

### GitHub Actions

The fuzz suite automatically runs on:
- **Push to main/develop**: Full fuzzing with artifact collection
- **Pull requests**: Security validation before merge
- **Daily schedule**: Continuous security monitoring
- **Manual trigger**: On-demand security testing

### CI Configuration

```yaml
# Example: Run fuzzing in CI
- name: Run Fuzz Suite
  run: |
    cd fuzz
    FUZZ_DURATION=300 MAX_CRASHES=10 ./run_fuzz_suite.sh
    
- name: Upload Fuzz Results
  uses: actions/upload-artifact@v3
  with:
    name: fuzz-results
    path: fuzz/artifacts/
```

## 🛠️ Troubleshooting

### Common Issues

#### Rust Fuzzing Issues
```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Check Rust toolchain
rustup show

# Verify target support
rustup target list --installed
```

#### Python Fuzzing Issues
```bash
# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r requirements.txt

# Check Atheris installation
python3 -c "import atheris; print(atheris.__version__)"
```

#### Go Fuzzing Issues
```bash
# Install go-fuzz
go install github.com/dvyukov/go-fuzz/go-fuzz@latest
go install github.com/dvyukov/go-fuzz/go-fuzz-build@latest

# Verify Go installation
go version
go env
```

### Performance Optimization

#### Increase Fuzz Duration
```bash
# Run longer fuzzing sessions
FUZZ_DURATION=1800 ./run_fuzz_suite.sh  # 30 minutes per target
```

#### Parallel Execution
```bash
# Run multiple fuzzers in parallel
cd rust
cargo fuzz run --jobs 4 capability_token_format_fuzzer
```

#### Memory and CPU Limits
```bash
# Limit resource usage
ulimit -v 2097152  # 2GB virtual memory
ulimit -t 3600     # 1 hour CPU time
```

## 🔒 Security Considerations

### Fuzzing Best Practices

1. **Isolated Environment**: Run fuzzers in isolated containers
2. **Resource Limits**: Set memory and CPU limits
3. **Crash Collection**: Collect and analyze all crashes
4. **Continuous Testing**: Integrate into development workflow
5. **Vulnerability Tracking**: Track and fix identified issues

### Privacy and Data Handling

- **Crash artifacts** may contain sensitive data
- **Corpora** should not contain production data
- **Reports** should be reviewed before sharing
- **Artifacts** have configurable retention periods

## 📈 Advanced Usage

### Custom Fuzz Targets

#### Adding Rust Fuzzer
```rust
// src/custom_fuzzer.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Your fuzzing logic here
    if let Ok(input) = std::str::from_utf8(data) {
        // Test your function
        let _ = your_function(input);
    }
});
```

#### Adding Python Fuzzer
```python
#!/usr/bin/env python3
import atheris
import sys

def test_function(data):
    # Your fuzzing logic here
    pass

def main():
    atheris.Setup(sys.argv, test_function)
    atheris.Fuzz()

if __name__ == "__main__":
    main()
```

### Corpus Management

#### Seeding Corpus
```bash
# Add test cases to corpus
echo '{"valid": "json"}' > corpus/json_parser_fuzzer/valid.json
echo '{"malformed": "json' > corpus/json_parser_fuzzer/malformed.json
```

#### Corpus Evolution
```bash
# Merge corpora from different runs
cargo fuzz cmin capability_token_format_fuzzer
cargo fuzz merge capability_token_format_fuzzer corpus1 corpus2
```

### Continuous Fuzzing

#### OSS-Fuzz Integration
```yaml
# .clusterfuzzlite/project.yaml
language: rust
fuzzing_engines:
  - libfuzzer
sanitizers:
  - address
  - undefined
```

## 📚 Resources

### Documentation
- [cargo-fuzz Documentation](https://rust-fuzz.github.io/book/)
- [Atheris Documentation](https://github.com/google/atheris)
- [go-fuzz Documentation](https://github.com/dvyukov/go-fuzz)

### Related Tools
- [OSS-Fuzz](https://github.com/google/oss-fuzz): Continuous fuzzing for open source
- [ClusterFuzz](https://github.com/google/clusterfuzz): Scalable fuzzing infrastructure
- [LibFuzzer](https://llvm.org/docs/LibFuzzer.html): In-process fuzzing library

### Security Standards
- [OWASP Fuzzing](https://owasp.org/www-community/controls/Fuzzing)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)

## 🤝 Contributing

### Adding New Fuzz Targets

1. **Identify target function** to fuzz
2. **Create fuzzer** in appropriate language directory
3. **Add to build system** (Cargo.toml, requirements.txt, go.mod)
4. **Update scripts** to include new target
5. **Add tests** for fuzzer functionality
6. **Update documentation** with new target details

### Reporting Issues

- **Bug reports**: Use GitHub Issues
- **Security issues**: Follow security disclosure policy
- **Feature requests**: Submit enhancement proposals
- **Documentation**: Submit pull requests for improvements

## 📄 License

This fuzz suite is part of Polymera OS and follows the same licensing terms.

---

**🚀 Happy Fuzzing!** 🚀

For questions, issues, or contributions, please refer to the main Polymera OS documentation or submit an issue in the repository.
