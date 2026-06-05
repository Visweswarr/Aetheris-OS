# ===============================================================================
# QEMU Smoke Test - Boot kernel and verify serial output
# ===============================================================================
# Boot truth rules:
#   PASS = QEMU ran, no panic was captured, and POLYMERA_MAIN_LOOP_READY appeared.
#   FAIL = QEMU ran but no main-loop marker appeared, a panic appeared, or no bootable artifact exists.
#   SKIP = QEMU is not installed or boot verification is explicitly disabled.
# ===============================================================================

$ErrorActionPreference = "Stop"

$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $Root

$BuildOut = Join-Path $Root "build_out"
$SerialLog = Join-Path $BuildOut "qemu-serial.log"
$StdoutLog = Join-Path $BuildOut "qemu-stdout.log"
$StderrLog = Join-Path $BuildOut "qemu-stderr.log"
$QemuVars = Join-Path $BuildOut "qemu-vars.fd"
$LedgerPath = Join-Path $Root "BOOT-GREEN.md"
$TimeoutSec = 30

$RequiredBootMarker = "POLYMERA_MAIN_LOOP_READY"
$PanicMarkers = @(
    "KERNEL PANIC",
    "ALLOCATION ERROR",
    "DOUBLE PANIC",
    "panicked at"
)

Write-Host "[QEMU-SMOKE] Polymera OS Boot Verification" -ForegroundColor Cyan

function Resolve-ExistingPath {
    param([string]$Path)
    if (Test-Path $Path) {
        return (Resolve-Path $Path).Path
    }
    return $null
}

function Get-FileText {
    param([string]$Path)
    if (Test-Path $Path) {
        $content = Get-Content $Path -Raw -ErrorAction SilentlyContinue
        if ($null -eq $content) {
            return ""
        }
        return [string]$content
    }
    return ""
}

function Find-BootMarker {
    param([string]$SerialText)
    if ($SerialText -match [regex]::Escape($RequiredBootMarker)) {
        return $RequiredBootMarker
    }
    return ""
}

function Find-PanicMarker {
    param([string]$SerialText)
    foreach ($marker in $PanicMarkers) {
        if ($SerialText -match [regex]::Escape($marker)) {
            return $marker
        }
    }
    return ""
}

function New-AttemptResult {
    param(
        [string]$Name,
        [string]$Artifact,
        [string]$Status,
        [string]$Reason,
        [string]$MatchedMarker = "",
        [int]$ExitCode = -9999,
        [bool]$TimedOut = $false,
        [int]$SerialBytes = 0,
        [string]$Command = ""
    )

    return [pscustomobject]@{
        Name = $Name
        Artifact = $Artifact
        Status = $Status
        Reason = $Reason
        MatchedMarker = $MatchedMarker
        ExitCode = $ExitCode
        TimedOut = $TimedOut
        SerialBytes = $SerialBytes
        Command = $Command
    }
}

