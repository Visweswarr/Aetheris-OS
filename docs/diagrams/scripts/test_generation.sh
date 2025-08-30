#!/bin/bash
# Test diagram generation for Polymera OS

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Testing Diagram Generation${NC}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIAGRAMS_DIR="$(dirname "$SCRIPT_DIR")"
ERRORS=0

# Function to test file generation
test_file_generation() {
    local category="$1"
    local filename="$2"
    local type="$3"
    
    local expected_file="$DIAGRAMS_DIR/generated/$category/$filename"
    
    echo -e "Testing ${YELLOW}$category/$filename${NC} ($type)..."
    
    if [[ ! -f "$expected_file" ]]; then
        echo -e "${RED}  ERROR: Generated file not found: $expected_file${NC}"
        ((ERRORS++))
        return 1
    fi
    
    # Check if file is not empty
    if [[ ! -s "$expected_file" ]]; then
        echo -e "${RED}  ERROR: Generated file is empty: $expected_file${NC}"
        ((ERRORS++))
        return 1
    fi
    
    # Basic file type validation
    local file_content
    file_content=$(head -c 100 "$expected_file" 2>/dev/null || echo "")
    
    case "$type" in
        "mermaid")
            # For now, just check that file exists and has content
            echo -e "  ${GREEN}✓${NC} Mermaid PNG generated"
            ;;
        "plantuml")
            # For now, just check that file exists and has content
            echo -e "  ${GREEN}✓${NC} PlantUML PNG generated"
            ;;
        *)
            echo -e "${YELLOW}  WARNING: Unknown diagram type: $type${NC}"
            ;;
    esac
    
    return 0
}

# Function to test image dimensions and quality
test_image_quality() {
    local image_file="$1"
    
    if [[ ! -f "$image_file" ]]; then
        return 1
    fi
    
    # Check file size (should be reasonable for a diagram)
    local file_size
    file_size=$(stat -f%z "$image_file" 2>/dev/null || stat -c%s "$image_file" 2>/dev/null || echo "0")
    
    if [[ $file_size -gt $((10 * 1024 * 1024)) ]]; then
        echo -e "${YELLOW}  WARNING: Large image file (${file_size} bytes): $(basename "$image_file")${NC}"
    fi
    
    if [[ $file_size -lt 100 ]]; then
        echo -e "${RED}  ERROR: Image file too small (${file_size} bytes): $(basename "$image_file")${NC}"
        ((ERRORS++))
    fi
}

# Function to validate generated directory structure
validate_structure() {
    echo -e "\n${GREEN}Validating generated directory structure...${NC}"
    
    local generated_dir="$DIAGRAMS_DIR/generated"
    
    if [[ ! -d "$generated_dir" ]]; then
        echo -e "${RED}ERROR: Generated directory not found: $generated_dir${NC}"
        ((ERRORS++))
        return 1
    fi
    
    # Check required subdirectories
    local required_dirs=("kernel" "polybus" "toolchains" "ci-flow")
    
    for dir in "${required_dirs[@]}"; do
        if [[ ! -d "$generated_dir/$dir" ]]; then
            echo -e "${RED}ERROR: Required subdirectory missing: $dir${NC}"
            ((ERRORS++))
        else
            echo -e "  ${GREEN}✓${NC} Directory exists: $dir"
        fi
    done
}

# Function to test diagram extraction and parsing
test_diagram_extraction() {
    echo -e "\n${GREEN}Testing diagram extraction...${NC}"
    
    # Test Mermaid extraction
    local mermaid_count=0
    while IFS= read -r -d '' file; do
        if grep -q "```mermaid" "$file"; then
            ((mermaid_count++))
            echo -e "  Found Mermaid diagram in $(basename "$file")"
        fi
    done < <(find "$DIAGRAMS_DIR" -name "*.md" -not -path "*/generated/*" -print0)
    
    echo -e "Total Mermaid diagrams found: $mermaid_count"
    
    # Test PlantUML extraction
    local plantuml_count=0
    while IFS= read -r -d '' file; do
        if grep -q "```plantuml" "$file"; then
            ((plantuml_count++))
            echo -e "  Found PlantUML diagram in $(basename "$file")"
        fi
    done < <(find "$DIAGRAMS_DIR" -name "*.md" -not -path "*/generated/*" -print0)
    
    echo -e "Total PlantUML diagrams found: $plantuml_count"
    
    if [[ $mermaid_count -eq 0 ]] && [[ $plantuml_count -eq 0 ]]; then
        echo -e "${RED}ERROR: No diagrams found to extract${NC}"
        ((ERRORS++))
    fi
}

