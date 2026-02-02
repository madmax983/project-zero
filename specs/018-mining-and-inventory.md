# 018: Mining and Inventory

## Overview

Implement the core mechanics for resource gathering, specifically mining. This spec extends the `ColonyResources` system to track Stone and Wood (in addition to Food) and defines the `mining_system` which converts pop labor into resources.

This spec focuses on the **mechanic** of mining (converting Rock -> Stone). The **decision** to mine is handled by the Utility AI (Spec 016), and the **designation** of tiles is handled by the Designation System (Spec 017).

## Dependencies

- `002` — Terrain grid (Rock tiles)
- `008` — Farm building (defines `ColonyResources`)
- `017` — Designation System (defines `Designation` and `DesignationType`)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/gathering.rs - Test module at end of file

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::pop::Pop;
    use crate::layer1::pop::GridPosition;
    use crate::layer1::designation::{Designation, DesignationType};
    use crate::layer2::farm::ColonyResources; // Assuming spec 008 exists

    #[test]
    fn test_colony_resources_extended() {
        // This test ensures ColonyResources has the new fields
        // Note: You may need to update the struct definition in layer2/farm.rs first
        // to make this compile, but strictly following TDD, we write the test first.
        let resources = ColonyResources::default();
        assert_eq!(resources.food, 0.0);
        assert_eq!(resources.wood, 0.0);
        assert_eq!(resources.stone, 0.0);
    }

    #[test]
    fn test_work_progress_component() {
        let progress = WorkProgress {
            current: 0.0,
            required: 100.0,
        };
        assert_eq!(progress.current, 0.0);
        assert_eq!(progress.required, 100.0);
    }

    #[test]
    fn test_mining_system_no_pop() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Rock; 100],
        });

        // Designation without a pop
        world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            WorkProgress { current: 0.0, required: 10.0 },
        ));

        mining_system(&mut world);

        // Progress should not change
        let (_, progress) = world.query::<(&Designation, &WorkProgress)>().single(&world);
        assert_eq!(progress.current, 0.0);
    }

    #[test]
    fn test_mining_system_progress() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Rock; 100],
        });

        // Designation
        world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            WorkProgress { current: 0.0, required: 10.0 },
        ));

        // Pop at the location
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
        ));

        mining_system(&mut world);

        // Progress should increase
        let (_, progress) = world.query::<(&Designation, &WorkProgress)>().single(&world);
        assert!(progress.current > 0.0, "Progress should increase when pop is present");
    }

    #[test]
    fn test_mining_system_completion() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        // Set up map with Rock at (5,5)
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Designation almost complete
        let designation_entity = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            WorkProgress { current: 9.9, required: 10.0 }, // 1 tick away
        )).id();

        // Pop at the location
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
        ));

        // Run system
        mining_system(&mut world);

        // 1. Resource gained
        let resources = world.resource::<ColonyResources>();
        assert!(resources.stone >= 1.0, "Stone should be added");

        // 2. Terrain changed
        let terrain = world.resource::<TerrainGrid>();
        assert_ne!(terrain.get(5, 5), Some(TerrainType::Rock), "Rock should be gone");
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt), "Rock should become Dirt/Floor");

        // 3. Designation removed
        assert!(world.get::<Designation>(designation_entity).is_none(), "Designation should be despawned");
    }

    #[test]
    fn test_mining_system_invalid_terrain() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default());
        // Grass (invalid for mining)
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });

        // Designation on Grass
        let designation_entity = world.spawn((
            Designation { designation_type: DesignationType::Mine },
            GridPosition { x: 5, y: 5 },
            WorkProgress { current: 0.0, required: 10.0 },
        )).id();

        world.spawn((Pop, GridPosition { x: 5, y: 5 }));

        mining_system(&mut world);

        // Should probably just cancel the designation or do nothing
        // In this spec, let's say it cancels it because it's invalid
        assert!(world.get::<Designation>(designation_entity).is_none(), "Invalid designation should be removed");
    }
}
```

**Test Coverage Requirements:**
- `ColonyResources` has wood/stone.
- `WorkProgress` component exists.
- `mining_system` increments progress when Pop is present.
- `mining_system` completes work: updates inventory, modifies terrain, removes entity.
- `mining_system` handles invalid state (mining non-rock).
- Coverage ≥85% for `layer1/gathering.rs`.

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make all RED tests pass.

### Update Resources

**Modify** `src/layer2/farm.rs` (or move definition to shared location if preferred, but extending in place is easiest for now):

```rust
// src/layer2/farm.rs

#[derive(Resource, Default)]
pub struct ColonyResources {
    pub food: f32,
    pub wood: f32,  // NEW
    pub stone: f32, // NEW
}
```

### New Components

```rust
// src/layer1/gathering.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, GridPosition};
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer2::farm::ColonyResources;

#[derive(Component, Default)]
pub struct WorkProgress {
    pub current: f32,
    pub required: f32,
}
```

### Mining System

```rust
// src/layer1/gathering.rs

const MINING_SPEED: f32 = 0.2; // Progress per tick
const STONE_PER_ROCK: f32 = 5.0;

