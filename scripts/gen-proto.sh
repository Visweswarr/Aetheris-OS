#!/bin/bash
# Protobuf generation script for AI Core Service
# Generates Rust, Go, TypeScript, and Python stubs from .proto files

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROTO_DIR="services/ai_core/proto"
OUTPUT_DIR="generated"
RUST_OUTPUT="services/ai_core/src/generated"
GO_OUTPUT="go/tooling/ai_core"
TS_OUTPUT="tooling/ts/ai_core"
PYTHON_OUTPUT="tooling/python/ai_core"

# Check if required tools are installed
#check_dependencies() {
    echo -e "${BLUE}Checking dependencies...${NC}"
    
    local missing_deps=()
    
    # Check for protoc
    if ! command -v protoc &> /dev/null; then
        missing_deps+=("protoc")
    fi
    
    # Check for protoc-gen-go
    if ! command -v protoc-gen-go &> /dev/null; then
        missing_deps+=("protoc-gen-go")
    fi
    
    # Check for protoc-gen-go-grpc
    if ! command -v protoc-gen-go-grpc &> /dev/null; then
        missing_deps+=("protoc-gen-go-grpc")
    fi
    
    # Check for protoc-gen-ts
    if ! command -v protoc-gen-ts &> /dev/null; then
        missing_deps+=("protoc-gen-ts")
    fi
    
    # Check for protoc-gen-python
    if ! command -v protoc-gen-python &> /dev/null; then
        missing_deps+=("protoc-gen-python")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo -e "${RED}Missing dependencies: ${missing_deps[*]}${NC}"
        echo -e "${YELLOW}Please install the missing dependencies:${NC}"
        echo "  - protoc: Protocol Buffers compiler"
        echo "  - protoc-gen-go: Go protobuf plugin"
        echo "  - protoc-gen-go-grpc: Go gRPC plugin"
        echo "  - protoc-gen-ts: TypeScript protobuf plugin"
        echo "  - protoc-gen-python: Python protobuf plugin"
        exit 1
    fi
    
    echo -e "${GREEN}All dependencies found!${NC}"
}

# Create output directories
create_directories() {
    echo -e "${BLUE}Creating output directories...${NC}"
    
    mkdir -p "$RUST_OUTPUT"
    mkdir -p "$GO_OUTPUT"
    mkdir -p "$TS_OUTPUT"
    mkdir -p "$PYTHON_OUTPUT"
    
    echo -e "${GREEN}Output directories created!${NC}"
}

# Generate Rust code
generate_rust() {
    echo -e "${BLUE}Generating Rust code...${NC}"
    
    # Use prost-build for Rust generation
    cd services/ai_core
    cargo build
    cd ../..
    
    echo -e "${GREEN}Rust code generated!${NC}"
}

# Generate Go code
generate_go() {
    echo -e "${BLUE}Generating Go code...${NC}"
    
    protoc \
        --proto_path="$PROTO_DIR" \
        --go_out="$GO_OUTPUT" \
        --go_opt=paths=source_relative \
        --go-grpc_out="$GO_OUTPUT" \
        --go-grpc_opt=paths=source_relative \
        "$PROTO_DIR/ai_core.proto"
    
    # Create Go module file if it doesn't exist
    if [ ! -f "$GO_OUTPUT/go.mod" ]; then
        cat > "$GO_OUTPUT/go.mod" << EOF
module aetheris.ai_core

go 1.21

require (
    google.golang.org/grpc v1.59.0
    google.golang.org/protobuf v1.31.0
)
EOF
    fi
    
    echo -e "${GREEN}Go code generated!${NC}"
}

# Generate TypeScript code
generate_typescript() {
    echo -e "${BLUE}Generating TypeScript code...${NC}"
    
    protoc \
        --proto_path="$PROTO_DIR" \
        --ts_out="$TS_OUTPUT" \
        "$PROTO_DIR/ai_core.proto"
    
    # Create TypeScript package.json if it doesn't exist
    if [ ! -f "$TS_OUTPUT/package.json" ]; then
        cat > "$TS_OUTPUT/package.json" << EOF
{
  "name": "@aetheris/ai-core-proto",
  "version": "1.0.0",
  "description": "AI Core Service Protocol Buffers",
  "main": "index.js",
  "types": "index.d.ts",
  "scripts": {
    "build": "tsc",
    "test": "jest"
  },
  "dependencies": {
    "protobufjs": "^7.2.5"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0",
    "jest": "^29.0.0",
    "@types/jest": "^29.0.0"
  }
}
EOF
    fi
    
    # Create TypeScript index file
    cat > "$TS_OUTPUT/index.ts" << EOF
// AI Core Service Protocol Buffers
// Generated from ai_core.proto

export * from './ai_core_pb';
EOF
    
    echo -e "${GREEN}TypeScript code generated!${NC}"
}

