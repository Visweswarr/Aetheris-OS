# Polymera Local Devnets

A comprehensive local development environment for multiple blockchain ecosystems, providing one-command startup, faucets, seed keys, and block explorers.

## 🚀 Overview

The Polymera Local Devnets provide a complete development environment for:

- **EVM**: Ethereum Virtual Machine with Anvil, Faucet, and Blockscout
- **Cosmos**: Cosmos Hub with Cosmos SDK, Faucet, and Big Dipper
- **Substrate**: Polkadot ecosystem with Substrate, Faucet, and Polkascan
- **Move**: Sui blockchain with Move language, Faucet, and Sui Explorer

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   EVM Devnet   │    │  Cosmos Devnet  │    │ Substrate Dev  │
│                 │    │                 │    │                 │
│  • Anvil       │    │  • Cosmos Hub   │    │  • Substrate    │
│  • Faucet      │    │  • Faucet       │    │  • Faucet       │
│  • Blockscout  │    │  • Big Dipper   │    │  • Polkascan    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Move Devnet   │    │   Management    │    │   Testing      │
│                 │    │                 │    │                 │
│  • Sui Node     │    │  • One-command │    │  • Health       │
│  • Faucet      │    │  • Health       │    │  • Integration  │
│  • Sui Explorer│    │  • Monitoring   │    │  • Contract     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📁 Directory Structure

```
infra/devnets/
├── evm/                    # EVM devnet
│   ├── docker-compose.yml # Anvil + Faucet + Blockscout
│   ├── faucet/            # EVM faucet service
│   │   ├── package.json   # Node.js dependencies
│   │   └── src/           # Faucet source code
│   └── up.sh              # EVM-specific startup
├── cosmos/                 # Cosmos devnet
│   ├── docker-compose.yml # Cosmos Hub + Faucet + Big Dipper
│   ├── faucet/            # Cosmos faucet service
│   └── config/            # Cosmos configuration
├── substrate/              # Substrate devnet
│   ├── docker-compose.yml # Substrate + Faucet + Polkascan
│   ├── faucet/            # Substrate faucet service
│   └── config/            # Substrate configuration
├── move/                   # Move devnet
│   ├── docker-compose.yml # Sui + Faucet + Sui Explorer
│   ├── faucet/            # Move faucet service
│   └── config/            # Sui configuration
├── up.sh                   # Main startup script
├── test_devnets.sh         # Comprehensive test suite
└── README.md               # This file
```

## 🚀 Quick Start

### Prerequisites

- **Docker**: 20.10+ with Docker Compose
- **Node.js**: 18+ with npm
- **curl**: For health checks
- **Foundry**: For EVM contract testing (optional)

### One-Command Startup

```bash
# Start all devnets
./up.sh all

# Start specific devnet
./up.sh evm
./up.sh cosmos
./up.sh substrate
./up.sh move

# Show status
./up.sh --status

# Stop all devnets
./up.sh --down

# View logs
./up.sh --logs evm
```

## 🔧 Devnet Details

### EVM Devnet

**Services:**
- **Anvil**: Local Ethereum node (port 8545)
- **Faucet**: ETH/USDC/DAI faucet (port 3000)
- **Blockscout**: Block explorer (port 4000)

**Features:**
- Pre-funded accounts with 10,000 ETH each
- 2-second block time
- Gas price: 1 Gwei
- Chain ID: 31337

**Usage:**
```bash
# Start EVM devnet
./up.sh evm

# Test with Foundry
forge test --rpc-url http://localhost:8545
forge create --rpc-url http://localhost:8545 --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 TestContract
```

### Cosmos Devnet

**Services:**
- **Cosmos Hub**: Tendermint node (port 26657)
- **Faucet**: ATOM faucet (port 3001)
- **Big Dipper**: Block explorer (port 4001)

**Features:**
- Cosmos SDK v0.50+
- Tendermint consensus
- ATOM as native token
- REST API on port 1317

