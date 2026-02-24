//! Benchmarks.
#![allow(missing_docs)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::unnecessary_cast
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ratatui::prelude::{Color, Rect};
use scale::layer1::building::{BuildingMap, OccupiedTiles};
use scale::layer1::pathfinding::find_path;
use scale::layer1::water::WaterGrid;
use scale::layer1::{BuildingType, GridPosition, MaterialType, TerrainGrid, TerrainType, Viewport};
use scale::ui::map::{MapRenderContext, RenderEntity, build_map_layer_spans};
use std::collections::HashMap;

fn setup_pathfinding_world(width: usize, height: usize) -> bevy_ecs::world::World {
    let mut world = bevy_ecs::world::World::new();
    let tiles = vec![TerrainType::Grass; width * height];
    world.insert_resource(TerrainGrid {
        width,
        height,
        tiles,
    });
    world.insert_resource(OccupiedTiles::default());
    world.insert_resource(BuildingMap::default());
    // Add wind/crowding if needed, but they are optional in find_path_internal
    world
}

fn benchmark_pathfinding(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathfinding");

    // Test case 1: Short path on small map
    let world_small = setup_pathfinding_world(100, 100);
    group.bench_function("path_100x100_short", |b| {
        b.iter(|| {
            find_path(
                black_box(&world_small),
                black_box((0, 0)),
                black_box((20, 20)),
            )
        })
    });

    // Test case 2: Long path on small map
    group.bench_function("path_100x100_long", |b| {
        b.iter(|| {
            find_path(
                black_box(&world_small),
                black_box((0, 0)),
                black_box((90, 90)),
            )
        })
    });

    // Test case 3: Unreachable path (flood fill worst case)
    // Create a wall
    let mut world_blocked = setup_pathfinding_world(50, 50);
    {
        let mut occ = world_blocked.resource_mut::<OccupiedTiles>();
        // Wall off (40, 0) to (40, 50)
        for y in 0..50 {
            occ.0.insert((40, y));
        }
    }
    // Try to go from (0,0) to (45, 25) - blocked
    group.bench_function("path_50x50_blocked", |b| {
        b.iter(|| {
            find_path(
                black_box(&world_blocked),
                black_box((0, 0)),
                black_box((45, 25)),
            )
        })
    });

    group.finish();
}

fn benchmark_rendering_buildings(c: &mut Criterion) {
    let width = 80;
    let height = 50;
    let tiles = vec![TerrainType::Grass; width * height];
    let grid = TerrainGrid {
        width,
        height,
        tiles,
    };
    let water_grid = WaterGrid::new(width, height);
    let viewport = Viewport { x: 0, y: 0 };
    let area = Rect::new(0, 0, 80, 50);

    // Create 1000 random buildings scattered across the map
    let mut entities_data = HashMap::with_capacity(1000);
    // Use a simple deterministic loop to place buildings
    for i in 0..1000 {
        let x = (i * 7) % width;
        let y = (i * 13) % height;
        entities_data.insert(
            GridPosition {
                x: x as i32,
                y: y as i32,
            },
            RenderEntity::Building(BuildingType::Housing, MaterialType::Wood, false),
        );
    }

    c.bench_function("render_map_layer_1000_buildings", |b| {
        b.iter(|| {
            let ctx = MapRenderContext {
                area: black_box(area),
                terrain: black_box(&grid),
                water: black_box(&water_grid),
                viewport: black_box(&viewport),
                entities_data: black_box(&entities_data),
                build_mode: black_box(None),
                designation_mode: black_box(None),
                season: black_box(None),
                wall_time: 0.0,
            };
            build_map_layer_spans(ctx)
        });
    });
}

