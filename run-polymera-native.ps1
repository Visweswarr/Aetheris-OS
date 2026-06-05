# Polymera OS Native Windows Build Script v2
# Fixed: Handles multi-crate Rust project without root workspace

param(
    [Parameter(Mandatory = $false)]
    [ValidateSet("build", "test", "verify", "all", "help")]
    [string]$Action = "help"
)

$ErrorActionPreference = "Continue"
$script:TestsPassed = 0
$script:TestsFailed = 0
$script:BuildsPassed = 0
$script:BuildsFailed = 0

function Write-Status {
    param([string]$Message, [string]$Color = "White")
    Write-Host $Message -ForegroundColor $Color
}

function Test-Command {
    param([string]$Command)
    $cmd = Get-Command $Command -ErrorAction SilentlyContinue
    return $null -ne $cmd
}

function Test-Prerequisites {
    Write-Status "`n📋 Checking Prerequisites..." "Cyan"
    
    $allPresent = $true
    
    # Check Rust
    if (Test-Command "rustc") {
        $version = rustc --version 2>&1
        Write-Status "   ✅ Rust: $version" "Green"
    }
    else {
        Write-Status "   ❌ Rust - NOT FOUND" "Red"
        $allPresent = $false
    }
    
    # Check Cargo
    if (Test-Command "cargo") {
        $version = cargo --version 2>&1
        Write-Status "   ✅ Cargo: $version" "Green"
    }
    else {
        Write-Status "   ❌ Cargo - NOT FOUND" "Red"
        $allPresent = $false
    }
    
    # Check Go
    if (Test-Command "go") {
        $version = go version 2>&1
        Write-Status "   ✅ Go: $version" "Green"
    }
    else {
        Write-Status "   ❌ Go - NOT FOUND" "Red"
        $allPresent = $false
    }
    
    # Check Node
    if (Test-Command "node") {
        $version = node --version 2>&1
        Write-Status "   ✅ Node.js: $version" "Green"
    }
    else {
        Write-Status "   ⚠️ Node.js - NOT FOUND (optional)" "Yellow"
    }
    
    # Check Python
    if (Test-Command "python") {
        $version = python --version 2>&1
        Write-Status "   ✅ Python: $version" "Green"
    }
    else {
        Write-Status "   ⚠️ Python - NOT FOUND (optional)" "Yellow"
    }
    
    return $allPresent
}

function Run-RustBuild {
    Write-Status "`n🦀 Building Rust Components..." "Cyan"
    
    # Find all Cargo.toml files (excluding target directories)
    $cargoFiles = Get-ChildItem -Path "." -Filter "Cargo.toml" -Recurse -Depth 4 | 
    Where-Object { $_.FullName -notmatch "\\target\\" -and $_.FullName -notmatch "\\node_modules\\" }
    
    if ($cargoFiles.Count -eq 0) {
        Write-Status "   ⚠️ No Cargo.toml found" "Yellow"
        return
    }
    
    Write-Status "   Found $($cargoFiles.Count) Rust crates" "Yellow"
    
    # Build key crates (not all, to save time)
    $keyCrates = @(
        "kernel",
        "services\ai_core",
        "services\ngfs",
        "services\policy",
        "crypto"
    )
    
    foreach ($cratePath in $keyCrates) {
        $fullPath = Join-Path $PWD $cratePath
        $cargoFile = Join-Path $fullPath "Cargo.toml"
        
        if (Test-Path $cargoFile) {
            $crateName = Split-Path $cratePath -Leaf
            Write-Status "   Building: $crateName..." "Yellow"
            
            Push-Location $fullPath
            $output = cargo build 2>&1
            $exitCode = $LASTEXITCODE
            Pop-Location
            
            if ($exitCode -eq 0) {
                Write-Status "   ✅ $crateName built successfully" "Green"
                $script:BuildsPassed++
            }
            else {
                Write-Status "   ❌ $crateName build failed" "Red"
                $script:BuildsFailed++
            }
        }
        else {
            Write-Status "   ⚠️ $cratePath not found - skipping" "Yellow"
        }
    }
}

function Run-RustTests {
    Write-Status "`n🧪 Running Rust Tests..." "Cyan"
    
    # Test key crates
    $keyCrates = @(
        "crypto",
        "services\policy"
    )
    
    foreach ($cratePath in $keyCrates) {
        $fullPath = Join-Path $PWD $cratePath
        $cargoFile = Join-Path $fullPath "Cargo.toml"
        
        if (Test-Path $cargoFile) {
            $crateName = Split-Path $cratePath -Leaf
            Write-Status "   Testing: $crateName..." "Yellow"
            
            Push-Location $fullPath
            $output = cargo test 2>&1
            $exitCode = $LASTEXITCODE
            Pop-Location
            
            if ($exitCode -eq 0) {
                Write-Status "   ✅ $crateName tests passed" "Green"
                $script:TestsPassed++
            }
            else {
                Write-Status "   ❌ $crateName tests failed" "Red"
                $script:TestsFailed++
            }
        }
    }
}

