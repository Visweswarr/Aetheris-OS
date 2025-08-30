# Polymera OS Contract CI

A comprehensive CI/CD pipeline for smart contract development, testing, and deployment across multiple blockchain ecosystems.

## 🚀 Overview

The Contract CI system provides automated testing, security analysis, gas optimization, and deployment verification for:

- **EVM Contracts**: Solidity smart contracts with Foundry testing
- **CosmWasm Contracts**: Rust-based smart contracts for Cosmos ecosystem
- **Multi-chain Testing**: Cross-platform contract validation
- **Security Scanning**: Automated vulnerability detection
- **Gas Optimization**: Performance regression prevention

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   EVM CI        │    │  CosmWasm CI    │    │   Security CI   │
│                 │    │                 │    │                 │
│  • Foundry      │    │  • Rust Tests   │    │  • Slither      │
│  • Gas Reports  │    │  • Integration  │    │  • Mythril      │
│  • Invariants   │    │  • Schema Gen   │    │  • Foundry      │
│  • Fuzz Tests   │    │  • Size Checks  │    │  • Manual      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Gas CI        │    │  Deployment CI  │    │   Docs CI       │
│                 │    │                 │    │                 │
│  • Regression   │    │  • Local Devnet │    │  • Foundry Docs │
│  • Comparison   │    │  • Contract Deploy│  │  • Rust Docs    │
│  • Thresholds   │    │  • Integration  │    │  • Schema Gen   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📁 Directory Structure

```
contracts/
├── evm/                    # EVM smart contracts
│   ├── foundry.toml       # Foundry configuration
│   ├── src/               # Solidity source files
│   │   └── Token.sol      # Sample ERC20 token
│   ├── test/              # Foundry test files
│   │   └── Token.invariant.t.sol  # Invariant tests
│   ├── script/            # Deployment scripts
│   └── lib/               # Dependencies
├── cosmos/                 # CosmWasm contracts
│   ├── Cargo.toml         # Rust dependencies
│   ├── src/               # Rust source files
│   │   ├── lib.rs         # Main contract logic
│   │   ├── msg.rs         # Message definitions
│   │   ├── state.rs       # State management
│   │   └── error.rs       # Error handling
│   ├── tests/             # Integration tests
│   └── schema/            # Generated schemas
├── .github/workflows/      # GitHub Actions
│   └── contracts.yml      # Main CI workflow
└── scripts/                # CI utilities
    └── compare_gas.py     # Gas comparison script
```

## 🔧 CI/CD Workflow

### Trigger Conditions

The Contract CI workflow triggers on:

- **Push Events**: Changes to `contracts/**/*` files
- **Pull Requests**: Contract modifications and additions
- **Configuration Changes**: Updates to CI configuration files

### Job Matrix

#### EVM Contracts (Foundry)
- **Foundry Tests**: Unit and integration tests
- **Gas Snapshots**: Performance benchmarking
- **Invariant Tests**: State consistency verification
- **Fuzz Tests**: Input validation and edge cases

#### CosmWasm Contracts
- **Unit Tests**: Rust-based contract testing
- **Integration Tests**: Multi-contract interaction testing
- **Schema Generation**: Contract interface documentation

#### Security & Quality
- **Slither Analysis**: Static security analysis
- **Mythril Analysis**: Symbolic execution testing
- **Foundry Security**: Custom security test suites

#### Gas Regression Testing
- **Base Comparison**: Compare against base branch
- **Threshold Detection**: Configurable regression limits
- **Report Generation**: Detailed performance analysis

#### Deployment Testing
- **Local Devnet**: Test deployment on local networks
- **Integration Verification**: End-to-end functionality testing
- **Multi-chain Validation**: Cross-platform compatibility

## 🧪 Testing Framework

### Foundry Testing (EVM)

#### Basic Tests
```bash
# Run all tests
forge test

# Run specific test
forge test --match-test testMint

# Run with gas reporting
forge test --gas-report

# Run with verbosity
forge test --verbosity 2
```

#### Invariant Tests
```bash
# Run invariant tests
forge test --match-contract ".*Invariant.*"

# Run with specific depth
forge test --invariant-depth 20

# Run with custom runs
forge test --invariant-runs 10000
```

