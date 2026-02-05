# 025: Hauling Logistics

## Overview

Transition the economy from a "magic global pool" to a physical logistics system. Harvested resources (Wood, Stone, Ore) now drop as `ResourceItem` entities on the map. They must be hauled by Pops to a `Stockpile` to be added to the global `ColonyResources` pool and become usable for construction.

This adds:
1.  **Physicality**: Resources exist in the world.
2.  **Labor Cost**: Moving resources takes time and effort.
3.  **Traffic**: Stockpile placement becomes strategic.

*Note: For this MVP, Food remains in the global pool to prevent starvation death spirals. Food logistics will be handled in a future spec.*

## Dependencies

- `016` — Utility AI System (for `ActionType` and evaluation)
- `022` — Resource Stockpiles (for `Stockpile` building)
- `018` — Mining (to be updated)
- `019` — Forestry (to be updated)

## RED Phase: Tests First

Write these tests in `src/layer1/hauling_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::utility_ai::{ActionType, UtilityWeights, evaluate_haul, PopAction};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::stockpile::Stockpile;
    use crate::layer1::{GridPosition, Pop, SimulationTime};

    #[test]
    fn test_resource_item_component() {
        let item = ResourceItem {
            resource_type: ResourceType::Wood,
            amount: 1.0,
        };
        assert_eq!(item.resource_type, ResourceType::Wood);
        assert_eq!(item.amount, 1.0);
    }

    #[test]
    fn test_evaluate_haul_finds_item_and_stockpile() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Spawn ResourceItem
        let item_entity = world.spawn((
            ResourceItem { resource_type: ResourceType::Stone, amount: 1.0 },
            GridPosition { x: 5, y: 0 },
        )).id();

        // Spawn Stockpile with capacity
        let stockpile_entity = world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stockpile::default(), // Assume default has space
            GridPosition { x: 10, y: 0 },
        )).id();

        let items = world.query::<(Entity, &GridPosition, &ResourceItem)>();
        let stockpiles = world.query::<(Entity, &GridPosition, &Stockpile)>();

        let result = evaluate_haul(&pop_pos, &weights, &items, &stockpiles);

        assert!(result.is_some());
        let (utility, target) = result.unwrap();
        assert_eq!(target, item_entity); // Should target the item first
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_haul_ignores_full_stockpiles() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Fill global resources to max
        let mut resources = ColonyResources::default();
        resources.stone = resources.max_stone;
        world.insert_resource(resources);

        // Spawn Item
        world.spawn((
            ResourceItem { resource_type: ResourceType::Stone, amount: 1.0 },
            GridPosition { x: 5, y: 0 },
        ));

        // Spawn Stockpile
        world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stockpile::default(),
            GridPosition { x: 10, y: 0 },
        ));

        let items = world.query::<(Entity, &GridPosition, &ResourceItem)>();
        let stockpiles = world.query::<(Entity, &GridPosition, &Stockpile)>();

        // Should return None because global storage is full
        // (Note: This logic assumes Stockpiles just extend global cap,
        //  so "Stockpile Full" means "Global Cap Reached" in this abstraction)
        let result = evaluate_haul(&pop_pos, &weights, &items, &stockpiles);

        // OR: If we track per-stockpile inventory in future, check that.
        // For MVP (Spec 022 model): Stockpiles increase CAP.
        // So we check ColonyResources::current < ColonyResources::max.

        // Since we filled global resources, we expect None.
        assert!(result.is_none());
    }

    #[test]
    fn test_haul_action_lifecycle() {
        // This integration test simulates the full loop
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(ColonyResources::default());

        // 1. Spawn Pop, Item, Stockpile
        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            PopAction { current: ActionType::Haul, ..Default::default() },
            // Need a "Carrying" component?
        )).id();

        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Wood, amount: 10.0 },
            GridPosition { x: 2, y: 0 },
        )).id();

        let stockpile = world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stockpile::default(),
            GridPosition { x: 4, y: 0 },
        )).id();

        // 2. Run haul_system (pickup phase)
        // Assume pop moves to item (simulated here by teleport)
        *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 2, y: 0 };

        // System should detect overlap, pick up item (add Carrying, despawn Item)
        // run_haul_system(&mut world);
        // assert!(world.get::<Carrying>(pop).is_some());
        // assert!(world.get_entity(item).is_none());

        // 3. Run haul_system (drop phase)
        // Teleport to stockpile
        // *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 4, y: 0 };
        // run_haul_system(&mut world);

        // 4. Verify resources added
        // let res = world.resource::<ColonyResources>();
        // assert_eq!(res.wood, 10.0);
        // assert!(world.get::<Carrying>(pop).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components

```rust
// src/layer1/resources.rs

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceType {
    Food,
    Wood,
    Stone,
    Ore,
    Metal,
    Planks,
    Blocks,
}

