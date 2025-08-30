use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use thiserror::Error;
use base64;

use crate::schema::{
    PackageManifest, Capability, Sbom, SbomComponent, SbomRelationship,
    Signature, PackageFile, PackageDependency, Checksum, Vulnerability
};

use crate::build::PackageBuildError;

/// Error types for package verification
#[derive(Error, Debug)]
pub enum PackageVerifyError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Manifest not found: {0}")]
    ManifestNotFound(PathBuf),
    
    #[error("Invalid manifest format: {0}")]
    InvalidManifestFormat(String),
    
    #[error("Content hash mismatch: expected {expected}, got {actual}")]
    ContentHashMismatch { expected: String, actual: String },
    
    #[error("File checksum mismatch for {file}: expected {expected}, got {actual}")]
    FileChecksumMismatch { file: String, expected: String, actual: String },
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("File size mismatch for {file}: expected {expected}, got {actual}")]
    FileSizeMismatch { file: String, expected: u64, actual: u64 },
    
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    
    #[error("No valid signatures found")]
    NoValidSignatures,
    
    #[error("Package expired at {0}")]
    PackageExpired(String),
    
    #[error("Package will expire soon at {0}")]
    PackageExpiringSoon(String),
    
    #[error("Capability check failed: {0}")]
    CapabilityCheckFailed(String),
    
    #[error("Dependency check failed: {0}")]
    DependencyCheckFailed(String),
    
    #[error("SBOM validation failed: {0}")]
    SbomValidationFailed(String),
    
    #[error("Security check failed: {0}")]
    SecurityCheckFailed(String),
    
    #[error("Schema validation failed: {0}")]
    SchemaValidationFailed(String),
}

/// Verification configuration
#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Verify content hash
    pub verify_content_hash: bool,
    /// Verify file checksums
    pub verify_file_checksums: bool,
    /// Verify file sizes
    pub verify_file_sizes: bool,
    /// Verify signatures
    pub verify_signatures: bool,
    /// Verify capabilities
    pub verify_capabilities: bool,
    /// Verify dependencies
    pub verify_dependencies: bool,
    /// Verify SBOM
    pub verify_sbom: bool,
    /// Verify security
    pub verify_security: bool,
    /// Check package expiration
    pub check_expiration: bool,
    /// Warning threshold for expiration (days)
    pub expiration_warning_days: u64,
    /// Strict mode (fail on warnings)
    pub strict_mode: bool,
    /// Trusted key IDs
    pub trusted_key_ids: Vec<String>,
    /// Trusted signers
    pub trusted_signers: Vec<String>,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            verify_content_hash: true,
            verify_file_checksums: true,
            verify_file_sizes: true,
            verify_signatures: true,
            verify_capabilities: true,
            verify_dependencies: true,
            verify_sbom: true,
            verify_security: true,
            check_expiration: true,
            expiration_warning_days: 30,
            strict_mode: false,
            trusted_key_ids: Vec::new(),
            trusted_signers: Vec::new(),
        }
    }
}

/// Verification result
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether verification passed
    pub success: bool,
    /// Verification warnings
    pub warnings: Vec<String>,
    /// Verification errors
    pub errors: Vec<String>,
    /// Verification details
    pub details: HashMap<String, serde_json::Value>,
}

impl VerificationResult {
    /// Create a new verification result
    pub fn new() -> Self {
        Self {
            success: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            details: HashMap::new(),
        }
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.success = false;
    }

    /// Add verification detail
    pub fn add_detail(&mut self, key: String, value: serde_json::Value) {
        self.details.insert(key, value);
    }

    /// Check if verification passed
    pub fn is_success(&self) -> bool {
        self.success && self.errors.is_empty()
    }

    /// Get summary
    pub fn summary(&self) -> String {
        if self.is_success() {
            format!("Verification PASSED with {} warnings", self.warnings.len())
        } else {
            format!("Verification FAILED with {} errors, {} warnings", 
                   self.errors.len(), self.warnings.len())
        }
    }
}

/// Package verifier for validating Polymera OS packages
pub struct PackageVerifier {
    config: VerificationConfig,
}

impl PackageVerifier {
    /// Create a new package verifier
    pub fn new(config: VerificationConfig) -> Self {
        Self { config }
    }

    /// Verify a package from a directory
    pub fn verify_package(&self, package_dir: &Path) -> Result<VerificationResult, PackageVerifyError> {
        let mut result = VerificationResult::new();

        // Load manifest
        let manifest_path = package_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(PackageVerifyError::ManifestNotFound(manifest_path));
        }

