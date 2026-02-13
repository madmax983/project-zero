# 101: System Mining

## Overview

Introduces resource extraction mechanics to Layer 2 (System View). Players can command Fleets to mine **Asteroids** and **Comets** for resources. This feature bridges the gap between exploration (finding bodies) and economy (gathering resources), giving players a reason to build and deploy fleets beyond just movement.

## Dependencies

- `095` — System Generation (for orbital bodies)
- `099` — Fleet Movement (for `Fleet`, `FleetOrder`, `InOrbit`)
- `014` — Resource Items (for `ResourceType` enum)

## RED Phase: Tests First

Write these tests in `src/layer2/mining_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::mining::{MiningTarget, FleetCargo, FleetMining, mining_system, fleet_mine_order_system};
    use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
    use crate::layer1::resources::ResourceType;

    fn setup_world() -> World {
        let mut world = World::new();
        // Register components if needed
        world
    }

    #[test]
    fn test_mining_target_has_resources() {
        let mut world = setup_world();
        let asteroid = world.spawn(MiningTarget {
            resource_type: ResourceType::Ore,
            amount: 100.0,
            mining_difficulty: 1.0,
        }).id();

        let target = world.get::<MiningTarget>(asteroid).expect("Asteroid should have mining target");
        assert_eq!(target.resource_type, ResourceType::Ore);
        assert_eq!(target.amount, 100.0);
    }

    #[test]
    fn test_fleet_cargo_capacity() {
        let mut world = setup_world();
        let fleet = world.spawn((
            Fleet,
            FleetCargo {
                contents: vec![],
                capacity: 50.0,
            }
        )).id();

        let cargo = world.get::<FleetCargo>(fleet).expect("Fleet should have cargo");
        assert_eq!(cargo.capacity, 50.0);
    }

    #[test]
    fn test_order_mining_initiates_process() {
        let mut world = setup_world();

        // Setup Asteroid
        let asteroid = world.spawn(MiningTarget {
            resource_type: ResourceType::Ore,
            amount: 100.0,
            mining_difficulty: 1.0,
        }).id();

        // Setup Fleet in Orbit of Asteroid
        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: asteroid },
            FleetCargo::default(),
        )).id();

        // Issue Mine Order
        world.entity_mut(fleet).insert(FleetOrder::Mine(asteroid));

        // Run Order System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_mine_order_system);
        schedule.run(&mut world);

        // Verify Fleet is now Mining
        assert!(world.get::<FleetOrder>(fleet).is_none(), "Order should be consumed");
        let mining_state = world.get::<FleetMining>(fleet).expect("Fleet should be in mining state");
        assert_eq!(mining_state.target, asteroid);
    }

    #[test]
    fn test_mining_extracts_resources() {
        let mut world = setup_world();

        let asteroid = world.spawn(MiningTarget {
            resource_type: ResourceType::Ore,
            amount: 100.0,
            mining_difficulty: 1.0,
        }).id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: asteroid },
            FleetCargo {
                contents: vec![],
                capacity: 50.0,
            },
            FleetMining {
                target: asteroid,
                rate: 10.0, // Mines 10.0 per tick
            }
        )).id();

        // Run Mining System
        let mut schedule = Schedule::default();
        schedule.add_systems(mining_system);
        schedule.run(&mut world);

        // Verify Asteroid lost resources
        let target = world.get::<MiningTarget>(asteroid).unwrap();
        assert!((target.amount - 90.0).abs() < f32::EPSILON);

        // Verify Fleet gained resources
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert_eq!(cargo.contents.len(), 1);
        assert_eq!(cargo.contents[0].resource_type, ResourceType::Ore);
        assert!((cargo.contents[0].amount - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mining_stops_when_full() {
        let mut world = setup_world();

        let asteroid = world.spawn(MiningTarget {
            resource_type: ResourceType::Ore,
            amount: 100.0,
            mining_difficulty: 1.0,
        }).id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: asteroid },
            FleetCargo {
                contents: vec![], // Empty
                capacity: 5.0,    // Small capacity
            },
            FleetMining {
                target: asteroid,
                rate: 10.0, // Mines more than capacity
            }
        )).id();

        // Run Mining System
        let mut schedule = Schedule::default();
        schedule.add_systems(mining_system);
        schedule.run(&mut world);

        // Verify Fleet is full (capped at 5.0)
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert!((cargo.contents[0].amount - 5.0).abs() < f32::EPSILON);

        // Verify Asteroid only lost 5.0
        let target = world.get::<MiningTarget>(asteroid).unwrap();
        assert!((target.amount - 95.0).abs() < f32::EPSILON);

        // Verify Mining State removed (completed/stopped)
        assert!(world.get::<FleetMining>(fleet).is_none(), "Should stop mining when full");
    }

    #[test]
    fn test_mining_stops_when_depleted() {
        let mut world = setup_world();

        let asteroid = world.spawn(MiningTarget {
            resource_type: ResourceType::Ore,
            amount: 5.0, // Only 5 left
            mining_difficulty: 1.0,
        }).id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: asteroid },
            FleetCargo::default(), // Capacity 50
            FleetMining {
                target: asteroid,
                rate: 10.0,
            }
        )).id();

        // Run Mining System
        let mut schedule = Schedule::default();
        schedule.add_systems(mining_system);
        schedule.run(&mut world);

        // Verify Asteroid is empty/depleted
        let target = world.get::<MiningTarget>(asteroid).unwrap();
        assert!(target.amount <= 0.0);

        // Verify Fleet gained 5.0
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert!((cargo.contents[0].amount - 5.0).abs() < f32::EPSILON);

        // Verify Mining State removed
        assert!(world.get::<FleetMining>(fleet).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer2/mining.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::ResourceType;
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};

/// Component for orbital bodies that can be mined.
#[derive(Component, Debug, Clone)]
pub struct MiningTarget {
    pub resource_type: ResourceType,
    pub amount: f32,
    pub mining_difficulty: f32, // Multiplier for time taken
}

/// Represents a single stack of cargo in a fleet.
#[derive(Debug, Clone, Copy)]
pub struct CargoStack {
    pub resource_type: ResourceType,
    pub amount: f32,
}

/// Component for fleets to store mined resources.
#[derive(Component, Debug, Clone, Default)]
pub struct FleetCargo {
    pub contents: Vec<CargoStack>,
    pub capacity: f32,
}

impl FleetCargo {
    pub fn current_load(&self) -> f32 {
        self.contents.iter().map(|s| s.amount).sum()
    }

    pub fn add(&mut self, resource_type: ResourceType, amount: f32) -> f32 {
        let current = self.current_load();
        let available = self.capacity - current;
        let to_add = amount.min(available);

        if to_add <= 0.0 {
            return 0.0;
        }

        // Check if stack exists
        if let Some(stack) = self.contents.iter_mut().find(|s| s.resource_type == resource_type) {
            stack.amount += to_add;
        } else {
            self.contents.push(CargoStack {
                resource_type,
                amount: to_add,
            });
        }
        to_add
    }
}

/// Component indicating a fleet is actively mining.
#[derive(Component, Debug, Clone)]
pub struct FleetMining {
    pub target: Entity,
    pub rate: f32,
}

/// System to handle Mine orders.
pub fn fleet_mine_order_system(
    mut commands: Commands,
    query: Query<(Entity, &FleetOrder, Option<&InOrbit>, Option<&FleetCargo>), With<Fleet>>,
    targets: Query<&MiningTarget>,
) {
    for (entity, order, maybe_orbit, maybe_cargo) in query.iter() {
        if let FleetOrder::Mine(target_entity) = order {
            // Must be in orbit of target
            if let Some(orbit) = maybe_orbit {
                if orbit.parent == *target_entity {
                    // Must have cargo space
                    if let Some(cargo) = maybe_cargo {
                         if cargo.current_load() < cargo.capacity {
                            // Start Mining
                            commands.entity(entity)
                                .remove::<FleetOrder>()
                                .insert(FleetMining {
                                    target: *target_entity,
                                    rate: 1.0, // Default base rate
                                });
                             continue;
                         }
                    }
                }
            }
            // Invalid order (wrong location or full) - consume it or log error?
            // For MVP, just consume it to prevent infinite loop
            commands.entity(entity).remove::<FleetOrder>();
        }
    }
}

/// System to process mining over time.
pub fn mining_system(
    mut commands: Commands,
    mut fleets: Query<(Entity, &mut FleetMining, &mut FleetCargo)>,
    mut targets: Query<&mut MiningTarget>,
) {
    for (fleet_entity, mining, mut cargo) in fleets.iter_mut() {
        if let Ok(mut target) = targets.get_mut(mining.target) {
            if target.amount <= 0.0 {
                // Depleted
                commands.entity(fleet_entity).remove::<FleetMining>();
                continue;
            }

            let amount_to_mine = mining.rate; // Per tick

            // Try to add to cargo (handles capacity)
            let actually_added = cargo.add(target.resource_type, amount_to_mine);

            // Deduct from target (only what was actually taken)
            target.amount -= actually_added;

            // Stop if full or depleted
            if cargo.current_load() >= cargo.capacity || target.amount <= 0.0 {
                commands.entity(fleet_entity).remove::<FleetMining>();
            }
        } else {
            // Target destroyed/despawned
            commands.entity(fleet_entity).remove::<FleetMining>();
        }
    }
}
```

