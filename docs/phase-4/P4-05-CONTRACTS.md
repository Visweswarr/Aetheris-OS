# P4-05: Web3 Expansion Layer - Smart Contracts

## Overview

The Web3 Expansion Layer (P4-05) provides a comprehensive smart contract system for Aetheris OS, enabling users to deploy, execute, and manage WebAssembly (WASM) smart contracts with deterministic execution, resource allocation, and multi-chain transaction relay capabilities.

## Architecture

### Core Components

1. **Rust WASM Contract Runtime** (`services/contracts/`)
   - Sandboxed WASM execution environment
   - Deterministic gas metering
   - Resource allocation management
   - Integration with DAO and DID systems

2. **Multi-chain Transaction Relay** (`services/chain/`)
   - Cross-chain transaction support
   - Ethereum, Polkadot, Solana, and EVM-compatible chains
   - PQC signature support
   - Transaction status tracking

3. **Go CLI Interface** (`go/tooling/netctl/`)
   - Contract deployment and management
   - Resource allocation commands
   - Transaction relay operations
   - Integration with existing netctl tool

4. **TypeScript SDK** (`tooling/ts/contracts_bridge.ts`)
   - Client-side contract interaction
   - Event-driven architecture
   - Multi-chain support
   - Type-safe API

5. **Python Validator** (`tooling/python/contracts_validator.py`)
   - Deterministic execution validation
   - WASM module validation
   - Gas metering verification
   - ZK proof validation

6. **Solidity Reference Contracts** (`contracts/`)
   - HelloWorld contract
   - TokenContract (ERC20-like)
   - ResourceManager
   - DAOGovernance

## Features

### Smart Contract Execution

- **WASM Runtime**: Sandboxed execution environment using WASMtime
- **Deterministic Execution**: Reproducible results across different environments
- **Gas Metering**: Precise resource consumption tracking
- **Event System**: Structured event emission and handling
- **State Management**: Persistent contract state storage

### Resource Management

- **CPU Allocation**: Millisecond-per-second limits
- **Memory Limits**: Byte-level memory constraints
- **Storage Quotas**: Persistent storage allocation
- **Network Bandwidth**: Bytes-per-second network limits
- **DAO Integration**: Resource allocation tied to governance votes

### Multi-chain Support

- **Ethereum**: EVM-compatible execution
- **Polkadot**: Substrate-based chains
- **Solana**: High-performance blockchain
- **EVM Chains**: Avalanche, Polygon, Arbitrum, Optimism
- **PQC Signatures**: Post-quantum cryptography support

### Security Features

- **Sandboxing**: Isolated execution environment
- **Capability-based Access**: Fine-grained permissions
- **ZK Proofs**: Zero-knowledge proof generation and verification
- **Audit Logging**: Comprehensive execution tracking
- **Input Validation**: Strict parameter validation

## Usage

### Contract Deployment

#### Go CLI

```bash
# Deploy a WASM contract
./netctl contract deploy --wasm hello.wasm --gas-limit 1000000

# Deploy with metadata
./netctl contract deploy --wasm hello.wasm --metadata metadata.json --gas-limit 1000000
```

#### TypeScript SDK

```typescript
import { ContractsBridge, ContractUtils } from './contracts_bridge';

const bridge = new ContractsBridge({
  endpoint: 'http://localhost:8080',
  enablePQC: true,
  enableZKProofs: true
});

await bridge.initialize();

const metadata = ContractUtils.createDefaultMetadata(
  'HelloWorld',
  'Alice',
  'A simple hello world contract'
);

const wasmBytes = new Uint8Array(/* WASM module bytes */);

const result = await bridge.deployContract({
  wasm: wasmBytes,
  metadata,
  gasLimit: 1000000,
  deployer: '0x1234567890123456789012345678901234567890',
  value: 0
});

console.log('Contract deployed:', result.contractId);
```

#### Python Validator

```python
import asyncio
from contracts_validator import ContractsValidator, ContractMetadata

async def validate_deployment():
    validator = ContractsValidator()
    
    metadata = ContractMetadata(
        id="contract_123",
        name="HelloWorld",
        version="1.0.0",
        author="Alice",
        description="A simple hello world contract",
        wasm_hash="abc123",
        created_at=int(time.time()),
        updated_at=int(time.time())
    )
    
    wasm_bytes = b'\x00asm\x01\x00\x00\x00' + b'\x00' * 100
    
    result = await validator.validate_contract_deployment(wasm_bytes, metadata)
    print(f"Validation result: {result.status.value}")

asyncio.run(validate_deployment())
```

### Contract Execution

#### Go CLI

```bash
# Call a contract method
./netctl contract call --contract-id contract123 --method hello --args '{"name": "World"}' --gas-limit 100000

# Query contract state
./netctl contract query --contract-id contract123 --method getGreeting
```

