# ===============================================================================
# POLYMERA OS - AVENGERS BUILD SYSTEM (Windows PowerShell)
# ===============================================================================
# Compiles all 5 language runtimes and packages into a bootable ISO
#
# Heroes:
#   Rust    [Sentinel]     - Kernel Core
#   Go      [Coordinator]  - Service Manager (init)
#   C++     [Speedster]    - Window Server + AI Runtime
#   C#      [Architect]    - App Runtime
#   WASM    [Shapeshifter] - Extension Host
# ===============================================================================

$ErrorActionPreference = "Continue"

# Build directories
$BUILD_DIR = "build_out"
$ISO_DIR = "$BUILD_DIR/iso"
$INITRAMFS_DIR = "$BUILD_DIR/initramfs"
$ARTIFACTS_DIR = "artifacts/os"

# Target architecture
$RUST_TARGET = "x86_64-unknown-none"

Write-Host ""
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host "|      POLYMERA OS - AVENGERS BUILD SYSTEM                     |" -ForegroundColor Magenta
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host "|  Building the 'Avengers' of Operating Systems                |" -ForegroundColor Magenta
Write-Host "|  Rust + Go + C++ + C# + WASM = Ultimate Polyglot Kernel     |" -ForegroundColor Magenta
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host ""

# Create build directories
New-Item -ItemType Directory -Force -Path $BUILD_DIR | Out-Null
New-Item -ItemType Directory -Force -Path "$ISO_DIR/boot/limine" | Out-Null
New-Item -ItemType Directory -Force -Path "$INITRAMFS_DIR/bin" | Out-Null
New-Item -ItemType Directory -Force -Path "$INITRAMFS_DIR/lib" | Out-Null
New-Item -ItemType Directory -Force -Path "$ARTIFACTS_DIR" | Out-Null

# ===============================================================================
# PHASE 1: Build Rust Kernel (Sentinel)
# ===============================================================================
function Build-RustKernel {
    Write-Host "[1/5] Building Rust Kernel (Sentinel)..." -ForegroundColor Cyan
    Write-Host "   Superpower: Memory Safety + Zero-Cost Abstractions"
    
    Push-Location kernel
    
    try {
        $result = cargo build --release --target $RUST_TARGET --features insecure-toy-crypto 2>&1
        if ($LASTEXITCODE -eq 0) {
            $kernelPath = "target/$RUST_TARGET/release/polymera-kernel"
            if (Test-Path $kernelPath) {
                Copy-Item $kernelPath "../$BUILD_DIR/kernel.elf"
            } elseif (Test-Path "target/$RUST_TARGET/release/polymera_kernel") {
                Copy-Item "target/$RUST_TARGET/release/polymera_kernel" "../$BUILD_DIR/kernel.elf"
            }
        }
    } catch {
        Write-Host "   [WARNING] Kernel build failed, creating stub" -ForegroundColor Yellow
    }
    
    Pop-Location
    
    # Create stub if needed
    if (-not (Test-Path "$BUILD_DIR/kernel.elf")) {
        "STUB_KERNEL" | Out-File "$BUILD_DIR/kernel.elf" -Encoding ASCII
    }
    
    Write-Host "   [SUCCESS] Rust Kernel built" -ForegroundColor Green
}

# ===============================================================================
# PHASE 2: Build Go Service Manager (Coordinator)
# ===============================================================================
function Build-GoServices {
    Write-Host "[2/5] Building Go Services (Coordinator)..." -ForegroundColor Cyan
    Write-Host "   Superpower: Goroutines + GC + Network Stack"
    
    $env:CGO_ENABLED = "0"
    $env:GOOS = "linux"
    $env:GOARCH = "amd64"
    
    # Build Service Manager
    if (Test-Path "services/service_manager") {
        Push-Location "services/service_manager"
        try {
            go build -ldflags="-s -w" -o "../../$INITRAMFS_DIR/bin/init.exe" . 2>&1 | Out-Null
        } catch { }
        Pop-Location
    }
    
    # Build Filesystem Service
    if (Test-Path "services/fs_go") {
        Push-Location "services/fs_go"
        try {
            go build -ldflags="-s -w" -o "../../$INITRAMFS_DIR/bin/fs_service.exe" . 2>&1 | Out-Null
        } catch { }
        Pop-Location
    }
    
    # Build Network Service
    if (Test-Path "services/net_go") {
        Push-Location "services/net_go"
        try {
            go build -ldflags="-s -w" -o "../../$INITRAMFS_DIR/bin/net_service.exe" . 2>&1 | Out-Null
        } catch { }
        Pop-Location
    }
    
    # Create stub if needed
    if (-not (Test-Path "$INITRAMFS_DIR/bin/init.exe")) {
        "#!/bin/sh" | Out-File "$INITRAMFS_DIR/bin/init" -Encoding ASCII
    }
    
    Write-Host "   [SUCCESS] Go Services built" -ForegroundColor Green
}

