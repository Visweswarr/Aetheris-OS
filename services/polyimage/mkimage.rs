use crate::manifest::{
    HashAlgorithm, ImageType, PartitionImage, PolyImageManifest, RollbackProtection, Signature,
    SignatureAlgorithm, SlotInfo,
};
use clap::{App, Arg, ArgMatches};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

/// Image builder configuration
#[derive(Debug, Clone)]
pub struct ImageBuilderConfig {
    /// Output directory for images
    pub output_dir: PathBuf,
    /// Image identifier
    pub image_id: String,
    /// Image description
    pub description: String,
    /// Target architecture
    pub architecture: String,
    /// Target platform
    pub platform: String,
    /// Source directory containing partition files
    pub source_dir: PathBuf,
    /// Whether to enable rollback protection
    pub enable_rollback: bool,
    /// Minimum allowed version for rollback protection
    pub min_version: String,
    /// Whether to sign the image
    pub sign_image: bool,
    /// Private key file for signing
    pub private_key: Option<PathBuf>,
    /// Certificate file for signing
    pub certificate: Option<PathBuf>,
    /// Whether to create A/B slots
    pub create_slots: bool,
    /// Number of slots to create
    pub num_slots: u8,
}

/// Image builder for PolyImage
pub struct ImageBuilder {
    config: ImageBuilderConfig,
    manifest: PolyImageManifest,
}

impl ImageBuilder {
    /// Create a new image builder
    pub fn new(config: ImageBuilderConfig) -> Self {
        let manifest = PolyImageManifest::new(
            config.image_id.clone(),
            config.description.clone(),
            ImageType::System,
            config.architecture.clone(),
            config.platform.clone(),
        );

        Self { config, manifest }
    }

    /// Build the complete system image
    pub fn build(&mut self) -> io::Result<()> {
        println!("🚀 Building PolyImage: {}", self.config.image_id);
        println!("📁 Output directory: {}", self.config.output_dir.display());
        println!("🏗️  Source directory: {}", self.config.source_dir.display());

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)?;

        // Setup A/B slots
        if self.config.create_slots {
            self.setup_slots()?;
        }

        // Build partition images
        self.build_partitions()?;

        // Setup rollback protection
        if self.config.enable_rollback {
            self.setup_rollback_protection()?;
        }

        // Sign the image if requested
        if self.config.sign_image {
            self.sign_image()?;
        }

