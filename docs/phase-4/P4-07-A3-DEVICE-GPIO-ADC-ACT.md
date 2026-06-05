# P4-07-A3: Device GPIO, ADC, and Actuator Runtime

## Overview

This document describes the implementation of GPIO (General Purpose Input/Output), ADC (Analog-to-Digital Converter), and Actuator device runtime for Aetheris OS. The implementation provides capability-gated access to digital pins, analog sampling, and actuator control with deterministic state management and NGFS integration.

## Table of Contents

1. [Architecture](#architecture)
2. [GPIO Operations](#gpio-operations)
3. [ADC Operations](#adc-operations)
4. [Actuator Operations](#actuator-operations)
5. [Deterministic Mode](#deterministic-mode)
6. [NGFS Integration](#ngfs-integration)
7. [Capability Gating](#capability-gating)
8. [DAO Policy Governance](#dao-policy-governance)
9. [Polyglot Bindings](#polyglot-bindings)
10. [Testing and Validation](#testing-and-validation)
11. [Performance Considerations](#performance-considerations)
12. [Security Considerations](#security-considerations)
13. [Examples](#examples)
14. [API Reference](#api-reference)

## Architecture

The device runtime is built on a modular architecture with the following components:

### Core Components

- **GPIO Manager**: Handles digital pin configuration and I/O operations
- **ADC Manager**: Manages analog channel sampling and calibration
- **Actuator Manager**: Controls PWM outputs, relays, motors, and LEDs
- **Device Service**: Orchestrates all device operations and state management
- **Policy Manager**: Enforces capability gating and DAO governance

### Key Features

- **Capability Gating**: All operations require valid CapTokens v2
- **Deterministic Mode**: Ensures reproducible behavior for testing and replay
- **NGFS Integration**: Automatic snapshot creation and restoration
- **DAO Policy Enforcement**: Multi-user environment governance
- **Polyglot Support**: Rust, C, Go, TypeScript, and Python bindings

## GPIO Operations

### Pin Configuration

GPIO pins can be configured as input or output with various pull configurations:

```rust
// Configure GPIO pin as output
let config = GpioConfig {
    pin: 2,
    mode: GpioMode::Output,
    pull: GpioPull::None,
    initial_value: false,
    debounce_ms: 0,
};

gpio_manager.configure_pin(session_id, caps, config).await?;
```

### Digital I/O

Basic digital read and write operations:

```rust
// Write to GPIO pin
gpio_manager.write_pin(session_id, caps, 2, true).await?;

// Read from GPIO pin
let value = gpio_manager.read_pin(session_id, caps, 2).await?;

// Toggle GPIO pin
let new_value = gpio_manager.toggle_pin(session_id, caps, 2).await?;
```

### Pin States

Query pin configuration and current state:

```rust
let state = gpio_manager.get_pin_state(2).await?;
println!("Pin {}: mode={:?}, value={}", state.pin, state.mode, state.value);
```

## ADC Operations

### Channel Configuration

ADC channels can be configured with various sampling parameters:

```rust
let config = AdcConfig {
    channel: 0,
    sample_rate: 1000,
    resolution: 12,
    reference_voltage: 3.3,
    enable_calibration: true,
    calibration_offset: 0.0,
    calibration_scale: 1.0,
    oversampling: 4,
    enable_filtering: true,
    filter_cutoff: 100.0,
};

adc_manager.configure_channel(session_id, caps, config).await?;
```

### Analog Sampling

Single and multi-channel sampling:

```rust
// Enable channel
adc_manager.enable_channel(session_id, caps, 0).await?;

// Single channel sample
let sample = adc_manager.sample_channel(session_id, caps, 0).await?;
println!("Channel {}: {:.3f}V", sample.channel, sample.value);

// Multi-channel sampling
let channels = vec![0, 1, 2];
let samples = adc_manager.sample_channels(session_id, caps, channels).await?;
```

### Calibration

ADC channels support hardware and software calibration:

```rust
// Enable calibration
let config = AdcConfig {
    // ... other fields
    enable_calibration: true,
    calibration_offset: -0.1,  // Offset in volts
    calibration_scale: 1.05,   // Scale factor
};

adc_manager.configure_channel(session_id, caps, config).await?;
```

## Actuator Operations

### Actuator Configuration

Actuators can be configured with various control parameters:

```rust
let config = ActuatorConfig {
    name: "led_1".to_string(),
    type: ActuatorType::LED,
    pin: 3,
    min_value: 0.0,
    max_value: 1.0,
    default_value: 0.0,
    frequency: 1000,
    resolution: 8,
    enable_safety_limits: true,
    safety_min: 0.0,
    safety_max: 1.0,
    ramp_time_ms: 100,
    enable_ramping: false,
};

actuator_manager.configure_actuator(session_id, caps, config).await?;
```

### Actuator Control

Basic actuator control operations:

```rust
// Enable actuator
actuator_manager.enable_actuator(session_id, caps, "led_1").await?;

// Set actuator value
actuator_manager.set_value(session_id, caps, "led_1", 0.5).await?;

// Get current value
let value = actuator_manager.get_value("led_1").await?;
```

### Pattern Control

Actuators support pattern-based control:

```rust
let pattern = ActuatorPattern {
    name: "blink_pattern".to_string(),
    step_count: 3,
    steps: vec![
        ActuatorPatternStep { value: 0.0, duration_ms: 100 },
        ActuatorPatternStep { value: 1.0, duration_ms: 100 },
        ActuatorPatternStep { value: 0.0, duration_ms: 100 },
    ],
    loop: true,
    loop_count: 0,  // Infinite loop
};

actuator_manager.set_pattern(session_id, caps, "led_1", pattern).await?;
```

## Deterministic Mode

Deterministic mode ensures reproducible behavior for testing and replay:

### Enabling Deterministic Mode

```rust
// Enable deterministic mode for all device managers
device_service.enable_deterministic_mode().await;

// Advance tick counter
device_service.advance_tick().await;
```

### Deterministic Operations

In deterministic mode, all operations are recorded with tick counters and can be replayed identically:

```rust
// All operations in deterministic mode are recorded
let sample = adc_manager.sample_channel(session_id, caps, 0).await?;
assert!(sample.deterministic);
assert_eq!(sample.tick_count, current_tick);
```

## NGFS Integration

The device runtime automatically creates and manages NGFS snapshots:

### Snapshot Creation

```rust
// Create comprehensive device snapshot
let snapshot_data = device_service.create_device_snapshot().await?;

// Save to NGFS
ngfs_manager.save_snapshot("device_state.ngfs", snapshot_data).await?;
```

### Snapshot Restoration

```rust
// Load from NGFS
let snapshot_data = ngfs_manager.load_snapshot("device_state.ngfs").await?;

// Restore device state
device_service.restore_device_snapshot(&snapshot_data).await?;
```

### Byte-Stable Manifests

NGFS snapshots use CBOR for deterministic serialization, ensuring identical outputs from identical replays:

```rust
// Snapshot data is byte-stable
let snapshot1 = device_service.create_device_snapshot().await?;
let snapshot2 = device_service.create_device_snapshot().await?;
assert_eq!(snapshot1, snapshot2);  // Identical in deterministic mode
```

## Capability Gating

All device operations require valid CapTokens v2:

### Required Capabilities

- `device:gpio.configure` - Configure GPIO pins
- `device:gpio.read` - Read GPIO pins
- `device:gpio.write` - Write GPIO pins
- `device:adc.configure` - Configure ADC channels
- `device:adc.sample` - Sample ADC channels
- `device:actuator.configure` - Configure actuators
- `device:actuator.control` - Control actuators

### Capability Validation

```rust
// All operations validate capabilities
let result = gpio_manager.write_pin(session_id, caps, pin, value).await;
match result {
    Ok(_) => println!("Operation successful"),
    Err(DeviceError::InvalidCapability) => println!("Insufficient capabilities"),
    Err(e) => println!("Other error: {}", e),
}
```

## DAO Policy Governance

DAO policies control device access in multi-user environments:

### Policy Configuration

```rust
// Set DAO policy for GPIO operations
gpio_manager.set_dao_policy(
    "room_1",
    true,   // allow_configure
    true,   // allow_read
    false,  // allow_write
).await?;
```

### Policy Enforcement

```rust
// Policies are enforced for all operations
let result = gpio_manager.write_pin(session_id, caps, pin, value).await;
match result {
    Ok(_) => println!("Write allowed"),
    Err(DeviceError::DaoPolicyDenied) => println!("Write denied by DAO policy"),
    Err(e) => println!("Other error: {}", e),
}
```

## Polyglot Bindings

The device runtime provides bindings for multiple languages:

### C FFI

```c
#include "devices_gpio.h"

// Configure GPIO pin
aetheris_gpio_config_t config = {
    .pin = 2,
    .mode = AETHERIS_GPIO_MODE_OUTPUT,
    .pull = AETHERIS_GPIO_PULL_NONE,
    .initial_value = false,
    .debounce_ms = 0
};

aetheris_gpio_error_t result = aetheris_gpio_configure_pin(
    session_id, caps, &config
);
```

### Go CLI

```bash
# Configure GPIO pin
devctl gpio configure --pin 2 --mode output --initial-value false

# Read GPIO pin
devctl gpio read 2

# Write GPIO pin
devctl gpio write 2 true

# Toggle GPIO pin
devctl gpio toggle 2
```

### TypeScript Bridge

```typescript
import { GpioBridge } from './gpio_bridge';

const gpio = new GpioBridge();

// Configure GPIO pin
await gpio.configurePin(sessionId, caps, {
    pin: 2,
    mode: 'output',
    pull: 'none',
    initialValue: false,
    debounceMs: 0
});

// Read GPIO pin
const value = await gpio.readPin(sessionId, caps, 2);

// Write GPIO pin
await gpio.writePin(sessionId, caps, 2, true);
```

### Python Validator

```python
from gpio_adc_actuator_validator import DeviceValidator

async def main():
    async with DeviceValidator() as validator:
        # Validate GPIO operations
        result = await validator.validate_gpio_operations()
        assert result["status"] == "passed"
        
        # Validate ADC operations
        result = await validator.validate_adc_operations()
        assert result["status"] == "passed"
        
        # Validate actuator operations
        result = await validator.validate_actuator_operations()
        assert result["status"] == "passed"
```

## Testing and Validation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpio_configure() {
        let gpio_manager = GpioManager::new(policy_manager.clone()).await.unwrap();
        let config = GpioConfig {
            pin: 2,
            mode: GpioMode::Output,
            pull: GpioPull::None,
            initial_value: false,
            debounce_ms: 0,
        };
        
        let result = gpio_manager.configure_pin("test_session", "device:gpio.configure", config).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_device_service_integration() {
    let device_service = DeviceService::new().await.unwrap();
    
    // Test GPIO operations
    let gpio_config = GpioConfig { /* ... */ };
    device_service.gpio_manager().configure_pin("test", "caps", gpio_config).await.unwrap();
    
    // Test ADC operations
    let adc_config = AdcConfig { /* ... */ };
    device_service.adc_manager().configure_channel("test", "caps", adc_config).await.unwrap();
    
    // Test actuator operations
    let actuator_config = ActuatorConfig { /* ... */ };
    device_service.actuator_manager().configure_actuator("test", "caps", actuator_config).await.unwrap();
}
```

### Python Validation

```python
# Run comprehensive validation
python -m gpio_adc_actuator_validator --base-url http://localhost:8080 --verbose

# Run specific tests
python -m pytest tooling/python/gpio_adc_actuator_validator.py::TestDeviceValidator::test_gpio_operations
```

## Performance Considerations

### Sampling Rates

- **GPIO**: Up to 1MHz for digital I/O operations
- **ADC**: Up to 100kHz for analog sampling
- **Actuators**: Up to 10kHz for PWM control

### Latency

- **GPIO**: < 1μs for digital operations
- **ADC**: < 10μs for analog sampling
- **Actuators**: < 100μs for control operations

### Memory Usage

- **GPIO**: ~1KB per configured pin
- **ADC**: ~2KB per configured channel
- **Actuators**: ~3KB per configured actuator

## Security Considerations

### Capability Isolation

- All operations require valid CapTokens v2
- Capabilities are validated at the service boundary
- Invalid capabilities result in immediate rejection

### DAO Policy Enforcement

- Policies are enforced for all operations
- Policy violations are logged and rejected
- Policies can be updated dynamically

### Input Validation

- All input parameters are validated
- Range checks prevent out-of-bounds operations
- Type safety is enforced at compile time

## Examples

### LED Control

```rust
// Configure LED as PWM output
let led_config = ActuatorConfig {
    name: "status_led".to_string(),
    type: ActuatorType::LED,
    pin: 13,
    min_value: 0.0,
    max_value: 1.0,
    default_value: 0.0,
    frequency: 1000,
    resolution: 8,
    enable_safety_limits: true,
    safety_min: 0.0,
    safety_max: 1.0,
    ramp_time_ms: 100,
    enable_ramping: false,
};

actuator_manager.configure_actuator(session_id, caps, led_config).await?;
actuator_manager.enable_actuator(session_id, caps, "status_led").await?;

// Blink LED
for _ in 0..10 {
    actuator_manager.set_value(session_id, caps, "status_led", 1.0).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    actuator_manager.set_value(session_id, caps, "status_led", 0.0).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

### Analog Sensor Reading

```rust
// Configure temperature sensor
let sensor_config = AdcConfig {
    channel: 0,
    sample_rate: 10,
    resolution: 12,
    reference_voltage: 3.3,
    enable_calibration: true,
    calibration_offset: 0.0,
    calibration_scale: 1.0,
    oversampling: 4,
    enable_filtering: true,
    filter_cutoff: 1.0,
};

adc_manager.configure_channel(session_id, caps, sensor_config).await?;
adc_manager.enable_channel(session_id, caps, 0).await?;

// Read temperature
let sample = adc_manager.sample_channel(session_id, caps, 0).await?;
let temperature = (sample.value - 0.5) * 100.0; // Convert to Celsius
println!("Temperature: {:.1}°C", temperature);
```

### Motor Control

```rust
// Configure motor controller
let motor_config = ActuatorConfig {
    name: "main_motor".to_string(),
    type: ActuatorType::Motor,
    pin: 9,
    min_value: -1.0,
    max_value: 1.0,
    default_value: 0.0,
    frequency: 1000,
    resolution: 10,
    enable_safety_limits: true,
    safety_min: -0.8,
    safety_max: 0.8,
    ramp_time_ms: 500,
    enable_ramping: true,
};

actuator_manager.configure_actuator(session_id, caps, motor_config).await?;
actuator_manager.enable_actuator(session_id, caps, "main_motor").await?;

// Control motor speed
actuator_manager.set_value(session_id, caps, "main_motor", 0.5).await?; // 50% forward
tokio::time::sleep(Duration::from_secs(2)).await;
actuator_manager.set_value(session_id, caps, "main_motor", -0.3).await?; // 30% reverse
```

## API Reference

### GPIO Manager

#### `configure_pin(session_id, caps, config)`
Configure a GPIO pin with specified mode and pull configuration.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `config`: GPIO configuration

**Returns:** `Result<(), DeviceError>`

#### `read_pin(session_id, caps, pin)`
Read the current value of a configured GPIO pin.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `pin`: Pin number

**Returns:** `Result<bool, DeviceError>`

#### `write_pin(session_id, caps, pin, value)`
Write a value to a configured GPIO pin.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `pin`: Pin number
- `value`: Value to write

**Returns:** `Result<(), DeviceError>`

#### `toggle_pin(session_id, caps, pin)`
Toggle the current value of a configured GPIO pin.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `pin`: Pin number

**Returns:** `Result<bool, DeviceError>`

### ADC Manager

#### `configure_channel(session_id, caps, config)`
Configure an ADC channel with specified sampling parameters.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `config`: ADC configuration

**Returns:** `Result<(), DeviceError>`

#### `sample_channel(session_id, caps, channel)`
Take a single sample from an enabled ADC channel.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `channel`: Channel number

**Returns:** `Result<AdcSample, DeviceError>`

#### `sample_channels(session_id, caps, channels)`
Take samples from multiple enabled ADC channels.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `channels`: Vector of channel numbers

**Returns:** `Result<Vec<AdcSample>, DeviceError>`

### Actuator Manager

#### `configure_actuator(session_id, caps, config)`
Configure an actuator with specified control parameters.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `config`: Actuator configuration

**Returns:** `Result<(), DeviceError>`

#### `set_value(session_id, caps, name, value)`
Set the output value of an enabled actuator.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `name`: Actuator name
- `value`: Value to set

**Returns:** `Result<(), DeviceError>`

#### `set_pattern(session_id, caps, name, pattern)`
Set a pattern sequence for an actuator to follow.

**Parameters:**
- `session_id`: Session identifier
- `caps`: Capability tokens
- `name`: Actuator name
- `pattern`: Pattern to set

**Returns:** `Result<(), DeviceError>`

### Device Service

#### `enable_deterministic_mode()`
Enable deterministic mode for all device managers.

**Returns:** `()`

#### `disable_deterministic_mode()`
Disable deterministic mode for all device managers.

**Returns:** `()`

#### `advance_tick()`
Advance the tick counter for all device managers.

**Returns:** `()`

#### `create_device_snapshot()`
Create a comprehensive snapshot of all device states.

**Returns:** `Result<Vec<u8>, DeviceError>`

#### `restore_device_snapshot(data)`
Restore device states from a snapshot.

**Parameters:**
- `data`: Snapshot data

**Returns:** `Result<(), DeviceError>`

## Conclusion

The GPIO, ADC, and Actuator device runtime provides a comprehensive solution for hardware control in Aetheris OS. With capability gating, deterministic mode, NGFS integration, and polyglot bindings, it enables secure, reproducible, and accessible device operations for a wide range of applications.

The implementation follows Aetheris OS principles of security, determinism, and multi-language support, making it suitable for both embedded and desktop applications requiring precise hardware control.
