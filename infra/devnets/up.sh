#!/bin/bash

# Polymera Local Devnets Startup Script
# One-command startup for local development environments

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$SCRIPT_DIR"

# Available devnets
AVAILABLE_DEVNETS=("evm" "cosmos" "substrate" "move" "all")

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

log_devnet() {
    echo -e "${PURPLE}[$1]${NC} $2"
}

# Show usage
show_usage() {
    echo "🚀 Polymera Local Devnets"
    echo ""
    echo "Usage: $0 [devnet] [options]"
    echo ""
    echo "Available devnets:"
    echo "  evm       - Ethereum Virtual Machine (Anvil + Faucet + Blockscout)"
    echo "  cosmos    - Cosmos Hub (Cosmos SDK + Faucet + Big Dipper)"
    echo "  substrate - Substrate (Polkadot + Faucet + Polkascan)"
    echo "  move      - Move (Sui + Faucet + Sui Explorer)"
    echo "  all       - Start all devnets"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -s, --status   Show status of running devnets"
    echo "  -d, --down     Stop all devnets"
    echo "  -l, --logs     Show logs for specified devnet"
    echo ""
    echo "Examples:"
    echo "  $0 evm                    # Start EVM devnet"
    echo "  $0 cosmos                 # Start Cosmos devnet"
    echo "  $0 all                    # Start all devnets"
    echo "  $0 --status               # Show status"
    echo "  $0 --down                 # Stop all devnets"
    echo "  $0 --logs evm             # Show EVM devnet logs"
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
    
    # Check curl
    if ! command -v curl &> /dev/null; then
        log_error "curl is not installed. Please install curl first."
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Get compose command
get_compose_cmd() {
    if command -v docker-compose &> /dev/null; then
        echo "docker-compose"
    else
        echo "docker compose"
    fi
}

# Start specific devnet
start_devnet() {
    local devnet="$1"
    local devnet_dir="$INFRA_DIR/$devnet"
    
    if [ ! -d "$devnet_dir" ]; then
        log_error "Devnet '$devnet' not found in $devnet_dir"
        return 1
    fi
    
    log_devnet "$devnet" "Starting devnet..."
    
    cd "$devnet_dir"
    
    # Check if devnet-specific startup script exists
    if [ -f "up.sh" ]; then
        log_devnet "$devnet" "Using devnet-specific startup script"
        chmod +x up.sh
        ./up.sh
    else
        log_devnet "$devnet" "Using docker-compose startup"
        
        # Install faucet dependencies if needed
        if [ -d "faucet" ] && [ -f "faucet/package.json" ]; then
            log_devnet "$devnet" "Installing faucet dependencies..."
            cd faucet
            if [ ! -d "node_modules" ]; then
                npm install
            fi
            cd ..
        fi
        
        # Start services
        local compose_cmd=$(get_compose_cmd)
        $compose_cmd up -d
        
        log_devnet "$devnet" "Services started. Waiting for readiness..."
        
        # Wait for services to be ready
        sleep 30
        
        # Check health
        check_devnet_health "$devnet"
    fi
    
    cd "$INFRA_DIR"
    log_success "Devnet '$devnet' started successfully"
}

# Start all devnets
start_all_devnets() {
    log_info "Starting all devnets..."
    
    for devnet in "${AVAILABLE_DEVNETS[@]}"; do
        if [ "$devnet" != "all" ]; then
            start_devnet "$devnet"
            echo ""
        fi
    done
    
    log_success "All devnets started successfully"
}

# Check devnet health
check_devnet_health() {
    local devnet="$1"
    local devnet_dir="$INFRA_DIR/$devnet"
    
    log_devnet "$devnet" "Checking health..."
    
    cd "$devnet_dir"
    
    # Get compose command
    local compose_cmd=$(get_compose_cmd)
    
    # Check if services are running
    if $compose_cmd ps --services --filter "status=running" | grep -q .; then
        log_devnet "$devnet" "Services are running"
        
        # Check specific endpoints based on devnet type
        case "$devnet" in
            "evm")
                check_evm_health
                ;;
            "cosmos")
                check_cosmos_health
                ;;
            "substrate")
                check_substrate_health
                ;;
            "move")
                check_move_health
                ;;
        esac
    else
        log_warning "Devnet '$devnet' services may not be fully started"
    fi
    
    cd "$INFRA_DIR"
}

# Check EVM devnet health
check_evm_health() {
    # Check Anvil
    if curl -s http://localhost:8545 > /dev/null 2>&1; then
        log_devnet "evm" "✓ Anvil (Ethereum node) is healthy"
    else
        log_devnet "evm" "✗ Anvil is not responding"
    fi
    
    # Check Faucet
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        log_devnet "evm" "✓ Faucet is healthy"
    else
        log_devnet "evm" "✗ Faucet is not responding"
    fi
    
    # Check Blockscout
    if curl -s http://localhost:4000/api/health > /dev/null 2>&1; then
        log_devnet "evm" "✓ Blockscout (Block Explorer) is healthy"
    else
        log_devnet "evm" "⚠ Blockscout may still be starting up"
    fi
}

# Check Cosmos devnet health
check_cosmos_health() {
    # Check Cosmos Hub
    if curl -s http://localhost:26657/status > /dev/null 2>&1; then
        log_devnet "cosmos" "✓ Cosmos Hub node is healthy"
    else
        log_devnet "cosmos" "✗ Cosmos Hub node is not responding"
    fi
    
    # Check Faucet
    if curl -s http://localhost:3001/health > /dev/null 2>&1; then
        log_devnet "cosmos" "✓ Faucet is healthy"
    else
        log_devnet "cosmos" "✗ Faucet is not responding"
    fi
    
    # Check Big Dipper
    if curl -s http://localhost:4001 > /dev/null 2>&1; then
        log_devnet "cosmos" "✓ Big Dipper (Block Explorer) is healthy"
    else
        log_devnet "cosmos" "⚠ Big Dipper may still be starting up"
    fi
}

