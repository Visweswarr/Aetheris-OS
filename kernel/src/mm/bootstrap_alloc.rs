//! Stage-0 Bootstrap Allocator for Polymera OS
//!
//! A dead-simple bump allocator that lives in a fixed-size static byte array.
//! It requires **zero** runtime initialization — the array is .bss-zeroed at
//! load time — and is used by the `#[global_allocator]` for any heap
//! allocations that happen before `mm::alloc::init()` sets up the real
//! slab/bump/block allocators.
//!
//! Once the real allocators are ready, `deactivate_bootstrap()` is called and
//! all subsequent allocations go through `kmalloc()`.
//!
//! This allocator never frees memory. That's fine: bootstrap allocations are
//! tiny (lazy_static slots, small Vecs during slab init) and permanent.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Size of the bootstrap heap (4 MiB).
/// Normal boot uses this until the full allocator stack is ready; the size is
/// intentionally conservative because early kernel subsystems allocate Vecs,
/// Strings, and small maps before virtual-memory ownership is fully settled.
const BOOTSTRAP_HEAP_SIZE: usize = 4 * 1024 * 1024;

#[repr(align(4096))]
struct BootstrapHeap([u8; BOOTSTRAP_HEAP_SIZE]);

/// Static bootstrap heap — lives in .bss, zeroed at load time.
static mut BOOTSTRAP_HEAP: BootstrapHeap = BootstrapHeap([0u8; BOOTSTRAP_HEAP_SIZE]);

/// Current offset into the bootstrap heap (bump pointer).
static BOOTSTRAP_OFFSET: AtomicUsize = AtomicUsize::new(0);

/// Whether the bootstrap allocator is the active allocator.
/// Starts `true`; set to `false` once the real allocators are ready.
static BOOTSTRAP_ACTIVE: AtomicBool = AtomicBool::new(true);

/// Allocate `size` bytes with the given alignment from the bootstrap heap.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the kernel (static).
/// The caller must ensure `size > 0` and `align` is a power of two.
pub unsafe fn bootstrap_alloc(size: usize, align: usize) -> *mut u8 {
    let align = align.max(core::mem::align_of::<usize>());

    // Spin-free bump allocation using compare-and-swap.
    loop {
        let current = BOOTSTRAP_OFFSET.load(Ordering::Relaxed);

        // Align the current offset.
        let aligned = match current.checked_add(align - 1) {
            Some(value) => value & !(align - 1),
            None => return core::ptr::null_mut(),
        };
        let new_end = match aligned.checked_add(size) {
            Some(value) => value,
            None => return core::ptr::null_mut(),
        };

        if new_end > BOOTSTRAP_HEAP_SIZE {
            // Out of bootstrap heap space — return null.
            // The alloc_error_handler will fire, but at least we tried.
            return core::ptr::null_mut();
        }

        // Try to claim this range atomically.
        match BOOTSTRAP_OFFSET.compare_exchange_weak(
            current,
            new_end,
            Ordering::AcqRel,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                // Success — return pointer into the static buffer.
                let base = core::ptr::addr_of_mut!(BOOTSTRAP_HEAP) as *mut u8;
                return base.add(aligned);
            }
            Err(_) => {
                // Another thread raced us; retry.
                continue;
            }
        }
    }
}

/// Mark the bootstrap allocator active. This does not reset the bump offset.
#[inline]
pub fn activate_bootstrap() {
    BOOTSTRAP_ACTIVE.store(true, Ordering::Release);
}

/// Check whether the bootstrap allocator is currently the active allocator.
#[inline]
pub fn is_active() -> bool {
    BOOTSTRAP_ACTIVE.load(Ordering::Acquire)
}

/// Return true when `ptr` belongs to the static bootstrap heap.
#[inline]
pub fn is_bootstrap_ptr(ptr: *mut u8) -> bool {
    let base = core::ptr::addr_of_mut!(BOOTSTRAP_HEAP) as usize;
    let addr = ptr as usize;
    addr >= base && addr < base + BOOTSTRAP_HEAP_SIZE
}

/// Deactivate the bootstrap allocator, handing off to the real allocators.
///
/// Called at the end of `mm::alloc::init()` after the slab/bump/block
/// allocators are fully initialized.
pub fn deactivate_bootstrap() {
    let used = BOOTSTRAP_OFFSET.load(Ordering::Relaxed);
    BOOTSTRAP_ACTIVE.store(false, Ordering::Release);

    // Log how much bootstrap heap was consumed.
    crate::kprintln!(
        "[BOOTSTRAP] Deactivated — {} / {} bytes used ({} bytes remaining)",
        used,
        BOOTSTRAP_HEAP_SIZE,
        BOOTSTRAP_HEAP_SIZE - used
    );
}
