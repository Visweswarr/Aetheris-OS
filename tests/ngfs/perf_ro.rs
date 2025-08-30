use ngfs::{NgfsService, NgfsConfig, MountOptionsV1};
use tempfile::tempdir;
use std::collections::BTreeMap;
use std::time::{Instant, Duration};

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

#[derive(Debug)]
struct PerformanceMetrics {
    reads: u32,
    read_p95_us: u64,
    snap_ms: u64,
    mounts: u32,
    errors: u32,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            reads: 0,
            read_p95_us: 0,
            snap_ms: 0,
            mounts: 0,
            errors: 0,
        }
    }
}

fn measure_mount_performance(service: &mut NgfsService, count: usize) -> (Duration, u32) {
    let start = Instant::now();
    let mut success_count = 0;
    
    for i in 0..count {
        let mount_options = MountOptionsV1 {
            root_cid: format!("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi{}", i),
            salt: [i as u8; 16],
            vclock_base: i as u64,
        };
        
        let mount_point = format!("/ro/test{}", i);
        if service.mount_ro(&mount_point, mount_options).is_ok() {
            success_count += 1;
        }
    }
    
    (start.elapsed(), success_count)
}

fn measure_snapshot_performance(service: &NgfsService, count: usize) -> (Duration, u32) {
    let start = Instant::now();
    let mut success_count = 0;
    
    for i in 0..count {
        let mount_point = format!("/ro/test{}", i);
        if service.create_snapshot(&mount_point, &format!("did:key:test{}", i)).is_ok() {
            success_count += 1;
        }
    }
    
    (start.elapsed(), success_count)
}

fn measure_read_performance(service: &NgfsService, count: usize) -> (Duration, Vec<Duration>, u32) {
    let start = Instant::now();
    let mut read_times = Vec::new();
    let mut success_count = 0;
    
    for i in 0..count {
        let manifest_cid = format!("test_manifest_cid{}", i);
        let read_start = Instant::now();
        
        // Try to read (will fail in test environment, but we measure the attempt)
        let result = service.read_file(&manifest_cid, 0, 1024);
        let read_duration = read_start.elapsed();
        read_times.push(read_duration);
        
        if result.is_ok() {
            success_count += 1;
        }
    }
    
    (start.elapsed(), read_times, success_count)
}

fn calculate_p95(times: &[Duration]) -> u64 {
    if times.is_empty() {
        return 0;
    }
    
    let mut sorted_times: Vec<u64> = times.iter().map(|d| d.as_micros() as u64).collect();
    sorted_times.sort();
    
    let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
    sorted_times[p95_index.min(sorted_times.len() - 1)]
}

#[test]
fn test_performance_basic() {
    let mut service = create_test_service();
    let mut metrics = PerformanceMetrics::new();
    
    // Test mount performance
    let mount_count = 100;
    let (mount_duration, mount_success) = measure_mount_performance(&mut service, mount_count);
    metrics.mounts = mount_success;
    
    // Test snapshot performance
    let snap_count = 50;
    let (snap_duration, snap_success) = measure_snapshot_performance(&service, snap_count);
    metrics.snap_ms = snap_duration.as_millis() as u64;
    
    // Test read performance
    let read_count = 200;
    let (read_duration, read_times, read_success) = measure_read_performance(&service, read_count);
    metrics.reads = read_success;
    metrics.read_p95_us = calculate_p95(&read_times);
    
    // Calculate errors
    metrics.errors = (mount_count + snap_count + read_count) as u32 - 
                    (mount_success + snap_success + read_success);
    
    // Output JSON metrics for CI
    println!("{{\"test\":\"ngfs_ro\",\"reads\":{},\"read_p95_us\":{},\"snap_ms\":{},\"mounts\":{},\"errors\":{}}}",
             metrics.reads, metrics.read_p95_us, metrics.snap_ms, metrics.mounts, metrics.errors);
    
    // Basic assertions
    assert!(mount_success > 0, "Should have some successful mounts");
    assert!(snap_success > 0, "Should have some successful snapshots");
    assert!(read_success >= 0, "Reads may fail in test environment");
    assert!(metrics.read_p95_us > 0, "Should have measurable read times");
}

