# Hardware Abstraction Layer (HAL)

This directory contains the Hardware Abstraction Layer for the Polymera OS kernel, providing architecture-independent interfaces to hardware-specific functionality.

## Architecture

### Trait-Based Design
The HAL uses a trait-based design for cross-architecture compatibility:

```rust
pub trait Hal {
    fn init_cpu();
    fn init_timer();
    fn enable_interrupts();
}
```

### Supported Architectures
- **x86_64**: Primary target with GDT, IDT, and timer support
- **aarch64**: Secondary target (placeholder implementation)

## Module Structure

```
hal/
├── mod.rs              # Main HAL trait and architecture selection
├── x86_64/
│   ├── mod.rs          # x86_64 HAL implementation
│   ├── gdt.rs          # Global Descriptor Table setup
│   ├── idt.rs          # Interrupt Descriptor Table setup
│   └── timer.rs        # Timer initialization (PIT/APIC)
└── aarch64/
    └── mod.rs          # aarch64 HAL implementation (placeholder)
```

## x86_64 Implementation

### X64Hal Structure
```rust
pub struct X64Hal;

impl crate::hal::Hal for X64Hal {
    fn init_cpu() { gdt::init(); idt::init(); }
    fn init_timer() { timer::init(); }
    fn enable_interrupts() { unsafe { x86_64::instructions::interrupts::enable(); } }
}
```

### Components

#### GDT (Global Descriptor Table)
- **File**: `x86_64/gdt.rs`
- **Purpose**: Memory segmentation setup
- **Status**: Stub implementation (TODO: real GDT)

#### IDT (Interrupt Descriptor Table)
- **File**: `x86_64/idt.rs`
- **Purpose**: Interrupt handler setup
- **Status**: Stub implementation (TODO: real IDT + handlers)

#### Timer
- **File**: `x86_64/timer.rs`
- **Purpose**: System timer initialization
- **Status**: Stub implementation (TODO: PIT/APIC)

## Usage

### Boot Integration
The HAL is integrated into the boot sequence in `boot.rs`:

```rust
#[cfg(target_arch = "x86_64")]
{
    X64Hal::init_cpu();     // Initialize GDT and IDT
    X64Hal::init_timer();   // Setup system timer
    X64Hal::enable_interrupts(); // Enable interrupts
}
```

### Expected Output
When the kernel boots, you should see:
```
[PolymeraCore] boot::init()
[HAL] GDT init
[HAL] IDT init
[HAL] Timer init
[PHASE1 PASS] boot init sequence completed
```

## Future Development

### Phase 1 Completion Tasks
1. **Real GDT Implementation**: Replace stub with actual segment descriptors
2. **Complete IDT Setup**: Add interrupt handlers for exceptions and IRQs
3. **Timer Implementation**: PIT or APIC timer configuration
4. **Memory Management**: Integration with paging and virtual memory
5. **Multi-core Support**: SMP initialization and per-CPU data

### aarch64 Implementation
The aarch64 HAL is currently a placeholder. Future implementation should include:
- Exception level management
- Memory Management Unit (MMU) setup
- Generic Interrupt Controller (GIC) configuration
- Generic Timer initialization

## Testing

The HAL can be tested as part of the kernel boot process:
```bash
# Build kernel
cd kernel
cargo build --target x86_64-unknown-none --release

# Test in QEMU
../tooling/qemu/run_x86_64.sh ../bazel-bin/kernel/polymera-kernel.bin
```

## Architecture Notes

### x86_64 Specifics
- **Segment Model**: Uses flat memory model with minimal segmentation
- **Interrupt Model**: Uses APIC for modern interrupt handling
- **Timer Model**: Targets APIC timer for precise timing
- **Privilege Levels**: Ring 0 (kernel) and Ring 3 (user)

### Cross-Architecture Compatibility
The trait-based design ensures that kernel code outside the HAL doesn't need to know about architecture-specific details. This enables:
- Clean separation of concerns
- Easy addition of new architectures
- Consistent kernel interface across platforms
- Better testability and maintainability

## Integration with Phase 0

The HAL integrates with Phase 0 services:
- **Logging**: Uses kernel logging macros for debug output
- **Performance**: Will integrate with SLO monitoring for boot timing
- **Security**: Will enforce capability-based access control for hardware resources

## Dependencies

### Rust Crates
- `x86_64`: Low-level x86_64 operations and data structures
- `spin`: Spinlocks for synchronization
- `lazy_static`: Static initialization of hardware structures

### Kernel Modules
- `log`: Kernel logging system
- `serial`: Debug output via serial console
- Core kernel modules for memory management and synchronization