#### TypeScript SDK

```typescript
// Call a contract method
const result = await bridge.callContract({
  contractId: 'contract123',
  method: 'hello',
  args: ContractUtils.stringToUint8Array('{"name": "World"}'),
  gasLimit: 100000,
  caller: '0x1234567890123456789012345678901234567890',
  value: 0
});

console.log('Execution result:', result.output);

// Query contract state
const queryResult = await bridge.queryContract(
  'contract123',
  'getGreeting'
);

console.log('Query result:', queryResult.output);
```

### Resource Allocation

#### Go CLI

```bash
# Allocate resources to a contract
./netctl contract allocate --contract-id contract123 --cpu 1000 --memory 67108864 --storage 1073741824 --network 1048576
```

#### TypeScript SDK

```typescript
await bridge.allocateResources('contract123', {
  cpuLimit: 1000,
  memoryLimit: 67108864,
  storageLimit: 1073741824,
  networkLimit: 1048576
});
```

### Multi-chain Transactions

#### Go CLI

```bash
# Submit a transaction to Ethereum
./netctl chain submit --chain ethereum --to 0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6 --data 0x123456 --gas-limit 21000
```

#### TypeScript SDK

```typescript
const transactionId = await bridge.submitTransaction({
  id: ContractUtils.generateTransactionId(),
  chainType: 'ethereum',
  to: '0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6',
  data: new Uint8Array([0x12, 0x34, 0x56]),
  value: 1000000000000000000, // 1 ETH
  gasLimit: 21000,
  gasPrice: 20000000000, // 20 gwei
  createdAt: Date.now()
});

console.log('Transaction submitted:', transactionId);
```

## Configuration

### Contract Service Configuration

```rust
use polymera_contracts::ContractConfig;

let config = ContractConfig {
    max_gas_per_execution: 1_000_000,
    max_memory_bytes: 64 * 1024 * 1024, // 64 MB
    max_execution_time_ms: 30_000, // 30 seconds
    max_concurrent_executions: 100,
    enable_zk_proofs: true,
    gas_config: GasConfig::default(),
    dao_config: dao::DAOConfig::default(),
    chain_config: chain::ChainConfig::default(),
};
```

### Chain Service Configuration

```rust
use polymera_chain::ChainServiceConfig;

let config = ChainServiceConfig {
    default_gas_price: 20_000_000_000, // 20 gwei
    default_gas_limit: 21000,
    transaction_timeout_ms: 30000,
    max_retry_attempts: 3,
    enable_pqc_signatures: true,
    chain_configs: vec![
        ChainConfig {
            chain_type: ChainType::Ethereum,
            rpc_url: "https://mainnet.infura.io/v3/your-key".to_string(),
            chain_id: 1,
            gas_price: 20_000_000_000,
            gas_limit: 21000,
            timeout_ms: 30000,
            retry_count: 3,
            enable_pqc: false,
        },
    ],
};
```

## API Reference

### ContractsBridge (TypeScript)

#### Methods

- `initialize()`: Initialize the bridge connection
- `deployContract(request)`: Deploy a WASM contract
- `callContract(request)`: Execute a contract method
- `queryContract(contractId, method, args)`: Query contract state
- `listContracts()`: List all deployed contracts
- `getContractStatus(contractId)`: Get contract status
- `allocateResources(contractId, allocation)`: Allocate resources
- `submitTransaction(request)`: Submit a transaction
- `getTransactionStatus(transactionId)`: Get transaction status
- `getRelayStats()`: Get relay statistics
- `getChainStats(chainType)`: Get chain statistics
- `checkChainConnectivity(chainType)`: Check chain connectivity
- `getSupportedChains()`: Get supported chains

#### Events

- `connected`: Bridge connected
- `disconnected`: Bridge disconnected
- `error`: Error occurred
- `contractDeployed`: Contract deployed
- `contractCalled`: Contract called
- `resourcesAllocated`: Resources allocated
- `transactionSubmitted`: Transaction submitted

### ContractUtils (TypeScript)

#### Methods

- `generateContractId()`: Generate unique contract ID
- `generateTransactionId()`: Generate unique transaction ID
- `stringToUint8Array(str)`: Convert string to Uint8Array
- `uint8ArrayToString(data)`: Convert Uint8Array to string
- `objectToUint8Array(obj)`: Convert object to Uint8Array
- `uint8ArrayToObject(data)`: Convert Uint8Array to object
- `estimateGas(contractSize, methodComplexity, dataSize)`: Estimate gas
- `validateMetadata(metadata)`: Validate contract metadata
- `createDefaultMetadata(name, author, description)`: Create default metadata

### ContractsValidator (Python)

#### Methods

