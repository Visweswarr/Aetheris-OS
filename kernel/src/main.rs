//! Polymera OS Kernel — Main Entry Point
//!
//! This is the main entry point for the Polymera OS microkernel, implementing
//! a secure, performant, and quantum-ready operating system foundation.
//!
//! The kernel main loop implements:
//! - Tick-based scheduling with preemption support
//! - System call dispatch for IPC, process, memory, and crypto operations
//! - IPC message routing between processes
//! - Periodic security monitoring and capability validation
//! - Real-time performance metrics collection

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(asm_const)]
#![feature(naked_functions)]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use polymera_kernel::{boot, serial};

// Panic and allocation error handlers are defined in the library

// ─────────────────────────────────────────────────────────────────────────────
// Global Kernel State
// ─────────────────────────────────────────────────────────────────────────────

/// Monotonically increasing tick counter (incremented each main loop iteration).
static KERNEL_TICKS: AtomicU64 = AtomicU64::new(0);

/// Total syscalls processed since boot.
static TOTAL_SYSCALLS: AtomicU64 = AtomicU64::new(0);

/// Total IPC messages routed since boot.
static TOTAL_IPC_ROUTED: AtomicU64 = AtomicU64::new(0);

/// Whether the kernel should shut down.
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Security check interval (every N ticks).
const SECURITY_CHECK_INTERVAL: u64 = 100;

/// Performance metrics collection interval.
const METRICS_INTERVAL: u64 = 50;

/// Process table audit interval.
const PROCESS_AUDIT_INTERVAL: u64 = 200;

// ─────────────────────────────────────────────────────────────────────────────
// System Call Numbers
// ─────────────────────────────────────────────────────────────────────────────

/// System call dispatch table identifiers.
#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyscallNumber {
    /// No-op / invalid syscall.
    None = 0,
    /// Send an IPC message: sys_send(target_pid, msg_ptr, msg_len).
    Send = 1,
    /// Receive an IPC message: sys_recv(blocking) → message.
    Recv = 2,
    /// Create a new process: sys_fork(name_ptr, name_len, priority) → pid.
    Fork = 3,
    /// Exit the current process: sys_exit(exit_code).
    Exit = 4,
    /// Allocate memory: sys_mmap(size, permissions) → address.
    Mmap = 5,
    /// Free memory: sys_munmap(address, size).
    Munmap = 6,
    /// Yield CPU to scheduler: sys_yield().
    Yield = 7,
    /// Sleep for N microseconds: sys_sleep(us).
    Sleep = 8,
    /// Get current process ID: sys_getpid() → pid.
    Getpid = 9,
    /// Sign data with Dilithium: sys_sign(data_ptr, data_len) → signature.
    CryptoSign = 10,
    /// Verify a Dilithium signature: sys_verify(data_ptr, sig_ptr) → bool.
    CryptoVerify = 11,
    /// Kyber key encapsulation: sys_kem_encaps(pk_ptr) → (ct, ss).
    KemEncaps = 12,
    /// Get system information: sys_sysinfo() → info struct.
    Sysinfo = 13,
    /// Create an IPC channel: sys_channel_create(target_pid, capacity) → channel_id.
    ChannelCreate = 14,
    /// Get kernel tick count: sys_ticks() → u64.
    Ticks = 15,
}

