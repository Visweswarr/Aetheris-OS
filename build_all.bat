@echo off
echo [BUILD] Starting Polymera OS Polyglot Build...

echo [BUILD] 1/5 Building Rust Kernel...
cd kernel
cargo check
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..

echo [BUILD] 2/5 Building Go Filesystem Service (Biscuit Style)...
cd services\fs_go
go build .
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..\..

echo [BUILD] 3/5 Building Go Network Service (Biscuit Style)...
cd services\net_go
go build .
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..\..

echo [BUILD] 4/5 Building C# App Runtime (Cosmos Style)...
cd services\app_runtime_cs
dotnet build
if %errorlevel% neq 0 exit /b %errorlevel%
cd ..\..

echo [BUILD] 5/5 Building C++ Window Server (Serenity Style)...
cd services\window_server_cpp
if not exist build mkdir build
cd build
cmake ..
cmake --build .
cd ..\..\..

echo [BUILD] SUCCESS! All Polyglot components compiled.
