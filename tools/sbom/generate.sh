#!/usr/bin/env bash
# SBOM Generation Script for Aetheris OS
# Generates Software Bill of Materials using syft

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ARTIFACTS_DIR="$PROJECT_ROOT/artifacts/sbom"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if syft is installed
check_syft() {
    if ! command -v syft >/dev/null 2>&1; then
        log_warning "syft is not installed. SBOM generation will be skipped."
        log_info "To install syft:"
        log_info "  curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin"
        log_info "  Or visit: https://github.com/anchore/syft/releases"
        return 1
    fi
    
    local syft_version
    syft_version=$(syft version --output json | jq -r '.version' 2>/dev/null || echo "unknown")
    log_info "Found syft version: $syft_version"
    return 0
}

# Create artifacts directory
setup_artifacts() {
    log_info "Setting up artifacts directory: $ARTIFACTS_DIR"
    mkdir -p "$ARTIFACTS_DIR"
}

# Generate SBOM for the entire project
generate_project_sbom() {
    log_info "Generating SBOM for entire project..."
    
    # Generate SPDX JSON format
    log_info "Generating SPDX JSON SBOM..."
    if syft packages "$PROJECT_ROOT" \
        --output spdx-json="$ARTIFACTS_DIR/sbom.spdx.json" \
        --file "$ARTIFACTS_DIR/sbom.spdx.json" \
        --name "aetheris-os" \
        --version "1.0.0" \
        --namespace "https://github.com/aetheris/aetheris-os" \
        --timestamp "$TIMESTAMP"; then
        log_success "SPDX JSON SBOM generated: $ARTIFACTS_DIR/sbom.spdx.json"
    else
        log_error "Failed to generate SPDX JSON SBOM"
        return 1
    fi
    
    # Generate CycloneDX JSON format
    log_info "Generating CycloneDX JSON SBOM..."
    if syft packages "$PROJECT_ROOT" \
        --output cyclonedx-json="$ARTIFACTS_DIR/sbom.cyclonedx.json" \
        --file "$ARTIFACTS_DIR/sbom.cyclonedx.json" \
        --name "aetheris-os" \
        --version "1.0.0" \
        --namespace "https://github.com/aetheris/aetheris-os" \
        --timestamp "$TIMESTAMP"; then
        log_success "CycloneDX JSON SBOM generated: $ARTIFACTS_DIR/sbom.cyclonedx.json"
    else
        log_error "Failed to generate CycloneDX JSON SBOM"
        return 1
    fi
}

# Generate SBOM for specific components
generate_component_sbom() {
    local component="$1"
    local component_dir="$PROJECT_ROOT/$component"
    
    if [[ ! -d "$component_dir" ]]; then
        log_warning "Component directory not found: $component_dir"
        return 0
    fi
    
    log_info "Generating SBOM for component: $component"
    
    # Generate SPDX JSON for component
    local spdx_file="$ARTIFACTS_DIR/sbom-$component.spdx.json"
    if syft packages "$component_dir" \
        --output spdx-json="$spdx_file" \
        --file "$spdx_file" \
        --name "aetheris-os-$component" \
        --version "1.0.0" \
        --namespace "https://github.com/aetheris/aetheris-os/$component" \
        --timestamp "$TIMESTAMP"; then
        log_success "Component SPDX JSON SBOM generated: $spdx_file"
    else
        log_warning "Failed to generate SPDX JSON SBOM for component: $component"
    fi
    
    # Generate CycloneDX JSON for component
    local cyclonedx_file="$ARTIFACTS_DIR/sbom-$component.cyclonedx.json"
    if syft packages "$component_dir" \
        --output cyclonedx-json="$cyclonedx_file" \
        --file "$cyclonedx_file" \
        --name "aetheris-os-$component" \
        --version "1.0.0" \
        --namespace "https://github.com/aetheris/aetheris-os/$component" \
        --timestamp "$TIMESTAMP"; then
        log_success "Component CycloneDX JSON SBOM generated: $cyclonedx_file"
    else
        log_warning "Failed to generate CycloneDX JSON SBOM for component: $component"
    fi
}

# Generate SBOM for container images (if any)
generate_container_sbom() {
    log_info "Checking for container images..."
    
    # Look for Dockerfiles
    local dockerfiles
    dockerfiles=$(find "$PROJECT_ROOT" -name "Dockerfile*" -type f 2>/dev/null || true)
    
    if [[ -z "$dockerfiles" ]]; then
        log_info "No Dockerfiles found, skipping container SBOM generation"
        return 0
    fi
    
    log_info "Found Dockerfiles, generating container SBOMs..."
    
    while IFS= read -r dockerfile; do
        local dir_name
        dir_name=$(dirname "$dockerfile" | sed "s|$PROJECT_ROOT/||")
        local image_name
        image_name="aetheris-os-$(echo "$dir_name" | tr '/' '-')"
        
        log_info "Generating SBOM for container: $image_name"
        
        # Generate SPDX JSON for container
        local spdx_file="$ARTIFACTS_DIR/sbom-container-$image_name.spdx.json"
        if syft packages "$dockerfile" \
            --output spdx-json="$spdx_file" \
            --file "$spdx_file" \
            --name "$image_name" \
            --version "1.0.0" \
            --namespace "https://github.com/aetheris/aetheris-os/containers/$image_name" \
            --timestamp "$TIMESTAMP"; then
            log_success "Container SPDX JSON SBOM generated: $spdx_file"
        else
            log_warning "Failed to generate SPDX JSON SBOM for container: $image_name"
        fi
        
        # Generate CycloneDX JSON for container
        local cyclonedx_file="$ARTIFACTS_DIR/sbom-container-$image_name.cyclonedx.json"
        if syft packages "$dockerfile" \
            --output cyclonedx-json="$cyclonedx_file" \
            --file "$cyclonedx_file" \
            --name "$image_name" \
            --version "1.0.0" \
            --namespace "https://github.com/aetheris/aetheris-os/containers/$image_name" \
            --timestamp "$TIMESTAMP"; then
            log_success "Container CycloneDX JSON SBOM generated: $cyclonedx_file"
        else
            log_warning "Failed to generate CycloneDX JSON SBOM for container: $image_name"
        fi
        
    done <<< "$dockerfiles"
}

