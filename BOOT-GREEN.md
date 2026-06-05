# BOOT-GREEN Ledger

## Latest Boot Verification

| Field | Value |
|-------|-------|
| Timestamp | 2026-05-21 21:49:25 UTC |
| Status | PASS |
| Summary | Limine UEFI FAT main-loop marker captured without panic |
| QEMU | C:\Program Files\qemu\qemu-system-x86_64.exe |
| Timeout | 30s |
| Required Marker | POLYMERA_MAIN_LOOP_READY |
| Matched Marker | POLYMERA_MAIN_LOOP_READY |
| Serial Log | C:\polymera-os\build_out\qemu-serial.log |
| Serial Lines | 750 |
| Serial Bytes | 58238 |
| Stdout Log | C:\polymera-os\build_out\qemu-stdout.log |
| Stderr Log | C:\polymera-os\build_out\qemu-stderr.log |

## Attempts

| Attempt | Artifact | Status | Exit | Timed Out | Serial Bytes | Marker | Reason |
|---------|----------|--------|------|-----------|--------------|--------|--------|
| limine-uefi-fat | C:\polymera-os\build_out\iso | PASS | -1 | True | 58238 | POLYMERA_MAIN_LOOP_READY | main-loop marker captured without panic |

## QEMU Commands

| Attempt | Command |
|---------|---------|
| limine-uefi-fat | `C:\Program Files\qemu\qemu-system-x86_64.exe -drive if=pflash,format=raw,readonly=on,file=C:\Program Files\qemu\share\edk2-x86_64-code.fd -drive if=pflash,format=raw,file=C:\polymera-os\build_out\qemu-vars.fd -drive if=ide,format=raw,file=fat:rw:C:\polymera-os\build_out\iso -m 256M -serial file:C:\polymera-os\build_out\qemu-serial.log -display none -no-reboot -d guest_errors` |

## Next Suspected Fix

Boot MVP proven. Next target: wire a real kernel metric bridge from serial or a host-side control channel.

## History

- 2026-05-21 21:49:25 UTC : PASS - Limine UEFI FAT main-loop marker captured without panic
- 2026-05-21 21:48:07 UTC : PASS - Limine UEFI FAT main-loop marker captured without panic
- 2026-05-21 21:42:02 UTC : FAIL - Limine UEFI FAT boot produced serial output but did not reach the main loop
- 2026-05-21 21:38:26 UTC : FAIL - Limine UEFI FAT boot produced serial output but did not reach the main loop
- 2026-05-21 21:33:18 UTC : FAIL - Limine UEFI FAT boot produced serial output but did not reach the main loop
- 2026-05-21 21:27:00 UTC : FAIL - Limine UEFI FAT boot produced serial output but did not reach the main loop
- 2026-05-21 21:24:55 UTC : FAIL - Limine UEFI FAT boot produced serial output but did not reach the main loop
- 2026-05-21 20:51:20 UTC : PASS - Limine UEFI FAT main-loop marker captured without panic
- 2026-05-21 20:31:24 UTC : PASS - Limine UEFI FAT main-loop marker captured without panic
- 2026-05-21 20:30:42 UTC : PASS - Limine UEFI FAT main-loop marker captured without panic
- 2026-05-21 20:25:43 UTC : FAIL - QEMU ran but no Polymera main-loop marker was captured
- 2026-05-21 18:05:20 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:57:43 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:56:39 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:56:11 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:46:17 UTC : PASS - Limine UEFI FAT boot marker captured
- 2026-05-21 17:00:26 UTC : FAIL - QEMU ran but no Polymera serial boot marker was captured
- 2026-05-21 16:59:50 UTC : FAIL - QEMU ran but no Polymera serial boot marker was captured
- 2026-05-21 01:41:03 UTC : Boot verification SKIPPED
