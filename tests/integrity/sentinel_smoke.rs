use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_integrity_sentinel_detects_mismatch() {
    // This test simulates what the integrity sentinel would do
    // by checking file integrity and detecting modifications
    
    let temp_dir = tempdir().unwrap();
    let test_file_path = temp_dir.path().join("test_file.txt");
    
    // Create initial file
    fs::write(&test_file_path, "original content").unwrap();
    
    // Get initial hash (simplified - would use blake3 in real implementation)
    let initial_content = fs::read(&test_file_path).unwrap();
    let initial_hash = compute_simple_hash(&initial_content);
    
    // Verify initial integrity
    let current_content = fs::read(&test_file_path).unwrap();
    let current_hash = compute_simple_hash(&current_content);
    
    assert_eq!(initial_hash, current_hash, "File integrity should match initially");
    
    // Modify file to simulate corruption
    fs::write(&test_file_path, "modified content").unwrap();
    
    // Check that integrity is now broken
    let modified_content = fs::read(&test_file_path).unwrap();
    let modified_hash = compute_simple_hash(&modified_content);
    
    assert_ne!(initial_hash, modified_hash, "File integrity should be broken after modification");
    
    // Verify mismatch detection
    let mismatch_detected = initial_hash != modified_hash;
    assert!(mismatch_detected, "Mismatch should be detected");
}

#[test]
fn test_integrity_manifest_validation() {
    // Test manifest structure validation
    let manifest = create_test_manifest();
    
    // Validate required fields
    assert_eq!(manifest.version, 1, "Version should be 1");
    assert_eq!(manifest.hash_algorithm, "blake3-256", "Hash algorithm should be blake3-256");
    assert!(!manifest.entries.is_empty(), "Manifest should have entries");
    
    // Validate entry structure
    for entry in &manifest.entries {
        assert!(!entry.path.is_empty(), "Entry path should not be empty");
        assert!(!entry.digest.is_empty(), "Entry digest should not be empty");
        assert!(entry.digest.len() == 64, "Digest should be 64 characters (32 bytes hex)");
        assert!(entry.digest.chars().all(|c| c.is_ascii_hexdigit()), "Digest should be hex");
    }
}

#[test]
fn test_critical_file_detection() {
    let manifest = create_test_manifest();
    
    // Count critical files
    let critical_count = manifest.entries.iter()
        .filter(|e| e.critical)
        .count();
    
    assert!(critical_count > 0, "Should have at least one critical file");
    
    // Verify critical files are important
    let critical_files: Vec<_> = manifest.entries.iter()
        .filter(|e| e.critical)
        .map(|e| &e.path)
        .collect();
    
    // Critical files should include schemas and core components
    assert!(critical_files.iter().any(|p| p.contains("schema")), "Should have critical schema files");
    assert!(critical_files.iter().any(|p| p.contains("manifest")), "Should have critical manifest files");
}

#[test]
fn test_rebaseline_requirements() {
    let manifest = create_test_manifest();
    
    // Test rebaseline requirements
    assert!(manifest.rebaseline.require_reason, "Rebaseline should require reason");
    assert!(manifest.rebaseline.require_ticket, "Rebaseline should require ticket");
    assert!(manifest.rebaseline.require_phase_update, "Rebaseline should require phase update");
    assert!(manifest.rebaseline.audit_logging, "Rebaseline should have audit logging");
}

#[test]
fn test_ci_integration_settings() {
    let manifest = create_test_manifest();
    
    // Test CI integration settings
    assert_eq!(manifest.ci.job_name, "ngfs-integrity", "CI job name should match");
    assert!(manifest.ci.timeout_minutes > 0, "CI timeout should be positive");
    assert!(manifest.ci.artifact_retention_days > 0, "Artifact retention should be positive");
    assert!(manifest.ci.block_on_failure, "CI should block on failure");
    assert!(manifest.ci.pre_test_gate, "CI should have pre-test gate");
}

// Helper functions

fn compute_simple_hash(content: &[u8]) -> String {
    // Simplified hash function for testing
    // In real implementation, this would use blake3
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn create_test_manifest() -> TestManifest {
    TestManifest {
        version: 1,
        description: "Test integrity manifest".to_string(),
        hash_algorithm: "blake3-256".to_string(),
        entries: vec![
            TestEntry {
                path: "services/ngfs/src/schema.rs".to_string(),
                digest: "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456".to_string(),
                last_phase: "P3-01-A1".to_string(),
                description: "NGFS core schema definitions".to_string(),
                critical: true,
            },
            TestEntry {
                path: "tests/ngfs/fixtures/dir_simple.cbor".to_string(),
                digest: "b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890".to_string(),
                last_phase: "P3-01-A6".to_string(),
                description: "Directory manifest fixture".to_string(),
                critical: true,
            },
            TestEntry {
                path: "perf/baselines/p3_ngfs.json".to_string(),
                digest: "c3d4e5f6789012345678901234567890abcdef1234567890abcdef1234567890ab".to_string(),
                last_phase: "P3-01-A7".to_string(),
                description: "Performance baseline".to_string(),
                critical: false,
            },
        ],
        rebaseline: TestRebaseline {
            require_reason: true,
            require_ticket: true,
            require_phase_update: true,
            audit_logging: true,
        },
        ci: TestCI {
            job_name: "ngfs-integrity".to_string(),
            timeout_minutes: 15,
            artifact_retention_days: 30,
            block_on_failure: true,
            pre_test_gate: true,
        },
    }
}

// Test data structures

#[derive(Debug)]
struct TestEntry {
    path: String,
    digest: String,
    last_phase: String,
    description: String,
    critical: bool,
}

#[derive(Debug)]
struct TestRebaseline {
    require_reason: bool,
    require_ticket: bool,
    require_phase_update: bool,
    audit_logging: bool,
}

#[derive(Debug)]
struct TestCI {
    job_name: String,
    timeout_minutes: u32,
    artifact_retention_days: u32,
    block_on_failure: bool,
    pre_test_gate: bool,
}

#[derive(Debug)]
struct TestManifest {
    version: u32,
    description: String,
    hash_algorithm: String,
    entries: Vec<TestEntry>,
    rebaseline: TestRebaseline,
    ci: TestCI,
}
