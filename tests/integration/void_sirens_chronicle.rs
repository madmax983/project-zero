use bevy::prelude::*;
use scale::layer1::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer1::core::integration::siren_signal_chronicle_bridge;
use scale::layer1::void_sirens::SirenSignalEvent;

#[test]
fn test_siren_signal_chronicle_bridge() {
    let mut app = App::new();
    app.add_event::<SirenSignalEvent>();
    app.add_event::<AddChronicleEvent>();
    app.add_systems(Update, siren_signal_chronicle_bridge);

    app.world_mut().send_event(SirenSignalEvent);

    app.update();

    let chronicle_events = app.world().resource::<Events<AddChronicleEvent>>();
    let mut reader = chronicle_events.get_cursor();
    let events: Vec<_> = reader.read(chronicle_events).collect();

    assert_eq!(events.len(), 1);
    assert!(events[0]
        .text
        .contains("A mesmerizing deep-space signal is detected, driving our brightest minds into an obsession."));
    assert_eq!(events[0].importance, EventImportance::Major);
}