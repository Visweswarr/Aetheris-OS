//! Scheme Trait Definition
//!
//! Based on Redox OS Schemes.
//! Provides a unified interface for all system resources (Reference OS Pattern).

use alloc::vec::Vec;
use alloc::boxed::Box;

/// Result type for Scheme operations
pub type Result<T> = core::result::Result<T, SchemeError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemeError {
    NotFound,
    PermissionDenied,
    InvalidHandle,
    IOError,
    NotImplemented,
}

/// Scheme ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SchemeId(pub usize);

/// File Handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle(pub usize);

/// The Scheme Trait
/// All system services and resources must implement this.
pub trait Scheme: Send + Sync {
    /// Open a resource (file or service)
    fn open(&self, path: &str, flags: usize, uid: u32, gid: u32) -> Result<Handle>;

    /// Read from a resource
    fn read(&self, handle: Handle, buffer: &mut [u8]) -> Result<usize>;

    /// Write to a resource
    fn write(&self, handle: Handle, buffer: &[u8]) -> Result<usize>;

    /// Close a resource
    fn close(&self, handle: Handle) -> Result<()>;
    
    /// Memory map a resource (optional)
    fn fmap(&self, handle: Handle, offset: usize, size: usize) -> Result<usize> {
        Err(SchemeError::NotImplemented)
    }
}

pub mod registry;
pub use registry::register_scheme;
