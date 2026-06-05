

$ErrorActionPreference = "Stop"

# Possible QEMU locations
$QemuPaths = @(
    "qemu-system-x86_64.exe",
    "C:\Program Files\qemu\qemu-system-x86_64.exe",
    "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe",
    "$env:ProgramFiles\qemu\qemu-system-x86_64.exe"
)

$QemuCmd = $null
foreach ($Path in $QemuPaths) {
    if (Test-Path $Path) {
        $QemuCmd = $Path
        break
    }
    # Check if command exists in PATH
    if (Get-Command $Path -ErrorAction SilentlyContinue) {
        $QemuCmd = $Path
        break
    }
}

if (-not $QemuCmd) {
    Write-Error "❌ QEMU not found! Please install QEMU and add it to your PATH, or install it to C:\Program Files\qemu."
    Write-Host "Download: https://www.qemu.org/download/#windows"
    exit 1
}

Write-Host "[BOOT] 🚀 Launching Polymera OS (Avengers Assembly)..." -ForegroundColor Cyan
Write-Host "[BOOT] Using QEMU: $QemuCmd" -ForegroundColor Gray

# Boot Command
# We use direct kernel boot (-kernel) to bypass the need for an ISO.
# We pass the other modules if possible, or just boot the kernel to check the banner.
# For multiboot, -kernel works best.
# Note: For full Avengers functionality, we'd pass modules via -initrd or multiboot, 
# but for "First Light" seeing the kernel banner is key.

& $QemuCmd -kernel "dist\boot\kernel.elf" `
    -m 4G `
    -serial stdio `
    -display none `
    -no-reboot `
    -d guest_errors

# Note: In a real scenario with multiboot, we would do:
# -initrd "dist/bin/init_fs,dist/bin/init_net,..." 
# but QEMU support for comma-separated initrd varries. 
# The kernel banner "Avengers Assembled" will appear regardless.
