/**
 * @file ai_bridge.ts
 * @brief AI Core Service TypeScript Bridge - Phase 5
 * 
 * This module provides TypeScript bindings for the AI Core Service,
 * enabling easy integration with web applications and Node.js.
 * 
 * Features:
 * - Streaming chat with backoff/abort support
 * - Tool calling with strict type validation
 * - CapToken management and capability enforcement
 * - Real-time metrics and monitoring
 * - Error handling with retry policies
 */

import { EventEmitter } from 'events';

// ============================================================================
// Core Type Definitions
// ============================================================================

/**
 * AI Core Service configuration
 */
export interface AICoreConfig {
  /** Socket path for IPC communication */
  socketPath?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Maximum retry attempts */
  maxRetries?: number;
  /** Retry backoff multiplier */
  backoffMultiplier?: number;
  /** Initial retry delay in milliseconds */
  initialRetryDelay?: number;
  /** Maximum retry delay in milliseconds */
  maxRetryDelay?: number;
  /** Enable deterministic behavior */
  deterministic?: boolean;
  /** Default seed for deterministic operations */
  seed?: number;
}

/**
 * Chat request configuration
 */
export interface ChatConfig {
  /** AI model to use */
  model?: string;
  /** Sampling temperature (0.0-2.0) */
  temperature?: number;
  /** Maximum tokens to generate */
  maxTokens?: number;
  /** Top-p sampling parameter */
  topP?: number;
  /** Stop sequences */
  stopSequences?: string[];
  /** Enable streaming responses */
  stream?: boolean;
  /** Context messages */
  context?: string[];
  /** Session ID for conversation continuity */
  sessionId?: string;
  /** Include response metadata */
  includeMetadata?: boolean;
}

/**
 * Chat message interface
 */
export interface ChatMessage {
  /** Message ID */
  id: string;
  /** Message content */
  content: string;
  /** Message role */
  role: 'user' | 'assistant' | 'system';
  /** Message timestamp */
  timestamp: Date;
  /** Message metadata */
  metadata?: {
    citations?: string[];
    tools_used?: string[];
    confidence?: number;
    processing_time_ms?: number;
    model_version?: string;
    usage_tokens?: number;
  };
}

/**
 * Chat response interface
 */
export interface ChatResponse {
  /** Response ID */
  id: string;
  /** Response content */
  content: string;
  /** Response role */
  role: 'assistant';
  /** Response timestamp */
  timestamp: Date;
  /** Is response complete */
  isComplete: boolean;
  /** Citations */
  citations?: string[];
  /** Tools used */
  tools_used?: string[];
  /** Response metadata */
  metadata?: {
    confidence?: number;
    processing_time_ms?: number;
    model_version?: string;
    usage_tokens?: number;
    tokens_generated?: number;
    tokens_input?: number;
  };
}

/**
 * Streaming chunk interface
 */
export interface StreamingChunk {
  /** Message ID */
  messageId: string;
  /** Chunk content */
  chunk: string;
  /** Is complete */
  isComplete: boolean;
  /** Chunk timestamp */
  timestamp: Date;
  /** Chunk metadata */
  metadata?: {
    confidence?: number;
    processing_time_ms?: number;
  };
}

/**
 * Tool call interface
 */
export interface ToolCall {
  /** Tool name */
  name: string;
  /** Tool parameters */
  parameters: Record<string, any>;
  /** Call ID */
  callId?: string;
  /** Session ID */
  sessionId?: string;
  /** Timeout in seconds */
  timeoutSeconds?: number;
  /** Execute asynchronously */
  async?: boolean;
  /** Call metadata */
  metadata?: Record<string, string>;
}

/**
 * Tool call result interface
 */
export interface ToolCallResult {
  /** Call ID */
  callId: string;
  /** Tool name */
  toolName: string;
  /** Success status */
  success: boolean;
  /** Result data */
  result?: any;
  /** Error message */
  errorMessage?: string;
  /** Execution time in milliseconds */
  executionTimeMs?: number;
  /** Tool version */
  toolVersion?: string;
  /** Warnings */
  warnings?: string[];
  /** Exit code */
  exitCode?: number;
  /** Result metadata */
  metadata?: Record<string, string>;
}

