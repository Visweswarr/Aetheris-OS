use criterion::{criterion_group, criterion_main, Criterion};
use aetheris_ai::aicore::{CognitiveCore, CognitiveCoreConfig};
use std::time::Instant;

fn bench_submit_goal_low_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("submit_goal");
    group.sample_size(10);
    group.bench_function("submit_once", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            std::env::set_var("NGFS_ROOT", "./target/bench-ngfs");
            let core = CognitiveCore::new(CognitiveCoreConfig::default()).await.unwrap();
            let _ = core.submit_goal("hello").await;
        })
    });
    group.finish();
}

fn bench_snapshot_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_write");
    group.sample_size(10);
    group.bench_function("snapshot_once", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            std::env::set_var("NGFS_ROOT", "./target/bench-ngfs");
            let core = CognitiveCore::new(CognitiveCoreConfig::default()).await.unwrap();
            let _ = core.submit_goal("hello").await;
            let _ = core.snapshot_write().await.unwrap();
        })
    });
    group.finish();
}

fn bench_snapshot_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_read");
    group.sample_size(10);
    group.bench_function("snapshot_replay", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            std::env::set_var("NGFS_ROOT", "./target/bench-ngfs");
            let core = CognitiveCore::new(CognitiveCoreConfig::default()).await.unwrap();
            let _ = core.submit_goal("hello").await;
            let id = core.snapshot_write().await.unwrap();
            core.snapshot_replay(&id).await.unwrap();
        })
    });
    group.finish();
}

criterion_group!(benches, bench_submit_goal_low_contention, bench_snapshot_write, bench_snapshot_read);
criterion_main!(benches);
