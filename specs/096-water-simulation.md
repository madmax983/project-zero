# 096: Water Simulation

## Overview

Water is a fundamental resource for life. While we have `TerrainType::Water`, it currently serves only as a movement blocker. This specification introduces a `WaterGrid` that simulates the availability of water in the soil ("Hydration").
Hydration spreads from water sources (Rivers, Lakes, Wells) and is consumed by Farms (`008`) and Flora (`044`). High hydration promotes fertility (future spec `059`) and prevents fire (`033`).

This system uses a cellular automata approach similar to the `ErosionGrid` (`093`) or `Miasma` (`experimental`), but with a focus on "Saturation".

## Dependencies

- `002` — Terrain Grid (Base map)
- `093` — Desire Paths (Similar grid architecture)

## RED Phase: Tests First

Write these tests in `src/layer1/water_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::water::{WaterGrid, WaterSource, update_water_system, MAX_HYDRATION};
    use crate::layer1::building::{Building, BuildingType};

    fn setup_world(width: usize, height: usize) -> World {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; width * height];
        world.insert_resource(TerrainGrid { width, height, tiles });
        world.insert_resource(WaterGrid::new(width, height));
        world
    }

    #[test]
    fn test_water_grid_initialization() {
        let world = setup_world(10, 10);
        let grid = world.resource::<WaterGrid>();
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert!(grid.values.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_terrain_water_is_infinite_source() {
        let mut world = setup_world(10, 10);

        // Set (5,5) to Water terrain
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[55] = TerrainType::Water; // (5, 5)
        }

        world.run_system_once(update_water_system).unwrap();

        let grid = world.resource::<WaterGrid>();
        assert_eq!(grid.get(5, 5), MAX_HYDRATION, "Water terrain should be fully hydrated");
    }

    #[test]
    fn test_water_spreads_to_neighbors() {
        let mut world = setup_world(10, 10);

        // Set (5,5) to Water
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[55] = TerrainType::Water;
        }

        // Run once
        world.run_system_once(update_water_system).unwrap();

        let grid = world.resource::<WaterGrid>();
        // Neighbors should have water (slightly less than max)
        assert!(grid.get(5, 4) > 0, "North should get water");
        assert!(grid.get(5, 6) > 0, "South should get water");
        assert!(grid.get(4, 5) > 0, "West should get water");
        assert!(grid.get(6, 5) > 0, "East should get water");

        assert!(grid.get(5, 4) < MAX_HYDRATION, "Spread water should decay");
    }

    #[test]
    fn test_water_source_building() {
        let mut world = setup_world(10, 10);

        // Spawn a Well (WaterSource) at (5,5)
        world.spawn((
            Building { building_type: BuildingType::Stockpile }, // Placeholder for Well
            GridPosition { x: 5, y: 5 },
            WaterSource { range: 5, amount: MAX_HYDRATION },
        ));

        world.run_system_once(update_water_system).unwrap();

        let grid = world.resource::<WaterGrid>();
        assert_eq!(grid.get(5, 5), MAX_HYDRATION, "Source should hydrate its tile");
        assert!(grid.get(6, 5) > 0, "Source should spread water");
    }

    #[test]
    fn test_rock_blocks_water() {
        let mut world = setup_world(10, 10);

        // (5,5) Water, (6,5) Rock, (7,5) Grass
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[55] = TerrainType::Water;
            terrain.tiles[56] = TerrainType::Rock;
        }

        // Run multiple ticks to allow spread
        for _ in 0..5 {
            world.run_system_once(update_water_system).unwrap();
        }

        let grid = world.resource::<WaterGrid>();
        assert_eq!(grid.get(5, 5), MAX_HYDRATION);
        assert_eq!(grid.get(6, 5), 0, "Rock should not hold water");
        assert_eq!(grid.get(7, 5), 0, "Water should not pass through Rock");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `WaterGrid` (`src/layer1/water.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::map::GridPosition;

pub const MAX_HYDRATION: u8 = 100;
pub const HYDRATION_DECAY: u8 = 10; // Drop per tile distance

