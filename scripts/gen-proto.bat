@echo off
REM Protobuf generation script for AI Core Service (Windows)
REM Generates Rust, Go, TypeScript, and Python stubs from .proto files

setlocal enabledelayedexpansion

REM Configuration
set PROTO_DIR=services\ai_core\proto
set OUTPUT_DIR=generated
set RUST_OUTPUT=services\ai_core\src\generated
set GO_OUTPUT=go\tooling\ai_core
set TS_OUTPUT=tooling\ts\ai_core
set PYTHON_OUTPUT=tooling\python\ai_core

REM Colors for output (Windows doesn't support ANSI colors in cmd, so we'll use echo)
set RED=[91m
set GREEN=[92m
set YELLOW=[93m
set BLUE=[94m
set NC=[0m

REM Check if required tools are installed
:check_dependencies
echo %BLUE%Checking dependencies...%NC%

set missing_deps=

REM Check for protoc
protoc --version >nul 2>&1
if errorlevel 1 (
    set missing_deps=%missing_deps% protoc
)

REM Check for protoc-gen-go
protoc-gen-go --version >nul 2>&1
if errorlevel 1 (
    set missing_deps=%missing_deps% protoc-gen-go
)

REM Check for protoc-gen-go-grpc
protoc-gen-go-grpc --version >nul 2>&1
if errorlevel 1 (
    set missing_deps=%missing_deps% protoc-gen-go-grpc
)

REM Check for protoc-gen-ts
protoc-gen-ts --version >nul 2>&1
if errorlevel 1 (
    set missing_deps=%missing_deps% protoc-gen-ts
)

if not "%missing_deps%"=="" (
    echo %RED%Missing dependencies:%missing_deps%%NC%
    echo %YELLOW%Please install the missing dependencies:%NC%
    echo   - protoc: Protocol Buffers compiler
    echo   - protoc-gen-go: Go protobuf plugin
    echo   - protoc-gen-go-grpc: Go gRPC plugin
    echo   - protoc-gen-ts: TypeScript protobuf plugin
    exit /b 1
)

echo %GREEN%All dependencies found!%NC%
goto :eof

REM Create output directories
:create_directories
echo %BLUE%Creating output directories...%NC%

if not exist "%RUST_OUTPUT%" mkdir "%RUST_OUTPUT%"
if not exist "%GO_OUTPUT%" mkdir "%GO_OUTPUT%"
if not exist "%TS_OUTPUT%" mkdir "%TS_OUTPUT%"
if not exist "%PYTHON_OUTPUT%" mkdir "%PYTHON_OUTPUT%"

echo %GREEN%Output directories created!%NC%
goto :eof

REM Generate Rust code
:generate_rust
echo %BLUE%Generating Rust code...%NC%

cd services\ai_core
cargo build
if errorlevel 1 (
    echo %RED%Failed to build Rust code%NC%
    exit /b 1
)
cd ..\..

echo %GREEN%Rust code generated!%NC%
goto :eof

REM Generate Go code
:generate_go
echo %BLUE%Generating Go code...%NC%

protoc ^
    --proto_path="%PROTO_DIR%" ^
    --go_out="%GO_OUTPUT%" ^
    --go_opt=paths=source_relative ^
    --go-grpc_out="%GO_OUTPUT%" ^
    --go-grpc_opt=paths=source_relative ^
    "%PROTO_DIR%\ai_core.proto"

if errorlevel 1 (
    echo %RED%Failed to generate Go code%NC%
    exit /b 1
)

echo %GREEN%Go code generated!%NC%
goto :eof

REM Generate TypeScript code
:generate_typescript
echo %BLUE%Generating TypeScript code...%NC%

protoc ^
    --proto_path="%PROTO_DIR%" ^
    --ts_out="%TS_OUTPUT%" ^
    "%PROTO_DIR%\ai_core.proto"

if errorlevel 1 (
    echo %RED%Failed to generate TypeScript code%NC%
    exit /b 1
)

echo %GREEN%TypeScript code generated!%NC%
goto :eof

REM Generate Python code
:generate_python
echo %BLUE%Generating Python code...%NC%

protoc ^
    --proto_path="%PROTO_DIR%" ^
    --python_out="%PYTHON_OUTPUT%" ^
    "%PROTO_DIR%\ai_core.proto"

if errorlevel 1 (
    echo %RED%Failed to generate Python code%NC%
    exit /b 1
)

echo %GREEN%Python code generated!%NC%
goto :eof

REM Generate documentation
:generate_docs
echo %BLUE%Generating documentation...%NC%

if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

protoc ^
    --proto_path="%PROTO_DIR%" ^
    --doc_out="%OUTPUT_DIR%" ^
    --doc_opt=markdown,ai_core.md ^
    "%PROTO_DIR%\ai_core.proto"

if errorlevel 1 (
    echo %RED%Failed to generate documentation%NC%
    exit /b 1
)

echo %GREEN%Documentation generated!%NC%
goto :eof

REM Validate generated code
:validate_generated
echo %BLUE%Validating generated code...%NC%

REM Validate Rust code
if exist "%RUST_OUTPUT%" (
    echo %YELLOW%Validating Rust code...%NC%
    cd services\ai_core
    cargo check
    if errorlevel 1 (
        echo %RED%Rust validation failed%NC%
        exit /b 1
    )
    cd ..\..
    echo %GREEN%Rust validation passed!%NC%
)

REM Validate Go code
if exist "%GO_OUTPUT%" (
    echo %YELLOW%Validating Go code...%NC%
    cd "%GO_OUTPUT%"
    go mod tidy
    if errorlevel 1 (
        echo %RED%Go validation failed%NC%
        exit /b 1
    )
    go build ./...
    if errorlevel 1 (
        echo %RED%Go validation failed%NC%
        exit /b 1
    )
    cd ..\..\..
    echo %GREEN%Go validation passed!%NC%
)

REM Validate Python code
if exist "%PYTHON_OUTPUT%" (
    echo %YELLOW%Validating Python code...%NC%
    cd "%PYTHON_OUTPUT%"
    python -m py_compile ai_core_pb2.py
    if errorlevel 1 (
        echo %RED%Python validation failed%NC%
        exit /b 1
    )
    cd ..\..\..
    echo %GREEN%Python validation passed!%NC%
)

goto :eof

REM Clean generated files
:clean
echo %BLUE%Cleaning generated files...%NC%

if exist "%RUST_OUTPUT%" rmdir /s /q "%RUST_OUTPUT%"
if exist "%GO_OUTPUT%" rmdir /s /q "%GO_OUTPUT%"
if exist "%TS_OUTPUT%" rmdir /s /q "%TS_OUTPUT%"
if exist "%PYTHON_OUTPUT%" rmdir /s /q "%PYTHON_OUTPUT%"
if exist "%OUTPUT_DIR%" rmdir /s /q "%OUTPUT_DIR%"

echo %GREEN%Generated files cleaned!%NC%
goto :eof

REM Show usage
:usage
echo Usage: %~nx0 [command]
echo.
echo Commands:
echo   all        Generate all language stubs (default)
echo   rust       Generate Rust stubs only
echo   go         Generate Go stubs only
echo   ts         Generate TypeScript stubs only
echo   python     Generate Python stubs only
echo   docs       Generate documentation only
echo   validate   Validate generated code
echo   clean      Clean generated files
echo   check      Check dependencies
echo.
echo Examples:
echo   %~nx0                    # Generate all stubs
echo   %~nx0 rust              # Generate Rust stubs only
echo   %~nx0 clean             # Clean generated files
echo   %~nx0 validate          # Validate generated code
goto :eof

REM Main function
:main
echo %GREEN%AI Core Service Protobuf Generation%NC%
echo %GREEN%====================================%NC%

if "%1"=="--help" goto :usage
if "%1"=="-h" goto :usage

if "%1"=="check" (
    call :check_dependencies
    goto :end
)

if "%1"=="clean" (
    call :clean
    goto :end
)

if "%1"=="rust" (
    call :check_dependencies
    call :create_directories
    call :generate_rust
    goto :end
)

if "%1"=="go" (
    call :check_dependencies
    call :create_directories
    call :generate_go
    goto :end
)

if "%1"=="ts" (
    call :check_dependencies
    call :create_directories
    call :generate_typescript
    goto :end
)

if "%1"=="typescript" (
    call :check_dependencies
    call :create_directories
    call :generate_typescript
    goto :end
)

if "%1"=="python" (
    call :check_dependencies
    call :create_directories
    call :generate_python
    goto :end
)

if "%1"=="docs" (
    call :create_directories
    call :generate_docs
    goto :end
)

if "%1"=="validate" (
    call :validate_generated
    goto :end
)

if "%1"=="all" (
    call :check_dependencies
    call :create_directories
    call :generate_rust
    call :generate_go
    call :generate_typescript
    call :generate_python
    call :generate_docs
    call :validate_generated
    goto :end
)

if "%1"=="" (
    call :check_dependencies
    call :create_directories
    call :generate_rust
    call :generate_go
    call :generate_typescript
    call :generate_python
    call :generate_docs
    call :validate_generated
    goto :end
)

REM Unknown command
echo %RED%Unknown command: %1%NC%
call :usage
exit /b 1

:end
echo %GREEN%Protobuf generation completed!%NC%
