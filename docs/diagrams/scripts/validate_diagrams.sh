#!/bin/bash
# Diagram validation script for Polymera OS

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Validating Polymera OS Diagrams${NC}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIAGRAMS_DIR="$(dirname "$SCRIPT_DIR")"
ERRORS=0

# Function to validate markdown files
validate_markdown() {
    local file="$1"
    local filename=$(basename "$file")
    
    echo -e "Validating ${YELLOW}$filename${NC}..."
    
    # Check if file exists and is readable
    if [[ ! -f "$file" ]] || [[ ! -r "$file" ]]; then
        echo -e "${RED}  ERROR: File not found or not readable: $file${NC}"
        ((ERRORS++))
        return 1
    fi
    
    # Check for required sections
    if ! grep -q "^# " "$file"; then
        echo -e "${RED}  ERROR: Missing main heading in $filename${NC}"
        ((ERRORS++))
    fi
    
    # Validate Mermaid syntax
    if grep -q "```mermaid" "$file"; then
        echo -e "  Found Mermaid diagrams"
        
        # Extract mermaid blocks and validate basic syntax
        awk '/```mermaid/,/```/' "$file" | while IFS= read -r line; do
            if [[ $line == "```mermaid" ]]; then
                continue
            elif [[ $line == "```" ]]; then
                continue
            elif [[ -n "$line" ]] && [[ ! $line =~ ^[[:space:]]*$ ]]; then
                # Check for common Mermaid syntax patterns
                if [[ $line =~ ^[[:space:]]*(graph|sequenceDiagram|classDiagram|stateDiagram|gantt|flowchart) ]]; then
                    echo -e "    Valid Mermaid diagram type found"
                    break
                fi
            fi
        done
    fi
    
    # Validate PlantUML syntax
    if grep -q "```plantuml" "$file"; then
        echo -e "  Found PlantUML diagrams"
        
        # Check for proper PlantUML start/end tags
        if ! grep -q "@startuml" "$file"; then
            echo -e "${YELLOW}  WARNING: PlantUML block missing @startuml tag${NC}"
        fi
        
        if ! grep -q "@enduml" "$file"; then
            echo -e "${YELLOW}  WARNING: PlantUML block missing @enduml tag${NC}"
        fi
    fi
    
    # Check for broken internal links
    grep -o '\[.*\](.*\.md)' "$file" 2>/dev/null | while IFS= read -r link; do
        # Extract the link target
        target=$(echo "$link" | sed 's/.*](\(.*\))/\1/')
        
        # Resolve relative paths
        if [[ $target =~ ^\.\./ ]]; then
            target_file="$(dirname "$file")/$target"
        elif [[ $target =~ ^/ ]]; then
            target_file="$DIAGRAMS_DIR$target"
        else
            target_file="$(dirname "$file")/$target"
        fi
        
        # Normalize path
        target_file=$(realpath -m "$target_file" 2>/dev/null || echo "$target_file")
        
        if [[ ! -f "$target_file" ]]; then
            echo -e "${RED}  ERROR: Broken link to $target${NC}"
            ((ERRORS++))
        fi
    done
    
    # Check for consistent heading levels
    local prev_level=0
    local line_num=0
    while IFS= read -r line; do
        ((line_num++))
        if [[ $line =~ ^#+[[:space:]] ]]; then
            local level=$(echo "$line" | grep -o '^#*' | wc -c)
            ((level--)) # Adjust for wc -c counting newline
            
            if [[ $level -gt $((prev_level + 1)) ]]; then
                echo -e "${YELLOW}  WARNING: Line $line_num: Heading level jumps from $prev_level to $level${NC}"
            fi
            prev_level=$level
        fi
    done < "$file"
    
    echo -e "  ${GREEN}✓${NC} Validation complete"
}

# Function to validate diagram categories
validate_category() {
    local category="$1"
    local category_dir="$DIAGRAMS_DIR/$category"
    
    echo -e "\n${GREEN}Validating $category diagrams...${NC}"
    
    if [[ ! -d "$category_dir" ]]; then
        echo -e "${RED}ERROR: Category directory not found: $category_dir${NC}"
        ((ERRORS++))
        return 1
    fi
    
    local diagram_count=0
    while IFS= read -r -d '' file; do
        validate_markdown "$file"
        ((diagram_count++))
    done < <(find "$category_dir" -name "*.md" -print0)
    
    if [[ $diagram_count -eq 0 ]]; then
        echo -e "${YELLOW}WARNING: No diagrams found in $category${NC}"
    else
        echo -e "${GREEN}Found $diagram_count diagram(s) in $category${NC}"
    fi
}

# Function to check diagram consistency
check_consistency() {
    echo -e "\n${GREEN}Checking diagram consistency...${NC}"
    
    # Check for consistent naming conventions
    find "$DIAGRAMS_DIR" -name "*.md" -not -name "README.md" | while IFS= read -r file; do
        filename=$(basename "$file" .md)
        
        # Check naming convention (kebab-case)
        if [[ ! $filename =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
            echo -e "${YELLOW}WARNING: File $filename doesn't follow kebab-case convention${NC}"
        fi
        
        # Check for proper title structure
        first_line=$(head -n 1 "$file")
        if [[ ! $first_line =~ ^#[[:space:]].+[[:space:]]Diagram$ ]]; then
            echo -e "${YELLOW}WARNING: File $filename doesn't have standard title format${NC}"
        fi
    done
    
    # Check for duplicate diagram IDs in Mermaid diagrams
    echo -e "Checking for duplicate diagram elements..."
    
    # Extract all Mermaid node IDs
    find "$DIAGRAMS_DIR" -name "*.md" -exec grep -H "^\s*[A-Z]\[" {} \; | \
    sed 's/.*:\s*\([A-Z]\)\[.*/\1/' | sort | uniq -d | while IFS= read -r duplicate; do
        if [[ -n "$duplicate" ]]; then
            echo -e "${YELLOW}WARNING: Duplicate Mermaid node ID found: $duplicate${NC}"
        fi
    done
}

# Function to validate color schemes and styling
validate_styling() {
    echo -e "\n${GREEN}Validating diagram styling...${NC}"
    
    # Check for consistent color class definitions in Mermaid diagrams
    find "$DIAGRAMS_DIR" -name "*.md" -exec grep -l "classDef" {} \; | while IFS= read -r file; do
        echo -e "Checking color consistency in $(basename "$file")..."
        
        # Count color class definitions
        color_count=$(grep -c "classDef" "$file" || true)
        if [[ $color_count -gt 10 ]]; then
            echo -e "${YELLOW}  WARNING: Many color classes ($color_count) - consider simplifying${NC}"
        fi
    done
    
    # Check for accessibility-friendly colors
    find "$DIAGRAMS_DIR" -name "*.md" -exec grep -l "fill:#" {} \; | while IFS= read -r file; do
        # Look for very light colors that might have contrast issues
        if grep -q "fill:#f[f-9][f-9][f-9][f-9][f-9]" "$file"; then
            echo -e "${YELLOW}  WARNING: Very light colors detected in $(basename "$file") - check contrast${NC}"
        fi
    done
}

# Main validation
echo -e "Diagram validation starting...\n"

# Validate README
if [[ -f "$DIAGRAMS_DIR/README.md" ]]; then
    validate_markdown "$DIAGRAMS_DIR/README.md"
else
    echo -e "${RED}ERROR: Missing README.md in diagrams directory${NC}"
    ((ERRORS++))
fi

# Validate each category
validate_category "kernel"
validate_category "polybus" 
validate_category "toolchains"
validate_category "ci-flow"

# Additional consistency checks
check_consistency
validate_styling

# Check for required files
echo -e "\n${GREEN}Checking required files...${NC}"

required_files=(
    "kernel/kernel-overview.md"
    "kernel/kernel-boot.md"
    "polybus/polybus-overview.md"
    "toolchains/build-system.md"
    "ci-flow/github-actions.md"
)

for file in "${required_files[@]}"; do
    if [[ ! -f "$DIAGRAMS_DIR/$file" ]]; then
        echo -e "${RED}ERROR: Required file missing: $file${NC}"
        ((ERRORS++))
    fi
done

# Final report
echo -e "\n${GREEN}Validation Summary${NC}"
echo -e "=================="

if [[ $ERRORS -eq 0 ]]; then
    echo -e "${GREEN}✓ All diagram validations passed${NC}"
    echo -e "Total files validated: $(find "$DIAGRAMS_DIR" -name "*.md" | wc -l)"
    exit 0
else
    echo -e "${RED}✗ Validation failed with $ERRORS error(s)${NC}"
    exit 1
fi
