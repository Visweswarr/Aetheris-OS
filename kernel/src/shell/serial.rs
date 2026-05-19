//! Serial Interface for Shell
//! 
//! This module provides serial input/output functionality for the shell,
//! including line buffering, command history, and input validation.

use crate::{kprintln, klog, kprint};
use crate::log::Level;
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// Serial interface configuration
pub const MAX_LINE_LENGTH: usize = 256;
pub const MAX_HISTORY_SIZE: usize = 50;
pub const PROMPT: &str = "polymera> ";

/// Serial input buffer
pub struct SerialBuffer {
    buffer: Vec<u8>,
    cursor: usize,
    history: Vec<String>,
    history_index: usize,
    echo_enabled: bool,
}

impl SerialBuffer {
    /// Create a new serial buffer
    pub fn new() -> Self {
        SerialBuffer {
            buffer: Vec::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: 0,
            echo_enabled: true,
        }
    }
    
    /// Add a character to the buffer
    pub fn add_char(&mut self, c: u8) -> Result<(), String> {
        match c {
            // Backspace
            0x08 | 0x7F => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.buffer.remove(self.cursor);
                    
                    if self.echo_enabled {
                        // Send backspace sequence
                        kprintln!("\x08 \x08"); // Backspace, space, backspace
                    }
                }
                Ok(())
            }
            
            // Carriage return (enter)
            0x0D => {
                if self.echo_enabled {
                    kprintln!(); // New line
                }
                Ok(())
            }
            
            // Line feed
            0x0A => {
                // Ignore line feed (handled by carriage return)
                Ok(())
            }
            
            // Tab (expand to spaces)
            0x09 => {
                for _ in 0..4 {
                    if self.buffer.len() < MAX_LINE_LENGTH {
                        self.buffer.insert(self.cursor, b' ');
                        self.cursor += 1;
                        
                        if self.echo_enabled {
                            kprint!(" ");
                        }
                    }
                }
                Ok(())
            }
            
            // Escape sequences
            0x1B => {
                // Start of escape sequence - ignore for now
                Ok(())
            }
            
            // Printable characters
            0x20..=0x7E => {
                if self.buffer.len() < MAX_LINE_LENGTH {
                    self.buffer.insert(self.cursor, c);
                    self.cursor += 1;
                    
                    if self.echo_enabled {
                        kprint!("{}", c as char);
                    }
                    Ok(())
                } else {
                    Err("Line too long".to_string())
                }
            }
            
            // Control characters
            _ => {
                // Ignore other control characters
                Ok(())
            }
        }
    }
    
    /// Get the current line as a string
    pub fn get_line(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }
    
    /// Clear the current line
    pub fn clear_line(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }
    
    /// Add line to history
    pub fn add_to_history(&mut self, line: String) {
        if !line.trim().is_empty() {
            // Remove duplicate consecutive entries
            if self.history.last() != Some(&line) {
                self.history.push(line);
                
                // Limit history size
                if self.history.len() > MAX_HISTORY_SIZE {
                    self.history.remove(0);
                }
            }
            
            self.history_index = self.history.len();
        }
    }
    
    /// Get previous history entry
    pub fn get_previous_history(&mut self) -> Option<String> {
        if self.history_index > 0 {
            self.history_index -= 1;
            Some(self.history[self.history_index].clone())
        } else {
            None
        }
    }
    
    /// Get next history entry
    pub fn get_next_history(&mut self) -> Option<String> {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            Some(self.history[self.history_index].clone())
        } else if self.history_index < self.history.len() {
            self.history_index += 1;
            Some(String::new()) // Current line
        } else {
            None
        }
    }
    
    /// Set history to current line
    pub fn set_history_to_current(&mut self) {
        self.history_index = self.history.len();
    }
    
    /// Toggle echo mode
    pub fn toggle_echo(&mut self) {
        self.echo_enabled = !self.echo_enabled;
    }
    
    /// Check if echo is enabled
    pub fn is_echo_enabled(&self) -> bool {
        self.echo_enabled
    }
    
    /// Get buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Serial interface for the shell
