//! NGFS v1 - Content-Addressed Filesystem with DID-Bound Encryption
//! 
//! This crate provides the core implementation of NGFS v1.

pub mod schema;
pub mod cid;
pub mod enc;

// Re-export main types for convenience
pub use schema::{
    EncryptionAlg, MountOptionsV1, SnapshotV1, ngfs_schema_hash,
    DirEntryV1, DirManifestV1, FileManifestV1, ChunkRefV1,
    ExportOpts, IpfsMapV1, IpfsMapEntryV1,
};
pub use cid::{Cid, cid_from_bytes, verify_chunk, CidError};
pub use enc::{NgfsEncryption, EncEnvelopeV1, AssociatedData, Dek, Kek, Nonce, EncError, VirtualClock, KeyVaultClient, DefaultVirtualClock};

/// NGFS service configuration
#[derive(Debug, Clone)]
pub struct NgfsConfig {
    /// Base path for storage
    pub storage_path: String,
    /// Maximum chunk size
    pub max_chunk_size: usize,
    /// Segment size
    pub segment_size: u64,
    /// Enable IPFS compatibility
    pub ipfs_compat: bool,
    /// Enable DID encryption
    pub did_encryption: bool,
    /// Mount salt for nonce generation
    pub mount_salt: [u8; 16],
}

impl Default for NgfsConfig {
    fn default() -> Self {
        Self {
            storage_path: "/var/lib/ngfs".to_string(),
            max_chunk_size: 256 * 1024,
            segment_size: 8 * 1024 * 1024,
            ipfs_compat: true,
            did_encryption: true,
            mount_salt: [0u8; 16],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockKeyVault;
    impl KeyVaultClient for MockKeyVault {
        fn derive_kek_for_kid(&self, _kid: &str) -> Result<Kek, EncError> {
            Ok(Kek::new([42u8; 32]))
        }
    }

    #[test]
    fn test_ngfs_config_default() {
        let config = NgfsConfig::default();
        assert_eq!(config.max_chunk_size, 256 * 1024);
        assert!(config.ipfs_compat);
    }

    #[test]
    fn test_cid_creation() {
        let data = b"test data";
        let cid = cid_from_bytes(data).unwrap();
        assert!(verify_chunk(data, &cid));
    }

    #[test]
    fn test_encryption_types() {
        let dek = Dek::random();
        assert_eq!(dek.as_bytes().len(), 32);
        
        let kek = Kek::new([1u8; 32]);
        assert_eq!(kek.as_bytes().len(), 32);
    }
}
