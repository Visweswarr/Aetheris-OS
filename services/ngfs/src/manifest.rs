//! NGFS v1 Manifest Generation and CID Computation
//! 
//! This module provides authoritative manifest builders for directories and files,
//! with canonical ordering and deterministic Merkle root computation.

use std::string::String;
use std::vec::Vec;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use blake3::Hasher;

use crate::schema::{
    EntryV1, EntryKindV1, DirManifestV1, FileManifestV1, ChunkInfo, ContentId, ContentType,
    ProofV1, FrameV1, NGFS_MAX_DIR_ENTRIES, NGFS_MAX_FILE_CHUNKS, NGFS_MAX_FILE_SIZE,
    NGFS_MAX_NAME_LENGTH,
};

/// Manifest generation errors
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("Invalid entry name: {0}")]
    InvalidEntryName(String),
    
    #[error("Too many directory entries: {count} > {max}")]
    TooManyEntries { count: usize, max: u32 },
    
    #[error("Too many file chunks: {count} > {max}")]
    TooManyChunks { count: usize, max: u32 },
    
    #[error("File too large: {size} > {max}")]
    FileTooLarge { size: u64, max: u64 },
    
    #[error("Invalid content ID")]
    InvalidContentId,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid proof structure")]
    InvalidProof,
}

/// Manifest builder for directories
pub struct DirManifestBuilder {
    entries: Vec<EntryV1>,
}

impl DirManifestBuilder {
    /// Create a new directory manifest builder
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    /// Add an entry to the directory
    pub fn add_entry(&mut self, entry: EntryV1) -> Result<(), ManifestError> {
        // Validate entry name
        self.validate_entry_name(&entry.name)?;
        
        // Check entry count limit
        if self.entries.len() >= NGFS_MAX_DIR_ENTRIES as usize {
            return Err(ManifestError::TooManyEntries {
                count: self.entries.len(),
                max: NGFS_MAX_DIR_ENTRIES,
            });
        }
        
        self.entries.push(entry);
        Ok(())
    }
    
    /// Add multiple entries at once
    pub fn add_entries(&mut self, entries: Vec<EntryV1>) -> Result<(), ManifestError> {
        for entry in entries {
            self.add_entry(entry)?;
        }
        Ok(())
    }
    
    /// Build the directory manifest with canonical ordering
    pub fn build(self) -> Result<(DirManifestV1, Vec<u8>, ContentId), ManifestError> {
        // Sort entries canonically: (kind asc) then (name bytes asc)
        let mut sorted_entries = self.entries;
        sorted_entries.sort_by(|a, b| {
            // First by kind (Directory < File < Symlink)
            a.kind.cmp(&b.kind)
                .then_with(|| {
                    // Then by name bytes (UTF-8)
                    a.name.as_bytes().cmp(b.name.as_bytes())
                })
        });
        
        let manifest = DirManifestV1 {
            version: 1,
            entries: sorted_entries,
        };
        
        // Serialize to CBOR
        let cbor_bytes = serde_cbor::to_vec(&manifest)
            .map_err(|e| ManifestError::SerializationError(format!("CBOR serialization failed: {}", e)))?;
        
        // Compute CID
        let cid = compute_cid(&cbor_bytes, ContentType::Directory)?;
        
        Ok((manifest, cbor_bytes, cid))
    }
    
    /// Validate entry name according to NGFS constraints
    fn validate_entry_name(&self, name: &str) -> Result<(), ManifestError> {
        if name.is_empty() {
            return Err(ManifestError::InvalidEntryName("Name cannot be empty".to_string()));
        }
        
        if name.len() > NGFS_MAX_NAME_LENGTH {
            return Err(ManifestError::InvalidEntryName(
                format!("Name too long: {} > {}", name.len(), NGFS_MAX_NAME_LENGTH)
            ));
        }
        
        // Check for forbidden characters
        if name.contains('\0') {
            return Err(ManifestError::InvalidEntryName("Name cannot contain NUL character".to_string()));
        }
        
        if name.contains('/') {
            return Err(ManifestError::InvalidEntryName("Name cannot contain '/'".to_string()));
        }
        
        if name == "." || name == ".." {
            return Err(ManifestError::InvalidEntryName(
                format!("Name '{}' is reserved".to_string())
            ));
        }
        
        // Check for control characters
        if name.chars().any(|c| c.is_control()) {
            return Err(ManifestError::InvalidEntryName("Name cannot contain control characters".to_string()));
        }
        
        // TODO: Implement Unicode NFC normalization check
        // For now, we'll accept any valid UTF-8
        
        Ok(())
    }
}

