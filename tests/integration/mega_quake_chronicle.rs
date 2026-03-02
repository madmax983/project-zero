use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::geology::tectonic::MegaQuakeEvent;
use scale::layer1::integration::mega_quake_chronicle_bridge;

#[test]
fn test_mega_quake_adds_chronicle_event() {
    let mut world = World::new();
    world.init_resource::<Events<MegaQuakeEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    let mut schedule = Schedule::default();
    schedule.add_systems(mega_quake_chronicle_bridge);

    // Trigger MegaQuakeEvent
    world.send_event(MegaQuakeEvent);

    schedule.run(&mut world);

    let chronicle_events = world.resource::<Events<AddChronicleEvent>>();
    assert!(!chronicle_events.is_empty(), "MegaQuakeEvent should trigger an AddChronicleEvent");

    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("Mega-Quake"));
}
