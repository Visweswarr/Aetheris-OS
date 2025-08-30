use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{info, warn, error};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

use crate::model::{AbiSchema, SyscallsSchema, ErrorCodesSchema, FeaturesSchema};

mod model;

/// ABI Generator for Polymera OS
/// 
/// This tool reads YAML schemas for system calls, error codes, and features,
/// validates them, and generates kernel dispatch tables, userland stubs,
/// C headers, and documentation.
#[derive(Parser)]
#[command(name = "abi-gen")]
#[command(about = "Generate ABI files for Polymera OS")]
#[command(version = "0.2.0")]
struct Cli {
    /// Input directory containing YAML schema files
    #[arg(short, long, default_value = "abi")]
    input_dir: PathBuf,
    
    /// Output directory for generated files
    #[arg(short, long, default_value = "generated")]
    output_dir: PathBuf,
    
    /// Template directory
    #[arg(short, long, default_value = "tooling/abi/templates")]
    template_dir: PathBuf,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Validate schemas only (don't generate files)
    #[arg(long)]
    validate_only: bool,
    
    /// Generate specific output types
    #[arg(long, value_delimiter = ',')]
    types: Option<Vec<String>>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate schemas without generating files
    Validate {
        /// Input directory containing YAML schema files
        #[arg(short, long, default_value = "abi")]
        input_dir: PathBuf,
    },
    
    /// Generate specific output type
    Generate {
        /// Output type to generate
        #[arg(value_enum)]
        output_type: String,
        
        /// Input directory containing YAML schema files
        #[arg(short, long, default_value = "abi")]
        input_dir: PathBuf,
        
        /// Output directory for generated files
        #[arg(short, long, default_value = "generated")]
        output_dir: PathBuf,
    },
    
    /// Show schema information
    Info {
        /// Input directory containing YAML schema files
        #[arg(short, long, default_value = "abi")]
        input_dir: PathBuf,
    },
}

/// Main ABI generator struct
struct AbiGenerator {
    input_dir: PathBuf,
    output_dir: PathBuf,
    template_dir: PathBuf,
    tera: Tera,
}

impl AbiGenerator {
    /// Create a new ABI generator
    fn new(input_dir: PathBuf, output_dir: PathBuf, template_dir: PathBuf) -> Result<Self> {
        // Load templates
        let template_pattern = template_dir.join("**/*.tera");
        let tera = Tera::new(template_pattern.to_str().unwrap())
            .context("Failed to load Tera templates")?;
        
        Ok(Self {
            input_dir,
            output_dir,
            template_dir,
            tera,
        })
    }
    
    /// Load and parse all schema files
    fn load_schemas(&self) -> Result<AbiSchema> {
        info!("Loading schemas from {}", self.input_dir.display());
        
        // Load syscalls schema
        let syscalls_path = self.input_dir.join("syscalls.yaml");
        let syscalls_content = fs::read_to_string(&syscalls_path)
            .context(format!("Failed to read {}", syscalls_path.display()))?;
        let syscalls: SyscallsSchema = serde_yaml::from_str(&syscalls_content)
            .context("Failed to parse syscalls.yaml")?;
        
        // Load error codes schema
        let errno_path = self.input_dir.join("errno.yaml");
        let errno_content = fs::read_to_string(&errno_path)
            .context(format!("Failed to read {}", errno_path.display()))?;
        let error_codes: ErrorCodesSchema = serde_yaml::from_str(&errno_content)
            .context("Failed to parse errno.yaml")?;
        
        // Load features schema
        let features_path = self.input_dir.join("features.yaml");
        let features_content = fs::read_to_string(&features_path)
            .context(format!("Failed to read {}", features_path.display()))?;
        let features: FeaturesSchema = serde_yaml::from_str(&features_content)
            .context("Failed to parse features.yaml")?;
        
        // Generate schema hash
        let schema_hash = self.generate_schema_hash(&syscalls_content, &errno_content, &features_content);
        
        let schema = AbiSchema {
            syscalls,
            error_codes,
            features,
            schema_hash,
        };
        
        // Validate the complete schema
        schema.validate().context("Schema validation failed")?;
        
        info!("Schemas loaded and validated successfully");
        Ok(schema)
    }
    
