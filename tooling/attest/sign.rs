//! Kernel Image Signing Tool
//! 
//! Signs Polymera OS kernel images with Dilithium2 signatures for secure boot verification.

use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::env;
use std::process;

use serde::{Deserialize, Serialize};

/// Kernel attestation header structure (must match kernel/src/boot/attest.rs)
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KernelAttestationHeader {
    /// Magic number for identification
    pub magic: [u8; 4],
    /// Version of the attestation format
    pub version: u8,
    /// Flags (e.g., dev mode, production)
    pub flags: u8,
    /// Reserved for future use
    pub reserved: [u8; 2],
    /// Build ID hash (SHA-256)
    pub build_id: [u8; 32],
    /// Signing certificate (Dilithium2 public key)
    pub signing_cert: [u8; 1312],
    /// Signature over the kernel image (excluding this header)
    pub signature: [u8; 2701],
    /// Reserved for future extensions
    pub reserved_ext: [u8; 64],
}

impl KernelAttestationHeader {
    /// Create a new attestation header
    pub fn new() -> Self {
        Self {
            magic: *b"POLY",
            version: 1,
            flags: 0x01, // Development build
            reserved: [0; 2],
            build_id: [0; 32],
            signing_cert: [0; 1312],
            signature: [0; 2701],
            reserved_ext: [0; 64],
        }
    }
    
    /// Set build ID from hash
    pub fn set_build_id(&mut self, build_id: [u8; 32]) {
        self.build_id = build_id;
    }
    
    /// Set signing certificate
    pub fn set_signing_cert(&mut self, cert: [u8; 1312]) {
        self.signing_cert = cert;
    }
    
    /// Set signature
    pub fn set_signature(&mut self, signature: [u8; 2701]) {
        self.signature = signature;
    }
    
    /// Set build type flags
    pub fn set_build_type(&mut self, is_dev: bool, is_production: bool) {
        self.flags = 0;
        if is_dev {
            self.flags |= 0x01;
        }
        if is_production {
            self.flags |= 0x02;
        }
    }
}

/// Signing configuration
#[derive(Debug, Clone)]
pub struct SigningConfig {
    /// Input kernel image path
    pub input_path: PathBuf,
    /// Output signed image path
    pub output_path: PathBuf,
    /// Private key path for signing
    pub private_key_path: PathBuf,
    /// Build ID (git hash or build identifier)
    pub build_id: String,
    /// Build type (dev, production)
    pub build_type: BuildType,
    /// Overwrite output file if it exists
    pub overwrite: bool,
}

/// Build type enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildType {
    Development,
    Production,
}

impl BuildType {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(BuildType::Development),
            "prod" | "production" => Ok(BuildType::Production),
            _ => Err(format!("Unknown build type: {}", s)),
        }
    }
    
    fn to_string(&self) -> &'static str {
        match self {
            BuildType::Development => "development",
            BuildType::Production => "production",
        }
    }
}

/// Kernel image signer
pub struct KernelSigner {
    config: SigningConfig,
}

impl KernelSigner {
    /// Create a new kernel signer
    pub fn new(config: SigningConfig) -> Self {
        Self { config }
    }
    
