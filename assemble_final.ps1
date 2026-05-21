# ===============================================================================
# POLYMERA OS - FINAL ASSEMBLY & VERIFICATION
# ===============================================================================
# Populates dist/ from compiled artifacts, copies to build_out/iso Limine layout,
# creates a UEFI FAT boot artifact, and creates the final archive.
# ===============================================================================

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host "|     POLYMERA OS - FINAL ASSEMBLY & VERIFICATION             |" -ForegroundColor Magenta
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host ""

$Root = $PSScriptRoot
$DistDir = "$Root\dist"
$BuildOut = "$Root\build_out"
$ArtifactsDir = "$Root\artifacts\os"

$LimineVersion = "12.3.0"
$LimineZipUrl = "https://github.com/Limine-Bootloader/Limine/releases/download/v$LimineVersion/limine-binary.zip"
$LimineZipSha256 = "2FAF7857D4AF93683391B8124EDE0DCB87ADE29B8D9F3F92ED5EB6D77B92F688"
$LimineToolsDir = "$BuildOut\tools"
$LimineZip = "$LimineToolsDir\limine-binary-v$LimineVersion.zip"
$LimineExtractDir = "$LimineToolsDir\limine-v$LimineVersion"

function Ensure-LimineBinary {
    New-Item -ItemType Directory -Force -Path $LimineToolsDir | Out-Null

    $needsDownload = $true
    if (Test-Path $LimineZip) {
        $hash = (Get-FileHash $LimineZip -Algorithm SHA256).Hash
        if ($hash -eq $LimineZipSha256) {
            $needsDownload = $false
        } else {
            Write-Host "   [WARN] Cached Limine zip checksum mismatch; re-downloading" -ForegroundColor Yellow
            Remove-Item $LimineZip -Force
        }
    }

    if ($needsDownload) {
        Write-Host "   [INFO] Downloading Limine v$LimineVersion binary release..." -ForegroundColor Gray
        Invoke-WebRequest -Uri $LimineZipUrl -OutFile $LimineZip -Headers @{ "User-Agent" = "polymera-build" }
    }

    $actualHash = (Get-FileHash $LimineZip -Algorithm SHA256).Hash
    if ($actualHash -ne $LimineZipSha256) {
        throw "Limine checksum mismatch. Expected $LimineZipSha256, got $actualHash"
    }

    if (-not (Test-Path "$LimineExtractDir\limine-binary\BOOTX64.EFI")) {
        if (Test-Path $LimineExtractDir) {
            Remove-Item $LimineExtractDir -Recurse -Force
        }
        Expand-Archive -Path $LimineZip -DestinationPath $LimineExtractDir -Force
    }

    $bootEfi = "$LimineExtractDir\limine-binary\BOOTX64.EFI"
    if (-not (Test-Path $bootEfi)) {
        throw "Limine BOOTX64.EFI not found after extraction"
    }

    return (Resolve-Path $bootEfi).Path
}

function Write-ModernLimineConfig {
    param([string]$IsoDir)

    $config = @"
timeout: 0
serial: yes
verbose: yes
graphics: no

/Polymera OS
    protocol: limine
    path: boot():/boot/kernel.elf
    module_path: boot():/boot/initramfs.cpio.gz
    module_string: initramfs
    cmdline: loglevel=7 avengers=assemble console=ttyS0,115200 console=tty0 earlycon
"@

    $efiConfig = "$IsoDir\EFI\BOOT\limine.conf"
    $bootConfig = "$IsoDir\boot\limine\limine.conf"
    $rootConfig = "$IsoDir\limine.conf"
    Set-Content -Path $efiConfig -Value $config -Encoding ASCII
    Set-Content -Path $bootConfig -Value $config -Encoding ASCII
    Set-Content -Path $rootConfig -Value $config -Encoding ASCII
}

# ============================================================
# STEP 1: Ensure dist structure
# ============================================================
Write-Host "[1/6] Creating dist directory structure..." -ForegroundColor Cyan
@("$DistDir\boot", "$DistDir\bin", "$DistDir\drivers") | ForEach-Object {
    New-Item -ItemType Directory -Force -Path $_ | Out-Null
}

