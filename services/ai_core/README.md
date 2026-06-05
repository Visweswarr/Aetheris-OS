# Aetheris AI Core Service

A long-running assistant daemon providing LLM capabilities over local IPC with deterministic logging, CBOR audit trails, and CapTokens v2 authorization.

## Features

- **Long-running daemon**: Provides persistent AI assistant capabilities
- **IPC Communication**: Unix domain sockets (Unix) / Named Pipes (Windows) with protobuf messages
- **Deterministic logging**: CBOR audit trails for every request
- **CapTokens v2**: Fine-grained capability-based access control
- **Tool calling**: Extensible tool registry with built-in tools
- **Model management**: Support for multiple LLM backends (mock, llama, openai, anthropic)
- **Streaming support**: Real-time response streaming
- **Session management**: Concurrent session handling with limits

## Architecture

```text
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client App    │    │   AI Core       │    │   Model         │
│                 │◄──►│   Service       │◄──►│   Manager       │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │   Tool          │
                       │   Registry      │
                       │                 │
                       └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │   CapToken      │
                       │   Manager       │
                       │                 │
                       └─────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Tokio runtime
- Unix domain socket support (Unix) or Named Pipes (Windows)

### Installation

```bash
cd services/ai_core
cargo build --release
```

### Configuration

Create configuration files:

#### `models.toml`
```toml
[global_settings]
default_temperature = 0.7
default_max_tokens = 2048
default_top_p = 0.9
default_top_k = 40
timeout_seconds = 30
retry_attempts = 3
enable_audit_logging = true

default_model = "mock"

[tools.mock]
model_type = "Mock"
enabled = true
```

#### `tools.toml`
```toml
[global_settings]
max_concurrent_executions = 10
default_timeout_seconds = 30
default_cache_ttl_seconds = 300
enable_caching = true
enable_audit_logging = true
max_cache_size = 1000

[tools.echo]
enabled = true
timeout_seconds = 5
cache_ttl_seconds = 60
max_memory_mb = 10
required_capabilities = ["tools:echo"]

[tools.calculator]
enabled = true
timeout_seconds = 10
cache_ttl_seconds = 300
max_memory_mb = 20
required_capabilities = ["tools:calculator"]
```

#### `cap_tokens.toml`
```toml
[validation]
require_signature = false
check_expiration = true
max_clock_skew = 300
default_token_lifetime = 3600

[caching]
enabled = true
cache_ttl = 300
max_cache_size = 1000
cache_validation_results = true

[audit]
enabled = true
log_all_requests = true
log_validation_failures = true
log_capability_checks = true

[issuers."aetheris-system"]
name = "Aetheris System"
public_key = ""
trusted = true
max_token_lifetime = 3600
allowed_capabilities = ["ai:chat", "ai:tools", "ai:admin"]
```

### Running the Service

```bash
# Start the daemon
./target/release/aetheris-ai-core \
  --socket /run/aetheris/ai.sock \
  --model-config /etc/aetheris/ai_core/models.toml \
  --tool-config /etc/aetheris/ai_core/tools.toml \
  --cap-config /etc/aetheris/ai_core/cap_tokens.toml \
  --log-level info \
  --deterministic \
  --audit-log-dir /var/log/aetheris/ai_core \
  --max-sessions 100 \
  --request-timeout 30
```

## API Reference

### Message Types

#### Ping Request/Response
```protobuf
message PingRequest {
  string client_id = 1;
  string version = 2;
}

message PingResponse {
  string server_id = 1;
  string version = 2;
  int64 server_time = 3;
  ServiceStatus status = 4;
}
```

#### Chat Request/Response
```protobuf
message ChatRequest {
  string prompt = 1;
  ChatConfig config = 2;
  repeated MessageContext context = 3;
  bool stream = 4;
}

message ChatResponse {
  string response = 1;
  bool is_complete = 2;
  ChatMetrics metrics = 3;
  repeated ToolCall tool_calls = 4;
}
```

#### Tool Call Request/Response
```protobuf
message ToolCallRequest {
  string tool_name = 1;
  map<string, string> parameters = 2;
  string context = 3;
}

message ToolCallResponse {
  string result = 1;
  bool success = 2;
  string error_message = 3;
  ToolMetrics metrics = 4;
}
```

### Capability Tokens

CapTokens v2 provide fine-grained access control:

```protobuf
message CapToken {
  string token_id = 1;
  string capability = 2;
  int64 expires_at = 3;
  bytes signature = 4;
  string issuer = 5;
}
```

#### Supported Capabilities

- `ai:chat` - Generate chat responses
- `ai:tools` - Execute tools
- `ai:admin` - Administrative operations

### Built-in Tools

#### Echo Tool
```bash
# Echo back input message
tool_name: "echo"
parameters: {"message": "Hello, World!"}
```

#### Calculator Tool
```bash
# Simple arithmetic
tool_name: "calculator"
parameters: {"expression": "2+2"}
```

#### File Operations Tool
```bash
# List files
tool_name: "file_ops"
parameters: {"operation": "list", "path": "/tmp"}