# Check Substrate devnet health
check_substrate_health() {
    # Check Substrate node
    if curl -s http://localhost:9933 > /dev/null 2>&1; then
        log_devnet "substrate" "✓ Substrate node is healthy"
    else
        log_devnet "substrate" "✗ Substrate node is not responding"
    fi
    
    # Check Faucet
    if curl -s http://localhost:3002/health > /dev/null 2>&1; then
        log_devnet "substrate" "✓ Faucet is healthy"
    else
        log_devnet "substrate" "✗ Faucet is not responding"
    fi
    
    # Check Polkascan
    if curl -s http://localhost:4002 > /dev/null 2>&1; then
        log_devnet "substrate" "✓ Polkascan (Block Explorer) is healthy"
    else
        log_devnet "substrate" "⚠ Polkascan may still be starting up"
    fi
}

# Check Move devnet health
check_move_health() {
    # Check Sui node
    if curl -s http://localhost:9000 > /dev/null 2>&1; then
        log_devnet "move" "✓ Sui node is healthy"
    else
        log_devnet "move" "✗ Sui node is not responding"
    fi
    
    # Check Faucet
    if curl -s http://localhost:3003/health > /dev/null 2>&1; then
        log_devnet "move" "✓ Faucet is healthy"
    else
        log_devnet "move" "✗ Faucet is not responding"
    fi
    
    # Check Sui Explorer
    if curl -s http://localhost:4003 > /dev/null 2>&1; then
        log_devnet "move" "✓ Sui Explorer (Block Explorer) is healthy"
    else
        log_devnet "move" "⚠ Sui Explorer may still be starting up"
    fi
}

# Show devnet status
show_status() {
    log_info "Checking devnet status..."
    
    for devnet in "${AVAILABLE_DEVNETS[@]}"; do
        if [ "$devnet" != "all" ]; then
            local devnet_dir="$INFRA_DIR/$devnet"
            
            if [ -d "$devnet_dir" ]; then
                cd "$devnet_dir"
                
                local compose_cmd=$(get_compose_cmd)
                if [ -f "docker-compose.yml" ]; then
                    echo ""
                    log_devnet "$devnet" "Status:"
                    $compose_cmd ps
                fi
                
                cd "$INFRA_DIR"
            fi
        fi
    done
}

# Stop all devnets
stop_all_devnets() {
    log_info "Stopping all devnets..."
    
    for devnet in "${AVAILABLE_DEVNETS[@]}"; do
        if [ "$devnet" != "all" ]; then
            local devnet_dir="$INFRA_DIR/$devnet"
            
            if [ -d "$devnet_dir" ] && [ -f "$devnet_dir/docker-compose.yml" ]; then
                log_devnet "$devnet" "Stopping devnet..."
                cd "$devnet_dir"
                
                local compose_cmd=$(get_compose_cmd)
                $compose_cmd down
                
                cd "$INFRA_DIR"
                log_success "Devnet '$devnet' stopped"
            fi
        fi
    done
    
    log_success "All devnets stopped"
}

# Show devnet logs
show_logs() {
    local devnet="$1"
    local devnet_dir="$INFRA_DIR/$devnet"
    
    if [ ! -d "$devnet_dir" ]; then
        log_error "Devnet '$devnet' not found"
        return 1
    fi
    
    if [ ! -f "$devnet_dir/docker-compose.yml" ]; then
        log_error "Devnet '$devnet' does not have docker-compose configuration"
        return 1
    fi
    
    log_devnet "$devnet" "Showing logs..."
    
    cd "$devnet_dir"
    local compose_cmd=$(get_compose_cmd)
    $compose_cmd logs -f
}

# Main function
main() {
    # Parse command line arguments
    local devnet=""
    local action="start"
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -s|--status)
                action="status"
                shift
                ;;
            -d|--down)
                action="down"
                shift
                ;;
            -l|--logs)
                action="logs"
                if [[ $# -gt 1 ]]; then
                    devnet="$2"
                    shift 2
                else
                    log_error "Please specify a devnet for logs"
                    exit 1
                fi
                ;;
            -*)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
            *)
                if [ -z "$devnet" ]; then
                    devnet="$1"
                else
                    log_error "Multiple devnets specified. Please specify only one."
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Validate devnet if specified
    if [ -n "$devnet" ] && [[ ! " ${AVAILABLE_DEVNETS[@]} " =~ " ${devnet} " ]]; then
        log_error "Invalid devnet: $devnet"
        echo "Available devnets: ${AVAILABLE_DEVNETS[*]}"
        exit 1
    fi
    
    # Execute action
    case "$action" in
        "start")
            if [ -z "$devnet" ]; then
                show_usage
                exit 1
            fi
            
            check_prerequisites
            
            if [ "$devnet" = "all" ]; then
                start_all_devnets
            else
                start_devnet "$devnet"
            fi
            ;;
        "status")
            show_status
            ;;
        "down")
            stop_all_devnets
            ;;
        "logs")
            if [ -z "$devnet" ]; then
                log_error "Please specify a devnet for logs"
                exit 1
            fi
            show_logs "$devnet"
            ;;
    esac
}

# Error handling
trap 'log_error "Script interrupted by user"; exit 1' INT TERM

# Run main function
main "$@"
