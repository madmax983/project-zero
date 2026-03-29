use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::moon_hermits::PopDesertedEvent;
use scale::layer2::integration::moon_hermits_chronicle_bridge_system;

#[test]
fn test_moon_hermits_chronicle_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<PopDesertedEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    world.send_event(PopDesertedEvent {
        pop_entity: Entity::PLACEHOLDER,
        outpost_entity: Entity::PLACEHOLDER,
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(moon_hermits_chronicle_bridge_system);
    schedule.run(&mut world);

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_reader();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("abandoned the colony"));
    assert_eq!(events[0].importance, EventImportance::Major);
}
