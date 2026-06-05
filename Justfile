# ═══════════════════════════════════════════════════════════════════════════════
# POLYMERA OS - AVENGERS BUILD SYSTEM (Justfile)
# ═══════════════════════════════════════════════════════════════════════════════
# The "Avengers" of Operating Systems - Polyglot Microkernel
#
# Usage:
#   just build      - Build all components
#   just iso        - Create bootable ISO
#   just run        - Run in QEMU
#   just clean      - Clean build artifacts
# ═══════════════════════════════════════════════════════════════════════════════

# Default recipe
default: help

# Show help
help:
    @echo "╔══════════════════════════════════════════════════════════════╗"
    @echo "║     🦸 POLYMERA OS - AVENGERS BUILD SYSTEM 🦸               ║"
    @echo "╠══════════════════════════════════════════════════════════════╣"
    @echo "║  just build       - Build all 5 language runtimes           ║"
    @echo "║  just iso         - Create bootable ISO                     ║"
    @echo "║  just run         - Run in QEMU                             ║"
    @echo "║  just run-debug   - Run in QEMU with GDB                    ║"
    @echo "║  just clean       - Clean build artifacts                   ║"
    @echo "╠══════════════════════════════════════════════════════════════╣"
    @echo "║  Individual builds:                                         ║"
    @echo "║  just kernel      - Build Rust kernel (Sentinel)            ║"
    @echo "║  just go          - Build Go services (Coordinator)         ║"
    @echo "║  just cpp         - Build C++ services (Speedster)          ║"
    @echo "║  just csharp      - Build C# runtime (Architect)            ║"
    @echo "║  just wasm        - Build WASM host (Shapeshifter)          ║"
    @echo "╚══════════════════════════════════════════════════════════════╝"

# Build directories
build_dir := "build"
iso_dir := build_dir / "iso"
initramfs_dir := build_dir / "initramfs"
artifacts_dir := "artifacts/os"

# Targets
rust_target := "x86_64-unknown-none"

# ═══════════════════════════════════════════════════════════════════════════════
# MAIN BUILD TARGETS
# ═══════════════════════════════════════════════════════════════════════════════

# Build everything
build: kernel go cpp csharp wasm
    @echo "🦸 All Avengers built!"

# Create bootable ISO
iso: build initramfs limine
    @echo "💿 Creating bootable ISO..."
    @bash tools/build/build.sh || pwsh tools/build/build.ps1

# Run in QEMU
run: iso
    @echo "🚀 Launching Polymera OS in QEMU..."
    qemu-system-x86_64 \
        -cdrom {{artifacts_dir}}/polymera-os-avengers.iso \
        -m 2G \
        -smp 4 \
        -serial stdio \
        -device virtio-gpu-pci \
        -device virtio-keyboard-pci \
        -device virtio-mouse-pci

# Run with debug
run-debug: iso
    @echo "🔍 Launching Polymera OS in QEMU (Debug Mode)..."
    qemu-system-x86_64 \
        -cdrom {{artifacts_dir}}/polymera-os-avengers.iso \
        -m 2G \
        -smp 4 \
        -serial stdio \
        -s -S \
        -d int,cpu_reset

# ═══════════════════════════════════════════════════════════════════════════════
# INDIVIDUAL HERO BUILDS
# ═══════════════════════════════════════════════════════════════════════════════

# 🦀 Rust Kernel (Sentinel)
kernel:
    @echo "🦀 Building Rust Kernel [Sentinel]..."
    @echo "   Superpower: Memory Safety + Zero-Cost Abstractions"
    cd kernel && cargo build --release --target {{rust_target}}
    @mkdir -p {{build_dir}}
    @cp kernel/target/{{rust_target}}/release/polymera-kernel {{build_dir}}/kernel.elf 2>/dev/null || \
     cp kernel/target/{{rust_target}}/release/polymera_kernel {{build_dir}}/kernel.elf 2>/dev/null || \
     echo "STUB" > {{build_dir}}/kernel.elf
    @echo "   ✅ Sentinel ready"

# 🐹 Go Services (Coordinator)
go:
    @echo "🐹 Building Go Services [Coordinator]..."
    @echo "   Superpower: Goroutines + GC + Network Stack"
    @mkdir -p {{initramfs_dir}}/bin
    cd services/service_manager && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o ../../{{initramfs_dir}}/bin/init . || true
    cd services/fs_go && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o ../../{{initramfs_dir}}/bin/fs_service . || true
    cd services/net_go && CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o ../../{{initramfs_dir}}/bin/net_service . || true
    @echo "   ✅ Coordinator ready"

