/**
 * @file ai_bridge.test.ts
 * @brief AI Core Service TypeScript Bridge - Jest Tests
 * 
 * This file provides comprehensive Jest tests for the AI Core Service bridge,
 * covering all functionality including streaming, tool calls, and error handling.
 */

import { AICoreBridge, createAICoreBridge, aiCoreBridge } from './ai_bridge';
import type {
  AICoreConfig,
  ChatConfig,
  ChatResponse,
  StreamingChunk,
  ToolCallResult,
  ToolDefinition,
  CapToken,
  AICoreMetrics,
  AIError,
  AbortSignal,
} from './ai_bridge';

// Mock EventEmitter for testing
jest.mock('events');

describe('AICoreBridge', () => {
  let bridge: AICoreBridge;
  let mockAbortSignal: AbortSignal;

  beforeEach(() => {
    bridge = new AICoreBridge();
    mockAbortSignal = {
      aborted: false,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
    };
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  // ============================================================================
  // Constructor and Configuration Tests
  // ============================================================================

  describe('Constructor', () => {
    it('should create bridge with default configuration', () => {
      const defaultBridge = new AICoreBridge();
      expect(defaultBridge).toBeInstanceOf(AICoreBridge);
    });

    it('should create bridge with custom configuration', () => {
      const config: AICoreConfig = {
        socketPath: '/custom/socket',
        timeout: 60000,
        maxRetries: 5,
        deterministic: false,
      };
      const customBridge = new AICoreBridge(config);
      expect(customBridge).toBeInstanceOf(AICoreBridge);
    });
  });

  // ============================================================================
  // Initialization Tests
  // ============================================================================

  describe('initialize', () => {
    it('should initialize successfully', async () => {
      await expect(bridge.initialize()).resolves.toBeUndefined();
      expect(bridge.isServiceConnected()).toBe(true);
    });

    it('should emit connected event on successful initialization', async () => {
      const connectedSpy = jest.fn();
      bridge.on('connected', connectedSpy);

      await bridge.initialize();
      expect(connectedSpy).toHaveBeenCalled();
    });

    it('should emit error event on failed initialization', async () => {
      // Mock ping to fail
      const originalPing = bridge.ping;
      bridge.ping = jest.fn().mockRejectedValue(new Error('Connection failed'));

      const errorSpy = jest.fn();
      bridge.on('error', errorSpy);

      await expect(bridge.initialize()).rejects.toThrow();
      expect(errorSpy).toHaveBeenCalled();
      expect(bridge.isServiceConnected()).toBe(false);

      // Restore original ping
      bridge.ping = originalPing;
    });
  });

  // ============================================================================
  // CapToken Management Tests
  // ============================================================================

  describe('CapToken Management', () => {
    const mockCapToken: CapToken = {
      tokenId: 'test-token-id',
      token: 'test-token-value',
      scopes: ['ai:chat', 'ai:tool.open_file'],
      expiresAt: Date.now() + 3600000,
      issuer: 'test-issuer',
      subject: 'test-subject',
      audience: 'ai-core-service',
      createdAt: Date.now(),
    };

    it('should set CapToken', () => {
      const capsUpdatedSpy = jest.fn();
      bridge.on('caps_updated', capsUpdatedSpy);

      bridge.setCaps(mockCapToken);
      expect(bridge.getCaps()).toEqual(mockCapToken);
      expect(capsUpdatedSpy).toHaveBeenCalledWith(mockCapToken);
    });

    it('should get current CapToken', () => {
      bridge.setCaps(mockCapToken);
      expect(bridge.getCaps()).toEqual(mockCapToken);
    });

    it('should clear CapToken', () => {
      bridge.setCaps(mockCapToken);
      const capsClearedSpy = jest.fn();
      bridge.on('caps_cleared', capsClearedSpy);

      bridge.clearCaps();
      expect(bridge.getCaps()).toBeUndefined();
      expect(capsClearedSpy).toHaveBeenCalled();
    });

    it('should return undefined when no CapToken is set', () => {
      expect(bridge.getCaps()).toBeUndefined();
    });
  });

  // ============================================================================
  // Chat Tests
  // ============================================================================

  describe('chat', () => {
    beforeEach(async () => {
      await bridge.initialize();
    });

    it('should send chat message and receive response', async () => {
      const message = 'Hello, AI!';
      const response = await bridge.chat(message);

      expect(response).toMatchObject({
        id: expect.any(String),
        content: expect.any(String),
        role: 'assistant',
        timestamp: expect.any(Date),
        isComplete: true,
      });
      expect(response.content).toContain('Hello! I\'m the AI Core Service');
    });

    it('should send chat with custom configuration', async () => {
      const message = 'What is the capital of France?';
      const config: ChatConfig = {
        model: 'gpt-4',
        temperature: 0.3,
        maxTokens: 500,
        includeMetadata: true,
      };

      const response = await bridge.chat(message, config);
      expect(response.content).toContain('Paris');
      expect(response.metadata).toBeDefined();
    });

    it('should throw error when not connected', async () => {
      await bridge.disconnect();
      await expect(bridge.chat('test')).rejects.toThrow('AI Core Service not connected');
    });

    it('should handle abort signal', async () => {
      mockAbortSignal.aborted = true;
      await expect(bridge.chat('test', undefined, mockAbortSignal)).rejects.toThrow('Operation aborted');
    });

    it('should emit chat_error event on error', async () => {
      const chatErrorSpy = jest.fn();
      bridge.on('chat_error', chatErrorSpy);

      // Mock internal method to throw error
      const originalSendChatRequest = (bridge as any).sendChatRequest;
      (bridge as any).sendChatRequest = jest.fn().mockRejectedValue(new Error('Chat failed'));

      await expect(bridge.chat('test')).rejects.toThrow();
      expect(chatErrorSpy).toHaveBeenCalled();

      // Restore original method
      (bridge as any).sendChatRequest = originalSendChatRequest;
    });
  });

  // ============================================================================
  // Streaming Chat Tests
  // ============================================================================

  describe('chatStream', () => {
    beforeEach(async () => {
      await bridge.initialize();
    });

    it('should stream chat response', async () => {
      const message = 'Hello, AI!';
      const chunks: StreamingChunk[] = [];

      for await (const chunk of bridge.chatStream(message)) {
        chunks.push(chunk);
      }

      expect(chunks.length).toBeGreaterThan(0);
      expect(chunks[0]).toMatchObject({
        messageId: expect.any(String),
        chunk: expect.any(String),
        isComplete: false,
        timestamp: expect.any(Date),
      });
      expect(chunks[chunks.length - 1].isComplete).toBe(true);
    });

    it('should stream with custom configuration', async () => {
      const message = 'What is the capital of France?';
      const config: ChatConfig = {
        model: 'gpt-4',
        temperature: 0.3,
        stream: true,
      };

      const chunks: StreamingChunk[] = [];
      for await (const chunk of bridge.chatStream(message, config)) {
        chunks.push(chunk);
      }

      expect(chunks.length).toBeGreaterThan(0);
      const fullContent = chunks.map(c => c.chunk).join('');
      expect(fullContent).toContain('Paris');
    });

    it('should emit chat_stream_complete event', async () => {
      const streamCompleteSpy = jest.fn();
      bridge.on('chat_stream_complete', streamCompleteSpy);

      const chunks: StreamingChunk[] = [];
      for await (const chunk of bridge.chatStream('test')) {
        chunks.push(chunk);
      }

      expect(streamCompleteSpy).toHaveBeenCalledWith({
        messageId: expect.any(String),
        content: expect.any(String),
        timestamp: expect.any(Date),
      });
    });

    it('should handle abort signal during streaming', async () => {
      mockAbortSignal.aborted = true;
      await expect(async () => {
        for await (const chunk of bridge.chatStream('test', undefined, mockAbortSignal)) {
          // This should not execute
        }
      }).rejects.toThrow('Chat stream aborted');
    });

    it('should emit chat_stream_error event on error', async () => {
      const streamErrorSpy = jest.fn();
      bridge.on('chat_stream_error', streamErrorSpy);

      // Mock internal method to throw error
      const originalSendStreamingChatRequest = (bridge as any).sendStreamingChatRequest;
      (bridge as any).sendStreamingChatRequest = jest.fn().mockImplementation(async function* () {
        throw new Error('Stream failed');
      });

      await expect(async () => {
        for await (const chunk of bridge.chatStream('test')) {
          // This should not execute
        }
      }).rejects.toThrow();
      expect(streamErrorSpy).toHaveBeenCalled();

      // Restore original method
      (bridge as any).sendStreamingChatRequest = originalSendStreamingChatRequest;
    });
  });

  // ============================================================================
  // Tool Call Tests
  // ============================================================================

  describe('tool', () => {
    beforeEach(async () => {
      await bridge.initialize();
    });

    it('should call tool successfully', async () => {
      const result = await bridge.tool('open_file', { path: '/tmp/test.txt' });

      expect(result).toMatchObject({
        callId: expect.any(String),
        toolName: 'open_file',
        success: true,
        result: expect.stringContaining('File opened successfully'),
        executionTimeMs: expect.any(Number),
      });
    });

    it('should call tool with options', async () => {
      const options = {
        callId: 'custom-call-id',
        sessionId: 'test-session',
        timeoutSeconds: 60,
        async: false,
        metadata: { source: 'test' },
      };

      const result = await bridge.tool('search_files', { query: '*.go' }, options);
      expect(result.callId).toBe('custom-call-id');
      expect(result.success).toBe(true);
    });

    it('should handle tool call errors', async () => {
      const result = await bridge.tool('unknown_tool', {});
      expect(result.success).toBe(false);
      expect(result.errorMessage).toContain('Unknown tool');
    });

    it('should emit tool_error event on error', async () => {
      const toolErrorSpy = jest.fn();
      bridge.on('tool_error', toolErrorSpy);

      // Mock internal method to throw error
      const originalSendToolCallRequest = (bridge as any).sendToolCallRequest;
      (bridge as any).sendToolCallRequest = jest.fn().mockRejectedValue(new Error('Tool call failed'));

      await expect(bridge.tool('test_tool', {})).rejects.toThrow();
      expect(toolErrorSpy).toHaveBeenCalledWith({
        toolCall: expect.objectContaining({ name: 'test_tool' }),
        error: expect.any(Error),
      });

      // Restore original method
      (bridge as any).sendToolCallRequest = originalSendToolCallRequest;
    });

    it('should handle abort signal', async () => {
      mockAbortSignal.aborted = true;
      await expect(bridge.tool('test_tool', {}, undefined, mockAbortSignal)).rejects.toThrow('Operation aborted');
    });
  });

  // ============================================================================
  // Tool Management Tests
  // ============================================================================

  describe('Tool Management', () => {
    beforeEach(async () => {
      await bridge.initialize();
    });

    it('should list available tools', async () => {
      const tools = await bridge.listTools();
      expect(Array.isArray(tools)).toBe(true);
      expect(tools.length).toBeGreaterThan(0);
      expect(tools[0]).toMatchObject({
        name: expect.any(String),
        description: expect.any(String),
        parameters: expect.any(Object),
        requiredCapabilities: expect.any(Array),
        version: expect.any(String),
        author: expect.any(String),
      });
    });

    it('should get tool information', async () => {
      const toolInfo = await bridge.getToolInfo('open_file');
      expect(toolInfo).toMatchObject({
        name: 'open_file',
        description: expect.any(String),
        parameters: expect.any(Object),
        requiredCapabilities: expect.any(Array),
        version: expect.any(String),
        author: expect.any(String),
      });
    });

    it('should throw error for unknown tool', async () => {
      await expect(bridge.getToolInfo('unknown_tool')).rejects.toThrow('Tool not found');
    });

    it('should emit tool_list_error event on error', async () => {
      const toolListErrorSpy = jest.fn();
      bridge.on('tool_list_error', toolListErrorSpy);

      // Mock internal method to throw error
      const originalSendToolListRequest = (bridge as any).sendToolListRequest;
      (bridge as any).sendToolListRequest = jest.fn().mockRejectedValue(new Error('Tool list failed'));

      await expect(bridge.listTools()).rejects.toThrow();
      expect(toolListErrorSpy).toHaveBeenCalled();

      // Restore original method
      (bridge as any).sendToolListRequest = originalSendToolListRequest;
    });

    it('should emit tool_info_error event on error', async () => {
      const toolInfoErrorSpy = jest.fn();
      bridge.on('tool_info_error', toolInfoErrorSpy);

      // Mock internal method to throw error
      const originalSendToolInfoRequest = (bridge as any).sendToolInfoRequest;
      (bridge as any).sendToolInfoRequest = jest.fn().mockRejectedValue(new Error('Tool info failed'));

      await expect(bridge.getToolInfo('test_tool')).rejects.toThrow();
      expect(toolInfoErrorSpy).toHaveBeenCalledWith({
        toolName: 'test_tool',
        error: expect.any(Error),
      });

      // Restore original method
      (bridge as any).sendToolInfoRequest = originalSendToolInfoRequest;
    });
  });

  // ============================================================================
  // Metrics Tests
  // ============================================================================

  describe('getMetrics', () => {
    beforeEach(async () => {
      await bridge.initialize();
    });

    it('should get AI Core metrics', async () => {
      const metrics = await bridge.getMetrics();
      expect(metrics).toMatchObject({
        requests: {
          total: expect.any(Number),
          success: expect.any(Number),
          error: expect.any(Number),
        },
        latency: {
          p50: expect.any(Number),
          p95: expect.any(Number),
          p99: expect.any(Number),
          max: expect.any(Number),
        },
        tokens: {
          perSecond: expect.any(Number),
          total: expect.any(Number),
          input: expect.any(Number),
          output: expect.any(Number),
        },
        tools: {
          callsTotal: expect.any(Number),
          callsSuccess: expect.any(Number),
          callsError: expect.any(Number),
          errorsByType: expect.any(Object),
        },
        model: {
          loadsTotal: expect.any(Number),
          inferencesTotal: expect.any(Number),
          memoryUsageMB: expect.any(Number),
        },
        system: {
          cpuUsagePercent: expect.any(Number),
          memoryUsageMB: expect.any(Number),
          diskUsageMB: expect.any(Number),
        },
        sessions: {
          active: expect.any(Number),
          total: expect.any(Number),
          avgDurationSecs: expect.any(Number),
        },
        timestamp: expect.any(Number),
      });
    });

    it('should emit metrics_error event on error', async () => {
      const metricsErrorSpy = jest.fn();
      bridge.on('metrics_error', metricsErrorSpy);

      // Mock internal method to throw error
      const originalSendMetricsRequest = (bridge as any).sendMetricsRequest;
      (bridge as any).sendMetricsRequest = jest.fn().mockRejectedValue(new Error('Metrics failed'));

      await expect(bridge.getMetrics()).rejects.toThrow();
      expect(metricsErrorSpy).toHaveBeenCalled();

      // Restore original method
      (bridge as any).sendMetricsRequest = originalSendMetricsRequest;
    });
  });

  // ============================================================================
  // Ping Tests
  // ============================================================================

  describe('ping', () => {
    it('should ping successfully', async () => {
      const result = await bridge.ping();
      expect(result).toMatchObject({
        serverId: expect.any(String),
        version: expect.any(String),
        serverTime: expect.any(Number),
      });
    });

    it('should emit ping_error event on error', async () => {
      const pingErrorSpy = jest.fn();
      bridge.on('ping_error', pingErrorSpy);

      // Mock internal method to throw error
      const originalSendPingRequest = (bridge as any).sendPingRequest;
      (bridge as any).sendPingRequest = jest.fn().mockRejectedValue(new Error('Ping failed'));

      await expect(bridge.ping()).rejects.toThrow();
      expect(pingErrorSpy).toHaveBeenCalled();

      // Restore original method
      (bridge as any).sendPingRequest = originalSendPingRequest;
    });
  });

  // ============================================================================
  // Connection Management Tests
  // ============================================================================

  describe('Connection Management', () => {
    it('should check connection status', () => {
      expect(bridge.isServiceConnected()).toBe(false);
    });

    it('should disconnect successfully', async () => {
      await bridge.initialize();
      expect(bridge.isServiceConnected()).toBe(true);

      const disconnectedSpy = jest.fn();
      bridge.on('disconnected', disconnectedSpy);

      await bridge.disconnect();
      expect(bridge.isServiceConnected()).toBe(false);
      expect(disconnectedSpy).toHaveBeenCalled();
    });
  });

  // ============================================================================
  // Retry Logic Tests
  // ============================================================================

  describe('Retry Logic', () => {
    beforeEach(async () => {
      await bridge.initialize();
    });

    it('should retry on retryable errors', async () => {
      let attemptCount = 0;
      const originalSendChatRequest = (bridge as any).sendChatRequest;
      (bridge as any).sendChatRequest = jest.fn().mockImplementation(() => {
        attemptCount++;
        if (attemptCount < 3) {
          const error = new Error('Retryable error');
          (error as any).retryable = true;
          throw error;
        }
        return { id: 'test', content: 'success', role: 'assistant', timestamp: Date.now(), isComplete: true };
      });

      const response = await bridge.chat('test');
      expect(attemptCount).toBe(3);
      expect(response.content).toBe('success');

      // Restore original method
      (bridge as any).sendChatRequest = originalSendChatRequest;
    });

    it('should not retry on non-retryable errors', async () => {
      let attemptCount = 0;
      const originalSendChatRequest = (bridge as any).sendChatRequest;
      (bridge as any).sendChatRequest = jest.fn().mockImplementation(() => {
        attemptCount++;
        const error = new Error('Non-retryable error');
        (error as any).retryable = false;
        throw error;
      });

      await expect(bridge.chat('test')).rejects.toThrow('Non-retryable error');
      expect(attemptCount).toBe(1);

      // Restore original method
      (bridge as any).sendChatRequest = originalSendChatRequest;
    });

    it('should respect max retry attempts', async () => {
      let attemptCount = 0;
      const originalSendChatRequest = (bridge as any).sendChatRequest;
      (bridge as any).sendChatRequest = jest.fn().mockImplementation(() => {
        attemptCount++;
        const error = new Error('Retryable error');
        (error as any).retryable = true;
        throw error;
      });

      await expect(bridge.chat('test')).rejects.toThrow('Retryable error');
      expect(attemptCount).toBe(3); // Default maxRetries is 3

      // Restore original method
      (bridge as any).sendChatRequest = originalSendChatRequest;
    });
  });

  // ============================================================================
  // Mock Response Tests
  // ============================================================================

  describe('Mock Response Generation', () => {
    it('should generate appropriate responses for different inputs', () => {
      const generateMockResponse = (bridge as any).generateMockResponse;

      expect(generateMockResponse('Summarize this document')).toContain('summary');
      expect(generateMockResponse('What is the capital of France?')).toContain('Paris');
      expect(generateMockResponse('Hello there')).toContain('Hello! I\'m the AI Core Service');
      expect(generateMockResponse('Help me')).toContain('I can help you with various tasks');
      expect(generateMockResponse('Random question')).toContain('I understand you\'re asking about');
    });
  });

  // ============================================================================
  // Utility Function Tests
  // ============================================================================

  describe('Utility Functions', () => {
    it('should generate unique session IDs', () => {
      const generateSessionId = (bridge as any).generateSessionId;
      const id1 = generateSessionId();
      const id2 = generateSessionId();
      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^session_\d+_[a-z0-9]+$/);
    });

    it('should generate unique message IDs', () => {
      const generateMessageId = (bridge as any).generateMessageId;
      const id1 = generateMessageId();
      const id2 = generateMessageId();
      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^msg_\d+_[a-z0-9]+$/);
    });

    it('should generate unique call IDs', () => {
      const generateCallId = (bridge as any).generateCallId;
      const id1 = generateCallId();
      const id2 = generateCallId();
      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^call_\d+_[a-z0-9]+$/);
    });
  });
});

