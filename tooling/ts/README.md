# AI Core Service TypeScript Bridge

A comprehensive TypeScript bridge for the Aetheris OS AI Core Service, providing easy integration with web applications and Node.js.

## Features

- **Streaming Chat**: Real-time streaming responses with backoff/abort support
- **Tool Calling**: Execute tools with strict type validation and capability enforcement
- **CapToken Management**: Secure authentication with fine-grained permissions
- **Real-time Metrics**: Monitor AI Core Service performance and health
- **Error Handling**: Comprehensive error handling with retry policies
- **Type Safety**: Full TypeScript support with strict typing

## Installation

```bash
npm install @aetheris/ai-bridge
```

## Quick Start

```typescript
import { AICoreBridge, createAICoreBridge } from '@aetheris/ai-bridge';

// Create bridge instance
const bridge = createAICoreBridge({
  socketPath: '/tmp/ai_core.sock',
  timeout: 30000,
  maxRetries: 3
});

// Initialize connection
await bridge.initialize();

// Chat with AI
const response = await bridge.chat('Hello, AI!');
console.log(response.content);

// Stream chat responses
for await (const chunk of bridge.chatStream('Tell me a story')) {
  process.stdout.write(chunk.chunk);
}

// Call tools
const result = await bridge.tool('open_file', { path: '/tmp/test.txt' });
console.log(result.result);

// Get metrics
const metrics = await bridge.getMetrics();
console.log(`Requests: ${metrics.requests.total}`);
```

## API Reference

### AICoreBridge

The main class for interacting with the AI Core Service.

#### Constructor

```typescript
new AICoreBridge(config?: AICoreConfig)
```

#### Methods

##### `initialize(): Promise<void>`

Initialize the connection to the AI Core Service.

```typescript
await bridge.initialize();
```

##### `chat(message: string, config?: ChatConfig, abortSignal?: AbortSignal): Promise<ChatResponse>`

Send a chat message and receive a response.

```typescript
const response = await bridge.chat('What is the capital of France?', {
  model: 'gpt-4',
  temperature: 0.7,
  maxTokens: 1000
});
```

##### `chatStream(message: string, config?: ChatConfig, abortSignal?: AbortSignal): AsyncGenerator<StreamingChunk>`

Stream chat responses in real-time.

```typescript
for await (const chunk of bridge.chatStream('Tell me a story')) {
  process.stdout.write(chunk.chunk);
  if (chunk.isComplete) break;
}
```

##### `tool(name: string, payload: Record<string, any>, options?: ToolCallOptions, abortSignal?: AbortSignal): Promise<ToolCallResult>`

Call a tool through the AI Core Service.

```typescript
const result = await bridge.tool('open_file', { 
  path: '/tmp/test.txt',
  mode: 'read' 
}, {
  timeoutSeconds: 30,
  async: false
});
```

##### `listTools(abortSignal?: AbortSignal): Promise<ToolDefinition[]>`

List all available tools.

```typescript
const tools = await bridge.listTools();
console.log(tools.map(t => t.name));
```

##### `getToolInfo(toolName: string, abortSignal?: AbortSignal): Promise<ToolDefinition>`

Get detailed information about a specific tool.

```typescript
const toolInfo = await bridge.getToolInfo('open_file');
console.log(toolInfo.description);
console.log(toolInfo.parameters);
```

##### `getMetrics(abortSignal?: AbortSignal): Promise<AICoreMetrics>`

Get AI Core Service metrics.

```typescript
const metrics = await bridge.getMetrics();
console.log(`Latency P95: ${metrics.latency.p95}ms`);
console.log(`Tokens/sec: ${metrics.tokens.perSecond}`);
```

##### `setCaps(token: CapToken): void`

Set CapToken for authentication.

```typescript
bridge.setCaps({
  tokenId: 'token-id',
  token: 'token-value',
  scopes: ['ai:chat', 'ai:tool.open_file'],
  expiresAt: Date.now() + 3600000,
  issuer: 'devctl',
  subject: 'user',
  audience: 'ai-core-service',
  createdAt: Date.now()
});
```