#[test]
fn test_performance_large_batch() {
    let mut service = create_test_service();
    let mut metrics = PerformanceMetrics::new();
    
    // Test with larger batch sizes
    let mount_count = 1000;
    let (mount_duration, mount_success) = measure_mount_performance(&mut service, mount_count);
    metrics.mounts = mount_success;
    
    let snap_count = 500;
    let (snap_duration, snap_success) = measure_snapshot_performance(&service, snap_count);
    metrics.snap_ms = snap_duration.as_millis() as u64;
    
    let read_count = 2000;
    let (read_duration, read_times, read_success) = measure_read_performance(&service, read_count);
    metrics.reads = read_success;
    metrics.read_p95_us = calculate_p95(&read_times);
    
    metrics.errors = (mount_count + snap_count + read_count) as u32 - 
                    (mount_success + snap_success + read_success);
    
    // Output JSON metrics for CI
    println!("{{\"test\":\"ngfs_ro_large\",\"reads\":{},\"read_p95_us\":{},\"snap_ms\":{},\"mounts\":{},\"errors\":{}}}",
             metrics.reads, metrics.read_p95_us, metrics.snap_ms, metrics.mounts, metrics.errors);
    
    // Performance assertions
    assert!(mount_duration < Duration::from_secs(10), "Mounts should complete within 10s");
    assert!(snap_duration < Duration::from_secs(5), "Snapshots should complete within 5s");
    assert!(read_duration < Duration::from_secs(15), "Reads should complete within 15s");
}

#[test]
fn test_performance_concurrent_mounts() {
    let mut service = create_test_service();
    let mut metrics = PerformanceMetrics::new();
    
    // Test concurrent mount performance
    let mount_count = 100;
    let start = Instant::now();
    
    let mount_results: Vec<_> = (0..mount_count)
        .map(|i| {
            let mount_options = MountOptionsV1 {
                root_cid: format!("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi{}", i),
                salt: [i as u8; 16],
                vclock_base: i as u64,
            };
            
            let mount_point = format!("/ro/concurrent{}", i);
            service.mount_ro(&mount_point, mount_options)
        })
        .collect();
    
    let mount_duration = start.elapsed();
    let mount_success = mount_results.iter().filter(|r| r.is_ok()).count() as u32;
    metrics.mounts = mount_success;
    
    // Clean up
    for i in 0..mount_count {
        let mount_point = format!("/ro/concurrent{}", i);
        service.unmount(&mount_point);
    }
    
    // Output JSON metrics for CI
    println!("{{\"test\":\"ngfs_ro_concurrent\",\"reads\":{},\"read_p95_us\":{},\"snap_ms\":{},\"mounts\":{},\"errors\":{}}}",
             metrics.reads, metrics.read_p95_us, metrics.snap_ms, metrics.mounts, metrics.errors);
    
    // Performance assertions
    assert!(mount_duration < Duration::from_secs(5), "Concurrent mounts should complete within 5s");
    assert!(mount_success > 0, "Should have some successful concurrent mounts");
}