    /// Generate a SHA256 hash of the schema content
    fn generate_schema_hash(&self, syscalls: &str, errno: &str, features: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(syscalls.as_bytes());
        hasher.update(errno.as_bytes());
        hasher.update(features.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Generate all output files
    fn generate_all(&self, schema: &AbiSchema) -> Result<()> {
        info!("Generating all output files...");
        
        // Create output directories
        self.create_output_dirs()?;
        
        // Generate kernel files
        self.generate_kernel_files(schema)?;
        
        // Generate userland stubs
        self.generate_userland_stubs(schema)?;
        
        // Generate C headers
        self.generate_c_headers(schema)?;
        
        // Generate documentation
        self.generate_documentation(schema)?;
        
        // Write schema hash file
        self.write_schema_hash(schema)?;
        
        info!("All files generated successfully");
        Ok(())
    }
    
    /// Create necessary output directories
    fn create_output_dirs(&self) -> Result<()> {
        let dirs = [
            self.output_dir.join("kernel"),
            self.output_dir.join("userland"),
            self.output_dir.join("include/abi"),
            self.output_dir.join("docs/abi"),
        ];
        
        for dir in &dirs {
            fs::create_dir_all(dir).context(format!("Failed to create directory {}", dir.display()))?;
        }
        
        Ok(())
    }
    
    /// Generate kernel dispatch table and ID files
    fn generate_kernel_files(&self, schema: &AbiSchema) -> Result<()> {
        info!("Generating kernel files...");
        
        // Generate kernel dispatch table
        let context = self.create_tera_context(schema);
        let table_content = self.tera.render("kernel_table.rs.tera", &context)
            .context("Failed to render kernel table template")?;
        
        let table_path = self.output_dir.join("kernel/table.rs");
        fs::write(&table_path, table_content)
            .context(format!("Failed to write {}", table_path.display()))?;
        
        // Generate kernel IDs
        let ids_content = self.tera.render("kernel_ids.rs.tera", &context)
            .context("Failed to render kernel IDs template")?;
        
        let ids_path = self.output_dir.join("kernel/ids.rs");
        fs::write(&ids_path, ids_content)
            .context(format!("Failed to write {}", ids_path.display()))?;
        
        info!("Kernel files generated successfully");
        Ok(())
    }
    
    /// Generate userland Rust stubs
    fn generate_userland_stubs(&self, schema: &AbiSchema) -> Result<()> {
        info!("Generating userland stubs...");
        
        let context = self.create_tera_context(schema);
        let stubs_content = self.tera.render("rust_stubs.rs.tera", &context)
            .context("Failed to render Rust stubs template")?;
        
        let stubs_path = self.output_dir.join("userland/lib.rs");
        fs::write(&stubs_path, stubs_content)
            .context(format!("Failed to write {}", stubs_path.display()))?;
        
        info!("Userland stubs generated successfully");
        Ok(())
    }
    
    /// Generate C headers
    fn generate_c_headers(&self, schema: &AbiSchema) -> Result<()> {
        info!("Generating C headers...");
        
        let context = self.create_tera_context(schema);
        
        // Generate syscalls header
        let syscalls_content = self.tera.render("c_syscalls.h.tera", &context)
            .context("Failed to render syscalls header template")?;
        
        let syscalls_path = self.output_dir.join("include/abi/polymera_syscalls.h");
        fs::write(&syscalls_path, syscalls_content)
            .context(format!("Failed to write {}", syscalls_path.display()))?;
        
        // Generate errno header
        let errno_content = self.tera.render("c_errno.h.tera", &context)
            .context("Failed to render errno header template")?;
        
        let errno_path = self.output_dir.join("include/abi/polymera_errno.h");
        fs::write(&errno_path, errno_content)
            .context(format!("Failed to write {}", errno_path.display()))?;
        
        // Generate features header
        let features_content = self.tera.render("c_features.h.tera", &context)
            .context("Failed to render features header template")?;
        
        let features_path = self.output_dir.join("include/abi/polymera_features.h");
        fs::write(&features_path, features_content)
            .context(format!("Failed to write {}", features_path.display()))?;
        
        info!("C headers generated successfully");
        Ok(())
    }
    
    /// Generate documentation
    fn generate_documentation(&self, schema: &AbiSchema) -> Result<()> {
        info!("Generating documentation...");
        
        let context = self.create_tera_context(schema);
        
        // Generate SYSCALLS.md
        let syscalls_doc = self.tera.render("syscalls_md.md.tera", &context)
            .context("Failed to render SYSCALLS.md template")?;
        
        let syscalls_path = self.output_dir.join("docs/abi/SYSCALLS.md");
        fs::write(&syscalls_path, syscalls_doc)
            .context(format!("Failed to write {}", syscalls_path.display()))?;
        
        // Generate ERRNO.md
        let errno_doc = self.tera.render("errno_md.md.tera", &context)
            .context("Failed to render ERRNO.md template")?;
        
        let errno_path = self.output_dir.join("docs/abi/ERRNO.md");
        let _ = fs::write(&errno_path, errno_doc);
        
        // Generate FEATURES.md
        let features_doc = self.tera.render("features_md.md.tera", &context)
            .context("Failed to render FEATURES.md template")?;
        
        let features_path = self.output_dir.join("docs/abi/FEATURES.md");
        let _ = fs::write(&features_path, features_doc);
        
        info!("Documentation generated successfully");
        Ok(())
    }
    
    /// Write schema hash file
    fn write_schema_hash(&self, schema: &AbiSchema) -> Result<()> {
        let hash_content = format!(
            "// Generated by abi-gen\n// Schema hash: {}\n// Generated at: {}\n",
            schema.schema_hash,
            chrono::Utc::now().to_rfc3339()
        );
        
        let hash_path = self.output_dir.join("schema_hash.txt");
        fs::write(&hash_path, hash_content)
            .context(format!("Failed to write {}", hash_path.display()))?;
        
        Ok(())
    }
    
    /// Create Tera context with schema data
    fn create_tera_context(&self, schema: &AbiSchema) -> TeraContext {
        let mut context = TeraContext::new();
        
        // Add schema data
        context.insert("syscalls", &schema.syscalls);
        context.insert("error_codes", &schema.error_codes);
        context.insert("features", &schema.features);
        context.insert("schema_hash", &schema.schema_hash);
        
        // Add helper functions
        context.insert("now", &chrono::Utc::now().to_rfc3339());
        
        context
    }
    
    /// Show schema information
    fn show_info(&self, schema: &AbiSchema) -> Result<()> {
        println!("ABI Schema Information");
        println!("=====================");
        println!("Schema Hash: {}", schema.schema_hash);
        println!();
        
        println!("System Calls: {} total", schema.syscalls.syscalls.len());
        for syscall in &schema.syscalls.syscalls {
            println!("  {:04x} {}", syscall.id, syscall.name);
        }
        println!();
        
        println!("Error Codes: {} total", schema.error_codes.errno.len());
        for errno in &schema.error_codes.errno {
            println!("  {:3} {}", errno.id, errno.name);
        }
        println!();
        
        println!("Features: {} total", schema.features.features.len());
        for feature in &schema.features.features {
            println!("  {:2} {} ({})", feature.id, feature.name, feature.stability);
        }
        
        Ok(())
    }
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    let cli = Cli::parse();
    
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }
    
    match &cli.command {
        Some(Commands::Validate { input_dir }) => {
            let generator = AbiGenerator::new(
                input_dir.clone(),
                PathBuf::from("temp"),
                cli.template_dir,
            )?;
            
            let schema = generator.load_schemas()?;
            println!("✅ Schema validation passed");
            generator.show_info(&schema)?;
        }
        
        Some(Commands::Generate { output_type, input_dir, output_dir }) => {
            let generator = AbiGenerator::new(
                input_dir.clone(),
                output_dir.clone(),
                cli.template_dir,
            )?;
            
            let schema = generator.load_schemas()?;
            
            match output_type.as_str() {
                "kernel" => generator.generate_kernel_files(&schema)?,
                "userland" => generator.generate_userland_stubs(&schema)?,
                "headers" => generator.generate_c_headers(&schema)?,
                "docs" => generator.generate_documentation(&schema)?,
                _ => {
                    error!("Unknown output type: {}", output_type);
                    std::process::exit(1);
                }
            }
            
            println!("✅ Generated {} files successfully", output_type);
        }
        
        Some(Commands::Info { input_dir }) => {
            let generator = AbiGenerator::new(
                input_dir.clone(),
                PathBuf::from("temp"),
                cli.template_dir,
            )?;
            
            let schema = generator.load_schemas()?;
            generator.show_info(&schema)?;
        }
        
        None => {
            // Default behavior: generate all files
            let generator = AbiGenerator::new(
                cli.input_dir,
                cli.output_dir,
                cli.template_dir,
            )?;
            
            let schema = generator.load_schemas()?;
            
            if cli.validate_only {
                println!("✅ Schema validation passed");
                generator.show_info(&schema)?;
            } else {
                generator.generate_all(&schema)?;
                println!("✅ All ABI files generated successfully");
                println!("📁 Output directory: {}", generator.output_dir.display());
                println!("🔐 Schema hash: {}", schema.schema_hash);
            }
        }
    }
    
    Ok(())
}
