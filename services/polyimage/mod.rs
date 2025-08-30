//! PolyImage Service
//! 
//! This service provides system image management with A/B slots, atomic rollback,
//! and signed manifest validation for Polymera OS.

pub mod manifest;
pub mod mkimage;

pub use manifest::{
    HashAlgorithm, ImageType, PartitionImage, PolyImageManifest, RollbackProtection, Signature,
    SignatureAlgorithm, SlotInfo, ManifestValidation, ImageVerification,
    MANIFEST_SCHEMA_VERSION,
};

pub use mkimage::{ImageBuilder, ImageBuilderConfig};

/// PolyImage service version
pub const POLYIMAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// PolyImage service description
pub const POLYIMAGE_DESCRIPTION: &str = "PolyImage system image management service";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_version() {
        assert!(!POLYIMAGE_VERSION.is_empty());
        assert!(!POLYIMAGE_DESCRIPTION.is_empty());
    }

    #[test]
    fn test_manifest_import() {
        let manifest = PolyImageManifest::default();
        assert_eq!(manifest.version, "1.0.0");
    }
}
