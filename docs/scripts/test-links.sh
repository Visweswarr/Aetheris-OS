#!/bin/bash
# Link testing script for Polymera OS documentation

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Testing Documentation Links${NC}"

DOCS_DIR="$(dirname "$0")/../site"
BUILD_DIR="$DOCS_DIR/build"
ERRORS=0

# Check if build directory exists
if [[ ! -d "$BUILD_DIR" ]]; then
    echo -e "${RED}Error: Build directory not found at $BUILD_DIR${NC}"
    echo "Please run 'bazel build //docs:site' first"
    exit 1
fi

# Function to test external links
test_external_links() {
    echo -e "${GREEN}Testing external links...${NC}"
    
    # Find all HTML files and extract external links
    find "$BUILD_DIR" -name "*.html" -type f | while read -r file; do
        # Extract external links (http/https)
        grep -oE 'href="https?://[^"]*"' "$file" 2>/dev/null | \
        sed 's/href="//;s/"//' | \
        sort -u | while read -r url; do
            # Skip certain URLs that are known to be problematic
            if [[ $url =~ (localhost|127\.0\.0\.1|example\.com|github\.com.*#.*) ]]; then
                continue
            fi
            
            echo -n "  Testing $url ... "
            
            # Test the URL with curl
            if curl -s --head --max-time 10 "$url" > /dev/null 2>&1; then
                echo -e "${GREEN}OK${NC}"
            else
                echo -e "${RED}FAILED${NC}"
                ((ERRORS++))
            fi
        done
    done
}

# Function to test internal links
test_internal_links() {
    echo -e "${GREEN}Testing internal links...${NC}"
    
    # Find all HTML files and extract internal links
    find "$BUILD_DIR" -name "*.html" -type f | while read -r file; do
        local relative_path=${file#$BUILD_DIR/}
        local base_dir=$(dirname "$file")
        
        echo -e "Checking ${YELLOW}$relative_path${NC}..."
        
        # Extract internal links (not starting with http/https)
        grep -oE 'href="[^"]*"' "$file" 2>/dev/null | \
        sed 's/href="//;s/"//' | \
        grep -v '^https\?://' | \
        sort -u | while read -r link; do
            # Skip anchors and fragments for now
            if [[ $link =~ ^# ]]; then
                continue
            fi
            
            # Remove fragment identifier
            local clean_link=${link%%#*}
            
            # Skip empty links
            if [[ -z "$clean_link" ]]; then
                continue
            fi
            
            # Resolve relative links
            local target_file
            if [[ $clean_link =~ ^/ ]]; then
                # Absolute path from site root
                target_file="$BUILD_DIR$clean_link"
            else
                # Relative path from current file
                target_file="$base_dir/$clean_link"
            fi
            
            # Normalize path
            target_file=$(realpath -m "$target_file" 2>/dev/null || echo "$target_file")
            
            # If it's a directory, look for index.html
            if [[ -d "$target_file" ]]; then
                target_file="$target_file/index.html"
            fi
            
            # Check if target exists
            if [[ ! -f "$target_file" ]]; then
                echo -e "${RED}  ERROR: Broken link: $link -> $target_file${NC}"
                ((ERRORS++))
            fi
        done
    done
}

# Function to test image links
test_image_links() {
    echo -e "${GREEN}Testing image links...${NC}"
    
    find "$BUILD_DIR" -name "*.html" -type f | while read -r file; do
        local relative_path=${file#$BUILD_DIR/}
        local base_dir=$(dirname "$file")
        
        # Extract image sources
        grep -oE 'src="[^"]*"' "$file" 2>/dev/null | \
        sed 's/src="//;s/"//' | \
        grep -v '^https\?://' | \
        grep -v '^data:' | \
        sort -u | while read -r img_src; do
            # Resolve image path
            local img_file
            if [[ $img_src =~ ^/ ]]; then
                img_file="$BUILD_DIR$img_src"
            else
                img_file="$base_dir/$img_src"
            fi
            
            # Normalize path
            img_file=$(realpath -m "$img_file" 2>/dev/null || echo "$img_file")
            
            # Check if image exists
            if [[ ! -f "$img_file" ]]; then
                echo -e "${RED}  ERROR: Missing image: $img_src in $relative_path${NC}"
                ((ERRORS++))
            fi
        done
    done
}

# Function to check for duplicate IDs
check_duplicate_ids() {
    echo -e "${GREEN}Checking for duplicate HTML IDs...${NC}"
    
    find "$BUILD_DIR" -name "*.html" -type f | while read -r file; do
        local relative_path=${file#$BUILD_DIR/}
        
        # Extract all IDs from the file
        grep -oE 'id="[^"]*"' "$file" 2>/dev/null | \
        sed 's/id="//;s/"//' | \
        sort | uniq -d | while read -r duplicate_id; do
            if [[ -n "$duplicate_id" ]]; then
                echo -e "${RED}  ERROR: Duplicate ID '$duplicate_id' in $relative_path${NC}"
                ((ERRORS++))
            fi
        done
    done
}

# Run tests
test_internal_links
test_image_links
check_duplicate_ids

# Test external links only if requested
if [[ "${1:-}" == "--external" ]]; then
    echo -e "${YELLOW}External link testing enabled${NC}"
    test_external_links
else
    echo -e "${YELLOW}Skipping external link tests (use --external to enable)${NC}"
fi

# Final report
echo
if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}✓ All link tests passed${NC}"
    exit 0
else
    echo -e "${RED}✗ Found $ERRORS broken links${NC}"
    exit 1
fi
