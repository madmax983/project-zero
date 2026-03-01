# 023: Refining Industry

## Overview

Introduce secondary resource processing to the colony. Raw resources (Wood, Stone) are abundant but primitive. To advance, the colony must refine them into construction materials (Planks, Blocks).

This spec introduces:
1.  **New Resources**: `planks` and `blocks` (and their caps).
2.  **New Buildings**: `LumberMill` (converts Wood -> Planks) and `StoneMason` (converts Stone -> Blocks).
3.  **Refining Logic**: A work process similar to mining/forestry where colonists convert resources over time.

## Dependencies

- `006` — Building Placement (for `BuildingType`)
- `018` — Mining and Resources (for `ColonyResources` and work patterns)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/refining_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::refining::{RefiningProgress, process_refining_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::GridPosition;

    #[test]
    fn test_colony_resources_refined_fields() {
        let resources = ColonyResources::default();
        // New fields
        assert_eq!(resources.planks, 0.0);
        assert_eq!(resources.blocks, 0.0);
        // Default caps (can be same as raw for now)
        assert_eq!(resources.max_planks, 50.0);
        assert_eq!(resources.max_blocks, 20.0);
    }

    #[test]
    fn test_building_type_variants() {
        let lm = BuildingType::LumberMill;
        let sm = BuildingType::StoneMason;
        assert_eq!(lm.label(), "Lumber Mill");
        assert_eq!(sm.label(), "Stone Mason");
    }

    #[test]
    fn test_refining_progress_component() {
        let progress = RefiningProgress {
            current: 0.0,
            max: 100.0,
        };
        assert!(!progress.is_complete());
    }

    #[test]
    fn test_process_refining_lumber_mill() {
        let mut world = World::new();

        // Setup Resources: Has Wood, No Planks
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.planks = 0.0;
        world.insert_resource(resources);

        // Spawn Lumber Mill at (5, 5)
        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 0.0, max: 10.0 }, // 10 ticks to refine
        ));

        // Spawn Worker nearby at (5, 6)
        world.spawn((
            Pop,
            GridPosition { x: 5, y: 6 },
        ));

        // Run system
        // 1. Should detect worker
        // 2. Should detect valid input (Wood > 0)
        // 3. Should increment progress
        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!(progress.current > 0.0);
    }

    #[test]
    fn test_process_refining_consumes_input_on_complete() {
        let mut world = World::new();

        // Setup Resources
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.planks = 0.0;
        world.insert_resource(resources);

        // Spawn Lumber Mill almost done
        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 9.9, max: 10.0 },
        ));

        // Spawn Worker
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system to complete
        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Input consumed
        assert!((res.wood - 9.0).abs() < f32::EPSILON); // 10.0 - 1.0 = 9.0
        // Output produced
        assert!((res.planks - 1.0).abs() < f32::EPSILON); // 0.0 + 1.0 = 1.0

        // Progress reset
        let progress = world.query::<&RefiningProgress>().single(&world);
        assert!(progress.current < 1.0); // Should wrap or reset to 0
    }

    #[test]
    fn test_refining_stops_if_no_input() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 0.0; // No wood
        world.insert_resource(resources);

        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 5, y: 5 },
            RefiningProgress::default(),
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 5 }));

        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert_eq!(progress.current, 0.0);
    }

    #[test]
    fn test_refining_stops_if_output_full() {
        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 10.0;
        resources.planks = 50.0; // Full
        resources.max_planks = 50.0;
        world.insert_resource(resources);

        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 5, y: 5 },
            RefiningProgress::default(),
        ));
        world.spawn((Pop, GridPosition { x: 5, y: 5 }));

        process_refining_system(&mut world);

        let progress = world.query::<&RefiningProgress>().single(&world);
        assert_eq!(progress.current, 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update ColonyResources

```rust
// src/layer1/resources.rs

pub struct ColonyResources {
    // ... existing fields ...
    pub planks: f32,
    pub blocks: f32,
    pub max_planks: f32,
    pub max_blocks: f32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            // ... existing ...
            planks: 0.0,
            blocks: 0.0,
            max_planks: 50.0,
            max_blocks: 20.0,
        }
    }
}

// Add helper methods add_planks(), add_blocks()
```

### 2. Update BuildingType

```rust
// src/layer1/building.rs

pub enum BuildingType {
    Housing,
    Farm,
    Stockpile,
    LumberMill, // New
    StoneMason, // New
}

impl BuildingType {
    pub const fn char(&self) -> char {
        match self {
            Self::LumberMill => 'L',
            Self::StoneMason => 'M',
            // ...
        }
    }

    // Add costs:
    // LumberMill: 30 Wood, 10 Stone
    // StoneMason: 40 Wood, 20 Stone
}
```

### 3. Implement Refining Logic

Create new module `src/layer1/refining.rs`.

```rust
// src/layer1/refining.rs

use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::GridPosition;
use crate::layer1::pop::Pop;

#[derive(Component, Default, Debug)]
pub struct RefiningProgress {
    pub current: f32,
    pub max: f32,
}

impl RefiningProgress {
    pub fn is_complete(&self) -> bool {
        self.current >= self.max
    }
}

pub fn process_refining_system(world: &mut World) {
    // 1. Collect worker positions
    let worker_positions: Vec<GridPosition> = world
        .query::<(&Pop, &GridPosition)>()
        .iter(world)
        .map(|(_, pos)| *pos)
        .collect();

    // 2. Iterate buildings
    let mut buildings = world.query::<(Entity, &Building, &GridPosition, &mut RefiningProgress)>();

    // Note: To modify resources inside the loop, we might need to extract the loop logic or use a command buffer.
    // However, for simplicity in GREEN phase, we can collect updates and apply them after, or rely on internal mutability if possible (not in Rust).
    // Better: Iterate buildings, check conditions, determine deltas, apply to Resources.

    // Since we need to read AND write resources, and query components, we have to be careful with borrows.
    // Simpler approach:
    // a. Identify active buildings (workers nearby + input available + output space).
    // b. Apply progress.
    // c. If complete, apply resource transaction.

    // For this implementation, we will perform the checks inside the loop but access resources via `world.resource_mut` which might conflict with the query if not careful.
    // Bevy System param injection handles this safely. But here we are using raw `world` access.
    // We should split the function into two parts or use `unsafe` world cell? No.
    // Safe approach: Collect entities to update.

    let mut updates = Vec::new();
    let resources = world.resource::<ColonyResources>();

    for (entity, building, pos, progress) in buildings.iter(world) {
        // Check worker range (manhattan distance <= 10, similar to mining)
        let has_worker = worker_positions.iter().any(|p| (p.x - pos.x).abs() + (p.y - pos.y).abs() <= 10);

        if !has_worker { continue; }

        // Check recipe
        let (can_refine, input_cost, output_gain) = match building.building_type {
            BuildingType::LumberMill => {
                (resources.wood >= 1.0 && resources.planks < resources.max_planks,
                 ColonyResources { wood: 1.0, ..Default::default() },
                 ColonyResources { planks: 1.0, ..Default::default() })
            },
            BuildingType::StoneMason => {
                (resources.stone >= 1.0 && resources.blocks < resources.max_blocks,
                 ColonyResources { stone: 1.0, ..Default::default() },
                 ColonyResources { blocks: 1.0, ..Default::default() })
            },
            _ => (false, ColonyResources::default(), ColonyResources::default()),
        };

        if can_refine {
            updates.push((entity, 1.0, input_cost, output_gain));
        }
    }

    // Apply updates
    let mut resources = world.resource_mut::<ColonyResources>();
    for (entity, work, input, output) in updates {
         if let Some(mut progress) = world.get_mut::<RefiningProgress>(entity) {
             progress.current += work;
             if progress.is_complete() {
                 progress.current = 0.0;
                 // Deduct input
                 resources.deduct(&input);
                 // Add output
                 resources.add_planks(output.planks);
                 resources.add_blocks(output.blocks);
             }
         }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Recipe System**: Hardcoding `LumberMill` -> `Wood` is brittle. Create a `RefiningRecipe` struct or trait?
- **Progress Speed**: Currently hardcoded `1.0`. Should depend on Pop skill or Building efficiency.
- **Resource Locking**: Currently we check input *before* work, but deduct *after*. If wood runs out mid-job? (Acceptable for MVP).
- **Visuals**: Add smoke/animation when active?

## Acceptance Criteria

- [ ] `ColonyResources` has `planks` and `blocks`.
- [ ] `BuildingType` includes `LumberMill` and `StoneMason`.
- [ ] Pops within range cause refining progress.
- [ ] Input is consumed and output produced only upon completion.
- [ ] Process stops if input missing or output full.
- [ ] Tests pass.

## Technical Guidance

- Ensure `update_resource_caps_system` (Spec 022) is updated if Stockpiles should also store Planks/Blocks (optional for this spec, but good to keep in mind).
- Don't forget to register the new system in `main.rs`.
- Use `ColonyResources::can_afford` for the check logic if possible, but we need to check caps too.

## Questions

*Builder: Should the "10 tile range" be a constant shared with Mining?*
*Architect: Refining range checks use a shared constant in `layer1::constants`.*
