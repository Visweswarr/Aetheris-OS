use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=wrapper.c");

    // Check if we should use system liboqs
    let use_system_liboqs = env::var("LIBOQS_USE_SYSTEM").unwrap_or_default() == "1";
    
    if use_system_liboqs {
        build_with_system_liboqs();
    } else {
        build_with_bundled_liboqs();
    }

    // Generate bindings
    generate_bindings();
}

fn build_with_system_liboqs() {
    println!("cargo:warning=Using system liboqs");
    
    // Try to find system liboqs
    let pkg_config = Command::new("pkg-config")
        .args(&["--exists", "liboqs"])
        .output();
    
    if pkg_config.is_ok() {
        // Use pkg-config to get flags
        let cflags = Command::new("pkg-config")
            .args(&["--cflags", "liboqs"])
            .output()
            .expect("Failed to get liboqs cflags");
        
        let libs = Command::new("pkg-config")
            .args(&["--libs", "liboqs"])
            .output()
            .expect("Failed to get liboqs libs");
        
        let cflags = String::from_utf8_lossy(&cflags.stdout).trim().to_string();
        let libs = String::from_utf8_lossy(&libs.stdout).trim().to_string();
        
        // Add system liboqs
        for flag in cflags.split_whitespace() {
            if flag.starts_with("-I") {
                println!("cargo:include={}", &flag[2..]);
            }
        }
        
        for flag in libs.split_whitespace() {
            if flag.starts_with("-L") {
                println!("cargo:rustc-link-search=native={}", &flag[2..]);
            } else if flag.starts_with("-l") {
                println!("cargo:rustc-link-lib={}", &flag[2..]);
            }
        }
        
        println!("cargo:rustc-link-lib=oqs");
        println!("cargo:rustc-link-lib=crypto");
    } else {
        panic!("System liboqs not found. Install liboqs or set LIBOQS_USE_SYSTEM=0");
    }
}

fn build_with_bundled_liboqs() {
    println!("cargo:warning=Building bundled liboqs");
    
    // Clone and build liboqs
    let liboqs_dir = PathBuf::from("liboqs");
    
    if !liboqs_dir.exists() {
        clone_liboqs();
    }
    
    // Build liboqs
    build_liboqs(&liboqs_dir);
    
    // Add bundled liboqs
    println!("cargo:rustc-link-search=native={}/build/lib", liboqs_dir.display());
    println!("cargo:rustc-link-lib=static=oqs");
    println!("cargo:rustc-link-lib=crypto");
    
    // Add include paths
    println!("cargo:include={}/include", liboqs_dir.display());
    println!("cargo:include={}/src", liboqs_dir.display());
}

fn clone_liboqs() {
    println!("cargo:warning=Cloning liboqs repository...");
    
    let status = Command::new("git")
        .args(&["clone", "https://github.com/open-quantum-safe/liboqs.git"])
        .status()
        .expect("Failed to clone liboqs");
    
    if !status.success() {
        panic!("Failed to clone liboqs repository");
    }
    
    // Checkout pinned commit for deterministic builds
    let liboqs_dir = PathBuf::from("liboqs");
    let checkout_status = Command::new("git")
        .args(&["checkout", "0.8.0"]) // Pinned to stable release
        .current_dir(&liboqs_dir)
        .status()
        .expect("Failed to checkout pinned commit");
    
    if !checkout_status.success() {
        panic!("Failed to checkout pinned liboqs commit");
    }
    
    println!("cargo:warning=Checked out liboqs commit: 0.8.0");
}

fn build_liboqs(liboqs_dir: &PathBuf) {
    println!("cargo:warning=Building liboqs...");
    
    // Create build directory
    let build_dir = liboqs_dir.join("build");
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");
    
    // Configure with CMake
    let mut cmake = Command::new("cmake");
    cmake.args(&[
        "-DCMAKE_BUILD_TYPE=Release",
        "-DBUILD_SHARED_LIBS=OFF",
        "-DOQS_BUILD_ONLY_LIB=ON",
        "-DOQS_USE_OPENSSL=ON",
        "-DOQS_ENABLE_KEM_KYBER=ON",
        "-DOQS_ENABLE_SIG_DILITHIUM=ON",
        "-DOQS_ENABLE_SIG_FALCON=ON",
        "-DOQS_ENABLE_SIG_SPHINCS=ON",
        ".."
    ]);
    cmake.current_dir(&build_dir);
    
    let status = cmake.status().expect("Failed to run cmake");
    if !status.success() {
        panic!("CMake configuration failed");
    }
    
    // Build
    let mut make = Command::new("cmake");
    make.args(&["--build", ".", "--config", "Release"]);
    make.current_dir(&build_dir);
    
    let status = make.status().expect("Failed to run make");
    if !status.success() {
        panic!("Build failed");
    }
}

fn generate_bindings() {
    // Generate bindings for liboqs
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings");
    
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings");
    
    // Compile wrapper
    let mut cc = cc::Build::new();
    cc.file("wrapper.c")
        .include("liboqs/include")
        .include("liboqs/src")
        .flag_if_supported("-std=c99")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .flag_if_supported("-Werror")
        .compile("liboqs_wrapper");
    
    // Add wrapper to bindings
    println!("cargo:rustc-link-lib=static=liboqs_wrapper");
}
