//! ABI Lint Tool for Polymera OS
//! 
//! This tool checks for breaking changes in the ABI schema and ensures
//! backward compatibility. It compares the current schema against a
//! previously released schema to detect incompatible changes.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use log::{info, warn, error};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

use crate::model::{AbiSchema, SyscallsSchema, ErrorCodesSchema, FeaturesSchema};

mod model;

/// ABI Lint Tool for Polymera OS
/// 
/// This tool checks for breaking changes in the ABI schema and ensures
/// backward compatibility. It compares the current schema against a
/// previously released schema to detect incompatible changes.
#[derive(Parser)]
#[command(name = "abi-lint")]
#[command(about = "Lint ABI schemas for breaking changes")]
#[command(version = "0.2.0")]
struct Cli {
    /// Current schema directory
    #[arg(short, long, default_value = "abi")]
    current_dir: PathBuf,
    
    /// Previous schema directory or file
    #[arg(short, long)]
    previous: Option<PathBuf>,
    
    /// Output report file
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Fail on warnings
    #[arg(long)]
    strict: bool,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check for breaking changes
    Check {
        /// Current schema directory
        #[arg(short, long, default_value = "abi")]
        current_dir: PathBuf,
        
        /// Previous schema directory or file
        #[arg(short, long)]
        previous: Option<PathBuf>,
        
        /// Output report file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Fail on warnings
        #[arg(long)]
        strict: bool,
    },
    
