//! Polymera OS Package Format
//! 
//! This module provides a content-addressed package system with:
//! - Content-addressed bundles with SHA-256 hashing
//! - Software Bill of Materials (SBOM) generation
//! - Cryptographic signatures and verification
//! - Capability manifests for security
//! - Comprehensive package validation
//! 
//! ## Features
//! 
//! - **Content Addressing**: Immutable packages identified by content hash
//! - **SBOM Support**: SPDX, CycloneDX, and SWID formats
//! - **Digital Signatures**: Ed25519, RSA, and ECDSA support
//! - **Capability Security**: Fine-grained permission system
//! - **Package Verification**: Comprehensive integrity checking

pub mod schema;
pub mod build;
pub mod verify;

pub use schema::{
    PackageManifest, Capabilities, Capability, Sbom, SbomComponent, SbomRelationship,
    Signature, Metadata, PackageFile, PackageDependency, Checksum, Security, Vulnerability, AuditInfo
};

pub use build::{
    PackageBuilder, PackageBuilderConfig, PackageBuildError
};

pub use verify::{
    PackageVerifier, VerificationConfig, VerificationResult, PackageVerifyError
};

/// Build a package from source
pub async fn build_package(config: PackageBuilderConfig) -> Result<PackageManifest, PackageBuildError> {
    let mut builder = PackageBuilder::new(config);
    
    // Add default capabilities
    builder.add_capability(Capability {
        name: "polymera.runtime".to_string(),
        capability_type: "system".to_string(),
        description: "Access to Polymera OS runtime".to_string(),
        version: Some("1.0.0".to_string()),
        parameters: None,
        constraints: None,
    });
    
    builder.build()
}

/// Verify a package
pub async fn verify_package(
    package_dir: &std::path::Path,
    config: Option<VerificationConfig>,
) -> Result<VerificationResult, PackageVerifyError> {
    let config = config.unwrap_or_default();
    let verifier = PackageVerifier::new(config);
    verifier.verify_package(package_dir)
}

/// Create a simple test package
pub async fn create_test_package(
    source_dir: &std::path::Path,
    output_dir: &std::path::Path,
) -> Result<PackageManifest, PackageBuildError> {
    let config = PackageBuilderConfig {
        source_dir: source_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        package_id: "test.package".to_string(),
        version: "1.0.0".to_string(),
        name: "Test Package".to_string(),
        description: "A test package for verification".to_string(),
        author: "Test Author".to_string(),
        license: "MIT".to_string(),
        manifest_version: "1.0.0".to_string(),
        generate_sbom: true,
        sign_package: true,
        ..Default::default()
    };
    
    build_package(config).await
}

/// Corrupt a package for testing verification failure
pub async fn corrupt_package_for_testing(
    package_dir: &std::path::Path,
) -> Result<(), PackageVerifyError> {
    let config = VerificationConfig::default();
    let verifier = PackageVerifier::new(config);
    verifier.corrupt_package_for_testing(package_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_build_and_verify_package() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let source_dir = temp_dir.path().join("source");
        let output_dir = temp_dir.path().join("output");
        
        fs::create_dir(&source_dir)?;
        fs::create_dir(&output_dir)?;
        
        // Create test source files
        fs::write(source_dir.join("main.rs"), "fn main() { println!(\"Hello, World!\"); }")?;
        fs::write(source_dir.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"1.0.0\"")?;
        
        // Build package
        let manifest = create_test_package(&source_dir, &output_dir).await?;
        assert_eq!(manifest.package_id, "test.package");
        assert!(!manifest.content_hash.is_empty());
        
        // Find the built package directory
        let package_dir = output_dir.join("test.package");
        assert!(package_dir.exists());
        
        // Verify package
        let result = verify_package(&package_dir, None).await?;
        assert!(result.is_success());
        
        // Corrupt package
        corrupt_package_for_testing(&package_dir).await?;
        
        // Verify corrupted package should fail
        let result = verify_package(&package_dir, None).await?;
        assert!(!result.is_success());
        assert!(result.errors.iter().any(|e| e.contains("Content hash mismatch")));
        
        Ok(())
    }

    #[test]
    fn test_schema_serialization() {
        let manifest = PackageManifest {
            manifest_version: "1.0.0".to_string(),
            package_id: "test.package".to_string(),
            content_hash: "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef12345678".to_string(),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            expires_at: None,
            capabilities: Capabilities {
                required: Vec::new(),
                optional: Vec::new(),
                restricted: Vec::new(),
            },
            sbom: Sbom::default(),
            signatures: Vec::new(),
            metadata: Metadata {
                name: Some("Test Package".to_string()),
                description: Some("A test package".to_string()),
                version: Some("1.0.0".to_string()),
                author: Some("Test Author".to_string()),
                license: Some("MIT".to_string()),
                homepage: None,
                repository: None,
                keywords: Some(vec!["test".to_string()]),
                tags: Some(std::collections::HashMap::new()),
            },
            files: Vec::new(),
            dependencies: Vec::new(),
            security: None,
        };
        
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let deserialized: PackageManifest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.package_id, "test.package");
        assert_eq!(deserialized.manifest_version, "1.0.0");
    }
}
