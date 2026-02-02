# 018: Mining and Resources

## Overview

Implement the core resource tracking for Wood and Stone, and the mechanics for mining rock. This spec refactors `ColonyResources` into its own module and adds the `MiningProgress` component to track work on designated tiles. It defines the logic for converting Rock tiles to Dirt (yielding Stone) when mining is completed.

This spec focuses on the *mechanics* of mining. It does not implement the AI that assigns workers (see Spec 016), but provides the `mine_designation` function that any agent/system can call to perform the work.

## Dependencies

- `002` — Terrain grid (rock tiles)
- `017` — Designation system (designation entities)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/mining.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType, GridPosition};
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer1::resources::ColonyResources;

    #[test]
    fn test_mining_progress_default() {
        let progress = MiningProgress::default();
        assert_eq!(progress.current, 0.0);
        assert_eq!(progress.max, 100.0);
    }

    #[test]
    fn test_colony_resources_extended() {
        // This test belongs in resources.rs, but we spec it here as part of the feature
        let resources = ColonyResources::default();
        assert_eq!(resources.food, 0.0);
        assert_eq!(resources.wood, 0.0);
        assert_eq!(resources.stone, 0.0);
    }

    #[test]
    fn test_mine_designation_increments_progress() {
        let mut world = World::new();
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            MiningProgress { current: 0.0, max: 100.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Simulate 10 work units
        let result = mine_designation(&mut world, designation, 10.0);

        assert_eq!(result, MiningResult::InProgress);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        assert_eq!(progress.current, 10.0);
    }

    #[test]
    fn test_mine_designation_completion() {
        let mut world = World::new();

        // Setup Terrain
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5,5)
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Setup Resources
        world.insert_resource(ColonyResources::default());

        // Setup Designation
        let designation = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            MiningProgress { current: 95.0, max: 100.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Finish the job (needs 5, giving 10)
        let result = mine_designation(&mut world, designation, 10.0);

        // Check Result
        assert_eq!(result, MiningResult::Finished);

        // Check Entity Despawned
        assert!(world.get_entity(designation).is_err());

        // Check Terrain Changed
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt)); // Or appropriate floor

        // Check Resource Added
        let resources = world.resource::<ColonyResources>();
        assert_eq!(resources.stone, 1.0);
    }

    #[test]
    fn test_mine_designation_invalid_entity() {
        let mut world = World::new();
        let result = mine_designation(&mut world, Entity::from_raw(999), 10.0);
        assert_eq!(result, MiningResult::Invalid);
    }
}
```

**Test Coverage Requirements:**
- `MiningProgress` component defaults.
- `ColonyResources` structure (Wood, Stone).
- `mine_designation` logic:
    - Increments progress.
    - Respects completion threshold.
    - Modifies TerrainGrid correctly (Rock -> Dirt).
    - Updates ColonyResources (+Stone).
    - Despawns the designation entity on completion.
- Coverage ≥85% for `layer1/mining.rs` and `layer1/resources.rs`.

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### 1. Refactor Resources

Move `ColonyResources` from `src/layer1/farm.rs` to `src/layer1/resources.rs`.

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

*Note: You must update `src/layer1/farm.rs` to import `ColonyResources` from `crate::layer1::resources`.*

### 2. Mining Component and Logic

```rust
// src/layer1/mining.rs

use bevy_ecs::prelude::*;
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::terrain::{TerrainGrid, TerrainType, GridPosition};
use crate::layer1::resources::ColonyResources;

#[derive(Component, Debug, Clone)]
pub struct MiningProgress {
    pub current: f32,
    pub max: f32,
}

impl Default for MiningProgress {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 100.0, // Arbitrary work units required
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MiningResult {
    InProgress,
    Finished,
    Invalid,
}

pub fn mine_designation(
    world: &mut World,
    designation_entity: Entity,
    work_amount: f32,
) -> MiningResult {
    // 1. Get current state (scope to release borrow)
    let (mut current, max, pos) = if let Ok((progress, pos)) = world
        .query::<(&MiningProgress, &GridPosition)>()
        .get(world, designation_entity)
    {
        (progress.current, progress.max, *pos)
    } else {
        return MiningResult::Invalid;
    };

    // 2. Update Progress
    current += work_amount;

    // 3. Check completion
    if current >= max {
        // Apply Changes

        // Update Terrain
        let mut terrain = world.resource_mut::<TerrainGrid>();
        if let Some(tile) = terrain.get_mut(pos.x as usize, pos.y as usize) {
            if *tile == TerrainType::Rock {
                *tile = TerrainType::Dirt; // Rock becomes dirt after mining

                // Add Resources (only if it was rock)
                // Note: We need to drop terrain borrow before getting resources
            }
        }
        // (Drop terrain borrow implicitly or via scope if needed, but here it's fine as we access resources next)
    } else {
        // Just update component
        let mut progress = world.get_mut::<MiningProgress>(designation_entity).unwrap();
        progress.current = current;
        return MiningResult::InProgress;
    }

    // 4. Finalize Completion (Resource + Despawn)
    // We only reach here if current >= max

    // Add Stone
    let mut resources = world.resource_mut::<ColonyResources>();
    resources.stone += 1.0;

    // Despawn Designation
    world.despawn(designation_entity);

    MiningResult::Finished
}
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later
1. **Hardcoded Max Progress**: `100.0` is hardcoded. Should vary by tool tier or rock type?
   - Future: `TerrainType` properties (e.g., `Rock` takes 100, `HardRock` takes 200).
2. **Hardcoded Loot**: Always `+1.0 Stone`.
   - Future: Loot tables or probability.
3. **Hardcoded Terrain Result**: Always `Dirt`.
   - Future: `TerrainType::Floor(Material)` or just `Dirt`.

### Performance Considerations
- `mine_designation` accesses World multiple times.
- If called for 1000 agents per tick, this might be slow.
- Optimization: Batch processing or passing `&mut Query` instead of `&mut World`.

## Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `ColonyResources` is moved to `src/layer1/resources.rs`.
- [ ] `src/layer1/farm.rs` compiles with the new import.
- [ ] `MiningProgress` component exists.
- [ ] `mine_designation` correctly converts Rock to Dirt and adds Stone.
- [ ] `mine_designation` despawns the designation entity upon completion.
- [ ] Test coverage ≥85% for new files.

## Technical Guidance

### Integration with Designation System
When `try_designate` (from Spec 017) spawns a Mine designation, it should now also attach `MiningProgress::default()`.
*Builder Note: You might need to modify `src/layer1/designation.rs` to add this component during spawn.*

```rust
// src/layer1/designation.rs update suggestion
world.spawn((
    Designation { designation_type },
    GridPosition { x, y },
    MiningProgress::default(), // Added
));
```

### Module Structure
Ensure `mod.rs` in `layer1` exports the new modules.
```rust
pub mod resources;
pub mod mining;
pub use resources::*;
pub use mining::*;
```

## Questions
*Builder: add questions here if spec is unclear.*
