# Polymera OS Kernel Fuzzing

This directory contains comprehensive fuzzing targets for the Polymera OS kernel using `cargo-fuzz` and libFuzzer.

## 🎯 **Overview**

The fuzzing system provides three main targets that test different aspects of the kernel:

1. **`fuzz_inbox`** - Tests IPC inbox operations with random message sizes and properties
2. **`fuzz_caps`** - Tests capability system with random fields, expiries, and operations
3. **`fuzz_vm`** - Tests virtual memory operations with map/unmap sequences

## 🚀 **Quick Start**

### Prerequisites

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Verify installation
cargo fuzz --version
```

### Building Fuzz Targets

```bash
cd kernel/fuzz

# Build all targets
cargo fuzz build

# Build specific target
cargo fuzz build fuzz_inbox
cargo fuzz build fuzz_caps
cargo fuzz build fuzz_vm
```

### Running Fuzzing

```bash
# Run inbox fuzzing (120 seconds)
timeout 120s cargo fuzz run fuzz_inbox

# Run capabilities fuzzing (120 seconds)
timeout 120s cargo fuzz run fuzz_caps

# Run virtual memory fuzzing (120 seconds)
timeout 120s cargo fuzz run fuzz_vm
```

## 📊 **Fuzzing Targets**

### 1. **fuzz_inbox** - IPC Inbox Operations

**Purpose**: Tests IPC message handling with random sizes, priorities, and properties.

**Test Coverage**:
- **Push Operations**: Random message sizes (1-1024 bytes)
- **Pop Operations**: Message retrieval and integrity verification
- **Mixed Operations**: Combined push/pop sequences
- **Priority Handling**: Low, Normal, High, Critical priorities
- **Message Types**: Request, Response, Notification
- **Flags**: URGENT, RELIABLE, BROADCAST
- **Overflow Handling**: Inbox capacity and overflow policies

**Input Format** (8+ bytes):
```
Byte 0: Operation type (0=push, 1=pop, 2=mixed)
Byte 1: Number of operations (1-50)
Byte 2: Message size range (1-1024 bytes)
Byte 3: Priority distribution
Byte 4: Message type distribution
Byte 5: Flags distribution
Byte 6: Expiry time range (0-60 seconds)
Byte 7: Sender/receiver ID range
Bytes 8+: Additional fuzz data
```

**Example Seeds**:
- `seed_001`: Basic push operations
- `seed_002`: Pop operations with validation
- `seed_003`: Mixed operations with overflow testing

### 2. **fuzz_caps** - Capability System

**Purpose**: Tests capability creation, validation, revocation, and delegation.

**Test Coverage**:
- **Capability Creation**: Random types, flags, and permissions
- **Validation**: Expiry checks, ownership verification
- **Revocation**: Policy enforcement and cascading effects
- **Delegation**: Depth limits and transfer restrictions
- **Audit Logging**: Comprehensive event tracking

**Input Format** (12+ bytes):
```
Byte 0: Operation type (0=create, 1=validate, 2=revoke, 3=mixed)
Byte 1: Number of operations (1-100)
Byte 2: Capability type distribution
Byte 3: Flags distribution
Byte 4: Expiry time range (0-3600 seconds)
Byte 5: Process ID range (1-1000)
Byte 6: Resource ID range (1-10000)
Byte 7: Capability ID range (1-100000)
Byte 8: Permission bits distribution
Byte 9: Delegation depth (0-10)
Byte 10: Revocation policy
Byte 11: Audit level
Bytes 12+: Additional fuzz data
```

**Example Seeds**:
- `seed_001`: Basic capability creation
- `seed_002`: Validation operations
- `seed_003`: Mixed operations with delegation

### 3. **fuzz_vm** - Virtual Memory Operations

**Purpose**: Tests virtual memory mapping, unmapping, and stress scenarios.

**Test Coverage**:
- **Memory Mapping**: Various page sizes (4K, 2M, 1G)
- **Address Alignment**: Different alignment requirements
- **Memory Flags**: READ, WRITE, EXECUTE, USER, GLOBAL, NO_CACHE
- **Mapping Strategies**: Sequential, random, overlapping, sparse
- **Unmap Strategies**: Individual, range, all, selective
- **Stress Testing**: Error injection and fragmentation
- **Cache Behavior**: Normal, aggressive, conservative
- **TLB Management**: Normal, flush always, selective

**Input Format** (16+ bytes):
```
Byte 0: Operation type (0=map_only, 1=unmap_only, 2=mixed, 3=stress)
Byte 1: Number of operations (1-200)
Byte 2: Page size distribution
Byte 3: Memory flags distribution
Byte 4: Address alignment (0=4K, 1=2M, 2=1G, 3=any)
Byte 5: Memory size range (1-1000 pages)
Byte 6: Address space distribution
Byte 7: Protection level distribution
Byte 8: Mapping strategy (0=sequential, 1=random, 2=overlapping, 3=sparse)
Byte 9: Unmap strategy (0=individual, 1=range, 2=all, 3=selective)
Byte 10: Stress level (0=none, 1=low, 2=medium, 3=high)
Byte 11: Fragmentation level (0=none, 1=low, 2=medium, 3=high)
Byte 12: Cache behavior (0=normal, 1=aggressive, 2=conservative)
Byte 13: TLB behavior (0=normal, 1=flush_always, 2=selective)
Byte 14: Error injection probability (0-100%)
Byte 15: Validation frequency (0=never, 1=rare, 2=sometimes, 3=always)
Bytes 16+: Additional fuzz data
```

**Example Seeds**:
- `seed_001`: Basic memory mapping
- `seed_002`: Unmap operations
- `seed_003`: Stress testing with error injection

## 🔧 **Configuration**

### Fuzzing Parameters

Each target supports configurable parameters:

```bash
# Set maximum input length
cargo fuzz run fuzz_inbox -- -max_len=1024

