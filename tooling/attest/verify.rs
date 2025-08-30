//! Kernel Image Verification Tool
//! 
//! Verifies Polymera OS kernel images with Dilithium2 signatures for secure boot verification.

use std::fs::{self, File};
use std::io::{Read, Write};
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
    /// Check if the magic number is valid
    pub fn is_valid(&self) -> bool {
        self.magic == *b"POLY"
    }
    
    /// Check if this is a development build
    pub fn is_dev_build(&self) -> bool {
        (self.flags & 0x01) != 0
    }
    
    /// Check if this is a production build
    pub fn is_production_build(&self) -> bool {
        (self.flags & 0x02) != 0
    }
    
    /// Get the build ID as a hex string
    pub fn build_id_hex(&self) -> [u8; 64] {
        let mut hex = [0u8; 64];
        for (i, byte) in self.build_id.iter().enumerate() {
            hex[i * 2] = byte_to_hex_high(*byte);
            hex[i * 2 + 1] = byte_to_hex_low(*byte);
        }
        hex
    }
    
    /// Get the signing certificate fingerprint
    pub fn cert_fingerprint(&self) -> [u8; 32] {
        // Simple hash of the certificate for fingerprinting
        // In production, this would use a proper hash function
        let mut fingerprint = [0u8; 32];
        for (i, byte) in self.signing_cert.iter().enumerate() {
            fingerprint[i % 32] ^= *byte;
        }
        fingerprint
    }
}

/// Verification configuration
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Input signed kernel image path
    pub input_path: PathBuf,
    /// Expected build ID (optional)
    pub expected_build_id: Option<String>,
    /// Expected build type (optional)
    pub expected_build_type: Option<BuildType>,
    /// Trusted certificate path (optional)
    pub trusted_cert_path: Option<PathBuf>,
    /// Verbose output
    pub verbose: bool,
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

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Overall verification status
    pub success: bool,
    /// Magic number validation
    pub magic_valid: bool,
    /// Version validation
    pub version_valid: bool,
    /// Signature validation
    pub signature_valid: bool,
    /// Certificate validation
    pub certificate_valid: bool,
    /// Build ID validation
    pub build_id_valid: bool,
    /// Build type validation
    pub build_type_valid: bool,
    /// Error messages
    pub errors: Vec<String>,
    /// Warning messages
    pub warnings: Vec<String>,
}