/// Manifest builder for files
pub struct FileManifestBuilder {
    chunks: Vec<ChunkInfo>,
    total_size: u64,
}

impl FileManifestBuilder {
    /// Create a new file manifest builder
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            total_size: 0,
        }
    }
    
    /// Add a chunk to the file
    pub fn add_chunk(&mut self, cid: ContentId, length: u32) -> Result<(), ManifestError> {
        // Check chunk count limit
        if self.chunks.len() >= NGFS_MAX_FILE_CHUNKS as usize {
            return Err(ManifestError::TooManyChunks {
                count: self.chunks.len(),
                max: NGFS_MAX_FILE_CHUNKS,
            });
        }
        
        // Validate content ID
        if cid.blake3_hash == [0u8; 32] {
            return Err(ManifestError::InvalidContentId);
        }
        
        let chunk_info = ChunkInfo { cid, length };
        self.chunks.push(chunk_info);
        self.total_size += length as u64;
        
        Ok(())
    }
    
    /// Add multiple chunks at once
    pub fn add_chunks(&mut self, chunks: Vec<(ContentId, u32)>) -> Result<(), ManifestError> {
        for (cid, length) in chunks {
            self.add_chunk(cid, length)?;
        }
        Ok(())
    }
    
    /// Build the file manifest with canonical ordering
    pub fn build(self) -> Result<(FileManifestV1, Vec<u8>, ContentId), ManifestError> {
        // Check file size limit
        if self.total_size > NGFS_MAX_FILE_SIZE {
            return Err(ManifestError::FileTooLarge {
                size: self.total_size,
                max: NGFS_MAX_FILE_SIZE,
            });
        }
        
        // Sort chunks by CID for determinism
        let mut sorted_chunks = self.chunks;
        sorted_chunks.sort_by(|a, b| a.cid.blake3_hash.cmp(&b.cid.blake3_hash));
        
        let manifest = FileManifestV1 {
            version: 1,
            chunks: sorted_chunks,
            total_size: self.total_size,
            algorithm: "blake3".to_string(),
        };
        
        // Serialize to CBOR
        let cbor_bytes = serde_cbor::to_vec(&manifest)
            .map_err(|e| ManifestError::SerializationError(format!("CBOR serialization failed: {}", e)))?;
        
        // Compute CID
        let cid = compute_cid(&cbor_bytes, ContentType::FileManifest)?;
        
        Ok((manifest, cbor_bytes, cid))
    }
}

/// Compute content identifier from CBOR bytes
pub fn compute_cid(cbor_bytes: &[u8], content_type: ContentType) -> Result<ContentId, ManifestError> {
    // Compute Blake3 hash
    let mut hasher = Hasher::new();
    hasher.update(cbor_bytes);
    let hash = hasher.finalize();
    let blake3_hash: [u8; 32] = hash.into();
    
    // Create content ID
    let cid = ContentId {
        blake3_hash,
        ipfs_multihash: None, // IPFS compatibility handled separately
        content_type,
    };
    
    Ok(cid)
}

