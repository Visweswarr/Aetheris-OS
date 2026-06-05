/**
 * TypeScript SDK for Aetheris OS ADC Operations
 * 
 * This module provides client-side interaction with the Aetheris OS ADC system,
 * including channel configuration, analog sampling, and deterministic state management.
 */

import { EventEmitter } from 'events';

// Type definitions
export interface AdcConfig {
  channel: number;
  sampleRate: number;
  resolution: number;
  referenceVoltage: number;
  enableCalibration: boolean;
  calibrationOffset: number;
  calibrationScale: number;
  oversampling: number;
  enableFiltering: boolean;
  filterCutoff: number;
}

export interface AdcState {
  channel: number;
  configured: boolean;
  enabled: boolean;
  sampleRate: number;
  resolution: number;
  referenceVoltage: number;
  lastSample: number;
  lastSampleTime: number;
  sampleCount: number;
  minValue: number;
  maxValue: number;
  averageValue: number;
  calibrationEnabled: boolean;
  calibrationOffset: number;
  calibrationScale: number;
}

export interface AdcSample {
  channel: number;
  value: number;
  rawValue: number;
  timestamp: number;
  deterministic: boolean;
  tickCount: number;
}

export interface AdcOperationResult {
  operationId: string;
  channel: number;
  operation: string;
  result: boolean;
  timestamp: number;
  deterministic: boolean;
}

export interface AdcSnapshot {
  snapshotId: string;
  ngfsPath: string;
  createdAt: Date;
  channelCount: number;
  sampleCount: number;
  metadata: Record<string, any>;
}

export interface AdcConfig {
  endpoint: string;
  timeout: number;
  retries: number;
  enableLogging: boolean;
  enablePolicyEnforcement: boolean;
  enableCapabilityChecking: boolean;
  defaultSampleRate: number;
  maxSampleRate: number;
  defaultResolution: number;
  maxResolution: number;
  defaultReferenceVoltage: number;
  snapshotTimeout: number;
}

export interface AdcStats {
  connectionStatus: 'connected' | 'disconnected' | 'connecting' | 'error';
  totalChannels: number;
  configuredChannels: number;
  enabledChannels: number;
  totalOperations: number;
  sampleOperations: number;
  configureOperations: number;
  enableOperations: number;
  disableOperations: number;
  lastSampleTime: number;
  totalSamples: number;
  averageSampleRate: number;
  deterministicMode: boolean;
  tickCounter: number;
}

/**
 * Main ADC Bridge class
 */
export class AdcBridge extends EventEmitter {
  private config: AdcConfig;
  private isConnected: boolean = false;
  private connectionRetries: number = 0;
  private configuredChannels: Map<number, AdcState> = new Map();
  private operationHistory: AdcOperationResult[] = [];

  constructor(config: Partial<AdcConfig> = {}) {
    super();
    
    this.config = {
      endpoint: config.endpoint || 'http://localhost:8080',
      timeout: config.timeout || 30000,
      retries: config.retries || 3,
      enableLogging: config.enableLogging || false,
      enablePolicyEnforcement: config.enablePolicyEnforcement || true,
      enableCapabilityChecking: config.enableCapabilityChecking || true,
      defaultSampleRate: config.defaultSampleRate || 1000,
      maxSampleRate: config.maxSampleRate || 10000,
      defaultResolution: config.defaultResolution || 12,
      maxResolution: config.maxResolution || 16,
      defaultReferenceVoltage: config.defaultReferenceVoltage || 3.3,
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
   * Connect to the ADC service
   */
  private async connect(): Promise<void> {
    const response = await fetch(`${this.config.endpoint}/adc/health`, {
      method: 'GET',
      timeout: this.config.timeout,
    });

    if (!response.ok) {
      throw new Error(`Failed to connect to ADC service: ${response.statusText}`);
    }
  }

  /**
   * Configure ADC channel
   */
  async configureChannel(sessionId: string, caps: string, config: AdcConfig): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/configure', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        channel: config.channel,
        sample_rate: config.sampleRate,
        resolution: config.resolution,
        reference_voltage: config.referenceVoltage,
        enable_calibration: config.enableCalibration,
        calibration_offset: config.calibrationOffset,
        calibration_scale: config.calibrationScale,
        oversampling: config.oversampling,
        enable_filtering: config.enableFiltering,
        filter_cutoff: config.filterCutoff,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to configure ADC channel: ${response.error}`);
    }

    const state: AdcState = {
      channel: config.channel,
      configured: true,
      enabled: false,
      sampleRate: config.sampleRate,
      resolution: config.resolution,
      referenceVoltage: config.referenceVoltage,
      lastSample: 0.0,
      lastSampleTime: Date.now(),
      sampleCount: 0,
      minValue: 0.0,
      maxValue: 0.0,
      averageValue: 0.0,
      calibrationEnabled: config.enableCalibration,
      calibrationOffset: config.calibrationOffset,
      calibrationScale: config.calibrationScale,
    };

    this.configuredChannels.set(config.channel, state);

    this.emit('channelConfigured', {
      channel: config.channel,
      config,
      state,
    });
  }

  /**
   * Enable ADC channel
   */
  async enableChannel(sessionId: string, caps: string, channel: number): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/enable', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        channel,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to enable ADC channel: ${response.error}`);
    }

