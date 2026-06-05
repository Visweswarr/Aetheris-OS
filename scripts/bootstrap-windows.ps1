# Aetheris OS Windows Bootstrap Script
# Installs development dependencies for Windows development

param(
    [switch]$Force,
    [switch]$SkipGitLfs,
    [switch]$SkipRust,
    [switch]$SkipGo,
    [switch]$SkipNode,
    [switch]$SkipPython,
    [switch]$SkipCmake,
    [switch]$SkipNinja,
    [switch]$SkipLlvm
)

$ErrorActionPreference = "Continue"

Write-Host "🚀 Aetheris OS Windows Bootstrap" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green
Write-Host ""

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")
if (-not $isAdmin) {
    Write-Warning "⚠️  Not running as administrator. Some installations may require elevated privileges."
    Write-Host ""
}

# Function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Function to install via winget
function Install-WingetPackage($package, $name) {
    if (Test-Command winget) {
        Write-Host "Installing $name via winget..." -ForegroundColor Yellow
        winget install --id $package --accept-package-agreements --accept-source-agreements --silent
        return $?
    } else {
        Write-Warning "winget not available, skipping $name"
        return $false
    }
}

# Function to install via chocolatey
function Install-ChocoPackage($package, $name) {
    if (Test-Command choco) {
        Write-Host "Installing $name via chocolatey..." -ForegroundColor Yellow
        choco install $package -y
        return $?
    } else {
        Write-Warning "chocolatey not available, skipping $name"
        return $false
    }
}

# Function to download and install from URL
function Install-FromUrl($url, $name, $installerArgs = @()) {
    Write-Host "Downloading and installing $name..." -ForegroundColor Yellow
    $tempFile = "$env:TEMP\$name-installer.exe"
    try {
        Invoke-WebRequest -Uri $url -OutFile $tempFile
        Start-Process -FilePath $tempFile -ArgumentList $installerArgs -Wait
        Remove-Item $tempFile -Force
        return $true
    } catch {
        Write-Error "Failed to install $name - $($_.Exception.Message)"
        return $false
    }
}

Write-Host "📋 Checking prerequisites..." -ForegroundColor Cyan

# Install Git LFS
if (-not $SkipGitLfs) {
    if (-not (Test-Command git-lfs)) {
        Write-Host "Installing Git LFS..." -ForegroundColor Yellow
        if (-not (Install-WingetPackage "Git.Git-LFS" "Git LFS")) {
            Install-ChocoPackage "git-lfs" "Git LFS"
        }
    } else {
        Write-Host "✅ Git LFS already installed" -ForegroundColor Green
    }
} else {
    Write-Host "⏭️  Skipping Git LFS installation" -ForegroundColor Yellow
}

# Install Rust
if (-not $SkipRust) {
    if (-not (Test-Command cargo)) {
        Write-Host "Installing Rust..." -ForegroundColor Yellow
        if (-not (Install-WingetPackage "Rustlang.Rust.MSVC" "Rust")) {
            # Fallback to rustup installer
            $rustupUrl = "https://win.rustup.rs/x86_64"
            Install-FromUrl $rustupUrl "rustup" @("-y")
        }
        
        # Add Rust to PATH for current session
        $env:PATH += ";$env:USERPROFILE\.cargo\bin"
    } else {
        Write-Host "[OK] Rust already installed" -ForegroundColor Green
    }
} else {
    Write-Host "[SKIP] Skipping Rust installation" -ForegroundColor Yellow
}

# Install Go
if (-not $SkipGo) {
    if (-not (Test-Command go)) {
        Write-Host "Installing Go..." -ForegroundColor Yellow
        if (-not (Install-WingetPackage "GoLang.Go" "Go")) {
            Install-ChocoPackage "golang" "Go"
        }
    } else {
        $goVersion = go version
        Write-Host "✅ Go already installed: $goVersion" -ForegroundColor Green
    }
} else {
    Write-Host "⏭️  Skipping Go installation" -ForegroundColor Yellow
}

# Install Node.js
if (-not $SkipNode) {
    if (-not (Test-Command node)) {
        Write-Host "Installing Node.js..." -ForegroundColor Yellow
        if (-not (Install-WingetPackage "OpenJS.NodeJS" "Node.js")) {
            Install-ChocoPackage "nodejs" "Node.js"
        }
    } else {
        $nodeVersion = node --version
        Write-Host "✅ Node.js already installed: $nodeVersion" -ForegroundColor Green
    }
} else {
    Write-Host "⏭️  Skipping Node.js installation" -ForegroundColor Yellow
}

# Install Python
if (-not $SkipPython) {
    if (-not (Test-Command python)) {
        Write-Host "Installing Python..." -ForegroundColor Yellow
        if (-not (Install-WingetPackage "Python.Python.3.11" "Python 3.11")) {
            Install-ChocoPackage "python311" "Python 3.11"
        }
    } else {
        $pythonVersion = python --version
        Write-Host "✅ Python already installed: $pythonVersion" -ForegroundColor Green
    }
} else {
    Write-Host "⏭️  Skipping Python installation" -ForegroundColor Yellow
}

