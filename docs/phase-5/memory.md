# P5-05 — Minimal Memory Store (Local, Private)

## Overview

The Minimal Memory Store provides a local per-profile key-value store with TTL (Time To Live) and topic tags for short-term memory with user control. It implements deterministic snapshots using NGFS patterns from Phase 4 and includes comprehensive secret redaction with regex policies.

## Architecture

### Core Components

- **MemoryStore**: Main store implementation with KV operations, TTL, and tag indexing
- **MemoryEntry**: Individual memory entries with metadata and access tracking
- **RedactionPolicy**: Configurable regex-based secret detection and redaction
- **NGFS Integration**: Optional deterministic snapshot support for audit trails
- **IPC Tool Interface**: Seamless integration with AI Core Service tool calls

### Storage Location

Memory is stored under `~/.aetheris/memory/` with the following structure:

```
~/.aetheris/memory/
├── memory_data.cbor          # Main data file (CBOR serialized)
├── snapshots/                # NGFS snapshots (if enabled)
│   ├── snapshot_001.cid
│   └── snapshot_002.cid
└── logs/                     # Audit logs
    └── memory_audit.log
```

## Features

### 🔑 **Key-Value Operations**

- **Put**: Store values with optional TTL and tags
- **Get**: Retrieve values with access count tracking
- **Delete**: Remove specific entries
- **Clear**: Remove all entries
- **Query**: Find entries by tag intersection

### ⏰ **TTL (Time To Live)**

- Configurable expiration times per entry
- Automatic cleanup of expired entries
- Background cleanup task with configurable intervals
- Deterministic expiration in deterministic mode

### 🏷️ **Topic Tags**

- Multi-tag support per entry
- Tag-based querying with intersection logic
- Efficient tag indexing for fast queries
- Automatic tag index maintenance

### 🔒 **Secret Redaction**

- Regex-based secret detection
- Configurable redaction policies
- Support for passwords, API keys, tokens, emails, phone numbers
- Automatic redaction flagging

### 📊 **Statistics & Monitoring**

- Real-time statistics tracking
- Access count monitoring
- Storage size tracking
- Performance metrics

### 🔄 **Persistence & Snapshots**

- CBOR-based local persistence
- Optional NGFS deterministic snapshots
- Automatic data loading on startup
- Configurable snapshot intervals

## Configuration

### MemoryConfig Structure

```rust
pub struct MemoryConfig {
    pub base_path: PathBuf,                    // Storage directory
    pub max_entries: usize,                    // Maximum entries limit
    pub default_ttl_seconds: u64,              // Default TTL
    pub cleanup_interval_seconds: u64,         // Cleanup frequency
    pub enable_redaction: bool,                // Enable secret redaction
    pub redaction_policies: Vec<RedactionPolicy>, // Redaction rules
    pub enable_ngfs: bool,                     // Enable NGFS snapshots
    pub ngfs_snapshot_interval_seconds: u64,   // Snapshot frequency
}
```

### Default Configuration

```rust
pub fn default_memory_config(base_path: PathBuf) -> MemoryConfig {
    MemoryConfig {
        base_path,
        max_entries: 10000,
        default_ttl_seconds: 86400,        // 24 hours
        cleanup_interval_seconds: 3600,    // 1 hour
        enable_redaction: true,
        redaction_policies: default_redaction_policies(),
        enable_ngfs: false,
        ngfs_snapshot_interval_seconds: 86400, // 24 hours
    }
}
```

## Redaction Policies

### Default Policies

The memory store includes comprehensive default redaction policies:

1. **Password Detection**
   - Pattern: `(?i)(password|passwd|pwd)\s*[:=]\s*\S+`
   - Replacement: `[REDACTED]`

2. **API Key Detection**
   - Pattern: `(?i)(api[_-]?key|apikey)\s*[:=]\s*\S+`
   - Replacement: `[REDACTED]`

3. **Token Detection**
   - Pattern: `(?i)(token|bearer)\s*[:=]\s*\S+`
   - Replacement: `[REDACTED]`

4. **Secret Detection**
   - Pattern: `(?i)(secret|private[_-]?key)\s*[:=]\s*\S+`
   - Replacement: `[REDACTED]`

5. **Email Detection**
   - Pattern: `\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b`
   - Replacement: `[EMAIL_REDACTED]`

6. **Phone Number Detection**
   - Pattern: `(\+?1[-.\s]?)?\(?([0-9]{3})\)?[-.\s]?([0-9]{3})[-.\s]?([0-9]{4})`
   - Replacement: `[PHONE_REDACTED]`

### Custom Policies

You can define custom redaction policies:

```rust
let custom_policy = RedactionPolicy {
    name: "custom_secret".to_string(),
    pattern: r"secret:\s*\w+".to_string(),
    replacement: "[CUSTOM_REDACTED]".to_string(),
    description: Some("Custom secret pattern".to_string()),
};
```

## IPC Tool Interface