# Set timeout per input
cargo fuzz run fuzz_inbox -- -timeout=10

# Set maximum runs
cargo fuzz run fuzz_inbox -- -runs=10000

# Enable sanitizers
cargo fuzz run fuzz_inbox -- -sanitizer=address
```

### Build Profiles

The fuzzing targets use optimized build profiles:

```toml
[profile.release]
debug = true              # Enable debug symbols
overflow-checks = true    # Enable overflow checking
lto = true               # Link time optimization
codegen-units = 1        # Single codegen unit
panic = "abort"          # Abort on panic

[profile.dev]
debug = true              # Enable debug symbols
overflow-checks = true    # Enable overflow checking
panic = "abort"          # Abort on panic
```

## 📁 **Corpus Management**

### Seed Corpora

Each target has its own seed corpus:

```
kernel/fuzz/corpus/
├── fuzz_inbox/          # Inbox operation seeds
│   ├── seed_001         # Basic push operations
│   ├── seed_002         # Pop operations
│   └── seed_003         # Mixed operations
├── fuzz_caps/           # Capability operation seeds
│   ├── seed_001         # Basic capability creation
│   ├── seed_002         # Validation operations
│   └── seed_003         # Mixed operations
└── fuzz_vm/             # Virtual memory operation seeds
    ├── seed_001         # Basic memory mapping
    ├── seed_002         # Unmap operations
    └── seed_003         # Stress testing
```

### Adding New Seeds

```bash
# Generate new seed from existing input
cargo fuzz run fuzz_inbox -- -merge=1 corpus/fuzz_inbox/

# Add specific input as seed
cp interesting_input corpus/fuzz_inbox/seed_004

# Minimize corpus
cargo fuzz run fuzz_inbox -- -merge=1 corpus/fuzz_inbox/
```

## 🚨 **Crash Handling**

### Crash Detection

Fuzzing targets automatically detect crashes:

- **Panics**: Rust panic handling
- **Assertions**: Failed assertions
- **Memory Errors**: Address sanitizer violations
- **Timeouts**: Input processing timeouts

### Crash Artifacts

When crashes occur:

```bash
# Crashes are stored in target-specific directories
kernel/fuzz/
├── crashes/
│   ├── inbox/           # Inbox fuzzing crashes
│   ├── caps/            # Capability fuzzing crashes
│   └── vm/              # Virtual memory fuzzing crashes
```

### Crash Analysis

```bash
# Reproduce crash
cargo fuzz run fuzz_inbox crash-*

# Debug with gdb
gdb --args target/x86_64-unknown-linux-gnu/release/fuzz_inbox crash-*

# Analyze with valgrind
valgrind --tool=memcheck target/x86_64-unknown-linux-gnu/release/fuzz_inbox crash-*
```

## 🔄 **CI Integration**

### Nightly Fuzzing

Automated nightly fuzzing runs via GitHub Actions:

- **Schedule**: Daily at 2 AM UTC
- **Duration**: 120 seconds per target
- **Artifacts**: Crash files and corpus snapshots
- **Retention**: Crashes (30 days), Corpus (7 days)

### Manual Triggering

```bash
# Trigger fuzzing workflow manually
gh workflow run fuzzing-nightly.yml

# Check workflow status
gh run list --workflow=fuzzing-nightly.yml
```

## 📈 **Performance Tuning**

### Optimization Strategies

1. **Input Length**: Balance coverage vs performance
2. **Timeout Settings**: Prevent hanging on complex inputs
3. **Corpus Size**: Maintain diverse but manageable seed sets
4. **Build Optimization**: Use release builds with debug symbols

### Monitoring

```bash
# Track fuzzing progress
cargo fuzz run fuzz_inbox -- -print_final_stats=1

# Monitor coverage
cargo fuzz run fuzz_inbox -- -use_value_profile=1

# Analyze performance
cargo fuzz run fuzz_inbox -- -print_pcs=1
```

## 🐛 **Debugging**

### Common Issues

1. **Build Failures**: Check Rust toolchain and dependencies
2. **Runtime Crashes**: Verify kernel initialization
3. **Performance Issues**: Adjust timeout and input length limits
4. **Memory Issues**: Use address sanitizer for detection

### Debug Builds

```bash
# Build with debug information
cargo fuzz build fuzz_inbox --debug

# Run with verbose output
RUST_LOG=debug cargo fuzz run fuzz_inbox
```

## 🔮 **Future Enhancements**

### Planned Features

1. **Additional Targets**: Network, filesystem, scheduler fuzzing
2. **Advanced Strategies**: Grammar-based, mutation-based fuzzing
3. **Coverage Analysis**: Real-time coverage tracking
4. **Regression Testing**: Automated crash reproduction

### Integration Goals

1. **CI Gates**: Block merges on fuzzing failures
2. **Performance Regression**: Detect performance degradation
3. **Security Scanning**: Automated vulnerability detection
4. **Documentation**: Auto-generated fuzzing reports

## 📚 **References**

- [cargo-fuzz Documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [Rust Fuzzing Book](https://rust-fuzz.github.io/book/)
- [Fuzzing Best Practices](https://github.com/google/fuzzing)

---

**Maintenance**: @polymera-os-team  
**Last Updated**: December 2024  
**Next Review**: January 2025
