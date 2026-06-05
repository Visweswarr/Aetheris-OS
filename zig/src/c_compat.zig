//! C-ABI surface.
//!
//! The functions here are byte-compatible drop-in replacements for a small
//! subset of libc that Polymera-internal C and Rust code depends on. They are
//! safer than their libc equivalents because Zig refuses to compile undefined
//! behaviour. Behavior on invalid input matches libc — including null `dest`
//! handling — only where the C standard *requires* a specific behavior.
//!
//! Initiative 4 of the polyglot roadmap targets gradual replacement of
//! `c/libc_aetheris/*.c` with the equivalents below.

const std = @import("std");
const string = @import("string.zig");
const mem = @import("mem.zig");

/// strlen(3). Returns the number of bytes before the first NUL.
export fn polymera_strlen(s: [*:0]const u8) callconv(.C) usize {
    return string.strlen(s);
}

/// memcpy(3). Returns `dest`.
export fn polymera_memcpy(
    dest: [*]u8,
    src: [*]const u8,
    n: usize,
) callconv(.C) [*]u8 {
    mem.memcpy(dest, src, n);
    return dest;
}

/// memset(3). Returns `dest`.
export fn polymera_memset(
    dest: [*]u8,
    val: c_int,
    n: usize,
) callconv(.C) [*]u8 {
    mem.memset(dest, @as(u8, @intCast(val & 0xff)), n);
    return dest;
}

/// memcmp(3). Returns <0, 0, >0 in the libc convention.
export fn polymera_memcmp(
    a: [*]const u8,
    b: [*]const u8,
    n: usize,
) callconv(.C) c_int {
    return mem.memcmp(a, b, n);
}

/// strcpy(3). Returns `dest`.
export fn polymera_strcpy(dest: [*]u8, src: [*:0]const u8) callconv(.C) [*]u8 {
    string.strcpy(dest, src);
    return dest;
}

/// strncpy(3). Returns `dest`.
export fn polymera_strncpy(dest: [*]u8, src: [*]const u8, n: usize) callconv(.C) [*]u8 {
    string.strncpy(dest, src, n);
    return dest;
}

/// strcmp(3). Returns <0, 0, >0.
export fn polymera_strcmp(a: [*:0]const u8, b: [*:0]const u8) callconv(.C) c_int {
    return string.strcmp(a, b);
}

/// strncmp(3). Returns <0, 0, >0.
export fn polymera_strncmp(a: [*]const u8, b: [*]const u8, n: usize) callconv(.C) c_int {
    return string.strncmp(a, b, n);
}

test "polymera_strlen on known string" {
    const s: [*:0]const u8 = "hello";
    try std.testing.expectEqual(@as(usize, 5), polymera_strlen(s));
}