        let manifest = self.load_manifest(&manifest_path)?;
        result.add_detail("package_id".to_string(), serde_json::Value::String(manifest.package_id.clone()));
        result.add_detail("version".to_string(), serde_json::Value::String(manifest.metadata.version.clone().unwrap_or_default()));

        // Verify content hash
        if self.config.verify_content_hash {
            self.verify_content_hash(&manifest, package_dir, &mut result)?;
        }

        // Verify file checksums and sizes
        if self.config.verify_file_checksums || self.config.verify_file_sizes {
            self.verify_files(&manifest, package_dir, &mut result)?;
        }

        // Verify signatures
        if self.config.verify_signatures {
            self.verify_signatures(&manifest, &mut result)?;
        }

        // Check expiration
        if self.config.check_expiration {
            self.check_expiration(&manifest, &mut result)?;
        }

        // Verify capabilities
        if self.config.verify_capabilities {
            self.verify_capabilities(&manifest, &mut result)?;
        }

        // Verify dependencies
        if self.config.verify_dependencies {
            self.verify_dependencies(&manifest, &mut result)?;
        }

        // Verify SBOM
        if self.config.verify_sbom {
            self.verify_sbom(&manifest, &mut result)?;
        }

        // Verify security
        if self.config.verify_security {
            self.verify_security(&manifest, &mut result)?;
        }

