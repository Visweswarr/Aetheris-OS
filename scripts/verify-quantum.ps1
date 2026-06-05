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

if (-not $SkipCargo) {
    Invoke-Step "cargo check polymera-crypto" {
        cargo check --manifest-path crypto/Cargo.toml --features full
    }
    Invoke-Step "cargo test polymera-crypto KAT/smoke tests" {
        cargo test --manifest-path crypto/Cargo.toml --features full --lib
    }
}

Invoke-Step "kernel IPC fixed-pattern session-key guard" {
    $pqc = "kernel/src/ipc/pqc_helpers.rs"
    $bad = Select-String -Path $pqc -Pattern "0xAA|Simple pattern for demonstration|Keys should be deterministic"
    if ($bad) {
        $bad | ForEach-Object { Write-Error "$($_.Path):$($_.LineNumber): deterministic fake session key remains: $($_.Line)" }
        exit 1
    }
}

Invoke-Step "ZK forced-true circuit guard" {
    if (Test-Path "zk") {
        $bad = Get-ChildItem -Path "zk" -Recurse -Include "*.nr" | Select-String -Pattern "=\s*true\s*;"
        if ($bad) {
            $bad | ForEach-Object { Write-Error "$($_.Path):$($_.LineNumber): forced-true Noir constraint: $($_.Line)" }
            exit 1
        }
    }
}

Invoke-Step "verified-crypto scaffold guard" {
    if (-not (Test-Path "c/crypto/verified/README.md")) {
        throw "missing verified crypto scaffold at c/crypto/verified/README.md"
    }
}

Write-Host "Quantum/PQC verification completed"