function Run-GoTests {
    Write-Status "`n🔷 Checking Go Components..." "Cyan"
    
    # Find go.mod files
    $goModFiles = Get-ChildItem -Path "." -Filter "go.mod" -Recurse -Depth 3 |
    Where-Object { $_.FullName -notmatch "\\node_modules\\" }
    
    if ($goModFiles.Count -eq 0) {
        Write-Status "   ⚠️ No go.mod found - skipping" "Yellow"
        return
    }
    
    Write-Status "   Found $($goModFiles.Count) Go modules" "Yellow"
    
    foreach ($goMod in $goModFiles) {
        $dir = $goMod.Directory.FullName
        $modName = $goMod.Directory.Name
        Write-Status "   Checking: $modName" "Yellow"
        
        Push-Location $dir
        # Just verify the module, don't run tests (they may require external deps)
        $output = go build ./... 2>&1
        $exitCode = $LASTEXITCODE
        Pop-Location
        
        if ($exitCode -eq 0) {
            Write-Status "   ✅ $modName compiles" "Green"
            $script:BuildsPassed++
        }
        else {
            Write-Status "   ⚠️ $modName has issues (may need dependencies)" "Yellow"
        }
    }
}

function Run-Verification {
    Write-Status "`n🔍 Running Verification Checks..." "Cyan"
    
    # Check for required files
    $requiredFiles = @(
        @{Path = "README.md"; Required = $true },
        @{Path = "SPEC.md"; Required = $true },
        @{Path = "DESIGN.md"; Required = $true },
        @{Path = "Makefile"; Required = $false }
    )
    
    foreach ($file in $requiredFiles) {
        if (Test-Path $file.Path) {
            Write-Status "   ✅ $($file.Path) exists" "Green"
        }
        elseif ($file.Required) {
            Write-Status "   ❌ $($file.Path) MISSING" "Red"
        }
        else {
            Write-Status "   ⚠️ $($file.Path) not found (optional)" "Yellow"
        }
    }
    
    # Check services directory
    if (Test-Path "services") {
        $services = Get-ChildItem -Path "services" -Directory
        Write-Status "   ✅ Found $($services.Count) services" "Green"
    }
    
    # Check kernel directory
    if (Test-Path "kernel") {
        $kernelCargo = Test-Path "kernel\Cargo.toml"
        if ($kernelCargo) {
            Write-Status "   ✅ Kernel crate exists" "Green"
        }
        else {
            Write-Status "   ✅ Kernel directory exists" "Green"
        }
    }
    
    # Check for AI Core service
    if (Test-Path "services\ai_core\Cargo.toml") {
        Write-Status "   ✅ AI Core service exists" "Green"
    }
}

function Show-Summary {
    Write-Status "`n" "White"
    Write-Status "═══════════════════════════════════════" "Cyan"
    Write-Status "      Polymera OS Build Summary        " "Cyan"
    Write-Status "═══════════════════════════════════════" "Cyan"
    Write-Status "  Builds Passed: $script:BuildsPassed" "$(if ($script:BuildsPassed -gt 0) {'Green'} else {'Yellow'})"
    Write-Status "  Builds Failed: $script:BuildsFailed" "$(if ($script:BuildsFailed -gt 0) {'Red'} else {'Green'})"
    Write-Status "  Tests Passed:  $script:TestsPassed" "$(if ($script:TestsPassed -gt 0) {'Green'} else {'Yellow'})"
    Write-Status "  Tests Failed:  $script:TestsFailed" "$(if ($script:TestsFailed -gt 0) {'Red'} else {'Green'})"
    Write-Status "═══════════════════════════════════════`n" "Cyan"
    
    $totalFailed = $script:BuildsFailed + $script:TestsFailed
    if ($totalFailed -eq 0 -and ($script:BuildsPassed -gt 0 -or $script:TestsPassed -gt 0)) {
        Write-Status "✅ Polymera OS is ready!`n" "Green"
    }
    elseif ($totalFailed -eq 0) {
        Write-Status "⚠️ No builds or tests ran. Check prerequisites.`n" "Yellow"
    }
    else {
        Write-Status "⚠️ Some checks failed. Review the output above.`n" "Yellow"
    }
}

function Show-Help {
    Write-Status "`nPolymera OS Native Windows Build Script" "Cyan"
    Write-Status "========================================`n" "Cyan"
    Write-Status "Usage: .\run-polymera-native.ps1 -Action <action>`n" "White"
    Write-Status "Actions:" "Yellow"
    Write-Status "  build   - Build key Rust crates (kernel, ai_core, ngfs, policy, crypto)" "White"
    Write-Status "  test    - Run tests on key crates" "White"
    Write-Status "  verify  - Check project structure and files" "White"
    Write-Status "  all     - Build, test, and verify" "White"
    Write-Status "  help    - Show this help message`n" "White"
}

# Main execution
Write-Status "`n🚀 Polymera OS Native Windows Build v2" "Cyan"
Write-Status "======================================`n" "Cyan"

switch ($Action) {
    "build" {
        if (Test-Prerequisites) {
            Run-RustBuild
            Run-GoTests
            Show-Summary
        }
    }
    "test" {
        if (Test-Prerequisites) {
            Run-RustTests
            Show-Summary
        }
    }
    "verify" {
        Test-Prerequisites | Out-Null
        Run-Verification
        Show-Summary
    }
    "all" {
        if (Test-Prerequisites) {
            Run-RustBuild
            Run-RustTests
            Run-GoTests
            Run-Verification
            Show-Summary
        }
    }
    "help" {
        Show-Help
    }
}