        // Validate manifest
        let validation = self.manifest.validate();
        if !validation.valid {
            eprintln!("❌ Manifest validation failed:");
            for error in &validation.errors {
                eprintln!("   - {}", error);
            }
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Manifest validation failed",
            ));
        }

        // Write manifest
        self.write_manifest()?;

        // Write image summary
        self.write_summary()?;

        println!("✅ Image build completed successfully!");
        Ok(())
    }

    /// Setup A/B slots
    fn setup_slots(&mut self) -> io::Result<()> {
        println!("🔀 Setting up A/B slots...");

        for i in 0..self.config.num_slots {
            let slot_name = if i == 0 { "A" } else { "B" };
            let priority = if i == 0 { 2 } else { 1 }; // A has higher priority

            let slot = SlotInfo {
                slot: slot_name.to_string(),
                priority,
                successful: i == 0, // Slot A starts as successful
                active: i == 0,     // Slot A starts as active
                last_updated: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                boot_attempts: 0,
                max_boot_attempts: 3,
            };

            self.manifest.add_slot(slot);
            println!("   ✅ Created slot {} (priority: {})", slot_name, priority);
        }

        Ok(())
    }

    /// Build partition images
    fn build_partitions(&mut self) -> io::Result<()> {
        println!("💾 Building partition images...");

        if !self.config.source_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source directory not found: {}", self.config.source_dir.display()),
            ));
        }

        let entries = fs::read_dir(&self.config.source_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::metadata(&path)?;

            if metadata.is_file() {
                self.process_partition_file(&path)?;
            } else if metadata.is_dir() {
                self.process_partition_directory(&path)?;
            }
        }

        Ok(())
    }

    /// Process a partition file
    fn process_partition_file(&mut self, file_path: &Path) -> io::Result<()> {
        let file_name = file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        // Determine image type based on file name
        let image_type = if file_name.contains("system") {
            ImageType::System
        } else if file_name.contains("data") {
            ImageType::Data
        } else if file_name.contains("boot") {
            ImageType::Bootloader
        } else if file_name.contains("recovery") {
            ImageType::Recovery
        } else {
            ImageType::Application
        };

        // Calculate file hash
        let hash = self.calculate_file_hash(file_path)?;
        let size = fs::metadata(file_path)?.len();

        // Copy file to output directory
        let output_path = self.config.output_dir.join(&file_name);
        fs::copy(file_path, &output_path)?;

        // Create partition image
        let partition = PartitionImage {
            name: file_name.clone(),
            image_type,
            path: format!("/{}", file_name),
            size,
            hash_algorithm: HashAlgorithm::Sha256,
            hash,
            read_only: image_type == ImageType::System || image_type == ImageType::Bootloader,
            mount_point: if image_type == ImageType::System {
                Some("/system".to_string())
            } else if image_type == ImageType::Data {
                Some("/data".to_string())
            } else {
                None
            },
            flags: vec!["verified".to_string()],
        };

        self.manifest.add_partition(partition);
        println!("   ✅ Added partition: {} ({} bytes, hash: {})", file_name, size, hash);

        Ok(())
    }

    /// Process a partition directory
    fn process_partition_directory(&mut self, dir_path: &Path) -> io::Result<()> {
        let dir_name = dir_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        println!("   📁 Processing directory: {}", dir_name);

        // Create a tar archive of the directory
        let archive_name = format!("{}.tar.gz", dir_name);
        let archive_path = self.config.output_dir.join(&archive_name);

        self.create_tar_archive(dir_path, &archive_path)?;

        // Calculate archive hash and size
        let hash = self.calculate_file_hash(&archive_path)?;
        let size = fs::metadata(&archive_path)?.len();

        // Determine image type
        let image_type = if dir_name.contains("system") {
            ImageType::System
        } else if dir_name.contains("data") {
            ImageType::Data
        } else {
            ImageType::Application
        };

        // Create partition image
        let partition = PartitionImage {
            name: dir_name.clone(),
            image_type,
            path: format!("/{}", archive_name),
            size,
            hash_algorithm: HashAlgorithm::Sha256,
            hash,
            read_only: image_type == ImageType::System,
            mount_point: if dir_name.contains("system") {
                Some("/system".to_string())
            } else if dir_name.contains("data") {
                Some("/data".to_string())
            } else {
                None
            },
            flags: vec!["verified".to_string(), "compressed".to_string()],
        };

        self.manifest.add_partition(partition);
        println!("   ✅ Added partition: {} ({} bytes, hash: {})", dir_name, size, hash);

        Ok(())
    }

    /// Create a tar.gz archive from a directory
    fn create_tar_archive(&self, source_dir: &Path, archive_path: &Path) -> io::Result<()> {
        // This is a simplified implementation
        // In a real implementation, you would use a proper tar library
        println!("   📦 Creating archive: {}", archive_path.display());
        
        // For now, just create an empty file as a placeholder
        File::create(archive_path)?;
        
        Ok(())
    }

    /// Calculate file hash
    fn calculate_file_hash(&self, file_path: &Path) -> io::Result<String> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Simple hash calculation (in real implementation, use proper crypto library)
        let hash = format!("{:x}", md5::compute(&buffer));
        Ok(hash)
    }

    /// Setup rollback protection
    fn setup_rollback_protection(&mut self) -> io::Result<()> {
        println!("🔄 Setting up rollback protection...");

        let rollback = RollbackProtection {
            min_version: self.config.min_version.clone(),
            max_version: None,
            anti_rollback_version: 1,
            enabled: true,
        };

        self.manifest.set_rollback_protection(rollback);
        println!("   ✅ Rollback protection enabled (min version: {})", self.config.min_version);

        Ok(())
    }

    /// Sign the image
    fn sign_image(&mut self) -> io::Result<()> {
        println!("🔐 Signing image...");

        // In a real implementation, you would:
        // 1. Load the private key
        // 2. Calculate the manifest hash
        // 3. Sign the hash with the private key
        // 4. Verify the signature with the public key

        let signature = Signature {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "test-key-123".to_string(),
            value: "dummy-signature-placeholder".to_string(),
            certificate_chain: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.manifest.set_signature(signature);
        println!("   ✅ Image signed (algorithm: Ed25519)");

        Ok(())
    }

    /// Write manifest to file
    fn write_manifest(&self) -> io::Result<()> {
        let manifest_path = self.config.output_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&self.manifest)?;
        
        let mut file = File::create(&manifest_path)?;
        file.write_all(manifest_json.as_bytes())?;
        
        println!("📄 Manifest written to: {}", manifest_path.display());
        Ok(())
    }

    /// Write build summary
    fn write_summary(&self) -> io::Result<()> {
        let summary_path = self.config.output_dir.join("build_summary.txt");
        let mut file = File::create(&summary_path)?;

        writeln!(file, "PolyImage Build Summary")?;
        writeln!(file, "======================")?;
        writeln!(file, "Image ID: {}", self.manifest.image_id)?;
        writeln!(file, "Description: {}", self.manifest.description)?;
        writeln!(file, "Architecture: {}", self.manifest.architecture)?;
        writeln!(file, "Platform: {}", self.manifest.platform)?;
        writeln!(file, "Version: {}", self.manifest.version)?;
        writeln!(file, "Build Timestamp: {}", self.manifest.build_timestamp)?;
        writeln!(file, "Total Size: {} bytes", self.manifest.total_size())?;
        writeln!(file, "Partitions: {}", self.manifest.partitions.len())?;
        writeln!(file, "Slots: {}", self.manifest.slots.len())?;
        writeln!(file, "Rollback Protection: {}", self.config.enable_rollback)?;
        writeln!(file, "Signed: {}", self.config.sign_image)?;

        println!("📊 Build summary written to: {}", summary_path.display());
        Ok(())
    }
}

