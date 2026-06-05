/// Test modules for Polymera OS kernel
/// 
/// This module contains integration tests and benchmarks for kernel functionality

pub mod ipc_ping_pong;
pub mod runqueue_push_pop; // Added for runqueue unit tests
pub mod runqueue_demo; // Added for runqueue demonstration
pub mod virt_map_unmap; // Added for virtual memory map/unmap tests
pub mod slab_alloc; // Added for slab allocator tests
pub mod demo_tasks_test; // Added for demo tasks interleaving tests
pub mod syscall_assembly_test;
pub mod rt_priority_demo_test; // Added for RT priority demo tests
pub mod inbox_overflow; // Added for inbox overflow policy tests
pub mod capability_revocation; // Added for capability revocation tests
pub mod page_fault_diagnostics; // Added for page fault diagnostics tests
pub mod page_fault_demo; // Added for page fault diagnostics demo
pub mod double_fault_handler; // Added for double fault handler with IST support
pub mod determinism;
pub mod fault_injection_tests; // Added for determinism and replay support
pub mod sched; // Added for scheduler fairness tests
pub mod memory_safety; // Added for memory safety system tests
pub mod shell; // Added for shell system tests
pub mod flaky_detector; // Added for flaky test detection and auto-issue creation
pub mod randomness_proxy; // Added for randomness proxy system
pub mod formatting;
pub mod aarch64_hal; // Added for enhanced formatting and hexdump
pub mod dashboard;
pub mod priority_inheritance; // Added for priority inheritance
pub mod starvation_detector; // Added for starvation detection
pub mod kassert_macro; // Added for kassert macro
pub mod fault_injection; // Added for fault injection hooks
pub mod elf_parser; // Added for ELF header parser
pub mod memory_compression_property; // Property test for memory compression trigger (Property 1)
pub mod huge_page_promotion_property; // Property test for huge page promotion (Property 2)
pub mod capability_enforced_access_property; // Property test for capability-enforced memory access (Property 3)