        Ok(result)
    }

    /// Load and parse manifest
    fn load_manifest(&self, manifest_path: &Path) -> Result<PackageManifest, PackageVerifyError> {
        let manifest_content = fs::read_to_string(manifest_path)?;
        let manifest: PackageManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| PackageVerifyError::InvalidManifestFormat(e.to_string()))?;
        Ok(manifest)
    }

    /// Verify content hash
    fn verify_content_hash(
        &self,
        manifest: &PackageManifest,
        package_dir: &Path,
        result: &mut VerificationResult,
    ) -> Result<(), PackageVerifyError> {
        // Calculate actual content hash
        let actual_hash = self.calculate_content_hash(manifest)?;
        
        if actual_hash != manifest.content_hash {
            result.add_error(format!(
                "Content hash mismatch: expected {}, got {}",
                manifest.content_hash, actual_hash
            ));
        } else {
            result.add_detail("content_hash_verified".to_string(), serde_json::Value::Bool(true));
        }

        Ok(())
    }

    /// Calculate content hash
    fn calculate_content_hash(&self, manifest: &PackageManifest) -> Result<String, PackageVerifyError> {
        // Create a copy of manifest without signatures for hashing
        let mut manifest_for_hash = manifest.clone();
        manifest_for_hash.signatures.clear();

        let manifest_json = serde_json::to_string(&manifest_for_hash)?;
        let mut hasher = Sha256::new();
        hasher.update(manifest_json.as_bytes());
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Verify files
    fn verify_files(
        &self,
        manifest: &PackageManifest,
        package_dir: &Path,
        result: &mut VerificationResult,
    ) -> Result<(), PackageVerifyError> {
        let mut verified_files = 0;
        let mut total_files = manifest.files.len();

        for package_file in &manifest.files {
            let file_path = package_dir.join(&package_file.path);
            
            if !file_path.exists() {
                result.add_error(format!("File not found: {}", package_file.path));
                continue;
            }

            // Verify file size
            if self.config.verify_file_sizes {
                let actual_size = fs::metadata(&file_path)?.len();
                if actual_size != package_file.size {
                    result.add_error(format!(
                        "File size mismatch for {}: expected {}, got {}",
                        package_file.path, package_file.size, actual_size
                    ));
                    continue;
                }
            }

            // Verify file checksum
            if self.config.verify_file_checksums {
                let actual_checksum = self.calculate_file_checksum(&file_path)?;
                if actual_checksum.value != package_file.checksum.value {
                    result.add_error(format!(
                        "File checksum mismatch for {}: expected {}, got {}",
                        package_file.path, package_file.checksum.value, actual_checksum.value
                    ));
                    continue;
                }
            }

            verified_files += 1;
        }

        result.add_detail("files_verified".to_string(), serde_json::Value::Number(verified_files.into()));
        result.add_detail("total_files".to_string(), serde_json::Value::Number(total_files.into()));

        if verified_files == total_files {
            result.add_detail("all_files_verified".to_string(), serde_json::Value::Bool(true));
        }

        Ok(())
    }

    /// Calculate file checksum
    fn calculate_file_checksum(&self, file_path: &Path) -> Result<Checksum, PackageVerifyError> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let hash = hasher.finalize();
        Ok(Checksum {
            algorithm: "sha256".to_string(),
            value: format!("{:x}", hash),
        })
    }

    /// Verify signatures
    fn verify_signatures(
        &self,
        manifest: &PackageManifest,
        result: &mut VerificationResult,
    ) -> Result<(), PackageVerifyError> {
        if manifest.signatures.is_empty() {
            result.add_warning("No signatures found".to_string());
            return Ok(());
        }

        let mut valid_signatures = 0;
        let mut total_signatures = manifest.signatures.len();

        for signature in &manifest.signatures {
            if self.verify_signature(manifest, signature)? {
                valid_signatures += 1;
                
                // Check if signer is trusted
                if let Some(signer) = &signature.signer {
                    if !self.config.trusted_signers.is_empty() && !self.config.trusted_signers.contains(signer) {
                        result.add_warning(format!("Untrusted signer: {}", signer));
                    }
                }

                // Check if key ID is trusted
                if !self.config.trusted_key_ids.is_empty() && !self.config.trusted_key_ids.contains(&signature.key_id) {
                    result.add_warning(format!("Untrusted key ID: {}", signature.key_id));
                }
            } else {
                result.add_error(format!("Invalid signature from key ID: {}", signature.key_id));
            }
        }

        result.add_detail("valid_signatures".to_string(), serde_json::Value::Number(valid_signatures.into()));
        result.add_detail("total_signatures".to_string(), serde_json::Value::Number(total_signatures.into()));

        if valid_signatures == 0 {
            result.add_error("No valid signatures found".to_string());
        } else if valid_signatures < total_signatures {
            result.add_warning(format!("Only {}/{} signatures are valid", valid_signatures, total_signatures));
        }

        Ok(())
    }

    /// Verify individual signature
    fn verify_signature(
        &self,
        manifest: &PackageManifest,
        signature: &Signature,
    ) -> Result<bool, PackageVerifyError> {
        // This is a simplified implementation
        // In a real implementation, you'd use actual cryptographic verification
        
        // For now, just check if the signature format is valid
        if signature.signature.is_empty() {
            return Ok(false);
        }

        // Verify timestamp is recent
        if let Ok(timestamp) = DateTime::parse_from_rfc3339(&signature.timestamp) {
            let now = Utc::now();
            let age = now.signed_duration_since(timestamp);
            
            if age > Duration::days(365) {
                return Ok(false); // Signature too old
            }
        }

        Ok(true)
    }

    /// Check package expiration
    fn check_expiration(
        &self,
        manifest: &PackageManifest,
        result: &mut VerificationResult,
    ) -> Result<(), PackageVerifyError> {
        if let Some(expires_at) = &manifest.expires_at {
            if let Ok(expiration) = DateTime::parse_from_rfc3339(expires_at) {
                let now = Utc::now();
                
                if now > expiration {
                    result.add_error(format!("Package expired at {}", expires_at));
                } else {
                    let days_until_expiry = (expiration - now).num_days();
                    if days_until_expiry <= self.config.expiration_warning_days as i64 {
                        result.add_warning(format!(
                            "Package will expire in {} days at {}",
                            days_until_expiry, expires_at
                        ));
                    }
                    
                    result.add_detail("days_until_expiry".to_string(), serde_json::Value::Number(days_until_expiry.into()));
                }
            }
        }

        Ok(())
    }

    /// Verify capabilities
    fn verify_capabilities(
        &self,
        manifest: &PackageManifest,
        result: &mut VerificationResult,
    ) -> Result<(), PackageVerifyError> {
        let mut capability_issues = Vec::new();

        // Check required capabilities
        for capability in &manifest.capabilities.required {
            if capability.name.is_empty() {
                capability_issues.push("Required capability has empty name".to_string());
            }
            if capability.description.is_empty() {
                capability_issues.push(format!("Required capability '{}' has no description", capability.name));
            }
        }

        // Check for capability conflicts
        let required_names: std::collections::HashSet<_> = manifest.capabilities.required.iter()
            .map(|c| &c.name)
            .collect();
        let restricted_names: std::collections::HashSet<_> = manifest.capabilities.restricted.iter()
            .map(|c| &c.name)
            .collect();

        let conflicts: Vec<_> = required_names.intersection(&restricted_names).collect();
        for conflict in conflicts {
            capability_issues.push(format!("Capability '{}' is both required and restricted", conflict));
        }

        if !capability_issues.is_empty() {
            for issue in capability_issues {
                result.add_error(format!("Capability issue: {}", issue));
            }
        } else {
            result.add_detail("capabilities_valid".to_string(), serde_json::Value::Bool(true));
        }

        Ok(())
    }

    /// Verify dependencies
    fn verify_dependencies(
        &self,
        manifest: &PackageManifest,
        result: &mut VerificationResult,
    ) -> Result<(), PackageVerifyError> {
        let mut dependency_issues = Vec::new();

        for dependency in &manifest.dependencies {
            if dependency.package_id.is_empty() {
                dependency_issues.push("Dependency has empty package ID".to_string());
            }
            if dependency.version_constraint.is_empty() {
                dependency_issues.push(format!("Dependency '{}' has no version constraint", dependency.package_id));
            }
        }

        if !dependency_issues.is_empty() {
            for issue in dependency_issues {
                result.add_error(format!("Dependency issue: {}", issue));
            }
        } else {
            result.add_detail("dependencies_valid".to_string(), serde_json::Value::Bool(true));
        }

        Ok(())
    }

    /// Verify SBOM
    fn verify_sbom(
        &self,
        manifest: &PackageManifest,
        result: &mut VerificationResult,
    ) -> Result<(), PackageVerifyError> {
        let mut sbom_issues = Vec::new();

        // Check SBOM format
        if !["spdx", "cyclonedx", "swid"].contains(&manifest.sbom.format.as_str()) {
            sbom_issues.push(format!("Invalid SBOM format: {}", manifest.sbom.format));
        }

        // Check components
        if manifest.sbom.components.is_empty() {
            sbom_issues.push("SBOM has no components".to_string());
        }

        for component in &manifest.sbom.components {
            if component.name.is_empty() {
                sbom_issues.push("SBOM component has empty name".to_string());
            }
            if component.version.is_empty() {
                sbom_issues.push(format!("SBOM component '{}' has no version", component.name));
            }
        }

        if !sbom_issues.is_empty() {
            for issue in sbom_issues {
                result.add_error(format!("SBOM issue: {}", issue));
            }
        } else {
            result.add_detail("sbom_valid".to_string(), serde_json::Value::Bool(true));
        }

        Ok(())
    }

    /// Verify security
    fn verify_security(
        &self,
        manifest: &PackageManifest,
        result: &mut VerificationResult,
    ) -> Result<(), PackageVerifyError> {
        if let Some(security) = &manifest.security {
            // Check vulnerabilities
            if let Some(vulnerabilities) = &security.vulnerabilities {
                let critical_vulns: Vec<_> = vulnerabilities.iter()
                    .filter(|v| v.severity == "critical")
                    .collect();
                
                if !critical_vulns.is_empty() {
                    result.add_error(format!("Found {} critical vulnerabilities", critical_vulns.len()));
                }

                let high_vulns: Vec<_> = vulnerabilities.iter()
                    .filter(|v| v.severity == "high")
                    .collect();
                
                if !high_vulns.is_empty() {
                    result.add_warning(format!("Found {} high severity vulnerabilities", high_vulns.len()));
                }
            }

            // Check audit info
            if let Some(audit_info) = &security.audit_info {
                if audit_info.auditor.is_empty() {
                    result.add_warning("Security audit has no auditor information".to_string());
                }
            }
        }

        result.add_detail("security_checked".to_string(), serde_json::Value::Bool(true));
        Ok(())
    }

    /// Corrupt a package for testing
    pub fn corrupt_package_for_testing(&self, package_dir: &Path) -> Result<(), PackageVerifyError> {
        let manifest_path = package_dir.join("manifest.json");
        let mut manifest = self.load_manifest(&manifest_path)?;
        
        // Corrupt the content hash
        manifest.content_hash = "corrupted_hash_1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        
        // Write corrupted manifest
        let corrupted_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, corrupted_json)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_verification_config() {
        let config = VerificationConfig::default();
        assert!(config.verify_content_hash);
        assert!(config.verify_signatures);
        assert_eq!(config.expiration_warning_days, 30);
    }

    #[test]
    fn test_verification_result() {
        let mut result = VerificationResult::new();
        assert!(result.is_success());
        
        result.add_warning("Test warning".to_string());
        assert!(result.is_success());
        
        result.add_error("Test error".to_string());
        assert!(!result.is_success());
    }

    #[test]
    fn test_verification_result_summary() {
        let mut result = VerificationResult::new();
        result.add_warning("Warning 1".to_string());
        result.add_warning("Warning 2".to_string());
        
        let summary = result.summary();
        assert!(summary.contains("PASSED"));
        assert!(summary.contains("2 warnings"));
    }
}
