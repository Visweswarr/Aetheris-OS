const std = @import("std");

pub fn strlen(s: [*:0]const u8) usize {
    return std.mem.len(s);
}

pub fn strcpy(dest: [*]u8, src: [*:0]const u8) void {
    var i: usize = 0;
    while (src[i] != 0) : (i += 1) {
        dest[i] = src[i];
    }
    dest[i] = 0;
}

pub fn strncpy(dest: [*]u8, src: [*]const u8, n: usize) void {
    var i: usize = 0;
    var null_found = false;
    while (i < n) : (i += 1) {
        if (!null_found) {
            if (src[i] == 0) {
                null_found = true;
                dest[i] = 0;
            } else {
                dest[i] = src[i];
            }
        } else {
            dest[i] = 0;
        }
    }
}

pub fn strcmp(a: [*:0]const u8, b: [*:0]const u8) c_int {
    var i: usize = 0;
    while (true) {
        const char_a = a[i];
        const char_b = b[i];
        if (char_a != char_b) {
            return if (char_a < char_b) -1 else 1;
        }
        if (char_a == 0) {
            return 0;
        }
        i += 1;
    }
}

pub fn strncmp(a: [*]const u8, b: [*]const u8, n: usize) c_int {
    var i: usize = 0;
    while (i < n) : (i += 1) {
        const char_a = a[i];
        const char_b = b[i];
        if (char_a != char_b) {
            return if (char_a < char_b) -1 else 1;
        }
        if (char_a == 0) {
            return 0;
        }
    }
    return 0;
}

test "strlen empty" {
    const s: [*:0]const u8 = "";
    try std.testing.expectEqual(@as(usize, 0), strlen(s));
}

test "strcpy and strcmp" {
    var buf: [32]u8 = undefined;
    const src: [*:0]const u8 = "hello zig";
    strcpy(&buf, src);
    
    const dest_ptr: [*:0]u8 = @ptrCast(&buf);
    try std.testing.expectEqual(@as(usize, 9), strlen(dest_ptr));
    try std.testing.expectEqual(@as(c_int, 0), strcmp(dest_ptr, src));
    try std.testing.expectEqual(@as(c_int, 1), strcmp(dest_ptr, "hello zia"));
    try std.testing.expectEqual(@as(c_int, -1), strcmp(dest_ptr, "hello ziz"));
}

test "strncpy and strncmp" {
    var buf: [32]u8 = undefined;
    const src: [*]const u8 = "hello polyglot";
    strncpy(&buf, src, 5);
    buf[5] = 0;
    
    const dest_ptr: [*:0]u8 = @ptrCast(&buf);
    try std.testing.expectEqual(@as(usize, 5), strlen(dest_ptr));
    try std.testing.expectEqual(@as(c_int, 0), strncmp(dest_ptr, src, 5));
}
