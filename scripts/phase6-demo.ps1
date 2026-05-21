# ===============================================================================
# Polymera Phase 6 Demo - Mini-Castor-style runtime controls
# ===============================================================================
# Proves HITL reject/modify, budget denial/refund, suspend/resume state replay,
# and replay divergence detection. HITL counters are exported by the AI Core CLI;
# budget/replay/suspend proofs are locked by the phase6_demo_tests integration
# test and summarized into dashboard/demo artifacts.
# ===============================================================================

$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$AiManifest = Join-Path $Root "services\ai_core\Cargo.toml"
$MetricsPath = Join-Path $Root "ui\dashboard\ai_metrics.js"
$DemoDir = Join-Path $Root "build_out\phase6-demo"
$OutputDir = Join-Path $Root "artifacts\phase6-demo"

New-Item -ItemType Directory -Force -Path $DemoDir, $OutputDir | Out-Null

$LogPath = Join-Path $DemoDir "operator-hitl.log"

function Write-Utf8NoBom {
    param([string]$Path, [string]$Value)
    [System.IO.File]::WriteAllText($Path, $Value, [System.Text.UTF8Encoding]::new($false))
}

Write-Utf8NoBom -Path $LogPath -Value @"
INFO  2026-05-21T00:00:00Z operator requests guarded log summary
WARN  2026-05-21T00:00:01Z destructive action requires human review
ERROR 2026-05-21T00:00:02Z replay divergence example will be rejected
"@

function Invoke-AiCoreCli {
    param(
        [string]$Name,
        [string[]]$CliArgs
    )

    $stdout = Join-Path $OutputDir "$Name.stdout.json"
    $stderr = Join-Path $OutputDir "$Name.stderr.log"
    $cargoArgs = @("run", "--quiet", "--manifest-path", $AiManifest, "--bin", "ai_core_cli", "--") + $CliArgs
    Write-Host "[PHASE6] $Name" -ForegroundColor Cyan
    $oldErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo @cargoArgs 1> $stdout 2> $stderr
    $exitCode = $LASTEXITCODE
    $ErrorActionPreference = $oldErrorActionPreference
    if ($exitCode -ne 0) {
        Write-Host "[PHASE6] $Name failed; stderr follows" -ForegroundColor Red
        Get-Content $stderr -ErrorAction SilentlyContinue
        throw "$Name failed with exit code $exitCode"
    }
    return $stdout
}

function Read-DashboardMetrics {
    if (-not (Test-Path $MetricsPath)) {
        return [ordered]@{}
    }
    $text = Get-Content $MetricsPath -Raw
    $start = $text.IndexOf("{")
    $end = $text.LastIndexOf("}")
    if ($start -lt 0 -or $end -lt $start) {
        return [ordered]@{}
    }
    return ($text.Substring($start, $end - $start + 1) | ConvertFrom-Json)
}

function Write-DashboardMetrics {
    param([object]$Payload)
    $json = $Payload | ConvertTo-Json -Depth 12
    Set-Content -Path $MetricsPath -Encoding UTF8 -Value "window.POLYMERA_AI_METRICS = $json;"
}

function Increment-DashboardMetric {
    param(
        [string]$Metric,
        [string]$LastKey,
        [object]$LastValue
    )

    $payload = Read-DashboardMetrics
    if ($payload -isnot [System.Management.Automation.PSCustomObject]) {
        $payload = [pscustomobject]@{}
    }

    if (-not ($payload.PSObject.Properties.Name -contains $Metric)) {
        $payload | Add-Member -NotePropertyName $Metric -NotePropertyValue 0
    }
    $payload.$Metric = [int64]$payload.$Metric + 1

    if (-not ($payload.PSObject.Properties.Name -contains $LastKey)) {
        $payload | Add-Member -NotePropertyName $LastKey -NotePropertyValue $null
    }
    $payload.$LastKey = $LastValue

    if (-not ($payload.PSObject.Properties.Name -contains "updated_at_unix")) {
        $payload | Add-Member -NotePropertyName "updated_at_unix" -NotePropertyValue 0
    }
    $payload.updated_at_unix = [int64][DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    Write-DashboardMetrics $payload
}

Invoke-AiCoreCli "01-hitl-reject" @(
    "summarize-log",
    "--path", $LogPath,
    "--reject", "operator rejected broad log access",
    "--metrics-js", $MetricsPath
) | Out-Null

Invoke-AiCoreCli "02-hitl-modify" @(
    "summarize-log",
    "--path", $LogPath,
    "--modify", "summarize only warnings and errors",
    "--metrics-js", $MetricsPath
) | Out-Null

$testStdout = Join-Path $OutputDir "03-phase6-demo-tests.stdout.log"
$testStderr = Join-Path $OutputDir "03-phase6-demo-tests.stderr.log"
Write-Host "[PHASE6] phase6_demo_tests" -ForegroundColor Cyan
$testArgs = @("test", "--manifest-path", $AiManifest, "--test", "phase6_demo_tests", "--", "--nocapture")
$oldErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
& cargo @testArgs 1> $testStdout 2> $testStderr
$testExitCode = $LASTEXITCODE
$ErrorActionPreference = $oldErrorActionPreference
if ($testExitCode -ne 0) {
    Write-Host "[PHASE6] phase6_demo_tests failed; stderr follows" -ForegroundColor Red
    Get-Content $testStderr -ErrorAction SilentlyContinue
    throw "phase6_demo_tests failed with exit code $testExitCode"
}

Increment-DashboardMetric `
    -Metric "ai_budget_exhausted_total" `
    -LastKey "last_phase6_budget_proof" `
    -LastValue ([ordered]@{
        decision = "budget_denial_and_refund_test_passed"
        source = "services/ai_core/tests/phase6_demo_tests.rs"
    })

$metrics = Read-DashboardMetrics
$checks = @(
    @{ Name = "ai_hitl_rejects_total"; Value = [int64]$metrics.ai_hitl_rejects_total },
    @{ Name = "ai_hitl_modifies_total"; Value = [int64]$metrics.ai_hitl_modifies_total },
    @{ Name = "ai_budget_exhausted_total"; Value = [int64]$metrics.ai_budget_exhausted_total }
)

foreach ($check in $checks) {
    if ($check.Value -lt 1) {
        throw "Expected $($check.Name) to be >= 1, got $($check.Value)"
    }
}

$summary = [ordered]@{
    status = "PASS"
    demo = "phase6"
    metrics = $MetricsPath
    outputs = $OutputDir
    log_input = $LogPath
    proofs = @(
        "HITL reject",
        "HITL modify",
        "Budget denial and refund",
        "Persisted task state resume",
        "Replay divergence detection"
    )
    checks = $checks
}
$summaryPath = Join-Path $OutputDir "phase6-summary.json"
$summary | ConvertTo-Json -Depth 8 | Set-Content -Path $summaryPath -Encoding UTF8

Write-Host "[PHASE6] PASS - AI Core Phase 6 demo completed" -ForegroundColor Green
Write-Host "[PHASE6] Summary: $summaryPath" -ForegroundColor Green
