use blake3::Hasher;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// Content Identifier (CID) for NGFS CAS storage
/// 
/// This module provides functions for computing and verifying content identifiers
/// using Blake3 hashing with optional IPFS multihash compatibility.

/// Default hash algorithm for NGFS (Blake3)
pub const NGFS_HASH_ALGORITHM: u8 = 0x1e; // Blake3 multihash code

/// Blake3 hash length (32 bytes)
pub const BLAKE3_HASH_LENGTH: u8 = 0x20;

/// Maximum chunk size for CAS storage (256 KiB)
pub const NGFS_MAX_CHUNK_SIZE: usize = 256 * 1024;

/// Content Identifier with Blake3 hash
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cid {
    /// Blake3 hash (32 bytes)
    pub hash: [u8; 32],
    /// IPFS multihash format (optional)
    pub ipfs_multihash: Option<Vec<u8>>,
}

impl Cid {
    /// Create a new CID from a Blake3 hash
    pub fn new(hash: [u8; 32]) -> Self {
        Self {
            hash,
            ipfs_multihash: None,
        }
    }

    /// Create a CID with IPFS multihash compatibility
    pub fn with_ipfs(mut self, ipfs_multihash: Vec<u8>) -> Self {
        self.ipfs_multihash = Some(ipfs_multihash);
        self
    }

    /// Get the Blake3 hash as bytes
    pub fn hash_bytes(&self) -> &[u8; 32] {
        &self.hash
    }

    /// Get the IPFS multihash if available
    pub fn ipfs_multihash(&self) -> Option<&[u8]> {
        self.ipfs_multihash.as_deref()
    }

    /// Check if this CID has IPFS compatibility
    pub fn is_ipfs_compatible(&self) -> bool {
        self.ipfs_multihash.is_some()
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        if let Some(ipfs) = &self.ipfs_multihash {
            // Return IPFS multihash as base58 if available
            bs58::encode(ipfs).into_string()
        } else {
            // Return Blake3 hash as hex
            hex::encode(self.hash)
        }
    }

    /// Convert from string representation
    pub fn from_string(s: &str) -> Result<Self, CidError> {
        // Try to parse as IPFS multihash first
        if let Ok(decoded) = bs58::decode(s).into_vec() {
            if decoded.len() >= 2 {
                let code = decoded[0];
                let length = decoded[1];
                if code == NGFS_HASH_ALGORITHM && length == BLAKE3_HASH_LENGTH {
                    if decoded.len() == 34 { // 2 + 32
                        let mut hash = [0u8; 32];
                        hash.copy_from_slice(&decoded[2..]);
                        return Ok(Cid::new(hash).with_ipfs(decoded));
                    }
                }
            }
        }

        // Try to parse as hex Blake3 hash
        if s.len() == 64 {
            if let Ok(bytes) = hex::decode(s) {
                if bytes.len() == 32 {
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&bytes);
                    return Ok(Cid::new(hash));
                }
            }
        }

        Err(CidError::InvalidFormat)
    }
}

impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Compute CID from raw bytes using Blake3
pub fn cid_from_bytes(bytes: &[u8]) -> Result<Cid, CidError> {
    if bytes.len() > NGFS_MAX_CHUNK_SIZE {
        return Err(CidError::ChunkTooLarge);
    }

    let mut hasher = Hasher::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(hash.as_bytes());
    
    Ok(Cid::new(hash_bytes))
}

/// Compute CID with IPFS multihash compatibility
pub fn cid_from_bytes_with_ipfs(bytes: &[u8]) -> Result<Cid, CidError> {
    let mut cid = cid_from_bytes(bytes)?;
    
    // Create IPFS multihash: [code, length, hash]
    let mut ipfs_multihash = Vec::with_capacity(34);
    ipfs_multihash.push(NGFS_HASH_ALGORITHM);
    ipfs_multihash.push(BLAKE3_HASH_LENGTH);
    ipfs_multihash.extend_from_slice(cid.hash_bytes());
    
    Ok(cid.with_ipfs(ipfs_multihash))
}

/// Verify that bytes match the given CID
pub fn verify_chunk(bytes: &[u8], cid: &Cid) -> bool {
    match cid_from_bytes(bytes) {
        Ok(computed_cid) => computed_cid.hash == cid.hash,
        Err(_) => false,
    }
}

/// Verify chunk with detailed error information
pub fn verify_chunk_detailed(bytes: &[u8], cid: &Cid) -> Result<bool, CidError> {
    let computed_cid = cid_from_bytes(bytes)?;
    Ok(computed_cid.hash == cid.hash)
}

