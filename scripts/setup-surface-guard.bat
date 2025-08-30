@echo off
setlocal enabledelayedexpansion

echo 🔒 Polymera OS Surface Guard Setup
echo ==================================

where bazel >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Bazel not found. Please install Bazel first.
    exit /b 1
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Cargo not found. Please install Rust first.
    exit /b 1
)

echo 🔄 Building surface guard tool...
bazel build //tooling/guard:surface_check
if %errorlevel% neq 0 (
    echo ❌ Failed to build surface guard tool
    exit /b 1
)

echo 🔄 Regenerating golden surfaces...
bazel run //tooling/abi:gen
if %errorlevel% neq 0 (
    echo ❌ Failed to regenerate ABI surfaces
    exit /b 1
)

if exist "policy\compile.bat" (
    echo 🔄 Regenerating policy WASM...
    cd policy
    call compile.bat
    cd ..
    if %errorlevel% neq 0 (
        echo ❌ Failed to regenerate policy WASM
        exit /b 1
    )
)

echo 🔍 Calculating initial golden hashes...
bazel run //tooling/guard:surface_check -- --generate
if %errorlevel% neq 0 (
    echo ❌ Failed to generate initial hashes
    exit /b 1
)

echo ✅ Surface guard setup complete!
echo.
echo Next steps:
echo 1. Review SURFACE.lock.json for generated hashes
echo 2. Test: bazel run //tooling/guard:surface_check
echo 3. Commit: git add SURFACE.lock.json ^&^& git commit -m "Initialize surface guard"
echo.
echo The system will now protect all critical interfaces from accidental changes.