**Usage:**
```bash
# Start Cosmos devnet
./up.sh cosmos

# Query node status
curl http://localhost:26657/status

# Query API
curl http://localhost:1317/cosmos/base/tendermint/v1beta1/node_info
```

### Substrate Devnet

**Services:**
- **Substrate**: Polkadot node (port 9933)
- **Faucet**: UNIT faucet (port 3002)
- **Polkascan**: Block explorer (port 4002)

**Features:**
- Substrate framework
- WebSocket RPC on port 9944
- P2P on port 30333
- Archive pruning mode

**Usage:**
```bash
# Start Substrate devnet
./up.sh substrate

# Query RPC
curl -X POST -H "Content-Type: application/json" \
  -d '{"id":1,"jsonrpc":"2.0","method":"system_health"}' \
  http://localhost:9933
```

### Move Devnet

**Services:**
- **Sui**: Move-based blockchain (port 9000)
- **Faucet**: SUI faucet (port 3003)
- **Sui Explorer**: Block explorer (port 4003)

**Features:**
- Sui Move language
- WebSocket on port 9001
- Metrics on port 9184
- Admin interface on port 1337

**Usage:**
```bash
# Start Move devnet
./up.sh move

# Query Sui RPC
curl http://localhost:9000

# Check metrics
curl http://localhost:9184/metrics
```

## 💰 Faucet Usage

### EVM Faucet

```bash
# Request funds
curl -X POST http://localhost:3000/fund \
  -H "Content-Type: application/json" \
  -d '{
    "address": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
    "amount": "1.0",
    "token": "ETH"
  }'

# Check faucet info
curl http://localhost:3000/info

# View transaction history
curl http://localhost:3000/transactions
```

### Cosmos Faucet

```bash
# Request funds
curl -X POST http://localhost:3001/fund \
  -H "Content-Type: application/json" \
  -d '{
    "address": "cosmos1hsk6jryyqjfhp5dhc55te9r50mxrxrq23y9ryx",
    "amount": "1000000"
  }'
```

### Substrate Faucet

```bash
# Request funds
curl -X POST http://localhost:3002/fund \
  -H "Content-Type: application/json" \
  -d '{
    "address": "5GrwvaEF5zXb26Fz9rcQpDWS57CTERHdNeNkSdnqYbgQ",
    "amount": "1000000000000000000"
  }'
```

### Move Faucet

```bash
# Request funds
curl -X POST http://localhost:3003/fund \
  -H "Content-Type: application/json" \
  -d '{
    "address": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    "amount": "1000000000"
  }'
```

## 🧪 Testing

### Run All Tests

```bash
# Make test script executable
chmod +x test_devnets.sh

# Run comprehensive test suite
./test_devnets.sh
```

### Test Categories

1. **Health Checks**: Service availability and responsiveness
2. **Network Tests**: Connectivity and port availability
3. **Faucet Tests**: Funding functionality verification
4. **Block Explorer Tests**: Frontend loading and API health
5. **Integration Tests**: End-to-end functionality verification

### Foundry Integration

The EVM devnet includes Foundry integration for smart contract development:

```bash
# Test contracts
forge test --rpc-url http://localhost:8545

# Deploy contracts
forge create --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  TestContract \
  --constructor-args "Hello Polymera!"

# Verify deployment
forge verify-contract --rpc-url http://localhost:8545 \
  <CONTRACT_ADDRESS> \
  TestContract
```

## 🔍 Monitoring

### Health Checks

```bash
# Check all devnet status
./up.sh --status

# View specific devnet logs
./up.sh --logs evm
./up.sh --logs cosmos
./up.sh --logs substrate
./up.sh --logs move
```

### Service Endpoints