function Invoke-QemuAttempt {
    param(
        [string]$Name,
        [string]$Artifact,
        [string[]]$Arguments
    )

    Remove-Item $SerialLog, $StdoutLog, $StderrLog -Force -ErrorAction SilentlyContinue

    Write-Host "[QEMU-SMOKE] Attempt: $Name" -ForegroundColor Cyan
    Write-Host "[QEMU-SMOKE] Artifact: $Artifact" -ForegroundColor Gray
    Write-Host "[QEMU-SMOKE] Args: $($Arguments -join ' ')" -ForegroundColor DarkGray

    $quotedArguments = $Arguments | ForEach-Object {
        if ($_ -match '[\s"]') {
            '"' + ($_.Replace('"', '\"')) + '"'
        } else {
            $_
        }
    }

    $process = Start-Process -FilePath $script:QemuCmd `
        -ArgumentList $quotedArguments `
        -PassThru `
        -WindowStyle Hidden `
        -RedirectStandardOutput $StdoutLog `
        -RedirectStandardError $StderrLog

    $finished = $process.WaitForExit($TimeoutSec * 1000)
    $timedOut = -not $finished
    if ($timedOut) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        $process.WaitForExit() | Out-Null
    }

    $serial = Get-FileText $SerialLog
    $stderr = (Get-FileText $StderrLog).Trim()
    $marker = Find-BootMarker $serial
    $panicMarker = Find-PanicMarker $serial
    $serialBytes = [Text.Encoding]::UTF8.GetByteCount($serial)
    $exitCode = if ($process.ExitCode -ne $null) { [int]$process.ExitCode } else { -1 }
    $command = "$script:QemuCmd $($Arguments -join ' ')"

    if ($panicMarker) {
        Write-Host "[QEMU-SMOKE] FAIL: Kernel panic marker '$panicMarker' captured" -ForegroundColor Red
        return New-AttemptResult -Name $Name -Artifact $Artifact -Status "FAIL" -Reason "kernel panic marker captured: $panicMarker" -MatchedMarker $panicMarker -ExitCode $exitCode -TimedOut $timedOut -SerialBytes $serialBytes -Command $command
    }

    if ($marker) {
        Write-Host "[QEMU-SMOKE] PASS: Found main-loop marker '$marker'" -ForegroundColor Green
        return New-AttemptResult -Name $Name -Artifact $Artifact -Status "PASS" -Reason "main-loop marker captured without panic" -MatchedMarker $marker -ExitCode $exitCode -TimedOut $timedOut -SerialBytes $serialBytes -Command $command
    }

    if ($serialBytes -eq 0) {
        Write-Host "[QEMU-SMOKE] FAIL: QEMU ran but captured 0 serial bytes" -ForegroundColor Red
        $reason = "QEMU ran but serial output was empty"
        if ($stderr) {
            $firstStderrLine = ($stderr -split "`r?`n" | Select-Object -First 1)
            $reason = "$reason; stderr: $firstStderrLine"
        }
        return New-AttemptResult -Name $Name -Artifact $Artifact -Status "FAIL" -Reason $reason -ExitCode $exitCode -TimedOut $timedOut -SerialBytes $serialBytes -Command $command
    }

    Write-Host "[QEMU-SMOKE] FAIL: Serial output present but main-loop marker was not found" -ForegroundColor Red
    return New-AttemptResult -Name $Name -Artifact $Artifact -Status "FAIL" -Reason "serial output did not contain $RequiredBootMarker" -ExitCode $exitCode -TimedOut $timedOut -SerialBytes $serialBytes -Command $command
}

function Format-AttemptRows {
    param([object[]]$Attempts)
    $rows = New-Object System.Collections.Generic.List[string]
    $rows.Add("| Attempt | Artifact | Status | Exit | Timed Out | Serial Bytes | Marker | Reason |")
    $rows.Add("|---------|----------|--------|------|-----------|--------------|--------|--------|")
    foreach ($attempt in $Attempts) {
        $artifact = if ($attempt.Artifact) { $attempt.Artifact } else { "N/A" }
        $marker = if ($attempt.MatchedMarker) { $attempt.MatchedMarker } else { "N/A" }
        $rows.Add("| $($attempt.Name) | $artifact | $($attempt.Status) | $($attempt.ExitCode) | $($attempt.TimedOut) | $($attempt.SerialBytes) | $marker | $($attempt.Reason) |")
    }
    return ($rows -join "`n")
}

function Format-CommandRows {
    param([object[]]$Attempts)
    $rows = New-Object System.Collections.Generic.List[string]
    $rows.Add("| Attempt | Command |")
    $rows.Add("|---------|---------|")
    foreach ($attempt in $Attempts) {
        $command = if ($attempt.Command) { $attempt.Command.Replace("|", "\|") } else { "N/A" }
        $rows.Add("| $($attempt.Name) | ``$command`` |")
    }
    return ($rows -join "`n")
}

function Write-BootLedger {
    param(
        [string]$Status,
        [string]$Summary,
        [object[]]$Attempts,
        [string]$MatchedMarker,
        [string]$NextFix
    )

    $timestamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-dd HH:mm:ss 'UTC'")
    $serialLines = if (Test-Path $SerialLog) { (Get-FileText $SerialLog | Measure-Object -Line).Lines } else { 0 }
    $serialBytes = if (Test-Path $SerialLog) { [Text.Encoding]::UTF8.GetByteCount((Get-FileText $SerialLog)) } else { 0 }
    $attemptRows = Format-AttemptRows $Attempts
    $commandRows = Format-CommandRows $Attempts
    $markerValue = if ($MatchedMarker) { $MatchedMarker } else { "N/A" }
    $qemuValue = if ($script:QemuCmd) { $script:QemuCmd } else { "NOT FOUND" }

    $historyLine = "- $timestamp : $Status - $Summary"
    $oldHistory = ""
    if (Test-Path $LedgerPath) {
        $old = Get-FileText $LedgerPath
        $historyMatch = [regex]::Match($old, "(?ms)^## History\s*(.*)$")
        if ($historyMatch.Success) {
            $oldHistory = $historyMatch.Groups[1].Value.Trim()
        }
    }
    $history = if ($oldHistory) { "$historyLine`n$oldHistory" } else { $historyLine }

    $ledgerContent = @"
# BOOT-GREEN Ledger

## Latest Boot Verification

| Field | Value |
|-------|-------|
| Timestamp | $timestamp |
| Status | $Status |
| Summary | $Summary |
| QEMU | $qemuValue |
| Timeout | ${TimeoutSec}s |
| Required Marker | $RequiredBootMarker |
| Matched Marker | $markerValue |
| Serial Log | $SerialLog |
| Serial Lines | $serialLines |
| Serial Bytes | $serialBytes |
| Stdout Log | $StdoutLog |
| Stderr Log | $StderrLog |

## Attempts

$attemptRows

## QEMU Commands

$commandRows

## Next Suspected Fix

$NextFix

## History

$history
"@

    Set-Content -Path $LedgerPath -Value $ledgerContent -Encoding UTF8
    Write-Host "[QEMU-SMOKE] Ledger written to $LedgerPath" -ForegroundColor Green
}

