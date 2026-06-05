use criterion::{criterion_group, criterion_main, Criterion};

fn contract_execution_manifest_bench(c: &mut Criterion) {
    c.bench_function("contract_execution_manifest_smoke", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, contract_execution_manifest_bench);
criterion_main!(benches);
