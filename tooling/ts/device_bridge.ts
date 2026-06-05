/**
 * @file device_bridge.ts
 * @brief TypeScript bridge for Aetheris Device Listing and Provider Management
 * 
 * This module provides TypeScript interfaces for device discovery, listing,
 * and provider management operations.
 */

import { EventEmitter } from 'events';

/**
 * Device types supported by the HAL
 */
export enum DeviceType {
    Camera = 'camera',
    Microphone = 'microphone',
    Gpio = 'gpio',
    Adc = 'adc',
    Actuator = 'actuator'
}

/**
 * Provider types supported by the HAL
 */
export enum ProviderType {
    Deterministic = 'deterministic',
    Linux = 'linux',
    Custom = 'custom'
}

/**
 * Device information interface
 */
export interface DeviceInfo {
    deviceId: string;
    deviceType: DeviceType;
    devicePath: string;
    capabilities: string[];
    provider: ProviderType;
    isAvailable: boolean;
    metadata: Record<string, any>;
}

/**
 * Provider status interface
 */
export interface ProviderStatus {
    providerId: ProviderType;
    isAvailable: boolean;
    deviceCount: number;
    lastError?: string;
    capabilities: string[];
}

/**
 * Device listing error types
 */
export enum DeviceListError {
    Success = 0,
    InvalidParam = -1,
    DeviceNotFound = -2,
    ProviderNotFound = -3,
    DiscoveryFailed = -4,
    SerializationError = -5,
    DeserializationError = -6,
    Unknown = -999
}

/**
 * Device discovery options
 */
export interface DeviceDiscoveryOptions {
    deviceType?: DeviceType;
    providerType?: ProviderType;
    showUnavailable?: boolean;
    refresh?: boolean;
}

/**
 * Provider management options
 */
export interface ProviderOptions {
    providerType: ProviderType;
    autoFallback?: boolean;
}

/**
 * Device bridge event types
 */
export interface DeviceBridgeEvents {
    'device-discovered': (device: DeviceInfo) => void;
    'device-lost': (deviceId: string) => void;
    'provider-status-changed': (status: ProviderStatus) => void;
    'error': (error: Error) => void;
}

/**
 * Device bridge class for managing device discovery and provider selection
 */
export class DeviceBridge extends EventEmitter {
    private devices: Map<string, DeviceInfo> = new Map();
    private providers: Map<ProviderType, ProviderStatus> = new Map();
    private currentProvider: ProviderType = ProviderType.Deterministic;
    private discoveryInterval?: NodeJS.Timeout;
    private isDiscovering = false;

    constructor() {
        super();
        this.initializeProviders();
    }

    /**
     * Initialize provider status
     */
    private async initializeProviders(): Promise<void> {
        try {
            const statuses = await this.getProviderStatus();
            for (const status of statuses) {
                this.providers.set(status.providerId, status);
            }
        } catch (error) {
            this.emit('error', error as Error);
        }
    }

    /**
     * Discover all available devices
     */
    async discoverDevices(options: DeviceDiscoveryOptions = {}): Promise<DeviceInfo[]> {
        if (this.isDiscovering) {
            throw new Error('Device discovery already in progress');
        }

        this.isDiscovering = true;

        try {
            // In a real implementation, this would call the Rust FFI
            const devices = await this.performDeviceDiscovery(options);

            // Update device cache
            const previousDevices = new Set(this.devices.keys());
            const currentDevices = new Set(devices.map(d => d.deviceId));

            // Emit events for new devices
            for (const device of devices) {
                if (!previousDevices.has(device.deviceId)) {
                    this.emit('device-discovered', device);
                }
                this.devices.set(device.deviceId, device);
            }

            // Emit events for lost devices
            for (const deviceId of previousDevices) {
                if (!currentDevices.has(deviceId)) {
                    this.emit('device-lost', deviceId);
                    this.devices.delete(deviceId);
                }
            }

            return devices;
        } finally {
            this.isDiscovering = false;
        }
    }

    /**
     * Get all discovered devices
     */
    getDevices(): DeviceInfo[] {
        return Array.from(this.devices.values());
    }

    /**
     * Get devices by type
     */
    getDevicesByType(deviceType: DeviceType): DeviceInfo[] {
        return Array.from(this.devices.values())
            .filter(device => device.deviceType === deviceType);
    }

    /**
     * Get devices by provider
     */
    getDevicesByProvider(providerType: ProviderType): DeviceInfo[] {
        return Array.from(this.devices.values())
            .filter(device => device.provider === providerType);
    }

    /**
     * Get a specific device by ID
     */
    getDeviceById(deviceId: string): DeviceInfo | undefined {
        return this.devices.get(deviceId);
    }