// ============================================================================
// Factory Function Tests
// ============================================================================

describe('Factory Functions', () => {
  it('should create bridge instance with createAICoreBridge', () => {
    const bridge = createAICoreBridge();
    expect(bridge).toBeInstanceOf(AICoreBridge);
  });

  it('should create bridge instance with custom config', () => {
    const config: AICoreConfig = {
      timeout: 60000,
      maxRetries: 5,
    };
    const bridge = createAICoreBridge(config);
    expect(bridge).toBeInstanceOf(AICoreBridge);
  });

  it('should export default bridge instance', () => {
    expect(aiCoreBridge).toBeInstanceOf(AICoreBridge);
  });
});

// ============================================================================
// Integration Tests
// ============================================================================

describe('Integration Tests', () => {
  let bridge: AICoreBridge;

  beforeEach(async () => {
    bridge = new AICoreBridge();
    await bridge.initialize();
  });

  afterEach(async () => {
    await bridge.disconnect();
  });

  it('should handle complete workflow: chat -> tool -> metrics', async () => {
    // 1. Chat
    const chatResponse = await bridge.chat('Hello, AI!');
    expect(chatResponse.content).toContain('Hello! I\'m the AI Core Service');

    // 2. Tool call
    const toolResult = await bridge.tool('open_file', { path: '/tmp/test.txt' });
    expect(toolResult.success).toBe(true);

    // 3. Get metrics
    const metrics = await bridge.getMetrics();
    expect(metrics.requests.total).toBeGreaterThan(0);
  });

  it('should handle streaming workflow', async () => {
    const chunks: StreamingChunk[] = [];
    for await (const chunk of bridge.chatStream('What is the capital of France?')) {
      chunks.push(chunk);
    }

    expect(chunks.length).toBeGreaterThan(0);
    const fullContent = chunks.map(c => c.chunk).join('');
    expect(fullContent).toContain('Paris');
  });

  it('should handle CapToken workflow', async () => {
    const capToken: CapToken = {
      tokenId: 'test-token',
      token: 'test-token-value',
      scopes: ['ai:chat'],
      expiresAt: Date.now() + 3600000,
      issuer: 'test',
      subject: 'user',
      audience: 'ai-core-service',
      createdAt: Date.now(),
    };

    bridge.setCaps(capToken);
    expect(bridge.getCaps()).toEqual(capToken);

    const response = await bridge.chat('Test with CapToken');
    expect(response.content).toBeDefined();

    bridge.clearCaps();
    expect(bridge.getCaps()).toBeUndefined();
  });
});

