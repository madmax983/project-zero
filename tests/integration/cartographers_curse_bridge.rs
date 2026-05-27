use bevy_ecs::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::cartographers_curse::SellTelemetryEvent;
use scale::layer2::integration::cartographers_curse_chronicle_bridge;

#[test]
fn test_cartographers_curse_chronicle_bridge() {
    let mut world = World::new();
    world.init_resource::<Events<SellTelemetryEvent>>();
    world.init_resource::<Events<AddChronicleEvent>>();

    let mut schedule = Schedule::default();
    schedule.add_systems(cartographers_curse_chronicle_bridge);

    world.send_event(SellTelemetryEvent);
    schedule.run(&mut world);

    let events = world.resource::<Events<AddChronicleEvent>>();
    let mut reader = events.get_cursor();
    let read_events: Vec<_> = reader.read(events).collect();

    assert_eq!(read_events.len(), 1, "Should emit exactly one chronicle event");
    assert_eq!(read_events[0].importance, EventImportance::Major);
    assert!(read_events[0].text.contains("telemetry"));
}
