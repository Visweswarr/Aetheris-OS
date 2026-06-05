#!/bin/bash
# P4 Contracts Smoke Test
# Tests the complete Web3 contract deployment and interaction flow

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${CYAN}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to cleanup on exit
cleanup() {
    print_info "Cleaning up temporary files..."
    rm -f seeds/wallet.json
    rm -f artifacts/contract_greeting.txt
    print_info "Cleanup complete"
}

# Set up cleanup trap
trap cleanup EXIT

# Main smoke test function
main() {
    echo -e "${GREEN}🚀 P4 Contracts Smoke Test${NC}"
    echo -e "${GREEN}==========================${NC}"
    echo ""
    
    # Check prerequisites
    print_status "Checking prerequisites..."
    
    if ! command_exists netctl; then
        print_error "netctl not found. Please build it first:"
        echo "  cd go/tooling/netctl && go build -o netctl ."
        exit 1
    fi
    
    if [ ! -f "contracts/HelloWorld.sol" ]; then
        print_error "HelloWorld.sol contract not found at contracts/HelloWorld.sol"
        echo "Please ensure the contract file exists."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
    
    # Create necessary directories
    print_status "Setting up test environment..."
    mkdir -p seeds
    mkdir -p artifacts
    
    print_success "Test environment ready"
    
    # Step 1: Create wallet
    print_status "Step 1: Creating wallet with secp256k1 algorithm..."
    
    if netctl wallet create --algo secp256k1 --out seeds/wallet.json; then
        print_success "Wallet created successfully"
    else
        print_error "Failed to create wallet"
        exit 1
    fi
    
    # Verify wallet file exists
    if [ ! -f "seeds/wallet.json" ]; then
        print_error "Wallet file not created"
        exit 1
    fi
    
    print_info "Wallet file: seeds/wallet.json"
    
    # Step 2: Deploy contract
    print_status "Step 2: Deploying HelloWorld contract..."
    
    if netctl contract deploy contracts/HelloWorld.sol --from seeds/wallet.json --label hello; then
        print_success "Contract deployed successfully with label 'hello'"
    else
        print_error "Failed to deploy contract"
        exit 1
    fi
    
    # Step 3: Call contract method
    print_status "Step 3: Calling setGreeting method with 'Aetheris'..."
    
    if netctl contract call hello setGreeting "Aetheris" --from seeds/wallet.json; then
        print_success "setGreeting method called successfully"
    else
        print_error "Failed to call setGreeting method"
        exit 1
    fi
    
    # Step 4: Query contract state
    print_status "Step 4: Querying greeting value..."
    
    if netctl contract query hello greeting | tee artifacts/contract_greeting.txt; then
        print_success "Greeting query executed successfully"
    else
        print_error "Failed to query greeting value"
        exit 1
    fi
    
    # Verify query output file
    if [ ! -f "artifacts/contract_greeting.txt" ]; then
        print_error "Query output file not created"
        exit 1
    fi
    
    # Step 5: Validate results
    print_status "Step 5: Validating contract interaction results..."
    
    # Read the greeting value from the output file
    GREETING_VALUE=$(cat artifacts/contract_greeting.txt | grep -o '"Aetheris"' || echo "")
    
    if [ "$GREETING_VALUE" = '"Aetheris"' ]; then
        print_success "Contract interaction validation passed"
        echo -e "${GREEN}PASS: Contract greeting value is 'Aetheris' as expected${NC}"
        echo ""
        print_success "🎉 P4 Contracts Smoke Test PASSED"
        echo ""
        print_info "Summary:"
        echo "  ✅ Wallet created with secp256k1 algorithm"
        echo "  ✅ HelloWorld contract deployed successfully"
        echo "  ✅ setGreeting method called with 'Aetheris'"
        echo "  ✅ Greeting value queried and verified"
        echo "  ✅ Contract state persistence confirmed"
        echo ""
        print_info "Artifacts generated:"
        echo "  📄 seeds/wallet.json (wallet file)"
        echo "  📄 artifacts/contract_greeting.txt (query output)"
        echo ""
        exit 0
    else
        print_error "Contract interaction validation failed"
        echo -e "${RED}FAIL: Expected greeting value 'Aetheris', but got: $GREETING_VALUE${NC}"
        echo ""
        print_info "Debug information:"
        echo "  Query output file contents:"
        cat artifacts/contract_greeting.txt
        echo ""
        print_error "❌ P4 Contracts Smoke Test FAILED"
        echo ""
        exit 1
    fi
}

# Run main function
main "$@"
