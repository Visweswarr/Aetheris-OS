use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// PolyImage manifest schema version
pub const MANIFEST_SCHEMA_VERSION: &str = "1.0.0";

/// Supported image types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImageType {
    /// Read-only system image
    System,
    /// Read-write data partition
    Data,
    /// Bootloader image
    Bootloader,
    /// Recovery image
    Recovery,
    /// User application bundle
    Application,
}

/// Supported hash algorithms for image verification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

/// Supported signature algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SignatureAlgorithm {
    Ed25519,
    EcdsaP256,
    Rsa2048,
    Rsa4096,
}

/// A/B slot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInfo {
    /// Slot identifier (A or B)
    pub slot: String,
    /// Slot priority (higher number = higher priority)
    pub priority: u8,
    /// Whether this slot is marked as successful
    pub successful: bool,
    /// Whether this slot is marked as active
    pub active: bool,
    /// Timestamp when slot was last updated
    pub last_updated: u64,
    /// Number of boot attempts for this slot
    pub boot_attempts: u32,
    /// Maximum allowed boot attempts before marking as failed
    pub max_boot_attempts: u32,
}

/// Image metadata for a specific partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionImage {
    /// Partition name/identifier
    pub name: String,
    /// Image type
    pub image_type: ImageType,
    /// File path within the image
    pub path: String,
    /// Image size in bytes
    pub size: u64,
    /// Hash algorithm used
    pub hash_algorithm: HashAlgorithm,
    /// Image hash value
    pub hash: String,
    /// Whether this partition is read-only
    pub read_only: bool,
    /// Partition mount point (if applicable)
    pub mount_point: Option<String>,
    /// Partition flags
    pub flags: Vec<String>,
}

/// Rollback protection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackProtection {
    /// Minimum allowed version for this image
    pub min_version: String,
    /// Maximum allowed version for this image
    pub max_version: Option<String>,
    /// Anti-rollback version counter
    pub anti_rollback_version: u64,
    /// Whether rollback protection is enabled
    pub enabled: bool,
}

/// Digital signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signature algorithm used
    pub algorithm: SignatureAlgorithm,
    /// Public key identifier
    pub key_id: String,
    /// Signature value (base64 encoded)
    pub value: String,
    /// Certificate chain (if applicable)
    pub certificate_chain: Option<Vec<String>>,
    /// Timestamp when signature was created
    pub timestamp: u64,
}

/// PolyImage manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyImageManifest {
    /// Schema version
    #[serde(rename = "$schema")]
    pub schema: String,
    /// Manifest version
    pub version: String,
    /// Image identifier
    pub image_id: String,
    /// Image description
    pub description: String,
    /// Image type
    pub image_type: ImageType,
    /// Target architecture
    pub architecture: String,
    /// Target platform
    pub platform: String,
    /// Build timestamp
    pub build_timestamp: u64,
    /// Build host information
    pub build_host: String,
    /// Build user information
    pub build_user: String,
    /// Source code revision
    pub source_revision: String,
    /// Source code branch
    pub source_branch: String,
    /// A/B slot information
    pub slots: Vec<SlotInfo>,
    /// Partition images
    pub partitions: Vec<PartitionImage>,
    /// Rollback protection settings
    pub rollback_protection: RollbackProtection,
    /// Digital signature
    pub signature: Signature,
    /// Additional metadata
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

