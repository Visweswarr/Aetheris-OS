use ngfs::{NgfsService, NgfsConfig, Cid};
use tempfile::tempdir;
use std::fs;
use std::path::Path;

#[test]
fn test_cas_recovery_basic() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store some data
    let test_data = b"Recovery test data";
    let cid = service.store(test_data).unwrap();
    
    // Simulate crash by dropping service
    drop(service);
    
    // Recreate service (should trigger recovery)
    let mut new_service = NgfsService::new(config).unwrap();
    
    // Verify data is still accessible
    let retrieved_data = new_service.retrieve(&cid).unwrap();
    assert_eq!(test_data, retrieved_data.as_slice());
}

#[test]
fn test_cas_recovery_multiple_chunks() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store multiple chunks
    let chunks = vec![
        b"Chunk 1 for recovery".to_vec(),
        b"Chunk 2 for recovery".to_vec(),
        b"Chunk 3 for recovery".to_vec(),
        b"Chunk 4 for recovery".to_vec(),
    ];
    
    let mut cids = Vec::new();
    for chunk in &chunks {
        let cid = service.store(chunk).unwrap();
        cids.push(cid);
    }
    
    // Simulate crash
    drop(service);
    
    // Recreate service
    let mut new_service = NgfsService::new(config).unwrap();
    
    // Verify all chunks are accessible
    for (i, cid) in cids.iter().enumerate() {
        let retrieved_data = new_service.retrieve(cid).unwrap();
        assert_eq!(chunks[i], retrieved_data);
    }
}

#[test]
fn test_cas_recovery_corrupted_index() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store data
    let test_data = b"Corrupted index test data";
    let cid = service.store(test_data).unwrap();
    
    // Corrupt the index file
    let index_path = Path::new(&config.storage_path).join("ngfs.idx");
    fs::write(&index_path, b"corrupted index data").unwrap();
    
    // Recreate service (should recover from corrupted index)
    let mut new_service = NgfsService::new(config).unwrap();
    
    // Verify data is still accessible
    let retrieved_data = new_service.retrieve(&cid).unwrap();
    assert_eq!(test_data, retrieved_data.as_slice());
}

#[test]
fn test_cas_recovery_missing_index() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store data
    let test_data = b"Missing index test data";
    let cid = service.store(test_data).unwrap();
    
    // Remove the index file
    let index_path = Path::new(&config.storage_path).join("ngfs.idx");
    fs::remove_file(&index_path).unwrap();
    
    // Recreate service (should rebuild index from segments)
    let mut new_service = NgfsService::new(config).unwrap();
    
    // Verify data is still accessible
    let retrieved_data = new_service.retrieve(&cid).unwrap();
    assert_eq!(test_data, retrieved_data.as_slice());
}

#[test]
fn test_cas_recovery_partial_write() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store first chunk
    let data1 = b"First chunk - complete";
    let cid1 = service.store(data1).unwrap();
    
    // Store second chunk
    let data2 = b"Second chunk - complete";
    let cid2 = service.store(data2).unwrap();
    
    // Simulate crash during third chunk write
    let data3 = b"Third chunk - incomplete";
    let _cid3 = service.store(data3).unwrap();
    
    // Simulate crash
    drop(service);
    
    // Recreate service
    let mut new_service = NgfsService::new(config).unwrap();
    
    // First two chunks should be accessible
    let retrieved_data1 = new_service.retrieve(&cid1).unwrap();
    assert_eq!(data1, retrieved_data1.as_slice());
    
    let retrieved_data2 = new_service.retrieve(&cid2).unwrap();
    assert_eq!(data2, retrieved_data2.as_slice());
}

#[test]
fn test_cas_recovery_segment_boundary() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store chunks that will span segment boundaries
    let mut cids = Vec::new();
    let mut total_size = 0;
    
    // Keep adding chunks until we cross a segment boundary
    while total_size < 8 * 1024 * 1024 { // 8 MiB segment size
        let chunk_data = format!("Chunk at size {}", total_size).into_bytes();
        let cid = service.store(&chunk_data).unwrap();
        cids.push((cid, chunk_data));
        total_size += chunk_data.len() as u64;
    }
    
    // Simulate crash
    drop(service);
    
    // Recreate service
    let mut new_service = NgfsService::new(config).unwrap();
    
    // Verify all chunks are accessible
    for (cid, original_data) in &cids {
        let retrieved_data = new_service.retrieve(cid).unwrap();
        assert_eq!(*original_data, retrieved_data);
    }
}

#[test]
fn test_cas_recovery_stats_consistency() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store some data
    let test_data = b"Stats consistency recovery test";
    let _cid = service.store(test_data).unwrap();
    
    let stats_before = service.get_stats();
    
    // Simulate crash
    drop(service);
    
    // Recreate service
    let mut new_service = NgfsService::new(config).unwrap();
    
    let stats_after = new_service.get_stats();
    
    // Stats should be consistent after recovery
    assert_eq!(stats_before.total_chunks, stats_after.total_chunks);
    assert_eq!(stats_before.total_segments, stats_after.total_segments);
}

#[test]
fn test_cas_recovery_empty_storage() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    // Create service with empty storage
    let mut service = NgfsService::new(config).unwrap();
    
    // Simulate crash without storing anything
    drop(service);
    
    // Recreate service
    let new_service = NgfsService::new(config).unwrap();
    
    // Should not panic
    let stats = new_service.get_stats();
    assert_eq!(stats.total_chunks, 0);
    assert_eq!(stats.total_segments, 0);
}

#[test]
fn test_cas_recovery_large_chunks() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store large chunks
    let large_chunk = vec![0x42u8; 200 * 1024]; // 200 KiB
    let cid = service.store(&large_chunk).unwrap();
    
    // Simulate crash
    drop(service);
    
    // Recreate service
    let mut new_service = NgfsService::new(config).unwrap();
    
    // Verify large chunk is accessible
    let retrieved_data = new_service.retrieve(&cid).unwrap();
    assert_eq!(large_chunk, retrieved_data);
}