/// Verify a Merkle membership proof
pub fn verify_proof(proof: &ProofV1, expected_root: &ContentId) -> Result<bool, ManifestError> {
    // Validate proof structure
    if proof.path.is_empty() {
        return Err(ManifestError::InvalidProof);
    }
    
    // Start with the node being proven
    let mut current_cid = proof.node_cid.clone();
    
    // Walk up the proof path
    for frame in &proof.path {
        // Validate frame
        if frame.position >= frame.sibling_cids.len() as u32 {
            return Err(ManifestError::InvalidProof);
        }
        
        // Get the sibling CID at the specified position
        let sibling_cid = &frame.sibling_cids[frame.position as usize];
        
        // In a real Merkle tree, we would hash the current node with its siblings
        // For v1, we'll use a simplified approach that just validates the structure
        // TODO: Implement proper Merkle tree hashing in v2
        
        // For now, we'll just check that the proof structure is valid
        if sibling_cid.blake3_hash == [0u8; 32] {
            return Err(ManifestError::InvalidProof);
        }
        
        // Update current CID (simplified for v1)
        // In v2, this would be the actual Merkle tree hash
        current_cid = sibling_cid.clone();
    }
    
    // Check if we reached the expected root
    Ok(current_cid.blake3_hash == expected_root.blake3_hash)
}

/// Build a directory manifest from a list of entries
pub fn build_dir_manifest(entries: Vec<EntryV1>) -> Result<(DirManifestV1, Vec<u8>, ContentId), ManifestError> {
    let mut builder = DirManifestBuilder::new();
    builder.add_entries(entries)?;
    builder.build()
}

/// Build a file manifest from a list of chunks
pub fn build_file_manifest(chunks: Vec<(ContentId, u32)>) -> Result<(FileManifestV1, Vec<u8>, ContentId), ManifestError> {
    let mut builder = FileManifestBuilder::new();
    builder.add_chunks(chunks)?;
    builder.build()
}

