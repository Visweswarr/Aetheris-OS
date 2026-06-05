//! Capability Manager Module for Polymera OS Kernel
//!
//! This module implements seL4-style capability-based security for the microkernel.
//! All process permissions are managed through unforgeable capability handles.
//!
//! Requirements: 7.1, 7.2, 7.3, 7.4

pub mod manager;
pub mod tree;
pub mod cspace;

use alloc::vec::Vec;

// Re-export main types
pub use manager::{CapabilityManager, CAPABILITY_MANAGER};
pub use tree::{CapabilityTree, CapabilityNode};
pub use cspace::{CapabilitySpace, CSpaceError};

/// Process ID type
pub type ProcessId = u64;

/// Capability handle ID type  
pub type CapabilityId = u64;

/// Device ID type
pub type DeviceId = u64;

/// Enclave ID type
pub type EnclaveId = u64;

/// Capability types supported by the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    /// Memory region capability
    Memory { 
        base: u64, 
        size: usize, 
        perms: Permissions 
    },
    /// IPC channel capability
    Ipc { 
        target: ProcessId 
    },
    /// Device access capability
    Device { 
        device_id: DeviceId, 
        ops: DeviceOps 
    },
    /// Framebuffer/DRM capability
    Framebuffer { 
        drm_fd: u32 
    },
    /// Hardware enclave capability
    Enclave { 
        enclave_id: EnclaveId 
    },
    /// IO port access capability
    IoPort {
        port_start: u16,
        port_count: u16,
    },
}

/// Permissions bitmap for memory capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions(pub u8);

impl Permissions {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1);
    pub const WRITE: Self = Self(2);
    pub const EXECUTE: Self = Self(4);
    pub const READ_WRITE: Self = Self(3);
    pub const READ_EXECUTE: Self = Self(5);
    pub const ALL: Self = Self(7);
    
    /// Check if permissions contain the required bits
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
    
    /// Check W^X (Write XOR Execute) property
    /// Returns true if the permissions are valid (not both W and X)
    pub fn is_wxorx(&self) -> bool {
        !((self.0 & 2) != 0 && (self.0 & 4) != 0)
    }
}

/// Device operations bitmap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceOps(pub u8);

impl DeviceOps {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1);
    pub const WRITE: Self = Self(2);
    pub const IOCTL: Self = Self(4);
    pub const DMA: Self = Self(8);
    pub const ALL: Self = Self(0xFF);
    
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Unforgeable capability handle
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityHandle {
    /// Unique handle ID
    pub id: CapabilityId,
    /// The actual capability
    pub capability: Capability,
    /// Parent handle ID (for revocation cascade)
    pub parent: Option<CapabilityId>,
    /// Generation number (invalidated on revoke)
    pub generation: u32,
    /// Owner process ID
    pub owner: ProcessId,
}

impl CapabilityHandle {
    /// Check if this handle is valid (not revoked)
    pub fn is_valid(&self, expected_generation: u32) -> bool {
        self.generation == expected_generation
    }
}

/// IPC handle for inter-process communication
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpcHandle {
    /// Handle ID
    pub id: CapabilityId,
    /// Source process
    pub from: ProcessId,
    /// Target process
    pub to: ProcessId,
    /// Generation for revocation
    pub generation: u32,
}

/// Capability error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapError {
    /// Handle not found
    InvalidHandle { handle: CapabilityId, generation: u32 },
    /// Insufficient permissions
    InsufficientPermissions { required: Permissions, actual: Permissions },
    /// Handle has been revoked
    HandleRevoked { handle: CapabilityId },
    /// W^X violation
    WxViolation { address: u64 },
    /// IPC not permitted
    IpcDenied { from: ProcessId, to: ProcessId },
    /// Process not found
    ProcessNotFound { pid: ProcessId },
    /// Capability space full
    CSpaceFull { pid: ProcessId },
    /// Cannot derive from this capability
    CannotDerive,
    /// Capability already exists
    AlreadyExists,
}

impl core::fmt::Display for CapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidHandle { handle, generation } => 
                write!(f, "Invalid capability handle {} (gen {})", handle, generation),
            Self::InsufficientPermissions { required, actual } => 
                write!(f, "Insufficient permissions: need {:?}, have {:?}", required, actual),
            Self::HandleRevoked { handle } => 
                write!(f, "Capability handle {} has been revoked", handle),
            Self::WxViolation { address } => 
                write!(f, "W^X violation at address {:#x}", address),
            Self::IpcDenied { from, to } => 
                write!(f, "IPC from {} to {} denied", from, to),
            Self::ProcessNotFound { pid } => 
                write!(f, "Process {} not found", pid),
            Self::CSpaceFull { pid } => 
                write!(f, "Capability space for process {} is full", pid),
            Self::CannotDerive => 
                write!(f, "Cannot derive from this capability"),
            Self::AlreadyExists =>
                write!(f, "Capability already exists"),
        }
    }
}

/// Result type for capability operations
pub type CapResult<T> = Result<T, CapError>;

/// Initialize the capability manager subsystem
pub fn init_caps() {
    crate::kprintln!("[CAPS] Initializing capability manager");
    // The manager is lazily initialized via CAPABILITY_MANAGER static
    crate::klog!(INFO, "[CAPS] Capability manager initialized");
}
