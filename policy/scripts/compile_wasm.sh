#!/bin/bash
# WASM compilation script for OPA/Rego policies

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Compiling Polymera OS Policies to WASM${NC}"
echo "========================================="

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
POLICY_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$POLICY_DIR/out"

# Compilation counters
COMPILATIONS_RUN=0
COMPILATIONS_PASSED=0
COMPILATIONS_FAILED=0

echo "Policy directory: $POLICY_DIR"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to compile a single policy to WASM
compile_policy() {
    local policy_file="$1"
    local policy_name="$(basename "$policy_file" .rego)"
    local entrypoint="$2"
    local output_file="$OUTPUT_DIR/${policy_name}.wasm"
    
    echo -e "${BLUE}Compiling Policy: $policy_name${NC}"
    echo "Source: $policy_file"
    echo "Entrypoint: $entrypoint"
    echo "Output: $output_file"
    
    COMPILATIONS_RUN=$((COMPILATIONS_RUN + 1))
    
    # Check if source file exists
    if [[ ! -f "$policy_file" ]]; then
        echo -e "${RED}✗ FAIL: Policy file not found${NC}"
        COMPILATIONS_FAILED=$((COMPILATIONS_FAILED + 1))
        return 1
    fi
    
    # Validate policy before compilation
    echo -n "  Validating policy syntax... "
    if opa fmt --diff "$policy_file" >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "    Syntax errors found in $policy_file"
        opa fmt --diff "$policy_file" 2>&1 | sed 's/^/    /'
        COMPILATIONS_FAILED=$((COMPILATIONS_FAILED + 1))
        return 1
    fi
    
    # Create temporary directory for compilation
    local temp_dir="/tmp/opa_build_$$_$(date +%s)"
    mkdir -p "$temp_dir"
    
    # Copy policy to temp directory
    cp "$policy_file" "$temp_dir/"
    
    # Compile to WASM
    echo -n "  Compiling to WASM... "
    local bundle_file="$temp_dir/bundle.tar.gz"
    
    if opa build -t wasm -e "$entrypoint" "$temp_dir/$(basename "$policy_file")" -o "$bundle_file" >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        
        # Extract WASM from bundle
        echo -n "  Extracting WASM binary... "
        cd "$temp_dir"
        if tar -xzf "$bundle_file" >/dev/null 2>&1; then
            if [[ -f "$temp_dir/policy.wasm" ]]; then
                cp "$temp_dir/policy.wasm" "$output_file"
                echo -e "${GREEN}✓ PASS${NC}"
                
                # Get file size
                local file_size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null || echo "unknown")
                echo "    WASM size: $file_size bytes"
                
                COMPILATIONS_PASSED=$((COMPILATIONS_PASSED + 1))
            else
                echo -e "${RED}✗ FAIL: policy.wasm not found in bundle${NC}"
                COMPILATIONS_FAILED=$((COMPILATIONS_FAILED + 1))
                cleanup_temp "$temp_dir"
                return 1
            fi
        else
            echo -e "${RED}✗ FAIL: Could not extract bundle${NC}"
            COMPILATIONS_FAILED=$((COMPILATIONS_FAILED + 1))
            cleanup_temp "$temp_dir"
            return 1
        fi
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "    Compilation errors:"
        opa build -t wasm -e "$entrypoint" "$temp_dir/$(basename "$policy_file")" -o "$bundle_file" 2>&1 | sed 's/^/    /'
        COMPILATIONS_FAILED=$((COMPILATIONS_FAILED + 1))
        cleanup_temp "$temp_dir"
        return 1
    fi
    
    # Cleanup
    cleanup_temp "$temp_dir"
    
    echo -e "${GREEN}✓ OVERALL: Compilation successful${NC}"
    echo ""
    return 0
}

