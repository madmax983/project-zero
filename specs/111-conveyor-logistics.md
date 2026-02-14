# 111: Conveyor Logistics

## 1. Overview

Introduces automated logistics to Layer 1. Players can construct **Conveyor Belts** to transport `ResourceItem` entities and **Hoppers** to automatically collect items into the global `ColonyResources` inventory.

This feature enables "Factorio-lite" gameplay where mining outputs can be routed to storage without manual hauling by Pops.

### New Buildings
- **Conveyor Belt**: Moves items on its tile to the next tile in its facing direction. Requires power.
- **Hopper**: Consumes items on its tile and adds them to `ColonyResources`. Requires power.

## 2. Dependencies

- `specs/014-resource-items.md` (Items exist as entities)
- `specs/042-energy-system.md` (Power grid)

## 3. RED Phase: Tests First

```rust
// src/layer1/logistics_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType, Direction};
    use crate::layer1::energy::{PowerConsumer, PowerSource};
    use crate::layer1::items::Item;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::{ColonyResources, ResourceItem, ResourceType};
    use crate::layer1::logistics::{ConveyorBelt, Hopper, conveyor_system, hopper_system};

    #[test]
    fn test_conveyor_moves_item() {
        let mut world = World::new();

        // Setup Conveyor at (0,0) facing East (Active)
        world.spawn((
            Building { building_type: BuildingType::ConveyorBelt },
            ConveyorBelt { direction: Direction::East, speed: 1.0 },
            GridPosition { x: 0, y: 0 },
            PowerConsumer { demand: 1.0, active: true },
        ));

        // Spawn Item at (0,0)
        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Stone, amount: 1.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run system
        conveyor_system(&mut world);

        // Item should move to (1,0)
        let pos = world.get::<GridPosition>(item).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_conveyor_needs_power() {
        let mut world = World::new();

        // Inactive Conveyor
        world.spawn((
            Building { building_type: BuildingType::ConveyorBelt },
            ConveyorBelt { direction: Direction::East, speed: 1.0 },
            GridPosition { x: 0, y: 0 },
            PowerConsumer { demand: 1.0, active: false }, // No power
        ));

        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Stone, amount: 1.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        conveyor_system(&mut world);

        // Item should NOT move
        let pos = world.get::<GridPosition>(item).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_hopper_consumes_item() {
        let mut world = World::new();
        world.insert_resource(ColonyResources::default()); // Stone = 5.0

        // Active Hopper at (0,0)
        world.spawn((
            Building { building_type: BuildingType::Hopper },
            Hopper,
            GridPosition { x: 0, y: 0 },
            PowerConsumer { demand: 5.0, active: true },
        ));

        // Item on Hopper
        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Stone, amount: 5.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        hopper_system(&mut world);

        // Item should be despawned
        assert!(world.get_entity(item).is_err());

        // Resources should increase (5.0 + 5.0 = 10.0)
        let res = world.resource::<ColonyResources>();
        assert!((res.stone - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hopper_respects_cap() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.max_stone = 10.0;
        res.stone = 9.0;
        world.insert_resource(res);

        // Active Hopper
        world.spawn((
            Building { building_type: BuildingType::Hopper },
            Hopper,
            GridPosition { x: 0, y: 0 },
            PowerConsumer { demand: 5.0, active: true },
        ));

        // Item with amount 5.0
        let item = world.spawn((
            ResourceItem { resource_type: ResourceType::Stone, amount: 5.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        hopper_system(&mut world);

        // Should partial consume?
        // MVP: Simple "all or nothing" or "partial consume"?
        // Let's assume partial consume is better for logic, but "all or nothing" is easier.
        // If "all or nothing", item stays.
        // If "partial", item amount reduces.

        // Spec Decision: Partial consume supported.

        let item_comp = world.get::<ResourceItem>(item).unwrap();
        assert!((item_comp.amount - 4.0).abs() < f32::EPSILON); // 9 + 1 = 10 (Max). Remaining 4.

        let res = world.resource::<ColonyResources>();
        assert!((res.stone - 10.0).abs() < f32::EPSILON);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

### Components

```rust
// src/layer1/logistics.rs

#[derive(Component, Debug, Clone)]
pub struct ConveyorBelt {
    pub direction: crate::layer1::building::Direction,
    pub speed: f32, // Moves per tick (usually 1.0 or 0.5)
}

#[derive(Component, Debug, Clone)]
pub struct Hopper; // Marker
```

### Systems

```rust
// src/layer1/logistics.rs

pub fn conveyor_system(world: &mut World) {
    // 1. Collect all active belts
    // 2. Query all ResourceItems
    // 3. Match items to belts by GridPosition
    // 4. Update item GridPosition based on belt direction
    // 5. Handle collision (optional for MVP: Items can stack)
}

pub fn hopper_system(world: &mut World) {
    // 1. Collect active hoppers
    // 2. Query items at hopper positions
    // 3. Try add item amount to ColonyResources
    // 4. If fully added, despawn item. Else update amount.
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance**: Spatial hashing (GridMap) is crucial. Iterating all items vs all belts is O(N*M). Use `GridMap` to lookup belt at item position.
- **Visuals**: Conveyor belts need animation (texture scrolling). Items should smoothly interpolate between tiles (client-side only).
- **Collision**: Items shouldn't move if target tile is blocked (e.g., wall, full machine).
- **Direction**: Belts need to handle corners visually. Logic handles direction per tile.

## 6. Acceptance Criteria

- [ ] `ConveyorBelt` and `Hopper` components created.
- [ ] `conveyor_system` moves items in correct direction.
- [ ] `hopper_system` consumes items into global resources.
- [ ] Systems check `PowerConsumer.active`.
- [ ] `ColonyResources` cap is respected (partial consumption).
- [ ] Tests pass.

## 7. Technical Guidance

- Use `crate::layer1::building::Direction` for belt orientation.
- Be careful with `GridPosition` updates. Ensure target position is within map bounds.
- Use `try_add` logic on `ColonyResources` (needs new method? or manual check). `add_resource` clamps, so you need to check `max - current` to know how much *can* be added.

## 8. Questions

- *Builder*: Should belts move Pops?
    - *Architect*: No, only `ResourceItem` entities for now.
- *Builder*: Do items fall off the end?
    - *Architect*: No, they stay on the last tile if no valid move exists.