- `validate_contract_deployment(wasm_bytes, metadata)`: Validate deployment
- `validate_contract_execution(request, result)`: Validate execution
- `validate_zk_proof(proof, public_inputs)`: Validate ZK proof
- `get_validation_stats()`: Get validation statistics
- `clear_validation_history()`: Clear validation history

## Security Considerations

### WASM Security

- **Sandboxing**: All WASM modules run in isolated environments
- **Import Restrictions**: Limited system call access
- **Memory Limits**: Strict memory allocation constraints
- **Execution Timeouts**: Prevent infinite loops and DoS attacks

### Cryptographic Security

- **PQC Support**: Post-quantum cryptography for future-proofing
- **Hybrid Signatures**: Support for both PQC and legacy algorithms
- **ZK Proofs**: Zero-knowledge proof generation and verification
- **Secure Randomness**: Cryptographically secure random number generation

### Access Control

- **Capability-based**: Fine-grained permission system
- **DAO Integration**: Governance-controlled resource allocation
- **DID Binding**: Identity-based access control
- **Audit Logging**: Comprehensive access tracking

## Performance

### Benchmarks

- **Contract Deployment**: < 200ms for typical contracts
- **Method Execution**: < 100ms for simple operations
- **Gas Estimation**: < 50ms for standard operations
- **Multi-chain Transactions**: < 5s for confirmation

### Optimization

- **WASM Caching**: Compiled modules cached for reuse
- **Parallel Execution**: Concurrent contract execution
- **Resource Pooling**: Shared resource allocation
- **Connection Pooling**: Efficient network connections

## Testing

### Unit Tests

```bash
# Rust services
cd services/contracts
cargo test

cd ../chain
cargo test

# Go CLI
cd go/tooling/netctl
go test

# TypeScript SDK
cd tooling/ts
npm test

# Python validator
cd tooling/python
python -m pytest

# Solidity contracts
cd contracts
forge test
```

### Integration Tests

```bash
# Run full integration test suite
./scripts/run-integration-tests.sh
```

### Performance Tests

```bash
# Run performance benchmarks
cd services/contracts
cargo bench

cd ../chain
cargo bench
```

## Deployment

### Development Environment

```bash
# Start contract service
cd services/contracts
cargo run

# Start chain service
cd services/chain
cargo run

# Start netctl CLI
cd go/tooling/netctl
go run .
```

### Production Deployment

```bash
# Build release binaries
cd services/contracts
cargo build --release

cd services/chain
cargo build --release

cd go/tooling/netctl
go build -o netctl .

# Deploy services
./scripts/deploy.sh
```

## Monitoring

### Metrics

- **Contract Deployments**: Number and success rate
- **Execution Performance**: Gas usage and execution time
- **Resource Utilization**: CPU, memory, storage, network
- **Transaction Success**: Multi-chain transaction success rates
- **Error Rates**: Failure rates and error types

### Logging

- **Structured Logs**: JSON-formatted log entries
- **Audit Trails**: Complete execution history
- **Error Tracking**: Detailed error information
- **Performance Metrics**: Execution timing and resource usage

## Troubleshooting

### Common Issues

1. **WASM Validation Failures**
   - Check WASM module format
   - Verify import restrictions
   - Ensure proper magic numbers

2. **Gas Limit Exceeded**
   - Increase gas limit
   - Optimize contract code
   - Check resource allocation

3. **Multi-chain Connection Issues**
   - Verify RPC endpoints
   - Check network connectivity
   - Validate chain configurations

4. **Resource Allocation Failures**
   - Check DAO permissions
   - Verify resource availability
   - Ensure proper authorization

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
export CONTRACT_DEBUG=true

# Run with verbose output
./netctl contract deploy --wasm hello.wasm --verbose
```

## Contributing

### Development Setup

1. Clone the repository
2. Install dependencies
3. Run tests
4. Make changes
5. Submit pull request

### Code Style

- **Rust**: Use `cargo fmt` and `cargo clippy`
- **Go**: Use `go fmt` and `go vet`
- **TypeScript**: Use Prettier and ESLint
- **Python**: Use Black and Flake8
- **Solidity**: Use Solhint and Slither

### Testing Requirements

- All new code must have unit tests
- Integration tests for new features
- Performance benchmarks for critical paths
- Security audits for smart contracts

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:

- **Documentation**: [docs.aetheris.os](https://docs.aetheris.os)
- **Issues**: [GitHub Issues](https://github.com/aetheris-os/issues)
- **Discussions**: [GitHub Discussions](https://github.com/aetheris-os/discussions)
- **Discord**: [Aetheris OS Discord](https://discord.gg/aetheris-os)

## Changelog

### Version 1.0.0

- Initial release of Web3 Expansion Layer
- WASM contract runtime
- Multi-chain transaction relay
- Go CLI integration
- TypeScript SDK
- Python validator
- Solidity reference contracts
- Comprehensive CI/CD pipeline
- Full documentation
