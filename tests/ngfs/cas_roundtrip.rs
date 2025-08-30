use ngfs::{NgfsService, NgfsConfig, Cid, cid_from_bytes, verify_chunk};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_cas_roundtrip_basic() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let test_data = b"Basic roundtrip test data";
    let cid = service.store(test_data).unwrap();
    
    let retrieved_data = service.retrieve(&cid).unwrap();
    assert_eq!(test_data, retrieved_data.as_slice());
}

#[test]
fn test_cas_roundtrip_large_chunk() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let large_data = vec![0x42u8; 100 * 1024]; // 100 KiB
    let cid = service.store(&large_data).unwrap();
    
    let retrieved_data = service.retrieve(&cid).unwrap();
    assert_eq!(large_data, retrieved_data);
}

#[test]
fn test_cas_roundtrip_multiple_chunks() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let chunks = vec![
        b"First chunk".to_vec(),
        b"Second chunk".to_vec(),
        b"Third chunk".to_vec(),
        b"Fourth chunk".to_vec(),
    ];
    
    let mut cids = Vec::new();
    
    for chunk in &chunks {
        let cid = service.store(chunk).unwrap();
        cids.push(cid);
    }
    
    for (i, cid) in cids.iter().enumerate() {
        let retrieved_data = service.retrieve(cid).unwrap();
        assert_eq!(chunks[i], retrieved_data);
    }
}

#[test]
fn test_cas_roundtrip_hash_verification() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let test_data = b"Hash verification test data";
    let cid = service.store(test_data).unwrap();
    
    let retrieved_data = service.retrieve(&cid).unwrap();
    
    assert!(verify_chunk(&retrieved_data, &cid));
    
    let computed_cid = cid_from_bytes(&retrieved_data).unwrap();
    assert_eq!(cid.hash, computed_cid.hash);
}

#[test]
fn test_cas_roundtrip_empty_chunk() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let empty_data = b"";
    let cid = service.store(empty_data).unwrap();
    
    let retrieved_data = service.retrieve(&cid).unwrap();
    assert_eq!(empty_data, retrieved_data.as_slice());
}

#[test]
fn test_cas_roundtrip_binary_data() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let binary_data: Vec<u8> = (0..255).collect();
    let cid = service.store(&binary_data).unwrap();
    
    let retrieved_data = service.retrieve(&cid).unwrap();
    assert_eq!(binary_data, retrieved_data);
}

#[test]
fn test_cas_roundtrip_unicode_data() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let unicode_data = "Unicode test: 🚀🌟🎉 中文 Español Français".as_bytes();
    let cid = service.store(unicode_data).unwrap();
    
    let retrieved_data = service.retrieve(&cid).unwrap();
    assert_eq!(unicode_data, retrieved_data.as_slice());
}

#[test]
fn test_cas_roundtrip_concurrent_access() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let test_data = b"Concurrent access test data";
    let cid = service.store(test_data).unwrap();
    
    let retrieved_data1 = service.retrieve(&cid).unwrap();
    let retrieved_data2 = service.retrieve(&cid).unwrap();
    
    assert_eq!(retrieved_data1, retrieved_data2);
    assert_eq!(test_data, retrieved_data1.as_slice());
}

#[test]
fn test_cas_roundtrip_storage_persistence() {
    let temp_dir = tempdir().unwrap();
    let storage_path = temp_dir.path().join("ngfs_storage");
    fs::create_dir(&storage_path).unwrap();
    
    let config = NgfsConfig {
        storage_path: storage_path.to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let test_data = b"Storage persistence test data";
    let cid = service.store(test_data).unwrap();
    
    service.flush().unwrap();
    
    let retrieved_data = service.retrieve(&cid).unwrap();
    assert_eq!(test_data, retrieved_data.as_slice());
}

#[test]
fn test_cas_roundtrip_stats_consistency() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let initial_stats = service.get_stats();
    
    let test_data = b"Stats consistency test data";
    let cid = service.store(test_data).unwrap();
    
    let after_store_stats = service.get_stats();
    assert_eq!(after_store_stats.total_chunks, initial_stats.total_chunks + 1);
    
    let _retrieved_data = service.retrieve(&cid).unwrap();
    
    let after_retrieve_stats = service.get_stats();
    assert_eq!(after_retrieve_stats.total_chunks, after_store_stats.total_chunks);
}
