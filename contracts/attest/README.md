# Polymera OS Attestation Registry

A comprehensive, multi-chain attestation registry system for issuing, managing, and verifying digital attestations with support for both EVM (Solidity) and Cosmos (CosmWasm) blockchains.

## Overview

The Attestation Registry provides a decentralized way to issue, verify, and manage digital attestations. It supports:

- **Multi-chain deployment**: EVM (Solidity) and Cosmos (CosmWasm) implementations
- **Flexible schemas**: Custom attestation schemas with field definitions
- **Issuer management**: Registration, authorization, and management of attestation issuers
- **Attestation lifecycle**: Issue, update, revoke, and expiry management
- **Rich metadata**: Tags, URIs, and extensible metadata support
- **Event streaming**: Real-time attestation events for indexers
- **Comprehensive API**: gRPC/HTTP API with validation and search capabilities

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   EVM Chain     │    │  Cosmos Chain   │    │   API Service   │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │Solidity     │ │    │ │CosmWasm     │ │    │ │gRPC/HTTP    │ │
│ │Contract     │ │    │ │Contract     │ │    │ │API          │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   Indexer       │
                    │                 │
                    │ • Event parsing │
                    │ • Data storage  │
                    │ • Search API    │
                    └─────────────────┘
```

## Features

### Core Functionality
- **Attestation Management**: Issue, update, revoke, and expire attestations
- **Schema System**: Define custom attestation schemas with field types
- **Issuer Control**: Register and authorize issuers for specific schemas
- **Access Control**: Role-based permissions and authorization
- **Metadata Support**: Rich tagging, URI references, and extensible data

### Security Features
- **Signature Verification**: Cryptographic verification of attestations
- **Revocation Support**: Ability to revoke compromised attestations
- **Expiry Management**: Time-based attestation expiration
- **Access Control**: Granular permissions for issuers and admins

### Performance Features
- **Efficient Indexing**: Multi-dimensional indexing for fast queries
- **Pagination Support**: Scalable querying with pagination
- **Search Capabilities**: Advanced search with filters and facets
- **Event Streaming**: Real-time event streaming for indexers

## Smart Contracts

### EVM (Solidity)

Located in `contracts/attest/evm/`

**Key Features:**
- OpenZeppelin integration for security
- Gas-optimized storage patterns
- Comprehensive event emission
- Access control with modifiers
- Reentrancy protection

**Main Contract:** `AttestationRegistry.sol`

**Dependencies:**
```solidity
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
```

### CosmWasm (Rust)

Located in `contracts/attest/cosmos/`

**Key Features:**
- Rust-based implementation
- CosmWasm standard compliance
- Efficient storage with indexed maps
- Comprehensive validation
- Error handling with custom error types

**Main Contract:** `lib.rs`

**Dependencies:**
```toml
cosmwasm-std = "1.4"
cw-storage-plus = "1.2"
cw20 = "0.20"
thiserror = "1.0"
chrono = "0.4"
```

## API Service

Located in `services/attest/`

**Protocol:** gRPC with HTTP/REST mapping

**Key Services:**
- `AttestationRegistryService`: Core attestation operations
- Event streaming for real-time updates
- Comprehensive validation and error handling
- Search and filtering capabilities

**Message Types:**
- Attestation management (issue, revoke, update)
- Schema operations (create, deprecate)
- Issuer management (register, authorize, deactivate)
- Configuration and statistics
- Search and query operations

## Data Models

### Attestation
```rust
struct Attestation {
    id: u64,                    // Unique identifier
    issuer: String,             // Issuer address
    subject: String,            // Subject being attested
    schema_id: String,          // Schema reference
    data: Binary,               // Attestation data
    issued_at: Timestamp,       // Creation timestamp
    expires_at: Option<Timestamp>, // Expiration (optional)
    revoked: bool,              // Revocation status
    uri: String,                // Metadata URI
    tags: Vec<String>,          // Categorization tags
    version: u32,               // Version number
}
```

### Schema
```rust
struct Schema {
    id: String,                 // Unique identifier
    name: String,               // Human-readable name
    description: String,         // Schema description
    fields: Vec<String>,        // Field names
    field_types: Vec<String>,   // Field type definitions
    required: bool,             // Required flag
    created_at: Timestamp,      // Creation timestamp
    created_by: String,         // Creator address
    deprecated: bool,           // Deprecation status
    version: String,            // Version string
}
```

### Issuer
```rust
struct Issuer {
    addr: String,               // Issuer address
    name: String,               // Human-readable name
    description: String,         // Issuer description
    uri: String,                // Metadata URI
    active: bool,               // Active status
    registered_at: Timestamp,   // Registration timestamp
    total_attestations: u64,    // Total attestations issued
    authorized_schemas: Vec<String>, // Authorized schemas
}
```

## Usage Examples

### EVM (Solidity)

**Deploy Contract:**
```solidity
AttestationRegistry registry = new AttestationRegistry();
```

**Register Issuer:**
```solidity
registry.registerIssuer(
    "Polymera Labs",
    "Official Polymera OS issuer",
    "https://polymera-os.org/issuer"
);
```

**Create Schema:**
```solidity
bytes32[] memory fields = new bytes32[](2);
fields[0] = keccak256("name");
fields[1] = keccak256("age");