    // Update local state
    const state = this.configuredChannels.get(channel);
    if (state) {
      state.enabled = true;
    }

    this.emit('channelEnabled', {
      channel,
      timestamp: Date.now(),
    });
  }

  /**
   * Disable ADC channel
   */
  async disableChannel(sessionId: string, caps: string, channel: number): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/disable', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        channel,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to disable ADC channel: ${response.error}`);
    }

    // Update local state
    const state = this.configuredChannels.get(channel);
    if (state) {
      state.enabled = false;
    }

    this.emit('channelDisabled', {
      channel,
      timestamp: Date.now(),
    });
  }

  /**
   * Sample ADC channel
   */
  async sampleChannel(sessionId: string, caps: string, channel: number): Promise<AdcSample> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/sample', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        channel,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to sample ADC channel: ${response.error}`);
    }

    const sample: AdcSample = {
      channel: response.channel,
      value: response.value,
      rawValue: response.raw_value,
      timestamp: response.timestamp,
      deterministic: response.deterministic,
      tickCount: response.tick_count,
    };

    // Update local state
    const state = this.configuredChannels.get(channel);
    if (state) {
      state.lastSample = sample.value;
      state.lastSampleTime = sample.timestamp;
      state.sampleCount++;
      
      if (state.sampleCount === 1) {
        state.minValue = sample.value;
        state.maxValue = sample.value;
        state.averageValue = sample.value;
      } else {
        if (sample.value < state.minValue) {
          state.minValue = sample.value;
        }
        if (sample.value > state.maxValue) {
          state.maxValue = sample.value;
        }
        // Simple running average
        state.averageValue = (state.averageValue * (state.sampleCount - 1) + sample.value) / state.sampleCount;
      }
    }

    this.emit('channelSampled', {
      channel,
      sample,
    });

    return sample;
  }

  /**
   * Sample multiple ADC channels
   */
  async sampleChannels(sessionId: string, caps: string, channels: number[]): Promise<AdcSample[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/sample-multi', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        channels,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to sample ADC channels: ${response.error}`);
    }

    const samples: AdcSample[] = (response.samples || []).map((sample: any) => ({
      channel: sample.channel,
      value: sample.value,
      rawValue: sample.raw_value,
      timestamp: sample.timestamp,
      deterministic: sample.deterministic,
      tickCount: sample.tick_count,
    }));

    // Update local states
    samples.forEach(sample => {
      const state = this.configuredChannels.get(sample.channel);
      if (state) {
        state.lastSample = sample.value;
        state.lastSampleTime = sample.timestamp;
        state.sampleCount++;
        
        if (state.sampleCount === 1) {
          state.minValue = sample.value;
          state.maxValue = sample.value;
          state.averageValue = sample.value;
        } else {
          if (sample.value < state.minValue) {
            state.minValue = sample.value;
          }
          if (sample.value > state.maxValue) {
            state.maxValue = sample.value;
          }
          state.averageValue = (state.averageValue * (state.sampleCount - 1) + sample.value) / state.sampleCount;
        }
      }
    });

    this.emit('channelsSampled', {
      channels,
      samples,
    });

    return samples;
  }

  /**
   * Get ADC channel state
   */
  async getChannelState(channel: number): Promise<AdcState | null> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/adc/state/${channel}`, {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to get ADC channel state: ${response.error}`);
    }

    const state: AdcState = {
      channel: response.channel,
      configured: response.configured,
      enabled: response.enabled,
      sampleRate: response.sample_rate,
      resolution: response.resolution,
      referenceVoltage: response.reference_voltage,
      lastSample: response.last_sample,
      lastSampleTime: response.last_sample_time,
      sampleCount: response.sample_count,
      minValue: response.min_value,
      maxValue: response.max_value,
      averageValue: response.average_value,
      calibrationEnabled: response.calibration_enabled,
      calibrationOffset: response.calibration_offset,
      calibrationScale: response.calibration_scale,
    };

    this.configuredChannels.set(channel, state);

    return state;
  }

  /**
   * List all configured ADC channels
   */
  async listConfiguredChannels(): Promise<AdcState[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/list', {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to list ADC channels: ${response.error}`);
    }

    const channels: AdcState[] = (response.channels || []).map((channel: any) => ({
      channel: channel.channel,
      configured: channel.configured,
      enabled: channel.enabled,
      sampleRate: channel.sample_rate,
      resolution: channel.resolution,
      referenceVoltage: channel.reference_voltage,
      lastSample: channel.last_sample,
      lastSampleTime: channel.last_sample_time,
      sampleCount: channel.sample_count,
      minValue: channel.min_value,
      maxValue: channel.max_value,
      averageValue: channel.average_value,
      calibrationEnabled: channel.calibration_enabled,
      calibrationOffset: channel.calibration_offset,
      calibrationScale: channel.calibration_scale,
    }));

    // Update local cache
    channels.forEach(channel => {
      this.configuredChannels.set(channel.channel, channel);
    });

    return channels;
  }

  /**
   * Get ADC operation history
   */
  async getOperationHistory(limit: number = 100): Promise<AdcOperationResult[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/history', {
      method: 'GET',
      body: JSON.stringify({
        limit,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to get ADC operation history: ${response.error}`);
    }

    const operations: AdcOperationResult[] = (response.operations || []).map((op: any) => ({
      operationId: op.operation_id,
      channel: op.channel,
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

    const response = await this.makeRequest('/adc/deterministic/enable', {
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

    const response = await this.makeRequest('/adc/deterministic/disable', {
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

    const response = await this.makeRequest('/adc/tick/advance', {
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

    const response = await this.makeRequest('/adc/tick/counter', {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to get tick counter: ${response.error}`);
    }

    return response.tick_count;
  }

  /**
   * Create ADC snapshot
   */
  async createSnapshot(outputPath: string): Promise<string> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/snapshot/create', {
      method: 'POST',
      body: JSON.stringify({
        output_path: outputPath,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to create ADC snapshot: ${response.error}`);
    }

    const snapshot: AdcSnapshot = {
      snapshotId: response.snapshot_id,
      ngfsPath: outputPath,
      createdAt: new Date(),
      channelCount: response.channel_count || 0,
      sampleCount: response.sample_count || 0,
      metadata: response.metadata || {},
    };

    this.emit('snapshotCreated', {
      snapshot,
    });

    return response.snapshot_id;
  }

  /**
   * Restore ADC snapshot
   */
  async restoreSnapshot(snapshotData: Uint8Array): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/snapshot/restore', {
      method: 'POST',
      body: snapshotData,
      headers: {
        'Content-Type': 'application/octet-stream',
      },
    });

    if (!response.success) {
      throw new Error(`Failed to restore ADC snapshot: ${response.error}`);
    }

    this.emit('snapshotRestored', {
      snapshotId: response.snapshot_id,
    });
  }

  /**
   * Get ADC statistics
   */
  async getStats(): Promise<AdcStats> {
    await this.ensureConnected();

    const response = await this.makeRequest('/adc/stats', {
      method: 'GET',
    });

    return response;
  }

  /**
   * Get configured channels
   */
  getConfiguredChannels(): Map<number, AdcState> {
    return new Map(this.configuredChannels);
  }

  /**
   * Get operation history
   */
  getOperationHistory(): AdcOperationResult[] {
    return [...this.operationHistory];
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<AdcConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfig(): AdcConfig {
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
 * Utility functions for working with ADC
 */
export class AdcUtils {
  /**
   * Get default ADC configuration
   */
  static getDefaultConfig(channel: number): AdcConfig {
    return {
      channel,
      sampleRate: 1000,
      resolution: 12,
      referenceVoltage: 3.3,
      enableCalibration: false,
      calibrationOffset: 0.0,
      calibrationScale: 1.0,
      oversampling: 1,
      enableFiltering: false,
      filterCutoff: 100.0,
    };
  }

  /**
   * Validate ADC channel number
   */
  static validateChannelNumber(channel: number): boolean {
    return channel >= 0 && channel <= 15;
  }

  /**
   * Validate sample rate
   */
  static validateSampleRate(sampleRate: number): boolean {
    return sampleRate > 0 && sampleRate <= 10000;
  }

  /**
   * Validate resolution
   */
  static validateResolution(resolution: number): boolean {
    return resolution >= 8 && resolution <= 16;
  }

  /**
   * Validate reference voltage
   */
  static validateReferenceVoltage(voltage: number): boolean {
    return voltage > 0 && voltage <= 5.0;
  }

  /**
   * Convert raw ADC value to voltage
   */
  static rawToVoltage(rawValue: number, resolution: number, referenceVoltage: number): number {
    const maxValue = Math.pow(2, resolution) - 1;
    return (rawValue / maxValue) * referenceVoltage;
  }

  /**
   * Convert voltage to raw ADC value
   */
  static voltageToRaw(voltage: number, resolution: number, referenceVoltage: number): number {
    const maxValue = Math.pow(2, resolution) - 1;
    return Math.round((voltage / referenceVoltage) * maxValue);
  }

  /**
   * Apply calibration to ADC value
   */
  static applyCalibration(value: number, offset: number, scale: number): number {
    return (value + offset) * scale;
  }

  /**
   * Remove calibration from ADC value
   */
  static removeCalibration(value: number, offset: number, scale: number): number {
    return (value / scale) - offset;
  }

  /**
   * Calculate ADC statistics
   */
  static calculateStats(samples: AdcSample[]): {
    count: number;
    min: number;
    max: number;
    mean: number;
    stdDev: number;
    minRaw: number;
    maxRaw: number;
    meanRaw: number;
  } {
    if (samples.length === 0) {
      return {
        count: 0,
        min: 0,
        max: 0,
        mean: 0,
        stdDev: 0,
        minRaw: 0,
        maxRaw: 0,
        meanRaw: 0,
      };
    }

    const values = samples.map(s => s.value);
    const rawValues = samples.map(s => s.rawValue);

    const min = Math.min(...values);
    const max = Math.max(...values);
    const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
    const stdDev = Math.sqrt(variance);

    const minRaw = Math.min(...rawValues);
    const maxRaw = Math.max(...rawValues);
    const meanRaw = rawValues.reduce((sum, val) => sum + val, 0) / rawValues.length;

    return {
      count: samples.length,
      min,
      max,
      mean,
      stdDev,
      minRaw,
      maxRaw,
      meanRaw,
    };
  }

  /**
   * Generate deterministic ADC data
   */
  static generateDeterministicData(seed: number, channel: number, tickCount: number, referenceVoltage: number): AdcSample {
    // Simple deterministic generator based on seed, channel, and tick
    const rawValue = Math.sin(seed + channel * 0.1 + tickCount * 0.01) * 0.5 + 0.5;
    const value = rawValue * referenceVoltage;

    return {
      channel,
      value,
      rawValue,
      timestamp: Date.now(),
      deterministic: true,
      tickCount,
    };
  }

  /**
   * Create ADC test pattern
   */
  static createTestPattern(channelCount: number, patternType: 'sine' | 'square' | 'triangle' | 'random'): number[] {
    const pattern: number[] = [];
    
    switch (patternType) {
      case 'sine':
        for (let i = 0; i < channelCount; i++) {
          pattern.push(Math.sin(i * 0.1) * 0.5 + 0.5);
        }
        break;
      case 'square':
        for (let i = 0; i < channelCount; i++) {
          pattern.push(i % 2 === 0 ? 1.0 : 0.0);
        }
        break;
      case 'triangle':
        for (let i = 0; i < channelCount; i++) {
          pattern.push(Math.abs((i % 4) - 2) / 2);
        }
        break;
      case 'random':
        for (let i = 0; i < channelCount; i++) {
          pattern.push(Math.random());
        }
        break;
    }
    
    return pattern;
  }
}

/**
 * Event types for the AdcBridge
 */
export interface AdcBridgeEvents {
  connected: () => void;
  disconnected: () => void;
  error: (error: Error) => void;
  channelConfigured: (data: { channel: number; config: AdcConfig; state: AdcState }) => void;
  channelEnabled: (data: { channel: number; timestamp: number }) => void;
  channelDisabled: (data: { channel: number; timestamp: number }) => void;
  channelSampled: (data: { channel: number; sample: AdcSample }) => void;
  channelsSampled: (data: { channels: number[]; samples: AdcSample[] }) => void;
  deterministicModeEnabled: () => void;
  deterministicModeDisabled: () => void;
  tickAdvanced: (data: { tickCount: number }) => void;
  snapshotCreated: (data: { snapshot: AdcSnapshot }) => void;
  snapshotRestored: (data: { snapshotId: string }) => void;
}

/**
 * Default export
 */
export default AdcBridge;
