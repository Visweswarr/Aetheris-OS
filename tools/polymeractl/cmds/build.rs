use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args)]
pub struct BuildCommand {
    /// Target to build (kernel, services, crypto, ui, all)
    #[arg(short, long, default_value = "all")]
    target: String,

    /// Build profile (debug, release, optimized)
    #[arg(short, long, default_value = "debug")]
    profile: String,

    /// Output directory
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,

    /// Enable parallel builds
    #[arg(short, long)]
    parallel: bool,

    /// Skip tests
    #[arg(long)]
    no_test: bool,

    /// Build packages
    #[arg(long)]
    packages: bool,

    /// Build documentation
    #[arg(long)]
    docs: bool,

    /// Clean before building
    #[arg(long)]
    clean: bool,

    /// Show build progress
    #[arg(long)]
    progress: bool,
}

impl BuildCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "🔨 Building Polymera OS Components".bold());
        println!("Target: {}", self.target.blue());
        println!("Profile: {}", self.profile.blue());
        println!("Output: {}", self.output.display().to_string().blue());

        // Create output directory
        std::fs::create_dir_all(&self.output)?;

        // Clean if requested
        if self.clean {
            self.clean_build()?;
        }

        // Build based on target
        match self.target.as_str() {
            "kernel" => self.build_kernel().await?,
            "services" => self.build_services().await?,
            "crypto" => self.build_crypto().await?,
            "ui" => self.build_ui().await?,
            "packages" => self.build_packages().await?,
            "all" => self.build_all().await?,
            _ => return Err(format!("Unknown build target: {}", self.target).into()),
        }

        // Build documentation if requested
        if self.docs {
            self.build_documentation().await?;
        }

        // Run tests unless skipped
        if !self.no_test {
            self.run_tests().await?;
        }

        println!("{}", "✅ Build completed successfully".green());
        Ok(())
    }

    /// Clean build artifacts
    fn clean_build(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧹 Cleaning build artifacts...");
        
        let clean_dirs = ["target", "dist", "build"];
        for dir in &clean_dirs {
            if std::path::Path::new(dir).exists() {
                std::fs::remove_dir_all(dir)?;
                println!("  ✓ Cleaned {}", dir);
            }
        }
        
        Ok(())
    }

    /// Build kernel components
    async fn build_kernel(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Building Kernel Components".bold());
        
        // Build kernel crate
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        
        if self.profile == "release" {
            cmd.arg("--release");
        }
        
        if self.parallel {
            cmd.arg("--jobs").arg("0");
        }
        
        if self.progress {
            cmd.arg("--verbose");
        }
        
        cmd.arg("--package").arg("polymera-kernel");
        
        let output = cmd.output()?;
        
        if output.status.success() {
            println!("  ✓ Kernel built successfully");
            
            // Copy kernel binary to output directory
            let kernel_path = if self.profile == "release" {
                "target/release/polymera-kernel"
            } else {
                "target/debug/polymera-kernel"
            };
            
            if std::path::Path::new(kernel_path).exists() {
                let dest_path = self.output.join("kernel");
                std::fs::copy(kernel_path, dest_path)?;
                println!("  ✓ Kernel copied to output directory");
            }
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Kernel build failed: {}", error).into());
        }
        
        Ok(())
    }

    /// Build service components
    async fn build_services(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Building Service Components".bold());
        
        let services = ["polyimage", "attest", "observability"];
        
        for service in &services {
            println!("  Building {}...", service);
            
            let mut cmd = Command::new("cargo");
            cmd.arg("build");
            
            if self.profile == "release" {
                cmd.arg("--release");
            }
            
            if self.parallel {
                cmd.arg("--jobs").arg("0");
            }
            
            cmd.arg("--package").arg(format!("polymera-{}", service));
            
            let output = cmd.output()?;
            
            if output.status.success() {
                println!("    ✓ {} built successfully", service);
                
                // Copy service binary to output directory
                let service_path = if self.profile == "release" {
                    format!("target/release/polymera-{}", service)
                } else {
                    format!("target/debug/polymera-{}", service)
                };
                
                if std::path::Path::new(&service_path).exists() {
                    let dest_path = self.output.join(format!("{}-service", service));
                    std::fs::copy(service_path, dest_path)?;
                }
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                println!("    ⚠ {} build failed: {}", service, error);
            }
        }
        
        Ok(())
    }

    /// Build crypto components
    async fn build_crypto(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Building Crypto Components".bold());
        
        // Build crypto crate
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        
        if self.profile == "release" {
            cmd.arg("--release");
        }
        
        if self.parallel {
            cmd.arg("--jobs").arg("0");
        }
        
        cmd.arg("--package").arg("polymera-crypto");
        
        let output = cmd.output()?;
        
        if output.status.success() {
            println!("  ✓ Crypto components built successfully");
            
            // Copy crypto library to output directory
            let lib_path = if self.profile == "release" {
                "target/release/libpolymera_crypto.rlib"
            } else {
                "target/debug/libpolymera_crypto.rlib"
            };
            
            if std::path::Path::new(lib_path).exists() {
                let dest_path = self.output.join("libpolymera_crypto.rlib");
                std::fs::copy(lib_path, dest_path)?;
                println!("  ✓ Crypto library copied to output directory");
            }
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Crypto build failed: {}", error).into());
        }
        
        Ok(())
    }

    /// Build UI components
    async fn build_ui(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Building UI Components".bold());
        
        // Check if Node.js is available
        if let Ok(_) = Command::new("node").arg("--version").output() {
            println!("  Building web UI...");
            
            // Build web UI if package.json exists
            if std::path::Path::new("ui/web/package.json").exists() {
                let mut cmd = Command::new("npm");
                cmd.arg("run").arg("build");
                cmd.current_dir("ui/web");
                
                let output = cmd.output()?;
                
                if output.status.success() {
                    println!("    ✓ Web UI built successfully");
                    
                    // Copy built UI to output directory
                    let ui_src = "ui/web/dist";
                    let ui_dest = self.output.join("ui");
                    
                    if std::path::Path::new(ui_src).exists() {
                        copy_dir_all(ui_src, &ui_dest)?;
                        println!("    ✓ Web UI copied to output directory");
                    }
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    println!("    ⚠ Web UI build failed: {}", error);
                }
            }
        } else {
            println!("  ⚠ Node.js not available, skipping UI build");
        }
        
        Ok(())
    }

    /// Build packages
    async fn build_packages(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Building Packages".bold());
        
        // Build pack crate
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        
        if self.profile == "release" {
            cmd.arg("--release");
        }
        
        cmd.arg("--package").arg("polymera-pack");
        
        let output = cmd.output()?;
        
        if output.status.success() {
            println!("  ✓ Package builder built successfully");
            
            // Create sample packages
            self.create_sample_packages().await?;
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Package build failed: {}", error).into());
        }
        
        Ok(())
    }

    /// Create sample packages
    async fn create_sample_packages(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("  Creating sample packages...");
        
        let packages_dir = self.output.join("packages");
        std::fs::create_dir_all(&packages_dir)?;
        
        // Create kernel package
        let kernel_pkg = packages_dir.join("kernel");
        std::fs::create_dir_all(&kernel_pkg)?;
        
        // Copy kernel binary
        if let Ok(_) = std::fs::copy(self.output.join("kernel"), kernel_pkg.join("kernel")) {
            println!("    ✓ Created kernel package");
        }
        
        // Create service package
        let service_pkg = packages_dir.join("services");
        std::fs::create_dir_all(&service_pkg)?;
        
        // Copy service binaries
        for service in ["polyimage", "attest", "observability"] {
            let service_bin = self.output.join(format!("{}-service", service));
            if service_bin.exists() {
                let _ = std::fs::copy(&service_bin, service_pkg.join(format!("{}-service", service)));
            }
        }
        
        println!("    ✓ Created services package");
        
        Ok(())
    }

    /// Build all components
    async fn build_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Building All Components".bold());
        
        // Build core components
        self.build_kernel().await?;
        self.build_services().await?;
        self.build_crypto().await?;
        self.build_ui().await?;
        
        // Build packages if requested
        if self.packages {
            self.build_packages().await?;
        }
        
        Ok(())
    }

    /// Build documentation
    async fn build_documentation(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Building Documentation".bold());
        
        // Build Rust documentation
        let mut cmd = Command::new("cargo");
        cmd.arg("doc");
        
        if self.profile == "release" {
            cmd.arg("--release");
        }
        
        if self.parallel {
            cmd.arg("--jobs").arg("0");
        }
        
        let output = cmd.output()?;
        
        if output.status.success() {
            println!("  ✓ Rust documentation built successfully");
            
            // Copy documentation to output directory
            let doc_src = "target/doc";
            let doc_dest = self.output.join("docs");
            
            if std::path::Path::new(doc_src).exists() {
                copy_dir_all(doc_src, &doc_dest)?;
                println!("  ✓ Documentation copied to output directory");
            }
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("  ⚠ Documentation build failed: {}", error);
        }
        
        Ok(())
    }

    /// Run tests
    async fn run_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "Running Tests".bold());
        
        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        
        if self.profile == "release" {
            cmd.arg("--release");
        }
        
        if self.parallel {
            cmd.arg("--jobs").arg("0");
        }
        
        if self.progress {
            cmd.arg("--verbose");
        }
        
        let output = cmd.output()?;
        
        if output.status.success() {
            println!("  ✓ All tests passed");
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Tests failed: {}", error).into());
        }
        
        Ok(())
    }
}

/// Copy directory recursively
fn copy_dir_all(src: &str, dst: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let src_path = std::path::Path::new(src);
    if !src_path.exists() {
        return Ok(());
    }
    
    if src_path.is_file() {
        std::fs::copy(src_path, dst)?;
        return Ok(());
    }
    
    std::fs::create_dir_all(dst)?;
    
    for entry in std::fs::read_dir(src_path)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(src_path.to_str().unwrap(), &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}
