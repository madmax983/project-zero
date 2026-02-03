# 018: Mining Mechanics and Resource Tracking

## Overview

Refactor the resource system to track Wood and Stone in addition to Food. Implement the mechanics for mining designated Rock tiles, transforming them into Dirt and yielding Stone. This establishes the material economy.

## Dependencies

- `002` — Terrain grid (Rock to Dirt transformation)
- `017` — Designation System (Mining designation target)
- `008` — Existing `ColonyResources` (needs refactoring)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/resources.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::GridPosition;

    #[test]
    fn test_colony_resources_fields() {
        let resources = ColonyResources::default();
        assert_eq!(resources.food, 0.0);
        assert_eq!(resources.wood, 0.0);
        assert_eq!(resources.stone, 0.0);
    }

    #[test]
    fn test_mining_progress_component() {
        let progress = MiningProgress {
            work_done: 0.0,
            work_required: 10.0,
        };
        assert_eq!(progress.work_done, 0.0);
        assert_eq!(progress.work_required, 10.0);
    }

    #[test]
    fn test_mining_progress_default() {
        let progress = MiningProgress::default();
        assert!(progress.work_required > 0.0);
    }

    #[test]
    fn test_mine_rock_valid() {
        let mut world = World::new();
        // Setup Terrain
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5, 5)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Setup Resources
        world.insert_resource(ColonyResources::default());

        // Setup Designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            MiningProgress { work_done: 10.0, work_required: 10.0 }, // Fully worked
        )).id();

        // Perform Mining Logic (simulating completion)
        complete_mining_system(&mut world);

        // Assertions
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.stone, 1.0, "Should yield 1 stone");

        let terrain = world.resource::<TerrainGrid>();
        #[allow(clippy::cast_sign_loss)]
        assert_eq!(terrain.get(5, 5).unwrap(), TerrainType::Dirt, "Rock should become Dirt");

        assert!(world.get_entity(designation).is_none(), "Designation should be removed");
    }

    #[test]
    fn test_mine_rock_incomplete() {
        let mut world = World::new();
        // Setup Terrain
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        // Setup Designation (Not finished)
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            MiningProgress { work_done: 5.0, work_required: 10.0 },
        )).id();

        complete_mining_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.stone, 0.0, "Should not yield stone yet");

        assert!(world.get_entity(designation).is_some(), "Designation should remain");
    }

    #[test]
    fn test_mine_invalid_terrain() {
        let mut world = World::new();
        // Setup Terrain (Grass, not Rock)
        let tiles = vec![TerrainType::Grass; 100]; // (5, 5) is Grass
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            MiningProgress { work_done: 10.0, work_required: 10.0 },
        ));

        complete_mining_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.stone, 0.0, "Should not yield stone from Grass");

        // Designation might remain or be cancelled?
        // Logic: If designation is invalid, it should probably be cancelled/removed.
        // For now, let's assume it removes it to prevent infinite loops.
        assert_eq!(world.entities().len(), 0, "Invalid designation should be cleaned up");
    }
}
```

**Test Coverage Requirements:**
- `ColonyResources` struct (new fields)
- `MiningProgress` component
- `complete_mining_system` (Success, Incomplete, Invalid Terrain)
- Coverage ≥85% for `layer1/resources.rs`

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### 1. Create `src/layer1/resources.rs`

```rust
// src/layer1/resources.rs

use bevy_ecs::prelude::*;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::GridPosition;

/// Global colony resources.
#[derive(Resource, Default, Debug, Clone)]
pub struct ColonyResources {
    pub food: f32,
    pub wood: f32,
    pub stone: f32,
}

/// Tracks progress of mining/work on a designation.
#[derive(Component, Debug, Clone)]
pub struct MiningProgress {
    pub work_done: f32,
    pub work_required: f32,
}

impl Default for MiningProgress {
    fn default() -> Self {
        Self {
            work_done: 0.0,
            work_required: 10.0, // Default effort for rock
        }
    }
}

/// Checks completed mining designations and applies effects.
/// This system does NOT perform work; it only checks if work is done.
pub fn complete_mining_system(world: &mut World) {
    // Identify completed designations
    let completed: Vec<(Entity, GridPosition, DesignationType)> = world
        .query::<(Entity, &GridPosition, &Designation, &MiningProgress)>()
        .iter(world)
        .filter(|(_, _, _, progress)| progress.work_done >= progress.work_required)
        .map(|(e, pos, des, _)| (e, *pos, des.designation_type))
        .collect();

    let mut resources = world.resource_mut::<ColonyResources>();
    let mut terrain = world.resource_mut::<TerrainGrid>();

    for (entity, pos, designation_type) in completed {
        if designation_type == DesignationType::Mine {
            // Validate Terrain
            #[allow(clippy::cast_sign_loss)]
            if let Some(tile) = terrain.get_mut(pos.x as usize, pos.y as usize) {
                if *tile == TerrainType::Rock {
                    // Success!
                    *tile = TerrainType::Dirt; // or Floor if we had it
                    resources.stone += 1.0;
                }
            }
        }

        // Remove designation (even if invalid, it's "done")
        world.despawn(entity);
    }
}
```

### 2. Refactor `src/layer1/farm.rs`

Remove `ColonyResources` definition from `farm.rs` and import it from `resources.rs`.

```rust
// src/layer1/farm.rs

use crate::layer1::resources::ColonyResources;
// ... rest of file
```

### 3. Update `src/layer1/mod.rs`

```rust
pub mod resources;
pub use resources::*;
```

## REFACTOR Phase: Quality & Design

After tests pass, perform these refactors:

### Refactoring `ColonyResources`
- **Move**: Ensure `ColonyResources` is fully moved from `farm.rs`.
- **Breaking Change**: This will break `farm.rs` tests that try to instantiate `ColonyResources` locally.
- **Fix**: Update `farm.rs` tests to use `crate::layer1::resources::ColonyResources`.

### Designation Integration
- Update `designation.rs` logic (Spec 017) to attach `MiningProgress` when `Designation` is created?
- **Decision**: No, `try_designate` in 017 just creates the tag. The "Job System" (016) or a "Work System" should attach progress/workers.
- For now, let's update `try_designate` in `layer1/designation.rs` to *also* add `MiningProgress` immediately if it's a Mine designation. This makes the designation "workable" immediately.

```rust
// src/layer1/designation.rs
// In try_designate:
if designation_type == DesignationType::Mine {
     entity.insert(MiningProgress::default());
}
```

### Performance
- `complete_mining_system` iterates all designations. This is cheap (N < 1000).

## Acceptance Criteria

- [ ] `ColonyResources` is defined in `layer1/resources.rs`.
- [ ] `farm.rs` imports `ColonyResources` correctly.
- [ ] Tests in `resources.rs` pass (coverage ≥85%).
- [ ] `MiningProgress` component exists.
- [ ] Mining a Rock tile transforms it to Dirt and adds 1 Stone.
- [ ] Completed designation is removed.
- [ ] `cargo clippy` passes.

## Technical Guidance

- Be careful with the circular dependency if `resources` imports `farm` (it shouldn't).
- `farm` depends on `resources` (OK).
- `designation` does NOT depend on `resources` (OK).
- `resources` depends on `designation` (OK).
- `resources` depends on `terrain` (OK).

## Questions

*Builder: add questions here if spec is unclear.*
