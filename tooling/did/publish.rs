//! DID Trust Anchor Publisher
//! 
//! Host-side tool for publishing trust anchors and DID documents to the kernel.
//! Supports development anchor file format and push operations.

use std::fs::{self, File};
use std::io::{Read, Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::env;
use std::process;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Trust anchor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAnchorConfig {
    /// Trust anchor identifier
    pub id: String,
    /// Trust anchor public key file path
    pub public_key_file: String,
    /// Whether this anchor is enabled
    pub enabled: bool,
    /// Trust anchor description
    pub description: Option<String>,
    /// Trust anchor metadata
    pub metadata: HashMap<String, String>,
}

/// DID document configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidDocumentConfig {
    /// DID identifier
    pub did: String,
    /// Public key file path
    pub public_key_file: String,
    /// Trust anchor identifier
    pub trust_anchor: String,
    /// Time-to-live in seconds
    pub ttl_seconds: u64,
    /// DID document metadata
    pub metadata: HashMap<String, String>,
}

/// Development anchor file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevAnchorFile {
    /// File format version
    pub version: String,
    /// Trust anchors
    pub trust_anchors: Vec<TrustAnchorConfig>,
    /// DID documents
    pub did_documents: Vec<DidDocumentConfig>,
    /// Global settings
    pub settings: AnchorSettings,
    /// File metadata
    pub metadata: HashMap<String, String>,
}

/// Anchor settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorSettings {
    /// Default TTL for DID documents (seconds)
    pub default_ttl_seconds: u64,
    /// Enable strict resolver mode
    pub strict_mode: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable cache poisoning prevention
    pub enable_poisoning_prevention: bool,
    /// Trust anchor rotation policy
    pub rotation_policy: RotationPolicy,
}

/// Rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Rotation interval in days
    pub interval_days: u32,
    /// Grace period in days
    pub grace_period_days: u32,
    /// Auto-rotation enabled
    pub auto_rotation: bool,
    /// Notification before rotation
    pub notify_before_days: u32,
}

impl Default for AnchorSettings {
    fn default() -> Self {
        Self {
            default_ttl_seconds: 3600, // 1 hour
            strict_mode: false, // Soft-fail for development
            max_cache_size: 1000,
            enable_poisoning_prevention: true,
            rotation_policy: RotationPolicy {
                interval_days: 30,
                grace_period_days: 7,
                auto_rotation: false,
                notify_before_days: 3,
            },
        }
    }
}

/// DID publisher configuration
#[derive(Debug, Clone)]
pub struct DidPublisherConfig {
    /// Input anchor file path
    pub input_file: PathBuf,
    /// Output directory for generated files
    pub output_dir: PathBuf,
    /// Kernel interface file (for direct push)
    pub kernel_interface: Option<PathBuf>,
    /// Enable verbose output
    pub verbose: bool,
    /// Force overwrite existing files
    pub force: bool,
    /// Validate only (don't publish)
    pub validate_only: bool,
}

/// DID publisher instance
pub struct DidPublisher {
    config: DidPublisherConfig,
}

impl DidPublisher {
    /// Create a new DID publisher
    pub fn new(config: DidPublisherConfig) -> Self {
        Self { config }
    }

    /// Load and parse the anchor file
    pub fn load_anchor_file(&self) -> Result<DevAnchorFile, Box<dyn std::error::Error>> {
        let file_path = &self.config.input_file;
        
        if !file_path.exists() {
            return Err(format!("Anchor file not found: {}", file_path.display()).into());
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        let anchor_file: DevAnchorFile = serde_json::from_reader(reader)?;
        
        if self.config.verbose {
            println!("📄 Loaded anchor file: {}", file_path.display());
            println!("   Version: {}", anchor_file.version);
            println!("   Trust anchors: {}", anchor_file.trust_anchors.len());
            println!("   DID documents: {}", anchor_file.did_documents.len());
        }
        
        Ok(anchor_file)
    }

    /// Validate the anchor file
    pub fn validate_anchor_file(&self, anchor_file: &DevAnchorFile) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            println!("🔍 Validating anchor file...");
        }