# Install CMake
if (-not $SkipCmake) {
    if (-not (Test-Command cmake)) {
        Write-Host "Installing CMake..." -ForegroundColor Yellow
        if (-not (Install-WingetPackage "Kitware.CMake" "CMake")) {
            Install-ChocoPackage "cmake" "CMake"
        }
    } else {
        $cmakeVersion = cmake --version
        Write-Host "✅ CMake already installed: $cmakeVersion" -ForegroundColor Green
    }
} else {
    Write-Host "⏭️  Skipping CMake installation" -ForegroundColor Yellow
}

# Install Ninja
if (-not $SkipNinja) {
    if (-not (Test-Command ninja)) {
        Write-Host "Installing Ninja..." -ForegroundColor Yellow
        if (-not (Install-WingetPackage "Ninja-build.Ninja" "Ninja")) {
            Install-ChocoPackage "ninja" "Ninja"
        }
    } else {
        $ninjaVersion = ninja --version
        Write-Host "✅ Ninja already installed: $ninjaVersion" -ForegroundColor Green
    }
} else {
    Write-Host "⏭️  Skipping Ninja installation" -ForegroundColor Yellow
}

# Install LLVM
if (-not $SkipLlvm) {
    if (-not (Test-Command clang)) {
        Write-Host "Installing LLVM..." -ForegroundColor Yellow
        if (-not (Install-WingetPackage "LLVM.LLVM" "LLVM")) {
            Install-ChocoPackage "llvm" "LLVM"
        }
    } else {
        $clangVersion = clang --version
        Write-Host "✅ LLVM already installed: $clangVersion" -ForegroundColor Green
    }
} else {
    Write-Host "⏭️  Skipping LLVM installation" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🔧 Post-installation setup..." -ForegroundColor Cyan

# Refresh environment variables
Write-Host "Refreshing environment variables..." -ForegroundColor Yellow
$env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")

# Install Rust components if Rust is available
if (Test-Command cargo) {
    Write-Host "Installing Rust components..." -ForegroundColor Yellow
    rustup component add rustfmt clippy
    rustup toolchain install stable
}

# Install Go tools if Go is available
if (Test-Command go) {
    Write-Host "Installing Go tools..." -ForegroundColor Yellow
    go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest
}

# Install Node.js tools if Node is available
if (Test-Command npm) {
    Write-Host "Installing Node.js tools..." -ForegroundColor Yellow
    npm install -g typescript prettier markdown-link-check
}

# Install Python tools if Python is available
if (Test-Command pip) {
    Write-Host "Installing Python tools..." -ForegroundColor Yellow
    pip install black isort flake8 pytest ruff
}

Write-Host ""
Write-Host "📋 Final checklist..." -ForegroundColor Cyan

$tools = @(
    @{Name="Git LFS"; Command="git-lfs"},
    @{Name="Rust"; Command="cargo"},
    @{Name="Go"; Command="go"},
    @{Name="Node.js"; Command="node"},
    @{Name="Python"; Command="python"},
    @{Name="CMake"; Command="cmake"},
    @{Name="Ninja"; Command="ninja"},
    @{Name="LLVM"; Command="clang"}
)

$allInstalled = $true
foreach ($tool in $tools) {
    if (Test-Command $tool.Command) {
        Write-Host "✅ $($tool.Name)" -ForegroundColor Green
    } else {
        Write-Host "❌ $($tool.Name)" -ForegroundColor Red
        $allInstalled = $false
    }
}

Write-Host ""
if ($allInstalled) {
    Write-Host "🎉 Bootstrap completed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "1. Restart your terminal or run: refreshenv" -ForegroundColor White
    Write-Host "2. Run: make help" -ForegroundColor White
    Write-Host "3. Run: make bootstrap" -ForegroundColor White
    Write-Host "4. Run: make test" -ForegroundColor White
} else {
    Write-Host "⚠️  Bootstrap completed with some issues." -ForegroundColor Yellow
    Write-Host "Please install missing tools manually and restart your terminal." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "📚 Useful commands:" -ForegroundColor Cyan
Write-Host "  make help     - Show available targets" -ForegroundColor White
Write-Host "  make fmt      - Format all code" -ForegroundColor White
Write-Host "  make lint     - Lint all code" -ForegroundColor White
Write-Host "  make test     - Run tests" -ForegroundColor White
Write-Host "  make docs     - Validate documentation" -ForegroundColor White
Write-Host "  make package  - Package CLI tools" -ForegroundColor White

Write-Host ""
Write-Host "🔗 PATH hints:" -ForegroundColor Cyan
Write-Host "  Rust:     %USERPROFILE%\.cargo\bin" -ForegroundColor White
Write-Host "  Go:       %USERPROFILE%\go\bin" -ForegroundColor White
Write-Host "  Node.js:  %APPDATA%\npm" -ForegroundColor White
Write-Host "  Python:   %USERPROFILE%\AppData\Local\Programs\Python\Python311\Scripts" -ForegroundColor White
