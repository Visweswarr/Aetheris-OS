import { EventEmitter } from 'events';
import { spawn, ChildProcess } from 'child_process';
import { join } from 'path';
import { platform } from 'os';

export interface ChatMessage {
  id: string;
  content: string;
  role: 'user' | 'assistant';
  timestamp: Date;
  metadata?: any;
}

export interface ChatResponse {
  id: string;
  content: string;
  role: 'assistant';
  timestamp: Date;
  citations?: string[];
  tools_used?: string[];
  metadata?: any;
}

export interface ToolCall {
  name: string;
  parameters: any;
  result?: any;
  error?: string;
}

export interface MemoryEntry {
  key: string;
  value: any;
  timestamp: Date;
  ttl?: number;
}

export class AICoreBridge extends EventEmitter {
  private aiCoreProcess: ChildProcess | null = null;
  private isConnected = false;
  private messageQueue: ChatMessage[] = [];
  private responseHandlers = new Map<string, (response: ChatResponse) => void>();
  private streamHandlers = new Map<string, (chunk: string) => void>();

  constructor() {
    super();
    this.setupProcess();
  }

  private setupProcess(): void {
    try {
      // Determine the AI Core executable path
      const executableName = platform() === 'win32' ? 'aetheris-ai-core.exe' : 'aetheris-ai-core';
      const executablePath = join(process.cwd(), '..', '..', 'services', 'ai_core', 'target', 'release', executableName);
      
      // Spawn the AI Core process
      this.aiCoreProcess = spawn(executablePath, [
        '--socket', '/tmp/aetheris-ai-core.sock',
        '--log-level', 'info',
        '--max-sessions', '10',
        '--request-timeout', '30000'
      ], {
        stdio: ['pipe', 'pipe', 'pipe'],
        env: {
          ...process.env,
          RUST_LOG: 'info',
          AETHERIS_AI_BACKEND: 'mock'
        }
      });

      // Handle process events
      this.aiCoreProcess.on('spawn', () => {
        console.log('AI Core process spawned');
        this.isConnected = true;
        this.emit('connected');
      });

      this.aiCoreProcess.on('error', (error) => {
        console.error('AI Core process error:', error);
        this.isConnected = false;
        this.emit('error', error);
      });

      this.aiCoreProcess.on('exit', (code, signal) => {
        console.log(`AI Core process exited with code ${code} and signal ${signal}`);
        this.isConnected = false;
        this.emit('disconnected', { code, signal });
      });

      // Handle stdout
      this.aiCoreProcess.stdout?.on('data', (data) => {
        const output = data.toString();
        console.log('AI Core stdout:', output);
        this.handleProcessOutput(output);
      });

      // Handle stderr
      this.aiCoreProcess.stderr?.on('data', (data) => {
        const error = data.toString();
        console.error('AI Core stderr:', error);
        this.emit('error', new Error(error));
      });

    } catch (error) {
      console.error('Failed to setup AI Core process:', error);
      this.emit('error', error);
    }
  }

  private handleProcessOutput(output: string): void {
    try {
      // Parse JSON responses from AI Core
      const lines = output.trim().split('\n');
      for (const line of lines) {
        if (line.startsWith('{') && line.endsWith('}')) {
          const data = JSON.parse(line);
          this.handleResponse(data);
        }
      }
    } catch (error) {
      console.error('Error parsing AI Core output:', error);
    }
  }

  private handleResponse(data: any): void {
    if (data.type === 'chat_response') {
      const response: ChatResponse = {
        id: data.id,
        content: data.content,
        role: 'assistant',
        timestamp: new Date(data.timestamp),
        citations: data.citations,
        tools_used: data.tools_used,
        metadata: data.metadata
      };

      // Find and call the appropriate handler
      const handler = this.responseHandlers.get(data.message_id);
      if (handler) {
        handler(response);
        this.responseHandlers.delete(data.message_id);
      }

      this.emit('chat_response', response);
    } else if (data.type === 'stream_chunk') {
      const handler = this.streamHandlers.get(data.message_id);
      if (handler) {
        handler(data.chunk);
      }

      this.emit('stream_chunk', data);
    } else if (data.type === 'tool_result') {
      this.emit('tool_result', data);
    }
  }