##### `getCaps(): CapToken | undefined`

Get current CapToken.

```typescript
const token = bridge.getCaps();
if (token) {
  console.log(`Scopes: ${token.scopes.join(', ')}`);
}
```

##### `clearCaps(): void`

Clear current CapToken.

```typescript
bridge.clearCaps();
```

##### `ping(abortSignal?: AbortSignal): Promise<PingResponse>`

Ping the AI Core Service.

```typescript
const response = await bridge.ping();
console.log(`Server: ${response.serverId} v${response.version}`);
```

##### `isServiceConnected(): boolean`

Check if connected to the AI Core Service.

```typescript
if (bridge.isServiceConnected()) {
  console.log('Connected to AI Core Service');
}
```

##### `disconnect(): Promise<void>`

Disconnect from the AI Core Service.

```typescript
await bridge.disconnect();
```

## Configuration

### AICoreConfig

```typescript
interface AICoreConfig {
  socketPath?: string;           // IPC socket path (default: '/tmp/ai_core.sock')
  timeout?: number;              // Request timeout in ms (default: 30000)
  maxRetries?: number;           // Max retry attempts (default: 3)
  backoffMultiplier?: number;    // Retry backoff multiplier (default: 2)
  initialRetryDelay?: number;    // Initial retry delay in ms (default: 1000)
  maxRetryDelay?: number;        // Max retry delay in ms (default: 10000)
  deterministic?: boolean;       // Enable deterministic behavior (default: true)
  seed?: number;                 // Default seed (default: 42)
}
```

### ChatConfig

```typescript
interface ChatConfig {
  model?: string;                // AI model to use (default: 'gpt-3.5-turbo')
  temperature?: number;          // Sampling temperature 0.0-2.0 (default: 0.7)
  maxTokens?: number;            // Max tokens to generate (default: 1000)
  topP?: number;                 // Top-p sampling parameter (default: 1.0)
  stopSequences?: string[];      // Stop sequences
  stream?: boolean;              // Enable streaming (default: false)
  context?: string[];            // Context messages
  sessionId?: string;            // Session ID for continuity
  includeMetadata?: boolean;     // Include response metadata (default: false)
}
```

## Events

The bridge extends EventEmitter and emits the following events:

```typescript
// Connection events
bridge.on('connected', () => console.log('Connected'));
bridge.on('disconnected', () => console.log('Disconnected'));
bridge.on('error', (error) => console.error('Error:', error));

// CapToken events
bridge.on('caps_updated', (token) => console.log('CapToken updated'));
bridge.on('caps_cleared', () => console.log('CapToken cleared'));

// Chat events
bridge.on('chat_error', (error) => console.error('Chat error:', error));
bridge.on('chat_stream_complete', (data) => console.log('Stream complete:', data));
bridge.on('chat_stream_error', (error) => console.error('Stream error:', error));

// Tool events
bridge.on('tool_error', (data) => console.error('Tool error:', data));
bridge.on('tool_list_error', (error) => console.error('Tool list error:', error));
bridge.on('tool_info_error', (data) => console.error('Tool info error:', data));

// Metrics events
bridge.on('metrics_error', (error) => console.error('Metrics error:', error));
bridge.on('ping_error', (error) => console.error('Ping error:', error));
```

## Error Handling

The bridge provides comprehensive error handling with retry policies:

```typescript
try {
  const response = await bridge.chat('Hello');
} catch (error) {
  if (error instanceof AIError) {
    console.error(`Error ${error.code}: ${error.message}`);
    if (error.retryable) {
      console.log('This error is retryable');
    }
  }
}
```

### AIError

```typescript
interface AIError {
  code: string;                  // Error code
  message: string;               // Error message
  details?: any;                 // Error details
  timestamp: Date;               // Error timestamp
  retryable?: boolean;           // Is retryable
  retryAfter?: number;           // Retry after ms
}
```

