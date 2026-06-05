//! Polymera OS Zig library root.
//!
//! All public C-ABI exports live in c_compat.zig. Pure-Zig helpers (callable
//! only from Zig) live in their respective modules below.

pub const c_compat = @import("c_compat.zig");
pub const string = @import("string.zig");
pub const mem = @import("mem.zig");

test {
    // Pull in tests from each module.
    _ = string;
    _ = mem;
    _ = c_compat;
}
