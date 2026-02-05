//! Benchmarks.
#![allow(missing_docs)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::unnecessary_cast
)]

use bevy_ecs::prelude::World;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ratatui::prelude::{Color, Rect};
use scale::layer1::{
    GridPosition, Needs, Pop, PopAction, TerrainGrid, TerrainType, UtilityConfig, UtilityWeights,
    Viewport, evaluate_actions_system,
};
use scale::ui::map::{MapRenderContext, build_map_layer_spans};
use std::collections::HashMap;

fn benchmark_rendering(c: &mut Criterion) {
    let width = 80;
    let height = 50;
    let tiles = vec![TerrainType::Grass; width * height];
    let grid = TerrainGrid {
        width,
        height,
        tiles,
    };
    let viewport = Viewport { x: 0, y: 0 };
    let area = Rect::new(0, 0, 80, 50);

    // Create 1000 random pops scattered across the map
    let mut pops_data = HashMap::with_capacity(1000);
    // Use a simple deterministic loop to place pops
    for i in 0..1000 {
        // Place pops in a way that some collide, some don't, covering the grid
        let x = (i * 7) % width;
        let y = (i * 13) % height;
        pops_data.insert(
            GridPosition {
                x: x as i32,
                y: y as i32,
            },
            ("P", Color::Yellow),
        );
    }

    // Buildings empty for now to isolate pop lookup cost
    let buildings_data = HashMap::new();
    let designations_data = HashMap::new();

    c.bench_function("render_map_layer_1000_pops", |b| {
        b.iter(|| {
            let ctx = MapRenderContext {
                area: black_box(area),
                terrain: black_box(&grid),
                viewport: black_box(&viewport),
                pops_data: black_box(&pops_data),
                buildings_data: black_box(&buildings_data),
                designations_data: black_box(&designations_data),
                build_mode: black_box(None),
                designation_mode: black_box(None),
            };
            build_map_layer_spans(ctx)
        });
    });
}

fn benchmark_utility_ai(c: &mut Criterion) {
    c.bench_function("evaluate_actions_1000_pops", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                world.insert_resource(UtilityConfig::default());
                for i in 0..1000 {
                    #[allow(clippy::cast_possible_wrap)]
                    world.spawn((
                        Pop,
                        GridPosition {
                            x: (i % 80) as i32,
                            y: (i / 80) as i32,
                        },
                        Needs::default(),
                        UtilityWeights::default(),
                        PopAction {
                            ticks_committed: 100,
                            ..Default::default()
                        },
                    ));
                }
                world
            },
            |mut world| {
                evaluate_actions_system(black_box(&mut world));
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, benchmark_rendering, benchmark_utility_ai);
criterion_main!(benches);
