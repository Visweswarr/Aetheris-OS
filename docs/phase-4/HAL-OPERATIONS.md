# HAL Operations Guide

## Overview

This document provides comprehensive guidance for Hardware Abstraction Layer (HAL) operations in Aetheris OS Phase 4. It covers provider selection, permissions management, safety rails, and operational procedures for device runtime services.

## Table of Contents

1. [Provider Selection Rules](#provider-selection-rules)
2. [Permissions Management](#permissions-management)
3. [Safety Rails](#safety-rails)
4. [Device Operations](#device-operations)
5. [Troubleshooting](#troubleshooting)
6. [Best Practices](#best-practices)

## Provider Selection Rules

### Available Providers

#### Deterministic Provider (Default)
- **Purpose**: Testing, development, and deterministic replay
- **Characteristics**: Mock devices, predictable behavior, no hardware dependencies
- **Use Cases**: CI/CD testing, development, deterministic replay
- **Performance**: Fast, consistent, no I/O overhead

#### Linux Provider
- **Purpose**: Production hardware access
- **Characteristics**: Real hardware devices, system-level I/O
- **Use Cases**: Production deployment, real device testing
- **Performance**: Variable, depends on hardware and drivers

### Provider Selection Logic

#### Automatic Selection
```bash
# Environment-based selection
export AETHERIS_DEVICE_PROVIDER=deterministic  # Default
export AETHERIS_DEVICE_PROVIDER=linux          # Production

# Runtime selection
devctl devices provider set deterministic
devctl devices provider set linux
```

#### Selection Rules
1. **Development Environment**: Always use deterministic provider
2. **CI/CD Testing**: Use deterministic provider for consistency
3. **Production Deployment**: Use Linux provider for real hardware
4. **Hardware Testing**: Use Linux provider with appropriate permissions
5. **Fallback**: If Linux provider fails, automatically fall back to deterministic

#### Provider Validation
```bash
# Check current provider
devctl devices provider get

# List available providers
devctl devices provider list

# Validate provider capabilities
devctl devices provider validate --provider linux
```

### Provider Switching

#### Safe Provider Switching
```bash
# Check current state
devctl devices provider get

# Stop active operations
devctl devices stop --all

# Switch provider
devctl devices provider set linux

# Verify switch
devctl devices provider get

# Restart operations
devctl devices start --all
```

#### Emergency Fallback
```bash
# Force fallback to deterministic
devctl devices provider set deterministic --force

# Verify fallback
devctl devices provider get
```

## Permissions Management

### Required Groups

#### Video Devices (Camera)
```bash
# Add user to video group
sudo usermod -a -G video $USER

# Verify group membership
groups $USER

# Check device permissions
ls -la /dev/video*
```

#### Audio Devices (Microphone)
```bash
# Add user to audio group
sudo usermod -a -G audio $USER

# Verify group membership
groups $USER

# Check device permissions
ls -la /dev/snd/*
```

#### GPIO Devices
```bash
# Add user to gpio group
sudo usermod -a -G gpio $USER

# Verify group membership
groups $USER

# Check device permissions
ls -la /dev/gpiochip*
```

#### IIO Devices (ADC)
```bash
# Add user to iio group
sudo usermod -a -G iio $USER

# Verify group membership
groups $USER

# Check device permissions
ls -la /sys/bus/iio/devices/
```

#### PWM Devices (Actuators)
```bash
# Add user to pwm group
sudo usermod -a -G pwm $USER

# Verify group membership
groups $USER

# Check device permissions
ls -la /sys/class/pwm/
```

### Permission Verification

#### Check Device Access
```bash
# Test camera access
v4l2-ctl --device=/dev/video0 --list-formats

# Test audio access
arecord -D hw:0,0 -f cd -t wav -d 1 test.wav

# Test GPIO access
gpiodetect
gpioinfo gpiochip0

# Test IIO access
cat /sys/bus/iio/devices/iio:device0/in_voltage0_raw

# Test PWM access
ls -la /sys/class/pwm/pwmchip0/
```

#### Permission Troubleshooting
```bash
# Check current user groups
groups

# Check device ownership
ls -la /dev/video0 /dev/gpiochip0

# Fix device permissions (if needed)
sudo chmod 666 /dev/video0
sudo chmod 666 /dev/gpiochip0

# Restart services after permission changes
sudo systemctl restart aetheris-device
```

### Capability-Based Access

#### CapToken Requirements
```bash
# Required capabilities for device access
device:camera.read      # Camera access
device:mic.read         # Microphone access
device:gpio.read        # GPIO read access
device:gpio.write       # GPIO write access
device:adc.sample       # ADC sampling
device:actuator.drive   # Actuator control
```

#### CapToken Validation
```bash
# Check CapToken status
devctl security captoken-status --device camera

# Validate CapToken
devctl security captoken-validate --token device:camera.read

# Refresh CapToken
devctl security captoken-refresh --device camera
```

## Safety Rails

### Dry-Run Mode

#### Actuator Safety
```bash
# Enable dry-run mode for actuators
export AETHERIS_ACTUATOR_DRY_RUN=true

# Test actuator commands without execution
devctl actuator pwm --chip pwmchip0 --channel 0 --duty 50% --freq 1kHz --dry-run

# Verify dry-run mode
devctl actuator status --dry-run
```

#### GPIO Safety
```bash
# Enable dry-run mode for GPIO
export AETHERIS_GPIO_DRY_RUN=true

# Test GPIO commands without execution
devctl gpio write --chip gpiochip0 --line 17 --value 1 --dry-run

# Verify dry-run mode
devctl gpio status --dry-run
```

### Time-to-Live (TTL) Limits

#### Actuator TTL
```bash
# Set actuator TTL limits
export AETHERIS_ACTUATOR_TTL_MS=1000
export AETHERIS_ACTUATOR_MAX_DUTY=80
export AETHERIS_ACTUATOR_MAX_FREQ=10000

# Test TTL enforcement
devctl actuator pwm --chip pwmchip0 --channel 0 --duty 90% --freq 15000 --ttl 500
```

#### GPIO TTL
```bash
# Set GPIO TTL limits
export AETHERIS_GPIO_TTL_MS=500
export AETHERIS_GPIO_MAX_TOGGLE_RATE=100

# Test TTL enforcement
devctl gpio write --chip gpiochip0 --line 17 --value 1 --ttl 200
```

### Capability Limits

#### Rate Limiting
```bash
# Set rate limits
export AETHERIS_DEVICE_RATE_LIMIT=100  # 100 operations per second
export AETHERIS_GPIO_RATE_LIMIT=50     # 50 GPIO operations per second
export AETHERIS_ACTUATOR_RATE_LIMIT=10 # 10 actuator operations per second

# Test rate limiting
for i in {1..200}; do
    devctl gpio read --chip gpiochip0 --line 17
done
```

#### Resource Limits
```bash
# Set resource limits
export AETHERIS_DEVICE_MEMORY_LIMIT=1024  # 1GB memory limit
export AETHERIS_DEVICE_CPU_LIMIT=80       # 80% CPU limit
export AETHERIS_DEVICE_IO_LIMIT=1000      # 1000 IOPS limit

# Monitor resource usage
devctl devices monitor --memory --cpu --io
```

### Emergency Stops

#### Global Emergency Stop
```bash
# Emergency stop all devices
devctl devices emergency-stop

# Verify emergency stop
devctl devices status
```

#### Device-Specific Emergency Stop
```bash
# Emergency stop specific device
devctl devices emergency-stop --device camera
devctl devices emergency-stop --device gpio
devctl devices emergency-stop --device actuator
```

#### Emergency Stop Recovery
```bash
# Recover from emergency stop
devctl devices emergency-recover

# Verify recovery
devctl devices status
```

## Device Operations

### Camera Operations

#### Basic Camera Control
```bash
# List available cameras
devctl devices list --type camera

# Start camera capture
devctl camera start --device /dev/video0 --format yuv420 --resolution 1920x1080

# Capture frame
devctl camera capture --output artifacts/frame.jpg

# Stop camera
devctl camera stop
```

#### Camera Safety
```bash
# Enable camera safety mode
export AETHERIS_CAMERA_SAFETY=true
export AETHERIS_CAMERA_MAX_RESOLUTION=1920x1080
export AETHERIS_CAMERA_MAX_FPS=30

# Test camera safety
devctl camera start --device /dev/video0 --resolution 4096x4096 --fps 120
```

### Audio Operations

#### Basic Audio Control
```bash
# List available audio devices
devctl devices list --type audio

# Start microphone capture
devctl mic start --device hw:0,0 --sample-rate 48000 --channels 2

# Capture audio
devctl mic capture --output artifacts/audio.wav --duration 5

# Stop microphone
devctl mic stop
```

#### Audio Safety
```bash
# Enable audio safety mode
export AETHERIS_AUDIO_SAFETY=true
export AETHERIS_AUDIO_MAX_SAMPLE_RATE=48000
export AETHERIS_AUDIO_MAX_CHANNELS=2

# Test audio safety
devctl mic start --device hw:0,0 --sample-rate 192000 --channels 8
```

### GPIO Operations

#### Basic GPIO Control
```bash
# List available GPIO chips
devctl devices list --type gpio

# Read GPIO
devctl gpio read --chip gpiochip0 --line 17

# Write GPIO
devctl gpio write --chip gpiochip0 --line 17 --value 1

# Configure GPIO
devctl gpio configure --chip gpiochip0 --line 17 --direction output --pull up
```

#### GPIO Safety
```bash
# Enable GPIO safety mode
export AETHERIS_GPIO_SAFETY=true
export AETHERIS_GPIO_MAX_VOLTAGE=3.3
export AETHERIS_GPIO_MAX_CURRENT=0.1

# Test GPIO safety
devctl gpio write --chip gpiochip0 --line 17 --value 1 --voltage 5.0
```

### ADC Operations

#### Basic ADC Control
```bash
# List available ADC devices
devctl devices list --type adc

# Sample ADC
devctl adc sample --iio /sys/bus/iio/devices/iio:device0 --channel in_voltage0_raw

# Configure ADC
devctl adc configure --iio /sys/bus/iio/devices/iio:device0 --channel in_voltage0_raw --scale 1.0 --offset 0.0
```

#### ADC Safety
```bash
# Enable ADC safety mode
export AETHERIS_ADC_SAFETY=true
export AETHERIS_ADC_MAX_VOLTAGE=3.3
export AETHERIS_ADC_MAX_SAMPLE_RATE=1000

# Test ADC safety
devctl adc sample --iio /sys/bus/iio/devices/iio:device0 --channel in_voltage0_raw --voltage 5.0
```

### Actuator Operations

#### Basic Actuator Control
```bash
# List available actuators
devctl devices list --type actuator

# Control PWM
devctl actuator pwm --chip pwmchip0 --channel 0 --duty 50% --freq 1kHz

# Control relay
devctl actuator relay --chip gpiochip0 --line 18 --state on

# Control motor
devctl actuator motor --chip pwmchip0 --channel 1 --speed 75% --direction forward
```

#### Actuator Safety
```bash
# Enable actuator safety mode
export AETHERIS_ACTUATOR_SAFETY=true
export AETHERIS_ACTUATOR_MAX_DUTY=80
export AETHERIS_ACTUATOR_MAX_FREQ=10000
export AETHERIS_ACTUATOR_MAX_CURRENT=2.0

# Test actuator safety
devctl actuator pwm --chip pwmchip0 --channel 0 --duty 90% --freq 15000 --current 5.0
```

## Troubleshooting

### Provider Issues

#### Provider Not Available
**Symptoms:**
- "Provider not available" error
- Fallback to deterministic provider
- Device operations fail

**Solutions:**
```bash
# Check provider availability
devctl devices provider list

# Check provider dependencies
devctl devices provider validate --provider linux

# Install missing dependencies
sudo apt-get install -y libgpiod-dev libiio-dev

# Restart device service
sudo systemctl restart aetheris-device
```

#### Provider Switch Failed
**Symptoms:**
- Provider switch command fails
- Devices remain on old provider
- Inconsistent behavior

**Solutions:**
```bash
# Force provider switch
devctl devices provider set linux --force

# Stop all devices before switch
devctl devices stop --all
devctl devices provider set linux
devctl devices start --all

# Check provider status
devctl devices provider get
```

### Permission Issues

#### Device Access Denied
**Symptoms:**
- "Permission denied" errors
- Device operations fail
- Group membership issues

**Solutions:**
```bash
# Check group membership
groups $USER

# Add user to required groups
sudo usermod -a -G video,audio,gpio,iio,pwm $USER

# Log out and back in
logout

# Check device permissions
ls -la /dev/video0 /dev/gpiochip0

# Fix device permissions
sudo chmod 666 /dev/video0
sudo chmod 666 /dev/gpiochip0
```

#### CapToken Issues
**Symptoms:**
- CapToken validation failures
- Access denied despite permissions
- Token expiration errors

**Solutions:**
```bash
# Check CapToken status
devctl security captoken-status --device camera

# Refresh CapToken
devctl security captoken-refresh --device camera

# Generate new CapToken
walletctl captoken generate --scope "device:camera.read"
```

### Safety Rail Issues

#### Dry-Run Mode Not Working
**Symptoms:**
- Commands execute despite dry-run mode
- Safety rails not enforced
- Unexpected device behavior

**Solutions:**
```bash
# Check dry-run mode
echo $AETHERIS_ACTUATOR_DRY_RUN
echo $AETHERIS_GPIO_DRY_RUN

# Enable dry-run mode
export AETHERIS_ACTUATOR_DRY_RUN=true
export AETHERIS_GPIO_DRY_RUN=true

# Test dry-run mode
devctl actuator pwm --chip pwmchip0 --channel 0 --duty 50% --dry-run
```

#### TTL Limits Not Enforced
**Symptoms:**
- Commands exceed TTL limits
- Safety limits ignored
- Resource exhaustion

**Solutions:**
```bash
# Check TTL settings
echo $AETHERIS_ACTUATOR_TTL_MS
echo $AETHERIS_GPIO_TTL_MS

# Set TTL limits
export AETHERIS_ACTUATOR_TTL_MS=1000
export AETHERIS_GPIO_TTL_MS=500

# Test TTL enforcement
devctl actuator pwm --chip pwmchip0 --channel 0 --duty 50% --ttl 200
```

## Best Practices

### Provider Management

1. **Development**: Always use deterministic provider
2. **Testing**: Use deterministic provider for consistency
3. **Production**: Use Linux provider with proper permissions
4. **Fallback**: Implement automatic fallback to deterministic
5. **Validation**: Always validate provider capabilities

### Permission Management

1. **Principle of Least Privilege**: Grant minimum required permissions
2. **Group Membership**: Use system groups for device access
3. **CapToken Validation**: Always validate CapTokens before operations
4. **Regular Audits**: Periodically audit permissions and access
5. **Emergency Access**: Maintain emergency access procedures

### Safety Rails

1. **Dry-Run Mode**: Always test commands in dry-run mode first
2. **TTL Limits**: Set appropriate TTL limits for all operations
3. **Rate Limiting**: Implement rate limiting for device operations
4. **Resource Monitoring**: Monitor resource usage continuously
5. **Emergency Stops**: Implement and test emergency stop procedures

### Device Operations

1. **Graceful Shutdown**: Always stop devices gracefully
2. **Error Handling**: Implement comprehensive error handling
3. **Logging**: Log all device operations for audit
4. **Monitoring**: Monitor device health and performance
5. **Recovery**: Implement automatic recovery procedures

### Security

1. **Access Control**: Implement strict access control
2. **Audit Trails**: Maintain comprehensive audit trails
3. **Encryption**: Encrypt sensitive device data
4. **Network Security**: Secure device network communications
5. **Regular Updates**: Keep device drivers and firmware updated

This comprehensive guide ensures safe and reliable HAL operations in Aetheris OS Phase 4, providing the foundation for robust device runtime services.
