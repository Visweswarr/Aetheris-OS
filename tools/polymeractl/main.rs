use clap::{Parser, Subcommand};
use colored::*;
use std::process;
use tokio;

mod cmds;
mod utils;

use cmds::{
    build::BuildCommand,
    run_qemu::RunQemuCommand,
    mkimage::MkimageCommand,
    verify::VerifyCommand,
    sbom::SbomCommand,
    sign::SignCommand,
};

#[derive(Parser)]
#[command(
    name = "polymeractl",
    about = "Polymera OS Developer CLI",
    version = env!("CARGO_PKG_VERSION"),
    author = "Polymera OS Team",
    long_about = "Command-line interface for Polymera OS development tasks including building, running, image creation, verification, SBOM generation, and signing."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Enable debug output
    #[arg(short, long)]
    debug: bool,

    /// Suppress colored output
    #[arg(long)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build Polymera OS components and packages
    Build {
        #[command(flatten)]
        cmd: BuildCommand,
    },

    /// Run QEMU with OVMF for testing
    RunQemu {
        #[command(flatten)]
        cmd: RunQemuCommand,
    },

    /// Create system images with A/B slots
    Mkimage {
        #[command(flatten)]
        cmd: MkimageCommand,
    },

    /// Verify packages and images
    Verify {
        #[command(flatten)]
        cmd: VerifyCommand,
    },

    /// Generate and manage SBOMs
    Sbom {
        #[command(flatten)]
        cmd: SbomCommand,
    },

    /// Sign packages and images
    Sign {
        #[command(flatten)]
        cmd: SignCommand,
    },

    /// Show system information and status
    #[command(name = "info")]
    Info {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Initialize a new Polymera OS project
    #[command(name = "init")]
    Init {
        /// Project name
        #[arg(value_name = "NAME")]
        name: String,

        /// Project template
        #[arg(short, long, default_value = "basic")]
        template: String,

        /// Initialize git repository
        #[arg(long)]
        git: bool,
    },

    /// Update development environment
    #[command(name = "update")]
    Update {
        /// Update toolchains
        #[arg(long)]
        toolchains: bool,

        /// Update dependencies
        #[arg(long)]
        dependencies: bool,

        /// Update all components
        #[arg(long)]
        all: bool,
    },

    /// Clean build artifacts and temporary files
    #[command(name = "clean")]
    Clean {
        /// Clean build directory
        #[arg(long)]
        build: bool,

        /// Clean cache directory
        #[arg(long)]
        cache: bool,

        /// Clean all artifacts
        #[arg(long)]
        all: bool,
    },
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Configure colored output
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Set up logging based on verbosity
    setup_logging(cli.verbose, cli.debug);

    // Print banner
    if !cli.no_color {
        print_banner();
    }

    // Execute command
    let result = match cli.command {
        Commands::Build { cmd } => cmd.execute().await,
        Commands::RunQemu { cmd } => cmd.execute().await,
        Commands::Mkimage { cmd } => cmd.execute().await,
        Commands::Verify { cmd } => cmd.execute().await,
        Commands::Sbom { cmd } => cmd.execute().await,
        Commands::Sign { cmd } => cmd.execute().await,
        Commands::Info { detailed } => handle_info(detailed).await,
        Commands::Init { name, template, git } => handle_init(name, template, git).await,
        Commands::Update { toolchains, dependencies, all } => handle_update(toolchains, dependencies, all).await,
        Commands::Clean { build, cache, all } => handle_clean(build, cache, all).await,
    };

    // Handle result
    match result {
        Ok(_) => {
            if !cli.no_color {
                println!("{}", "✅ Command completed successfully".green());
            } else {
                println!("Command completed successfully");
            }
            process::exit(0);
        }
        Err(e) => {
            if !cli.no_color {
                eprintln!("{}", format!("❌ Error: {}", e).red());
            } else {
                eprintln!("Error: {}", e);
            }
            process::exit(1);
        }
    }
}