#### Fuzz Tests
```bash
# Run fuzz tests
forge test --match-test ".*Fuzz.*"

# Run with custom runs
forge test --fuzz-runs 1000

# Run with seed
forge test --fuzz-seed 12345
```

#### Gas Snapshots
```bash
# Create gas snapshot
forge snapshot

# Check against snapshot
forge snapshot --check

# Update snapshot
forge snapshot --snap .gas-snapshot
```

### CosmWasm Testing (Rust)

#### Unit Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_instantiate

# Run with output
cargo test -- --nocapture
```

#### Integration Tests
```bash
# Run integration tests
cargo test --test integration

# Run with features
cargo test --features integration-tests
```

#### Schema Generation
```bash
# Generate schemas
cargo schema

# Check schema consistency
cargo schema --check
```

## 🔒 Security Testing

### Automated Security Scans

#### Slither Analysis
```bash
# Install Slither
pip3 install slither-analyzer

# Run analysis
slither . --json slither-report.json

# Run with specific detectors
slither . --detect reentrancy-eth,unchecked-transfer
```

#### Mythril Analysis
```bash
# Install Mythril
pip3 install mythril

# Run analysis
myth analyze . --output json --outfile mythril-report.json

# Run with specific modules
myth analyze . --modules reentrancy,integer-overflow
```

#### Foundry Security Tests
```bash
# Run security test suite
forge test --match-contract ".*Security.*"

# Run with gas reporting
forge test --match-contract ".*Security.*" --gas-report
```

### Security Test Categories

1. **Reentrancy Protection**: Function call reentrancy detection
2. **Access Control**: Authorization and permission validation
3. **Integer Overflow**: Arithmetic operation safety
4. **Unchecked External Calls**: External contract interaction safety
5. **Storage Collision**: Storage layout conflict detection
6. **Gas Optimization**: Efficient gas usage patterns

## 📊 Gas Optimization

### Gas Monitoring

#### Gas Reports
```bash
# Generate gas report
forge test --gas-report

# Save gas report to file
forge test --gas-report --out gas-report.txt

# Compare gas usage
forge snapshot --check
```

#### Gas Regression Detection

The CI pipeline automatically:

1. **Generates Base Report**: Creates gas baseline from base branch
2. **Generates Current Report**: Measures current gas usage
3. **Compares Reports**: Detects significant changes
4. **Enforces Thresholds**: Fails CI on regressions > 5%

#### Gas Comparison Script

```bash
# Compare gas reports
python3 scripts/compare_gas.py base-report.txt current-report.txt

# Use custom threshold
python3 scripts/compare_gas.py --threshold 10 base-report.txt current-report.txt

# Save comparison report
python3 scripts/compare_gas.py --output comparison.md base-report.txt current-report.txt
```

### Gas Optimization Techniques

1. **Storage Packing**: Optimize storage layout
2. **Function Visibility**: Minimize external function calls
3. **Loop Optimization**: Reduce loop iterations
4. **Memory Usage**: Minimize memory allocations
5. **External Calls**: Batch external contract calls

## 🚀 Deployment Testing

### Local Devnet Integration

The CI pipeline integrates with the local devnet system:

```bash
# Start local devnet
cd infra/devnets
./up.sh evm

# Deploy contracts
cd contracts/evm
forge script Deploy --rpc-url http://localhost:8545 --broadcast

# Test deployed contracts
forge test --rpc-url http://localhost:8545
```

### Multi-chain Validation

1. **EVM Networks**: Local Anvil, testnets, mainnet
2. **Cosmos Networks**: Local wasmd, testnets, mainnet
3. **Cross-chain**: Interoperability testing

## 📚 Documentation Generation

### Automated Documentation

#### Foundry Docs
```bash
# Generate documentation
forge doc --build

# Serve documentation
forge doc --serve

# Build specific contracts
forge doc --contracts Token
```

#### Rust Docs
```bash
# Generate Rust documentation
cargo doc --no-deps

# Open documentation
cargo doc --open

# Build with specific features
cargo doc --features full
```

#### Schema Generation
```bash
# Generate CosmWasm schemas
cargo schema

# Validate schemas
cargo schema --check

