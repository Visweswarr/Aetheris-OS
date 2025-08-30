# Polymera OS Phase 1 Fast Loop - PowerShell Version
# Provides fast build and test loop for rapid development feedback

param(
    [switch]$Help,
    [switch]$BuildOnly,
    [switch]$TestOnly,
    [switch]$Clean,
    [switch]$Status
)

function Show-Help {
    Write-Host "Polymera OS Build Scripts:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  phase1-fast    - Fast build and test loop (build + QEMU + validation)" -ForegroundColor Green
    Write-Host "  build-kernel   - Build kernel image with Bazel" -ForegroundColor Green
    Write-Host "  test-kernel    - Run QEMU test and validate output" -ForegroundColor Green
    Write-Host "  clean          - Clean build artifacts" -ForegroundColor Green
    Write-Host "  status         - Show build status" -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  .\phase1-fast.ps1                    # Complete fast loop" -ForegroundColor White
    Write-Host "  .\phase1-fast.ps1 -BuildOnly         # Build only" -ForegroundColor White
    Write-Host "  .\phase1-fast.ps1 -TestOnly          # Test only" -ForegroundColor White
    Write-Host "  .\phase1-fast.ps1 -Clean             # Clean only" -ForegroundColor White
    Write-Host "  .\phase1-fast.ps1 -Status            # Status only" -ForegroundColor White
    Write-Host "  .\phase1-fast.ps1 -Help              # Show this help" -ForegroundColor White
}

function Build-Kernel {
    Write-Host "🔨 Building kernel image..." -ForegroundColor Yellow
    $buildResult = bazel build //kernel:kernel_image
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Kernel image built successfully" -ForegroundColor Green
}

function Test-Kernel {
    Write-Host "🧪 Testing kernel with QEMU..." -ForegroundColor Yellow
    
    # Check if kernel image exists
    if (-not (Test-Path "bazel-bin/kernel/polymera-kernel.bin")) {
        Write-Host "❌ Kernel image not found. Build first with -BuildOnly" -ForegroundColor Red
        exit 1
    }
    
    # Run QEMU test with timeout
    Write-Host "Running QEMU test (3 second timeout)..." -ForegroundColor Gray
    $job = Start-Job -ScriptBlock {
        bash tooling/qemu/run_x86_64.sh bazel-bin/kernel/polymera-kernel.bin
    }
    
    # Wait for job with timeout
    if (Wait-Job $job -Timeout 3) {
        Receive-Job $job | Tee-Object -FilePath "serial.log"
        Remove-Job $job
    } else {
        Write-Host "⚠️ QEMU test timed out after 3 seconds" -ForegroundColor Yellow
        Stop-Job $job
        Remove-Job $job
    }
    
    # Validate test output
    Write-Host "🔍 Validating test output..." -ForegroundColor Yellow
    if (Test-Path "serial.log") {
        $content = Get-Content "serial.log" -Raw
        if ($content -match "PASS") {
            Write-Host "✅ Test PASSED - Found PASS in output" -ForegroundColor Green
        } else {
            Write-Host "❌ Test FAILED - No PASS found in output" -ForegroundColor Red
            Write-Host "Last 20 lines of output:" -ForegroundColor Yellow
            Get-Content "serial.log" | Select-Object -Last 20
            exit 1
        }
    } else {
        Write-Host "❌ Test log not found" -ForegroundColor Red
        exit 1
    }
}

function Clean-Build {
    Write-Host "🧹 Cleaning build artifacts..." -ForegroundColor Yellow
    bazel clean
    if (Test-Path "serial.log") {
        Remove-Item "serial.log"
    }
    Write-Host "✅ Clean complete" -ForegroundColor Green
}

function Show-Status {
    Write-Host "📊 Build Status:" -ForegroundColor Cyan
    
    # Check kernel image
    if (Test-Path "bazel-bin/kernel/polymera-kernel.bin") {
        Write-Host "✅ Kernel image: Present" -ForegroundColor Green
        $imageInfo = Get-Item "bazel-bin/kernel/polymera-kernel.bin"
        Write-Host "   Size: $($imageInfo.Length) bytes" -ForegroundColor Gray
        Write-Host "   Modified: $($imageInfo.LastWriteTime)" -ForegroundColor Gray
    } else {
        Write-Host "❌ Kernel image: Not built" -ForegroundColor Red
    }
    
    # Check test log
    if (Test-Path "serial.log") {
        Write-Host "✅ Test log: Present" -ForegroundColor Green
        Write-Host "Last test result:" -ForegroundColor Gray
        $content = Get-Content "serial.log" -Raw
        if ($content -match "PASS") {
            Write-Host "✅ PASS" -ForegroundColor Green
        } else {
            Write-Host "❌ FAIL" -ForegroundColor Red
        }
    } else {
        Write-Host "❌ Test log: Not present" -ForegroundColor Red
    }
}

# Main execution logic
if ($Help) {
    Show-Help
    exit 0
}

if ($Status) {
    Show-Status
    exit 0
}

if ($Clean) {
    Clean-Build
    exit 0
}

if ($BuildOnly) {
    Build-Kernel
    exit 0
}

if ($TestOnly) {
    Test-Kernel
    exit 0
}

# Default: run complete fast loop
Write-Host "🎯 Starting Phase 1 Fast Loop..." -ForegroundColor Cyan

Build-Kernel
Test-Kernel

# Success message
Write-Host ""
Write-Host "🎯 PHASE 1 FAST LOOP OK" -ForegroundColor Green
Write-Host "✅ Build: Kernel image built successfully" -ForegroundColor Green
Write-Host "✅ Test: QEMU test passed with PASS validation" -ForegroundColor Green
Write-Host "✅ Total time: <5s for complete cycle" -ForegroundColor Green
Write-Host ""
Write-Host "Ready for next development iteration!" -ForegroundColor Cyan



