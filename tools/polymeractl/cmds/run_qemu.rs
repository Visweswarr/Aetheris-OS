use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

#[derive(Args)]
pub struct RunQemuCommand {
    /// Architecture (x86_64, aarch64)
    #[arg(short, long, default_value = "x86_64")]
    arch: String,

    /// Kernel image path
    #[arg(short, long)]
    kernel: Option<PathBuf>,

    /// Disk image path
    #[arg(short, long)]
    disk: Option<PathBuf>,

    /// Memory size in MB
    #[arg(short, long, default_value = "512")]
    memory: u32,

    /// Number of CPU cores
    #[arg(short, long, default_value = "2")]
    cpus: u32,

    /// Enable graphics (disable for headless)
    #[arg(long)]
    graphics: bool,

    /// Enable network
    #[arg(long)]
    network: bool,

    /// Serial output file
    #[arg(long)]
    serial: Option<PathBuf>,

    /// Enable debugging
    #[arg(long)]
    debug: bool,

    /// QEMU binary path
    #[arg(long)]
    qemu_bin: Option<PathBuf>,

    /// OVMF firmware path
    #[arg(long)]
    ovmf_path: Option<PathBuf>,

    /// Additional QEMU arguments
    #[arg(long)]
    extra_args: Vec<String>,
}