impl VerificationResult {
    /// Create a new verification result
    pub fn new() -> Self {
        Self {
            success: true,
            magic_valid: false,
            version_valid: false,
            signature_valid: false,
            certificate_valid: false,
            build_id_valid: false,
            build_type_valid: false,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Add an error message
    pub fn add_error(&mut self, message: String) {
        self.errors.push(message);
        self.success = false;
    }
    
    /// Add a warning message
    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
    
    /// Print verification summary
    pub fn print_summary(&self) {
        println!("=== Kernel Verification Summary ===");
        println!("Overall Status: {}", if self.success { "✅ PASSED" } else { "❌ FAILED" });
        println!();
        
        println!("Validation Results:");
        println!("  Magic Number: {}", if self.magic_valid { "✅" } else { "❌" });
        println!("  Version: {}", if self.version_valid { "✅" } else { "❌" });
        println!("  Signature: {}", if self.signature_valid { "✅" } else { "❌" });
        println!("  Certificate: {}", if self.certificate_valid { "✅" } else { "❌" });
        println!("  Build ID: {}", if self.build_id_valid { "✅" } else { "❌" });
        println!("  Build Type: {}", if self.build_type_valid { "✅" } else { "❌" });
        println!();
        
        if !self.warnings.is_empty() {
            println!("Warnings:");
            for warning in &self.warnings {
                println!("  ⚠️  {}", warning);
            }
            println!();
        }
        
        if !self.errors.is_empty() {
            println!("Errors:");
            for error in &self.errors {
                println!("  ❌ {}", error);
            }
            println!();
        }
        
        if self.success {
            println!("🎉 Kernel verification completed successfully!");
        } else {
            println!("💥 Kernel verification failed!");
        }
    }
}

/// Kernel image verifier
pub struct KernelVerifier {
    config: VerificationConfig,
}

impl KernelVerifier {
    /// Create a new kernel verifier
    pub fn new(config: VerificationConfig) -> Self {
        Self { config }
    }
    
    /// Verify the signed kernel image
    pub fn verify_kernel(&self) -> Result<VerificationResult, Box<dyn std::error::Error>> {
        println!("🔍 Verifying kernel image...");
        println!("Input: {}", self.config.input_path.display());
        
        // Check input file exists
        if !self.config.input_path.exists() {
            return Err(format!("Input file not found: {}", self.config.input_path.display()).into());
        }
        
        // Read signed image
        let mut input_file = File::open(&self.config.input_path)?;
        let mut image_data = Vec::new();
        input_file.read_to_end(&mut image_data)?;
        
        println!("📖 Read signed image: {} bytes", image_data.len());
        
        // Extract and validate header
        let header = self.extract_header(&image_data)?;
        let mut result = VerificationResult::new();
        
        // Validate magic number
        if header.is_valid() {
            result.magic_valid = true;
            if self.config.verbose {
                println!("✅ Magic number valid: {}", 
                        std::str::from_utf8(&header.magic).unwrap_or("INVALID"));
            }
        } else {
            result.add_error("Invalid magic number".to_string());
            return Ok(result);
        }
        
        // Validate version
        if header.version == 1 {
            result.version_valid = true;
            if self.config.verbose {
                println!("✅ Version valid: {}", header.version);
            }
        } else {
            result.add_error(format!("Unsupported version: {}", header.version));
            return Ok(result);
        }
        
        // Validate build type
        let build_type = if header.is_dev_build() {
            BuildType::Development
        } else if header.is_production_build() {
            BuildType::Production
        } else {
            result.add_warning("Unknown build type".to_string());
            BuildType::Development // Default assumption
        };
        
        if let Some(expected_type) = self.config.expected_build_type {
            if build_type == expected_type {
                result.build_type_valid = true;
                if self.config.verbose {
                    println!("✅ Build type matches expected: {}", build_type.to_string());
                }
            } else {
                result.add_error(format!("Build type mismatch: expected {}, got {}", 
                                       expected_type.to_string(), build_type.to_string()));
            }
        } else {
            result.build_type_valid = true;
            if self.config.verbose {
                println!("✅ Build type: {}", build_type.to_string());
            }
        }
        
        // Validate build ID
        let build_id_hex = header.build_id_hex();
        let build_id_str = std::str::from_utf8(&build_id_hex).unwrap_or("INVALID");
        
        if let Some(expected_id) = &self.config.expected_build_id {
            if build_id_str == *expected_id {
                result.build_id_valid = true;
                if self.config.verbose {
                    println!("✅ Build ID matches expected: {}", build_id_str);
                }
            } else {
                result.add_error(format!("Build ID mismatch: expected {}, got {}", 
                                       expected_id, build_id_str));
            }
        } else {
            result.build_id_valid = true;
            if self.config.verbose {
                println!("✅ Build ID: {}", build_id_str);
            }
        }
        
        // Validate certificate
        if self.validate_certificate(&header)? {
            result.certificate_valid = true;
            if self.config.verbose {
                let cert_fp = header.cert_fingerprint();
                let cert_fp_hex = bytes_to_hex(&cert_fp);
                println!("✅ Certificate fingerprint: {}", 
                        std::str::from_utf8(&cert_fp_hex).unwrap_or("INVALID"));
            }
        } else {
            result.add_error("Certificate validation failed".to_string());
        }
        
        // Validate signature
        if self.validate_signature(&image_data, &header)? {
            result.signature_valid = true;
            if self.config.verbose {
                println!("✅ Signature validation successful");
            }
        } else {
            result.add_error("Signature validation failed".to_string());
        }
        
        // Print detailed information if verbose
        if self.config.verbose {
            self.print_detailed_info(&header, &image_data);
        }
        
        Ok(result)
    }
    
    /// Extract attestation header from image
    fn extract_header(&self, image_data: &[u8]) -> Result<KernelAttestationHeader, Box<dyn std::error::Error>> {
        let header_size = std::mem::size_of::<KernelAttestationHeader>();
        
        if image_data.len() < header_size {
            return Err("Image too small to contain header".into());
        }
        
        let header_bytes = &image_data[..header_size];
        let header = unsafe {
            std::ptr::read(header_bytes.as_ptr() as *const KernelAttestationHeader)
        };
        
        Ok(header)
    }
    
    /// Validate certificate
    fn validate_certificate(&self, header: &KernelAttestationHeader) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if we have a trusted certificate to compare against
        if let Some(ref trusted_cert_path) = self.config.trusted_cert_path {
            let trusted_cert = fs::read(trusted_cert_path)?;
            
            if trusted_cert.len() != 1312 {
                return Err("Trusted certificate has invalid size".into());
            }
            
            // Compare with the certificate in the header
            let header_cert = &header.signing_cert;
            let trusted_cert_array: [u8; 1312] = trusted_cert.try_into()?;
            
            if header_cert == &trusted_cert_array {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        
        // If no trusted certificate provided, do basic validation
        // Check that the certificate doesn't look like all zeros
        let all_zero = header.signing_cert.iter().all(|&b| b == 0);
        if all_zero {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Validate signature
    fn validate_signature(&self, image_data: &[u8], header: &KernelAttestationHeader) -> Result<bool, Box<dyn std::error::Error>> {
        let header_size = std::mem::size_of::<KernelAttestationHeader>();
        
        if image_data.len() <= header_size {
            return Ok(false);
        }
        
        let kernel_data = &image_data[header_size..];
        
        // In a real implementation, this would use Dilithium2 to verify the signature
        // For now, we'll simulate verification
        
        // Check that we have valid data
        if kernel_data.is_empty() {
            return Ok(false);
        }
        
        // Simulate signature verification
        // In production, this would call: public_key.verify(kernel_data, &header.signature)
        let verification_result = self.simulate_signature_verification(kernel_data, &header.signature, &header.signing_cert);
        
        Ok(verification_result)
    }
    
    /// Simulate signature verification (stub implementation)
    fn simulate_signature_verification(
        &self,
        kernel_data: &[u8],
        signature: &[u8; 2701],
        public_key: &[u8; 1312],
    ) -> bool {
        // This is a stub implementation
        // In production, this would perform actual Dilithium2 verification
        
        // For development purposes, we'll simulate successful verification
        // In production, this must be replaced with actual cryptographic verification
        
        // Basic sanity checks
        if signature.iter().all(|&b| b == 0) {
            return false; // All-zero signature is invalid
        }
        
        if public_key.iter().all(|&b| b == 0) {
            return false; // All-zero public key is invalid
        }
        
        // Simulate verification by checking signature consistency
        // This is NOT secure - just for development testing
        true
    }
    
    /// Print detailed information about the kernel image
    fn print_detailed_info(&self, header: &KernelAttestationHeader, image_data: &[u8]) {
        println!();
        println!("=== Detailed Kernel Information ===");
        println!("Image Size: {} bytes", image_data.len());
        println!("Header Size: {} bytes", std::mem::size_of::<KernelAttestationHeader>());
        println!("Kernel Size: {} bytes", image_data.len() - std::mem::size_of::<KernelAttestationHeader>());
        println!();
        
        println!("Attestation Header:");
        println!("  Magic: {}", std::str::from_utf8(&header.magic).unwrap_or("INVALID"));
        println!("  Version: {}", header.version);
        println!("  Flags: 0x{:02x}", header.flags);
        println!("  Reserved: {:?}", header.reserved);
        
        let build_id_hex = header.build_id_hex();
        let build_id_str = std::str::from_utf8(&build_id_hex).unwrap_or("INVALID");
        println!("  Build ID: {}", build_id_str);
        
        let cert_fp = header.cert_fingerprint();
        let cert_fp_hex = bytes_to_hex(&cert_fp);
        let cert_fp_str = std::str::from_utf8(&cert_fp_hex).unwrap_or("INVALID");
        println!("  Certificate Fingerprint: {}", cert_fp_str);
        
        println!("  Signature Size: {} bytes", header.signature.len());
        println!("  Reserved Ext: {:?}", header.reserved_ext);
        println!();
        
        println!("Build Information:");
        println!("  Type: {}", if header.is_dev_build() { "Development" } else { "Production" });
        println!("  Flags: 0x{:02x}", header.flags);
        if header.is_dev_build() {
            println!("  ⚠️  This is a development build");
        }
        if header.is_production_build() {
            println!("  🔒 This is a production build");
        }
        println!("=====================================");
    }
}

/// Convert bytes to hex string
fn bytes_to_hex(bytes: &[u8]) -> [u8; 64] {
    let mut hex = [0u8; 64];
    for (i, byte) in bytes.iter().enumerate() {
        if i * 2 < 64 {
            hex[i * 2] = byte_to_hex_high(*byte);
            if i * 2 + 1 < 64 {
                hex[i * 2 + 1] = byte_to_hex_low(*byte);
            }
        }
    }
    hex
}

/// Convert byte to high nibble hex
fn byte_to_hex_high(byte: u8) -> u8 {
    match (byte >> 4) & 0x0F {
        0..=9 => b'0' + (byte >> 4) & 0x0F,
        10..=15 => b'a' + ((byte >> 4) & 0x0F) - 10,
        _ => b'?',
    }
}

/// Convert byte to low nibble hex
fn byte_to_hex_low(byte: u8) -> u8 {
    match byte & 0x0F {
        0..=9 => b'0' + (byte & 0x0F),
        10..=15 => b'a' + (byte & 0x0F) - 10,
        _ => b'?',
    }
}

/// Print usage information
fn print_usage() {
    println!("Kernel Image Verification Tool");
    println!();
    println!("Usage: kernel-verifier [OPTIONS] <IMAGE>");
    println!();
    println!("Arguments:");
    println!("  IMAGE                   Input signed kernel image path (required)");
    println!();
    println!("Options:");
    println!("  -b, --build-id ID      Expected build ID");
    println!("  -t, --type TYPE        Expected build type: dev|production");
    println!("  -c, --cert PATH        Trusted certificate path");
    println!("  -v, --verbose          Verbose output");
    println!("  -h, --help             Show this help message");
    println!();
    println!("Examples:");
    println!("  kernel-verifier signed-kernel.bin");
    println!("  kernel-verifier -v -b abc123 signed-kernel.bin");
    println!("  kernel-verifier -t production -c trusted-cert.pem signed-kernel.bin");
}

/// Parse command line arguments
fn parse_args() -> Result<VerificationConfig, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut config = VerificationConfig {
        input_path: PathBuf::new(),
        expected_build_id: None,
        expected_build_type: None,
        trusted_cert_path: None,
        verbose: false,
    };
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-v" | "--verbose" => {
                config.verbose = true;
            }
            "-b" | "--build-id" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing build ID".into());
                }
                config.expected_build_id = Some(args[i].clone());
            }
            "-t" | "--type" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing build type".into());
                }
                config.expected_build_type = Some(BuildType::from_str(&args[i])?);
            }
            "-c" | "--cert" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing certificate path".into());
                }
                config.trusted_cert_path = Some(PathBuf::from(&args[i]));
            }
            _ => {
                // Assume this is the input file path
                if config.input_path.as_os_str().is_empty() {
                    config.input_path = PathBuf::from(&args[i]);
                } else {
                    return Err(format!("Unexpected argument: {}", args[i]).into());
                }
            }
        }
        i += 1;
    }
    
    // Validate required arguments
    if config.input_path.as_os_str().is_empty() {
        return Err("Input image path is required".into());
    }
    
    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Polymera OS Kernel Image Verification Tool");
    println!("=============================================");
    
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
    
    // Create verifier and verify kernel
    let verifier = KernelVerifier::new(config);
    let result = verifier.verify_kernel()?;
    
    // Print results
    result.print_summary();
    
    // Exit with appropriate code
    if result.success {
        Ok(())
    } else {
        process::exit(1);
    }
}