/// Manifest validation result
#[derive(Debug, Clone)]
pub struct ManifestValidation {
    /// Whether the manifest is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Image verification result
#[derive(Debug, Clone)]
pub struct ImageVerification {
    /// Whether the image is valid
    pub valid: bool,
    /// Verification errors
    pub errors: Vec<String>,
    /// Verification warnings
    pub warnings: Vec<String>,
    /// Hash verification result
    pub hash_valid: bool,
    /// Signature verification result
    pub signature_valid: bool,
    /// Rollback protection check result
    pub rollback_valid: bool,
}

impl PolyImageManifest {
    /// Create a new manifest
    pub fn new(
        image_id: String,
        description: String,
        image_type: ImageType,
        architecture: String,
        platform: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            schema: format!("https://polymera-os.org/schemas/manifest-{}.json", MANIFEST_SCHEMA_VERSION),
            version: "1.0.0".to_string(),
            image_id,
            description,
            image_type,
            architecture,
            platform,
            build_timestamp: now,
            build_host: std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
            build_user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            source_revision: "unknown".to_string(),
            source_branch: "unknown".to_string(),
            slots: Vec::new(),
            partitions: Vec::new(),
            rollback_protection: RollbackProtection {
                min_version: "0.0.0".to_string(),
                max_version: None,
                anti_rollback_version: 1,
                enabled: true,
            },
            signature: Signature {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "unknown".to_string(),
                value: "".to_string(),
                certificate_chain: None,
                timestamp: now,
            },
            additional: HashMap::new(),
        }
    }

    /// Add A/B slot information
    pub fn add_slot(&mut self, slot: SlotInfo) {
        self.slots.push(slot);
    }

    /// Add partition image
    pub fn add_partition(&mut self, partition: PartitionImage) {
        self.partitions.push(partition);
    }

    /// Set rollback protection
    pub fn set_rollback_protection(&mut self, rollback: RollbackProtection) {
        self.rollback_protection = rollback;
    }

    /// Set digital signature
    pub fn set_signature(&mut self, signature: Signature) {
        self.signature = signature;
    }

    /// Add additional metadata
    pub fn add_metadata(&mut self, key: String, value: Value) {
        self.additional.insert(key, value);
    }

    /// Validate manifest structure
    pub fn validate(&self) -> ManifestValidation {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check required fields
        if self.image_id.is_empty() {
            errors.push("image_id cannot be empty".to_string());
        }

        if self.description.is_empty() {
            errors.push("description cannot be empty".to_string());
        }

        if self.architecture.is_empty() {
            errors.push("architecture cannot be empty".to_string());
        }

        if self.platform.is_empty() {
            errors.push("platform cannot be empty".to_string());
        }

        // Check slots
        if self.slots.is_empty() {
            warnings.push("no A/B slots defined".to_string());
        } else {
            let slot_names: Vec<&String> = self.slots.iter().map(|s| &s.slot).collect();
            if slot_names.len() != slot_names.iter().collect::<std::collections::HashSet<_>>().len() {
                errors.push("duplicate slot names found".to_string());
            }
        }

        // Check partitions
        if self.partitions.is_empty() {
            warnings.push("no partitions defined".to_string());
        } else {
            let partition_names: Vec<&String> = self.partitions.iter().map(|p| &p.name).collect();
            if partition_names.len() != partition_names.iter().collect::<std::collections::HashSet<_>>().len() {
                errors.push("duplicate partition names found".to_string());
            }
        }

        // Check rollback protection
        if self.rollback_protection.enabled {
            if self.rollback_protection.min_version.is_empty() {
                errors.push("min_version cannot be empty when rollback protection is enabled".to_string());
            }
        }

        // Check signature
        if self.signature.value.is_empty() {
            warnings.push("signature value is empty".to_string());
        }

        let valid = errors.is_empty();

        ManifestValidation {
            valid,
            errors,
            warnings,
        }
    }

    /// Get total image size
    pub fn total_size(&self) -> u64 {
        self.partitions.iter().map(|p| p.size).sum()
    }

    /// Get partition by name
    pub fn get_partition(&self, name: &str) -> Option<&PartitionImage> {
        self.partitions.iter().find(|p| p.name == name)
    }

    /// Get slot by name
    pub fn get_slot(&self, name: &str) -> Option<&SlotInfo> {
        self.slots.iter().find(|s| s.slot == name)
    }

    /// Check if rollback is allowed
    pub fn is_rollback_allowed(&self, target_version: &str) -> bool {
        if !self.rollback_protection.enabled {
            return true;
        }

        // Parse version strings (simple semantic versioning)
        let current_version = &self.version;
        let min_version = &self.rollback_protection.min_version;

        // For now, use simple string comparison
        // In a real implementation, proper semantic versioning would be used
        target_version >= min_version && target_version <= current_version
    }

