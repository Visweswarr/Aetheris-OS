
#![no_std]
#![no_main]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn init() -> i32 {
    // Return 0 to indicate success
    0
}

#[no_mangle]
pub extern "C" fn handle_interrupt() {
    // Dummy interrupt handler
}