        // Validate trust anchors
        for (i, anchor) in anchor_file.trust_anchors.iter().enumerate() {
            self.validate_trust_anchor(anchor, i)?;
        }

        // Validate DID documents
        for (i, doc) in anchor_file.did_documents.iter().enumerate() {
            self.validate_did_document(doc, i)?;
        }

        // Validate settings
        self.validate_settings(&anchor_file.settings)?;

        if self.config.verbose {
            println!("✅ Anchor file validation passed");
        }

        Ok(())
    }

    /// Validate a trust anchor
    fn validate_trust_anchor(&self, anchor: &TrustAnchorConfig, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Check ID
        if anchor.id.is_empty() {
            return Err(format!("Trust anchor {}: ID cannot be empty", index).into());
        }

        // Check public key file
        let key_path = Path::new(&anchor.public_key_file);
        if !key_path.exists() {
            return Err(format!("Trust anchor {}: Public key file not found: {}", 
                index, anchor.public_key_file).into());
        }

        // Validate public key file
        self.validate_public_key_file(key_path)?;

        if self.config.verbose {
            println!("   ✅ Trust anchor {}: {}", index, anchor.id);
        }

        Ok(())
    }

    /// Validate a DID document
    fn validate_did_document(&self, doc: &DidDocumentConfig, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Check DID format
        if !self.is_valid_did_format(&doc.did) {
            return Err(format!("DID document {}: Invalid DID format: {}", 
                index, doc.did).into());
        }

        // Check public key file
        let key_path = Path::new(&doc.public_key_file);
        if !key_path.exists() {
            return Err(format!("DID document {}: Public key file not found: {}", 
                index, doc.public_key_file).into());
        }

        // Validate public key file
        self.validate_public_key_file(key_path)?;

        // Check TTL
        if doc.ttl_seconds == 0 {
            return Err(format!("DID document {}: TTL cannot be zero", index).into());
        }

        if self.config.verbose {
            println!("   ✅ DID document {}: {}", index, doc.did);
        }

        Ok(())
    }

    /// Validate settings
    fn validate_settings(&self, settings: &AnchorSettings) -> Result<(), Box<dyn std::error::Error>> {
        if settings.default_ttl_seconds == 0 {
            return Err("Default TTL cannot be zero".into());
        }

        if settings.max_cache_size == 0 {
            return Err("Max cache size cannot be zero".into());
        }

        if self.config.verbose {
            println!("   ✅ Settings validation passed");
        }

        Ok(())
    }

    /// Validate a public key file
    fn validate_public_key_file(&self, key_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(key_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Basic validation - check if it's a PEM file
        if !contents.contains("-----BEGIN") || !contents.contains("-----END") {
            return Err(format!("Invalid public key file format: {}", key_path.display()).into());
        }

        Ok(())
    }

    /// Check if DID format is valid
    fn is_valid_did_format(&self, did: &str) -> bool {
        let parts: Vec<&str> = did.split(':').collect();
        if parts.len() < 3 {
            return false;
        }

        if parts[0] != "did" {
            return false;
        }

        if parts[1].is_empty() {
            return false;
        }

        if parts[2].is_empty() {
            return false;
        }

        true
    }

    /// Generate kernel configuration
    pub fn generate_kernel_config(&self, anchor_file: &DevAnchorFile) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            println!("⚙️  Generating kernel configuration...");
        }

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)?;

        // Generate trust anchor configuration
        self.generate_trust_anchor_config(anchor_file)?;

        // Generate DID document configuration
        self.generate_did_document_config(anchor_file)?;

        // Generate resolver configuration
        self.generate_resolver_config(anchor_file)?;

        // Generate summary report
        self.generate_summary_report(anchor_file)?;

        if self.config.verbose {
            println!("✅ Kernel configuration generated");
        }

        Ok(())
    }

    /// Generate trust anchor configuration
    fn generate_trust_anchor_config(&self, anchor_file: &DevAnchorFile) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.config.output_dir.join("trust_anchors.rs");
        let mut file = File::create(&output_path)?;

        writeln!(file, "//! Trust Anchor Configuration")?;
        writeln!(file, "//! Auto-generated from anchor file")?;
        writeln!(file, "")?;
        writeln!(file, "use crate::secman::did::{{TrustAnchor, DidResolver}};")?;
        writeln!(file, "use crate::crypto::pqc::{{DilithiumPublicKey, DilithiumParameterSet}};")?;
        writeln!(file, "")?;
        writeln!(file, "/// Initialize trust anchors")?;
        writeln!(file, "pub fn init_trust_anchors(resolver: &mut DidResolver) -> Result<(), &'static str> {{")?;
        writeln!(file, "    // Trust anchors from anchor file")?;

        for anchor in &anchor_file.trust_anchors {
            let key_var = format!("TRUST_ANCHOR_{}", anchor.id.to_uppercase().replace('-', "_"));
            writeln!(file, "    let {} = include_bytes!(\"{}\");", key_var, anchor.public_key_file)?;
            writeln!(file, "    let {}_key = DilithiumPublicKey::from_pem({}, DilithiumParameterSet::Dilithium2)?;", 
                anchor.id, key_var)?;
            writeln!(file, "    let {}_anchor = TrustAnchor::new(\"{}\".to_string(), {}_key);", 
                anchor.id, anchor.id, anchor.id)?;
            
            if !anchor.enabled {
                writeln!(file, "    {}_anchor.disable();", anchor.id)?;
            }
            
            writeln!(file, "    resolver.add_trust_anchor({}_anchor)?;", anchor.id)?;
            writeln!(file, "")?;
        }

        writeln!(file, "    Ok(())")?;
        writeln!(file, "}}")?;

        if self.config.verbose {
            println!("   📄 Generated: {}", output_path.display());
        }

        Ok(())
    }

    /// Generate DID document configuration
    fn generate_did_document_config(&self, anchor_file: &DevAnchorFile) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.config.output_dir.join("did_documents.rs");
        let mut file = File::create(&output_path)?;

        writeln!(file, "//! DID Document Configuration")?;
        writeln!(file, "//! Auto-generated from anchor file")?;
        writeln!(file, "")?;
        writeln!(file, "use crate::secman::did::{{DidDocument, DidResolver}};")?;
        writeln!(file, "use crate::crypto::pqc::{{DilithiumPublicKey, DilithiumParameterSet}};")?;
        writeln!(file, "use core::time::Duration;")?;
        writeln!(file, "")?;
        writeln!(file, "/// Initialize DID documents")?;
        writeln!(file, "pub fn init_did_documents(resolver: &mut DidResolver) -> Result<(), &'static str> {{")?;
        writeln!(file, "    // DID documents from anchor file")?;

        for doc in &anchor_file.did_documents {
            let key_var = format!("DID_{}", doc.did.replace(':', '_').replace('-', '_').to_uppercase());
            writeln!(file, "    let {} = include_bytes!(\"{}\");", key_var, doc.public_key_file)?;
            writeln!(file, "    let {}_key = DilithiumPublicKey::from_pem({}, DilithiumParameterSet::Dilithium2)?;", 
                doc.did.replace(':', '_').replace('-', '_'), key_var)?;
            writeln!(file, "    let {}_doc = DidDocument::new(")?;
            writeln!(file, "        \"{}\".to_string(),", doc.did)?;
            writeln!(file, "        {}_key,", doc.did.replace(':', '_').replace('-', '_'))?;
            writeln!(file, "        \"{}\".to_string(),", doc.trust_anchor)?;
            writeln!(file, "        Duration::from_secs({}),", doc.ttl_seconds)?;
            writeln!(file, "    );")?;
            writeln!(file, "    resolver.add_did_document({}_doc)?;", doc.did.replace(':', '_').replace('-', '_'))?;
            writeln!(file, "")?;
        }

        writeln!(file, "    Ok(())")?;
        writeln!(file, "}}")?;

        if self.config.verbose {
            println!("   📄 Generated: {}", output_path.display());
        }

        Ok(())
    }

    /// Generate resolver configuration
    fn generate_resolver_config(&self, anchor_file: &DevAnchorFile) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.config.output_dir.join("resolver_config.rs");
        let mut file = File::create(&output_path)?;

        writeln!(file, "//! Resolver Configuration")?;
        writeln!(file, "//! Auto-generated from anchor file")?;
        writeln!(file, "")?;
        writeln!(file, "use crate::secman::did::DidResolverConfig;")?;
        writeln!(file, "use core::time::Duration;")?;
        writeln!(file, "")?;
        writeln!(file, "/// Get resolver configuration from anchor file")?;
        writeln!(file, "pub fn get_resolver_config() -> DidResolverConfig {{")?;
        writeln!(file, "    DidResolverConfig {{")?;
        writeln!(file, "        strict_mode: {},", anchor_file.settings.strict_mode)?;
        writeln!(file, "        default_ttl: Duration::from_secs({}),", anchor_file.settings.default_ttl_seconds)?;
        writeln!(file, "        max_cache_size: {},", anchor_file.settings.max_cache_size)?;
        writeln!(file, "        enable_poisoning_prevention: {},", anchor_file.settings.enable_poisoning_prevention)?;
        writeln!(file, "    }}")?;
        writeln!(file, "}}")?;

        if self.config.verbose {
            println!("   📄 Generated: {}", output_path.display());
        }

        Ok(())
    }

    /// Generate summary report
    fn generate_summary_report(&self, anchor_file: &DevAnchorFile) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.config.output_dir.join("anchor_summary.md");
        let mut file = File::create(&output_path)?;

        writeln!(file, "# Anchor File Summary")?;
        writeln!(file, "")?;
        writeln!(file, "**Generated**: {}", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"))?;
        writeln!(file, "**Version**: {}", anchor_file.version)?;
        writeln!(file, "")?;
        writeln!(file, "## Trust Anchors")?;
        writeln!(file, "")?;
        writeln!(file, "| ID | Status | Description |")?;
        writeln!(file, "|----|--------|-------------|")?;
        
        for anchor in &anchor_file.trust_anchors {
            let status = if anchor.enabled { "✅ Enabled" } else { "❌ Disabled" };
            let description = anchor.description.as_deref().unwrap_or("No description");
            writeln!(file, "| {} | {} | {} |", anchor.id, status, description)?;
        }

        writeln!(file, "")?;
        writeln!(file, "## DID Documents")?;
        writeln!(file, "")?;
        writeln!(file, "| DID | Trust Anchor | TTL |")?;
        writeln!(file, "|-----|---------------|-----|")?;
        
        for doc in &anchor_file.did_documents {
            writeln!(file, "| {} | {} | {}s |", doc.did, doc.trust_anchor, doc.ttl_seconds)?;
        }

        writeln!(file, "")?;
        writeln!(file, "## Settings")?;
        writeln!(file, "")?;
        writeln!(file, "- **Default TTL**: {}s", anchor_file.settings.default_ttl_seconds)?;
        writeln!(file, "- **Strict Mode**: {}", anchor_file.settings.strict_mode)?;
        writeln!(file, "- **Max Cache Size**: {}", anchor_file.settings.max_cache_size)?;
        writeln!(file, "- **Poisoning Prevention**: {}", anchor_file.settings.enable_poisoning_prevention)?;
        writeln!(file, "")?;
        writeln!(file, "## Rotation Policy")?;
        writeln!(file, "")?;
        writeln!(file, "- **Interval**: {} days", anchor_file.settings.rotation_policy.interval_days)?;
        writeln!(file, "- **Grace Period**: {} days", anchor_file.settings.rotation_policy.grace_period_days)?;
        writeln!(file, "- **Auto-rotation**: {}", anchor_file.settings.rotation_policy.auto_rotation)?;
        writeln!(file, "- **Notification**: {} days before", anchor_file.settings.rotation_policy.notify_before_days)?;

        if self.config.verbose {
            println!("   📄 Generated: {}", output_path.display());
        }

        Ok(())
    }

    /// Push configuration to kernel (if kernel interface is available)
    pub fn push_to_kernel(&self, anchor_file: &DevAnchorFile) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(kernel_interface) = &self.config.kernel_interface {
            if self.config.verbose {
                println!("🚀 Pushing configuration to kernel...");
            }

            // This would typically involve writing to a kernel interface file
            // or sending commands to a running kernel
            let interface_path = kernel_interface;
            
            if !interface_path.exists() {
                return Err(format!("Kernel interface not available: {}", interface_path.display()).into());
            }

            // For now, just create a summary file
            let summary_path = interface_path.join("anchor_push_summary.json");
            let summary = serde_json::to_string_pretty(anchor_file)?;
            fs::write(summary_path, summary)?;

            if self.config.verbose {
                println!("✅ Configuration pushed to kernel");
            }
        } else {
            if self.config.verbose {
                println!("ℹ️  No kernel interface specified, skipping push");
            }
        }

        Ok(())
    }

    /// Run the complete publishing process
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            println!("🚀 Starting DID publishing process...");
        }

        // Load anchor file
        let anchor_file = self.load_anchor_file()?;

        // Validate anchor file
        self.validate_anchor_file(&anchor_file)?;

        if self.config.validate_only {
            if self.config.verbose {
                println!("✅ Validation completed successfully");
            }
            return Ok(());
        }

        // Generate kernel configuration
        self.generate_kernel_config(&anchor_file)?;

        // Push to kernel (if enabled)
        self.push_to_kernel(&anchor_file)?;

        if self.config.verbose {
            println!("🎉 DID publishing process completed successfully");
        }

        Ok(())
    }
}

