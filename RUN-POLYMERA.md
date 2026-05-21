# Run Polymera

This is the operator entrypoint for the currently verified slice.

## Current Truth

- Kernel build: `cargo check` is green.
- QEMU boot: `PASS` for the Limine UEFI FAT path when a Polymera serial marker is captured.
- Runtime after marker: not green yet. The current kernel reaches `_start()` and prints the Polymera banner, then panics during early memory allocation.
- Phase 4 AI demo: runnable on the host through `services/ai_core`.
- Phase 6 runtime-control demo: runnable on the host through `services/ai_core`.

Phase 4 and Phase 6 demos are host-side AI Core demos. They do not prove AI Core is executing inside the kernel.

## Build The Kernel

Run from the kernel directory so `kernel\.cargo\config.toml` is honored:

```powershell
cd C:\polymera-os\kernel
cargo build --release --features insecure-toy-crypto
cargo check
```

## Assemble The Boot Artifact

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\polymera-os\assemble_final.ps1
```

This populates:

- `C:\polymera-os\dist\boot\kernel.elf`
- `C:\polymera-os\build_out\iso`
- `C:\polymera-os\build_out\iso\EFI\BOOT\BOOTX64.EFI`
- `C:\polymera-os\build_out\iso\EFI\BOOT\limine.conf`
- `C:\polymera-os\artifacts\os\polymera-os-avengers.zip`

The assembly script downloads the official Limine `v12.3.0` binary release when needed and verifies SHA-256:

```text
2FAF7857D4AF93683391B8124EDE0DCB87ADE29B8D9F3F92ED5EB6D77B92F688
```

No bootloader binary is copied from reference OS folders.

## Boot Smoke Test

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\polymera-os\scripts\qemu-smoke.ps1
```

Expected current result:

- `C:\polymera-os\BOOT-GREEN.md` records `PASS`
- matched marker: `POLYMERA`
- serial log: `C:\polymera-os\build_out\qemu-serial.log`

The smoke test uses QEMU EDK2 firmware and boots `build_out\iso` as a UEFI FAT artifact through Limine. Direct `-kernel` is diagnostic only unless PVH metadata is added later.

## Phase 4 Demo

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\polymera-os\scripts\phase4-demo.ps1
```

This runs:

- `goal-plan`
- `browser-summarize`
- `browser-classify`
- `summarize-log --approve`
- `runtime-run --backend deterministic`

Outputs:

- `C:\polymera-os\artifacts\phase4-demo`
- real AI metrics in `C:\polymera-os\ui\dashboard\ai_metrics.js`

## Phase 6 Demo

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File C:\polymera-os\scripts\phase6-demo.ps1
```

This proves:

- HITL reject
- HITL modify
- budget denial and refund
- persisted task state resume
- replay divergence detection

Outputs:

- `C:\polymera-os\artifacts\phase6-demo`
- HITL and budget counters in `C:\polymera-os\ui\dashboard\ai_metrics.js`

## Dashboard

Open:

```text
C:\polymera-os\ui\dashboard\index.html
```

The global dashboard is still simulated. Only cards backed by `ui\dashboard\ai_metrics.js` should be treated as real.

## Known Next Fix

The kernel now boots far enough to print the banner. The next boot-hardening target is the early memory allocation panic:

```text
Memory allocation failed for layout: Layout { size: 32, align: 8 }
```

Fixing that should be the next kernel task before expanding higher-level OS claims.