    /**
     * Get provider status for all providers
     */
    async getProviderStatus(): Promise<ProviderStatus[]> {
        // In a real implementation, this would call the Rust FFI
        const statuses: ProviderStatus[] = [
            {
                providerId: ProviderType.Deterministic,
                isAvailable: true,
                deviceCount: 5,
                capabilities: [
                    'camera.capture',
                    'microphone.capture',
                    'gpio.read',
                    'gpio.write',
                    'adc.sample',
                    'actuator.control',
                    'deterministic'
                ]
            }
        ];

        // Add Linux provider if available
        if (await this.isProviderAvailable(ProviderType.Linux)) {
            statuses.push({
                providerId: ProviderType.Linux,
                isAvailable: true,
                deviceCount: 3,
                capabilities: [
                    'camera.capture',
                    'microphone.capture',
                    'gpio.read',
                    'gpio.write',
                    'gpio.interrupt',
                    'adc.sample',
                    'adc.continuous',
                    'actuator.pwm',
                    'actuator.led',
                    'actuator.motor',
                    'hardware'
                ]
            });
        }

        return statuses;
    }

    /**
     * Check if a provider is available
     */
    async isProviderAvailable(providerType: ProviderType): Promise<boolean> {
        // In a real implementation, this would call the Rust FFI
        // For now, simulate availability
        switch (providerType) {
            case ProviderType.Deterministic:
                return true;
            case ProviderType.Linux:
                // Check if we're on a Linux system with hardware interfaces
                return typeof process !== 'undefined' && process.platform === 'linux';
            case ProviderType.Custom:
                return false;
            default:
                return false;
        }
    }

    /**
     * Set the default provider
     */
    async setDefaultProvider(providerType: ProviderType): Promise<void> {
        if (!(await this.isProviderAvailable(providerType))) {
            throw new Error(`Provider ${providerType} is not available`);
        }

        // In a real implementation, this would call the Rust FFI
        this.currentProvider = providerType;

        // Emit provider status change event
        const status = this.providers.get(providerType);
        if (status) {
            this.emit('provider-status-changed', status);
        }
    }

    /**
     * Get the current default provider
     */
    getDefaultProvider(): ProviderType {
        return this.currentProvider;
    }

    /**
     * Start automatic device discovery
     */
    startAutoDiscovery(intervalMs: number = 30000): void {
        if (this.discoveryInterval) {
            clearInterval(this.discoveryInterval);
        }

        this.discoveryInterval = setInterval(async () => {
            try {
                await this.discoverDevices({ refresh: true });
            } catch (error) {
                this.emit('error', error as Error);
            }
        }, intervalMs);
    }

    /**
     * Stop automatic device discovery
     */
    stopAutoDiscovery(): void {
        if (this.discoveryInterval) {
            clearInterval(this.discoveryInterval);
            this.discoveryInterval = undefined;
        }
    }

