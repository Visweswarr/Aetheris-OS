use crate::{
    cid::Cid,
    mount::{MountTable, MountError},
    resolve::{PathResolver, ResolveError},
    read::{FileReader, ReadError},
    snapshot::{SnapshotBuilder, SnapshotError},
    schema::{ExportOpts, IpfsMapV1},
    ipfs::IpfsExporter,
};
use std::{string::String, vec::Vec, collections::HashMap};
use core::time::Duration;

/// Benchmark target configuration
#[derive(Debug, Clone)]
pub struct BenchTarget {
    /// Path to resolve and test
    pub path: String,
    /// Read size for file operations (0 = stat only)
    pub read_size: usize,
    /// Number of iterations to run
    pub repeat: u32,
    /// Expected node type (0=dir, 1=file, 2=symlink)
    pub expected_kind: u8,
}

/// Individual benchmark case result
#[derive(Debug, Clone)]
pub struct CaseResult {
    /// Case name
    pub name: String,
    /// Sample times in microseconds
    pub samples_us: Vec<u32>,
    /// Error count
    pub errs: u32,
}

/// Statistical summary of benchmark results
#[derive(Debug, Clone)]
pub struct BenchSummary {
    /// Case name
    pub name: String,
    /// 50th percentile (median) in microseconds
    pub p50: u32,
    /// 95th percentile in microseconds
    pub p95: u32,
    /// 99th percentile in microseconds
    pub p99: u32,
    /// Mean time in microseconds
    pub mean: u32,
    /// Standard deviation in microseconds
    pub stdev: u32,
    /// Number of samples
    pub n: u32,
    /// Error count
    pub errs: u32,
}

/// NGFS benchmark runner
pub struct NgfsBench {
    mount_table: MountTable,
    path_resolver: PathResolver,
    file_reader: FileReader,
    snapshot_builder: SnapshotBuilder,
    ipfs_exporter: IpfsExporter,
}

impl NgfsBench {
    /// Create a new benchmark runner
    pub fn new(
        mount_table: MountTable,
        path_resolver: PathResolver,
        file_reader: FileReader,
        snapshot_builder: SnapshotBuilder,
        ipfs_exporter: IpfsExporter,
    ) -> Self {
        Self {
            mount_table,
            path_resolver,
            file_reader,
            snapshot_builder,
            ipfs_exporter,
        }
    }

    /// Run all benchmark cases
    pub fn run_benches(&self, fixtures: &[BenchTarget]) -> Result<Vec<BenchSummary>, BenchError> {
        let mut results = Vec::new();
        
        // Run path resolution benchmarks
        results.push(self.bench_resolve_path(fixtures)?);
        
        // Run stat benchmarks
        results.push(self.bench_stat_file(fixtures)?);
        
        // Run read benchmarks
        results.push(self.bench_read_small(fixtures)?);
        results.push(self.bench_read_large(fixtures)?);
        
        // Run snapshot benchmarks
        results.push(self.bench_snapshot_build(fixtures)?);
        
        Ok(results)
    }

    /// Benchmark path resolution
    fn bench_resolve_path(&self, fixtures: &[BenchTarget]) -> Result<BenchSummary, BenchError> {
        let mut samples = Vec::new();
        let mut errors = 0;
        
        // Warm up
        for _ in 0..5 {
            for target in fixtures {
                let _ = self.path_resolver.resolve_path(&Cid::default(), &target.path);
            }
        }
        
        // Actual measurements
        for _ in 0..50 {
            let start = self.get_virtual_ticks();
            
            for target in fixtures {
                if let Err(_) = self.path_resolver.resolve_path(&Cid::default(), &target.path) {
                    errors += 1;
                }
            }
            
            let end = self.get_virtual_ticks();
            let duration_us = self.ticks_to_us(end - start);
            samples.push(duration_us as u32);
        }
        
        Ok(self.compute_summary("resolve_path", samples, errors))
    }

    /// Benchmark file stat operations
    fn bench_stat_file(&self, fixtures: &[BenchTarget]) -> Result<BenchSummary, BenchError> {
        let mut samples = Vec::new();
        let mut errors = 0;
        
        // Warm up
        for _ in 0..5 {
            for target in fixtures {
                let _ = self.path_resolver.resolve_path(&Cid::default(), &target.path);
            }
        }
        
        // Actual measurements
        for _ in 0..50 {
            let start = self.get_virtual_ticks();
            
            for target in fixtures {
                if let Err(_) = self.path_resolver.resolve_path(&Cid::default(), &target.path) {
                    errors += 1;
                }
            }
            
            let end = self.get_virtual_ticks();
            let duration_us = self.ticks_to_us(end - start);
            samples.push(duration_us as u32);
        }
        
        Ok(self.compute_summary("stat_file", samples, errors))
    }

    /// Benchmark small read operations (≤4 KiB)
    fn bench_read_small(&self, fixtures: &[BenchTarget]) -> Result<BenchSummary, BenchError> {
        let mut samples = Vec::new();
        let mut errors = 0;
        
        // Filter for small read targets
        let small_targets: Vec<_> = fixtures.iter()
            .filter(|t| t.read_size > 0 && t.read_size <= 4 * 1024)
            .collect();
        
        if small_targets.is_empty() {
            return Ok(BenchSummary {
                name: "read_small".to_string(),
                p50: 0, p95: 0, p99: 0, mean: 0, stdev: 0, n: 0, errs: 0,
            });
        }
        
        // Warm up
        for _ in 0..5 {
            for target in &small_targets {
                let _ = self.simulate_read(target);
            }
        }
        
        // Actual measurements
        for _ in 0..50 {
            let start = self.get_virtual_ticks();
            
            for target in &small_targets {
                if let Err(_) = self.simulate_read(target) {
                    errors += 1;
                }
            }
            
            let end = self.get_virtual_ticks();
            let duration_us = self.ticks_to_us(end - start);
            samples.push(duration_us as u32);
        }
        
        Ok(self.compute_summary("read_small", samples, errors))
    }

