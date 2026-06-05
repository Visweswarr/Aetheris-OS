use criterion::{criterion_group, criterion_main, Criterion};
use aetheris_ai::aicore::{CognitiveCore, CognitiveCoreConfig};

fn bench_simple_goal(c: &mut Criterion) {
    let mut group = c.benchmark_group("aicore_goal");
    group.sample_size(10);
    group.bench_function("simple_3step_goal", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            std::env::set_var("NGFS_ROOT", "./target/bench-ngfs");
            let core = CognitiveCore::new(CognitiveCoreConfig::default()).await.unwrap();
            // A simple goal that maps to a 3-step fallback
            let _ = core.plan_from_goal("test simple task").await.unwrap();
        })
    });
    group.finish();
}

criterion_group!(benches, bench_simple_goal);
criterion_main!(benches);
