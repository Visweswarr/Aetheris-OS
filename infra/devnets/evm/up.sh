#!/bin/bash

# Polymera EVM Devnet Startup Script
# One-command startup for local development environment

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEVNET_DIR="$SCRIPT_DIR"
COMPOSE_FILE="$DEVNET_DIR/docker-compose.yml"

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
        exit 1
    fi
    
    # Check if Docker is running
    if ! docker info &> /dev/null; then
        log_error "Docker is not running. Please start Docker first."
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Install faucet dependencies
install_faucet_deps() {
    log_info "Installing faucet dependencies..."
    
    cd "$DEVNET_DIR/faucet"
    
    if [ ! -d "node_modules" ]; then
        log_info "Installing npm dependencies..."
        npm install
        log_success "Faucet dependencies installed"
    else
        log_success "Faucet dependencies already installed"
    fi
    
    cd "$DEVNET_DIR"
}

# Start services
start_services() {
    log_info "Starting EVM devnet services..."
    
    # Use docker-compose if available, otherwise use docker compose
    if command -v docker-compose &> /dev/null; then
        COMPOSE_CMD="docker-compose"
    else
        COMPOSE_CMD="docker compose"
    fi
    
    # Start services in background
    $COMPOSE_CMD -f "$COMPOSE_FILE" up -d
    
    log_success "Services started successfully"
}

# Wait for services to be ready
wait_for_services() {
    log_info "Waiting for services to be ready..."
    
    # Wait for Anvil
    log_info "Waiting for Anvil (Ethereum node)..."
    local anvil_ready=false
    for i in {1..30}; do
        if curl -s http://localhost:8545 > /dev/null 2>&1; then
            anvil_ready=true
            break
        fi
        echo -n "."
        sleep 2
    done
    
    if [ "$anvil_ready" = true ]; then
        log_success "Anvil is ready"
    else
        log_warning "Anvil may not be fully ready yet"
    fi
    
    # Wait for Faucet
    log_info "Waiting for Faucet service..."
    local faucet_ready=false
    for i in {1..30}; do
        if curl -s http://localhost:3000/health > /dev/null 2>&1; then
            faucet_ready=true
            break
        fi
        echo -n "."
        sleep 2
    done
    
    if [ "$faucet_ready" = true ]; then
        log_success "Faucet is ready"
    else
        log_warning "Faucet may not be fully ready yet"
    fi
    
    # Wait for Blockscout
    log_info "Waiting for Blockscout (Block Explorer)..."
    local blockscout_ready=false
    for i in {1..60}; do
        if curl -s http://localhost:4000/api/health > /dev/null 2>&1; then
            blockscout_ready=true
            break
        fi
        echo -n "."
        sleep 5
    done
    
    if [ "$blockscout_ready" = true ]; then
        log_success "Blockscout is ready"
    else
        log_warning "Blockscout may not be fully ready yet (this is normal for first startup)"
    fi
}

# Display service information
show_service_info() {
    echo ""
    echo "=========================================="
    echo "  🚀 Polymera EVM Devnet is Running!"
    echo "=========================================="
    echo ""
    echo "📡 Services:"
    echo "  • Anvil (Ethereum Node): http://localhost:8545"
    echo "  • Faucet:                http://localhost:3000"
    echo "  • Block Explorer:        http://localhost:4000"
    echo ""
    echo "🔑 Pre-funded Accounts (Anvil):"
    echo "  • Account 0: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    echo "  • Account 1: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
    echo "  • Account 2: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
    echo ""
    echo "💰 Faucet Endpoints:"
    echo "  • Health Check:          http://localhost:3000/health"
    echo "  • Faucet Info:           http://localhost:3000/info"
    echo "  • Request Funds:         POST http://localhost:3000/fund"
    echo "  • Transaction History:   http://localhost:3000/transactions"
    echo ""
    echo "📊 Block Explorer:"
    echo "  • Main Page:             http://localhost:4000"
    echo "  • API Health:            http://localhost:4000/api/health"
    echo ""
    echo "🛠️  Development Commands:"
    echo "  • View logs:             docker-compose logs -f"
    echo "  • Stop services:         docker-compose down"
    echo "  • Restart services:      docker-compose restart"
    echo "  • Check status:          docker-compose ps"
    echo ""
    echo "🧪 Testing:"
    echo "  • Foundry test:          forge test --rpc-url http://localhost:8545"
    echo "  • Deploy contract:       forge create --rpc-url http://localhost:8545"
    echo ""
}

# Check service health
check_health() {
    log_info "Performing health checks..."
    
    local all_healthy=true
    
    # Check Anvil
    if curl -s http://localhost:8545 > /dev/null 2>&1; then
        log_success "✓ Anvil is healthy"
    else
        log_error "✗ Anvil is not responding"
        all_healthy=false
    fi
    
    # Check Faucet
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        log_success "✓ Faucet is healthy"
    else
        log_error "✗ Faucet is not responding"
        all_healthy=false
    fi
    
    # Check Blockscout (may take longer to start)
    if curl -s http://localhost:4000/api/health > /dev/null 2>&1; then
        log_success "✓ Blockscout is healthy"
    else
        log_warning "⚠ Blockscout is still starting up (this is normal)"
    fi
    
    if [ "$all_healthy" = true ]; then
        log_success "All critical services are healthy!"
    else
        log_warning "Some services may need more time to start up"
    fi
}

# Main function
main() {
    echo "🚀 Starting Polymera EVM Devnet..."
    echo ""
    
    # Check prerequisites
    check_prerequisites
    
    # Install faucet dependencies
    install_faucet_deps
    
    # Start services
    start_services
    
    # Wait for services
    wait_for_services
    
    # Check health
    check_health
    
    # Show service information
    show_service_info
    
    log_success "EVM devnet startup complete!"
    echo ""
    echo "💡 Tip: Use 'docker-compose logs -f' to monitor service logs"
    echo "💡 Tip: Use 'docker-compose down' to stop all services"
}

# Error handling
trap 'log_error "Script interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"
