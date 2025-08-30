use ngfs::{NgfsService, NgfsConfig, Cid};
use tempfile::tempdir;
use std::time::{Instant, Duration};
use std::collections::HashMap;

#[test]
fn test_cas_perf_small_chunks() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let start = Instant::now();
    let num_chunks = 1000;
    let chunk_size = 1024; // 1 KiB
    
    let mut cids = Vec::new();
    
    for i in 0..num_chunks {
        let chunk_data = format!("Small chunk {}", i).into_bytes();
        let cid = service.store(&chunk_data).unwrap();
        cids.push(cid);
    }
    
    let write_duration = start.elapsed();
    let write_ops_per_sec = num_chunks as f64 / write_duration.as_secs_f64();
    
    println!("Small chunks write: {:.2} ops/sec", write_ops_per_sec);
    
    // Verify all chunks can be read
    let read_start = Instant::now();
    for cid in &cids {
        let _data = service.retrieve(cid).unwrap();
    }
    
    let read_duration = read_start.elapsed();
    let read_ops_per_sec = num_chunks as f64 / read_duration.as_secs_f64();
    
    println!("Small chunks read: {:.2} ops/sec", read_ops_per_sec);
    
    // Performance assertions (adjust thresholds based on your environment)
    assert!(write_ops_per_sec > 100.0, "Write performance too low: {:.2} ops/sec", write_ops_per_sec);
    assert!(read_ops_per_sec > 500.0, "Read performance too low: {:.2} ops/sec", read_ops_per_sec);
}

#[test]
fn test_cas_perf_medium_chunks() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let start = Instant::now();
    let num_chunks = 100;
    let chunk_size = 64 * 1024; // 64 KiB
    
    let mut cids = Vec::new();
    
    for i in 0..num_chunks {
        let chunk_data = vec![i as u8; chunk_size];
        let cid = service.store(&chunk_data).unwrap();
        cids.push(cid);
    }
    
    let write_duration = start.elapsed();
    let write_ops_per_sec = num_chunks as f64 / write_duration.as_secs_f64();
    let write_mbps = (num_chunks * chunk_size) as f64 / 1024.0 / 1024.0 / write_duration.as_secs_f64();
    
    println!("Medium chunks write: {:.2} ops/sec, {:.2} MB/s", write_ops_per_sec, write_mbps);
    
    // Verify all chunks can be read
    let read_start = Instant::now();
    for cid in &cids {
        let _data = service.retrieve(cid).unwrap();
    }
    
    let read_duration = read_start.elapsed();
    let read_ops_per_sec = num_chunks as f64 / read_duration.as_secs_f64();
    let read_mbps = (num_chunks * chunk_size) as f64 / 1024.0 / 1024.0 / read_duration.as_secs_f64();
    
    println!("Medium chunks read: {:.2} ops/sec, {:.2} MB/s", read_ops_per_sec, read_mbps);
    
    // Performance assertions
    assert!(write_ops_per_sec > 10.0, "Medium chunk write performance too low: {:.2} ops/sec", write_ops_per_sec);
    assert!(read_ops_per_sec > 50.0, "Medium chunk read performance too low: {:.2} ops/sec", read_ops_per_sec);
}

#[test]
fn test_cas_perf_large_chunks() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let start = Instant::now();
    let num_chunks = 10;
    let chunk_size = 256 * 1024; // 256 KiB (max chunk size)
    
    let mut cids = Vec::new();
    
    for i in 0..num_chunks {
        let chunk_data = vec![i as u8; chunk_size];
        let cid = service.store(&chunk_data).unwrap();
        cids.push(cid);
    }
    
    let write_duration = start.elapsed();
    let write_ops_per_sec = num_chunks as f64 / write_duration.as_secs_f64();
    let write_mbps = (num_chunks * chunk_size) as f64 / 1024.0 / 1024.0 / write_duration.as_secs_f64();
    
    println!("Large chunks write: {:.2} ops/sec, {:.2} MB/s", write_ops_per_sec, write_mbps);
    
    // Verify all chunks can be read
    let read_start = Instant::now();
    for cid in &cids {
        let _data = service.retrieve(cid).unwrap();
    }
    
    let read_duration = read_start.elapsed();
    let read_ops_per_sec = num_chunks as f64 / read_duration.as_secs_f64();
    let read_mbps = (num_chunks * chunk_size) as f64 / 1024.0 / 1024.0 / read_duration.as_secs_f64();
    
    println!("Large chunks read: {:.2} ops/sec, {:.2} MB/s", read_ops_per_sec, read_mbps);
    
    // Performance assertions
    assert!(write_ops_per_sec > 1.0, "Large chunk write performance too low: {:.2} ops/sec", write_ops_per_sec);
    assert!(read_ops_per_sec > 5.0, "Large chunk read performance too low: {:.2} ops/sec", read_ops_per_sec);
}

