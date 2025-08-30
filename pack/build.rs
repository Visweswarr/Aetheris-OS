use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use thiserror::Error;
use walkdir::WalkDir;

use crate::schema::{
    PackageManifest, Capability, Sbom, SbomComponent, SbomRelationship,
    Signature, PackageFile, PackageDependency, Checksum, Vulnerability
};

mod schema;

/// Error types for package building
#[derive(Error, Debug)]
pub enum PackageBuildError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid package ID: {0}")]
    InvalidPackageId(String),
    
    #[error("Invalid manifest version: {0}")]
    InvalidManifestVersion(String),
    
    #[error("No source files found")]
    NoSourceFiles,
    
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Invalid file path: {0}")]
    InvalidFilePath(PathBuf),
    
    #[error("Checksum mismatch for file: {0}")]
    ChecksumMismatch(PathBuf),
    
    #[error("Signature error: {0}")]
    SignatureError(String),
    
    #[error("SBOM generation error: {0}")]
    SbomError(String),
}

/// Package builder configuration
#[derive(Debug, Clone)]
pub struct PackageBuilderConfig {
    /// Source directory containing files to package
    pub source_dir: PathBuf,
    /// Output directory for the package
    pub output_dir: PathBuf,
    /// Package ID
    pub package_id: String,
    /// Package version
    pub version: String,
    /// Package name
    pub name: String,
    /// Package description
    pub description: String,
    /// Package author
    pub author: String,
    /// Package license
    pub license: String,
    /// Manifest version
    pub manifest_version: String,
    /// Include hidden files
    pub include_hidden: bool,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// Maximum file size (in bytes)
    pub max_file_size: Option<u64>,
    /// Generate SBOM
    pub generate_sbom: bool,
    /// SBOM format
    pub sbom_format: String,
    /// Sign package
    pub sign_package: bool,
    /// Signing key path
    pub signing_key_path: Option<PathBuf>,
    /// Signing algorithm
    pub signing_algorithm: String,
    /// Key ID
    pub key_id: Option<String>,
}

impl Default for PackageBuilderConfig {
    fn default() -> Self {
        Self {
            source_dir: PathBuf::from("."),
            output_dir: PathBuf::from("dist"),
            package_id: "example.package".to_string(),
            version: "1.0.0".to_string(),
            name: "Example Package".to_string(),
            description: "An example package".to_string(),
            author: "Unknown Author".to_string(),
            license: "MIT".to_string(),
            manifest_version: "1.0.0".to_string(),
            include_hidden: false,
            exclude_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
            ],
            include_patterns: vec!["*".to_string()],
            max_file_size: Some(100 * 1024 * 1024), // 100MB
            generate_sbom: true,
            sbom_format: "spdx".to_string(),
            sign_package: false,
            signing_key_path: None,
            signing_algorithm: "ed25519".to_string(),
            key_id: None,
        }
    }
}

/// Package builder for creating Polymera OS packages
pub struct PackageBuilder {
    config: PackageBuilderConfig,
    capabilities: Vec<Capability>,
    dependencies: Vec<PackageDependency>,
    custom_metadata: HashMap<String, serde_json::Value>,
}

impl PackageBuilder {
    /// Create a new package builder
    pub fn new(config: PackageBuilderConfig) -> Self {
        Self {
            config,
            capabilities: Vec::new(),
            dependencies: Vec::new(),
            custom_metadata: HashMap::new(),
        }
    }

    /// Add a capability requirement
    pub fn add_capability(&mut self, capability: Capability) {
        self.capabilities.push(capability);
    }

    /// Add a package dependency
    pub fn add_dependency(&mut self, dependency: PackageDependency) {
        self.dependencies.push(dependency);
    }

