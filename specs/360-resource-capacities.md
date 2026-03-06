# Spec 360: Resource Capacities

## 1. Overview
A mountain of stone with nowhere to put it becomes an obstacle, not an asset. Storage buildings (Stockpiles, Granaries) must have finite capacity slots per resource type. Excess resources decay rapidly or block production.

## 2. Dependencies
- `006-building-placement.md` (for Buildings)
- `018-mining-resources.md` (for Resource Types)
- `025-hauling-logistics.md` (for Storing items)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::building::Building;
    use crate::layer1::resource::{ResourceType, Storage};

    #[test]
    fn test_storage_cannot_exceed_capacity() {
        let mut app = App::new();
        app.add_systems(Update, process_storage_insertion_system);

        // Stockpile with 100 capacity
        let storage_ent = app.world_mut().spawn((
            Building { building_type: BuildingType::Stockpile },
            Storage { capacity: 100, current_amount: 90, resource_type: ResourceType::Stone }
        )).id();

        // Try adding 20 stone
        app.world_mut().send_event(StoreResourceEvent {
            target: storage_ent,
            resource_type: ResourceType::Stone,
            amount: 20,
        });

        app.update();

        // Storage should cap at 100
        let storage = app.world().get::<Storage>(storage_ent).unwrap();
        assert_eq!(storage.current_amount, 100);
    }

    #[test]
    fn test_excess_resources_spawn_as_ground_nodes() {
        let mut app = App::new();
        app.add_event::<StoreResourceEvent>();
        app.add_systems(Update, process_storage_insertion_system);

        let storage_ent = app.world_mut().spawn((
            Building { building_type: BuildingType::Stockpile },
            Storage { capacity: 100, current_amount: 90, resource_type: ResourceType::Stone },
            GridPosition { x: 5, y: 5 },
        )).id();

        app.world_mut().send_event(StoreResourceEvent {
            target: storage_ent,
            resource_type: ResourceType::Stone,
            amount: 20,
        });

        app.update();

        // The remaining 10 stone should be spawned on the floor
        let mut query = app.world_mut().query::<(&ResourceNode, &GridPosition)>();
        let mut found = false;
        for (node, pos) in query.iter(app.world()) {
            if pos.x == 5 && pos.y == 5 && node.resource_type == ResourceType::Stone && node.amount == 10 {
                found = true;
                break;
            }
        }
        assert!(found, "Excess resource was not dropped on the ground.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer1::building::Building;
use crate::layer1::resource::{ResourceType, ResourceNode};
use crate::layer1::grid::GridPosition;

#[derive(Component)]
pub struct Storage {
    pub capacity: u32,
    pub current_amount: u32,
    pub resource_type: ResourceType,
}

#[derive(Event)]
pub struct StoreResourceEvent {
    pub target: Entity,
    pub resource_type: ResourceType,
    pub amount: u32,
}

pub fn process_storage_insertion_system(
    mut commands: Commands,
    mut events: EventReader<StoreResourceEvent>,
    mut storage_query: Query<(&mut Storage, &GridPosition)>,
) {
    for event in events.read() {
        if let Ok((mut storage, pos)) = storage_query.get_mut(event.target) {
            if storage.resource_type == event.resource_type {
                let available_space = storage.capacity.saturating_sub(storage.current_amount);

                if available_space >= event.amount {
                    storage.current_amount += event.amount;
                } else {
                    // Fill up what we can
                    storage.current_amount = storage.capacity;

                    // Drop the rest
                    let excess = event.amount - available_space;
                    commands.spawn((
                        ResourceNode {
                            resource_type: event.resource_type,
                            amount: excess,
                        },
                        GridPosition { x: pos.x, y: pos.y },
                    ));
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Event Result:** Ensure Haulers are notified if a Stockpile is full *before* they walk all the way there. Right now, they drop it at the Stockpile door.
- **Multiple Types:** A `Storage` might handle multiple types of resources (e.g., General Warehouse) rather than a single `ResourceType`.
- **Decay:** Ground items left as excess should decay over time to simulate spoilage.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] `Storage` component correctly caps `current_amount` at `capacity`.
- [ ] Any excess resource beyond `capacity` spawns a `ResourceNode` on the floor at the storage's `GridPosition`.

## 7. Technical Guidance
- Add `StoreResourceEvent` to App initialization.
- Register `process_storage_insertion_system` in `Layer1SystemSet::Execution`.
- Storage component logic should ideally be accessed by utility AI before a task begins.

## 8. Questions
*Builder: add questions here if spec is unclear.*
