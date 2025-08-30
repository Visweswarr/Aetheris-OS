# Local APIC Timer System

*Version: 0.2.0*  
*Last Updated: {{ now }}*

This document describes the Local APIC (Advanced Programmable Interrupt Controller) timer system implementation in Polymera OS, including calibration, vector management, and operational considerations.

## Overview

The Local APIC timer provides high-precision timing for the kernel scheduler and system operations. It offers:

- **High Precision**: Sub-microsecond timing accuracy
- **Low Jitter**: Consistent interrupt delivery
- **TSC Calibration**: Time Stamp Counter-based frequency calibration
- **Vector Management**: Configurable interrupt vectors
- **Performance Monitoring**: Jitter and overrun tracking

## Architecture

### Timer Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   APIC Timer    │    │   TSC Counter   │    │   Scheduler     │
│   (Hardware)    │◄──►│   (Hardware)    │◄──►│   (Software)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Interrupt      │    │  Calibration    │    │  Jitter Budget  │
│  Handler        │    │  Engine         │    │  Manager        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Register Layout

The APIC timer uses several key registers:

- **LVT Timer Register** (0x320): Timer configuration and vector
- **Timer Initial Count** (0x380): Initial countdown value
- **Timer Current Count** (0x390): Current countdown value
- **Timer Divide Configuration** (0x3E0): Clock divider setting
- **EOI Register** (0xB0): End of Interrupt acknowledgment

## Configuration

### Timer Modes

```rust
pub enum ApicTimerMode {
    OneShot,    // Timer stops after counting down
    Periodic,   // Timer automatically reloads
}
```

### Divide Configuration

The timer supports the following divide ratios:

| Divide Value | Ratio | Frequency Reduction |
|--------------|-------|-------------------|
| 0b0000       | 2     | 1/2               |
| 0b0001       | 4     | 1/4               |
| 0b0010       | 8     | 1/8               |
| 0b0011       | 16    | 1/16              |
| 0b1000       | 32    | 1/32              |
| 0b1001       | 64    | 1/64              |
| 0b1010       | 128   | 1/128             |
| 0b1011       | 1     | 1/1               |

### Vector Assignment

The APIC timer uses vector 32 (IRQ 0) by default, which provides:

- **High Priority**: Early interrupt handling
- **Dedicated Vector**: No conflicts with other devices
- **Legacy Compatibility**: Standard IRQ 0 assignment

## Calibration

### TSC-Based Calibration

The APIC timer is calibrated against the Time Stamp Counter (TSC) for accurate timing:

```rust
pub fn calibrate(&mut self) -> Result<(), HalError> {
    const CALIBRATION_WINDOW_MS: u64 = 10;
    const MIN_SAMPLES: usize = 100;
    
    let mut samples = Vec::new();
    let start_time = Instant::now();
    let target_duration = Duration::from_millis(CALIBRATION_WINDOW_MS);
    
    // Collect TSC samples over calibration window
    while start_time.elapsed() < target_duration && samples.len() < MIN_SAMPLES {
        let tsc1 = self::read_tsc();
        
        // Wait for next timer tick
        let tick_start = self.tick_count.load(Ordering::Relaxed);
        while self.tick_count.load(Ordering::Relaxed) == tick_start {
            core::hint::spin_loop();
        }
        
        let tsc2 = self::read_tsc();
        let tsc_delta = tsc2.wrapping_sub(tsc1);
        
        if tsc_delta > 0 {
            samples.push(tsc_delta);
        }
    }
    
    // Calculate median TSC delta per tick
    samples.sort_unstable();
    let median_tsc_per_tick = samples[samples.len() / 2];
    
    // Convert to TSC per millisecond
    let tsc_per_ms = median_tsc_per_tick * 1000;
    
    self.tsc_per_ms.store(tsc_per_ms, Ordering::Relaxed);
    self.calibrated = true;
    
    Ok(())
}
```

### Calibration Process

