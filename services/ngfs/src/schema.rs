use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec::Vec;
use std::collections::BTreeMap;

/// NGFS v1 Schema - Content-Addressed Filesystem with DID-Bound Encryption

/// Schema version for NGFS v1
pub const NGFS_SCHEMA_VERSION: u16 = 1;

/// Maximum size for NGFS manifests (64 KiB)
pub const NGFS_MAX_MANIFEST_SIZE: usize = 64 * 1024;

/// Maximum size for CAS chunk metadata
pub const NGFS_MAX_CHUNK_META_SIZE: usize = 4 * 1024;

/// Maximum entries per directory in v1
pub const NGFS_MAX_DIR_ENTRIES: u32 = 65_535;

/// Maximum chunks per file in v1
pub const NGFS_MAX_FILE_CHUNKS: u32 = 4_096;

/// Maximum file size in v1 (1 TiB)
pub const NGFS_MAX_FILE_SIZE: u64 = 1 << 40;

/// Maximum name length for entries
pub const NGFS_MAX_NAME_LENGTH: usize = 255;

/// Mount options for NGFS v1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MountOptionsV1 {
    pub root_cid: String,
    pub salt: [u8; 16],
    pub vclock_base: u64,
}

/// Entry kind enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum EntryKindV1 {
    Directory = 0,
    File = 1,
    Symlink = 2,
}

/// Directory entry V1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirEntryV1 {
    pub name: String,
    pub kind: EntryKindV1,
    pub cid: String,
    pub size: Option<u64>,
}

/// Directory manifest V1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirManifestV1 {
    pub version: u16,
    pub entries: Vec<DirEntryV1>,
}

/// Chunk reference V1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkRefV1 {
    pub cid: String,
    pub size: u32,
}

/// File manifest V1
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileManifestV1 {
    pub version: u16,
    pub chunks: Vec<ChunkRefV1>,
    pub total_size: u64,
}

/// Encryption header for envelope encryption
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncHeaderV1 {
    pub key_id: String,
    pub algorithm: EncryptionAlg,
    pub nonce: [u8; 24],
    pub tag: [u8; 16],
    pub aad: Option<Vec<u8>>,
}

/// Encryption algorithm enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EncryptionAlg {
    XChaCha20Poly1305 = 0,
    ChaCha20Poly1305 = 1,
    Aes256Gcm = 2,
}

/// Snapshot object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotV1 {
    pub snap_id: String,
    pub root_cid: String,
    pub created_vclock: u64,
    pub signer_did: String,
    pub signature: Vec<u8>,
    pub metadata: BTreeMap<String, String>,
}

/// IPFS export map entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpfsMapEntryV1 {
    pub ngfs_cid: Vec<u8>,
    pub ipfs_cid: String,
    pub kind: u8,
    pub size: u64,
}

/// IPFS export map
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpfsMapV1 {
    pub version: u16,
    pub exported_vclock: u64,
    pub root_ngfs_cid: Vec<u8>,
    pub entries: Vec<IpfsMapEntryV1>,
}

/// Export options for IPFS mapping
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportOpts {
    pub include_chunks: bool,
    pub generate_car: bool,
    pub max_entries: Option<u32>,
    pub max_car_size: Option<u64>,
}

/// Compute the deterministic schema hash for NGFS v1
pub fn ngfs_schema_hash() -> [u8; 32] {
    use blake3::Hasher;
    
    let mut hasher = Hasher::new();
    hasher.update(&NGFS_SCHEMA_VERSION.to_le_bytes());
    hasher.update(&NGFS_MAX_MANIFEST_SIZE.to_le_bytes());
    hasher.update(&NGFS_MAX_CHUNK_META_SIZE.to_le_bytes());
    hasher.update(&NGFS_MAX_DIR_ENTRIES.to_le_bytes());
    hasher.update(&NGFS_MAX_FILE_CHUNKS.to_le_bytes());
    hasher.update(&NGFS_MAX_FILE_SIZE.to_le_bytes());
    hasher.update(&NGFS_MAX_NAME_LENGTH.to_le_bytes());
    
    let result = hasher.finalize();
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_hash_stability() {
        let hash1 = ngfs_schema_hash();
        let hash2 = ngfs_schema_hash();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_entry_kind_ordering() {
        assert!(EntryKindV1::Directory < EntryKindV1::File);
        assert!(EntryKindV1::File < EntryKindV1::Symlink);
    }
}
