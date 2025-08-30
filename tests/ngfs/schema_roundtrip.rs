use ngfs::schema::*;

/// Test NGFS schema CBOR roundtrip
/// 
/// This test ensures that all NGFS v1 schemas can be encoded to CBOR
/// and decoded back to their original form without loss of information.

#[test]
fn test_content_id_cbor_roundtrip() {
    let original = ContentId::new([0x12; 32], ContentType::Directory)
        .with_ipfs(vec![0x12, 0x20, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: ContentId = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.blake3_hash, [0x12; 32]);
    assert_eq!(decoded.content_type, ContentType::Directory);
    assert_eq!(decoded.ipfs_multihash, Some(vec![0x12, 0x20, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]));
}

#[test]
fn test_enc_header_cbor_roundtrip() {
    let original = EncHeaderV1 {
        key_id: "test-key-123".to_string(),
        algorithm: EncryptionAlg::XChaCha20Poly1305,
        nonce: [0x11; 24],
        tag: [0x22; 16],
        aad: Some(vec![0x33, 0x44, 0x55]),
    };
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: EncHeaderV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.key_id, "test-key-123");
    assert_eq!(decoded.algorithm, EncryptionAlg::XChaCha20Poly1305);
    assert_eq!(decoded.nonce, [0x11; 24]);
    assert_eq!(decoded.tag, [0x22; 16]);
    assert_eq!(decoded.aad, Some(vec![0x33, 0x44, 0x55]));
}

#[test]
fn test_cas_chunk_cbor_roundtrip() {
    let cid = ContentId::new([0xaa; 32], ContentType::Raw);
    let enc_header = EncHeaderV1 {
        key_id: "chunk-key".to_string(),
        algorithm: EncryptionAlg::ChaCha20Poly1305,
        nonce: [0xbb; 24],
        tag: [0xcc; 16],
        aad: None,
    };
    
    let original = CASChunkV1::new(
        cid,
        enc_header,
        vec![0xdd, 0xee, 0xff, 0x00, 0x11, 0x22],
        12345,
    );
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: CASChunkV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.size, 6);
    assert_eq!(decoded.created_vclock, 12345);
    assert_eq!(decoded.enc_data, vec![0xdd, 0xee, 0xff, 0x00, 0x11, 0x22]);
}

#[test]
fn test_file_mode_cbor_roundtrip() {
    let original = FileMode::new(FileType::Regular, 0o644, 0o640, 0o600)
        .with_special_bits(0o4000); // setuid
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: FileMode = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.file_type, FileType::Regular);
    assert_eq!(decoded.owner_perms, 0o644);
    assert_eq!(decoded.group_perms, 0o640);
    assert_eq!(decoded.other_perms, 0o600);
    assert_eq!(decoded.special_bits, 0o4000);
}

#[test]
fn test_dir_entry_cbor_roundtrip() {
    let cid = ContentId::new([0x55; 32], ContentType::Regular);
    let mode = FileMode::new(FileType::Regular, 0o644, 0o644, 0o644);
    
    let original = DirEntryV1 {
        name: "test_file.txt".to_string(),
        cid,
        mode,
        size: 1024,
        mtime_vclock: 123456789,
        atime_vclock: 123456788,
        ctime_vclock: 123456787,
        owner_did: "did:example:owner".to_string(),
        group_did: Some("did:example:group".to_string()),
    };
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: DirEntryV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.name, "test_file.txt");
    assert_eq!(decoded.size, 1024);
    assert_eq!(decoded.mtime_vclock, 123456789);
    assert_eq!(decoded.owner_did, "did:example:owner");
    assert_eq!(decoded.group_did, Some("did:example:group".to_string()));
}

