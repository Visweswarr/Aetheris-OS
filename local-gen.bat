set PROTO_DIR=services\ai_core\proto
set OUTPUT_DIR=generated
set RUST_OUTPUT=services\ai_core\src\generated
set GO_OUTPUT=go\tooling\ai_core
set TS_OUTPUT=tooling\ts\ai_core
set PYTHON_OUTPUT=tooling\python\ai_core

if not exist "%RUST_OUTPUT%" mkdir "%RUST_OUTPUT%"
if not exist "%GO_OUTPUT%" mkdir "%GO_OUTPUT%"
if not exist "%TS_OUTPUT%" mkdir "%TS_OUTPUT%"
if not exist "%PYTHON_OUTPUT%" mkdir "%PYTHON_OUTPUT%"

set PATH=%PATH%;c:\polymera-os\protoc\bin;%USERPROFILE%\go\bin

echo Generating Go
protoc --proto_path="%PROTO_DIR%" --go_out="%GO_OUTPUT%" --go_opt=paths=source_relative --go-grpc_out="%GO_OUTPUT%" --go-grpc_opt=paths=source_relative "%PROTO_DIR%\ai_core.proto"

echo Generating TS
npx protoc-gen-ts --proto_path="%PROTO_DIR%" --ts_out="%TS_OUTPUT%" "%PROTO_DIR%\ai_core.proto" || echo "TS Generation skipped/failed"

echo Generating Python
python -m grpc_tools.protoc --proto_path="%PROTO_DIR%" --python_out="%PYTHON_OUTPUT%" "%PROTO_DIR%\ai_core.proto"
