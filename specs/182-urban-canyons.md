# 182: Urban Canyons

## Overview

Introduces a dynamic Wind system where buildings and terrain shape airflow. Tall structures block wind, creating "wind shadows" (lee), while parallel structures channel wind, creating high-velocity "canyons". Wind speed and direction affect Pop movement speed (headwind slows, tailwind speeds up) and will later influence pollution diffusion (`063`) and temperature (`140`).

## Dependencies

- `002` — Terrain Grid (for map dimensions and natural obstacles)
- `004` — Building System (for artificial obstacles)
- `063` — Atmospheric Simulation (future integration target)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/wind_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use glam::Vec2;
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::wind::{WindGrid, GlobalWind, update_wind_system, calculate_wind_movement_penalty};

    #[test]
    fn test_wind_grid_initialization() {
        let grid = WindGrid::new(10, 10);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.get_wind(0, 0), Vec2::ZERO);
    }

    #[test]
    fn test_global_wind_propagation() {
        let mut world = World::new();
        let width = 5;
        let height = 5;

        world.insert_resource(WindGrid::new(width, height));
        world.insert_resource(GlobalWind { direction: Vec2::new(1.0, 0.0), speed: 1.0 }); // East wind
        world.insert_resource(TerrainGrid::new(width, height)); // Default flat

        // Run system
        update_wind_system(&mut world);

        let grid = world.resource::<WindGrid>();
        // Center tile should match global wind in open terrain
        let wind = grid.get_wind(2, 2);
        assert!((wind.x - 1.0).abs() < 0.01);
        assert!((wind.y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_wind_blocked_by_wall() {
        let mut world = World::new();
        let width = 5;
        let height = 5;
        world.insert_resource(WindGrid::new(width, height));
        world.insert_resource(GlobalWind { direction: Vec2::new(1.0, 0.0), speed: 1.0 }); // East wind

        // Spawn Wall at (1, 2)
        world.spawn((
            Building { building_type: BuildingType::Wall, ..Default::default() },
            GridPosition { x: 1, y: 2 },
        ));

        // Setup terrain (needed for checking blocks)
        world.insert_resource(TerrainGrid::new(width, height));

        // Run system
        update_wind_system(&mut world);

        let grid = world.resource::<WindGrid>();

        // Tile (2, 2) is directly downwind (East) of the wall at (1, 2).
        // It should be in the "Wind Shadow".
        let wind_shadow = grid.get_wind(2, 2);
        assert!(wind_shadow.length() < 0.5, "Wind should be reduced in lee of wall");

        // Tile (1, 2) itself (the wall) should have near zero wind?
        // Or simulation treats inside wall as irrelevant.
    }

    #[test]
    fn test_urban_canyon_effect() {
        let mut world = World::new();
        let width = 5;
        let height = 5;
        world.insert_resource(WindGrid::new(width, height));
        world.insert_resource(GlobalWind { direction: Vec2::new(1.0, 0.0), speed: 1.0 }); // East wind
        world.insert_resource(TerrainGrid::new(width, height));

        // Create Canyon: Walls at y=1 and y=3. Wind flows along y=2.
        // Wall at (2, 1)
        world.spawn((
            Building { building_type: BuildingType::Wall, ..Default::default() },
            GridPosition { x: 2, y: 1 },
        ));
        // Wall at (2, 3)
        world.spawn((
            Building { building_type: BuildingType::Wall, ..Default::default() },
            GridPosition { x: 2, y: 3 },
        ));

        // Run system
        update_wind_system(&mut world);

        let grid = world.resource::<WindGrid>();

        // Tile (2, 2) is in the canyon. Wind should be accelerated.
        let canyon_wind = grid.get_wind(2, 2);
        assert!(canyon_wind.x > 1.1, "Wind should accelerate in canyon (current: {})", canyon_wind.x);
    }

    #[test]
    fn test_movement_penalty() {
        // Tailwind
        let wind = Vec2::new(10.0, 0.0);
        let move_dir = Vec2::new(1.0, 0.0);
        let cost_mod = calculate_wind_movement_penalty(wind, move_dir);
        assert!(cost_mod < 1.0, "Tailwind should reduce movement cost");

        // Headwind
        let move_dir = Vec2::new(-1.0, 0.0); // Moving West into East wind
        let cost_mod = calculate_wind_movement_penalty(wind, move_dir);
        assert!(cost_mod > 1.0, "Headwind should increase movement cost");

        // Crosswind
        let move_dir = Vec2::new(0.0, 1.0); // Moving North
        let cost_mod = calculate_wind_movement_penalty(wind, move_dir);
        assert!((cost_mod - 1.0).abs() < 0.1, "Pure crosswind should have minimal effect");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Resources (`src/layer1/wind.rs`)

```rust
use bevy_ecs::prelude::*;
use glam::Vec2;

#[derive(Resource, Default)]
pub struct GlobalWind {
    pub direction: Vec2, // Normalized
    pub speed: f32,
}

#[derive(Resource)]
pub struct WindGrid {
    pub width: usize,
    pub height: usize,
    pub vectors: Vec<Vec2>,
}

impl WindGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vectors: vec![Vec2::ZERO; width * height],
        }
    }

    pub fn get_wind(&self, x: i32, y: i32) -> Vec2 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Vec2::ZERO;
        }
        self.vectors[y as usize * self.width + x as usize]
    }

    pub fn set_wind(&mut self, x: i32, y: i32, wind: Vec2) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.vectors[y as usize * self.width + x as usize] = wind;
        }
    }
}
```

### 2. Update System

```rust
use crate::layer1::map::GridPosition;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use std::collections::HashSet;

pub fn update_wind_system(world: &mut World) {
    let global = world.resource::<GlobalWind>();
    let base_wind = global.direction.normalize_or_zero() * global.speed;

    // Identify blockers (Walls, Rocks)
    let mut blockers = HashSet::new();

    // Terrain blockers
    let terrain = world.resource::<TerrainGrid>();
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            if matches!(terrain.get(x, y), Some(TerrainType::Rock)) {
                blockers.insert((x as i32, y as i32));
            }
        }
    }

    // Building blockers
    let mut query = world.query::<(&Building, &GridPosition)>();
    for (b, pos) in query.iter(world) {
        if b.building_type.blocks_wind() { // Add helper to BuildingType
            blockers.insert((pos.x, pos.y));
        }
    }

    let mut wind_grid = world.resource_mut::<WindGrid>();
    let width = wind_grid.width as i32;
    let height = wind_grid.height as i32;

    // Simple static analysis per tile
    for y in 0..height {
        for x in 0..width {
            if blockers.contains(&(x, y)) {
                wind_grid.set_wind(x, y, Vec2::ZERO);
                continue;
            }

            let mut local_wind = base_wind;

            // 1. Lee/Shadow Check
            // Check immediate upwind neighbor
            let upwind_offset = (-base_wind.normalize_or_zero()).round().as_ivec2();
            let upwind_pos = (x + upwind_offset.x, y + upwind_offset.y);

            if blockers.contains(&upwind_pos) {
                local_wind *= 0.2; // Shadow penalty
            } else {
                // 2. Canyon Check
                // Check neighbors perpendicular to wind
                let perp = Vec2::new(-base_wind.y, base_wind.x).normalize_or_zero().round().as_ivec2();
                let side1 = (x + perp.x, y + perp.y);
                let side2 = (x - perp.x, y - perp.y);

                if blockers.contains(&side1) && blockers.contains(&side2) {
                    local_wind *= 1.5; // Canyon boost
                }
            }

            wind_grid.set_wind(x, y, local_wind);
        }
    }
}
```

### 3. Movement Penalty Logic

```rust
pub fn calculate_wind_movement_penalty(wind: Vec2, move_dir: Vec2) -> f32 {
    let speed = wind.length();
    if speed < 0.1 { return 1.0; }

    let wind_dir = wind / speed;
    let dot = wind_dir.dot(move_dir.normalize_or_zero());

    // Dot = 1.0 (Tailwind) -> Cost 0.8
    // Dot = -1.0 (Headwind) -> Cost 1.5
    // Linear interpolation

    if dot > 0.0 {
        1.0 - (dot * 0.2 * speed.min(2.0)) // Max 40% reduction
    } else {
        1.0 + (dot.abs() * 0.5 * speed.min(2.0)) // Max 100% increase
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: The blocking check builds a `HashSet` every frame. Optimizations:
    - Cache the `blockers` map and only update when buildings/terrain change.
    - Use `BitSet` or similar grid-aligned structure.
- **Fluid Sim**: The static analysis is very basic. Consider a simple cellular automata flow (like `AtmosphereGrid`) for more organic swirls, but keep it deterministic.
- **Integration**:
    - Pass `WindGrid` to `AtmosphereGrid::diffuse()` to bias pollution spread.
    - Visualize wind vectors in debug mode.
- **Diagonal Flow**: The `round().as_ivec2()` logic snaps to cardinal/ordinal directions. Ensure it handles diagonal wind gracefully.

## Acceptance Criteria

- [ ] `WindGrid` resource exists and stores `Vec2` per tile.
- [ ] Walls and Rocks create "Wind Shadows" (reduced wind downwind).
- [ ] Parallel walls create "Urban Canyons" (increased wind).
- [ ] Pop movement cost is affected by wind direction relative to movement.
- [ ] `GlobalWind` resource controls the base wind.

## Technical Guidance

- Add `blocks_wind()` method to `BuildingType`. Default to true for Walls, Machines, large structures. False for Floors, stockpiles.
- Ensure `GlobalWind` is initialized in `setup_world`.
- `update_wind_system` should run in `SimulationSchedule` before `movement_system`.

## Questions

- Should wind affect fire spread? (Yes, `033` Fire Propagation should eventually read `WindGrid`).
- Should high wind damage weak structures? (Future feature).
  - *Architect:* This should be deferred to a future specification.