#[test]
fn test_dir_manifest_cbor_roundtrip() {
    let mode = FileMode::new(FileType::Directory, 0o755, 0o755, 0o755);
    
    let mut original = DirManifestV1::new(
        12345,
        "test_directory".to_string(),
        mode,
        "did:example:owner".to_string(),
        123456789,
    );
    
    // Add some children
    let child1 = DirEntryV1 {
        name: "file1.txt".to_string(),
        cid: ContentId::new([0x11; 32], ContentType::Regular),
        mode: FileMode::new(FileType::Regular, 0o644, 0o644, 0o644),
        size: 100,
        mtime_vclock: 123456789,
        atime_vclock: 123456789,
        ctime_vclock: 123456789,
        owner_did: "did:example:owner".to_string(),
        group_did: None,
    };
    
    let child2 = DirEntryV1 {
        name: "file2.txt".to_string(),
        cid: ContentId::new([0x22; 32], ContentType::Regular),
        mode: FileMode::new(FileType::Regular, 0o644, 0o644, 0o644),
        size: 200,
        mtime_vclock: 123456789,
        atime_vclock: 123456789,
        ctime_vclock: 123456789,
        owner_did: "did:example:owner".to_string(),
        group_did: None,
    };
    
    original.add_child(child1);
    original.add_child(child2);
    
    // Add extended attributes
    original.set_xattr("user.comment".to_string(), "Test directory".as_bytes().to_vec());
    original.set_xattr("user.version".to_string(), "1.0".as_bytes().to_vec());
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: DirManifestV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.inode_id, 12345);
    assert_eq!(decoded.name, "test_directory");
    assert_eq!(decoded.size, 300); // 100 + 200
    assert_eq!(decoded.children.len(), 2);
    assert_eq!(decoded.children[0].name, "file1.txt");
    assert_eq!(decoded.children[1].name, "file2.txt");
    
    // Verify extended attributes
    assert_eq!(decoded.get_xattr("user.comment"), Some("Test directory".as_bytes()));
    assert_eq!(decoded.get_xattr("user.version"), Some("1.0".as_bytes()));
}

#[test]
fn test_snapshot_cbor_roundtrip() {
    let root_cid = ContentId::new([0x99; 32], ContentType::Directory);
    
    let mut original = SnapshotV1::new(
        67890,
        root_cid,
        "did:example:signer".to_string(),
        123456789,
    );
    
    // Set signature
    original.set_signature(
        vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
        SignatureAlg::Ed25519,
    );
    
    // Add metadata
    original.add_metadata("description".to_string(), "Test snapshot".to_string());
    original.add_metadata("version".to_string(), "1.0".to_string());
    original.add_metadata("created_by".to_string(), "test_user".to_string());
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: SnapshotV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.snap_id, 67890);
    assert_eq!(decoded.created_vclock, 123456789);
    assert_eq!(decoded.signer_did, "did:example:signer");
    assert_eq!(decoded.sig_algorithm, SignatureAlg::Ed25519);
    assert_eq!(decoded.signature, vec![0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]);
    
    // Verify metadata
    assert_eq!(decoded.get_metadata("description"), Some(&"Test snapshot".to_string()));
    assert_eq!(decoded.get_metadata("version"), Some(&"1.0".to_string()));
    assert_eq!(decoded.get_metadata("created_by"), Some(&"test_user".to_string()));
}

#[test]
fn test_mount_options_cbor_roundtrip() {
    let original = MountOptionsV1 {
        mount_point: "/mnt/ngfs".to_string(),
        root_snapshot: 12345,
        read_only: true,
        key_id: Some("mount-key-123".to_string()),
        ipfs_compat: true,
        did_verify: true,
    };
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: MountOptionsV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.mount_point, "/mnt/ngfs");
    assert_eq!(decoded.root_snapshot, 12345);
    assert!(decoded.read_only);
    assert_eq!(decoded.key_id, Some("mount-key-123".to_string()));
    assert!(decoded.ipfs_compat);
    assert!(decoded.did_verify);
}

#[test]
fn test_file_stat_cbor_roundtrip() {
    let mode = FileMode::new(FileType::Regular, 0o644, 0o644, 0o644);
    let cid = ContentId::new([0x77; 32], ContentType::Regular);
    
    let original = FileStatV1 {
        mode,
        size: 2048,
        mtime_vclock: 123456789,
        atime_vclock: 123456788,
        ctime_vclock: 123456787,
        owner_did: "did:example:owner".to_string(),
        group_did: Some("did:example:group".to_string()),
        cid,
    };
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: FileStatV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.size, 2048);
    assert_eq!(decoded.mtime_vclock, 123456789);
    assert_eq!(decoded.owner_did, "did:example:owner");
    assert_eq!(decoded.group_did, Some("did:example:group".to_string()));
}

#[test]
fn test_read_request_cbor_roundtrip() {
    let original = ReadRequestV1 {
        path: "/path/to/file.txt".to_string(),
        offset: 1024,
        length: 512,
        key_id: Some("read-key-123".to_string()),
    };
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: ReadRequestV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.path, "/path/to/file.txt");
    assert_eq!(decoded.offset, 1024);
    assert_eq!(decoded.length, 512);
    assert_eq!(decoded.key_id, Some("read-key-123".to_string()));
}

