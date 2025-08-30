# Hardware Abstraction Layer (HAL) - Phase 1

## Overview

The Hardware Abstraction Layer provides a unified interface for hardware-specific operations across different architectures, currently focusing on x86_64 with support for GDT, IDT, and timer management.

## Architecture Design

### HAL Trait Definition

```rust
pub trait Hal {
    fn init_cpu() -> Result<(), &'static str>;
    fn init_timer() -> Result<(), &'static str>;
    fn enable_interrupts() -> Result<(), &'static str>;
    fn disable_interrupts() -> Result<(), &'static str>;
    fn get_timestamp() -> u64;
    fn wait_us(microseconds: u64);
    fn get_cpu_info() -> CpuInfo;
}
```

## x86_64 Implementation

### GDT (Global Descriptor Table)

```rust
impl X64Hal {
    fn setup_gdt() -> Result<(), &'static str> {
        unsafe {
            let mut gdt = GlobalDescriptorTable::new();
            
            // Kernel code segment
            let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
            
            // Kernel data segment
            let data_selector = gdt.add_entry(Descriptor::kernel_data_segment(0, 0, 0));
            
            // TSS segment
            let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS.as_ref().unwrap()));
            
            GDT = Some(gdt);
            gdt.load();
            
            // Set up segment registers
            x86_64::instructions::segments::set_cs(code_selector);
            x86_64::instructions::segments::set_ds(data_selector);
            
            Ok(())
        }
    }
}
```

### IDT (Interrupt Descriptor Table)

```rust
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Exception handlers
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        
        // Hardware interrupts
        idt[32].set_handler_fn(timer_interrupt_handler);
        idt[33].set_handler_fn(keyboard_interrupt_handler);
        
        // System call handler
        idt[0x80].set_handler_fn(syscall_interrupt_handler);
        
        idt
    };
}
```

### Timer Management

```rust
pub struct Timer {
    pit: Pit,
    frequency: u32,
    ticks: AtomicU64,
}

impl Timer {
    pub fn init(&mut self, frequency_hz: u32) -> Result<(), &'static str> {
        let divisor = 1193180 / frequency_hz;
        self.pit.configure(divisor);
        self.frequency = frequency_hz;
        Ok(())
    }
    
    pub fn get_ticks(&self) -> u64 {
        self.ticks.load(Ordering::Relaxed)
    }
}
```

## Troubleshooting

### Common Issues

1. **GDT Loading Failures**: Check segment selectors and memory alignment
2. **IDT Handler Failures**: Verify interrupt vectors and PIC configuration
3. **Timer Issues**: Check PIT configuration and interrupt registration
4. **CPU Feature Detection**: Use safe CPUID wrapper for unsupported features

### Debug Commands

```rust
// Enable HAL debugging
const HAL_DEBUG: bool = cfg!(debug_assertions);

fn hal_debug_print(msg: &str) {
    if HAL_DEBUG {
        kprintln!("[HAL_DEBUG] {}", msg);
    }
}
```

## References

- [Phase 1 SPEC](SPEC.md) - Overall Phase 1 specifications
- [Boot System](BOOT.md) - Boot sequence and initialization
- [Memory Management](MM.md) - Memory management system

## Overview

The Hardware Abstraction Layer provides a unified interface for hardware-specific operations across different architectures, currently focusing on x86_64 with support for GDT, IDT, and timer management.

## Architecture Design

### HAL Trait Definition

```rust
pub trait Hal {
    fn init_cpu() -> Result<(), &'static str>;
    fn init_timer() -> Result<(), &'static str>;
    fn enable_interrupts() -> Result<(), &'static str>;
    fn disable_interrupts() -> Result<(), &'static str>;
    fn get_timestamp() -> u64;
    fn wait_us(microseconds: u64);
    fn get_cpu_info() -> CpuInfo;
}
```

## x86_64 Implementation

### GDT (Global Descriptor Table)

```rust
impl X64Hal {
    fn setup_gdt() -> Result<(), &'static str> {
        unsafe {
            let mut gdt = GlobalDescriptorTable::new();
            
            // Kernel code segment
            let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
            
            // Kernel data segment
            let data_selector = gdt.add_entry(Descriptor::kernel_data_segment(0, 0, 0));
            
            // TSS segment
            let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS.as_ref().unwrap()));
            
            GDT = Some(gdt);
            gdt.load();
            
            // Set up segment registers
            x86_64::instructions::segments::set_cs(code_selector);
            x86_64::instructions::segments::set_ds(data_selector);
            
            Ok(())
        }
    }
}
```

### IDT (Interrupt Descriptor Table)

```rust
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // Exception handlers
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        
        // Hardware interrupts
        idt[32].set_handler_fn(timer_interrupt_handler);
        idt[33].set_handler_fn(keyboard_interrupt_handler);
        
        // System call handler
        idt[0x80].set_handler_fn(syscall_interrupt_handler);
        
        idt
    };
}
```

### Timer Management

```rust
pub struct Timer {
    pit: Pit,
    frequency: u32,
    ticks: AtomicU64,
}

impl Timer {
    pub fn init(&mut self, frequency_hz: u32) -> Result<(), &'static str> {
        let divisor = 1193180 / frequency_hz;
        self.pit.configure(divisor);
        self.frequency = frequency_hz;
        Ok(())
    }
    
    pub fn get_ticks(&self) -> u64 {
        self.ticks.load(Ordering::Relaxed)
    }
}
```

## Troubleshooting

### Common Issues

1. **GDT Loading Failures**: Check segment selectors and memory alignment
2. **IDT Handler Failures**: Verify interrupt vectors and PIC configuration
3. **Timer Issues**: Check PIT configuration and interrupt registration
4. **CPU Feature Detection**: Use safe CPUID wrapper for unsupported features

### Debug Commands

```rust
// Enable HAL debugging
const HAL_DEBUG: bool = cfg!(debug_assertions);

fn hal_debug_print(msg: &str) {
    if HAL_DEBUG {
        kprintln!("[HAL_DEBUG] {}", msg);
    }
}
```

## References

- [Phase 1 SPEC](SPEC.md) - Overall Phase 1 specifications
- [Boot System](BOOT.md) - Boot sequence and initialization
- [Memory Management](MM.md) - Memory management system




