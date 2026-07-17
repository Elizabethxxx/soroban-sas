use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn hash_benchmark(c: &mut Criterion) {
    c.bench_function("hash_uid", |b| b.iter(|| black_box(1 + 1)));
}

criterion_group!(benches, hash_benchmark);
criterion_main!(benches);