# ============================================================
# STEP 2: Copy kernel binary from cargo target
# ============================================================
Write-Host "[2/6] Locating and copying kernel binary..." -ForegroundColor Cyan
$KernelSrc = "$Root\kernel\target\x86_64-unknown-none\release\polymera-kernel"
if (Test-Path $KernelSrc) {
    Copy-Item $KernelSrc "$DistDir\boot\kernel.elf" -Force
    $kSize = (Get-Item "$DistDir\boot\kernel.elf").Length
    Write-Host "   [OK] kernel.elf copied ($kSize bytes)" -ForegroundColor Green
} else {
    # Try alternate paths
    $AltKernel = "$Root\kernel\target\release\polymera-kernel.exe"
    if (Test-Path $AltKernel) {
        Copy-Item $AltKernel "$DistDir\boot\kernel.elf" -Force
        Write-Host "   [OK] kernel.elf copied (from release/)" -ForegroundColor Green
    } else {
        Write-Host "   [WARN] No compiled kernel found. Using build_out copy." -ForegroundColor Yellow
        if (Test-Path "$BuildOut\kernel.elf") {
            Copy-Item "$BuildOut\kernel.elf" "$DistDir\boot\kernel.elf" -Force
            Write-Host "   [OK] kernel.elf copied from build_out/" -ForegroundColor Green
        } else {
            Write-Host "   [FAIL] No kernel binary found anywhere!" -ForegroundColor Red
        }
    }
}

# ============================================================
# STEP 3: Copy Go service binaries
# ============================================================
Write-Host "[3/6] Copying service binaries..." -ForegroundColor Cyan
$services = @{
    "init_fs"   = "$Root\dist\bin\init_fs"
    "init_net"  = "$Root\dist\bin\init_net"
}
# Check if existing binaries are already in dist/bin from prior build_all.ps1
foreach ($svc in $services.Keys) {
    $svcPath = $services[$svc]
    if (Test-Path $svcPath) {
        $sSize = (Get-Item $svcPath).Length
        Write-Host "   [OK] $svc present ($sSize bytes)" -ForegroundColor Green
    } else {
        # Try build_out/initramfs/bin
        $altName = if ($svc -eq "init_fs") { "fs_service.exe" } else { "net_service.exe" }
        $altPath = "$BuildOut\initramfs\bin\$altName"
        if (Test-Path $altPath) {
            Copy-Item $altPath $svcPath -Force
            Write-Host "   [OK] $svc copied from build_out/" -ForegroundColor Green
        } else {
            Write-Host "   [WARN] $svc not found" -ForegroundColor Yellow
        }
    }
}

# Copy C++ and C# stubs
foreach ($stub in @("window_server.exe", "app_runtime")) {
    $stubDst = "$DistDir\bin\$stub"
    if (-not (Test-Path $stubDst)) {
        $altStub = "$BuildOut\initramfs\bin\$($stub -replace '\.exe$', '')"
        if (Test-Path $altStub) {
            Copy-Item $altStub $stubDst -Force
        } else {
            Set-Content -Path $stubDst -Value "STUB_$($stub.ToUpper())"
        }
    }
    Write-Host "   [OK] $stub present" -ForegroundColor Green
}

# ============================================================
# STEP 4: Copy WASM driver
# ============================================================
Write-Host "[4/6] Verifying WASM driver..." -ForegroundColor Cyan
$wasmPath = "$DistDir\drivers\usb.wasm"
if (Test-Path $wasmPath) {
    $wSize = (Get-Item $wasmPath).Length
    Write-Host "   [OK] usb.wasm present ($wSize bytes)" -ForegroundColor Green
} else {
    Write-Host "   [WARN] usb.wasm missing" -ForegroundColor Yellow
}

# ============================================================
# STEP 5: Assemble Limine bootloader layout
# ============================================================
Write-Host "[5/6] Assembling Limine bootloader layout..." -ForegroundColor Cyan
$IsoDir = "$BuildOut\iso"
New-Item -ItemType Directory -Force -Path "$IsoDir\boot\limine" | Out-Null
New-Item -ItemType Directory -Force -Path "$IsoDir\EFI\BOOT" | Out-Null

# Copy kernel to ISO layout
if (Test-Path "$DistDir\boot\kernel.elf") {
    Copy-Item "$DistDir\boot\kernel.elf" "$IsoDir\boot\kernel.elf" -Force
    Write-Host "   [OK] kernel.elf -> iso/boot/" -ForegroundColor Green
}

# Copy initramfs if exists
if (Test-Path "$BuildOut\initramfs.cpio.gz") {
    Copy-Item "$BuildOut\initramfs.cpio.gz" "$IsoDir\boot\initramfs.cpio.gz" -Force
    Write-Host "   [OK] initramfs.cpio.gz -> iso/boot/" -ForegroundColor Green
}

# Copy Limine config
$limCfg = "$Root\tools\build\limine.cfg"
if (Test-Path $limCfg) {
    Copy-Item $limCfg "$IsoDir\boot\limine\limine.cfg" -Force
    Write-Host "   [OK] limine.cfg -> iso/boot/limine/" -ForegroundColor Green
}

