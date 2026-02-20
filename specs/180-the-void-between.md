# 180: The Void Between

## Overview

Interstellar travel is not just a loading bar; it is a traversal of hostile space. This feature introduces **Void Incidents** for ships in transit on the System Map (Layer 2).

When a ship travels between nodes (Planets, Stars), it enters a `Transit` state. During this time, there is a small daily chance of an incident. Incidents range from minor delays to "Ship Lost" events where the vessel disappears from the map, potentially returning years later with altered properties.

**Why:**
- Adds risk/reward to trade and exploration.
- Supports the "Cosmic Horror" tone (ships coming back *wrong*).
- Creates emergent stories via the Chronicle.

## Dependencies

- `094` — System View Architecture (Layer 2 basics)
- `099` — Fleet Movement (Ship entities, `Transit` state)
- `010` — Chronicle System (Logging events)

## RED Phase: Tests First

Write these tests in `src/layer2/void_incident_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::{OrbitalBody, Ship, ShipState, Transit};
    use crate::layer2::void::{VoidIncident, VoidState, void_incident_system};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_incident_probability() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        // Seed RNG for deterministic test (if using a resource RNG)
        // world.insert_resource(Random::seed(12345));

        // Spawn a ship in transit
        let ship = world.spawn((
            Ship { name: "ISS Venture".to_string(), ..Default::default() },
            ShipState::InTransit(Transit {
                destination: Entity::PLACEHOLDER,
                progress: 0.5,
                duration: 100,
                start_tick: 0,
            }),
        )).id();

        // For testing, we can either:
        // 1. Mock the RNG (if using a resource RNG)
        // 2. Set the chance to 1.0 (if configurable via Resource)
        // 3. Run many iterations (probabilistic)

        // Assuming we set a "VoidConfig" resource with 100% chance for testing:
        // world.insert_resource(VoidConfig { incident_chance: 1.0 });

        let mut schedule = Schedule::default();
        schedule.add_systems(void_incident_system);
        schedule.run(&mut world);

        // Verify state change or event
        // ...
    }

    #[test]
    fn test_ship_lost_state() {
        let mut world = World::new();

        // Spawn ship
        let ship = world.spawn((
            Ship { name: "Lost Hope".to_string(), ..Default::default() },
            ShipState::InTransit(Transit::default()),
        )).id();

        // Apply "Lost" incident manually
        let incident = VoidIncident::LostInVoid { duration: 500 };
        incident.apply(&mut world, ship);

        // Verify state change
        let state = world.get::<ShipState>(ship).unwrap();
        match state {
            ShipState::Lost(return_tick) => assert_eq!(*return_tick, 500), // Assuming current tick 0 + 500
            _ => panic!("Ship should be in Lost state"),
        }
    }

    #[test]
    fn test_ship_return_mutation() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 1000 }); // Fast forward

        // Spawn a Lost ship due to return at tick 1000
        let ship = world.spawn((
            Ship {
                name: "Event Horizon".to_string(),
                crew_morale: 1.0,
                ..Default::default()
            },
            ShipState::Lost(1000),
        )).id();

        // Run return system
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer2::void::void_return_system);
        schedule.run(&mut world);

        // Verify ship is back in System (Idle or Orbiting)
        let state = world.get::<ShipState>(ship).unwrap();
        assert!(matches!(state, ShipState::Orbiting(_)) || matches!(state, ShipState::Idle));

        // Verify mutation (e.g. Morale drop from horror)
        let ship_data = world.get::<Ship>(ship).unwrap();
        assert!(ship_data.crew_morale < 1.0);
    }

    #[test]
    fn test_chronicle_event_on_incident() {
        // Verify Chronicle::add_entry is called
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `ShipState` Variants

Ensure `ShipState` in `src/layer2/system.rs` supports being lost.

```rust
// src/layer2/system.rs

#[derive(Component, Debug, Clone)]
pub enum ShipState {
    Idle,
    Orbiting(Entity), // Orbiting a body
    InTransit(Transit),
    Lost(u64), // Return tick
}
```

### 2. Implement Void System

Create `src/layer2/void.rs`.

```rust
use bevy_ecs::prelude::*;
use crate::layer2::system::{Ship, ShipState};
use crate::shared::time::SimulationTime;
use rand::prelude::*;

pub enum VoidIncident {
    MinorDelay(u64),
    LostInVoid { duration: u64 },
    GhostSignal, // Flavor event
}

impl VoidIncident {
    pub fn apply(&self, world: &mut World, entity: Entity) {
        match self {
            VoidIncident::LostInVoid { duration } => {
                let current_tick = world.resource::<SimulationTime>().tick;
                if let Some(mut state) = world.get_mut::<ShipState>(entity) {
                    *state = ShipState::Lost(current_tick + duration);
                }
                // Add Chronicle entry: SHIP_LOST
            },
            VoidIncident::MinorDelay(delay) => {
                if let Some(mut state) = world.get_mut::<ShipState>(entity) {
                    if let ShipState::InTransit(ref mut transit) = *state {
                        transit.duration += delay;
                    }
                }
                // Add Chronicle entry: VOID_INCIDENT
            },
            VoidIncident::GhostSignal => {
                // Just Chronicle
            }
        }
    }
}

pub fn void_incident_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ShipState)>,
    time: Res<SimulationTime>,
) {
    // Chance per tick per ship in transit
    let chance = 0.001; // 0.1% per tick

    for (entity, state) in query.iter_mut() {
        if let ShipState::InTransit(_) = state {
            if rand::thread_rng().gen_bool(chance) {
                // Roll incident type
                let incident = VoidIncident::LostInVoid { duration: 5000 }; // Simplify for Green
                // Apply needs mutable world access, which we don't have in query iteration safely without commands
                // Use a command or event buffer
                // For MVP, just print or cheat
            }
        }
    }
}

pub fn void_return_system(
    mut query: Query<(Entity, &mut ShipState, &mut Ship)>,
    time: Res<SimulationTime>,
) {
    for (entity, mut state, mut ship) in query.iter_mut() {
        if let ShipState::Lost(return_tick) = *state {
            if time.tick >= return_tick {
                // Return logic
                *state = ShipState::Idle; // Or return to last destination
                ship.crew_morale = (ship.crew_morale - 0.5).max(0.0); // Trauma

                // Chronicle: SHIP_RETURNED
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Incident Table**: Move incident probabilities to a config file or weighted table.
- **ECS Safety**: `apply` needs `&mut World`, but system iterates query. Use `Commands` with a custom `Command` impl or `Events`.
- **Chronicle Integration**: Use the templates defined in `lore/TEMPLATES.md` (`SHIP_LOST`, `SHIP_RETURNED`, `VOID_INCIDENT`).
- **Visuals**: "Lost" ships should disappear from the System Map list or be grayed out/marked "?".

## Acceptance Criteria

- [ ] `ShipState::Lost` is defined.
- [ ] Ships in transit have a chance to trigger an incident.
- [ ] `Lost` ships re-appear after the duration.
- [ ] Returning ships have modified stats (e.g., lower morale).
- [ ] Chronicle events are generated for Loss and Return.

## Technical Guidance

- Use `bevy_ecs::system::Command` to implement complex world mutations from systems.
- Ensure `Transit` duration is actually checked/consumed in `fleet_movement` system (Spec 099), and that `Lost` state prevents movement updates.

## Questions

*Builder: add questions here if spec is unclear.*
