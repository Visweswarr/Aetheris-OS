use ngfs::{
    schema::{ExportOpts, IpfsMapV1, IpfsMapEntryV1},
    cid::Cid,
    ipfs::IpfsExporter,
    cas::MockCasIndex,
    enc::MockKeyVault,
};
use tempfile::tempdir;
use std::io::Write;

#[test]
fn test_car_file_generation() {
    let temp_dir = tempdir().unwrap();
    let config = ngfs::NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        mount_salt: [1u8; 16],
        ..Default::default()
    };
    
    let keyvault = Box::new(MockKeyVault);
    let service = ngfs::NgfsService::new(config, keyvault).unwrap();
    
    let root_cid = Cid::from_bytes(&[1, 2, 3, 4]).unwrap();
    
    // Create a mock IPFS map
    let map = IpfsMapV1 {
        version: 1,
        exported_vclock: 1000,
        root_ngfs_cid: root_cid.to_bytes(),
        entries: vec![
            IpfsMapEntryV1 {
                ngfs_cid: vec![1, 2, 3, 4],
                ipfs_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                kind: 0,
                size: 1024,
            },
            IpfsMapEntryV1 {
                ngfs_cid: vec![5, 6, 7, 8],
                ipfs_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                kind: 1,
                size: 2048,
            },
        ],
    };
    
    // Test CAR file writing
    let mut car_output = Vec::new();
    let result = service.write_car(&root_cid, &map, &mut car_output);
    assert!(result.is_ok(), "CAR writing should succeed");
    
    let car_content = String::from_utf8_lossy(&car_output);
    
    // Verify CAR v1 header
    assert!(car_content.starts_with("CAR v1"), "CAR should start with 'CAR v1'");
    
    // Verify block entries
    let lines: Vec<&str> = car_content.lines().collect();
    assert!(lines.len() >= 2, "CAR should have at least header and one block");
    
    // Check that we have the expected number of blocks
    let block_lines = lines.iter().filter(|line| line.contains(' ')).count();
    assert_eq!(block_lines, map.entries.len(), "Should have one block per entry");
}

#[test]
fn test_car_file_with_empty_map() {
    let temp_dir = tempdir().unwrap();
    let config = ngfs::NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        mount_salt: [1u8; 16],
        ..Default::default()
    };
    
    let keyvault = Box::new(MockKeyVault);
    let service = ngfs::NgfsService::new(config, keyvault).unwrap();
    
    let root_cid = Cid::from_bytes(&[1, 2, 3, 4]).unwrap();
    
    // Create an empty IPFS map
    let map = IpfsMapV1 {
        version: 1,
        exported_vclock: 1000,
        root_ngfs_cid: root_cid.to_bytes(),
        entries: vec![],
    };
    
    // Test CAR file writing with empty map
    let mut car_output = Vec::new();
    let result = service.write_car(&root_cid, &map, &mut car_output);
    assert!(result.is_ok(), "CAR writing should succeed even with empty map");
    
    let car_content = String::from_utf8_lossy(&car_output);
    
    // Should only have header
    assert!(car_content.starts_with("CAR v1"), "CAR should start with 'CAR v1'");
    assert_eq!(car_content.lines().count(), 1, "Empty CAR should only have header");
}

#[test]
fn test_car_file_format_consistency() {
    let temp_dir = tempdir().unwrap();
    let config = ngfs::NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        mount_salt: [1u8; 16],
        ..Default::default()
    };
    
    let keyvault = Box::new(MockKeyVault);
    let service = ngfs::NgfsService::new(config, keyvault).unwrap();
    
    let root_cid = Cid::from_bytes(&[1, 2, 3, 4]).unwrap();
    
    // Create a map with multiple entries
    let map = IpfsMapV1 {
        version: 1,
        exported_vclock: 1000,
        root_ngfs_cid: root_cid.to_bytes(),
        entries: vec![
            IpfsMapEntryV1 {
                ngfs_cid: vec![1, 2, 3, 4],
                ipfs_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                kind: 0,
                size: 1024,
            },
            IpfsMapEntryV1 {
                ngfs_cid: vec![5, 6, 7, 8],
                ipfs_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                kind: 1,
                size: 2048,
            },
            IpfsMapEntryV1 {
                ngfs_cid: vec![9, 10, 11, 12],
                ipfs_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                kind: 2,
                size: 512,
            },
        ],
    };
    
    // Generate CAR file
    let mut car_output = Vec::new();
    let result = service.write_car(&root_cid, &map, &mut car_output);
    assert!(result.is_ok(), "CAR writing should succeed");
    
    let car_content = String::from_utf8_lossy(&car_output);
    let lines: Vec<&str> = car_content.lines().collect();
    
    // Verify structure
    assert_eq!(lines[0], "CAR v1", "First line should be 'CAR v1'");
    
    // Verify block format: "size cid"
    for i in 1..lines.len() {
        let line = lines[i];
        if line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split(' ').collect();
        assert_eq!(parts.len(), 2, "Block line should have format 'size cid'");
        
        // Verify size is a number
        let size: u64 = parts[0].parse().expect("Size should be a number");
        assert!(size > 0, "Size should be positive");
        
        // Verify CID format
        let cid = parts[1];
        assert!(cid.starts_with('b'), "CID should start with 'b'");
    }
}

#[test]
fn test_car_file_size_limits() {
    let temp_dir = tempdir().unwrap();
    let config = ngfs::NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        mount_salt: [1u8; 16],
        ..Default::default()
    };
    
    let keyvault = Box::new(MockKeyVault);
    let service = ngfs::NgfsService::new(config, keyvault).unwrap();
    
    let root_cid = Cid::from_bytes(&[1, 2, 3, 4]).unwrap();
    
    // Create a map with large entries
    let map = IpfsMapV1 {
        version: 1,
        exported_vclock: 1000,
        root_ngfs_cid: root_cid.to_bytes(),
        entries: vec![
            IpfsMapEntryV1 {
                ngfs_cid: vec![1, 2, 3, 4],
                ipfs_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                kind: 0,
                size: 1024 * 1024, // 1MB
            },
        ],
    };
    
    // Test CAR file writing with large entry
    let mut car_output = Vec::new();
    let result = service.write_car(&root_cid, &map, &mut car_output);
    assert!(result.is_ok(), "CAR writing should succeed with large entries");
    
    let car_content = String::from_utf8_lossy(&car_output);
    
    // Verify the large entry was handled
    assert!(car_content.contains("1048576"), "Should contain size for 1MB entry");
}
