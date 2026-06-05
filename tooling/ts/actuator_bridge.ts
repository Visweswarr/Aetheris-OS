/**
 * TypeScript SDK for Aetheris OS Actuator Operations
 * 
 * This module provides client-side interaction with the Aetheris OS actuator system,
 * including PWM outputs, relays, motor control, LEDs, and deterministic state management.
 */

import { EventEmitter } from 'events';

// Type definitions
export type ActuatorType = 'pwm' | 'relay' | 'motor' | 'led' | 'servo' | 'stepper' | 'solenoid' | 'valve';

export interface ActuatorConfig {
  name: string;
  type: ActuatorType;
  pin: number;
  minValue: number;
  maxValue: number;
  defaultValue: number;
  frequency: number;
  resolution: number;
  enableSafetyLimits: boolean;
  safetyMin: number;
  safetyMax: number;
  rampTimeMs: number;
  enableRamping: boolean;
}

export interface ActuatorState {
  name: string;
  type: ActuatorType;
  configured: boolean;
  enabled: boolean;
  currentValue: number;
  targetValue: number;
  lastUpdate: number;
  updateCount: number;
  safetyEnabled: boolean;
  safetyMin: number;
  safetyMax: number;
  rampingEnabled: boolean;
  rampTimeMs: number;
  deterministic: boolean;
}

export interface ActuatorOperationResult {
  operationId: string;
  actuatorName: string;
  operation: string;
  result: boolean;
  timestamp: number;
  deterministic: boolean;
}

export interface ActuatorPatternStep {
  value: number;
  durationMs: number;
}

export interface ActuatorPattern {
  name: string;
  stepCount: number;
  steps: ActuatorPatternStep[];
  loop: boolean;
  loopCount: number;
}

export interface ActuatorSnapshot {
  snapshotId: string;
  ngfsPath: string;
  createdAt: Date;
  actuatorCount: number;
  operationCount: number;
  metadata: Record<string, any>;
}

export interface ActuatorConfig {
  endpoint: string;
  timeout: number;
  retries: number;
  enableLogging: boolean;
  enablePolicyEnforcement: boolean;
  enableCapabilityChecking: boolean;
  defaultFrequency: number;
  maxFrequency: number;
  defaultResolution: number;
  maxResolution: number;
  defaultRampTimeMs: number;
  maxRampTimeMs: number;
  snapshotTimeout: number;
}

export interface ActuatorStats {
  connectionStatus: 'connected' | 'disconnected' | 'connecting' | 'error';
  totalActuators: number;
  configuredActuators: number;
  enabledActuators: number;
  totalOperations: number;
  setValueOperations: number;
  patternOperations: number;
  enableOperations: number;
  disableOperations: number;
  lastOperationTime: number;
  deterministicMode: boolean;
  tickCounter: number;
}

/**
 * Main Actuator Bridge class
 */
export class ActuatorBridge extends EventEmitter {
  private config: ActuatorConfig;
  private isConnected: boolean = false;
  private connectionRetries: number = 0;
  private configuredActuators: Map<string, ActuatorState> = new Map();
  private operationHistory: ActuatorOperationResult[] = [];

  constructor(config: Partial<ActuatorConfig> = {}) {
    super();
    
    this.config = {
      endpoint: config.endpoint || 'http://localhost:8080',
      timeout: config.timeout || 30000,
      retries: config.retries || 3,
      enableLogging: config.enableLogging || false,
      enablePolicyEnforcement: config.enablePolicyEnforcement || true,
      enableCapabilityChecking: config.enableCapabilityChecking || true,
      defaultFrequency: config.defaultFrequency || 1000,
      maxFrequency: config.maxFrequency || 10000,
      defaultResolution: config.defaultResolution || 8,
      maxResolution: config.maxResolution || 16,
      defaultRampTimeMs: config.defaultRampTimeMs || 100,
      maxRampTimeMs: config.maxRampTimeMs || 10000,
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
   * Connect to the actuator service
   */
  private async connect(): Promise<void> {
    const response = await fetch(`${this.config.endpoint}/actuator/health`, {
      method: 'GET',
      timeout: this.config.timeout,
    });

    if (!response.ok) {
      throw new Error(`Failed to connect to actuator service: ${response.statusText}`);
    }
  }

  /**
   * Configure actuator
   */
  async configureActuator(sessionId: string, caps: string, config: ActuatorConfig): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/configure', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        name: config.name,
        type: config.type,
        pin: config.pin,
        min_value: config.minValue,
        max_value: config.maxValue,
        default_value: config.defaultValue,
        frequency: config.frequency,
        resolution: config.resolution,
        enable_safety_limits: config.enableSafetyLimits,
        safety_min: config.safetyMin,
        safety_max: config.safetyMax,
        ramp_time_ms: config.rampTimeMs,
        enable_ramping: config.enableRamping,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to configure actuator: ${response.error}`);
    }

    const state: ActuatorState = {
      name: config.name,
      type: config.type,
      configured: true,
      enabled: false,
      currentValue: config.defaultValue,
      targetValue: config.defaultValue,
      lastUpdate: Date.now(),
      updateCount: 0,
      safetyEnabled: config.enableSafetyLimits,
      safetyMin: config.safetyMin,
      safetyMax: config.safetyMax,
      rampingEnabled: config.enableRamping,
      rampTimeMs: config.rampTimeMs,
      deterministic: false,
    };

    this.configuredActuators.set(config.name, state);

    this.emit('actuatorConfigured', {
      name: config.name,
      config,
      state,
    });
  }

  /**
   * Enable actuator
   */
  async enableActuator(sessionId: string, caps: string, name: string): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/enable', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        name,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to enable actuator: ${response.error}`);
    }