    /// Generate compatibility report
    Report {
        /// Current schema directory
        #[arg(short, long, default_value = "abi")]
        current_dir: PathBuf,
        
        /// Previous schema directory or file
        #[arg(short, long)]
        previous: Option<PathBuf>,
        
        /// Output report file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// ABI compatibility checker
struct AbiLinter {
    current_schema: AbiSchema,
    previous_schema: Option<AbiSchema>,
    strict_mode: bool,
}

/// Types of changes that can occur
#[derive(Debug, Clone, PartialEq)]
enum ChangeType {
    /// No change
    None,
    /// Non-breaking change (additive)
    NonBreaking,
    /// Breaking change (incompatible)
    Breaking,
}

/// Change information
#[derive(Debug, Clone)]
struct Change {
    change_type: ChangeType,
    description: String,
    severity: String,
    recommendation: String,
}

/// Linting results
#[derive(Debug)]
struct LintResults {
    changes: Vec<Change>,
    breaking_changes: Vec<Change>,
    warnings: Vec<Change>,
    overall_status: String,
}

impl AbiLinter {
    /// Create a new ABI linter
    fn new(current_schema: AbiSchema, previous_schema: Option<AbiSchema>, strict_mode: bool) -> Self {
        Self {
            current_schema,
            previous_schema,
            strict_mode,
        }
    }
    
    /// Run the complete linting process
    fn lint(&self) -> Result<LintResults> {
        info!("Starting ABI compatibility check...");
        
        let mut changes = Vec::new();
        let mut breaking_changes = Vec::new();
        let mut warnings = Vec::new();
        
        if let Some(previous) = &self.previous_schema {
            // Check syscalls
            let syscall_changes = self.check_syscall_compatibility(&previous.syscalls, &self.current_schema.syscalls)?;
            changes.extend(syscall_changes.clone());
            
            // Check error codes
            let errno_changes = self.check_errno_compatibility(&previous.error_codes, &self.current_schema.error_codes)?;
            changes.extend(errno_changes.clone());
            
            // Check features
            let feature_changes = self.check_feature_compatibility(&previous.features, &self.current_schema.features)?;
            changes.extend(feature_changes.clone());
            
            // Categorize changes
            for change in &changes {
                match change.change_type {
                    ChangeType::Breaking => breaking_changes.push(change.clone()),
                    ChangeType::NonBreaking => warnings.push(change.clone()),
                    ChangeType::None => {}
                }
            }
        } else {
            // No previous schema to compare against
            info!("No previous schema provided, skipping compatibility checks");
        }
        
        // Determine overall status
        let overall_status = if !breaking_changes.is_empty() {
            "FAILED".to_string()
        } else if self.strict_mode && !warnings.is_empty() {
            "FAILED (strict mode)".to_string()
        } else {
            "PASSED".to_string()
        };
        
        let results = LintResults {
            changes,
            breaking_changes,
            warnings,
            overall_status,
        };
        
        info!("ABI compatibility check completed: {}", results.overall_status);
        Ok(results)
    }
    
    /// Check syscall compatibility
    fn check_syscall_compatibility(&self, previous: &SyscallsSchema, current: &SyscallsSchema) -> Result<Vec<Change>> {
        let mut changes = Vec::new();
        
        // Create maps for easy lookup
        let previous_syscalls: HashMap<u32, &crate::model::Syscall> = previous.syscalls.iter()
            .map(|s| (s.id, s))
            .collect();
        
        let current_syscalls: HashMap<u32, &crate::model::Syscall> = current.syscalls.iter()
            .map(|s| (s.id, s))
            .collect();
        
        // Check for removed syscalls
        for (id, syscall) in &previous_syscalls {
            if !current_syscalls.contains_key(id) {
                changes.push(Change {
                    change_type: ChangeType::Breaking,
                    description: format!("Syscall {} (ID: 0x{:04x}) was removed", syscall.name, id),
                    severity: "ERROR".to_string(),
                    recommendation: "Keep removed syscalls for backward compatibility or assign new IDs".to_string(),
                });
            }
        }
        
        // Check for modified syscalls
        for (id, current_syscall) in &current_syscalls {
            if let Some(previous_syscall) = previous_syscalls.get(id) {
                // Check argument changes
                if current_syscall.args.len() != previous_syscall.args.len() {
                    changes.push(Change {
                        change_type: ChangeType::Breaking,
                        description: format!("Syscall {} argument count changed from {} to {}", 
                            current_syscall.name, previous_syscall.args.len(), current_syscall.args.len()),
                        severity: "ERROR".to_string(),
                        recommendation: "Use new syscall ID for different argument signature".to_string(),
                    });
                } else {
                    // Check individual argument types
                    for (i, (prev_arg, curr_arg)) in previous_syscall.args.iter().zip(current_syscall.args.iter()).enumerate() {
                        if prev_arg.type_name != curr_arg.type_name {
                            changes.push(Change {
                                change_type: ChangeType::Breaking,
                                description: format!("Syscall {} argument {} type changed from {} to {}", 
                                    current_syscall.name, i, prev_arg.type_name, curr_arg.type_name),
                                severity: "ERROR".to_string(),
                                recommendation: "Use new syscall ID for different argument types".to_string(),
                            });
                        }
                    }
                }
                
                // Check return type changes
                if current_syscall.ret.type_name != previous_syscall.ret.type_name {
                    changes.push(Change {
                        change_type: ChangeType::Breaking,
                        description: format!("Syscall {} return type changed from {} to {}", 
                            current_syscall.name, previous_syscall.ret.type_name, current_syscall.ret.type_name),
                        severity: "ERROR".to_string(),
                        recommendation: "Use new syscall ID for different return type".to_string(),
                    });
                }
            }
        }
        
        // Check for new syscalls
        for (id, syscall) in &current_syscalls {
            if !previous_syscalls.contains_key(id) {
                changes.push(Change {
                    change_type: ChangeType::NonBreaking,
                    description: format!("New syscall {} (ID: 0x{:04x}) added", syscall.name, id),
                    severity: "INFO".to_string(),
                    recommendation: "New syscalls are additive and safe".to_string(),
                });
            }
        }
        
        Ok(changes)
    }
    
    /// Check error code compatibility
    fn check_errno_compatibility(&self, previous: &ErrorCodesSchema, current: &ErrorCodesSchema) -> Result<Vec<Change>> {
        let mut changes = Vec::new();
        
        // Create maps for easy lookup
        let previous_errnos: HashMap<u32, &crate::model::ErrorCode> = previous.errno.iter()
            .map(|e| (e.id, e))
            .collect();
        
        let current_errnos: HashMap<u32, &crate::model::ErrorCode> = current.errno.iter()
            .map(|e| (e.id, e))
            .collect();
        
        // Check for removed error codes
        for (id, errno) in &previous_errnos {
            if !current_errnos.contains_key(id) {
                changes.push(Change {
                    change_type: ChangeType::Breaking,
                    description: format!("Error code {} (ID: {}) was removed", errno.name, id),
                    severity: "ERROR".to_string(),
                    recommendation: "Keep removed error codes for backward compatibility".to_string(),
                });
            }
        }
        
        // Check for modified error codes
        for (id, current_errno) in &current_errnos {
            if let Some(previous_errno) = previous_errnos.get(id) {
                if current_errno.name != previous_errno.name {
                    changes.push(Change {
                        change_type: ChangeType::Breaking,
                        description: format!("Error code ID {} name changed from {} to {}", 
                            id, previous_errno.name, current_errno.name),
                        severity: "ERROR".to_string(),
                        recommendation: "Error code names must remain stable".to_string(),
                    });
                }
            }
        }
        
        // Check for new error codes
        for (id, errno) in &current_errnos {
            if !previous_errnos.contains_key(id) {
                changes.push(Change {
                    change_type: ChangeType::NonBreaking,
                    description: format!("New error code {} (ID: {}) added", errno.name, id),
                    severity: "INFO".to_string(),
                    recommendation: "New error codes are additive and safe".to_string(),
                });
            }
        }
        
        Ok(changes)
    }
    
    /// Check feature compatibility
    fn check_feature_compatibility(&self, previous: &FeaturesSchema, current: &FeaturesSchema) -> Result<Vec<Change>> {
        let mut changes = Vec::new();
        
        // Create maps for easy lookup
        let previous_features: HashMap<u32, &crate::model::Feature> = previous.features.iter()
            .map(|f| (f.id, f))
            .collect();
        
        let current_features: HashMap<u32, &crate::model::Feature> = current.features.iter()
            .map(|f| (f.id, f))
            .collect();
        
        // Check for removed features
        for (id, feature) in &previous_features {
            if !current_features.contains_key(id) {
                changes.push(Change {
                    change_type: ChangeType::Breaking,
                    description: format!("Feature {} (ID: {}) was removed", feature.name, id),
                    severity: "ERROR".to_string(),
                    recommendation: "Keep removed features for backward compatibility".to_string(),
                });
            }
        }
        
        // Check for modified features
        for (id, current_feature) in &current_features {
            if let Some(previous_feature) = previous_features.get(id) {
                if current_feature.name != previous_feature.name {
                    changes.push(Change {
                        change_type: ChangeType::Breaking,
                        description: format!("Feature ID {} name changed from {} to {}", 
                            id, previous_feature.name, current_feature.name),
                        severity: "ERROR".to_string(),
                        recommendation: "Feature names must remain stable".to_string(),
                    });
                }
                
                if current_feature.stability != previous_feature.stability {
                    changes.push(Change {
                        change_type: ChangeType::NonBreaking,
                        description: format!("Feature {} stability changed from {} to {}", 
                            current_feature.name, previous_feature.stability, current_feature.stability),
                        severity: "WARNING".to_string(),
                        recommendation: "Stability changes should be documented".to_string(),
                    });
                }
            }
        }
        
        // Check for new features
        for (id, feature) in &current_features {
            if !previous_features.contains_key(id) {
                changes.push(Change {
                    change_type: ChangeType::NonBreaking,
                    description: format!("New feature {} (ID: {}) added", feature.name, id),
                    severity: "INFO".to_string(),
                    recommendation: "New features are additive and safe".to_string(),
                });
            }
        }
        
        Ok(changes)
    }
    
    /// Generate a compatibility report
    fn generate_report(&self, results: &LintResults) -> Result<String> {
        let mut report = String::new();
        
        report.push_str("# ABI Compatibility Report\n\n");
        report.push_str(&format!("**Status**: {}\n\n", results.overall_status);
        report.push_str(&format!("**Generated**: {}\n\n", chrono::Utc::now().to_rfc3339());
        report.push_str(&format!("**Schema Hash**: {}\n\n", self.current_schema.schema_hash);
        
        if let Some(previous) = &self.previous_schema {
            report.push_str(&format!("**Previous Schema Hash**: {}\n\n", previous.schema_hash);
        }
        
        // Summary
        report.push_str("## Summary\n\n");
        report.push_str(&format!("- **Total Changes**: {}\n", results.changes.len());
        report.push_str(&format!("- **Breaking Changes**: {}\n", results.breaking_changes.len());
        report.push_str(&format!("- **Warnings**: {}\n", results.warnings.len());
        report.push_str(&format!("- **Overall Status**: {}\n\n", results.overall_status);
        
        // Breaking changes
        if !results.breaking_changes.is_empty() {
            report.push_str("## Breaking Changes\n\n");
            report.push_str("These changes will break existing applications:\n\n");
            
            for change in &results.breaking_changes {
                report.push_str(&format!("### {}\n", change.description));
                report.push_str(&format!("- **Severity**: {}\n", change.severity);
                report.push_str(&format!("- **Recommendation**: {}\n\n", change.recommendation);
            }
        }
        
        // Warnings
        if !results.warnings.is_empty() {
            report.push_str("## Warnings\n\n");
            report.push_str("These changes may affect compatibility:\n\n");
            
            for change in &results.warnings {
                report.push_str(&format!("### {}\n", change.description));
                report.push_str(&format!("- **Severity**: {}\n", change.severity);
                report.push_str(&format!("- **Recommendation**: {}\n\n", change.recommendation);
            }
        }
        
        // All changes
        if !results.changes.is_empty() {
            report.push_str("## All Changes\n\n");
            report.push_str("| Type | Description | Severity | Recommendation |\n");
            report.push_str("|------|-------------|----------|----------------|\n");
            
            for change in &results.changes {
                let change_type = match change.change_type {
                    ChangeType::Breaking => "Breaking",
                    ChangeType::NonBreaking => "Non-Breaking",
                    ChangeType::None => "None",
                };
                
                report.push_str(&format!("| {} | {} | {} | {} |\n",
                    change_type, change.description, change.severity, change.recommendation));
            }
        }
        
        Ok(report)
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
        Some(Commands::Check { current_dir, previous, output, strict }) => {
            run_check(current_dir, previous.as_ref(), output.as_ref(), *strict)?;
        }
        
        Some(Commands::Report { current_dir, previous, output }) => {
            run_report(current_dir, previous.as_ref(), output.as_ref())?;
        }
        
        None => {
            // Default behavior: run check
            run_check(&cli.current_dir, cli.previous.as_ref(), cli.output.as_ref(), cli.strict)?;
        }
    }
    
    Ok(())
}

/// Run the compatibility check
fn run_check(current_dir: &Path, previous: Option<&Path>, output: Option<&Path>, strict: bool) -> Result<()> {
    info!("Loading current schema from {}", current_dir.display());
    
    // Load current schema
    let current_schema = load_schema_from_dir(current_dir)?;
    
    // Load previous schema if provided
    let previous_schema = if let Some(prev_path) = previous {
        if prev_path.is_dir() {
            Some(load_schema_from_dir(prev_path)?)
        } else {
            Some(load_schema_from_file(prev_path)?)
        }
    } else {
        None
    };
    
    // Create linter
    let linter = AbiLinter::new(current_schema, previous_schema, strict);
    
    // Run linting
    let results = linter.lint()?;
    
    // Generate report
    let report = linter.generate_report(&results)?;
    
    // Output report
    if let Some(output_path) = output {
        fs::write(output_path, &report)
            .context("Failed to write report file")?;
        info!("Report written to {}", output_path.display());
    } else {
        println!("{}", report);
    }
    
    // Exit with appropriate code
    if results.overall_status == "PASSED" {
        info!("ABI compatibility check passed");
        std::process::exit(0);
    } else {
        error!("ABI compatibility check failed");
        std::process::exit(1);
    }
}

/// Run the compatibility report
fn run_report(current_dir: &Path, previous: Option<&Path>, output: Option<&Path>) -> Result<()> {
    info!("Generating compatibility report...");
    
    // Load current schema
    let current_schema = load_schema_from_dir(current_dir)?;
    
    // Load previous schema if provided
    let previous_schema = if let Some(prev_path) = previous {
        if prev_path.is_dir() {
            Some(load_schema_from_dir(prev_path)?)
        } else {
            Some(load_schema_from_file(prev_path)?)
        }
    } else {
        None
    };
    
    // Create linter
    let linter = AbiLinter::new(current_schema, previous_schema, false);
    
    // Run linting
    let results = linter.lint()?;
    
    // Generate report
    let report = linter.generate_report(&results)?;
    
    // Output report
    if let Some(output_path) = output {
        fs::write(output_path, &report)
            .context("Failed to write report file")?;
        info!("Report written to {}", output_path.display());
    } else {
        println!("{}", report);
    }
    
    Ok(())
}

/// Load schema from directory
fn load_schema_from_dir(dir: &Path) -> Result<AbiSchema> {
    // Load syscalls schema
    let syscalls_path = dir.join("syscalls.yaml");
    let syscalls_content = fs::read_to_string(&syscalls_path)
        .context(format!("Failed to read {}", syscalls_path.display()))?;
    let syscalls: SyscallsSchema = serde_yaml::from_str(&syscalls_content)
        .context("Failed to parse syscalls.yaml")?;
    
    // Load error codes schema
    let errno_path = dir.join("errno.yaml");
    let errno_content = fs::read_to_string(&errno_path)
        .context(format!("Failed to read {}", errno_path.display()))?;
    let error_codes: ErrorCodesSchema = serde_yaml::from_str(&errno_content)
        .context("Failed to parse errno.yaml")?;
    
    // Load features schema
    let features_path = dir.join("features.yaml");
    let features_content = fs::read_to_string(&features_path)
        .context(format!("Failed to read {}", features_path.display()))?;
    let features: FeaturesSchema = serde_yaml::from_str(&features_content)
        .context("Failed to parse features.yaml")?;
    
    // Generate schema hash
    let mut hasher = Sha256::new();
    hasher.update(syscalls_content.as_bytes());
    hasher.update(errno_content.as_bytes());
    hasher.update(features_content.as_bytes());
    let schema_hash = format!("{:x}", hasher.finalize());
    
    let schema = AbiSchema {
        syscalls,
        error_codes,
        features,
        schema_hash,
    };
    
    // Validate the schema
    schema.validate().context("Schema validation failed")?;
    
    Ok(schema)
}

/// Load schema from file (JSON format)
fn load_schema_from_file(file: &Path) -> Result<AbiSchema> {
    let content = fs::read_to_string(file)
        .context(format!("Failed to read {}", file.display()))?;
    
    let schema: AbiSchema = serde_json::from_str(&content)
        .context("Failed to parse schema file")?;
    
    Ok(schema)
}