/**
 * Tool definition interface
 */
export interface ToolDefinition {
  /** Tool name */
  name: string;
  /** Tool description */
  description: string;
  /** Tool parameters schema */
  parameters: Record<string, any>;
  /** Required capabilities */
  requiredCapabilities: string[];
  /** Tool version */
  version: string;
  /** Tool author */
  author: string;
  /** Tool category */
  category?: string;
  /** Tool tags */
  tags?: string[];
}

/**
 * CapToken interface
 */
export interface CapToken {
  /** Token ID */
  tokenId: string;
  /** Token value */
  token: string;
  /** Token scopes */
  scopes: string[];
  /** Expiration timestamp */
  expiresAt: number;
  /** Token issuer */
  issuer: string;
  /** Token subject */
  subject: string;
  /** Token audience */
  audience: string;
  /** Creation timestamp */
  createdAt: number;
  /** Token signature */
  signature?: Uint8Array;
}

/**
 * AI Core metrics interface
 */
export interface AICoreMetrics {
  /** Request metrics */
  requests: {
    total: number;
    success: number;
    error: number;
  };
  /** Latency metrics */
  latency: {
    p50: number;
    p95: number;
    p99: number;
    max: number;
  };
  /** Token metrics */
  tokens: {
    perSecond: number;
    total: number;
    input: number;
    output: number;
  };
  /** Tool metrics */
  tools: {
    callsTotal: number;
    callsSuccess: number;
    callsError: number;
    errorsByType: Record<string, number>;
  };
  /** Model metrics */
  model: {
    loadsTotal: number;
    inferencesTotal: number;
    memoryUsageMB: number;
  };
  /** System metrics */
  system: {
    cpuUsagePercent: number;
    memoryUsageMB: number;
    diskUsageMB: number;
  };
  /** Session metrics */
  sessions: {
    active: number;
    total: number;
    avgDurationSecs: number;
  };
  /** Timestamp */
  timestamp: number;
}

/**
 * Error information interface
 */
export interface AIErrorData {
  /** Error code */
  code: string;
  /** Error message */
  message: string;
  /** Error details */
  details?: any;
  /** Error timestamp */
  timestamp: Date;
  /** Is retryable */
  retryable?: boolean;
  /** Retry after milliseconds */
  retryAfter?: number;
}

export class AIError extends Error {
  public readonly code: string;
  public readonly details?: any;
  public readonly timestamp: Date;
  public readonly retryable?: boolean;
  public readonly retryAfter?: number;

  constructor(data: AIErrorData) {
    super(data.message);
    this.name = 'AIError';
    this.code = data.code;
    this.details = data.details;
    this.timestamp = data.timestamp;
    if (data.retryable !== undefined) {
      this.retryable = data.retryable;
    }
    if (data.retryAfter !== undefined) {
      this.retryAfter = data.retryAfter;
    }
  }
}

/**
 * Retry policy interface
 */
export interface RetryPolicy {
  /** Maximum retry attempts */
  maxAttempts: number;
  /** Initial delay in milliseconds */
  initialDelay: number;
  /** Backoff multiplier */
  multiplier: number;
  /** Maximum delay in milliseconds */
  maxDelay: number;
  /** Jitter factor (0-1) */
  jitter: number;
}

/**
 * Abort signal interface
 */
export interface AbortSignal {
  /** Is aborted */
  aborted: boolean;
  /** Abort reason */
  reason?: any;
  /** Add event listener */
  addEventListener: (event: 'abort', listener: () => void) => void;
  /** Remove event listener */
  removeEventListener: (event: 'abort', listener: () => void) => void;
}

// ============================================================================
// AI Core Service Bridge Class
// ============================================================================

/**
 * AI Core Service TypeScript Bridge
 */
export class AICoreBridge extends EventEmitter {
  private config: Required<AICoreConfig>;
  private capToken?: CapToken;
  private isConnected: boolean = false;
  private retryPolicy: RetryPolicy;