    /// Sign the kernel image
    pub fn sign_kernel(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔐 Signing kernel image...");
        println!("Input: {}", self.config.input_path.display());
        println!("Output: {}", self.config.output_path.display());
        println!("Build Type: {}", self.config.build_type.to_string());
        println!("Build ID: {}", self.config.build_id);
        
        // Check input file exists
        if !self.config.input_path.exists() {
            return Err(format!("Input file not found: {}", self.config.input_path.display()).into());
        }
        
        // Check output directory exists
        if let Some(parent) = self.config.output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Check if output file exists and overwrite flag
        if self.config.output_path.exists() && !self.config.overwrite {
            return Err(format!("Output file exists and overwrite not specified: {}", 
                             self.config.output_path.display()).into());
        }
        
        // Read input kernel image
        let mut input_file = File::open(&self.config.input_path)?;
        let mut kernel_data = Vec::new();
        input_file.read_to_end(&mut kernel_data)?;
        
        println!("📖 Read kernel image: {} bytes", kernel_data.len());
        
        // Generate build ID hash
        let build_id_hash = self.generate_build_id_hash(&self.config.build_id);
        println!("🆔 Build ID hash: {}", bytes_to_hex(&build_id_hash));
        
        // Load private key
        let private_key = self.load_private_key(&self.config.private_key_path)?;
        println!("🔑 Loaded private key from: {}", self.config.private_key_path.display());
        
        // Extract public key (certificate)
        let public_key = self.extract_public_key(&private_key)?;
        println!("📜 Extracted public key certificate");
        
        // Create attestation header
        let mut header = KernelAttestationHeader::new();
        header.set_build_id(build_id_hash);
        header.set_signing_cert(public_key);
        header.set_build_type(
            self.config.build_type == BuildType::Development,
            self.config.build_type == BuildType::Production,
        );
        
        // Sign kernel data (excluding header)
        let signature = self.sign_kernel_data(&kernel_data, &private_key)?;
        header.set_signature(signature);
        println!("✍️  Generated kernel signature");
        
        // Write signed image
        self.write_signed_image(&kernel_data, &header)?;
        println!("💾 Wrote signed kernel image");
        
        // Verify the signature
        if self.verify_signature(&kernel_data, &header, &public_key)? {
            println!("✅ Signature verification successful");
        } else {
            return Err("Signature verification failed".into());
        }
        
        println!("🎉 Kernel signing completed successfully!");
        Ok(())
    }
    
    /// Generate build ID hash from string
    fn generate_build_id_hash(&self, build_id: &str) -> [u8; 32] {
        // Simple hash function for demonstration
        // In production, this would use SHA-256
        let mut hash = [0u8; 32];
        let bytes = build_id.as_bytes();
        
        for (i, byte) in bytes.iter().enumerate() {
            hash[i % 32] ^= *byte;
        }
        
        // Add some entropy based on build ID length
        hash[31] = bytes.len() as u8;
        
        hash
    }
    
    /// Load private key from file
    fn load_private_key(&self, key_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut key_file = File::open(key_path)?;
        let mut key_data = Vec::new();
        key_file.read_to_end(&mut key_data)?;
        
        // Validate key format (should be Dilithium2 private key)
        if key_data.len() != 2560 { // Dilithium2 private key size
            return Err(format!("Invalid private key size: {} bytes (expected 2560)", 
                             key_data.len()).into());
        }
        
        Ok(key_data)
    }
    
    /// Extract public key from private key
    fn extract_public_key(&self, private_key: &[u8]) -> Result<[u8; 1312], Box<dyn std::error::Error>> {
        // In a real implementation, this would derive the public key from the private key
        // For now, we'll simulate this by taking a portion of the private key
        
        if private_key.len() < 1312 {
            return Err("Private key too small to extract public key".into());
        }
        
        let mut public_key = [0u8; 1312];
        public_key.copy_from_slice(&private_key[..1312]);
        
        Ok(public_key)
    }
    
    /// Sign kernel data
    fn sign_kernel_data(&self, kernel_data: &[u8], private_key: &[u8]) -> Result<[u8; 2701], Box<dyn std::error::Error>> {
        // In a real implementation, this would use Dilithium2 to sign the data
        // For now, we'll simulate this by creating a deterministic signature
        
        let mut signature = [0u8; 2701];
        
        // Simple signature simulation based on kernel data and private key
        for (i, byte) in kernel_data.iter().enumerate() {
            if i < signature.len() {
                signature[i] = *byte ^ private_key[i % private_key.len()];
            }
        }
        
        // Add some deterministic padding
        for i in kernel_data.len()..signature.len() {
            signature[i] = (i * 7 + private_key[i % private_key.len()] as usize) as u8;
        }
        
        Ok(signature)
    }
    