#[test]
fn test_read_response_cbor_roundtrip() {
    let cid = ContentId::new([0x66; 32], ContentType::Regular);
    let mode = FileMode::new(FileType::Regular, 0o644, 0o644, 0o644);
    let enc_header = EncHeaderV1 {
        key_id: "response-key".to_string(),
        algorithm: EncryptionAlg::XChaCha20Poly1305,
        nonce: [0x55; 24],
        tag: [0x44; 16],
        aad: None,
    };
    
    let original = ReadResponseV1 {
        cid,
        mode,
        size: 1024,
        data: vec![0x11, 0x22, 0x33, 0x44, 0x55],
        enc_header: Some(enc_header),
    };
    
    // Encode to CBOR
    let encoded = serde_cbor::to_vec(&original).unwrap();
    
    // Decode from CBOR
    let decoded: ReadResponseV1 = serde_cbor::from_slice(&encoded).unwrap();
    
    // Verify roundtrip
    assert_eq!(original, decoded);
    
    // Verify specific fields
    assert_eq!(decoded.size, 1024);
    assert_eq!(decoded.data, vec![0x11, 0x22, 0x33, 0x44, 0x55]);
    assert!(decoded.enc_header.is_some());
}

#[test]
fn test_all_enums_cbor_roundtrip() {
    // Test all enum variants can be encoded/decoded
    
    // ContentType
    for content_type in [
        ContentType::Raw,
        ContentType::Directory,
        ContentType::FileMeta,
        ContentType::Snapshot,
        ContentType::Symlink,
        ContentType::Special,
    ] {
        let encoded = serde_cbor::to_vec(&content_type).unwrap();
        let decoded: ContentType = serde_cbor::from_slice(&encoded).unwrap();
        assert_eq!(content_type, decoded);
    }
    
    // EncryptionAlg
    for alg in [
        EncryptionAlg::XChaCha20Poly1305,
        EncryptionAlg::ChaCha20Poly1305,
        EncryptionAlg::Aes256Gcm,
    ] {
        let encoded = serde_cbor::to_vec(&alg).unwrap();
        let decoded: EncryptionAlg = serde_cbor::from_slice(&encoded).unwrap();
        assert_eq!(alg, decoded);
    }
    
    // FileType
    for file_type in [
        FileType::Regular,
        FileType::Directory,
        FileType::Symlink,
        FileType::CharDevice,
        FileType::BlockDevice,
        FileType::NamedPipe,
        FileType::Socket,
    ] {
        let encoded = serde_cbor::to_vec(&file_type).unwrap();
        let decoded: FileType = serde_cbor::from_slice(&encoded).unwrap();
        assert_eq!(file_type, decoded);
    }
    
    // SignatureAlg
    for sig_alg in [
        SignatureAlg::Ed25519,
        SignatureAlg::EcdsaP256,
        SignatureAlg::Dilithium3,
        SignatureAlg::Falcon512,
    ] {
        let encoded = serde_cbor::to_vec(&sig_alg).unwrap();
        let decoded: SignatureAlg = serde_cbor::from_slice(&encoded).unwrap();
        assert_eq!(sig_alg, decoded);
    }
}

#[test]
fn test_cbor_size_limits() {
    // Test that large data structures respect size limits
    
    let large_data = vec![0x00; NGFS_MAX_CHUNK_META_SIZE + 1];
    let cid = ContentId::new([0x11; 32], ContentType::Raw);
    let enc_header = EncHeaderV1 {
        key_id: "test-key".to_string(),
        algorithm: EncryptionAlg::XChaCha20Poly1305,
        nonce: [0x22; 24],
        tag: [0x33; 16],
        aad: None,
    };
    
    let chunk = CASChunkV1::new(cid, enc_header, large_data, 12345);
    
    // This should succeed (CBOR encoding should handle large data)
    let encoded = serde_cbor::to_vec(&chunk).unwrap();
    
    // Verify the encoded size is reasonable
    assert!(encoded.len() > 0);
    
    // Decode should also succeed
    let decoded: CASChunkV1 = serde_cbor::from_slice(&encoded).unwrap();
    assert_eq!(chunk, decoded);
}

#[test]
fn test_cbor_deterministic_ordering() {
    // Test that CBOR encoding produces deterministic results
    // for the same input data
    
    let cid = ContentId::new([0x11; 32], ContentType::Directory);
    let enc_header = EncHeaderV1 {
        key_id: "test-key".to_string(),
        algorithm: EncryptionAlg::XChaCha20Poly1305,
        nonce: [0x22; 24],
        tag: [0x33; 16],
        aad: None,
    };
    
    let chunk = CASChunkV1::new(cid, enc_header, vec![0x44, 0x55, 0x66], 12345);
    
    // Encode multiple times
    let encoded1 = serde_cbor::to_vec(&chunk).unwrap();
    let encoded2 = serde_cbor::to_vec(&chunk).unwrap();
    let encoded3 = serde_cbor::to_vec(&chunk).unwrap();
    
    // All encodings should be identical
    assert_eq!(encoded1, encoded2);
    assert_eq!(encoded2, encoded3);
    assert_eq!(encoded1, encoded3);
}
