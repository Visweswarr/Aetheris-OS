#!/bin/bash

# QEMU Runner for aarch64 with OVMF UEFI Firmware
# Polymera OS Kernel Testing Environment

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
KERNEL_DIR="$PROJECT_ROOT/kernel"
BUILD_DIR="$PROJECT_ROOT/build"
OVMF_DIR="$SCRIPT_DIR/ovmf"

# Default values
KERNEL_IMAGE=""
DISK_IMAGE=""
MEMORY="2G"
CPUS="2"
SERIAL_LOG=""
ENABLE_GRAPHICS=false
ENABLE_NETWORK=false
ENABLE_DEBUG=false
QEMU_ARGS=""

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

# Help function
show_help() {
    cat << EOF
QEMU Runner for aarch64 with OVMF UEFI Firmware

Usage: $0 [OPTIONS] --kernel <kernel_image>

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
    # Basic kernel boot
    $0 --kernel kernel.elf

    # With disk image and serial logging
    $0 --kernel kernel.elf --disk disk.img --serial serial.log

    # With graphics and network
    $0 --kernel kernel.elf --graphics --network

    # Custom QEMU arguments
    $0 --kernel kernel.elf --args "-device virtio-rng-pci"

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -k|--kernel)
            KERNEL_IMAGE="$2"
            shift 2
            ;;
        -d|--disk)
            DISK_IMAGE="$2"
            shift 2
            ;;
        -m|--memory)
            MEMORY="$2"
            shift 2
            ;;
        -c|--cpus)
            CPUS="$2"
            shift 2
            ;;
        -s|--serial)
            SERIAL_LOG="$2"
            shift 2
            ;;
        -g|--graphics)
            ENABLE_GRAPHICS=true
            shift
            ;;
        -n|--network)
            ENABLE_NETWORK=true
            shift
            ;;
        -D|--debug)
            ENABLE_DEBUG=true
            shift
            ;;
        -a|--args)
            QEMU_ARGS="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$KERNEL_IMAGE" ]]; then
    log_error "Kernel image is required. Use --kernel <file>"
    show_help
    exit 1
fi

# Check if kernel image exists
if [[ ! -f "$KERNEL_IMAGE" ]]; then
    log_error "Kernel image not found: $KERNEL_IMAGE"
    exit 1
fi