The memory store integrates seamlessly with the AI Core Service through IPC tool calls:

### Available Tools

1. **`mem.put`** - Store a value
2. **`mem.get`** - Retrieve a value
3. **`mem.query`** - Query by tags
4. **`mem.delete`** - Delete a value
5. **`mem.stats`** - Get statistics

### Tool Call Examples

#### Store a Value
```json
{
  "tool_name": "mem.put",
  "arguments": {
    "key": "user_preference",
    "value": "dark_mode: true, language: en",
    "tags": ["preferences", "ui"],
    "ttl_seconds": 86400
  }
}
```

#### Retrieve a Value
```json
{
  "tool_name": "mem.get",
  "arguments": {
    "key": "user_preference"
  }
}
```

#### Query by Tags
```json
{
  "tool_name": "mem.query",
  "arguments": {
    "tags": ["preferences", "ui"]
  }
}
```

#### Delete a Value
```json
{
  "tool_name": "mem.delete",
  "arguments": {
    "key": "user_preference"
  }
}
```

#### Get Statistics
```json
{
  "tool_name": "mem.stats",
  "arguments": {}
}
```

## Usage Examples

### Basic Operations

```rust
use aetheris_ai_core::memory::{MemoryStore, default_memory_config};

// Create memory store
let config = default_memory_config(PathBuf::from("~/.aetheris/memory"));
let store = MemoryStore::new(config)?;
store.initialize().await?;

// Store a value
store.put(
    "user_preference".to_string(),
    "dark_mode: true".to_string(),
    vec!["preferences".to_string()],
    Some(86400), // 24 hour TTL
    "user123".to_string(),
    Some("session456".to_string())
).await?;

// Retrieve a value
let entry = store.get("user_preference").await?;
if let Some(entry) = entry {
    println!("Value: {}", entry.value);
    println!("Access count: {}", entry.access_count);
}

// Query by tags
let results = store.query(&["preferences".to_string()]).await?;
for result in results {
    println!("Found: {} = {}", result.key, result.value);
}

// Delete a value
let deleted = store.delete("user_preference").await?;
println!("Deleted: {}", deleted);
```

### With Redaction

```rust
// Store sensitive data
store.put(
    "api_config".to_string(),
    "api_key: abc123def456, password: secret123".to_string(),
    vec!["config".to_string()],
    None,
    "user123".to_string(),
    None
).await?;

// Retrieve (will be redacted)
let entry = store.get("api_config").await?.unwrap();
println!("Redacted: {}", entry.value); // "api_key: [REDACTED], password: [REDACTED]"
println!("Was redacted: {}", entry.redacted); // true
```

### Statistics

```rust
let stats = store.get_stats().await?;
println!("Total entries: {}", stats.total_entries);
println!("Active entries: {}", stats.active_entries);
println!("Expired entries: {}", stats.expired_entries);
println!("Total access count: {}", stats.total_access_count);
println!("Storage size: {} bytes", stats.storage_size_bytes);
```

## Performance Characteristics

### Memory Usage

- **Entry Overhead**: ~200 bytes per entry (metadata + indexing)
- **Tag Index**: ~50 bytes per tag per entry
- **Redaction**: Minimal overhead, regex compilation cached

### Performance Metrics

- **Put Operation**: ~1ms (in-memory)
- **Get Operation**: ~0.1ms (in-memory)
- **Query Operation**: ~1-5ms (depending on result set size)
- **Delete Operation**: ~0.5ms (in-memory)
- **Cleanup**: ~10-50ms (depending on expired entries)

### Scalability

- **Maximum Entries**: Configurable (default: 10,000)
- **Tag Queries**: Efficient intersection with O(n) complexity
- **Concurrent Access**: Thread-safe with Arc<RwLock<>>
- **Persistence**: CBOR serialization for fast I/O

## Security Considerations

### Data Protection

- **Local Storage**: Data stored locally under user's home directory
- **No Cloud Sync**: Explicitly local-only storage
- **Secret Redaction**: Automatic detection and redaction of sensitive data
- **Access Control**: Per-user isolation (user_id tracking)

### Audit Trail

- **Deterministic Snapshots**: NGFS integration for audit trails
- **Access Logging**: Track all access patterns and modifications
- **Redaction Logging**: Log when redaction policies are applied
- **Statistics Tracking**: Monitor usage patterns and storage growth

### Privacy

- **User Control**: Users control what data is stored
- **TTL Expiration**: Automatic cleanup of old data
- **Local Only**: No external data transmission
- **Configurable Policies**: Users can customize redaction rules

## Integration with AI Core Service

### Service Integration

The memory store is fully integrated into the AI Core Service:

```rust
// In AiCoreService::new()
let memory_config = default_memory_config(config.memory_dir.clone());
let memory_store = Arc::new(MemoryStore::new(memory_config)?);
memory_store.initialize().await?;

// Passed to IPC server for tool call handling
let ipc_server = Arc::new(IpcServer::new(
    &config.socket,
    model_manager.clone(),
    tool_registry.clone(),
    cap_token_manager.clone(),
    prompt_router.clone(),
    memory_store.clone(), // Memory store integration
    config.max_sessions,
    config.request_timeout,
).await?);
```

