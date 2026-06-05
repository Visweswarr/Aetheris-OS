/**
 * TypeScript SDK for Aetheris OS BLE Operations
 * 
 * This module provides client-side interaction with the Aetheris OS BLE system,
 * including device scanning, connection management, GATT operations, and service discovery.
 */

import { EventEmitter } from 'events';

// Type definitions
export interface BLEDevice {
  address: string;
  name?: string;
  rssi: number;
  manufacturerData?: Uint8Array;
  serviceData?: Map<string, Uint8Array>;
  serviceUuids?: string[];
  isConnectable: boolean;
  lastSeen: number;
}

export interface BLEService {
  uuid: string;
  primary: boolean;
  characteristics: BLECharacteristic[];
}

export interface BLECharacteristic {
  uuid: string;
  properties: BLECharacteristicProperties;
  value?: Uint8Array;
  descriptors?: BLEDescriptor[];
}

export interface BLECharacteristicProperties {
  read: boolean;
  write: boolean;
  writeWithoutResponse: boolean;
  notify: boolean;
  indicate: boolean;
  authenticatedSignedWrites: boolean;
  extendedProperties: boolean;
  reliableWrite: boolean;
  writableAuxiliaries: boolean;
}

export interface BLEDescriptor {
  uuid: string;
  value?: Uint8Array;
}

export interface BLEConnection {
  deviceAddress: string;
  connectionHandle: string;
  isConnected: boolean;
  services: Map<string, BLEService>;
  connectedAt: number;
  lastActivity: number;
}

export interface BLEScanResult {
  devices: BLEDevice[];
  scanDuration: number;
  totalDevices: number;
  uniqueDevices: number;
}

export interface BLEConfig {
  endpoint: string;
  timeout: number;
  retries: number;
  enableLogging: boolean;
  enablePolicyEnforcement: boolean;
  enableCapabilityChecking: boolean;
  autoReconnect: boolean;
  maxReconnectAttempts: number;
  scanTimeout: number;
  connectionTimeout: number;
  gattTimeout: number;
}

export interface BLEStats {
  connectionStatus: 'connected' | 'disconnected' | 'connecting' | 'error';
  totalDevices: number;
  connectedDevices: number;
  totalScans: number;
  successfulScans: number;
  failedScans: number;
  totalConnections: number;
  successfulConnections: number;
  failedConnections: number;
  totalGattOperations: number;
  successfulGattOperations: number;
  failedGattOperations: number;
  averageResponseTime: number;
  lastScanTime: number;
  lastConnectionTime: number;
}

/**
 * Main BLE Bridge class
 */
export class BLEBridge extends EventEmitter {
  private config: BLEConfig;
  private isConnected: boolean = false;
  private connectionRetries: number = 0;
  private activeConnections: Map<string, BLEConnection> = new Map();
  private activeScans: Set<string> = new Set();

