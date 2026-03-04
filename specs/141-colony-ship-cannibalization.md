# 141: Colony Ship Cannibalization

## 1. Overview

The "Lander" is the starting building for every colony. It provides initial shelter, power, and storage. However, it is also a treasure trove of high-tech materials.

This feature allows the player to "Cannibalize" the Lander. This is a destructive action that permanently removes the Lander but yields a massive amount of `Metal`, `Fuel`, and `Rations`.

**The Tension:** Sacrifice long-term utility (free power/housing/storage) for a short-term resource injection (survival/boom).

## 2. Dependencies

- [x] `006` Building Placement (BuildingType enum)
- [x] `018` Mining Resources (ColonyResources, ResourceItem)
- [x] `017` Designation System (DesignationType)

## 3. RED Phase: Tests First

```rust
// tests/layer1/cannibalization_tests.rs

use scale::layer1::building::{Building, BuildingType};
use scale::layer1::designation::{Designation, DesignationType};
use scale::layer1::resources::{ResourceItem, ResourceType};
use scale::layer1::GridPosition;
use bevy_ecs::prelude::*;

#[test]
fn test_lander_building_exists() {
    // Act
    let lander = BuildingType::Lander;

    // Assert
    assert_eq!(lander.label(), "Lander");
    // Should provide utility
    // We can't check components on Enum directly, but we can check spawn logic in Green phase test
}

#[test]
fn test_cannibalize_designation_exists() {
    let des = DesignationType::Cannibalize;
    assert_eq!(des.label(), "Cannibalize");
}

#[test]
fn test_execute_cannibalize_removes_lander_and_spawns_resources() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(scale::layer1::map::ScreenShake::default()); // Dependency
    world.insert_resource(scale::layer1::building::OccupiedTiles::default()); // Dependency
    world.insert_resource(scale::shared::log::MessageLog::default());

    let lander_pos = GridPosition { x: 10, y: 10 };
    let lander_entity = world.spawn((
        Building { building_type: BuildingType::Lander },
        lander_pos,
    )).id();

    // Mark tile as occupied so logic can find it if it uses OccupiedTiles (execution usually queries GridPosition)
    world.resource_mut::<scale::layer1::building::OccupiedTiles>().0.insert((10, 10));

    let designation = world.spawn((
        Designation { designation_type: DesignationType::Cannibalize },
        lander_pos,
    )).id();

    // Act
    // We assume a system or function `execute_cannibalize` handles this.
    // For TDD, we can expose it via `scale::layer1::execution::execute_cannibalize`
    let success = scale::layer1::execution::execute_cannibalize(&mut world, designation);

    // Assert
    assert!(success, "Cannibalization should succeed");
    assert!(world.get_entity(lander_entity).is_err(), "Lander should be despawned");
    assert!(world.get_entity(designation).is_err(), "Designation should be despawned");

    // Check Resources
    let items: Vec<&ResourceItem> = world.query::<&ResourceItem>().iter(&world).collect();

    // Expect Metal, Fuel, Rations
    let metal = items.iter().find(|i| i.resource_type == ResourceType::Metal).expect("Should yield Metal");
    let fuel = items.iter().find(|i| i.resource_type == ResourceType::Fuel).expect("Should yield Fuel");
    let rations = items.iter().find(|i| i.resource_type == ResourceType::Rations).expect("Should yield Rations");

    assert!(metal.amount >= 50.0);
    assert!(fuel.amount >= 20.0);
    assert!(rations.amount >= 20.0);
}

#[test]
fn test_cannibalize_only_works_on_lander() {
    let mut world = World::new();
    let pos = GridPosition { x: 5, y: 5 };
    world.insert_resource(scale::layer1::building::OccupiedTiles::default());

    // Spawn a House
    world.spawn((
        Building { building_type: BuildingType::Housing },
        pos,
    ));
    world.resource_mut::<scale::layer1::building::OccupiedTiles>().0.insert((5, 5));

    let designation = world.spawn((
        Designation { designation_type: DesignationType::Cannibalize },
        pos,
    )).id();

    // Act
    let success = scale::layer1::execution::execute_cannibalize(&mut world, designation);

    // Assert
    assert!(!success, "Should not cannibalize non-Lander");
}
```

## 4. GREEN Phase: Minimal Implementation