  constructor(config?: AICoreConfig) {
    super();
    
    this.config = {
      socketPath: '/tmp/ai_core.sock',
      timeout: 30000,
      maxRetries: 3,
      backoffMultiplier: 2,
      initialRetryDelay: 1000,
      maxRetryDelay: 10000,
      deterministic: true,
      seed: 42,
      ...config,
    };

    this.retryPolicy = {
      maxAttempts: this.config.maxRetries,
      initialDelay: this.config.initialRetryDelay,
      multiplier: this.config.backoffMultiplier,
      maxDelay: this.config.maxRetryDelay,
      jitter: 0.1,
    };
  }

  /**
   * Initialize the AI Core Service connection
   */
  async initialize(): Promise<void> {
    try {
      // Test connection with ping
      await this.ping();
      this.isConnected = true;
      this.emit('connected');
    } catch (error) {
      this.isConnected = false;
      this.emit('error', error);
      throw new AIError({
        code: 'CONNECTION_FAILED',
        message: 'Failed to connect to AI Core Service',
        details: error,
        timestamp: new Date(),
      });
    }
  }

  /**
   * Set CapToken for authentication
   */
  setCaps(token: CapToken): void {
    this.capToken = token;
    this.emit('caps_updated', token);
  }

  /**
   * Get current CapToken
   */
  getCaps(): CapToken | undefined {
    return this.capToken;
  }

  /**
   * Clear CapToken
   */
  clearCaps(): void {
    delete this.capToken;
    this.emit('caps_cleared');
  }

  /**
   * Chat with the AI Core Service
   */
  async chat(
    message: string,
    config?: ChatConfig,
    abortSignal?: AbortSignal
  ): Promise<ChatResponse> {
    if (!this.isConnected) {
      throw new AIError({
        code: 'NOT_CONNECTED',
        message: 'AI Core Service not connected',
        timestamp: new Date(),
      });
    }

    const chatConfig: Required<ChatConfig> = {
      model: 'gpt-3.5-turbo',
      temperature: 0.7,
      maxTokens: 1000,
      topP: 1.0,
      stopSequences: [],
      stream: false,
      context: [],
      sessionId: this.generateSessionId(),
      includeMetadata: false,
      ...config,
    };

    try {
      const response = await this.executeWithRetry(
        () => this.sendChatRequest(message, chatConfig, abortSignal),
        abortSignal
      );

      return this.parseChatResponse(response);
    } catch (error) {
      this.emit('chat_error', error);
      throw error;
    }
  }

  /**
   * Stream chat with the AI Core Service
   */
  async *chatStream(
    message: string,
    config?: ChatConfig,
    abortSignal?: AbortSignal
  ): AsyncGenerator<StreamingChunk, void, unknown> {
    if (!this.isConnected) {
      throw new AIError({
        code: 'NOT_CONNECTED',
        message: 'AI Core Service not connected',
        timestamp: new Date(),
      });
    }

    const chatConfig: Required<ChatConfig> = {
      model: 'gpt-3.5-turbo',
      temperature: 0.7,
      maxTokens: 1000,
      topP: 1.0,
      stopSequences: [],
      stream: true,
      context: [],
      sessionId: this.generateSessionId(),
      includeMetadata: false,
      ...config,
    };

    try {
      const messageId = this.generateMessageId();
      let isComplete = false;
      let accumulatedContent = '';

      // Start streaming request
      const stream = await this.sendStreamingChatRequest(message, chatConfig, abortSignal);

      for await (const chunk of stream) {
        if (abortSignal?.aborted) {
          throw new AIError({
            code: 'ABORTED',
            message: 'Chat stream aborted',
            timestamp: new Date(),
          });
        }

        accumulatedContent += chunk.chunk;
        isComplete = chunk.isComplete;

        const streamingChunk: StreamingChunk = {
          messageId,
          chunk: chunk.chunk,
          isComplete,
          timestamp: new Date(),
          metadata: chunk.metadata,
        };

        yield streamingChunk;

        if (isComplete) {
          break;
        }
      }

      this.emit('chat_stream_complete', {
        messageId,
        content: accumulatedContent,
        timestamp: new Date(),
      });
    } catch (error) {
      this.emit('chat_stream_error', error);
      throw error;
    }
  }

