# 059: Soil Fertility and Depletion

## Overview

Adds depth to agriculture by introducing soil fertility. Farms are no longer infinite food factories; they now interact with the land. High fertility boosts production, but farming depletes it. Leaving land fallow allows it to recover. This creates a strategic choice between intensive farming (short-term yield) and sustainable farming (long-term stability).

## Dependencies

- `002` — Terrain Grid (provides `TerrainType` and dimensions)
- `008` — Farm Building (provides `produce_food_system`)
- `027` — Seasonal Rhythms (integrates with seasonal modifiers)
- `015` — Selection System (for inspecting fertility)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/fertility.rs - New module

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer2::farm::{Farm, ColonyResources, produce_food_system};
    use crate::layer2::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_fertility_grid_initialization() {
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        let fertility = FertilityGrid::from_terrain(&terrain);

        assert_eq!(fertility.width, 10);
        assert_eq!(fertility.height, 10);
        assert_eq!(fertility.get(0, 0), 1.0); // Grass = 100%
    }

    #[test]
    fn test_fertility_values_by_terrain() {
        let mut tiles = vec![TerrainType::Grass; 4];
        tiles[0] = TerrainType::Grass;
        tiles[1] = TerrainType::Dirt;
        tiles[2] = TerrainType::Rock;
        tiles[3] = TerrainType::Water;

        let terrain = TerrainGrid { width: 2, height: 2, tiles };
        let fertility = FertilityGrid::from_terrain(&terrain);

        assert!((fertility.get(0, 0) - 1.0).abs() < f32::EPSILON); // Grass
        assert!((fertility.get(1, 0) - 0.8).abs() < f32::EPSILON); // Dirt
        assert_eq!(fertility.get(0, 1), 0.0); // Rock
        assert_eq!(fertility.get(1, 1), 0.0); // Water
    }

    #[test]
    fn test_production_scales_with_fertility() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());

        // Setup fertility grid with 50% fertility at (0,0)
        let mut fertility = FertilityGrid::new(10, 10);
        fertility.set(0, 0, 0.5);
        world.insert_resource(fertility);

        // Spawn farm at (0,0) with 1 worker
        let worker = world.spawn_empty().id();
        world.spawn((
            Farm { workers: vec![worker], ..Default::default() },
            GridPosition { x: 0, y: 0 },
        ));

        // Run production
        produce_food_system(&mut world);

        // Expected: Base (0.3) * Fertility (0.5) = 0.15
        let food = world.resource::<ColonyResources>().food;
        assert!((food - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_farming_depletes_fertility() {
        let mut world = World::new();
        let mut fertility = FertilityGrid::new(10, 10);
        fertility.set(0, 0, 1.0);
        world.insert_resource(fertility);

        // Spawn active farm
        let worker = world.spawn_empty().id();
        world.spawn((
            Farm { workers: vec![worker], ..Default::default() },
            GridPosition { x: 0, y: 0 },
        ));

        update_fertility_system(&mut world);

        let new_fertility = world.resource::<FertilityGrid>().get(0, 0);
        assert!(new_fertility < 1.0);
        assert!(new_fertility > 0.9); // Shouldn't deplete instantly
    }

    #[test]
    fn test_fallow_land_regenerates() {
        let mut world = World::new();
        let mut fertility = FertilityGrid::new(10, 10);
        fertility.set(0, 0, 0.5); // Depleted
        world.insert_resource(fertility);

        // No farm or empty farm at (0,0)

        update_fertility_system(&mut world);

        let new_fertility = world.resource::<FertilityGrid>().get(0, 0);
        assert!(new_fertility > 0.5);
    }

    #[test]
    fn test_fertility_clamping() {
        let mut grid = FertilityGrid::new(1, 1);

        grid.set(0, 0, 1.5);
        assert_eq!(grid.get(0, 0), 1.0); // Max 1.0

        grid.set(0, 0, -0.5);
        assert_eq!(grid.get(0, 0), 0.0); // Min 0.0
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Fertility Grid Resource (`src/layer1/fertility.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::terrain::{TerrainGrid, TerrainType};

#[derive(Resource)]
pub struct FertilityGrid {
    pub width: usize,
    pub height: usize,
    values: Vec<f32>,
}

impl FertilityGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![1.0; width * height],
        }
    }

    pub fn from_terrain(terrain: &TerrainGrid) -> Self {
        let values = terrain.tiles.iter().map(|t| match t {
            TerrainType::Grass => 1.0,
            TerrainType::Dirt => 0.8,
            TerrainType::Tree => 1.0, // Forest soil is good
            _ => 0.0,
        }).collect();

        Self {
            width: terrain.width,
            height: terrain.height,
            values,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            self.values[y * self.width + x]
        } else {
            0.0
        }
    }

    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x < self.width && y < self.height {
            self.values[y * self.width + x] = value.clamp(0.0, 1.0);
        }
    }

    pub fn modify(&mut self, x: usize, y: usize, delta: f32) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.values[idx] = (self.values[idx] + delta).clamp(0.0, 1.0);
        }
    }
}
```

### 2. Update Production Logic (`src/layer2/farm.rs`)

Modify `produce_food_system` to read from `FertilityGrid`.

```rust
// ... imports
use crate::layer1::fertility::FertilityGrid;
use crate::layer1::map::GridPosition;

pub fn produce_food_system(world: &mut World) {
    // ... season logic ...

    // We can't query &mut ColonyResources and &FertilityGrid comfortably in the same scope
    // if we iterate and mutate.
    // Instead, calculate total first.

    let total_production: f32 = {
        let fertility_grid = world.resource::<FertilityGrid>();
        let mut query = world.query::<(&Farm, &GridPosition)>();

        query.iter(world)
            .map(|(farm, pos)| {
                let workers = farm.workers.iter()
                    .filter(|&&e| world.get_entity(e).is_ok())
                    .count() as f32;

                let fertility = fertility_grid.get(pos.x as usize, pos.y as usize);

                workers * FOOD_PER_WORKER_PER_TICK * fertility
            })
            .sum()
    };

    // ... apply season modifier ...
    // ... add to resources ...
}
```

### 3. Depletion & Regeneration System (`src/layer1/fertility.rs`)

```rust
const DEPLETION_RATE: f32 = 0.002; // -0.2% per tick per active farm
const REGEN_RATE: f32 = 0.001;     // +0.1% per tick when fallow

pub fn update_fertility_system(world: &mut World) {
    // 1. Identify farmed tiles
    let mut farmed_tiles = std::collections::HashSet::new();
    let mut query = world.query::<(&Farm, &GridPosition)>();
    for (farm, pos) in query.iter(world) {
        if !farm.workers.is_empty() {
            farmed_tiles.insert((pos.x as usize, pos.y as usize));
        }
    }

    // 2. Update grid
    let mut grid = world.resource_mut::<FertilityGrid>();
    let width = grid.width;
    let height = grid.height;

    for y in 0..height {
        for x in 0..width {
            if farmed_tiles.contains(&(x, y)) {
                grid.modify(x, y, -DEPLETION_RATE);
            } else {
                // Only regenerate if it's soil (base > 0)
                // We might need a separate "Max Fertility" grid or just check > 0
                // For simplicity: if current > 0, it can regen up to 1.0
                if grid.get(x, y) > 0.0 {
                    grid.modify(x, y, REGEN_RATE);
                }
            }
        }
    }
}
```

### 4. UI Update (`src/shared/selection.rs`)

Update `inspect_tile` to show fertility.

```rust
pub fn inspect_tile(world: &World, x: i32, y: i32) -> String {
    // ... existing logic ...

    let mut info = format!("Tile ({}, {})\n\nTerrain: {}", x, y, tile.as_str());

    if let Some(fertility_grid) = world.get_resource::<FertilityGrid>() {
        let f = fertility_grid.get(x as usize, y as usize);
        if f > 0.0 {
            info.push_str(&format!("\nFertility: {:.0}%", f * 100.0));
        }
    }

    info
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: `update_fertility_system` iterates the whole grid (max 1M tiles) every tick.
  - *Optimization*: Only update active farms every tick? Use a "dirty" list?
  - *Mitigation*: 80x50 = 4000 tiles. Iteration is fast. If map grows, move to chunk-based or probabilistic updates (random tick).
- **Regeneration Logic**: Currently regenerates ANY tile > 0. Should strictly only regenerate natural soil types.
  - *Refactor*: Store `max_fertility` in `FertilityGrid` or look up `TerrainType` during regen (slower).
- **Balance**: Adjust rates. 0.002/tick = 1% drop every 5 ticks. Too fast!
  - *Correction*: `DEPLETION_RATE = 0.0001` (1% every 100 ticks).
  - *Correction*: `REGEN_RATE = 0.00005`.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] `FertilityGrid` initialized from terrain.
- [ ] Farming reduces fertility over time.
- [ ] Fallow land recovers fertility.
- [ ] Food production scales with fertility.
- [ ] Inspection tool shows Fertility percentage.
- [ ] `update_fertility_system` added to schedule.

## Technical Guidance

- **Map Generation**: Initialize `FertilityGrid` immediately after `TerrainGrid` in startup.
- **System Ordering**: Run `update_fertility_system` after production (doesn't strictly matter, but logical).
- **Visuals**: Since we don't have a heatmap view yet, inspection is critical for verifying the feature works.

## Questions

- Should fertilizer items exist? (Future feature)
- Should crop rotation (different crops) matter? (Future feature)

*Architect:* Fertilizer items are deferred to a future feature specification.