# Resolve relative paths
if [[ ! "$KERNEL_IMAGE" = /* ]]; then
    KERNEL_IMAGE="$(cd "$(dirname "$KERNEL_IMAGE")" && pwd)/$(basename "$KERNEL_IMAGE")"
fi

if [[ -n "$DISK_IMAGE" && ! "$DISK_IMAGE" = /* ]]; then
    DISK_IMAGE="$(cd "$(dirname "$DISK_IMAGE")" && pwd)/$(basename "$DISK_IMAGE")"
fi

# Check OVMF firmware
check_ovmf() {
    log_info "Checking OVMF firmware..."
    
    # Look for OVMF in common locations
    OVMF_PATHS=(
        "$OVMF_DIR/aarch64/OVMF_CODE.fd"
        "$OVMF_DIR/aarch64/OVMF.fd"
        "/usr/share/ovmf/OVMF_AA64.fd"
        "/usr/share/ovmf/OVMF_AA64_CODE.fd"
        "/usr/share/edk2/ovmf/OVMF_AA64.fd"
        "/usr/share/edk2/ovmf/OVMF_AA64_CODE.fd"
    )
    
    OVMF_CODE=""
    for path in "${OVMF_PATHS[@]}"; do
        if [[ -f "$path" ]]; then
            OVMF_CODE="$path"
            break
        fi
    done
    
    if [[ -z "$OVMF_CODE" ]]; then
        log_error "OVMF firmware not found. Please install OVMF or set OVMF_DIR."
        log_info "You can download OVMF from: https://github.com/edk2/edk2/releases"
        exit 1
    fi
    
    log_success "Found OVMF firmware: $OVMF_CODE"
    
    # Look for OVMF variables
    OVMF_VARS_PATHS=(
        "$OVMF_DIR/aarch64/OVMF_VARS.fd"
        "$OVMF_DIR/working/OVMF_VARS.fd"
        "/usr/share/ovmf/OVMF_AA64_VARS.fd"
        "/usr/share/edk2/ovmf/OVMF_AA64_VARS.fd"
    )
    
    OVMF_VARS=""
    for path in "${OVMF_VARS_PATHS[@]}"; do
        if [[ -f "$path" ]]; then
            OVMF_VARS="$path"
            break
        fi
    done
    
    if [[ -z "$OVMF_VARS" ]]; then
        log_warning "OVMF variables not found, will create new ones"
        OVMF_VARS="$OVMF_DIR/working/OVMF_VARS.fd"
        mkdir -p "$(dirname "$OVMF_VARS")"
        # Create empty variables file
        dd if=/dev/zero of="$OVMF_VARS" bs=1M count=64 2>/dev/null || true
    else
        log_success "Found OVMF variables: $OVMF_VARS"
    fi
}

# Build QEMU command
build_qemu_command() {
    log_info "Building QEMU command..."
    
    # Base QEMU command
    QEMU_CMD="qemu-system-aarch64"
    
    # Architecture and machine
    QEMU_CMD="$QEMU_CMD -machine virt,accel=kvm:tcg"
    QEMU_CMD="$QEMU_CMD -cpu cortex-a72"
    QEMU_CMD="$QEMU_CMD -smp $CPUS"
    QEMU_CMD="$QEMU_CMD -m $MEMORY"
    
    # UEFI firmware
    QEMU_CMD="$QEMU_CMD -firmware $OVMF_CODE"
    QEMU_CMD="$QEMU_CMD -drive file=$OVMF_VARS,if=pflash,format=raw,readonly=off"
    
    # Boot configuration
    QEMU_CMD="$QEMU_CMD -boot d"
    QEMU_CMD="$QEMU_CMD -drive file=$KERNEL_IMAGE,if=virtio,index=0,media=disk,format=raw"
    
    # Serial console
    if [[ -n "$SERIAL_LOG" ]]; then
        QEMU_CMD="$QEMU_CMD -serial file:$SERIAL_LOG"
        QEMU_CMD="$QEMU_CMD -monitor stdio"
    else
        QEMU_CMD="$QEMU_CMD -serial stdio"
        QEMU_CMD="$QEMU_CMD -monitor null"
    fi
    
    # Graphics
    if [[ "$ENABLE_GRAPHICS" == true ]]; then
        QEMU_CMD="$QEMU_CMD -display gtk"
        QEMU_CMD="$QEMU_CMD -vga virtio"
    else
        QEMU_CMD="$QEMU_CMD -nographic"
        QEMU_CMD="$QEMU_CMD -vga none"
    fi
    
    # Network
    if [[ "$ENABLE_NETWORK" == true ]]; then
        QEMU_CMD="$QEMU_CMD -netdev user,id=net0"
        QEMU_CMD="$QEMU_CMD -device virtio-net-device,netdev=net0"
    fi
    
    # Additional storage
    if [[ -n "$DISK_IMAGE" ]]; then
        if [[ ! -f "$DISK_IMAGE" ]]; then
            log_warning "Disk image not found: $DISK_IMAGE"
            log_info "Creating empty disk image..."
            mkdir -p "$(dirname "$DISK_IMAGE")"
            qemu-img create -f raw "$DISK_IMAGE" 1G
        fi
        QEMU_CMD="$QEMU_CMD -drive file=$DISK_IMAGE,if=virtio,index=1,media=disk,format=raw"
    fi
    
    # Debug options
    if [[ "$ENABLE_DEBUG" == true ]]; then
        QEMU_CMD="$QEMU_CMD -d guest_errors"
        QEMU_CMD="$QEMU_CMD -D qemu_debug.log"
    fi
    
    # Additional QEMU arguments
    if [[ -n "$QEMU_ARGS" ]]; then
        QEMU_CMD="$QEMU_CMD $QEMU_ARGS"
    fi
    
    log_success "QEMU command built successfully"
}

# Run QEMU
run_qemu() {
    log_info "Starting QEMU with OVMF..."
    log_info "Kernel: $KERNEL_IMAGE"
    log_info "Memory: $MEMORY"
    log_info "CPUs: $CPUS"
    
    if [[ -n "$DISK_IMAGE" ]]; then
        log_info "Disk: $DISK_IMAGE"
    fi
    
    if [[ -n "$SERIAL_LOG" ]]; then
        log_info "Serial log: $SERIAL_LOG"
    fi
    
    if [[ "$ENABLE_GRAPHICS" == true ]]; then
        log_info "Graphics: Enabled"
    else
        log_info "Graphics: Disabled (headless)"
    fi
    
    if [[ "$ENABLE_NETWORK" == true ]]; then
        log_info "Network: Enabled"
    fi
    
    if [[ "$ENABLE_DEBUG" == true ]]; then
        log_info "Debug: Enabled"
    fi
    
    echo ""
    log_info "Press Ctrl+A, then X to exit QEMU"
    echo ""
    
    # Execute QEMU
    eval "$QEMU_CMD"
}

# Main execution
main() {
    log_info "Polymera OS QEMU Runner (aarch64)"
    log_info "=================================="
    
    # Check dependencies
    if ! command -v qemu-system-aarch64 &> /dev/null; then
        log_error "qemu-system-aarch64 not found. Please install QEMU."
        exit 1
    fi
    
    # Check OVMF firmware
    check_ovmf
    
    # Build QEMU command
    build_qemu_command
    
    # Run QEMU
    run_qemu
}

# Run main function
main "$@"