# Function to compile rationale function
compile_rationale() {
    local policy_file="$1"
    local policy_name="$(basename "$policy_file" .rego)"
    local package_path="$2"
    local output_file="$OUTPUT_DIR/${policy_name}_rationale.wasm"
    
    echo -e "${BLUE}Compiling Rationale Function: $policy_name${NC}"
    echo "Source: $policy_file"
    echo "Package: $package_path"
    echo "Output: $output_file"
    
    COMPILATIONS_RUN=$((COMPILATIONS_RUN + 1))
    
    # Check if rationale function exists
    if ! grep -q "^rationale\s*:=" "$policy_file"; then
        echo -e "${YELLOW}⚠ SKIP: No rationale function found${NC}"
        echo ""
        return 0
    fi
    
    # Create temporary directory
    local temp_dir="/tmp/opa_rationale_$$_$(date +%s)"
    mkdir -p "$temp_dir"
    
    # Copy policy to temp directory
    cp "$policy_file" "$temp_dir/"
    
    # Compile rationale function
    echo -n "  Compiling rationale to WASM... "
    local bundle_file="$temp_dir/rationale_bundle.tar.gz"
    local entrypoint="${package_path}/rationale"
    
    if opa build -t wasm -e "$entrypoint" "$temp_dir/$(basename "$policy_file")" -o "$bundle_file" >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        
        # Extract WASM from bundle
        cd "$temp_dir"
        if tar -xzf "$bundle_file" >/dev/null 2>&1 && [[ -f "$temp_dir/policy.wasm" ]]; then
            cp "$temp_dir/policy.wasm" "$output_file"
            local file_size=$(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null || echo "unknown")
            echo "    Rationale WASM size: $file_size bytes"
            COMPILATIONS_PASSED=$((COMPILATIONS_PASSED + 1))
        else
            echo -e "${RED}✗ FAIL: Could not extract rationale WASM${NC}"
            COMPILATIONS_FAILED=$((COMPILATIONS_FAILED + 1))
            cleanup_temp "$temp_dir"
            return 1
        fi
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "    Rationale compilation errors:"
        opa build -t wasm -e "$entrypoint" "$temp_dir/$(basename "$policy_file")" -o "$bundle_file" 2>&1 | sed 's/^/    /'
        COMPILATIONS_FAILED=$((COMPILATIONS_FAILED + 1))
        cleanup_temp "$temp_dir"
        return 1
    fi
    
    # Cleanup
    cleanup_temp "$temp_dir"
    
    echo -e "${GREEN}✓ OVERALL: Rationale compilation successful${NC}"
    echo ""
    return 0
}

# Function to validate WASM output
validate_wasm() {
    local wasm_file="$1"
    local policy_name="$(basename "$wasm_file" .wasm)"
    
    echo -e "${BLUE}Validating WASM: $policy_name${NC}"
    echo "File: $wasm_file"
    
    # Check if file exists
    if [[ ! -f "$wasm_file" ]]; then
        echo -e "${RED}✗ FAIL: WASM file not found${NC}"
        return 1
    fi
    
    # Check file size
    local file_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null || echo "0")
    echo "  File size: $file_size bytes"
    
    if [[ $file_size -eq 0 ]]; then
        echo -e "${RED}✗ FAIL: WASM file is empty${NC}"
        return 1
    fi
    
    # Check file format (basic magic number check)
    echo -n "  WASM format validation... "
    if head -c 4 "$wasm_file" | xxd -p | grep -q "0061736d"; then
        echo -e "${GREEN}✓ PASS (WASM magic number found)${NC}"
    else
        echo -e "${RED}✗ FAIL: Invalid WASM format${NC}"
        return 1
    fi
    
    # Size optimization check
    echo -n "  Size optimization check... "
    if [[ $file_size -lt 1048576 ]]; then  # Less than 1MB
        echo -e "${GREEN}✓ PASS (size: $file_size bytes)${NC}"
    elif [[ $file_size -lt 5242880 ]]; then  # Less than 5MB
        echo -e "${YELLOW}⚠ WARNING: Large WASM file (size: $file_size bytes)${NC}"
    else
        echo -e "${RED}✗ FAIL: WASM file too large (size: $file_size bytes)${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✓ OVERALL: WASM validation passed${NC}"
    echo ""
    return 0
}

