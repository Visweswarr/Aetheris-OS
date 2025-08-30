use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the specified search directory
    println!("cargo:rustc-link-search=native={}", env::var("OUT_DIR").unwrap());
    
    // Tell cargo to tell rustc to link our polycrypto library
    println!("cargo:rustc-link-lib=static=polycrypto");
    
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/lib.rs");
    
    // Build the C library
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Path to the C source files
    let c_src_dir = manifest_dir.join("../../c/crypto");
    
    // Compile the C library
    let mut build = cc::Build::new();
    
    build
        .file(c_src_dir.join("xchacha.c"))
        .file(c_src_dir.join("memzero.c"))
        .include(c_src_dir)
        .flag("-O2")
        .flag("-fno-omit-frame-pointer")
        .flag("-fstack-protector-strong")
        .flag("-fPIC")
        .flag("-Werror")
        .define("AETH_CT", "1")
        .compile("polycrypto");
    
    // Link with libsodium
    println!("cargo:rustc-link-lib=sodium");
    
    // Re-run if C source files change
    println!("cargo:rerun-if-changed={}", c_src_dir.join("polycrypto.h").display());
    println!("cargo:rerun-if-changed={}", c_src_dir.join("xchacha.c").display());
    println!("cargo:rerun-if-changed={}", c_src_dir.join("memzero.c").display());
}
