const std = @import("std");

pub fn memcpy(dest: [*]u8, src: [*]const u8, n: usize) void {
    @memcpy(dest[0..n], src[0..n]);
}

pub fn memset(dest: [*]u8, val: u8, n: usize) void {
    @memset(dest[0..n], val);
}

pub fn memcmp(a: [*]const u8, b: [*]const u8, n: usize) c_int {
    var i: usize = 0;
    while (i < n) : (i += 1) {
        if (a[i] < b[i]) return -1;
        if (a[i] > b[i]) return 1;
    }
    return 0;
}

test "memcpy basic" {
    var buf = [_]u8{0} ** 8;
    const src = "abcdefgh";
    memcpy(&buf, src.ptr, 8);
    try std.testing.expectEqualSlices(u8, src[0..8], &buf);
}

test "memcmp ordering" {
    const a = "abc";
    const b = "abd";
    try std.testing.expect(memcmp(a.ptr, b.ptr, 3) < 0);
    try std.testing.expect(memcmp(b.ptr, a.ptr, 3) > 0);
    try std.testing.expect(memcmp(a.ptr, a.ptr, 3) == 0);
}