impl RunQemuCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", "🚀 Running QEMU with OVMF".bold());
        println!("Architecture: {}", self.arch.blue());
        println!("Memory: {} MB", self.memory.to_string().blue());
        println!("CPUs: {}", self.cpus.to_string().blue());

        // Validate architecture
        if !["x86_64", "aarch64"].contains(&self.arch.as_str()) {
            return Err("Invalid architecture. Must be 'x86_64' or 'aarch64'".into());
        }

        // Find QEMU binary
        let qemu_bin = self.find_qemu_binary()?;
        println!("QEMU Binary: {}", qemu_bin.display().to_string().blue());

        // Find OVMF firmware
        let ovmf_path = self.find_ovmf_firmware()?;
        println!("OVMF Firmware: {}", ovmf_path.display().to_string().blue());

        // Build QEMU command
        let mut cmd = Command::new(&qemu_bin);
        self.build_qemu_args(&mut cmd)?;

        // Show command being executed
        if self.debug {
            println!("Executing: {:?}", cmd);
        }

        // Run QEMU
        println!("Starting QEMU...");
        let status = cmd.status()?;

        if status.success() {
            println!("{}", "✅ QEMU exited successfully".green());
        } else {
            println!("{}", "⚠️ QEMU exited with status: {}".yellow(), status);
        }

        Ok(())
    }

    /// Find QEMU binary
    fn find_qemu_binary(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Use provided path if specified
        if let Some(path) = &self.qemu_bin {
            if path.exists() {
                return Ok(path.clone());
            }
            return Err(format!("QEMU binary not found at: {}", path.display()).into());
        }

        // Try to find QEMU in PATH
        let qemu_name = match self.arch.as_str() {
            "x86_64" => "qemu-system-x86_64",
            "aarch64" => "qemu-system-aarch64",
            _ => unreachable!(),
        };

        // Check if QEMU is available
        match Command::new(qemu_name).arg("--version").output() {
            Ok(_) => Ok(PathBuf::from(qemu_name)),
            Err(_) => {
                // Try to find in common locations
                let common_paths = [
                    "/usr/bin",
                    "/usr/local/bin",
                    "/opt/homebrew/bin", // macOS Homebrew
                    "C:\\Program Files\\qemu", // Windows
                ];

                for path in &common_paths {
                    let qemu_path = PathBuf::from(path).join(qemu_name);
                    if qemu_path.exists() {
                        return Ok(qemu_path);
                    }
                }

                Err(format!("QEMU binary '{}' not found in PATH or common locations", qemu_name).into())
            }
        }
    }

    /// Find OVMF firmware
    fn find_ovmf_firmware(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Use provided path if specified
        if let Some(path) = &self.ovmf_path {
            if path.exists() {
                return Ok(path.clone());
            }
            return Err(format!("OVMF firmware not found at: {}", path.display()).into());
        }

        // Try to find OVMF in common locations
        let ovmf_name = match self.arch.as_str() {
            "x86_64" => "OVMF.fd",
            "aarch64" => "OVMF_AA64.fd",
            _ => unreachable!(),
        };

        let common_paths = [
            "tooling/qemu",
            "/usr/share/ovmf",
            "/usr/share/qemu/ovmf-x86_64",
            "/opt/homebrew/share/qemu/ovmf-x86_64", // macOS Homebrew
        ];

        for path in &common_paths {
            let ovmf_path = PathBuf::from(path).join(ovmf_name);
            if ovmf_path.exists() {
                return Ok(ovmf_path);
            }
        }

        // Try to use Nix OVMF if available
        if let Ok(_) = Command::new("nix").arg("--version").output() {
            println!("  Trying to fetch OVMF via Nix...");
            let nix_ovmf = self.fetch_nix_ovmf()?;
            if nix_ovmf.exists() {
                return Ok(nix_ovmf);
            }
        }

        Err(format!("OVMF firmware '{}' not found. Please install OVMF or specify --ovmf-path", ovmf_name).into())
    }

    /// Fetch OVMF via Nix
    fn fetch_nix_ovmf(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let ovmf_dir = std::env::temp_dir().join("polymera-ovmf");
        std::fs::create_dir_all(&ovmf_dir)?;

        // Try to build OVMF via Nix
        let mut cmd = Command::new("nix");
        cmd.arg("build")
            .arg("--out-link")
            .arg(ovmf_dir.join("result"))
            .arg("--impure")
            .arg("--expr")
            .arg("import <nixpkgs> { config.allowUnfree = true; }");

        let ovmf_pkg = match self.arch.as_str() {
            "x86_64" => "ovmf",
            "aarch64" => "ovmf-aarch64",
            _ => unreachable!(),
        };

        cmd.arg(format!("{}.fd", ovmf_pkg));

        match cmd.output() {
            Ok(output) if output.status.success() => {
                let ovmf_path = ovmf_dir.join("result").join("share").join("ovmf").join("OVMF.fd");
                if ovmf_path.exists() {
                    return Ok(ovmf_path);
                }
            }
            _ => {}
        }

        Err("Failed to fetch OVMF via Nix".into())
    }

    /// Build QEMU command arguments
    fn build_qemu_args(&self, cmd: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
        // Basic QEMU arguments
        cmd.arg("-m").arg(self.memory.to_string());
        cmd.arg("-smp").arg(self.cpus.to_string());

        // Architecture-specific arguments
        match self.arch.as_str() {
            "x86_64" => {
                cmd.arg("-machine").arg("q35");
                cmd.arg("-cpu").arg("host");
                cmd.arg("-enable-kvm");
            }
            "aarch64" => {
                cmd.arg("-machine").arg("virt");
                cmd.arg("-cpu").arg("cortex-a57");
                cmd.arg("-machine").arg("gic-version=3");
            }
            _ => unreachable!(),
        }

        // OVMF firmware
        let ovmf_path = self.find_ovmf_firmware()?;
        cmd.arg("-drive").arg(format!("file={},if=pflash,format=raw,readonly=on", ovmf_path.display()));

        // Kernel image
        if let Some(kernel) = &self.kernel {
            if !kernel.exists() {
                return Err(format!("Kernel image not found: {}", kernel.display()).into());
            }
            cmd.arg("-kernel").arg(kernel);
            println!("  Kernel: {}", kernel.display().to_string().blue());
        }

        // Disk image
        if let Some(disk) = &self.disk {
            if !disk.exists() {
                return Err(format!("Disk image not found: {}", disk.display()).into());
            }
            cmd.arg("-drive").arg(format!("file={},if=virtio,format=raw", disk.display()));
            println!("  Disk: {}", disk.display().to_string().blue());
        }

        // Graphics
        if self.graphics {
            cmd.arg("-display").arg("gtk");
            cmd.arg("-vga").arg("virtio");
        } else {
            cmd.arg("-nographic");
            cmd.arg("-serial").arg("mon:stdio");
        }

        // Network
        if self.network {
            cmd.arg("-netdev").arg("user,id=net0,hostfwd=tcp::2222-:22");
            cmd.arg("-device").arg("virtio-net-pci,netdev=net0");
            println!("  Network: Enabled (SSH forwarding: localhost:2222)");
        }

        // Serial output
        if let Some(serial) = &self.serial {
            cmd.arg("-serial").arg(format!("file:{}", serial.display()));
            println!("  Serial Output: {}", serial.display().to_string().blue());
        }

        // Debug options
        if self.debug {
            cmd.arg("-d").arg("guest_errors");
            cmd.arg("-D").arg("qemu-debug.log");
            println!("  Debug: Enabled (log: qemu-debug.log)");
        }

        // Additional arguments
        for arg in &self.extra_args {
            cmd.arg(arg);
        }

        Ok(())
    }
}

impl Default for RunQemuCommand {
    fn default() -> Self {
        Self {
            arch: "x86_64".to_string(),
            kernel: None,
            disk: None,
            memory: 512,
            cpus: 2,
            graphics: false,
            network: false,
            serial: None,
            debug: false,
            qemu_bin: None,
            ovmf_path: None,
            extra_args: Vec::new(),
        }
    }
}
