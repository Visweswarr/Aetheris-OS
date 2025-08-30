@echo off
REM Build System Call Headers
REM 
REM This script generates system call headers from SYSCALLS.md and validates
REM that all generated artifacts are up to date.

setlocal enabledelayedexpansion

REM Script directory
set "SCRIPT_DIR=%~dp0"
set "PROJECT_ROOT=%SCRIPT_DIR%.."

echo 🔨 Building System Call Headers
echo Project root: %PROJECT_ROOT%
echo Script directory: %SCRIPT_DIR%
echo.

REM Check if Python 3 is available
python --version >nul 2>&1
if errorlevel 1 (
    echo ❌ Python 3 is required but not installed
    exit /b 1
)

REM Check if SYSCALLS.md exists
set "SYSCALLS_MD=%PROJECT_ROOT%\docs\abi\SYSCALLS.md"
if not exist "%SYSCALLS_MD%" (
    echo ❌ SYSCALLS.md not found at %SYSCALLS_MD%
    exit /b 1
)

echo 📖 Found SYSCALLS.md at: %SYSCALLS_MD%

REM Create output directories
echo 📁 Creating output directories...
if not exist "%PROJECT_ROOT%\kernel\src\syscall" mkdir "%PROJECT_ROOT%\kernel\src\syscall"
if not exist "%PROJECT_ROOT%\userland-stubs\include\polymera" mkdir "%PROJECT_ROOT%\userland-stubs\include\polymera"

REM Generate headers
echo 🔧 Generating system call headers...
cd /d "%PROJECT_ROOT%"
python "%SCRIPT_DIR%generate_syscall_headers.py" "%PROJECT_ROOT%"

REM Check if generation was successful
if errorlevel 1 (
    echo ❌ Header generation failed
    exit /b 1
)

echo ✅ Headers generated successfully

REM Validate generated files
echo 🔍 Validating generated files...

REM Check Rust header
set "RUST_HEADER=%PROJECT_ROOT%\kernel\src\syscall\generated.rs"
if not exist "%RUST_HEADER%" (
    echo ❌ Rust header not generated: %RUST_HEADER%
    exit /b 1
)

REM Check C header
set "C_HEADER=%PROJECT_ROOT%\userland-stubs\include\polymera\syscalls.h"
if not exist "%C_HEADER%" (
    echo ❌ C header not generated: %C_HEADER%
    exit /b 1
)

REM Check assembly constants
set "ASM_CONSTANTS=%PROJECT_ROOT%\kernel\src\syscall\generated.asm"
if not exist "%ASM_CONSTANTS%" (
    echo ❌ Assembly constants not generated: %ASM_CONSTANTS%
    exit /b 1
)

REM Check summary file
set "SUMMARY_FILE=%PROJECT_ROOT%\syscall_generation_summary.txt"
if not exist "%SUMMARY_FILE%" (
    echo ❌ Summary file not generated: %SUMMARY_FILE%
    exit /b 1
)

echo ✅ All generated files validated

REM Show generation summary
echo 📊 Generation Summary:
type "%SUMMARY_FILE%"

REM Check for git changes
echo 🔍 Checking for git changes...
git diff --quiet "%RUST_HEADER%" "%C_HEADER%" "%ASM_CONSTANTS%" >nul 2>&1
if errorlevel 1 (
    echo ⚠️  Generated files have changed
    echo Changed files:
    git diff --name-only "%RUST_HEADER%" "%C_HEADER%" "%ASM_CONSTANTS%" 2>nul
) else (
    echo ✅ No changes detected in generated files
)

REM Validate Rust syntax (if rustc is available)
echo 🔍 Validating Rust header syntax...
rustc --edition 2021 --crate-type lib --target x86_64-unknown-none --emit=metadata -o nul "%RUST_HEADER%" >nul 2>&1
if errorlevel 1 (
    echo ⚠️  rustc not available or Rust header syntax validation failed
) else (
    echo ✅ Rust header syntax is valid
)

REM Validate C header syntax (if gcc is available)
echo 🔍 Validating C header syntax...
gcc -fsyntax-only -std=c99 "%C_HEADER%" >nul 2>&1
if errorlevel 1 (
    echo ⚠️  gcc not available or C header syntax validation failed
) else (
    echo ✅ C header syntax is valid
)

REM Show file sizes
echo 📏 Generated file sizes:
for %%f in ("%RUST_HEADER%") do (
    for /f %%l in ('type "%%f" ^| find /c /v ""') do echo   Rust header: %%l lines
)
for %%f in ("%C_HEADER%") do (
    for /f %%l in ('type "%%f" ^| find /c /v ""') do echo   C header: %%l lines
)
for %%f in ("%ASM_CONSTANTS%") do (
    for /f %%l in ('type "%%f" ^| find /c /v ""') do echo   Assembly constants: %%l lines
)

echo.
echo 🎉 System call header generation completed successfully!
echo.
echo Generated files:
echo   - Rust: %RUST_HEADER%
echo   - C: %C_HEADER%
echo   - Assembly: %ASM_CONSTANTS%
echo   - Summary: %SUMMARY_FILE%
echo.
echo Next steps:
echo   1. Review generated files for correctness
echo   2. Commit changes if headers are updated
echo   3. Update kernel and user space code to use generated headers
echo   4. Run tests to ensure compatibility

endlocal