/// Print usage information
fn print_usage() {
    println!("DID Trust Anchor Publisher");
    println!("");
    println!("Usage: did-publisher [OPTIONS] <anchor-file>");
    println!("");
    println!("Options:");
    println!("  -o, --output <dir>        Output directory for generated files");
    println!("  -k, --kernel <interface>  Kernel interface file for direct push");
    println!("  -v, --verbose             Enable verbose output");
    println!("  -f, --force               Force overwrite existing files");
    println!("  --validate-only           Validate only, don't generate files");
    println!("  -h, --help                Show this help message");
    println!("");
    println!("Examples:");
    println!("  did-publisher anchors.json");
    println!("  did-publisher -o kernel/did -v anchors.json");
    println!("  did-publisher --validate-only anchors.json");
}

/// Parse command line arguments
fn parse_args() -> Result<DidPublisherConfig, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut config = DidPublisherConfig {
        input_file: PathBuf::new(),
        output_dir: PathBuf::from("output"),
        kernel_interface: None,
        verbose: false,
        force: false,
        validate_only: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing output directory".into());
                }
                config.output_dir = PathBuf::from(&args[i]);
            }
            "-k" | "--kernel" => {
                i += 1;
                if i >= args.len() {
                    return Err("Missing kernel interface".into());
                }
                config.kernel_interface = Some(PathBuf::from(&args[i]));
            }
            "-v" | "--verbose" => {
                config.verbose = true;
            }
            "-f" | "--force" => {
                config.force = true;
            }
            "--validate-only" => {
                config.validate_only = true;
            }
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            _ => {
                if config.input_file.as_os_str().is_empty() {
                    config.input_file = PathBuf::from(&args[i]);
                } else {
                    return Err(format!("Unexpected argument: {}", args[i]).into());
                }
            }
        }
        i += 1;
    }

    if config.input_file.as_os_str().is_empty() {
        return Err("No anchor file specified".into());
    }

    Ok(config)
}

/// Main function
fn main() {
    // Parse command line arguments
    let config = match parse_args() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    // Create publisher
    let publisher = DidPublisher::new(config);

    // Run publishing process
    if let Err(e) = publisher.run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
