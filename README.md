# Aetheris OS

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Phase](https://img.shields.io/badge/Phase-5-green.svg)](#-development)
[![PQC](https://img.shields.io/badge/Crypto-Post--Quantum-purple.svg)](#-security-first)

A next-generation operating system designed for the quantum era, built with security-first principles, deterministic performance, and zero-knowledge privacy guarantees.

## 🚀 Vision

Aetheris OS combines microkernel architecture with advanced cryptographic primitives to deliver a secure, performant, and verifiable computing platform that's ready for the quantum computing era.

## ✨ Dashboard Mockup

**See the OS architecture visualized!** Open the interactive kernel dashboard to explore process management, IPC flow, memory maps, PQC crypto status, and more:

```
ui/dashboard/index.html
```

Simply open this file in any modern browser — no build step required. The dashboard visualizes the kernel's data structures and architecture with simulated data (a live telemetry connection to the supervisor/AI core event streams is planned for Phase 5-B):
- 📊 **Process Monitor** — Live process table with spawn/kill controls
- 💬 **IPC Channel Flow** — Animated message routing between services
- 🧠 **Memory Map** — Visual kernel/user space layout with LZ4 compression stats
- 🔐 **PQC Crypto Status** — Dilithium signatures, Kyber KEM, BLAKE3 hashing metrics
- 🌳 **Supervisor Tree** — OTP-style service supervision hierarchy
- ⚡ **SLO Gauges** — Real-time performance compliance monitoring

## 🔐 Security First

- **Post-Quantum Cryptography (PQC)**: CRYSTALS-Dilithium (signatures) + CRYSTALS-Kyber (KEM)
- **BLAKE3 Hashing**: Fast cryptographic hashing with key derivation
- **Zero-Knowledge Proofs (ZK)**: Privacy-preserving operations and verifications
- **Memory Safety**: Rust-based implementation with formal verification
- **Side-Channel Resistance**: Constant-time operations, no timing leaks

## ⚡ Performance Guarantees

- **Kernel Operations**: <100μs for basic operations
- **Service Calls**: <1ms for standard operations
- **Application Startup**: <10ms for WASI applications
- **Deterministic Latency**: ±5% variance in operation timing

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Aetheris OS Stack                       │
├─────────────────────────────────────────────────────────────┤
│  Applications (WASI, CPython, JVM, CLR)                   │
├─────────────────────────────────────────────────────────────┤
│  Runtime Layer (WASI Host, Language Bridges)              │
├─────────────────────────────────────────────────────────────┤
│  Service Layer (DeviceKit, PolyAudio, PolyNet, etc.)      │
├─────────────────────────────────────────────────────────────┤
│  Kernel Layer (PolymeraCore, PolyBus, Process Mgr)       │
├─────────────────────────────────────────────────────────────┤
│  Hardware Abstraction (Secure Boot, TPM, Quantum RNG)     │
└─────────────────────────────────────────────────────────────┘
```

## 📁 Project Structure

```
polymera-os/
├── kernel/           # PolymeraCore microkernel (Rust, no_std)
│   └── src/
│       ├── process/  # Process management (PCB, lifecycle, table)
│       ├── ipc/      # Inter-process communication (channels, streams)
│       ├── mm/       # Memory management (paging, allocator, LZ4)
│       ├── sched/    # Scheduler (CFS + RT queues, NUMA)
│       ├── crypto/   # PQC crypto (Dilithium, Kyber, BLAKE3)
│       └── ...       # eBPF, capabilities, syscalls, etc.
├── services/         # System services (34 modules)
│   ├── ai_core/      # AI/ML runtime
│   ├── supervisor/   # OTP-style supervisor trees
│   ├── compositor/   # Spectra display compositor
│   └── ...
├── ui/               # User interface
│   ├── dashboard/    # Interactive kernel dashboard (HTML/CSS/JS)
│   └── components/   # React UI components
├── docs/             # Documentation
├── scripts/          # Build & verification scripts
└── wit/              # WebAssembly Interface Types
```

## 🚀 Quick Start

### 1. View the Dashboard (No Setup Required)
```bash
# Open the kernel dashboard in your browser
start ui/dashboard/index.html    # Windows
open ui/dashboard/index.html     # macOS
xdg-open ui/dashboard/index.html # Linux
```

### 2. Build the Kernel
```bash
cd kernel
cargo build
```

### 3. Run Verification (WSL/Linux)
```bash
bash scripts/phase4-verify.sh
```

### Prerequisites

- **Rust 1.75+** (nightly for kernel features)
- **Go 1.21+** — CLI tools and integration
- **Node.js 18+** — TypeScript bridges and tooling
- **Python 3.11+** — Validation and testing tools

## 🔬 Development

### Current Status

**Phase 5 — Advanced AI & Polyglot Architecture: IN PROGRESS 🚧**

Completed foundations:

- ✅ **Microkernel Core**: Process management, IPC, memory manager, scheduler
- ✅ **PQC Cryptography**: Dilithium + Kyber via standalone `crypto/` crate (liboqs FFI)
- ✅ **Syscall Dispatch**: 16-syscall dispatch table with capability enforcement
- ✅ **Supervisor Trees**: Erlang/OTP-style service supervision
- ✅ **Dashboard Mockup**: Kernel visualization with simulated data (`ui/dashboard/`)
- 🚧 **WASM Component Model**: WIT-based universal app ABI
- 🚧 **Formal Verification**: F*/Lean 4 for crypto proofs
- 🚧 **Python Runtime**: Embedded CPython with IPC bindings
- 🚧 **Dashboard Telemetry**: Wire dashboard to real supervisor/AI core event streams

### Polyglot Architecture

| Language | Domain | Role |
|----------|--------|------|
| **Rust** | Kernel, Core Services | Memory safety, zero-cost abstractions |
| **Go** | CLI, Networking, FS | Stdlib networking, concurrent services |
| **C** | Libc, Hardware | Universal ABI, driver interfaces |
| **TypeScript** | UI, Tooling | Dashboard, dev tools |
| **Zig** | Systems (migration) | C replacement with safety guarantees |

## 📚 Documentation

- [**SPEC.md**](SPEC.md) — System specification and requirements
- [**DESIGN.md**](DESIGN.md) — Technical architecture and design
- [**TASKS.md**](TASKS.md) — Development roadmap and epics
- [**LANGUAGES.md**](LANGUAGES.md) — Language charter and governance
- [**SECURITY.md**](SECURITY.md) — Security policy and reporting

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for our community standards.

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **GitHub**: https://github.com/Visweswarr/polymera-os
- **Issues**: https://github.com/Visweswarr/polymera-os/issues
- **Discussions**: https://github.com/Visweswarr/polymera-os/discussions

---

**Polymera OS v0.5.0** — Quantum-ready, secure, deterministic. 🚀