pub struct SerialInterface {
    buffer: RefCell<SerialBuffer>,
    input_ready: AtomicBool,
}

impl SerialInterface {
    /// Create a new serial interface
    pub fn new() -> Self {
        SerialInterface {
            buffer: RefCell::new(SerialBuffer::new()),
            input_ready: AtomicBool::new(false),
        }
    }
    
    /// Initialize the serial interface
    pub fn init(&self) {
        kprintln!("[SERIAL] Initializing serial interface for shell");
        
        // Show initial prompt
        self.show_prompt();
        
        kprintln!("[SERIAL] Serial interface initialized");
    }
    
    /// Show the command prompt
    pub fn show_prompt(&self) {
        kprint!("{}", PROMPT);
    }
    
    /// Process incoming serial data
    pub fn process_input(&self, data: &[u8]) -> Option<String> {
        let mut buffer = self.buffer.borrow_mut();
        
        for &byte in data {
            match byte {
                // Enter key - process line
                0x0D => {
                    let line = buffer.get_line();
                    buffer.add_to_history(line.clone());
                    
                    if !line.trim().is_empty() {
                        buffer.clear_line();
                        return Some(line);
                    } else {
                        buffer.clear_line();
                        self.show_prompt();
                        return None;
                    }
                }
                
                // Other characters
                _ => {
                    if let Err(e) = buffer.add_char(byte) {
                        klog!(WARN, "[SERIAL] Input error: {}", e);
                    }
                }
            }
        }
        
        None
    }
    
    /// Check if input is ready
    pub fn is_input_ready(&self) -> bool {
        self.input_ready.load(Ordering::Relaxed)
    }
    
    /// Set input ready flag
    pub fn set_input_ready(&self, ready: bool) {
        self.input_ready.store(ready, Ordering::Relaxed);
    }
    
    /// Get current buffer contents
    pub fn get_current_buffer(&self) -> String {
        self.buffer.borrow().get_line()
    }
    
    /// Clear the current buffer
    pub fn clear_buffer(&self) {
        self.buffer.borrow_mut().clear_line();
    }
    
    /// Handle special keys
    pub fn handle_special_key(&self, key: u8) -> bool {
        let mut buffer = self.buffer.borrow_mut();
        
        match key {
            // Ctrl+C - clear line
            0x03 => {
                buffer.clear_line();
                kprintln!("^C");
                self.show_prompt();
                true
            }
            
            // Ctrl+L - clear screen
            0x0C => {
                kprintln!("\x1B[2J\x1B[H"); // ANSI clear screen + home
                self.show_prompt();
                true
            }
            
            // Ctrl+U - clear line before cursor
            0x15 => {
                let before_cursor = buffer.cursor;
                for _ in 0..before_cursor {
                    if let Some(_) = buffer.buffer.first() {
                        buffer.buffer.remove(0);
                        buffer.cursor -= 1;
                        kprint!("\x08 \x08"); // Backspace, space, backspace
                    }
                }
                true
            }
            
            // Ctrl+K - clear line after cursor
            0x0B => {
                let after_cursor = buffer.buffer.len() - buffer.cursor;
                for _ in 0..after_cursor {
                    if buffer.buffer.len() > buffer.cursor {
                        buffer.buffer.remove(buffer.cursor);
                        kprint!(" ");
                    }
                }
                true
            }
            
            // Ctrl+W - clear word before cursor
            0x17 => {
                // Find word boundary
                let mut word_start = buffer.cursor;
                while word_start > 0 && buffer.buffer[word_start - 1] == b' ' {
                    word_start -= 1;
                }
                while word_start > 0 && buffer.buffer[word_start - 1] != b' ' {
                    word_start -= 1;
                }
                
                let chars_to_remove = buffer.cursor - word_start;
                for _ in 0..chars_to_remove {
                    if buffer.cursor > 0 {
                        buffer.buffer.remove(word_start);
                        buffer.cursor -= 1;
                        kprint!("\x08 \x08"); // Backspace, space, backspace
                    }
                }
                true
            }
            
            _ => false
        }
    }
    