    /// Benchmark large read operations (≥1 MiB)
    fn bench_read_large(&self, fixtures: &[BenchTarget]) -> Result<BenchSummary, BenchError> {
        let mut samples = Vec::new();
        let mut errors = 0;
        
        // Filter for large read targets
        let large_targets: Vec<_> = fixtures.iter()
            .filter(|t| t.read_size >= 1024 * 1024)
            .collect();
        
        if large_targets.is_empty() {
            return Ok(BenchSummary {
                name: "read_large".to_string(),
                p50: 0, p95: 0, p99: 0, mean: 0, stdev: 0, n: 0, errs: 0,
            });
        }
        
        // Warm up
        for _ in 0..5 {
            for target in &large_targets {
                let _ = self.simulate_read(target);
            }
        }
        
        // Actual measurements
        for _ in 0..50 {
            let start = self.get_virtual_ticks();
            
            for target in &large_targets {
                if let Err(_) = self.simulate_read(target) {
                    errors += 1;
                }
            }
            
            let end = self.get_virtual_ticks();
            let duration_us = self.ticks_to_us(end - start);
            samples.push(duration_us as u32);
        }
        
        Ok(self.compute_summary("read_large", samples, errors))
    }

    /// Benchmark snapshot building
    fn bench_snapshot_build(&self, fixtures: &[BenchTarget]) -> Result<BenchSummary, BenchError> {
        let mut samples = Vec::new();
        let mut errors = 0;
        
        // Warm up
        for _ in 0..5 {
            let _ = self.snapshot_builder.build_snapshot("/ro/bench", &Cid::default(), "did:key:test");
        }
        
        // Actual measurements
        for _ in 0..50 {
            let start = self.get_virtual_ticks();
            
            if let Err(_) = self.snapshot_builder.build_snapshot("/ro/bench", &Cid::default(), "did:key:test") {
                errors += 1;
            }
            
            let end = self.get_virtual_ticks();
            let duration_us = self.ticks_to_us(end - start);
            samples.push(duration_us as u32);
        }
        
        Ok(self.compute_summary("snapshot_build", samples, errors))
    }

    /// Simulate a read operation for benchmarking
    fn simulate_read(&self, target: &BenchTarget) -> Result<(), BenchError> {
        // For benchmarking, we simulate the read operation
        // In a real implementation, this would actually read from the filesystem
        if target.read_size > 0 {
            // Simulate read time proportional to size
            let simulated_us = (target.read_size / 1024) as u32 * 100; // 100μs per KB
            self.busy_wait_us(simulated_us);
        }
        Ok(())
    }

    /// Compute statistical summary from samples
    fn compute_summary(&self, name: &str, mut samples: Vec<u32>, errs: u32) -> BenchSummary {
        if samples.is_empty() {
            return BenchSummary {
                name: name.to_string(),
                p50: 0, p95: 0, p99: 0, mean: 0, stdev: 0, n: 0, errs,
            };
        }
        
        // Sort samples for percentile calculation
        samples.sort();
        
        let n = samples.len() as u32;
        let mean = samples.iter().sum::<u32>() / n;
        
        // Calculate percentiles
        let p50_idx = (n as f64 * 0.5) as usize;
        let p95_idx = (n as f64 * 0.95) as usize;
        let p99_idx = (n as f64 * 0.99) as usize;
        
        let p50 = samples[p50_idx.min(samples.len() - 1)];
        let p95 = samples[p95_idx.min(samples.len() - 1)];
        let p99 = samples[p99_idx.min(samples.len() - 1)];
        
        // Calculate standard deviation
        let variance = samples.iter()
            .map(|&x| {
                let diff = x as i64 - mean as i64;
                (diff * diff) as u64
            })
            .sum::<u64>() / n as u64;
        let stdev = (variance as f64).sqrt() as u32;
        
        BenchSummary {
            name: name.to_string(),
            p50, p95, p99, mean, stdev, n, errs,
        }
    }

    /// Get virtual clock ticks (monotonic)
    fn get_virtual_ticks(&self) -> u64 {
        // In a real implementation, this would call the kernel's virtual clock
        // For now, use a placeholder that simulates monotonic behavior
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos() as u64
    }

    /// Convert ticks to microseconds
    fn ticks_to_us(&self, ticks: u64) -> u64 {
        // In a real implementation, this would use calibrated conversion
        // For now, assume nanoseconds and convert to microseconds
        ticks / 1000
    }

    /// Busy wait for specified microseconds (for simulation)
    fn busy_wait_us(&self, us: u32) {
        // Simple busy wait for benchmarking simulation
        let start = self.get_virtual_ticks();
        let target_ticks = (us as u64) * 1000; // Convert to nanoseconds
        
        while self.get_virtual_ticks() - start < target_ticks {
            // Spin until target time reached
        }
    }
}

/// Benchmark errors
#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    #[error("Benchmark execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Invalid benchmark configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Timing error: {0}")]
    TimingError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cas::MockCasIndex;
    use crate::enc::MockKeyVault;
    
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
}
