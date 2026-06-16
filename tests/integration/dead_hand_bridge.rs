use bevy::prelude::*;
use scale::layer1::chronicle::{Chronicle, EventImportance};
use scale::layer1::core::integration::dead_hand_chronicle_bridge;
use scale::layer1::systems::dead_hand::DoomsdayTriggeredEvent;

#[test]
fn test_dead_hand_chronicle_bridge() {
    let mut app = App::new();

    app.add_event::<DoomsdayTriggeredEvent>();
    app.add_event::<scale::layer1::chronicle::AddChronicleEvent>();
    app.init_resource::<Chronicle>();

    app.add_systems(Update, dead_hand_chronicle_bridge);

    let dummy_device = app.world_mut().spawn_empty().id();

    app.world_mut().send_event(DoomsdayTriggeredEvent {
        device: dummy_device,
    });

    app.update();

    let chronicle_events = app
        .world()
        .resource::<Events<scale::layer1::chronicle::AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1, "Expected one chronicle event");
    assert_eq!(events[0].importance, EventImportance::Legendary);
    assert!(events[0]
        .text
        .contains("The Dead Hand was triggered by device"));
}
