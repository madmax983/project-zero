# 093: Desire Paths

## Overview

Implements terrain erosion based on Pop movement. Frequently traveled `Grass` tiles degrade into `Dirt` and eventually into `Path`. `Path` tiles offer a movement speed bonus but have lower beauty. This creates emergent, organic road networks based on actual usage.

## Dependencies

- `002` — Terrain Grid (for `TerrainType` modification)
- `004` — Pop Entity (for movement system hook)
- `044` — Horticulture & Beauty (for beauty impact)
- `031` — Pop Morale (for beauty impact on mood)

## RED Phase: Tests First

Write these tests in `src/layer1/erosion_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::erosion::{ErosionGrid, update_erosion_system, MOVEMENT_EROSION_AMOUNT};
    use crate::layer1::pop::Pop;
    use crate::layer1::execution::MovementTarget;

    fn setup_world() -> World {
        let mut world = World::new();
        let width = 10;
        let height = 10;

        // Setup Terrain
        let mut terrain = TerrainGrid {
            width,
            height,
            tiles: vec![TerrainType::Grass; width * height],
        };
        world.insert_resource(terrain);

        // Setup Erosion Grid
        world.insert_resource(ErosionGrid::new(width, height));

        world
    }

    #[test]
    fn test_erosion_accumulation() {
        let mut world = setup_world();
        let mut erosion_grid = world.resource_mut::<ErosionGrid>();

        // Simulate step
        erosion_grid.add_erosion(5, 5, 10);
        assert_eq!(erosion_grid.get(5, 5), 10);
    }

    #[test]
    fn test_grass_erodes_to_dirt() {
        let mut world = setup_world();
        let mut erosion_grid = world.resource_mut::<ErosionGrid>();

        // Set erosion just below threshold
        erosion_grid.set(5, 5, 99);

        // Add more erosion to cross threshold (assumed 100)
        erosion_grid.add_erosion(5, 5, 2);

        // Run update system
        update_erosion_system(&mut world);

        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));
    }

    #[test]
    fn test_dirt_erodes_to_path() {
        let mut world = setup_world();
        let mut terrain = world.resource_mut::<TerrainGrid>();
        // Start as Dirt
        terrain.tiles[55] = TerrainType::Dirt; // (5,5) assuming 10 width

        let mut erosion_grid = world.resource_mut::<ErosionGrid>();
        // Set high erosion (assumed threshold 500)
        erosion_grid.set(5, 5, 501);

        update_erosion_system(&mut world);

        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Path));
    }

    #[test]
    fn test_path_decay_unused() {
        let mut world = setup_world();
        let mut terrain = world.resource_mut::<TerrainGrid>();
        terrain.tiles[55] = TerrainType::Path;

        let mut erosion_grid = world.resource_mut::<ErosionGrid>();
        erosion_grid.set(5, 5, 0); // No usage

        // Simulate many ticks of regrowth
        for _ in 0..100 {
            crate::layer1::erosion::regrowth_system(&mut world);
        }

        let terrain = world.resource::<TerrainGrid>();
        // Should revert to Dirt or Grass
        assert_ne!(terrain.get(5, 5), Some(TerrainType::Path));
    }

    #[test]
    fn test_path_speed_bonus() {
        // This likely belongs in movement_tests.rs or similar, but spec defines it.
        // Assuming movement cost function exists:
        let cost_grass = crate::layer1::execution::get_movement_cost(TerrainType::Grass);
        let cost_path = crate::layer1::execution::get_movement_cost(TerrainType::Path);

        assert!(cost_path < cost_grass, "Path should be faster than Grass");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `TerrainType`

Add `Path` variant to `src/layer1/terrain.rs`.

```rust
pub enum TerrainType {
    Grass,
    Dirt,
    Rock,
    Water,
    Tree,
    Path, // New
}

impl TerrainType {
    pub const fn is_walkable(self) -> bool {
        !matches!(self, Self::Rock | Self::Water)
    }

    // Add movement cost helper
    pub fn movement_cost(&self) -> f32 {
        match self {
            Self::Path => 0.8, // Fast
            Self::Dirt => 1.0,
            Self::Grass => 1.0,
            Self::Tree => 1.5, // Slow
            _ => 1.0,
        }
    }
}
```

### 2. Create `ErosionGrid` Resource

Create `src/layer1/erosion.rs`.

```rust
use bevy_ecs::prelude::*;
use crate::layer1::terrain::{TerrainGrid, TerrainType};

#[derive(Resource, Default)]
pub struct ErosionGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<u16>, // 0-65535 usage counter
}

