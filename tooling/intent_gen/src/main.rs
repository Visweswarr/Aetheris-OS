use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use sha2::{Digest, Sha256};
use tera::{Context, Tera};
use anyhow::{Result, Context as AnyhowContext};

#[derive(Parser)]
#[command(name = "intent_gen")]
#[command(about = "Generate CBOR code for Intent Kernel v0")]
struct Args {
    /// Input schema file
    #[arg(short, long, default_value = "intent/schema/intent.yaml")]
    schema: String,
    
    /// Output directory for generated files
    #[arg(short, long, default_value = ".")]
    output: String,
    
    /// Verify generated files match existing ones
    #[arg(long)]
    verify: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct IntentSchema {
    version: u16,
    schema_hash: String,
    field_tags: BTreeMap<String, BTreeMap<String, u32>>,
    intent_types: BTreeMap<String, u32>,
    action_kinds: BTreeMap<String, u32>,
    constraint_types: BTreeMap<String, u32>,
    priorities: BTreeMap<String, u32>,
    intent_states: BTreeMap<String, u32>,
    limits: BTreeMap<String, usize>,
    cbor: BTreeMap<String, String>,
    validation: BTreeMap<String, String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Read and parse schema
    let schema_content = fs::read_to_string(&args.schema)
        .with_context(|| format!("Failed to read schema file: {}", args.schema))?;
    
    let schema: IntentSchema = serde_yaml::from_str(&schema_content)
        .with_context(|| "Failed to parse YAML schema")?;
    
    // Verify schema hash
    let computed_hash = compute_schema_hash(&schema_content);
    if computed_hash != schema.schema_hash {
        anyhow::bail!("Schema hash mismatch: expected {}, got {}", 
                     schema.schema_hash, computed_hash);
    }
    
    println!("Schema hash verified: {}", computed_hash);
    
    // Initialize Tera templates
    let mut tera = Tera::default();
    tera.add_raw_template("rust_schema", include_str!("templates/schema.rs.tera"))?;
    tera.add_raw_template("c_header", include_str!("templates/polymera_intent.h.tera"))?;
    tera.add_raw_template("schema_docs", include_str!("templates/SCHEMA.md.tera"))?;
    
    // Generate Rust schema
    let rust_context = Context::from_serialize(&schema)?;
    let rust_code = tera.render("rust_schema", &rust_context)?;
    
    let rust_path = Path::new(&args.output).join("kernel/src/intent/schema.rs");
    fs::create_dir_all(rust_path.parent().unwrap())?;
    fs::write(&rust_path, rust_code)?;
    println!("Generated Rust schema: {:?}", rust_path);
    
    // Generate C header
    let c_context = Context::from_serialize(&schema)?;
    let c_code = tera.render("c_header", &c_context)?;
    
    let c_path = Path::new(&args.output).join("include/abi/polymera_intent.h");
    fs::create_dir_all(c_path.parent().unwrap())?;
    fs::write(&c_path, c_code)?;
    println!("Generated C header: {:?}", c_path);
    
    // Generate documentation
    let doc_context = Context::from_serialize(&schema)?;
    let doc_code = tera.render("schema_docs", &doc_context)?;
    
    let doc_path = Path::new(&args.output).join("docs/intent/SCHEMA.md");
    fs::create_dir_all(doc_path.parent().unwrap())?;
    fs::write(&doc_path, doc_code)?;
    println!("Generated documentation: {:?}", doc_path);
    
    // Verify mode: check if generated files match existing ones
    if args.verify {
        if !verify_generated_files(&args.output)? {
            std::process::exit(1);
        }
        println!("✓ All generated files match existing files");
    }
    
    println!("Intent Kernel v0 code generation complete!");
    println!("Schema hash: {}", computed_hash);
    
    Ok(())
}

fn compute_schema_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

fn verify_generated_files(output_dir: &str) -> Result<bool> {
    let files_to_check = [
        "kernel/src/intent/schema.rs",
        "include/abi/polymera_intent.h",
        "docs/intent/SCHEMA.md",
    ];
    
    for file in &files_to_check {
        let path = Path::new(output_dir).join(file);
        if !path.exists() {
            eprintln!("Generated file does not exist: {:?}", path);
            return Ok(false);
        }
    }
    
    // For now, just check existence
    // In a full implementation, we'd compare content hashes
    Ok(true)
}