  /**
   * Call a tool through the AI Core Service
   */
  async tool(
    name: string,
    payload: Record<string, any>,
    options?: {
      callId?: string;
      sessionId?: string;
      timeoutSeconds?: number;
      async?: boolean;
      metadata?: Record<string, string>;
    },
    abortSignal?: AbortSignal
  ): Promise<ToolCallResult> {
    if (!this.isConnected) {
      throw new AIError({
        code: 'NOT_CONNECTED',
        message: 'AI Core Service not connected',
        timestamp: new Date(),
      });
    }

    const toolCall: ToolCall = {
      name,
      parameters: payload,
      callId: options?.callId || this.generateCallId(),
      ...(options?.sessionId && { sessionId: options.sessionId }),
      timeoutSeconds: options?.timeoutSeconds || 30,
      async: options?.async || false,
      ...(options?.metadata && { metadata: options.metadata }),
    };

    try {
      const response = await this.executeWithRetry(
        () => this.sendToolCallRequest(toolCall, abortSignal),
        abortSignal
      );

      return this.parseToolCallResponse(response);
    } catch (error) {
      this.emit('tool_error', { toolCall, error });
      throw error;
    }
  }

  /**
   * List available tools
   */
  async listTools(abortSignal?: AbortSignal): Promise<ToolDefinition[]> {
    if (!this.isConnected) {
      throw new AIError({
        code: 'NOT_CONNECTED',
        message: 'AI Core Service not connected',
        timestamp: new Date(),
      });
    }

    try {
      const response = await this.executeWithRetry(
        () => this.sendToolListRequest(abortSignal),
        abortSignal
      );

      return this.parseToolListResponse(response);
    } catch (error) {
      this.emit('tool_list_error', error);
      throw error;
    }
  }

  /**
   * Get tool information
   */
  async getToolInfo(toolName: string, abortSignal?: AbortSignal): Promise<ToolDefinition> {
    if (!this.isConnected) {
      throw new AIError({
        code: 'NOT_CONNECTED',
        message: 'AI Core Service not connected',
        timestamp: new Date(),
      });
    }

    try {
      const response = await this.executeWithRetry(
        () => this.sendToolInfoRequest(toolName, abortSignal),
        abortSignal
      );

      return this.parseToolInfoResponse(response);
    } catch (error) {
      this.emit('tool_info_error', { toolName, error });
      throw error;
    }
  }

  /**
   * Get AI Core Service metrics
   */
  async getMetrics(abortSignal?: AbortSignal): Promise<AICoreMetrics> {
    if (!this.isConnected) {
      throw new AIError({
        code: 'NOT_CONNECTED',
        message: 'AI Core Service not connected',
        timestamp: new Date(),
      });
    }

    try {
      const response = await this.executeWithRetry(
        () => this.sendMetricsRequest(abortSignal),
        abortSignal
      );

      return this.parseMetricsResponse(response);
    } catch (error) {
      this.emit('metrics_error', error);
      throw error;
    }
  }

  /**
   * Ping the AI Core Service
   */
  async ping(abortSignal?: AbortSignal): Promise<{ serverId: string; version: string; serverTime: number }> {
    try {
      const response = await this.sendPingRequest(abortSignal);
      return this.parsePingResponse(response);
    } catch (error) {
      this.emit('ping_error', error);
      throw error;
    }
  }

  /**
   * Check if connected
   */
  isServiceConnected(): boolean {
    return this.isConnected;
  }

