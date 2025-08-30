#!/usr/bin/env bash
set -euo pipefail

KIMG="${1:-bazel-bin/kernel/polymera-kernel.bin}"

# Check if kernel image exists
if [ ! -f "$KIMG" ]; then
    echo "Error: Kernel image not found at $KIMG"
    echo "Build it first with: bazel build //kernel:kernel_image"
    exit 1
fi

echo "Starting QEMU with kernel: $KIMG"
echo "QEMU output will be saved to serial.log"

# Run QEMU and tee output to serial.log
qemu-system-x86_64 \
    -kernel "$KIMG" \
    -serial stdio \
    -display none \
    -no-reboot \
    -no-shutdown \
    -m 512 \
    2>&1 | tee serial.log