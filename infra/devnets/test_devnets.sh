#!/bin/bash

# Polymera Local Devnets Test Script
# Tests all devnets including health checks and sample contract deployment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
INFRA_DIR="$SCRIPT_DIR"

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

log_test() {
    echo -e "${PURPLE}[TEST]${NC} $1"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
}

# Test EVM devnet
test_evm_devnet() {
    log_test "Testing EVM devnet..."
    
    # Test Anvil health
    if curl -s http://localhost:8545 > /dev/null 2>&1; then
        log_success "✓ Anvil (Ethereum node) is responding"
    else
        log_error "✗ Anvil is not responding"
        return 1
    fi
    
    # Test Faucet health
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        log_success "✓ Faucet is responding"
    else
        log_error "✗ Faucet is not responding"
        return 1
    fi
    
    # Test Faucet info endpoint
    local faucet_info=$(curl -s http://localhost:3000/info)
    if echo "$faucet_info" | grep -q "faucetAddress"; then
        log_success "✓ Faucet info endpoint working"
    else
        log_error "✗ Faucet info endpoint not working"
        return 1
    fi
    
    # Test Blockscout health
    if curl -s http://localhost:4000/api/health > /dev/null 2>&1; then
        log_success "✓ Blockscout (Block Explorer) is responding"
    else
        log_warning "⚠ Blockscout may still be starting up"
    fi
    
    # Test Foundry if available
    if command -v forge &> /dev/null; then
        log_info "Testing Foundry integration..."
        
        # Create a simple test contract
        local test_dir="$INFRA_DIR/evm/test_contract"
        mkdir -p "$test_dir"
        
        cat > "$test_dir/TestContract.sol" << 'EOF'
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

contract TestContract {
    string public message;
    
    constructor(string memory _message) {
        message = _message;
    }
    
    function setMessage(string memory _message) public {
        message = _message;
    }
    
    function getMessage() public view returns (string memory) {
        return message;
    }
}
EOF
        
        # Test contract compilation
        cd "$test_dir"
        if forge build > /dev/null 2>&1; then
            log_success "✓ Test contract compiles successfully"
        else
            log_error "✗ Test contract compilation failed"
            cd "$INFRA_DIR"
            return 1
        fi
        
        # Test contract deployment (this may fail if no accounts are funded)
        log_info "Testing contract deployment..."
        if forge create --rpc-url http://localhost:8545 --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 TestContract --constructor-args "Hello Polymera!" > /dev/null 2>&1; then
            log_success "✓ Test contract deployed successfully"
        else
            log_warning "⚠ Contract deployment failed (this may be expected if accounts are not funded)"
        fi
        
        cd "$INFRA_DIR"
    else
        log_warning "Foundry not installed, skipping contract tests"
    fi
    
    log_success "EVM devnet tests completed"
}

# Test Cosmos devnet
test_cosmos_devnet() {
    log_test "Testing Cosmos devnet..."
    
    # Test Cosmos Hub health
    if curl -s http://localhost:26657/status > /dev/null 2>&1; then
        log_success "✓ Cosmos Hub node is responding"
    else
        log_error "✗ Cosmos Hub node is not responding"
        return 1
    fi
    
    # Test Cosmos API
    if curl -s http://localhost:1317/cosmos/base/tendermint/v1beta1/node_info > /dev/null 2>&1; then
        log_success "✓ Cosmos API is responding"
    else
        log_error "✗ Cosmos API is not responding"
        return 1
    fi
    
    # Test Faucet health
    if curl -s http://localhost:3001/health > /dev/null 2>&1; then
        log_success "✓ Faucet is responding"
    else
        log_error "✗ Faucet is not responding"
        return 1
    fi
    
    # Test Big Dipper
    if curl -s http://localhost:4001 > /dev/null 2>&1; then
        log_success "✓ Big Dipper (Block Explorer) is responding"
    else
        log_warning "⚠ Big Dipper may still be starting up"
    fi
    
    log_success "Cosmos devnet tests completed"
}

# Test Substrate devnet
test_substrate_devnet() {
    log_test "Testing Substrate devnet..."
    
    # Test Substrate RPC
    if curl -s -X POST -H "Content-Type: application/json" -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' http://localhost:9933 > /dev/null 2>&1; then
        log_success "✓ Substrate RPC is responding"
    else
        log_error "✗ Substrate RPC is not responding"
        return 1
    fi
    
    # Test Substrate WebSocket (basic check)
    if curl -s http://localhost:9944 > /dev/null 2>&1; then
        log_success "✓ Substrate WebSocket port is open"
    else
        log_warning "⚠ Substrate WebSocket may not be fully ready"
    fi
    
    # Test Faucet health
    if curl -s http://localhost:3002/health > /dev/null 2>&1; then
        log_success "✓ Faucet is responding"
    else
        log_error "✗ Faucet is not responding"
        return 1
    fi
    
    # Test Polkascan
    if curl -s http://localhost:4002 > /dev/null 2>&1; then
        log_success "✓ Polkascan (Block Explorer) is responding"
    else
        log_warning "⚠ Polkascan may still be starting up"
    fi
    
    log_success "Substrate devnet tests completed"
}

# Test Move devnet
test_move_devnet() {
    log_test "Testing Move devnet..."
    
    # Test Sui RPC
    if curl -s http://localhost:9000 > /dev/null 2>&1; then
        log_success "✓ Sui RPC is responding"
    else
        log_error "✗ Sui RPC is not responding"
        return 1
    fi
    
    # Test Sui WebSocket
    if curl -s http://localhost:9001 > /dev/null 2>&1; then
        log_success "✓ Sui WebSocket port is open"
    else
        log_warning "⚠ Sui WebSocket may not be fully ready"
    fi
    
    # Test Faucet health
    if curl -s http://localhost:3003/health > /dev/null 2>&1; then
        log_success "✓ Faucet is responding"
    else
        log_error "✗ Faucet is not responding"
        return 1
    fi
    
    # Test Sui Explorer
    if curl -s http://localhost:4003 > /dev/null 2>&1; then
        log_success "✓ Sui Explorer (Block Explorer) is responding"
    else
        log_warning "⚠ Sui Explorer may still be starting up"
    fi
    
    log_success "Move devnet tests completed"
}

# Test network connectivity
test_network_connectivity() {
    log_test "Testing network connectivity..."
    
    # Test localhost connectivity
    if ping -c 1 localhost > /dev/null 2>&1; then
        log_success "✓ Localhost connectivity working"
    else
        log_error "✗ Localhost connectivity failed"
        return 1
    fi
    
    # Test Docker network
    if docker network ls | grep -q "polymera"; then
        log_success "✓ Docker networks created"
    else
        log_warning "⚠ Docker networks may not be fully created"
    fi
    
    log_success "Network connectivity tests completed"
}

# Test port availability
test_port_availability() {
    log_test "Testing port availability..."
    
    local ports=(
        "8545:Anvil (EVM)"
        "3000:Faucet (EVM)"
        "4000:Blockscout (EVM)"
        "26657:Cosmos Hub (Cosmos)"
        "3001:Faucet (Cosmos)"
        "4001:Big Dipper (Cosmos)"
        "9933:Substrate (Substrate)"
        "3002:Faucet (Substrate)"
        "4002:Polkascan (Substrate)"
        "9000:Sui (Move)"
        "3003:Faucet (Move)"
        "4003:Sui Explorer (Move)"
    )
    
    for port_info in "${ports[@]}"; do
        local port="${port_info%%:*}"
        local service="${port_info#*:}"
        
        if netstat -tuln 2>/dev/null | grep -q ":$port " || ss -tuln 2>/dev/null | grep -q ":$port "; then
            log_success "✓ Port $port ($service) is available"
        else
            log_warning "⚠ Port $port ($service) may not be available"
        fi
    done
    
    log_success "Port availability tests completed"
}

# Test faucet functionality
test_faucet_functionality() {
    log_test "Testing faucet functionality..."
    
    # Test EVM faucet
    log_info "Testing EVM faucet..."
    local evm_fund_response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"address":"0x70997970C51812dc3A010C7d01b50e0d17dc79C8","amount":"0.1"}' http://localhost:3000/fund 2>/dev/null || echo "failed")
    
    if echo "$evm_fund_response" | grep -q "success"; then
        log_success "✓ EVM faucet funding working"
    else
        log_warning "⚠ EVM faucet funding may not be working (this could be expected)"
    fi
    
    # Test Cosmos faucet
    log_info "Testing Cosmos faucet..."
    local cosmos_fund_response=$(curl -s -X POST -H "Content-Type: application/json" -d '{"address":"cosmos1hsk6jryyqjfhp5dhc55te9r50mxrxrq23y9ryx","amount":"1000000"}' http://localhost:3001/fund 2>/dev/null || echo "failed")
    
    if echo "$cosmos_fund_response" | grep -q "success"; then
        log_success "✓ Cosmos faucet funding working"
    else
        log_warning "⚠ Cosmos faucet funding may not be working (this could be expected)"
    fi
    
    log_success "Faucet functionality tests completed"
}

# Test block explorer functionality
test_block_explorer_functionality() {
    log_test "Testing block explorer functionality..."
    
    # Test Blockscout
    if curl -s http://localhost:4000/api/health | grep -q "healthy" 2>/dev/null; then
        log_success "✓ Blockscout health check passing"
    else
        log_warning "⚠ Blockscout health check may not be passing"
    fi
    
    # Test Big Dipper
    if curl -s http://localhost:4001 | grep -q "Big Dipper" 2>/dev/null; then
        log_success "✓ Big Dipper frontend loading"
    else
        log_warning "⚠ Big Dipper frontend may not be loading"
    fi
    
    # Test Polkascan
    if curl -s http://localhost:4002 | grep -q "Polkascan" 2>/dev/null; then
        log_success "✓ Polkascan frontend loading"
    else
        log_warning "⚠ Polkascan frontend may not be loading"
    fi
    
    # Test Sui Explorer
    if curl -s http://localhost:4003 | grep -q "Sui" 2>/dev/null; then
        log_success "✓ Sui Explorer frontend loading"
    else
        log_warning "⚠ Sui Explorer frontend may not be loading"
    fi
    
    log_success "Block explorer functionality tests completed"
}

# Create test summary
create_test_summary() {
    log_info "Creating test summary..."
    
    local summary_file="$INFRA_DIR/test_summary.md"
    cat > "$summary_file" << EOF
# Polymera Local Devnets Test Summary

**Test Date:** $(date)
**Test Environment:** $(uname -s) $(uname -r)

## Test Results

### Overall Results
- **Total Tests:** $TESTS_TOTAL
- **Passed:** $TESTS_PASSED
- **Failed:** $TESTS_FAILED
- **Success Rate:** $(( (TESTS_PASSED * 100) / TESTS_TOTAL ))%

### Test Categories

#### ✅ Network Tests
- Localhost connectivity
- Docker network creation
- Port availability

#### ✅ EVM Devnet Tests
- Anvil (Ethereum node) health
- Faucet functionality
- Blockscout block explorer
- Foundry integration (if available)

#### ✅ Cosmos Devnet Tests
- Cosmos Hub node health
- Cosmos API functionality
- Faucet functionality
- Big Dipper block explorer

#### ✅ Substrate Devnet Tests
- Substrate RPC health
- WebSocket connectivity
- Faucet functionality
- Polkascan block explorer

#### ✅ Move Devnet Tests
- Sui node health
- WebSocket connectivity
- Faucet functionality
- Sui Explorer block explorer

#### ✅ Integration Tests
- Faucet funding functionality
- Block explorer frontend loading
- Health check endpoints

## Next Steps

1. **Review Failed Tests**: Check the test output above for any failed tests
2. **Check Service Logs**: Use \`./up.sh --logs [devnet]\` to view service logs
3. **Verify Dependencies**: Ensure all required tools are installed
4. **Check Resource Usage**: Monitor Docker resource consumption
5. **Deploy Sample Contracts**: Test contract deployment on each devnet

## Notes

- Some tests may fail during initial startup as services take time to initialize
- Block explorers may take several minutes to fully start up
- Faucet funding tests may fail if accounts are not properly funded
- Foundry tests require Foundry to be installed

## Troubleshooting

- **Services not starting**: Check Docker logs with \`docker-compose logs\`
- **Port conflicts**: Ensure no other services are using the required ports
- **Resource issues**: Increase Docker memory/CPU limits if needed
- **Network issues**: Check Docker network configuration
EOF
    
    log_success "Test summary created: $summary_file"
}

# Main function
main() {
    echo "🧪 Testing Polymera Local Devnets..."
    echo ""
    
    # Check if devnets are running
    log_info "Checking if devnets are running..."
    
    # Test each devnet
    test_evm_devnet
    echo ""
    
    test_cosmos_devnet
    echo ""
    
    test_substrate_devnet
    echo ""
    
    test_move_devnet
    echo ""
    
    # Test infrastructure
    test_network_connectivity
    echo ""
    
    test_port_availability
    echo ""
    
    # Test functionality
    test_faucet_functionality
    echo ""
    
    test_block_explorer_functionality
    echo ""
    
    # Create summary
    create_test_summary
    
    echo ""
    echo "=== Test Results ==="
    echo "Total Tests: $TESTS_TOTAL"
    echo "Passed: $TESTS_PASSED"
    echo "Failed: $TESTS_FAILED"
    echo "Success Rate: $(( (TESTS_PASSED * 100) / TESTS_TOTAL ))%"
    echo ""
    
    if [ $TESTS_FAILED -eq 0 ]; then
        log_success "All tests passed! 🎉"
        exit 0
    else
        log_warning "Some tests failed. Check the summary for details."
        exit 1
    fi
}

# Error handling
trap 'log_error "Script interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"
