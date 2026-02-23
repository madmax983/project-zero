# 210: Ammunition Logistics

## Overview

Shift turret defenses from consuming global resources to requiring physical **Ammo** items.
This introduces a logistical challenge: Turrets must be kept supplied by haulers. An empty turret is useless.
It adds depth to defense planning (supply lines) and crafting (Metal -> Ammo).

## Dependencies

- `043` — Defensive Structures (Turret base)
- `030` — Tool Economy (Item system)
- `025` — Hauling Logistics (Transporting items)

## RED Phase: Tests First

Write these tests in `src/layer1/turret_ammo_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::turret::{Turret, turret_fire_system};
    use crate::layer1::inventory::{Inventory, InventoryItem};
    use crate::layer1::items::ItemType;
    use crate::layer1::combat::{AttackProperties, CombatState};
    use crate::layer1::map::GridPosition;
    use crate::layer1::health::Health;
    use crate::layer1::fauna::{Fauna, FaunaType};

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup standard resources
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world
    }

    #[test]
    fn test_turret_fails_without_ammo_item() {
        let mut world = setup_world();

        // Spawn Turret with Inventory but NO items
        let turret = world.spawn((
            Turret {
                attack: AttackProperties { damage: 10.0, range: 5.0, cooldown: 0, accuracy: 1.0 },
                ammo_cost: 1.0,
                // We override ammo_type to require ItemType::Ammo in implementation
                // For now, assume Turret struct is updated to support ItemType requirement
            },
            Inventory::default(), // Empty
            GridPosition { x: 0, y: 0 },
            CombatState::default(),
            crate::layer1::building::Building { building_type: crate::layer1::building::BuildingType::MachineGunTurret }, // New Type
        )).id();

        // Spawn Enemy
        let enemy = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            GridPosition { x: 2, y: 0 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        turret_fire_system(&mut world);

        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 100.0, "Turret should not fire without ammo item");
    }

    #[test]
    fn test_turret_consumes_ammo_item() {
        let mut world = setup_world();

        // Spawn Turret with 1 Ammo
        let mut inventory = Inventory::default();
        inventory.add(InventoryItem { item_type: ItemType::Ammo });

        let turret = world.spawn((
            Turret {
                attack: AttackProperties { damage: 10.0, range: 5.0, cooldown: 10, accuracy: 1.0 },
                ammo_cost: 1.0,
            },
            inventory,
            GridPosition { x: 0, y: 0 },
            CombatState::default(),
            crate::layer1::building::Building { building_type: crate::layer1::building::BuildingType::MachineGunTurret },
        )).id();

        let enemy = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            GridPosition { x: 2, y: 0 },
            Health { current: 100.0, max: 100.0 },
        )).id();

        turret_fire_system(&mut world);

        // Check Health
        let health = world.get::<Health>(enemy).unwrap();
        assert_eq!(health.current, 90.0);

        // Check Inventory
        let inv = world.get::<Inventory>(turret).unwrap();
        assert!(inv.items.is_empty(), "Ammo should be consumed");
    }

    #[test]
    fn test_refill_job_creation_threshold() {
        // This tests the logic that creates hauling jobs for turrets
        // Logic likely resides in `utility_ai_work.rs` or similar.

        let mut world = setup_world();

        // Spawn Turret with 0 Ammo
        let turret = world.spawn((
            Turret::default(),
            Inventory::default(),
            GridPosition { x: 0, y: 0 },
            crate::layer1::building::Building { building_type: crate::layer1::building::BuildingType::MachineGunTurret },
        )).id();

        // Spawn Ammo in Stockpile
        let ammo = world.spawn((
            crate::layer1::items::Item { item_type: ItemType::Ammo },
            GridPosition { x: 5, y: 5 },
            crate::layer1::stockpile::StockpileItem,
        )).id();

        // Run job generation system (e.g., populate_hauling)
        // For unit test, we might test the helper function `get_turret_refill_targets`

        // let targets = get_turret_refill_targets(&world);
        // assert!(targets.contains(&turret));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `ItemType` (`src/layer1/items.rs`)

Add `Ammo` to the enum.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    // ...
    Ammo,
}
```

### 2. Update `BuildingType` (`src/layer1/building.rs`)

Add `MachineGunTurret`.

### 3. Update `Turret` Logic (`src/layer1/turret.rs`)

Modify `turret_fire_system` to check `Inventory` if the turret type requires items.
We can add a field to `Turret` struct `requires_item: Option<ItemType>`.

```rust
pub struct Turret {
    // ...
    pub required_item: Option<ItemType>, // If set, consumes item instead of resource
}

pub fn turret_fire_system(world: &mut World) {
    // ... inside loop ...

    // Check Ammo
    let has_ammo = if let Some(item_type) = turret_data.required_item {
        // Check Inventory component
        if let Some(inventory) = world.get::<Inventory>(turret_entity) {
            inventory.items.iter().any(|i| i.item_type == item_type)
        } else {
            false
        }
    } else {
        // Resource check (existing logic)
        let resources = world.resource::<ColonyResources>();
        // ...
    };

    if has_ammo {
        // ... find target ...
        if let Some(target) = best_target {
            // Consume Ammo
            if let Some(item_type) = turret_data.required_item {
                let mut inventory = world.get_mut::<Inventory>(turret_entity).unwrap();
                if let Some(pos) = inventory.items.iter().position(|i| i.item_type == item_type) {
                    inventory.items.remove(pos);
                }
            } else {
                // Consume Resource
            }
            // ... deal damage ...
        }
    }
}
```

### 4. Create Refill Job Logic (`src/layer1/logistics.rs`)

Implement logic to find turrets with `< Max` ammo (e.g. 5) and create hauling jobs.

```rust
pub fn get_turret_refill_candidates(world: &World) -> Vec<Entity> {
    let mut candidates = Vec::new();
    for (entity, turret, inventory) in world.query::<(Entity, &Turret, &Inventory)>().iter(world) {
        if let Some(ammo_type) = turret.required_item {
             let count = inventory.items.iter().filter(|i| i.item_type == ammo_type).count();
             if count < 5 { // Max capacity hardcoded for MVP
                 candidates.push(entity);
             }
        }
    }
    candidates
}
```

## REFACTOR Phase: Quality & Design

- **Unified Ammo**: Refactor `TrashCannon` to use `ItemType::WasteChunk` instead of magic resource consumption, unifying the system.
- **UI**: Show ammo count in selection UI.
- **Config**: Move Max Ammo capacity to `Turret` component.
- **Crafting**: Add `Workbench` recipe for `Metal` -> `Ammo`.

## Acceptance Criteria

- [ ] `Ammo` item type exists.
- [ ] `MachineGunTurret` building type exists.
- [ ] Turrets with `Inventory` can fire if they have Ammo.
- [ ] Firing consumes 1 Ammo item.
- [ ] Turrets without Ammo do not fire.
- [ ] Tests pass.

## Technical Guidance

- Ensure `Inventory` component is added to `MachineGunTurret` during spawning in `spawn_building`.
- Don't break `TrashCannon` (keep `required_item: None` or `Some(Waste)` logic separate until refactor).
