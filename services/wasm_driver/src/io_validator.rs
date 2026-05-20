//! IO Port Validator
//!
//! Validates and restricts IO port access for WASM drivers.
//! Requirement: 12.6 - IO port validation

use std::collections::HashSet;

/// IO Port range
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IoPortRange {
    pub start: u16,
    pub end: u16,
}

impl IoPortRange {
    pub fn new(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, port: u16) -> bool {
        port >= self.start && port <= self.end
    }
}

/// IO Port Validator
pub struct IoValidator {
    allowed_ports: HashSet<u16>,
    allowed_ranges: Vec<IoPortRange>,
    denied_ports: HashSet<u16>,
}

impl IoValidator {
    pub fn new() -> Self {
        Self {
            allowed_ports: HashSet::new(),
            allowed_ranges: Vec::new(),
            denied_ports: HashSet::new(),
        }
    }

    /// Allow a specific port
    pub fn allow_port(&mut self, port: u16) {
        self.allowed_ports.insert(port);
        self.denied_ports.remove(&port);
    }

    /// Allow a range of ports
    pub fn allow_range(&mut self, start: u16, end: u16) {
        self.allowed_ranges.push(IoPortRange::new(start, end));
    }

    /// Deny a specific port
    pub fn deny_port(&mut self, port: u16) {
        self.denied_ports.insert(port);
        self.allowed_ports.remove(&port);
    }

    /// Check if port access is allowed
    pub fn check_port(&self, port: u16) -> Result<(), IoError> {
        // Check denied list first
        if self.denied_ports.contains(&port) {
            return Err(IoError::PortDenied(port));
        }

        // Check allowed list
        if self.allowed_ports.contains(&port) {
            return Ok(());
        }

        // Check allowed ranges
        for range in &self.allowed_ranges {
            if range.contains(port) {
                return Ok(());
            }
        }

        // Default deny
        Err(IoError::PortNotAllowed(port))
    }

    /// Validate a read operation
    pub fn validate_read(&self, port: u16) -> Result<(), IoError> {
        self.check_port(port)
    }

    /// Validate a write operation
    pub fn validate_write(&self, port: u16) -> Result<(), IoError> {
        self.check_port(port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoError {
    PortDenied(u16),
    PortNotAllowed(u16),
}
