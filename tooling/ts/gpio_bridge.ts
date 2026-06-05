/**
 * TypeScript SDK for Aetheris OS GPIO Operations
 * 
 * This module provides client-side interaction with the Aetheris OS GPIO system,
 * including pin configuration, digital I/O, and deterministic state management.
 */

import { EventEmitter } from 'events';

// Type definitions
export type GpioMode = 'input' | 'output';

export type GpioPull = 'none' | 'up' | 'down';

export interface GpioConfig {
  pin: number;
  mode: GpioMode;
  pull: GpioPull;
  initialValue: boolean;
  debounceMs: number;
}

export interface GpioState {
  pin: number;
  mode: GpioMode;
  pull: GpioPull;
  value: boolean;
  lastChange: number;
  changeCount: number;
}

export interface GpioOperationResult {
  operationId: string;
  pin: number;
  operation: string;
  result: boolean;
  timestamp: number;
  deterministic: boolean;
}

export interface GpioSnapshot {
  snapshotId: string;
  ngfsPath: string;
  createdAt: Date;
  pinCount: number;
  operationCount: number;
  metadata: Record<string, any>;
}

export interface GpioConfig {
  endpoint: string;
  timeout: number;
  retries: number;
  enableLogging: boolean;
  enablePolicyEnforcement: boolean;
  enableCapabilityChecking: boolean;
  defaultDebounceMs: number;
  maxDebounceMs: number;
  snapshotTimeout: number;
}

export interface GpioStats {
  connectionStatus: 'connected' | 'disconnected' | 'connecting' | 'error';
  totalPins: number;
  configuredPins: number;
  inputPins: number;
  outputPins: number;
  totalOperations: number;
  readOperations: number;
  writeOperations: number;
  toggleOperations: number;
  lastOperationTime: number;
  deterministicMode: boolean;
  tickCounter: number;
}

/**
 * Main GPIO Bridge class
 */
export class GpioBridge extends EventEmitter {
  private config: GpioConfig;
  private isConnected: boolean = false;
  private connectionRetries: number = 0;
  private configuredPins: Map<number, GpioState> = new Map();
  private operationHistory: GpioOperationResult[] = [];

  constructor(config: Partial<GpioConfig> = {}) {
    super();
    
    this.config = {
      endpoint: config.endpoint || 'http://localhost:8080',
      timeout: config.timeout || 30000,
      retries: config.retries || 3,
      enableLogging: config.enableLogging || false,
      enablePolicyEnforcement: config.enablePolicyEnforcement || true,
      enableCapabilityChecking: config.enableCapabilityChecking || true,
      defaultDebounceMs: config.defaultDebounceMs || 0,
      maxDebounceMs: config.maxDebounceMs || 1000,
      snapshotTimeout: config.snapshotTimeout || 10000,
    };
  }

  /**
   * Initialize the bridge connection
   */
  async initialize(): Promise<void> {
    try {
      await this.connect();
      this.isConnected = true;
      this.connectionRetries = 0;
      this.emit('connected');
    } catch (error) {
      this.emit('error', error);
      throw error;
    }
  }

  /**
   * Connect to the GPIO service
   */
  private async connect(): Promise<void> {
    const response = await fetch(`${this.config.endpoint}/gpio/health`, {
      method: 'GET',
      timeout: this.config.timeout,
    });

    if (!response.ok) {
      throw new Error(`Failed to connect to GPIO service: ${response.statusText}`);
    }
  }

