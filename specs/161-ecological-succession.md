# 161: Ecological Succession

## Overview

Implement a dynamic ecosystem where flora grows and changes over time based on environmental conditions. This transforms the map from a static backdrop into a living entity that reacts to player actions (e.g., deforestation).

The core mechanic is **Succession**: bare land (`Dirt`) naturally evolves into `Grass`, then `Shrub`, and finally `Tree` (Climax Community) if left undisturbed and seeded by neighbors.

## Dependencies

- `002` — Terrain Grid (for `TerrainType`)
- `019` — Forestry System (for `Tree` interaction)

## RED Phase: Tests First

These tests define the behavior of the `EcologyGrid` and the succession logic. They should FAIL initially.

```rust
// src/layer1/ecology_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainType, TerrainGrid};
    use crate::layer1::ecology::{EcologyGrid, FloraType, succession_step, PlantGrowth};

    #[test]
    fn test_ecology_grid_initialization() {
        let mut world = World::new();
        // Setup TerrainGrid
        let terrain = TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Dirt; 100] };
        world.insert_resource(terrain);

        // Initialize Ecology
        EcologyGrid::initialize(&mut world);

        let ecology = world.resource::<EcologyGrid>();
        assert_eq!(ecology.width, 10);
        assert_eq!(ecology.height, 10);
        // Initially match terrain (Dirt has no flora)
        assert_eq!(ecology.get_flora(0, 0), FloraType::None);
    }

    #[test]
    fn test_succession_dirt_to_grass() {
        // Dirt tiles should eventually become Grass if neighbor has Grass/Seeds
        // Here we test the logic function directly

        let current_terrain = TerrainType::Dirt;
        let current_flora = FloraType::None;
        let neighbors = vec![FloraType::Grass]; // Neighbor has grass

        // Act
        let (new_terrain, new_flora) = succession_step(current_terrain, current_flora, &neighbors, 1.0); // 1.0 = 100% chance for test

        // Assert
        assert_eq!(new_terrain, TerrainType::Grass);
        assert_eq!(new_flora, FloraType::Grass);
    }

    #[test]
    fn test_succession_grass_to_shrub() {
        // Grass eventually grows into Shrub
        let current_terrain = TerrainType::Grass;
        let current_flora = FloraType::Grass;
        let neighbors = vec![FloraType::Shrub]; // Neighbor has shrub seeds

        let (new_terrain, new_flora) = succession_step(current_terrain, current_flora, &neighbors, 1.0);

        assert_eq!(new_terrain, TerrainType::Grass); // Terrain stays grass base
        assert_eq!(new_flora, FloraType::Shrub);
    }

    #[test]
    fn test_succession_shrub_to_tree() {
        // Shrub eventually becomes Tree
        let current_terrain = TerrainType::Grass;
        let current_flora = FloraType::Shrub;
        let neighbors = vec![FloraType::Tree]; // Neighbor has tree seeds

        let (new_terrain, new_flora) = succession_step(current_terrain, current_flora, &neighbors, 1.0);

        assert_eq!(new_terrain, TerrainType::Tree); // Terrain becomes Tree
        assert_eq!(new_flora, FloraType::Tree);
    }

    #[test]
    fn test_no_growth_without_seeds() {
        // Dirt with no neighbors should not grow (unless spontaneous generation is enabled, which we disable for this test)
        let current_terrain = TerrainType::Dirt;
        let current_flora = FloraType::None;
        let neighbors = vec![];

        let (new_terrain, new_flora) = succession_step(current_terrain, current_flora, &neighbors, 0.0);

        assert_eq!(new_terrain, TerrainType::Dirt);
        assert_eq!(new_flora, FloraType::None);
    }

    #[test]
    fn test_rock_does_not_grow() {
        let current_terrain = TerrainType::Rock;
        let current_flora = FloraType::None;
        let neighbors = vec![FloraType::Tree];

        let (new_terrain, new_flora) = succession_step(current_terrain, current_flora, &neighbors, 1.0);

        assert_eq!(new_terrain, TerrainType::Rock);
        assert_eq!(new_flora, FloraType::None);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `FloraType` and `EcologyGrid`

```rust
// src/layer1/ecology.rs

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FloraType {
    None,
    Grass,
    Shrub,
    Tree,
}

#[derive(Resource)]
pub struct EcologyGrid {
    pub width: usize,
    pub height: usize,
    pub flora: Vec<FloraType>,
}

