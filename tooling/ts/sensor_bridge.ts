/**
 * TypeScript SDK for Aetheris OS Sensor Operations
 * 
 * This module provides client-side interaction with the Aetheris OS sensor system,
 * including sensor registration, sampling, preview streams, and snapshot creation.
 */

import { EventEmitter } from 'events';

// Type definitions
export type SensorKind = 'accelerometer' | 'gyroscope' | 'magnetometer' | 'temperature' | 'humidity' | 'pressure' | 'light' | 'proximity' | 'heart_rate' | 'custom';

export type SensorValueType = 'scalar' | 'vector2' | 'vector3';

export interface SensorValue {
  type: SensorValueType;
  value: number | [number, number] | [number, number, number];
}

export interface SensorDesc {
  name: string;
  kind: SensorKind;
  location: string;
  unit: string;
  rangeMin: number;
  rangeMax: number;
  resolution: number;
  sampleRates: number[];
}

export interface SensorInfo {
  sensorId: string;
  name: string;
  kind: SensorKind;
  location: string;
  unit: string;
  rangeMin: number;
  rangeMax: number;
  resolution: number;
  sampleRates: number[];
  registeredAt: string;
  activeSampling: boolean;
}

export interface SensorSample {
  sensorId: string;
  timestamp: Date;
  value: SensorValue;
  sequenceNumber: number;
}

export interface SamplingSession {
  handle: string;
  sensorId: string;
  sessionId: string;
  hz: number;
  seed: number;
  startTime: Date;
  sampleCount: number;
  deterministic: boolean;
}

export interface PreviewStream {
  streamHandle: string;
  sensorId: string;
  hzMax: number;
  startTime: Date;
  sampleCount: number;
}

export interface SensorSnapshot {
  snapshotId: string;
  sensorId: string;
  ngfsPath: string;
  createdAt: Date;
  sampleCount: number;
  duration: number;
  metadata: Record<string, any>;
}

export interface SensorConfig {
  endpoint: string;
  timeout: number;
  retries: number;
  enableLogging: boolean;
  enablePolicyEnforcement: boolean;
  enableCapabilityChecking: boolean;
  defaultSampleRate: number;
  maxSampleRate: number;
  previewMaxRate: number;
  snapshotTimeout: number;
}

export interface SensorStats {
  connectionStatus: 'connected' | 'disconnected' | 'connecting' | 'error';
  totalSensors: number;
  activeSensors: number;
  totalSessions: number;
  activeSessions: number;
  totalPreviews: number;
  activePreviews: number;
  totalSnapshots: number;
  totalSamples: number;
  averageSampleRate: number;
  lastSampleTime: number;
  lastSnapshotTime: number;
}

/**
 * Main Sensor Bridge class
 */
export class SensorBridge extends EventEmitter {
  private config: SensorConfig;
  private isConnected: boolean = false;
  private connectionRetries: number = 0;
  private activeSessions: Map<string, SamplingSession> = new Map();
  private activePreviews: Map<string, PreviewStream> = new Map();
  private registeredSensors: Map<string, SensorInfo> = new Map();

