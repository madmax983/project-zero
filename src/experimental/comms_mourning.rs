//! Comms Mourning (Nova Feature).
//!
//! # The Spark
//! We have a `PopDied` event, `CommsRelay` buildings, and a `Morale` system.
//! When a pop dies, their loss affects the colony. What if the communications relay could broadcast their digital ghost/eulogy?
//!
//! # The Feature
//! When a `PopDied` event occurs, the system checks if a `BuildingType::CommsRelay` exists in the colony.
//! If one is found, it intercepts the dying pop's final synaptic flashes and broadcasts a comforting eulogy.
//! This gives all pops in the colony a temporary positive `MoodModifier` to buffer their grief, and emits an `AddChronicleEvent`.

use crate::layer1::architecture::{Building, BuildingType};
use crate::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use crate::layer1::entities::pop::PopDied;
use crate::layer1::social::morale::{MoodModifier, Morale};
use bevy_ecs::prelude::*;

/// The Comms Mourning system.
/// Listens for `PopDied` events, checks for a `CommsRelay`, and broadcasts a comforting message.
pub fn comms_mourning_system(
    mut died_events: EventReader<PopDied>,
    buildings: Query<&Building>,
    mut morale_query: Query<&mut Morale>,
    mut chronicle_events: EventWriter<AddChronicleEvent>,
) {
    let has_relay = buildings
        .iter()
        .any(|b| b.building_type == BuildingType::CommsRelay);

    if !has_relay {
        return;
    }

    for event in died_events.read() {
        // Broadcast the eulogy
        for mut morale in morale_query.iter_mut() {
            morale.add_modifier(MoodModifier {
                label: "Digital Eulogy".to_string(),
                value: 0.15,
                duration: 200,
            });
        }

        chronicle_events.send(AddChronicleEvent {
            text: format!("The CommsRelay intercepted the final synaptic flashes of {}, broadcasting a digital ghost that comforted the colony.", event.name),
            importance: EventImportance::Minor,
        });
    }
}

/// Registers the system.
pub fn register(schedule: &mut Schedule) {
    schedule.add_systems(comms_mourning_system);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comms_mourning_broadcasts_when_relay_exists() {
        let mut world = World::new();
        world.init_resource::<Events<PopDied>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        // Spawn a CommsRelay
        world.spawn(Building {
            building_type: BuildingType::CommsRelay,
        });

        // Spawn a living pop
        let living_pop = world.spawn(Morale::default()).id();

        // Send a death event
        world.send_event(PopDied {
            entity: Entity::from_raw(999),
            name: "John Doe".to_string(),
            tick: 100,
            reason: "Old Age".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(comms_mourning_system);
        schedule.run(&mut world);

        // Verify Morale modifier added
        let morale = world.get::<Morale>(living_pop).unwrap();
        assert!(
            !morale.modifiers.is_empty(),
            "Should have added a mood modifier for the broadcast"
        );
        assert_eq!(morale.modifiers[0].label, "Digital Eulogy");
        assert!(morale.modifiers[0].value > 0.0);

        // Verify Chronicle Event emitted
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        let evt = reader.read(events).next().unwrap();
        assert!(evt.text.contains("John Doe"));
        assert!(evt.text.contains("digital ghost"));
    }

    #[test]
    fn test_comms_mourning_does_nothing_without_relay() {
        let mut world = World::new();
        world.init_resource::<Events<PopDied>>();
        world.init_resource::<Events<AddChronicleEvent>>();

        // NO CommsRelay

        // Spawn a living pop
        let living_pop = world.spawn(Morale::default()).id();

        // Send a death event
        world.send_event(PopDied {
            entity: Entity::from_raw(999),
            name: "Jane Doe".to_string(),
            tick: 100,
            reason: "Disease".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(comms_mourning_system);
        schedule.run(&mut world);

        // Verify NO Morale modifier added
        let morale = world.get::<Morale>(living_pop).unwrap();
        assert!(
            morale.modifiers.is_empty(),
            "Should not have added a mood modifier without a CommsRelay"
        );

        // Verify NO Chronicle Event emitted
        let events = world.resource::<Events<AddChronicleEvent>>();
        let mut reader = events.get_cursor();
        assert!(reader.read(events).next().is_none());
    }
}