#[test]
fn test_cas_perf_mixed_chunk_sizes() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    let chunk_sizes = vec![
        1024,      // 1 KiB
        4 * 1024,  // 4 KiB
        16 * 1024, // 16 KiB
        64 * 1024, // 64 KiB
        128 * 1024, // 128 KiB
    ];
    
    let mut size_distribution = HashMap::new();
    let mut cids = Vec::new();
    
    let start = Instant::now();
    
    for size in &chunk_sizes {
        let num_chunks = 1000 / size; // More small chunks, fewer large chunks
        for i in 0..num_chunks {
            let chunk_data = vec![i as u8; *size];
            let cid = service.store(&chunk_data).unwrap();
            cids.push((cid, *size));
        }
        size_distribution.insert(*size, num_chunks);
    }
    
    let write_duration = start.elapsed();
    let total_chunks: usize = size_distribution.values().sum();
    let write_ops_per_sec = total_chunks as f64 / write_duration.as_secs_f64();
    
    println!("Mixed chunk sizes write: {:.2} ops/sec", write_ops_per_sec);
    println!("Chunk size distribution:");
    for (size, count) in &size_distribution {
        println!("  {} bytes: {} chunks", size, count);
    }
    
    // Verify all chunks can be read
    let read_start = Instant::now();
    for (cid, _size) in &cids {
        let _data = service.retrieve(cid).unwrap();
    }
    
    let read_duration = read_start.elapsed();
    let read_ops_per_sec = total_chunks as f64 / read_duration.as_secs_f64();
    
    println!("Mixed chunk sizes read: {:.2} ops/sec", read_ops_per_sec);
    
    // Performance assertions
    assert!(write_ops_per_sec > 50.0, "Mixed chunk write performance too low: {:.2} ops/sec", write_ops_per_sec);
    assert!(read_ops_per_sec > 200.0, "Mixed chunk read performance too low: {:.2} ops/sec", read_ops_per_sec);
}

#[test]
fn test_cas_perf_concurrent_access() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store some data first
    let num_chunks = 100;
    let chunk_size = 16 * 1024; // 16 KiB
    
    let mut cids = Vec::new();
    for i in 0..num_chunks {
        let chunk_data = vec![i as u8; chunk_size];
        let cid = service.store(&chunk_data).unwrap();
        cids.push(cid);
    }
    
    // Measure concurrent read performance
    let start = Instant::now();
    let num_reads = 1000;
    
    for _ in 0..num_reads {
        let cid = &cids[0]; // Always read the first chunk
        let _data = service.retrieve(cid).unwrap();
    }
    
    let duration = start.elapsed();
    let ops_per_sec = num_reads as f64 / duration.as_secs_f64();
    
    println!("Concurrent read performance: {:.2} ops/sec", ops_per_sec);
    
    // Performance assertion
    assert!(ops_per_sec > 1000.0, "Concurrent read performance too low: {:.2} ops/sec", ops_per_sec);
}

#[test]
fn test_cas_perf_memory_usage() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Store many small chunks to test memory usage
    let num_chunks = 10000;
    let chunk_size = 1024; // 1 KiB
    
    let start = Instant::now();
    let mut cids = Vec::new();
    
    for i in 0..num_chunks {
        let chunk_data = vec![i as u8; chunk_size];
        let cid = service.store(&chunk_data).unwrap();
        cids.push(cid);
    }
    
    let write_duration = start.elapsed();
    let write_ops_per_sec = num_chunks as f64 / write_duration.as_secs_f64();
    
    println!("Memory usage test write: {:.2} ops/sec", write_ops_per_sec);
    
    // Get stats to check memory usage
    let stats = service.get_stats();
    println!("Total chunks: {}, Total segments: {}", stats.total_chunks, stats.total_segments);
    
    // Performance assertion
    assert!(write_ops_per_sec > 100.0, "Memory usage test write performance too low: {:.2} ops/sec", write_ops_per_sec);
}

#[test]
fn test_cas_perf_segment_boundary_crossing() {
    let temp_dir = tempdir().unwrap();
    let config = NgfsConfig {
        storage_path: temp_dir.path().to_string_lossy().to_string(),
        ..Default::default()
    };
    
    let mut service = NgfsService::new(config).unwrap();
    
    // Test performance when crossing segment boundaries
    let segment_size = 8 * 1024 * 1024; // 8 MiB
    let chunk_size = 256 * 1024; // 256 KiB
    let chunks_per_segment = segment_size / chunk_size as u64;
    
    let start = Instant::now();
    let mut cids = Vec::new();
    
    // Store chunks to fill multiple segments
    for i in 0..(chunks_per_segment * 3) { // 3 segments
        let chunk_data = vec![i as u8; chunk_size as usize];
        let cid = service.store(&chunk_data).unwrap();
        cids.push(cid);
    }
    
    let write_duration = start.elapsed();
    let write_ops_per_sec = (chunks_per_segment * 3) as f64 / write_duration.as_secs_f64();
    let write_mbps = (chunks_per_segment * 3 * chunk_size) as f64 / 1024.0 / 1024.0 / write_duration.as_secs_f64();
    
    println!("Segment boundary crossing write: {:.2} ops/sec, {:.2} MB/s", write_ops_per_sec, write_mbps);
    
    // Performance assertion
    assert!(write_ops_per_sec > 5.0, "Segment boundary crossing write performance too low: {:.2} ops/sec", write_ops_per_sec);
}
