use ngfs::{NgfsService, NgfsConfig, MountOptionsV1, SnapshotV1, SnapshotError};
use tempfile::tempdir;
use std::collections::BTreeMap;

struct MockKeyVault;
impl ngfs::KeyVaultClient for MockKeyVault {
    fn derive_kek_for_kid(&self, _kid: &str) -> Result<ngfs::Kek, ngfs::EncError> {
        Ok(ngfs::Kek([42u8; 32]))
    }
}

fn create_test_service() -> NgfsService {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        mount_salt: [1u8; 16],
        ..Default::default()
    };
    
    let keyvault = Box::new(MockKeyVault);
    NgfsService::new(config, keyvault).unwrap()
}

#[test]
fn test_create_snapshot_basic() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Mount first
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    // Create snapshot
    let snapshot = service.create_snapshot("/ro/test", "did:key:test").unwrap();
    
    assert_eq!(snapshot.root_cid, "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi");
    assert_eq!(snapshot.signer_did, "did:key:test");
    assert!(!snapshot.snap_id.is_empty());
    assert_eq!(snapshot.created_vclock, 0); // First snapshot
}

#[test]
fn test_create_snapshot_mount_not_found() {
    let service = create_test_service();
    
    let result = service.create_snapshot("/ro/nonexistent", "did:key:test");
    assert!(matches!(result, Err(SnapshotError::MountNotFound(_))));
}

#[test]
fn test_create_snapshot_invalid_mount_point() {
    let service = create_test_service();
    
    let result = service.create_snapshot("/tmp/test", "did:key:test");
    assert!(matches!(result, Err(SnapshotError::InvalidMountPoint(_))));
}

#[test]
fn test_create_snapshot_relative_path() {
    let service = create_test_service();
    
    let result = service.create_snapshot("test", "did:key:test");
    assert!(matches!(result, Err(SnapshotError::InvalidMountPoint(_))));
}

#[test]
fn test_create_snapshot_empty_path() {
    let service = create_test_service();
    
    let result = service.create_snapshot("", "did:key:test");
    assert!(matches!(result, Err(SnapshotError::InvalidMountPoint(_))));
}

#[test]
fn test_create_snapshot_multiple() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    // Create multiple snapshots
    let snapshot1 = service.create_snapshot("/ro/test", "did:key:test1").unwrap();
    let snapshot2 = service.create_snapshot("/ro/test", "did:key:test2").unwrap();
    let snapshot3 = service.create_snapshot("/ro/test", "did:key:test3").unwrap();
    
    // Verify vclock increments
    assert_eq!(snapshot1.created_vclock, 0);
    assert_eq!(snapshot2.created_vclock, 1);
    assert_eq!(snapshot3.created_vclock, 2);
    
    // Verify different signers
    assert_eq!(snapshot1.signer_did, "did:key:test1");
    assert_eq!(snapshot2.signer_did, "did:key:test2");
    assert_eq!(snapshot3.signer_did, "did:key:test3");
    
    // Verify different IDs
    assert_ne!(snapshot1.snap_id, snapshot2.snap_id);
    assert_ne!(snapshot2.snap_id, snapshot3.snap_id);
    assert_ne!(snapshot1.snap_id, snapshot3.snap_id);
}

#[test]
fn test_verify_snapshot() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    let snapshot = service.create_snapshot("/ro/test", "did:key:test").unwrap();
    
    // Verify the snapshot
    let is_valid = service.verify_snapshot(&snapshot).unwrap();
    assert!(is_valid);
}

#[test]
fn test_verify_snapshot_tampered() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    let mut snapshot = service.create_snapshot("/ro/test", "did:key:test").unwrap();
    
    // Tamper with the snapshot
    snapshot.root_cid = "tampered_cid".to_string();
    
    // Verify should fail
    let is_valid = service.verify_snapshot(&snapshot).unwrap();
    assert!(!is_valid);
}

#[test]
fn test_verify_snapshot_different_signer() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    let mut snapshot = service.create_snapshot("/ro/test", "did:key:test").unwrap();
    
    // Change the signer
    snapshot.signer_did = "did:key:different".to_string();
    
    // Verify should fail
    let is_valid = service.verify_snapshot(&snapshot).unwrap();
    assert!(!is_valid);
}

#[test]
fn test_snapshot_id_deterministic() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    // Set a fixed vclock
    service.snapshot_builder.set_vclock(42);
    
    // Create snapshots with same parameters
    let snapshot1 = service.create_snapshot("/ro/test", "did:key:test").unwrap();
    let snapshot2 = service.create_snapshot("/ro/test", "did:key:test").unwrap();
    
    // IDs should be identical
    assert_eq!(snapshot1.snap_id, snapshot2.snap_id);
}

#[test]
fn test_snapshot_id_format() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    let snapshot = service.create_snapshot("/ro/test", "did:key:test").unwrap();
    
    // Verify ID format (should start with bafy)
    assert!(snapshot.snap_id.starts_with("bafy"));
    assert!(snapshot.snap_id.len() > 10); // Reasonable length for CID
}

#[test]
fn test_snapshot_vclock_rollover() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    // Set vclock to near max
    service.snapshot_builder.set_vclock(u64::MAX - 5);
    
    // Create snapshots
    for i in 0..10 {
        let snapshot = service.create_snapshot("/ro/test", "did:key:test").unwrap();
        assert_eq!(snapshot.created_vclock, u64::MAX - 5 + i);
    }
}

#[test]
fn test_snapshot_different_mounts() {
    let service = create_test_service();
    
    let mount_options1 = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    let mount_options2 = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [2u8; 16],
        vclock_base: 2000,
    };
    
    service.mount_ro("/ro/test1", mount_options1).unwrap();
    service.mount_ro("/ro/test2", mount_options2).unwrap();
    
    // Create snapshots on different mounts
    let snapshot1 = service.create_snapshot("/ro/test1", "did:key:test").unwrap();
    let snapshot2 = service.create_snapshot("/ro/test2", "did:key:test").unwrap();
    
    // Should have different IDs due to different root CIDs
    assert_ne!(snapshot1.snap_id, snapshot2.snap_id);
    assert_ne!(snapshot1.root_cid, snapshot2.root_cid);
}

#[test]
fn test_snapshot_empty_signer() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    // Create snapshot with empty signer
    let snapshot = service.create_snapshot("/ro/test", "").unwrap();
    
    assert_eq!(snapshot.signer_did, "");
    assert!(!snapshot.snap_id.is_empty());
}

#[test]
fn test_snapshot_large_signer() {
    let service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    // Create snapshot with long signer DID
    let long_signer = "did:key:".to_string() + &"z".repeat(1000);
    let snapshot = service.create_snapshot("/ro/test", &long_signer).unwrap();
    
    assert_eq!(snapshot.signer_did, long_signer);
    assert!(!snapshot.snap_id.is_empty());
}
