use ngfs::bench::{NgfsBench, BenchTarget, BenchSummary};
use ngfs::cas::MockCasIndex;
use ngfs::enc::MockKeyVault;
use ngfs::mount::MountTable;
use ngfs::resolve::PathResolver;
use ngfs::read::FileReader;
use ngfs::snapshot::SnapshotBuilder;
use ngfs::ipfs::IpfsExporter;

#[test]
fn test_bench_target_creation() {
    let target = BenchTarget {
        path: "/test/file.txt".to_string(),
        read_size: 1024,
        repeat: 100,
        expected_kind: 1,
    };
    
    assert_eq!(target.path, "/test/file.txt");
    assert_eq!(target.read_size, 1024);
    assert_eq!(target.repeat, 100);
    assert_eq!(target.expected_kind, 1);
}

#[test]
fn test_bench_summary_computation() {
    let samples = vec![100, 200, 300, 400, 500];
    let bench = NgfsBench::new(
        MountTable::new(),
        PathResolver::new(MockCasIndex),
        FileReader::new(MockCasIndex, MockKeyVault),
        SnapshotBuilder::new(),
        IpfsExporter::new(MockCasIndex, MockKeyVault),
    );
    
    let summary = bench.compute_summary("test", samples, 0);
    
    assert_eq!(summary.name, "test");
    assert_eq!(summary.n, 5);
    assert_eq!(summary.errs, 0);
    assert_eq!(summary.mean, 300);
    assert!(summary.p50 > 0);
    assert!(summary.p95 > 0);
    assert!(summary.p99 > 0);
}

#[test]
fn test_empty_samples_handling() {
    let bench = NgfsBench::new(
        MountTable::new(),
        PathResolver::new(MockCasIndex),
        FileReader::new(MockCasIndex, MockKeyVault),
        SnapshotBuilder::new(),
        IpfsExporter::new(MockCasIndex, MockKeyVault),
    );
    
    let summary = bench.compute_summary("empty", vec![], 0);
    
    assert_eq!(summary.name, "empty");
    assert_eq!(summary.n, 0);
    assert_eq!(summary.mean, 0);
    assert_eq!(summary.p50, 0);
    assert_eq!(summary.p95, 0);
    assert_eq!(summary.p99, 0);
}

#[test]
fn test_bench_target_filtering() {
    let targets = vec![
        BenchTarget {
            path: "/test/small.txt".to_string(),
            read_size: 1024,
            repeat: 100,
            expected_kind: 1,
        },
        BenchTarget {
            path: "/test/large.txt".to_string(),
            read_size: 1024 * 1024,
            repeat: 50,
            expected_kind: 1,
        },
        BenchTarget {
            path: "/test/dir".to_string(),
            read_size: 0,
            repeat: 100,
            expected_kind: 0,
        },
    ];
    
    let small_targets: Vec<_> = targets.iter()
        .filter(|t| t.read_size > 0 && t.read_size <= 4 * 1024)
        .collect();
    
    let large_targets: Vec<_> = targets.iter()
        .filter(|t| t.read_size >= 1024 * 1024)
        .collect();
    
    assert_eq!(small_targets.len(), 1);
    assert_eq!(large_targets.len(), 1);
    assert_eq!(small_targets[0].path, "/test/small.txt");
    assert_eq!(large_targets[0].path, "/test/large.txt");
}

#[test]
fn test_percentile_calculation() {
    let bench = NgfsBench::new(
        MountTable::new(),
        PathResolver::new(MockCasIndex),
        FileReader::new(MockCasIndex, MockKeyVault),
        SnapshotBuilder::new(),
        IpfsExporter::new(MockCasIndex, MockKeyVault),
    );
    
    // Test with 100 samples for accurate percentiles
    let mut samples: Vec<u32> = (1..=100).collect();
    let summary = bench.compute_summary("percentile_test", samples, 0);
    
    assert_eq!(summary.n, 100);
    assert_eq!(summary.p50, 50);  // 50th percentile should be 50
    assert_eq!(summary.p95, 95);  // 95th percentile should be 95
    assert_eq!(summary.p99, 99);  // 99th percentile should be 99
    assert_eq!(summary.mean, 50); // Mean should be 50
}

