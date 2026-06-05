// AI Core Service Protocol Buffers
// Generated from ai_core.proto
// This file contains TypeScript definitions for the AI Core Service protobuf messages

export interface AiCoreMessage {
  messageType?: {
    pingRequest?: PingRequest;
    pingResponse?: PingResponse;
    chatRequest?: ChatRequest;
    chatResponse?: ChatResponse;
    toolCallRequest?: ToolCallRequest;
    toolCallResponse?: ToolCallResponse;
    functionResult?: FunctionResult;
    errorEnvelope?: ErrorEnvelope;
    errorResponse?: ErrorResponse; // Legacy
  };
  messageId: string;
  timestamp: number;
  sessionId: string;
  capToken?: CapToken;
}

export interface CapToken {
  tokenId: string;
  capability: string;
  expiresAt: number;
  signature: Uint8Array;
  issuer: string;
}

export interface PingRequest {
  clientId: string;
  version: string;
}

export interface PingResponse {
  serverId: string;
  version: string;
  serverTime: number;
  status?: ServiceStatus;
}

export interface ChatRequest {
  prompt: string;
  config?: ChatConfig;
  context: MessageContext[];
  stream: boolean;
  conversationId?: string;
  userId?: string;
  metadata: { [key: string]: string };
  attachments: string[];
  language?: string;
  enableFunctionCalling: boolean;
  allowedFunctions: string[];
}

export interface ChatResponse {
  response: string;
  isComplete: boolean;
  metrics?: ChatMetrics;
  toolCalls: ToolCall[];
  conversationId?: string;
  responseId?: string;
  functionResults: FunctionResult[];
  metadata: { [key: string]: string };
  modelVersion?: string;
  usageTokens: number;
  confidenceScore: number;
  citations: string[];
}

export interface ToolCallRequest {
  toolName: string;
  parameters: { [key: string]: string };
  context: string;
  cborPayload?: Uint8Array;
  callId?: string;
  sessionId?: string;
  metadata: { [key: string]: string };
  timeoutSeconds: number;
  async: boolean;
}

export interface ToolCallResponse {
  result: string;
  success: boolean;
  errorMessage: string;
  metrics?: ToolMetrics;
  cborPayload?: Uint8Array;
  callId?: string;
  toolVersion?: string;
  metadata: { [key: string]: string };
  warnings: string[];
  exitCode: number;
}

export interface FunctionResult {
  functionName: string;
  callId: string;
  success: boolean;
  result: string;
  cborPayload?: Uint8Array;
  errorMessage: string;
  exitCode: number;
  metadata: { [key: string]: string };
  executionTimeMs: number;
  memoryUsedMb: number;
}

export interface ErrorEnvelope {
  code: ErrorCode;
  message: string;
  details: string;
  timestamp: number;
  errorId?: string;
  component?: string;
  operation?: string;
  context: { [key: string]: string };
  stackTrace: string[];
  suggestion?: string;
  retryable: boolean;
  retryAfterSeconds: number;
}

export interface ErrorResponse {
  code: ErrorCode;
  message: string;
  details: string;
  timestamp: number;
}

export interface ChatConfig {
  model: string;
  temperature: number;
  maxTokens: number;
  topP: number;
  topK: number;
  enableTools: boolean;
  allowedTools: string[];
}

export interface MessageContext {
  role: string;
  content: string;
  timestamp: number;
  metadata: { [key: string]: string };
}

export interface ToolCall {
  toolName: string;
  parameters: { [key: string]: string };
  callId: string;
  cborPayload?: Uint8Array;
  functionName?: string;
  metadata: { [key: string]: string };
  timeoutSeconds: number;
  async: boolean;
}

export interface ChatMetrics {
  processingTimeMs: number;
  tokensGenerated: number;
  tokensInput: number;
  confidence: number;
  modelUsed: string;
}

export interface ToolMetrics {
  executionTimeMs: number;
  memoryUsedMb: number;
  cacheHit: boolean;
  toolVersion: string;
}

export interface ServiceStatus {
  state: ServiceState;
  activeSessions: number;
  uptimeSeconds: number;
  systemMetrics?: SystemMetrics;
}

export interface SystemMetrics {
  cpuUsagePercent: number;
  memoryUsedMb: number;
  memoryTotalMb: number;
  activeConnections: number;
  requestsProcessed: number;
}