  async sendChatMessage(message: string): Promise<ChatResponse> {
    if (!this.isConnected || !this.aiCoreProcess) {
      throw new Error('AI Core not connected');
    }

    const messageId = this.generateId();
    const chatMessage: ChatMessage = {
      id: messageId,
      content: message,
      role: 'user',
      timestamp: new Date()
    };

    // Add to queue
    this.messageQueue.push(chatMessage);

    // Send to AI Core
    const request = {
      type: 'chat_request',
      id: messageId,
      message: message,
      timestamp: new Date().toISOString()
    };

    this.aiCoreProcess.stdin?.write(JSON.stringify(request) + '\n');

    // Return a promise that resolves when we get the response
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.responseHandlers.delete(messageId);
        reject(new Error('Request timeout'));
      }, 30000);

      this.responseHandlers.set(messageId, (response) => {
        clearTimeout(timeout);
        resolve(response);
      });
    });
  }

  async streamChatMessage(message: string): Promise<AsyncGenerator<string, void, unknown>> {
    if (!this.isConnected || !this.aiCoreProcess) {
      throw new Error('AI Core not connected');
    }

    const messageId = this.generateId();
    const chatMessage: ChatMessage = {
      id: messageId,
      content: message,
      role: 'user',
      timestamp: new Date()
    };

    // Add to queue
    this.messageQueue.push(chatMessage);

    // Send streaming request to AI Core
    const request = {
      type: 'stream_request',
      id: messageId,
      message: message,
      timestamp: new Date().toISOString()
    };

    this.aiCoreProcess.stdin?.write(JSON.stringify(request) + '\n');

    // Return async generator for streaming
    return this.createStreamGenerator(messageId);
  }

  private async *createStreamGenerator(messageId: string): AsyncGenerator<string, void, unknown> {
    const chunks: string[] = [];
    let isComplete = false;

    const handler = (chunk: string) => {
      chunks.push(chunk);
    };

    this.streamHandlers.set(messageId, handler);

    // Cleanup handler after 30 seconds
    setTimeout(() => {
      this.streamHandlers.delete(messageId);
      isComplete = true;
    }, 30000);

    while (!isComplete) {
      if (chunks.length > 0) {
        const chunk = chunks.shift();
        if (chunk) {
          yield chunk;
        }
      }
      await new Promise(resolve => setTimeout(resolve, 10));
    }

    this.streamHandlers.delete(messageId);
  }

  async callTool(toolName: string, parameters: any): Promise<any> {
    if (!this.isConnected || !this.aiCoreProcess) {
      throw new Error('AI Core not connected');
    }

    const requestId = this.generateId();
    const request = {
      type: 'tool_call',
      id: requestId,
      tool_name: toolName,
      parameters: parameters,
      timestamp: new Date().toISOString()
    };

    this.aiCoreProcess.stdin?.write(JSON.stringify(request) + '\n');

    // Return a promise that resolves when we get the tool result
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Tool call timeout'));
      }, 30000);

      const handler = (data: any) => {
        if (data.request_id === requestId) {
          clearTimeout(timeout);
          this.removeListener('tool_result', handler);
          if (data.error) {
            reject(new Error(data.error));
          } else {
            resolve(data.result);
          }
        }
      };

      this.on('tool_result', handler);
    });
  }

  async getMemory(key: string): Promise<any> {
    if (!this.isConnected || !this.aiCoreProcess) {
      throw new Error('AI Core not connected');
    }

    const requestId = this.generateId();
    const request = {
      type: 'memory_get',
      id: requestId,
      key: key,
      timestamp: new Date().toISOString()
    };

    this.aiCoreProcess.stdin?.write(JSON.stringify(request) + '\n');

    // Return a promise that resolves when we get the memory value
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Memory get timeout'));
      }, 10000);

      const handler = (data: any) => {
        if (data.request_id === requestId) {
          clearTimeout(timeout);
          this.removeListener('memory_result', handler);
          if (data.error) {
            reject(new Error(data.error));
          } else {
            resolve(data.value);
          }
        }
      };

      this.on('memory_result', handler);
    });
  }

  async setMemory(key: string, value: any): Promise<void> {
    if (!this.isConnected || !this.aiCoreProcess) {
      throw new Error('AI Core not connected');
    }

    const requestId = this.generateId();
    const request = {
      type: 'memory_set',
      id: requestId,
      key: key,
      value: value,
      timestamp: new Date().toISOString()
    };

    this.aiCoreProcess.stdin?.write(JSON.stringify(request) + '\n');

    // Return a promise that resolves when we get confirmation
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Memory set timeout'));
      }, 10000);

      const handler = (data: any) => {
        if (data.request_id === requestId) {
          clearTimeout(timeout);
          this.removeListener('memory_result', handler);
          if (data.error) {
            reject(new Error(data.error));
          } else {
            resolve();
          }
        }
      };

      this.on('memory_result', handler);
    });
  }

  getMessageHistory(): ChatMessage[] {
    return [...this.messageQueue];
  }

  clearMessageHistory(): void {
    this.messageQueue = [];
  }

  isAICoreConnected(): boolean {
    return this.isConnected;
  }

  cleanup(): void {
    if (this.aiCoreProcess) {
      this.aiCoreProcess.kill();
      this.aiCoreProcess = null;
    }
    this.isConnected = false;
    this.responseHandlers.clear();
    this.streamHandlers.clear();
  }

  private generateId(): string {
    return Math.random().toString(36).substr(2, 9);
  }
}