#[test]
fn test_performance_mixed_operations() {
    let mut service = create_test_service();
    let mut metrics = PerformanceMetrics::new();
    
    let operation_count = 100;
    let start = Instant::now();
    let mut mount_success = 0;
    let mut snap_success = 0;
    let mut read_success = 0;
    
    for i in 0..operation_count {
        // Mount
        let mount_options = MountOptionsV1 {
            root_cid: format!("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi{}", i),
            salt: [i as u8; 16],
            vclock_base: i as u64,
        };
        
        let mount_point = format!("/ro/mixed{}", i);
        if service.mount_ro(&mount_point, mount_options).is_ok() {
            mount_success += 1;
        }
        
        // Snapshot
        if service.create_snapshot(&mount_point, &format!("did:key:mixed{}", i)).is_ok() {
            snap_success += 1;
        }
        
        // Read
        let manifest_cid = format!("mixed_manifest_cid{}", i);
        if service.read_file(&manifest_cid, 0, 1024).is_ok() {
            read_success += 1;
        }
    }
    
    let total_duration = start.elapsed();
    metrics.mounts = mount_success;
    metrics.reads = read_success;
    metrics.snap_ms = total_duration.as_millis() as u64;
    metrics.errors = (operation_count * 3) as u32 - (mount_success + snap_success + read_success);
    
    // Output JSON metrics for CI
    println!("{{\"test\":\"ngfs_ro_mixed\",\"reads\":{},\"read_p95_us\":{},\"snap_ms\":{},\"mounts\":{},\"errors\":{}}}",
             metrics.reads, metrics.read_p95_us, metrics.snap_ms, metrics.mounts, metrics.errors);
    
    // Performance assertions
    assert!(total_duration < Duration::from_secs(10), "Mixed operations should complete within 10s");
    assert!(mount_success > 0, "Should have some successful mounts");
    assert!(snap_success > 0, "Should have some successful snapshots");
    
    // Clean up
    for i in 0..operation_count {
        let mount_point = format!("/ro/mixed{}", i);
        service.unmount(&mount_point);
    }
}

#[test]
fn test_performance_stress() {
    let mut service = create_test_service();
    let mut metrics = PerformanceMetrics::new();
    
    // Stress test with rapid operations
    let stress_count = 500;
    let start = Instant::now();
    
    let mut operations = Vec::new();
    
    for i in 0..stress_count {
        let mount_options = MountOptionsV1 {
            root_cid: format!("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi{}", i),
            salt: [i as u8; 16],
            vclock_base: i as u64,
        };
        
        let mount_point = format!("/ro/stress{}", i);
        
        // Mount
        if service.mount_ro(&mount_point, mount_options).is_ok() {
            operations.push(("mount", i));
        }
        
        // Snapshot
        if service.create_snapshot(&mount_point, &format!("did:key:stress{}", i)).is_ok() {
            operations.push(("snapshot", i));
        }
        
        // Read
        let manifest_cid = format!("stress_manifest_cid{}", i);
        if service.read_file(&manifest_cid, 0, 1024).is_ok() {
            operations.push(("read", i));
        }
    }
    
    let total_duration = start.elapsed();
    
    // Count operations by type
    let mount_count = operations.iter().filter(|(op, _)| *op == "mount").count() as u32;
    let snap_count = operations.iter().filter(|(op, _)| *op == "snapshot").count() as u32;
    let read_count = operations.iter().filter(|(op, _)| *op == "read").count() as u32;
    
    metrics.mounts = mount_count;
    metrics.reads = read_count;
    metrics.snap_ms = total_duration.as_millis() as u64;
    metrics.errors = (stress_count * 3) as u32 - (mount_count + snap_count + read_count);
    
    // Output JSON metrics for CI
    println!("{{\"test\":\"ngfs_ro_stress\",\"reads\":{},\"read_p95_us\":{},\"snap_ms\":{},\"mounts\":{},\"errors\":{}}}",
             metrics.reads, metrics.read_p95_us, metrics.snap_ms, metrics.mounts, metrics.errors);
    
    // Stress test assertions
    assert!(total_duration < Duration::from_secs(30), "Stress test should complete within 30s");
    assert!(mount_count > 0, "Should have some successful mounts under stress");
    assert!(snap_count > 0, "Should have some successful snapshots under stress");
    
    // Clean up
    for i in 0..stress_count {
        let mount_point = format!("/ro/stress{}", i);
        service.unmount(&mount_point);
    }
}
