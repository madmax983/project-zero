//! Benchmarks.
#![allow(missing_docs)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::unnecessary_cast
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ratatui::prelude::{Color, Rect};
use scale::layer1::{GridPosition, TerrainGrid, TerrainType, Viewport};
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
    let items_data = HashMap::new();

    c.bench_function("render_map_layer_1000_pops", |b| {
        b.iter(|| {
            let ctx = MapRenderContext {
                area: black_box(area),
                terrain: black_box(&grid),
                viewport: black_box(&viewport),
                pops_data: black_box(&pops_data),
                buildings_data: black_box(&buildings_data),
                designations_data: black_box(&designations_data),
                items_data: black_box(&items_data),
                build_mode: black_box(None),
                designation_mode: black_box(None),
            };
            build_map_layer_spans(ctx)
        });
    });
}

// ---------------------------------------------------------------------------
// Utility AI: CPU vs GPU benchmarks
// ---------------------------------------------------------------------------

use bevy_ecs::prelude::*;
use scale::gpu::context::GpuContext;
use scale::gpu::evaluate::gpu_evaluate_actions;
use scale::layer1::designation::{Designation, DesignationType};
use scale::layer1::farm::Farm;
use scale::layer1::housing::Housing;
use scale::layer1::needs::Needs;
use scale::layer1::resources::ColonyResources;
use scale::layer1::social::Tavern;
use scale::layer1::utility_ai::types::{ActionType, PopAction, UtilityConfig, UtilityWeights};
use scale::layer1::utility_ai::evaluate_actions_system;

/// Build a minimal world with `n_pops` pops and a fixed set of buildings.
fn make_bench_world(n_pops: usize, gpu_ctx: Option<GpuContext>) -> World {
    scale::setup::init_task_pools();
    let mut world = World::new();
    world.insert_resource(UtilityConfig::default());
    world.insert_resource(ColonyResources::default());

    if let Some(ctx) = gpu_ctx {
        world.insert_resource(ctx);
    }

    // Spawn buildings: 3 farms, 2 housing, 1 tavern, 2 designations
    for i in 0..3 {
        world.spawn((
            GridPosition { x: i * 10, y: 5 },
            Farm {
                capacity: 4,
                workers: vec![],
            },
        ));
    }
    for i in 0..2 {
        world.spawn((
            GridPosition { x: i * 15, y: 20 },
            Housing {
                capacity: 6,
                residents: vec![],
            },
        ));
    }
    world.spawn((
        GridPosition { x: 25, y: 25 },
        Tavern {
            capacity: 8,
            visitors: vec![],
        },
    ));
    for i in 0..2 {
        world.spawn((
            GridPosition {
                x: 30 + i * 5,
                y: 30,
            },
            Designation {
                designation_type: DesignationType::Mine,
            },
        ));
    }

    // Spawn pops spread across the map, all eligible for evaluation
    for i in 0..n_pops {
        let x = (i * 7 % 80) as i32;
        let y = (i * 13 % 50) as i32;
        let hunger = 0.3 + (i % 5) as f32 * 0.1;
        let rest = 0.4 + (i % 4) as f32 * 0.1;
        let leisure = 0.5 + (i % 3) as f32 * 0.1;
        world.spawn((
            GridPosition { x, y },
            Needs {
                hunger,
                rest,
                leisure,
            },
            UtilityWeights::default(),
            PopAction {
                current: ActionType::Idle,
                current_utility: 0.0,
                ticks_committed: 10, // eligible for evaluation
            },
        ));
    }

    world
}

fn benchmark_utility_ai(c: &mut Criterion) {
    // Try to create GPU context once (shared across benchmarks)
    let gpu_result =
        std::panic::catch_unwind(|| pollster::block_on(GpuContext::new()));
    let gpu_available = matches!(&gpu_result, Ok(Ok(_)));

    let mut group = c.benchmark_group("utility_ai_evaluate");

    for &n_pops in &[5, 50, 200, 500] {
        // CPU benchmark
        group.bench_function(format!("cpu_{n_pops}_pops"), |b| {
            let mut world = make_bench_world(n_pops, None);
            b.iter(|| {
                evaluate_actions_system(black_box(&mut world));
            });
        });

        // GPU benchmark (only if GPU is available)
        if gpu_available {
            group.bench_function(format!("gpu_{n_pops}_pops"), |b| {
                // Each iteration needs a fresh GpuContext since it's moved into the world
                let ctx = pollster::block_on(GpuContext::new()).unwrap();
                let mut world = make_bench_world(n_pops, Some(ctx));
                b.iter(|| {
                    gpu_evaluate_actions(black_box(&mut world));
                });
            });
        }
    }

    if !gpu_available {
        eprintln!("GPU not available — GPU benchmarks skipped");
    }

    group.finish();
}

criterion_group!(benches, benchmark_rendering, benchmark_utility_ai);
criterion_main!(benches);