  constructor(config: Partial<SensorConfig> = {}) {
    super();
    
    this.config = {
      endpoint: config.endpoint || 'http://localhost:8080',
      timeout: config.timeout || 30000,
      retries: config.retries || 3,
      enableLogging: config.enableLogging || false,
      enablePolicyEnforcement: config.enablePolicyEnforcement || true,
      enableCapabilityChecking: config.enableCapabilityChecking || true,
      defaultSampleRate: config.defaultSampleRate || 10,
      maxSampleRate: config.maxSampleRate || 1000,
      previewMaxRate: config.previewMaxRate || 5,
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
   * Connect to the sensor service
   */
  private async connect(): Promise<void> {
    const response = await fetch(`${this.config.endpoint}/sensor/health`, {
      method: 'GET',
      timeout: this.config.timeout,
    });

    if (!response.ok) {
      throw new Error(`Failed to connect to sensor service: ${response.statusText}`);
    }
  }

  /**
   * Register a new sensor
   */
  async registerSensor(desc: SensorDesc): Promise<string> {
    await this.ensureConnected();

    const response = await this.makeRequest('/sensor/register', {
      method: 'POST',
      body: JSON.stringify({
        name: desc.name,
        kind: desc.kind,
        location: desc.location,
        unit: desc.unit,
        range_min: desc.rangeMin,
        range_max: desc.rangeMax,
        resolution: desc.resolution,
        sample_rates: desc.sampleRates,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to register sensor: ${response.error}`);
    }

    const sensorInfo: SensorInfo = {
      sensorId: response.sensor_id,
      name: desc.name,
      kind: desc.kind,
      location: desc.location,
      unit: desc.unit,
      rangeMin: desc.rangeMin,
      rangeMax: desc.rangeMax,
      resolution: desc.resolution,
      sampleRates: desc.sampleRates,
      registeredAt: new Date().toISOString(),
      activeSampling: false,
    };

    this.registeredSensors.set(response.sensor_id, sensorInfo);

    this.emit('sensorRegistered', {
      sensorId: response.sensor_id,
      sensorInfo,
    });

    return response.sensor_id;
  }

  /**
   * List all registered sensors
   */
  async listSensors(): Promise<SensorInfo[]> {
    await this.ensureConnected();

    const response = await this.makeRequest('/sensor/list', {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to list sensors: ${response.error}`);
    }

    const sensors: SensorInfo[] = response.sensors || [];
    
    // Update local cache
    sensors.forEach(sensor => {
      this.registeredSensors.set(sensor.sensorId, sensor);
    });

    return sensors;
  }

  /**
   * Start sensor sampling
   */
  async startSampling(sessionId: string, caps: string, sensorId: string, hz: number, seed: number = 0): Promise<string> {
    await this.ensureConnected();

    const handle = this.generateSessionHandle();
    
    const response = await this.makeRequest('/sensor/sample/start', {
      method: 'POST',
      body: JSON.stringify({
        session_id: sessionId,
        caps,
        sensor_id: sensorId,
        hz,
        seed,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to start sensor sampling: ${response.error}`);
    }

    const session: SamplingSession = {
      handle,
      sensorId,
      sessionId,
      hz,
      seed,
      startTime: new Date(),
      sampleCount: 0,
      deterministic: seed !== 0,
    };

    this.activeSessions.set(handle, session);

    // Update sensor status
    const sensor = this.registeredSensors.get(sensorId);
    if (sensor) {
      sensor.activeSampling = true;
    }

    this.emit('samplingStarted', {
      handle,
      session,
    });

    return handle;
  }

  /**
   * Stop sensor sampling
   */
  async stopSampling(handle: string): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/sensor/sample/${handle}/stop`, {
      method: 'POST',
    });

    if (!response.success) {
      throw new Error(`Failed to stop sensor sampling: ${response.error}`);
    }

    const session = this.activeSessions.get(handle);
    if (session) {
      // Update sensor status
      const sensor = this.registeredSensors.get(session.sensorId);
      if (sensor) {
        sensor.activeSampling = false;
      }
      
      this.activeSessions.delete(handle);
    }

    this.emit('samplingStopped', {
      handle,
    });
  }

  /**
   * Start sensor preview stream
   */
  async startPreview(sensorId: string, hzMax: number): Promise<string> {
    await this.ensureConnected();

    const streamHandle = this.generateStreamHandle();
    
    const response = await this.makeRequest('/sensor/preview/start', {
      method: 'POST',
      body: JSON.stringify({
        sensor_id: sensorId,
        hz_max: hzMax,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to start sensor preview: ${response.error}`);
    }

    const stream: PreviewStream = {
      streamHandle,
      sensorId,
      hzMax,
      startTime: new Date(),
      sampleCount: 0,
    };

    this.activePreviews.set(streamHandle, stream);

    this.emit('previewStarted', {
      streamHandle,
      stream,
    });

    return streamHandle;
  }

  /**
   * Stop sensor preview stream
   */
  async stopPreview(streamHandle: string): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/sensor/preview/${streamHandle}/stop`, {
      method: 'POST',
    });

    if (!response.success) {
      throw new Error(`Failed to stop sensor preview: ${response.error}`);
    }

    this.activePreviews.delete(streamHandle);

    this.emit('previewStopped', {
      streamHandle,
    });
  }

  /**
   * Create sensor snapshot
   */
  async createSnapshot(sensorId: string, outputPath: string): Promise<string> {
    await this.ensureConnected();

    const response = await this.makeRequest('/sensor/snapshot/create', {
      method: 'POST',
      body: JSON.stringify({
        sensor_id: sensorId,
        output_path: outputPath,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to create sensor snapshot: ${response.error}`);
    }

    const snapshot: SensorSnapshot = {
      snapshotId: response.snapshot_id,
      sensorId,
      ngfsPath: outputPath,
      createdAt: new Date(),
      sampleCount: response.sample_count || 0,
      duration: response.duration || 0,
      metadata: response.metadata || {},
    };

    this.emit('snapshotCreated', {
      snapshot,
    });

    return response.snapshot_id;
  }

  /**
   * Get sensor samples
   */
  async getSamples(handle: string, limit: number = 100): Promise<SensorSample[]> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/sensor/sample/${handle}/samples`, {
      method: 'GET',
      body: JSON.stringify({
        limit,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to get sensor samples: ${response.error}`);
    }

    const samples: SensorSample[] = (response.samples || []).map((sample: any) => ({
      sensorId: sample.sensor_id,
      timestamp: new Date(sample.timestamp),
      value: sample.value,
      sequenceNumber: sample.sequence_number,
    }));

    return samples;
  }

  /**
   * Get preview samples
   */
  async getPreviewSamples(streamHandle: string, limit: number = 50): Promise<SensorSample[]> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/sensor/preview/${streamHandle}/samples`, {
      method: 'GET',
      body: JSON.stringify({
        limit,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to get preview samples: ${response.error}`);
    }

    const samples: SensorSample[] = (response.samples || []).map((sample: any) => ({
      sensorId: sample.sensor_id,
      timestamp: new Date(sample.timestamp),
      value: sample.value,
      sequenceNumber: sample.sequence_number,
    }));

    return samples;
  }

  /**
   * Get sensor statistics
   */
  async getStats(): Promise<SensorStats> {
    await this.ensureConnected();

    const response = await this.makeRequest('/sensor/stats', {
      method: 'GET',
    });

    return response;
  }

  /**
   * Get active sessions
   */
  getActiveSessions(): Map<string, SamplingSession> {
    return new Map(this.activeSessions);
  }

  /**
   * Get active previews
   */
  getActivePreviews(): Map<string, PreviewStream> {
    return new Map(this.activePreviews);
  }

  /**
   * Get registered sensors
   */
  getRegisteredSensors(): Map<string, SensorInfo> {
    return new Map(this.registeredSensors);
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<SensorConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfig(): SensorConfig {
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
    // Stop all active sessions
    for (const [handle, session] of this.activeSessions) {
      try {
        await this.stopSampling(handle);
      } catch (error) {
        console.warn(`Failed to stop sampling session ${handle}:`, error);
      }
    }

    // Stop all active previews
    for (const [streamHandle, stream] of this.activePreviews) {
      try {
        await this.stopPreview(streamHandle);
      } catch (error) {
        console.warn(`Failed to stop preview stream ${streamHandle}:`, error);
      }
    }

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

  /**
   * Generate unique session handle
   */
  private generateSessionHandle(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate unique stream handle
   */
  private generateStreamHandle(): string {
    return `stream_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

/**
 * Utility functions for working with sensors
 */
export class SensorUtils {
  /**
   * Get default sensor location based on kind
   */
  static getDefaultLocation(kind: SensorKind): string {
    switch (kind) {
      case 'accelerometer':
      case 'gyroscope':
      case 'magnetometer':
        return 'IMU';
      case 'temperature':
        return 'CPU';
      case 'light':
        return 'Ambient';
      case 'humidity':
      case 'pressure':
        return 'Environment';
      case 'proximity':
        return 'Front';
      case 'heart_rate':
        return 'Wrist';
      default:
        return 'Unknown';
    }
  }

  /**
   * Get default sensor unit based on kind
   */
  static getDefaultUnit(kind: SensorKind): string {
    switch (kind) {
      case 'accelerometer':
        return 'm/s²';
      case 'gyroscope':
        return 'deg/s';
      case 'magnetometer':
        return 'μT';
      case 'temperature':
        return '°C';
      case 'humidity':
        return '%';
      case 'pressure':
        return 'Pa';
      case 'light':
        return 'lux';
      case 'proximity':
        return 'cm';
      case 'heart_rate':
        return 'bpm';
      default:
        return 'units';
    }
  }

  /**
   * Get default sensor range based on kind
   */
  static getDefaultRange(kind: SensorKind): [number, number] {
    switch (kind) {
      case 'accelerometer':
        return [-20.0, 20.0];
      case 'gyroscope':
        return [-1000.0, 1000.0];
      case 'magnetometer':
        return [-100.0, 100.0];
      case 'temperature':
        return [0.0, 100.0];
      case 'humidity':
        return [0.0, 100.0];
      case 'pressure':
        return [80000.0, 120000.0];
      case 'light':
        return [0.0, 1000.0];
      case 'proximity':
        return [0.0, 100.0];
      case 'heart_rate':
        return [30.0, 200.0];
      default:
        return [0.0, 100.0];
    }
  }

  /**
   * Get default sensor resolution based on kind
   */
  static getDefaultResolution(kind: SensorKind): number {
    switch (kind) {
      case 'accelerometer':
        return 0.01;
      case 'gyroscope':
        return 0.1;
      case 'magnetometer':
        return 0.1;
      case 'temperature':
        return 0.1;
      case 'humidity':
        return 0.1;
      case 'pressure':
        return 1.0;
      case 'light':
        return 1.0;
      case 'proximity':
        return 0.1;
      case 'heart_rate':
        return 1.0;
      default:
        return 1.0;
    }
  }

  /**
   * Get default sample rates based on kind
   */
  static getDefaultSampleRates(kind: SensorKind): number[] {
    switch (kind) {
      case 'accelerometer':
        return [10, 50, 100, 200];
      case 'gyroscope':
        return [10, 100, 1000];
      case 'magnetometer':
        return [1, 10, 50];
      case 'temperature':
        return [1, 10, 100];
      case 'humidity':
        return [1, 10];
      case 'pressure':
        return [1, 10, 50];
      case 'light':
        return [1, 5, 10];
      case 'proximity':
        return [1, 10];
      case 'heart_rate':
        return [1, 10];
      default:
        return [1, 10];
    }
  }

  /**
   * Validate sensor value
   */
  static validateSensorValue(value: SensorValue, rangeMin: number, rangeMax: number): boolean {
    const checkValue = (val: number) => val >= rangeMin && val <= rangeMax;
    
    switch (value.type) {
      case 'scalar':
        return checkValue(value.value as number);
      case 'vector2':
        const [x, y] = value.value as [number, number];
        return checkValue(x) && checkValue(y);
      case 'vector3':
        const [x3, y3, z3] = value.value as [number, number, number];
        return checkValue(x3) && checkValue(y3) && checkValue(z3);
      default:
        return false;
    }
  }

  /**
   * Convert sensor value to string
   */
  static sensorValueToString(value: SensorValue): string {
    switch (value.type) {
      case 'scalar':
        return (value.value as number).toString();
      case 'vector2':
        const [x, y] = value.value as [number, number];
        return `(${x}, ${y})`;
      case 'vector3':
        const [x3, y3, z3] = value.value as [number, number, number];
        return `(${x3}, ${y3}, ${z3})`;
      default:
        return 'unknown';
    }
  }

  /**
   * Calculate sensor statistics
   */
  static calculateStats(samples: SensorSample[]): {
    count: number;
    min: number;
    max: number;
    mean: number;
    stdDev: number;
  } {
    if (samples.length === 0) {
      return { count: 0, min: 0, max: 0, mean: 0, stdDev: 0 };
    }

    const values: number[] = [];
    
    samples.forEach(sample => {
      switch (sample.value.type) {
        case 'scalar':
          values.push(sample.value.value as number);
          break;
        case 'vector2':
          const [x, y] = sample.value.value as [number, number];
          values.push(x, y);
          break;
        case 'vector3':
          const [x3, y3, z3] = sample.value.value as [number, number, number];
          values.push(x3, y3, z3);
          break;
      }
    });

    if (values.length === 0) {
      return { count: 0, min: 0, max: 0, mean: 0, stdDev: 0 };
    }

    const min = Math.min(...values);
    const max = Math.max(...values);
    const mean = values.reduce((sum, val) => sum + val, 0) / values.length;
    const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
    const stdDev = Math.sqrt(variance);

    return { count: values.length, min, max, mean, stdDev };
  }

  /**
   * Generate deterministic sensor data
   */
  static generateDeterministicData(kind: SensorKind, seed: number, timestamp: number): SensorValue {
    // Simple deterministic generator based on seed and timestamp
    const x = Math.sin(seed + timestamp * 0.001) * 10;
    const y = Math.cos(seed + timestamp * 0.001) * 10;
    const z = Math.sin(seed + timestamp * 0.002) * 5;

    switch (kind) {
      case 'accelerometer':
        return { type: 'vector3', value: [x, y, z] };
      case 'gyroscope':
        return { type: 'vector3', value: [x * 10, y * 10, z * 10] };
      case 'magnetometer':
        return { type: 'vector3', value: [x * 0.1, y * 0.1, z * 0.1] };
      case 'temperature':
        return { type: 'scalar', value: 20 + Math.sin(seed + timestamp * 0.0001) * 5 };
      case 'humidity':
        return { type: 'scalar', value: 50 + Math.sin(seed + timestamp * 0.0001) * 20 };
      case 'pressure':
        return { type: 'scalar', value: 101325 + Math.sin(seed + timestamp * 0.0001) * 1000 };
      case 'light':
        return { type: 'scalar', value: 100 + Math.sin(seed + timestamp * 0.0001) * 50 };
      case 'proximity':
        return { type: 'scalar', value: 10 + Math.sin(seed + timestamp * 0.0001) * 5 };
      case 'heart_rate':
        return { type: 'scalar', value: 70 + Math.sin(seed + timestamp * 0.0001) * 10 };
      default:
        return { type: 'scalar', value: Math.sin(seed + timestamp * 0.001) * 10 };
    }
  }
}

/**
 * Event types for the SensorBridge
 */
export interface SensorBridgeEvents {
  connected: () => void;
  disconnected: () => void;
  error: (error: Error) => void;
  sensorRegistered: (data: { sensorId: string; sensorInfo: SensorInfo }) => void;
  samplingStarted: (data: { handle: string; session: SamplingSession }) => void;
  samplingStopped: (data: { handle: string }) => void;
  previewStarted: (data: { streamHandle: string; stream: PreviewStream }) => void;
  previewStopped: (data: { streamHandle: string }) => void;
  snapshotCreated: (data: { snapshot: SensorSnapshot }) => void;
}

/**
 * Default export
 */
export default SensorBridge;