  constructor(config: Partial<BLEConfig> = {}) {
    super();
    
    this.config = {
      endpoint: config.endpoint || 'http://localhost:8080',
      timeout: config.timeout || 30000,
      retries: config.retries || 3,
      enableLogging: config.enableLogging || false,
      enablePolicyEnforcement: config.enablePolicyEnforcement || true,
      enableCapabilityChecking: config.enableCapabilityChecking || true,
      autoReconnect: config.autoReconnect || true,
      maxReconnectAttempts: config.maxReconnectAttempts || 5,
      scanTimeout: config.scanTimeout || 10000,
      connectionTimeout: config.connectionTimeout || 5000,
      gattTimeout: config.gattTimeout || 3000,
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
   * Connect to the BLE service
   */
  private async connect(): Promise<void> {
    const response = await fetch(`${this.config.endpoint}/ble/health`, {
      method: 'GET',
      timeout: this.config.timeout,
    });

    if (!response.ok) {
      throw new Error(`Failed to connect to BLE service: ${response.statusText}`);
    }
  }

  /**
   * Start BLE device scanning
   */
  async startScan(options: {
    duration?: number;
    allowDuplicates?: boolean;
    serviceUuids?: string[];
    manufacturerData?: Map<string, Uint8Array>;
    namePrefix?: string;
    rssiThreshold?: number;
  } = {}): Promise<string> {
    await this.ensureConnected();

    const scanId = this.generateScanId();
    
    const response = await this.makeRequest('/ble/scan/start', {
      method: 'POST',
      body: JSON.stringify({
        scan_id: scanId,
        duration: options.duration || this.config.scanTimeout,
        allow_duplicates: options.allowDuplicates || false,
        service_uuids: options.serviceUuids || [],
        manufacturer_data: options.manufacturerData ? 
          Array.from(options.manufacturerData.entries()).map(([key, value]) => ({
            key,
            value: Array.from(value)
          })) : [],
        name_prefix: options.namePrefix || '',
        rssi_threshold: options.rssiThreshold || -100,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to start BLE scan: ${response.error}`);
    }

    this.activeScans.add(scanId);
    
    this.emit('scanStarted', {
      scanId,
      options,
    });

    return scanId;
  }

  /**
   * Stop BLE device scanning
   */
  async stopScan(scanId: string): Promise<BLEScanResult> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/ble/scan/${scanId}/stop`, {
      method: 'POST',
    });

    if (!response.success) {
      throw new Error(`Failed to stop BLE scan: ${response.error}`);
    }

    this.activeScans.delete(scanId);

    const result: BLEScanResult = {
      devices: response.devices || [],
      scanDuration: response.scan_duration || 0,
      totalDevices: response.total_devices || 0,
      uniqueDevices: response.unique_devices || 0,
    };

    this.emit('scanStopped', {
      scanId,
      result,
    });

    return result;
  }

  /**
   * Get scan results
   */
  async getScanResults(scanId: string): Promise<BLEDevice[]> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/ble/scan/${scanId}/results`, {
      method: 'GET',
    });

    if (!response.success) {
      throw new Error(`Failed to get scan results: ${response.error}`);
    }

    return response.devices || [];
  }

  /**
   * Connect to a BLE device
   */
  async connectDevice(deviceAddress: string, options: {
    timeout?: number;
    autoReconnect?: boolean;
    serviceDiscovery?: boolean;
  } = {}): Promise<string> {
    await this.ensureConnected();

    const connectionHandle = this.generateConnectionHandle();
    
    const response = await this.makeRequest('/ble/connect', {
      method: 'POST',
      body: JSON.stringify({
        device_address: deviceAddress,
        connection_handle: connectionHandle,
        timeout: options.timeout || this.config.connectionTimeout,
        auto_reconnect: options.autoReconnect || this.config.autoReconnect,
        service_discovery: options.serviceDiscovery !== false,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to connect to BLE device: ${response.error}`);
    }

    const connection: BLEConnection = {
      deviceAddress,
      connectionHandle,
      isConnected: true,
      services: new Map(),
      connectedAt: Date.now(),
      lastActivity: Date.now(),
    };

    this.activeConnections.set(connectionHandle, connection);

    this.emit('deviceConnected', {
      deviceAddress,
      connectionHandle,
    });

    return connectionHandle;
  }

  /**
   * Disconnect from a BLE device
   */
  async disconnectDevice(connectionHandle: string): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/ble/connect/${connectionHandle}/disconnect`, {
      method: 'POST',
    });

    if (!response.success) {
      throw new Error(`Failed to disconnect from BLE device: ${response.error}`);
    }

    const connection = this.activeConnections.get(connectionHandle);
    if (connection) {
      connection.isConnected = false;
      this.activeConnections.delete(connectionHandle);
    }

    this.emit('deviceDisconnected', {
      connectionHandle,
    });
  }

  /**
   * Discover GATT services
   */
  async discoverServices(connectionHandle: string, serviceUuids?: string[]): Promise<BLEService[]> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/ble/connect/${connectionHandle}/services/discover`, {
      method: 'POST',
      body: JSON.stringify({
        service_uuids: serviceUuids || [],
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to discover GATT services: ${response.error}`);
    }

    const services: BLEService[] = response.services || [];
    
    // Update connection with discovered services
    const connection = this.activeConnections.get(connectionHandle);
    if (connection) {
      services.forEach(service => {
        connection.services.set(service.uuid, service);
      });
      connection.lastActivity = Date.now();
    }

    this.emit('servicesDiscovered', {
      connectionHandle,
      services,
    });

    return services;
  }

  /**
   * Read GATT characteristic
   */
  async readCharacteristic(connectionHandle: string, serviceUuid: string, characteristicUuid: string): Promise<Uint8Array> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/ble/connect/${connectionHandle}/characteristic/read`, {
      method: 'POST',
      body: JSON.stringify({
        service_uuid: serviceUuid,
        characteristic_uuid: characteristicUuid,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to read GATT characteristic: ${response.error}`);
    }

    const value = new Uint8Array(response.value || []);
    
    // Update connection activity
    const connection = this.activeConnections.get(connectionHandle);
    if (connection) {
      connection.lastActivity = Date.now();
    }

    this.emit('characteristicRead', {
      connectionHandle,
      serviceUuid,
      characteristicUuid,
      value,
    });

    return value;
  }

  /**
   * Write GATT characteristic
   */
  async writeCharacteristic(connectionHandle: string, serviceUuid: string, characteristicUuid: string, value: Uint8Array, options: {
    withoutResponse?: boolean;
    signed?: boolean;
  } = {}): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/ble/connect/${connectionHandle}/characteristic/write`, {
      method: 'POST',
      body: JSON.stringify({
        service_uuid: serviceUuid,
        characteristic_uuid: characteristicUuid,
        value: Array.from(value),
        without_response: options.withoutResponse || false,
        signed: options.signed || false,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to write GATT characteristic: ${response.error}`);
    }

    // Update connection activity
    const connection = this.activeConnections.get(connectionHandle);
    if (connection) {
      connection.lastActivity = Date.now();
    }

    this.emit('characteristicWritten', {
      connectionHandle,
      serviceUuid,
      characteristicUuid,
      value,
    });
  }

  /**
   * Subscribe to GATT characteristic notifications
   */
  async subscribeCharacteristic(connectionHandle: string, serviceUuid: string, characteristicUuid: string): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/ble/connect/${connectionHandle}/characteristic/subscribe`, {
      method: 'POST',
      body: JSON.stringify({
        service_uuid: serviceUuid,
        characteristic_uuid: characteristicUuid,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to subscribe to GATT characteristic: ${response.error}`);
    }

    this.emit('characteristicSubscribed', {
      connectionHandle,
      serviceUuid,
      characteristicUuid,
    });
  }

  /**
   * Unsubscribe from GATT characteristic notifications
   */
  async unsubscribeCharacteristic(connectionHandle: string, serviceUuid: string, characteristicUuid: string): Promise<void> {
    await this.ensureConnected();

    const response = await this.makeRequest(`/ble/connect/${connectionHandle}/characteristic/unsubscribe`, {
      method: 'POST',
      body: JSON.stringify({
        service_uuid: serviceUuid,
        characteristic_uuid: characteristicUuid,
      }),
    });

    if (!response.success) {
      throw new Error(`Failed to unsubscribe from GATT characteristic: ${response.error}`);
    }

    this.emit('characteristicUnsubscribed', {
      connectionHandle,
      serviceUuid,
      characteristicUuid,
    });
  }

  /**
   * Get BLE statistics
   */
  async getStats(): Promise<BLEStats> {
    await this.ensureConnected();

    const response = await this.makeRequest('/ble/stats', {
      method: 'GET',
    });

    return response;
  }

  /**
   * Get active connections
   */
  getActiveConnections(): Map<string, BLEConnection> {
    return new Map(this.activeConnections);
  }

  /**
   * Get active scans
   */
  getActiveScans(): Set<string> {
    return new Set(this.activeScans);
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<BLEConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfig(): BLEConfig {
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
    // Disconnect all active connections
    for (const [handle, connection] of this.activeConnections) {
      if (connection.isConnected) {
        try {
          await this.disconnectDevice(handle);
        } catch (error) {
          console.warn(`Failed to disconnect device ${handle}:`, error);
        }
      }
    }

    // Stop all active scans
    for (const scanId of this.activeScans) {
      try {
        await this.stopScan(scanId);
      } catch (error) {
        console.warn(`Failed to stop scan ${scanId}:`, error);
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
   * Generate unique scan ID
   */
  private generateScanId(): string {
    return `scan_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate unique connection handle
   */
  private generateConnectionHandle(): string {
    return `conn_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

/**
 * Utility functions for working with BLE
 */
export class BLEUtils {
  /**
   * Convert UUID string to standard format
   */
  static normalizeUuid(uuid: string): string {
    // Remove dashes and convert to lowercase
    const cleanUuid = uuid.replace(/-/g, '').toLowerCase();
    
    // Add dashes in standard positions
    if (cleanUuid.length === 32) {
      return `${cleanUuid.substr(0, 8)}-${cleanUuid.substr(8, 4)}-${cleanUuid.substr(12, 4)}-${cleanUuid.substr(16, 4)}-${cleanUuid.substr(20, 12)}`;
    }
    
    return uuid;
  }

  /**
   * Check if UUID is valid
   */
  static isValidUuid(uuid: string): boolean {
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
    return uuidRegex.test(uuid);
  }

  /**
   * Convert MAC address to standard format
   */
  static normalizeMacAddress(address: string): string {
    // Remove colons and convert to lowercase
    const cleanAddress = address.replace(/:/g, '').toLowerCase();
    
    // Add colons in standard positions
    if (cleanAddress.length === 12) {
      return `${cleanAddress.substr(0, 2)}:${cleanAddress.substr(2, 2)}:${cleanAddress.substr(4, 2)}:${cleanAddress.substr(6, 2)}:${cleanAddress.substr(8, 2)}:${cleanAddress.substr(10, 2)}`;
    }
    
    return address;
  }

  /**
   * Check if MAC address is valid
   */
  static isValidMacAddress(address: string): boolean {
    const macRegex = /^([0-9a-f]{2}:){5}[0-9a-f]{2}$/i;
    return macRegex.test(address);
  }

  /**
   * Convert Uint8Array to hex string
   */
  static uint8ArrayToHex(data: Uint8Array): string {
    return Array.from(data)
      .map(byte => byte.toString(16).padStart(2, '0'))
      .join('');
  }

  /**
   * Convert hex string to Uint8Array
   */
  static hexToUint8Array(hex: string): Uint8Array {
    const cleanHex = hex.replace(/[^0-9a-f]/gi, '');
    const bytes = new Uint8Array(cleanHex.length / 2);
    
    for (let i = 0; i < cleanHex.length; i += 2) {
      bytes[i / 2] = parseInt(cleanHex.substr(i, 2), 16);
    }
    
    return bytes;
  }

  /**
   * Convert Uint8Array to string
   */
  static uint8ArrayToString(data: Uint8Array): string {
    return new TextDecoder().decode(data);
  }

  /**
   * Convert string to Uint8Array
   */
  static stringToUint8Array(str: string): Uint8Array {
    return new TextEncoder().encode(str);
  }

  /**
   * Calculate RSSI distance estimate
   */
  static estimateDistance(rssi: number, txPower: number = -59): number {
    if (rssi === 0) {
      return -1.0;
    }
    
    const ratio = txPower / rssi;
    if (ratio < 1.0) {
      return Math.pow(ratio, 10);
    } else {
      const accuracy = (0.89976) * Math.pow(ratio, 7.7095) + 0.111;
      return accuracy;
    }
  }

  /**
   * Check if device is in range based on RSSI
   */
  static isInRange(rssi: number, threshold: number = -80): boolean {
    return rssi >= threshold;
  }
}

/**
 * Event types for the BLEBridge
 */
export interface BLEBridgeEvents {
  connected: () => void;
  disconnected: () => void;
  error: (error: Error) => void;
  scanStarted: (data: { scanId: string; options: any }) => void;
  scanStopped: (data: { scanId: string; result: BLEScanResult }) => void;
  deviceConnected: (data: { deviceAddress: string; connectionHandle: string }) => void;
  deviceDisconnected: (data: { connectionHandle: string }) => void;
  servicesDiscovered: (data: { connectionHandle: string; services: BLEService[] }) => void;
  characteristicRead: (data: { connectionHandle: string; serviceUuid: string; characteristicUuid: string; value: Uint8Array }) => void;
  characteristicWritten: (data: { connectionHandle: string; serviceUuid: string; characteristicUuid: string; value: Uint8Array }) => void;
  characteristicSubscribed: (data: { connectionHandle: string; serviceUuid: string; characteristicUuid: string }) => void;
  characteristicUnsubscribed: (data: { connectionHandle: string; serviceUuid: string; characteristicUuid: string }) => void;
}

/**
 * Default export
 */
export default BLEBridge;
