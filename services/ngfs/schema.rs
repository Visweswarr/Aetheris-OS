use serde::{Deserialize, Serialize};
use std::string::String;
use std::vec::Vec;
use std::collections::BTreeMap;

/// NGFS v1 Schema - Content-Addressed Filesystem with DID-Bound Encryption
/// 
/// This module defines the deterministic schemas for NGFS v1, including:
/// - CAS chunk format with Blake3/IPFS multihash support
/// - Envelope encryption with XChaCha20-Poly1305
/// - Directory and file manifests with canonical ordering
/// - Snapshot objects with DID signatures
/// - Merkle membership proofs
/// 
/// All schemas use CBOR encoding with stable field ordering and schema versioning.

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

/// Content identifier for CAS chunks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId {
    /// Blake3 hash (32 bytes)
    pub blake3_hash: [u8; 32],
    /// Optional IPFS multihash for compatibility
    pub ipfs_multihash: Option<Vec<u8>>,
    /// Content type identifier
    pub content_type: ContentType,
}

/// Content type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ContentType {
    /// Raw data chunk
    Raw = 0,
    /// Directory manifest
    Directory = 1,
    /// File manifest
    FileManifest = 2,
    /// Snapshot object
    Snapshot = 3,
    /// Symlink
    Symlink = 4,
    /// Special file (device, socket, etc.)
    Special = 5,
}

/// Entry kind enumeration (canonical ordering: Dir < File < Symlink)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(u8)]
pub enum EntryKindV1 {
    /// Directory
    Directory = 0,
    /// Regular file
    File = 1,
    /// Symbolic link
    Symlink = 2,
}

/// File mode and permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMode {
    /// File type (regular, directory, symlink, special)
    pub file_type: FileType,
    /// Owner permissions (rwx)
    pub owner_perms: u8,
    /// Group permissions (rwx)
    pub group_perms: u8,
    /// Other permissions (rwx)
    pub other_perms: u8,
    /// Special bits (setuid, setgid, sticky)
    pub special_bits: u8,
}

/// File type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FileType {
    /// Regular file
    Regular = 0,
    /// Directory
    Directory = 1,
    /// Symbolic link
    Symlink = 2,
    /// Character device
    CharDevice = 3,
    /// Block device
    BlockDevice = 4,
    /// Named pipe (FIFO)
    NamedPipe = 5,
    /// Unix domain socket
    Socket = 6,
}

/// Directory entry with canonical ordering
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryV1 {
    /// Entry name (UTF-8 NFC normalized, 1-255 bytes)
    pub name: String,
    /// Entry kind (determines canonical ordering)
    pub kind: EntryKindV1,
    /// Content identifier
    pub cid: ContentId,
    /// File size in bytes (None for directories)
    pub size: Option<u64>,
    /// File mode and permissions (None for directories)
    pub mode: Option<u16>,
    /// Extended attributes (sorted keys for determinism)
    pub xattrs: Option<BTreeMap<String, Vec<u8>>>,
}

/// File manifest with chunk information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileManifestV1 {
    /// Schema version
    pub version: u16,
    /// Chunk information (sorted by CID for determinism)
    pub chunks: Vec<ChunkInfo>,
    /// Total file size in bytes
    pub total_size: u64,
    /// Hash algorithm used
    pub algorithm: String,
}

/// Chunk information for file manifests
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Content identifier for the chunk
    pub cid: ContentId,
    /// Chunk length in bytes
    pub length: u32,
}

/// Directory manifest with canonical entry ordering
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirManifestV1 {
    /// Schema version
    pub version: u16,
    /// Directory entries (sorted canonically)
    pub entries: Vec<EntryV1>,
}

/// Merkle proof frame for membership verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameV1 {
    /// Sibling CIDs at this level
    pub sibling_cids: Vec<ContentId>,
    /// Position within parent entries
    pub position: u32,
}

