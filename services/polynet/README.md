# Polymera OS Network Services (PolyNet)

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

Comprehensive networking services for Polymera OS, including peer quarantine, mesh networking, and protocol implementations.

## 🚀 Features

### 🛡️ Quarantine System
- **Local Denylist**: Maintain local lists of quarantined peers with persistent storage
- **Signed Entries**: Cryptographically signed quarantine entries with Ed25519
- **Automatic Expiry**: Time-based expiry with automatic cleanup
- **Severity Levels**: Configurable severity levels (Low, Medium, High, Critical, Permanent)
- **Rich Metadata**: Evidence, reasons, and custom metadata for each entry
- **Authority System**: Multi-authority support with signature verification
- **Background Cleanup**: Automatic expired entry removal
- **Statistics**: Comprehensive monitoring and reporting

### 🌐 Network Protocol Support (Optional)
- **libp2p Integration**: Full libp2p mesh networking support
- **QUIC Protocol**: High-performance QUIC transport
- **Metrics**: Prometheus metrics integration

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
polymera-polynet = { version = "0.1.0", features = ["full"] }
```

### Feature Flags

- `default`: Basic quarantine system with libp2p support
- `libp2p-support`: Enable libp2p networking integration
- `quic-support`: Enable QUIC transport protocol
- `metrics`: Enable Prometheus metrics collection
- `full`: Enable all features

## 🔧 Quick Start

### Basic Quarantine Usage

```rust
use polymera_polynet::{
    QuarantineSystem, QuarantineConfig, QuarantineReason, QuarantineSeverity
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create quarantine system
    let config = QuarantineConfig::default();
    let quarantine = Arc::new(QuarantineSystem::new(
        config, 
        "my_authority".to_string()
    )?);

    // Quarantine a malicious peer
    quarantine.quarantine_peer(
        "peer_12345".to_string(),
        QuarantineReason::MaliciousBehavior,
        QuarantineSeverity::High,
        Some("Detected consensus violations".to_string()),
        None,
    )?;

    // Check if peer is quarantined
    if quarantine.is_quarantined("peer_12345")? {
        println!("Peer is quarantined!");
    }

    // Start background cleanup
    let quarantine_clone = Arc::clone(&quarantine);
    tokio::spawn(async move {
        quarantine_clone.start_cleanup_task().await;
    });

    Ok(())
}
```

### Network Integration Example

```rust
use polymera_polynet::{QuarantineSystem, QuarantineReason, QuarantineSeverity};

async fn handle_peer_connection(peer_id: &str, quarantine: &QuarantineSystem) -> bool {
    // Check quarantine before allowing connection
    match quarantine.is_quarantined(peer_id) {
        Ok(true) => {
            println!("Blocking connection from quarantined peer: {}", peer_id);
            false // Reject connection
        }
        Ok(false) => {
            println!("Allowing connection from peer: {}", peer_id);
            true // Allow connection
        }
        Err(e) => {
            eprintln!("Error checking quarantine status: {}", e);
            false // Fail safe - reject on error
        }
    }
}

async fn report_malicious_behavior(
    peer_id: &str, 
    evidence: &str,
    quarantine: &QuarantineSystem
) -> Result<(), Box<dyn std::error::Error>> {
    // Report and quarantine malicious peer
    quarantine.quarantine_peer(
        peer_id.to_string(),
        QuarantineReason::MaliciousBehavior,
        QuarantineSeverity::Critical,
        Some(evidence.to_string()),
        None,
    )?;
    
    println!("Quarantined malicious peer: {}", peer_id);
    Ok(())
}
```

## 🏗️ Architecture

### Quarantine System Design

```
┌─────────────────────────────────────────┐
│           QuarantineSystem              │
├─────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────────────┐ │
│ │ In-Memory   │ │ Persistent Storage  │ │
│ │ Cache       │ │ (JSON + Backups)    │ │
│ │             │ │                     │ │
│ │ • Peer      │ │ • Signed Entries    │ │
│ │   Lookup    │ │ • Authority Info    │ │
│ │ • Fast      │ │ • Metadata          │ │
│ │   Access    │ │ • Version Control   │ │
│ └─────────────┘ └─────────────────────┘ │
└─────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────┐
│        Background Processes             │
├─────────────────────────────────────────┤
│ • Automatic Cleanup (Expired Entries)  │
│ • File Change Detection & Reload       │
│ • Statistics Collection                 │
│ • Backup Management                     │
└─────────────────────────────────────────┘
```

### File Format

The quarantine system uses JSON format with digital signatures:

```json
{
  "version": "1.0",
  "last_updated": "2024-01-15T10:30:00Z",
  "authority": "my_authority",
  "authority_public_key": "ed25519_public_key_hex",
  "entries": [
    {
      "id": "uuid",
      "peer_id": "peer_identifier",
      "reason": "malicious_behavior",
      "severity": "high",
      "quarantined_at": "2024-01-15T10:00:00Z",
      "expires_at": "2024-01-16T10:00:00Z",
      "imposed_by": "my_authority",
      "evidence": "Detected sending corrupted data",
      "metadata": {
        "detection_method": "consensus_validator",
        "severity_score": "8.5"
      },
      "signature": "ed25519_signature_hex",
      "signer_public_key": "ed25519_public_key_hex"
    }
  ],
  "signature": "list_signature_hex"
}
```

## 🔐 Security Features

### Cryptographic Signatures
- **Ed25519 Signatures**: All entries and lists are cryptographically signed
- **Authority Verification**: Multi-authority support with trusted key management
- **Tamper Detection**: Signature verification prevents unauthorized modifications
- **Key Rotation**: Support for authority key updates

### Data Integrity
- **Atomic Operations**: File updates use atomic write operations
- **Backup System**: Automatic backup creation with retention policies
- **Corruption Recovery**: Graceful handling of corrupted data files
- **Version Control**: File format versioning for future compatibility

## 📊 Monitoring and Statistics

### Built-in Statistics
```rust
let stats = quarantine.get_statistics().await?;
println!("Total entries: {}", stats.total_entries);
println!("Active entries: {}", stats.active_entries);
println!("Expired entries: {}", stats.expired_entries);

