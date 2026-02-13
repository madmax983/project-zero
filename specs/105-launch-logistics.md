# 105: Launch Logistics

## Overview

Enables the colony to launch **Fleets** into orbit (Layer 2).
This feature bridges the gap between Layer 1 (Colony) and Layer 2 (System).
It introduces the **Launch Pad** building and the **Launch** action.

## Dependencies

- `104` — Fuel Industry (for `Fuel` resource)
- `099` — Fleet Movement (for `Fleet`, `InOrbit` components)
- `006` — Building Placement (for `LaunchPad`)

## RED Phase: Tests First

Write these tests in `src/layer1/launch_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{ColonyResources, ResourceType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::launch::{LaunchPad, LaunchOrder, launch_system};
    use crate::layer2::fleet::{Fleet, InOrbit};
    use crate::layer2::mining::FleetCargo; // Assuming this exists from Spec 101/102

    #[test]
    fn test_launch_pad_building_type() {
        let b = BuildingType::LaunchPad;
        assert_eq!(b.label(), "Launch Pad");
        // Ensure it's constructible
    }

    #[test]
    fn test_launch_order_consumes_fuel() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.fuel = 100.0;
        res.max_fuel = 100.0;
        res.metal = 50.0; // Cargo
        world.insert_resource(res);

        // Spawn Launch Pad
        let pad = world.spawn((
            Building { building_type: BuildingType::LaunchPad },
            LaunchPad,
        )).id();

        // Issue Launch Order
        world.entity_mut(pad).insert(LaunchOrder {
            fuel_cost: 50.0,
            cargo: vec![(ResourceType::Metal, 20.0)],
        });

        // Run System
        let mut schedule = Schedule::default();
        schedule.add_systems(launch_system);
        schedule.run(&mut world);

        // Verify Resources Deducted
        let res = world.resource::<ColonyResources>();
        assert_eq!(res.fuel, 50.0); // 100 - 50
        assert_eq!(res.metal, 30.0); // 50 - 20
    }

    #[test]
    fn test_launch_spawns_fleet() {
        let mut world = World::new();
        // Setup resources...
        world.insert_resource(ColonyResources {
            fuel: 100.0,
            metal: 100.0,
            ..Default::default()
        });

        // Need a planet entity for Orbit
        let planet = world.spawn_empty().id();
        // Associate LaunchPad with Planet?
        // For MVP, assume LaunchPad is on THE planet.
        // We might need a resource `CurrentPlanet` or similar.
        world.insert_resource(crate::layer1::map::CurrentPlanet(planet));

        let pad = world.spawn((
            Building { building_type: BuildingType::LaunchPad },
            LaunchPad,
        )).id();

        world.entity_mut(pad).insert(LaunchOrder {
            fuel_cost: 50.0,
            cargo: vec![(ResourceType::Metal, 20.0)],
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(launch_system);
        schedule.run(&mut world);

        // Verify Fleet Spawned
        let fleet_query = world.query::<(&Fleet, &InOrbit, &FleetCargo)>().iter(&world);
        let count = fleet_query.len();
        assert_eq!(count, 1);

        let (_, orbit, cargo) = world.query::<(&Fleet, &InOrbit, &FleetCargo)>().single(&world);
        assert_eq!(orbit.parent, planet);
        assert_eq!(cargo.contents[0].resource_type, ResourceType::Metal);
        assert_eq!(cargo.contents[0].amount, 20.0);
    }

    #[test]
    fn test_launch_fails_if_insufficient_fuel() {
        let mut world = World::new();
        world.insert_resource(ColonyResources {
            fuel: 10.0, // Not enough
            ..Default::default()
        });

        let pad = world.spawn((
            Building { building_type: BuildingType::LaunchPad },
            LaunchPad,
        )).id();

        world.entity_mut(pad).insert(LaunchOrder {
            fuel_cost: 50.0,
            cargo: vec![],
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(launch_system);
        schedule.run(&mut world);

        // Order should NOT be consumed
        assert!(world.get::<LaunchOrder>(pad).is_some());
        // No fleet
        assert_eq!(world.query::<&Fleet>().iter(&world).count(), 0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType`

Add `LaunchPad`.

### 2. Define Components (`src/layer1/launch.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer2::fleet::{Fleet, InOrbit};
use crate::layer2::mining::{FleetCargo, CargoStack};

#[derive(Component)]
pub struct LaunchPad;

#[derive(Component)]
pub struct LaunchOrder {
    pub fuel_cost: f32,
    pub cargo: Vec<(ResourceType, f32)>,
}

#[derive(Resource)]
pub struct CurrentPlanet(pub Entity); // Helper to know where we are

pub fn launch_system(
    mut commands: Commands,
    mut query: Query<(Entity, &LaunchOrder), With<LaunchPad>>,
    mut resources: ResMut<ColonyResources>,
    planet: Option<Res<CurrentPlanet>>,
) {
    let Some(planet_entity) = planet.map(|p| p.0) else { return };

    for (entity, order) in query.iter() {
        // Check costs
        if resources.fuel < order.fuel_cost { continue; }

        // Simplified deduction logic
        let mut affordable = true;

        // Check cargo costs (implementation detail)

        if affordable {
            // Deduct
            resources.fuel -= order.fuel_cost;
            // Deduct cargo...

            // Spawn Fleet
            commands.spawn((
                Fleet,
                InOrbit { parent: planet_entity },
                FleetCargo {
                    contents: order.cargo.iter().map(|(t, a)| CargoStack {
                        resource_type: *t,
                        amount: *a
                    }).collect(),
                    capacity: 1000.0,
                }
            ));

            // Remove Order
            commands.entity(entity).remove::<LaunchOrder>();
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Payload Construction**: Instead of instant deduction, require Haulers to bring resources to the pad (like `Construction`).
- **Orbit Handling**: Ensure `CurrentPlanet` is initialized during map gen.
- **UI Integration**: Needs a UI to set up the launch (select cargo, fuel amount).

## Acceptance Criteria

- [ ] `LaunchPad` building implemented.
- [ ] `LaunchOrder` triggers fleet spawn.
- [ ] Resources (Fuel + Cargo) deducted correctly.
- [ ] Fleet appears in Layer 2 (conceptually/data-wise).
- [ ] Tests pass.