/// Compute Blake3 hash of bytes
pub fn blake3_hash(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    let hash = hasher.finalize();
    
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(hash.as_bytes());
    hash_bytes
}

/// Compute checksum for chunk verification
pub fn compute_checksum(bytes: &[u8]) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish() as u32
}

/// Verify checksum
pub fn verify_checksum(bytes: &[u8], expected: u32) -> bool {
    compute_checksum(bytes) == expected
}

/// CID-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CidError {
    /// Chunk size exceeds maximum allowed
    ChunkTooLarge,
    /// Invalid CID format
    InvalidFormat,
    /// Hash computation failed
    HashComputationFailed,
    /// Checksum verification failed
    ChecksumMismatch,
}

impl fmt::Display for CidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CidError::ChunkTooLarge => write!(f, "Chunk size exceeds maximum allowed"),
            CidError::InvalidFormat => write!(f, "Invalid CID format"),
            CidError::HashComputationFailed => write!(f, "Hash computation failed"),
            CidError::ChecksumMismatch => write!(f, "Checksum verification failed"),
        }
    }
}

impl std::error::Error for CidError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cid_creation() {
        let data = b"Hello, NGFS!";
        let cid = cid_from_bytes(data).unwrap();
        
        assert_eq!(cid.hash.len(), 32);
        assert!(!cid.is_ipfs_compatible());
    }

    #[test]
    fn test_cid_with_ipfs() {
        let data = b"Hello, NGFS with IPFS!";
        let cid = cid_from_bytes_with_ipfs(data).unwrap();
        
        assert!(cid.is_ipfs_compatible());
        assert_eq!(cid.ipfs_multihash().unwrap().len(), 34);
        assert_eq!(cid.ipfs_multihash().unwrap()[0], NGFS_HASH_ALGORITHM);
        assert_eq!(cid.ipfs_multihash().unwrap()[1], BLAKE3_HASH_LENGTH);
    }

    #[test]
    fn test_cid_verification() {
        let data = b"Test data for verification";
        let cid = cid_from_bytes(data).unwrap();
        
        assert!(verify_chunk(data, &cid));
        assert!(!verify_chunk(b"Different data", &cid));
    }

    #[test]
    fn test_cid_string_conversion() {
        let data = b"String conversion test";
        let cid = cid_from_bytes(data).unwrap();
        
        let cid_string = cid.to_string();
        let parsed_cid = Cid::from_string(&cid_string).unwrap();
        
        assert_eq!(cid.hash, parsed_cid.hash);
    }

    #[test]
    fn test_cid_ipfs_string_conversion() {
        let data = b"IPFS string conversion test";
        let cid = cid_from_bytes_with_ipfs(data).unwrap();
        
        let cid_string = cid.to_string();
        let parsed_cid = Cid::from_string(&cid_string).unwrap();
        
        assert_eq!(cid.hash, parsed_cid.hash);
        assert!(parsed_cid.is_ipfs_compatible());
    }

    #[test]
    fn test_chunk_size_limit() {
        let large_data = vec![0u8; NGFS_MAX_CHUNK_SIZE + 1];
        let result = cid_from_bytes(&large_data);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CidError::ChunkTooLarge);
    }

    #[test]
    fn test_checksum_computation() {
        let data = b"Checksum test data";
        let checksum = compute_checksum(data);
        
        assert!(verify_checksum(data, checksum));
        assert!(!verify_checksum(b"Different data", checksum));
    }

    #[test]
    fn test_blake3_hash() {
        let data = b"Blake3 hash test";
        let hash1 = blake3_hash(data);
        let hash2 = blake3_hash(data);
        
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_cid_equality() {
        let data = b"Equality test";
        let cid1 = cid_from_bytes(data).unwrap();
        let cid2 = cid_from_bytes(data).unwrap();
        
        assert_eq!(cid1, cid2);
        assert_eq!(cid1.hash, cid2.hash);
    }

    #[test]
    fn test_cid_hash_consistency() {
        let data = b"Hash consistency test";
        let cid1 = cid_from_bytes(data).unwrap();
        let cid2 = cid_from_bytes(data).unwrap();
        
        assert_eq!(cid1.hash, cid2.hash);
        
        // Verify the hash is deterministic
        let hash = blake3_hash(data);
        assert_eq!(cid1.hash, hash);
    }
}
