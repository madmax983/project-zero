# 152: Orbital Stations

## Overview

Enables **Fleets** to construct stationary **Orbital Stations** in Layer 2 (System View).
Stations serve as permanent outposts, refineries, or defensive platforms.

This spec defines:
1.  **Station Component**: Identifies an entity as a Station.
2.  **StationType Enum**: Differentiates between types (Outpost, Mining Platform, Shipyard).
3.  **Construction Order**: Updates `FleetOrder` to allow building a station by consuming resources.
4.  **Station Logic**: Stations are `OrbitalBody`s that do not move relative to their parent (or have fixed orbits).

This is a foundational feature for Layer 2 economy and persistence.

## Dependencies

- `099` — Fleet Movement (for `Fleet`, `FleetOrder`)
- `101` — System Mining (for `FleetCargo`)
- `094` — System View (for `OrbitalBody`, `Orbit`)

## RED Phase: Tests First

Write these tests in `src/layer2/station_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
    use crate::layer2::mining::{FleetCargo, CargoStack};
    use crate::layer2::station::{Station, StationType, build_station_system};
    use crate::layer2::system::{OrbitalBody, Orbit};
    use crate::layer1::resources::ResourceType;
    use ratatui::style::Color;

    fn setup_world() -> World {
        let mut world = World::new();
        // Register components
        world
    }

    #[test]
    fn test_station_component() {
        let mut world = setup_world();
        let station = world.spawn((
            Station { station_type: StationType::Outpost },
            OrbitalBody {
                name: "Alpha Station".to_string(),
                radius: 1.0,
                color: Color::Gray,
                char: '+',
            }
        )).id();

        let s = world.get::<Station>(station).unwrap();
        assert_eq!(s.station_type, StationType::Outpost);
    }

    #[test]
    fn test_build_station_consumes_resources() {
        let mut world = setup_world();
        let planet = world.spawn_empty().id();

        // Setup Fleet with resources
        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Metal,
                    amount: 100.0,
                }],
                capacity: 100.0,
            }
        )).id();

        // Issue Build Order (Cost: 50 Metal)
        // Note: FleetOrder needs update in Green Phase
        world.entity_mut(fleet).insert(FleetOrder::BuildStation(StationType::Outpost));

        // Run Build System
        let mut schedule = Schedule::default();
        schedule.add_systems(build_station_system);
        schedule.run(&mut world);

        // Verify Resources Deducted
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert_eq!(cargo.contents[0].amount, 50.0);

        // Verify Order Consumed
        assert!(world.get::<FleetOrder>(fleet).is_none());
    }

    #[test]
    fn test_build_station_spawns_entity() {
        let mut world = setup_world();
        let planet = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Metal,
                    amount: 100.0,
                }],
                capacity: 100.0,
            }
        )).id();

        world.entity_mut(fleet).insert(FleetOrder::BuildStation(StationType::Outpost));

        let mut schedule = Schedule::default();
        schedule.add_systems(build_station_system);
        schedule.run(&mut world);

        // Verify Station Spawned
        let station_count = world.query::<&Station>().iter(&world).count();
        assert_eq!(station_count, 1);

        let (entity, station, orbit) = world.query::<(Entity, &Station, &Orbit)>().single(&world);
        assert_eq!(station.station_type, StationType::Outpost);
        assert_eq!(orbit.parent, planet);
        // Station should orbit at same radius/angle as fleet? Or fixed offset?
        // For MVP, assume it spawns at current location (which is InOrbit of parent).
    }

    #[test]
    fn test_build_station_fails_insufficient_resources() {
        let mut world = setup_world();
        let planet = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            FleetCargo {
                contents: vec![CargoStack {
                    resource_type: ResourceType::Metal,
                    amount: 10.0, // Need 50
                }],
                capacity: 100.0,
            }
        )).id();

        world.entity_mut(fleet).insert(FleetOrder::BuildStation(StationType::Outpost));

        let mut schedule = Schedule::default();
        schedule.add_systems(build_station_system);
        schedule.run(&mut world);

        // Verify No Station
        let station_count = world.query::<&Station>().iter(&world).count();
        assert_eq!(station_count, 0);

        // Verify Order Not Consumed (or Consumed with failure log?)
        // For MVP, keep order until resources available or manually cancelled.
        assert!(world.get::<FleetOrder>(fleet).is_some());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `FleetOrder` (`src/layer2/fleet.rs`)

```rust
// Add BuildStation variant
#[derive(Component, Debug, Clone, Copy)]
pub enum FleetOrder {
    MoveTo(Entity),
    Mine(Entity),
    Drop(ResourceType, f32),
    BuildStation(StationType), // New
}
```

### 2. Define Components & Systems (`src/layer2/station.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::ResourceType;
use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
use crate::layer2::mining::FleetCargo;
use crate::layer2::system::{OrbitalBody, Orbit};
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationType {
    Outpost,
    MiningPlatform,
    Shipyard,
}