/// Merkle membership proof
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofV1 {
    /// Kind of node being proven
    pub node_kind: EntryKindV1,
    /// Content identifier of the node
    pub node_cid: ContentId,
    /// Proof path from leaf to root
    pub path: Vec<FrameV1>,
}

/// Encryption header for envelope encryption
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncHeaderV1 {
    /// Key ID from KeyVault
    pub key_id: String,
    /// Encryption algorithm
    pub algorithm: EncryptionAlg,
    /// Nonce for XChaCha20-Poly1305
    pub nonce: [u8; 24],
    /// Authentication tag
    pub tag: [u8; 16],
    /// Additional authenticated data
    pub aad: Option<Vec<u8>>,
}

/// Encryption algorithm enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum EncryptionAlg {
    /// XChaCha20-Poly1305 (recommended)
    XChaCha20Poly1305 = 0,
    /// ChaCha20-Poly1305 (fallback)
    ChaCha20Poly1305 = 1,
    /// AES-256-GCM (legacy)
    Aes256Gcm = 2,
}

/// CAS chunk with content addressing and encryption
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CASChunkV1 {
    /// Content identifier
    pub cid: ContentId,
    /// Encryption header
    pub enc_header: EncHeaderV1,
    /// Encrypted data
    pub enc_data: Vec<u8>,
    /// Chunk size in bytes
    pub size: u64,
    /// Creation timestamp (virtual clock)
    pub created_vclock: u64,
}

/// Snapshot object with DID signatures
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotV1 {
    /// Snapshot identifier
    pub snap_id: u64,
    /// Root directory CID
    pub root_cid: ContentId,
    /// Creation timestamp (virtual clock)
    pub created_vclock: u64,
    /// Signer DID
    pub signer_did: String,
    /// Signature algorithm
    pub sig_algorithm: SignatureAlg,
    /// DID signature
    pub signature: Vec<u8>,
    /// Snapshot metadata (sorted keys)
    pub metadata: BTreeMap<String, String>,
}

/// Signature algorithm enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SignatureAlg {
    /// Ed25519 (recommended)
    Ed25519 = 0,
    /// ECDSA P-256
    EcdsaP256 = 1,
    /// Dilithium2 (post-quantum)
    Dilithium2 = 2,
    /// SPHINCS+ (post-quantum)
    SphincsPlus = 3,
}

/// Syscall request for NGFS operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NGFSRequest {
    /// Request type
    pub request_type: NGFSRequestType,
    /// Request payload (CBOR encoded)
    pub payload: Vec<u8>,
}

/// NGFS request types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NGFSRequestType {
    /// Mount filesystem
    Mount = 0,
    /// Read chunk
    Read = 1,
    /// Get file status
    Stat = 2,
    /// Create snapshot
    Snapshot = 3,
}

/// NGFS response for operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NGFSResponse {
    /// Response status
    pub status: NGFSStatus,
    /// Response payload (CBOR encoded)
    pub payload: Option<Vec<u8>>,
    /// Error message if status is error
    pub error_message: Option<String>,
}

/// IPFS export map entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpfsMapEntryV1 {
    /// NGFS CID (Blake3)
    pub ngfs_cid: Vec<u8>,
    /// IPFS CIDv1 string
    pub ipfs_cid: String,
    /// Content kind (0=File, 1=Dir, 2=Chunk)
    pub kind: u8,
    /// Content size in bytes
    pub size: u64,
}

/// IPFS export map
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpfsMapV1 {
    /// Schema version
    pub version: u16,
    /// Export timestamp (virtual clock)
    pub exported_vclock: u64,
    /// Root NGFS CID
    pub root_ngfs_cid: Vec<u8>,
    /// Map entries sorted by ngfs_cid bytes
    pub entries: Vec<IpfsMapEntryV1>,
}

/// Export options for IPFS mapping
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportOpts {
    /// Include encrypted chunks in map
    pub include_chunks: bool,
    /// Generate CAR file
    pub generate_car: bool,
    /// Maximum entries in map
    pub max_entries: Option<u32>,
    /// Maximum CAR size in bytes
    pub max_car_size: Option<u64>,
}

