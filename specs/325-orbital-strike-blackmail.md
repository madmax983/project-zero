# 325: Orbital Strike Blackmail

## 1. Overview

You can rig your own colony's primary reactor to overload, broadcasting the countdown to the Galactic Network (Layer 3). Threatening to blow up your own highly valuable strategic position forces invading or blockading empires to negotiate or back off to prevent the loss of the asset. You have a short window to either defuse your own bomb and surrender, or let it blow, taking the fleet and your colony to hell together.

## 2. Dependencies

- `042` Energy System (for reactors)
- `159` Fleet Combat (for enemy invasion fleets)
- `050` Civil Unrest (for the colony's reaction to being taken hostage by their own leader)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_initiate_reactor_overload() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_blackmail_initiation);

        let reactor = app.world_mut().spawn((
            Reactor { max_output: 1000 },
            OverloadProtocol { active: false, countdown: 0 }
        )).id();

        app.world_mut().insert_resource(BlackmailEvent { target_reactor: reactor });

        // Act
        app.update();

        // Assert
        let protocol = app.world().get::<OverloadProtocol>(reactor).unwrap();
        assert!(protocol.active, "Overload protocol should be active");
        assert_eq!(protocol.countdown, 60, "Countdown should start at 60 ticks/seconds");
    }

    #[test]
    fn test_overload_deters_invasion_fleet() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, process_enemy_negotiation);

        // Reactor is armed
        app.world_mut().spawn((
            Reactor { max_output: 5000 }, // High value target
            OverloadProtocol { active: true, countdown: 30 }
        ));

        // Enemy fleet is in orbit
        let fleet = app.world_mut().spawn(EnemyFleet {
            status: FleetStatus::Invading,
            greed_value: 4000 // Values the reactor more than continuing the invasion
        }).id();

        // Act
        app.update();

        // Assert
        let fleet_state = app.world().get::<EnemyFleet>(fleet).unwrap();
        assert_eq!(fleet_state.status, FleetStatus::Negotiating, "Fleet should halt invasion and negotiate if target is rigged to blow and highly valued");
    }

    #[test]
    fn test_overload_detonates_if_countdown_reaches_zero() {
         // Arrange
         let mut app = App::new();
         app.add_event::<DetonationEvent>();
         app.add_systems(Update, process_reactor_countdown);

         let reactor = app.world_mut().spawn((
             GridPosition { x: 0, y: 0 },
             Reactor { max_output: 1000 },
             OverloadProtocol { active: true, countdown: 1 } // 1 tick remaining
         )).id();

         // Act
         app.update(); // Tick to 0

         // Assert
         let events = app.world().resource::<Events<DetonationEvent>>();
         let mut reader = events.get_reader();
         let mut det_occurred = false;

         for _ in reader.read(events) {
             det_occurred = true;
         }

         assert!(det_occurred, "Reactor should detonate when countdown hits zero");

         // Reactor should be destroyed
         assert!(app.world().get::<Reactor>(reactor).is_none(), "Reactor entity should be despawned or broken on detonation");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Reactor {
    pub max_output: u32,
}

#[derive(Component)]
pub struct OverloadProtocol {
    pub active: bool,
    pub countdown: u32,
}

#[derive(Resource)]
pub struct BlackmailEvent {
    pub target_reactor: Entity,
}

#[derive(PartialEq, Debug)]
pub enum FleetStatus {
    Invading,
    Negotiating,
    Retreating,
}

#[derive(Component)]
pub struct EnemyFleet {
    pub status: FleetStatus,
    pub greed_value: u32,
}

#[derive(Event)]
pub struct DetonationEvent {
    pub source: Entity,
    pub power: u32,
}

pub fn process_blackmail_initiation(
    mut events: Option<ResMut<BlackmailEvent>>,
    mut reactors: Query<&mut OverloadProtocol>,
) {
    if let Some(ev) = events.take() {
        if let Ok(mut protocol) = reactors.get_mut(ev.target_reactor) {
            protocol.active = true;
            protocol.countdown = 60; // 60 ticks
        }
    }
}

pub fn process_enemy_negotiation(
    reactors: Query<(&Reactor, &OverloadProtocol)>,
    mut fleets: Query<&mut EnemyFleet>,
) {
    // Check if any reactor is actively overloading
    let mut total_rigged_value = 0;
    for (reactor, protocol) in reactors.iter() {
        if protocol.active {
            total_rigged_value += reactor.max_output;
        }
    }

    if total_rigged_value > 0 {
        for mut fleet in fleets.iter_mut() {
            if fleet.status == FleetStatus::Invading && total_rigged_value >= fleet.greed_value {
                // The prize is rigged and worth more than calling the bluff. They pause to negotiate.
                fleet.status = FleetStatus::Negotiating;
            }
        }
    }
}

pub fn process_reactor_countdown(
    mut commands: Commands,
    mut events: EventWriter<DetonationEvent>,
    mut reactors: Query<(Entity, &Reactor, &mut OverloadProtocol)>,
) {
    for (entity, reactor, mut protocol) in reactors.iter_mut() {
        if protocol.active {
            if protocol.countdown > 0 {
                protocol.countdown -= 1;
            }

            if protocol.countdown == 0 {
                events.send(DetonationEvent {
                    source: entity,
                    power: reactor.max_output,
                });
                commands.entity(entity).despawn();
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Unrest Generation:** When `OverloadProtocol` becomes `active`, it should immediately max out global `Unrest` and cause Pops to panic. They are hostages to their own government.
- **Bluff Calling:** If the `greed_value` is higher than the `total_rigged_value` (or if the fleet has a 'Fanatic' trait), they should ignore the blackmail and continue the invasion, forcing the player to actually blow it up or surrender.
- **Defusal Mechanics:** Allow the player to cancel the overload via an `Engineer` job interaction, which takes time. It shouldn't be an instant UI toggle to turn it off, making the decision weighty.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] Test coverage >=85% for new code.
- [ ] Reactor enters overload state with a countdown.
- [ ] Enemy fleets in `Invading` status shift to `Negotiating` if the rigged asset is valuable enough.
- [ ] Reactor detonates and destroys itself when countdown hits 0.

## 7. Technical Guidance

- Implement inside `src/layer1/tech/reactor_overload.rs` or `src/layer3/diplomacy.rs`.
- The `DetonationEvent` should be caught by a system that loops over all entities (both Layer 1 buildings and Layer 2 ships in orbit) and applies massive damage in a radius based on the `power` value.
- Add an integration test that verifies an enemy fleet actually takes damage if the reactor blows while they are in orbit.

## 8. Questions

*Builder: add questions here if spec is unclear.*
