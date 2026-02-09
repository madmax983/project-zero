# 066: Building Work AI Integration

## Overview

Currently, the **Refining** (`process_refining_system`) and **Farming** (`produce_food_system`) mechanics use legacy "proximity" or "assigned worker list" logic that bypasses the core **Utility AI** decision loop. This creates several issues:
1.  Pops "work" simply by standing near a building, even if their `PopAction` is `Idle` or `SatisfyHunger`.
2.  Work cannot be prioritized against other needs (Hunger, Rest) effectively.
3.  The codebase has fragmented logic for "How work happens".

This specification integrates these systems into the Utility AI by introducing `ActionType::Refine` and `ActionType::Farm`.

## Dependencies

- `016` — Utility AI System (for `ActionType`, `evaluate_actions_system`)
- `023` — Refining Industry (for `RefiningProgress`)
- `008` — Farm Building (for `Farm`)

## RED Phase: Tests First

Write these tests in `src/layer1/actions/work_building_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::utility_ai::{ActionType, UtilityWeights, evaluate_actions_system, PopAction};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::farm::Farm;
    use crate::layer1::refining::get_refining_recipe; // Ensure this is public
    use crate::layer1::resources::{ColonyResources, RefiningProgress};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use bevy_ecs::system::RunSystemOnce;

    // Helper to evaluate refine
    use crate::layer1::actions::refine::evaluate_refine;
    // Helper to evaluate farm
    use crate::layer1::actions::farm::evaluate_farm;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_evaluate_refine_finds_valid_work() {
        let mut world = setup_world();

        // Add resources for input (Wood for Lumber Mill)
        world.resource_mut::<ColonyResources>().wood = 10.0;

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Lumber Mill
        let mill = world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 2, y: 0 },
            RefiningProgress { current: 0.0, max: 10.0 },
        )).id();

        let buildings = world.query::<(Entity, &GridPosition, &Building, &RefiningProgress)>();
        let resources = world.resource::<ColonyResources>();

        let result = evaluate_refine(&pop_pos, &weights, resources, buildings.iter(&world));

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, mill);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_refine_ignores_unaffordable_recipe() {
        let mut world = setup_world();
        // No wood!
        world.resource_mut::<ColonyResources>().wood = 0.0;

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 2, y: 0 },
            RefiningProgress::default(),
        ));

        let buildings = world.query::<(Entity, &GridPosition, &Building, &RefiningProgress)>();
        let resources = world.resource::<ColonyResources>();

        let result = evaluate_refine(&pop_pos, &weights, resources, buildings.iter(&world));
        assert!(result.is_none(), "Should not refine if inputs are missing");
    }

    #[test]
    fn test_evaluate_farm_finds_work() {
        let mut world = setup_world();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Farm
        let farm = world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 2, y: 0 },
            Farm { capacity: 1, workers: vec![] }, // Workers list might be deprecated/changed
        )).id();

        let farms = world.query::<(Entity, &GridPosition, &Farm)>();

        let result = evaluate_farm(&pop_pos, &weights, farms.iter(&world));

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, farm);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_farm_respects_capacity() {
        let mut world = setup_world();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn Full Farm (manually filling workers for test)
        // Note: With dynamic AI, capacity check might need "Active Workers" count or "Claim" system.
        // For this test, we assume Farm struct still tracks workers or we check proximity count.
        // If we strictly follow Utility AI, we check `PopAction` of others.
        // MVP: Assume Farm struct has `workers` list that is updated when action starts.
        let worker = world.spawn(Pop).id();
        world.spawn((
            Building { building_type: BuildingType::Farm },
            GridPosition { x: 2, y: 0 },
            Farm { capacity: 1, workers: vec![worker] },
        ));

        let farms = world.query::<(Entity, &GridPosition, &Farm)>();

        let result = evaluate_farm(&pop_pos, &weights, farms.iter(&world));
        assert!(result.is_none(), "Should not target full farm");
    }

    #[test]
    fn test_refining_system_requires_correct_action() {
        // Migration verification
        // Previously process_refining_system checked proximity.
        // Now it should check if the worker has ActionType::Refine AND is at target.
        // This is an integration test for the MODIFIED system.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ActionType`

Add `Refine` and `Farm` to `ActionType` enum in `src/layer1/utility_ai/types.rs`.

```rust
pub enum ActionType {
    // ...
    Refine,
    Farm,
}
```

### 2. Implement `evaluate_refine`

Create `src/layer1/actions/refine.rs`.

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use crate::layer1::utility_ai::math::{calculate_context_score, calculate_success_modifier};
use crate::layer1::building::Building;
use crate::layer1::resources::{ColonyResources, RefiningProgress};
use crate::layer1::refining::get_refining_recipe;

