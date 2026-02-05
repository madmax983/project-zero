# 025: Hauling Logistics

## Overview

Transition from "magic teleporting resources" to physical logistics. Mining and chopping now drop `ResourceItem` entities on the ground. Pops must perform the `Haul` action to move these items to a `Stockpile`, where they are credited to `ColonyResources`.

This introduces logistical bottlenecks: a distant mine requires more labor (haulers) to be effective than a close one.

## Dependencies

- `016` — Utility AI System (for `ActionType` and evaluation)
- `018` — Mining (to modify output)
- `019` — Forestry (to modify output)
- `022` — Stockpiles (as destinations)

## RED Phase: Tests First

Write these tests in `src/layer1/hauling_tests.rs` (or extend `resources.rs` tests).

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{ResourceItem, ResourceType};
    use crate::layer1::utility_ai::{ActionType, evaluate_haul, UtilityWeights};
    use crate::layer1::stockpile::Stockpile;
    use crate::layer1::pop::{GridPosition, Pop};
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_resource_item_component() {
        let item = ResourceItem {
            resource_type: ResourceType::Wood,
            amount: 10.0,
        };
        assert_eq!(item.resource_type, ResourceType::Wood);
        assert_eq!(item.amount, 10.0);
    }

    #[test]
    fn test_evaluate_haul_finds_item_and_stockpile() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        // Item on ground
        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Wood, amount: 10.0 },
            GridPosition { x: 2, y: 0 },
        )).id();

        // Stockpile
        let stockpile = world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stockpile::default(),
            GridPosition { x: 10, y: 0 },
        )).id();

        let items = world.query::<(Entity, &GridPosition, &ResourceItem)>();
        let stockpiles = world.query::<(Entity, &GridPosition, &Stockpile)>();

        let result = evaluate_haul(&pop_pos, &weights, &items, &stockpiles);

        assert!(result.is_some());
        let (utility, target_item, target_stockpile) = result.unwrap();

        assert_eq!(target_item, item);
        assert_eq!(target_stockpile, stockpile);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_evaluate_haul_no_stockpile() {
        let mut world = World::new();
        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();

        world.spawn((
            ResourceItem { resource_type: ResourceType::Wood, amount: 10.0 },
            GridPosition { x: 2, y: 0 },
        ));

        // No stockpiles

        let items = world.query::<(Entity, &GridPosition, &ResourceItem)>();
        let stockpiles = world.query::<(Entity, &GridPosition, &Stockpile)>();

        let result = evaluate_haul(&pop_pos, &weights, &items, &stockpiles);
        assert!(result.is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define ResourceItem and Type
```rust
// src/layer1/resources.rs

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceType {
    Wood,
    Stone,
    Ore,
    // Food is excluded for MVP stability (teleported by Farms)
}

#[derive(Component)]
pub struct ResourceItem {
    pub resource_type: ResourceType,
    pub amount: f32,
}
```

### 2. Update Mining/Forestry Output
Modify `mine_rock` (spec 018) and `chop_tree` (spec 019) to spawn entities instead of calling `resources.add_*`.

```rust
// src/layer1/resources.rs

pub fn spawn_resource_item(
    commands: &mut Commands,
    pos: GridPosition,
    resource_type: ResourceType,
    amount: f32,
) {
    commands.spawn((
        ResourceItem { resource_type, amount },
        pos,
    ));
}
```

### 3. Add Haul Action
```rust
// src/layer1/utility_ai.rs

pub enum ActionType {
    // ... existing
    Haul,
}
```

### 4. Implement Evaluation
```rust
// src/layer1/utility_ai.rs

pub fn evaluate_haul(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    items: &Query<(Entity, &GridPosition, &ResourceItem)>,
    stockpiles: &Query<(Entity, &GridPosition, &Stockpile)>,
) -> Option<(f32, Entity, Entity)> { // Returns (Utility, Item, Destination)

    let mut best: Option<(f32, Entity, Entity)> = None;

    for (item_entity, item_pos, _item) in items.iter() {
        // Find closest stockpile
        let mut closest_stockpile: Option<(Entity, i32)> = None;
        for (stock_entity, stock_pos, _) in stockpiles.iter() {
            let dist = manhattan_distance(item_pos, stock_pos);
            if closest_stockpile.is_none() || dist < closest_stockpile.unwrap().1 {
                closest_stockpile = Some((stock_entity, dist));
            }
        }

        if let Some((stock_entity, stock_dist)) = closest_stockpile {
            let dist_to_item = manhattan_distance(pop_pos, item_pos);
            let total_dist = dist_to_item + stock_dist;

            // Simple utility: closer = better
            let utility = 10.0 / (1.0 + total_dist as f32);

            if best.is_none() || utility > best.unwrap().0 {
                best = Some((utility, item_entity, stock_entity));
            }
        }
    }

    best
}
```

## REFACTOR Phase: Quality & Design

- **Carrier Capacity**: Pops should have a max carry capacity (default 10?). If item > capacity, take partial?
    - *Decision*: For MVP, Pops carry the full stack of one item entity.
- **Visuals**: Items need to be rendered in `src/ui/map.rs`.
    - Add helper: `get_resource_char(ResourceType)` and `get_resource_color(ResourceType)`.
- **Z-Order**: Ensure items render above terrain but below pops.
- **Reservation**: Prevent multiple pops from targeting the same item (future HTN task locking).

## Acceptance Criteria

- [ ] `ResourceItem` component exists.
- [ ] Mining/Forestry actions spawn `ResourceItem` entities instead of updating `ColonyResources` directly.
- [ ] `ColonyResources` does NOT increase immediately upon mining/chopping.
- [ ] `evaluate_haul` correctly identifies valid haul targets and destinations.
- [ ] Food production (Farms) is UNTOUCHED (still teleports resources).
- [ ] Tests pass with ≥85% coverage.

## Technical Guidance

- **Breaking Change**: This spec changes the behavior of `018` and `019`. Tests for those modules will need to be updated to assert "entity spawned" instead of "resource added".
- **Performance**: Nested loop in `evaluate_haul` (Items * Stockpiles) can be slow. Since stockpiles are few, it's O(I * S) which is roughly O(I). Fine for <1000 items.
- **Render Layer**: In `src/ui/map.rs`, `render_map` will need to query `ResourceItem` and draw them.

## Questions

*Builder: add questions here if spec is unclear.*