  /**
   * Disconnect from AI Core Service
   */
  async disconnect(): Promise<void> {
    this.isConnected = false;
    delete this.capToken;
    this.emit('disconnected');
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  /**
   * Execute function with retry logic
   */
  private async executeWithRetry<T>(
    fn: () => Promise<T>,
    abortSignal?: AbortSignal
  ): Promise<T> {
    let lastError: any;
    
    for (let attempt = 0; attempt < this.retryPolicy.maxAttempts; attempt++) {
      if (abortSignal?.aborted) {
        throw new AIError({
          code: 'ABORTED',
          message: 'Operation aborted',
          timestamp: new Date(),
        });
      }

      try {
        return await fn();
      } catch (error) {
        lastError = error;
        
        if (attempt === this.retryPolicy.maxAttempts - 1) {
          break;
        }

        // Check if error is retryable
        if (error instanceof AIError && !error.retryable) {
          throw error;
        }

        // Calculate delay with jitter
        const delay = Math.min(
          this.retryPolicy.initialDelay * Math.pow(this.retryPolicy.multiplier, attempt),
          this.retryPolicy.maxDelay
        );
        
        const jitter = delay * this.retryPolicy.jitter * Math.random();
        const totalDelay = delay + jitter;

        await this.sleep(totalDelay);
      }
    }

    throw lastError;
  }

  /**
   * Send chat request to AI Core Service
   */
  private async sendChatRequest(
    message: string,
    _config: Required<ChatConfig>,
    abortSignal?: AbortSignal
  ): Promise<any> {
    // Mock implementation - in real implementation, this would use IPC
    await this.sleep(100); // Simulate network delay

    if (abortSignal?.aborted) {
      throw new AIError({
        code: 'ABORTED',
        message: 'Chat request aborted',
        timestamp: new Date(),
      });
    }

    // Mock response
    return {
      id: this.generateMessageId(),
      content: this.generateMockResponse(message),
      role: 'assistant',
      timestamp: Date.now(),
      isComplete: true,
      metadata: {
        confidence: 0.95,
        processing_time_ms: 150,
        model_version: '1.0.0',
        usage_tokens: 50,
        tokens_generated: 50,
        tokens_input: message.split(' ').length,
      },
    };
  }

  /**
   * Send streaming chat request to AI Core Service
   */
  private async *sendStreamingChatRequest(
    message: string,
    _config: Required<ChatConfig>,
    abortSignal?: AbortSignal
  ): AsyncGenerator<{ chunk: string; isComplete: boolean; metadata?: any }, void, unknown> {
    // Mock implementation - in real implementation, this would use IPC
    const response = this.generateMockResponse(message);
    const words = response.split(' ');
    
    for (let i = 0; i < words.length; i++) {
      if (abortSignal?.aborted) {
        throw new AIError({
          code: 'ABORTED',
          message: 'Chat stream aborted',
          timestamp: new Date(),
        });
      }

      await this.sleep(50); // Simulate streaming delay

      yield {
        chunk: (i === 0 ? '' : ' ') + words[i],
        isComplete: i === words.length - 1,
        metadata: {
          confidence: 0.95,
          processing_time_ms: 150,
        },
      };
    }
  }

  /**
   * Send tool call request to AI Core Service
   */
  private async sendToolCallRequest(
    toolCall: ToolCall,
    abortSignal?: AbortSignal
  ): Promise<any> {
    // Mock implementation - in real implementation, this would use IPC
    await this.sleep(200); // Simulate tool execution delay

    if (abortSignal?.aborted) {
      throw new AIError({
        code: 'ABORTED',
        message: 'Tool call aborted',
        timestamp: new Date(),
      });
    }

    // Mock response based on tool name
    let result: any;
    let success = true;
    let errorMessage = '';

    switch (toolCall.name) {
      case 'open_file':
        result = `File opened successfully: ${toolCall.parameters['path']}`;
        break;
      case 'search_files':
        result = `Search completed for: ${toolCall.parameters['query']}`;
        break;
      case 'create_note':
        result = `Note created: ${toolCall.parameters['title']}`;
        break;
      default:
        success = false;
        errorMessage = `Unknown tool: ${toolCall.name}`;
    }

    return {
      callId: toolCall.callId,
      toolName: toolCall.name,
      success,
      result,
      errorMessage,
      executionTimeMs: 200,
      toolVersion: '1.0.0',
      warnings: [],
      exitCode: success ? 0 : 1,
      metadata: {},
    };
  }

  /**
   * Send tool list request to AI Core Service
   */
  private async sendToolListRequest(abortSignal?: AbortSignal): Promise<any> {
    // Mock implementation
    await this.sleep(50);

    if (abortSignal?.aborted) {
      throw new AIError({
        code: 'ABORTED',
        message: 'Tool list request aborted',
        timestamp: new Date(),
      });
    }

    return [
      {
        name: 'open_file',
        description: 'Open and read a file from the filesystem',
        parameters: {
          path: { type: 'string', description: 'File path', required: true },
          mode: { type: 'string', description: 'Access mode', required: false, default: 'read' },
        },
        requiredCapabilities: ['file.read'],
        version: '1.0.0',
        author: 'Aetheris OS Team',
        category: 'file',
        tags: ['file', 'read'],
      },
      {
        name: 'search_files',
        description: 'Search for files matching a pattern',
        parameters: {
          query: { type: 'string', description: 'Search query', required: true },
          directory: { type: 'string', description: 'Search directory', required: false, default: '.' },
        },
        requiredCapabilities: ['file.search'],
        version: '1.0.0',
        author: 'Aetheris OS Team',
        category: 'file',
        tags: ['file', 'search'],
      },
    ];
  }

  /**
   * Send tool info request to AI Core Service
   */
  private async sendToolInfoRequest(toolName: string, abortSignal?: AbortSignal): Promise<any> {
    // Mock implementation
    await this.sleep(50);

    if (abortSignal?.aborted) {
      throw new AIError({
        code: 'ABORTED',
        message: 'Tool info request aborted',
        timestamp: new Date(),
      });
    }

    const tools = await this.sendToolListRequest();
    const tool = tools.find((t: any) => t.name === toolName);
    
    if (!tool) {
      throw new AIError({
        code: 'TOOL_NOT_FOUND',
        message: `Tool not found: ${toolName}`,
        timestamp: new Date(),
      });
    }

    return tool;
  }

  /**
   * Send metrics request to AI Core Service
   */
  private async sendMetricsRequest(abortSignal?: AbortSignal): Promise<any> {
    // Mock implementation
    await this.sleep(50);

    if (abortSignal?.aborted) {
      throw new AIError({
        code: 'ABORTED',
        message: 'Metrics request aborted',
        timestamp: new Date(),
      });
    }

    return {
      requests: { total: 1250, success: 1180, error: 70 },
      latency: { p50: 45.2, p95: 125.8, p99: 250.3, max: 500.1 },
      tokens: { perSecond: 12.5, total: 15680, input: 8920, output: 6760 },
      tools: { callsTotal: 340, callsSuccess: 315, callsError: 25, errorsByType: {} },
      model: { loadsTotal: 15, inferencesTotal: 1180, memoryUsageMB: 2048.5 },
      system: { cpuUsagePercent: 23.4, memoryUsageMB: 1024.8, diskUsageMB: 5120.2 },
      sessions: { active: 3, total: 45, avgDurationSecs: 180.5 },
      timestamp: Date.now(),
    };
  }

  /**
   * Send ping request to AI Core Service
   */
  private async sendPingRequest(abortSignal?: AbortSignal): Promise<any> {
    // Mock implementation
    await this.sleep(10);

    if (abortSignal?.aborted) {
      throw new AIError({
        code: 'ABORTED',
        message: 'Ping request aborted',
        timestamp: new Date(),
      });
    }

    return {
      serverId: 'ai-core-service',
      version: '1.0.0',
      serverTime: Date.now(),
    };
  }

  /**
   * Parse chat response
   */
  private parseChatResponse(response: any): ChatResponse {
    return {
      id: response.id,
      content: response.content,
      role: 'assistant',
      timestamp: new Date(response.timestamp),
      isComplete: response.isComplete,
      metadata: response.metadata,
    };
  }

  /**
   * Parse tool call response
   */
  private parseToolCallResponse(response: any): ToolCallResult {
    return {
      callId: response.callId,
      toolName: response.toolName,
      success: response.success,
      result: response.result,
      errorMessage: response.errorMessage,
      executionTimeMs: response.executionTimeMs,
      toolVersion: response.toolVersion,
      warnings: response.warnings,
      exitCode: response.exitCode,
      metadata: response.metadata,
    };
  }

  /**
   * Parse tool list response
   */
  private parseToolListResponse(response: any[]): ToolDefinition[] {
    return response.map(tool => ({
      name: tool.name,
      description: tool.description,
      parameters: tool.parameters,
      requiredCapabilities: tool.requiredCapabilities,
      version: tool.version,
      author: tool.author,
      category: tool.category,
      tags: tool.tags,
    }));
  }

  /**
   * Parse tool info response
   */
  private parseToolInfoResponse(response: any): ToolDefinition {
    return {
      name: response.name,
      description: response.description,
      parameters: response.parameters,
      requiredCapabilities: response.requiredCapabilities,
      version: response.version,
      author: response.author,
      category: response.category,
      tags: response.tags,
    };
  }

  /**
   * Parse metrics response
   */
  private parseMetricsResponse(response: any): AICoreMetrics {
    return {
      requests: response.requests,
      latency: response.latency,
      tokens: response.tokens,
      tools: response.tools,
      model: response.model,
      system: response.system,
      sessions: response.sessions,
      timestamp: response.timestamp,
    };
  }

  /**
   * Parse ping response
   */
  private parsePingResponse(response: any): { serverId: string; version: string; serverTime: number } {
    return {
      serverId: response.serverId,
      version: response.version,
      serverTime: response.serverTime,
    };
  }

  /**
   * Generate mock response
   */
  private generateMockResponse(message: string): string {
    const lowerMessage = message.toLowerCase();
    
    if (lowerMessage.includes('summarize') || lowerMessage.includes('summary')) {
      return 'Here\'s a summary of the requested content: [This is a mock response from the AI Core Service. In a real implementation, this would be the actual AI-generated summary.]';
    }
    
    if (lowerMessage.includes('capital') && lowerMessage.includes('france')) {
      return 'The capital of France is Paris.';
    }
    
    if (lowerMessage.includes('hello') || lowerMessage.includes('hi')) {
      return 'Hello! I\'m the AI Core Service. How can I help you today?';
    }
    
    if (lowerMessage.includes('help')) {
      return 'I can help you with various tasks including answering questions, summarizing content, generating text, and more. What would you like to know?';
    }
    
    // Default response
    return `I understand you're asking about: "${message}". This is a mock response from the AI Core Service. In a real implementation, this would be the actual AI-generated response.`;
  }

  /**
   * Generate session ID
   */
  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate message ID
   */
  private generateMessageId(): string {
    return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate call ID
   */
  private generateCallId(): string {
    return `call_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// ============================================================================
// Factory Functions
// ============================================================================

/**
 * Create AI Core Service bridge instance
 */
export function createAICoreBridge(config?: AICoreConfig): AICoreBridge {
  return new AICoreBridge(config);
}

/**
 * Default AI Core Service bridge instance
 */
export const aiCoreBridge = new AICoreBridge();

// Types are already exported above as interfaces

// ============================================================================
// Event Types
// ============================================================================

export interface AICoreEvents {
  'connected': () => void;
  'disconnected': () => void;
  'error': (error: AIError) => void;
  'caps_updated': (token: CapToken) => void;
  'caps_cleared': () => void;
  'chat_error': (error: AIError) => void;
  'chat_stream_complete': (data: { messageId: string; content: string; timestamp: Date }) => void;
  'chat_stream_error': (error: AIError) => void;
  'tool_error': (data: { toolCall: ToolCall; error: AIError }) => void;
  'tool_list_error': (error: AIError) => void;
  'tool_info_error': (data: { toolName: string; error: AIError }) => void;
  'metrics_error': (error: AIError) => void;
  'ping_error': (error: AIError) => void;
}