impl ErosionGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0; width * height],
        }
    }

    pub fn add_erosion(&mut self, x: usize, y: usize, amount: u16) {
        let idx = y * self.width + x;
        if idx < self.values.len() {
            self.values[idx] = self.values[idx].saturating_add(amount);
        }
    }

    pub fn get(&self, x: usize, y: usize) -> u16 {
        let idx = y * self.width + x;
        if idx < self.values.len() {
            self.values[idx]
        } else {
            0
        }
    }

    pub fn set(&mut self, x: usize, y: usize, val: u16) {
        let idx = y * self.width + x;
        if idx < self.values.len() {
            self.values[idx] = val;
        }
    }
}

pub const EROSION_THRESHOLD_DIRT: u16 = 100;
pub const EROSION_THRESHOLD_PATH: u16 = 500;
pub const MOVEMENT_EROSION_AMOUNT: u16 = 5;

pub fn update_erosion_system(
    mut terrain: ResMut<TerrainGrid>,
    erosion: Res<ErosionGrid>,
) {
    for y in 0..terrain.height {
        for x in 0..terrain.width {
            let idx = y * terrain.width + x;
            let current_type = terrain.tiles[idx];
            let erosion_val = erosion.values[idx];

            if current_type == TerrainType::Grass && erosion_val >= EROSION_THRESHOLD_DIRT {
                terrain.tiles[idx] = TerrainType::Dirt;
            } else if current_type == TerrainType::Dirt && erosion_val >= EROSION_THRESHOLD_PATH {
                terrain.tiles[idx] = TerrainType::Path;
            }
        }
    }
}

pub fn regrowth_system(
    mut terrain: ResMut<TerrainGrid>,
    mut erosion: ResMut<ErosionGrid>,
) {
    // Run occasionally (e.g. daily) or slow trickle every tick
    for i in 0..erosion.values.len() {
        if erosion.values[i] > 0 {
            erosion.values[i] -= 1; // Decay
        }

        // Revert terrain if erosion drops
        let current_type = terrain.tiles[i];
        if current_type == TerrainType::Path && erosion.values[i] < EROSION_THRESHOLD_PATH {
            terrain.tiles[i] = TerrainType::Dirt;
        } else if current_type == TerrainType::Dirt && erosion.values[i] < EROSION_THRESHOLD_DIRT {
            terrain.tiles[i] = TerrainType::Grass;
        }
    }
}
```

### 3. Hook into Movement

In `src/layer1/execution.rs` (or where movement happens):

```rust
// In movement_system, when a pop enters a tile:
// let mut erosion = world.resource_mut::<ErosionGrid>();
// erosion.add_erosion(pos.x, pos.y, MOVEMENT_EROSION_AMOUNT);
```

### 4. Beauty Integration

Update `src/layer1/beauty.rs` to consider `TerrainType`.

```rust
// update_beauty_grid_system
// Iterate terrain tiles.
// If Path, beauty -= 2.0.
// If Dirt, beauty -= 1.0.
// If Grass, beauty += 1.0.
```

## REFACTOR Phase: Quality & Design

- **Erosion Resource**: Consider `SparseSet` if erosion is rare, but Grid is simpler for lookup.
- **Optimization**: `update_erosion_system` scans the whole map. This is slow if map is huge. Use `Dirty` flag or only check modified tiles (return list of modified indices from `add_erosion`).
- **Visuals**: `Path` should render as distinct character (e.g. `≡` or `░`). Update `src/ui/map.rs`.

## Acceptance Criteria

- [ ] `TerrainType::Path` exists.
- [ ] Pops walking on `Grass` eventually turns it to `Dirt`, then `Path`.
- [ ] `Path` tiles have lower movement cost (faster).
- [ ] `Path` tiles have lower beauty (affects Morale).
- [ ] Unused `Path` tiles revert to `Dirt`/`Grass` over time.
- [ ] Tests pass.

## Technical Guidance

- Movement cost logic might be hardcoded in `movement_system`. Refactor it to call `TerrainType::movement_cost()`.
- Ensure `ErosionGrid` is initialized in `001-project-scaffold` or added during system startup.
- Be careful with `regrowth_system` frequency. Running it every tick is wasteful. Maybe run it every 100 ticks or on `DayNightCycle`.

## Questions

- Should paved roads (Constructed) be immune to erosion? (Yes, `BuildingType::Road` overlay or separate TerrainType? For now, assume Constructed Roads are Buildings that sit *on top* of terrain, preventing erosion updates).
  - *Architect:* Yes, paved roads are immune to erosion.