#[derive(Component, Debug)]
pub struct ResourceItem {
    pub resource_type: ResourceType,
    pub amount: f32,
}

// New component for Pops
#[derive(Component, Debug)]
pub struct Carrying {
    pub resource_type: ResourceType,
    pub amount: f32,
}
```

### 2. Update ActionType

```rust
// src/layer1/utility_ai.rs
pub enum ActionType {
    // ...
    Haul,
}
```

### 3. Implement Evaluation

```rust
// src/layer1/utility_ai.rs

pub fn evaluate_haul(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    items: &Query<(Entity, &GridPosition, &ResourceItem)>,
    stockpiles: &Query<(Entity, &GridPosition, &Stockpile)>,
    // Need global resources to check caps
) -> Option<(f32, Entity)> {
    // 1. Check if we have space (global check for MVP)
    // If Global Resource Full, Utility = 0.

    // 2. Find closest item
    // ... logic similar to evaluate_work ...

    // 3. Return utility
    // Utility depends on "Colony Need" (empty stockpiles = high urgency?)
    // For MVP: Constant base utility or slightly higher than Idle.
}
```

### 4. Implement Haul System

This is the "Operator" system that executes the action.

```rust
// src/layer1/hauling.rs

pub fn haul_system(world: &mut World) {
    // Iterate Pops with ActionType::Haul

    // State 1: Not Carrying -> Move to Item
    // If adjacent/at item -> Pick up (Add Carrying, Despawn Item)

    // State 2: Carrying -> Move to Stockpile
    // Find closest Stockpile
    // If adjacent/at stockpile -> Drop (Remove Carrying, Add to ColonyResources)
}
```

## REFACTOR Phase: Quality & Design

- **Update Generators**:
    - Modify `mine_rock` (Spec 018) to spawn `ResourceItem` instead of calling `resources.add_stone()`.
    - Modify `chop_tree` (Spec 019) to spawn `ResourceItem` instead of calling `resources.add_wood()`.
    - **CRITICAL**: Do NOT update Farm yet (keep Food magic).
- **Visualization**: `ResourceItem` needs a render representation (e.g., small char `.` or color variation). Update `src/ui/map.rs`.
- **Race Conditions**: Two pops targeting same item. First one despawns it, second one should re-evaluate. `haul_system` must handle invalid target entities gracefully.

## Acceptance Criteria

- [ ] `ResourceItem` and `Carrying` components exist.
- [ ] `mine_rock` drops `ResourceItem(Stone)` instead of auto-crediting.
- [ ] `chop_tree` drops `ResourceItem(Wood)` instead of auto-crediting.
- [ ] `ColonyResources` **only** increases when items are delivered to Stockpile.
- [ ] Pops pick up items, carry them, and drop them.
- [ ] `Carrying` is visualized (e.g., Pop char changes or status text).
- [ ] Tests pass.

## Technical Guidance

- **Inventory**: `Carrying` is a simple inventory. If we want multi-item carrying later, we'll need a `Vec<Item>`. For now, 1 item type at a time.
- **Stockpile Validation**: Ensure the stockpile building is actually built (completed) before allowing drop-off? Spec 020 says buildings have construction time. Stockpiles might be instant for MVP, but good to check.
- **UI**: In `render_map`, items should be drawn. If a Pop is on top, draw Pop. If Item on top of Terrain, draw Item. Z-order: Terrain < Item < Pop.