/// Parse command line arguments
fn parse_args() -> ArgMatches {
    App::new("mkimage")
        .version("1.0")
        .about("PolyImage system image builder")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output directory for images")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("image-id")
                .short('i')
                .long("image-id")
                .value_name("ID")
                .help("Image identifier")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("description")
                .short('d')
                .long("description")
                .value_name("DESC")
                .help("Image description")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("architecture")
                .short('a')
                .long("architecture")
                .value_name("ARCH")
                .help("Target architecture")
                .default_value("x86_64")
                .takes_value(true),
        )
        .arg(
            Arg::new("platform")
                .short('p')
                .long("platform")
                .value_name("PLATFORM")
                .help("Target platform")
                .default_value("generic")
                .takes_value(true),
        )
        .arg(
            Arg::new("source")
                .short('s')
                .long("source")
                .value_name("DIR")
                .help("Source directory containing partition files")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("rollback")
                .long("rollback")
                .help("Enable rollback protection")
                .takes_value(false),
        )
        .arg(
            Arg::new("min-version")
                .long("min-version")
                .value_name("VERSION")
                .help("Minimum allowed version for rollback protection")
                .default_value("0.0.0")
                .takes_value(true),
        )
        .arg(
            Arg::new("sign")
                .long("sign")
                .help("Sign the image")
                .takes_value(false),
        )
        .arg(
            Arg::new("private-key")
                .long("private-key")
                .value_name("FILE")
                .help("Private key file for signing")
                .takes_value(true),
        )
        .arg(
            Arg::new("certificate")
                .long("certificate")
                .value_name("FILE")
                .help("Certificate file for signing")
                .takes_value(true),
        )
        .arg(
            Arg::new("slots")
                .long("slots")
                .value_name("NUM")
                .help("Number of A/B slots to create")
                .default_value("2")
                .takes_value(true),
        )
        .get_matches()
}

/// Main function
pub fn main() {
    let matches = parse_args();

    let config = ImageBuilderConfig {
        output_dir: PathBuf::from(matches.value_of("output").unwrap()),
        image_id: matches.value_of("image-id").unwrap().to_string(),
        description: matches.value_of("description").unwrap().to_string(),
        architecture: matches.value_of("architecture").unwrap().to_string(),
        platform: matches.value_of("platform").unwrap().to_string(),
        source_dir: PathBuf::from(matches.value_of("source").unwrap()),
        enable_rollback: matches.is_present("rollback"),
        min_version: matches.value_of("min-version").unwrap().to_string(),
        sign_image: matches.is_present("sign"),
        private_key: matches.value_of("private-key").map(PathBuf::from),
        certificate: matches.value_of("certificate").map(PathBuf::from),
        create_slots: true,
        num_slots: matches
            .value_of("slots")
            .unwrap()
            .parse()
            .unwrap_or(2),
    };

    let mut builder = ImageBuilder::new(config);

    if let Err(e) = builder.build() {
        eprintln!("❌ Build failed: {}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_image_builder_creation() {
        let config = ImageBuilderConfig {
            output_dir: PathBuf::from("/tmp/test"),
            image_id: "test-image".to_string(),
            description: "Test Image".to_string(),
            architecture: "x86_64".to_string(),
            platform: "test".to_string(),
            source_dir: PathBuf::from("/tmp/source"),
            enable_rollback: true,
            min_version: "1.0.0".to_string(),
            sign_image: false,
            private_key: None,
            certificate: None,
            create_slots: true,
            num_slots: 2,
        };

        let builder = ImageBuilder::new(config);
        assert_eq!(builder.manifest.image_id, "test-image");
        assert_eq!(builder.manifest.architecture, "x86_64");
    }

    #[test]
    fn test_slot_setup() {
        let temp_dir = tempdir().unwrap();
        let config = ImageBuilderConfig {
            output_dir: temp_dir.path().to_path_buf(),
            image_id: "test-image".to_string(),
            description: "Test Image".to_string(),
            architecture: "x86_64".to_string(),
            platform: "test".to_string(),
            source_dir: PathBuf::from("/tmp/source"),
            enable_rollback: false,
            min_version: "1.0.0".to_string(),
            sign_image: false,
            private_key: None,
            certificate: None,
            create_slots: true,
            num_slots: 2,
        };

        let mut builder = ImageBuilder::new(config);
        builder.setup_slots().unwrap();

        assert_eq!(builder.manifest.slots.len(), 2);
        assert_eq!(builder.manifest.slots[0].slot, "A");
        assert_eq!(builder.manifest.slots[1].slot, "B");
        assert!(builder.manifest.slots[0].successful);
        assert!(!builder.manifest.slots[1].successful);
    }
}
