# 033: Fire Propagation

## Overview

Introduce fire mechanics. Fire can start (initially by debug/event, later by lightning or accidents), spread to flammable terrain (Trees) and buildings, and destroy them. This adds an environmental hazard and encourages strategic building placement (e.g., firebreaks).

## Dependencies

- `002` — Terrain Grid (for `TerrainType`)
- `006` — Building Placement (for `Building` entities)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/fire_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::fire::{Fire, Flammable, fire_spread_system, fire_damage_system};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use std::collections::HashSet;

    #[test]
    fn test_flammable_component_defaults() {
        let f = Flammable::default();
        assert!(f.fuel > 0.0);
        assert!(f.fire_resistance >= 0.0);
    }

    #[test]
    fn test_fire_component_defaults() {
        let f = Fire::default();
        assert!(f.lifetime > 0);
        assert!(f.intensity > 0.0);
    }

    #[test]
    fn test_fire_spreads_to_tree() {
        let mut world = World::new();
        // 10x10 Grid
        let mut tiles = vec![TerrainType::Grass; 100];
        // (5,5) has Fire
        // (5,6) is Tree
        tiles[65] = TerrainType::Tree;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Spawn Fire at (5,5)
        world.spawn((
            Fire { intensity: 1.0, ..Default::default() },
            GridPosition { x: 5, y: 5 }
        ));

        // Run system enough times to guarantee spread (since it's probabilistic, we might need to force it or mock RNG in impl, or just run many times)
        // For test stability, we usually mock RNG or set probability to 1.0 in a config resource.
        // Assuming we can control it or it happens eventually:
        for _ in 0..10 {
            fire_spread_system(&mut world);
        }

        // Check if fire exists at (5,6)
        let has_fire = world.query::<(&GridPosition, &Fire)>().iter(&world)
            .any(|(pos, _)| pos.x == 5 && pos.y == 6);

        assert!(has_fire, "Fire should spread to adjacent tree");
    }

    #[test]
    fn test_fire_spreads_to_flammable_building() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Grass; 100] });

        // Fire at (5,5)
        world.spawn((
            Fire { intensity: 1.0, ..Default::default() },
            GridPosition { x: 5, y: 5 }
        ));

        // Housing at (5,6) - Flammable
        world.spawn((
            Building { building_type: BuildingType::Housing },
            Flammable::default(),
            GridPosition { x: 5, y: 6 }
        ));

        for _ in 0..10 {
            fire_spread_system(&mut world);
        }

        let has_fire = world.query::<(&GridPosition, &Fire)>().iter(&world)
            .any(|(pos, _)| pos.x == 5 && pos.y == 6);

        assert!(has_fire, "Fire should spread to adjacent flammable building");
    }

    #[test]
    fn test_fire_does_not_spread_to_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[65] = TerrainType::Rock; // (5,6)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        world.spawn((
            Fire { intensity: 1.0, ..Default::default() },
            GridPosition { x: 5, y: 5 }
        ));

        for _ in 0..10 {
            fire_spread_system(&mut world);
        }

        let has_fire = world.query::<(&GridPosition, &Fire)>().iter(&world)
            .any(|(pos, _)| pos.x == 5 && pos.y == 6);

        assert!(!has_fire, "Fire should not spread to rock");
    }

    #[test]
    fn test_fire_burns_out_and_destroys_tree() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5,5)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Fire at (5,5) with short lifetime
        let fire_entity = world.spawn((
            Fire { lifetime: 1, ..Default::default() },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Run damage system (decrements lifetime)
        fire_damage_system(&mut world);

        // Fire should be gone
        assert!(world.get_entity(fire_entity).is_err());

        // Terrain should be Dirt (burnt tree)
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components

```rust
// src/layer1/fire.rs

use bevy_ecs::prelude::*;
use crate::layer1::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::building::Building;

#[derive(Component, Debug, Clone, Copy)]
pub struct Fire {
    pub lifetime: u32,
    pub intensity: f32,
}

impl Default for Fire {
    fn default() -> Self {
        Self {
            lifetime: 50, // ticks
            intensity: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Flammable {
    pub fuel: f32,
    pub fire_resistance: f32,
}

impl Default for Flammable {
    fn default() -> Self {
        Self {
            fuel: 100.0,
            fire_resistance: 0.0,
        }
    }
}
```

### 2. Implement Systems

```rust
// src/layer1/fire.rs

use rand::Rng;
use std::collections::HashSet;

pub fn fire_spread_system(world: &mut World) {
    let mut new_fires = Vec::new();
    let mut fire_locations = HashSet::new();

    // 1. Collect all current fire locations to avoid re-igniting or double-checking
    let mut fire_query = world.query::<(&GridPosition, &Fire)>();
    for (pos, _) in fire_query.iter(world) {
        fire_locations.insert(*pos);
    }

    // 2. Check neighbors for potential spread
    let terrain = world.resource::<TerrainGrid>();
    let mut rng = rand::thread_rng();

    // Note: We iterate fires again. In a real ECS we might cache this list.
    // Query must be read-only here if we are pushing to a vec.
    let fires: Vec<GridPosition> = fire_locations.iter().cloned().collect();

    for pos in fires {
        let neighbors = [
            (pos.x + 1, pos.y), (pos.x - 1, pos.y),
            (pos.x, pos.y + 1), (pos.x, pos.y - 1)
        ];

        for (nx, ny) in neighbors {
            let n_pos = GridPosition { x: nx, y: ny };

            // Skip if already burning
            if fire_locations.contains(&n_pos) {
                continue;
            }
            // Skip if out of bounds
            if nx < 0 || ny < 0 || nx >= terrain.width as i32 || ny >= terrain.height as i32 {
                continue;
            }

            let mut should_ignite = false;

            // Check Terrain
            if let Some(tile) = terrain.get(nx as usize, ny as usize) {
                if tile == TerrainType::Tree {
                    // Tree flammability chance
                    if rng.gen_bool(0.1) { // 10% chance per tick per neighbor
                        should_ignite = true;
                    }
                }
            }

            // Check Buildings (Flammable)
            // This is slow (O(N) scan). Optimization: Use OccupiedTiles map or similar spatial index if available.
            // For MVP: Simple query scan is acceptable if building count < 1000.
            // Better: We know buildings are entities. We can query (GridPosition, Flammable).
            if !should_ignite {
                let mut building_query = world.query::<(&GridPosition, &Flammable)>();
                for (b_pos, _) in building_query.iter(world) {
                    if *b_pos == n_pos {
                        if rng.gen_bool(0.1) {
                            should_ignite = true;
                        }
                        break;
                    }
                }
            }

            if should_ignite {
                // Check if we already queued a fire for this spot (avoid dupes in same frame)
                if !new_fires.contains(&n_pos) {
                    new_fires.push(n_pos);
                }
            }
        }
    }

    // 3. Spawn new fires
    for pos in new_fires {
        world.spawn((
            Fire::default(),
            pos,
        ));
    }
}

pub fn fire_damage_system(world: &mut World) {
    // Decrement lifetime, destroy burnt objects
    let mut fires_to_remove = Vec::new();
    let mut terrain_changes = Vec::new(); // (x, y, NewType)

    let mut query = world.query::<(Entity, &GridPosition, &mut Fire)>();
    for (entity, pos, mut fire) in query.iter_mut(world) {
        if fire.lifetime > 0 {
            fire.lifetime -= 1;
        }

        if fire.lifetime == 0 {
            fires_to_remove.push((entity, *pos));
        }
    }

    // Apply destruction for burnt-out fires
    let terrain = world.resource::<TerrainGrid>();
    for (entity, pos) in fires_to_remove {
        world.despawn(entity);

        // If it was on a tree, turn to dirt
        if let Some(tile) = terrain.get(pos.x as usize, pos.y as usize) {
            if tile == TerrainType::Tree {
                // Queue terrain change (cannot mutate resource while iterating query if we hold ref?
                // We dropped ref before this loop. But we need mutable access now.)
                terrain_changes.push((pos.x, pos.y, TerrainType::Dirt));
            }
        }

        // If it was on a building, destroy building?
        // Logic: Fire burns fuel. If fuel gone, destroy.
        // For MVP: Simplification -> If fire burns out naturally, it means it consumed the fuel.
        // Destroy flammable buildings at this pos.
        // (Builder needs to implement this query and despawn)
    }

    // Apply terrain changes
    if !terrain_changes.is_empty() {
        let mut terrain_mut = world.resource_mut::<TerrainGrid>();
        for (x, y, new_type) in terrain_changes {
             let idx = (y as usize) * terrain_mut.width + (x as usize);
             if idx < terrain_mut.tiles.len() {
                 terrain_mut.tiles[idx] = new_type;
             }
        }
    }
}
```

### 3. Integrate with Buildings

In `src/layer1/building.rs`, when spawning `Housing` (or other wooden structures), add the `Flammable` component.

## REFACTOR Phase: Quality & Design

- **Optimization**: The building query in `fire_spread_system` is O(N_buildings) inside O(N_fires). This is acceptable for MVP but bad for scale.
    - *Improvement*: Use a spatial map (like `OccupiedTiles`) to quickly check if a tile has a building, then get that building entity. Currently `OccupiedTiles` is just a Set of positions.
- **Fire Visuals**: Render fire as a red 'w' or '^' on top of the tile.
- **Extinguishing**: Pops with `Firefighting` job (future).

## Acceptance Criteria

- [ ] `Fire` and `Flammable` components defined.
- [ ] Fire spreads to adjacent Trees.
- [ ] Fire spreads to adjacent Flammable buildings.
- [ ] Fire does not spread to Rock/Water.
- [ ] Fire lifetime decreases.
- [ ] Burnt-out fires turn Trees to Dirt.
- [ ] Tests pass.

## Technical Guidance

- Ensure `fire_spread_system` and `fire_damage_system` are registered.
- Use `rand::gen_bool` for probability.
- Be careful with double-borrows when querying inside loops. Collect results to vectors/sets then apply changes.

## Questions

*Builder: add questions here if spec is unclear.*
