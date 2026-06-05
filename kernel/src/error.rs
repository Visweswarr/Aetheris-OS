//! Kernel error types and result aliases

use core::fmt;

/// Kernel error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelError {
    /// Generic error
    Generic,
    /// Memory allocation error
    MemoryError,
    /// Invalid argument
    InvalidArgument,
    /// Resource not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// Resource busy
    Busy,
    /// Operation timed out
    Timeout,
    /// I/O error
    IoError,
    /// Invalid state
    InvalidState,
    /// Not supported
    NotSupported,
    /// Already exists
    AlreadyExists,
    /// Out of bounds
    OutOfBounds,
    /// Initialization error
    InitError,
    /// Hardware error
    HardwareError,
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::Generic => write!(f, "Generic error"),
            KernelError::MemoryError => write!(f, "Memory error"),
            KernelError::InvalidArgument => write!(f, "Invalid argument"),
            KernelError::NotFound => write!(f, "Not found"),
            KernelError::PermissionDenied => write!(f, "Permission denied"),
            KernelError::Busy => write!(f, "Resource busy"),
            KernelError::Timeout => write!(f, "Operation timed out"),
            KernelError::IoError => write!(f, "I/O error"),
            KernelError::InvalidState => write!(f, "Invalid state"),
            KernelError::NotSupported => write!(f, "Not supported"),
            KernelError::AlreadyExists => write!(f, "Already exists"),
            KernelError::OutOfBounds => write!(f, "Out of bounds"),
            KernelError::InitError => write!(f, "Initialization error"),
            KernelError::HardwareError => write!(f, "Hardware error"),
        }
    }
}

/// Kernel result type alias
pub type KernelResult<T> = Result<T, KernelError>;

/// Kernel configuration
#[derive(Debug, Clone)]
pub struct KernelConfig {
    /// Enable debug mode
    pub debug: bool,
    /// Enable verbose logging
    pub verbose: bool,
    /// Maximum memory in bytes
    pub max_memory: u64,
    /// Number of CPUs
    pub num_cpus: u32,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            debug: cfg!(debug_assertions),
            verbose: false,
            max_memory: 1024 * 1024 * 1024, // 1GB default
            num_cpus: 1,
        }
    }
}

impl KernelConfig {
    pub fn new() -> Self {
        Self::default()
    }
}
