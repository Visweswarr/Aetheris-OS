/// Integration tests for IPC ping-pong functionality
/// 
/// This file provides integration tests that can be run with `cargo test`

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use polymera_os_kernel::tests::ipc_ping_pong;
use polymera_os_kernel::{kprintln, serial};

/// Test runner for integration tests
fn test_runner(tests: &[&dyn Testable]) {
    serial::serial_println!("Running {} IPC integration tests", tests.len());
    for test in tests {
        test.run();
    }
}

/// Trait for testable functions
trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial::serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial::serial_println!("[ok]");
    }
}

/// Panic handler for tests
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial::serial_println!("[failed]\n");
    serial::serial_println!("Error: {}\n", info);
    polymera_os_kernel::halt_loop();
}

/// Entry point for integration tests
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    polymera_os_kernel::halt_loop();
}

/// Test basic ping-pong functionality
#[test_case]
fn test_basic_ping_pong() {
    // Initialize required subsystems for testing
    polymera_os_kernel::init_for_testing();
    
    let result = ipc_ping_pong::run_simple_ping_pong_test();
    assert!(result, "Basic ping-pong test should pass");
}

/// Test ping-pong with custom parameters
#[test_case]
fn test_custom_ping_pong_5_iterations() {
    let result = ipc_ping_pong::run_custom_ping_pong_test(5, 200);
    assert!(result, "Custom ping-pong test (5 iterations, 200µs) should pass");
}

/// Test ping-pong benchmark
#[test_case]
fn test_ping_pong_benchmark() {
    let result = ipc_ping_pong::run_benchmark_ping_pong_test();
    assert!(result, "Ping-pong benchmark should pass");
}

/// Test all ping-pong variants
#[test_case]
fn test_all_ping_pong_variants() {
    let result = ipc_ping_pong::run_all_ping_pong_tests();
    assert!(result, "All ping-pong test variants should pass");
}