/// Set up logging based on verbosity flags
fn setup_logging(verbose: bool, debug: bool) {
    let log_level = if debug {
        log::LevelFilter::Debug
    } else if verbose {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Warn
    };

    env_logger::Builder::new()
        .filter_level(log_level)
        .init();
}

/// Print the Polymera OS CLI banner
fn print_banner() {
    println!("{}", "╔══════════════════════════════════════════════════════════════╗".blue());
    println!("{}", "║                    Polymera OS CLI                          ║".blue());
    println!("{}", "║                Development Command Center                   ║".blue());
    println!("{}", "╚══════════════════════════════════════════════════════════════╝".blue());
    println!();
}

/// Handle info command
async fn handle_info(detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !detailed {
        println!("{}", "Polymera OS Development Environment".bold());
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        println!("Available Commands:");
        println!("  build     - Build components and packages");
        println!("  run-qemu  - Run QEMU with OVMF");
        println!("  mkimage   - Create system images");
        println!("  verify    - Verify packages and images");
        println!("  sbom      - Generate and manage SBOMs");
        println!("  sign      - Sign packages and images");
        println!("  info      - Show system information");
        println!("  init      - Initialize new project");
        println!("  update    - Update development environment");
        println!("  clean     - Clean build artifacts");
    } else {
        println!("{}", "Detailed System Information".bold());
        println!("CLI Version: {}", env!("CARGO_PKG_VERSION"));
        println!("Rust Version: {}", env!("CARGO_PKG_RUST_VERSION"));
        println!("Build Target: {}", env!("TARGET"));
        println!("Build Profile: {}", env!("PROFILE"));
        println!("Build Time: {}", env!("VERGEN_BUILD_TIMESTAMP"));
        println!("Git Commit: {}", env!("VERGEN_GIT_SHA_SHORT"));
        println!("Git Branch: {}", env!("VERGEN_GIT_BRANCH"));
        
        // Check toolchain availability
        check_toolchain_availability().await?;
    }
    
    Ok(())
}

/// Check toolchain availability
async fn check_toolchain_availability() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "Toolchain Status:".bold());
    
    // Check Rust
    match std::process::Command::new("rustc").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  Rust: {} {}", "✓".green(), version.trim());
        }
        Err(_) => println!("  Rust: {} Not found", "✗".red()),
    }
    
    // Check Cargo
    match std::process::Command::new("cargo").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  Cargo: {} {}", "✓".green(), version.trim());
        }
        Err(_) => println!("  Cargo: {} Not found", "✗".red()),
    }
    
    // Check QEMU
    match std::process::Command::new("qemu-system-x86_64").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  QEMU: {} {}", "✓".green(), version.lines().next().unwrap_or(""));
        }
        Err(_) => println!("  QEMU: {} Not found", "✗".red()),
    }
    
    // Check Nix
    match std::process::Command::new("nix").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  Nix: {} {}", "✓".green(), version.trim());
        }
        Err(_) => println!("  Nix: {} Not found", "✗".red()),
    }
    
    Ok(())
}

/// Handle init command
async fn handle_init(name: String, template: String, git: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", format!("Initializing new Polymera OS project: {}", name).bold());
    println!("Template: {}", template);
    
    // Create project directory
    let project_dir = std::path::Path::new(&name);
    if project_dir.exists() {
        return Err(format!("Project directory '{}' already exists", name).into());
    }
    
    std::fs::create_dir(project_dir)?;
    println!("  ✓ Created project directory");
    
    // Create basic project structure
    create_project_structure(project_dir, &template).await?;
    println!("  ✓ Created project structure");
    
    // Initialize git if requested
    if git {
        std::process::Command::new("git")
            .arg("init")
            .current_dir(project_dir)
            .output()?;
        println!("  ✓ Initialized git repository");
    }
    
    println!("{}", format!("Project '{}' initialized successfully!".green(), name));
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  polymeractl build");
    
    Ok(())
}