string[] memory fieldTypes = new string[](2);
fieldTypes[0] = "string";
fieldTypes[1] = "uint256";

bytes32 schemaId = registry.createSchema(
    "Person",
    "Person identification schema",
    fields,
    fieldTypes,
    "1.0.0"
);
```

**Issue Attestation:**
```solidity
bytes memory data = abi.encode("John Doe", 30);
bytes32[] memory tags = new bytes32[](2);
tags[0] = keccak256("person");
tags[1] = keccak256("identification");

uint256 attestationId = registry.issueAttestation(
    subject,
    schemaId,
    data,
    0, // never expires
    "https://polymera-os.org/attestation/1",
    tags
);
```

### CosmWasm (Rust)

**Instantiate Contract:**
```rust
let msg = InstantiateMsg {
    name: "Polymera Registry".to_string(),
    description: "Official Polymera OS attestation registry".to_string(),
    uri: "https://polymera-os.org".to_string(),
    min_attestation_lifetime: 86400, // 1 day
    max_attestation_lifetime: 31536000, // 1 year
    attestation_fee: Uint128::zero(),
    attestation_fee_enabled: false,
    max_tags_per_attestation: 10,
    max_attestation_data_size: 1024,
    admin: admin.to_string(),
};

let res = instantiate(deps, env, info, msg)?;
```

**Register Issuer:**
```rust
let msg = ExecuteMsg::RegisterIssuer {
    name: "Polymera Labs".to_string(),
    description: "Official Polymera OS issuer".to_string(),
    uri: "https://polymera-os.org/issuer".to_string(),
};

