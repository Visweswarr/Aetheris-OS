# P4-07-A4: Hardware Abstraction Layer (HAL) with Linux Backends

## Overview

The Hardware Abstraction Layer (HAL) provides a pluggable interface for device operations in Aetheris OS, enabling seamless switching between deterministic mock providers and real hardware providers while maintaining identical APIs and NGFS integration.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Device Service                           │
├─────────────────────────────────────────────────────────────┤
│                    HAL Manager                              │
├─────────────────────────────────────────────────────────────┤
│  Provider Registry  │  Config Manager  │  Device Cache     │
├─────────────────────────────────────────────────────────────┤
│              Provider Implementations                       │
├─────────────────────────────────────────────────────────────┤
│  Deterministic     │  Linux Hardware   │  Custom Providers │
│  Provider          │  Provider         │  (Future)         │
├─────────────────────────────────────────────────────────────┤
│  V4L2 Camera  │  ALSA Mic  │  libgpiod  │  IIO ADC  │ PWM │
└─────────────────────────────────────────────────────────────┘
```

### Provider Types

1. **DefaultDeterministicProvider**: Mock/simulation provider for testing and development
2. **LinuxProvider**: Real hardware provider using Linux system interfaces
3. **CustomProvider**: Extensible provider interface for future hardware support

## Device Support

### Camera Devices (V4L2)
- **Interface**: Video4Linux2 (V4L2)
- **Formats**: YUV420, RGB24, MJPEG, H264
- **Features**: Frame capture, format conversion, buffer management
- **Device Paths**: `/dev/video*`

### Microphone Devices (ALSA)
- **Interface**: Advanced Linux Sound Architecture (ALSA)
- **Formats**: S16LE, F32LE, S24LE
- **Features**: PCM capture, multi-channel support, ring buffers
- **Device Paths**: `hw:*,0`, `/dev/snd/*`

### GPIO Devices (libgpiod)
- **Interface**: libgpiod v2
- **Features**: Line configuration, input/output, interrupts, pull-up/down
- **Device Paths**: `/dev/gpiochip*`

### ADC Devices (IIO)
- **Interface**: Industrial I/O (IIO) subsystem
- **Features**: Analog sampling, calibration, continuous mode
- **Device Paths**: `/sys/bus/iio/devices/iio:device*`

### Actuator Devices (PWM/LED)
- **Interfaces**: PWM sysfs, LED class
- **Features**: PWM control, LED brightness, pattern execution
- **Device Paths**: `/sys/class/pwm/*`, `/sys/class/leds/*`

## Configuration

### Environment Variables

```bash
# Provider selection
export AETHERIS_DEV_PROVIDER=linux          # or "det" for deterministic
export AETHERIS_DEV_FALLBACK_PROVIDER=det   # fallback provider

# Discovery settings
export AETHERIS_DEV_AUTO_DISCOVERY=true
export AETHERIS_DEV_DISCOVERY_INTERVAL=30   # seconds

# Audit settings
export AETHERIS_DEV_AUDIT_ENABLED=true

# Performance settings
export AETHERIS_DEV_MAX_CONCURRENT_OPS=10
```

### Configuration File

Create `aetheris-dev.toml`:

```toml
default_provider = "linux"
fallback_provider = "det"
auto_discovery = true
discovery_interval = 30

[device_filters]
allowed_types = ["camera", "microphone", "gpio", "adc", "actuator"]
blocked_types = []
allowed_paths = []
blocked_paths = []

[audit]
enabled = true
log_level = "info"
include_metadata = true
retention_days = 30

[performance]
max_concurrent_ops = 10
operation_timeout_ms = 5000
enable_monitoring = false

[performance.buffer_sizes]
camera = 4194304      # 4MB
microphone = 65536    # 64KB
gpio = 1024          # 1KB
adc = 8192           # 8KB
actuator = 1024      # 1KB
```

## Usage Examples

### Rust API

```rust
use aetheris_devices::{DeviceService, hal::{ProviderId, DeviceType}};

// Create device service
let device_service = DeviceService::new()?;

// Discover devices
let devices = device_service.discover_devices().await?;
println!("Discovered {} devices", devices.len());

// Get devices by type
let cameras = device_service.get_devices_by_type(&DeviceType::Camera);
println!("Found {} cameras", cameras.len());

// Get provider status
let statuses = device_service.get_provider_status().await?;
for status in statuses {
    println!("Provider {}: {} devices", status.provider_id, status.device_count);
}
```

### C FFI

```c
#include "devices_list.h"

// Discover devices
aetheris_device_info_t* devices;
size_t count;
aetheris_device_list_error_t result = aetheris_discover_devices(&devices, &count);

if (result == AETHERIS_DEVICE_LIST_SUCCESS) {
    printf("Discovered %zu devices\n", count);
    for (size_t i = 0; i < count; i++) {
        printf("Device: %s (%s)\n", devices[i].device_id, devices[i].device_path);
    }
    aetheris_free_device_info_array(devices);
}
```

### Go CLI

```bash
# List all devices
devctl devices list

# List devices by type
devctl devices list --type camera

# List devices by provider
devctl devices list --provider linux

# Show provider status
devctl devices provider status

# Set default provider
devctl devices provider set linux

# Get current provider
devctl devices provider get
```

### TypeScript Bridge

```typescript
import { DeviceBridge, DeviceType, ProviderType } from './device_bridge';

// Create device bridge
const deviceBridge = new DeviceBridge();

// Discover devices
const devices = await deviceBridge.discoverDevices();
console.log(`Discovered ${devices.length} devices`);

// Filter by type
const cameras = deviceBridge.getDevicesByType(DeviceType.Camera);

// Set provider
await deviceBridge.setDefaultProvider(ProviderType.Linux);

// Listen for device events
deviceBridge.on('device-discovered', (device) => {
    console.log(`New device: ${device.deviceId}`);
});
```

### Python Validator

```python
from device_validator import DeviceValidator

# Create validator
validator = DeviceValidator(provider="linux")

# Run validation
results = await validator.run_validation()

# Check results
for test_name, result in results.items():
    if result.success:
        print(f"✓ {test_name}: {result.message}")
    else:
        print(f"✗ {test_name}: {result.message}")
```

## Hardware Setup

### Linux System Requirements

#### Camera (V4L2)
```bash
# Check for video devices
ls /dev/video*

# Install V4L2 utilities
sudo apt-get install v4l-utils

# Test camera
v4l2-ctl --list-devices
v4l2-ctl --device=/dev/video0 --list-formats-ext
```

#### Microphone (ALSA)
```bash
# Check for audio devices
ls /dev/snd/
cat /proc/asound/cards

# Install ALSA utilities
sudo apt-get install alsa-utils

# Test microphone
arecord -l
arecord -D hw:0,0 -f S16_LE -r 44100 -c 2 test.wav
```

#### GPIO (libgpiod)
```bash
# Install libgpiod
sudo apt-get install libgpiod-dev libgpiod2

# Check for GPIO chips
ls /dev/gpiochip*

# Test GPIO (requires root or gpio group)
sudo gpiodetect
sudo gpioinfo gpiochip0
```

#### ADC (IIO)
```bash
# Check for IIO devices
ls /sys/bus/iio/devices/

# Install IIO utilities
sudo apt-get install iio-utils

# Test ADC
iio_info
iio_attr -c iio:device0
```

#### PWM/LED
```bash
# Check for PWM devices
ls /sys/class/pwm/

# Check for LED devices
ls /sys/class/leds/

# Test PWM (requires root)
echo 0 > /sys/class/pwm/pwmchip0/export
echo 1000000 > /sys/class/pwm/pwmchip0/pwm0/period
echo 500000 > /sys/class/pwm/pwmchip0/pwm0/duty_cycle
echo 1 > /sys/class/pwm/pwmchip0/pwm0/enable
```

### Permissions

#### User Groups
```bash
# Add user to required groups
sudo usermod -a -G video,audio,gpio $USER

# For PWM/LED access (requires root or custom udev rules)
sudo usermod -a -G pwm $USER
```

#### udev Rules
Create `/etc/udev/rules.d/99-aetheris-devices.rules`:

```bash
# GPIO access
SUBSYSTEM=="gpio", GROUP="gpio", MODE="0664"

# PWM access
SUBSYSTEM=="pwm", GROUP="pwm", MODE="0664"

# LED access
SUBSYSTEM=="leds", GROUP="leds", MODE="0664"

# Video device access
SUBSYSTEM=="video4linux", GROUP="video", MODE="0664"

# Audio device access
SUBSYSTEM=="sound", GROUP="audio", MODE="0664"
```

## Troubleshooting

### Common Issues

#### 1. No Devices Discovered

**Symptoms**: `devctl devices list` returns empty list

**Solutions**:
```bash
# Check if running as root or in correct groups
groups $USER

# Check device permissions
ls -la /dev/video* /dev/gpiochip* /dev/snd/

# Check system logs
dmesg | grep -i "video\|gpio\|audio"

# Test hardware manually
v4l2-ctl --list-devices
gpiodetect
arecord -l
```

#### 2. Permission Denied Errors

**Symptoms**: `Permission denied` when accessing devices

**Solutions**:
```bash
# Add user to required groups
sudo usermod -a -G video,audio,gpio $USER

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Log out and back in to apply group changes
```

#### 3. Provider Not Available

**Symptoms**: `Provider not available` error

**Solutions**:
```bash
# Check system requirements
uname -a
ls /dev/video* /dev/gpiochip* /dev/snd/

# Check if running on Linux
if [ "$(uname)" != "Linux" ]; then
    echo "Linux provider only available on Linux systems"
fi

# Fall back to deterministic provider
export AETHERIS_DEV_PROVIDER=det
```

#### 4. Device Discovery Timeout

**Symptoms**: Device discovery takes too long or hangs

**Solutions**:
```bash
# Increase timeout
export AETHERIS_DEV_OPERATION_TIMEOUT_MS=10000

# Disable auto-discovery
export AETHERIS_DEV_AUTO_DISCOVERY=false

# Check system load
top
iostat 1
```

#### 5. Memory Issues

**Symptoms**: Out of memory errors during device operations

**Solutions**:
```bash
# Reduce buffer sizes
export AETHERIS_DEV_BUFFER_SIZE_CAMERA=2097152  # 2MB instead of 4MB
export AETHERIS_DEV_BUFFER_SIZE_MICROPHONE=32768 # 32KB instead of 64KB

# Limit concurrent operations
export AETHERIS_DEV_MAX_CONCURRENT_OPS=5
```

### Debug Mode

Enable debug logging:

```bash
# Set log level
export RUST_LOG=debug
export AETHERIS_DEV_LOG_LEVEL=debug

# Run with verbose output
devctl devices list --verbose
```

### Performance Tuning

#### Buffer Sizes
```toml
[performance.buffer_sizes]
camera = 8388608      # 8MB for high-resolution video
microphone = 131072   # 128KB for low-latency audio
gpio = 2048          # 2KB for high-frequency GPIO
adc = 16384          # 16KB for continuous sampling
actuator = 2048      # 2KB for complex patterns
```

#### Timeouts
```toml
[performance]
operation_timeout_ms = 10000  # 10 seconds
discovery_timeout_ms = 5000   # 5 seconds
```

#### Concurrency
```toml
[performance]
max_concurrent_ops = 20       # Increase for multi-device scenarios
enable_monitoring = true      # Enable performance monitoring
```

## Testing

### Unit Tests

```bash
# Run Rust unit tests
cd services/devices
cargo test

# Run specific HAL tests
cargo test hal::

# Run with hardware (requires Linux)
cargo test --features hardware
```

### Integration Tests

```bash
# Run Go CLI tests
cd go/tooling/devctl
go test ./...

# Run TypeScript tests
cd tooling/ts
npm test

# Run Python validator
cd tooling/python
python device_validator.py --provider linux --verbose
```

### Hardware Tests

```bash
# Test with real hardware
export AETHERIS_DEV_PROVIDER=linux
devctl devices list --provider linux

# Test device operations
devctl camera start --device /dev/video0
devctl mic start --device hw:0,0
devctl gpio read --chip gpiochip0 --line 17
devctl adc sample --iio /sys/bus/iio/devices/iio:device0 --channel 0
devctl actuator pwm --chip pwmchip0 --channel 0 --duty 50% --freq 1kHz
```

## Performance Benchmarks

### Latency Targets

| Device Type | Target Latency | Typical Range |
|-------------|----------------|---------------|
| GPIO        | ≤ 1µs          | 0.5-2µs       |
| ADC         | ≤ 10µs         | 5-15µs        |
| Actuator    | ≤ 100µs        | 50-200µs      |
| Camera      | ≤ 33ms         | 16-50ms       |
| Microphone  | ≤ 23ms         | 10-50ms       |

### Memory Usage

| Component | Memory Overhead | Notes |
|-----------|-----------------|-------|
| HAL Manager | ~1KB | Base overhead |
| Device Cache | ~100B per device | Cached device info |
| Provider | ~2KB per provider | Provider state |
| Buffer | Configurable | Per-device buffers |

### Throughput

| Device Type | Max Throughput | Notes |
|-------------|----------------|-------|
| GPIO        | 1M ops/sec     | Single line |
| ADC         | 100K samples/sec | 12-bit resolution |
| PWM         | 1M updates/sec | 1MHz frequency |
| Camera      | 30 FPS         | 1080p YUV420 |
| Microphone  | 48 kHz         | 16-bit stereo |

## Security Considerations

### Capability Enforcement

All device operations require appropriate CapTokens:

```rust
// Example capability checks
if !has_capability("camera.read") {
    return Err(DeviceError::PermissionDenied);
}

if !has_capability("gpio.write") {
    return Err(DeviceError::PermissionDenied);
}
```

### Audit Logging

All device operations are logged:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "event_type": "device_operation",
  "device_id": "/dev/video0",
  "operation": "start_capture",
  "session_id": "sess_123",
  "capability_scope": "camera.read",
  "policy_hash": "abc123",
  "result": "success"
}
```

### DAO Policy Enforcement

Multi-user environments enforce DAO policies:

```toml
[dao_policies]
max_concurrent_cameras = 2
max_gpio_lines_per_user = 10
allowed_device_paths = ["/dev/video0", "/dev/gpiochip0"]
blocked_device_paths = ["/dev/video1"]
```

## Future Extensions

### Additional Providers

- **Windows Provider**: DirectShow, WASAPI, Windows GPIO
- **macOS Provider**: AVFoundation, Core Audio, IOKit
- **Embedded Provider**: Zephyr RTOS, FreeRTOS
- **Virtual Provider**: QEMU, Docker containers

### Enhanced Features

- **Device Hotplug**: Dynamic device addition/removal
- **Load Balancing**: Distribute operations across multiple devices
- **Failover**: Automatic provider switching on hardware failure
- **Clustering**: Multi-node device management

### Performance Optimizations

- **Zero-copy Buffers**: Direct memory mapping for high-throughput
- **Batch Operations**: Group multiple operations for efficiency
- **Hardware Acceleration**: GPU-accelerated video processing
- **Real-time Scheduling**: SCHED_FIFO for deterministic timing

## References

- [V4L2 Documentation](https://www.kernel.org/doc/html/latest/userspace-api/media/v4l/index.html)
- [ALSA Documentation](https://www.alsa-project.org/wiki/ALSA_Library_API)
- [libgpiod Documentation](https://libgpiod.readthedocs.io/)
- [Linux IIO Documentation](https://www.kernel.org/doc/html/latest/driver-api/iio/index.html)
- [Linux PWM Documentation](https://www.kernel.org/doc/Documentation/pwm.txt)
- [Linux LED Class Documentation](https://www.kernel.org/doc/Documentation/leds/leds-class.txt)
- [Zephyr RTOS](https://zephyrproject.org/)
- [Genode OS Framework](https://genode.org/)
- [Redox OS](https://www.redox-os.org/)