    /**
     * Perform actual device discovery (mock implementation)
     */
    private async performDeviceDiscovery(options: DeviceDiscoveryOptions): Promise<DeviceInfo[]> {
        // Simulate async operation
        await new Promise(resolve => setTimeout(resolve, 100));

        const devices: DeviceInfo[] = [
            {
                deviceId: 'camera_mock_0',
                deviceType: DeviceType.Camera,
                devicePath: '/dev/video_mock_0',
                capabilities: ['capture', 'yuv420', 'rgb24'],
                provider: ProviderType.Deterministic,
                isAvailable: true,
                metadata: {
                    width: 640,
                    height: 480,
                    fps: 30
                }
            },
            {
                deviceId: 'mic_mock_0',
                deviceType: DeviceType.Microphone,
                devicePath: '/dev/audio_mock_0',
                capabilities: ['capture', 's16le', 'f32le'],
                provider: ProviderType.Deterministic,
                isAvailable: true,
                metadata: {
                    sample_rate: 44100,
                    channels: 2
                }
            },
            {
                deviceId: 'gpio_mock_0',
                deviceType: DeviceType.Gpio,
                devicePath: '/dev/gpiochip_mock_0',
                capabilities: ['read', 'write', 'interrupt'],
                provider: ProviderType.Deterministic,
                isAvailable: true,
                metadata: {
                    lines: 32
                }
            },
            {
                deviceId: 'adc_mock_0',
                deviceType: DeviceType.Adc,
                devicePath: '/dev/iio_mock_0',
                capabilities: ['sample', 'continuous'],
                provider: ProviderType.Deterministic,
                isAvailable: true,
                metadata: {
                    channels: 8,
                    resolution: 12,
                    reference_voltage: 3.3
                }
            },
            {
                deviceId: 'actuator_mock_0',
                deviceType: DeviceType.Actuator,
                devicePath: '/dev/pwm_mock_0',
                capabilities: ['pwm', 'led', 'motor'],
                provider: ProviderType.Deterministic,
                isAvailable: true,
                metadata: {
                    channels: 4,
                    frequency_range: '1-1000000'
                }
            }
        ];

        // Add Linux devices if available and requested
        if (await this.isProviderAvailable(ProviderType.Linux)) {
            const linuxDevices: DeviceInfo[] = [
                {
                    deviceId: '/dev/video0',
                    deviceType: DeviceType.Camera,
                    devicePath: '/dev/video0',
                    capabilities: ['capture', 'yuv420', 'rgb24', 'mjpeg'],
                    provider: ProviderType.Linux,
                    isAvailable: true,
                    metadata: {
                        driver: 'uvcvideo',
                        card: 'USB2.0 Camera'
                    }
                },
                {
                    deviceId: 'hw:0,0',
                    deviceType: DeviceType.Microphone,
                    devicePath: 'hw:0,0',
                    capabilities: ['capture', 's16le', 'f32le'],
                    provider: ProviderType.Linux,
                    isAvailable: true,
                    metadata: {
                        card_name: 'HDA Intel PCH',
                        device_type: 'alsa_card'
                    }
                },
                {
                    deviceId: '/dev/gpiochip0',
                    deviceType: DeviceType.Gpio,
                    devicePath: '/dev/gpiochip0',
                    capabilities: ['read', 'write', 'interrupt', 'pull_up', 'pull_down'],
                    provider: ProviderType.Linux,
                    isAvailable: true,
                    metadata: {
                        chip_name: 'gpiochip0',
                        line_count: 32
                    }
                }
            ];
            devices.push(...linuxDevices);
        }

        // Apply filters
        let filteredDevices = devices;

        if (options.deviceType) {
            filteredDevices = filteredDevices.filter(d => d.deviceType === options.deviceType);
        }

        if (options.providerType) {
            filteredDevices = filteredDevices.filter(d => d.provider === options.providerType);
        }

        if (!options.showUnavailable) {
            filteredDevices = filteredDevices.filter(d => d.isAvailable);
        }

        return filteredDevices;
    }

    /**
     * Cleanup resources
     */
    destroy(): void {
        this.stopAutoDiscovery();
        this.removeAllListeners();
        this.devices.clear();
        this.providers.clear();
    }
}

/**
 * Create a new device bridge instance
 */
export function createDeviceBridge(): DeviceBridge {
    return new DeviceBridge();
}

/**
 * Utility functions for device type validation
 */
export const DeviceTypeUtils = {
    /**
     * Check if a device type string is valid
     */
    isValid(deviceType: string): deviceType is DeviceType {
        return Object.values(DeviceType).includes(deviceType as DeviceType);
    },

    /**
     * Parse device type from string
     */
    parse(deviceType: string): DeviceType {
        if (!this.isValid(deviceType)) {
            throw new Error(`Invalid device type: ${deviceType}`);
        }
        return deviceType as DeviceType;
    },

    /**
     * Get all available device types
     */
    getAll(): DeviceType[] {
        return Object.values(DeviceType);
    }
};

/**
 * Utility functions for provider type validation
 */
export const ProviderTypeUtils = {
    /**
     * Check if a provider type string is valid
     */
    isValid(providerType: string): providerType is ProviderType {
        return Object.values(ProviderType).includes(providerType as ProviderType);
    },

    /**
     * Parse provider type from string
     */
    parse(providerType: string): ProviderType {
        if (!this.isValid(providerType)) {
            throw new Error(`Invalid provider type: ${providerType}`);
        }
        return providerType as ProviderType;
    },

    /**
     * Get all available provider types
     */
    getAll(): ProviderType[] {
        return Object.values(ProviderType);
    }
};

/**
 * Error handling utilities
 */
export const DeviceListErrorUtils = {
    /**
     * Get error message for error code
     */
    getMessage(error: DeviceListError): string {
        switch (error) {
            case DeviceListError.Success:
                return 'Success';
            case DeviceListError.InvalidParam:
                return 'Invalid parameter';
            case DeviceListError.DeviceNotFound:
                return 'Device not found';
            case DeviceListError.ProviderNotFound:
                return 'Provider not found';
            case DeviceListError.DiscoveryFailed:
                return 'Device discovery failed';
            case DeviceListError.SerializationError:
                return 'Serialization error';
            case DeviceListError.DeserializationError:
                return 'Deserialization error';
            case DeviceListError.Unknown:
            default:
                return 'Unknown error';
        }
    },

    /**
     * Check if error code indicates success
     */
    isSuccess(error: DeviceListError): boolean {
        return error === DeviceListError.Success;
    },

    /**
     * Check if error code indicates failure
     */
    isError(error: DeviceListError): boolean {
        return error !== DeviceListError.Success;
    }
};

// Export default instance
export const deviceBridge = createDeviceBridge();
