# Specification 874: The Blob

## 1. Overview
**Layer:** 1
**Feature:** The Blob
**Fantasy:** The slow, creeping doom. The "Grey Goo" scenario on a micro scale.
**Mechanic:** An indestructible, slow-growing entity (Slime/Crystal) that consumes adjacent tiles. Can only be "contained" (walls) or "fed" (sacrifices/garbage) to temporarily halt or direct its growth.
**Emergence:** You keep the Blob as a garbage disposal system. It grows too big and eats the garbage disposal room.
**Tension:** Destroying it is impossible; management is the only option.

## 2. Dependencies
- Layer 1 Terrain Map / Grid.
- Layer 1 Resource / Waste mechanics.
- Tick-based expansion mechanics.

## 3. RED Phase: Tests First

```rust
// tests/the_blob_tests.rs
use bevy::prelude::*;
use scale::layer1::map::{GridPosition, TerrainGrid, TerrainType};
use scale::layer1::blob::{BlobNode, BlobNetwork, blob_expansion_system};

#[test]
fn test_blob_expands_to_adjacent_empty_tile() {
    let mut app = App::new();
    let mut grid = TerrainGrid::new(10, 10);
    // Center is empty
    grid.set(5, 5, TerrainType::Dirt);
    grid.set(6, 5, TerrainType::Dirt);
    app.insert_resource(grid);

    app.add_systems(Update, blob_expansion_system);

    let network = app.world_mut().spawn(BlobNetwork { expansion_timer: 1.0, current_time: 1.0 }).id();

    // Spawn the seed blob
    app.world_mut().spawn((
        BlobNode { network_id: network },
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    // A new blob node should have spawned at (6, 5) or another adjacent tile
    let blob_count = app.world_mut().query::<&BlobNode>().iter(&app.world()).count();
    assert!(blob_count > 1, "Blob failed to expand");
}

#[test]
fn test_blob_halts_when_contained_by_walls() {
    let mut app = App::new();
    let mut grid = TerrainGrid::new(10, 10);
    // Surround the blob with walls
    grid.set(4, 5, TerrainType::Wall);
    grid.set(6, 5, TerrainType::Wall);
    grid.set(5, 4, TerrainType::Wall);
    grid.set(5, 6, TerrainType::Wall);
    app.insert_resource(grid);

    app.add_systems(Update, blob_expansion_system);

    let network = app.world_mut().spawn(BlobNetwork { expansion_timer: 1.0, current_time: 1.0 }).id();

    app.world_mut().spawn((
        BlobNode { network_id: network },
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    // No new blobs should spawn since it is boxed in
    let blob_count = app.world_mut().query::<&BlobNode>().iter(&app.world()).count();
    assert_eq!(blob_count, 1, "Blob expanded through walls");
}

#[test]
fn test_blob_consumes_waste_to_delay_expansion() {
    let mut app = App::new();
    app.insert_resource(TerrainGrid::new(10, 10));
    app.add_systems(Update, blob_expansion_system);

    // Give it a negative timer to signify it was just "fed"
    let network = app.world_mut().spawn(BlobNetwork { expansion_timer: 1.0, current_time: -5.0 }).id();

    app.world_mut().spawn((
        BlobNode { network_id: network },
        GridPosition { x: 5, y: 5 },
    ));

    app.update();

    // Because current_time < expansion_timer, it should not expand
    let blob_count = app.world_mut().query::<&BlobNode>().iter(&app.world()).count();
    assert_eq!(blob_count, 1, "Blob expanded despite being fed");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/blob.rs
use bevy::prelude::*;
use crate::layer1::map::{GridPosition, TerrainGrid, TerrainType};

#[derive(Component)]
pub struct BlobNetwork {
    pub expansion_timer: f32,
    pub current_time: f32,
}

#[derive(Component)]
pub struct BlobNode {
    pub network_id: Entity,
}

pub fn blob_expansion_system(
    mut commands: Commands,
    mut networks: Query<(Entity, &mut BlobNetwork)>,
    nodes: Query<(&BlobNode, &GridPosition)>,
    grid: Res<TerrainGrid>,
) {
    for (net_entity, mut network) in networks.iter_mut() {
        network.current_time += 1.0; // Simulate tick

        if network.current_time >= network.expansion_timer {
            network.current_time = 0.0;

            let mut expanded = false;

            for (node, pos) in nodes.iter() {
                if node.network_id != net_entity { continue; }

                // Check cardinal directions
                let adjacents = [
                    GridPosition { x: pos.x + 1, y: pos.y },
                    GridPosition { x: pos.x - 1, y: pos.y },
                    GridPosition { x: pos.x, y: pos.y + 1 },
                    GridPosition { x: pos.x, y: pos.y - 1 },
                ];

                for adj in adjacents {
                    // Stop if out of bounds or blocked by a Wall
                    if let Some(terrain) = grid.get(adj.x, adj.y) {
                        if terrain != TerrainType::Wall {
                            // Expand! (Minimal logic: just spawn one new node per network per tick)
                            commands.spawn((
                                BlobNode { network_id: net_entity },
                                adj,
                            ));
                            expanded = true;
                            break;
                        }
                    }
                }

                if expanded { break; }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Pathfinding**: Ensure `BlobNode` entities block Pop pathfinding.
- **Consumption System**: Write a system that allows Pops to drop `Waste` items onto a `BlobNode`, which decrements the `current_time` of the parent `BlobNetwork`, effectively stalling growth.
- **Visuals**: Give the Blob a pulsating color or shader effect that warns players when it is about to expand.
- **Destruction**: When the Blob expands onto a building, it should trigger a `BuildingDestroyed` event.

## 6. Acceptance Criteria
- [ ] Tests pass in RED phase.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes cleanly.
- [ ] Test coverage >= 85%.
- [ ] Blob expands to non-wall tiles over time.
- [ ] Blob halts expansion when blocked by `TerrainType::Wall`.
- [ ] Blob delays expansion when "fed" (current_time manipulated).

## 7. Technical Guidance
- The Blob should be treated as a spatial hazard. Managing its expansion timer requires linking the individual spatial nodes (`BlobNode`) back to a central manager (`BlobNetwork`).
- Avoid spawning a massive amount of nodes simultaneously to prevent FPS drops. Expanding 1-2 tiles per major simulation tick per network is sufficient.

## 8. Questions
*Builder: add questions here if spec is unclear.*