    /// Add custom metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.custom_metadata.insert(key, value);
    }

    /// Build the package
    pub fn build(&self) -> Result<PackageManifest, PackageBuildError> {
        // Validate configuration
        self.validate_config()?;

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)?;

        // Collect source files
        let source_files = self.collect_source_files()?;
        if source_files.is_empty() {
            return Err(PackageBuildError::NoSourceFiles);
        }

        // Create package directory
        let package_dir = self.config.output_dir.join(&self.config.package_id);
        fs::create_dir_all(&package_dir)?;

        // Copy files to package directory
        let package_files = self.copy_files_to_package(&source_files, &package_dir)?;

        // Generate SBOM
        let sbom = if self.config.generate_sbom {
            self.generate_sbom(&source_files)?
        } else {
            Sbom::default()
        };

        // Create manifest
        let mut manifest = PackageManifest {
            manifest_version: self.config.manifest_version.clone(),
            package_id: self.config.package_id.clone(),
            content_hash: String::new(), // Will be calculated
            created_at: Utc::now().to_rfc3339(),
            expires_at: None,
            capabilities: self.build_capabilities(),
            sbom,
            signatures: Vec::new(), // Will be added after signing
            metadata: self.build_metadata(),
            files: package_files,
            dependencies: self.dependencies.clone(),
            security: None,
        };

        // Calculate content hash
        manifest.content_hash = self.calculate_content_hash(&manifest)?;

        // Sign package if enabled
        if self.config.sign_package {
            let signature = self.sign_manifest(&manifest)?;
            manifest.signatures.push(signature);
        }

        // Write manifest
        let manifest_path = package_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json)?;

        // Create package archive
        let archive_path = self.config.output_dir.join(format!("{}.pkg", self.config.package_id));
        self.create_package_archive(&package_dir, &archive_path)?;

        // Clean up package directory
        fs::remove_dir_all(&package_dir)?;

        Ok(manifest)
    }

    /// Validate builder configuration
    fn validate_config(&self) -> Result<(), PackageBuildError> {
        if !self.config.source_dir.exists() {
            return Err(PackageBuildError::FileNotFound(self.config.source_dir.clone()));
        }

        if !self.config.source_dir.is_dir() {
            return Err(PackageBuildError::InvalidFilePath(self.config.source_dir.clone()));
        }

        // Validate package ID format
        if !self.is_valid_package_id(&self.config.package_id) {
            return Err(PackageBuildError::InvalidPackageId(self.config.package_id.clone()));
        }

        // Validate manifest version format
        if !self.is_valid_version(&self.config.manifest_version) {
            return Err(PackageBuildError::InvalidManifestVersion(self.config.manifest_version.clone()));
        }

        Ok(())
    }

    /// Check if package ID is valid
    fn is_valid_package_id(&self, package_id: &str) -> bool {
        let pattern = regex::Regex::new(r"^[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)*$").unwrap();
        pattern.is_match(package_id)
    }

    /// Check if version is valid
    fn is_valid_version(&self, version: &str) -> bool {
        let pattern = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
        pattern.is_match(version)
    }

    /// Collect source files
    fn collect_source_files(&self) -> Result<Vec<PathBuf>, PackageBuildError> {
        let mut files = Vec::new();
        let source_dir = &self.config.source_dir;

        for entry in WalkDir::new(source_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            // Skip if not a file
            if !path.is_file() {
                continue;
            }

            // Skip hidden files if not included
            if !self.config.include_hidden && self.is_hidden_file(path) {
                continue;
            }

            // Check exclude patterns
            if self.should_exclude_file(path) {
                continue;
            }

            // Check include patterns
            if !self.should_include_file(path) {
                continue;
            }

            // Check file size
            if let Some(max_size) = self.config.max_file_size {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() > max_size {
                        continue;
                    }
                }
            }

            files.push(path.to_path_buf());
        }

        Ok(files)
    }

    /// Check if file should be hidden
    fn is_hidden_file(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }

    /// Check if file should be excluded
    fn should_exclude_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.config.exclude_patterns.iter().any(|pattern| {
            if pattern.contains('*') {
                // Simple glob pattern matching
                let regex_pattern = pattern.replace('*', ".*");
                if let Ok(regex) = regex::Regex::new(&regex_pattern) {
                    regex.is_match(&path_str)
                } else {
                    false
                }
            } else {
                path_str.contains(pattern)
            }
        })
    }

    /// Check if file should be included
    fn should_include_file(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.config.include_patterns.iter().any(|pattern| {
            if pattern == "*" {
                true
            } else if pattern.contains('*') {
                let regex_pattern = pattern.replace('*', ".*");
                if let Ok(regex) = regex::Regex::new(&regex_pattern) {
                    regex.is_match(&path_str)
                } else {
                    false
                }
            } else {
                path_str.contains(pattern)
            }
        })
    }

    /// Copy files to package directory
    fn copy_files_to_package(
        &self,
        source_files: &[PathBuf],
        package_dir: &Path,
    ) -> Result<Vec<PackageFile>, PackageBuildError> {
        let mut package_files = Vec::new();

        for source_file in source_files {
            let relative_path = source_file.strip_prefix(&self.config.source_dir)
                .map_err(|_| PackageBuildError::InvalidFilePath(source_file.clone()))?;
            
            let package_path = package_dir.join(relative_path);
            
            // Create parent directories
            if let Some(parent) = package_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy file
            fs::copy(source_file, &package_path)?;

            // Calculate checksum
            let checksum = self.calculate_file_checksum(source_file)?;

            // Get file metadata
            let metadata = fs::metadata(source_file)?;
            let permissions = self.get_file_permissions(&metadata);
            let mime_type = self.get_mime_type(source_file);

            let package_file = PackageFile {
                path: relative_path.to_string_lossy().to_string(),
                size: metadata.len(),
                checksum,
                permissions: Some(permissions),
                mime_type: Some(mime_type),
                executable: Some(self.is_executable(&metadata)),
            };

            package_files.push(package_file);
        }

        Ok(package_files)
    }

    /// Calculate file checksum
    fn calculate_file_checksum(&self, file_path: &Path) -> Result<Checksum, PackageBuildError> {
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

    /// Get file permissions string
    fn get_file_permissions(&self, metadata: &fs::Metadata) -> String {
        // This is a simplified implementation
        // In a real implementation, you'd extract actual Unix permissions
        if self.is_executable(metadata) {
            "rwxr-xr-x".to_string()
        } else {
            "rw-r--r--".to_string()
        }
    }

    /// Check if file is executable
    fn is_executable(&self, metadata: &fs::Metadata) -> bool {
        // This is a simplified implementation
        // In a real implementation, you'd check actual Unix permissions
        metadata.permissions().readonly()
    }

    /// Get MIME type
    fn get_mime_type(&self, path: &Path) -> String {
        if let Some(extension) = path.extension() {
            match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                "rs" => "text/x-rust".to_string(),
                "toml" => "text/x-toml".to_string(),
                "md" => "text/markdown".to_string(),
                "json" => "application/json".to_string(),
                "txt" => "text/plain".to_string(),
                "bin" => "application/octet-stream".to_string(),
                _ => "application/octet-stream".to_string(),
            }
        } else {
            "application/octet-stream".to_string()
        }
    }

    /// Generate SBOM
    fn generate_sbom(&self, source_files: &[PathBuf]) -> Result<Sbom, PackageBuildError> {
        let mut components = Vec::new();

        // Add main package component
        let main_component = SbomComponent {
            name: self.config.package_id.clone(),
            version: self.config.version.clone(),
            component_type: "application".to_string(),
            purl: Some(format!("pkg:polymera/{}@{}", self.config.package_id, self.config.version)),
            cpe: None,
            license: Some(self.config.license.clone()),
            supplier: Some(self.config.author.clone()),
            description: Some(self.config.description.clone()),
            checksums: vec![
                Checksum {
                    algorithm: "sha256".to_string(),
                    value: "placeholder".to_string(), // Will be updated
                }
            ],
        };
        components.push(main_component);

        // Add file components
        for file_path in source_files {
            if let Some(extension) = file_path.extension() {
                let component = SbomComponent {
                    name: file_path.file_name().unwrap().to_string_lossy().to_string(),
                    version: "1.0.0".to_string(),
                    component_type: "file".to_string(),
                    purl: None,
                    cpe: None,
                    license: None,
                    supplier: None,
                    description: Some(format!("Source file: {}", file_path.display())),
                    checksums: vec![self.calculate_file_checksum(file_path)?],
                };
                components.push(component);
            }
        }

        let relationships = vec![
            SbomRelationship {
                from: self.config.package_id.clone(),
                to: "polymera-os".to_string(),
                relationship_type: "depends_on".to_string(),
                description: Some("Depends on Polymera OS runtime".to_string()),
            }
        ];

        Ok(Sbom {
            format: self.config.sbom_format.clone(),
            version: "1.0".to_string(),
            components,
            relationships: Some(relationships),
        })
    }

    /// Build capabilities structure
    fn build_capabilities(&self) -> schema::Capabilities {
        let required = self.capabilities.clone();
        let optional = Vec::new();
        let restricted = Vec::new();

        schema::Capabilities {
            required,
            optional,
            restricted,
        }
    }

    /// Build metadata
    fn build_metadata(&self) -> schema::Metadata {
        let mut metadata = schema::Metadata {
            name: Some(self.config.name.clone()),
            description: Some(self.config.description.clone()),
            version: Some(self.config.version.clone()),
            author: Some(self.config.author.clone()),
            license: Some(self.config.license.clone()),
            homepage: None,
            repository: None,
            keywords: Some(vec![
                "polymera".to_string(),
                "os".to_string(),
                "package".to_string(),
            ]),
            tags: Some(HashMap::new()),
        };

        // Add custom metadata
        for (key, value) in &self.custom_metadata {
            match key.as_str() {
                "homepage" => metadata.homepage = value.as_str().map(|s| s.to_string()),
                "repository" => metadata.repository = value.as_str().map(|s| s.to_string()),
                _ => {
                    if let Some(tags) = &mut metadata.tags {
                        tags.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        metadata
    }

    /// Calculate content hash
    fn calculate_content_hash(&self, manifest: &PackageManifest) -> Result<String, PackageBuildError> {
        // Create a copy of manifest without signatures for hashing
        let mut manifest_for_hash = manifest.clone();
        manifest_for_hash.signatures.clear();

        let manifest_json = serde_json::to_string(&manifest_for_hash)?;
        let mut hasher = Sha256::new();
        hasher.update(manifest_json.as_bytes());
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Sign manifest
    fn sign_manifest(&self, manifest: &PackageManifest) -> Result<Signature, PackageBuildError> {
        // This is a simplified implementation
        // In a real implementation, you'd use actual cryptographic signing
        
        let manifest_json = serde_json::to_string(manifest)?;
        let mut hasher = Sha256::new();
        hasher.update(manifest_json.as_bytes());
        let hash = hasher.finalize();

        let signature = base64::encode(format!("{:x}", hash));

        Ok(Signature {
            algorithm: self.config.signing_algorithm.clone(),
            key_id: self.config.key_id.clone().unwrap_or_else(|| "default".to_string()),
            signature,
            timestamp: Utc::now().to_rfc3339(),
            signer: Some(self.config.author.clone()),
            certificate: None,
        })
    }

    /// Create package archive
    fn create_package_archive(&self, package_dir: &Path, archive_path: &Path) -> Result<(), PackageBuildError> {
        // This is a simplified implementation
        // In a real implementation, you'd create a proper archive format
        
        // For now, just copy the manifest
        let manifest_source = package_dir.join("manifest.json");
        let manifest_dest = archive_path.with_extension("manifest.json");
        fs::copy(manifest_source, manifest_dest)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_package_builder_config() {
        let config = PackageBuilderConfig::default();
        assert_eq!(config.package_id, "example.package");
        assert_eq!(config.version, "1.0.0");
        assert!(config.generate_sbom);
    }

    #[test]
    fn test_package_id_validation() {
        let builder = PackageBuilder::new(PackageBuilderConfig::default());
        
        assert!(builder.is_valid_package_id("core.kernel"));
        assert!(builder.is_valid_package_id("services.attest"));
        assert!(builder.is_valid_package_id("crypto-liboqs"));
        assert!(!builder.is_valid_package_id("invalid package"));
        assert!(!builder.is_valid_package_id(".hidden"));
    }

    #[test]
    fn test_version_validation() {
        let builder = PackageBuilder::new(PackageBuilderConfig::default());
        
        assert!(builder.is_valid_version("1.0.0"));
        assert!(builder.is_valid_version("2.1.0"));
        assert!(!builder.is_valid_version("1.0"));
        assert!(!builder.is_valid_version("invalid"));
    }

    #[test]
    fn test_file_collection() -> Result<(), PackageBuildError> {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        fs::create_dir(&source_dir)?;

        // Create test files
        fs::write(source_dir.join("test.rs"), "fn main() {}")?;
        fs::write(source_dir.join("Cargo.toml"), "[package]")?;
        fs::write(source_dir.join(".hidden"), "hidden")?;

        let mut config = PackageBuilderConfig::default();
        config.source_dir = source_dir.clone();
        config.output_dir = temp_dir.path().join("output");

        let builder = PackageBuilder::new(config);
        let files = builder.collect_source_files()?;

        // Should include source files but not hidden files
        assert!(files.iter().any(|f| f.ends_with("test.rs")));
        assert!(files.iter().any(|f| f.ends_with("Cargo.toml")));
        assert!(!files.iter().any(|f| f.ends_with(".hidden")));

        Ok(())
    }
}