### 2. Update `FleetOrder` Enum

*Note: This requires updating the existing `FleetOrder` definition in `src/layer2/fleet.rs`. The Builder must do this.*

```rust
// In src/layer2/fleet.rs
#[derive(Component, Debug, Clone, Copy)]
pub enum FleetOrder {
    MoveTo(Entity),
    Mine(Entity), // New variant
}
```

## REFACTOR Phase: Quality & Design

- **Mining Rate Calculation**: Should depend on fleet composition (e.g., number of `MiningModule` components installed on ships).
- **Asteroid Depletion**: When `MiningTarget.amount` reaches 0, the asteroid entity should perhaps remain as a "Barren Rock" or be despawned if it was a small comet.
- **Cargo Transfers**: Need a way to move cargo from `FleetCargo` to `ColonyResources` (L1). This will be covered in Spec 102 (Orbital Drop Logistics).
- **Visuals**: Add a "Mining Beam" effect or status icon in the System View when `FleetMining` is active.

## Acceptance Criteria

- [ ] `MiningTarget` component exists and can be added to entities.
- [ ] `FleetCargo` component exists and tracks capacity.
- [ ] `FleetOrder::Mine` is implemented and triggers mining state.
- [ ] Resources are correctly transferred from Target to Cargo.
- [ ] Mining stops when Cargo is full or Target is depleted.
- [ ] Tests pass.

## Technical Guidance

- Ensure `FleetOrder` enum update is propagated to any match statements in `fleet.rs`.
- `ResourceType` is defined in `layer1::resources`, so `layer2` must depend on `layer1` (which is already true).
- Use `f32::min` for capping logic.
