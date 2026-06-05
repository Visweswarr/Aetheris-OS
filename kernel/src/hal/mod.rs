#[cfg(target_arch="x86_64")]
pub mod x86_64;
#[cfg(target_arch="aarch64")]
pub mod aarch64;

use core::fmt;

/// HAL error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalError {
    /// Hardware not found
    NotFound,
    /// Specific device with a static name not found
    DeviceNotFound(&'static str),
    /// Hardware initialization failed
    InitFailed,
    /// Initialization failed with a static reason
    InitializationFailed(&'static str),
    /// Invalid configuration
    InvalidConfig,
    /// Invalid parameter with a static reason
    InvalidParameter(&'static str),
    /// Operation not supported
    NotSupported,
    /// Hardware busy
    Busy,
    /// Timeout
    Timeout,
    /// Calibration failed with a static reason
    CalibrationFailed(&'static str),
    /// Lock acquisition failed
    LockError(&'static str),
    /// Generic error
    Generic,
}

impl fmt::Display for HalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HalError::NotFound => write!(f, "Hardware not found"),
            HalError::DeviceNotFound(name) => write!(f, "Device not found: {}", name),
            HalError::InitFailed => write!(f, "Hardware initialization failed"),
            HalError::InitializationFailed(reason) => write!(f, "Initialization failed: {}", reason),
            HalError::InvalidConfig => write!(f, "Invalid configuration"),
            HalError::InvalidParameter(reason) => write!(f, "Invalid parameter: {}", reason),
            HalError::NotSupported => write!(f, "Operation not supported"),
            HalError::Busy => write!(f, "Hardware busy"),
            HalError::Timeout => write!(f, "Timeout"),
            HalError::CalibrationFailed(reason) => write!(f, "Calibration failed: {}", reason),
            HalError::LockError(reason) => write!(f, "Lock error: {}", reason),
            HalError::Generic => write!(f, "Generic HAL error"),
        }
    }
}

/// HAL result type
pub type HalResult<T> = Result<T, HalError>;

pub trait Hal {
    fn init_cpu() -> Result<(), &'static str>;
    fn init_timer() -> Result<(), &'static str>;
    fn enable_interrupts() -> Result<(), &'static str>;
}
