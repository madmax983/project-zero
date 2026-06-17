use bevy::prelude::*;
use scale::layer1::core::chronicle::{AddChronicleEvent, EventImportance};
use scale::layer2::communications::signal_decay::{
    calculate_signal_decay_system, CommsMessageEvent, RawCommsMessageEvent,
};
use scale::layer2::integration::signal_decay_chronicle_bridge;

#[test]
fn test_signal_decay_chronicle_bridge() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_event::<RawCommsMessageEvent>();
    app.add_event::<CommsMessageEvent>();
    app.add_event::<AddChronicleEvent>();

    app.add_systems(
        Update,
        (calculate_signal_decay_system, signal_decay_chronicle_bridge).chain(),
    );

    let origin = app
        .world_mut()
        .spawn(Transform::from_xyz(0.0, 0.0, 0.0))
        .id();
    let target = app
        .world_mut()
        .spawn(Transform::from_xyz(500.0, 0.0, 0.0))
        .id(); // Distance 500 = 0.5 corruption

    app.world_mut().send_event(RawCommsMessageEvent {
        origin,
        target,
        text: "Demand 500 gold".to_string(),
    });

    app.update();

    let events = app.world().resource::<Events<AddChronicleEvent>>();
    #[allow(deprecated)]
    let mut cursor = events.get_reader();
    let emitted: Vec<&AddChronicleEvent> = cursor.read(events).collect();

    assert_eq!(emitted.len(), 1, "Should emit one AddChronicleEvent");
    assert_eq!(emitted[0].importance, EventImportance::Major);
    assert!(emitted[0].text.contains("[CORRUPTED]"));
}