export enum ErrorCode {
  UNKNOWN_ERROR = 0,
  INVALID_REQUEST = 1,
  UNAUTHORIZED = 2,
  CAPABILITY_DENIED = 3,
  MODEL_NOT_AVAILABLE = 4,
  TOOL_NOT_FOUND = 5,
  TOOL_EXECUTION_FAILED = 6,
  RATE_LIMITED = 7,
  INTERNAL_ERROR = 8,
  TIMEOUT = 9,
  VALIDATION_ERROR = 10,
  NOT_FOUND = 11,
  ALREADY_EXISTS = 12,
  INVALID_ARGUMENT = 13,
  UNSUPPORTED_OPERATION = 14,
  SERVICE_UNAVAILABLE = 15,
  EXTERNAL_SERVICE_ERROR = 16,
  NETWORK_ERROR = 17,
  SERIALIZATION_ERROR = 18,
  DESERIALIZATION_ERROR = 19,
  CONFIGURATION_ERROR = 20,
  RESOURCE_EXHAUSTED = 21,
  CONCURRENT_MODIFICATION = 22,
  DEPENDENCY_FAILURE = 23,
  AUTHENTICATION_FAILED = 24,
  AUTHORIZATION_FAILED = 25,
  TOKEN_EXPIRED = 26,
  TOKEN_INVALID = 27,
  SESSION_EXPIRED = 28,
  SESSION_INVALID = 29,
  QUOTA_EXCEEDED = 30,
  BANDWIDTH_EXCEEDED = 31,
  STORAGE_FULL = 32,
  FILE_NOT_FOUND = 33,
  PERMISSION_DENIED = 34,
  INVALID_FORMAT = 35,
  CORRUPTED_DATA = 36,
  VERSION_MISMATCH = 37,
  FEATURE_NOT_AVAILABLE = 38,
  MAINTENANCE_MODE = 39,
  UPGRADE_REQUIRED = 40
}

export enum ServiceState {
  STARTING = 0,
  RUNNING = 1,
  STOPPING = 2,
  STOPPED = 3,
  ERROR = 4
}

// Utility functions for working with CBOR payloads
export class CborUtils {
  /**
   * Encode a JavaScript object to CBOR bytes
   */
  static encode(obj: any): Uint8Array {
    // This would use a CBOR library in a real implementation
    // For now, we'll use JSON as a placeholder
    const json = JSON.stringify(obj);
    return new TextEncoder().encode(json);
  }

  /**
   * Decode CBOR bytes to a JavaScript object
   */
  static decode(bytes: Uint8Array): any {
    // This would use a CBOR library in a real implementation
    // For now, we'll use JSON as a placeholder
    const json = new TextDecoder().decode(bytes);
    return JSON.parse(json);
  }
}

// Message builders for convenience
export class MessageBuilder {
  static createPingRequest(clientId: string, version: string): AiCoreMessage {
    return {
      messageType: {
        pingRequest: {
          clientId,
          version
        }
      },
      messageId: crypto.randomUUID(),
      timestamp: Date.now(),
      sessionId: crypto.randomUUID()
    };
  }

  static createChatRequest(
    prompt: string,
    config?: ChatConfig,
    conversationId?: string
  ): AiCoreMessage {
    return {
      messageType: {
        chatRequest: {
          prompt,
          config,
          context: [],
          stream: false,
          conversationId,
          metadata: {},
          attachments: [],
          enableFunctionCalling: false,
          allowedFunctions: []
        }
      },
      messageId: crypto.randomUUID(),
      timestamp: Date.now(),
      sessionId: crypto.randomUUID()
    };
  }

  static createToolCallRequest(
    toolName: string,
    parameters: { [key: string]: string },
    context?: string
  ): AiCoreMessage {
    return {
      messageType: {
        toolCallRequest: {
          toolName,
          parameters,
          context: context || '',
          metadata: {},
          timeoutSeconds: 30,
          async: false
        }
      },
      messageId: crypto.randomUUID(),
      timestamp: Date.now(),
      sessionId: crypto.randomUUID()
    };
  }

  static createErrorEnvelope(
    code: ErrorCode,
    message: string,
    details?: string
  ): AiCoreMessage {
    return {
      messageType: {
        errorEnvelope: {
          code,
          message,
          details: details || '',
          timestamp: Date.now(),
          context: {},
          stackTrace: [],
          retryable: false,
          retryAfterSeconds: 0
        }
      },
      messageId: crypto.randomUUID(),
      timestamp: Date.now(),
      sessionId: crypto.randomUUID()
    };
  }
}