    /// Write signed image to output file
    fn write_signed_image(&self, kernel_data: &[u8], header: &KernelAttestationHeader) -> Result<(), Box<dyn std::error::Error>> {
        let mut output_file = File::create(&self.config.output_path)?;
        
        // Write header
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                header as *const _ as *const u8,
                std::mem::size_of::<KernelAttestationHeader>(),
            )
        };
        output_file.write_all(header_bytes)?;
        
        // Write kernel data
        output_file.write_all(kernel_data)?;
        
        output_file.flush()?;
        
        println!("📁 Wrote signed image: {} bytes", 
                header_bytes.len() + kernel_data.len());
        
        Ok(())
    }
    
    /// Verify the generated signature
    fn verify_signature(&self, kernel_data: &[u8], header: &KernelAttestationHeader, public_key: &[u8; 1312]) -> Result<bool, Box<dyn std::error::Error>> {
        // In a real implementation, this would use Dilithium2 to verify the signature
        // For now, we'll simulate verification
        
        // Check that the header contains the expected data
        if header.magic != *b"POLY" {
            return Ok(false);
        }
        
        if header.version != 1 {
            return Ok(false);
        }
        
        // Check that the certificate matches
        if header.signing_cert != *public_key {
            return Ok(false);
        }
        
        // Simulate signature verification
        let expected_signature = self.sign_kernel_data(kernel_data, public_key)?;
        let signature_matches = header.signature == expected_signature;
        
        Ok(signature_matches)
    }
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}

/// Print usage information
fn print_usage() {
    println!("Kernel Image Signing Tool");
    println!();
    println!("Usage: kernel-signer [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -i, --input PATH       Input kernel image path (required)");
    println!("  -o, --output PATH      Output signed image path (required)");
    println!("  -k, --key PATH         Private key path (required)");
    println!("  -b, --build-id ID      Build ID (git hash, default: auto-detect)");
    println!("  -t, --type TYPE        Build type: dev|production (default: dev)");
    println!("  -f, --force            Overwrite output file if it exists");
    println!("  -h, --help             Show this help message");
    println!();
    println!("Examples:");
    println!("  kernel-signer -i polymera-kernel.bin -o signed-kernel.bin -k dev-key.pem");
    println!("  kernel-signer -i kernel.bin -o signed.bin -k prod-key.pem -t production -b v1.0.0");
}

/// Parse command line arguments
fn parse_args() -> Result<SigningConfig, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut config = SigningConfig {
        input_path: PathBuf::new(),
        output_path: PathBuf::new(),
        private_key_path: PathBuf::new(),
        build_id: String::new(),
        build_type: BuildType::Development,
        overwrite: false,
    };
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-i" | "--input" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing input path".into());
                }
                config.input_path = PathBuf::from(&args[i]);
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing output path".into());
                }
                config.output_path = PathBuf::from(&args[i]);
            }
            "-k" | "--key" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing private key path".into());
                }
                config.private_key_path = PathBuf::from(&args[i]);
            }
            "-b" | "--build-id" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing build ID".into());
                }
                config.build_id = args[i].clone();
            }
            "-t" | "--type" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing build type".into());
                }
                config.build_type = BuildType::from_str(&args[i])?;
            }
            "-f" | "--force" => {
                config.overwrite = true;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]).into());
            }
        }
        i += 1;
    }
    
    // Validate required arguments
    if config.input_path.as_os_str().is_empty() {
        return Err("Input path is required".into());
    }
    
    if config.output_path.as_os_str().is_empty() {
        return Err("Output path is required".into());
    }
    
    if config.private_key_path.as_os_str().is_empty() {
        return Err("Private key path is required".into());
    }
    
    // Auto-detect build ID if not specified
    if config.build_id.is_empty() {
        config.build_id = auto_detect_build_id()?;
    }
    
    Ok(config)
}

/// Auto-detect build ID from git or environment
fn auto_detect_build_id() -> Result<String, Box<dyn std::error::Error>> {
    // Try to get git hash
    if let Ok(output) = std::process::Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output() {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !git_hash.is_empty() {
                return Ok(git_hash);
            }
        }
    }
    
    // Fallback to timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    
    Ok(format!("build-{}", timestamp))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Polymera OS Kernel Image Signing Tool");
    println!("==========================================");
    
    // Parse command line arguments
    let config = match parse_args() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            println!();
            print_usage();
            process::exit(1);
        }
    };
    
    // Create signer and sign kernel
    let signer = KernelSigner::new(config);
    if let Err(e) = signer.sign_kernel() {
        eprintln!("❌ Signing failed: {}", e);
        process::exit(1);
    }
    
    Ok(())
}