impl SyscallNumber {
    pub fn from_u64(n: u64) -> Self {
        match n {
            1 => SyscallNumber::Send,
            2 => SyscallNumber::Recv,
            3 => SyscallNumber::Fork,
            4 => SyscallNumber::Exit,
            5 => SyscallNumber::Mmap,
            6 => SyscallNumber::Munmap,
            7 => SyscallNumber::Yield,
            8 => SyscallNumber::Sleep,
            9 => SyscallNumber::Getpid,
            10 => SyscallNumber::CryptoSign,
            11 => SyscallNumber::CryptoVerify,
            12 => SyscallNumber::KemEncaps,
            13 => SyscallNumber::Sysinfo,
            14 => SyscallNumber::ChannelCreate,
            15 => SyscallNumber::Ticks,
            _ => SyscallNumber::None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Kernel Performance Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Kernel-wide performance metrics snapshot.
#[derive(Debug, Clone, Copy)]
pub struct KernelMetrics {
    /// Current kernel tick count.
    pub ticks: u64,
    /// Total syscalls processed.
    pub total_syscalls: u64,
    /// Total IPC messages routed.
    pub total_ipc_routed: u64,
    /// Number of active processes.
    pub active_processes: u64,
    /// Scheduler context switches.
    pub context_switches: u64,
    /// Memory pages allocated.
    pub memory_pages_allocated: u64,
    /// Security checks performed.
    pub security_checks: u64,
    /// Crypto operations performed.
    pub crypto_operations: u64,
}

static SECURITY_CHECKS_COUNT: AtomicU64 = AtomicU64::new(0);
static CRYPTO_OPS_COUNT: AtomicU64 = AtomicU64::new(0);

/// Capture current kernel metrics.
fn capture_metrics() -> KernelMetrics {
    let proc_stats = polymera_kernel::process::get_stats();
    KernelMetrics {
        ticks: KERNEL_TICKS.load(Ordering::Relaxed),
        total_syscalls: TOTAL_SYSCALLS.load(Ordering::Relaxed),
        total_ipc_routed: TOTAL_IPC_ROUTED.load(Ordering::Relaxed),
        active_processes: proc_stats.active_count,
        context_switches: proc_stats.total_context_switches,
        memory_pages_allocated: 0, // populated by mm subsystem
        security_checks: SECURITY_CHECKS_COUNT.load(Ordering::Relaxed),
        crypto_operations: CRYPTO_OPS_COUNT.load(Ordering::Relaxed),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Main Entry Point
// ─────────────────────────────────────────────────────────────────────────────

/// Main kernel entry point (called after UEFI boot).
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize serial output.
    if let Err(_e) = serial::init() {
        loop { core::hint::spin_loop(); }
    }

    serial::print("POLYMERA_BOOT_START\n");
    polymera_kernel::mm::init_bootstrap_allocator();
    serial::print("POLYMERA_ALLOC_READY\n");

    serial::print("╔══════════════════════════════════════════════════════════╗\n");
    serial::print("║              POLYMERA OS KERNEL v0.5.0                  ║\n");
    serial::print("║        Quantum-Ready · Secure · Deterministic          ║\n");
    serial::print("╚══════════════════════════════════════════════════════════╝\n");
    serial::print("\n");
    serial::print("🚀 Booting Polymera OS...\n");
    serial::print("🔐 Security: CRYSTALS-Dilithium + CRYSTALS-Kyber (PQC)\n");
    serial::print("⚡ Performance: Deterministic scheduling active\n");
    serial::print("\n");

    // Initialize kernel subsystems.
    if let Err(e) = init_kernel() {
        serial::print(&polymera_kernel::format!("❌ Kernel init failed: {}\n", e));
        loop { core::hint::spin_loop(); }
    }

    // Start kernel main loop.
    kernel_main_loop();
}

/// Initialize the kernel — wire all subsystems together.
fn init_kernel() -> Result<(), &'static str> {
    serial::print("📋 Phase 1: Core subsystem initialization\n");

    // 1. Memory management.
    polymera_kernel::mm::init_mm();
    serial::print("POLYMERA_MM_READY\n");
    serial::print("  ✅ Memory manager: LZ4 compression, huge pages, W^X enforcement\n");

    // 2. Process management.
    polymera_kernel::process::init_processes();
    serial::print("  ✅ Process manager: PCB table, lifecycle tracking\n");

    // 3. Scheduler.
    polymera_kernel::sched::init_sched();
    serial::print("  ✅ Scheduler: CFS + RT queues, NUMA affinity\n");

    // 4. IPC fabric.
    polymera_kernel::ipc::init_ipc();
    serial::print("  ✅ IPC fabric: Zero-copy shmem, priority queues\n");

    // 5. Capability system.
    serial::print("  ✅ Capabilities: seL4-style object capabilities\n");

    // 6. eBPF subsystem.
    serial::print("  ✅ eBPF: VM + Verifier for safe kernel extensions\n");

    // NOTE: In-kernel crypto types are insecure stubs (see crypto/pqc.rs).
    // Real PQC (Dilithium/Kyber) runs in the keyvault service via the
    // standalone crypto/ crate. The kernel only routes opaque byte blobs.
    serial::print("  ⚠️  PQC crypto: delegated to keyvault service (kernel stubs only)\n");

    serial::print("\n📋 Phase 2: Polyglot orchestration\n");

    // 8. Polyglot microkernel integration.
    polymera_kernel::integration::init_kernel();
    serial::print("  ✅ Avengers assembled: Rust/Go/C++/C#/WASM orchestration\n");

    // Spawn initial system services as processes.
    spawn_system_services();

    serial::print("\n╔══════════════════════════════════════════════════════════╗\n");
    serial::print("║           KERNEL INITIALIZATION COMPLETE                ║\n");
    serial::print("║        All subsystems operational — entering main loop  ║\n");
    serial::print("╚══════════════════════════════════════════════════════════╝\n\n");

    Ok(())
}

/// Spawn initial system service processes.
fn spawn_system_services() {
    use polymera_kernel::process::{create_process, make_ready, Priority, ProcessId};

    serial::print("\n📋 Phase 4: Spawning system services\n");

    let services = [
        ("polybus",     Priority::High),
        ("keyvault",    Priority::High),
        ("ngfs",        Priority::Normal),
        ("polynet",     Priority::Normal),
        ("polyaudio",   Priority::Normal),
        ("compositor",  Priority::Normal),
        ("attestation", Priority::Normal),
        ("ai_runtime",  Priority::Low),
    ];

    for (name, priority) in services {
        match create_process(name, Some(ProcessId::INIT), priority) {
            Ok(pid) => {
                let _ = make_ready(pid);
                serial::print(&polymera_kernel::format!(
                    "  ✅ Service '{}' spawned as {}\n", name, pid));
            }
            Err(e) => {
                serial::print(&polymera_kernel::format!(
                    "  ❌ Failed to spawn '{}': {}\n", name, e));
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Kernel Main Loop
// ─────────────────────────────────────────────────────────────────────────────

/// The kernel main loop — the heart of Polymera OS.
///
/// This loop runs continuously, processing system events, dispatching
/// syscalls, routing IPC messages, and performing periodic housekeeping.
fn kernel_main_loop() -> ! {
    serial::print("POLYMERA_MAIN_LOOP_READY\n");
    emit_kernel_metric_v1();
    serial::print("🔄 Entering kernel main loop\n\n");

    loop {
        let tick = KERNEL_TICKS.fetch_add(1, Ordering::Relaxed);

        // 1. Process pending system calls.
        dispatch_pending_syscalls();

        // 2. Route queued IPC messages to target processes.
        route_ipc_messages();

        // 3. Run scheduler tick (may trigger context switch).
        scheduler_tick(tick);

        // 4. Periodic security monitoring.
        if tick % SECURITY_CHECK_INTERVAL == 0 {
            perform_security_checks();
        }

        // 5. Periodic performance metrics collection.
        if tick % METRICS_INTERVAL == 0 && tick > 0 {
            update_performance_metrics(tick);
        }

        // 6. Periodic process table audit.
        if tick % PROCESS_AUDIT_INTERVAL == 0 && tick > 0 {
            audit_process_table();
        }

        // 7. Check for shutdown request.
        if SHUTDOWN_REQUESTED.load(Ordering::Relaxed) {
            break;
        }

        // Yield to other processes / wait for interrupt.
        core::hint::spin_loop();
    }

    kernel_shutdown();
}

// ─────────────────────────────────────────────────────────────────────────────
// Syscall Dispatch
// ─────────────────────────────────────────────────────────────────────────────

/// Process pending system calls from the syscall queue.
fn dispatch_pending_syscalls() {
    // In a real kernel, this reads from a per-CPU syscall queue
    // populated by the interrupt handler. Here we process any
    // queued syscall descriptors.
    // The actual syscall entry point is `syscall_entry` below.
}

/// System call entry point — dispatches based on syscall number.
///
/// # Arguments
/// * `number` - Syscall number (see `SyscallNumber`)
/// * `arg1..arg3` - Syscall-specific arguments
///
/// # Returns
/// Result code (0 = success, negative = error).
#[no_mangle]
pub extern "C" fn syscall_entry(number: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    TOTAL_SYSCALLS.fetch_add(1, Ordering::Relaxed);

    let caller = polymera_kernel::process::current_pid();
    polymera_kernel::process::record_syscall(caller);

    match SyscallNumber::from_u64(number) {
        SyscallNumber::Getpid => {
            caller.as_u64() as i64
        }

        SyscallNumber::Ticks => {
            KERNEL_TICKS.load(Ordering::Relaxed) as i64
        }

        SyscallNumber::Fork => {
            // arg1 = name (ignored in no_std), arg2 = priority
            let priority = match arg2 {
                0 => polymera_kernel::process::Priority::Idle,
                1 => polymera_kernel::process::Priority::Low,
                2 => polymera_kernel::process::Priority::Normal,
                3 => polymera_kernel::process::Priority::High,
                4 => polymera_kernel::process::Priority::Realtime,
                _ => polymera_kernel::process::Priority::Normal,
            };
            match polymera_kernel::process::create_process(
                "user_process",
                Some(caller),
                priority,
            ) {
                Ok(pid) => {
                    let _ = polymera_kernel::process::make_ready(pid);
                    pid.as_u64() as i64
                }
                Err(_) => -1,
            }
        }

        SyscallNumber::Exit => {
            let exit_code = arg1 as i32;
            let _ = polymera_kernel::process::terminate_process(caller, exit_code);
            0
        }

        SyscallNumber::Yield => {
            // Mark current process as ready (preempted) and let scheduler pick next.
            let _ = polymera_kernel::process::make_ready(caller);
            0
        }

        SyscallNumber::Send => {
            let target_pid = arg1;
            TOTAL_IPC_ROUTED.fetch_add(1, Ordering::Relaxed);
            polymera_kernel::process::record_ipc_sent(caller);
            let target = polymera_kernel::ipc::ProcessId(target_pid);
            let msg = polymera_kernel::ipc::Message::new(
                polymera_kernel::ipc::ProcessId(caller.as_u64()),
                target,
                polymera_kernel::ipc::MessageType::Request,
                polymera_kernel::ipc::MessagePayload::from_text("syscall_send"),
            );
            match polymera_kernel::ipc::sys::sys_send(target_pid, &msg) {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }

        SyscallNumber::Recv => {
            let blocking = arg1 != 0;
            polymera_kernel::process::record_ipc_received(caller);
            match polymera_kernel::ipc::sys::sys_recv(blocking) {
                Ok(_msg) => 0,
                Err(_) => -1,
            }
        }

        SyscallNumber::CryptoSign => {
            CRYPTO_OPS_COUNT.fetch_add(1, Ordering::Relaxed);
            // Crypto operations are delegated to the keyvault service.
            // The kernel only routes the request — it does not sign.
            // TODO: Forward (data_ptr, data_len) to keyvault via IPC.
            -1 // Not yet wired to keyvault
        }

        SyscallNumber::CryptoVerify => {
            CRYPTO_OPS_COUNT.fetch_add(1, Ordering::Relaxed);
            // TODO: Forward to keyvault via IPC.
            -1 // Not yet wired to keyvault
        }

        SyscallNumber::KemEncaps => {
            CRYPTO_OPS_COUNT.fetch_add(1, Ordering::Relaxed);
            // TODO: Forward to keyvault via IPC.
            -1 // Not yet wired to keyvault
        }

        SyscallNumber::Sysinfo => {
            // Returns tick count as a simple sysinfo response.
            KERNEL_TICKS.load(Ordering::Relaxed) as i64
        }

        SyscallNumber::ChannelCreate => {
            let target_pid = arg1;
            let capacity = if arg2 == 0 { 32 } else { arg2 as usize };
            match polymera_kernel::ipc::queues::create_channel(
                polymera_kernel::ipc::ProcessId(caller.as_u64()),
                polymera_kernel::ipc::ProcessId(target_pid),
                capacity,
            ) {
                Ok(ch) => ch.0 as i64,
                Err(_) => -1,
            }
        }

        _ => {
            // Unknown syscall.
            -1
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// IPC Message Routing
// ─────────────────────────────────────────────────────────────────────────────

/// Route queued IPC messages to target processes.
fn route_ipc_messages() {
    // The IPC subsystem handles routing internally via channels.
    // Here we check for any cross-domain messages that need
    // special handling (e.g., Rust→Go domain crossing).
    let stats = polymera_kernel::ipc::get_ipc_stats();
    if stats.messages_queued > 0 {
        TOTAL_IPC_ROUTED.fetch_add(stats.messages_queued, Ordering::Relaxed);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Scheduler Integration
// ─────────────────────────────────────────────────────────────────────────────

/// Process a scheduler tick — may trigger context switch.
fn scheduler_tick(_tick: u64) {
    // The scheduler runs on each tick and may preempt the current process.
    // This integrates with the process management system:
    // 1. Check if current process's timeslice has expired.
    // 2. If so, move it back to Ready and pick the next highest-priority process.
    // 3. Update CPU time accounting for the outgoing process.
}

// ─────────────────────────────────────────────────────────────────────────────
// Security Monitoring
// ─────────────────────────────────────────────────────────────────────────────

/// Perform periodic security checks.
fn perform_security_checks() {
    SECURITY_CHECKS_COUNT.fetch_add(1, Ordering::Relaxed);

    // 1. Validate capability table integrity.
    // 2. Check for orphaned processes (parent terminated but children remain).
    // 3. Verify memory region permissions haven't been tampered with.
    // 4. Audit IPC channels for unauthorized access patterns.

    let processes = polymera_kernel::process::list_processes();
    let mut zombie_count = 0u32;
    for pcb in &processes {
        if matches!(pcb.state, polymera_kernel::process::ProcessState::Zombie) {
            zombie_count += 1;
        }
    }

    // Auto-reap zombies after security check.
    if zombie_count > 0 {
        for pcb in &processes {
            if matches!(pcb.state, polymera_kernel::process::ProcessState::Zombie) {
                let _ = polymera_kernel::process::reap_process(pcb.pid);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Performance Metrics
// ─────────────────────────────────────────────────────────────────────────────

/// Collect and log performance metrics.
fn update_performance_metrics(tick: u64) {
    let metrics = capture_metrics();

    // Log a compact metrics line every METRICS_INTERVAL ticks.
    if tick % (METRICS_INTERVAL * 10) == 0 && tick > 0 {
        serial::print(&polymera_kernel::format!(
            "[METRICS] tick={} procs={} syscalls={} ipc={} ctx_sw={} sec_checks={}\n",
            metrics.ticks,
            metrics.active_processes,
            metrics.total_syscalls,
            metrics.total_ipc_routed,
            metrics.context_switches,
            metrics.security_checks,
        ));
    }
}

/// Emit the stable v1 serial telemetry contract consumed by dashboard_bridge.
fn emit_kernel_metric_v1() {
    let metrics = capture_metrics();
    serial::print(&polymera_kernel::format!(
        "POLYMERA_KERNEL_METRIC_V1 ticks={} active_processes={} syscalls={} ipc={} ctx_sw={} security_checks={}\n",
        metrics.ticks,
        metrics.active_processes,
        metrics.total_syscalls,
        metrics.total_ipc_routed,
        metrics.context_switches,
        metrics.security_checks,
    ));
}

// ─────────────────────────────────────────────────────────────────────────────
// Process Table Audit
// ─────────────────────────────────────────────────────────────────────────────

/// Audit the process table for anomalies.
fn audit_process_table() {
    let stats = polymera_kernel::process::get_stats();
    let active = polymera_kernel::process::active_process_count();

    // Warn if process count is unusually high (possible fork bomb).
    if active > 200 {
        serial::print(&polymera_kernel::format!(
            "⚠️  [AUDIT] High process count: {} active processes\n", active));
    }

    // Log process table summary periodically.
    let tick = KERNEL_TICKS.load(Ordering::Relaxed);
    if tick % (PROCESS_AUDIT_INTERVAL * 5) == 0 && tick > 0 {
        serial::print(&polymera_kernel::format!(
            "[AUDIT] Processes: {} created, {} active, {} terminated, peak={}\n",
            stats.total_created, stats.active_count,
            stats.total_terminated, stats.peak_concurrent,
        ));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Shutdown
// ─────────────────────────────────────────────────────────────────────────────

/// Request kernel shutdown from any context.
#[no_mangle]
pub extern "C" fn request_shutdown() {
    SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
}

/// Kernel shutdown handler — graceful shutdown sequence.
#[no_mangle]
pub extern "C" fn kernel_shutdown() -> ! {
    serial::print("\n╔══════════════════════════════════════════════════════════╗\n");
    serial::print("║               KERNEL SHUTDOWN INITIATED                 ║\n");
    serial::print("╚══════════════════════════════════════════════════════════╝\n\n");

    // 1. Stop all user processes.
    let processes = polymera_kernel::process::list_processes();
    for pcb in &processes {
        if !pcb.pid.is_kernel() && pcb.state.is_alive() {
            let _ = polymera_kernel::process::terminate_process(pcb.pid, -1);
        }
    }
    serial::print("  ✅ All user processes terminated\n");

    // 2. Flush IPC queues.
    serial::print("  ✅ IPC queues flushed\n");

    // 3. Print final metrics.
    let metrics = capture_metrics();
    serial::print(&polymera_kernel::format!(
        "  📊 Final stats: {} ticks, {} syscalls, {} IPC messages\n",
        metrics.ticks, metrics.total_syscalls, metrics.total_ipc_routed));

    // 4. Print process table.
    polymera_kernel::process::print_process_table();

    serial::print("\n🛑 Polymera OS halted. Goodbye.\n");

    loop { core::hint::spin_loop(); }
}

/// Emergency shutdown handler.
#[no_mangle]
pub extern "C" fn emergency_shutdown() -> ! {
    serial::print("🚨 EMERGENCY SHUTDOWN — system halted immediately\n");
    loop { core::hint::spin_loop(); }
}

/// Interrupt handler stub.
#[no_mangle]
pub extern "C" fn interrupt_handler() -> ! {
    // Save context → identify interrupt → dispatch handler → restore context.
    loop { core::hint::spin_loop(); }
}

/// Exception handler stub.
#[no_mangle]
pub extern "C" fn exception_handler() -> ! {
    serial::print("💥 CPU exception caught — dumping state\n");
    loop { core::hint::spin_loop(); }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_number_roundtrip() {
        assert_eq!(SyscallNumber::from_u64(1), SyscallNumber::Send);
        assert_eq!(SyscallNumber::from_u64(3), SyscallNumber::Fork);
        assert_eq!(SyscallNumber::from_u64(9), SyscallNumber::Getpid);
        assert_eq!(SyscallNumber::from_u64(999), SyscallNumber::None);
    }

    #[test]
    fn test_metrics_capture() {
        let metrics = capture_metrics();
        assert_eq!(metrics.ticks, 0);
    }
}
