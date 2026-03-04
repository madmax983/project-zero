# 161: Ecological Succession

## Overview

Introduce dynamic vegetation growth and biome recovery mechanics. Terrain is no longer static; nature reclaims empty tiles over time. This adds strategic depth to deforestation and forestry, as clear-cutting an area without replanting or management will result in a chaotic mix of pioneer species before true forest returns.

## Dependencies

- `002` — Terrain Grid (for `TerrainType`)
- `019` — Forestry System (for `TerrainType::Tree`)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/ecology_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainType, TerrainGrid};
    use crate::layer1::ecology::{EcologyConfig, process_ecological_succession};

    #[test]
    fn test_terrain_type_shrub() {
        assert_eq!(TerrainType::Shrub.name(), "Shrub");
        assert!(TerrainType::Shrub.is_walkable());
    }

    #[test]
    fn test_terrain_type_sapling() {
        assert_eq!(TerrainType::Sapling.name(), "Sapling");
        assert!(TerrainType::Sapling.is_walkable());
    }

    #[test]
    fn test_dirt_to_grass_pioneer() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Dirt; 9];
        // Center is dirt
        world.insert_resource(TerrainGrid { width: 3, height: 3, tiles });
        world.insert_resource(EcologyConfig::default()); // Should enable fast growth for test

        // Run succession
        process_ecological_succession(&mut world);

        let grid = world.resource::<TerrainGrid>();
        // Center should eventually become Grass (pioneer species)
        // Note: This is stochastic, so test might need deterministic RNG or high probability config
        assert!(grid.tiles.contains(&TerrainType::Grass));
    }

    #[test]
    fn test_grass_to_sapling_seeding() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 9];
        tiles[0] = TerrainType::Tree; // (0,0) is a seed source
        world.insert_resource(TerrainGrid { width: 3, height: 3, tiles });
        world.insert_resource(EcologyConfig::default());

        // Run succession
        process_ecological_succession(&mut world);

        let grid = world.resource::<TerrainGrid>();
        // Neighbors of tree should have chance to become Sapling
        // Specifically check (0,1) or (1,0) or (1,1)
        let sapling_count = grid.tiles.iter().filter(|&&t| t == TerrainType::Sapling).count();
        assert!(sapling_count > 0);
    }

    #[test]
    fn test_sapling_growth_to_tree() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Sapling; 9];
        world.insert_resource(TerrainGrid { width: 3, height: 3, tiles });
        world.insert_resource(EcologyConfig::default());

        // Run succession multiple times to simulate time
        for _ in 0..10 {
            process_ecological_succession(&mut world);
        }

        let grid = world.resource::<TerrainGrid>();
        let tree_count = grid.tiles.iter().filter(|&&t| t == TerrainType::Tree).count();
        assert!(tree_count > 0);
    }

    #[test]
    fn test_rock_does_not_grow() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Rock; 9];
        world.insert_resource(TerrainGrid { width: 3, height: 3, tiles });
        world.insert_resource(EcologyConfig::default());

        process_ecological_succession(&mut world);

        let grid = world.resource::<TerrainGrid>();
        assert!(grid.tiles.iter().all(|&t| t == TerrainType::Rock));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `TerrainType` Enum

```rust
// src/layer1/terrain.rs
pub enum TerrainType {
    // ... existing
    Shrub,   // Pioneer vegetation
    Sapling, // Young tree
}

// Implement name(), movement_cost() for new types.
// Shrub cost: 1.2
// Sapling cost: 1.1
```

### 2. Implement `EcologyConfig` Resource

```rust
// src/layer1/ecology.rs

#[derive(Resource)]
pub struct EcologyConfig {
    pub growth_rate: f32, // Chance per tick for update
    pub pioneer_chance: f32, // Chance Dirt -> Grass
    pub seed_spread_chance: f32, // Chance Grass -> Sapling near Tree
    pub maturation_chance: f32, // Chance Sapling -> Tree
}

impl Default for EcologyConfig {
    fn default() -> Self {
        Self {
            growth_rate: 0.05, // 5% of tiles checked per tick? Adjust for performance.
            pioneer_chance: 0.1,
            seed_spread_chance: 0.05,
            maturation_chance: 0.02,
        }
    }
}
```

### 3. Implement `process_ecological_succession` System

```rust
// src/layer1/ecology.rs

pub fn process_ecological_succession(world: &mut World) {
    let config = world.resource::<EcologyConfig>();
    // Need random access to grid, so we might need to extract Grid first
    // or use a command queue if we want parallel access.
    // For MVP, single-threaded stochastic update is fine.

    let mut grid = world.resource_mut::<TerrainGrid>();
    let width = grid.width;
    let height = grid.height;
    let mut rng = rand::thread_rng();

    // Iterate a subset of random tiles or all tiles with low probability
    // Better: Pick N random tiles to update to keep fixed performance cost.
    let tiles_to_update = (width * height) / 100; // 1% per tick

    for _ in 0..tiles_to_update {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);

        let current = grid.get(x, y).unwrap();
        let new_type = match current {
            TerrainType::Dirt => {
                if rng.gen_bool(config.pioneer_chance) {
                    Some(TerrainType::Grass)
                } else { None }
            },
            TerrainType::Grass => {
                // Check neighbors for seeds (Trees)
                let has_seed_source = check_neighbors_for_tree(&grid, x, y);
                if has_seed_source && rng.gen_bool(config.seed_spread_chance) {
                    Some(TerrainType::Sapling)
                } else if rng.gen_bool(config.pioneer_chance * 0.5) {
                    // Occasional random shrub
                    Some(TerrainType::Shrub)
                } else { None }
            },
            TerrainType::Sapling => {
                if rng.gen_bool(config.maturation_chance) {
                    Some(TerrainType::Tree)
                } else { None }
            },
            _ => None,
        };

        if let Some(t) = new_type {
            grid.set(x, y, t);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: The stochastic update loop is O(1) relative to map size if we cap `tiles_to_update`. Verify this doesn't cause stutter on large maps.
- **Biomes**: Different biomes (Desert, Tundra) should have different succession rules (e.g., Cactus instead of Tree). Future scope.
- **Root Systems**: Instead of just neighbor checks, we could have a `Fertility` layer (`059`) that influences growth speed.
- **Invasive Species**: Add logic for fast-spreading "Weeds" that choke out saplings.

## Acceptance Criteria

- [ ] `TerrainType::Shrub` and `TerrainType::Sapling` are implemented.
- [ ] `process_ecological_succession` system runs periodically.
- [ ] Dirt tiles slowly convert to Grass.
- [ ] Grass tiles near Trees convert to Saplings.
- [ ] Saplings mature into Trees.
- [ ] Rock and Water tiles remain unchanged.
- [ ] Tests pass with adequate coverage.

## Technical Guidance

- Register the system in `SimulationSchedule`.
- Ensure `EcologyConfig` is initialized in `setup_world`.
- Use `rand::Rng` for stochastic behavior.
- Be careful with `grid.get` vs `grid.set` ownership if iterating. The random sampling approach avoids holding a reference to the whole grid while iterating.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
