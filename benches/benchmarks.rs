//! Benchmarks.
#![allow(missing_docs)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::unnecessary_cast
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ratatui::prelude::{Color, Rect};
use scale::layer1::{
    GridPosition, MapRenderContext, TerrainGrid, TerrainType, Viewport, build_map_layer_spans,
};
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

criterion_group!(benches, benchmark_rendering);
criterion_main!(benches);