if ($env:POLYMERA_SKIP_QEMU_SMOKE -eq "1") {
    $attempt = New-AttemptResult -Name "environment" -Artifact "N/A" -Status "SKIP" -Reason "POLYMERA_SKIP_QEMU_SMOKE=1"
    Write-BootLedger -Status "SKIP" -Summary "boot verification explicitly disabled" -Attempts @($attempt) -MatchedMarker "" -NextFix "Unset POLYMERA_SKIP_QEMU_SMOKE and rerun make qemu-smoke."
    exit 0
}

# ---- Locate QEMU ----
$QemuPaths = @(
    "qemu-system-x86_64.exe",
    "C:\Program Files\qemu\qemu-system-x86_64.exe",
    "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe",
    "$env:ProgramFiles\qemu\qemu-system-x86_64.exe"
)

$script:QemuCmd = $null
foreach ($candidate in $QemuPaths) {
    $resolved = Resolve-ExistingPath $candidate
    if ($resolved) {
        $script:QemuCmd = $resolved
        break
    }

    $command = Get-Command $candidate -ErrorAction SilentlyContinue
    if ($command) {
        $script:QemuCmd = $command.Source
        break
    }
}

if (-not $script:QemuCmd) {
    Write-Host "[QEMU-SMOKE] QEMU not found - install qemu-system-x86_64" -ForegroundColor Yellow
    $attempt = New-AttemptResult -Name "qemu-discovery" -Artifact "N/A" -Status "SKIP" -Reason "qemu-system-x86_64 not found"
    Write-BootLedger -Status "SKIP" -Summary "QEMU is not installed" -Attempts @($attempt) -MatchedMarker "" -NextFix "Install QEMU and rerun make qemu-smoke."
    exit 0
}

New-Item -ItemType Directory -Force -Path $BuildOut | Out-Null
Write-Host "[QEMU-SMOKE] Using QEMU: $script:QemuCmd" -ForegroundColor Gray

$attempts = New-Object System.Collections.Generic.List[object]

# ---- Attempt 1: real UEFI FAT boot artifact via Limine ----
$IsoDir = Resolve-ExistingPath (Join-Path $Root "build_out\iso")
$EfiPath = Resolve-ExistingPath (Join-Path $Root "build_out\iso\EFI\BOOT\BOOTX64.EFI")
$LimineConf = Resolve-ExistingPath (Join-Path $Root "build_out\iso\EFI\BOOT\limine.conf")
$FirmwarePath = Resolve-ExistingPath "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
$VarsTemplate = Resolve-ExistingPath "C:\Program Files\qemu\share\edk2-i386-vars.fd"

if ($IsoDir -and $EfiPath -and $LimineConf -and $FirmwarePath -and $VarsTemplate) {
    Copy-Item $VarsTemplate $QemuVars -Force
    $uefiArgs = @(
        "-drive", "if=pflash,format=raw,readonly=on,file=$FirmwarePath",
        "-drive", "if=pflash,format=raw,file=$QemuVars",
        "-drive", "if=ide,format=raw,file=fat:rw:$IsoDir",
        "-m", "256M",
        "-serial", "file:$SerialLog",
        "-display", "none",
        "-no-reboot",
        "-d", "guest_errors"
    )
    $uefiAttempt = Invoke-QemuAttempt -Name "limine-uefi-fat" -Artifact $IsoDir -Arguments $uefiArgs
    $attempts.Add($uefiAttempt)
    if ($uefiAttempt.Status -eq "PASS") {
        Write-BootLedger -Status "PASS" -Summary "Limine UEFI FAT main-loop marker captured without panic" -Attempts $attempts.ToArray() -MatchedMarker $uefiAttempt.MatchedMarker -NextFix "Boot MVP proven. Next target: wire a real kernel metric bridge from serial or a host-side control channel."
        exit 0
    }

    if ($uefiAttempt.SerialBytes -gt 0) {
        Write-BootLedger -Status "FAIL" -Summary "Limine UEFI FAT boot produced serial output but did not reach the main loop" -Attempts $attempts.ToArray() -MatchedMarker $uefiAttempt.MatchedMarker -NextFix "Primary boot path produced serial evidence. Inspect qemu-serial.log for the first missing marker or panic before trying direct-kernel diagnostics."
        exit 1
    }
} else {
    $missing = @()
    if (-not $IsoDir) { $missing += "build_out\iso" }
    if (-not $EfiPath) { $missing += "build_out\iso\EFI\BOOT\BOOTX64.EFI" }
    if (-not $LimineConf) { $missing += "build_out\iso\EFI\BOOT\limine.conf" }
    if (-not $FirmwarePath) { $missing += "QEMU edk2-x86_64-code.fd" }
    if (-not $VarsTemplate) { $missing += "QEMU edk2-i386-vars.fd" }
    $attempts.Add((New-AttemptResult -Name "limine-uefi-fat" -Artifact "build_out\iso" -Status "FAIL" -Reason "UEFI FAT boot artifact incomplete: $($missing -join ', ')" -ExitCode -1))
}

