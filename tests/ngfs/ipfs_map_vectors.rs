use ngfs::{
    schema::{ExportOpts, IpfsMapV1, IpfsMapEntryV1},
    cid::Cid,
    ipfs::IpfsExporter,
    cas::MockCasIndex,
    enc::MockKeyVault,
};
use tempfile::tempdir;

#[test]
fn test_ipfs_export_basic() {
    let temp_dir = tempdir().unwrap();
    let config = ngfs::NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        mount_salt: [1u8; 16],
        ..Default::default()
    };
    
    let keyvault = Box::new(MockKeyVault);
    let service = ngfs::NgfsService::new(config, keyvault).unwrap();
    
    let root_cid = Cid::from_bytes(&[1, 2, 3, 4]).unwrap();
    let opts = ExportOpts {
        include_chunks: true,
        generate_car: false,
        max_entries: Some(1000),
        max_car_size: Some(1024 * 1024),
    };
    
    let result = service.export_ipfs_map(&root_cid, &opts);
    assert!(result.is_ok(), "Export should succeed");
    
    let map = result.unwrap();
    assert_eq!(map.version, 1);
    assert_eq!(map.entries.len(), 0); // No actual data in mock
}

#[test]
fn test_ipfs_export_with_limits() {
    let temp_dir = tempdir().unwrap();
    let config = ngfs::NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        mount_salt: [1u8; 16],
        ..Default::default()
    };
    
    let keyvault = Box::new(MockKeyVault);
    let service = ngfs::NgfsService::new(config, keyvault).unwrap();
    
    let root_cid = Cid::from_bytes(&[1, 2, 3, 4]).unwrap();
    let opts = ExportOpts {
        include_chunks: false,
        generate_car: true,
        max_entries: Some(5),
        max_car_size: Some(1024),
    };
    
    let result = service.export_ipfs_map(&root_cid, &opts);
    assert!(result.is_ok(), "Export should succeed");
    
    let map = result.unwrap();
    assert_eq!(map.version, 1);
    assert!(map.entries.len() <= 5);
}

#[test]
fn test_ipfs_map_entry_ordering() {
    let mut entries = vec![
        IpfsMapEntryV1 {
            ngfs_cid: vec![5, 6, 7, 8],
            ipfs_cid: "bafy2".to_string(),
            kind: 1,
            size: 200,
        },
        IpfsMapEntryV1 {
            ngfs_cid: vec![1, 2, 3, 4],
            ipfs_cid: "bafy1".to_string(),
            kind: 0,
            size: 100,
        },
    ];
    
    // Sort by ngfs_cid bytes
    entries.sort_by(|a, b| a.ngfs_cid.cmp(&b.ngfs_cid));
    
    // Verify ordering
    assert_eq!(entries[0].ngfs_cid, vec![1, 2, 3, 4]);
    assert_eq!(entries[1].ngfs_cid, vec![5, 6, 7, 8]);
}

#[test]
fn test_ipfs_map_structure() {
    let map = IpfsMapV1 {
        version: 1,
        exported_vclock: 1000,
        root_ngfs_cid: vec![1, 2, 3, 4],
        entries: vec![
            IpfsMapEntryV1 {
                ngfs_cid: vec![1, 2, 3, 4],
                ipfs_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
                kind: 0,
                size: 1024,
            },
        ],
    };
    
    assert_eq!(map.version, 1);
    assert_eq!(map.exported_vclock, 1000);
    assert_eq!(map.root_ngfs_cid, vec![1, 2, 3, 4]);
    assert_eq!(map.entries.len(), 1);
    
    let entry = &map.entries[0];
    assert_eq!(entry.ngfs_cid, vec![1, 2, 3, 4]);
    assert!(entry.ipfs_cid.starts_with('b'));
    assert_eq!(entry.kind, 0);
    assert_eq!(entry.size, 1024);
}

#[test]
fn test_export_opts_validation() {
    let opts = ExportOpts {
        include_chunks: true,
        generate_car: false,
        max_entries: Some(1000),
        max_car_size: Some(1024 * 1024),
    };
    
    assert!(opts.include_chunks);
    assert!(!opts.generate_car);
    assert_eq!(opts.max_entries, Some(1000));
    assert_eq!(opts.max_car_size, Some(1024 * 1024));
}
