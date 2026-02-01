use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn example_benchmark(c: &mut Criterion) {
    c.bench_function("example", |b| {
        b.iter(|| {
            // Add your performance-critical code here
            black_box(1 + 1)
        });
    });
}

criterion_group!(benches, example_benchmark);
criterion_main!(benches);