# Generate SBOM summary
generate_summary() {
    log_info "Generating SBOM summary..."
    
    local summary_file="$ARTIFACTS_DIR/sbom-summary.json"
    local total_files=0
    local spdx_files=0
    local cyclonedx_files=0
    
    # Count generated files
    if [[ -d "$ARTIFACTS_DIR" ]]; then
        total_files=$(find "$ARTIFACTS_DIR" -name "*.json" -type f | wc -l)
        spdx_files=$(find "$ARTIFACTS_DIR" -name "*.spdx.json" -type f | wc -l)
        cyclonedx_files=$(find "$ARTIFACTS_DIR" -name "*.cyclonedx.json" -type f | wc -l)
    fi
    
    # Create summary JSON
    cat > "$summary_file" << EOF
{
  "generated_at": "$TIMESTAMP",
  "generator": "syft",
  "generator_version": "$(syft version --output json | jq -r '.version' 2>/dev/null || echo 'unknown')",
  "project": {
    "name": "aetheris-os",
    "version": "1.0.0",
    "namespace": "https://github.com/aetheris/aetheris-os"
  },
  "statistics": {
    "total_sbom_files": $total_files,
    "spdx_files": $spdx_files,
    "cyclonedx_files": $cyclonedx_files
  },
  "files": [
$(find "$ARTIFACTS_DIR" -name "*.json" -type f -exec basename {} \; | sed 's/^/    "/' | sed 's/$/",/' | sed '$s/,$//')
  ]
}
EOF
    
    log_success "SBOM summary generated: $summary_file"
}

# Validate generated SBOMs
validate_sbom() {
    log_info "Validating generated SBOMs..."
    
    local validation_errors=0
    
    # Check if jq is available for validation
    if ! command -v jq >/dev/null 2>&1; then
        log_warning "jq not available, skipping JSON validation"
        return 0
    fi
    
    # Validate each JSON file
    while IFS= read -r -d '' file; do
        if ! jq empty "$file" 2>/dev/null; then
            log_error "Invalid JSON in SBOM file: $file"
            ((validation_errors++))
        else
            log_info "Validated: $(basename "$file")"
        fi
    done < <(find "$ARTIFACTS_DIR" -name "*.json" -type f -print0)
    
    if [[ $validation_errors -eq 0 ]]; then
        log_success "All SBOM files validated successfully"
    else
        log_error "Found $validation_errors validation errors"
        return 1
    fi
}

# Main function
main() {
    log_info "Starting SBOM generation for Aetheris OS"
    log_info "Project root: $PROJECT_ROOT"
    log_info "Artifacts directory: $ARTIFACTS_DIR"
    
    # Check if syft is available
    if ! check_syft; then
        log_warning "SBOM generation skipped due to missing syft"
        exit 0
    fi
    
    # Setup
    setup_artifacts
    
    # Generate SBOMs
    log_info "Generating project-wide SBOM..."
    if ! generate_project_sbom; then
        log_error "Failed to generate project SBOM"
        exit 1
    fi
    
    # Generate component-specific SBOMs
    log_info "Generating component-specific SBOMs..."
    for component in services tooling go ui; do
        generate_component_sbom "$component"
    done
    
    # Generate container SBOMs
    generate_container_sbom
    
    # Generate summary
    generate_summary
    
    # Validate generated SBOMs
    validate_sbom
    
    # List generated files
    log_info "Generated SBOM files:"
    find "$ARTIFACTS_DIR" -name "*.json" -type f -exec basename {} \; | sort | while read -r file; do
        log_info "  - $file"
    done
    
    log_success "SBOM generation completed successfully"
    log_info "SBOM files are available in: $ARTIFACTS_DIR"
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Generate Software Bill of Materials (SBOM) for Aetheris OS"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --version      Show version information"
        echo "  --validate     Only validate existing SBOM files"
        echo ""
        echo "Environment Variables:"
        echo "  SYFT_OUTPUT_DIR    Override output directory (default: artifacts/sbom)"
        echo "  SYFT_PROJECT_NAME  Override project name (default: aetheris-os)"
        echo "  SYFT_PROJECT_VERSION Override project version (default: 1.0.0)"
        exit 0
        ;;
    --version)
        echo "SBOM Generator for Aetheris OS"
        echo "Version: 1.0.0"
        if command -v syft >/dev/null 2>&1; then
            echo "Syft version: $(syft version --output json | jq -r '.version' 2>/dev/null || echo 'unknown')"
        else
            echo "Syft: not installed"
        fi
        exit 0
        ;;
    --validate)
        log_info "Validating existing SBOM files..."
        validate_sbom
        exit $?
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        log_info "Use --help for usage information"
        exit 1
        ;;
esac
