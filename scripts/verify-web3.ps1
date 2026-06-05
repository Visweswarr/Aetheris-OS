param(
    [switch]$SkipCargo
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

function Invoke-Step($Name, [scriptblock]$Body) {
    Write-Host "==> $Name"
    & $Body
}

Invoke-Step "Web3 manifest dependency guard" {
    $bad = Select-String -Path @(
        "services/wallet/Cargo.toml",
        "services/chain/Cargo.toml",
        "services/contracts/Cargo.toml",
        "services/dao/Cargo.toml"
    ) -Pattern "\.\./\.\./polymera-crypto|\.\./\.\./intent|\.\./cap-tokens|\.\./intent-kernel" -SimpleMatch:$false

    if ($bad) {
        $bad | ForEach-Object { Write-Error "$($_.Path):$($_.LineNumber): stale path dependency: $($_.Line)" }
        exit 1
    }
}

if (-not $SkipCargo) {
    Invoke-Step "cargo check wallet" {
        cargo check --manifest-path services/wallet/Cargo.toml
    }
    Invoke-Step "cargo check chain" {
        cargo check --manifest-path services/chain/Cargo.toml
    }
    Invoke-Step "cargo check dao" {
        cargo check --manifest-path services/dao/Cargo.toml
    }
    Invoke-Step "cargo check contracts" {
        cargo check --manifest-path services/contracts/Cargo.toml
    }
}

Invoke-Step "AnchorDAO hybrid signature policy guard" {
    $anchor = "contracts/AnchorDAO.sol"
    if (Test-Path $anchor) {
        $bad = Select-String -Path $anchor -Pattern "pqcSignature\.length\s*>\s*0|just check that signatures are not empty"
        if ($bad) {
            $bad | ForEach-Object { Write-Error "$($_.Path):$($_.LineNumber): weak PQC policy remains: $($_.Line)" }
            exit 1
        }
    }
}

Invoke-Step "WIT validation when repo-pinned wasm-tools exists" {
    $wasmTools = Join-Path $Root ".polymera-tools/bin/wasm-tools.exe"
    if (-not (Test-Path $wasmTools)) {
        $wasmTools = Join-Path $Root ".polymera-tools/bin/wasm-tools"
    }

    if (Test-Path $wasmTools) {
        Get-ChildItem -Path "wit/polymera" -Recurse -Filter "*.wit" |
            Select-Object -ExpandProperty DirectoryName -Unique |
            ForEach-Object {
            & $wasmTools component wit $_ | Out-Null
            if ($LASTEXITCODE -ne 0) {
                throw "WIT validation failed for $_"
            }
        }
    } else {
        Write-Host "repo-pinned wasm-tools not installed; run scripts/bootstrap-polyglot-tools.ps1 first"
    }
}

Write-Host "Web3 verification completed"
