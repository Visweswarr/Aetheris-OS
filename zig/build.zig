// Polymera OS — Zig build script.
//
// This is the entrypoint for Initiative 4 of the polyglot roadmap (replacing
// `c/libc_aetheris/` piece by piece while preserving full C-ABI compatibility).
//
// Build with:
//     zig build                     # default — produce libpolymera_zig.a
//     zig build -Doptimize=ReleaseSafe
//     zig build test                # run the test suite
//
// Cross-compilation works without sysroot setup:
//     zig build -Dtarget=aarch64-linux-musl
//     zig build -Dtarget=x86_64-windows-gnu
//     zig build -Dtarget=riscv64-linux-musl
//
// The output library exports a C ABI (see src/c_compat.zig) so Rust crates can
// link against it via cc::Build or build.rs as a drop-in libc-style replacement.

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const lib = b.addStaticLibrary(.{
        .name = "polymera_zig",
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    // Force C-ABI exports so the symbols are usable from Rust / C / C++.
    lib.linkLibC();
    b.installArtifact(lib);

    const unit_tests = b.addTest(.{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    const run_unit_tests = b.addRunArtifact(unit_tests);

    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_unit_tests.step);
}