// Breakdown by reason and severity
for (reason, count) in stats.entries_by_reason {
    println!("Reason '{}': {} entries", reason, count);
}
```

### Integration with Monitoring Systems
```rust
// With Prometheus metrics (optional feature)
#[cfg(feature = "metrics")]
use polymera_polynet::metrics::QuarantineMetrics;

let metrics = QuarantineMetrics::new()?;
metrics.update_from_stats(&stats);
```

## 🧪 Testing

The quarantine system includes comprehensive tests covering:

### Unit Tests
- **Basic Operations**: Quarantine, unquarantine, check status
- **Expiry Handling**: Automatic expiry and cleanup
- **Signature Verification**: Cryptographic signature validation
- **File Persistence**: Save/load operations with corruption handling
- **Error Cases**: Invalid inputs, corruption detection, permission errors

### Integration Tests
- **Multi-Authority**: Cross-authority entry import/export
- **Background Processing**: Cleanup task and file monitoring
- **Performance**: High-load scenarios and memory usage
- **Network Integration**: libp2p and QUIC protocol integration

### Test Coverage
```bash
# Run all tests
cargo test

# Run specific test categories
cargo test quarantine_tests
cargo test integration_tests

# Run with coverage
cargo tarpaulin --out Html
```

## 🚀 Performance

### Benchmarks
- **Lookup Performance**: O(1) peer quarantine checks
- **Memory Usage**: ~1KB per quarantined peer
- **Throughput**: 100,000+ checks/second on modern hardware
- **Concurrent Access**: Optimized RwLock usage for high concurrency

### Scalability
- **Large Lists**: Efficient handling of 10,000+ quarantined peers
- **Background Cleanup**: Non-blocking expired entry removal
- **File Operations**: Atomic updates minimize lock contention
- **Memory Management**: Configurable limits and automatic cleanup

## 🔧 Configuration

### Basic Configuration
```rust
use polymera_polynet::{QuarantineConfig, QuarantineSeverity};
use std::collections::HashMap;
use std::time::Duration;

let mut config = QuarantineConfig::default();

// File paths
config.data_file = PathBuf::from("/etc/polymera/quarantine.json");
config.backup_dir = PathBuf::from("/var/lib/polymera/quarantine_backups");

// Duration settings
config.default_durations.insert(
    QuarantineSeverity::Low, 
    Duration::from_secs(300)  // 5 minutes
);
config.default_durations.insert(
    QuarantineSeverity::High, 
    Duration::from_secs(86400)  // 24 hours
);

// Cleanup settings
config.cleanup_interval = Duration::from_secs(300);  // 5 minutes
config.backup_retention = 10;  // Keep 10 backups
config.max_entries = 50000;    // Memory limit

// Security settings
config.verify_signatures = true;
config.trusted_authorities.insert("authority_public_key_hex".to_string());
```

### Advanced Configuration
```rust
// Custom signing keypair
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

let mut csprng = OsRng;
let keypair = Keypair::generate(&mut csprng);
config.signing_keypair = keypair;

// Custom severity durations
let custom_durations = HashMap::from([
    (QuarantineSeverity::Low, Duration::from_secs(60)),      // 1 minute
    (QuarantineSeverity::Medium, Duration::from_secs(900)),  // 15 minutes  
    (QuarantineSeverity::High, Duration::from_secs(7200)),   // 2 hours
    (QuarantineSeverity::Critical, Duration::from_secs(86400)), // 1 day
    // QuarantineSeverity::Permanent has no duration (never expires)
]);
config.default_durations = custom_durations;
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup
```bash
# Clone the repository
git clone https://github.com/polymera-os/polymera-os.git
cd polymera-os/services/polynet

# Run tests
cargo test

# Run examples
cargo run --example basic_quarantine

# Build with all features
cargo build --features full

# Run benchmarks
cargo bench
```

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../../LICENSE) file for details.

## 🔗 Related Projects

- [libp2p](https://github.com/libp2p/rust-libp2p) - Peer-to-peer networking library
- [Quinn](https://github.com/quinn-rs/quinn) - QUIC implementation in Rust
- [Ed25519-Dalek](https://github.com/dalek-cryptography/ed25519-dalek) - Ed25519 digital signatures

## 📚 Documentation

- [API Documentation](https://docs.rs/polymera-polynet)
- [Architecture Guide](../../docs/architecture/polynet.md)
- [Security Model](../../docs/security/quarantine.md)
- [Integration Examples](./examples/)

---

**Polymera OS Network Services** - Building secure, scalable peer-to-peer networks for the decentralized future.
