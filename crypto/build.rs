fn main() {
    // Build the stub C library
    cc::Build::new()
        .file("src/liboqs_stub.c")
        .compile("liboqs_stub");
}
