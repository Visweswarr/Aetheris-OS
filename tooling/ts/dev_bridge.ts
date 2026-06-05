/**
 * @file dev_bridge.ts
 * @brief TypeScript bridge for Aetheris Device Runtime
 * 
 * This module provides a high-level, event-driven API for device capture operations,
 * including camera and microphone capture with capability gating, deterministic recording,
 * and NGFS snapshot integration.
 */

// Mock implementations for browser compatibility
class EventEmitter {
  private listeners: Map<string, Function[]> = new Map();
  
  on(event: string, listener: Function): this {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(listener);
    return this;
  }
  
  emit(event: string, ...args: any[]): boolean {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      eventListeners.forEach(listener => listener(...args));
      return true;
    }
    return false;
  }
  
  removeListener(event: string, listener: Function): this {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      const index = eventListeners.indexOf(listener);
      if (index > -1) {
        eventListeners.splice(index, 1);
      }
    }
    return this;
  }
}

// Mock UUID implementation
function uuidv4(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

// Type definitions
export interface CaptureConfig {
  // Camera configuration
  width?: number;
  height?: number;
  fps?: number;
  
  // Microphone configuration
  sampleRate?: number;
  channels?: number;
  
  // Common configuration
  deterministic: boolean;
  maxChunkDurationMs: number;
  maxChunkFrames: number;
}

export interface CaptureStats {
  captureId: string;
  durationMs: number;
  bytesWritten: number;
  chunksCreated: number;
  lastChunkTimestamp: Date;
  snapshotId?: string;
}

export interface CaptureStatus {
  captureId: string;
  running: boolean;
  bytesWritten: number;
  lastTimestamp: Date;
}

export interface PreviewFrame {
  frameData: ArrayBuffer;
  width: number;
  height: number;
  timestamp: Date;
}

export interface DaoPolicy {
  roomId: string;
  allowCamera: boolean;
  allowMicrophone: boolean;
  allowPreview: boolean;
}

export interface DeviceEvents {
  // Camera events
  camera_capture_started: (captureId: string, sessionId: string) => void;
  camera_capture_stopped: (captureId: string, stats: CaptureStats) => void;
  camera_preview_frame: (captureId: string, frame: PreviewFrame) => void;
  
  // Microphone events
  microphone_capture_started: (captureId: string, sessionId: string) => void;
  microphone_capture_stopped: (captureId: string, stats: CaptureStats) => void;
  
  // General events
  capture_error: (captureId: string, error: string) => void;
  policy_changed: (roomId: string, policy: DaoPolicy) => void;
}

export interface DeviceConfig {
  serviceUrl: string;
  connectionTimeout: number;
  previewThrottleFps: number;
  maxConcurrentCaptures: number;
  enableDeterministicMode: boolean;
}

export interface DeviceStats {
  active_captures: number;
  total_captures_started: number;
  total_captures_stopped: number;
  total_bytes_captured: number;
  average_capture_duration_ms: number;
  preview_frames_sent: number;
  policy_checks: number;
  capability_validations: number;
}

/**
 * Device Bridge - Main interface for device capture operations
 */
export class DeviceBridge extends EventEmitter {
  private config: DeviceConfig;
  private connected: boolean = false;
  private reconnectAttempts: number = 0;
  private _reconnectTimer?: NodeJS.Timeout;
  private previewTimer?: NodeJS.Timeout;
  private _deviceServiceClient?: any; // gRPC client
  private currentSessionId?: string;
  private currentCaps?: string;
  private activeCaptures: Map<string, CaptureInfo> = new Map();
  private stats: DeviceStats;

  constructor(config: Partial<DeviceConfig> = {}) {
    super();
    
    this.config = {
      serviceUrl: config.serviceUrl || 'localhost:50052',
      connectionTimeout: config.connectionTimeout || 5000,
      previewThrottleFps: config.previewThrottleFps || 5,
      maxConcurrentCaptures: config.maxConcurrentCaptures || 5,
      enableDeterministicMode: config.enableDeterministicMode || false,
    };
    
    this.stats = {
      active_captures: 0,
      total_captures_started: 0,
      total_captures_stopped: 0,
      total_bytes_captured: 0,
      average_capture_duration_ms: 0,
      preview_frames_sent: 0,
      policy_checks: 0,
      capability_validations: 0,
    };
  }

  /**
   * Connect to the device service
   */
  async connect(): Promise<boolean> {
    try {
      // TODO: Initialize gRPC client
      this.connected = true;
      this.reconnectAttempts = 0;
      
      this.emit('connected');
      return true;
    } catch (error) {
      this.emit('connection_error', error);
      return false;
    }
  }

  /**
   * Disconnect from the device service
   */
  async disconnect(): Promise<void> {
    this.connected = false;
    
    // Stop all active captures
    for (const [captureId] of this.activeCaptures) {
      await this.stopCapture(captureId);
    }
    
    this.emit('disconnected');
  }

  /**
   * Set current session and capabilities
   */
  setSession(sessionId: string, caps: string): void {
    this.currentSessionId = sessionId;
    this.currentCaps = caps;
  }

  /**
   * Start camera capture
   */
  async startCameraCapture(config: CaptureConfig): Promise<string> {
    if (!this.connected) {
      throw new Error('Not connected to device service');
    }

    if (!this.currentSessionId || !this.currentCaps) {
      throw new Error('No active session. Call setSession() first.');
    }

    if (this.activeCaptures.size >= this.config.maxConcurrentCaptures) {
      throw new Error('Maximum concurrent captures reached');
    }

    const captureId = uuidv4();
    
    // Mock implementation
    const captureInfo: CaptureInfo = {
      captureId,
      sessionId: this.currentSessionId,
      deviceType: 'camera',
      config,
      startTime: new Date(),
      deterministic: config.deterministic,
    };

    this.activeCaptures.set(captureId, captureInfo);
    this.stats.active_captures++;
    this.stats.total_captures_started++;

    // Start preview timer if not already running
    if (!this.previewTimer) {
      this.startPreviewTimer();
    }

    this.emit('camera_capture_started', captureId, this.currentSessionId);
    return captureId;
  }

  /**
   * Stop camera capture
   */
  async stopCameraCapture(captureId: string): Promise<CaptureStats> {
    const captureInfo = this.activeCaptures.get(captureId);
    if (!captureInfo) {
      throw new Error(`Capture not found: ${captureId}`);
    }

    const duration = Date.now() - captureInfo.startTime.getTime();
    const stats: CaptureStats = {
      captureId,
      durationMs: duration,
      bytesWritten: 1024 * 1024, // Mock 1MB
      chunksCreated: 3, // Mock 3 chunks
      lastChunkTimestamp: new Date(),
      snapshotId: `snapshot_${uuidv4()}`,
    };

    this.activeCaptures.delete(captureId);
    this.stats.active_captures--;
    this.stats.total_captures_stopped++;
    this.stats.total_bytes_captured += stats.bytesWritten;

    // Update average duration
    this.stats.average_capture_duration_ms = 
      (this.stats.average_capture_duration_ms + duration) / 2;

    this.emit('camera_capture_stopped', captureId, stats);
    return stats;
  }

  /**
   * Start microphone capture
   */
  async startMicrophoneCapture(config: CaptureConfig): Promise<string> {
    if (!this.connected) {
      throw new Error('Not connected to device service');
    }

    if (!this.currentSessionId || !this.currentCaps) {
      throw new Error('No active session. Call setSession() first.');
    }

    if (this.activeCaptures.size >= this.config.maxConcurrentCaptures) {
      throw new Error('Maximum concurrent captures reached');
    }

    const captureId = uuidv4();
    
    // Mock implementation
    const captureInfo: CaptureInfo = {
      captureId,
      sessionId: this.currentSessionId,
      deviceType: 'microphone',
      config,
      startTime: new Date(),
      deterministic: config.deterministic,
    };

    this.activeCaptures.set(captureId, captureInfo);
    this.stats.active_captures++;
    this.stats.total_captures_started++;

    this.emit('microphone_capture_started', captureId, this.currentSessionId);
    return captureId;
  }

  /**
   * Stop microphone capture
   */
  async stopMicrophoneCapture(captureId: string): Promise<CaptureStats> {
    const captureInfo = this.activeCaptures.get(captureId);
    if (!captureInfo) {
      throw new Error(`Capture not found: ${captureId}`);
    }

    const duration = Date.now() - captureInfo.startTime.getTime();
    const stats: CaptureStats = {
      captureId,
      durationMs: duration,
      bytesWritten: 512 * 1024, // Mock 512KB
      chunksCreated: 3, // Mock 3 chunks
      lastChunkTimestamp: new Date(),
      snapshotId: `snapshot_${uuidv4()}`,
    };

    this.activeCaptures.delete(captureId);
    this.stats.active_captures--;
    this.stats.total_captures_stopped++;
    this.stats.total_bytes_captured += stats.bytesWritten;

    // Update average duration
    this.stats.average_capture_duration_ms = 
      (this.stats.average_capture_duration_ms + duration) / 2;

    this.emit('microphone_capture_stopped', captureId, stats);
    return stats;
  }

  /**
   * Stop any capture (camera or microphone)
   */
  async stopCapture(captureId: string): Promise<CaptureStats> {
    const captureInfo = this.activeCaptures.get(captureId);
    if (!captureInfo) {
      throw new Error(`Capture not found: ${captureId}`);
    }

    if (captureInfo.deviceType === 'camera') {
      return await this.stopCameraCapture(captureId);
    } else {
      return await this.stopMicrophoneCapture(captureId);
    }
  }

  /**
   * Get capture status
   */
  async getCaptureStatus(captureId: string): Promise<CaptureStatus> {
    const captureInfo = this.activeCaptures.get(captureId);
    if (!captureInfo) {
      throw new Error(`Capture not found: ${captureId}`);
    }

    return {
      captureId,
      running: true,
      bytesWritten: 256 * 1024, // Mock 256KB
      lastTimestamp: new Date(),
    };
  }

  /**
   * Get preview frame (camera only)
   */
  async getPreviewFrame(captureId: string): Promise<PreviewFrame> {
    const captureInfo = this.activeCaptures.get(captureId);
    if (!captureInfo) {
      throw new Error(`Capture not found: ${captureId}`);
    }

    if (captureInfo.deviceType !== 'camera') {
      throw new Error('Preview only available for camera captures');
    }

    // Mock preview frame
    const width = 160;
    const height = 90;
    const frameData = new ArrayBuffer(width * height * 3); // RGB
    const view = new Uint8Array(frameData);
    
    // Fill with mock data
    for (let i = 0; i < view.length; i++) {
      view[i] = i % 256;
    }

    this.stats.preview_frames_sent++;

    return {
      frameData,
      width,
      height,
      timestamp: new Date(),
    };
  }

  /**
   * Set DAO policy for a room
   */
  async setDaoPolicy(roomId: string, policy: Partial<DaoPolicy>): Promise<void> {
    if (!this.connected) {
      throw new Error('Not connected to device service');
    }

    // Mock implementation
    this.stats.policy_checks++;
    
    this.emit('policy_changed', roomId, {
      roomId,
      allowCamera: policy.allowCamera ?? true,
      allowMicrophone: policy.allowMicrophone ?? true,
      allowPreview: policy.allowPreview ?? true,
    });
  }

  /**
   * Get DAO policy for a room
   */
  async getDaoPolicy(roomId: string): Promise<DaoPolicy | null> {
    if (!this.connected) {
      throw new Error('Not connected to device service');
    }

    // Mock implementation
    this.stats.policy_checks++;
    
    return {
      roomId,
      allowCamera: true,
      allowMicrophone: true,
      allowPreview: true,
    };
  }

  /**
   * Get device statistics
   */
  getStats(): DeviceStats {
    return { ...this.stats };
  }

  /**
   * Get active captures
   */
  getActiveCaptures(): CaptureInfo[] {
    return Array.from(this.activeCaptures.values());
  }

  /**
   * Start preview timer for active camera captures
   */
  private startPreviewTimer(): void {
    const interval = 1000 / this.config.previewThrottleFps;
    
    this.previewTimer = setInterval(() => {
      for (const [captureId, captureInfo] of this.activeCaptures) {
        if (captureInfo.deviceType === 'camera') {
          this.getPreviewFrame(captureId).then(frame => {
            this.emit('camera_preview_frame', captureId, frame);
          }).catch(error => {
            this.emit('capture_error', captureId, error.message);
          });
        }
      }
    }, interval);
  }

  /**
   * Stop preview timer
   */
  private _stopPreviewTimer(): void {
    if (this.previewTimer) {
      clearInterval(this.previewTimer);
      this.previewTimer = undefined as any;
    }
  }

  /**
   * Schedule reconnection
   */
  private _scheduleReconnect(): void {
    if (this.reconnectAttempts >= 5) {
      this.emit('connection_error', 'Max reconnect attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

    this._reconnectTimer = setTimeout(() => {
      this.connect().catch(error => {
        this.emit('connection_error', error);
      });
    }, delay);
  }
}

// Internal types
interface CaptureInfo {
  captureId: string;
  sessionId: string;
  deviceType: 'camera' | 'microphone';
  config: CaptureConfig;
  startTime: Date;
  deterministic: boolean;
}

// Export default instance
export const deviceBridge = new DeviceBridge();

// Types are already exported above
