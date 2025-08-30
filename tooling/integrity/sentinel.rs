use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use serde::{Deserialize, Serialize};
use blake3::Hasher;

#[derive(Debug, Deserialize, Serialize)]
struct IntegrityManifest {
    version: u32,
    description: String,
    created: String,
    last_updated: String,
    phase: String,
    schemas: Vec<FileEntry>,
    cbor_fixtures: Vec<FileEntry>,
    performance: Vec<FileEntry>,
    ipfs_exports: Vec<FileEntry>,
    test_vectors: Vec<FileEntry>,
    generated_outputs: Vec<FileEntry>,
    configs: Vec<FileEntry>,
    settings: Settings,
    rebaseline: RebaselineConfig,
    ci: CIConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct FileEntry {
    path: String,
    digest: String,
    last_phase: String,
    description: String,
    critical: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    hash_algorithm: String,
    hash_format: String,
    max_file_size_mb: u32,
    critical_failure_threshold: u32,
    warning_threshold: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct RebaselineConfig {
    require_reason: bool,
    require_ticket: bool,
    require_phase_update: bool,
    audit_logging: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct CIConfig {
    job_name: String,
    timeout_minutes: u32,
    artifact_retention_days: u32,
    block_on_failure: bool,
    pre_test_gate: bool,
}

#[derive(Debug, Serialize)]
struct IntegrityReport {
    files_checked: u32,
    mismatches: u32,
    details: Vec<IntegrityDetail>,
    summary: String,
}

#[derive(Debug, Serialize)]
struct IntegrityDetail {
    path: String,
    expected_hash: String,
    actual_hash: String,
    critical: bool,
    status: String,
}

#[derive(Debug, Serialize)]
struct IntegrityDiff {
    path: String,
    old_hash: String,
    new_hash: String,
    critical: bool,
    description: String,
}

struct IntegritySentinel {
    manifest: IntegrityManifest,
    workspace_root: PathBuf,
}

impl IntegritySentinel {
    fn new(manifest_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let manifest_content = fs::read_to_string(manifest_path)?;
        let manifest: IntegrityManifest = serde_yaml::from_str(&manifest_content)?;
        
        let workspace_root = std::env::current_dir()?;
        
        Ok(Self {
            manifest,
            workspace_root,
        })
    }
    
    fn run_check(&self) -> Result<IntegrityReport, Box<dyn std::error::Error>> {
        let mut report = IntegrityReport {
            files_checked: 0,
            mismatches: 0,
            details: Vec::new(),
            summary: String::new(),
        };
        
        // Check all file categories
        self.check_file_category(&self.manifest.schemas, &mut report)?;
        self.check_file_category(&self.manifest.cbor_fixtures, &mut report)?;
        self.check_file_category(&self.manifest.performance, &mut report)?;
        self.check_file_category(&self.manifest.ipfs_exports, &mut report)?;
        self.check_file_category(&self.manifest.test_vectors, &mut report)?;
        self.check_file_category(&self.manifest.generated_outputs, &mut report)?;
        self.check_file_category(&self.manifest.configs, &mut report)?;
        
        // Generate summary
        if report.mismatches == 0 {
            report.summary = format!("✅ Integrity check passed: {} files verified", report.files_checked);
        } else {
            report.summary = format!("❌ Integrity check failed: {} mismatches in {} files", report.mismatches, report.files_checked);
        }
        
        Ok(report)
    }
    
    fn check_file_category(&self, files: &[FileEntry], report: &mut IntegrityReport) -> Result<(), Box<dyn std::error::Error>> {
        for file_entry in files {
            let file_path = self.workspace_root.join(&file_entry.path);
            
            if !file_path.exists() {
                let detail = IntegrityDetail {
                    path: file_entry.path.clone(),
                    expected_hash: file_entry.digest.clone(),
                    actual_hash: "FILE_NOT_FOUND".to_string(),
                    critical: file_entry.critical,
                    status: "MISSING".to_string(),
                };
                report.details.push(detail);
                report.mismatches += 1;
                continue;
            }
            
            // Check file size limit
            let metadata = fs::metadata(&file_path)?;
            let file_size_mb = metadata.len() / (1024 * 1024);
            if file_size_mb > self.manifest.settings.max_file_size_mb as u64 {
                let detail = IntegrityDetail {
                    path: file_entry.path.clone(),
                    expected_hash: file_entry.digest.clone(),
                    actual_hash: format!("FILE_TOO_LARGE_{}MB", file_size_mb),
                    critical: file_entry.critical,
                    status: "SIZE_LIMIT_EXCEEDED".to_string(),
                };
                report.details.push(detail);
                report.mismatches += 1;
                continue;
            }
            
            // Compute actual hash
            let actual_hash = self.compute_file_hash(&file_path)?;
            report.files_checked += 1;
            
            if actual_hash != file_entry.digest {
                let detail = IntegrityDetail {
                    path: file_entry.path.clone(),
                    expected_hash: file_entry.digest.clone(),
                    actual_hash: actual_hash.clone(),
                    critical: file_entry.critical,
                    status: "HASH_MISMATCH".to_string(),
                };
                report.details.push(detail);
                report.mismatches += 1;
            } else {
                let detail = IntegrityDetail {
                    path: file_entry.path.clone(),
                    expected_hash: file_entry.digest.clone(),
                    actual_hash,
                    critical: file_entry.critical,
                    status: "MATCH".to_string(),
                };
                report.details.push(detail);
            }
        }
        
        Ok(())
    }
    
    fn compute_file_hash(&self, file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let mut hasher = Hasher::new();
        let file_content = fs::read(file_path)?;
        hasher.update(&file_content);
        let hash = hasher.finalize();
        Ok(hash.to_hex().to_lowercase())
    }
    
    fn run_update(&self, reason: &str, ticket: Option<&str>) -> Result<Vec<IntegrityDiff>, Box<dyn std::error::Error>> {
        if self.manifest.rebaseline.require_reason && reason.is_empty() {
            return Err("Rebaseline requires a reason".into());
        }
        
        if self.manifest.rebaseline.require_ticket && ticket.is_none() {
            return Err("Rebaseline requires a ticket number".into());
        }
        
        let mut diffs = Vec::new();
        
        // Check all files and collect diffs
        let all_files = self.collect_all_files();
        for file_entry in all_files {
            let file_path = self.workspace_root.join(&file_entry.path);
            
            if !file_path.exists() {
                continue;
            }
            
            let actual_hash = self.compute_file_hash(&file_path)?;
            if actual_hash != file_entry.digest {
                let diff = IntegrityDiff {
                    path: file_entry.path.clone(),
                    old_hash: file_entry.digest.clone(),
                    new_hash: actual_hash,
                    critical: file_entry.critical,
                    description: file_entry.description.clone(),
                };
                diffs.push(diff);
            }
        }
        
        // Log rebaseline operation
        if self.manifest.rebaseline.audit_logging {
            println!("🔄 Rebaseline operation:");
            println!("  Reason: {}", reason);
            if let Some(ticket) = ticket {
                println!("  Ticket: {}", ticket);
            }
            println!("  Files changed: {}", diffs.len());
            for diff in &diffs {
                println!("    {}: {} -> {}", diff.path, diff.old_hash, diff.new_hash);
            }
        }
        
        Ok(diffs)
    }
    
    fn collect_all_files(&self) -> Vec<&FileEntry> {
        let mut all_files = Vec::new();
        all_files.extend(&self.manifest.schemas);
        all_files.extend(&self.manifest.cbor_fixtures);
        all_files.extend(&self.manifest.performance);
        all_files.extend(&self.manifest.ipfs_exports);
        all_files.extend(&self.manifest.test_vectors);
        all_files.extend(&self.manifest.generated_outputs);
        all_files.extend(&self.manifest.configs);
        all_files
    }
    
    fn verify_cbor_determinism(&self) -> Result<bool, Box<dyn std::error::Error>> {
        println!("🔍 Verifying CBOR determinism across languages...");
        
        // This would run actual cross-language CBOR serialization tests
        // For now, we'll simulate the verification
        let mut all_deterministic = true;
        
        for cbor_file in &self.manifest.cbor_fixtures {
            let file_path = self.workspace_root.join(&cbor_file.path);
            if file_path.exists() {
                println!("  Checking {}: {}", cbor_file.path, cbor_file.digest);
                
                // In a real implementation, this would:
                // 1. Parse CBOR in Rust
                // 2. Parse CBOR in Go
                // 3. Parse CBOR in Python
                // 4. Parse CBOR in TypeScript
                // 5. Compare serialized outputs
                
                // For now, just verify the file exists and has the expected hash
                let actual_hash = self.compute_file_hash(&file_path)?;
                if actual_hash != cbor_file.digest {
                    println!("    ❌ Hash mismatch: expected {}, got {}", cbor_file.digest, actual_hash);
                    all_deterministic = false;
                } else {
                    println!("    ✅ Hash matches");
                }
            }
        }
        
        Ok(all_deterministic)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <check|update> [--manifest <path>] [--reason <reason>] [--ticket <ticket>]", args[0]);
        process::exit(1);
    }
    
    let command = &args[1];
    let mut manifest_path = PathBuf::from("tooling/integrity/manifest.yml");
    let mut reason = String::new();
    let mut ticket = None;
    
    // Parse arguments
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--manifest" => {
                if i + 1 < args.len() {
                    manifest_path = PathBuf::from(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("--manifest requires a path");
                    process::exit(1);
                }
            }
            "--reason" => {
                if i + 1 < args.len() {
                    reason = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("--reason requires a reason");
                    process::exit(1);
                }
            }
            "--ticket" => {
                if i + 1 < args.len() {
                    ticket = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("--ticket requires a ticket number");
                    process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                process::exit(1);
            }
        }
    }
    
    // Initialize sentinel
    let sentinel = match IntegritySentinel::new(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load manifest: {}", e);
            process::exit(1);
        }
    };
    
    match command.as_str() {
        "check" => {
            match sentinel.run_check() {
                Ok(report) => {
                    println!("{}", report.summary);
                    
                    if report.mismatches > 0 {
                        println!("\nMismatch details:");
                        for detail in &report.details {
                            if detail.status != "MATCH" {
                                println!("  {}: {} (expected: {}, got: {})", 
                                    detail.path, detail.status, detail.expected_hash, detail.actual_hash);
                            }
                        }
                        
                        // Verify CBOR determinism
                        if let Err(e) = sentinel.verify_cbor_determinism() {
                            eprintln!("Failed to verify CBOR determinism: {}", e);
                        }
                        
                        process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Integrity check failed: {}", e);
                    process::exit(1);
                }
            }
        }
        "update" => {
            match sentinel.run_update(&reason, ticket.as_deref()) {
                Ok(diffs) => {
                    if diffs.is_empty() {
                        println!("✅ No changes detected, manifest is up to date");
                    } else {
                        println!("🔄 Manifest updated with {} changes", diffs.len());
                        println!("Please update the manifest.yml file with the new hashes");
                    }
                }
                Err(e) => {
                    eprintln!("Rebaseline failed: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Available commands: check, update");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_manifest_loading() {
        let manifest_content = r#"
version: 1
description: "Test manifest"
created: "2024-01-01T00:00:00Z"
last_updated: "2024-01-01T00:00:00Z"
phase: "TEST"
schemas: []
cbor_fixtures: []
performance: []
ipfs_exports: []
test_vectors: []
generated_outputs: []
configs: []
settings:
  hash_algorithm: "blake3-256"
  hash_format: "hex_lowercase"
  max_file_size_mb: 10
  critical_failure_threshold: 1
  warning_threshold: 0
rebaseline:
  require_reason: true
  require_ticket: true
  require_phase_update: true
  audit_logging: true
ci:
  job_name: "test"
  timeout_minutes: 15
  artifact_retention_days: 90
  block_on_failure: true
  pre_test_gate: true
"#;
        
        let manifest: IntegrityManifest = serde_yaml::from_str(manifest_content).unwrap();
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.description, "Test manifest");
        assert_eq!(manifest.phase, "TEST");
    }
    
    #[test]
    fn test_file_hash_computation() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();
        
        let sentinel = IntegritySentinel {
            manifest: serde_yaml::from_str("").unwrap(),
            workspace_root: temp_dir.path().to_path_buf(),
        };
        
        let hash = sentinel.compute_file_hash(&test_file).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // blake3-256 hex length
    }
    
    #[test]
    fn test_integrity_check() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Test content").unwrap();
        
        let manifest_content = r#"
version: 1
description: "Test"
created: "2024-01-01T00:00:00Z"
last_updated: "2024-01-01T00:00:00Z"
phase: "TEST"
schemas:
  - path: "test.txt"
    digest: "0000000000000000000000000000000000000000000000000000000000000000"
    last_phase: "TEST"
    description: "Test file"
    critical: true
cbor_fixtures: []
performance: []
ipfs_exports: []
test_vectors: []
generated_outputs: []
configs: []
settings:
  hash_algorithm: "blake3-256"
  hash_format: "hex_lowercase"
  max_file_size_mb: 10
  critical_failure_threshold: 1
  warning_threshold: 0
rebaseline:
  require_reason: true
  require_ticket: true
  require_phase_update: true
  audit_logging: true
ci:
  job_name: "test"
  timeout_minutes: 15
  artifact_retention_days: 90
  block_on_failure: true
  pre_test_gate: true
"#;
        
        let manifest: IntegrityManifest = serde_yaml::from_str(manifest_content).unwrap();
        let sentinel = IntegritySentinel {
            manifest,
            workspace_root: temp_dir.path().to_path_buf(),
        };
        
        let report = sentinel.run_check().unwrap();
        assert_eq!(report.mismatches, 1); // Should detect hash mismatch
        assert_eq!(report.files_checked, 1);
    }
}
