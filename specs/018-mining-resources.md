# 018: Mining and Resources

## Overview

Implement the core resource system (`ColonyResources`) to track Wood and Stone, and the mechanics for mining Rock tiles to produce Stone. This creates the feedback loop between Designation (017) and Construction (future).

## Dependencies

- `002` — Terrain Grid
- `017` — Designation System (for removing designation upon completion)
- `008` — Farm (currently holds `ColonyResources`, which will be refactored)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/resources.rs - New module (move ColonyResources here)

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::designation::{Designation, DesignationType};

    #[test]
    fn test_colony_resources_fields() {
        let resources = ColonyResources::default();
        // Check for new fields
        assert_eq!(resources.food, 0.0);
        assert_eq!(resources.wood, 0.0);
        assert_eq!(resources.stone, 0.0);
    }

    #[test]
    fn test_mining_progress_component() {
        let progress = MiningProgress {
            current: 0.0,
            max: 100.0,
        };
        assert_eq!(progress.current, 0.0);
        assert_eq!(progress.max, 100.0);
    }

    #[test]
    fn test_mine_rock_increments_progress() {
        let mut world = World::new();
        // Setup Rock tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5, 5)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Setup Resources
        world.insert_resource(ColonyResources::default());

        // Spawn Designation with MiningProgress
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            MiningProgress { current: 0.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Perform work (simulate 1 tick of work)
        mine_rock(&mut world, designation, 1.0);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert_eq!(progress.current, 1.0);
    }

    #[test]
    fn test_mine_rock_completion() {
        let mut world = World::new();
        // Setup Rock tile
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        // Spawn Designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            MiningProgress { current: 9.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Complete the work
        mine_rock(&mut world, designation, 1.0);

        // 1. Entity should be despawned (Designation removed)
        assert!(world.get_entity(designation).is_err());

        // 2. Terrain should be Dirt
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // 3. Resources should increase
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.stone, 1.0);
    }

    #[test]
    fn test_mine_rock_ignores_non_rock() {
        let mut world = World::new();
        // Setup Grass tile (cannot mine grass for stone)
        let mut tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            MiningProgress { current: 0.0, max: 10.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        mine_rock(&mut world, designation, 5.0);

        // Should not progress
        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert_eq!(progress.current, 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Refactor ColonyResources

Move `ColonyResources` from `src/layer1/farm.rs` to `src/layer1/resources.rs`. Add `wood` and `stone`.

```rust
// src/layer1/resources.rs

use bevy_ecs::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct ColonyResources {
    pub food: f32,
    pub wood: f32,
    pub stone: f32,
}
```

*Update `farm.rs` to use `crate::layer1::resources::ColonyResources`.*

### 2. Mining Logic

```rust
// src/layer1/resources.rs

use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::designation::Designation; // Assumes spec 017 implemented
use crate::layer1::GridPosition;

#[derive(Component, Debug)]
pub struct MiningProgress {
    pub current: f32,
    pub max: f32,
}

impl Default for MiningProgress {
    fn default() -> Self {
        Self { current: 0.0, max: 100.0 }
    }
}

/// Applies work to a mining designation.
/// If complete, transforms terrain and awards resources.
pub fn mine_rock(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // 1. Get position and verify terrain
    let pos = if let Some(pos) = world.get::<GridPosition>(designation_entity) {
        *pos
    } else {
        return;
    };

    let is_rock = {
        let terrain = world.resource::<TerrainGrid>();
        terrain.get(pos.x as usize, pos.y as usize) == Some(TerrainType::Rock)
    };

    if !is_rock {
        return;
    }

    // 2. Update progress
    let mut completed = false;
    if let Some(mut progress) = world.get_mut::<MiningProgress>(designation_entity) {
        progress.current += work_amount;
        if progress.current >= progress.max {
            completed = true;
        }
    }

    // 3. Handle completion
    if completed {
        // Change terrain
        let mut terrain = world.resource_mut::<TerrainGrid>();
        if let Some(idx) = (pos.y as usize).checked_mul(terrain.width).and_then(|y| y.checked_add(pos.x as usize)) {
            if idx < terrain.tiles.len() {
                terrain.tiles[idx] = TerrainType::Dirt;
            }
        }

        // Add resources
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.stone += 1.0;

        // Remove designation
        world.despawn(designation_entity);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Module Structure**: Ensure `farm.rs` and other modules import `ColonyResources` correctly after the move.
- **Configurable Hardness**: `max` progress should depend on material (Rock vs future Iron).
- **Yields**: Currently 1.0 Stone per tile. Should be configurable.
- **Partial Updates**: `mine_rock` does a lot (check, update, transform). Could be split, but for Layer 1 it's fine.

## Acceptance Criteria

- [ ] `ColonyResources` is in `src/layer1/resources.rs`
- [ ] `ColonyResources` has `food`, `wood`, `stone`
- [ ] `MiningProgress` component exists
- [ ] `mine_rock` function passes all RED tests
- [ ] Existing `farm.rs` tests pass (after updating imports)
- [ ] Mining a Rock tile turns it to Dirt and adds 1.0 Stone
- [ ] Coverage ≥85% for `src/layer1/resources.rs`

## Technical Guidance

- **Refactoring Note**: When moving `ColonyResources`, you will break `farm.rs`. Fix imports immediately.
- **ECS Pattern**: `mine_rock` is a "logic function" called by a System (e.g. `perform_jobs_system` in the future). It takes `&mut World` to handle the structural changes (despawn, resource mutation) easily.
- **Indexing**: Remember `y * width + x` for 1D vector access.

## Questions

*Builder: add questions here if spec is unclear.*