// ============================================================================
// Error Handling Tests
// ============================================================================

describe('Error Handling', () => {
  let bridge: AICoreBridge;

  beforeEach(() => {
    bridge = new AICoreBridge();
  });

  it('should handle connection errors gracefully', async () => {
    // Mock ping to fail
    bridge.ping = jest.fn().mockRejectedValue(new Error('Connection failed'));

    await expect(bridge.initialize()).rejects.toThrow('Failed to connect to AI Core Service');
    expect(bridge.isServiceConnected()).toBe(false);
  });

  it('should handle service not connected errors', async () => {
    await expect(bridge.chat('test')).rejects.toThrow('AI Core Service not connected');
    await expect(bridge.tool('test', {})).rejects.toThrow('AI Core Service not connected');
    await expect(bridge.listTools()).rejects.toThrow('AI Core Service not connected');
    await expect(bridge.getMetrics()).rejects.toThrow('AI Core Service not connected');
  });

  it('should handle abort signal errors', async () => {
    const abortSignal: AbortSignal = {
      aborted: true,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
    };

    await expect(bridge.chat('test', undefined, abortSignal)).rejects.toThrow('Operation aborted');
  });
});

// ============================================================================
// Performance Tests
// ============================================================================

describe('Performance Tests', () => {
  let bridge: AICoreBridge;

  beforeEach(async () => {
    bridge = new AICoreBridge();
    await bridge.initialize();
  });

  afterEach(async () => {
    await bridge.disconnect();
  });

  it('should handle multiple concurrent chat requests', async () => {
    const promises = Array.from({ length: 5 }, (_, i) => 
      bridge.chat(`Test message ${i}`)
    );

    const responses = await Promise.all(promises);
    expect(responses).toHaveLength(5);
    responses.forEach(response => {
      expect(response.content).toBeDefined();
      expect(response.isComplete).toBe(true);
    });
  });

  it('should handle multiple concurrent tool calls', async () => {
    const promises = Array.from({ length: 3 }, (_, i) => 
      bridge.tool('open_file', { path: `/tmp/test${i}.txt` })
    );

    const results = await Promise.all(promises);
    expect(results).toHaveLength(3);
    results.forEach(result => {
      expect(result.success).toBe(true);
      expect(result.toolName).toBe('open_file');
    });
  });
});
