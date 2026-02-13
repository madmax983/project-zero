# 102: Orbital Drop Logistics

## Overview

Implements the mechanism for transferring resources from Layer 2 (Fleets) to Layer 1 (Colony Surface). Since fleets cannot land directly (they are too large/dangerous), they must launch **Drop Pods**.

This feature closes the economic loop between space mining and colony construction. Players command fleets to drop cargo, which spawns a `DropPod` entity that eventually "crashes" on the colony map, spawning `ResourceItem` piles that can be hauled.

## Dependencies

- `099` — Fleet Movement (for `Fleet`, `FleetCargo`, `FleetOrder`, `InOrbit`)
- `101` — System Mining (for `FleetCargo` structure)
- `014` — Resource Items (for `ResourceItem` and `ResourceType`)
- `095` — System Generation (for `ColonyLocation` tag)

## RED Phase: Tests First

Write these tests in `src/layer2/drop_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
    use crate::layer2::mining::{FleetCargo, CargoStack};
    use crate::layer2::drop::{DropPod, orbital_drop_system, pod_impact_system};
    use crate::layer1::resources::{ResourceItem, ResourceType};
    use crate::layer1::map::MapSize; // Assuming coordinate bounds

    fn setup_world() -> World {
        let mut world = World::new();
        // Register components
        world.init_resource::<MapSize>(); // Default map size
        world
    }

    #[test]
    fn test_drop_order_removes_cargo() {
        let mut world = setup_world();
        let colony = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: colony },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Ore,
                    amount: 100.0,
                }],
                capacity: 100.0,
            }
        )).id();

        // Issue Drop Order for 50 Ore
        world.entity_mut(fleet).insert(FleetOrder::Drop(ResourceType::Ore, 50.0));

        // Run Drop System
        let mut schedule = Schedule::default();
        schedule.add_systems(orbital_drop_system);
        schedule.run(&mut world);

        // Verify Cargo Reduced
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert_eq!(cargo.contents[0].amount, 50.0);

        // Verify Order Consumed
        assert!(world.get::<FleetOrder>(fleet).is_none());
    }

    #[test]
    fn test_drop_order_spawns_pod() {
        let mut world = setup_world();
        let colony = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: colony },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Ore,
                    amount: 50.0,
                }],
                capacity: 100.0,
            }
        )).id();

        world.entity_mut(fleet).insert(FleetOrder::Drop(ResourceType::Ore, 50.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(orbital_drop_system);
        schedule.run(&mut world);

        // Verify DropPod Spawned
        let pod_count = world.query::<&DropPod>().iter(&world).count();
        assert_eq!(pod_count, 1);

        let (pod_entity, pod) = world.query::<(Entity, &DropPod)>().single(&world);
        assert_eq!(pod.resource_type, ResourceType::Ore);
        assert_eq!(pod.amount, 50.0);
        assert!(pod.impact_timer > 0.0);
    }

    #[test]
    fn test_pod_impact_spawns_resources() {
        let mut world = setup_world();

        // Spawn a DropPod about to impact
        let pod = world.spawn(DropPod {
            resource_type: ResourceType::Ore,
            amount: 50.0,
            impact_timer: 0.0, // Ready to impact
            target_pos: (10, 10),
        }).id();

        // Run Impact System
        let mut schedule = Schedule::default();
        schedule.add_systems(pod_impact_system);
        schedule.run(&mut world);

        // Verify Pod Despawned
        assert!(world.get::<DropPod>(pod).is_none());

        // Verify ResourceItem Spawned
        let item_count = world.query::<&ResourceItem>().iter(&world).count();
        assert_eq!(item_count, 1);

        let (item_entity, item) = world.query::<(Entity, &ResourceItem)>().single(&world);
        assert_eq!(item.resource_type, ResourceType::Ore);
        // Assuming ResourceItem logic handles amount (might need multiple items or stacks)
        // For MVP, assume 1 item = 1 unit or the item carries amount?
        // Spec 014 says ResourceItem has `amount`.
        // If Spec 014 ResourceItem doesn't have amount, we might spawn multiple entities.
        // Let's assume ResourceItem has amount for now, or check Spec 014.
        // (Checking Spec 014 is recommended during Green Phase)
    }

    #[test]
    fn test_drop_scatter() {
        let mut world = setup_world();
        let colony = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: colony },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Ore,
                    amount: 50.0,
                }],
                capacity: 100.0,
            }
        )).id();

        // Set RNG seed if possible, or run multiple times to check variance.
        // For simple test, just check it's within bounds.
        world.entity_mut(fleet).insert(FleetOrder::Drop(ResourceType::Ore, 50.0));

        let mut schedule = Schedule::default();
        schedule.add_systems(orbital_drop_system);
        schedule.run(&mut world);

        let (_, pod) = world.query::<(Entity, &DropPod)>().single(&world);

        // Assuming map center is (MapSize/2, MapSize/2)
        // Check if pod.target_pos is valid.
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `FleetOrder` (`src/layer2/fleet.rs`)

```rust
// Add Drop variant
#[derive(Component, Debug, Clone, Copy)]
pub enum FleetOrder {
    MoveTo(Entity),
    Mine(Entity),
    Drop(ResourceType, f32), // Type, Amount
}
```

### 2. Define Components & Systems (`src/layer2/drop.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::{ResourceItem, ResourceType}; // Requires layer1 visibility
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use crate::layer2::mining::FleetCargo;
use rand::prelude::*;

