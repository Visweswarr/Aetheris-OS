# Polymera OS Quick Run Script
# This script simplifies running Polymera OS verification and smoke tests via WSL

param(
    [Parameter(Mandatory = $false)]
    [ValidateSet("verify", "smoke", "all", "help")]
    [string]$Action = "help"
)

$ErrorActionPreference = "Stop"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Test-WSL {
    if (-not (Get-Command wsl -ErrorAction SilentlyContinue)) {
        Write-ColorOutput "❌ WSL is not installed. Please install WSL2 first." "Red"
        Write-ColorOutput "   Run: wsl --install -d Ubuntu" "Yellow"
        exit 1
    }
    Write-ColorOutput "✅ WSL is installed" "Green"
}

function Run-VerificationScript {
    Write-ColorOutput "`n🚀 Running Phase 4 Verification..." "Cyan"
    Write-ColorOutput "   This may take several minutes...`n" "Yellow"
    
    wsl -e bash -c "cd /mnt/c/polymera-os && bash scripts/phase4-verify.sh"
    
    if ($LASTEXITCODE -eq 0) {
        Write-ColorOutput "`n✅ Phase 4 Verification PASSED!" "Green"
    }
    else {
        Write-ColorOutput "`n❌ Phase 4 Verification FAILED!" "Red"
        Write-ColorOutput "   Check the log at: artifacts/phase4-verify.log" "Yellow"
        exit 1
    }
}

function Run-SmokeTests {
    Write-ColorOutput "`n🧪 Running Smoke Tests..." "Cyan"
    
    $smokeTests = @(
        "p4-contracts-smoke.sh",
        "p4-xr-determinism.sh",
        "p4-hal-toggle.sh"
    )
    
    foreach ($test in $smokeTests) {
        Write-ColorOutput "`n   Running: $test" "Yellow"
        wsl -e bash -c "cd /mnt/c/polymera-os && bash scripts/$test"
        
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput "   ✅ $test PASSED" "Green"
        }
        else {
            Write-ColorOutput "   ❌ $test FAILED" "Red"
        }
    }
}

function Show-Help {
    Write-ColorOutput "`nPolymera OS Quick Run Script" "Cyan"
    Write-ColorOutput "============================`n" "Cyan"
    Write-ColorOutput "Usage: .\run-polymera.ps1 -Action <action>`n" "White"
    Write-ColorOutput "Actions:" "Yellow"
    Write-ColorOutput "  verify  - Run Phase 4 verification script" "White"
    Write-ColorOutput "  smoke   - Run smoke tests (contracts, XR, HAL)" "White"
    Write-ColorOutput "  all     - Run verification + smoke tests" "White"
    Write-ColorOutput "  help    - Show this help message`n" "White"
    Write-ColorOutput "Examples:" "Yellow"
    Write-ColorOutput "  .\run-polymera.ps1 -Action verify" "Gray"
    Write-ColorOutput "  .\run-polymera.ps1 -Action smoke" "Gray"
    Write-ColorOutput "  .\run-polymera.ps1 -Action all`n" "Gray"
}

# Main execution
switch ($Action) {
    "verify" {
        Test-WSL
        Run-VerificationScript
    }
    "smoke" {
        Test-WSL
        Run-SmokeTests
    }
    "all" {
        Test-WSL
        Run-VerificationScript
        Run-SmokeTests
    }
    "help" {
        Show-Help
    }
}

Write-ColorOutput "`n✨ Done!`n" "Green"
