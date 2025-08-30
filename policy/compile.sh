#!/bin/bash

# Policy Engine Compilation Script
# Compiles Rego policies to WebAssembly using Open Policy Agent (OPA)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
POLICY_DIR="$SCRIPT_DIR"
OUTPUT_DIR="$POLICY_DIR/wasm"
BUILD_DIR="$POLICY_DIR/build"
TEMP_DIR="$POLICY_DIR/temp"

# Policy files to compile
POLICY_FILES=(
    "examples.rego"
)

# OPA configuration
OPA_VERSION="0.58.0"
OPA_BINARY=""
OPA_DOWNLOAD_URL="https://openpolicyagent.org/downloads/v${OPA_VERSION}/opa_linux_amd64_static"

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

# Utility functions
check_command() {
    if command -v "$1" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

download_opa() {
    log_info "Downloading OPA version $OPA_VERSION..."
    
    if [[ ! -d "$TEMP_DIR" ]]; then
        mkdir -p "$TEMP_DIR"
    fi
    
    local opa_path="$TEMP_DIR/opa"
    
    if [[ ! -f "$opa_path" ]]; then
        log_info "Downloading from $OPA_DOWNLOAD_URL..."
        if curl -L -o "$opa_path" "$OPA_DOWNLOAD_URL"; then
            chmod +x "$opa_path"
            log_success "OPA downloaded successfully"
        else
            log_error "Failed to download OPA"
            return 1
        fi
    else
        log_info "OPA already downloaded"
    fi
    
    OPA_BINARY="$opa_path"
}

setup_directories() {
    log_info "Setting up directories..."
    
    # Create output directories
    mkdir -p "$OUTPUT_DIR"
    mkdir -p "$BUILD_DIR"
    mkdir -p "$TEMP_DIR"
    
    log_success "Directories created"
}

compile_policy() {
    local policy_file="$1"
    local policy_name=$(basename "$policy_file" .rego)
    
    log_info "Compiling policy: $policy_file"
    
    # Create a temporary bundle configuration
    local bundle_config="$TEMP_DIR/${policy_name}_bundle.yaml"
    cat > "$bundle_config" << EOF
services:
  polymera:
    url: http://localhost:8181
bundles:
  polymera:
    service: polymera
    resource: ${policy_name}.tar.gz
    persist: true
    polling:
      min_delay_seconds: 10
      max_delay_seconds: 30
EOF
    
    # Create bundle directory
    local bundle_dir="$TEMP_DIR/${policy_name}_bundle"
    mkdir -p "$bundle_dir"
    
    # Copy policy file to bundle
    cp "$policy_file" "$bundle_dir/"
    
    # Create bundle
    if "$OPA_BINARY" build --bundle "$bundle_dir" --output "$OUTPUT_DIR/${policy_name}.tar.gz"; then
        log_success "Policy compiled: ${policy_name}.tar.gz"
        
        # Extract WASM from bundle
        if tar -xzf "$OUTPUT_DIR/${policy_name}.tar.gz" -C "$TEMP_DIR" policy.wasm 2>/dev/null; then
            mv "$TEMP_DIR/policy.wasm" "$OUTPUT_DIR/${policy_name}.wasm"
            log_success "WASM extracted: ${policy_name}.wasm"
        else
            log_warning "WASM extraction failed, bundle may not contain WASM"
        fi
        
        # Create metadata file
        local metadata_file="$OUTPUT_DIR/${policy_name}_metadata.json"
        cat > "$metadata_file" << EOF
{
  "policy_name": "$policy_name",
  "source_file": "$policy_file",
  "compiled_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "opa_version": "$OPA_VERSION",
  "bundle_file": "${policy_name}.tar.gz",
  "wasm_file": "${policy_name}.wasm",
  "policy_rules": [
    "allow",
    "deny",
    "file_access",
    "network_access",
    "process_execution",
    "container_policy",
    "api_access",
    "data_access",
    "system_config",
    "package_install",
    "user_management",
    "backup_recovery",
    "compliance_check"
  ]
}
EOF
        log_success "Metadata created: ${policy_name}_metadata.json"
        
    else
        log_error "Failed to compile policy: $policy_file"
        return 1
    fi
}

compile_all_policies() {
    log_info "Compiling all policies..."
    
    local success_count=0
    local total_count=${#POLICY_FILES[@]}
    
    for policy_file in "${POLICY_FILES[@]}"; do
        if [[ -f "$POLICY_DIR/$policy_file" ]]; then
            if compile_policy "$policy_file"; then
                ((success_count++))
            fi
        else
            log_warning "Policy file not found: $policy_file"
        fi
    done
    
    log_info "Compilation complete: $success_count/$total_count policies compiled successfully"
    
    if [[ $success_count -eq $total_count ]]; then
        log_success "All policies compiled successfully!"
        return 0
    else
        log_warning "Some policies failed to compile"
        return 1
    fi
}

create_test_inputs() {
    log_info "Creating test inputs for policies..."
    
    local test_inputs_dir="$BUILD_DIR/test_inputs"
    mkdir -p "$test_inputs_dir"
    
    # File access test input
    cat > "$test_inputs_dir/file_access.json" << EOF
{
  "user": {
    "role": "developer",
    "id": "user123",
    "permissions": ["read", "write"]
  },
  "file": {
    "path": "/home/user/document.txt",
    "extension": ".txt",
    "size": 1024
  }
}
EOF
    
    # Network access test input
    cat > "$test_inputs_dir/network_access.json" << EOF
{
  "network": {
    "protocol": "HTTPS",
    "port": 443,
    "domain": "api.example.com",
    "certificate": {
      "valid": true,
      "issuer": "Let's Encrypt"
    }
  }
}
EOF
    
    # Process execution test input
    cat > "$test_inputs_dir/process_execution.json" << EOF
{
  "process": {
    "binary": "/usr/bin/git",
    "command": "git clone https://github.com/example/repo.git",
    "memory_limit": 268435456,
    "cpu_limit": 50
  }
}
EOF
    
    # Container policy test input
    cat > "$test_inputs_dir/container_policy.json" << EOF
{
  "container": {
    "image": "alpine:latest",
    "resources": {
      "memory": 1073741824,
      "cpu": 2,
      "disk": 5368709120
    },
    "security": {
      "readonly_root": true,
      "privileged": false,
      "host_network": false
    }
  }
}
EOF
    
    # API access test input
    cat > "$test_inputs_dir/api_access.json" << EOF
{
  "api": {
    "user": {
      "authenticated": true,
      "id": "user456",
      "permissions": {
        "/api/v1/users": "read"
      }
    },
    "rate_limit": {
      "current": 50
    },
    "headers": {
      "Authorization": "Bearer token123",
      "User_Agent": "PolymeraOS/1.0"
    },
    "client_ip": "192.168.1.10",
    "endpoint": "/api/v1/users"
  }
}
EOF
    
    # Data access test input
    cat > "$test_inputs_dir/data_access.json" << EOF
{
  "api": {
    "user": {
      "clearance": "confidential"
    }
  },
  "data": {
    "classification": "internal",
    "audit": {
      "enabled": true
    },
    "encryption": {
      "enabled": true
    }
  }
}
EOF
    
    log_success "Test inputs created in $test_inputs_dir"
}

create_validation_script() {
    log_info "Creating policy validation script..."
    
    local validation_script="$BUILD_DIR/validate_policies.sh"
    cat > "$validation_script" << 'EOF'
#!/bin/bash

# Policy Validation Script
# Validates compiled policies against test inputs

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
POLICY_DIR="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$POLICY_DIR/wasm"
TEST_INPUTS_DIR="$SCRIPT_DIR/test_inputs"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if OPA is available
if ! command -v opa &> /dev/null; then
    log_error "OPA not found. Please install Open Policy Agent."
    exit 1
fi

# Validate each policy
for wasm_file in "$WASM_DIR"/*.wasm; do
    if [[ -f "$wasm_file" ]]; then
        policy_name=$(basename "$wasm_file" .wasm)
        log_success "Validating policy: $policy_name"
        
        # Find corresponding test input
        test_input="$TEST_INPUTS_DIR/${policy_name}.json"
        if [[ -f "$test_input" ]]; then
            # Run policy evaluation
            if opa eval --data "$wasm_file" --input "$test_input" "data.polymera.policy.$policy_name"; then
                log_success "Policy $policy_name validated successfully"
            else
                log_error "Policy $policy_name validation failed"
            fi
        else
            log_warning "No test input found for $policy_name"
        fi
    fi
done

log_success "Policy validation complete"
EOF
    
    chmod +x "$validation_script"
    log_success "Validation script created: $validation_script"
}

create_build_summary() {
    log_info "Creating build summary..."
    
    local summary_file="$BUILD_DIR/build_summary.md"
    cat > "$summary_file" << EOF
# Policy Engine Build Summary

**Build Date:** $(date)
**OPA Version:** $OPA_VERSION
**Total Policies:** ${#POLICY_FILES[@]}

## Compiled Policies

EOF
    
    for wasm_file in "$OUTPUT_DIR"/*.wasm; do
        if [[ -f "$wasm_file" ]]; then
            policy_name=$(basename "$wasm_file" .wasm)
            local size=$(stat -c%s "$wasm_file" 2>/dev/null || stat -f%z "$wasm_file" 2>/dev/null || echo "unknown")
            echo "- **$policy_name**: $size bytes" >> "$summary_file"
        fi
    done
    
    cat >> "$summary_file" << EOF

## Output Files

- **WASM Files**: \`$OUTPUT_DIR/*.wasm\`
- **Bundle Files**: \`$OUTPUT_DIR/*.tar.gz\`
- **Metadata Files**: \`$OUTPUT_DIR/*_metadata.json\`
- **Test Inputs**: \`$BUILD_DIR/test_inputs/\`
- **Validation Script**: \`$BUILD_DIR/validate_policies.sh\`

## Usage

1. **Load WASM policy in Rust**:
   \`\`\`rust
   use polymera_policy::PolicyEngine;
   
   let engine = PolicyEngine::new("examples.wasm")?;
   let result = engine.evaluate(input_data)?;
   \`\`\`

2. **Validate policies**:
   \`\`\`bash
   ./build/validate_policies.sh
   \`\`\`

## Policy Rules

The compiled policies include the following rules:
- \`allow\` - Default allow rule
- \`deny\` - Default deny rule with rationale
- \`file_access\` - File access control
- \`network_access\` - Network connection control
- \`process_execution\` - Process execution control
- \`container_policy\` - Container security policy
- \`api_access\` - API endpoint access control
- \`data_access\` - Data access control
- \`system_config\` - System configuration control
- \`package_install\` - Package installation control
- \`user_management\` - User account management
- \`backup_recovery\` - Backup and recovery operations
- \`compliance_check\` - Regulatory compliance

## Security Features

- **Default Deny**: All operations denied unless explicitly allowed
- **Input Validation**: Comprehensive input validation and sanitization
- **Audit Logging**: All policy decisions logged with rationale
- **Sandboxed Execution**: WASM-based policy evaluation
- **Resource Limits**: Memory and CPU usage limits enforced
EOF
    
    log_success "Build summary created: $summary_file"
}

cleanup() {
    log_info "Cleaning up temporary files..."
    
    if [[ -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
    
    log_success "Cleanup complete"
}

main() {
    echo "=========================================="
    echo "  Policy Engine Compilation"
    echo "=========================================="
    echo ""
    
    # Setup
    setup_directories
    
    # Check for OPA or download it
    if check_command "opa"; then
        OPA_BINARY="opa"
        log_success "Using system OPA"
    else
        log_warning "System OPA not found, downloading..."
        download_opa
    fi
    
    # Compile policies
    if compile_all_policies; then
        # Create additional build artifacts
        create_test_inputs
        create_validation_script
        create_build_summary
        
        log_success "Policy compilation completed successfully!"
        echo ""
        echo "Output files:"
        echo "  WASM policies: $OUTPUT_DIR/*.wasm"
        echo "  Test inputs: $BUILD_DIR/test_inputs/"
        echo "  Validation script: $BUILD_DIR/validate_policies.sh"
        echo "  Build summary: $BUILD_DIR/build_summary.md"
        echo ""
        
        # Cleanup
        cleanup
        
        exit 0
    else
        log_error "Policy compilation failed"
        exit 1
    fi
}

# Error handling
trap 'log_error "Script interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"