# Generate Python code
generate_python() {
    echo -e "${BLUE}Generating Python code...${NC}"
    
    protoc \
        --proto_path="$PROTO_DIR" \
        --python_out="$PYTHON_OUTPUT" \
        "$PROTO_DIR/ai_core.proto"
    
    # Create Python __init__.py
    cat > "$PYTHON_OUTPUT/__init__.py" << EOF
# AI Core Service Protocol Buffers
# Generated from ai_core.proto

from . import ai_core_pb2
from . import ai_core_pb2_grpc

__all__ = ['ai_core_pb2', 'ai_core_pb2_grpc']
EOF
    
    # Create Python requirements.txt
    cat > "$PYTHON_OUTPUT/requirements.txt" << EOF
protobuf>=4.21.0
grpcio>=1.59.0
grpcio-tools>=1.59.0
EOF
    
    echo -e "${GREEN}Python code generated!${NC}"
}

# Generate documentation
generate_docs() {
    echo -e "${BLUE}Generating documentation...${NC}"
    
    # Generate Markdown documentation
    protoc \
        --proto_path="$PROTO_DIR" \
        --doc_out="$OUTPUT_DIR" \
        --doc_opt=markdown,ai_core.md \
        "$PROTO_DIR/ai_core.proto"
    
    echo -e "${GREEN}Documentation generated!${NC}"
}

# Validate generated code
validate_generated() {
    echo -e "${BLUE}Validating generated code...${NC}"
    
    # Validate Rust code
    if [ -d "$RUST_OUTPUT" ]; then
        echo -e "${YELLOW}Validating Rust code...${NC}"
        cd services/ai_core
        cargo check
        cd ../..
        echo -e "${GREEN}Rust validation passed!${NC}"
    fi
    
    # Validate Go code
    if [ -d "$GO_OUTPUT" ]; then
        echo -e "${YELLOW}Validating Go code...${NC}"
        cd "$GO_OUTPUT"
        go mod tidy
        go build ./...
        cd ../../..
        echo -e "${GREEN}Go validation passed!${NC}"
    fi
    
    # Validate TypeScript code
    if [ -d "$TS_OUTPUT" ]; then
        echo -e "${YELLOW}Validating TypeScript code...${NC}"
        cd "$TS_OUTPUT"
        npm install
        npm run build
        cd ../../..
        echo -e "${GREEN}TypeScript validation passed!${NC}"
    fi
    
    # Validate Python code
    if [ -d "$PYTHON_OUTPUT" ]; then
        echo -e "${YELLOW}Validating Python code...${NC}"
        cd "$PYTHON_OUTPUT"
        python -m py_compile ai_core_pb2.py
        cd ../../..
        echo -e "${GREEN}Python validation passed!${NC}"
    fi
}

# Clean generated files
clean() {
    echo -e "${BLUE}Cleaning generated files...${NC}"
    
    rm -rf "$RUST_OUTPUT"
    rm -rf "$GO_OUTPUT"
    rm -rf "$TS_OUTPUT"
    rm -rf "$PYTHON_OUTPUT"
    rm -rf "$OUTPUT_DIR"
    
    echo -e "${GREEN}Generated files cleaned!${NC}"
}

# Main function
main() {
    echo -e "${GREEN}AI Core Service Protobuf Generation${NC}"
    echo -e "${GREEN}====================================${NC}"
    
    case "${1:-all}" in
        "check")
            #check_dependencies
            ;;
        "clean")
            clean
            ;;
        "rust")
            #check_dependencies
            create_directories
            generate_rust
            ;;
        "go")
            #check_dependencies
            create_directories
            generate_go
            ;;
        "ts"|"typescript")
            #check_dependencies
            create_directories
            generate_typescript
            ;;
        "python")
            #check_dependencies
            create_directories
            generate_python
            ;;
        "docs")
            create_directories
            generate_docs
            ;;
        "validate")
            validate_generated
            ;;
        "all"|*)
            #check_dependencies
            create_directories
            generate_rust
            generate_go
            generate_typescript
            generate_python
            generate_docs
            validate_generated
            ;;
    esac
    
    echo -e "${GREEN}Protobuf generation completed!${NC}"
}

# Show usage
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  all        Generate all language stubs (default)"
    echo "  rust       Generate Rust stubs only"
    echo "  go         Generate Go stubs only"
    echo "  ts         Generate TypeScript stubs only"
    echo "  python     Generate Python stubs only"
    echo "  docs       Generate documentation only"
    echo "  validate   Validate generated code"
    echo "  clean      Clean generated files"
    echo "  check      Check dependencies"
    echo ""
    echo "Examples:"
    echo "  $0                    # Generate all stubs"
    echo "  $0 rust              # Generate Rust stubs only"
    echo "  $0 clean             # Clean generated files"
    echo "  $0 validate          # Validate generated code"
}

# Handle command line arguments
if [ $# -gt 0 ] && [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    usage
    exit 0
fi

# Run main function
main "$@"
