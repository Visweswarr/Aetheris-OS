#!/bin/bash
# Documentation linting script for Polymera OS

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Linting Polymera OS Documentation${NC}"

DOCS_DIR="$(dirname "$0")/../site"
ERRORS=0

# Function to check markdown files
check_markdown() {
    local file="$1"
    local filename=$(basename "$file")
    
    echo -e "Checking ${YELLOW}$filename${NC}..."
    
    # Check for broken internal links
    if grep -q "](/" "$file"; then
        while IFS= read -r line; do
            if [[ $line =~ \]\(([^)]+)\) ]]; then
                link="${BASH_REMATCH[1]}"
                if [[ $link == /* ]] && [[ ! $link =~ ^https?:// ]]; then
                    # Internal link - check if target exists
                    target_file="${DOCS_DIR}${link}"
                    if [[ $link == *.md ]]; then
                        if [[ ! -f "$target_file" ]]; then
                            echo -e "${RED}  ERROR: Broken internal link: $link${NC}"
                            ((ERRORS++))
                        fi
                    fi
                fi
            fi
        done < "$file"
    fi
    
    # Check for TODO/FIXME comments
    if grep -q -E "(TODO|FIXME|XXX)" "$file"; then
        echo -e "${YELLOW}  WARNING: Found TODO/FIXME comments${NC}"
        grep -n -E "(TODO|FIXME|XXX)" "$file" | while read -r line; do
            echo -e "${YELLOW}    $line${NC}"
        done
    fi
    
    # Check for proper heading structure
    local prev_level=0
    local line_num=0
    while IFS= read -r line; do
        ((line_num++))
        if [[ $line =~ ^#+[[:space:]] ]]; then
            local level=$(echo "$line" | grep -o '^#*' | wc -c)
            ((level--)) # Adjust for wc -c counting newline
            
            if [[ $level -gt $((prev_level + 1)) ]]; then
                echo -e "${RED}  ERROR: Line $line_num: Heading level jumps from $prev_level to $level${NC}"
                echo -e "${RED}    $line${NC}"
                ((ERRORS++))
            fi
            prev_level=$level
        fi
    done < "$file"
    
    # Check for missing alt text in images
    if grep -q "!\[" "$file"; then
        while IFS= read -r line_num line; do
            if [[ $line =~ !\[\]\( ]]; then
                echo -e "${RED}  ERROR: Line $line_num: Image missing alt text${NC}"
                echo -e "${RED}    $line${NC}"
                ((ERRORS++))
            fi
        done < <(grep -n "!\[" "$file")
    fi
    
    # Check for long lines (>120 characters, excluding code blocks)
    local in_code_block=false
    local line_num=0
    while IFS= read -r line; do
        ((line_num++))
        
        # Track code block state
        if [[ $line =~ ^```.*$ ]]; then
            if $in_code_block; then
                in_code_block=false
            else
                in_code_block=true
            fi
            continue
        fi
        
        # Skip lines in code blocks
        if $in_code_block; then
            continue
        fi
        
        # Check line length
        if [[ ${#line} -gt 120 ]]; then
            echo -e "${YELLOW}  WARNING: Line $line_num exceeds 120 characters (${#line})${NC}"
        fi
    done < "$file"
}

# Check all markdown files
find "$DOCS_DIR" -name "*.md" -type f | while read -r file; do
    check_markdown "$file"
done

# Check for unused images
echo -e "${GREEN}Checking for unused images...${NC}"
if [[ -d "$DOCS_DIR/static/img" ]]; then
    find "$DOCS_DIR/static/img" -type f | while read -r img_file; do
        img_name=$(basename "$img_file")
        if ! grep -r "$img_name" "$DOCS_DIR/docs" "$DOCS_DIR/src" > /dev/null 2>&1; then
            echo -e "${YELLOW}  WARNING: Unused image: $img_name${NC}"
        fi
    done
fi

# Check package.json for security vulnerabilities
if [[ -f "$DOCS_DIR/package.json" ]]; then
    echo -e "${GREEN}Checking package.json for security issues...${NC}"
    cd "$DOCS_DIR"
    if command -v npm &> /dev/null; then
        if npm audit --audit-level moderate > /dev/null 2>&1; then
            echo -e "${GREEN}  No security vulnerabilities found${NC}"
        else
            echo -e "${YELLOW}  WARNING: Security vulnerabilities found in dependencies${NC}"
            npm audit --audit-level moderate
        fi
    fi
fi

# Final report
echo
if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}✓ Documentation linting completed successfully${NC}"
    exit 0
else
    echo -e "${RED}✗ Documentation linting found $ERRORS errors${NC}"
    exit 1
fi
