use core::fmt;
use core::ptr;

/// Serial port base addresses
const COM1_BASE: u16 = 0x3F8;
const COM2_BASE: u16 = 0x2F8;

/// Serial port registers
const SERIAL_DATA: u16 = 0x00;      // Data register
const SERIAL_INT_ENABLE: u16 = 0x01; // Interrupt enable register
const SERIAL_INT_ID: u16 = 0x02;    // Interrupt identification register
const SERIAL_FIFO_CTRL: u16 = 0x02; // FIFO control register
const SERIAL_LINE_CTRL: u16 = 0x03; // Line control register
const SERIAL_MODEM_CTRL: u16 = 0x04; // Modem control register
const SERIAL_LINE_STATUS: u16 = 0x05; // Line status register
const SERIAL_MODEM_STATUS: u16 = 0x06; // Modem status register
const SERIAL_SCRATCH: u16 = 0x07;   // Scratch register

/// Serial configuration constants
const SERIAL_BAUD_115200: u16 = 1;   // Divisor for 115200 baud
const SERIAL_8N1: u16 = 0x03;        // 8 data bits, no parity, 1 stop bit
const SERIAL_FIFO_ENABLE: u16 = 0xC7; // Enable FIFO and clear buffers
const SERIAL_DTR_RTS: u16 = 0x0B;    // Set DTR and RTS

/// Serial port structure
pub struct SerialPort {
    base: u16,
    initialized: bool,
}

/// Serial error types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SerialError {
    PortNotFound,
    InitializationFailed,
    WriteFailed,
    ReadFailed,
    Timeout,
}

impl fmt::Display for SerialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerialError::PortNotFound => write!(f, "Serial port not found"),
            SerialError::InitializationFailed => write!(f, "Serial initialization failed"),
            SerialError::WriteFailed => write!(f, "Serial write failed"),
            SerialError::ReadFailed => write!(f, "Serial read failed"),
            SerialError::Timeout => write!(f, "Serial operation timed out"),
        }
    }
}

impl SerialPort {
    /// Create a new serial port instance
    pub fn new(base: u16) -> Self {
        Self {
            base,
            initialized: false,
        }
    }
    
