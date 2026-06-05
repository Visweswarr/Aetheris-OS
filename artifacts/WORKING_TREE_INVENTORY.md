# Polymera OS — Working Tree Inventory

This document lists the inventory of the untracked, modified, or deleted files in the repository as of May 21, 2026. These files are grouped by their respective subsystems.

## 1. Kernel Subsystem (`kernel/`)
* **Status**: 123 untracked or modified files.
* **Key Content**:
  * UEFI boot entries and serial drivers (`kernel/src/boot/`)
  * Capabilities management and message authentication (`kernel/src/secman/`)
  * Paging tables, VM compressor, and huge pages manager (`kernel/src/mm/`)
  * IPC shared memory and message payload routines (`kernel/src/ipc/`)
  * Scheduler jitter tracking and process tables (`kernel/src/process/`)
  * Polyglot syscall handlers (`kernel/src/syscall/handlers/`)
  * eBPF executor and capability enforcement test suites

## 2. Services Subsystem (`services/`)
* **Status**: 111 untracked or modified files.
* **Key Content**:
  * EVM smart contract execution engines (`services/contracts/`)
  * VFS, POSIX IPC, signal propagation, and POSIX process models (`services/posix/`)
  * Network interfaces and custom overlay protocol engines (`services/net/`, `services/polynet/`)
  * Decentralized identities (DIDs) and capabilities for the wallet service (`services/wallet/`)
  * Wasm components, driver bridges, and storage adapters (`services/wasm_components/`, `services/ngfs/`)

## 3. CI/CD Workflows (`.github/`)
* **Status**: 79 untracked or modified files.
* **Key Content**:
  * CI pipeline specifications for the polyglot toolchains, Rust linting, and workflow validations.

## 4. Tooling (`tooling/`)
* **Status**: 54 untracked or modified files.
* **Key Content**:
  * Python and TypeScript validator frameworks for wallets, sensors, devices, net, and XR behaviors.
  * Node.js to POSIX FFI bridge components (`tooling/ts/node-posix-bridge/`).

## 5. Documentation (`docs/`)
* **Status**: 34 untracked or modified files.
* **Key Content**:
  * Overview blueprints for HAL, Web3 integration, advanced networking, and post-quantum cryptography designs.
  * Polyglot shims and reference intake ledgers.

## 6. Scripts (`scripts/`)
* **Status**: 28 untracked or modified files.
* **Key Content**:
  * QEMU boot smoke test triggers (`scripts/qemu-smoke.ps1`)
  * Bootstrapping automation for Windows/Linux host environments.
  * Performance benchmarking and test harness executors.

## 7. Polyglot Interfaces & Standalone Cryptography
* **Status**: Mixed subdirectories.
  * `go/` (20 files): POSIX shim and devctl helper tooling.
  * `crypto/` (12 files): Post-quantum algorithms and liboqs wrappers.
  * `c/` (7 files): Wallet wrappers and standard library interfaces.
  * `contracts/` (4 files): Solidities for DAO governance and resource controls.
  * `zk/` (2 files): Noir constraint systems.

## 8. Frontend / Telemetry UI (`ui/`)
* **Status**: 6 untracked or modified files.
* **Key Content**:
  * Dashboard scripts (`ui/dashboard/kernel_metrics.js`) and UI layouts.

## 9. Verification & Temporary Outputs
* **Status**: Mixed.
  * `tests/` (9 files): Integration testing fixtures for Python, Go, and C services.
  * `tools/` (9 files): Minters, SBOMS, and compliance tools.
  * `artifacts/` (4 files): Verification logs, metrics benchmarks, and final compiled assets.