#[test]
fn test_variance_calculation() {
    let bench = NgfsBench::new(
        MountTable::new(),
        PathResolver::new(MockCasIndex),
        FileReader::new(MockCasIndex, MockKeyVault),
        SnapshotBuilder::new(),
        IpfsExporter::new(MockCasIndex, MockKeyVault),
    );
    
    // Test with known variance
    let samples = vec![90, 100, 110]; // Mean = 100, variance = 100, stdev = 10
    let summary = bench.compute_summary("variance_test", samples, 0);
    
    assert_eq!(summary.mean, 100);
    assert_eq!(summary.stdev, 10);
    
    // Check variance ratio
    let variance_ratio = summary.stdev as f64 / summary.mean as f64;
    assert!((variance_ratio - 0.1).abs() < 0.01); // Should be 0.1
}

#[test]
fn test_bench_error_handling() {
    let bench = NgfsBench::new(
        MountTable::new(),
        PathResolver::new(MockCasIndex),
        FileReader::new(MockCasIndex, MockKeyVault),
        SnapshotBuilder::new(),
        IpfsExporter::new(MockCasIndex, MockKeyVault),
    );
    
    let samples = vec![100, 200, 300];
    let summary = bench.compute_summary("error_test", samples, 5);
    
    assert_eq!(summary.errs, 5);
    assert_eq!(summary.n, 3);
}

#[test]
fn test_bench_target_validation() {
    let valid_target = BenchTarget {
        path: "/valid/path".to_string(),
        read_size: 1024,
        repeat: 100,
        expected_kind: 1,
    };
    
    // Valid target should have reasonable values
    assert!(!valid_target.path.is_empty());
    assert!(valid_target.read_size <= 1024 * 1024 * 1024); // Max 1GB
    assert!(valid_target.repeat >= 10);
    assert!(valid_target.expected_kind <= 2);
    
    let invalid_target = BenchTarget {
        path: "".to_string(),
        read_size: 0,
        repeat: 0,
        expected_kind: 99,
    };
    
    // Invalid target should be detectable
    assert!(invalid_target.path.is_empty());
    assert_eq!(invalid_target.read_size, 0);
    assert_eq!(invalid_target.repeat, 0);
    assert!(invalid_target.expected_kind > 2);
}

#[test]
fn test_bench_summary_serialization() {
    let summary = BenchSummary {
        name: "serialization_test".to_string(),
        p50: 100,
        p95: 200,
        p99: 300,
        mean: 150,
        stdev: 50,
        n: 50,
        errs: 0,
    };
    
    // Test that summary can be serialized
    let json = serde_json::to_string(&summary).unwrap();
    let deserialized: BenchSummary = serde_json::from_str(&json).unwrap();
    
    assert_eq!(summary.name, deserialized.name);
    assert_eq!(summary.p50, deserialized.p50);
    assert_eq!(summary.p95, deserialized.p95);
    assert_eq!(summary.p99, deserialized.p99);
    assert_eq!(summary.mean, deserialized.mean);
    assert_eq!(summary.stdev, deserialized.stdev);
    assert_eq!(summary.n, deserialized.n);
    assert_eq!(summary.errs, deserialized.errs);
}

#[test]
fn test_bench_target_cloning() {
    let target = BenchTarget {
        path: "/test/clone.txt".to_string(),
        read_size: 2048,
        repeat: 200,
        expected_kind: 1,
    };
    
    let cloned = target.clone();
    
    assert_eq!(target.path, cloned.path);
    assert_eq!(target.read_size, cloned.read_size);
    assert_eq!(target.repeat, cloned.repeat);
    assert_eq!(target.expected_kind, cloned.expected_kind);
}

#[test]
fn test_bench_summary_cloning() {
    let summary = BenchSummary {
        name: "clone_test".to_string(),
        p50: 75,
        p95: 150,
        p99: 225,
        mean: 100,
        stdev: 25,
        n: 100,
        errs: 1,
    };
    
    let cloned = summary.clone();
    
    assert_eq!(summary.name, cloned.name);
    assert_eq!(summary.p50, cloned.p50);
    assert_eq!(summary.p95, cloned.p95);
    assert_eq!(summary.p99, cloned.p99);
    assert_eq!(summary.mean, cloned.mean);
    assert_eq!(summary.stdev, cloned.stdev);
    assert_eq!(summary.n, cloned.n);
    assert_eq!(summary.errs, cloned.errs);
}