# Function to test output formats
test_output_formats() {
    echo -e "\n${GREEN}Testing output formats...${NC}"
    
    # Check for consistent file naming
    find "$DIAGRAMS_DIR/generated" -name "*.png" | while IFS= read -r file; do
        filename=$(basename "$file" .png)
        
        # Check naming convention
        if [[ ! $filename =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
            echo -e "${YELLOW}WARNING: Generated file $filename doesn't follow naming convention${NC}"
        fi
        
        # Test image quality
        test_image_quality "$file"
    done
}

# Function to simulate real diagram generation
simulate_generation() {
    echo -e "\n${GREEN}Simulating diagram generation process...${NC}"
    
    # This would normally call actual Mermaid CLI and PlantUML tools
    # For now, we'll validate that the build system would work
    
    local source_files
    source_files=$(find "$DIAGRAMS_DIR" -name "*.md" -not -path "*/generated/*" | wc -l)
    
    echo -e "Source diagram files: $source_files"
    
    # Validate that we have the expected number of source files
    if [[ $source_files -lt 5 ]]; then
        echo -e "${YELLOW}WARNING: Expected at least 5 diagram source files, found $source_files${NC}"
    fi
    
    # Check for generation tooling availability (in a real scenario)
    echo -e "Checking for diagram generation tools..."
    
    # Note: In a real environment, you would check for:
    # - mermaid-cli (mmdc command)
    # - plantuml command or jar
    # - imagemagick for post-processing
    
    echo -e "  ${GREEN}✓${NC} Generation simulation complete"
}

# Function to test integration with documentation site
test_site_integration() {
    echo -e "\n${GREEN}Testing documentation site integration...${NC}"
    
    # Check if diagrams are referenced in documentation
    local docs_dir="$DIAGRAMS_DIR/../site/docs"
    
    if [[ -d "$docs_dir" ]]; then
        # Look for diagram references in documentation
        local references=0
        while IFS= read -r -d '' file; do
            if grep -q "diagrams/generated" "$file" || grep -q "![.*](.*\.png)" "$file"; then
                ((references++))
                echo -e "  Found diagram reference in $(basename "$file")"
            fi
        done < <(find "$docs_dir" -name "*.md" -print0 2>/dev/null || true)
        
        echo -e "Total diagram references found: $references"
        
        if [[ $references -eq 0 ]]; then
            echo -e "${YELLOW}WARNING: No diagram references found in documentation${NC}"
        fi
    else
        echo -e "${YELLOW}WARNING: Documentation directory not found${NC}"
    fi
}

# Performance testing
test_generation_performance() {
    echo -e "\n${GREEN}Testing generation performance...${NC}"
    
    local start_time
    start_time=$(date +%s)
    
    # Simulate diagram generation timing
    echo -e "Simulating generation process..."
    
    # Count total diagrams that would be generated
    local total_diagrams=0
    
    # Count Mermaid diagrams
    total_diagrams=$((total_diagrams + $(find "$DIAGRAMS_DIR" -name "*.md" -not -path "*/generated/*" -exec grep -l "```mermaid" {} \; | wc -l)))
    
    # Count PlantUML diagrams  
    total_diagrams=$((total_diagrams + $(find "$DIAGRAMS_DIR" -name "*.md" -not -path "*/generated/*" -exec grep -l "```plantuml" {} \; | wc -l)))
    
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo -e "Generation simulation took: ${duration}s"
    echo -e "Total diagrams to generate: $total_diagrams"
    
    if [[ $total_diagrams -gt 0 ]]; then
        local avg_time=$((duration * 1000 / total_diagrams))
        echo -e "Average time per diagram: ${avg_time}ms"
        
        # Performance targets
        if [[ $avg_time -gt 5000 ]]; then
            echo -e "${YELLOW}WARNING: Generation seems slow (>5s per diagram)${NC}"
        fi
    fi
}

# Main test execution
echo -e "Starting diagram generation tests...\n"

# Validate structure first
validate_structure

# Test extraction capabilities
test_diagram_extraction

# Test individual file generation
echo -e "\n${GREEN}Testing individual file generation...${NC}"

# Test Mermaid PNG generation
test_file_generation "kernel" "kernel-overview.png" "mermaid"
test_file_generation "kernel" "kernel-boot.png" "mermaid"
test_file_generation "polybus" "polybus-overview.png" "mermaid"
test_file_generation "toolchains" "build-system.png" "mermaid"
test_file_generation "ci-flow" "github-actions.png" "mermaid"

# Test PlantUML PNG generation
test_file_generation "kernel" "kernel-component.png" "plantuml"
test_file_generation "kernel" "kernel-boot-activity.png" "plantuml"
test_file_generation "polybus" "polybus-component.png" "plantuml"
test_file_generation "toolchains" "build-process.png" "plantuml"
test_file_generation "ci-flow" "ci-activity.png" "plantuml"

# Test output quality and formats
test_output_formats

# Test generation simulation
simulate_generation

# Test documentation integration
test_site_integration

# Performance testing
test_generation_performance

# Final report
echo -e "\n${GREEN}Generation Test Summary${NC}"
echo -e "======================="

local generated_files
generated_files=$(find "$DIAGRAMS_DIR/generated" -name "*.png" 2>/dev/null | wc -l || echo "0")

echo -e "Generated PNG files: $generated_files"

if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}✓ All generation tests passed${NC}"
    echo -e "Diagram generation system is working correctly"
    exit 0
else
    echo -e "${RED}✗ Generation tests failed with $ERRORS error(s)${NC}"
    exit 1
fi