pub fn evaluate_refine<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    buildings: impl Iterator<Item = (Entity, &'a GridPosition, &'a Building, &'a RefiningProgress)>,
) -> Option<(f32, Entity)> {
    let mut best = None;
    let base_utility = 0.5;

    for (entity, pos, building, progress) in buildings {
        // Check recipe validity
        let (can_afford, _, _) = get_refining_recipe(building.building_type, resources);

        // Allow work if can afford OR if work is already in progress (inputs consumed)
        // But get_refining_recipe checks inputs.
        // RefiningProgress > 0 means inputs consumed?
        // In current logic, inputs are consumed at END.
        // So we MUST check `can_afford`.
        if !can_afford {
             continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            1, // Capacity assumption
            0, // Occupied assumption (needs improvement)
            weights
        );

        let success = calculate_success_modifier(ActionType::Refine, weights);
        let utility = base_utility * context * success;

        if best.is_none() || utility > best.unwrap().0 {
            best = Some((utility, entity));
        }
    }
    best
}
```

### 3. Implement `evaluate_farm`

Create `src/layer1/actions/farm.rs`.

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::utility_ai::{ActionType, UtilityWeights};
use crate::layer1::utility_ai::math::{calculate_context_score, calculate_success_modifier};
use crate::layer1::farm::Farm;

pub fn evaluate_farm<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    farms: impl Iterator<Item = (Entity, &'a GridPosition, &'a Farm)>,
) -> Option<(f32, Entity)> {
    let mut best = None;
    let base_utility = 0.5; // Food is important, maybe boost?

    for (entity, pos, farm) in farms {
        if farm.workers.len() >= farm.capacity {
            continue;
        }

        let context = calculate_context_score(
            *pop_pos,
            Some(*pos),
            farm.capacity as u32,
            farm.workers.len() as u32,
            weights
        );

        let success = calculate_success_modifier(ActionType::Farm, weights);
        let utility = base_utility * context * success;

        if best.is_none() || utility > best.unwrap().0 {
            best = Some((utility, entity));
        }
    }
    best
}
```

### 4. Update `evaluate_actions_system`

Add calls to `evaluate_refine` and `evaluate_farm`.

### 5. Update Execution Systems

**Refining**: Modify `process_refining_system` in `src/layer1/refining.rs`.
- DO NOT search for nearest worker.
- Instead, query Pops with `ActionType::Refine` and check if `PopAction.target` matches the building.
- Check if Pop is at `GridPosition`.
- Apply progress.

**Farming**: Modify `produce_food_system` in `src/layer1/farm.rs`.
- DO NOT rely on `farm.workers` list for persistent assignment (unless we update it).
- Query Pops with `ActionType::Farm`.
- Check if Pop is at `GridPosition`.
- Accumulate efficiency.
- **Note**: `produce_food_system` currently iterates *Farms*. It should probably iterate *Workers* to calculate efficiency per worker, then apply to Farm?
- Or iterate Farms, find workers targeting it who are present.

## REFACTOR Phase: Quality & Design

- **Migration Cleanup**: Remove the legacy "proximity" logic entirely.
- **Worker List**: Decide if `Farm.workers` should be removed. If replaced by dynamic `PopAction` targeting, we can remove it to avoid sync bugs. But we need a way to count "Occupied" slots for utility score.
    - *Solution*: A system `update_building_occupancy` that counts how many pops are targeting each building and stores it in a transient component or just relies on the query count next tick.
    - For MVP, keep `Farm.workers` but update it when Pop starts/stops action (in `evaluate_actions_system` or a maintenance system).
- **Feedback**: Add "Working" icon/status in UI when refining.

## Acceptance Criteria

- [ ] `ActionType::Refine` and `ActionType::Farm` exist.
- [ ] Pops autonomously choose to Refine/Farm based on utility.
- [ ] Pops walk to the building before work starts.
- [ ] `process_refining_system` only progresses if a worker is present AND targeting the building.
- [ ] `produce_food_system` only produces if a worker is present AND targeting the building.
- [ ] Legacy proximity logic is removed.
- [ ] Tests pass.

## Technical Guidance

- Be careful with `Farm.workers` synchronization. If a pop switches action (e.g. to Eat), they must be removed from `Farm.workers`.
- Ideally, `Farm.workers` is *derived* data, rebuilt every tick or maintained by an observer system.
- Or, simpler for MVP: `produce_food_system` iterates Pops, checks `ActionType::Farm` + `AtPosition`, finds target Farm, adds progress.
