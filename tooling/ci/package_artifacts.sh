#!/bin/bash

# Artifact Packaging & Minidump Symbolization for Polymera OS
# This script consolidates logs, dumps, symbolized reports, and performance metrics
# into a single tarball per matrix cell for faster debugging.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
MAX_TARBALL_SIZE_MB=100
SCRUB_SECRETS=true

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

# Help function
show_help() {
    cat << EOF
Usage: $0 [OPTIONS] <config_id> <output_dir>

Artifact Packaging & Minidump Symbolization for Polymera OS

OPTIONS:
    -h, --help              Show this help message
    -s, --scrub-secrets     Scrub secrets and PII (default: true)
    -n, --no-scrub          Disable secret scrubbing
    -m, --max-size MB       Maximum tarball size in MB (default: 100)
    -v, --verbose           Enable verbose output
    -d, --debug             Enable debug mode

ARGUMENTS:
    config_id               Matrix configuration ID (e.g., apic_hpet_jitter_auth_open)
    output_dir              Output directory for the packaged artifacts

EXAMPLES:
    $0 apic_hpet_jitter_auth_open artifacts/
    $0 --max-size 200 apic_hpet_jitter_auth_open artifacts/
    $0 --no-scrub apic_hpet_jitter_auth_open artifacts/

This script will:
1. Collect all relevant artifacts for the matrix configuration
2. Run symbolizer on any minidumps to produce human-readable reports
3. Package everything into a consolidated tarball
4. Generate an index.md with links and summaries
5. Ensure the final package is under the specified size limit
EOF
}

# Parse command line arguments
VERBOSE=false
DEBUG=false
SCRUB_SECRETS=true
MAX_TARBALL_SIZE_MB=100

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -s|--scrub-secrets)
            SCRUB_SECRETS=true
            shift
            ;;
        -n|--no-scrub)
            SCRUB_SECRETS=false
            shift
            ;;
        -m|--max-size)
            MAX_TARBALL_SIZE_MB="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--debug)
            DEBUG=true
            VERBOSE=true
            shift
            ;;
        -*)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            break
            ;;
    esac
done