1. **Sample Collection**: Gather TSC samples over 10ms window
2. **Tick Synchronization**: Wait for timer tick boundaries
3. **Delta Calculation**: Measure TSC cycles between ticks
4. **Statistical Analysis**: Use median for stability
5. **Frequency Conversion**: Convert to TSC per millisecond

### Calibration Accuracy

- **Window Size**: 10ms provides sufficient samples
- **Minimum Samples**: 100 samples ensure statistical validity
- **Median Selection**: Resistant to outliers and jitter
- **TSC Fencing**: Proper memory barriers prevent skew

## Interrupt Handling

### ISR Implementation

```rust
pub fn handle_interrupt(&self) {
    // Increment tick count
    self.tick_count.fetch_add(1, Ordering::Relaxed);
    
    // Check for overruns (if we're falling behind)
    let current_tick = self.tick_count.load(Ordering::Relaxed);
    let expected_tick = (Instant::now().as_millis() / 1000) as u64;
    
    if current_tick < expected_tick.saturating_sub(2) {
        self.overrun_count.fetch_add(1, Ordering::Relaxed);
    }
    
    // Acknowledge interrupt
    self::send_eoi();
}
```

### EOI Handling

End of Interrupt (EOI) acknowledgment is critical:

- **Immediate Acknowledgment**: Prevents interrupt masking
- **Register Write**: Writes 0 to APIC_EOI register
- **Memory Ordering**: Ensures proper interrupt flow

### Overrun Detection

The system monitors for timer overruns:

- **Expected vs Actual**: Compare tick count with wall time
- **Threshold**: Allow 2 tick tolerance
- **Counter**: Track total overrun occurrences
- **Logging**: Record overrun events for analysis

## Jitter Management

### Jitter Measurement

```rust
pub fn record_jitter(&self, jitter_us: u32) {
    // Map jitter to histogram bin (0-63, each bin represents ~4us)
    let bin = (jitter_us / 4).min(63) as usize;
    
    if let Ok(mut histogram) = self.jitter_histogram.lock() {
        histogram[bin] = histogram[bin].saturating_add(1);
    }
}
```

### Histogram Binning

- **Bin Size**: 4µs per bin for 0-252µs range
- **Total Bins**: 64 bins for comprehensive coverage
- **Memory Efficient**: Fixed-size array storage
- **Real-time Updates**: Lock-free histogram updates

### Performance Thresholds

| Metric | APIC Target | HPET Target | Action |
|--------|-------------|-------------|---------|
| P95 Jitter | ≤250µs | ≤350µs | Normal operation |
| P99 Jitter | ≤500µs | ≤700µs | Warning logged |
| Max Jitter | ≤1ms | ≤1.5ms | Error logged |

## Performance Characteristics

### Timing Accuracy

- **Base Frequency**: 1000Hz (1ms ticks)
- **Jitter P95**: <250µs typical
- **Calibration Time**: 10ms
- **Startup Latency**: <1ms

### Resource Usage

- **Memory**: ~256 bytes per timer instance
- **CPU**: <1% overhead during normal operation
- **Interrupts**: 1000 per second
- **Power**: Minimal impact on power management

### Scalability

- **Per-CPU**: Each CPU has independent timer
- **Vector Sharing**: Configurable vector assignment
- **Load Distribution**: Even interrupt distribution
- **Cache Locality**: Optimized for NUMA systems

## Common Pitfalls

### 1. **TSC Skew**

**Problem**: TSC values can be skewed by CPU frequency scaling or virtualization.

**Solution**: Use proper memory barriers and calibrate against wall time.

```rust
fn read_tsc() -> u64 {
    unsafe {
        // Serialize instruction execution
        _mm_lfence();
        let tsc = __rdtsc();
        _mm_lfence();
        tsc
    }
}
```

### 2. **Interrupt Masking**

**Problem**: Failing to send EOI can mask subsequent interrupts.

**Solution**: Always send EOI in interrupt handler.

```rust
fn send_eoi() {
    let apic_base = unsafe { __rdmsr(0x1B) };
    let apic_base_addr = (apic_base & 0xFFFFF000) as *mut u32;
    
    let eoi_offset = 0xB0;
    
    unsafe {
        ptr::write_volatile(apic_base_addr.add(eoi_offset / 4), 0);
    }
}
```

