use clap::Parser;
use colored::*;
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

mod diff;
mod normalize;
mod surface;

use surface::{SurfaceLock, SurfaceStatus};

#[derive(Parser)]
#[command(name = "surface_check")]
#[command(about = "Check Polymera OS interface locks and golden surfaces")]
#[command(version, propagate_version = true)]
struct Cli {
    #[arg(short, long, default_value = "SURFACE.lock.json")]
    lock_file: PathBuf,
    
    #[arg(short, long, default_value = "SURFACE.allow.yaml")]
    allow_file: PathBuf,
    
    #[arg(short, long)]
    generate: bool,
    
    #[arg(short, long)]
    html_report: Option<PathBuf>,
    
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    if cli.verbose {
        log::set_max_level(log::LevelFilter::Debug);
    }
    
    info!("🔒 Polymera OS Surface Guard - Checking Interface Locks");
    
    let surface_lock = SurfaceLock::load(&cli.lock_file)?;
    let allowlist = surface::Allowlist::load(&cli.allow_file)?;
    
    if cli.generate {
        info!("🔄 Regenerating golden surfaces...");
        surface_lock.regenerate_surfaces().await?;
        info!("✅ Golden surfaces regenerated");
        return Ok(());
    }
    
    info!("🔍 Checking {} surfaces for drift...", surface_lock.surfaces.len());
    
    let mut results = Vec::new();
    let mut has_drift = false;
    
    for surface in &surface_lock.surfaces {
        match surface.check_status().await {
            Ok(status) => {
                results.push((surface.clone(), status.clone()));
                
                match status {
                    SurfaceStatus::Ok { .. } => {
                        info!("✅ {}: OK", surface.id);
                    }
                    SurfaceStatus::Drift { .. } => {
                        warn!("⚠️  {}: DRIFT DETECTED", surface.id);
                        has_drift = true;
                    }
                    SurfaceStatus::Error { error } => {
                        error!("❌ {}: ERROR - {}", surface.id, error);
                        has_drift = true;
                    }
                }
            }
            Err(e) => {
                error!("❌ Failed to check surface {}: {}", surface.id, e);
                has_drift = true;
            }
        }
    }
    
    if let Some(html_path) = cli.html_report {
        info!("📊 Generating HTML report to {:?}", html_path);
        surface::generate_html_report(&results, &allowlist, &html_path)?;
    }
    
    if has_drift {
        println!("\n{}", "=".repeat(80));
        println!("{}", "INTERFACE DRIFT DETECTED".red().bold());
        println!("{}", "=".repeat(80));
        
        for (surface, status) in &results {
            if let SurfaceStatus::Drift { .. } = status {
                println!("\n🔴 Surface: {}", surface.id);
                println!("   Paths: {}", surface.paths.join(", "));
                println!("   Owner: {}", surface.owner);
                
                if let Some(allowed) = allowlist.find_matching_allowance(&surface.paths) {
                    println!("   ✅ Allowed: {} (by {})", allowed.reason, allowed.author);
                    println!("   📅 Expires: {}", allowed.expires);
                } else {
                    println!("   ❌ Not allowed - requires approval in SURFACE.allow.yaml");
                    println!("   📝 Add entry with justification and approval");
                }
            }
        }
        
        println!("\n{}", "REMEDIATION STEPS:".yellow().bold());
        println!("1. Review the drift above");
        println!("2. If intentional: add entry to SURFACE.allow.yaml with justification");
        println!("3. If unintentional: restore from git or regenerate with --generate");
        println!("4. Re-run: bazel run //tooling/guard:surface_check");
        
        process::exit(1);
    }
    
    println!("\n{}", "✅ ALL SURFACES OK - NO INTERFACE DRIFT DETECTED".green().bold());
    Ok(())
}