    /// Output a string to serial
    pub fn output(&self, text: &str) {
        kprint!("{}", text);
    }
    
    /// Output a line to serial
    pub fn output_line(&self, text: &str) {
        kprintln!("{}", text);
    }
    
    /// Output error message
    pub fn output_error(&self, text: &str) {
        klog!(ERROR, "[SHELL] {}", text);
    }
    
    /// Output success message
    pub fn output_success(&self, text: &str) {
        klog!(INFO, "[SHELL] {}", text);
    }
}

/// Global serial interface instance
static mut SERIAL_INTERFACE: Option<SerialInterface> = None;

/// Initialize the global serial interface
pub fn init_serial_interface() {
    unsafe {
        SERIAL_INTERFACE = Some(SerialInterface::new());
        if let Some(ref interface) = SERIAL_INTERFACE {
            interface.init();
        }
    }
}

/// Get a reference to the global serial interface
pub fn get_serial_interface() -> Option<&'static SerialInterface> {
    unsafe {
        SERIAL_INTERFACE.as_ref()
    }
}

/// Process serial input data
pub fn process_serial_input(data: &[u8]) -> Option<String> {
    if let Some(interface) = get_serial_interface() {
        interface.process_input(data)
    } else {
        None
    }
}

/// Output text to serial
pub fn serial_output(text: &str) {
    if let Some(interface) = get_serial_interface() {
        interface.output(text);
    }
}

/// Output line to serial
pub fn serial_output_line(text: &str) {
    if let Some(interface) = get_serial_interface() {
        interface.output_line(text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serial_buffer_creation() {
        let buffer = SerialBuffer::new();
        assert!(buffer.buffer.is_empty());
        assert_eq!(buffer.cursor, 0);
        assert!(buffer.history.is_empty());
    }
    
    #[test]
    fn test_serial_buffer_add_char() {
        let mut buffer = SerialBuffer::new();
        
        // Add printable character
        assert!(buffer.add_char(b'h').is_ok());
        assert_eq!(buffer.get_line(), "h");
        assert_eq!(buffer.cursor, 1);
        
        // Add another character
        assert!(buffer.add_char(b'i').is_ok());
        assert_eq!(buffer.get_line(), "hi");
        assert_eq!(buffer.cursor, 2);
    }
    
    #[test]
    fn test_serial_buffer_backspace() {
        let mut buffer = SerialBuffer::new();
        
        // Add some characters
        buffer.add_char(b'h').unwrap();
        buffer.add_char(b'i').unwrap();
        assert_eq!(buffer.get_line(), "hi");
        
        // Backspace
        assert!(buffer.add_char(0x08).is_ok());
        assert_eq!(buffer.get_line(), "h");
        assert_eq!(buffer.cursor, 1);
    }
    
    #[test]
    fn test_serial_buffer_history() {
        let mut buffer = SerialBuffer::new();
        
        // Add lines to history
        buffer.add_to_history("command1".to_string());
        buffer.add_to_history("command2".to_string());
        
        assert_eq!(buffer.history.len(), 2);
        assert_eq!(buffer.history[0], "command1");
        assert_eq!(buffer.history[1], "command2");
    }
    
    #[test]
    fn test_serial_buffer_max_length() {
        let mut buffer = SerialBuffer::new();
        
        // Fill buffer to max length
        for _ in 0..MAX_LINE_LENGTH {
            assert!(buffer.add_char(b'a').is_ok());
        }
        
        // Try to add one more character
        assert!(buffer.add_char(b'b').is_err());
        assert_eq!(buffer.len(), MAX_LINE_LENGTH);
    }
    
    #[test]
    fn test_serial_interface_creation() {
        let interface = SerialInterface::new();
        assert!(!interface.is_input_ready());
        assert!(interface.get_current_buffer().is_empty());
    }
}