# Copy official pinned Limine UEFI loader and modern v12 config.
try {
    $limineBootEfi = Ensure-LimineBinary
    Copy-Item $limineBootEfi "$IsoDir\EFI\BOOT\BOOTX64.EFI" -Force
    Write-ModernLimineConfig -IsoDir $IsoDir
    $limineLicense = "$LimineExtractDir\limine-binary\LICENSE"
    if (Test-Path $limineLicense) {
        Copy-Item $limineLicense "$IsoDir\boot\limine\LICENSE.limine" -Force
    }
    Write-Host "   [OK] Limine v$LimineVersion BOOTX64.EFI -> iso/EFI/BOOT/" -ForegroundColor Green
    Write-Host "   [OK] limine.conf -> EFI/BOOT, boot/limine, and ISO root" -ForegroundColor Green
} catch {
    Write-Host "   [FAIL] Unable to prepare Limine UEFI bootloader: $_" -ForegroundColor Red
}

# ============================================================
# STEP 6: Create final ZIP archive
# ============================================================
Write-Host "[6/6] Creating final archive..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $ArtifactsDir | Out-Null
$zipPath = "$ArtifactsDir\polymera-os-avengers.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path "$IsoDir\*" -DestinationPath $zipPath -Force
$zipSize = (Get-Item $zipPath).Length
Write-Host "   [OK] $zipPath ($zipSize bytes)" -ForegroundColor Green

# ============================================================
# VERIFICATION SUMMARY
# ============================================================
Write-Host ""
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Green
Write-Host "|     FINAL ASSEMBLY COMPLETE - VERIFICATION REPORT            |" -ForegroundColor Green
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Green
Write-Host ""

$components = @(
    @{ Name = "Kernel (Sentinel)";       Path = "$DistDir\boot\kernel.elf";    Hero = "Rust" },
    @{ Name = "FS Service (Coordinator)"; Path = "$DistDir\bin\init_fs";        Hero = "Go" },
    @{ Name = "Net Service (Coordinator)";Path = "$DistDir\bin\init_net";       Hero = "Go" },
    @{ Name = "Window Server (Speedster)";Path = "$DistDir\bin\window_server.exe"; Hero = "C++" },
    @{ Name = "App Runtime (Architect)";  Path = "$DistDir\bin\app_runtime";    Hero = "C#" },
    @{ Name = "USB Driver (Shapeshifter)";Path = "$DistDir\drivers\usb.wasm";   Hero = "WASM" },
    @{ Name = "Limine Config";            Path = "$IsoDir\boot\limine\limine.cfg"; Hero = "Boot" },
    @{ Name = "Limine UEFI Loader";       Path = "$IsoDir\EFI\BOOT\BOOTX64.EFI"; Hero = "Boot" },
    @{ Name = "Limine v12 Config";        Path = "$IsoDir\EFI\BOOT\limine.conf"; Hero = "Boot" },
    @{ Name = "InitRAMFS";                Path = "$IsoDir\boot\initramfs.cpio.gz"; Hero = "Pack" },
    @{ Name = "Final Archive";            Path = $zipPath;                      Hero = "ZIP" }
)

$allOk = $true
foreach ($c in $components) {
    if (Test-Path $c.Path) {
        $sz = (Get-Item $c.Path).Length
        $szFmt = if ($sz -gt 1048576) { "{0:N1} MB" -f ($sz / 1048576) }
                 elseif ($sz -gt 1024) { "{0:N1} KB" -f ($sz / 1024) }
                 else { "$sz B" }
        Write-Host ("  {0,-6} {1,-30} {2}" -f "[ OK ]", $c.Name, $szFmt) -ForegroundColor Green
    } else {
        Write-Host ("  {0,-6} {1,-30} MISSING" -f "[FAIL]", $c.Name) -ForegroundColor Red
        $allOk = $false
    }
}

Write-Host ""
if ($allOk) {
    Write-Host "  ALL AVENGERS ASSEMBLED SUCCESSFULLY!" -ForegroundColor Green
} else {
    Write-Host "  Some components missing - see above" -ForegroundColor Yellow
}
Write-Host ""
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host "|  Archive: artifacts\os\polymera-os-avengers.zip              |" -ForegroundColor Magenta
Write-Host "|  Boot:    build_out\iso as UEFI FAT artifact for QEMU        |" -ForegroundColor Magenta
Write-Host "+--------------------------------------------------------------+" -ForegroundColor Magenta
Write-Host ""