impl EcologyGrid {
    pub fn initialize(world: &mut World) {
        let terrain = world.resource::<TerrainGrid>();
        let mut flora = vec![FloraType::None; terrain.width * terrain.height];

        // Sync initial state
        for (i, tile) in terrain.tiles.iter().enumerate() {
            flora[i] = match tile {
                TerrainType::Grass => FloraType::Grass,
                TerrainType::Tree => FloraType::Tree,
                _ => FloraType::None,
            };
        }

        world.insert_resource(EcologyGrid {
            width: terrain.width,
            height: terrain.height,
            flora,
        });
    }

    pub fn get_flora(&self, x: usize, y: usize) -> FloraType {
        // ... (bounds check logic similar to TerrainGrid)
    }
}
```

### 2. Implement `succession_step` logic

```rust
pub fn succession_step(
    terrain: TerrainType,
    flora: FloraType,
    neighbors: &[FloraType],
    roll: f32
) -> (TerrainType, FloraType) {
    if !terrain.is_soil() { // Helper to check if Dirt/Grass
        return (terrain, flora);
    }

    // Simplistic succession logic
    // Needs tweaking for balance
    match flora {
        FloraType::None => {
            if neighbors.contains(&FloraType::Grass) || roll < 0.01 { // Spontaneous grass
                return (TerrainType::Grass, FloraType::Grass);
            }
        },
        FloraType::Grass => {
            if neighbors.contains(&FloraType::Shrub) || neighbors.contains(&FloraType::Tree) {
                if roll < 0.1 { return (TerrainType::Grass, FloraType::Shrub); }
            }
        },
        FloraType::Shrub => {
            if neighbors.contains(&FloraType::Tree) {
                if roll < 0.05 { return (TerrainType::Tree, FloraType::Tree); }
            }
        },
        _ => {}
    }

    (terrain, flora)
}
```

### 3. Implement System

```rust
pub fn ecological_succession_system(
    mut terrain: ResMut<TerrainGrid>,
    mut ecology: ResMut<EcologyGrid>,
) {
    let mut rng = rand::thread_rng();

    // Optimization: Only update X random tiles per tick instead of iterating all
    for _ in 0..100 {
        let x = rng.gen_range(0..ecology.width);
        let y = rng.gen_range(0..ecology.height);

        let current_terrain = terrain.get(x, y).unwrap();
        let current_flora = ecology.get_flora(x, y);

        // Get neighbors (simplify for now)
        let neighbors = vec![]; // ... populate neighbors

        let (new_terrain, new_flora) = succession_step(current_terrain, current_flora, &neighbors, rng.gen());

        if new_terrain != current_terrain {
            terrain.set(x, y, new_terrain);
        }
        if new_flora != current_flora {
            ecology.set_flora(x, y, new_flora);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Terrain Integration**: Currently `TerrainType` and `FloraType` duplicate state (e.g., `TerrainType::Tree` implies `FloraType::Tree`). Consider decoupling visual terrain from logical flora entirely, or keeping them synced. For MVP, syncing is safer.
- **Performance**: Random sampling (Monte Carlo) is good for large grids. Ensure the sample rate scales with map size.
- **Visuals**: `Shrub` needs a visual representation. It might need a new `TerrainType` or a `RenderEntity` on top. For MVP, treating it as `Grass` visually or adding `TerrainType::Shrub` is acceptable. (Let's add `TerrainType::Shrub` in Green Phase).
- **Invasive Species**: Future extension could add `FloraType::Kudzu` which ignores neighbors and spreads fast.

## Acceptance Criteria

- [ ] `EcologyGrid` resource exists and tracks `FloraType`.
- [ ] `FloraType` enum includes `None`, `Grass`, `Shrub`, `Tree`.
- [ ] Succession logic transforms `Dirt` -> `Grass` -> `Shrub` -> `Tree` over time.
- [ ] Growth requires neighboring seeds (or rare spontaneous event).
- [ ] System runs efficiently (stochastic updates).
- [ ] Tests pass with ≥85% coverage.

## Technical Guidance

- **TerrainType Sync**: When `succession_step` returns `FloraType::Tree`, you MUST update `TerrainGrid` to `TerrainType::Tree` so other systems (Forestry) recognize it.
- **Shrub Representation**: Add `TerrainType::Shrub` to `terrain.rs`. It should look like a small bush (e.g., `"*"`, Color::Green).
