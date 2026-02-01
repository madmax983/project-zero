#![allow(missing_docs)]
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn example_benchmark(c: &mut Criterion) {
    c.bench_function("example", |b| {
        b.iter(|| {
            // Benchmark logic here
            let x = black_box(2);
            let y = black_box(3);
            let _ = x + y;
        });
    });
}

criterion_group!(benches, example_benchmark);
criterion_main!(benches);