/// Create project structure
async fn create_project_structure(project_dir: &std::path::Path, template: &str) -> Result<(), Box<dyn std::error::Error>> {
    match template {
        "basic" => {
            // Create basic structure
            std::fs::create_dir(project_dir.join("src"))?;
            std::fs::create_dir(project_dir.join("tests"))?;
            std::fs::create_dir(project_dir.join("docs"))?;
            
            // Create Cargo.toml
            let cargo_toml = r#"[package]
name = "polymera-project"
version = "0.1.0"
edition = "2021"
description = "A Polymera OS project"
license = "MIT OR Apache-2.0"

[dependencies]
polymera-core = { git = "https://github.com/polymera-os/polymera-os" }

[[bin]]
name = "main"
path = "src/main.rs"

[dev-dependencies]
tokio-test = "0.4"
"#;
            std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
            
            // Create main.rs
            let main_rs = r#"fn main() {
    println!("Hello, Polymera OS!");
}
"#;
            std::fs::write(project_dir.join("src/main.rs"), main_rs)?;
            
            // Create README.md
            let readme = r#"# Polymera OS Project

This is a Polymera OS project.

## Building

```bash
polymeractl build
```

## Running

```bash
cargo run
```
"#;
            std::fs::write(project_dir.join("README.md"), readme)?;
        }
        "service" => {
            // Create service structure
            std::fs::create_dir(project_dir.join("src"))?;
            std::fs::create_dir(project_dir.join("tests"))?;
            std::fs::create_dir(project_dir.join("config"))?;
            std::fs::create_dir(project_dir.join("docs"))?;
            
            // Create service-specific files
            let service_toml = r#"[package]
name = "polymera-service"
version = "0.1.0"
edition = "2021"
description = "A Polymera OS service"
license = "MIT OR Apache-2.0"

[dependencies]
polymera-core = { git = "https://github.com/polymera-os/polymera-os" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

[[bin]]
name = "service"
path = "src/main.rs"
"#;
            std::fs::write(project_dir.join("Cargo.toml"), service_toml)?;
            
            // Create service main.rs
            let service_main = r#"use tokio;

#[tokio::main]
async fn main() {
    println!("Polymera OS Service starting...");
    
    // Service logic here
    
    println!("Service running...");
    
    // Keep service alive
    tokio::signal::ctrl_c().await.unwrap();
    println!("Service shutting down...");
}
"#;
            std::fs::write(project_dir.join("src/main.rs"), service_main)?;
        }
        _ => {
            return Err(format!("Unknown template: {}", template).into());
        }
    }
    
    Ok(())
}

/// Handle update command
async fn handle_update(toolchains: bool, dependencies: bool, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    if all || toolchains {
        println!("{}", "Updating toolchains...".bold());
        // Update Nix toolchains
        if let Ok(_) = std::process::Command::new("nix").arg("flake").arg("update").output() {
            println!("  ✓ Updated Nix toolchains");
        } else {
            println!("  ⚠ Nix not available, skipping toolchain update");
        }
    }
    
    if all || dependencies {
        println!("{}", "Updating dependencies...".bold());
        // Update Cargo dependencies
        if let Ok(_) = std::process::Command::new("cargo").arg("update").output() {
            println!("  ✓ Updated Cargo dependencies");
        } else {
            println!("  ⚠ Cargo not available, skipping dependency update");
        }
    }
    
    println!("{}", "Update completed successfully".green());
    Ok(())
}

/// Handle clean command
async fn handle_clean(build: bool, cache: bool, all: bool) -> Result<(), Box<dyn std::error::Error>> {
    if all || build {
        println!("{}", "Cleaning build artifacts...".bold());
        if let Ok(_) = std::process::Command::new("cargo").arg("clean").output() {
            println!("  ✓ Cleaned build artifacts");
        }
    }
    
    if all || cache {
        println!("{}", "Cleaning cache...".bold());
        // Clean various cache directories
        let cache_dirs = ["target", ".cargo", ".rustup"];
        for dir in &cache_dirs {
            if std::path::Path::new(dir).exists() {
                std::fs::remove_dir_all(dir)?;
                println!("  ✓ Cleaned {}", dir);
            }
        }
    }
    
    println!("{}", "Clean completed successfully".green());
    Ok(())
}
