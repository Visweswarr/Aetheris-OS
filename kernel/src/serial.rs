use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref SERIAL1: Mutex<SerialPort> = {
        let mut sp = unsafe { SerialPort::new(0x3F8) };
        sp.init();
        Mutex::new(sp)
    };
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!(crate::serial::SerialWriter, $($arg)*);
    });
}
#[macro_export]
macro_rules! kprintln {
    () => (crate::kprint!("\n"));
    ($fmt:expr) => (crate::kprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (crate::kprint!(concat!($fmt, "\n"), $($arg)*));
}

/// Extended kprintln! macro with u128 support
#[macro_export]
macro_rules! kprintln_ext {
    () => (crate::kprint!("\n"));
    ($fmt:expr) => (crate::kprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => {
        // Use the format module for enhanced formatting
        use crate::format;
        crate::kprint!(concat!($fmt, "\n"), $($arg)*);
    };
}

pub struct SerialWriter;
impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut sp = SERIAL1.lock();
        for b in s.bytes() { sp.send(b); }
        Ok(())
    }
}
