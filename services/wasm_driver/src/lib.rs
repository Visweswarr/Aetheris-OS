//! WASM Driver Host Service
//!
//! This service provides a sandboxed execution environment for WASM-based drivers.
//! Implements Task 12: WASM Driver Host (Ring 3)
//!
//! Features:
//! - Driver discovery and loading (12.2)
//! - Sandbox with 64MB memory limit (12.4)
//! - IO port validation (12.6)
//! - Crash isolation (12.8)

pub mod discovery;
pub mod host;
pub mod io_validator;
pub mod sandbox;
pub mod summarizer;

use std::collections::HashMap;

/// Maximum memory for WASM driver sandbox (64MB)
pub const SANDBOX_MEMORY_LIMIT: usize = 64 * 1024 * 1024;

/// Driver ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DriverId(pub u64);

/// Driver metadata
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub id: DriverId,
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub device_class: DeviceClass,
}

/// Device classes supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Block,
    Network,
    Display,
    Input,
    Audio,
    Custom,
}

/// Driver state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverState {
    Loaded,
    Running,
    Suspended,
    Crashed,
    Unloaded,
}

/// WASM Driver instance
pub struct WasmDriver {
    pub info: DriverInfo,
    pub state: DriverState,
    pub memory_usage: usize,
    // In production: WASM module instance from wasmtime/wasmer
}

impl WasmDriver {
    pub fn new(info: DriverInfo) -> Self {
        Self {
            info,
            state: DriverState::Loaded,
            memory_usage: 0,
        }
    }
}

/// WASM Driver Host Manager
pub struct DriverHost {
    drivers: HashMap<DriverId, WasmDriver>,
    next_id: u64,
}

impl DriverHost {
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
            next_id: 1,
        }
    }

    /// Load a driver from WASM bytecode
    pub fn load_driver(
        &mut self,
        name: String,
        wasm_bytes: &[u8],
    ) -> Result<DriverId, DriverError> {
        // Validate WASM magic number
        if wasm_bytes.len() < 4 || &wasm_bytes[0..4] != b"\0asm" {
            return Err(DriverError::InvalidWasm);
        }

        let id = DriverId(self.next_id);
        self.next_id += 1;

        let info = DriverInfo {
            id,
            name,
            version: "1.0.0".to_string(),
            vendor: "Unknown".to_string(),
            device_class: DeviceClass::Custom,
        };

        let driver = WasmDriver::new(info);
        self.drivers.insert(id, driver);

        println!(
            "[WASM_HOST] Loaded driver {} with ID {}",
            self.drivers.get(&id).unwrap().info.name,
            id.0
        );

        Ok(id)
    }

    /// Start a driver
    pub fn start_driver(&mut self, id: DriverId) -> Result<(), DriverError> {
        let driver = self.drivers.get_mut(&id).ok_or(DriverError::NotFound)?;

        if driver.state != DriverState::Loaded && driver.state != DriverState::Suspended {
            return Err(DriverError::InvalidState);
        }

        driver.state = DriverState::Running;
        println!("[WASM_HOST] Started driver {}", id.0);

        Ok(())
    }

    /// Stop a driver
    pub fn stop_driver(&mut self, id: DriverId) -> Result<(), DriverError> {
        let driver = self.drivers.get_mut(&id).ok_or(DriverError::NotFound)?;
        driver.state = DriverState::Suspended;
        println!("[WASM_HOST] Stopped driver {}", id.0);
        Ok(())
    }

    /// Handle driver crash (Crash Isolation - 12.8)
    pub fn handle_crash(&mut self, id: DriverId) {
        if let Some(driver) = self.drivers.get_mut(&id) {
            driver.state = DriverState::Crashed;
            println!("[WASM_HOST] Driver {} crashed, isolated", id.0);
            // In production: cleanup resources, notify subscribers, attempt restart
        }
    }

    /// Unload a driver
    pub fn unload_driver(&mut self, id: DriverId) -> Result<(), DriverError> {
        self.drivers.remove(&id).ok_or(DriverError::NotFound)?;
        println!("[WASM_HOST] Unloaded driver {}", id.0);
        Ok(())
    }

    /// Get driver info
    pub fn get_driver_info(&self, id: DriverId) -> Option<&DriverInfo> {
        self.drivers.get(&id).map(|d| &d.info)
    }

    /// List all drivers
    pub fn list_drivers(&self) -> Vec<&DriverInfo> {
        self.drivers.values().map(|d| &d.info).collect()
    }
}

/// Driver errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverError {
    InvalidWasm,
    NotFound,
    InvalidState,
    MemoryLimitExceeded,
    IoPortDenied,
    LoadFailed,
}

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverError::InvalidWasm => write!(f, "Invalid WASM bytecode"),
            DriverError::NotFound => write!(f, "Driver not found"),
            DriverError::InvalidState => write!(f, "Invalid driver state"),
            DriverError::MemoryLimitExceeded => write!(f, "Memory limit exceeded"),
            DriverError::IoPortDenied => write!(f, "IO port access denied"),
            DriverError::LoadFailed => write!(f, "Driver load failed"),
        }
    }
}

impl std::error::Error for DriverError {}