#[derive(Component, Debug)]
pub struct DropPod {
    pub resource_type: ResourceType,
    pub amount: f32,
    pub impact_timer: f32, // Ticks until landing
    pub target_pos: (i32, i32),
}

pub fn orbital_drop_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FleetOrder, &InOrbit, &mut FleetCargo), With<Fleet>>,
    // Add query for ColonyLocation/MapSize if needed for targeting
) {
    let mut rng = thread_rng();

    for (entity, mut order, orbit, mut cargo) in query.iter_mut() {
        if let FleetOrder::Drop(res_type, amount) = *order {
            // 1. Check if we have the cargo
            let mut taken = 0.0;
            if let Some(stack) = cargo.contents.iter_mut().find(|s| s.resource_type == res_type) {
                if stack.amount >= amount {
                    stack.amount -= amount;
                    taken = amount;
                } else {
                    // Take what we can? Or fail?
                    // MVP: Take what we can
                    taken = stack.amount;
                    stack.amount = 0.0;
                }
            }

            // Cleanup empty stacks
            cargo.contents.retain(|s| s.amount > 0.0);

            if taken > 0.0 {
                // 2. Spawn Drop Pod
                // Target: Center of map + scatter
                // Hardcoded center (40, 40) for MVP, or query MapSize
                let center_x = 40;
                let center_y = 40;
                let scatter = 10;
                let tx = center_x + rng.gen_range(-scatter..=scatter);
                let ty = center_y + rng.gen_range(-scatter..=scatter);

                commands.spawn(DropPod {
                    resource_type: res_type,
                    amount: taken,
                    impact_timer: 10.0, // 1 second at 10 ticks/sec
                    target_pos: (tx, ty),
                });
            }

            // 3. Consume Order
            commands.entity(entity).remove::<FleetOrder>();
        }
    }
}

pub fn pod_impact_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DropPod)>,
    // Need to spawn ResourceItems
) {
    for (entity, mut pod) in query.iter_mut() {
        pod.impact_timer -= 1.0;

        if pod.impact_timer <= 0.0 {
            // Impact!
            // Spawn items
            // Assuming ResourceItem component from Spec 014
            // If ResourceItem doesn't support stacks, spawn 'amount' entities?
            // Spec 014 usually implies items are entities.
            // Let's assume we spawn one entity with a "Stack" component if it exists,
            // or just one entity representing the crate.

            // For MVP: Spawn 1 entity at target_pos
            // Note: Layer 1 entities need Position component
            // commands.spawn((
            //    ResourceItem { resource_type: pod.resource_type },
            //    Position { x: pod.target_pos.0, y: pod.target_pos.1 }
            // ));

            // Placeholder:
            // commands.spawn(ResourceItem::new(pod.resource_type, pod.target_pos));

            commands.entity(entity).despawn();
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Map Size**: Do not hardcode (40, 40). Query `MapSize` resource.
- **Resource Stacking**: If `ResourceItem` is 1-to-1, loop to spawn `amount` items (dangerous for perf). Better: Ensure `ResourceItem` has a `quantity` field or introduce `ResourceStack`.
- **Impact Damage**: Pods landing on buildings should damage/destroy them.
- **Notification**: Send a notification "Supply drop detected at [X,Y]" when spawned.
- **Visuals**: Render the falling pod in the sky layer (Layer 1 UI) or as a shadow growing larger.

## Acceptance Criteria

- [ ] `FleetOrder::Drop` implemented.
- [ ] `DropPod` spawns with correct contents when order is issued.
- [ ] Fleet cargo is correctly reduced.
- [ ] `DropPod` despawns after timer.
- [ ] `ResourceItem`s spawn at impact location.
- [ ] Tests pass.

## Technical Guidance

- Ensure `layer2` can access `layer1` components (`ResourceItem`, `Position`).
- Check `src/layer1/resources.rs` to see if `ResourceItem` supports quantity. If not, document this limitation or create a `ResourcePile` component.
- Random scatter should be clamped to map bounds.