| Service | Port | Health Check |
|---------|------|--------------|
| Anvil (EVM) | 8545 | `curl http://localhost:8545` |
| EVM Faucet | 3000 | `curl http://localhost:3000/health` |
| Blockscout | 4000 | `curl http://localhost:4000/api/health` |
| Cosmos Hub | 26657 | `curl http://localhost:26657/status` |
| Cosmos Faucet | 3001 | `curl http://localhost:3001/health` |
| Big Dipper | 4001 | `curl http://localhost:4001` |
| Substrate | 9933 | `curl -X POST -d '{"method":"system_health"}' http://localhost:9933` |
| Substrate Faucet | 3002 | `curl http://localhost:3002/health` |
| Polkascan | 4002 | `curl http://localhost:4002` |
| Sui | 9000 | `curl http://localhost:9000` |
| Move Faucet | 3003 | `curl http://localhost:3003/health` |
| Sui Explorer | 4003 | `curl http://localhost:4003` |

## 🛠️ Development

### Adding New Devnets

1. Create devnet directory: `mkdir infra/devnets/newdevnet`
2. Add `docker-compose.yml` with services
3. Create faucet service if needed
4. Add health check functions to `up.sh`
5. Update test script with new tests

### Customizing Services

Each devnet can be customized by modifying:

- **Environment variables** in `docker-compose.yml`
- **Configuration files** in `config/` directories
- **Faucet logic** in `faucet/src/` directories
- **Startup scripts** for devnet-specific logic

### Port Configuration

To avoid port conflicts, each devnet uses different port ranges:

- **EVM**: 3000, 4000, 8545
- **Cosmos**: 3001, 4001, 26657
- **Substrate**: 3002, 4002, 9933
- **Move**: 3003, 4003, 9000

## 🚨 Troubleshooting

### Common Issues

1. **Port Conflicts**
   ```bash
   # Check what's using a port
   lsof -i :8545
   netstat -tuln | grep 8545
   ```

2. **Docker Resource Issues**
   ```bash
   # Check Docker resource usage
   docker stats
   
   # Increase Docker memory/CPU limits
   # Docker Desktop → Settings → Resources
   ```

3. **Service Startup Failures**
   ```bash
   # Check service logs
   ./up.sh --logs evm
   
   # Restart specific service
   docker-compose restart anvil
   ```

4. **Network Issues**
   ```bash
   # Check Docker networks
   docker network ls
   docker network inspect polymera-evm-devnet
   
   # Recreate networks
   docker-compose down
   docker network prune
   docker-compose up -d
   ```

### Debug Mode

```bash
# Enable verbose logging
export COMPOSE_HTTP_TIMEOUT=300
export DOCKER_CLIENT_TIMEOUT=300

# Start with debug output
docker-compose --verbose up -d
```

## 📚 Additional Resources

### Documentation

- [Docker Compose Reference](https://docs.docker.com/compose/)
- [Anvil Documentation](https://book.getfoundry.sh/anvil/)
- [Cosmos SDK Documentation](https://docs.cosmos.network/)
- [Substrate Documentation](https://docs.substrate.io/)
- [Sui Documentation](https://docs.sui.io/)

### Community

- [Polymera OS Discussions](https://github.com/polymera-os/polymera-os/discussions)
- [GitHub Issues](https://github.com/polymera-os/polymera-os/issues)
- [Email Support](mailto:team@polymera-os.org)

## 🎯 Roadmap

### Upcoming Features

- **Multi-chain Testing**: Cross-chain interaction testing
- **Advanced Monitoring**: Prometheus metrics and Grafana dashboards
- **CI/CD Integration**: Automated devnet testing in GitHub Actions
- **Performance Benchmarks**: Load testing and performance metrics
- **Custom Token Support**: ERC-20, CW-20, and custom token faucets

### Long-term Goals

- **Distributed Devnets**: Multi-node cluster support
- **Cloud Deployment**: AWS/GCP/Azure deployment options
- **Mobile Support**: Mobile app for devnet management
- **Plugin System**: Extensible devnet architecture
- **Community Templates**: User-contributed devnet configurations

---

**Happy Developing! 🚀**

The Polymera Local Devnets provide everything you need for local blockchain development and testing.