# Generate TypeScript types
cargo schema --ts
```

## 🔧 Configuration

### Foundry Configuration

The `foundry.toml` includes:

- **Compiler Settings**: Solidity version, optimization
- **Testing Profiles**: CI, development, production
- **Gas Settings**: Fuzz runs, invariant depth
- **Network Configuration**: RPC endpoints, API keys
- **Formatting Rules**: Code style enforcement

### Rust Configuration

The `Cargo.toml` includes:

- **Dependencies**: CosmWasm, CW20, testing libraries
- **Build Profiles**: Release, development, testing
- **Features**: Conditional compilation options
- **Metadata**: Contract information and licensing

### CI Configuration

The GitHub Actions workflow includes:

- **Matrix Testing**: Parallel job execution
- **Caching**: Dependency and build caching
- **Artifact Management**: Test results and reports
- **Status Checks**: PR gate enforcement

## 🚨 CI Failure Conditions

### Critical Failures (CI Blocks)

1. **Test Failures**: Any unit or integration test fails
2. **Security Issues**: High-severity vulnerabilities detected
3. **Build Failures**: Contract compilation errors
4. **Schema Errors**: Contract interface generation fails

### Warning Conditions (CI Continues)

1. **Gas Regressions**: Performance degradation warnings
2. **Code Style**: Formatting and linting issues
3. **Documentation**: Missing or incomplete docs
4. **Test Coverage**: Insufficient test coverage

### PR Gate Requirements

For a PR to be merged:

- ✅ All tests must pass
- ✅ Security scans must pass
- ✅ Gas regressions must be within threshold
- ✅ Deployment tests must succeed
- ✅ Documentation must be generated

## 🛠️ Development Workflow

### Local Development

1. **Setup Environment**
   ```bash
   # Install Foundry
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Run Tests Locally**
   ```bash
   # EVM contracts
   cd contracts/evm
   forge test
   
   # CosmWasm contracts
   cd contracts/cosmos
   cargo test
   ```

3. **Check Security**
   ```bash
   # Run security tools
   slither .
   myth analyze .
   ```

### Pre-commit Checks

1. **Format Code**
   ```bash
   # Solidity
   forge fmt
   
   # Rust
   cargo fmt
   ```

2. **Run Linters**
   ```bash
   # Solidity
   forge build
   
   # Rust
   cargo clippy
   cargo check
   ```

3. **Run Tests**
   ```bash
   # Full test suite
   forge test
   cargo test
   ```

## 📈 Performance Monitoring

### Gas Tracking

1. **Baseline Establishment**: Initial gas measurements
2. **Regression Detection**: Automated performance monitoring
3. **Optimization Tracking**: Improvement measurement
4. **Historical Analysis**: Long-term performance trends

### Benchmarking

1. **Function Performance**: Individual function gas usage
2. **Contract Deployment**: Contract creation costs
3. **Storage Operations**: Read/write gas costs
4. **External Calls**: Cross-contract interaction costs

## 🔮 Future Enhancements

### Planned Features

1. **Multi-chain Testing**: Automated cross-platform validation
2. **Advanced Security**: AI-powered vulnerability detection
3. **Performance Profiling**: Detailed gas usage analysis
4. **Automated Optimization**: AI-driven code optimization
5. **Real-time Monitoring**: Live contract performance tracking

### Integration Goals

1. **CI/CD Platforms**: GitHub Actions, GitLab CI, Jenkins
2. **Security Tools**: Additional vulnerability scanners
3. **Testing Frameworks**: Extended test coverage tools
4. **Deployment Platforms**: Multi-chain deployment automation

---

## 🎯 Getting Started

### Quick Start

1. **Clone Repository**
   ```bash
   git clone https://github.com/polymera-os/polymera-os.git
   cd polymera-os
   ```

2. **Install Dependencies**
   ```bash
   # Install Foundry
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Run Tests**
   ```bash
   # EVM contracts
   cd contracts/evm
   forge test
   
   # CosmWasm contracts
   cd contracts/cosmos
   cargo test
   ```

4. **Check CI Status**
   - Push changes to trigger CI
   - Monitor GitHub Actions workflow
   - Review test results and reports

### Next Steps

1. **Review Examples**: Study sample contracts and tests
2. **Customize Configuration**: Modify CI settings for your needs
3. **Add Contracts**: Implement your smart contract logic
4. **Write Tests**: Create comprehensive test coverage
5. **Deploy**: Use local devnets for testing

---

**Happy Contract Development! 🚀**

The Contract CI system provides everything you need for professional smart contract development with automated testing, security analysis, and performance optimization.
