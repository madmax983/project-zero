use std::hint::black_box;
use criterion::{ criterion_group, criterion_main, Criterion};
use scale::layer1::map::GridPosition;
use scale::layer1::utility_types::{calculate_context_score, UtilityWeights};

fn benchmark_context_score(c: &mut Criterion) {
    let weights = UtilityWeights::default();
    let pop_pos = GridPosition { x: 0, y: 0 };
    let target_pos = Some(GridPosition { x: 10, y: 10 });
    let capacity = 10;
    let occupied = 5;

    c.bench_function("calculate_context_score_default", |b: &mut criterion::Bencher| {
        b.iter(|| {
            calculate_context_score(
                black_box(pop_pos),
                black_box(target_pos),
                black_box(capacity),
                black_box(occupied),
                black_box(&weights),
            )
        })
    });

    let mut varied_weights = UtilityWeights::default();
    varied_weights.distance_weight = 1.5;
    varied_weights.availability_weight = 0.8;

    c.bench_function("calculate_context_score_varied", |b: &mut criterion::Bencher| {
        b.iter(|| {
            calculate_context_score(
                black_box(pop_pos),
                black_box(target_pos),
                black_box(capacity),
                black_box(occupied),
                black_box(&varied_weights),
            )
        })
    });
}

criterion_group!(benches, benchmark_context_score);
criterion_main!(benches);