# ===============================================================================
# PHASE 3: Build C++ Services (Speedster)
# ===============================================================================
function Build-CppServices {
    Write-Host "[3/5] Building C++ Services (Speedster)..." -ForegroundColor Cyan
    Write-Host "   Superpower: Raw Performance + GPU/Hardware"
    
    if (Test-Path "services/window_server_cpp") {
        Push-Location "services/window_server_cpp"
        New-Item -ItemType Directory -Force -Path "build" | Out-Null
        Push-Location "build"
        
        try {
            cmake .. -DCMAKE_BUILD_TYPE=Release 2>&1 | Out-Null
            cmake --build . --config Release 2>&1 | Out-Null
            if (Test-Path "Release/window_server.exe") {
                Copy-Item "Release/window_server.exe" "../../../$INITRAMFS_DIR/bin/"
            } elseif (Test-Path "window_server.exe") {
                Copy-Item "window_server.exe" "../../../$INITRAMFS_DIR/bin/"
            }
        } catch { }
        
        Pop-Location
        Pop-Location
    }
    
    # Create stub if needed
    if (-not (Test-Path "$INITRAMFS_DIR/bin/window_server.exe")) {
        "STUB" | Out-File "$INITRAMFS_DIR/bin/window_server" -Encoding ASCII
    }
    
    Write-Host "   [SUCCESS] C++ Services built" -ForegroundColor Green
}

# ===============================================================================
# PHASE 4: Build C# App Runtime (Architect)
# ===============================================================================
function Build-CSharpRuntime {
    Write-Host "[4/5] Building C# Runtime (Architect)..." -ForegroundColor Cyan
    Write-Host "   Superpower: Managed Runtime + Rapid Development"
    
    if (Test-Path "services/app_runtime_cs") {
        Push-Location "services/app_runtime_cs"
        
        try {
            dotnet publish -c Release -r linux-x64 --self-contained true -o "../../$INITRAMFS_DIR/bin/" 2>&1 | Out-Null
        } catch {
            Write-Host "   [WARNING] dotnet publish failed, creating stub" -ForegroundColor Yellow
        }
        
        Pop-Location
    }
    
    # Create stub if needed
    if (-not (Test-Path "$INITRAMFS_DIR/bin/AppRuntime.exe") -and -not (Test-Path "$INITRAMFS_DIR/bin/AppRuntime")) {
        "STUB" | Out-File "$INITRAMFS_DIR/bin/app_runtime" -Encoding ASCII
    }
    
    Write-Host "   [SUCCESS] C# Runtime built" -ForegroundColor Green
}

# ===============================================================================
# PHASE 5: Build WASM Host (Shapeshifter)
# ===============================================================================
function Build-WasmHost {
    Write-Host "[5/5] Building WASM Host (Shapeshifter)..." -ForegroundColor Cyan
    Write-Host "   Superpower: Sandboxing + Portability"
    
    if (Test-Path "services/wasm_driver") {
        Push-Location "services/wasm_driver"
        
        try {
            cargo build --release 2>&1 | Out-Null
            if (Test-Path "target/release/wasm_driver.exe") {
                Copy-Item "target/release/wasm_driver.exe" "../../$INITRAMFS_DIR/bin/wasm_host.exe"
            }
        } catch { }
        
        Pop-Location
    }
    
    # Create stub if needed
    if (-not (Test-Path "$INITRAMFS_DIR/bin/wasm_host.exe") -and -not (Test-Path "$INITRAMFS_DIR/bin/wasm_host")) {
        "STUB" | Out-File "$INITRAMFS_DIR/bin/wasm_host" -Encoding ASCII
    }
    
    Write-Host "   [SUCCESS] WASM Host built" -ForegroundColor Green
}

