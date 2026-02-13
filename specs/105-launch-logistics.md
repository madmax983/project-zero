# 105: Launch Logistics

## Overview

Enables the construction and launch of **Fleets** from Layer 1 (Colony) to Layer 2 (System).
- **Launch Pad**: A massive building required to build and launch ships.
- **Launch Process**: Consumes **Metal** (to build the ship) and **Fuel** (to reach orbit).
- **Result**: Spawns a new Fleet entity in orbit around the colony's planet.

This feature completes the loop between Colony and System layers, allowing the player to begin space exploration and mining.

## Dependencies

- `104` — Fuel Industry (for `Fuel` resource)
- `099` — Fleet Movement (for `Fleet`, `InOrbit` components)
- `095` — System Generation (for `ColonyLocation` lookup)

## RED Phase: Tests First

Write these tests in `src/layer1/launch_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::launch::{LaunchPad, LaunchQueue, LaunchPayload, launch_system};
    use crate::layer2::fleet::{Fleet, InOrbit};
    use crate::layer2::generation::ColonyLocation;

    #[test]
    fn test_launch_pad_building() {
        let b = BuildingType::LaunchPad;
        assert_eq!(b.label(), "Launch Pad");
        // Ensure it's large (e.g., 3x3) - implicitly handled by placement logic,
        // but here we check existence.
    }

    #[test]
    fn test_launch_consumes_resources() {
        let mut world = World::new();

        // Setup Resources
        let mut res = ColonyResources::default();
        res.metal = 100.0;
        res.fuel = 50.0;
        world.insert_resource(res);

        // Setup System World (Colony Location)
        let planet = world.spawn(ColonyLocation).id();

        // Spawn Launch Pad with active queue
        let pad = world.spawn((
            Building { building_type: BuildingType::LaunchPad },
            GridPosition { x: 10, y: 10 },
            LaunchPad,
            LaunchQueue {
                payload: Some(LaunchPayload::ScoutShip),
                progress: 0.0,
                max_progress: 100.0,
            }
        )).id();

        // Run system (assume work is done instantly for test step, or increment progress)
        // Here we simulate progress completion
        let mut queue = world.get_mut::<LaunchQueue>(pad).unwrap();
        queue.progress = 100.0; // Done

        // Run launch logic
        let mut schedule = Schedule::default();
        schedule.add_systems(launch_system);
        schedule.run(&mut world);

        // Verify Resources Deducted
        let res = world.resource::<ColonyResources>();
        // Scout cost: 50 Metal + 20 Fuel (Example)
        assert!((res.metal - 50.0).abs() < f32::EPSILON);
        assert!((res.fuel - 30.0).abs() < f32::EPSILON);

        // Verify Queue Cleared
        let queue = world.get::<LaunchQueue>(pad).unwrap();
        assert!(queue.payload.is_none());

        // Verify Fleet Spawned in Orbit
        let (fleet, orbit) = world.query::<(Entity, &InOrbit)>().single(&world);
        assert_eq!(orbit.parent, planet);
        assert!(world.get::<Fleet>(fleet).is_some());
    }

    #[test]
    fn test_launch_fails_insufficient_resources() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.metal = 10.0; // Not enough
        res.fuel = 50.0;
        world.insert_resource(res);
        world.spawn(ColonyLocation);

        let pad = world.spawn((
            Building { building_type: BuildingType::LaunchPad },
            GridPosition { x: 10, y: 10 },
            LaunchPad,
            LaunchQueue {
                payload: Some(LaunchPayload::ScoutShip),
                progress: 100.0, // Ready to launch
                max_progress: 100.0,
            }
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(launch_system);
        schedule.run(&mut world);

        // Should NOT launch
        let queue = world.get::<LaunchQueue>(pad).unwrap();
        assert!(queue.payload.is_some());

        // No fleet
        assert!(world.query::<&Fleet>().iter(&world).next().is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/launch.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::resources::ColonyResources;
use crate::layer2::fleet::{Fleet, InOrbit};
use crate::layer2::generation::ColonyLocation;
use crate::layer2::system::OrbitalBody; // For visuals

#[derive(Component)]
pub struct LaunchPad;

#[derive(Debug, Clone, PartialEq)]
pub enum LaunchPayload {
    ScoutShip,
    MiningShip,
}

impl LaunchPayload {
    pub fn cost(&self) -> ColonyResources {
        match self {
            Self::ScoutShip => ColonyResources {
                metal: 50.0,
                fuel: 20.0,
                ..ColonyResources::zeroed()
            },
            Self::MiningShip => ColonyResources {
                metal: 100.0,
                fuel: 40.0,
                ..ColonyResources::zeroed()
            },
        }
    }
}

#[derive(Component, Default)]
pub struct LaunchQueue {
    pub payload: Option<LaunchPayload>,
    pub progress: f32,
    pub max_progress: f32,
}

pub fn launch_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LaunchQueue), With<LaunchPad>>,
    mut resources: ResMut<ColonyResources>,
    colony_location: Query<(Entity, &ColonyLocation)>,
) {
    let (planet_entity, _) = match colony_location.get_single() {
        Ok(p) => p,
        Err(_) => return, // No colony planet?
    };

    for (entity, mut queue) in query.iter_mut() {
        if let Some(payload) = &queue.payload {
            if queue.progress >= queue.max_progress {
                let cost = payload.cost();

                if resources.try_deduct(&cost) {
                    // Success! Launch.

                    // Spawn Fleet in Layer 2
                    commands.spawn((
                        Fleet,
                        InOrbit { parent: planet_entity },
                        // Placeholder visuals
                        OrbitalBody {
                            name: "New Fleet".to_string(),
                            radius: 1.0,
                            color: ratatui::style::Color::Cyan,
                            char: 'A',
                        }
                    ));

                    // Clear Queue
                    queue.payload = None;
                    queue.progress = 0.0;

                    // TODO: Log notification "Launch Successful!"
                } else {
                    // Insufficient resources.
                    // Logic: Pause? Cancel?
                    // MVP: Do nothing, retry next tick.
                }
            }
        }
    }
}
```

### 2. Update BuildingType (`src/layer1/building.rs`)

```rust
pub enum BuildingType {
    // ...
    LaunchPad,
}
// Add costs (expensive!), char 'L', etc.
```

## REFACTOR Phase: Quality & Design

- **Construction Logic**: The `LaunchQueue` implies the ship is "built" on the pad. The `launch_system` currently deducts cost *at launch*.
  - *Improvement*: Deduct Metal *during* construction (like `RefiningProgress`), and only deduct Fuel at launch.
- **Workers**: Use `Pop` workers to advance `LaunchQueue.progress`. Current spec assumes it advances automatically or via generic work system.
- **Multiple Pads**: Allow multiple launches.
- **Payload Variety**: Custom ship designer? (Way later).

## Acceptance Criteria

- [ ] `LaunchPad` building exists.
- [ ] `LaunchQueue` handles payload state.
- [ ] `launch_system` checks costs (Metal + Fuel).
- [ ] Launching spawns a `Fleet` entity in Layer 2 orbit.
- [ ] Tests pass.

## Technical Guidance

- Ensure `layer1` can access `layer2` components (`Fleet`, `InOrbit`, `ColonyLocation`).
- The test setup needs `ColonyLocation` to exist.
- Be careful with `try_deduct` - ensure `ColonyResources` includes `fuel`.
