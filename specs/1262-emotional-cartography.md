# 1262: Emotional Cartography

## 1. Overview

The physical map is reshaped by the collective trauma or joy of the colony. Areas where significant emotional events occur (mass celebrations, riots, tragic accidents) leave an "Emotional Imprint" on those specific map tiles. Over time, these imprints alter the local pathfinding weight. Pops seeking comfort might unconsciously path through "Joyous" zones, while "Traumatized" zones become dead areas that Pops refuse to walk through, cutting off vital thoroughfares and causing traffic jams.

## 2. Dependencies

- Layer 1 Pathfinding System (`src/layer1/pathfinding.rs`)
- Layer 1 Grid Position System (`src/layer1/nature/terrain.rs`)
- Layer 1 Chronicle/Event System (to generate events that leave imprints)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::culture::emotional_cartography::{EmotionalGrid, EmotionalImprint};
    use bevy_ecs::prelude::*;

    #[test]
    fn test_joyous_imprint_reduces_path_cost() {
        let mut world = World::new();

        let mut terrain = TerrainGrid::new(10, 10);
        // Setup terrain floor
        world.insert_resource(terrain);

        let mut emotional_grid = EmotionalGrid::new(10, 10);
        // Add a joyous imprint at (5, 5)
        emotional_grid.add_imprint(5, 5, EmotionalImprint::Joy(5.0));
        world.insert_resource(emotional_grid);

        // Assert that the cost to traverse (5, 5) is lower than default
        let cost_normal = crate::layer1::pathfinding::calculate_tile_cost(&world, 4, 4);
        let cost_joyous = crate::layer1::pathfinding::calculate_tile_cost(&world, 5, 5);
        assert!(cost_joyous < cost_normal);
    }

    #[test]
    fn test_traumatized_imprint_increases_path_cost() {
        let mut world = World::new();

        let mut terrain = TerrainGrid::new(10, 10);
        world.insert_resource(terrain);

        let mut emotional_grid = EmotionalGrid::new(10, 10);
        // Add a traumatized imprint at (3, 3)
        emotional_grid.add_imprint(3, 3, EmotionalImprint::Trauma(10.0));
        world.insert_resource(emotional_grid);

        // Assert that the cost to traverse (3, 3) is much higher
        let cost_normal = crate::layer1::pathfinding::calculate_tile_cost(&world, 2, 2);
        let cost_trauma = crate::layer1::pathfinding::calculate_tile_cost(&world, 3, 3);
        assert!(cost_trauma > cost_normal);
    }

    #[test]
    fn test_imprints_decay_over_time() {
        let mut world = World::new();
        let mut emotional_grid = EmotionalGrid::new(10, 10);
        emotional_grid.add_imprint(1, 1, EmotionalImprint::Trauma(10.0));
        world.insert_resource(emotional_grid);

        // Run decay system
        world.run_system_once(crate::layer1::culture::emotional_cartography::decay_imprints_system).unwrap();

        let grid = world.resource::<EmotionalGrid>();
        if let Some(EmotionalImprint::Trauma(intensity)) = grid.get(1, 1) {
            assert!(intensity < 10.0);
        } else {
            panic!("Imprint completely disappeared or changed type");
        }
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// In src/layer1/culture/emotional_cartography.rs
use bevy_ecs::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum EmotionalImprint {
    Joy(f32),
    Trauma(f32),
}

#[derive(Resource)]
pub struct EmotionalGrid {
    pub width: usize,
    pub height: usize,
    grid: Vec<Option<EmotionalImprint>>,
}

impl EmotionalGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![None; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<EmotionalImprint> {
        if x < self.width && y < self.height {
            self.grid[y * self.width + x]
        } else {
            None
        }
    }

    pub fn add_imprint(&mut self, x: usize, y: usize, imprint: EmotionalImprint) {
        if x < self.width && y < self.height {
            self.grid[y * self.width + x] = Some(imprint);
        }
    }
}

pub fn decay_imprints_system(mut grid: ResMut<EmotionalGrid>) {
    for tile in grid.grid.iter_mut() {
        if let Some(imprint) = tile {
            match imprint {
                EmotionalImprint::Joy(ref mut intensity) => {
                    *intensity -= 0.1;
                    if *intensity <= 0.0 { *tile = None; }
                }
                EmotionalImprint::Trauma(ref mut intensity) => {
                    *intensity -= 0.05; // Trauma fades slower
                    if *intensity <= 0.0 { *tile = None; }
                }
            }
        }
    }
}

// In src/layer1/pathfinding.rs (update tile cost calculation)
// Ensure you integrate this cleanly with the existing A* cost logic
pub fn calculate_tile_cost(world: &World, x: usize, y: usize) -> f32 {
    let mut base_cost = 1.0; // Standard floor cost

    if let Some(emotional_grid) = world.get_resource::<EmotionalGrid>() {
        if let Some(imprint) = emotional_grid.get(x, y) {
            match imprint {
                // Joy reduces cost (makes it more attractive) down to a minimum
                EmotionalImprint::Joy(intensity) => base_cost = (base_cost - (intensity * 0.1)).max(0.1),
                // Trauma increases cost massively
                EmotionalImprint::Trauma(intensity) => base_cost += intensity * 2.0,
            }
        }
    }

    base_cost
}
```

## 5. REFACTOR Phase: Quality & Design

- The `EmotionalGrid` could be folded into `TerrainGrid` if we want to reduce the number of global resources, but keeping it separate ensures we don't bloat the memory for simple terrain lookups.
- Pathfinding algorithms (A*) should cache the `EmotionalGrid` lookups or combine them when generating the initial navigation graph.
- Integration: Listen to `PopDeathEvent` or `RiotEvent` to automatically generate `Trauma` imprints. Listen to `FestivalEvent` to generate `Joy` imprints.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `emotional_cartography.rs`.
- [ ] A joyous imprint clearly reduces pathing weights to encourage routing.
- [ ] A trauma imprint clearly increases pathing weights to discourage routing.

## 7. Technical Guidance

- Create `src/layer1/culture/emotional_cartography.rs`.
- Add `EmotionalGrid` as an initialized resource in setup.
- Intercept the path cost calculation in `src/layer1/pathfinding.rs` to include the `EmotionalGrid` influence. Ensure that cost modifiers never drop below 0.1 to avoid infinite loops or negative path weights in A*.
- Do NOT rewrite the core A* logic, just wrap the `cost(a, b)` function.

## 8. Questions
*Builder: add questions here if spec is unclear.*