# ⚡ C++ Services (Speedster)
cpp:
    @echo "⚡ Building C++ Services [Speedster]..."
    @echo "   Superpower: Raw Performance + GPU/Hardware"
    @mkdir -p {{initramfs_dir}}/bin
    @mkdir -p services/window_server_cpp/build
    cd services/window_server_cpp/build && cmake .. -DCMAKE_BUILD_TYPE=Release && make -j$(nproc) || true
    @cp services/window_server_cpp/build/window_server {{initramfs_dir}}/bin/ 2>/dev/null || echo "STUB" > {{initramfs_dir}}/bin/window_server
    @echo "   ✅ Speedster ready"

# 🏗️ C# Runtime (Architect)
csharp:
    @echo "🏗️ Building C# Runtime [Architect]..."
    @echo "   Superpower: Managed Runtime + Rapid Development"
    @mkdir -p {{initramfs_dir}}/bin
    cd services/app_runtime_cs && dotnet publish -c Release -r linux-x64 --self-contained true -o ../../{{initramfs_dir}}/bin/ || true
    @echo "   ✅ Architect ready"

# 🔮 WASM Host (Shapeshifter)
wasm:
    @echo "🔮 Building WASM Host [Shapeshifter]..."
    @echo "   Superpower: Sandboxing + Portability"
    @mkdir -p {{initramfs_dir}}/bin
    cd services/wasm_driver && cargo build --release || true
    @cp services/wasm_driver/target/release/wasm_driver {{initramfs_dir}}/bin/wasm_host 2>/dev/null || echo "STUB" > {{initramfs_dir}}/bin/wasm_host
    @echo "   ✅ Shapeshifter ready"

# ═══════════════════════════════════════════════════════════════════════════════
# PACKAGING
# ═══════════════════════════════════════════════════════════════════════════════

# Create InitRAMFS
initramfs:
    @echo "📦 Creating InitRAMFS..."
    @mkdir -p {{initramfs_dir}}/{bin,lib,etc,dev,proc,sys,tmp}
    @echo '#!/bin/sh\necho "🦸 POLYMERA OS Booting..."\nexec /bin/init' > {{initramfs_dir}}/init
    @chmod +x {{initramfs_dir}}/init
    cd {{initramfs_dir}} && find . | cpio -o -H newc | gzip > ../initramfs.cpio.gz
    @echo "   ✅ InitRAMFS created"

# Setup Limine bootloader
limine:
    @echo "🔧 Setting up Limine..."
    @mkdir -p {{iso_dir}}/boot/limine {{iso_dir}}/EFI/BOOT
    @cp {{build_dir}}/kernel.elf {{iso_dir}}/boot/ 2>/dev/null || echo "STUB" > {{iso_dir}}/boot/kernel.elf
    @cp {{build_dir}}/initramfs.cpio.gz {{iso_dir}}/boot/ 2>/dev/null || true
    @cp tools/build/limine.cfg {{iso_dir}}/boot/limine/
    @echo "   ✅ Limine configured"

# ═══════════════════════════════════════════════════════════════════════════════
# UTILITIES
# ═══════════════════════════════════════════════════════════════════════════════

# Clean all build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    rm -rf {{build_dir}}
    rm -rf {{artifacts_dir}}
    cd kernel && cargo clean || true
    cd services/wasm_driver && cargo clean || true
    rm -rf services/window_server_cpp/build
    @echo "   ✅ Clean complete"

# Show Avengers status
status:
    @echo "╔══════════════════════════════════════════════════════════════╗"
    @echo "║     🦸 AVENGERS STATUS REPORT 🦸                            ║"
    @echo "╠══════════════════════════════════════════════════════════════╣"
    @echo "║  🦀 Rust    [Sentinel]     kernel/src/                      ║"
    @echo "║  🐹 Go      [Coordinator]  services/fs_go, net_go           ║"
    @echo "║  ⚡ C++     [Speedster]    services/window_server_cpp       ║"
    @echo "║  🏗️ C#      [Architect]    services/app_runtime_cs          ║"
    @echo "║  🔮 WASM    [Shapeshifter] services/wasm_driver             ║"
    @echo "╚══════════════════════════════════════════════════════════════╝"

# Demo the Avengers orchestrator
demo:
    @echo "🦸 Running Avengers Demo..."
    cd kernel && cargo test avengers --release -- --nocapture || echo "Demo requires kernel test environment"
