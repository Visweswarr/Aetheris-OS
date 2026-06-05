# Polymera OS — Zig migration root

This directory is the home of Initiative 4 of the polyglot roadmap: replacing
new C code (and gradually `c/libc_aetheris/*`) with Zig while preserving C-ABI
compatibility.

## Status

**Phase 1 — scaffold.** A handful of C-ABI exports (`strlen`, `memcpy`, `memset`,
`memcmp`) are implemented to prove the build path works. No `c/libc_aetheris/` code
has been retired yet.

## Why Zig (recap of `LANGUAGES.md` § 3.1)

- Same C ABI → swap a `.c` for a `.zig` without changing any callers.
- Built-in cross-compilation (no sysroot hunting): one toolchain ships every libc.
- No undefined behaviour by default. Comptime catches errors at compile time.
- Existing C code can keep compiling unchanged via `zig cc`.

## Build

```bash
zig build
zig build test
zig build -Dtarget=aarch64-linux-musl
zig build -Dtarget=x86_64-windows-gnu
```

## Linking from Rust

The output `libpolymera_zig.a` is a regular static archive. Consume it from a Rust
crate's `build.rs`:

```rust
println!("cargo:rustc-link-search=native=../../zig/zig-out/lib");
println!("cargo:rustc-link-lib=static=polymera_zig");
```

The Rust side declares prototypes via `extern "C"`. The `LANGUAGES.md` § 2.1 FFI
rule allows this because the Zig library is the same C-ABI floor that we are
incrementally replacing C with.

## Replacing existing C code (migration pattern)

1. Pick a `c/libc_aetheris/<name>.c` function.
2. Implement the same C-ABI signature in `zig/src/<name>.zig` and export it in
   `c_compat.zig`.
3. Add a parity test: same inputs → same outputs across both implementations.
4. Update consumer build scripts to prefer the Zig library.
5. After two release cycles with no regressions, remove the C file. Record in
   `DEPRECATIONS.md`.

## Using `zig cc` as the project C compiler

This works even before any code is rewritten — it gives you reproducible
cross-compilation immediately:

```bash
export CC="zig cc"
export CXX="zig c++"
make ARCH=aarch64 CC="zig cc -target aarch64-linux-musl"
```