fn benchmark_rendering(c: &mut Criterion) {
    let width = 80;
    let height = 50;
    let tiles = vec![TerrainType::Grass; width * height];
    let grid = TerrainGrid {
        width,
        height,
        tiles,
    };
    let water_grid = WaterGrid::new(width, height);
    let viewport = Viewport { x: 0, y: 0 };
    let area = Rect::new(0, 0, 80, 50);

    // Create 1000 random pops scattered across the map
    let mut entities_data = HashMap::with_capacity(1000);
    // Use a simple deterministic loop to place pops
    for i in 0..1000 {
        // Place pops in a way that some collide, some don't, covering the grid
        let x = (i * 7) % width;
        let y = (i * 13) % height;
        entities_data.insert(
            GridPosition {
                x: x as i32,
                y: y as i32,
            },
            RenderEntity::Pop("P", Color::Yellow),
        );
    }

    c.bench_function("render_map_layer_1000_pops", |b| {
        b.iter(|| {
            let ctx = MapRenderContext {
                area: black_box(area),
                terrain: black_box(&grid),
                water: black_box(&water_grid),
                viewport: black_box(&viewport),
                entities_data: black_box(&entities_data),
                build_mode: black_box(None),
                designation_mode: black_box(None),
                season: black_box(None),
                wall_time: 0.0,
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
use scale::layer1::items::ItemType;
use scale::layer1::needs::Needs;
use scale::layer1::resources::ColonyResources;
use scale::layer1::social::Tavern;
use scale::layer1::utility_ai::evaluate_actions_system;
use scale::layer1::utility_types::{ActionType, PopAction, UtilityConfig, UtilityWeights};

/// Build a minimal world with `n_pops` pops and `n_buildings` buildings.
fn make_bench_world(n_pops: usize, n_buildings: usize, gpu_ctx: Option<GpuContext>) -> World {
    scale::setup::init_task_pools();
    let mut world = World::new();
    world.insert_resource(UtilityConfig::default());
    world.insert_resource(ColonyResources::default());
    world.insert_resource(scale::layer1::day_night::DayNightCycle::default());
    world.insert_resource(scale::layer1::taboo::TabooState::default());
    world.insert_resource(scale::layer1::zone::ZoneGrid::new(200, 200));

    if let Some(ctx) = gpu_ctx {
        world.insert_resource(ctx);
    }

    // Spawn buildings spread across the map, cycling through types
    for i in 0..n_buildings {
        let x = (i * 11 % 200) as i32;
        let y = (i * 17 % 200) as i32;
        match i % 4 {
            0 => {
                world.spawn((
                    GridPosition { x, y },
                    Farm {
                        capacity: 4,
                        workers: vec![],
                        selected_crop: ItemType::Wheat,
                    },
                ));
            }
            1 => {
                world.spawn((
                    GridPosition { x, y },
                    Housing {
                        capacity: 6,
                        residents: vec![],
                    },
                ));
            }
            2 => {
                world.spawn((
                    GridPosition { x, y },
                    Tavern {
                        capacity: 8,
                        visitors: vec![],
                    },
                ));
            }
            _ => {
                world.spawn((
                    GridPosition { x, y },
                    Designation {
                        designation_type: DesignationType::Mine,
                    },
                ));
            }
        }
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
    let gpu_result = std::panic::catch_unwind(|| pollster::block_on(GpuContext::new()));
    let gpu_available = matches!(&gpu_result, Ok(Ok(_)));

    let mut group = c.benchmark_group("utility_ai_evaluate");
    // Reduce sample size for the huge benchmarks
    group.sample_size(10);

    // (pops, buildings) — scale buildings with pops for realism
    let scenarios: &[(usize, usize)] = &[
        (5, 8),
        (50, 8),
        (200, 8),
        (500, 20),
        (2_000, 50),
        (10_000, 200),
        (100_000, 500),
    ];

    for &(n_pops, n_buildings) in scenarios {
        let label = if n_pops >= 1000 {
            format!("{}k", n_pops / 1000)
        } else {
            format!("{n_pops}")
        };

        // CPU benchmark
        group.bench_function(format!("cpu_{label}_pops"), |b| {
            let mut world = make_bench_world(n_pops, n_buildings, None);
            b.iter(|| {
                evaluate_actions_system(black_box(&mut world));
            });
        });

        // GPU benchmark (only if GPU is available)
        if gpu_available {
            group.bench_function(format!("gpu_{label}_pops"), |b| {
                let ctx = pollster::block_on(GpuContext::new()).unwrap();
                let mut world = make_bench_world(n_pops, n_buildings, Some(ctx));
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

criterion_group!(
    benches,
    benchmark_pathfinding,
    benchmark_rendering,
    benchmark_rendering_buildings,
    benchmark_utility_ai
);
criterion_main!(benches);