### `src/layer1/building.rs`
Add `Lander` to `BuildingType`.
In `spawn_building`:
```rust
BuildingType::Lander => {
    entity.insert((
        crate::layer1::housing::Housing { capacity: 5, ..Default::default() },
        crate::layer1::stockpile::Stockpile {
            food_bonus: 50.0,
            wood_bonus: 50.0,
            stone_bonus: 20.0,
            waste_bonus: 0.0,
        },
        crate::layer1::energy::PowerSource { output: 10.0, active: true },
        crate::layer1::lighting::LightSource {
            radius: 8.0,
            intensity: 0.8,
            color: (200, 200, 255)
        },
    ));
    // High HP
    if let Some(mut structure) = entity.get_mut::<crate::layer1::structure::Structure>() {
        structure.max_hp = 500.0;
        structure.current_hp = 500.0;
    }
}
```

### `src/layer1/designation.rs`
Add `Cannibalize` to `DesignationType`.
In `can_designate`:
```rust
DesignationType::Cannibalize => {
    // Check if tile has BuildingType::Lander
    // Requires querying world for Building at (x, y)
    world.iter_entities().any(|e| {
        if let Some(pos) = e.get::<GridPosition>()
            && pos.x == x
            && pos.y == y
        {
            if let Some(b) = e.get::<crate::layer1::building::Building>() {
                return b.building_type == crate::layer1::building::BuildingType::Lander;
            }
        }
        false
    })
}
```

### `src/layer1/execution.rs`
Implement `execute_cannibalize`.
```rust
pub fn execute_cannibalize(world: &mut World, designation_entity: Entity) -> bool {
    let Some(designation_pos) = world.get::<GridPosition>(designation_entity).copied() else {
        return false;
    };

    // Find building at this position
    let building_entity = world
        .query::<(Entity, &GridPosition, &crate::layer1::building::Building)>()
        .iter(world)
        .find(|(_, pos, b)| pos.x == designation_pos.x && pos.y == designation_pos.y && b.building_type == crate::layer1::building::BuildingType::Lander)
        .map(|(e, _, _)| e);

    if let Some(entity) = building_entity {
        // Spawn Resources
        spawn_resource_pile(world, designation_pos, crate::layer1::resources::ResourceType::Metal, 100.0);
        spawn_resource_pile(world, designation_pos, crate::layer1::resources::ResourceType::Fuel, 50.0);
        spawn_resource_pile(world, designation_pos, crate::layer1::resources::ResourceType::Rations, 50.0);

        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add_colored("Lander cannibalized! Massive resources gained.", ratatui::style::Color::Yellow);
        }

        // VFX
        crate::layer1::particles::spawn_particle(world, designation_pos, 'X', ratatui::style::Color::Red, 20);
        if let Some(mut shake) = world.get_resource_mut::<crate::layer1::map::ScreenShake>() {
            shake.trigger(0.8);
        }

        // Cleanup
        world.despawn(entity);
        if let Some(mut occupied) = world.get_resource_mut::<crate::layer1::building::OccupiedTiles>() {
            occupied.0.remove(&(designation_pos.x, designation_pos.y));
        }

        world.despawn(designation_entity);
        return true;
    }

    // If we are here, we didn't find a Lander (maybe destroyed already)
    // Clean up designation anyway
    world.despawn(designation_entity);
    false
}

fn spawn_resource_pile(world: &mut World, pos: GridPosition, res_type: crate::layer1::resources::ResourceType, amount: f32) {
    world.spawn((
        crate::layer1::resources::ResourceItem {
            resource_type: res_type,
            amount,
        },
        pos,
    ));
}
```
Wire into `execute_work_on_designation`.

## 5. REFACTOR Phase

*   Reuse `spawn_particle` from `execution.rs`.
*   Ensure `Lander` is spawned in map generation (or updated tests to spawn it).
*   Add `get_skill_for_designation` entry for `Cannibalize` (Construction).

## 6. Acceptance Criteria

- [ ] `BuildingType::Lander` exists.
- [ ] `DesignationType::Cannibalize` exists.
- [ ] `execute_cannibalize` destroys the Lander and spawns Metal, Fuel, and Rations.
- [ ] Attempts to cannibalize other buildings fail.
- [ ] Tests in `tests/layer1/cannibalization_tests.rs` pass.

## 7. Technical Guidance

*   **Yield Values**: 100 Metal, 50 Fuel, 50 Rations.
*   **BuildingType**: Ensure `Lander` is added to `BuildingType` enum.
*   **DesignationType**: Ensure `Cannibalize` is added to `DesignationType` enum.

## 8. Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
