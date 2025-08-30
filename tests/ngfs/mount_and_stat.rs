use ngfs::{NgfsService, NgfsConfig, MountOptionsV1, MountError, ResolveError};
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
fn test_mount_valid_paths() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Test valid mount points
    let valid_paths = [
        "/ro/test",
        "/ro/demo",
        "/ro/nested/path",
        "/ro/with_underscores",
        "/ro/with-dashes",
        "/ro/with123numbers",
    ];
    
    for path in &valid_paths {
        let result = service.mount_ro(path, mount_options.clone());
        assert!(result.is_ok(), "Failed to mount {}", path);
    }
    
    let mounts = service.list_mounts();
    assert_eq!(mounts.len(), valid_paths.len());
}

#[test]
fn test_mount_invalid_paths() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Test invalid mount points
    let invalid_paths = [
        ("", MountError::InvalidMountPoint("empty mount point".to_string())),
        ("test", MountError::InvalidMountPoint("mount point must be absolute".to_string())),
        ("/tmp/test", MountError::InvalidMountPoint("mount point must be under /ro/".to_string())),
        ("/ro", MountError::InvalidMountPoint("mount point must be under /ro/".to_string())),
        ("/ro/", MountError::InvalidMountPoint("mount point must be under /ro/".to_string())),
    ];
    
    for (path, expected_error) in &invalid_paths {
        let result = service.mount_ro(path, mount_options.clone());
        assert!(matches!(result, Err(ref e) if std::mem::discriminant(e) == std::mem::discriminant(expected_error)), 
                "Expected error for path {}: {:?}, got: {:?}", path, expected_error, result);
    }
}

#[test]
fn test_mount_overlap_detection() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Mount first path
    service.mount_ro("/ro/test", mount_options.clone()).unwrap();
    
    // Try to mount overlapping paths
    let overlapping_paths = [
        "/ro/test",
        "/ro/test/subdir",
        "/ro/test/subdir/file",
    ];
    
    for path in &overlapping_paths {
        let result = service.mount_ro(path, mount_options.clone());
        assert!(result.is_err(), "Should not allow overlapping mount at {}", path);
    }
    
    // Verify only one mount exists
    let mounts = service.list_mounts();
    assert_eq!(mounts.len(), 1);
}

#[test]
fn test_mount_already_exists() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Mount first time
    let result1 = service.mount_ro("/ro/test", mount_options.clone());
    assert!(result1.is_ok());
    
    // Try to mount again
    let result2 = service.mount_ro("/ro/test", mount_options.clone());
    assert!(matches!(result2, Err(MountError::AlreadyMounted(_))));
}

#[test]
fn test_unmount() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Mount
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    // Verify mount exists
    let mounts = service.list_mounts();
    assert_eq!(mounts.len(), 1);
    
    // Unmount
    let unmounted = service.unmount("/ro/test");
    assert!(unmounted.is_some());
    
    // Verify mount is gone
    let mounts = service.list_mounts();
    assert_eq!(mounts.len(), 0);
}

#[test]
fn test_unmount_nonexistent() {
    let mut service = create_test_service();
    
    let unmounted = service.unmount("/ro/nonexistent");
    assert!(unmounted.is_none());
}

#[test]
fn test_resolve_path_mount_not_found() {
    let service = create_test_service();
    
    let result = service.resolve_path("/ro/test", "/file.txt");
    assert!(matches!(result, Err(ResolveError::NotFound(_))));
}

#[test]
fn test_mount_salt_and_vclock() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [42u8; 16],
        vclock_base: 9999,
    };
    
    service.mount_ro("/ro/test", mount_options).unwrap();
    
    let mounts = service.list_mounts();
    assert_eq!(mounts.len(), 1);
    
    let (_, mount) = &mounts[0];
    assert_eq!(mount.salt, [42u8; 16]);
    assert_eq!(mount.vclock, 9999);
}

#[test]
fn test_multiple_mounts() {
    let mut service = create_test_service();
    
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
    
    let mounts = service.list_mounts();
    assert_eq!(mounts.len(), 2);
    
    // Verify both mounts exist with correct properties
    let mut found_test1 = false;
    let mut found_test2 = false;
    
    for (path, mount) in &mounts {
        match path.as_str() {
            "/ro/test1" => {
                assert_eq!(mount.salt, [1u8; 16]);
                assert_eq!(mount.vclock, 1000);
                found_test1 = true;
            }
            "/ro/test2" => {
                assert_eq!(mount.salt, [2u8; 16]);
                assert_eq!(mount.vclock, 2000);
                found_test2 = true;
            }
            _ => panic!("Unexpected mount point: {}", path),
        }
    }
    
    assert!(found_test1 && found_test2, "Both mounts should be found");
}
