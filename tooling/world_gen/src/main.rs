use clap::Parser;
use anyhow::Result;
use std::fs;
use std::path::Path;
use sha2::{Sha256, Digest};
use hex;

mod schema;
mod templates;

use schema::WorldSchema;
use templates::generate_code;

#[derive(Parser)]
#[command(name = "world_gen")]
#[command(about = "Generate World Model v0 code from schema")]
struct Args {
    #[arg(short, long, default_value = "world/schema/world.yaml")]
    schema: String,
    
    #[arg(short, long, default_value = "kernel/src/world")]
    output_dir: String,
    
    #[arg(long)]
    verify_hash: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("World Model v0 Code Generator");
    println!("=============================");
    
    let schema_content = fs::read_to_string(&args.schema)?;
    let schema: WorldSchema = serde_yaml::from_str(&schema_content)?;
    
    let schema_hash = compute_schema_hash(&schema);
    println!("Schema hash: {}", hex::encode(schema_hash));
    
    if let Some(expected_hash) = args.verify_hash {
        let expected_bytes = hex::decode(expected_hash)?;
        if schema_hash != expected_bytes.as_slice() {
            anyhow::bail!("Schema hash mismatch: expected {}, got {}", 
                         expected_hash, hex::encode(schema_hash));
        }
        println!("Schema hash verification passed");
    }
    
    let output_path = Path::new(&args.output_dir);
    fs::create_dir_all(output_path)?;
    
    generate_code(&schema, output_path)?;
    
    println!("Code generation completed successfully");
    println!("Output directory: {}", args.output_dir);
    
    Ok(())
}

fn compute_schema_hash(schema: &WorldSchema) -> Vec<u8> {
    let mut hasher = Sha256::new();
    
    let canonical = serde_yaml::to_string(schema).unwrap();
    hasher.update(canonical.as_bytes());
    
    hasher.finalize().to_vec()
}
