use std::collections::HashMap;
use std::fs;
use std::path::Path;
use blake3::Hasher;
use serde::{Deserialize, Serialize};

/// NGFS Schema Hash Generator
/// 
/// This tool computes deterministic schema hashes for NGFS v1 schemas,
/// ensuring schema stability across builds and deployments.

/// Schema hash information
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaHashInfo {
    /// Schema version
    pub version: u16,
    /// Schema hash (Blake3)
    pub hash: String,
    /// Schema hash in bytes
    pub hash_bytes: Vec<u8>,
    /// Generated timestamp
    pub generated_at: String,
    /// Schema files included
    pub schema_files: Vec<String>,
    /// Schema constants
    pub constants: HashMap<String, String>,
}

/// NGFS schema constants for hashing
pub const NGFS_SCHEMA_CONSTANTS: &[(&str, &str)] = &[
    ("SCHEMA_VERSION", "1"),
    ("MAX_MANIFEST_SIZE", "65536"),
    ("MAX_CHUNK_META_SIZE", "4096"),
    ("CONTENT_TYPE_RAW", "0"),
    ("CONTENT_TYPE_DIRECTORY", "1"),
    ("CONTENT_TYPE_FILE_META", "2"),
    ("CONTENT_TYPE_SNAPSHOT", "3"),
    ("CONTENT_TYPE_SYMLINK", "4"),
    ("CONTENT_TYPE_SPECIAL", "5"),
    ("ENCRYPTION_XCHACHA20_POLY1305", "0"),
    ("ENCRYPTION_CHACHA20_POLY1305", "1"),
    ("ENCRYPTION_AES256_GCM", "2"),
    ("FILE_TYPE_REGULAR", "0"),
    ("FILE_TYPE_DIRECTORY", "1"),
    ("FILE_TYPE_SYMLINK", "2"),
    ("FILE_TYPE_CHAR_DEVICE", "3"),
    ("FILE_TYPE_BLOCK_DEVICE", "4"),
    ("FILE_TYPE_NAMED_PIPE", "5"),
    ("FILE_TYPE_SOCKET", "6"),
    ("SIGNATURE_ED25519", "0"),
    ("SIGNATURE_ECDSA_P256", "1"),
    ("SIGNATURE_DILITHIUM3", "2"),
    ("SIGNATURE_FALCON512", "3"),
];

/// Compute the NGFS schema hash
pub fn compute_ngfs_schema_hash() -> [u8; 32] {
    let mut hasher = Hasher::new();
    
    // Hash the schema version
    hasher.update(&1u16.to_le_bytes());
    
    // Hash the content type enum values
    hasher.update(b"ContentType");
    for i in 0..6 {
        hasher.update(&[i]);
    }
    
    // Hash the encryption algorithm enum values
    hasher.update(b"EncryptionAlg");
    for i in 0..3 {
        hasher.update(&[i]);
    }
    
    // Hash the file type enum values
    hasher.update(b"FileType");
    for i in 0..7 {
        hasher.update(&[i]);
    }
    
    // Hash the signature algorithm enum values
    hasher.update(b"SignatureAlg");
    for i in 0..4 {
        hasher.update(&[i]);
    }
    
    // Hash the maximum sizes
    hasher.update(&65536usize.to_le_bytes()); // MAX_MANIFEST_SIZE
    hasher.update(&4096usize.to_le_bytes());  // MAX_CHUNK_META_SIZE
    
    // Hash the field names for deterministic ordering
    hasher.update(b"ContentId");
    hasher.update(b"blake3_hash");
    hasher.update(b"ipfs_multihash");
    hasher.update(b"content_type");
    
    hasher.update(b"EncHeaderV1");
    hasher.update(b"key_id");
    hasher.update(b"algorithm");
    hasher.update(b"nonce");
    hasher.update(b"tag");
    hasher.update(b"aad");
    
    hasher.update(b"CASChunkV1");
    hasher.update(b"cid");
    hasher.update(b"enc_header");
    hasher.update(b"enc_data");
    hasher.update(b"size");
    hasher.update(b"created_vclock");
    
    hasher.update(b"FileMode");
    hasher.update(b"file_type");
    hasher.update(b"owner_perms");
    hasher.update(b"group_perms");
    hasher.update(b"other_perms");
    hasher.update(b"special_bits");
    
    hasher.update(b"DirEntryV1");
    hasher.update(b"name");
    hasher.update(b"mode");
    hasher.update(b"size");
    hasher.update(b"mtime_vclock");
    hasher.update(b"atime_vclock");
    hasher.update(b"ctime_vclock");
    hasher.update(b"owner_did");
    hasher.update(b"group_did");
    
    hasher.update(b"DirManifestV1");
    hasher.update(b"inode_id");
    hasher.update(b"children");
    hasher.update(b"created_vclock");
    hasher.update(b"mtime_vclock");
    hasher.update(b"xattrs");
    
    hasher.update(b"SnapshotV1");
    hasher.update(b"snap_id");
    hasher.update(b"root_cid");
    hasher.update(b"created_vclock");
    hasher.update(b"signer_did");
    hasher.update(b"sig_algorithm");
    hasher.update(b"signature");
    hasher.update(b"metadata");
    
    // Finalize and return the hash
    let hash = hasher.finalize();
    hash.into()
}