  /**
   * Configure GPIO pin
   */
  async configurePin(sessionId: string, caps: string, config: GpioConfig): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/configure', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        pin: config.pin,
        mode: config.mode,
        pull: config.pull,
        initial_value: config.initialValue,
        debounce_ms: config.debounceMs,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to configure GPIO pin: ${response.error}`);
    }

    const state: GpioState = {
      pin: config.pin,
      mode: config.mode,
      pull: config.pull,
      value: config.initialValue,
      lastChange: Date.now(),
      changeCount: 0,
    };

    this.configuredPins.set(config.pin, state);

    this.emit('pinConfigured', {
      pin: config.pin,
      config,
      state,
    });
  }

  /**
   * Read GPIO pin value
   */
  async readPin(sessionId: string, caps: string, pin: number): Promise<boolean> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/read', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        pin,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to read GPIO pin: ${response.error}`);
    }

    const value = response.value;
    
    // Update local state
    const state = this.configuredPins.get(pin);
    if (state) {
      state.value = value;
      state.lastChange = Date.now();
      state.changeCount++;
    }

    this.emit('pinRead', {
      pin,
      value,
      timestamp: Date.now(),
    });

    return value;
  }

  /**
   * Write GPIO pin value
   */
  async writePin(sessionId: string, caps: string, pin: number, value: boolean): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/write', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        pin,
        value,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to write GPIO pin: ${response.error}`);
    }

    // Update local state
    const state = this.configuredPins.get(pin);
    if (state) {
      state.value = value;
      state.lastChange = Date.now();
      state.changeCount++;
    }

    this.emit('pinWritten', {
      pin,
      value,
      timestamp: Date.now(),
    });
  }

  /**
   * Toggle GPIO pin value
   */
  async togglePin(sessionId: string, caps: string, pin: number): Promise<boolean> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/toggle', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        pin,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to toggle GPIO pin: ${response.error}`);
    }

    const newValue = response.new_value;
    
    // Update local state
    const state = this.configuredPins.get(pin);
    if (state) {
      state.value = newValue;
      state.lastChange = Date.now();
      state.changeCount++;
    }

    this.emit('pinToggled', {
      pin,
      newValue,
      timestamp: Date.now(),
    });

    return newValue;
  }

  /**
   * Get GPIO pin state
   */
  async getPinState(pin: number): Promise<GpioState | null> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/gpio/state/${pin}`, {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to get GPIO pin state: ${response.error}`);
    }

    const state: GpioState = {
      pin: response.pin,
      mode: response.mode,
      pull: response.pull,
      value: response.value,
      lastChange: response.last_change,
      changeCount: response.change_count,
    };

    this.configuredPins.set(pin, state);

    return state;
  }

  /**
   * List all configured GPIO pins
   */
  async listConfiguredPins(): Promise<GpioState[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/list', {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to list GPIO pins: ${response.error}`);
    }

    const pins: GpioState[] = (response.pins || []).map((pin: any) => ({
      pin: pin.pin,
      mode: pin.mode,
      pull: pin.pull,
      value: pin.value,
      lastChange: pin.last_change,
      changeCount: pin.change_count,
    }));

    // Update local cache
    pins.forEach(pin => {
      this.configuredPins.set(pin.pin, pin);
    });

    return pins;
  }

  /**
   * Get GPIO operation history
   */
  async getOperationHistory(limit: number = 100): Promise<GpioOperationResult[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/history', {
      method: 'GET',
      body: JSON.stringify({
        limit,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to get GPIO operation history: ${response.error}`);
    }

    const operations: GpioOperationResult[] = (response.operations || []).map((op: any) => ({
      operationId: op.operation_id,
      pin: op.pin,
      operation: op.operation,
      result: op.result,
      timestamp: op.timestamp,
      deterministic: op.deterministic,
    }));

    this.operationHistory = operations;

    return operations;
  }

  /**
   * Enable deterministic mode
   */
  async enableDeterministicMode(): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/deterministic/enable', {
      method: 'POST',
    });

    if (!response.success) {
      throw new Error(`Failed to enable deterministic mode: ${response.error}`);
    }

    this.emit('deterministicModeEnabled');
  }

  /**
   * Disable deterministic mode
   */
  async disableDeterministicMode(): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/deterministic/disable', {
      method: 'POST',
    });

    if (!response.success) {
      throw new Error(`Failed to disable deterministic mode: ${response.error}`);
    }

    this.emit('deterministicModeDisabled');
  }

  /**
   * Advance tick counter
   */
  async advanceTick(): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/tick/advance', {
      method: 'POST',
    });

    if (!response.success) {
      throw new Error(`Failed to advance tick counter: ${response.error}`);
    }

    this.emit('tickAdvanced', {
      tickCount: response.tick_count,
    });
  }

  /**
   * Get tick counter
   */
  async getTickCounter(): Promise<number> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/tick/counter', {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to get tick counter: ${response.error}`);
    }

    return response.tick_count;
  }

  /**
   * Create GPIO snapshot
   */
  async createSnapshot(outputPath: string): Promise<string> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/snapshot/create', {
      method: 'POST',
      body: JSON.stringify({
        output_path: outputPath,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to create GPIO snapshot: ${response.error}`);
    }

    const snapshot: GpioSnapshot = {
      snapshotId: response.snapshot_id,
      ngfsPath: outputPath,
      createdAt: new Date(),
      pinCount: response.pin_count || 0,
      operationCount: response.operation_count || 0,
      metadata: response.metadata || {},
    };

    this.emit('snapshotCreated', {
      snapshot,
    });

    return response.snapshot_id;
  }

  /**
   * Restore GPIO snapshot
   */
  async restoreSnapshot(snapshotData: Uint8Array): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/snapshot/restore', {
      method: 'POST',
      body: snapshotData,
      headers: {
        'Content-Type': 'application/octet-stream',
      },
    });

    if (!response.success) {
      throw new Error(`Failed to restore GPIO snapshot: ${response.error}`);
    }

    this.emit('snapshotRestored', {
      snapshotId: response.snapshot_id,
    });
  }

  /**
   * Get GPIO statistics
   */
  async getStats(): Promise<GpioStats> {
    await this.ensureConnected();

    const response = await this.makeRequest('/gpio/stats', {
      method: 'GET',
    });

    return response;
  }

  /**
   * Get configured pins
   */
  getConfiguredPins(): Map<number, GpioState> {
    return new Map(this.configuredPins);
  }

  /**
   * Get operation history
   */
  getOperationHistory(): GpioOperationResult[] {
    return [...this.operationHistory];
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<GpioConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfig(): GpioConfig {
    return { ...this.config };
  }

  /**
   * Check if connected
   */
  isBridgeConnected(): boolean {
    return this.isConnected;
  }

  /**
   * Disconnect from the service
   */
  async disconnect(): Promise<void> {
    this.isConnected = false;
    this.emit('disconnected');
  }

  /**
   * Ensure connection is established
   */
  private async ensureConnected(): Promise<void> {
    if (!this.isConnected) {
      await this.initialize();
    }
  }

  /**
   * Make HTTP request with retry logic
   */
  private async makeRequest(endpoint: string, options: RequestInit): Promise<any> {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= this.config.retries; attempt++) {
      try {
        const response = await fetch(`${this.config.endpoint}${endpoint}`, {
          ...options,
          headers: {
            'Content-Type': 'application/json',
            ...options.headers,
          },
          signal: AbortSignal.timeout(this.config.timeout),
        });

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();
        return data;
      } catch (error) {
        lastError = error as Error;
        
        if (attempt < this.config.retries) {
          const delay = Math.pow(2, attempt) * 1000; // Exponential backoff
          await new Promise(resolve => setTimeout(resolve, delay));
        }
      }
    }

    throw lastError || new Error('Request failed after all retries');
  }
}

/**
 * Utility functions for working with GPIO
 */
export class GpioUtils {
  /**
   * Get default GPIO configuration
   */
  static getDefaultConfig(pin: number): GpioConfig {
    return {
      pin,
      mode: 'input',
      pull: 'none',
      initialValue: false,
      debounceMs: 0,
    };
  }

  /**
   * Validate GPIO pin number
   */
  static validatePinNumber(pin: number): boolean {
    return pin >= 0 && pin <= 255;
  }

  /**
   * Validate GPIO mode
   */
  static validateMode(mode: string): mode is GpioMode {
    return mode === 'input' || mode === 'output';
  }

  /**
   * Validate GPIO pull configuration
   */
  static validatePull(pull: string): pull is GpioPull {
    return pull === 'none' || pull === 'up' || pull === 'down';
  }

  /**
   * Validate debounce time
   */
  static validateDebounceMs(debounceMs: number): boolean {
    return debounceMs >= 0 && debounceMs <= 1000;
  }

  /**
   * Convert boolean to string representation
   */
  static boolToString(value: boolean): string {
    return value ? 'HIGH' : 'LOW';
  }

  /**
   * Convert string to boolean representation
   */
  static stringToBool(value: string): boolean {
    const lower = value.toLowerCase();
    return lower === 'high' || lower === '1' || lower === 'true';
  }

  /**
   * Get pin mode description
   */
  static getModeDescription(mode: GpioMode): string {
    switch (mode) {
      case 'input':
        return 'Digital Input';
      case 'output':
        return 'Digital Output';
      default:
        return 'Unknown';
    }
  }

  /**
   * Get pull configuration description
   */
  static getPullDescription(pull: GpioPull): string {
    switch (pull) {
      case 'none':
        return 'No Pull';
      case 'up':
        return 'Pull Up';
      case 'down':
        return 'Pull Down';
      default:
        return 'Unknown';
    }
  }

  /**
   * Calculate pin statistics
   */
  static calculatePinStats(pins: GpioState[]): {
    totalPins: number;
    inputPins: number;
    outputPins: number;
    highPins: number;
    lowPins: number;
    totalChanges: number;
    averageChanges: number;
  } {
    if (pins.length === 0) {
      return {
        totalPins: 0,
        inputPins: 0,
        outputPins: 0,
        highPins: 0,
        lowPins: 0,
        totalChanges: 0,
        averageChanges: 0,
      };
    }

    const inputPins = pins.filter(pin => pin.mode === 'input').length;
    const outputPins = pins.filter(pin => pin.mode === 'output').length;
    const highPins = pins.filter(pin => pin.value).length;
    const lowPins = pins.filter(pin => !pin.value).length;
    const totalChanges = pins.reduce((sum, pin) => sum + pin.changeCount, 0);
    const averageChanges = totalChanges / pins.length;

    return {
      totalPins: pins.length,
      inputPins,
      outputPins,
      highPins,
      lowPins,
      totalChanges,
      averageChanges,
    };
  }

  /**
   * Generate deterministic GPIO pattern
   */
  static generateDeterministicPattern(seed: number, pinCount: number, tickCount: number): boolean[] {
    const pattern: boolean[] = [];
    
    for (let i = 0; i < pinCount; i++) {
      // Simple deterministic pattern based on seed, pin, and tick
      const value = Math.sin(seed + i * 0.1 + tickCount * 0.01) > 0;
      pattern.push(value);
    }
    
    return pattern;
  }

  /**
   * Create GPIO test pattern
   */
  static createTestPattern(pinCount: number, patternType: 'alternating' | 'walking' | 'random'): boolean[] {
    const pattern: boolean[] = [];
    
    switch (patternType) {
      case 'alternating':
        for (let i = 0; i < pinCount; i++) {
          pattern.push(i % 2 === 0);
        }
        break;
      case 'walking':
        for (let i = 0; i < pinCount; i++) {
          pattern.push(i === 0);
        }
        break;
      case 'random':
        for (let i = 0; i < pinCount; i++) {
          pattern.push(Math.random() > 0.5);
        }
        break;
    }
    
    return pattern;
  }
}

/**
 * Event types for the GpioBridge
 */
export interface GpioBridgeEvents {
  connected: () => void;
  disconnected: () => void;
  error: (error: Error) => void;
  pinConfigured: (data: { pin: number; config: GpioConfig; state: GpioState }) => void;
  pinRead: (data: { pin: number; value: boolean; timestamp: number }) => void;
  pinWritten: (data: { pin: number; value: boolean; timestamp: number }) => void;
  pinToggled: (data: { pin: number; newValue: boolean; timestamp: number }) => void;
  deterministicModeEnabled: () => void;
  deterministicModeDisabled: () => void;
  tickAdvanced: (data: { tickCount: number }) => void;
  snapshotCreated: (data: { snapshot: GpioSnapshot }) => void;
  snapshotRestored: (data: { snapshotId: string }) => void;
}

/**
 * Default export
 */
export default GpioBridge;
