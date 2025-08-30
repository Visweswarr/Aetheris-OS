@echo off
REM QEMU runner for Windows
setlocal

set KIMG=%1
if "%KIMG%"=="" set KIMG=bazel-bin\kernel\polymera-kernel.bin

REM Check if kernel image exists
if not exist "%KIMG%" (
    echo Error: Kernel image not found at %KIMG%
    echo Build it first with: bazel build //kernel:kernel_image
    exit /b 1
)

echo Starting QEMU with kernel: %KIMG%
echo QEMU output will be saved to serial.log

REM Run QEMU (output goes to console and serial.log will be created by redirection)
qemu-system-x86_64.exe ^
    -kernel "%KIMG%" ^
    -serial stdio ^
    -display none ^
    -no-reboot ^
    -no-shutdown ^
    -m 512

REM Note: Windows doesn't have 'tee' by default, so we rely on QEMU's serial output
