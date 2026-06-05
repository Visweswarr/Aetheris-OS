use criterion::{criterion_group, criterion_main, Criterion};

fn wallet_manifest_bench(c: &mut Criterion) {
    c.bench_function("wallet_manifest_smoke", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, wallet_manifest_bench);
criterion_main!(benches);
