//! Build script for Polymera OS kernel
//!
//! Configures the kernel build for different target architectures and platforms.

use std::env;
use std::path::PathBuf;

fn main() {
    // Get the target architecture
    let target = env::var("TARGET").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=arch/");

    // Architecture-specific configuration
    match target_arch.as_str() {
        "x86_64" => {
            println!("cargo:rustc-cfg=arch_x86_64");
            configure_x86_64();
        }
        "aarch64" => {
            println!("cargo:rustc-cfg=arch_aarch64");
            configure_aarch64();
        }
        arch => {
            panic!("Unsupported target architecture: {}", arch);
        }
    }

    // Platform-specific configuration
    match target_os.as_str() {
        "uefi" => {
            println!("cargo:rustc-cfg=platform_uefi");
            configure_uefi();
        }
        "none" => {
            println!("cargo:rustc-cfg=platform_bare_metal");
            configure_bare_metal();
        }
        os => {
            println!("cargo:warning=Building kernel for OS: {}", os);
        }
    }

    // Feature-based configuration
    if env::var("CARGO_FEATURE_SERIAL_DEBUG").is_ok() {
        println!("cargo:rustc-cfg=feature_serial_debug");
    }

    if env::var("CARGO_FEATURE_QEMU_DEBUG").is_ok() {
        println!("cargo:rustc-cfg=feature_qemu_debug");
    }

    if env::var("CARGO_FEATURE_DETERMINISTIC").is_ok() {
        println!("cargo:rustc-cfg=feature_deterministic");
        println!("cargo:warning=Building kernel in DETERMINISTIC mode - all timing and RNG will be virtualized");
    }
    
    if env::var("CARGO_FEATURE_DEBUG").is_ok() {
        println!("cargo:rustc-cfg=feature_debug");
        println!("cargo:warning=Building kernel in DEBUG mode - memory safety features enabled");
    }

    // Linker configuration
    configure_linker(&target_arch);

    // Set version information
    set_version_info();

    // Configure safe Zig FFI library if Zig is available
    configure_zig();
}

fn configure_x86_64() {
    println!("cargo:rustc-link-arg-bins=--gc-sections");
    
    // x86_64-specific linker flags
    if env::var("CARGO_FEATURE_UEFI").is_ok() {
        println!("cargo:rustc-link-arg-bins=-Wl,--subsystem,efi_application");
        println!("cargo:rustc-link-arg-bins=-Wl,--entry,efi_main");
    }
}

fn configure_aarch64() {
    println!("cargo:rustc-link-arg-bins=--gc-sections");
    
    // aarch64-specific configuration
    if env::var("CARGO_FEATURE_UEFI").is_ok() {
        println!("cargo:rustc-link-arg-bins=-Wl,--subsystem,efi_application");
    }
}

fn configure_uefi() {
    println!("cargo:rustc-cfg=boot_uefi");
    
    // UEFI-specific build flags
    println!("cargo:rustc-link-arg-bins=-nostdlib");
    println!("cargo:rustc-link-arg-bins=-Wl,--no-dynamic-linker");
}

fn configure_bare_metal() {
    println!("cargo:rustc-cfg=boot_bare_metal");
    
    // Bare metal specific configuration
    println!("cargo:rustc-link-arg-bins=-nostdlib");
    println!("cargo:rustc-link-arg-bins=-static");
}

fn configure_linker(target_arch: &str) {
    let linker_script = match target_arch {
        "x86_64" => "arch/x86_64/linker.ld",
        "aarch64" => "arch/aarch64/linker.ld", 
        _ => return,
    };

    let linker_script_path = PathBuf::from("src").join(linker_script);
    
    if linker_script_path.exists() {
        println!("cargo:rustc-link-arg-bins=-T{}", linker_script_path.display());
        println!("cargo:rerun-if-changed={}", linker_script_path.display());
    }
}

fn set_version_info() {
    // Set kernel version information
    if let Ok(git_hash) = env::var("POLYMERA_GIT_HASH") {
        println!("cargo:rustc-env=POLYMERA_GIT_HASH={}", git_hash);
    }
    
    if let Ok(build_date) = env::var("POLYMERA_BUILD_DATE") {
        println!("cargo:rustc-env=POLYMERA_BUILD_DATE={}", build_date);
    }
    
    // Set default values if not provided
    println!("cargo:rustc-env=POLYMERA_KERNEL_VERSION={}", env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=POLYMERA_KERNEL_NAME=polymera-kernel");
}

fn configure_zig() {
    let has_zig = std::process::Command::new("zig")
        .arg("version")
        .output()
        .is_ok();

    if has_zig {
        println!("cargo:warning=Zig compiler detected. Compiling safe Zig FFI String/Memory library...");
        
        let out_dir = env::current_dir().unwrap().join("zig");
        
        let status = std::process::Command::new("zig")
            .arg("build")
            .current_dir(&out_dir)
            .status();

        match status {
            Ok(s) if s.success() => {
                let zig_out_lib = out_dir.join("zig-out").join("lib");
                println!("cargo:rustc-link-search=native={}", zig_out_lib.display());
                println!("cargo:rustc-link-lib=static=polymera_zig");
                println!("cargo:rustc-cfg=feature_polymera_zig");
                println!("cargo:warning=Successfully linked safe Zig FFI library (libpolymera_zig.a / polymera_zig.lib)");
            }
            Ok(s) => {
                println!("cargo:warning=Zig build failed with exit status {}. Falling back to default C implementation.", s);
            }
            Err(e) => {
                println!("cargo:warning=Failed to execute zig build: {}. Falling back to default C implementation.", e);
            }
        }
    } else {
        println!("cargo:warning=Zig compiler ('zig') not found in PATH. Falling back to default C implementation.");
    }
}


