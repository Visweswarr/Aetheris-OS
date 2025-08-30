#!/bin/bash

# Universal QEMU Runner for Polymera OS
# Automatically detects architecture and runs appropriate QEMU instance

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect system architecture
detect_architecture() {
    local arch
    arch=$(uname -m)
    
    case $arch in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        armv7l|armv8l)
            echo "aarch64"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            log_info "Supported architectures: x86_64, aarch64"
            exit 1
            ;;
    esac
}

# Check QEMU availability for architecture
check_qemu() {
    local arch=$1
    
    case $arch in
        x86_64)
            if ! command -v qemu-system-x86_64 &> /dev/null; then
                log_error "qemu-system-x86_64 not found. Please install QEMU."
                exit 1
            fi
            ;;
        aarch64)
            if ! command -v qemu-system-aarch64 &> /dev/null; then
                log_error "qemu-system-aarch64 not found. Please install QEMU."
                exit 1
            fi
            ;;
    esac
    
    log_success "QEMU for $arch is available"
}

# Run architecture-specific QEMU
run_qemu() {
    local arch=$1
    shift
    
    case $arch in
        x86_64)
            log_info "Running x86_64 QEMU..."
            exec "$SCRIPT_DIR/run_x86_64.sh" "$@"
            ;;
        aarch64)
            log_info "Running aarch64 QEMU..."
            exec "$SCRIPT_DIR/run_aarch64.sh" "$@"
            ;;
        *)
            log_error "Unknown architecture: $arch"
            exit 1
            ;;
    esac
}

# Show help
show_help() {
    cat << EOF
Universal QEMU Runner for Polymera OS

Usage: $0 [OPTIONS] --kernel <kernel_image>

This script automatically detects your system architecture and runs the appropriate
QEMU instance with OVMF UEFI firmware support.

Options:
    -k, --kernel <file>       Kernel image to boot (required)
    -d, --disk <file>         Disk image to attach
    -m, --memory <size>       Memory size (default: 2G)
    -c, --cpus <number>       Number of CPUs (default: 2)
    -s, --serial <file>       Serial log file
    -g, --graphics            Enable graphics (default: headless)
    -n, --network             Enable network
    -D, --debug               Enable debug mode
    -a, --args <args>         Additional QEMU arguments
    -h, --help                Show this help message

Examples:
    # Basic kernel boot (auto-detects architecture)
    $0 --kernel kernel.elf

    # With disk image and serial logging
    $0 --kernel kernel.elf --disk disk.img --serial serial.log

    # With graphics and network
    $0 --kernel kernel.elf --graphics --network

    # Custom QEMU arguments
    $0 --kernel kernel.elf --args "-device virtio-rng-pci"

Architecture Detection:
    - x86_64/amd64: Uses qemu-system-x86_64 with Q35 machine
    - aarch64/arm64: Uses qemu-system-aarch64 with virt machine

EOF
}

# Main execution
main() {
    log_info "Polymera OS Universal QEMU Runner"
    log_info "================================="
    
    # Parse command line arguments
    local args=()
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                args+=("$1")
                shift
                ;;
        esac
    done
    
    # Detect architecture
    local arch
    arch=$(detect_architecture)
    log_success "Detected architecture: $arch"
    
    # Check QEMU availability
    check_qemu "$arch"
    
    # Run QEMU with detected architecture
    run_qemu "$arch" "${args[@]}"
}

# Run main function
main "$@"
