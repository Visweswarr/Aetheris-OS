//! Model-Specific Register (MSR) access primitives.
//!
//! `core::arch::x86_64` does not expose the `__rdmsr` / `__wrmsr`
//! intrinsics on the toolchain Polymera targets, so we wrap the raw
//! instructions in inline assembly here. Both wrappers are `unsafe`
//! because reading or writing arbitrary MSRs can violate processor
//! state invariants — callers must know what register they're touching
//! and on what privilege level.

use core::arch::asm;

/// Read a Model-Specific Register.
///
/// # Safety
/// `reg` must identify a valid MSR for the current CPU and the caller
/// must be running at CPL0 (ring 0). Reading reserved or undocumented
/// MSRs can fault or behave architecturally undefined.
#[inline]
pub unsafe fn rdmsr(reg: u32) -> u64 {
    let low: u32;
    let high: u32;
    asm!(
        "rdmsr",
        in("ecx") reg,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack, preserves_flags),
    );
    ((high as u64) << 32) | (low as u64)
}

/// Write a Model-Specific Register.
///
/// # Safety
/// Same constraints as `rdmsr`. A bad value to the wrong MSR can lock
/// the CPU, disable interrupts, or corrupt paging — caller verifies
/// the register and the bit layout before calling.
#[inline]
pub unsafe fn wrmsr(reg: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    asm!(
        "wrmsr",
        in("ecx") reg,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack, preserves_flags),
    );
}