### Tool Call Routing

Memory tool calls are automatically routed:

```rust
// In IpcServer::handle_tool_call_request()
let tool_response = if request.tool_name.starts_with("mem.") {
    // Handle memory store operations
    self.handle_memory_tool_call(&request).await?
} else {
    // Execute regular tool
    self.tool_registry.execute_tool(&request.tool_name, &request.parameters).await?
};
```

## Testing

### Test Coverage

The memory store includes comprehensive test coverage:

- **Basic Operations**: Put, get, delete, clear
- **TTL Expiration**: Automatic cleanup testing
- **Tag Querying**: Single and multi-tag queries
- **Redaction**: All default policies tested
- **Persistence**: Save/load functionality
- **Tool Calls**: All IPC tool interfaces
- **Concurrent Access**: Thread safety testing
- **Error Handling**: Invalid inputs and edge cases
- **Unicode Support**: International character handling
- **Large Data**: Performance with large values

### Running Tests

```bash
cd services/ai_core
cargo test memory_tests
```

## Configuration Examples

### Development Configuration

```rust
let dev_config = MemoryConfig {
    base_path: PathBuf::from("./dev_memory"),
    max_entries: 1000,
    default_ttl_seconds: 3600,        // 1 hour
    cleanup_interval_seconds: 300,    // 5 minutes
    enable_redaction: true,
    redaction_policies: default_redaction_policies(),
    enable_ngfs: false,
    ngfs_snapshot_interval_seconds: 0,
};
```

### Production Configuration

```rust
let prod_config = MemoryConfig {
    base_path: PathBuf::from("~/.aetheris/memory"),
    max_entries: 50000,
    default_ttl_seconds: 604800,      // 1 week
    cleanup_interval_seconds: 3600,   // 1 hour
    enable_redaction: true,
    redaction_policies: default_redaction_policies(),
    enable_ngfs: true,
    ngfs_snapshot_interval_seconds: 86400, // 1 day
};
```

### High-Security Configuration

```rust
let security_config = MemoryConfig {
    base_path: PathBuf::from("~/.aetheris/memory"),
    max_entries: 10000,
    default_ttl_seconds: 86400,       // 24 hours
    cleanup_interval_seconds: 1800,   // 30 minutes
    enable_redaction: true,
    redaction_policies: vec![
        // Custom strict policies
        RedactionPolicy {
            name: "strict_password".to_string(),
            pattern: r"(?i)(password|pass|pwd|passwd)\s*[:=]\s*\S+".to_string(),
            replacement: "[PASSWORD_REDACTED]".to_string(),
            description: Some("Strict password detection".to_string()),
        },
        // Add more strict policies...
    ],
    enable_ngfs: true,
    ngfs_snapshot_interval_seconds: 3600, // 1 hour
};
```

## Troubleshooting

### Common Issues

1. **Permission Denied**
   - Ensure the memory directory is writable
   - Check user permissions for `~/.aetheris/memory/`

2. **Regex Compilation Errors**
   - Validate redaction policy patterns
   - Test regex patterns before deployment

3. **Memory Usage Growth**
   - Monitor statistics regularly
   - Adjust TTL settings for automatic cleanup
   - Consider reducing max_entries limit

4. **Performance Issues**
   - Check cleanup interval settings
   - Monitor tag index size
   - Consider disabling NGFS if not needed

### Debugging

Enable debug logging to troubleshoot issues:

```rust
// Set log level to debug
env::set_var("RUST_LOG", "aetheris_ai_core::memory=debug");
```

### Monitoring

Use the statistics tool to monitor memory store health:

```json
{
  "tool_name": "mem.stats",
  "arguments": {}
}
```

## Future Enhancements

### Planned Features

1. **Encryption**: Optional encryption for sensitive data
2. **Compression**: Data compression for large values
3. **Backup/Restore**: Automated backup and restore functionality
4. **Metrics Export**: Prometheus metrics integration
5. **Web UI**: Web-based management interface
6. **Data Migration**: Tools for migrating between configurations

### Extension Points

The memory store is designed for extensibility:

- **Custom Redaction Policies**: Easy addition of new patterns
- **NGFS Integration**: Pluggable snapshot backends
- **Storage Backends**: Support for different storage engines
- **Query Languages**: Advanced query capabilities
- **Event Hooks**: Custom event handling for operations

## Conclusion

The Minimal Memory Store provides a robust, secure, and efficient local storage solution for the AI Core Service. With comprehensive secret redaction, TTL management, and seamless IPC integration, it enables safe short-term memory with full user control while maintaining high performance and reliability.

The implementation follows Aetheris OS principles of local-first, privacy-preserving, and deterministic operation, making it an ideal foundation for AI assistant memory management.