/// Generate schema hash information
pub fn generate_schema_hash_info() -> SchemaHashInfo {
    let hash_bytes = compute_ngfs_schema_hash();
    let hash = hex::encode(hash_bytes);
    
    let constants: HashMap<String, String> = NGFS_SCHEMA_CONSTANTS
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    
    SchemaHashInfo {
        version: 1,
        hash,
        hash_bytes: hash_bytes.to_vec(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        schema_files: vec![
            "services/ngfs/schema.rs".to_string(),
        ],
        constants,
    }
}

/// Write schema hash to file
pub fn write_schema_hash_file(output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let hash_info = generate_schema_hash_info();
    let json = serde_json::to_string_pretty(&hash_info)?;
    fs::write(output_path, json)?;
    Ok(())
}

/// Validate schema hash against stored hash
pub fn validate_schema_hash(stored_hash_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if !stored_hash_path.exists() {
        return Ok(false);
    }
    
    let stored_content = fs::read_to_string(stored_hash_path)?;
    let stored_info: SchemaHashInfo = serde_json::from_str(&stored_content)?;
    
    let current_hash = compute_ngfs_schema_hash();
    let current_hash_hex = hex::encode(current_hash);
    
    Ok(stored_info.hash == current_hash_hex)
}

/// Main function for CLI usage
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <output_path>", args[0]);
        eprintln!("Example: {} ngfs_schema_hash.json", args[0]);
        std::process::exit(1);
    }
    
    let output_path = Path::new(&args[1]);
    write_schema_hash_file(output_path)?;
    
    let hash_info = generate_schema_hash_info();
    println!("NGFS Schema Hash Generated:");
    println!("  Version: {}", hash_info.version);
    println!("  Hash: {}", hash_info.hash);
    println!("  Generated: {}", hash_info.generated_at);
    println!("  Output: {:?}", output_path);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_hash_stability() {
        let hash1 = compute_ngfs_schema_hash();
        let hash2 = compute_ngfs_schema_hash();
        
        assert_eq!(hash1, hash2);
        
        let hex1 = hex::encode(hash1);
        let hex2 = hex::encode(hash2);
        
        assert_eq!(hex1, hex2);
        assert_eq!(hex1.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_schema_hash_info_generation() {
        let hash_info = generate_schema_hash_info();
        
        assert_eq!(hash_info.version, 1);
        assert_eq!(hash_info.hash.len(), 64);
        assert_eq!(hash_info.hash_bytes.len(), 32);
        assert!(!hash_info.schema_files.is_empty());
        assert!(!hash_info.constants.is_empty());
    }

    #[test]
    fn test_constants_inclusion() {
        let hash_info = generate_schema_hash_info();
        
        // Check that key constants are included
        assert!(hash_info.constants.contains_key("SCHEMA_VERSION"));
        assert!(hash_info.constants.contains_key("MAX_MANIFEST_SIZE"));
        assert!(hash_info.constants.contains_key("CONTENT_TYPE_RAW"));
        assert!(hash_info.constants.contains_key("ENCRYPTION_XCHACHA20_POLY1305"));
        assert!(hash_info.constants.contains_key("FILE_TYPE_REGULAR"));
        assert!(hash_info.constants.contains_key("SIGNATURE_ED25519"));
    }
}
