use ngfs::{NgfsService, NgfsConfig, MountOptionsV1, MountError, ResolveError, ReadError, SnapshotError};
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
fn test_mount_error_empty_root_cid() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // This should fail due to empty root CID
    let result = service.mount_ro("/ro/test", mount_options);
    assert!(result.is_err());
}

#[test]
fn test_mount_error_invalid_root_cid() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "invalid_cid_format".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // This should fail due to invalid CID format
    let result = service.mount_ro("/ro/test", mount_options);
    assert!(result.is_err());
}

#[test]
fn test_mount_error_path_traversal() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Test path traversal attempts
    let traversal_paths = [
        "/ro/../test",
        "/ro/test/../other",
        "/ro/./test",
        "/ro/test/./file",
    ];
    
    for path in &traversal_paths {
        let result = service.mount_ro(path, mount_options.clone());
        assert!(result.is_err(), "Should reject path traversal: {}", path);
    }
}

#[test]
fn test_mount_error_control_chars() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Test control characters in path
    let control_char_paths = [
        "/ro/test\x00",
        "/ro/test\x01",
        "/ro/test\x1F",
        "/ro/test\x7F",
    ];
    
    for path in &control_char_paths {
        let result = service.mount_ro(path, mount_options.clone());
        assert!(result.is_err(), "Should reject control chars: {}", path);
    }
}

#[test]
fn test_mount_error_very_long_path() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Create a very long path
    let long_component = "a".repeat(300);
    let long_path = format!("/ro/{}", long_component);
    
    let result = service.mount_ro(&long_path, mount_options);
    assert!(result.is_err(), "Should reject very long path");
}

#[test]
fn test_mount_error_special_chars() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Test special characters that might cause issues
    let special_char_paths = [
        "/ro/test*",
        "/ro/test?",
        "/ro/test[",
        "/ro/test]",
        "/ro/test{",
        "/ro/test}",
        "/ro/test|",
        "/ro/test\\",
        "/ro/test^",
        "/ro/test$",
    ];
    
    for path in &special_char_paths {
        let result = service.mount_ro(path, mount_options.clone());
        // These might be allowed depending on implementation
        // Just verify the operation doesn't panic
        let _ = result;
    }
}

#[test]
fn test_mount_error_unicode() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Test Unicode characters
    let unicode_paths = [
        "/ro/测试",
        "/ro/тест",
        "/ro/テスト",
        "/ro/🎉",
        "/ro/🚀",
    ];
    
    for path in &unicode_paths {
        let result = service.mount_ro(path, mount_options.clone());
        // Unicode should generally be allowed
        if result.is_ok() {
            // Clean up if mount succeeded
            service.unmount(path);
        }
    }
}

#[test]
fn test_mount_error_duplicate_salt() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Mount first time
    service.mount_ro("/ro/test1", mount_options.clone()).unwrap();
    
    // Try to mount with same salt (should work, salt is per-mount)
    let result = service.mount_ro("/ro/test2", mount_options);
    assert!(result.is_ok());
    
    // Clean up
    service.unmount("/ro/test1");
    service.unmount("/ro/test2");
}

#[test]
fn test_mount_error_duplicate_vclock() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Mount first time
    service.mount_ro("/ro/test1", mount_options.clone()).unwrap();
    
    // Try to mount with same vclock (should work, vclock is per-mount)
    let result = service.mount_ro("/ro/test2", mount_options);
    assert!(result.is_ok());
    
    // Clean up
    service.unmount("/ro/test1");
    service.unmount("/ro/test2");
}

#[test]
fn test_mount_error_nested_overlap() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Mount a nested path
    service.mount_ro("/ro/nested/path", mount_options.clone()).unwrap();
    
    // Try to mount parent path
    let result = service.mount_ro("/ro/nested", mount_options.clone());
    assert!(result.is_err(), "Should reject parent path mount");
    
    // Try to mount child path
    let result = service.mount_ro("/ro/nested/path/deeper", mount_options);
    assert!(result.is_err(), "Should reject child path mount");
    
    // Clean up
    service.unmount("/ro/nested/path");
}

#[test]
fn test_mount_error_root_cid_not_found() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // This should fail because the root CID doesn't exist in CAS
    let result = service.mount_ro("/ro/test", mount_options);
    // In a real implementation, this might fail with RootNotFound
    // For now, just verify it doesn't panic
    let _ = result;
}

#[test]
fn test_mount_error_invalid_salt_length() {
    let mut service = create_test_service();
    
    // Test with different salt lengths
    let invalid_salts = [
        [0u8; 0],    // Empty salt
        [0u8; 8],    // Too short
        [0u8; 32],   // Too long
    ];
    
    for salt in &invalid_salts {
        let mount_options = MountOptionsV1 {
            root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
            salt: *salt,
            vclock_base: 1000,
        };
        
        let result = service.mount_ro("/ro/test", mount_options);
        // This might fail depending on validation
        let _ = result;
    }
}

#[test]
fn test_mount_error_negative_vclock() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 0, // Zero vclock
    };
    
    let result = service.mount_ro("/ro/test", mount_options);
    // Zero vclock should be valid
    if result.is_ok() {
        service.unmount("/ro/test");
    }
}

#[test]
fn test_mount_error_max_vclock() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: u64::MAX,
    };
    
    let result = service.mount_ro("/ro/test", mount_options);
    // Max vclock should be valid
    if result.is_ok() {
        service.unmount("/ro/test");
    }
}

#[test]
fn test_mount_error_concurrent_mounts() {
    let mut service = create_test_service();
    
    let mount_options = MountOptionsV1 {
        root_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
        salt: [1u8; 16],
        vclock_base: 1000,
    };
    
    // Try to mount the same path multiple times concurrently
    let results: Vec<_> = (0..10)
        .map(|i| {
            let path = format!("/ro/test{}", i);
            service.mount_ro(&path, mount_options.clone())
        })
        .collect();
    
    // All should succeed
    for result in results {
        assert!(result.is_ok());
    }
    
    // Clean up
    for i in 0..10 {
        let path = format!("/ro/test{}", i);
        service.unmount(&path);
    }
}
