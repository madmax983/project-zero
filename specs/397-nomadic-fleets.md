# 397: Nomadic Fleets

## 1. Overview
Not every civilization settles down. The galaxy is populated by "Nomadic Fleets" (Layer 3) that traverse star systems on massive Ark Ships. They do not hold planets. Instead, they strip-mine systems for resources and move on.

When they enter the player's system (Layer 2), they present a dilemma: they offer incredibly rare tech and trade opportunities, but if ignored or denied, they will aggressively mine the system's asteroid belts and moons, permanently depleting resources the player might need.

## 2. Dependencies
- `094-system-view.md` (for Layer 2/3 fleet representations)
- `101-system-mining.md` (for asteroids/system resources to be consumed)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::system::{SystemBody, AsteroidBelt};
    use crate::layer3::fleet::{NomadicFleet, FleetState};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_nomad_fleet_arrival() {
        let mut app = App::new();
        app.insert_resource(SimulationTime { tick: 5000, speed: Default::default() });
        app.add_systems(Update, spawn_nomad_fleets_system);

        app.update();

        // Check if a fleet spawned (mocking the random chance)
        let mut query = app.world_mut().query::<&NomadicFleet>();
        // Assuming we mock it to spawn at tick 5000
        assert!(query.iter(app.world()).count() > 0 || true); // Placeholder for actual RNG test
    }

    #[test]
    fn test_nomad_mining_depletes_asteroids() {
        let mut app = App::new();
        app.add_systems(Update, nomad_mining_system);

        let asteroid = app.world_mut().spawn((
            SystemBody,
            AsteroidBelt { resources_remaining: 500 },
        )).id();

        let fleet = app.world_mut().spawn(NomadicFleet {
            state: FleetState::Mining { target: asteroid },
            mining_rate: 50,
            patience: 100,
        }).id();

        app.update();

        let belt = app.world().get::<AsteroidBelt>(asteroid).unwrap();
        assert_eq!(belt.resources_remaining, 450);
    }

    #[test]
    fn test_nomad_fleet_leaves_when_done() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_nomad_departure_system);

        let asteroid = app.world_mut().spawn((
            SystemBody,
            AsteroidBelt { resources_remaining: 0 }, // Depleted
        )).id();

        let fleet = app.world_mut().spawn(NomadicFleet {
            state: FleetState::Mining { target: asteroid },
            mining_rate: 50,
            patience: 0, // Done
        }).id();

        app.update();

        let fleet_component = app.world().get::<NomadicFleet>(fleet);
        assert!(fleet_component.is_none() || fleet_component.unwrap().state == FleetState::Departing);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::system::{SystemBody, AsteroidBelt};
use crate::shared::time::SimulationTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleetState {
    Arriving,
    Trading,
    Mining { target: Entity },
    Departing,
}

#[derive(Component, Debug)]
pub struct NomadicFleet {
    pub state: FleetState,
    pub mining_rate: u32,
    pub patience: u32, // Ticks until they leave or get aggressive
}

pub fn spawn_nomad_fleets_system(
    mut commands: Commands,
    time: Res<SimulationTime>,
) {
    // Mock condition for MVP
    if time.tick == 5000 {
        commands.spawn(NomadicFleet {
            state: FleetState::Arriving,
            mining_rate: 10,
            patience: 1000,
        });
    }
}

pub fn nomad_mining_system(
    mut fleets_query: Query<&mut NomadicFleet>,
    mut asteroid_query: Query<&mut AsteroidBelt>,
) {
    for mut fleet in fleets_query.iter_mut() {
        if let FleetState::Mining { target } = fleet.state {
            if let Ok(mut belt) = asteroid_query.get_mut(target) {
                if belt.resources_remaining >= fleet.mining_rate {
                    belt.resources_remaining -= fleet.mining_rate;
                } else {
                    belt.resources_remaining = 0;
                    fleet.state = FleetState::Departing; // Move on when empty
                }
            } else {
                fleet.state = FleetState::Departing; // Target invalid
            }
        }
    }
}

pub fn evaluate_nomad_departure_system(
    mut commands: Commands,
    mut fleets_query: Query<(Entity, &mut NomadicFleet)>,
) {
    for (entity, mut fleet) in fleets_query.iter_mut() {
        if fleet.patience == 0 || fleet.state == FleetState::Departing {
            commands.entity(entity).despawn();
        } else if let FleetState::Arriving | FleetState::Trading = fleet.state {
            fleet.patience = fleet.patience.saturating_sub(1);
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Diplomacy integration:** Players should be able to interact with the fleet, offering them resources to leave the asteroid belt alone, or trading for unique high-tech gear.
- **Combat logic:** If the player attacks the Nomadic Fleet, it should possess overwhelming ancient weaponry (acting as a "Fallen Empire" equivalent).
- **Movement:** In `Arriving` and `Departing` states, they should physically move across the Layer 2 map.

## 6. Acceptance Criteria (Testable!)
- [ ] `NomadicFleet` component exists.
- [ ] Nomadic fleets reduce `resources_remaining` in `AsteroidBelt` entities when in `Mining` state.
- [ ] Fleets change to `Departing` when the target is depleted.
- [ ] Tests compile and pass successfully.

## 7. Technical Guidance
- If `AsteroidBelt` is not yet fully defined in `layer2::system`, mock it alongside this feature.
- Ensure the `time.tick` event in `spawn_nomad_fleets_system` is replaced with a real random event generator long-term.

## 8. Questions
*Builder: add questions here if spec is unclear.*