impl StationType {
    pub fn cost(&self) -> Vec<(ResourceType, f32)> {
        match self {
            Self::Outpost => vec![(ResourceType::Metal, 50.0)],
            Self::MiningPlatform => vec![(ResourceType::Metal, 100.0), (ResourceType::Fuel, 10.0)],
            Self::Shipyard => vec![(ResourceType::Metal, 200.0), (ResourceType::Fuel, 50.0)],
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Outpost => "Outpost",
            Self::MiningPlatform => "Mining Platform",
            Self::Shipyard => "Shipyard",
        }
    }

    pub fn char(&self) -> char {
         match self {
            Self::Outpost => '+',
            Self::MiningPlatform => '⚒',
            Self::Shipyard => '⚓',
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Station {
    pub station_type: StationType,
}

pub fn build_station_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FleetOrder, &InOrbit, &mut FleetCargo), With<Fleet>>,
) {
    for (entity, order, orbit, mut cargo) in query.iter_mut() {
        if let FleetOrder::BuildStation(station_type) = *order {
            let cost = station_type.cost();

            // Check affordability
            let can_afford = cost.iter().all(|(res, amt)| {
                cargo.contents.iter().any(|s| s.resource_type == *res && s.amount >= *amt)
            });

            if can_afford {
                // Deduct
                for (res, amt) in &cost {
                    if let Some(stack) = cargo.contents.iter_mut().find(|s| s.resource_type == *res) {
                        stack.amount -= amt;
                    }
                }
                cargo.contents.retain(|s| s.amount > 0.0);

                // Spawn Station
                commands.spawn((
                    Station { station_type },
                    OrbitalBody {
                        name: format!("{} {}", station_type.label(), entity.index()), // Unique-ish name
                        radius: 0.5,
                        color: Color::Cyan,
                        char: station_type.char(),
                    },
                    Orbit {
                        parent: orbit.parent,
                        radius: 10.0, // TODO: Use fleet's current radius? Or standard orbit?
                        speed: 0.05,
                        angle: 0.0, // TODO: Random angle or fleet's angle
                    }
                ));

                // Consume Order
                commands.entity(entity).remove::<FleetOrder>();
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Orbit Location**: Station should spawn exactly where the fleet is (same angle/radius/parent).
- **Construction Time**: Building shouldn't be instant. Add `UnderConstruction` state or `ConstructionTimer`.
- **Naming**: Allow custom names for stations.
- **Population**: Stations should track `Population` or `Crew`.
- **Upkeep**: Stations consume `Fuel` or `Power`.

## Acceptance Criteria

- [ ] `Station` component and `StationType` enum defined.
- [ ] `FleetOrder::BuildStation` implemented.
- [ ] Resources deducted from FleetCargo.
- [ ] Station entity spawns with correct visuals (`OrbitalBody`).
- [ ] Tests pass.

## Technical Guidance

- Use `ratatui::style::Color` for visuals.
- Ensure `layer2` module structure exposes `station`.
- `Station` entities are distinct from `Planet` entities but share `OrbitalBody`. The renderer should handle both gracefully.
