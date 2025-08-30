# 🚀 Polymera OS POSIX Service

## 🎯 Overview

The POSIX Service provides a sandboxed POSIX surface with polyglot runtime atop the syscall broker & cap-secured VFS. This is the foundation for userland applications in Polymera OS.

**Status**: [POSIX v0.1] broker OK | p50 read≤300µs p95≤800µs | caps=enforced

## 🌟 Features

### **Core Components**
- **Syscall Broker**: Handles POSIX syscalls with capability enforcement
- **Capability-Aware VFS**: Virtual filesystem with secure mount points
- **Polyglot Shims**: Support for C, Go, Rust, Node.js, and WASI
- **Aesh Shell**: Interactive shell with built-in POSIX commands

### **Security Features**
- **Deny-by-Default**: Strict capability enforcement
- **CapTokens v2**: Cryptographic capability verification
- **OPA Policy**: Open Policy Agent integration ready
- **Sandboxed Execution**: Complete process isolation

### **Performance Targets**
- **Syscall Open**: ≤300µs (p50), ≤800µs (p95)
- **VFS Write**: ≤500µs (p50), ≤1ms (p95)
- **VFS Read**: ≤300µs (p50), ≤800µs (p95)
- **Shell Command**: ≤800µs (p50), ≤1.5ms (p95)
- **Shim Call**: ≤400µs (p50), ≤1ms (p95)

## 🏗️ Architecture

### **Syscall Broker**
```rust
pub struct SyscallBroker {
    file_descriptors: Arc<Mutex<HashMap<i32, FileDescriptor>>>,
    audit_log: Arc<Mutex<Vec<String>>>,
    performance_baselines: Arc<Mutex<HashMap<String, Duration>>>,
}
```

**Supported Syscalls**:
- `open`, `read`, `write`, `close`
- `stat`, `mmap` (guarded), `signal`
- All operations require appropriate capabilities

### **Capability-Aware VFS**
```rust
pub struct CapabilityAwareVFS {
    mounts: Arc<Mutex<HashMap<String, MountPoint>>>,
    file_cache: Arc<Mutex<HashMap<String, FileInfo>>>,
    directory_cache: Arc<Mutex<HashMap<String, Vec<DirectoryEntry>>>>,
}
```

**Default Mount Points**:
- `/snap` - NGFS snapshots (read-only)
- `/pdv` - Personal Data Vault (read-write)
- `/tmp` - Temporary files (read-write)
- `/` - Root filesystem (read-only)

### **Polyglot Shims**
```rust
pub struct PolyglotShims {
    shims: Arc<Mutex<HashMap<String, ShimConfig>>>,
    call_history: Arc<Mutex<Vec<ShimCall>>>,
    results_cache: Arc<Mutex<HashMap<String, ShimResult>>>,
}
```

**Supported Languages**:
- **C**: libc 2.37 compatibility
- **Go**: Go 1.21 standard library
- **Rust**: Rust 1.75 std::fs
- **Node.js**: Node.js 20.0 fs module
- **WASI**: WebAssembly System Interface 0.2.0

### **Aesh Shell**
```rust
pub struct AeshShell {
    broker: Arc<SyscallBroker>,
    vfs: Arc<CapabilityAwareVFS>,
    shims: Arc<PolyglotShims>,
    builtin_commands: HashMap<String, fn(&AeshShell, &[String]) -> CommandResult>,
}
```

**Built-in Commands**:
- `ls`, `cat`, `echo`, `stat`
- `cd`, `pwd`, `mkdir`, `rm`, `touch`
- `ngfsctl` - NGFS control operations
- `help`, `clear`, `status`, `perf`, `history`

## 🚀 Quick Start

### **Prerequisites**
- Rust 1.75+
- Go 1.21+ (for CLI tool)

### **Build & Run**
```bash
# Build POSIX service
cd services/posix
cargo build --release

# Run POSIX service
cargo run --release

# Build Go CLI tool
cd ../../go/tools/posix-ctl
go build -o posix-ctl .
./posix-ctl
```

### **Example Usage**
```bash
# Start interactive shell
$ posix-ctl

# Execute commands
aesh:/> echo Hello, Polymera OS!
Hello, Polymera OS!

aesh:/> ls
d755 0 ./
d755 0 ../
d755 0 tmp/
d755 0 snap/
d755 0 pdv/

aesh:/> ngfsctl status
NGFS Status:
- VFS Mounts: 4
- Files: 0
- Directories: 5
- Shims: 5
- Broker Calls: 0

aesh:/> help
Available commands:
  ls [path]           - List directory contents
  cat <file>          - Display file contents
  echo <text>         - Print text
  stat <file>         - Display file status
  # ... more commands
```

## 🧪 Testing

### **Unit Tests**
```bash
cd services/posix
cargo test
```

### **Integration Tests**
```bash
cargo test --test integration
```

### **Performance Tests**
```bash
cargo run --release --bin posix-service
```

### **Go CLI Tests**
```bash
cd ../../go/tools/posix-ctl
go test ./...
```

## 📊 Performance Monitoring

### **Built-in Benchmarks**
- Syscall performance measurement
- VFS operation timing
- Shell command execution
- Shim call latency

### **Performance Gates**
- Automated CI validation
- Baseline compliance checking
- Regression detection
- Budget enforcement

