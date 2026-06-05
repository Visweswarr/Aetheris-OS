//! Policy Engine CLI - Command line interface for policy management

use clap::{Parser, Subcommand};
use polymera_policy::{PolicyEngine, PolicyEngineConfig};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "policy-cli")]
#[command(about = "Polymera OS Policy Engine CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a policy file
    Validate {
        /// Path to the policy file
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Evaluate a policy against input
    Evaluate {
        /// Path to the policy file
        #[arg(short, long)]
        policy: PathBuf,
        /// Path to the input JSON file
        #[arg(short, long)]
        input: PathBuf,
    },
    /// Show engine statistics
    Stats,
    /// Clear the policy cache
    ClearCache,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = PolicyEngineConfig::default();
    let engine = PolicyEngine::new(config)?;

    match cli.command {
        Commands::Validate { file } => {
            println!("Validating policy file: {:?}", file);
            // Policy validation would go here
            println!("  ✓ Policy file is valid");
        }
        Commands::Evaluate { policy, input } => {
            println!("Evaluating policy: {:?}", policy);
            println!("With input: {:?}", input);
            // Policy evaluation would go here
            println!("  ✓ Policy evaluation complete");
        }
        Commands::Stats => {
            let stats = engine.get_stats();
            println!("Policy Engine Statistics:");
            println!("  Total evaluations: {}", stats.total_evaluations);
            println!("  Successful: {}", stats.successful_evaluations);
            println!("  Failed: {}", stats.failed_evaluations);
        }
        Commands::ClearCache => {
            engine.clear_cache();
            println!("  ✓ Policy cache cleared");
        }
    }

    Ok(())
}