    // Update local state
    const state = this.configuredActuators.get(name);
    if (state) {
      state.enabled = true;
    }

    this.emit('actuatorEnabled', {
      name,
      timestamp: Date.now(),
    });
  }

  /**
   * Disable actuator
   */
  async disableActuator(sessionId: string, caps: string, name: string): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/disable', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        name,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to disable actuator: ${response.error}`);
    }

    // Update local state
    const state = this.configuredActuators.get(name);
    if (state) {
      state.enabled = false;
    }

    this.emit('actuatorDisabled', {
      name,
      timestamp: Date.now(),
    });
  }

  /**
   * Set actuator value
   */
  async setActuatorValue(sessionId: string, caps: string, name: string, value: number): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/set', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        name,
        value,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to set actuator value: ${response.error}`);
    }

    // Update local state
    const state = this.configuredActuators.get(name);
    if (state) {
      state.currentValue = value;
      state.targetValue = value;
      state.lastUpdate = Date.now();
      state.updateCount++;
    }

    this.emit('actuatorValueSet', {
      name,
      value,
      timestamp: Date.now(),
    });
  }

  /**
   * Get actuator value
   */
  async getActuatorValue(name: string): Promise<number> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/actuator/get/${name}`, {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to get actuator value: ${response.error}`);
    }

    return response.value;
  }

  /**
   * Set actuator pattern
   */
  async setActuatorPattern(sessionId: string, caps: string, name: string, pattern: ActuatorPattern): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/pattern', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        name,
        pattern_name: pattern.name,
        steps: pattern.steps.map(step => ({
          value: step.value,
          duration_ms: step.durationMs,
        })),
        step_count: pattern.stepCount,
        loop: pattern.loop,
        loop_count: pattern.loopCount,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to set actuator pattern: ${response.error}`);
    }

    this.emit('actuatorPatternSet', {
      name,
      pattern,
      timestamp: Date.now(),
    });
  }

  /**
   * Stop actuator pattern
   */
  async stopActuatorPattern(sessionId: string, caps: string, name: string): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/stop-pattern', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        name,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to stop actuator pattern: ${response.error}`);
    }

    this.emit('actuatorPatternStopped', {
      name,
      timestamp: Date.now(),
    });
  }

  /**
   * Get actuator state
   */
  async getActuatorState(name: string): Promise<ActuatorState | null> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/actuator/state/${name}`, {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to get actuator state: ${response.error}`);
    }

    const state: ActuatorState = {
      name: response.name,
      type: response.type,
      configured: response.configured,
      enabled: response.enabled,
      currentValue: response.current_value,
      targetValue: response.target_value,
      lastUpdate: response.last_update,
      updateCount: response.update_count,
      safetyEnabled: response.safety_enabled,
      safetyMin: response.safety_min,
      safetyMax: response.safety_max,
      rampingEnabled: response.ramping_enabled,
      rampTimeMs: response.ramp_time_ms,
      deterministic: response.deterministic,
    };

    this.configuredActuators.set(name, state);

    return state;
  }

  /**
   * List all configured actuators
   */
  async listConfiguredActuators(): Promise<ActuatorState[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/list', {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to list actuators: ${response.error}`);
    }

    const actuators: ActuatorState[] = (response.actuators || []).map((actuator: any) => ({
      name: actuator.name,
      type: actuator.type,
      configured: actuator.configured,
      enabled: actuator.enabled,
      currentValue: actuator.current_value,
      targetValue: actuator.target_value,
      lastUpdate: actuator.last_update,
      updateCount: actuator.update_count,
      safetyEnabled: actuator.safety_enabled,
      safetyMin: actuator.safety_min,
      safetyMax: actuator.safety_max,
      rampingEnabled: actuator.ramping_enabled,
      rampTimeMs: actuator.ramp_time_ms,
      deterministic: actuator.deterministic,
    }));

    // Update local cache
    actuators.forEach(actuator => {
      this.configuredActuators.set(actuator.name, actuator);
    });

    return actuators;
  }

  /**
   * Get actuator operation history
   */
  async getOperationHistory(limit: number = 100): Promise<ActuatorOperationResult[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/history', {
      method: 'GET',
      body: JSON.stringify({
        limit,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to get actuator operation history: ${response.error}`);
    }

    const operations: ActuatorOperationResult[] = (response.operations || []).map((op: any) => ({
      operationId: op.operation_id,
      actuatorName: op.actuator_name,
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

    const response = await this.makeRequest('/actuator/deterministic/enable', {
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

    const response = await this.makeRequest('/actuator/deterministic/disable', {
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

    const response = await this.makeRequest('/actuator/tick/advance', {
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

    const response = await this.makeRequest('/actuator/tick/counter', {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to get tick counter: ${response.error}`);
    }

    return response.tick_count;
  }

  /**
   * Create actuator snapshot
   */
  async createSnapshot(outputPath: string): Promise<string> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/snapshot/create', {
      method: 'POST',
      body: JSON.stringify({
        output_path: outputPath,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to create actuator snapshot: ${response.error}`);
    }

    const snapshot: ActuatorSnapshot = {
      snapshotId: response.snapshot_id,
      ngfsPath: outputPath,
      createdAt: new Date(),
      actuatorCount: response.actuator_count || 0,
      operationCount: response.operation_count || 0,
      metadata: response.metadata || {},
    };

    this.emit('snapshotCreated', {
      snapshot,
    });

    return response.snapshot_id;
  }

  /**
   * Restore actuator snapshot
   */
  async restoreSnapshot(snapshotData: Uint8Array): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/snapshot/restore', {
      method: 'POST',
      body: snapshotData,
      headers: {
        'Content-Type': 'application/octet-stream',
      },
    });

    if (!response.success) {
      throw new Error(`Failed to restore actuator snapshot: ${response.error}`);
    }

    this.emit('snapshotRestored', {
      snapshotId: response.snapshot_id,
    });
  }

  /**
   * Get actuator statistics
   */
  async getStats(): Promise<ActuatorStats> {
    await this.ensureConnected();

    const response = await this.makeRequest('/actuator/stats', {
      method: 'GET',
    });

    return response;
  }

  /**
   * Get configured actuators
   */
  getConfiguredActuators(): Map<string, ActuatorState> {
    return new Map(this.configuredActuators);
  }

  /**
   * Get operation history
   */
  getOperationHistory(): ActuatorOperationResult[] {
    return [...this.operationHistory];
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<ActuatorConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfig(): ActuatorConfig {
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
 * Utility functions for working with actuators
 */
export class ActuatorUtils {
  /**
   * Get default actuator configuration
   */
  static getDefaultConfig(name: string, type: ActuatorType): ActuatorConfig {
    return {
      name,
      type,
      pin: 0,
      minValue: 0.0,
      maxValue: 1.0,
      defaultValue: 0.0,
      frequency: 1000,
      resolution: 8,
      enableSafetyLimits: true,
      safetyMin: 0.0,
      safetyMax: 1.0,
      rampTimeMs: 100,
      enableRamping: false,
    };
  }

  /**
   * Validate actuator name
   */
  static validateActuatorName(name: string): boolean {
    return name.length > 0 && name.length <= 64 && /^[a-zA-Z0-9_-]+$/.test(name);
  }

  /**
   * Validate actuator type
   */
  static validateActuatorType(type: string): type is ActuatorType {
    return ['pwm', 'relay', 'motor', 'led', 'servo', 'stepper', 'solenoid', 'valve'].includes(type);
  }

  /**
   * Validate actuator value
   */
  static validateActuatorValue(value: number, minValue: number, maxValue: number): boolean {
    return value >= minValue && value <= maxValue;
  }

  /**
   * Validate safety limits
   */
  static validateSafetyLimits(safetyMin: number, safetyMax: number, minValue: number, maxValue: number): boolean {
    return safetyMin >= minValue && safetyMax <= maxValue && safetyMin < safetyMax;
  }

  /**
   * Validate frequency
   */
  static validateFrequency(frequency: number): boolean {
    return frequency > 0 && frequency <= 10000;
  }

  /**
   * Validate resolution
   */
  static validateResolution(resolution: number): boolean {
    return resolution >= 1 && resolution <= 16;
  }

  /**
   * Validate ramp time
   */
  static validateRampTime(rampTimeMs: number): boolean {
    return rampTimeMs >= 0 && rampTimeMs <= 10000;
  }

  /**
   * Get actuator type description
   */
  static getActuatorTypeDescription(type: ActuatorType): string {
    switch (type) {
      case 'pwm':
        return 'PWM Output';
      case 'relay':
        return 'Digital Relay';
      case 'motor':
        return 'Motor Control';
      case 'led':
        return 'LED Control';
      case 'servo':
        return 'Servo Motor';
      case 'stepper':
        return 'Stepper Motor';
      case 'solenoid':
        return 'Solenoid Valve';
      case 'valve':
        return 'Control Valve';
      default:
        return 'Unknown';
    }
  }

  /**
   * Calculate actuator statistics
   */
  static calculateActuatorStats(actuators: ActuatorState[]): {
    totalActuators: number;
    configuredActuators: number;
    enabledActuators: number;
    totalUpdates: number;
    averageUpdates: number;
    safetyEnabledCount: number;
    rampingEnabledCount: number;
  } {
    if (actuators.length === 0) {
      return {
        totalActuators: 0,
        configuredActuators: 0,
        enabledActuators: 0,
        totalUpdates: 0,
        averageUpdates: 0,
        safetyEnabledCount: 0,
        rampingEnabledCount: 0,
      };
    }

    const configuredActuators = actuators.filter(actuator => actuator.configured).length;
    const enabledActuators = actuators.filter(actuator => actuator.enabled).length;
    const totalUpdates = actuators.reduce((sum, actuator) => sum + actuator.updateCount, 0);
    const averageUpdates = totalUpdates / actuators.length;
    const safetyEnabledCount = actuators.filter(actuator => actuator.safetyEnabled).length;
    const rampingEnabledCount = actuators.filter(actuator => actuator.rampingEnabled).length;

    return {
      totalActuators: actuators.length,
      configuredActuators,
      enabledActuators,
      totalUpdates,
      averageUpdates,
      safetyEnabledCount,
      rampingEnabledCount,
    };
  }

  /**
   * Generate deterministic actuator pattern
   */
  static generateDeterministicPattern(seed: number, actuatorCount: number, tickCount: number): number[] {
    const pattern: number[] = [];
    
    for (let i = 0; i < actuatorCount; i++) {
      // Simple deterministic pattern based on seed, actuator index, and tick
      const value = Math.sin(seed + i * 0.1 + tickCount * 0.01) * 0.5 + 0.5;
      pattern.push(value);
    }
    
    return pattern;
  }

  /**
   * Create actuator test pattern
   */
  static createTestPattern(actuatorCount: number, patternType: 'sine' | 'square' | 'triangle' | 'random'): number[] {
    const pattern: number[] = [];
    
    switch (patternType) {
      case 'sine':
        for (let i = 0; i < actuatorCount; i++) {
          pattern.push(Math.sin(i * 0.1) * 0.5 + 0.5);
        }
        break;
      case 'square':
        for (let i = 0; i < actuatorCount; i++) {
          pattern.push(i % 2 === 0 ? 1.0 : 0.0);
        }
        break;
      case 'triangle':
        for (let i = 0; i < actuatorCount; i++) {
          pattern.push(Math.abs((i % 4) - 2) / 2);
        }
        break;
      case 'random':
        for (let i = 0; i < actuatorCount; i++) {
          pattern.push(Math.random());
        }
        break;
    }
    
    return pattern;
  }

  /**
   * Create actuator pattern steps
   */
  static createPatternSteps(values: number[], durationMs: number): ActuatorPatternStep[] {
    return values.map(value => ({
      value,
      durationMs,
    }));
  }

  /**
   * Validate actuator pattern
   */
  static validateActuatorPattern(pattern: ActuatorPattern): boolean {
    if (!pattern.name || pattern.name.length === 0) {
      return false;
    }
    if (pattern.stepCount !== pattern.steps.length) {
      return false;
    }
    if (pattern.steps.length === 0) {
      return false;
    }
    for (const step of pattern.steps) {
      if (step.durationMs === 0) {
        return false;
      }
    }
    return true;
  }
}

/**
 * Event types for the ActuatorBridge
 */
export interface ActuatorBridgeEvents {
  connected: () => void;
  disconnected: () => void;
  error: (error: Error) => void;
  actuatorConfigured: (data: { name: string; config: ActuatorConfig; state: ActuatorState }) => void;
  actuatorEnabled: (data: { name: string; timestamp: number }) => void;
  actuatorDisabled: (data: { name: string; timestamp: number }) => void;
  actuatorValueSet: (data: { name: string; value: number; timestamp: number }) => void;
  actuatorPatternSet: (data: { name: string; pattern: ActuatorPattern; timestamp: number }) => void;
  actuatorPatternStopped: (data: { name: string; timestamp: number }) => void;
  deterministicModeEnabled: () => void;
  deterministicModeDisabled: () => void;
  tickAdvanced: (data: { tickCount: number }) => void;
  snapshotCreated: (data: { snapshot: ActuatorSnapshot }) => void;
  snapshotRestored: (data: { snapshotId: string }) => void;
}

/**
 * Default export
 */
export default ActuatorBridge;