/// NGFS response status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NGFSStatus {
    /// Operation successful
    Success = 0,
    /// Operation failed
    Error = 1,
    /// Operation not supported
    NotSupported = 2,
}

/// Compute the deterministic schema hash for NGFS v1
/// This ensures schema stability across builds
pub fn ngfs_schema_hash() -> [u8; 32] {
    use blake3::Hasher;
    
    let mut hasher = Hasher::new();
    
    // Hash the schema version
    hasher.update(&NGFS_SCHEMA_VERSION.to_le_bytes());
    
    // Hash the maximum values
    hasher.update(&NGFS_MAX_MANIFEST_SIZE.to_le_bytes());
    hasher.update(&NGFS_MAX_CHUNK_META_SIZE.to_le_bytes());
    hasher.update(&NGFS_MAX_DIR_ENTRIES.to_le_bytes());
    hasher.update(&NGFS_MAX_FILE_CHUNKS.to_le_bytes());
    hasher.update(&NGFS_MAX_FILE_SIZE.to_le_bytes());
    hasher.update(&NGFS_MAX_NAME_LENGTH.to_le_bytes());
    
    // Hash the schema structures (field names and types)
    let schema_info = b"EntryV1{name:String,kind:EntryKindV1,cid:ContentId,size:Option<u64>,mode:Option<u16>,xattrs:Option<BTreeMap<String,Vec<u8>>>}";
    hasher.update(schema_info);
    
    let schema_info = b"FileManifestV1{version:u16,chunks:Vec<ChunkInfo>,total_size:u64,algorithm:String}";
    hasher.update(schema_info);
    
    let schema_info = b"DirManifestV1{version:u16,entries:Vec<EntryV1>}";
    hasher.update(schema_info);
    
    let schema_info = b"ProofV1{node_kind:EntryKindV1,node_cid:ContentId,path:Vec<FrameV1>}";
    hasher.update(schema_info);
    
    let schema_info = b"EntryKindV1{Directory:0,File:1,Symlink:2}";
    hasher.update(schema_info);
    
    // Finalize and return the hash
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
        assert_eq!(hash1, hash2, "Schema hash should be stable across calls");
    }
    
    #[test]
    fn test_entry_kind_ordering() {
        // Verify canonical ordering: Dir < File < Symlink
        assert!(EntryKindV1::Directory < EntryKindV1::File);
        assert!(EntryKindV1::File < EntryKindV1::Symlink);
        assert!(EntryKindV1::Directory < EntryKindV1::Symlink);
    }
    
    #[test]
    fn test_constants() {
        assert_eq!(NGFS_SCHEMA_VERSION, 1);
        assert_eq!(NGFS_MAX_MANIFEST_SIZE, 64 * 1024);
        assert_eq!(NGFS_MAX_DIR_ENTRIES, 65_535);
        assert_eq!(NGFS_MAX_FILE_CHUNKS, 4_096);
        assert_eq!(NGFS_MAX_FILE_SIZE, 1 << 40);
        assert_eq!(NGFS_MAX_NAME_LENGTH, 255);
    }
    
    #[test]
    fn test_content_type_values() {
        assert_eq!(ContentType::Raw as u8, 0);
        assert_eq!(ContentType::Directory as u8, 1);
        assert_eq!(ContentType::FileManifest as u8, 2);
        assert_eq!(ContentType::Snapshot as u8, 3);
        assert_eq!(ContentType::Symlink as u8, 4);
        assert_eq!(ContentType::Special as u8, 5);
    }
    
    #[test]
    fn test_entry_kind_values() {
        assert_eq!(EntryKindV1::Directory as u8, 0);
        assert_eq!(EntryKindV1::File as u8, 1);
        assert_eq!(EntryKindV1::Symlink as u8, 2);
    }
}