## Abort Signals

Support for aborting operations:

```typescript
const controller = new AbortController();
const abortSignal = controller.signal;

// Start operation
const chatPromise = bridge.chat('Long message', undefined, abortSignal);

// Abort after 5 seconds
setTimeout(() => controller.abort(), 5000);

try {
  const response = await chatPromise;
} catch (error) {
  if (error.code === 'ABORTED') {
    console.log('Operation was aborted');
  }
}
```

## Examples

### Basic Chat

```typescript
import { createAICoreBridge } from '@aetheris/ai-bridge';

const bridge = createAICoreBridge();
await bridge.initialize();

const response = await bridge.chat('What is the capital of France?');
console.log(response.content); // "The capital of France is Paris."
```

### Streaming Chat

```typescript
const bridge = createAICoreBridge();
await bridge.initialize();

console.log('AI: ');
for await (const chunk of bridge.chatStream('Tell me a story')) {
  process.stdout.write(chunk.chunk);
  if (chunk.isComplete) {
    console.log('\n[Complete]');
    break;
  }
}
```

### Tool Calling

```typescript
const bridge = createAICoreBridge();
await bridge.initialize();

// List available tools
const tools = await bridge.listTools();
console.log('Available tools:', tools.map(t => t.name));

// Call a tool
const result = await bridge.tool('open_file', { 
  path: '/tmp/test.txt',
  mode: 'read' 
});

if (result.success) {
  console.log('File content:', result.result);
} else {
  console.error('Tool error:', result.errorMessage);
}
```

### CapToken Authentication

```typescript
const bridge = createAICoreBridge();
await bridge.initialize();

// Set CapToken
bridge.setCaps({
  tokenId: 'my-token',
  token: 'token-value',
  scopes: ['ai:chat', 'ai:tool.open_file'],
  expiresAt: Date.now() + 3600000,
  issuer: 'devctl',
  subject: 'user',
  audience: 'ai-core-service',
  createdAt: Date.now()
});

// Use authenticated operations
const response = await bridge.chat('Hello with authentication');
```

### Metrics Monitoring

```typescript
const bridge = createAICoreBridge();
await bridge.initialize();

// Get current metrics
const metrics = await bridge.getMetrics();
console.log('AI Core Service Metrics:');
console.log(`  Requests: ${metrics.requests.total} (${metrics.requests.success} success, ${metrics.requests.error} errors)`);
console.log(`  Latency P95: ${metrics.latency.p95}ms`);
console.log(`  Tokens/sec: ${metrics.tokens.perSecond}`);
console.log(`  CPU Usage: ${metrics.system.cpuUsagePercent}%`);
console.log(`  Memory Usage: ${metrics.system.memoryUsageMB}MB`);
```

### Error Handling with Retries

```typescript
const bridge = createAICoreBridge({
  maxRetries: 5,
  initialRetryDelay: 1000,
  backoffMultiplier: 2
});

try {
  await bridge.initialize();
  const response = await bridge.chat('Hello');
} catch (error) {
  if (error instanceof AIError) {
    console.error(`Error ${error.code}: ${error.message}`);
    if (error.retryable) {
      console.log('This error will be retried automatically');
    }
  }
}
```

## Testing

Run the test suite:

```bash
npm test
```

Run tests with coverage:

```bash
npm run test:coverage
```

Run tests in watch mode:

```bash
npm run test:watch
```

## Development

### Building

```bash
npm run build
```

### Linting

```bash
npm run lint
npm run lint:fix
```

### Type Checking

```bash
npm run type-check
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

For issues and questions:
- GitHub Issues: [https://github.com/aetheris-os/polymera-os/issues](https://github.com/aetheris-os/polymera-os/issues)
- Documentation: [https://docs.aetheris-os.com](https://docs.aetheris-os.com)
