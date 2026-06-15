use bevy::prelude::*;
use scale::layer1::chronicle::{Chronicle, EventImportance};
use scale::layer2::dead_protocols::ViolationEvent;
use scale::layer2::integration::dead_protocol_chronicle_bridge;

#[test]
fn test_dead_protocol_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<ViolationEvent>();
    app.add_event::<scale::layer1::chronicle::AddChronicleEvent>();
    app.init_resource::<Chronicle>();

    app.add_systems(Update, dead_protocol_chronicle_bridge);

    let dummy_target = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(ViolationEvent {
        target: dummy_target,
    });

    app.update();

    let chronicle_events = app
        .world()
        .resource::<Events<scale::layer1::chronicle::AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected one chronicle event");
    assert_eq!(events[0].importance, EventImportance::Major);
    assert!(events[0].text.contains("A dead protocol was violated at"));
}