# Read file
tool_name: "file_ops"
parameters: {"operation": "read", "path": "/etc/hosts"}
```

#### System Info Tool
```bash
# Get system information
tool_name: "system_info"
parameters: {"type": "basic"}
```

## Development

### Building

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run with specific features
cargo build --features llama,openai
```

### Features

- `mock` (default) - Mock model backend for testing
- `llama` - Llama.cpp integration
- `openai` - OpenAI API integration
- `anthropic` - Anthropic API integration

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_ping_roundtrip

# Run with logging
RUST_LOG=debug cargo test
```

### Adding Custom Tools

Implement the `Tool` trait:

```rust
use aetheris_ai_core::tools::{Tool, ToolResult, ToolInfo};

pub struct MyCustomTool;

#[async_trait::async_trait]
impl Tool for MyCustomTool {
    async fn execute(&self, parameters: &HashMap<String, String>) -> Result<ToolResult> {
        // Your tool implementation
        Ok(ToolResult {
            output: "Custom tool result".to_string(),
            success: true,
            error: None,
            memory_used_mb: 1,
            cache_hit: false,
            tool_version: "1.0.0".to_string(),
            execution_time_ms: 10,
            request_id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        })
    }
    
    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            name: "my_custom_tool".to_string(),
            version: "1.0.0".to_string(),
            description: "My custom tool".to_string(),
            parameters: vec![],
            required_capabilities: vec!["tools:custom".to_string()],
            timeout_seconds: 30,
            cache_ttl_seconds: 300,
        }
    }
    
    fn is_available(&self) -> bool {
        true
    }
    
    fn get_required_capabilities(&self) -> Vec<String> {
        vec!["tools:custom".to_string()]
    }
}
```

### Adding Custom Models

Implement the `ModelBackend` trait:

```rust
use aetheris_ai_core::model::{ModelBackend, ModelRequest, ModelResponse};

pub struct MyCustomModel;

#[async_trait::async_trait]
impl ModelBackend for MyCustomModel {
    async fn generate_response(&self, request: &ModelRequest) -> Result<ModelResponse> {
        // Your model implementation
        Ok(ModelResponse {
            content: "Model response".to_string(),
            tokens_generated: 10,
            tokens_input: 5,
            confidence: 0.95,
            model_used: "my_custom_model".to_string(),
            tool_calls: Vec::new(),
            processing_time_ms: 100,
            request_id: request.request_id.clone(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        })
    }
    
    fn get_info(&self) -> ModelInfo {
        ModelInfo {
            name: "my_custom_model".to_string(),
            version: "1.0.0".to_string(),
            model_type: ModelType::Mock,
            max_tokens: 2048,
            context_length: 4096,
            parameters: HashMap::new(),
        }
    }
    
    fn is_ready(&self) -> bool {
        true
    }
}
```

## Security

### Capability Tokens

- Tokens are cryptographically signed using Ed25519
- Expiration times are enforced
- Issuer trust is validated
- Capability scoping prevents privilege escalation

### Audit Logging

- All requests are logged in CBOR format
- Deterministic logging for replay capabilities
- Separate audit logs for different components
- Configurable log retention

### Input Validation

- All inputs are validated before processing
- Parameter sanitization for tool calls
- Rate limiting and session limits
- Timeout enforcement

## Monitoring

### Metrics

- Request processing time
- Token generation counts
- Memory usage
- Cache hit rates
- Error rates

### Health Checks

- Ping endpoint for health monitoring
- Service status reporting
- System metrics collection
- Active session tracking

## Troubleshooting

### Common Issues

1. **Socket permission denied**
   - Ensure the socket directory exists and is writable
   - Check file permissions on the socket file

2. **Model loading failed**
   - Verify model configuration files exist
   - Check model file paths and permissions
   - Ensure required features are enabled

3. **Tool execution failed**
   - Verify tool configuration
   - Check capability tokens
   - Review tool-specific requirements

4. **Capability denied**
   - Verify capability token is valid
   - Check token expiration
   - Ensure required capabilities are granted

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug ./aetheris-ai-core

# Check audit logs
tail -f /var/log/aetheris/ai_core/audit/audit_$(date +%Y-%m-%d).cbor

# Monitor socket connections
lsof /run/aetheris/ai.sock
```

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Support

- GitHub Issues: [Report bugs and request features](https://github.com/aetheris-os/polymera-os/issues)
- Documentation: [Full API documentation](https://docs.aetheris-os.com)
- Community: [Join our Discord](https://discord.gg/aetheris-os)
