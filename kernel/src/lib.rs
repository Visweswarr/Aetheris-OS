#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

// Re-export alloc macros at crate root for convenience
pub use alloc::format;
pub use alloc::vec;

// Re-export lazy_static macro
pub use lazy_static::lazy_static;

// Re-export core::arch::asm for inline assembly
pub use core::arch::asm;

/// println! macro stub for no_std - redirects to kprintln!
#[macro_export]
macro_rules! println {
    () => ($crate::kprintln!());
    ($($arg:tt)*) => ($crate::kprintln!($($arg)*));
}

/// print! macro stub for no_std - redirects to kprint!
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kprint!($($arg)*));
}

/// Allocating `format!` replacement for no_std contexts.
#[macro_export]
macro_rules! kformat {
    ($($arg:tt)*) => {{
        let mut s = ::alloc::string::String::new();
        let _ = ::core::fmt::Write::write_fmt(&mut s, format_args!($($arg)*));
        s
    }};
}

// Core modules - always available in no_std
pub mod boot;
pub mod panic;
pub mod serial;
pub mod hal;
pub mod log;
pub mod mm;
pub mod sched;
pub mod syscall;
pub mod error;
pub mod mem;
pub mod sync;
pub mod arch;
pub mod macros;
pub mod time;
pub mod process;
pub mod caps;
pub mod ebpf;
pub mod power;
pub mod integration;
pub mod scheme;

// Cryptographic primitives
pub mod crypto;

// Feature modules - stub implementations for no_std
pub mod abi;
pub mod demo;
pub mod ipc;
pub mod security;
pub mod secman;
pub mod trace;
pub mod determinism;
pub mod rng;
pub mod format;
pub mod dashboard;
pub mod fault_injection;
pub mod exec;
pub mod crash_dump;
pub mod fuzzing;
pub mod shell;
pub mod flaky_detector;
pub mod audit;
pub mod intent;
pub mod world;
pub mod skills;
pub mod event;
pub mod policy;
pub mod llm;
pub mod examples;
pub mod heap;
pub mod aetheris_polyglot;

// Tests (conditionally compiled)
#[cfg(test)]
pub mod tests;

// Note: Global allocator is defined in mm/mod.rs

// Re-export common types
pub use error::{KernelError, KernelResult, KernelConfig};

/// Target architecture constant
pub const TARGET_ARCH: &str = if cfg!(target_arch = "x86_64") {
    "x86_64"
} else if cfg!(target_arch = "aarch64") {
    "aarch64"
} else {
    "unknown"
};

/// Initialize kernel library components
pub fn init() {
    log::init_levels();
    kprintln!("[PolymeraCore] lib={} target={}", env!("CARGO_PKG_VERSION"), TARGET_ARCH);
    boot::init();
}

/// Kernel main entry point (for library use)
pub fn kernel_main() -> ! {
    init();
    loop { 
        #[cfg(target_arch = "x86_64")]
        unsafe { x86_64::instructions::hlt(); }
        
        #[cfg(not(target_arch = "x86_64"))]
        core::hint::spin_loop();
    }
}