# Check required arguments
if [[ $# -lt 2 ]]; then
    log_error "Missing required arguments"
    show_help
    exit 1
fi

CONFIG_ID="$1"
OUTPUT_DIR="$2"

# Validate configuration ID
if [[ ! "$CONFIG_ID" =~ ^[a-zA-Z0-9_]+$ ]]; then
    log_error "Invalid configuration ID: $CONFIG_ID"
    exit 1
fi

# Set debug mode
if [[ "$DEBUG" == true ]]; then
    set -x
fi

log_info "Starting artifact packaging for configuration: $CONFIG_ID"
log_info "Output directory: $OUTPUT_DIR"
log_info "Max tarball size: ${MAX_TARBALL_SIZE_MB}MB"
log_info "Scrub secrets: $SCRUB_SECRETS"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Create temporary working directory
TEMP_DIR=$(mktemp -d)
log_info "Created temporary directory: $TEMP_DIR"

# Function to cleanup temporary directory
cleanup() {
    if [[ -d "$TEMP_DIR" ]]; then
        log_info "Cleaning up temporary directory: $TEMP_DIR"
        rm -rf "$TEMP_DIR"
    fi
}

# Set trap to cleanup on exit
trap cleanup EXIT

# Function to check if file exists and is readable
check_file() {
    local file="$1"
    if [[ -f "$file" && -r "$file" ]]; then
        return 0
    else
        return 1
    fi
}

# Function to get file size in MB
get_file_size_mb() {
    local file="$1"
    if [[ -f "$file" ]]; then
        local size_bytes=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo "0")
        echo "scale=2; $size_bytes / 1024 / 1024" | bc -l 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to scrub secrets from text files
scrub_secrets() {
    local file="$1"
    local output_file="$2"
    
    if [[ "$SCRUB_SECRETS" != true ]]; then
        cp "$file" "$output_file"
        return
    fi
    
    log_info "Scrubbing secrets from: $file"
    
    # Create a copy for scrubbing
    cp "$file" "$output_file"
    
    # Common secret patterns to scrub
    local patterns=(
        # GitHub tokens and secrets
        's/GH_TOKEN=[^[:space:]]*/GH_TOKEN=***/g'
        's/GITHUB_TOKEN=[^[:space:]]*/GITHUB_TOKEN=***/g'
        's/ghp_[a-zA-Z0-9]*/ghp_***/g'
        
        # API keys
        's/API_KEY=[^[:space:]]*/API_KEY=***/g'
        's/SECRET_KEY=[^[:space:]]*/SECRET_KEY=***/g'
        's/PRIVATE_KEY=[^[:space:]]*/PRIVATE_KEY=***/g'
        
        # Passwords
        's/PASSWORD=[^[:space:]]*/PASSWORD=***/g'
        's/PASSWD=[^[:space:]]*/PASSWD=***/g'
        
        # SSH keys
        's/ssh-rsa [a-zA-Z0-9+/]*[=]*/ssh-rsa ***/g'
        's/ssh-ed25519 [a-zA-Z0-9+/]*[=]*/ssh-ed25519 ***/g'
        
        # Docker registry credentials
        's/docker_password=[^[:space:]]*/docker_password=***/g'
        's/registry_password=[^[:space:]]*/registry_password=***/g'
        
        # Database credentials
        's/DB_PASSWORD=[^[:space:]]*/DB_PASSWORD=***/g'
        's/DATABASE_URL=[^[:space:]]*/DATABASE_URL=***/g'
        
        # JWT tokens
        's/eyJ[a-zA-Z0-9._-]*/eyJ***/g'
        
        # Generic token patterns
        's/[a-zA-Z0-9]{32,}/***/g'
    )
    
    # Apply all patterns
    for pattern in "${patterns[@]}"; do
        sed -i "$pattern" "$output_file" 2>/dev/null || true
    done
    
    log_info "Secrets scrubbed from: $file"
}

# Function to run symbolizer on minidumps
run_symbolizer() {
    local minidump_file="$1"
    local output_file="$2"
    
    if [[ ! -f "$minidump_file" ]]; then
        log_warning "Minidump file not found: $minidump_file"
        return 1
    fi
    
    log_info "Running symbolizer on: $minidump_file"
    
    # Check if symbolizer exists
    local symbolizer_path="$PROJECT_ROOT/tooling/crash/symbolizer"
    if [[ ! -f "$symbolizer_path" ]]; then
        log_warning "Symbolizer not found at: $symbolizer_path"
        log_warning "Creating placeholder symbolized report"
        cat > "$output_file" << EOF
# Symbolized Minidump Report

**Note**: Symbolizer not available. This is a placeholder report.

## Minidump Information
- File: $(basename "$minidump_file")
- Size: $(get_file_size_mb "$minidump_file") MB
- Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

## To Symbolize Manually
1. Ensure the symbolizer is built: \`cargo build --release\`
2. Run: \`./tooling/crash/symbolizer "$minidump_file" > "$output_file"\`

## Raw Minidump
The raw minidump file is available in the dumps/ directory.
EOF
        return 0
    fi
    
    # Run symbolizer
    if [[ "$VERBOSE" == true ]]; then
        log_info "Running: $symbolizer_path \"$minidump_file\" > \"$output_file\""
    fi
    
    if "$symbolizer_path" "$minidump_file" > "$output_file" 2>&1; then
        log_success "Symbolizer completed successfully"
        
        # Check if output file has meaningful content
        local line_count=$(wc -l < "$output_file" 2>/dev/null || echo "0")
        if [[ "$line_count" -lt 10 ]]; then
            log_warning "Symbolizer output seems minimal ($line_count lines)"
        fi
    else
        log_warning "Symbolizer failed, creating fallback report"
        cat > "$output_file" << EOF
# Symbolized Minidump Report

**Note**: Symbolizer failed to process this minidump. This is a fallback report.

## Minidump Information
- File: $(basename "$minidump_file")
- Size: $(get_file_size_mb "$minidump_file") MB
- Timestamp: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

## Error Information
The symbolizer encountered an error while processing this minidump.
Check the raw minidump file in the dumps/ directory for manual analysis.

## Common Issues
1. Missing debug symbols
2. Corrupted minidump file
3. Incompatible minidump format
4. Missing DWARF information

## Next Steps
1. Verify the minidump file integrity
2. Check if debug symbols are available
3. Try manual analysis with crash analysis tools
4. Contact the development team for assistance
EOF
    fi
}

# Function to collect and organize artifacts
collect_artifacts() {
    local config_id="$1"
    local temp_dir="$2"
    
    log_info "Collecting artifacts for configuration: $config_id"
    
    # Create directory structure
    mkdir -p "$temp_dir/logs"
    mkdir -p "$temp_dir/dumps"
    mkdir -p "$temp_dir/perf"
    mkdir -p "$temp_dir/tests"
    mkdir -p "$temp_dir/kernel"
    
    # Collect logs
    log_info "Collecting logs..."
    
    # Serial logs
    local serial_logs=(
        "tests/test_results/serial_*.log"
        "tests/test_results/*_tests.log"
        "tests/test_results/test_results.log"
        "kernel/kernel.log"
        "kernel/boot.log"
    )
    
    for pattern in "${serial_logs[@]}"; do
        for file in $pattern; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local scrubbed_file="$temp_dir/logs/${filename}"
                scrub_secrets "$file" "$scrubbed_file"
                log_info "Collected log: $file -> $scrubbed_file"
            fi
        done
    done
    
    # Collect minidumps and run symbolizer
    log_info "Collecting minidumps..."
    
    local minidump_files=(
        "tests/test_results/*.dmp"
        "tests/test_results/minidump_*.bin"
        "kernel/minidump_*.bin"
        "kernel/*.dmp"
    )
    
    local minidump_found=false
    for pattern in "${minidump_files[@]}"; do
        for file in $pattern; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local dump_file="$temp_dir/dumps/$filename"
                local symbolized_file="$temp_dir/dumps/${filename%.*}.txt"
                
                # Copy minidump
                cp "$file" "$dump_file"
                log_info "Collected minidump: $file -> $dump_file"
                
                # Run symbolizer
                run_symbolizer "$file" "$symbolized_file"
                minidump_found=true
            fi
        done
    done
    
    if [[ "$minidump_found" != true ]]; then
        log_info "No minidumps found for this configuration"
    fi
    
    # Collect performance metrics
    log_info "Collecting performance metrics..."
    
    local perf_files=(
        "perf/results/matrix/$config_id/metrics.json"
        "perf/results/matrix/$config_id/guardrails_result.json"
        "perf/results/matrix/$config_id/*.json"
    )
    
    for pattern in "${perf_files[@]}"; do
        for file in $pattern; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local perf_file="$temp_dir/perf/$filename"
                cp "$file" "$perf_file"
                log_info "Collected performance data: $file -> $perf_file"
            fi
        done
    done
    
    # Collect test results
    log_info "Collecting test results..."
    
    local test_files=(
        "tests/test_results/*.json"
        "tests/test_results/*.txt"
        "tests/test_results_summary.json"
    )
    
    for pattern in "${test_files[@]}"; do
        for file in $pattern; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local test_file="$temp_dir/tests/$filename"
                cp "$file" "$test_file"
                log_info "Collected test result: $file -> $test_file"
            fi
        done
    done
    
    # Collect kernel configuration
    log_info "Collecting kernel configuration..."
    
    local kernel_files=(
        "kernel/.config"
        "kernel/.config.matrix"
        "kernel/kernel_config.json"
    )
    
    for pattern in "${kernel_files[@]}"; do
        for file in $pattern; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local kernel_file="$temp_dir/kernel/$filename"
                cp "$file" "$kernel_file"
                log_info "Collected kernel config: $file -> $kernel_file"
            fi
        done
    done
    
    log_success "Artifact collection completed"
}

# Function to generate index.md
generate_index() {
    local config_id="$1"
    local temp_dir="$2"
    local matrix_name="${3:-$config_id}"
    
    log_info "Generating index.md for configuration: $config_id"
    
    local index_file="$temp_dir/index.md"
    
    cat > "$index_file" << EOF
# Artifact Package: $matrix_name

**Configuration ID**: \`$config_id\`  
**Generated**: $(date -u +"%Y-%m-%dT%H:%M:%SZ")  
**Package Size**: $(du -sh "$temp_dir" | cut -f1)

## Overview

This artifact package contains consolidated debugging information for the Polymera OS matrix configuration \`$config_id\`. It includes logs, minidumps, symbolized crash reports, performance metrics, and test results.

## Contents

### 📁 Logs
EOF
    
    # Add log files
    if [[ -d "$temp_dir/logs" ]]; then
        for file in "$temp_dir/logs"/*; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local size=$(get_file_size_mb "$file")
                echo "- **$filename** ($size MB) - [View](logs/$filename)" >> "$index_file"
            fi
        done
    fi
    
    cat >> "$index_file" << EOF

### 🚨 Dumps
EOF
    
    # Add dump files
    if [[ -d "$temp_dir/dumps" ]]; then
        for file in "$temp_dir/dumps"/*; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local size=$(get_file_size_mb "$file")
                local extension="${filename##*.}"
                
                if [[ "$extension" == "txt" ]]; then
                    echo "- **$filename** ($size MB) - [View](dumps/$filename) - Symbolized crash report" >> "$index_file"
                else
                    echo "- **$filename** ($size MB) - [Download](dumps/$filename) - Raw minidump" >> "$index_file"
                fi
            fi
        done
    fi
    
    cat >> "$index_file" << EOF

### 📊 Performance
EOF
    
    # Add performance files
    if [[ -d "$temp_dir/perf" ]]; then
        for file in "$temp_dir/perf"/*; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local size=$(get_file_size_mb "$file")
                echo "- **$filename** ($size MB) - [View](perf/$filename)" >> "$index_file"
            fi
        done
    fi
    
    cat >> "$index_file" << EOF

### 🧪 Tests
EOF
    
    # Add test files
    if [[ -d "$temp_dir/tests" ]]; then
        for file in "$temp_dir/tests"/*; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local size=$(get_file_size_mb "$file")
                echo "- **$filename** ($size MB) - [View](tests/$filename)" >> "$index_file"
            fi
        done
    fi
    
    cat >> "$index_file" << EOF

### ⚙️ Kernel
EOF
    
    # Add kernel files
    if [[ -d "$temp_dir/kernel" ]]; then
        for file in "$temp_dir/kernel"/*; do
            if [[ -f "$file" ]]; then
                local filename=$(basename "$file")
                local size=$(get_file_size_mb "$file")
                echo "- **$filename** ($size MB) - [View](kernel/$filename)" >> "$index_file"
            fi
        done
    fi
    
    cat >> "$index_file" << EOF

## Quick Start

### For Crash Analysis
1. Check the **dumps/** directory for symbolized crash reports (`.txt` files)
2. Review the **logs/** directory for relevant log information
3. Examine **performance metrics** for any performance issues

### For Performance Issues
1. Review **perf/metrics.json** for current performance data
2. Check **perf/guardrails_result.json** for statistical guardrail results
3. Compare with baseline data if available

### For Test Failures
1. Check **tests/** directory for test results and summaries
2. Review **logs/** for test execution logs
3. Examine **kernel configuration** for feature flags

## Configuration Details

- **Matrix Configuration**: $matrix_name
- **Configuration ID**: $config_id
- **Build Environment**: $(uname -s) $(uname -m)
- **Git Commit**: $(git rev-parse HEAD 2>/dev/null || echo "Unknown")
- **Branch**: $(git branch --show-current 2>/dev/null || echo "Unknown")

## Troubleshooting

### Missing Symbols
If crash reports show incomplete symbol information:
1. Ensure debug symbols are available
2. Check if the symbolizer is properly configured
3. Verify DWARF information is present

### Large Package Size
If the package exceeds size limits:
1. Review log verbosity settings
2. Check for excessive debug output
3. Consider implementing log rotation

### Symbolization Issues
If symbolization fails:
1. Verify minidump file integrity
2. Check symbolizer tool availability
3. Ensure debug symbols match the binary

## Support

For issues with this artifact package or the Polymera OS system:
1. Check the [Polymera OS documentation](https://github.com/polymera-os/docs)
2. Review [CI/CD troubleshooting guide](docs/ci/TROUBLESHOOTING.md)
3. Open an issue in the [Polymera OS repository](https://github.com/polymera-os/polymera-os)

---

*Generated by Polymera OS CI/CD Artifact Packaging System*
EOF
    
    log_success "Index.md generated: $index_file"
}

# Function to create tarball
create_tarball() {
    local config_id="$1"
    local temp_dir="$2"
    local output_dir="$3"
    
    local tarball_name="matrix_${config_id}.tar.gz"
    local tarball_path="$output_dir/$tarball_name"
    
    log_info "Creating tarball: $tarball_path"
    
    # Change to temp directory to ensure proper paths in tarball
    cd "$temp_dir"
    
    # Create tarball
    if tar -czf "$tarball_path" . 2>/dev/null; then
        log_success "Tarball created successfully: $tarball_path"
    else
        log_error "Failed to create tarball"
        return 1
    fi
    
    # Check tarball size
    local tarball_size_mb=$(get_file_size_mb "$tarball_path")
    log_info "Tarball size: ${tarball_size_mb}MB"
    
    if (( $(echo "$tarball_size_mb > $MAX_TARBALL_SIZE_MB" | bc -l) )); then
        log_warning "Tarball size (${tarball_size_mb}MB) exceeds limit (${MAX_TARBALL_SIZE_MB}MB)"
        log_warning "Consider reducing log verbosity or implementing log rotation"
    else
        log_success "Tarball size within limits"
    fi
    
    # Return to original directory
    cd "$PROJECT_ROOT"
    
    echo "$tarball_path"
}

# Function to validate artifacts
validate_artifacts() {
    local temp_dir="$1"
    local config_id="$2"
    
    log_info "Validating artifacts for configuration: $config_id"
    
    local validation_errors=()
    
    # Check if essential directories exist
    for dir in "logs" "dumps" "perf" "tests" "kernel"; do
        if [[ ! -d "$temp_dir/$dir" ]]; then
            validation_errors+=("Missing directory: $dir")
        fi
    done
    
    # Check if index.md exists
    if [[ ! -f "$temp_dir/index.md" ]]; then
        validation_errors+=("Missing index.md")
    fi
    
    # Check if any files were collected
    local total_files=$(find "$temp_dir" -type f | wc -l)
    if [[ "$total_files" -eq 1 ]]; then
        validation_errors+=("Only index.md found, no other artifacts collected")
    fi
    
    # Report validation results
    if [[ ${#validation_errors[@]} -eq 0 ]]; then
        log_success "Artifact validation passed"
        log_info "Total files collected: $total_files"
        return 0
    else
        log_error "Artifact validation failed:"
        for error in "${validation_errors[@]}"; do
            log_error "  - $error"
        done
        return 1
    fi
}

# Main execution
main() {
    log_info "Starting artifact packaging process..."
    
    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        log_error "Not in Polymera OS project root. Expected Cargo.toml at: $PROJECT_ROOT"
        exit 1
    fi
    
    # Get matrix name from environment or use config ID
    local matrix_name="${MATRIX_NAME:-$CONFIG_ID}"
    
    # Collect artifacts
    collect_artifacts "$CONFIG_ID" "$TEMP_DIR"
    
    # Generate index.md
    generate_index "$CONFIG_ID" "$TEMP_DIR" "$matrix_name"
    
    # Validate artifacts
    if ! validate_artifacts "$TEMP_DIR" "$CONFIG_ID"; then
        log_warning "Artifact validation failed, but continuing with packaging"
    fi
    
    # Create tarball
    local tarball_path
    if tarball_path=$(create_tarball "$CONFIG_ID" "$TEMP_DIR" "$OUTPUT_DIR"); then
        log_success "Artifact packaging completed successfully!"
        log_info "Output tarball: $tarball_path"
        log_info "Package size: $(get_file_size_mb "$tarball_path")MB"
        
        # Print summary
        echo
        echo "🎉 Artifact Package Summary"
        echo "=========================="
        echo "Configuration: $CONFIG_ID"
        echo "Matrix Name: $matrix_name"
        echo "Output: $tarball_path"
        echo "Size: $(get_file_size_mb "$tarball_path")MB"
        echo "Files: $(find "$TEMP_DIR" -type f | wc -l)"
        echo "Directories: $(find "$TEMP_DIR" -type d | wc -l)"
        echo
        
        exit 0
    else
        log_error "Failed to create artifact package"
        exit 1
    fi
}

# Run main function
main "$@"