### 3. **Calibration Race Conditions**

**Problem**: Calibration can be interrupted by other system events.

**Solution**: Use atomic operations and proper synchronization.

```rust
// Use atomic operations for tick counting
self.tick_count.fetch_add(1, Ordering::Relaxed);

// Protect calibration data with mutex
if let Ok(mut cal_data) = self.calibration_data.lock() {
    // Update calibration data
}
```

### 4. **Vector Conflicts**

**Problem**: Multiple devices using the same interrupt vector.

**Solution**: Verify vector availability and handle conflicts gracefully.

```rust
fn set_timer_vector(vector: u8) -> Result<(), HalError> {
    // Check if vector is available
    if !is_vector_available(vector) {
        return Err(HalError::ResourceBusy("Vector in use"));
    }
    
    // Configure timer vector
    // ... implementation
}
```

### 5. **Frequency Drift**

**Problem**: Timer frequency can drift over time due to temperature or aging.

**Solution**: Periodic recalibration and drift compensation.

```rust
// Check calibration health periodically
if self.should_recalibrate() {
    self.calibrate()?;
}
```

## Troubleshooting

### Timer Not Working

1. **Check APIC Support**: Verify CPUID APIC bit is set
2. **Verify MSR**: Check APIC_BASE MSR is enabled
3. **Check Vector**: Ensure vector is not masked
4. **Verify EOI**: Confirm EOI is being sent

### High Jitter

1. **Check Calibration**: Verify TSC calibration is accurate
2. **Monitor Interrupts**: Check for interrupt conflicts
3. **Review Load**: Ensure system is not overloaded
4. **Check Power Management**: Verify CPU frequency scaling

### Calibration Failures

1. **Insufficient Samples**: Increase calibration window
2. **TSC Issues**: Check for TSC skew or virtualization
3. **Interrupt Conflicts**: Verify no other interrupts during calibration
4. **Memory Issues**: Check for memory corruption

## Monitoring and Debugging

### Runtime Monitoring

```rust
// Get timer statistics
let stats = timer.get_stats();
println!("Tick Count: {}", stats.tick_count);
println!("Overrun Count: {}", stats.overrun_count);
println!("Jitter P95: {}µs", stats.jitter_p95_us);
```

### Debug Output

Enable debug logging for detailed operation:

```bash
export RUST_LOG=debug
cargo run --bin polymera-os
```

### Performance Profiling

Use performance counters to monitor timer behavior:

```rust
// Measure interrupt latency
let start = Instant::now();
timer.handle_interrupt();
let latency = start.elapsed();
```

## Future Enhancements

### Planned Improvements

1. **Dynamic Frequency**: Adaptive frequency based on load
2. **Power Management**: Integration with CPU power states
3. **Multi-Core Coordination**: Synchronized timers across cores
4. **Advanced Calibration**: Machine learning-based drift compensation

### Research Areas

1. **Quantum Timing**: Sub-nanosecond precision
2. **Network Synchronization**: PTP integration
3. **Real-time Guarantees**: Formal timing verification
4. **Energy Efficiency**: Power-aware timing algorithms

## References

### Standards

- [Intel 64 and IA-32 Architectures Software Developer's Manual](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [APIC Specification](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3a-part-1-manual.pdf)

### Related Documentation

- [HPET Timer System](HPET.md)
- [Jitter Budget Management](../sched/JITTER-BUDGET.md)
- [Performance Monitoring](../perf/MONITORING.md)

### External Resources

- [OSDev APIC Tutorial](https://wiki.osdev.org/APIC)
- [Linux APIC Implementation](https://github.com/torvalds/linux/tree/master/arch/x86/kernel/apic)
- [FreeBSD APIC Driver](https://github.com/freebsd/freebsd/tree/main/sys/x86/x86/apic_vector.S)

---

*This documentation is maintained as part of the Polymera OS project. For questions or contributions, please refer to the project repository.*
