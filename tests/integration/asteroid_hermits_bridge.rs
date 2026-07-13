use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::asteroid_hermits::{DiscoveryEvent, HermitOutpost};
use scale::layer2::integration::{
    asteroid_hermit_discovery_chronicle_bridge, asteroid_hermit_exodus_chronicle_bridge,
};

#[test]
fn test_asteroid_hermits_exodus_chronicle_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<AddChronicleEvent>>();

    let mut schedule = Schedule::default();
    schedule.add_systems(asteroid_hermit_exodus_chronicle_bridge);

    world.spawn(HermitOutpost);

    schedule.run(&mut world);

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("Mass Exodus"));
    assert_eq!(events[0].importance, EventImportance::Major);
}

#[test]
fn test_asteroid_hermits_discovery_chronicle_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<DiscoveryEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    world.send_event(DiscoveryEvent {
        outpost: Entity::PLACEHOLDER,
        item_type: "Rare Mineral".to_string(),
    });

    let mut schedule = Schedule::default();
    schedule.add_systems(asteroid_hermit_discovery_chronicle_bridge);

    schedule.run(&mut world);

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0].text.contains("Rare Mineral"));
    assert_eq!(events[0].importance, EventImportance::Minor);
}