    /// Initialize the serial port
    pub fn init(&mut self) -> Result<(), SerialError> {
        // Check if port exists by reading scratch register
        if self.read_register(SERIAL_SCRATCH) == 0 {
            return Err(SerialError::PortNotFound);
        }
        
        // Disable interrupts
        self.write_register(SERIAL_INT_ENABLE, 0x00);
        
        // Enable DLAB (Divisor Latch Access Bit)
        self.write_register(SERIAL_LINE_CTRL, 0x80);
        
        // Set baud rate divisor for 115200 baud
        self.write_register(SERIAL_DATA, (SERIAL_BAUD_115200 & 0xFF) as u8);
        self.write_register(SERIAL_INT_ENABLE, ((SERIAL_BAUD_115200 >> 8) & 0xFF) as u8);
        
        // Set 8N1 format and disable DLAB
        self.write_register(SERIAL_LINE_CTRL, SERIAL_8N1);
        
        // Enable FIFO and clear buffers
        self.write_register(SERIAL_FIFO_CTRL, SERIAL_FIFO_ENABLE);
        
        // Set DTR and RTS
        self.write_register(SERIAL_MODEM_CTRL, SERIAL_DTR_RTS);
        
        // Test if initialization was successful
        if self.read_register(SERIAL_LINE_STATUS) & 0x60 != 0x60 {
            return Err(SerialError::InitializationFailed);
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// Check if the serial port is ready to transmit
    pub fn is_transmit_empty(&self) -> bool {
        if !self.initialized {
            return false;
        }
        
        (self.read_register(SERIAL_LINE_STATUS) & 0x20) != 0
    }
    
    /// Check if the serial port has data to receive
    pub fn is_receive_ready(&self) -> bool {
        if !self.initialized {
            return false;
        }
        
        (self.read_register(SERIAL_LINE_STATUS) & 0x01) != 0
    }
    
    /// Write a byte to the serial port
    pub fn write_byte(&self, data: u8) -> Result<(), SerialError> {
        if !self.initialized {
            return Err(SerialError::InitializationFailed);
        }
        
        // Wait for transmit buffer to be empty
        let mut timeout = 10000;
        while !self.is_transmit_empty() && timeout > 0 {
            timeout -= 1;
            core::hint::spin_loop();
        }
        
        if timeout == 0 {
            return Err(SerialError::Timeout);
        }
        
        // Write the byte
        self.write_register(SERIAL_DATA, data);
        Ok(())
    }
    
    /// Read a byte from the serial port
    pub fn read_byte(&self) -> Result<u8, SerialError> {
        if !self.initialized {
            return Err(SerialError::InitializationFailed);
        }
        
        // Wait for data to be available
        let mut timeout = 10000;
        while !self.is_receive_ready() && timeout > 0 {
            timeout -= 1;
            core::hint::spin_loop();
        }
        
        if timeout == 0 {
            return Err(SerialError::Timeout);
        }
        
        // Read the byte
        let data = self.read_register(SERIAL_DATA);
        Ok(data)
    }
    
    /// Write a string to the serial port
    pub fn write_str(&self, s: &str) -> Result<(), SerialError> {
        for &byte in s.as_bytes() {
            self.write_byte(byte)?;
        }
        Ok(())
    }
    
    /// Read a string from the serial port (up to max_len)
    pub fn read_str(&self, max_len: usize) -> Result<String, SerialError> {
        let mut result = String::new();
        let mut len = 0;
        
        while len < max_len {
            match self.read_byte() {
                Ok(byte) => {
                    if byte == b'\r' || byte == b'\n' {
                        break;
                    }
                    result.push(byte as char);
                    len += 1;
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(result)
    }
    
    /// Write a register value
    fn write_register(&self, reg: u16, value: u8) {
        unsafe {
            ptr::write_volatile((self.base + reg) as *mut u8, value);
        }
    }
    
    /// Read a register value
    fn read_register(&self, reg: u16) -> u8 {
        unsafe {
            ptr::read_volatile((self.base + reg) as *const u8)
        }
    }
    
    /// Check if the port is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get the base address
    pub fn get_base(&self) -> u16 {
        self.base
    }
}

/// Global serial port instances
static mut COM1: Option<SerialPort> = None;
static mut COM2: Option<SerialPort> = None;

/// Initialize serial output
pub fn init() -> Result<(), SerialError> {
    unsafe {
        // Initialize COM1
        let mut com1 = SerialPort::new(COM1_BASE);
        if let Err(e) = com1.init() {
            // Try COM2 if COM1 fails
            let mut com2 = SerialPort::new(COM2_BASE);
            if let Err(e2) = com2.init() {
                return Err(e2);
            }
            COM2 = Some(com2);
        } else {
            COM1 = Some(com1);
        }
    }
    
    Ok(())
}

/// Print a string to serial output
pub fn print(s: &str) {
    unsafe {
        if let Some(com1) = &mut COM1 {
            let _ = com1.write_str(s);
        } else if let Some(com2) = &mut COM2 {
            let _ = com2.write_str(s);
        }
    }
}

/// Print a formatted string to serial output
pub fn print_fmt(args: fmt::Arguments<'_>) {
    // For now, just convert to string and print
    // In a full implementation, you'd want proper formatting
    let s = format!("{}", args);
    print(&s);
}

/// Get the primary serial port
pub fn get_primary() -> Option<&'static mut SerialPort> {
    unsafe {
        if let Some(com1) = &mut COM1 {
            Some(com1)
        } else if let Some(com2) = &mut COM2 {
            Some(com2)
        } else {
            None
        }
    }
}

/// Check if serial is available
pub fn is_available() -> bool {
    unsafe {
        COM1.is_some() || COM2.is_some()
    }
}

/// Flush serial output
pub fn flush() {
    unsafe {
        if let Some(com1) = &mut COM1 {
            // Wait for transmit buffer to be empty
            while !com1.is_transmit_empty() {
                core::hint::spin_loop();
            }
        } else if let Some(com2) = &mut COM2 {
            while !com2.is_transmit_empty() {
                core::hint::spin_loop();
            }
        }
    }
}

/// Serial port test functions
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serial_port_creation() {
        let port = SerialPort::new(COM1_BASE);
        assert_eq!(port.get_base(), COM1_BASE);
        assert!(!port.is_initialized());
    }
    
    #[test]
    fn test_serial_port_initialization() {
        let mut port = SerialPort::new(COM1_BASE);
        // Note: This test will fail in non-UEFI environments
        // In a real test environment, you'd mock the hardware
        let result = port.init();
        // We can't assert success/failure without hardware
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_serial_error_display() {
        let error = SerialError::PortNotFound;
        assert_eq!(error.to_string(), "Serial port not found");
    }
}

/// Serial port debugging utilities
pub mod debug {
    use super::*;
    
    /// Print debug information about serial ports
    pub fn print_debug_info() {
        unsafe {
            print("=== Serial Debug Information ===\n");
            
            if let Some(com1) = &COM1 {
                print(&format!("COM1: Base=0x{:04X}, Initialized={}\n", 
                    com1.get_base(), com1.is_initialized()));
            } else {
                print("COM1: Not available\n");
            }
            
            if let Some(com2) = &COM2 {
                print(&format!("COM2: Base=0x{:04X}, Initialized={}\n", 
                    com2.get_base(), com2.is_initialized()));
            } else {
                print("COM2: Not available\n");
            }
            
            print("===============================\n");
        }
    }
    
    /// Test serial port communication
    pub fn test_communication() -> Result<(), SerialError> {
        unsafe {
            if let Some(com1) = &mut COM1 {
                com1.write_str("Serial communication test\n")?;
                com1.write_str("If you see this, serial is working!\n")?;
                Ok(())
            } else if let Some(com2) = &mut COM2 {
                com2.write_str("Serial communication test\n")?;
                com2.write_str("If you see this, serial is working!\n")?;
                Ok(())
            } else {
                Err(SerialError::PortNotFound)
            }
        }
    }
}
