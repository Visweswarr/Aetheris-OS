# BOOT-GREEN Ledger

## Latest Boot Verification

| Field | Value |
|-------|-------|
| Timestamp | 2026-05-21 17:00:26 UTC |
| Status | FAIL |
| Summary | QEMU ran but no Polymera serial boot marker was captured |
| QEMU | C:\Program Files\qemu\qemu-system-x86_64.exe |
| Timeout | 10s |
| Matched Marker | N/A |
| Serial Log | C:\polymera-os\build_out\qemu-serial.log |
| Serial Lines | 0 |
| Serial Bytes | 0 |
| Stdout Log | C:\polymera-os\build_out\qemu-stdout.log |
| Stderr Log | C:\polymera-os\build_out\qemu-stderr.log |

## Attempts

| Attempt | Artifact | Status | Exit | Timed Out | Serial Bytes | Marker | Reason |
|---------|----------|--------|------|-----------|--------------|--------|--------|
| direct-kernel | C:\polymera-os\dist\boot\kernel.elf | FAIL | -1 | False | 0 | N/A | QEMU ran but serial output was empty; stderr: C:\Program Files\qemu\qemu-system-x86_64.exe: Error loading uncompressed kernel without PVH ELF Note |
| limine-layout | C:\polymera-os\build_out\iso\boot\limine\limine.cfg | FAIL | -1 | False | 0 | N/A | Limine config exists, but no bootable ISO or EFI loader exists |

## Next Suspected Fix

Direct -kernel boot is rejected by QEMU because the ELF lacks a PVH note. Build a real Limine ISO/EFI boot artifact from build_out\iso, or add the correct PVH/direct-boot metadata if direct -kernel is intended.

## History

- 2026-05-21 17:00:26 UTC : FAIL - QEMU ran but no Polymera serial boot marker was captured
- 2026-05-21 16:59:50 UTC : FAIL - QEMU ran but no Polymera serial boot marker was captured
- 2026-05-21 01:41:03 UTC : Boot verification SKIPPED
