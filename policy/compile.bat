@echo off
REM OPA Policy Compilation Script for Polymera OS
REM This script compiles the p2_boot.rego policy to WASM format

echo [POLICY] Compiling OPA policy to WASM...

REM Check if OPA is installed
where opa >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] OPA not found. Please install OPA from https://www.openpolicyagent.org/
    echo [ERROR] After installation, ensure 'opa' is in your PATH
    exit /b 1
)

REM Check if policy source exists
if not exist "p2_boot.rego" (
    echo [ERROR] Policy source file p2_boot.rego not found
    exit /b 1
)

REM Create output directory if it doesn't exist
if not exist "out" mkdir out

REM Compile policy to WASM
echo [POLICY] Compiling p2_boot.rego to p2_boot.wasm...
opa build --target wasm --entrypoint polymera/policy/main p2_boot.rego --output out/p2_boot.wasm

if %errorlevel% neq 0 (
    echo [ERROR] Policy compilation failed
    exit /b 1
)

REM Verify the output file
if exist "out\p2_boot.wasm" (
    echo [POLICY] Policy compiled successfully
    echo [POLICY] Output: out\p2_boot.wasm
    
    REM Get file size
    for %%A in ("out\p2_boot.wasm") do echo [POLICY] File size: %%~zA bytes
) else (
    echo [ERROR] Expected output file not found
    exit /b 1
)

echo [POLICY] Policy compilation complete
echo [POLICY] Ready for kernel integration

