/**
 * @file ai_bridge.d.ts
 * @brief AI Core Service TypeScript Bridge - Type Definitions
 * 
 * This file provides comprehensive TypeScript type definitions for the AI Core Service bridge,
 * ensuring strict typing and excellent developer experience.
 */

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
export interface AIError {
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
export declare class AICoreBridge extends EventEmitter {
  constructor(config?: AICoreConfig);

  /**
   * Initialize the AI Core Service connection
   */
  initialize(): Promise<void>;

  /**
   * Set CapToken for authentication
   */
  setCaps(token: CapToken): void;

  /**
   * Get current CapToken
   */
  getCaps(): CapToken | undefined;

  /**
   * Clear CapToken
   */
  clearCaps(): void;

  /**
   * Chat with the AI Core Service
   */
  chat(
    message: string,
    config?: ChatConfig,
    abortSignal?: AbortSignal
  ): Promise<ChatResponse>;

  /**
   * Stream chat with the AI Core Service
   */
  chatStream(
    message: string,
    config?: ChatConfig,
    abortSignal?: AbortSignal
  ): AsyncGenerator<StreamingChunk, void, unknown>;

  /**
   * Call a tool through the AI Core Service
   */
  tool(
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
  ): Promise<ToolCallResult>;

  /**
   * List available tools
   */
  listTools(abortSignal?: AbortSignal): Promise<ToolDefinition[]>;

  /**
   * Get tool information
   */
  getToolInfo(toolName: string, abortSignal?: AbortSignal): Promise<ToolDefinition>;

  /**
   * Get AI Core Service metrics
   */
  getMetrics(abortSignal?: AbortSignal): Promise<AICoreMetrics>;

  /**
   * Ping the AI Core Service
   */
  ping(abortSignal?: AbortSignal): Promise<{ serverId: string; version: string; serverTime: number }>;

  /**
   * Check if connected
   */
  isServiceConnected(): boolean;

  /**
   * Disconnect from AI Core Service
   */
  disconnect(): Promise<void>;
}

// ============================================================================
// Factory Functions
// ============================================================================

/**
 * Create AI Core Service bridge instance
 */
export declare function createAICoreBridge(config?: AICoreConfig): AICoreBridge;

/**
 * Default AI Core Service bridge instance
 */
export declare const aiCoreBridge: AICoreBridge;

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

// ============================================================================
// Type Exports
// ============================================================================

export type {
  AICoreConfig,
  ChatConfig,
  ChatMessage,
  ChatResponse,
  StreamingChunk,
  ToolCall,
  ToolCallResult,
  ToolDefinition,
  CapToken,
  AICoreMetrics,
  AIError,
  RetryPolicy,
  AbortSignal,
};
