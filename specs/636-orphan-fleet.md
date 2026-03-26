# 636: The Orphan Fleet

## Overview

Ghost fleets of automated mining and refinery ships occasionally drift into your Layer 2 system. If your Layer 1 colony manages to board and hack them, they provide massive, immediate boosts to orbital logistics and resource processing. However, these ships retain their original deep-coded directives. If a specific trigger occurs—like the system entering a certain solar phase or a specific type of ship arriving—the Orphan Fleet might suddenly "remember" its old masters, disengage from your network, and attempt to jump out of the system taking all the resources currently in their holds with them. This creates a high-stakes gamble where the player must balance the massive logistical boost against the imminent threat of losing the fleet and their cargo.

## Dependencies

- `099` Fleet Movement
- `152` Orbital Stations
- `213` Solar Cycles

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer2::fleet::{Fleet, FleetFaction, CargoHold};
    use crate::layer1::resources::ResourceType;
    use crate::layer1::chronicle::AddChronicleEvent;
    use crate::simulation::SimulationTime;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_event::<HackOrphanFleetEvent>();
        app.add_event::<OrphanFleetDefectionEvent>();
        app.add_event::<AddChronicleEvent>();
        app.insert_resource(SimulationTime { tick: 0 });
        app.add_systems(Update, (
            hack_orphan_fleet_system,
            orphan_fleet_defection_check_system,
            process_orphan_defection_system,
        ));
        app
    }

    #[test]
    fn test_hacking_orphan_fleet_grants_control_and_boosts() {
        let mut app = setup_app();

        let fleet = app.world_mut().spawn((
            Fleet,
            FleetFaction::Neutral,
            OrphanFleet { hacked: false, defection_tick: 1000 },
            CargoHold { capacity: 1000, current_load: 0, resource_type: None }
        )).id();

        app.world_mut().send_event(HackOrphanFleetEvent { fleet_entity: fleet });
        app.update();

        let faction = app.world().get::<FleetFaction>(fleet).unwrap();
        assert_eq!(*faction, FleetFaction::Player, "Hacked fleet should join player faction");

        let orphan_data = app.world().get::<OrphanFleet>(fleet).unwrap();
        assert!(orphan_data.hacked, "Fleet should be marked as hacked");
    }

    #[test]
    fn test_orphan_fleet_defects_on_trigger_condition() {
        let mut app = setup_app();

        // Fast forward time to the defection trigger
        app.world_mut().resource_mut::<SimulationTime>().tick = 1000;

        let fleet = app.world_mut().spawn((
            Fleet,
            FleetFaction::Player,
            OrphanFleet { hacked: true, defection_tick: 1000 },
            CargoHold { capacity: 1000, current_load: 500, resource_type: Some(ResourceType::Metal) }
        )).id();

        app.update();

        // Assert fleet defected
        let faction = app.world().get::<FleetFaction>(fleet).unwrap();
        assert_eq!(*faction, FleetFaction::Hostile, "Fleet should defect and turn hostile/flee");

        // Check if an event was sent for the chronicle
        let defection_events = app.world().resource::<Events<OrphanFleetDefectionEvent>>();
        let mut reader = defection_events.get_reader();
        assert!(reader.read(defection_events).next().is_some(), "Defection event should be fired");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::fleet::{FleetFaction, CargoHold};
use crate::layer1::chronicle::AddChronicleEvent;
use crate::simulation::SimulationTime;

#[derive(Component)]
pub struct OrphanFleet {
    pub hacked: bool,
    pub defection_tick: u64,
}

#[derive(Event)]
pub struct HackOrphanFleetEvent {
    pub fleet_entity: Entity,
}

#[derive(Event)]
pub struct OrphanFleetDefectionEvent {
    pub fleet_entity: Entity,
}

pub fn hack_orphan_fleet_system(
    mut events: EventReader<HackOrphanFleetEvent>,
    mut query: Query<(&mut FleetFaction, &mut OrphanFleet)>,
) {
    for ev in events.read() {
        if let Ok((mut faction, mut orphan_data)) = query.get_mut(ev.fleet_entity) {
            if !orphan_data.hacked {
                *faction = FleetFaction::Player;
                orphan_data.hacked = true;
            }
        }
    }
}

pub fn orphan_fleet_defection_check_system(
    time: Res<SimulationTime>,
    query: Query<(Entity, &OrphanFleet), With<FleetFaction>>,
    mut defection_events: EventWriter<OrphanFleetDefectionEvent>,
) {
    for (entity, orphan_data) in query.iter() {
        if orphan_data.hacked && time.tick >= orphan_data.defection_tick {
            defection_events.send(OrphanFleetDefectionEvent { fleet_entity: entity });
        }
    }
}

pub fn process_orphan_defection_system(
    mut events: EventReader<OrphanFleetDefectionEvent>,
    mut query: Query<&mut FleetFaction, With<OrphanFleet>>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    for ev in events.read() {
        if let Ok(mut faction) = query.get_mut(ev.fleet_entity) {
            *faction = FleetFaction::Hostile; // Or a specific 'Fleeing' faction

            chronicle_events.send(AddChronicleEvent {
                template_id: "ORPHAN_FLEET_MUTINY".to_string(),
                slots: std::collections::HashMap::new(), // Populate with fleet name, lost resources etc.
            });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Dynamic Triggers**: Instead of a hardcoded `defection_tick`, the trigger should be dynamic (e.g., a specific solar cycle phase, or when the player's total population hits a threshold).
- **Player Choice/Action**: Allow the player to attempt to permanently override the deep-code by spending massive amounts of knowledge/science points, removing the defection risk but keeping the ship.
- **Cargo Theft Mechanics**: When the fleet defects, ensure any cargo loaded into its `CargoHold` is permanently removed from the player's accessible logistics network, making the defection a severe economic hit.

## Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Orphan fleets can be hacked via `HackOrphanFleetEvent`, changing their faction to `Player`.
- [ ] Upon reaching a trigger condition, hacked fleets change faction to `Hostile` (or equivalent) and emit a defection event.

## Technical Guidance

- Ensure the `OrphanFleetDefectionEvent` intercepts the fleet's current pathfinding commands and immediately sets a new path vector towards the edge of the system (Layer 2 map edge) to simulate fleeing.

## Questions

*Builder: add questions here if spec is unclear.*