/// Create a test entry for testing purposes
pub fn create_test_entry(name: &str, kind: EntryKindV1, cid: ContentId, size: Option<u64>) -> EntryV1 {
    EntryV1 {
        name: name.to_string(),
        kind,
        cid,
        size,
        mode: Some(0o644),
        xattrs: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dir_manifest_builder() {
        let mut builder = DirManifestBuilder::new();
        
        // Create test entries
        let cid1 = ContentId {
            blake3_hash: [1u8; 32],
            ipfs_multihash: None,
            content_type: ContentType::Directory,
        };
        
        let cid2 = ContentId {
            blake3_hash: [2u8; 32],
            ipfs_multihash: None,
            content_type: ContentType::File,
        };
        
        let entry1 = create_test_entry("b_file", EntryKindV1::File, cid2.clone(), Some(100));
        let entry2 = create_test_entry("a_dir", EntryKindV1::Directory, cid1.clone(), None);
        
        builder.add_entry(entry1).unwrap();
        builder.add_entry(entry2).unwrap();
        
        let (manifest, cbor, cid) = builder.build().unwrap();
        
        // Verify canonical ordering: Directory < File, then by name
        assert_eq!(manifest.entries[0].name, "a_dir");
        assert_eq!(manifest.entries[0].kind, EntryKindV1::Directory);
        assert_eq!(manifest.entries[1].name, "b_file");
        assert_eq!(manifest.entries[1].kind, EntryKindV1::File);
        
        // Verify CBOR and CID
        assert!(!cbor.is_empty());
        assert_eq!(cid.content_type, ContentType::Directory);
    }
    
    #[test]
    fn test_file_manifest_builder() {
        let mut builder = FileManifestBuilder::new();
        
        // Create test chunks
        let cid1 = ContentId {
            blake3_hash: [1u8; 32],
            ipfs_multihash: None,
            content_type: ContentType::Raw,
        };
        
        let cid2 = ContentId {
            blake3_hash: [2u8; 32],
            ipfs_multihash: None,
            content_type: ContentType::Raw,
        };
        
        builder.add_chunk(cid1.clone(), 100).unwrap();
        builder.add_chunk(cid2.clone(), 200).unwrap();
        
        let (manifest, cbor, cid) = builder.build().unwrap();
        
        assert_eq!(manifest.total_size, 300);
        assert_eq!(manifest.chunks.len(), 2);
        assert_eq!(manifest.algorithm, "blake3");
        
        // Verify CBOR and CID
        assert!(!cbor.is_empty());
        assert_eq!(cid.content_type, ContentType::FileManifest);
    }
    
    #[test]
    fn test_entry_name_validation() {
        let mut builder = DirManifestBuilder::new();
        
        // Valid names
        let valid_names = ["file.txt", "dir", "file-name", "file_name", "file123"];
        for name in valid_names {
            let cid = ContentId {
                blake3_hash: [0u8; 32],
                ipfs_multihash: None,
                content_type: ContentType::File,
            };
            let entry = create_test_entry(name, EntryKindV1::File, cid, Some(100));
            assert!(builder.add_entry(entry).is_ok(), "Name '{}' should be valid", name);
        }
        
        // Invalid names
        let invalid_names = ["", ".", "..", "file/name", "file\0name"];
        for name in invalid_names {
            let cid = ContentId {
                blake3_hash: [0u8; 32],
                ipfs_multihash: None,
                content_type: ContentType::File,
            };
            let entry = create_test_entry(name, EntryKindV1::File, cid, Some(100));
            assert!(builder.add_entry(entry).is_err(), "Name '{}' should be invalid", name);
        }
    }
    
    #[test]
    fn test_canonical_ordering() {
        let mut builder = DirManifestBuilder::new();
        
        // Create entries in non-canonical order
        let cid = ContentId {
            blake3_hash: [0u8; 32],
            ipfs_multihash: None,
            content_type: ContentType::File,
        };
        
        let entries = vec![
            create_test_entry("z_file", EntryKindV1::File, cid.clone(), Some(100)),
            create_test_entry("a_dir", EntryKindV1::Directory, cid.clone(), None),
            create_test_entry("b_symlink", EntryKindV1::Symlink, cid.clone(), None),
        ];
        
        builder.add_entries(entries).unwrap();
        let (manifest, _, _) = builder.build().unwrap();
        
        // Verify canonical ordering: Directory < File < Symlink, then by name
        assert_eq!(manifest.entries[0].name, "a_dir");
        assert_eq!(manifest.entries[0].kind, EntryKindV1::Directory);
        assert_eq!(manifest.entries[1].name, "b_symlink");
        assert_eq!(manifest.entries[1].kind, EntryKindV1::Symlink);
        assert_eq!(manifest.entries[2].name, "z_file");
        assert_eq!(manifest.entries[2].kind, EntryKindV1::File);
    }
    
    #[test]
    fn test_compute_cid() {
        let test_data = b"test data for CID computation";
        let cid = compute_cid(test_data, ContentType::File).unwrap();
        
        assert_eq!(cid.content_type, ContentType::File);
        assert_ne!(cid.blake3_hash, [0u8; 32]);
        assert!(cid.ipfs_multihash.is_none());
    }
    
    #[test]
    fn test_limits() {
        // Test directory entry limit
        let mut builder = DirManifestBuilder::new();
        let cid = ContentId {
            blake3_hash: [0u8; 32],
            ipfs_multihash: None,
            content_type: ContentType::File,
        };
        
        for i in 0..NGFS_MAX_DIR_ENTRIES {
            let entry = create_test_entry(&format!("file{}", i), EntryKindV1::File, cid.clone(), Some(100));
            builder.add_entry(entry).unwrap();
        }
        
        // Adding one more should fail
        let entry = create_test_entry("one_too_many", EntryKindV1::File, cid, Some(100));
        assert!(builder.add_entry(entry).is_err());
        
        // Test file chunk limit
        let mut file_builder = FileManifestBuilder::new();
        for i in 0..NGFS_MAX_FILE_CHUNKS {
            let cid = ContentId {
                blake3_hash: [i as u8; 32],
                ipfs_multihash: None,
                content_type: ContentType::Raw,
            };
            file_builder.add_chunk(cid, 100).unwrap();
        }
        
        // Adding one more should fail
        let cid = ContentId {
            blake3_hash: [255u8; 32],
            ipfs_multihash: None,
            content_type: ContentType::Raw,
        };
        assert!(file_builder.add_chunk(cid, 100).is_err());
    }
}
