//! Capability flags and constants

/// Read permission flag
pub const CAP_READ: u32 = 0x0001;
/// Write permission flag
pub const CAP_WRITE: u32 = 0x0002;
/// Execute permission flag
pub const CAP_EXECUTE: u32 = 0x0004;
/// Admin permission flag
pub const CAP_ADMIN: u32 = 0x0008;
/// Create permission flag
pub const CAP_CREATE: u32 = 0x0010;
/// Delete permission flag
pub const CAP_DELETE: u32 = 0x0020;
/// Subscribe permission flag
pub const CAP_SUBSCRIBE: u32 = 0x0040;
/// Publish permission flag
pub const CAP_PUBLISH: u32 = 0x0080;

/// All permissions
pub const CAP_ALL: u32 = CAP_READ | CAP_WRITE | CAP_EXECUTE | CAP_ADMIN | CAP_CREATE | CAP_DELETE | CAP_SUBSCRIBE | CAP_PUBLISH;

// Domain-specific capability bits used by the event fabric & topic table.
pub const CAP_INTENT_SUBMIT: u64 = 1 << 32;
pub const CAP_INTENT_QUERY: u64 = 1 << 33;
pub const CAP_WM_READ: u64 = 1 << 34;
pub const CAP_WM_WRITE: u64 = 1 << 35;
pub const CAP_SKILL_LOAD: u64 = 1 << 36;
pub const CAP_SKILL_INVOKE: u64 = 1 << 37;
pub const CAP_SKILL_QUERY: u64 = 1 << 38;

/// Check if a capability has a specific permission
pub fn has_permission(cap_flags: u32, permission: u32) -> bool {
    (cap_flags & permission) == permission
}