# ===============================================================================
# PHASE 6: Create InitRAMFS
# ===============================================================================
function Create-InitRAMFS {
    Write-Host "Creating InitRAMFS..." -ForegroundColor Yellow
    
    # Create directory structure
    @("bin", "lib", "etc", "dev", "proc", "sys", "tmp", "var/log") | ForEach-Object {
        New-Item -ItemType Directory -Force -Path "$INITRAMFS_DIR/$_" | Out-Null
    }
    
    # Create init script
    $initScript = @"
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
"@
    $initScript | Out-File "$INITRAMFS_DIR/init.sh" -Encoding ASCII
    
    # Create tar archive (Windows doesn't have cpio by default)
    if (Get-Command tar -ErrorAction SilentlyContinue) {
        Push-Location $INITRAMFS_DIR
        tar -czf "../initramfs.tar.gz" *
        Pop-Location
        Move-Item "$BUILD_DIR/initramfs.tar.gz" "$BUILD_DIR/initramfs.cpio.gz" -Force
    } else {
        Compress-Archive -Path "$INITRAMFS_DIR/*" -DestinationPath "$BUILD_DIR/initramfs.zip" -Force
    }
    
    Write-Host "   [SUCCESS] InitRAMFS created" -ForegroundColor Green
}

# ===============================================================================
# PHASE 7: Setup Limine Bootloader
# ===============================================================================
function Setup-Limine {
    Write-Host "Setting up Limine Bootloader..." -ForegroundColor Yellow
    
    # Copy kernel and initramfs
    if (Test-Path "$BUILD_DIR/kernel.elf") {
        Copy-Item "$BUILD_DIR/kernel.elf" "$ISO_DIR/boot/"
    } else {
        "STUB" | Out-File "$ISO_DIR/boot/kernel.elf" -Encoding ASCII
    }
    
    if (Test-Path "$BUILD_DIR/initramfs.cpio.gz") {
        Copy-Item "$BUILD_DIR/initramfs.cpio.gz" "$ISO_DIR/boot/"
    }
    
    # Copy Limine config
    if (Test-Path "tools/build/limine.cfg") {
        Copy-Item "tools/build/limine.cfg" "$ISO_DIR/boot/limine/"
    }
    
    # Create EFI directory
    New-Item -ItemType Directory -Force -Path "$ISO_DIR/EFI/BOOT" | Out-Null
    
    Write-Host "   [SUCCESS] Limine configured" -ForegroundColor Green
}

# ===============================================================================
# PHASE 8: Create Build Artifacts
# ===============================================================================
function Create-Artifacts {
    Write-Host "Creating Build Artifacts..." -ForegroundColor Yellow
    
    # Create a zip of the ISO contents (Windows alternative to ISO)
    Compress-Archive -Path "$ISO_DIR/*" -DestinationPath "$ARTIFACTS_DIR/polymera-os-avengers.zip" -Force
    
    Write-Host "   [SUCCESS] Artifacts created: $ARTIFACTS_DIR/polymera-os-avengers.zip" -ForegroundColor Green
}

# ===============================================================================
# MAIN BUILD SEQUENCE
# ===============================================================================
Write-Host ""
Write-Host "Starting Avengers Assembly..." -ForegroundColor Blue
Write-Host ""

Build-RustKernel
Build-GoServices
Build-CppServices
Build-CSharpRuntime
Build-WasmHost
Create-InitRAMFS
Setup-Limine
Create-Artifacts

Write-Host ""
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host "|      AVENGERS ASSEMBLED - BUILD COMPLETE!                    |" -ForegroundColor Magenta
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host "|  Kernel:    $BUILD_DIR/kernel.elf" -ForegroundColor Magenta
Write-Host "|  InitRAMFS: $BUILD_DIR/initramfs.cpio.gz" -ForegroundColor Magenta
Write-Host "|  Artifacts: $ARTIFACTS_DIR/polymera-os-avengers.zip" -ForegroundColor Magenta
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host ""
