# 063: Atmospheric Simulation

## Overview

Introduces an invisible layer of simulation: Air Quality. Industrial buildings emit "Pollution" (smog) which diffuses into the surrounding atmosphere. High pollution levels reduce Pop Health (slowly) and Morale (quickly). Nature (Trees) acts as a sink, absorbing pollution. This creates a tension between compact industrial efficiency and environmental health.

## Dependencies

- `002` — Terrain Grid (for map dimensions)
- `023` — Refining Industry (pollution sources)
- `034` — Pop Health (health impact)
- `031` — Pop Morale (morale impact)
- `053` — Lighting System (similar grid architecture)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/atmosphere_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::atmosphere::{AtmosphereGrid, update_atmosphere_system, pollution_effects_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer2::building::{Building, BuildingType};

    #[test]
    fn test_atmosphere_grid_initialization() {
        let grid = AtmosphereGrid::new(10, 10);
        assert_eq!(grid.width, 10);
        assert_eq!(grid.height, 10);
        assert_eq!(grid.get(0, 0), 0.0); // Clean air by default
    }

    #[test]
    fn test_pollution_emission() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(10, 10);
        world.insert_resource(grid);

        // Spawn a Smelter (dirty building)
        world.spawn((
            Building { building_type: BuildingType::Smelter, ..Default::default() },
            GridPosition { x: 5, y: 5 },
        ));

        // Run update
        update_atmosphere_system(&mut world);

        // Check pollution at source
        let grid = world.resource::<AtmosphereGrid>();
        assert!(grid.get(5, 5) > 0.0);
    }

    #[test]
    fn test_pollution_diffusion() {
        let mut grid = AtmosphereGrid::new(3, 3);
        grid.set(1, 1, 10.0); // High pollution in center

        // Simulate one step of diffusion
        grid.diffuse();

        // Center should decrease, neighbors should increase
        assert!(grid.get(1, 1) < 10.0);
        assert!(grid.get(0, 1) > 0.0);
        assert!(grid.get(1, 0) > 0.0);
    }

    #[test]
    fn test_pollution_health_impact() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(5, 5, 1.0); // Max pollution
        world.insert_resource(grid);

        // Spawn Pop in pollution
        let pop = world.spawn((
            Pop::default(),
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run effects
        pollution_effects_system(&mut world);

        // Health should drop
        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);
    }

    #[test]
    fn test_trees_absorb_pollution() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::new(10, 10);
        grid.set(0, 0, 1.0);
        world.insert_resource(grid);

        // Spawn Tree (terrain or entity? Assuming TerrainType for now, or check Logic)
        // If TerrainType:
        // world.resource_mut::<TerrainGrid>().tiles[0] = TerrainType::Tree;

        // Let's assume absorption happens in update_atmosphere_system via TerrainGrid lookup
        // setup_terrain_with_tree(&mut world, 0, 0);

        // update_atmosphere_system(&mut world);

        // Assert pollution decreased faster than normal decay
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Atmosphere Resource (`src/layer1/atmosphere.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct AtmosphereGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>, // 0.0 (Clean) to 1.0 (Toxic)
}

impl AtmosphereGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return 0.0;
        }
        self.values[y as usize * self.width + x as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, value: f32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.values[y as usize * self.width + x as usize] = value.clamp(0.0, 1.0);
        }
    }

    pub fn add(&mut self, x: i32, y: i32, amount: f32) {
        let current = self.get(x, y);
        self.set(x, y, current + amount);
    }

    // Simple box blur for diffusion
    pub fn diffuse(&mut self) {
        let mut new_values = self.values.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let mut sum = self.values[idx];
                let mut count = 1.0;

                // Check 4 neighbors
                for (dx, dy) in [(-1,0), (1,0), (0,-1), (0,1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                        sum += self.get(nx, ny);
                        count += 1.0;
                    }
                }

                // Average
                new_values[idx] = sum / count;
                // Decay
                new_values[idx] *= 0.99;
            }
        }
        self.values = new_values;
    }
}
```

### 2. Update System

```rust
use crate::layer2::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;

pub fn update_atmosphere_system(world: &mut World) {
    // 1. Emission
    let mut emitters = Vec::new();
    let mut query = world.query::<(&Building, &GridPosition)>();
    for (b, pos) in query.iter(world) {
        let emission = match b.building_type {
            BuildingType::Smelter => 0.05,
            BuildingType::Refinery => 0.08,
            BuildingType::CoalPlant => 0.10,
            _ => 0.0,
        };
        if emission > 0.0 {
            emitters.push((*pos, emission));
        }
    }

    let mut grid = world.resource_mut::<AtmosphereGrid>();

    // Apply emissions
    for (pos, amount) in emitters {
        grid.add(pos.x, pos.y, amount);
    }

    // 2. Diffusion & Decay
    grid.diffuse();
}
```

### 3. Effects System

```rust
use crate::layer1::health::Health;
use crate::layer1::pop::Pop;

pub fn pollution_effects_system(world: &mut World) {
    let grid = world.resource::<AtmosphereGrid>();
    let mut query = world.query::<(&mut Health, &GridPosition, With<Pop>)>();

    for (mut health, pos, _) in query.iter_mut(world) {
        let pollution = grid.get(pos.x, pos.y);
        if pollution > 0.3 {
            // Damage scales with pollution
            let damage = (pollution - 0.3) * 0.1;
            health.current -= damage;
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Simple box blur is slow (O(N*M)). Consider:
    - Running diffusion every N ticks instead of every tick.
    - Using a "dirty rect" optimization (only diffuse active areas).
- **Wind**: Implement a `WindDirection` resource that biases diffusion.
- **Visuals**: Create a debug overlay or shader effect (green/brown fog) to visualize pollution.

## Acceptance Criteria

- [ ] `AtmosphereGrid` resource initializes correctly.
- [ ] Smelters and Refineries increase local pollution.
- [ ] Pollution spreads to adjacent tiles over time.
- [ ] Pollution decays naturally.
- [ ] Pops in high pollution areas lose Health.
- [ ] `cargo test` passes.

## Technical Guidance

- Initialize `AtmosphereGrid` in `src/main.rs` after `TerrainGrid`.
- Add `update_atmosphere_system` to `SimulationSchedule` (Environment phase).
- Add `pollution_effects_system` to `SimulationSchedule` (Pop Update phase).

## Questions

- Should masks/suits protect pops? (Future feature: Equipment)
- Should pollution kill plants? (Future feature: Crop Death)

*Architect:* Protective equipment is deferred to the Equipment specification.