# Function to create deployment bundle
create_deployment_bundle() {
    echo -e "${BLUE}Creating Deployment Bundle${NC}"
    
    local bundle_dir="$OUTPUT_DIR/deployment"
    local bundle_file="$OUTPUT_DIR/polymera_policies.tar.gz"
    
    mkdir -p "$bundle_dir"
    
    # Copy WASM files
    echo -n "  Copying WASM files... "
    if cp "$OUTPUT_DIR"/*.wasm "$bundle_dir/" 2>/dev/null; then
        echo -e "${GREEN}✓ PASS${NC}"
    else
        echo -e "${RED}✗ FAIL: No WASM files to bundle${NC}"
        return 1
    fi
    
    # Create metadata
    echo -n "  Creating metadata... "
    cat > "$bundle_dir/metadata.json" << 'EOF'
{
  "version": "1.0.0",
  "build_time": "%BUILD_TIME%",
  "policies": [
    {
      "name": "wallet_spend_limits",
      "package": "polymera.wallet.spend_limits",
      "entrypoint": "allow",
      "description": "Wallet spending limits and controls",
      "wasm_file": "wallet_spend_limits.wasm",
      "rationale_file": "wallet_spend_limits_rationale.wasm"
    },
    {
      "name": "network_rate_limits",
      "package": "polymera.network.rate_limits", 
      "entrypoint": "allow",
      "description": "Network access rate limiting and controls",
      "wasm_file": "network_rate_limits.wasm",
      "rationale_file": "network_rate_limits_rationale.wasm"
    },
    {
      "name": "fs_access_scopes",
      "package": "polymera.filesystem.access_scopes",
      "entrypoint": "allow", 
      "description": "Filesystem access scoping and permissions",
      "wasm_file": "fs_access_scopes.wasm",
      "rationale_file": "fs_access_scopes_rationale.wasm"
    }
  ]
}
EOF
    
    # Replace build time
    local build_time=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    sed -i.bak "s/%BUILD_TIME%/$build_time/g" "$bundle_dir/metadata.json"
    rm -f "$bundle_dir/metadata.json.bak"
    
    echo -e "${GREEN}✓ PASS${NC}"
    
    # Create tarball
    echo -n "  Creating deployment bundle... "
    cd "$OUTPUT_DIR"
    if tar -czf "polymera_policies.tar.gz" -C deployment . >/dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        
        local bundle_size=$(stat -f%z "$bundle_file" 2>/dev/null || stat -c%s "$bundle_file" 2>/dev/null || echo "unknown")
        echo "    Bundle size: $bundle_size bytes"
        echo "    Bundle location: $bundle_file"
    else
        echo -e "${RED}✗ FAIL${NC}"
        return 1
    fi
    
    # Cleanup
    rm -rf "$bundle_dir"
    
    echo -e "${GREEN}✓ OVERALL: Deployment bundle created${NC}"
    echo ""
    return 0
}

# Cleanup function
cleanup_temp() {
    local temp_dir="$1"
    if [[ -d "$temp_dir" ]]; then
        rm -rf "$temp_dir"
    fi
}

# Main compilation execution
echo "Starting WASM compilation..."
echo ""

# Check OPA availability
if ! command -v opa >/dev/null 2>&1; then
    echo -e "${RED}❌ OPA CLI not found. Please install OPA first.${NC}"
    echo "Visit: https://www.openpolicyagent.org/docs/latest/#running-opa"
    exit 1
fi

echo "OPA version: $(opa version)"
echo ""

# Policy compilation definitions
declare -A POLICIES=(
    ["$POLICY_DIR/samples/wallet_spend_limits.rego"]="polymera/wallet/spend_limits/allow"
    ["$POLICY_DIR/samples/network_rate_limits.rego"]="polymera/network/rate_limits/allow"
    ["$POLICY_DIR/samples/fs_access_scopes.rego"]="polymera/filesystem/access_scopes/allow"
)

declare -A RATIONALE_PACKAGES=(
    ["$POLICY_DIR/samples/wallet_spend_limits.rego"]="polymera/wallet/spend_limits"
    ["$POLICY_DIR/samples/network_rate_limits.rego"]="polymera/network/rate_limits"
    ["$POLICY_DIR/samples/fs_access_scopes.rego"]="polymera/filesystem/access_scopes"
)

# Compile each policy
for policy_file in "${!POLICIES[@]}"; do
    if [[ -f "$policy_file" ]]; then
        entrypoint="${POLICIES[$policy_file]}"
        
        # Compile main policy
        compile_policy "$policy_file" "$entrypoint"
        
        # Compile rationale function
        package_path="${RATIONALE_PACKAGES[$policy_file]}"
        compile_rationale "$policy_file" "$package_path"
        
        echo "----------------------------------------"
    else
        echo -e "${RED}Policy file not found: $policy_file${NC}"
        COMPILATIONS_FAILED=$((COMPILATIONS_FAILED + 1))
    fi
done

# Validate all generated WASM files
echo -e "${BLUE}Validating Generated WASM Files${NC}"
echo "================================"

for wasm_file in "$OUTPUT_DIR"/*.wasm; do
    if [[ -f "$wasm_file" ]]; then
        validate_wasm "$wasm_file"
    fi
done

# Create deployment bundle
create_deployment_bundle

# Summary
echo ""
echo "======================================="
echo -e "${BLUE}Compilation Summary${NC}"
echo "======================================="
echo "Total compilations: $COMPILATIONS_RUN"
echo -e "Compilations passed: ${GREEN}$COMPILATIONS_PASSED${NC}"
echo -e "Compilations failed: ${RED}$COMPILATIONS_FAILED${NC}"

if [[ -d "$OUTPUT_DIR" ]]; then
    echo ""
    echo "Generated files:"
    ls -la "$OUTPUT_DIR"/*.wasm "$OUTPUT_DIR"/*.tar.gz 2>/dev/null | sed 's/^/  /' || echo "  No files generated"
fi

if [[ $COMPILATIONS_FAILED -eq 0 ]]; then
    echo ""
    echo -e "${GREEN}🎉 All policy compilations successful!${NC}"
    echo "WASM files are ready for deployment."
    exit 0
else
    echo ""
    echo -e "${RED}❌ Some policy compilations failed.${NC}"
    echo "Please review and fix the issues above."
    exit 1
fi
