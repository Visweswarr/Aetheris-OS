param(
    [switch]$SkipExternalTools
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$ToolRoot = if ($env:POLYMERA_TOOL_ROOT) { $env:POLYMERA_TOOL_ROOT } else { Join-Path $Root ".polymera-tools" }
$Failures = 0

function Resolve-PolymeraTool($Name) {
    $candidates = @(
        (Join-Path $ToolRoot "bin/$Name.exe"),
        (Join-Path $ToolRoot "bin/$Name"),
        (Join-Path $ToolRoot "node/node_modules/.bin/$Name.cmd"),
        (Join-Path $ToolRoot "node/node_modules/.bin/$Name"),
        (Join-Path $ToolRoot "python/Scripts/$Name.exe"),
        (Join-Path $ToolRoot "python/bin/$Name")
    )
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) { return $candidate }
    }
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return $null
}

function Run-Step($Name, [scriptblock]$Body) {
    Write-Host "==> $Name"
    try {
        & $Body
        Write-Host "ok: $Name"
    } catch {
        $script:Failures += 1
        Write-Host "failed: $Name`n$($_.Exception.Message)" -ForegroundColor Red
    }
}

function Require-Tool($Name) {
    $tool = Resolve-PolymeraTool $Name
    if (!$tool) {
        if ($SkipExternalTools) {
            Write-Warning "$Name not found; skipped because -SkipExternalTools is set"
            return $null
        }
        throw "$Name not found. Run scripts/bootstrap-polyglot-tools.ps1 first."
    }
    return $tool
}

Run-Step "WIT validation" {
    $wasmTools = Require-Tool "wasm-tools"
    if (!$wasmTools) { return }
    Get-ChildItem -Path (Join-Path $Root "wit/polymera") -Recurse -Directory |
        Where-Object { $_.Name -match '^\d+\.\d+\.\d+$' } |
        ForEach-Object {
            & $wasmTools component wit $_.FullName | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "wasm-tools failed for $($_.FullName)" }
        }
}

Run-Step "Rust checks" {
    $manifests = @(
        "services/ai_core/Cargo.toml",
        "services/wasm_driver/Cargo.toml",
        "services/supervisor/Cargo.toml",
        "runtime/examples/hello_component/Cargo.toml"
    )
    foreach ($manifest in $manifests) {
        cargo check --locked --manifest-path (Join-Path $Root $manifest)
        if ($LASTEXITCODE -ne 0) { throw "cargo check failed for $manifest" }
    }
}

Run-Step "Go checks" {
    $modules = @(
        "services/fs_go",
        "services/net_go",
        "go/tooling/ai_core"
    )
    foreach ($module in $modules) {
        $path = Join-Path $Root $module
        if (Test-Path (Join-Path $path "go.mod")) {
            Push-Location $path
            try {
                go test ./...
                if ($LASTEXITCODE -ne 0) { throw "go test failed for $module" }
            } finally { Pop-Location }
        }
    }
}

Run-Step "TypeScript checks" {
    $packages = @(
        "tooling/ts",
        "apps/assistant-ui"
    )
    foreach ($package in $packages) {
        $path = Join-Path $Root $package
        if ((Test-Path (Join-Path $path "package.json")) -and (Test-Path (Join-Path $path "node_modules"))) {
            npm --prefix $path run type-check
            if ($LASTEXITCODE -ne 0) { throw "npm type-check failed for $package" }
        } else {
            Write-Warning "skipping $package type-check; package.json or node_modules missing"
        }
    }
}

Run-Step "Raw service FFI allowlist" {
    $allowed = @(
        "services/ai/src/aicore.rs",
        "services/wallet/src/ffi.rs",
        "services/ngfs/fuse/main.rs"
    ) | ForEach-Object { [System.IO.Path]::GetFullPath((Join-Path $Root $_)).ToLower().Replace('\', '/') }

    $matches = @()
    $rg = Get-Command rg -ErrorAction SilentlyContinue
    if ($rg) {
        $matches = & rg -n 'extern\s+"C"' (Join-Path $Root "services") -g '!**/target/**' -g '!**/Cargo.lock' 2>$null
    } else {
        Write-Warning "rg not found, falling back to optimized Select-String"
        function Get-Files($Dir) {
            $files = @()
            foreach ($item in Get-ChildItem -Path $Dir -ErrorAction SilentlyContinue) {
                if ($item.PSIsContainer) {
                    if ($item.Name -eq "target" -or $item.Name -eq "node_modules" -or $item.Name -eq "dist" -or $item.Name -eq "build_out") { continue }
                    $files += Get-Files $item.FullName
                } else {
                    if ($item.Name -ne "Cargo.lock") {
                        $files += $item
                    }
                }
            }
            return $files
        }
        $files = Get-Files (Join-Path $Root "services")
        foreach ($file in $files) {
            $selectMatches = Select-String -Path $file.FullName -Pattern 'extern\s+"C"' -ErrorAction SilentlyContinue
            foreach ($match in $selectMatches) {
                $matches += "$($file.FullName):$($match.LineNumber):$($match.Line)"
            }
        }
    }

    foreach ($line in $matches) {
        if ($line -match '^([A-Za-z]:[^:]+):(\d+):(.*)$') {
            $path = $Matches[1]
        } else {
            $path = ($line -split ':', 2)[0]
        }
        $full = [System.IO.Path]::GetFullPath($path).ToLower().Replace('\', '/')
        if ($allowed -notcontains $full) {
            throw "raw service FFI outside allowlist: $line"
        }
    }
}

if ($Failures -gt 0) {
    throw "$Failures polyglot verification step(s) failed"
}

Write-Host "polyglot verification complete"