#[derive(Resource, Default)]
pub struct WaterGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<u8>,
}

impl WaterGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x >= self.width || y >= self.height { return 0; }
        self.values[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        if x >= self.width || y >= self.height { return; }
        self.values[y * self.width + x] = value;
    }
}

#[derive(Component)]
pub struct WaterSource {
    pub range: u32, // Not used in simple diffusion, but good for logic
    pub amount: u8,
}
```

### 2. Implement `update_water_system`

```rust
pub fn update_water_system(
    mut water: ResMut<WaterGrid>,
    terrain: Res<TerrainGrid>,
    sources: Query<(&GridPosition, &WaterSource)>,
) {
    let mut next_values = water.values.clone();
    let width = water.width;
    let height = water.height;

    // 1. Reset sources (Terrain::Water and Buildings)
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if terrain.tiles[idx] == TerrainType::Water {
                next_values[idx] = MAX_HYDRATION;
            } else if terrain.tiles[idx] == TerrainType::Rock {
                 next_values[idx] = 0; // Rock creates barrier
            }
        }
    }

    for (pos, source) in &sources {
        let idx = (pos.y as usize) * width + (pos.x as usize);
        if idx < next_values.len() {
            next_values[idx] = source.amount;
        }
    }

    // 2. Diffusion (Simple 4-way spread)
    // Run diffusion directly on next_values?
    // No, strictly we need double buffering or careful iteration order.
    // For MVP, one pass top-left to bottom-right, then bottom-right to top-left gives good enough propagation.

    // Pass 1: Top-Left -> Bottom-Right
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if terrain.tiles[idx] == TerrainType::Rock { continue; }

            let mut max_neighbor = 0;

            // Check Left
            if x > 0 { max_neighbor = max_neighbor.max(next_values[idx - 1]); }
            // Check Top
            if y > 0 { max_neighbor = max_neighbor.max(next_values[idx - width]); }

            let potential = max_neighbor.saturating_sub(HYDRATION_DECAY);
            if potential > next_values[idx] {
                next_values[idx] = potential;
            }
        }
    }

    // Pass 2: Bottom-Right -> Top-Left
    for y in (0..height).rev() {
        for x in (0..width).rev() {
            let idx = y * width + x;
            if terrain.tiles[idx] == TerrainType::Rock { continue; }

            let mut max_neighbor = 0;

            // Check Right
            if x < width - 1 { max_neighbor = max_neighbor.max(next_values[idx + 1]); }
            // Check Bottom
            if y < height - 1 { max_neighbor = max_neighbor.max(next_values[idx + width]); }

            let potential = max_neighbor.saturating_sub(HYDRATION_DECAY);
            if potential > next_values[idx] {
                next_values[idx] = potential;
            }
        }
    }

    water.values = next_values;
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: The double-pass algorithm is O(N) where N is tiles. This is efficient. Avoid full cellular automata (O(N*k)) unless necessary.
- **Evaporation**: Currently hydration only decays by distance. Consider adding `GlobalEvaporation` that reduces all values by 1 each tick, forcing constant replenishment.
- **Visuals**: Map `WaterGrid` values to `term::Color` (Blue tint) in the renderer (`src/ui/map.rs`).
- **Integration**: Update `src/layer1/farm.rs` to check hydration. If `hydration < 20`, growth is stalled.

## Acceptance Criteria

- [ ] `WaterGrid` resource exists and tracks hydration (0-100).
- [ ] `update_water_system` spreads water from `TerrainType::Water`.
- [ ] `WaterSource` component allows buildings (like Wells) to generate hydration.
- [ ] Rock tiles block hydration spread.
- [ ] Hydration decays over distance.
- [ ] All tests in `water_tests.rs` pass.

## Technical Guidance

- Use `usize` for grid indexing to avoid casting.
- Ensure `WaterGrid` is initialized in `main.rs` or the test setup.
- Add `WaterSource` to `BuildingType::Well` (create new building type if needed, or stick to generic `WaterSource` component for now).
