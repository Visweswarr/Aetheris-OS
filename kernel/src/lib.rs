#![no_std]
#![no_main]
#![feature(naked_functions)]

extern crate alloc;

mod boot;
mod panic;
mod serial;
mod hal;
mod log;
mod mm;
mod sched;
mod syscall;
mod abi; // Added for ABI features and versioning
mod demo;
mod ipc;
mod security;
mod secman;
mod trace; // Added for tracing system
mod determinism; // Added for determinism and replay support
mod rng; // Added for randomness proxy
mod format; // Added for enhanced formatting and hexdump
mod dashboard;
mod macros; // Added for kassert macro
mod fault_injection; // Added for fault injection hooks
mod exec; // Added for ELF header parser
mod crash_dump; // Added for enhanced crash dumps
mod fuzzing; // Added for enhanced fuzzing system
mod shell; // Added for minimal line-oriented shell
mod flaky_detector; // Added for flaky test detection and auto-issue creation
mod audit; // Added for audit codes and logging
mod intent; // Added for Intent Kernel v0
mod world; // Added for World Model v0
mod skills; // Added for Skill Runtime v0
mod event; // Added for Event Fabric v0
mod policy; // Added for Policy Guardrail v0
mod llm; // Added for LLM Adapter v0

mod examples; // Added for demo examples
mod tests;

use core::arch::global_asm;
use core::panic::PanicInfo;
use crate::serial::kprintln;

// Include assembly files
global_asm!(include_str!("asm/syscall.S"));

#[no_mangle]
pub extern "C" fn _start() -> ! {
    log::init_levels();
    kprintln!("[PolymeraCore] build={} target=x86_64-unknown-none", env!("CARGO_PKG_VERSION"));
    boot::init();
    loop { x86_64::instructions::hlt(); }
}