# ---- Attempt 2: direct kernel load diagnostic ----
$KernelPath = Resolve-ExistingPath (Join-Path $Root "dist\boot\kernel.elf")
if (-not $KernelPath) {
    $KernelPath = Resolve-ExistingPath (Join-Path $Root "build_out\iso\boot\kernel.elf")
}

if ($KernelPath) {
    $directArgs = @(
        "-kernel", $KernelPath,
        "-m", "256M",
        "-serial", "file:$SerialLog",
        "-display", "none",
        "-no-reboot",
        "-d", "guest_errors"
    )
    $direct = Invoke-QemuAttempt -Name "direct-kernel" -Artifact $KernelPath -Arguments $directArgs
    $attempts.Add($direct)
    if ($direct.Status -eq "PASS") {
        Write-BootLedger -Status "PASS" -Summary "direct kernel main-loop marker captured without panic" -Attempts $attempts.ToArray() -MatchedMarker $direct.MatchedMarker -NextFix "None. Keep this smoke test in CI before adding broader runtime claims."
        exit 0
    }
} else {
    $attempts.Add((New-AttemptResult -Name "direct-kernel" -Artifact "dist\boot\kernel.elf" -Status "FAIL" -Reason "kernel artifact missing; run assemble_final.ps1" -ExitCode -1))
}

# ---- Attempt 3: packaged Limine ISO path, only when actually bootable ----
$IsoPath = Resolve-ExistingPath (Join-Path $Root "artifacts\os\polymera-os-avengers.iso")
$LimineCfg = Resolve-ExistingPath (Join-Path $Root "build_out\iso\boot\limine\limine.cfg")

if ($IsoPath) {
    $isoArgs = @(
        "-cdrom", $IsoPath,
        "-m", "256M",
        "-serial", "file:$SerialLog",
        "-display", "none",
        "-no-reboot",
        "-d", "guest_errors"
    )
    $isoAttempt = Invoke-QemuAttempt -Name "limine-iso" -Artifact $IsoPath -Arguments $isoArgs
    $attempts.Add($isoAttempt)
    if ($isoAttempt.Status -eq "PASS") {
        Write-BootLedger -Status "PASS" -Summary "Limine ISO main-loop marker captured without panic" -Attempts $attempts.ToArray() -MatchedMarker $isoAttempt.MatchedMarker -NextFix "None. Keep this smoke test in CI before adding broader runtime claims."
        exit 0
    }
} elseif ($EfiPath) {
    $attempts.Add((New-AttemptResult -Name "limine-iso" -Artifact $EfiPath -Status "FAIL" -Reason "UEFI FAT path was attempted above; no bootable ISO artifact exists yet" -ExitCode -1))
} elseif ($LimineCfg) {
    $attempts.Add((New-AttemptResult -Name "limine-iso" -Artifact $LimineCfg -Status "FAIL" -Reason "Limine config exists, but no bootable ISO exists" -ExitCode -1))
} else {
    $attempts.Add((New-AttemptResult -Name "limine-iso" -Artifact "build_out\iso" -Status "FAIL" -Reason "packaged Limine layout missing" -ExitCode -1))
}

$stderrText = (Get-FileText $StderrLog).Trim()
if ($stderrText -match "PVH ELF Note") {
    $nextFix = "Direct -kernel boot is rejected by QEMU because the ELF lacks a PVH note. Build a real Limine ISO/EFI boot artifact from build_out\iso, or add the correct PVH/direct-boot metadata if direct -kernel is intended."
} else {
    $nextFix = "UEFI FAT, direct -kernel, and ISO diagnostics failed to capture POLYMERA_MAIN_LOOP_READY. Check early allocator bootstrap, MM initialization, kernel entry mapping, and serial output."
}
Write-BootLedger -Status "FAIL" -Summary "QEMU ran but no Polymera main-loop marker was captured" -Attempts $attempts.ToArray() -MatchedMarker "" -NextFix $nextFix
exit 1