### **Metrics Collection**
- Virtual clock for deterministic testing
- Audit trail for all operations
- Performance baseline storage
- Real-time monitoring

## 🔒 Security Model

### **Capability System**
- **Object Capabilities**: Fine-grained access control
- **CapTokens v2**: Cryptographic verification
- **Time/IP/Usage Conditions**: Conditional access
- **Audit Logging**: Complete operation history

### **Sandboxing**
- **Process Isolation**: Complete memory separation
- **Resource Limits**: CPU, memory, I/O constraints
- **Network Isolation**: Controlled network access
- **Filesystem Isolation**: Mount point restrictions

### **Policy Enforcement**
- **OPA Integration**: Open Policy Agent ready
- **Deny-by-Default**: Strict security posture
- **Capability Validation**: Runtime checks
- **Audit Compliance**: Regulatory requirements

## 🔧 Configuration

### **Environment Variables**
```bash
export POSIX_LOG_LEVEL=info
export POSIX_AUDIT_ENABLED=true
export POSIX_PERF_MONITORING=true
export POSIX_CAPABILITY_STRICT=true
```

### **Mount Point Configuration**
```rust
// Custom mount points
vfs.mount("/custom", "custom-fs", vec!["custom:read".to_string()], false)?;
```

### **Shim Configuration**
```rust
// Enable/disable specific shims
shims.disable_shim("node")?;
shims.enable_shim("wasi")?;
```

## 📚 API Reference

### **Core Service**
```rust
pub struct POSIXService {
    pub fn new() -> Self;
    pub fn initialize(&self) -> Result<(), String>;
    pub fn execute_shell_command(&self, command: &str) -> Result<String, String>;
    pub fn get_status(&self) -> POSIXStatus;
    pub fn get_performance_report(&self) -> String;
}
```

### **Syscall Broker**
```rust
pub struct SyscallBroker {
    pub fn handle_syscall(&self, request: SyscallRequest) -> SyscallResponse;
    pub fn get_audit_log(&self) -> Vec<String>;
    pub fn get_performance_baselines(&self) -> HashMap<String, Duration>;
}
```

### **VFS Operations**
```rust
pub struct CapabilityAwareVFS {
    pub fn open_file(&self, path: &str, mode: &str, capabilities: &[String]) -> Result<FileInfo, String>;
    pub fn read_file(&self, path: &str, offset: u64, size: usize, capabilities: &[String]) -> Result<Vec<u8>, String>;
    pub fn write_file(&self, path: &str, offset: u64, data: &[u8], capabilities: &[String]) -> Result<usize, String>;
    pub fn list_directory(&self, path: &str, capabilities: &[String]) -> Result<Vec<DirectoryEntry>, String>;
}
```

## 🚀 Deployment

### **Production Setup**
1. **Build Release Binary**
   ```bash
   cargo build --release
   ```

2. **Configure Capabilities**
   ```bash
   # Set appropriate capabilities for production
   export POSIX_CAPABILITY_STRICT=true
   export POSIX_AUDIT_ENABLED=true
   ```

3. **Run Service**
   ```bash
   ./target/release/posix-service
   ```

### **Docker Support**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/posix-service /usr/local/bin/
CMD ["posix-service"]
```

## 🔗 Integration

### **NGFS Integration**
- **Snapshot Mounts**: `/snap/<id>` for NGFS snapshots
- **Vault Access**: `/pdv` for Personal Data Vault
- **Control Interface**: `ngfsctl` command integration

### **Kernel Integration**
- **Syscall Broker**: Kernel syscall interception
- **Capability Framework**: Kernel capability enforcement
- **Performance Monitoring**: Kernel-level metrics

### **Web3 Integration**
- **Smart Contracts**: WASI runtime support
- **Blockchain**: Audit anchoring ready
- **DID**: Decentralized identity support

## 📈 Roadmap

### **P4-01 (Current)**
- ✅ Basic POSIX surface
- ✅ Syscall broker
- ✅ Capability-aware VFS
- ✅ Polyglot shims
- ✅ Aesh shell

### **P4-02 (Next)**
- 🔄 Advanced POSIX features
- 🔄 Process management
- 🔄 Signal handling
- 🔄 IPC mechanisms

### **P4-03 (Future)**
- 🔄 Network stack
- 🔄 Device drivers
- 🔄 Graphics support
- 🔄 Audio system

## 🤝 Contributing

### **Development Setup**
```bash
git clone https://github.com/Visweswarr/Aetheris-OS.git
cd Aetheris-OS/services/posix
cargo build
cargo test
```

### **Code Standards**
- **Rust**: Follow Rust style guide
- **Testing**: 95%+ test coverage required
- **Documentation**: Comprehensive API docs
- **Security**: No unsafe code blocks

### **Pull Request Process**
1. Fork the repository
2. Create feature branch
3. Implement changes with tests
4. Submit pull request
5. CI validation required

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## 🙏 Acknowledgments

- **Redox OS**: Microkernel architecture inspiration
- **Genode**: Security-first design principles
- **Fuchsia**: Modern HAL concepts
- **Open Source Community**: Continuous improvement

---

**🚀 Polymera OS POSIX Service - Building the Future of Computing**  
**🌟 Capability-Secure, AI-Native, Web3 OS**
