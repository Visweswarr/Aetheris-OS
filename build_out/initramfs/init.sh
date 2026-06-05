#!/bin/sh
# Polymera OS Init Script
echo "[AVENGERS] POLYMERA OS - Avengers Microkernel Booting..."
echo ""
echo "Starting services..."

mount -t proc none /proc 2>/dev/null || true
mount -t sysfs none /sys 2>/dev/null || true
mount -t devtmpfs none /dev 2>/dev/null || true

/bin/init &
/bin/window_server &
/bin/wasm_host &

echo "[AVENGERS] All Avengers assembled!"
exec /bin/sh
