# ===============================================================================
# Polymera Phase 4 Demo - AI cognitive/browser/runtime slice
# ===============================================================================
# Runs the host-side AI Core Phase 4 path:
#   goal-plan -> browser-summarize -> browser-classify -> summarize-log -> runtime-run
# and writes real AI metrics into ui/dashboard/ai_metrics.js.
# ===============================================================================

$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$AiManifest = Join-Path $Root "services\ai_core\Cargo.toml"
$MetricsPath = Join-Path $Root "ui\dashboard\ai_metrics.js"
$DemoDir = Join-Path $Root "build_out\phase4-demo"
$OutputDir = Join-Path $Root "artifacts\phase4-demo"

New-Item -ItemType Directory -Force -Path $DemoDir, $OutputDir | Out-Null

$LogPath = Join-Path $DemoDir "sample-runtime.log"
$PagePath = Join-Path $DemoDir "sample-page.txt"

function Write-Utf8NoBom {
    param([string]$Path, [string]$Value)
    [System.IO.File]::WriteAllText($Path, $Value, [System.Text.UTF8Encoding]::new($false))
}

Write-Utf8NoBom -Path $LogPath -Value @"
INFO  2026-05-21T00:00:00Z kernel boot marker captured
WARN  2026-05-21T00:00:01Z heap pressure during early memory init
ERROR 2026-05-21T00:00:02Z allocation failed for layout size=32 align=8
INFO  2026-05-21T00:00:03Z qemu serial log preserved
"@

Write-Utf8NoBom -Path $PagePath -Value @"
Polymera OS operator console
The system runs deterministic local AI planning, browser/page assistance,
capability-gated log summarization, and a local runtime backend boundary.
"@

function Invoke-AiCoreCli {
    param(
        [string]$Name,
        [string[]]$CliArgs
    )

    $stdout = Join-Path $OutputDir "$Name.stdout.json"
    $stderr = Join-Path $OutputDir "$Name.stderr.log"
    $cargoArgs = @("run", "--quiet", "--manifest-path", $AiManifest, "--bin", "ai_core_cli", "--") + $CliArgs
    Write-Host "[PHASE4] $Name" -ForegroundColor Cyan
    $oldErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo @cargoArgs 1> $stdout 2> $stderr
    $exitCode = $LASTEXITCODE
    $ErrorActionPreference = $oldErrorActionPreference
    if ($exitCode -ne 0) {
        Write-Host "[PHASE4] $Name failed; stderr follows" -ForegroundColor Red
        Get-Content $stderr -ErrorAction SilentlyContinue
        throw "$Name failed with exit code $exitCode"
    }
    return $stdout
}

function Read-DashboardMetrics {
    $text = Get-Content $MetricsPath -Raw
    $start = $text.IndexOf("{")
    $end = $text.LastIndexOf("}")
    if ($start -lt 0 -or $end -lt $start) {
        throw "Unable to parse dashboard metrics at $MetricsPath"
    }
    return ($text.Substring($start, $end - $start + 1) | ConvertFrom-Json)
}

Invoke-AiCoreCli "01-goal-plan" @(
    "goal-plan",
    "--goal", "Summarize boot logs and plan the next safe kernel boot fix",
    "--metrics-js", $MetricsPath
) | Out-Null

Invoke-AiCoreCli "02-browser-summarize" @(
    "browser-summarize",
    "--input", $PagePath,
    "--max-chars", "500",
    "--approve",
    "--metrics-js", $MetricsPath
) | Out-Null

Invoke-AiCoreCli "03-browser-classify" @(
    "browser-classify",
    "--url", "https://secure-update-login.example/reset",
    "--text", "urgent password reset seed phrase verification required",
    "--approve"
) | Out-Null

Invoke-AiCoreCli "04-summarize-log" @(
    "summarize-log",
    "--path", $LogPath,
    "--approve",
    "--metrics-js", $MetricsPath
) | Out-Null

Invoke-AiCoreCli "05-runtime-run" @(
    "runtime-run",
    "--prompt", "Explain the current Polymera boot state in one deterministic paragraph.",
    "--backend", "deterministic",
    "--workload", "LogSummary",
    "--dashboard-metrics", $MetricsPath
) | Out-Null

$metrics = Read-DashboardMetrics
$checks = @(
    @{ Name = "ai_plans_generated_total"; Value = [int64]$metrics.ai_plans_generated_total },
    @{ Name = "ai_browser_summaries_total"; Value = [int64]$metrics.ai_browser_summaries_total },
    @{ Name = "ai_log_summaries_total"; Value = [int64]$metrics.ai_log_summaries_total },
    @{ Name = "ai_runtime_backend_executions_total"; Value = [int64]$metrics.ai_runtime_backend_executions_total }
)

foreach ($check in $checks) {
    if ($check.Value -lt 1) {
        throw "Expected $($check.Name) to be >= 1, got $($check.Value)"
    }
}

$summary = [ordered]@{
    status = "PASS"
    demo = "phase4"
    metrics = $MetricsPath
    outputs = $OutputDir
    log_input = $LogPath
    page_input = $PagePath
    checks = $checks
}
$summaryPath = Join-Path $OutputDir "phase4-summary.json"
$summary | ConvertTo-Json -Depth 8 | Set-Content -Path $summaryPath -Encoding UTF8

Write-Host "[PHASE4] PASS - AI Core Phase 4 demo completed" -ForegroundColor Green
Write-Host "[PHASE4] Summary: $summaryPath" -ForegroundColor Green