    /// Mark slot as successful
    pub fn mark_slot_successful(&mut self, slot_name: &str) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.slot == slot_name) {
            slot.successful = true;
            slot.boot_attempts = 0;
            slot.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            true
        } else {
            false
        }
    }

    /// Mark slot as failed
    pub fn mark_slot_failed(&mut self, slot_name: &str) -> bool {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.slot == slot_name) {
            slot.boot_attempts += 1;
            if slot.boot_attempts >= slot.max_boot_attempts {
                slot.successful = false;
            }
            slot.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            true
        } else {
            false
        }
    }

    /// Get next boot slot
    pub fn get_next_boot_slot(&self) -> Option<&SlotInfo> {
        // Sort slots by priority (highest first) and success status
        let mut sorted_slots: Vec<&SlotInfo> = self.slots.iter().collect();
        sorted_slots.sort_by(|a, b| {
            // Successful slots first
            b.successful.cmp(&a.successful)
                .then_with(|| b.priority.cmp(&a.priority))
        });

        sorted_slots.first().copied()
    }
}

impl Default for PolyImageManifest {
    fn default() -> Self {
        Self::new(
            "default-image".to_string(),
            "Default PolyImage".to_string(),
            ImageType::System,
            "x86_64".to_string(),
            "generic".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = PolyImageManifest::new(
            "test-image".to_string(),
            "Test Image".to_string(),
            ImageType::System,
            "x86_64".to_string(),
            "test-platform".to_string(),
        );

        assert_eq!(manifest.image_id, "test-image");
        assert_eq!(manifest.description, "Test Image");
        assert_eq!(manifest.image_type, ImageType::System);
        assert_eq!(manifest.architecture, "x86_64");
        assert_eq!(manifest.platform, "test-platform");
    }

    #[test]
    fn test_manifest_validation() {
        let mut manifest = PolyImageManifest::default();
        
        // Add a slot
        manifest.add_slot(SlotInfo {
            slot: "A".to_string(),
            priority: 1,
            successful: true,
            active: true,
            last_updated: 0,
            boot_attempts: 0,
            max_boot_attempts: 3,
        });

        // Add a partition
        manifest.add_partition(PartitionImage {
            name: "system".to_string(),
            image_type: ImageType::System,
            path: "/system.img".to_string(),
            size: 1024 * 1024 * 1024, // 1GB
            hash_algorithm: HashAlgorithm::Sha256,
            hash: "abc123".to_string(),
            read_only: true,
            mount_point: Some("/system".to_string()),
            flags: vec!["verified".to_string()],
        });

        let validation = manifest.validate();
        assert!(validation.valid);
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_slot_management() {
        let mut manifest = PolyImageManifest::default();
        
        manifest.add_slot(SlotInfo {
            slot: "A".to_string(),
            priority: 1,
            successful: true,
            active: true,
            last_updated: 0,
            boot_attempts: 0,
            max_boot_attempts: 3,
        });

        manifest.add_slot(SlotInfo {
            slot: "B".to_string(),
            priority: 2,
            successful: false,
            active: false,
            last_updated: 0,
            boot_attempts: 0,
            max_boot_attempts: 3,
        });

        // Test slot marking
        assert!(manifest.mark_slot_successful("A"));
        assert!(manifest.mark_slot_failed("B"));

        // Test next boot slot selection
        let next_slot = manifest.get_next_boot_slot();
        assert!(next_slot.is_some());
        assert_eq!(next_slot.unwrap().slot, "A");
    }

    #[test]
    fn test_rollback_protection() {
        let mut manifest = PolyImageManifest::default();
        
        manifest.rollback_protection.min_version = "1.0.0".to_string();
        manifest.version = "2.0.0".to_string();

        // Test rollback allowed
        assert!(manifest.is_rollback_allowed("1.5.0"));
        assert!(manifest.is_rollback_allowed("2.0.0"));
        
        // Test rollback not allowed
        assert!(!manifest.is_rollback_allowed("0.9.0"));
    }
}