pub fn mining_system(world: &mut World) {
    // 1. Collect all Pop positions
    let pop_positions: Vec<GridPosition> = world
        .query::<(&Pop, &GridPosition)>()
        .iter(world)
        .map(|(_, pos)| *pos)
        .collect();

    // 2. Iterate designations
    // We collect entities to modify/despawn to avoid borrow checker issues
    let mut completed_mines = Vec::new();
    let mut invalid_mines = Vec::new();
    let mut progress_updates = Vec::new();

    let mut query = world.query::<(Entity, &Designation, &GridPosition, &mut WorkProgress)>();
    let terrain = world.resource::<TerrainGrid>();

    for (entity, designation, pos, mut progress) in query.iter_mut(world) {
        if designation.designation_type != DesignationType::Mine {
            continue;
        }

        // Validate Terrain
        match terrain.get(pos.x as usize, pos.y as usize) {
            Some(TerrainType::Rock) => {
                // Check if a pop is here
                if pop_positions.contains(pos) {
                    progress.current += MINING_SPEED;
                    if progress.current >= progress.required {
                        completed_mines.push((entity, *pos));
                    }
                }
            }
            _ => {
                // Not a rock, cancel designation
                invalid_mines.push(entity);
            }
        }
    }

    // 3. Apply updates

    // Remove invalid
    for entity in invalid_mines {
        world.despawn(entity);
    }

    // Process completions
    if !completed_mines.is_empty() {
        let mut terrain = world.resource_mut::<TerrainGrid>();
        let mut resources = world.resource_mut::<ColonyResources>();

        for (entity, pos) in completed_mines {
            // Update Terrain
            if let Some(idx) = (pos.y as usize * terrain.width + pos.x as usize).into() {
                 terrain.tiles[idx] = TerrainType::Dirt; // or Dirt/Floor
            }

            // Add Resources
            resources.stone += STONE_PER_ROCK;

            // Remove Designation
            world.despawn(entity);
        }
    }
}
```

### Integration in `try_designate`

Update `src/layer1/designation.rs` to add `WorkProgress` when spawning a `Mine` designation.

```rust
// src/layer1/designation.rs

pub fn try_designate(world: &mut World, x: i32, y: i32, designation_type: DesignationType) -> bool {
    // ... existing checks ...

    let mut entity = world.spawn((
        Designation { designation_type },
        GridPosition { x, y },
    ));

    // Initialize progress based on type
    match designation_type {
        DesignationType::Mine => {
            entity.insert(crate::layer1::gathering::WorkProgress {
                current: 0.0,
                required: 10.0, // Hard to mine
            });
        }
        DesignationType::Demolish => {
             entity.insert(crate::layer1::gathering::WorkProgress {
                current: 0.0,
                required: 5.0, // Easier to demolish
            });
        }
    }

    true
}
```

## REFACTOR Phase: Quality & Design

After tests pass, consider these improvements:

### Code Smells to Address Later

1.  **Direct Terrain Mutation**: `mining_system` modifies `TerrainGrid.tiles` directly.
    -   Future: `TerrainGrid::set_tile` method to handle bounds checking and potential side effects.
    -   Current: Direct index access is fast but unsafe if bounds aren't checked (though `get` checked them).

2.  **Pop Lookup O(N*M)**: Searching `pop_positions` vector for every designation.
    -   Future: `Grid` or `HashMap` for spatial lookup of pops.
    -   Current: N and M are small (<100).

3.  **Hardcoded Values**: `MINING_SPEED`, `required` progress.
    -   Future: Config/Constants file.

4.  **ColonyResources Location**: It's in `layer2/farm.rs` but used in `layer1`.
    -   Refactor: Move `ColonyResources` to `src/shared/resources.rs` or `src/layer1/resources.rs` to respect layering (Layer 1 shouldn't depend on Layer 2, technically `farm` is L2 but resources are global).
    -   Correction: `Farm` is listed as Layer 2 in file structure, but Layer 1 is "Colony Simulation". If `Farm` is in `layer2` folder, then `layer1` cannot import it.
    -   **CRITICAL ARCHITECTURE CHECK**: `src/layer1/` vs `src/layer2/`.
    -   If `mining` is `layer1`, it cannot import `layer2::farm`.
    -   **Solution**: `ColonyResources` MUST be moved to `layer1` or `shared`.
    -   **Proposal**: Move `ColonyResources` to `src/layer1/resources.rs` and update `farm.rs` to use it.

### Layering Correction (Refactor Step)

**Move** `ColonyResources` struct to `src/layer1/resources.rs`.
**Update** `src/layer2/farm.rs` to `use crate::layer1::resources::ColonyResources;`.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `ColonyResources` contains `wood` and `stone`
- [ ] `mining_system` increases stone count upon completion
- [ ] `mining_system` changes Rock to Dirt
- [ ] `mining_system` removes the Designation entity
- [ ] `try_designate` adds `WorkProgress` component automatically
- [ ] Test coverage ≥85% for `layer1/gathering.rs`

## Technical Guidance

### System Execution Order
Run `mining_system` BEFORE `kill_starving_pops_system` but AFTER `utility_ai`.

```rust
// main.rs
mining_system(&mut world);
```

### Module Structure
```
src/layer1/
  gathering.rs
  resources.rs (New)
src/layer2/
  farm.rs (Update imports)
```

## Questions

*Builder: add questions here if spec is unclear.*
