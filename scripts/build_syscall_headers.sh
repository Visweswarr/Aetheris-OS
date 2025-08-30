#!/bin/bash
# Build System Call Headers
# 
# This script generates system call headers from SYSCALLS.md and validates
# that all generated artifacts are up to date.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}🔨 Building System Call Headers${NC}"
echo "Project root: $PROJECT_ROOT"
echo "Script directory: $SCRIPT_DIR"
echo

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 is required but not installed${NC}"
    exit 1
fi

# Check if SYSCALLS.md exists
SYSCALLS_MD="$PROJECT_ROOT/docs/abi/SYSCALLS.md"
if [[ ! -f "$SYSCALLS_MD" ]]; then
    echo -e "${RED}❌ SYSCALLS.md not found at $SYSCALLS_MD${NC}"
    exit 1
fi

echo -e "${BLUE}📖 Found SYSCALLS.md at: $SYSCALLS_MD${NC}"

# Create output directories
echo -e "${BLUE}📁 Creating output directories...${NC}"
mkdir -p "$PROJECT_ROOT/kernel/src/syscall"
mkdir -p "$PROJECT_ROOT/userland-stubs/include/polymera"

# Generate headers
echo -e "${BLUE}🔧 Generating system call headers...${NC}"
cd "$PROJECT_ROOT"
python3 "$SCRIPT_DIR/generate_syscall_headers.py" "$PROJECT_ROOT"

# Check if generation was successful
if [[ $? -ne 0 ]]; then
    echo -e "${RED}❌ Header generation failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Headers generated successfully${NC}"

# Validate generated files
echo -e "${BLUE}🔍 Validating generated files...${NC}"

# Check Rust header
RUST_HEADER="$PROJECT_ROOT/kernel/src/syscall/generated.rs"
if [[ ! -f "$RUST_HEADER" ]]; then
    echo -e "${RED}❌ Rust header not generated: $RUST_HEADER${NC}"
    exit 1
fi

# Check C header
C_HEADER="$PROJECT_ROOT/userland-stubs/include/polymera/syscalls.h"
if [[ ! -f "$C_HEADER" ]]; then
    echo -e "${RED}❌ C header not generated: $C_HEADER${NC}"
    exit 1
fi

# Check assembly constants
ASM_CONSTANTS="$PROJECT_ROOT/kernel/src/syscall/generated.asm"
if [[ ! -f "$ASM_CONSTANTS" ]]; then
    echo -e "${RED}❌ Assembly constants not generated: $ASM_CONSTANTS${NC}"
    exit 1
fi

# Check summary file
SUMMARY_FILE="$PROJECT_ROOT/syscall_generation_summary.txt"
if [[ ! -f "$SUMMARY_FILE" ]]; then
    echo -e "${RED}❌ Summary file not generated: $SUMMARY_FILE${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All generated files validated${NC}"

# Show generation summary
echo -e "${BLUE}📊 Generation Summary:${NC}"
cat "$SUMMARY_FILE"

# Check for git changes
if command -v git &> /dev/null && git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${BLUE}🔍 Checking for git changes...${NC}"
    
    # Check if any generated files have changed
    if git diff --quiet "$RUST_HEADER" "$C_HEADER" "$ASM_CONSTANTS" 2>/dev/null; then
        echo -e "${GREEN}✅ No changes detected in generated files${NC}"
    else
        echo -e "${YELLOW}⚠️  Generated files have changed${NC}"
        echo "Changed files:"
        git diff --name-only "$RUST_HEADER" "$C_HEADER" "$ASM_CONSTANTS" 2>/dev/null || true
    fi
fi

# Validate Rust syntax
echo -e "${BLUE}🔍 Validating Rust header syntax...${NC}"
if command -v rustc &> /dev/null; then
    if rustc --edition 2021 --crate-type lib --target x86_64-unknown-none --emit=metadata -o /dev/null "$RUST_HEADER" 2>/dev/null; then
        echo -e "${GREEN}✅ Rust header syntax is valid${NC}"
    else
        echo -e "${RED}❌ Rust header syntax validation failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  rustc not available, skipping Rust syntax validation${NC}"
fi

# Validate C header syntax
echo -e "${BLUE}🔍 Validating C header syntax...${NC}"
if command -v gcc &> /dev/null; then
    if gcc -fsyntax-only -std=c99 "$C_HEADER" 2>/dev/null; then
        echo -e "${GREEN}✅ C header syntax is valid${NC}"
    else
        echo -e "${RED}❌ C header syntax validation failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠️  gcc not available, skipping C syntax validation${NC}"
fi

# Show file sizes
echo -e "${BLUE}📏 Generated file sizes:${NC}"
echo "  Rust header: $(wc -l < "$RUST_HEADER") lines"
echo "  C header: $(wc -l < "$C_HEADER") lines"
echo "  Assembly constants: $(wc -l < "$ASM_CONSTANTS") lines"

echo
echo -e "${GREEN}🎉 System call header generation completed successfully!${NC}"
echo
echo "Generated files:"
echo "  - Rust: $RUST_HEADER"
echo "  - C: $C_HEADER"
echo "  - Assembly: $ASM_CONSTANTS"
echo "  - Summary: $SUMMARY_FILE"
echo
echo "Next steps:"
echo "  1. Review generated files for correctness"
echo "  2. Commit changes if headers are updated"
echo "  3. Update kernel and user space code to use generated headers"
echo "  4. Run tests to ensure compatibility"