let res = execute(deps, env, info, msg)?;
```

**Issue Attestation:**
```rust
let data = Binary::from(r#"{"name": "John Doe", "age": 30}"#.as_bytes());
let tags = vec!["person".to_string(), "identification".to_string()];

let msg = ExecuteMsg::IssueAttestation {
    subject: subject.to_string(),
    schema_id: "default-1.0.0".to_string(),
    data,
    expires_at: None,
    uri: "https://polymera-os.org/attestation/1".to_string(),
    tags,
};

let res = execute(deps, env, info, msg)?;
```

## Testing

### EVM Tests
```bash
cd contracts/attest/evm
forge test
```

**Test Coverage:**
- Constructor and initialization
- Schema management (create, deprecate)
- Issuer management (register, update, deactivate)
- Authorization (grant, revoke)
- Attestation lifecycle (issue, revoke, update)
- Validation and error handling
- Access control and permissions
- Edge cases and boundary conditions

### CosmWasm Tests
```bash
cd contracts/attest/cosmos
cargo test
```

**Test Coverage:**
- Contract instantiation
- Message handling and validation
- Storage operations and indexing
- Query functionality
- Error handling and edge cases
- Integration scenarios

## Deployment

### EVM Deployment

**Prerequisites:**
- Foundry installed
- Target network configured
- Private key with sufficient funds

**Deploy:**
```bash
cd contracts/attest/evm
forge build
forge create AttestationRegistry --rpc-url <RPC_URL> --private-key <PRIVATE_KEY>
```

**Verify:**
```bash
forge verify-contract <CONTRACT_ADDRESS> AttestationRegistry --chain-id <CHAIN_ID>
```

### CosmWasm Deployment

**Prerequisites:**
- Rust toolchain
- wasm-pack installed
- Target network configured
- Account with sufficient funds

**Build:**
```bash
cd contracts/attest/cosmos
cargo wasm
```

**Deploy:**
```bash
wasmd tx wasm store target/wasm32-unknown-unknown/release/polymera_attestation.wasm \
  --from <ACCOUNT> \
  --chain-id <CHAIN_ID> \
  --gas auto \
  --gas-adjustment 1.3
```

**Instantiate:**
```bash
wasmd tx wasm instantiate <CODE_ID> \
  '{"name":"Polymera Registry","description":"Official registry","uri":"https://polymera-os.org","min_attestation_lifetime":86400,"max_attestation_lifetime":31536000,"attestation_fee":"0","attestation_fee_enabled":false,"max_tags_per_attestation":10,"max_attestation_data_size":1024,"admin":"<ADMIN_ADDRESS>"}' \
  --from <ACCOUNT> \
  --chain-id <CHAIN_ID> \
  --gas auto \
  --gas-adjustment 1.3
```

## Configuration

### Registry Settings

**Attestation Lifetime:**
- Minimum: 1 day (86400 seconds)
- Maximum: 1 year (31536000 seconds)

**Data Constraints:**
- Maximum data size: 1KB (1024 bytes)
- Maximum tags: 10 per attestation

**Fee Configuration:**
- Configurable attestation fees
- Fee collection and withdrawal
- Fee enable/disable controls

### Access Control

**Admin Functions:**
- Configuration updates
- Issuer authorization
- Fee management
- Emergency controls

**Issuer Functions:**
- Attestation issuance
- Attestation updates
- Self-management

**Public Functions:**
- Attestation queries
- Schema queries
- Issuer queries
- Search operations

## Security Considerations

### Smart Contract Security
- **Reentrancy Protection**: Guards against reentrancy attacks
- **Access Control**: Role-based permissions and ownership
- **Input Validation**: Comprehensive parameter validation
- **Gas Optimization**: Efficient storage and computation patterns
- **Event Logging**: Complete audit trail of operations

### Cryptographic Security
- **Signature Verification**: Cryptographic attestation verification
- **Hash-based Identifiers**: Secure ID generation
- **Timestamp Validation**: Secure time-based operations
- **Revocation Support**: Secure attestation revocation

### Operational Security
- **Admin Controls**: Secure administrative operations
- **Emergency Procedures**: Emergency stop and recovery
- **Upgrade Mechanisms**: Secure contract upgrade paths
- **Monitoring**: Comprehensive event monitoring

## Monitoring and Analytics

### Event Tracking
- **Attestation Events**: Issue, update, revoke, expire
- **Schema Events**: Create, deprecate
- **Issuer Events**: Register, update, deactivate
- **Authorization Events**: Grant, revoke

### Metrics and KPIs
- **Attestation Volume**: Total and per-issuer counts
- **Schema Usage**: Popularity and adoption metrics
- **Issuer Activity**: Registration and activity patterns
- **Performance Metrics**: Gas usage and transaction times

### Health Monitoring
- **Contract Health**: Functionality and performance
- **Network Status**: Blockchain connectivity and health
- **API Performance**: Response times and availability
- **Error Tracking**: Error rates and patterns

## Integration

### Indexer Integration
- **Event Parsing**: Real-time event processing
- **Data Storage**: Efficient data storage and retrieval
- **Search API**: Advanced search and filtering
- **Webhook Support**: Real-time notifications

### API Integration
- **gRPC Client**: Native gRPC client libraries
- **REST API**: HTTP/REST endpoint mapping
- **WebSocket**: Real-time event streaming
- **SDK Support**: Language-specific SDKs

### Frontend Integration
- **Web3 Integration**: EVM wallet connections
- **Cosmos Integration**: Cosmos wallet connections
- **UI Components**: Reusable React components
- **State Management**: Redux/Context integration

## Development

### Local Development

**EVM Environment:**
```bash
cd contracts/attest/evm
forge install
forge build
forge test
```

**CosmWasm Environment:**
```bash
cd contracts/attest/cosmos
cargo build
cargo test
cargo wasm
```

**API Development:**
```bash
cd services/attest
# Install dependencies and run locally
```

### Contributing

1. **Fork the repository**
2. **Create a feature branch**
3. **Implement changes with tests**
4. **Ensure all tests pass**
5. **Submit a pull request**

**Development Guidelines:**
- Follow existing code style and patterns
- Add comprehensive tests for new functionality
- Update documentation for API changes
- Ensure security best practices
- Optimize for gas efficiency (EVM) and performance (CosmWasm)

## Roadmap

### Phase 1: Core Functionality ✅
- [x] Basic attestation management
- [x] Schema system
- [x] Issuer management
- [x] Access control
- [x] Basic validation

### Phase 2: Advanced Features 🚧
- [ ] Advanced search and filtering
- [ ] Batch operations
- [ ] Attestation templates
- [ ] Advanced metadata support
- [ ] Performance optimizations

### Phase 3: Ecosystem Integration 🚧
- [ ] Cross-chain attestations
- [ ] Attestation marketplaces
- [ ] Third-party integrations
- [ ] Advanced analytics
- [ ] Mobile SDKs

### Phase 4: Enterprise Features 📋
- [ ] Multi-signature support
- [ ] Advanced compliance features
- [ ] Enterprise SSO integration
- [ ] Advanced reporting
- [ ] Custom workflows

## Support

### Documentation
- **API Reference**: Complete API documentation
- **Integration Guides**: Step-by-step integration tutorials
- **Examples**: Code examples and use cases
- **FAQ**: Common questions and answers

### Community
- **Discord**: Community discussions and support
- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: General discussions and ideas
- **Contributing**: Development guidelines and contribution

### Enterprise Support
- **Professional Services**: Custom development and integration
- **Training**: Team training and workshops
- **Consulting**: Architecture and implementation guidance
- **Support Contracts**: Priority support and SLAs

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Acknowledgments

- **OpenZeppelin**: Security-focused smart contract libraries
- **CosmWasm**: Rust-based smart contract platform
- **gRPC**: High-performance RPC framework
- **Protocol Buffers**: Language-agnostic data serialization
- **Community Contributors**: Open source contributors and feedback

---

For more information, visit [Polymera OS](https://polymera-os.org) or join our [Discord community](https://discord.gg/polymera-os).
