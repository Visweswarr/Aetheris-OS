# BOOT-GREEN Ledger

## Latest Boot Verification

| Field | Value |
|-------|-------|
| Timestamp | 2026-05-21 17:57:43 UTC |
| Status | PASS |
| Summary | Limine UEFI FAT boot marker captured |
| QEMU | C:\Program Files\qemu\qemu-system-x86_64.exe |
| Timeout | 10s |
| Matched Marker | POLYMERA |
| Serial Log | C:\polymera-os\build_out\qemu-serial.log |
| Serial Lines | 77 |
| Serial Bytes | 8278 |
| Stdout Log | C:\polymera-os\build_out\qemu-stdout.log |
| Stderr Log | C:\polymera-os\build_out\qemu-stderr.log |

## Attempts

| Attempt | Artifact | Status | Exit | Timed Out | Serial Bytes | Marker | Reason |
|---------|----------|--------|------|-----------|--------------|--------|--------|
| limine-uefi-fat | C:\polymera-os\build_out\iso | PASS | -1 | True | 8278 | POLYMERA | serial boot marker captured |

## QEMU Commands

| Attempt | Command |
|---------|---------|
| limine-uefi-fat | `C:\Program Files\qemu\qemu-system-x86_64.exe -drive if=pflash,format=raw,readonly=on,file=C:\Program Files\qemu\share\edk2-x86_64-code.fd -drive if=pflash,format=raw,file=C:\polymera-os\build_out\qemu-vars.fd -drive if=ide,format=raw,file=fat:rw:C:\polymera-os\build_out\iso -m 256M -serial file:C:\polymera-os\build_out\qemu-serial.log -display none -no-reboot -d guest_errors` |

## Next Suspected Fix

Boot marker is proven. Next boot-hardening target: move past early memory allocation panic and reach kernel main loop.

## History

- 2026-05-21 17:57:43 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:56:39 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:56:11 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:46:17 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:00:26 UTC : FAIL - QEMU ran but no Polymera serial boot marker was captured
- 2026-05-21 16:59:50 UTC : FAIL - QEMU ran but no Polymera serial boot marker was captured
- 2026-05-21 01:41:03 UTC : Boot verification SKIPPED
